//! Índice de nombres de clan (`guild-names.json`): una **caché de búsqueda**, no un dato vivo.
//!
//! Lo que el addon enseña en pantalla sale de `settings.json` (`Config::guild_names`), que se
//! escribe una vez por clan y se lee sin coste. Este fichero es otra cosa: es la lista con la que
//! el buscador encuentra clanes por un trozo del nombre, y tiene miles de entradas (~9 900 en
//! noviembre de 2026, 675 KB).
//!
//! Por eso **no se tiene en memoria**: se lee cuando el jugador busca, se filtra y se suelta
//! (petición del jugador, 2026-11-01: «importante no estar leyendo guild-names.json todo el
//! rato»). El fichero lo puede generar `scripts/guild-search.ps1 -BuildIndex` y llega en un ZIP
//! desde el repositorio de distribución (ver [`crate::guild_index`]).
//!
//! `append_async` es la única escritura: añade los clanes que el jugador empieza a seguir y que no
//! estaban, para que la próxima búsqueda por texto los encuentre.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::config;

/// Fichero del índice dentro de la carpeta del addon.
pub const FILE: &str = "guild-names.json";

/// Cuántos nombres tiene el índice, para que la ventana lo diga sin leer el fichero.
static INDEX_LEN: AtomicUsize = AtomicUsize::new(0);

/// Ruta del fichero, o `None` si la carpeta del addon no está disponible.
pub fn path() -> Option<PathBuf> {
    Some(config::addon_dir().ok()?.join(FILE))
}

/// Whether the index is on disk. `false` means the download has to bring it.
pub fn exists() -> bool {
    path().is_some_and(|path| path.exists())
}

/// Size of the index, as last seen by a reader or a writer. `0` means «not read yet».
pub fn index_len() -> usize {
    INDEX_LEN.load(Ordering::Relaxed)
}

/// Reads the index. **Call it from a thread that is not the render one**, and drop the result
/// when the search is done: it is several thousand entries.
///
/// A broken file is an empty index, not an error: the search still runs against the API.
pub fn read() -> BTreeMap<String, String> {
    let Some(path) = path().filter(|path| path.exists()) else {
        INDEX_LEN.store(0, Ordering::Relaxed);
        return BTreeMap::new();
    };
    let names = read_at(&path);
    INDEX_LEN.store(names.len(), Ordering::Relaxed);
    names
}

/// [`read`] from a given file.
fn read_at(path: &std::path::Path) -> BTreeMap<String, String> {
    let raw = match std::fs::read(path) {
        Ok(raw) => raw,
        Err(error) => {
            log::warn!("índice de nombres de clan ilegible ({error})");
            return BTreeMap::new();
        }
    };
    match serde_json::from_slice::<BTreeMap<String, String>>(&raw) {
        Ok(names) => names,
        Err(error) => {
            log::warn!("índice de nombres de clan mal formado ({error}); se busca solo en el API");
            BTreeMap::new()
        }
    }
}

/// Clanes del índice cuya etiqueta (nombre o tag) contiene `needle`, sin distinguir mayúsculas.
///
/// Es una **subcadena**, a diferencia del API: `/v2/guild/search?name=` devuelve **un solo** clan
/// y solo si el texto empieza su nombre, así que «dolyak» no encuentra *Dolyak Sanctuary* y «DIE»
/// encuentra *Die* pero no las demás (medido el 2026-11-01).
pub fn search_in(
    names: &BTreeMap<String, String>,
    needle: &str,
    limit: usize,
) -> Vec<(String, String)> {
    let needle = needle.trim().to_lowercase();
    if needle.len() < 2 {
        return Vec::new();
    }
    let mut out: Vec<(String, String)> = names
        .iter()
        .filter(|(id, label)| matches_text(label, id, &needle))
        .map(|(id, label)| (id.clone(), label.clone()))
        .collect();
    // Los más cortos primero: quien busca «die» quiere *Die Garde*, no un nombre de 40 letras.
    out.sort_by_key(|(id, label)| (label.len(), id.clone()));
    out.truncate(limit);
    out
}

/// Whether an entry matches what the player typed. Pure, so the rule can be tested.
pub fn matches_text(label: &str, id: &str, needle: &str) -> bool {
    let needle = needle.trim().to_lowercase();
    if needle.is_empty() {
        return false;
    }
    label.to_lowercase().contains(&needle) || id.to_lowercase().starts_with(&needle)
}

/// Añade nombres que faltaban al índice, en su propio hilo.
///
/// Se llama al empezar a seguir un clan: si no estaba en el índice, la próxima búsqueda por texto
/// tiene que encontrarlo (petición del jugador, 2026-11-01). Leer, mezclar y escribir 675 KB no
/// puede pasar por el hilo de render, así que va a un hilo corto que muere al terminar.
pub fn append_async(entries: Vec<(String, String)>) {
    if entries.is_empty() {
        return;
    }
    let spawned = std::thread::Builder::new()
        .name("gw2lurker-names".to_owned())
        .spawn(move || match append(&entries) {
            Ok(added) if added > 0 => log::info!("índice de clanes: {added} nombres nuevos"),
            Ok(_) => {}
            Err(error) => log::warn!("no se pudo ampliar el índice de clanes ({error})"),
        });
    if spawned.is_err() {
        log::warn!("índice de clanes: no se pudo lanzar la escritura");
    }
}

/// Adds the entries the index did not have yet and writes it back. Returns how many were new.
pub fn append(entries: &[(String, String)]) -> std::io::Result<usize> {
    let Some(path) = path() else {
        return Err(std::io::Error::other("sin carpeta del addon"));
    };
    append_at(&path, entries)
}

/// [`append`] against a given file, so it can be tested without touching the player's index.
pub fn append_at(path: &std::path::Path, entries: &[(String, String)]) -> std::io::Result<usize> {
    let mut names = read_at(path);
    let before = names.len();
    for (id, label) in entries {
        if id.trim().is_empty() || label.trim().is_empty() {
            continue;
        }
        names.entry(id.clone()).or_insert_with(|| label.clone());
    }
    if names.len() == before {
        return Ok(0);
    }
    let json = serde_json::to_vec(&names).map_err(std::io::Error::other)?;
    let temp = path.with_extension("json.tmp");
    std::fs::write(&temp, json)?;
    std::fs::rename(&temp, path)?;
    INDEX_LEN.store(names.len(), Ordering::Relaxed);
    Ok(names.len() - before)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index() -> BTreeMap<String, String> {
        let mut names = BTreeMap::new();
        names.insert("A".into(), "[DG] Die Garde".into());
        names.insert("B".into(), "[DK] die letzken Hunde".into());
        names.insert("C".into(), "[DA] DIE anmutige Garde".into());
        names.insert("6C45F3C8".into(), "[YaK] Dolyak Sanctuary".into());
        names.insert("D".into(), "[DV] A new name for bomb".into());
        names
    }

    #[test]
    fn t44_the_search_is_a_substring_and_ignores_case() {
        // Lo que pidió el jugador el 2026-11-01: «DIE» tiene que encontrar los tres.
        assert!(matches_text("[DG] Die Garde", "A", "die"));
        assert!(matches_text("[DK] die letzken Hunde", "B", "DIE"));
        assert!(matches_text("[DA] DIE anmutige Garde", "C", "die"));
        assert!(matches_text("[YaK] Dolyak Sanctuary", "6C45F3C8", "dolyak"));
        assert!(
            matches_text("[DV] A new name for bomb", "D", "a new"),
            "también con espacios"
        );
        assert!(!matches_text("[X] Otra cosa", "E", "die"));
        assert!(!matches_text("[X] Otra cosa", "E", "  "));
    }

    #[test]
    fn t44_two_letters_are_the_minimum() {
        // Con una sola letra la lista sería el índice entero.
        assert!(search_in(&index(), "d", 5).is_empty());
    }

    #[test]
    fn t44_die_finds_the_three_and_the_shortest_come_first() {
        let found = search_in(&index(), "die", 12);
        let labels: Vec<&str> = found.iter().map(|(_, label)| label.as_str()).collect();
        assert_eq!(
            labels.len(),
            3,
            "el jugador esperaba varios clanes con «DIE», no uno: {labels:?}"
        );
        assert_eq!(
            labels[0], "[DG] Die Garde",
            "primero el nombre más corto: {labels:?}"
        );
    }

    #[test]
    fn t44_the_limit_is_respected() {
        assert_eq!(search_in(&index(), "die", 2).len(), 2);
    }

    #[test]
    fn rf20d_a_new_guild_is_added_to_the_index_and_never_duplicated() {
        let dir = std::env::temp_dir().join(format!("gw2lurker-names-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(FILE);
        std::fs::write(&path, br#"{"A":"[DG] Die Garde"}"#).unwrap();

        let added = append_at(
            &path,
            &[("B".to_owned(), "[DK] die letzken Hunde".to_owned())],
        )
        .unwrap();
        assert_eq!(added, 1, "el clan nuevo se añade al índice");

        // Repetirlo no duplica ni reescribe: el índice solo crece con lo que faltaba.
        let again = append_at(
            &path,
            &[("B".to_owned(), "[DK] die letzken Hunde".to_owned())],
        )
        .unwrap();
        assert_eq!(again, 0);

        let index = read_at(&path);
        assert_eq!(index.len(), 2, "{index:?}");
        assert_eq!(
            index.get("B").map(String::as_str),
            Some("[DK] die letzken Hunde")
        );
        // Y el buscador lo encuentra por un trozo del nombre.
        assert_eq!(search_in(&index, "letzken", 5).len(), 1);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
