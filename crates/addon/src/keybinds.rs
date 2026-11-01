//! La combinación de teclas que **Nexus tiene guardada** para el atajo del addon (RF-18).
//!
//! Hasta la 0.23.2 el mensaje del click-through decía «CTRL+SHIFT+L» escrito a mano. Si el
//! jugador cambia el atajo en Nexus, ese texto miente: el 2026-11-01 el jugador lo había movido y
//! el addon le seguía diciendo otra combinación. Nexus no expone un lector de atajos a los addons
//! (`nexus-rs` 0.12 solo tiene registrar/desregistrar/invocar), así que la única fuente real es
//! `addons/Nexus/InputBinds.json`, que Nexus reescribe cuando el jugador cambia algo.
//!
//! El fichero guarda el **código de escaneo** de la tecla (set 1) y tres modificadores. Aquí se
//! traduce a texto; si la tecla no está en la tabla se dice el número en vez de inventarla.
//!
//! Se lee al cargar y, después, el worker vuelve a leerlo cuando cambia su fecha de modificación
//! (rule 6: nada de disco en el render). Hasta la 0.26.2 solo se leía al cargar: el jugador cambió
//! el atajo a Alt+Shift+Ñ con el juego abierto y el aviso seguía diciendo Ctrl+Shift+Ñ (informe del
//! 2026-11-01). El nombre de la tecla se calcula en el hilo que pide el texto, porque Windows lo da
//! según la distribución de teclado de ese hilo.

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;

use serde::Deserialize;

/// Identificador con el que se registra el atajo (Nexus lo lista con este nombre).
pub const IDENTIFIER: &str = "GW2LURKER_CLICK_THROUGH";

/// Lo que el jugador ve cuando el atajo no se puede leer: el identificador, que es inconfundible.
const UNKNOWN: &str = "GW2LURKER_CLICK_THROUGH";

/// Una entrada de `InputBinds.json`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct Bind {
    #[serde(rename = "Alt", default)]
    alt: bool,
    #[serde(rename = "Code", default)]
    code: u16,
    #[serde(rename = "Ctrl", default)]
    ctrl: bool,
    #[serde(rename = "Identifier", default)]
    identifier: String,
    #[serde(rename = "Shift", default)]
    shift: bool,
}

/// Lo último leído del fichero y su descripción. `pending` es una lectura nueva que aún no se ha
/// convertido en texto: se convierte en el hilo que llama a [`shortcut`].
struct Shown {
    described: String,
    /// `Some(None)`: el fichero ya no tiene nuestro atajo o no se puede leer.
    pending: Option<Option<Bind>>,
}

impl Shown {
    const fn new() -> Self {
        Self {
            described: String::new(),
            pending: None,
        }
    }

    /// Aplica la lectura pendiente, si la hay, y devuelve la descripción vigente.
    fn current(&mut self, layout: impl Fn(u16) -> Option<String>) -> &str {
        if let Some(read) = self.pending.take() {
            let described = read.map_or_else(|| UNKNOWN.to_owned(), |bind| describe(&bind, layout));
            if described != self.described {
                log::info!("atajo de Nexus para {IDENTIFIER}: {described}");
            }
            self.described = described;
        }
        if self.described.is_empty() {
            UNKNOWN
        } else {
            &self.described
        }
    }
}

static SHOWN: Mutex<Shown> = Mutex::new(Shown::new());
static WATCH: Mutex<Option<Watch>> = Mutex::new(None);

/// Descripción del atajo según lo último que Nexus ha guardado. No toca el disco.
pub fn shortcut() -> String {
    match SHOWN.lock() {
        Ok(mut shown) => shown.current(crate::ffi::keys::layout_key_name).to_owned(),
        Err(_) => UNKNOWN.to_owned(),
    }
}

/// Lee `InputBinds.json` por primera vez y deja la ruta para [`refresh`]. Se llama desde `load`
/// y nunca desde el render.
pub fn remember_shortcut() {
    let Some(path) = binds_path() else {
        log::info!("atajo de Nexus para {IDENTIFIER}: {UNKNOWN} (sin carpeta del juego)");
        return;
    };
    let mut watch = Watch::new(path);
    let read = watch.poll();
    if let Ok(mut slot) = WATCH.lock() {
        *slot = Some(watch);
    }
    if let Some(read) = read {
        store(read);
    }
    shortcut();
}

/// Vuelve a leer el atajo si Nexus ha reescrito el fichero. La llama el worker, nunca el render.
pub fn refresh() {
    let read = match WATCH.lock() {
        Ok(mut slot) => slot.as_mut().and_then(Watch::poll),
        Err(_) => None,
    };
    if let Some(read) = read {
        store(read);
    }
}

fn store(read: Option<Bind>) {
    if let Ok(mut shown) = SHOWN.lock() {
        shown.pending = Some(read);
    }
}

/// `addons/Nexus/InputBinds.json` dentro de la carpeta del juego.
fn binds_path() -> Option<PathBuf> {
    Some(
        nexus::paths::get_game_dir()
            .ok()?
            .join("addons")
            .join("Nexus")
            .join("InputBinds.json"),
    )
}

/// Vigila la fecha de modificación del fichero para leerlo solo cuando cambia.
struct Watch {
    path: PathBuf,
    stamp: Option<SystemTime>,
    seen: bool,
}

impl Watch {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            stamp: None,
            seen: false,
        }
    }

    /// `None` si el fichero no ha cambiado desde la última vez; si no, nuestro atajo tal como
    /// está ahora (`Some(None)` si ya no está o no se puede leer).
    fn poll(&mut self) -> Option<Option<Bind>> {
        let stamp = std::fs::metadata(&self.path)
            .and_then(|meta| meta.modified())
            .ok();
        if self.seen && stamp == self.stamp {
            return None;
        }
        self.seen = true;
        self.stamp = stamp;
        Some(
            std::fs::read_to_string(&self.path)
                .ok()
                .as_deref()
                .and_then(find_bind),
        )
    }
}

/// Nuestro atajo dentro del JSON de Nexus.
fn find_bind(json: &str) -> Option<Bind> {
    let binds: Vec<Bind> = serde_json::from_str(json).ok()?;
    binds.into_iter().find(|bind| bind.identifier == IDENTIFIER)
}

/// Descripción del atajo de `identifier` dentro del JSON. Pura: se prueba con el fichero real.
///
/// `layout` nombra la tecla según la distribución del jugador (en un teclado español el código
/// 0x27 es «Ñ», no el «;» de la tabla US: informe del 2026-11-01). Si no la conoce, la tabla.
#[cfg(test)]
fn describe_identifier(json: &str, layout: impl Fn(u16) -> Option<String>) -> Option<String> {
    find_bind(json).map(|found| describe(&found, layout))
}

/// `Ctrl+Shift+L` a partir de los modificadores y el código de escaneo.
fn describe(bind: &Bind, layout: impl Fn(u16) -> Option<String>) -> String {
    let mut parts: Vec<&str> = Vec::new();
    if bind.ctrl {
        parts.push("Ctrl");
    }
    if bind.alt {
        parts.push("Alt");
    }
    if bind.shift {
        parts.push("Shift");
    }
    let key = layout(bind.code)
        .filter(|name| !name.trim().is_empty())
        .or_else(|| key_name(bind.code).map(str::to_owned));
    if parts.is_empty() && key.is_none() {
        return UNKNOWN.to_owned();
    }
    let mut out = parts.join("+");
    if let Some(key) = key {
        if !out.is_empty() {
            out.push('+');
        }
        out.push_str(&key);
    }
    out
}

/// Nombre de la tecla en la distribución **US** a partir del código de escaneo (set 1): solo
/// cuando Windows no da el de la distribución del jugador. `None` si no se conoce, que es lo
/// que evita inventarse una tecla.
fn key_name(code: u16) -> Option<&'static str> {
    Some(match code {
        0x02 => "1",
        0x03 => "2",
        0x04 => "3",
        0x05 => "4",
        0x06 => "5",
        0x07 => "6",
        0x08 => "7",
        0x09 => "8",
        0x0A => "9",
        0x0B => "0",
        0x0C => "-",
        0x0D => "=",
        0x10 => "Q",
        0x11 => "W",
        0x12 => "E",
        0x13 => "R",
        0x14 => "T",
        0x15 => "Y",
        0x16 => "U",
        0x17 => "I",
        0x18 => "O",
        0x19 => "P",
        0x1A => "[",
        0x1B => "]",
        0x1E => "A",
        0x1F => "S",
        0x20 => "D",
        0x21 => "F",
        0x22 => "G",
        0x23 => "H",
        0x24 => "J",
        0x25 => "K",
        0x26 => "L",
        0x27 => ";",
        0x28 => "'",
        0x29 => "`",
        0x2B => "\\",
        0x2C => "Z",
        0x2D => "X",
        0x2E => "C",
        0x2F => "V",
        0x30 => "B",
        0x31 => "N",
        0x32 => "M",
        0x33 => ",",
        0x34 => ".",
        0x35 => "/",
        0x39 => "Espacio",
        0x3B => "F1",
        0x3C => "F2",
        0x3D => "F3",
        0x3E => "F4",
        0x3F => "F5",
        0x40 => "F6",
        0x41 => "F7",
        0x42 => "F8",
        0x43 => "F9",
        0x44 => "F10",
        0x57 => "F11",
        0x58 => "F12",
        0x47 => "Inicio",
        0x48 => "Arriba",
        0x49 => "RePág",
        0x4B => "Izquierda",
        0x4D => "Derecha",
        0x4F => "Fin",
        0x50 => "Abajo",
        0x51 => "AvPág",
        0x52 => "Insert",
        0x53 => "Supr",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// El fichero real de Nexus (copiado tal cual el 2026-11-01).
    const REAL: &str = r#"[
      {"Alt": false, "Code": 39, "Ctrl": true, "Identifier": "GW2LURKER_CLICK_THROUGH", "Passthrough": false, "Shift": true, "Type": 1},
      {"Alt": false, "Code": 35, "Ctrl": true, "Identifier": "KB_TOGGLEHIDEUI", "Passthrough": false, "Shift": false, "Type": 1}
    ]"#;

    /// Sin distribución: lo que queda es la tabla US.
    fn no_layout(_: u16) -> Option<String> {
        None
    }

    /// Lo que Windows devuelve con un teclado español para las teclas de la prueba.
    fn spanish(code: u16) -> Option<String> {
        match code {
            0x27 => Some("Ñ".to_owned()),
            0x26 => Some("L".to_owned()),
            _ => None,
        }
    }

    #[test]
    fn rf18_the_shortcut_is_read_from_what_nexus_has_saved() {
        // Lo guardado para nuestro identificador: Ctrl+Shift y el código 39, que es «;» en el set 1
        // —no la L que decía el literal de antes (informe del jugador, 2026-11-01)—.
        assert_eq!(
            describe_identifier(REAL, no_layout).as_deref(),
            Some("Ctrl+Shift+;")
        );
    }

    #[test]
    fn rf18_the_key_is_named_with_the_players_keyboard_layout() {
        // El mismo fichero real, en el teclado español del jugador: el código 39 es la «Ñ» que
        // él pulsa (informe del 2026-11-01: «tiene ctrl+shift+ñ pero me dice ctrl+shift+;»).
        assert_eq!(
            describe_identifier(REAL, spanish).as_deref(),
            Some("Ctrl+Shift+Ñ")
        );
        let blank = |_: u16| Some("  ".to_owned());
        assert_eq!(
            describe_identifier(REAL, blank).as_deref(),
            Some("Ctrl+Shift+;"),
            "un nombre vacío no tapa la tabla"
        );
    }

    #[test]
    fn rf18_windows_names_a_key_in_the_current_layout() {
        // Llamada real a user32 (no a Nexus): la L está en el mismo sitio en QWERTY, QWERTZ y AZERTY.
        assert_eq!(
            crate::ffi::keys::layout_key_name(0x26).as_deref(),
            Some("L")
        );
        assert_eq!(crate::ffi::keys::layout_key_name(0), None);
    }

    #[test]
    fn rf18_a_rebinding_made_while_playing_reaches_the_message() {
        // Informe del 2026-11-01: el jugador puso Alt+Shift+Ñ con el juego abierto y el aviso del
        // click-through seguía diciendo Ctrl+Shift+Ñ, porque el fichero solo se leía al cargar.
        let path = std::env::temp_dir().join(format!("gw2lurker_rf18_{}.json", std::process::id()));
        let write = |json: &str, at: SystemTime| {
            std::fs::write(&path, json).ok();
            if let Ok(file) = std::fs::File::options().write(true).open(&path) {
                file.set_modified(at).ok();
            }
        };
        let start = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_800_000_000);
        write(REAL, start);
        let mut watch = Watch::new(path.clone());
        let mut shown = Shown::new();

        shown.pending = watch.poll();
        assert_eq!(shown.current(spanish), "Ctrl+Shift+Ñ");
        assert_eq!(watch.poll(), None, "sin cambios no se vuelve a leer");

        let rebound = REAL.replacen(
            r#""Alt": false, "Code": 39, "Ctrl": true"#,
            r#""Alt": true, "Code": 39, "Ctrl": false"#,
            1,
        );
        assert_ne!(rebound, REAL, "la fixture tiene que cambiar");
        write(&rebound, start + std::time::Duration::from_secs(60));
        shown.pending = watch.poll();
        assert_eq!(shown.current(spanish), "Alt+Shift+Ñ");

        std::fs::remove_file(&path).ok();
        shown.pending = watch.poll();
        assert_eq!(
            shown.current(spanish),
            UNKNOWN,
            "sin fichero no se inventa un atajo"
        );
    }

    #[test]
    fn rf18_another_addons_binding_is_not_ours() {
        let only_other = r#"[{"Alt": false, "Code": 35, "Ctrl": true, "Identifier": "KB_TOGGLEHIDEUI", "Shift": false}]"#;
        assert_eq!(describe_identifier(only_other, spanish), None);
        assert_eq!(describe_identifier("no es json", spanish), None);
        assert_eq!(describe_identifier("[]", spanish), None);
    }

    #[test]
    fn rf18_a_key_we_do_not_know_is_not_invented() {
        let bind = Bind {
            alt: false,
            code: 0xEE,
            ctrl: true,
            identifier: IDENTIFIER.to_owned(),
            shift: false,
        };
        assert_eq!(
            describe(&bind, no_layout),
            "Ctrl",
            "mejor solo el modificador que una tecla falsa"
        );

        let empty = Bind {
            alt: false,
            code: 0,
            ctrl: false,
            identifier: IDENTIFIER.to_owned(),
            shift: false,
        };
        assert_eq!(
            describe(&empty, no_layout),
            UNKNOWN,
            "sin nada que decir, el identificador"
        );
    }

    #[test]
    fn rf18_modifiers_appear_in_a_stable_order() {
        let bind = Bind {
            alt: true,
            code: 0x26,
            ctrl: true,
            identifier: IDENTIFIER.to_owned(),
            shift: true,
        };
        assert_eq!(describe(&bind, spanish), "Ctrl+Alt+Shift+L");
    }

    #[test]
    fn rf18_alt_is_read_from_nexuss_saved_binding() {
        let json = r#"[{"Alt":true,"Code":38,"Ctrl":true,"Identifier":"GW2LURKER_CLICK_THROUGH","Shift":true}]"#;
        assert_eq!(
            describe_identifier(json, no_layout).as_deref(),
            Some("Ctrl+Alt+Shift+L")
        );
    }
}
