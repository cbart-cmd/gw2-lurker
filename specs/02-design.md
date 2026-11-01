# 02 – Diseño

> **Estado:** este documento describe **el código que existe hoy** (2026-11-01). Lo que se
> planificó y no se hizo, o se hizo de otra forma, vive en `03-tasks.md` (histórico) y en los ADR.

## 1. Estructura del workspace

```
gw2-lurker/                   ← raíz del repo (el nombre del remoto: github.com/cbart-cmd/gw2-lurker)
├── Cargo.toml                # workspace + [workspace.lints] + [profile.release]
├── Cargo.lock                # versionado
├── rustfmt.toml              # raíz: rustfmt lo busca aquí
├── rust-toolchain.toml       # versión de Rust fijada
├── deny.toml                 # raíz: cargo-deny lo busca aquí
├── install.bat / install.ps1 # copia la DLL a <juego>\addons comprobando tamaño, MZ y MD5
├── .github/workflows/ci.yml  # fmt · clippy · test · loc · trazabilidad · deny/audit · release msvc + tamaño
├── data/                     # datos del addon, versionados
│   ├── wvw-teams.json        # id de equipo → nombre en inglés (empaquetado en la DLL)
│   ├── team-names.json       # nombre inglés → es/fr/de (empaquetado en la DLL, se edita a mano)
│   ├── guild-names.json      # índice de búsqueda (9 886 nombres) — NO va en la DLL
│   ├── gw2_lurker_guilds.zip # ese índice empaquetado: es lo que descarga el addon
│   └── gw2_lurker_guilds.zip.md5
├── crates/
│   ├── core/                 # lógica pura; #![forbid(unsafe_code)]
│   ├── api/                  # cliente HTTP (transporte inyectable) y parseo; #![forbid(unsafe_code)]
│   └── addon/                # cdylib: Nexus + imgui; único sitio con `unsafe`, en ffi/
├── docs/                     # documentación de usuario y de publicación (RELEASE.md)
├── specs/                    # diseño: requisitos, diseño, tareas, ADR…
├── scripts/                  # comprobaciones de CI y utilidades
└── README.md · CHANGELOG.md · PROGRESO.md · bugs.md
```

**No hay crate `store`**: el historial vive en memoria (ADR-015). `api` se testea con un transporte
simulado; `core` con fixtures reales del API; `addon` con tests unitarios puros y unas pocas pruebas
de red marcadas `#[ignore]`.

## 2. Integración con Nexus (`zerthox/nexus-rs` 0.12.0)

Verificado leyendo su fuente y en el juego: `export!` (nombre, firma, `load`/`unload`, `flags`,
`provider`, `update_link`), `render` con `RenderType::{Render, OptionsRender}`,
`mumble_link` (`read_map_id`, `read_identity_string`…), `paths`, `keybind::register_keybind_with_struct`,
`event::extras` (idioma) y `arc` (`EV_ARCDPS_SELF_JOIN`). El estado de la DLL lo gestiona Nexus; el
addon solo avisa de que hace falta reiniciar (ADR-007).

## 3. Flujo de datos

```
Mumble (map_id/map_type, identity)  ─┬─> context/gate  -> ¿se sondea? ¿se dibuja?
                                     └─> team (id de equipo guardado, roster por cuenta, arcdps)
worker (hilo propio, cada poll_secs):
   GET /v2/wvw/matches?ids=<una partida>  -> Snapshot -> diff(prev, cur) -> Vec<Event>
   -> Dashboard (Mutex, memoria): historial del ciclo, alertas, claims, colores, nombres
   -> catálogo: objetivos/mapas/nombres de clan (una vez por sesión, con presupuesto por sondeo)
render (hilo del juego): lee el Dashboard (clon corto) y dibuja; escribe solo en `runtime()`
```

- `AssignmentMonitor` cachea `/v2/wvw/timers/teamAssignment` durante 30 min. Después de la fecha de
  la región, consulta `/v2/wvw/matches/overview` como mucho una vez por minuto; si el match vigilado
  desaparece, pide `ids=all` y vuelve a resolver el equipo. Si la selección cambia o queda obsoleta,
  limpia el historial/alertas y reinicia el baseline; una fecha vencida sin cambio confirmado no
  limpia nada.
- El **Dashboard** es la única fuente del render: se copia lo necesario (alertas ya filtradas, listas
  ya resueltas) para no tener el candado mientras se dibuja.
- El render **nunca** hace red ni E/S: la configuración se escribe con `writer` (hilo propio) y los
  ficheros de datos (índice, `temp.json`, teselas, ajustes) los tocan otros hilos.
- La configuración editable vive en `RUNTIME` (`OnceLock<Mutex<Config>>`); `crate::config()` es la
  copia **del arranque** y no cambia nunca.

## 4. Modelo de datos (núcleo)

```rust
enum TeamColor { Red, Green, Blue, Neutral }
enum ObjectiveType { Camp, Castle, Keep, Tower, Ruins, Mercenary, Spawn, Resource, Generic, Bloodlust, Unknown }

struct Objective { id, map_id, kind: ObjectiveType, owner: TeamColor, last_flipped,
                   claimed_by: Option<String>, claimed_at: Option<DateTime<Utc>>,
                   points_tick, points_capture, yaks_delivered, guild_upgrades }
struct Snapshot  { match_id, taken_at, objectives }
enum Event { Flip{..}, Claim{.., late}, Unclaim{..}, Gap{..}, MatchChanged{..}, Baseline{..} }
```

Datos estáticos: `/v2/wvw/objectives?ids=all&lang=<código>` (una vez por idioma),
`/v2/maps` (una vez por sesión), `/v2/wvw/guilds/:region` (caché en memoria 30 min; solo para el
buscador), `/v2/guild/:id` (uno por clan, presupuesto de 20 por sondeo) y las tablas empaquetadas
`data/wvw-teams.json` + `data/team-names.json`. Los UUID de clan se normalizan a mayúsculas y al
formato `8-4-4-4-12` al entrar desde API/configuración o claims, antes de compararlos. El equipo
propio se resuelve buscando su id en `all_worlds` del match (ADR-010).

## 5. Algoritmo de `diff`

1. `prev` vacío o de otro match → `MatchChanged` (si aplica) y baseline: un `Claim{late:true}` por
   objetivo con `claimed_by`, usando `claimed_at` como hora.
2. Hueco mayor que el umbral → `Gap` y los claims vistos, como `late`.
3. Por objetivo: `owner` distinto → `Flip`; `claimed_by` cambia → `Unclaim(X)` + `Claim(Y)`.
4. **Un claim caducado no se cree**: la API **no** borra `claimed_by` al cambiar el color, así que
   se descarta cuando `last_flipped > claimed_at` (`core::matchup::claim_holds`). Sin esta regla el
   mapa enseñaba un clan en un campa de otro color (bug de 2026-10-31).
5. Salida en orden determinista (tiempo, luego id).

`core::alerts::Filter` decide qué merece aviso: mapas, «solo los clanes que sigo», y **tipos de
estructura** (`Structures`: campamento, torre, fortaleza, castillo). Ruinas, mercenarios y el resto
del decorado nunca avisan (RF-19e). El pop-up consume la misma lista, de una en una.

## 6. Persistencia (ADR-015, ADR-016)

**No hay historial en disco.** Lo que se escribe:

| Fichero | Dónde | Qué | Cuánto dura |
|---|---|---|---|
| `settings.json` | `<Documentos>\Guild Wars 2\addons\gw2_lurker\` | ajustes, estado de la ventana, `followed` y **`guild_names` (solo los clanes seguidos)** | para siempre |
| `temp.json` | igual | `[TAG] Nombre` de los clanes que han reclamado algo **en la sesión**, tope 500 | **se borra en cada arranque** |
| `guild-names.json` | igual | índice para buscar por subcadena; **no** se tiene en memoria | hasta que se cambie el índice |
| `tiles/*.jpg` | igual | teselas del mapa (RF-37) | caché; se puede borrar |

- La escritura de los ajustes vive en `addon::writer`: la UI le manda una copia y el hilo se queda
  con la última, como mucho dos guardados por segundo; en `unload` se hace un guardado final y `join`.
- Escritura atómica (`.json.tmp` + rename) y `settings.json.bak` con la copia anterior.
- Un `settings.json` dañado se conserva como `.corrupt-<fecha>` y el addon arranca en modo seguro.
- El índice se descarga de `gw2_lurker_guilds.zip` (con su `.md5`) la primera vez y a petición
  (`addon::guild_index`), y la búsqueda lo lee y lo suelta en su propio hilo (`addon::search`).
- Al cargar se migra una sola vez el `settings.json`/`config.json` de la carpeta antigua de Nexus;
  el viejo no se borra.

## 7. Mapa simplificado (ADR-002)

- Sin texturas: cada mapa se dibuja como un rectángulo con fondo liso.
- `coord` de `/v2/wvw/objectives` da la posición del objetivo; los límites del mapa se calculan como el mínimo/máximo de coordenadas de sus objetivos más un margen. Se proyecta a la caja de la ventana con una transformación afín (`world_to_screen`), con Y invertida.
- Forma por tipo: desde 0.17.0, el **círculo del color del equipo con el dibujo del juego dentro**
  (tienda, torre, fortaleza con almenas, castillo con dos torres), dibujado con primitivas en
  `ui::icons`; el tamaño también cambia por tipo. Accesible con **forma + color**, nunca solo color
  (RF-30b). El marcador de alto valor es un anillo extra y el de clan seguido, un segundo anillo.
- Los dibujos son geometría, no assets: no puede faltar un fichero en una instalación y escalan con el
  mapa. Si en el futuro se quieren los PNG reales, se sustituye `icons::parts` y nada más.
- Opcional posterior: contorno esquemático por mapa dibujado a mano como polilínea estática.
- Mapa activo: el de Mumble por defecto; selector manual.

## 7b. Widget y ventana con pestañas (ADR-006, ADR-014)

Una sola ventana con pestañas: **Avisos · Mapa · Clanes · Registro · Ajustes** (esta última al final).
El widget de alertas es la misma ventana en su pestaña de avisos, sin decoración cuando está en
click-through.

- Estado por ventana en `settings.json` (`windows.<clave>`): `visible`, `collapsed`, `locked`,
  `click_through`, `opacity`, `position`, `size`.
- **Bloqueada**: no se mueve ni se redimensiona, pero acepta clics. **Click-through**: `NO_INPUTS`,
  los clics van al juego; se sale con clic derecho sobre la caja del widget o con el keybind
  `GW2LURKER_CLICK_THROUGH` (el texto muestra el atajo real de `Nexus/InputBinds.json`).
- El render calcula la antigüedad de cada línea con `now - at`; no hay timers.
- El pop-up es una ventana aparte, encima, que enseña una alerta cada vez durante `popup_secs`.
- **Ojo con imgui:** `get_window_draw_list()` es singleton por frame (pedirlo dos veces hace
  `panic`); se pide una vez y se pasa por referencia (`ui/map.rs`).

## 7c. Idioma y contrato de ventana (RF-44, RF-45, RF-46)

**Idioma**
- `crates/core/src/i18n/`: tabla `&[(&str, [&str; 4])]` indexada por el texto original en inglés,
  columnas `[es, en, fr, de]`. `t()`, `w()` (añade `###<clave>` para que el id de imgui no cambie
  con el idioma) y `f()` con `{0}`, `{1}`.
- La tabla vive en `core` (pura) para que el test de completitud corra en Linux; `addon::i18n`
  decide el idioma y aplica el override de Ctrl derecho.
- **Toda la tabla se queda en ASCII + latino-1** (test `rf44_the_table_stays_inside_latin_1`): el
  atlas de fuentes de Nexus no tiene más glifos y una flecha Unicode se dibuja como «?».
- Detección: preferencia del usuario (si no es *Auto*) → idioma del juego por **Unofficial Extras**
  (`EV_UNOFFICIAL_EXTRAS_LANGUAGE_CHANGED`, que también llega al inicializar) → idioma de **Nexus**
  (`Nexus/Settings.json` + `Locales/<id>_Main.json`) → idioma de Windows → inglés. Nexus no expone
  el idioma por su API.
- Se registran las claves con `nexus::localization::set_translation`.
- Nombres de la API: `?lang=<código>` y caché **por idioma** en memoria.

**Ctrl derecho (RF-45)**
- `crates/addon/src/ffi/keys.rs`: `GetAsyncKeyState(VK_RCONTROL)` con `// SAFETY`. `user32` entra por
  un `extern "system"` con `#[link]`, sin el crate `windows`.
- Solo se aplica si el proceso en primer plano es el nuestro, y se evalúa cada frame: afecta al
  render, no a lo persistido.

**Contrato de ventana (RF-46)**
- `crates/addon/src/ui/window.rs`: `WindowState` por ventana y una función única que dibuja el
  cromo: barra propia, `×` para ocultar, doble clic para plegar, redimensionar, opacidad sobre el
  fondo y menú contextual (clic derecho) con *Bloquear*, *Opacidad*, *Restablecer* y *Click-through*.
- La posición se impone desde config y solo se relee del ratón cuando la ventana no está bloqueada.
  El plegado no usa el `collapsed` de imgui (con `title_bar(false)` no se podría volver a abrir):
  lo controla `WindowState::collapsed`.
- Las ventanas escriben en `runtime()`; `persist_if_changed()` guarda como mucho dos veces por
  segundo. **Cuidado**: quien recibe `&mut Config` ya tiene el candado `RUNTIME`; llamar a
  `crate::runtime()` desde ahí cuelga el juego (pasó dos veces, 2026-10-31 y 2026-11-01).

**Persistencia (RF-41, alineado con los proyectos vecinos)**
- `settings.json` en **`<Documentos>\Guild Wars 2\addons\gw2_lurker\`**. La carpeta se resuelve con
  `SHGetKnownFolderPath(FOLDERID_Documents)` (`addon::ffi::paths`), que respeta Documentos
  redirigido a OneDrive. Nunca junto a la DLL: `addons` está en `Program Files`.
- Escritura atómica (`.json.tmp` + rename), copia anterior en `settings.json.bak` y dañado en
  `settings.json.corrupt-<fecha>`.
- La escritura vive en su hilo (`addon::writer`): la UI manda una copia por canal y se queda la
  última; en `unload` hay guardado final y `join`.
- Los mismos ajustes aparecen en el panel de opciones de Nexus (`RenderType::OptionsRender`), que
  funciona fuera de WvW.

## 8. Vista Histórico y ruta inferida (ADR-005)

- Entrada: el historial **en memoria** del ciclo de seguimiento (`Dashboard::guild_history`), filtrado por guild.
- Salida: nodos `(n, objetivo, hora, late)` y tramos; si dos nodos están en mapas distintos se
  escribe «salto a <mapa>».
- Dibujo: polilínea con flecha de dirección, nodos numerados y el más reciente resaltado.
- Banner fijo: «Ruta inferida a partir de los claims; no es el recorrido real.» Además, con un solo
  claim se dibuja el punto y una flecha con la dirección (`heading`) para no dejarlo vacío.

## 9. Concurrencia y errores

- Hilos: `worker` (sondeo), `writer` (ajustes), `search` (búsqueda de clanes), `names` (añadir al
  índice), `guild_index` (descarga del ZIP) y `tiles` (teselas del mapa). Todos mueren con el addon.
- `Dashboard` es un `Mutex` que el render copia en bloque y suelta enseguida.
- Errores de red → `Health::Degraded{since, error}`; backoff exponencial con tope.
- `catch_unwind` en la frontera FFI: un pánico dibujando una ventana se registra y esa parte deja de
  dibujarse, en vez de cerrar el juego.

## 10. Estrategia de pruebas

| Capa | Técnica |
|---|---|
| `core` (diff, filtros, rutas, proyección, i18n, equipos) | Tests unitarios con fixtures reales y valores sacados de los datos |
| `api` | Fixtures JSON reales en `crates/api/tests/fixtures/` + transporte simulado |
| `addon` (puro) | Reglas extraídas a funciones sin Nexus (`team::resolve`, `window::opacity_percent`, `names::search_in`, `md5::hex`, `guild_index::unpack`…) |
| Red | Tres pruebas `#[ignore]`: `live_get_returns_the_match_json`, `live_pipeline_builds_the_dashboard` y `live_the_index_zip_is_downloadable`. Se lanzan a mano antes de publicar |
| Juego | Checklist manual en `manual-tests.md` |

## 10b. Empaquetado, actualización y modo seguro

- `export!` con `provider: UpdateProvider::Direct`, `update_link` a
  `https://raw.githubusercontent.com/cbart-cmd/gw2addonsreleases/main/gw2_lurker.dll` y
  `flags: DisableHotloading` (ADR-007).
- `scripts/release.ps1` compila, verifica `MZ`/tamaño/MD5, publica la release (retención: actual +
  anterior), y **descarga la URL publicada** para comparar bytes y MD5. `install.ps1`/`install.bat`
  copian a `<juego>\addons` y se niegan con GW2 abierto.
- `scripts/pack-guild-index.ps1` empaqueta `data/guild-names.json` en `gw2_lurker_guilds.zip` +
  `.md5` y lo publica en el mismo repositorio. El addon lo baja la primera vez y a petición.
- Al cargar se guarda la huella de la DLL; si cambia en disco se avisa de que hay que reiniciar.
- Modo seguro: `settings.json` ilegible → copia `.corrupt-<fecha>`, valores por defecto y aviso.

## 11. Riesgos y preguntas abiertas

| ID | Riesgo | Mitigación |
|---|---|---|
| R-1 | API de Nexus/nexus-rs distinta a lo supuesto | Resuelto: verificado en su fuente y en el juego |
| R-2 | Caché real de la API mayor que el intervalo | Medido: 5 s es viable pidiendo **una** partida por id |
| R-3 | API atascada o congelada durante minutos | Salud + `gap` + huella (`digest`) + aviso naranja |
| R-4 | Ids de mapa WvW que cambian | Se derivan de la respuesta del match, con respaldo |
| R-5 | Claims perdidos entre sondeos | Documentado (RNF-05); `claimed_at` lo mitiga |
| R-6 | Mumble no da un `map_id` fiable en WvW | Se detecta por `map_type`; el `shard_id` medido no sirve para el equipo |
| R-7 | Nombre de guild ambiguo o inexistente | Índice propio por subcadena + API; los nombres se guardan al resolverlos |
| R-8 | Widget que interfiere con el input | Dos interruptores independentes (bloquear / click-through) + salida por clic derecho |
| R-10 | Presupuesto de tamaño | Medido: 2,08 MB con todo (límite 2,5 MB, ADR-013) |
| R-11 | Un glifo que la fuente no tiene se dibuja como «?» | La tabla de textos se queda en latino-1, con test |
