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
use std::sync::Mutex;

use crate::config;

/// Fichero del índice dentro de la carpeta del addon.
pub const FILE: &str = "guild-names.json";

/// Cuántos nombres tiene el índice, para que la ventana lo diga sin leer el fichero.
static INDEX_LEN: AtomicUsize = AtomicUsize::new(0);
static INDEX_IO: Mutex<()> = Mutex::new(());

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
        Ok(names) => normalize_names(names),
        Err(error) => {
            log::warn!("índice de nombres de clan mal formado ({error}); se busca solo en el API");
            BTreeMap::new()
        }
    }
}

fn normalize_names(names: BTreeMap<String, String>) -> BTreeMap<String, String> {
    let mut normalized = BTreeMap::new();
    for (id, label) in names {
        let id = gw2lurker_core::normalize_guild_id(&id).unwrap_or(id);
        normalized.entry(id).or_insert(label);
    }
    normalized
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
    let needle = needle.trim();
    if needle.chars().count() < 2 {
        return Vec::new();
    }
    let needle = needle.to_lowercase();
    let mut out: Vec<(String, String)> = names
        .iter()
        .filter(|(id, label)| matches_normalized_text(label, id, &needle))
        .map(|(id, label)| (id.clone(), label.clone()))
        .collect();
    // Primero los que **empiezan** por lo escrito (sin contar el `[TAG]`), y entre ellos los más
    // cortos: quien busca «die» quiere *Die Gäng* y *Die Garde* antes que *Never Die*
    // (2026-11-01). Después el resto por longitud, que sigue siendo más útil que el orden del id.
    out.sort_by_cached_key(|(id, label)| {
        (!starts_with_needle(label, &needle), label.len(), id.clone())
    });
    out.truncate(limit);
    out
}

/// Whether the name (not the tag) begins with what the player typed.
fn starts_with_needle(label: &str, needle: &str) -> bool {
    let name = match label.split_once("] ") {
        Some((_, name)) => name,
        None => label,
    };
    name.to_lowercase().starts_with(needle)
}

/// Whether an entry matches what the player typed. Pure, so the rule can be tested.
#[cfg(test)]
pub fn matches_text(label: &str, id: &str, needle: &str) -> bool {
    let needle = needle.trim().to_lowercase();
    matches_normalized_text(label, id, &needle)
}

fn matches_normalized_text(label: &str, id: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    label.to_lowercase().contains(needle) || id.to_lowercase().starts_with(needle)
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
    let spawned = crate::background::spawn("gw2lurker-names", move || match append(&entries) {
        Ok(added) if added > 0 => log::info!("índice de clanes: {added} nombres nuevos"),
        Ok(_) => {}
        Err(error) => log::warn!("no se pudo ampliar el índice de clanes ({error})"),
    });
    if let Err(error) = spawned {
        log::warn!("índice de clanes: no se pudo lanzar la escritura ({error})");
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
    let _guard = INDEX_IO.lock().unwrap_or_else(|error| error.into_inner());
    let mut names = read_at(path);
    let before = names.len();
    for (id, label) in entries {
        if id.trim().is_empty() || label.trim().is_empty() {
            continue;
        }
        let id = gw2lurker_core::normalize_guild_id(id).unwrap_or_else(|| id.clone());
        names.entry(id).or_insert_with(|| label.clone());
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

/// Installs a downloaded index while preserving names appended during this session.
pub(crate) fn install_at(path: &std::path::Path, downloaded: &[u8]) -> std::io::Result<usize> {
    let downloaded: BTreeMap<String, String> =
        serde_json::from_slice(downloaded).map_err(std::io::Error::other)?;
    let downloaded = normalize_names(downloaded);
    let _guard = INDEX_IO.lock().unwrap_or_else(|error| error.into_inner());
    let mut names = read_at(path);
    names.extend(downloaded);
    let json = serde_json::to_vec(&names).map_err(std::io::Error::other)?;
    let temp = path.with_extension("json.tmp");
    std::fs::write(&temp, json)?;
    std::fs::rename(&temp, path)?;
    INDEX_LEN.store(names.len(), Ordering::Relaxed);
    Ok(names.len())
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
    fn t44_a_single_multibyte_character_is_still_one_character() {
        let names = BTreeMap::from([("A".to_owned(), "[N] Ñandú".to_owned())]);
        assert!(
            search_in(&names, "ñ", 5).is_empty(),
            "el mínimo cuenta caracteres, no bytes UTF-8"
        );
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
    fn t44_names_that_start_with_the_text_come_before_the_rest() {
        // Con «die» el jugador quiere *Die …* delante de *Never Die*, aunque el otro sea más corto.
        let mut names = BTreeMap::new();
        names.insert("A".into(), "[ND] Never Die".into());
        names.insert("B".into(), "[GÄNG] Die Gäng".into());
        names.insert("C".into(), "[DB] Die Barbaren".into());
        let found: Vec<String> = search_in(&names, "die", 5)
            .into_iter()
            .map(|(_, label)| label)
            .collect();
        assert_eq!(
            found,
            vec![
                "[GÄNG] Die Gäng".to_owned(),
                "[DB] Die Barbaren".to_owned(),
                "[ND] Never Die".to_owned(),
            ],
            "los que empiezan por «die» primero y, entre ellos, el más corto"
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

    #[test]
    fn rf20d_index_guild_ids_are_normalized_before_storage() {
        const LOWER: &str = "230c9e95-e509-4b3e-aaf2-a320db7506e5";
        const CANONICAL: &str = "230C9E95-E509-4B3E-AAF2-A320DB7506E5";

        let dir =
            std::env::temp_dir().join(format!("gw2lurker-names-normalize-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(FILE);

        assert_eq!(
            append_at(&path, &[(LOWER.to_owned(), "[TAG] Clan".to_owned())]).unwrap(),
            1
        );
        assert_eq!(
            append_at(&path, &[(CANONICAL.to_owned(), "[TAG] Clan".to_owned())]).unwrap(),
            0,
            "el índice no duplica el mismo UUID con otra capitalización"
        );
        assert_eq!(
            read_at(&path).get(CANONICAL).map(String::as_str),
            Some("[TAG] Clan")
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rf20_concurrent_download_and_append_preserve_both_indexes() {
        let dir = std::env::temp_dir().join(format!("gw2lurker-index-io-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(FILE);
        std::fs::write(&path, br#"{"A":"[A] Existing"}"#).unwrap();
        let append_path = path.clone();
        let install_path = path.clone();

        std::thread::scope(|scope| {
            let append = scope.spawn(|| {
                append_at(&append_path, &[("B".to_owned(), "[B] Appended".to_owned())]).unwrap();
            });
            let install = scope.spawn(|| {
                install_at(&install_path, br#"{"C":"[C] Downloaded"}"#).unwrap();
            });
            append.join().unwrap();
            install.join().unwrap();
        });

        let index = read_at(&path);
        assert!(
            index.contains_key("B"),
            "la adición no se pierde: {index:?}"
        );
        assert!(
            index.contains_key("C"),
            "la descarga no se pierde: {index:?}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
