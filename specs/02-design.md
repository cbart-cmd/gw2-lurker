# 02 – Diseño

## 1. Estructura del workspace

```
wvw-watch/
wvw-watch/                    ← raíz del repo
├── Cargo.toml                # workspace (se crea en el scaffold)
├── Cargo.lock                # versionado
├── rustfmt.toml              # raíz: rustfmt lo busca aquí
├── deny.toml                 # raíz: cargo-deny lo busca aquí
├── scripts/
│   ├── check_loc.sh
│   └── check_size.sh
├── .github/workflows/        # CI (se crea en el scaffold)
├── crates/                   # core, api, store, addon
├── docs/                     # documentación de usuario final
├── specs/                    # solo documentación de diseño: requisitos, diseño, ADR…
├── README.md
└── CHANGELOG.md
```

`core` y `store` se testean sin el juego. `api` se testea con un cliente simulado. `addon` es fino.

## 2. Integración con Nexus (`zerthox/nexus-rs`)

Capacidades de la librería que se usarán, según su documentación: macro `export!` (nombre, firma, `load`/`unload`), `alert::send_alert`, `data_link` (MumbleLink), `paths` (directorio de addon), y la integración con `imgui`.

Pendiente de confirmar leyendo el código en la tarea **T-01** (marcado como riesgo): firma exacta para registrar el callback de render, cómo obtener `map_id` desde Mumble, e iconos de Quick Access.

## 3. Flujo de datos

```
[Mumble map_id] -> context -> (activa/desactiva worker)
worker (hilo): GET match -> Snapshot -> diff(prev,cur) -> Vec<Event>
   -> store.append(events de guilds seguidas)        (hilo worker)
   -> rules(events) -> Vec<Alert>
   -> canal mpsc / Arc<RwLock<UiState>> -> UI (render thread, solo lectura)
```

- `UiState` contiene: último `Snapshot`, `fetched_at`, estado de salud, alertas recientes, cache de nombres de guild.
- El hilo de render **solo** toma un lock de lectura breve y clona/lee referencias; nunca hace E/S.

## 4. Modelo de datos (núcleo)

```rust
enum Team { Red, Blue, Green, Neutral }

struct Objective {
    id: String,              // "95-36"
    map_id: u32,
    kind: ObjKind,           // Camp, Tower, Keep, Castle, Ruins, ...
    owner: Team,
    last_flipped: DateTime<Utc>,
    claimed_by: Option<GuildId>,
    claimed_at: Option<DateTime<Utc>>,
    yaks_delivered: Option<u32>,
    guild_upgrades: Vec<u32>,
}

struct Snapshot { match_id: String, taken_at: DateTime<Utc>, objectives: HashMap<String, Objective> }

enum Event {
    Flip   { obj: ObjRef, from: Team, to: Team, at: DateTime<Utc> },
    Claim  { obj: ObjRef, guild: GuildId, at: DateTime<Utc>, late: bool },
    Unclaim{ obj: ObjRef, guild: GuildId, detected_at: DateTime<Utc> },
    Gap    { from: DateTime<Utc>, to: DateTime<Utc> },
    MatchChanged { old: String, new: String },
}
```

Datos estáticos (cacheados en disco, refresco al cambiar de match o manual): `/v2/wvw/objectives?ids=all` (nombre, tipo, `map_id`, `coord`), `/v2/wvw/guilds/:region` (guild → equipo, ADR-010) y la tabla empaquetada `data/wvw-teams.json` (id de equipo → nombre, ADR-010). El equipo propio se resuelve buscando su id en `all_worlds` del match. Nombres de guild bajo demanda con `/v2/guild/:id`, cacheados. El calendario de reasignación se lee de `/v2/wvw/timers/teamAssignment` y `/v2/wvw/timers/lockout`.

> Nota: `/v2/worlds?ids=all` ya **no** es fuente de identidad. Los nombres de equipo no existen en la API oficial: solo ids.

## 5. Algoritmo de `diff`

1. Si `prev` es `None` o `prev.match_id != cur.match_id` → emitir `MatchChanged` (si aplica) y tratar `cur` como baseline: solo generar `Claim{late:true}` para guilds seguidas (RF-25).
2. Si `cur.taken_at - prev.taken_at > umbral_gap` → emitir `Gap`; para objetivos con claim vigente de guild seguida cuyo `claimed_at` > `prev.taken_at`, emitir `Claim{late:true}`.
3. Para cada objetivo presente en ambos:
   - `owner` distinto → `Flip`.
   - `claimed_by` pasa de `None/X` a `Y` → `Unclaim(X)` (si X) y `Claim(Y)` con `at = claimed_at`.
   - `claimed_by` pasa a `None` → `Unclaim`.
4. Orden determinista de salida (por tiempo, luego id) para facilitar tests.

`rules` filtra por equipo propio, ámbito (mapa actual / todos), guilds seguidas, y aplica el cooldown por objetivo.

## 6. Persistencia (ADR-003)

JSON-lines, un fichero por mes: `history/2026-10.jsonl`. Una línea por evento:

```json
{"v":1,"t":"claim","match":"2-5","obj":"95-36","map":95,"guild":"B27D...","at":"2026-10-31T12:17:34Z","late":false}
```

- `trait EventStore { append, query(guild, range), prune(policy) }`.
- Escritura *append-only* con `flush` por lote; lectura al abrir la vista Histórico (en hilo worker) con índice en memoria por guild.
- Línea corrupta → se omite y se registra; nunca bloquea la carga.
- Campo `v` para migraciones.
- Retención: `prune` elimina ficheros mensuales antiguos y recorta por tamaño.
- Configuración aparte: `config.json` (mundo, ajustes, guilds seguidas).

### 6b. Lo que sí se guarda hoy (ADR-015, ADR-016)

- **`settings.json`**: ajustes, ventanas, `followed` y **`guild_names`** (`id → "[TAG] Nombre"`).
  Es la fuente de todos los nombres que se dibujan: un nombre resuelto no se vuelve a pedir y se
  ve también fuera de WvW (Bastión del Armisticio).
- **`guild-names.json`**: solo el índice de búsqueda por texto (~9 900 nombres). **No se tiene en
  memoria**: lo lee el hilo del buscador, lo filtra y lo suelta. Se distribuye como
  `gw2_lurker_guilds.zip` + `.md5` en el repositorio compartido y el addon lo descarga la primera
  vez o a petición (`crate::guild_index`).
- **Nada de historial en disco** (ADR-015): vive en memoria mientras el addon está cargado.
- **Mapa**: las teselas se cachean en `tiles/` (`crate::catalog::ensure_tiles`).

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

## 7b. Widget fijo de alertas (ADR-006)

- Ventana imgui sin decoración principal, anclada a una posición guardada en config (`x`, `y`, `w`, `h`, `locked`, `opacity`, `font_scale`, `max_lines`, `ttl_secs`).
- **Desbloqueado:** cabecera mínima para arrastrar/redimensionar, botón de bloqueo y botón para abrir el log completo.
- **Bloqueado:** flags de no-interacción (`NO_INPUTS`, sin foco, sin barra de título, sin guardado de ini) para que no capture ratón ni teclado.
- Estado de UI: `VecDeque<AlertEntry { id, kind, team, obj, text, map, at, sticky }>` acotado; el render calcula antigüedad y *fade* con `now - at` (sin hilos ni timers).
- Fuente de datos: las alertas generadas por `rules` se empujan a `UiState` por el canal del worker; el widget y el log leen la misma lista.
- Pendiente de confirmar en T-01: flags exactos de imgui disponibles en `nexus-rs` y cómo se persiste la posición.

## 7c. Idioma y contrato de ventana (RF-44, RF-45, RF-46)

Patrón adoptado tras revisar `../fight_analysis_danger` y `../gw2-rotation-builds`.

**Idioma**
- `crates/core/src/i18n.rs`: tabla `TEXT: &[(&str, [&str; 4])]` indexada por el texto original en inglés, columnas `[es, en, fr, de]`. Funciones `t(&str) -> &str`, `w(&str) -> String` (añade `###<clave>` para que el id de imgui sea estable al cambiar de idioma) y `f(&str, args)` con `{0}`, `{1}`.
  - *Implementación:* la tabla vive en `core` (no en `addon`) porque es pura y así el test de completitud corre en el job de Linux, que no compila el addon. `addon::i18n` queda como la capa que decide el idioma y añade el override de Ctrl derecho.
- Detección, en este orden (actualizado 2026-10-31 tras medirlo): preferencia del usuario (si no es *Auto*) → idioma del juego por **Unofficial Extras** (`nexus::event::extras::LANGUAGE_CHANGED`, que Extras emite también justo al inicializar con el idioma actual) → RTAPI si está instalado → idioma de **Nexus**, leído de su propio `Settings.json` + `Locales/<id>_Main.json` → idioma de Windows → inglés.
  - Nexus **no expone** el idioma por su API (comprobado `NexusLink` y todos los campos de `AddonApi`). Lo guarda como nombre visible (`"Language": "Español"`) en `<juego>/Nexus/Settings.json` y carga el `Locales/<id>_Main.json` correspondiente, que lleva `"Identifier": "es"`.
  - Extras está instalado en la máquina de referencia y su callback *"will also be called directly after initialization, with the current language, to get the startup language"*, así que no hace falta sondeo ni fichero para el idioma real del juego.
  - Coste medido: la feature `extras` de `nexus-rs` suma **15 872 bytes** a la DLL (1 256 448 → 1 272 320).
- Se registran las claves propias con `nexus::localization::set_translation` para que Nexus traduzca lo suyo.
- Nombres de la API: `?lang=<código>` y caché en disco **por idioma**, para que el override no necesite red.

**Ctrl derecho (RF-45)**
- `crates/addon/src/ffi/keys.rs`: `right_ctrl_down() -> bool` con `GetAsyncKeyState(VK_RCONTROL)` y `// SAFETY`. Es el único punto `unsafe`; el resto del addon solo ve un `bool`.
  - *Implementación:* `user32` entra por un bloque `extern "system"` con `#[link(name = "user32")]`, no por el crate `windows`: evita una dependencia nueva y la única `unsafe` sigue siendo la llamada. `GameLanguage` no distingue izquierda/derecha (imgui solo expone `io.key_ctrl`), así que la tecla se lee de Win32.
- Se aplica si el proceso en primer plano es el nuestro (`GetForegroundWindow` + `GetWindowThreadProcessId` frente a `std::process::id()`, igual que en `../fight_analysis_danger`), para no reaccionar mientras se escribe en otra aplicación.
- Se evalúa **cada frame** y afecta al render, no a lo persistido.

**Contrato de ventana (RF-46)**- `crates/addon/src/ui/window.rs`: un `WindowConfig { visible,
position, size, opacity, locked, collapsed }` para **la única ventana con pestañas** (ADR-014) y
una función única que dibuja el cromo: barra de título propia (`.title_bar(false)`), `×` para
ocultar, doble clic para plegar, `resizable(true)`, `.bg_alpha(opacity)`, y menú contextual con
clic derecho (Click-through / Opacidad / Restablecer).
- La posición se impone siempre desde config (la arrastra la barra propia) y solo se relee del ratón cuando la ventana **no** está bloqueada y no se está arrastrando; si no, el guardado pisa el movimiento del frame.
  - *Implementación:* el plegado **no** usa el `collapsed` de imgui (con `title_bar(false)` una ventana plegada no se podría ni volver a abrir); lo controla `WindowState::collapsed` y, plegada, la ventana se autoajusta con `always_auto_resize`.
  - *Implementación:* las ventanas escriben en una copia mutable de la config (`lib::runtime()`, `OnceLock<Mutex<Config>>`) y el fichero se guarda en `unload`, porque el hilo de render no puede hacer E/S (regla 6). El guardado diferido con *debounce* llega con el worker de T-11c.
- Si la posición guardada cae fuera de pantalla, se corrige al cargar.

**Persistencia (RF-41, alineado con los proyectos vecinos)**
- Fichero `settings.json` en **`<Documentos>\Guild Wars 2\addons\gw2_lurker\`**, como `timers_wvw` y `gw2_builds_wvw`. La carpeta se resuelve con `SHGetKnownFolderPath(FOLDERID_Documents)` (`addon::ffi::paths`), que respeta Documentos redirigido a OneDrive; leer el registro daría la ruta equivocada. Nunca junto a la DLL: `addons` está en `Program Files` y escribir allí exige elevación. Se mantiene `settings.json.bak` con la copia anterior y la escritura es atómica (`.json.tmp` + rename).
- Al arrancar se migra una sola vez el `settings.json`/`config.json` de la carpeta antigua de Nexus; el viejo **no se borra** (puede no ser borrable sin administrador). El histórico de sesión no se migra: se descarta al cerrar (ADR-012).
- Un fichero dañado se conserva como `settings.json.corrupt-<fecha>`.
- La escritura vive en un hilo propio (`addon::writer`): la UI solo envía una copia por un canal y el hilo se queda con el último valor, como mucho dos guardados por segundo. Así se cumple la regla 6 (nada de E/S en render) y no se pierde lo movido si el juego se cierra de golpe. En `unload` se hace un guardado final y se espera (`join`) al hilo.
- Los mismos ajustes aparecen también en el panel de opciones de Nexus (`RenderType::OptionsRender`), que es accesible fuera de WvW (RF-02).

## 8. Vista Histórico y ruta simulada (ADR-005)

- Entrada: `history.query(guild, range)` → claims ordenados por `at`.
- Salida: lista de nodos `(n, obj, at, late)`, y tramos entre nodos consecutivos del mismo mapa; si el tiempo entre nodos supera el umbral de `gap`, el tramo es discontinuo.
- Dibujo: polilínea con flechas en el punto medio, nodos numerados, color por guild si hay varias.
- Banner fijo: "Ruta inferida a partir de claims; no es el recorrido real."

## 9. Concurrencia y errores

- Un hilo worker con bucle de espera que se puede interrumpir (parar al salir de WvW o al descargar el addon; `unload` hace `join`).
- Errores de red → `Health::Degraded{since, last_error}`; backoff 2^n con tope (por defecto 5 min).
- `catch_unwind` en los puntos de entrada FFI y en el worker.

## 10. Estrategia de pruebas

| Capa | Técnica |
|---|---|
| `diff`, `rules`, `history` | Tests unitarios con snapshots sintéticos + tests de propiedades (p. ej. diff(s, s) = ∅) |
| `api` | Fixtures JSON reales guardadas + cliente simulado (éxito, 503, JSON truncado) |
| `store` | Tests con directorio temporal: append, consulta, línea corrupta, rotación |
| `map` proyección | Tests de `world_to_screen` (inversa, bordes) |
| `addon` | Checklist manual en el juego (`tasks.md`, fase de aceptación) |

## 10b. Empaquetado, actualización y modo seguro

- `export!` con `provider: GitHub`, `update_link` al repositorio y `flags: DisableHotloading` (ADR-007). Nexus hace la descarga; el addon solo avisa del reinicio.
- Módulo `addon::update`: guarda SHA-256 y versión de su DLL al cargar y vigila el fichero cada 60 s en el hilo worker.
- Módulo `addon::safe_mode`: carga de config/historial con degradación (`*.corrupt-<fecha>`), estado `Health::SafeMode{reason}` visible en UI.
- Perfil release y presupuesto de tamaño: ADR-008 y `04-engineering-standards.md` §5.

## 11. Riesgos y preguntas abiertas

| ID | Riesgo | Mitigación |
|---|---|---|
| R-1 | API de Nexus/nexus-rs distinta a lo supuesto | Spike T-01 antes de comprometer UI |
| R-2 | Caché real de la API de matches mayor que el intervalo | Medir en T-02; fijar el intervalo por defecto con datos |
| R-3 | API atascada/503 durante largos periodos | Salud + `gap` + marca "obsoleto" |
| R-4 | IDs de mapa WvW cambian | Derivarlos de la respuesta del match, con lista de respaldo |
| R-5 | Claims perdidos entre polls | Documentado (RNF-05); `claimed_at` mitiga parcialmente |
| R-6 | Mumble no da `map_id` fiable en WvW | Alternativa: detectar por tipo de mapa de `/v2/maps`; verificar en T-01 |
| R-7 | Unicidad de nombre de guild en búsqueda | UI de selección de candidato (RF-20) |
| R-9 | Nexus no deja la DLL nueva donde se esperaba (aviso de reinicio inviable) | Spike T-25; alternativa: avisar solo con la versión que reporte Nexus |
| R-10 | Presupuesto de tamaño irreal por TLS/HTTP | Spike T-02b antes de fijar cifras |
| R-8 | Widget bloqueado que interfiere con el input del juego | Verificar flags en T-01 y checklist T-19 |
