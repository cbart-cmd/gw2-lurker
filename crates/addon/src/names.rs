//! Caché de `id → [TAG] Nombre` de los clanes, persistida en `guild-names.json` (RF-20, T-44).
//!
//! Los nombres de clan **no cambian**, así que se guardan para siempre: se piden una vez y no se
//! vuelven a pedir. Sin esto, fuera de WvW (el Bastión del Armisticio) las listas enseñaban el id
//! largo de 36 caracteres hasta que una partida volvía a resolverlo (informe del jugador,
//! 2026-11-01).
//!
//! El fichero es propio y no `settings.json` a propósito: la caché puede crecer a miles de
//! entradas (las que trae el índice del buscador) y `settings.json` se reescribe entero cada vez
//! que se mueve una ventana.
//!
//! Todo lo de aquí es datos planos; el disco lo tocan `load`/`save`, que se llaman desde el
//! arranque y desde el hilo del worker, nunca desde el render.

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use crate::config;

/// Fichero de la caché dentro de la carpeta del addon.
pub const FILE: &str = "guild-names.json";

fn slot() -> &'static Mutex<BTreeMap<String, String>> {
    static SLOT: OnceLock<Mutex<BTreeMap<String, String>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(BTreeMap::new()))
}

/// Ruta del fichero, o `None` si la carpeta del addon no está disponible.
pub fn path() -> Option<std::path::PathBuf> {
    Some(config::addon_dir().ok()?.join(FILE))
}

/// Lee la caché del disco. Se llama una vez al cargar; un fichero roto es una caché vacía.
pub fn load() {
    let Some(path) = path() else {
        return;
    };
    let Ok(raw) = std::fs::read(&path) else {
        return;
    };
    match serde_json::from_slice::<BTreeMap<String, String>>(&raw) {
        Ok(names) => {
            let count = names.len();
            *slot().lock().unwrap_or_else(|error| error.into_inner()) = names;
            log::info!("nombres de clan en caché: {count}");
        }
        Err(error) => log::warn!("caché de nombres de clan ilegible ({error}); se empieza vacía"),
    }
}

/// Escribe la caché en el disco. Devuelve cuántos nombres hay.
pub fn save() -> std::io::Result<usize> {
    let names = all();
    let path = path().ok_or_else(|| std::io::Error::other("sin carpeta del addon"))?;
    let json = serde_json::to_vec(&names).map_err(std::io::Error::other)?;
    let temp = path.with_extension("json.tmp");
    std::fs::write(&temp, json)?;
    std::fs::rename(&temp, &path)?;
    Ok(names.len())
}

/// Copia de toda la caché, para publicarla con el dashboard.
pub fn all() -> BTreeMap<String, String> {
    slot()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone()
}

/// `[TAG] Nombre` de un clan, si se conoce.
pub fn get(id: &str) -> Option<String> {
    slot()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .get(id)
        .cloned()
}

/// Añade nombres nuevos. Devuelve `true` si la caché creció (hay que guardarla).
pub fn merge(unresolved: &BTreeMap<String, String>) -> bool {
    let mut names = slot().lock().unwrap_or_else(|error| error.into_inner());
    let before = names.len();
    for (id, label) in unresolved {
        names.entry(id.clone()).or_insert_with(|| label.clone());
    }
    names.len() != before
}

/// Clanes de la caché cuya etiqueta (nombre o tag) contiene `needle`, sin distinguir mayúsculas.
///
/// Es una **subcadena**, a diferencia del API: `/v2/guild/search?name=` devuelve **un solo** clan
/// y solo si el texto empieza su nombre, así que «dolyak» no encuentra *Dolyak Sanctuary* y «DIE»
/// encuentra *Die* pero no las demás (medido el 2026-11-01).
pub fn search(needle: &str, limit: usize) -> Vec<(String, String)> {
    let needle = needle.trim().to_lowercase();
    if needle.len() < 2 {
        return Vec::new();
    }
    let names = slot().lock().unwrap_or_else(|error| error.into_inner());
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

/// Whether a cached entry matches what the player typed. Pure, so the rule can be tested.
pub fn matches_text(label: &str, id: &str, needle: &str) -> bool {
    let needle = needle.trim().to_lowercase();
    if needle.is_empty() {
        return false;
    }
    label.to_lowercase().contains(&needle) || id.to_lowercase().starts_with(&needle)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(search("d", 5).is_empty());
    }
}
