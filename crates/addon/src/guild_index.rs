//! Descarga del índice de nombres de clan (`guild-names.json`) desde el repositorio de distribución.
//!
//! El índice pesa cientos de KB y no cabe en la DLL, así que se publica como ZIP junto a su
//! `.md5` en `gw2addonreleases` y el addon se lo baja **la primera vez** (o cuando el jugador
//! pulsa «Descargar»). Es el mismo mecanismo que el addon hermano `gw2-rotation-builds` usa para
//! su `data.zip`: descargar → comprobar el MD5 publicado → descomprimir.
//!
//! Nada de esto pasa por el hilo de render: `ensure_present`/`download` lanzan un hilo y la
//! ventana solo lee [`status`].

use std::sync::{Mutex, OnceLock};

use gw2lurker_api::Transport;

use crate::net::WinHttpTransport;

/// Dónde se publica el ZIP del índice. El jugador puede cambiarlo en los ajustes.
pub const DEFAULT_URL: &str =
    "https://raw.githubusercontent.com/cbart-cmd/gw2addonreleases/main/gw2_lurker_guilds.zip";

/// Sufijo del hash publicado junto al ZIP (`x.zip` → `x.zip.md5`).
const MD5_SUFFIX: &str = ".md5";

/// Nombre que se busca dentro del ZIP.
const ENTRY: &str = "guild-names.json";

/// Topes de lo que se acepta descomprimir de un ZIP que no controlamos.
const MAX_ENTRIES: usize = 64;
const MAX_BYTES: u64 = 64 * 1024 * 1024;

/// What the settings window shows.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Status {
    /// Nadie ha pedido nada todavía.
    #[default]
    Idle,
    Downloading,
    /// Instalado, con cuántos nombres.
    Done(usize),
    Failed(String),
}

static STATUS: OnceLock<Mutex<Status>> = OnceLock::new();

fn slot() -> &'static Mutex<Status> {
    STATUS.get_or_init(|| Mutex::new(Status::Idle))
}

/// Current state, copied so the window never holds the lock while drawing.
pub fn status() -> Status {
    slot()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone()
}

fn set_status(status: Status) {
    *slot().lock().unwrap_or_else(|error| error.into_inner()) = status;
}

/// Descarga el índice si no está en disco. Se llama una vez al cargar el addon.
pub fn ensure_present() {
    if crate::names::exists() {
        return;
    }
    let url = crate::config().guild_index_url.clone();
    log::info!("índice de clanes: no está en disco, se pide a {url}");
    download(url);
}

/// Pide una descarga en su propio hilo. Devuelve `false` si ya hay una en marcha.
pub fn download(url: String) -> bool {
    {
        let mut status = slot().lock().unwrap_or_else(|error| error.into_inner());
        if *status == Status::Downloading {
            return false;
        }
        *status = Status::Downloading;
    }
    let spawned = std::thread::Builder::new()
        .name("gw2lurker-index".to_owned())
        .spawn(move || match run(&url) {
            Ok(count) => {
                log::info!("índice de clanes: {count} nombres instalados desde {url}");
                set_status(Status::Done(count));
            }
            Err(error) => {
                log::warn!("índice de clanes: {error}");
                set_status(Status::Failed(error));
            }
        });
    if spawned.is_err() {
        set_status(Status::Failed(
            "no se pudo lanzar la descarga del índice".to_owned(),
        ));
        return false;
    }
    true
}

/// Descarga, verifica y descomprime. Corre en el hilo de la descarga.
fn run(url: &str) -> Result<usize, String> {
    if !url.starts_with("https://") {
        return Err("la dirección del índice tiene que empezar por https://".to_owned());
    }
    let transport = WinHttpTransport::default();
    let bytes = transport
        .get(url)
        .map_err(|error| format!("no se pudo descargar {url}: {error}"))?;

    // El hash va **antes** de escribir nada: un ZIP truncado nunca llega al disco.
    let published = transport
        .get(&format!("{url}{MD5_SUFFIX}"))
        .ok()
        .and_then(|body| crate::md5::parse(&String::from_utf8_lossy(&body)));
    match published.as_deref() {
        Some(expected) => {
            let actual = crate::md5::hex(&bytes);
            if actual != expected {
                return Err(format!(
                    "el MD5 del índice no coincide (publicado {expected}, descargado {actual}); no se instala"
                ));
            }
            log::info!("índice de clanes: {actual} verificado ({expected})");
        }
        None => log::warn!(
            "índice de clanes: no hay {url}{MD5_SUFFIX} legible; se instala sin comprobar el hash"
        ),
    }

    let json = unpack(&bytes, Limits::default())?;
    let count = serde_json::from_slice::<std::collections::BTreeMap<String, String>>(&json)
        .map_err(|error| format!("el índice descargado no es un mapa de nombres: {error}"))?
        .len();
    let path = crate::names::path().ok_or_else(|| "sin carpeta del addon".to_owned())?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    }
    let temp = path.with_extension("json.tmp");
    std::fs::write(&temp, &json).map_err(|error| error.to_string())?;
    std::fs::rename(&temp, &path).map_err(|error| error.to_string())?;
    Ok(count)
}

/// Limits accepted while unpacking, so a hostile archive cannot fill the disk.
#[derive(Debug, Clone, Copy)]
struct Limits {
    entries: usize,
    bytes: u64,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            entries: MAX_ENTRIES,
            bytes: MAX_BYTES,
        }
    }
}

/// Bytes of `guild-names.json` inside the archive. Pure, so it can be tested with a fixture.
fn unpack(bytes: &[u8], limits: Limits) -> Result<Vec<u8>, String> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))
        .map_err(|error| format!("el ZIP del índice no se puede abrir: {error}"))?;
    if archive.len() > limits.entries {
        return Err(format!(
            "el ZIP trae {} entradas (máximo {})",
            archive.len(),
            limits.entries
        ));
    }
    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|error| format!("entrada {index} del ZIP: {error}"))?;
        if file.is_dir() {
            continue;
        }
        if file.is_symlink() {
            return Err(format!("el ZIP trae un enlace simbólico: {}", file.name()));
        }
        if file.size() > limits.bytes {
            return Err(format!("{} descomprime más de lo aceptado", file.name()));
        }
        let name = file.name().replace('\\', "/");
        if !name.ends_with(ENTRY) {
            continue;
        }
        let mut out = Vec::with_capacity(file.size() as usize);
        std::io::Read::read_to_end(&mut file, &mut out)
            .map_err(|error| format!("{name}: {error}"))?;
        return Ok(out);
    }
    Err(format!("el ZIP no trae ningún {ENTRY}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// ZIP en memoria con un fichero; `Stored` para no depender de un compresor al probar.
    fn zip_with(name: &str, content: &[u8]) -> Vec<u8> {
        let mut out = std::io::Cursor::new(Vec::new());
        {
            let mut writer = zip::ZipWriter::new(&mut out);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);
            writer.start_file(name, options).unwrap();
            writer.write_all(content).unwrap();
            writer.finish().unwrap();
        }
        out.into_inner()
    }

    #[test]
    fn rf20_the_index_is_found_inside_the_zip() {
        let json = br#"{"A":"[DG] Die Garde"}"#;
        let packed = zip_with("guild-names.json", json);
        assert_eq!(unpack(&packed, Limits::default()).unwrap(), json.to_vec());
    }

    #[test]
    fn rf20_the_index_is_found_inside_a_folder_of_the_zip() {
        let json = br#"{"A":"[DG] Die Garde"}"#;
        let packed = zip_with("data/guild-names.json", json);
        assert_eq!(unpack(&packed, Limits::default()).unwrap(), json.to_vec());
    }

    #[test]
    fn rf20_a_zip_without_the_index_is_an_error() {
        let packed = zip_with("otra-cosa.json", b"{}");
        assert!(unpack(&packed, Limits::default()).is_err());
    }

    #[test]
    fn rf20_a_zip_with_too_many_entries_is_refused() {
        let packed = zip_with("guild-names.json", b"{}");
        let limits = Limits {
            entries: 0,
            bytes: MAX_BYTES,
        };
        assert!(unpack(&packed, limits).is_err());
    }

    #[test]
    fn rf20_a_non_zip_is_refused() {
        assert!(unpack(b"no soy un zip", Limits::default()).is_err());
    }
}
