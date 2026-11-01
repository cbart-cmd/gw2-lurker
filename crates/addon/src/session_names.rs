//! Nombres de los clanes que han reclamado algo **en esta sesión**, en `temp.json` (RF-20b).
//!
//! No van a `settings.json` porque no son datos permanentes: este fichero se **regenera en cada
//! arranque** (se borra al cargar el addon) y solo crece con lo que se ve mientras el juego está
//! abierto (petición del jugador, 2026-11-01: «llévalo a un temp.json que se regenere cada vez que
//! entras en el juego; así no hace falta guardarlo en memoria pero no lo guardamos mucho tiempo»).
//!
//! El reparto completo de nombres de clan queda así (ADR-016):
//! - `settings.json` → **solo los clanes que sigues** (`Config::guild_names`).
//! - `temp.json` → los de la sesión: clanes que han reclamado algo desde que se arrancó el juego.
//! - `guild-names.json` → el índice de búsqueda por texto, que no se tiene en memoria.
//!
//! El disco lo tocan `reset`/`save`, llamados desde el arranque y desde el hilo del worker, nunca
//! desde el render.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use crate::config;

/// Fichero de la sesión dentro de la carpeta del addon.
pub const FILE: &str = "temp.json";

/// Tope de seguridad: una sesión muy larga no debe dejar un fichero enorme.
pub const MAX_NAMES: usize = 500;

static SLOT: OnceLock<Mutex<BTreeMap<String, String>>> = OnceLock::new();

fn slot() -> &'static Mutex<BTreeMap<String, String>> {
    SLOT.get_or_init(|| Mutex::new(BTreeMap::new()))
}

/// Ruta del fichero, o `None` si la carpeta del addon no está disponible.
pub fn path() -> Option<PathBuf> {
    Some(config::addon_dir().ok()?.join(FILE))
}

/// Vacía la sesión y **borra** el fichero: se llama al cargar el addon, así que cada vez que se
/// entra al juego se empieza de cero.
pub fn reset() {
    slot()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clear();
    let Some(path) = path() else {
        return;
    };
    match std::fs::remove_file(&path) {
        Ok(()) => log::info!("nombres de la sesión: {FILE} regenerado (se borra al arrancar)"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => log::warn!("no se pudo borrar {}: {error}", path.display()),
    }
}

/// Copia de lo que hay en la sesión, para publicarlo con el dashboard.
pub fn all() -> BTreeMap<String, String> {
    slot()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone()
}

/// Añade los nombres que faltaban. Devuelve `true` si creció (y hay que escribir el fichero).
pub fn merge(names: &BTreeMap<String, String>) -> bool {
    let mut slot = slot().lock().unwrap_or_else(|error| error.into_inner());
    let before = slot.len();
    for (id, label) in names {
        if slot.len() >= MAX_NAMES {
            break;
        }
        if id.trim().is_empty() || label.trim().is_empty() {
            continue;
        }
        slot.entry(id.clone()).or_insert_with(|| label.clone());
    }
    if slot.len() != before && slot.len() == MAX_NAMES {
        log::info!("nombres de la sesión: tope de {MAX_NAMES} alcanzado");
    }
    slot.len() != before
}

/// Escribe la sesión en `temp.json`. Devuelve cuántos nombres hay.
pub fn save() -> std::io::Result<usize> {
    let names = all();
    let path = path().ok_or_else(|| std::io::Error::other("sin carpeta del addon"))?;
    save_to(&path, &names)?;
    Ok(names.len())
}

/// [`save`] contra una ruta dada, para poder probarlo sin tocar los ficheros del jugador.
pub fn save_to(path: &std::path::Path, names: &BTreeMap<String, String>) -> std::io::Result<()> {
    let json = serde_json::to_vec(names).map_err(std::io::Error::other)?;
    let temp = path.with_extension("json.tmp");
    std::fs::write(&temp, json)?;
    std::fs::rename(&temp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "gw2lurker-session-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn rf20b_the_session_file_is_written_and_read_back() {
        let dir = dir();
        let path = dir.join(FILE);
        let mut names = BTreeMap::new();
        names.insert("SEGUIDO".to_owned(), "[YaK] Dolyak Sanctuary".to_owned());
        names.insert("DE_PASO".to_owned(), "[DB] Die Barbaren".to_owned());

        save_to(&path, &names).unwrap();
        let back: BTreeMap<String, String> =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(back, names);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rf20b_the_session_never_grows_past_the_cap() {
        // Una sesión larga no puede dejar un `temp.json` sin freno. Se llama al `merge` de verdad.
        slot().lock().unwrap().clear();
        let mut names = BTreeMap::new();
        for index in 0..(MAX_NAMES + 20) {
            names.insert(format!("{index:04}"), format!("[T{index}] Algo"));
        }

        assert!(merge(&names), "la sesión crece");
        assert_eq!(all().len(), MAX_NAMES, "pero no pasa del tope");

        slot().lock().unwrap().clear();
    }
}
