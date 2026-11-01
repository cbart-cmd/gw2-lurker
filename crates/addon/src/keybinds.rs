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
//! Se lee **una vez al cargar** (rule 6: nada de disco en el render) y se cachea.

use std::sync::OnceLock;

use serde::Deserialize;

/// Identificador con el que se registra el atajo (Nexus lo lista con este nombre).
pub const IDENTIFIER: &str = "GW2LURKER_CLICK_THROUGH";

/// Lo que el jugador ve cuando el atajo no se puede leer: el identificador, que es inconfundible.
const UNKNOWN: &str = "GW2LURKER_CLICK_THROUGH";

/// Una entrada de `InputBinds.json`.
#[derive(Debug, Clone, Deserialize)]
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

/// Descripción del atajo, calculada al cargar y cacheada para el hilo de render.
pub fn shortcut() -> &'static str {
    SLOT.get().map(String::as_str).unwrap_or(UNKNOWN)
}

static SLOT: OnceLock<String> = OnceLock::new();

/// Lee `InputBinds.json` y guarda la descripción del atajo. Se llama desde `load` y nunca desde
/// el render.
///
/// Devuelve la descripción que ha quedado guardada, para el log.
pub fn remember_shortcut() -> &'static str {
    let described = read_binds()
        .as_deref()
        .and_then(describe_identifier)
        .unwrap_or_else(|| UNKNOWN.to_owned());
    log::info!("atajo de Nexus para {IDENTIFIER}: {described}");
    let _ = SLOT.set(described);
    shortcut()
}

/// Contenido de `addons/Nexus/InputBinds.json`, si se puede leer.
fn read_binds() -> Option<String> {
    let path = nexus::paths::get_game_dir()
        .ok()?
        .join("addons")
        .join("Nexus")
        .join("InputBinds.json");
    std::fs::read_to_string(path).ok()
}

/// Descripción del atajo de `identifier` dentro del JSON. Pura: se prueba con el fichero real.
fn describe_identifier(json: &str) -> Option<String> {
    let binds: Vec<Bind> = serde_json::from_str(json).ok()?;
    let found = binds
        .into_iter()
        .find(|bind| bind.identifier == IDENTIFIER)?;
    Some(describe(&found))
}

/// `Ctrl+Shift+L` a partir de los modificadores y el código de escaneo.
fn describe(bind: &Bind) -> String {
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
    let key = key_name(bind.code);
    if parts.is_empty() && key.is_none() {
        return UNKNOWN.to_owned();
    }
    let mut out = parts.join("+");
    if let Some(key) = key {
        if !out.is_empty() {
            out.push('+');
        }
        out.push_str(key);
    }
    out
}

/// Nombre de la tecla a partir del **código de escaneo** (set 1). `None` si no se conoce, que es
/// lo que evita inventarse una tecla.
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

    #[test]
    fn rf18_the_shortcut_is_read_from_what_nexus_has_saved() {
        // Lo guardado para nuestro identificador: Ctrl+Shift y el código 39, que es «;» en el set 1
        // —no la L que decía el literal de antes (informe del jugador, 2026-11-01)—.
        assert_eq!(describe_identifier(REAL).as_deref(), Some("Ctrl+Shift+;"));
    }

    #[test]
    fn rf18_another_addons_binding_is_not_ours() {
        let only_other = r#"[{"Alt": false, "Code": 35, "Ctrl": true, "Identifier": "KB_TOGGLEHIDEUI", "Shift": false}]"#;
        assert_eq!(describe_identifier(only_other), None);
        assert_eq!(describe_identifier("no es json"), None);
        assert_eq!(describe_identifier("[]"), None);
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
            describe(&bind),
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
            describe(&empty),
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
        assert_eq!(describe(&bind), "Ctrl+Alt+Shift+L");
    }

    #[test]
    fn rf18_alt_is_read_from_nexuss_saved_binding() {
        let json = r#"[{"Alt":true,"Code":38,"Ctrl":true,"Identifier":"GW2LURKER_CLICK_THROUGH","Shift":true}]"#;
        assert_eq!(
            describe_identifier(json).as_deref(),
            Some("Ctrl+Alt+Shift+L")
        );
    }
}
