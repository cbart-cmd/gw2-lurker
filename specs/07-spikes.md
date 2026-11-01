# 07 – Spikes: guía y resultados

Un spike es un experimento corto y desechable para convertir una suposición en un hecho. **Los ejecuta una persona** (necesitan el juego o la red real). La IA solo debe confiar en lo que aparezca en la sección *Resultados*.

Regla: si un resultado contradice la spec, no se "arregla" el código; se abre un ADR.

---

## T-01 – Addon mínimo en Nexus (≈ 2–3 h)

**Objetivo:** confirmar la API real de `nexus-rs` y qué se puede hacer dentro del juego.

**Preparación**
1. Instala Nexus en una copia de GW2 y comprueba que abre con `Ctrl+O`.
2. Clona `https://github.com/zerthox/nexus-rs` y lee su README y los ejemplos que incluya.
3. Crea un crate `cdylib` aparte (no en este repo) que use `nexus::export!` con un nombre y una firma negativa aleatoria.

**Qué debe hacer el addon de prueba**
- Dibujar una ventana imgui con el `map_id` actual leído de MumbleLink, el tipo de mapa y el nombre del personaje si está disponible.
- Enviar una alerta con `send_alert` al pulsar un botón.
- Registrar un keybind gestionado por Nexus (`Alt+Shift+L`) que cambie un contador visible.
- Probar una segunda ventana con los flags de imgui para **no capturar ratón/teclado** (click-through) y comprobar si los clics llegan al juego.
- Guardar y leer un fichero en el directorio de addon (`paths`).

**Qué anotar (rellena Resultados):**
- Versión de Nexus, de `nexus-rs` y de Rust usadas.
- Firma exacta para registrar el callback de render y el código que funciona.
- `map_id` en cada mapa de WvW: Ojos de Aguas Eternas, Frontera Verde, Azul, Roja, y también el de Bastión del Armisticio y una ciudad (para excluirlos).
- ¿Mumble da `world_id` en WvW? ¿Coincide con tu mundo? Compáralo con el id de tu equipo en el match (worlds/all_worlds), no solo con el nombre.
- Nombre que da /v2/worlds para tu id frente al nombre de equipo que ves en el juego (panel de WvW). Anota ambos.
- ¿El cargador de texturas de Nexus admite imágenes desde disco? ¿Y desde URL?
- ¿/v2/maps devuelve continent_rect para los mapas de WvW? Guarda la respuesta
- Flags de imgui que logran el click-through y si hay que reaplicarlos cada frame.
- ¿El keybind es reasignable desde Nexus?
- Tamaño de la DLL release del addon mínimo (`ls -l`).
- Dónde escribe Nexus sus logs y cómo ver los del addon.

**Hecho cuando:** hay un addon que carga, dibuja, avisa, responde al atajo y el click-through está confirmado o descartado.

---

## T-02 – Frecuencia real de la API (≈ 40 min, mayormente esperando)

**Objetivo:** saber cada cuánto cambian de verdad los datos y qué caché declara la API.

1. Averigua el ID de tu mundo (`https://api.guildwars2.com/v2/worlds?ids=all&lang=es`).
2. Ejecuta, **en horario de actividad de WvW**:
   - Windows: `.\scripts\poll_match.ps1 -World <ID> -IntervalSec 10 -DurationMin 30`
   - Linux/macOS/Git Bash: `./scripts/poll_match.sh <ID> 10 30`
3. Se genera `spike-logs/poll_<fecha>.csv` y los cuerpos distintos en `spike-logs/bodies/`.
4. Con `./scripts/capture_fixtures.sh` (o `.ps1`) guarda además las fixtures estáticas. Repítelo 5–10 min después para tener dos snapshots del mismo match.

**Qué anotar:** intervalo mínimo entre cambios consecutivos, mediana, cabecera `Cache-Control`/`Age`, porcentaje de respuestas no-200 y duración de los huecos.

**Hecho cuando:** hay un intervalo de polling por defecto basado en datos (sustituye el provisional de 30 s).

---

## T-02b – Tamaño con distintos clientes HTTP (≈ 2 h)

**Objetivo:** fijar el presupuesto real de la DLL (ADR-008).

1. Usa el addon mínimo de T-01 con el perfil release de `04-engineering-standards.md` §5.
2. Añade una petición HTTPS simple con cada opción y anota el tamaño de la DLL: (a) `ureq` con `rustls`, (b) `ureq` con TLS nativo del sistema, (c) otra alternativa ligera que encuentres.
3. Verifica que cada una funciona dentro del juego (la petición a la API responde).

**Hecho cuando:** hay una tabla opción → tamaño → funciona sí/no, y una elección razonada.

---

## T-25 – Cómo actualiza Nexus un addon (≈ 1–2 h)

**Objetivo:** saber qué hace Nexus con la DLL nueva (ADR-007).

1. Publica dos versiones de prueba del addon mínimo (`0.0.1`, `0.0.2`) en un repo de GitHub con releases, usando `UpdateProvider::GitHub`.
2. Instala la `0.0.1`, abre el juego, anota el listado de la carpeta `addons/` (nombres, tamaños, fechas).
3. Con el juego abierto, fuerza la actualización desde Nexus y vuelve a listar la carpeta.
4. Cierra el juego, ábrelo de nuevo y comprueba qué versión carga.

**Qué anotar:** ¿Nexus sustituye el fichero, crea uno temporal o espera al reinicio? ¿Qué mensaje enseña Nexus? ¿Se puede detectar el cambio desde el addon (fichero nuevo, hash, fecha)?

**Hecho cuando:** ADR-007 pasa de "Propuesto" a "Aceptado" o se sustituye por otro.

---

## Resultados (se rellenan a mano)

**Leyenda de procedencia.** Cada dato lleva su origen para que nadie confunda una deducción con una medición:

- `[API]` – comprobado contra la API pública de GW2 el **2026-10-31** (respuesta real guardada en la sesión).
- `[src]` – comprobado leyendo el código fuente citado; **no** es una prueba en el juego.
- `pendiente` – exige ejecutarlo dentro de GW2. No se rellena por deducción.

### T-01

- Fecha / versiones: revisión `[src]`/`[API]` 2026-10-31. `nexus-rs` **0.12.0**, `edition = "2024"`, `rust-version = "1.87.0"` (`nexus/Cargo.toml`); ImGui vía `arcdps-imgui` **0.8.0**. Versión de Nexus instalada y de Rust local: `pendiente`.
- Firma de render verificada `[src]`:
  ```rust
  register_render(RenderType::Render, render!(|ui| { /* &imgui::Ui */ })).revert_on_unload();
  ```
  `render!` genera un `extern "C-unwind" fn()` sin parámetros que obtiene el `Ui` del frame (`nexus/src/api/gui.rs`). `RenderType` = `PreRender` | `Render` | `PostRender` | `OptionsRender`. Ejemplo completo en `nexus_example_addon/src/lib.rs`.
- `send_alert` `[src]`: `nexus::alert::send_alert(impl AsRef<str>)`, devuelve `()` y no es revertible (`nexus/src/api/alert.rs`).
- Export `[src]`: `nexus::export! { name, signature: -0x…, load, unload, flags: AddonFlags::None, provider: UpdateProvider::GitHub, update_link, log_filter }`; `signature` debe ser un entero **negativo** aleatorio si el addon no está en Raidcore (`nexus/src/lib.rs`, `SupportedFields`).
- `map_id` WvW `[API]` (`/v2/maps?ids=all`, 1086 mapas): EBG=**38** (`Center`), Verde=**95** (`Alpine Borderlands`, `GreenHome`), Azul=**96** (`Alpine Borderlands`, `BlueHome`), Roja=**1099** (`Desert Borderlands`, `RedHome`), Armisticio=**1315**, ciudad=**50** (`Lion's Arch`). Además EotM=**968** y Obsidian Sanctum=**899**.
  - Coincide exactamente con las constantes de `gw2-mumble-rs` (`src/map_id/wvw.rs`: 38, 95, 96, 899, 968, 1099), lo que valida ambos orígenes.
  - **Matiz importante:** 95/96/1099 son el mapa "home" de cada **color**, no de un servidor. Qué servidor está en Verde/Azul/Rojo depende del emparejamiento, así que no se pueden cablear como "mi frontera".
  - Armisticio tiene **0 objetivos** en `/v2/wvw/objectives` y `map_type` = **18** (`WVW_LOUNGE`, `gw2-mumble-rs` `src/map_type.rs`), distinto de los 9–12/15 de los mapas jugables: sirve para excluirlo.
- Mumble `world_id` fiable: **no** `[src]`. `Identity::world_id` está documentado como *"Shard id from [`Context`]. Formerly character homeworld per API:2/worlds. **Not usable since the switch to the megaserver system**"* (`gw2-mumble-rs` `src/identity.rs`). Motivo real de ADR-004 (selección manual de mundo). Confirmar igualmente en el juego.
  - Sí hay datos fiables para el contexto: `Context::map_id`, `Context::map_type`, `Context::shard_id`, `Context::instance`, `UiState::IS_IN_COMPETITIVE_MODE`, `UiState::GAME_HAS_FOCUS` (`gw2-mumble-rs` `src/context.rs`). El `map_type` es la vía limpia para RF-01 ("¿estoy en WvW?").
- Click-through: `pendiente`. `WindowFlags::NO_INPUTS` y `NO_MOUSE_INPUTS` **existen** en `arcdps-imgui` 0.8.0 (docs.rs), pero si el clic atraviesa hasta el juego o queda comido por la capa de Nexus solo se sabe probándolo.
- Keybind reasignable: muy probable `[src]`, pendiente de confirmar en la UI. Nexus persiste los binds de usuario en `<juego>/addons/Nexus/InputBinds.json` (`RaidcoreGG/Nexus`, `Nexus/Index/Index.cpp`, ruta `EPath::InputBinds`), lo que implica que el usuario los sobrescribe desde la configuración de Nexus. El registro desde el addon es `register_keybind_with_string(id, keybind_handler!(|id, is_release| …), "ALT+SHIFT+L")` (`nexus/src/api/keybind.rs`).
- Tamaño DLL mínima: `pendiente` (lo produce T-02b).
- Logs `[src]`: `<juego>/addons/Nexus/Nexus.log`, más `Crash.log` y `CrashStack.log` en la misma carpeta (`RaidcoreGG/Nexus`, `Nexus/Index/Index.cpp`). El log propio del addon se escribe ahí y se filtra con `log_filter` (feature `log`).
- Rutas de datos del addon `[src]`: `paths::get_addon_dir("gw2_lurker") -> io::Result<PathBuf>` (`nexus/src/api/paths.rs`), es decir `<juego>/addons/gw2_lurker/`. El nombre del directorio coincide con el de la DLL publicada (`gw2_lurker.dll`).

### T-02

- Fecha / hora local y mundo: **2026-10-31, sábado, 11:52–12:20 CET** (horario de actividad). Dos sondeos paralelos, muestreo cada 10 s, `Invoke-WebRequest` en bucle, 227 peticiones en total, **0 errores y 0 respuestas 429**:
  - **A** – mundo **1008** (Jade Quarry) vía `/v2/wvw/matches?world=1008` ⇒ match **1-4** (región NA: los `1-x` son NA). 167 muestras / 27,7 min.
  - **B** – match **2-4** (región EU: los `2-x` son EU), elegido por ser el único que cambió al comparar los 9 emparejamientos con `/v2/wvw/matches?ids=all`. 60 muestras / 9,8 min.
  - Elección obligada: el mundo 1008 resultó ser un emparejamiento **muerto** (ver más abajo). Medir con un solo mundo habría dado un resultado falso.
- Intervalo mínimo / mediana de cambio:
  - **Cuerpo JSON completo:** match activo (B) min **9 s** / mediana **10 s** / máx **21 s** ⇒ cambia en *casi todas* las muestras de 10 s. Match inactivo (A) **1360 s (22 min 40 s) sin un solo cambio**, y después un burst de cambios cada ~10 s. El intervalo está limitado por el muestreo: a 10 s no se puede resolver nada más rápido.
  - **Propiedad de objetivos** (lo que dispara las alertas, RF-10/RF-21): activo (B) min **9 s** / mediana **19 s** / máx **110 s** (24 cambios en 9,8 min). Inactivo (A) **22 min 40 s con cero cambios de propiedad**, y luego **7 cambios en 2,5 min** (min 10 s, mediana 20 s).
  - Lectura: el JSON cambia constantemente por contadores (`deaths`, `kills`, `skirmishes`), no por objetivos. Un emparejamiento puede estar **>20 min sin actividad y luego estallar**. El hueco máximo observado (22 min 40 s) no es caché: se descartó con `Date` en vivo, con dos rutas distintas (`?world=1008` y `/matches/1-4`) devolviendo el mismo cuerpo, y con el match 2-4 cambiando en 96 s en la misma ventana.
- Cache-Control / Age `[API]`: `Cache-Control: public, max-age=1`; **no se envía cabecera `Age`**; `Expires` = `Date` + 1 s. Otras cabeceras: `X-Rate-Limit-Limit: 600` (600 peticiones/hora), `Server: Quaggans`, `X-Result-Total`, `Content-Language` negociable por `lang`.
  - Consecuencia: la API declara que el dato puede cambiar cada segundo; el límite práctico es 600 req/h ⇒ 10 s = 360/h (60 % del presupuesto, deja poco para `/v2/guild`, `/v2/worlds`, objetivos); 30 s = 120/h (20 %); 5 s = 720/h (**excede** el límite).
- % de errores y huecos máximos: **0 %** de errores en 227 peticiones; latencia medida en el sondeo A: 45–327 ms. Hueco máximo sin cambios: **1360 s** (22 min 40 s) en un emparejamiento inactivo.
- Intervalo por defecto elegido: **30 s se mantiene** (provisional → confirmado), por tres razones medidas: (a) captura la mayoría de cambios de propiedad (mediana 19–20 s); (b) consume 120 de las 600 req/h, dejando margen para el resto de endpoints; (c) bajar a 10 s aumenta ×3 el consumo para ganar como mucho ~10 s de latencia de aviso. Queda documentado en RNF-05 que un flip+recaptura dentro del mismo intervalo **se pierde**; los huecos de >20 min hacen además que "sin noticias" no signifique "sin actividad".
- **Revisado el 2026-10-31 (0.23.0):** el jugador pidió la cadencia de gw2mists (5 s), y el coste cambia por dos motivos medidos después: (a) el sondeo pasa a pedir **una partida por id** (`/v2/wvw/matches/{id}`, 44 096 B con los cuatro mapas) en vez de resolver el equipo contra `ids=all` (400 KB), así que 5 s mueve ~9 KB/s, **menos** que los 30 s anteriores; (b) la ventana del límite **no está** en la cabecera (`X-Rate-Limit-Limit: 600`), así que 720 peticiones/hora solo incumplirían si la ventana fuera la hora — extremo que hay que confirmar con una sesión larga a 5 s (si llega un `429`, el worker ya retrocede hasta 300 s). Se adopta 5 s por defecto y 5 s de mínimo; RNF-05 sigue vigente (best effort).
  - `pendiente`: validar en juego si 30 s de retraso es aceptable para un aviso de captura (criterio humano, no medible desde la API).

### T-02b
| Opción | Tamaño DLL | Funciona en juego |
|---|---|---|
| ureq + rustls | `pendiente` | `pendiente` |
| ureq + TLS nativo | `pendiente` | `pendiente` |
| otra | `pendiente` | `pendiente` |
- **Base ya medida (2026-10-31):** con `nexus` 0.12.0 (features `log`, `log_filter`, `mumble`, `rtapi`) + ImGui y **sin cliente HTTP todavía**: **1.053.696 bytes (1,00 MB)**, PE válido (`MZ`) que exporta `GetAddonDef`. Queda 1,5 MB de objetivo menos eso para el cliente HTTP y el resto del addon.
- Elección y motivo: `pendiente`. Requiere compilar el addon mínimo y medir (nada de esto se puede deducir de la API).

### T-25
- Comportamiento de Nexus ante la actualización: `pendiente` en juego; indicios `[src]`:
  - `AddonFlags::DisableHotloading` está documentado como *"Prevents unloading at runtime. Will require a restart if updated, etc."* (`nexus/src/addon.rs`) ⇒ **sin** ese flag Nexus recarga el addon en caliente; **con** él exige reiniciar. Es la palanca que decide ADR-007.
  - Nexus descarga a `<NexusDLL>.update` y conserva `<NexusDLL>.old` (`Nexus/Index/Index.cpp`, rutas `EPath::NexusDLL_Update` / `NexusDLL_Old`) ⇒ indicio de intercambio de fichero con respaldo. Falta ver si hace lo mismo con addons de terceros.
- Método de detección viable: `pendiente`. Candidatos a probar en el spike: comparar fecha/tamaño de `<juego>/addons/<nombre>/*.dll` al arrancar, y confirmar si el `.old` aparece para el addon.

### Notas de verificación: nombres de equipo traducidos y color propio (2026-10-31)

Todo comprobado con peticiones reales y con el código de los vecinos.

- **`data/equipos.xlsx`** (facilitado por el usuario) traía una hoja con cuatro columnas —Inglés, Español, Francés, Alemán— y **21 equipos**. Se convirtió a `data/team-names.json`, **indexado por nombre inglés**, no por id, porque los nombres de equipo rotan cada temporada y los ids no. Desde el 2026-11-01 el jugador **edita el JSON a mano** y la hoja y el script `merge_team_names.ps1` se retiraron (petición: «el excel lo usé para rellenar la primera vez pero ya no lo quiero más, retoco directamente el json a mano»).
- De esos 21, **12 coinciden** con los 33 equipos vivos del 2026-10-31 y **9 son de una temporada anterior** (Lyssa's Muse, Balthazar's Blight, Mursaat's Sanctuary, Jormag's Glacier, Primordus's Caldera, Kralkatorrik's Eye, Soo-Won's Haven, Zhaitan's Rest, Mordremoth's Canopy). El pool conserva las 21 para que sirvan si el nombre vuelve.
- **Ojo: son traducciones literales, no los nombres del cliente.** El Excel dice `Great House Aviary → Pajar de la Gran Casa`, pero en el juego en español se llama **Bosquemugoso** (captura del usuario, ventana de Guerra de la Niebla), y `Lutgardis Conservatory → Invernadero de Lutgardis`, que ni siquiera está en el Excel. Sirven como etiqueta mientras no tengamos los oficiales; por eso cada entrada lleva `source: "spreadsheet"` y el mapa `names` de `wvw-teams.json` (por id) tiene prioridad para poner los oficiales cuando se confirmen.
- Errata normalizada en el script: `Modremoth's Canopy → Mordremoth's Canopy` (el dragón anciano es Mordremoth).
- **gw2mists no tiene traducciones de equipo**: `https://api.gw2mists.com/worlds/alliance` (200 con cabeceras de navegador; **403** sin `Referer`/`Origin`/`User-Agent`) devuelve las 33 entradas con `{id, name, nation, population, players, guilds, alliances, commanders}` y el nombre **solo en inglés**; `?lang=es` y `?locale=es` devuelven `"Moogooloo"` igual. `https://gw2mists.com/api/...` sirve el HTML del SPA, no JSON.
- **La API oficial no traduce equipos**: `/v2/wvw/teams` y `/v2/wvw/teams?lang=es` → **404**. `/v2/wvw/objectives/38-17?lang=es` → `"Zanja de Mendon"`, o sea que la traducción existe para objetivos pero no para equipos. Los nombres localizados de equipo (`Bosquemugoso`…) viven en los textos del cliente, que no son accesibles.
- Conclusión: `data/wvw-teams.json` mantiene `names: {es, fr, de}` **curadas a mano** y `scripts/update_wvw_teams.ps1` las **preserva** al regenerar (probado con `-DryRun`: 33 equipos, mismas entradas).
- **Cómo sabe el color `../fight_analysis_danger`**: no lo lee del juego, lo saca de **arcDPS**. `evtc::agent::realtime::Agent` lleva `is_self: u32` y `team: u16`, y Nexus expone `EV_ARCDPS_SELF_JOIN` con un `AgentUpdate` que también trae `team` (se dispara en cada carga de mapa y se puede repetir con `EV_REPLAY_ARCDPS_SELF_JOIN`). Su enum tiene `Team::Own` documentado como *"identified allied team in live events without a verified color mapping"*: identifica **tu** agente, pero no afirma el color.
- Por eso en este addon el `team` de arcdps se guarda como **etiqueta opaca** (`arc_team::Calibration`), no como color: sirve para detectar que estás en un equipo distinto del guardado (otra cuenta o sorteo mensual) y para calibrar cuando el jugador elige equipo. Cero dependencias nuevas: la feature `arc` de `nexus-rs` ya trae el evento.

### Notas de verificación: de dónde saca Nexus el idioma (2026-10-31)

Comprobado `[src]` en `nexus-rs` 0.12.0, en el crate `unofficial_extras` de `arcdps-rs` y en la instalación real (`C:\Program Files\Guild Wars 2\addons\Nexus`).

- **Nexus no expone el idioma por API**: `NexusLink` sólo tiene `width/height/scaling/is_moving/is_camera_moving/is_gameplay` y las fuentes; `AddonApi` no tiene campo de idioma. Lo que sí hay es `LocalizationApi::translate/translate_to/set`, que traduce identificadores pero no dice cuál es el idioma activo.
- **Cómo lo sabe Nexus**: `Nexus/Settings.json` guarda `"Language": "Español"` (nombre visible) y Nexus carga `Nexus/Locales/<id>_Main.json`. Cada locale declara `{"Identifier": "es", "DisplayName": "Español", "Texts": {...}}`; hay 10 (br, cn, cz, de, en, es, fr, it, pl, ru).
- **Idioma real del juego**: `unofficial_extras::Language` es `repr(i32)` con `English=0, French=2, German=3, Spanish=4, Chinese=5` (mismos códigos que RTAPI). Su documentación dice del callback de idioma: *"Called whenever the language is changed. Either by Changing it in the UI or by pressing the Right Ctrl (default) key. Will also be called directly after initialization, with the current language, to get the startup language."*
- **Puente ya hecho por Nexus**: `nexus::event::extras::LANGUAGE_CHANGED` (`EV_UNOFFICIAL_EXTRAS_LANGUAGE_CHANGED`) con la feature `extras`; `subscribe(...).revert_on_unload()` deja la desuscripción a Nexus.
- Coste medido en la DLL de release: **+15 872 bytes** (1 256 448 → 1 272 320), frente a los 1,5–2 MB de un cliente TLS embebido.
- Trampa del macro: `event_consume!` sólo acepta una función (`fn(Option<&T>)`), no una closure con capturas; con una closure la plantilla acaba infiriendo `<()>` y falla el tipo.

### Notas de verificación: cliente HTTP sobre WinHTTP (2026-10-31)

Medido `[medido]` ejecutando la prueba del propio crate del addon (`cargo test -p gw2lurker-addon live_get -- --ignored`) contra `https://api.guildwars2.com/v2/wvw/matches/2-2`:

- `WinHttpOpen` + `WinHttpConnect` + `WinHttpOpenRequest` con `WINHTTP_FLAG_SECURE` + `WinHttpSendRequest`/`ReceiveResponse`/`ReadData` devuelve **HTTP 200** y el JSON del match en **270 ms**, sin añadir ninguna dependencia y sin código TLS dentro de la DLL. La DLL de release sigue en ~1,26 MB.
- Consecuencia para T-02b: el cliente HTTP **ya no es una incógnita de tamaño**. Un cliente con TLS embebido (reqwest/rustls, ureq) habría añadido del orden de 1,5–2 MB y roto el objetivo de 1,5 MB de `check_size.sh`.
- `WinHTTP` está en el sistema desde Windows 2000; a diferencia de `GetAsyncKeyState`, aquí sí importa el proxy del usuario, y `WINHTTP_ACCESS_TYPE_DEFAULT_PROXY` lo respeta.
- Lo que **no** está verificado: que funcione con el juego en primer plano y con la DLL cargada por Nexus (debería, es una API de sistema), ni el comportamiento a través de VPN/proxy corporativo.

### Notas de verificación: idioma y Ctrl derecho (2026-10-31)

Comprobado `[src]` en el checkout de `nexus-rs` etiqueta 0.12.0 y en `arcdps-imgui` 0.8.0. No cambia ningún requisito; fija detalles de implementación de RF-44/RF-45.

- `GameData::language` es `Result<GameLanguage, u32>` (`nexus/src/api/rtapi/game.rs`). `GameLanguage` deriva `IntoPrimitive`, así que `u32::from(game.language.ok()?)` da el código crudo: **0=English, 1=Korean, 2=French, 3=German, 4=Spanish, 5=Chinese**. Coreano y chino no tienen columna en la tabla ⇒ caen a inglés, igual que un valor futuro.
- `RealTimeApi::get() -> Option<Self>` y `read_game(&self) -> Option<GameData>` (`nexus/src/api/rtapi/mod.rs`); `None` cuando el addon RealTime API no está instalado. No es un error: se cae a inglés.
- `nexus::localization::set_translation(identifier, language, string)` existe y no devuelve nada (`nexus/src/api/localization.rs`); necesita `AddonApi::get()`, por lo que solo se puede llamar desde `load`.
- **No existe** `UiState::GAME_HAS_FOCUS` en `nexus-rs` (la spec lo citaba de memoria). imgui tampoco distingue Ctrl izquierdo de derecho: solo hay `io.key_ctrl`. Por eso el override de RF-45 se lee de Win32 (`GetAsyncKeyState(VK_RCONTROL)`) y el foco se comprueba con `GetForegroundWindow` + `GetWindowThreadProcessId` comparado con `std::process::id()`, el mismo método que usa `../fight_analysis_danger/src/i18n.rs`. Ambas funciones viven en `user32`, al que se enlaza con `extern "system"`; no hace falta añadir el crate `windows`.

### Notas de verificación (fuera de los spikes; afectan a `02-design.md` §5, ADR-002/ADR-011 y ADR-010)

Comprobado `[API]` el 2026-10-31 sobre los 5 mapas jugables. **No se ha cambiado la spec**; ADR-011 ya recoge el cambio.

- `coord` de `/v2/wvw/objectives` **está en coordenadas de continente**, no locales: los valores caen dentro del `continent_rect` de `/v2/maps` en los 5 mapas (EBG: x 9414–11766 dentro de 8958–12030).
- Por eso el mínimo/máximo de `coord` **no** sirve como caja del mapa: se queda corto. Ejemplo EBG: caja de objetivos x 9414–11766 vs. rectángulo real 8958–12030 (un ~20 % más estrecho). `/v2/maps` da `continent_rect` ya calculado y `map_rect` (EBG: `[[-36864,-36864],[36864,36864]]`).
- **81 de 178** objetivos de `/v2/wvw/objectives?ids=all` tienen `coord: null` (Spawn, Mercenary y parte de Ruins/Camp/Tower/Keep). Cualquier proyección debe tolerar `coord` ausente.
- `claimed_by` / `claimed_at` **siempre están presentes** en `/v2/wvw/matches` y valen **`null`** cuando el objetivo no tiene claim (no se omiten). En el match `2-2`: **41 de 91** objetivos con claim y **50 con `null`**. `claimed_by` es **una sola guild** (cadena), no una lista: la API no puede representar varios claims simultáneos. Otros campos por objetivo: `id`, `type`, `owner`, `last_flipped`, `points_tick`, `points_capture`, `yaks_delivered`, `guild_upgrades`.
- `Context::player_x/player_y/map_center_x/map_center_y` están documentados como *"Not updated in competitive modes"* (`gw2-mumble-rs` `src/context.rs`). Afecta a cualquier idea de dibujar la posición del jugador sobre el mapa: hay que confirmar en WvW antes de depender de ello.

### Notas de verificación: equipos WvW y de dónde saca los datos gw2mists (2026-10-31)

Comprobado `[API]` contra `api.guildwars2.com` y abriendo `https://gw2mists.com/en/livemap/eu` en un navegador real, capturando sus peticiones. **No se ha cambiado la spec.**

- **Los "mundos" ya no son lo que la spec asume.** `/v2/wvw/matches?ids=all` devuelve por color dos listas distintas:
  - `worlds` → un id de mundo clásico (1001–1024 NA, 2xxx EU).
  - `all_worlds` → **incluye además los ids de equipo del sistema nuevo, 11001–11015 (NA) y 12001–12018 (EU)**.
  Ejemplo real: match `2-2` → `blue: [12014, 2014]`. Un jugador hoy se identifica por **equipo** (p. ej. `12014` "Great House Aviary"), no por mundo.
- **El prefijo del id de match indica región: `1-x` = NA, `2-x` = EU** (verificado con los mundos de cada `all_worlds`).
- **`/v2/wvw/timers/teamAssignment`** y **`/v2/wvw/timers/lockout`** existen y dan las fechas de reasignación de equipos y de bloqueo (hoy: NA `2026-11-07T02:00:00Z`, EU `2026-11-06T18:00:00Z`). Evita cablear "el reset es el viernes a las 02:00 UTC".
- **Cómo obtiene gw2mists lo que muestra** (peticiones observadas en la página, sin sesión iniciada):
  - `GET https://api.guildwars2.com/v2/wvw/objectives?ids=all&lang=en` — metadatos estáticos de objetivos.
  - `GET https://api.gw2mists.com/worlds/alliance` — 33 equipos con `players`, `guilds`, `alliances` y `commanders`; **es backend propio**, agregado a partir de las claves de API que sus usuarios ceden.
  - `GET https://api.guildwars2.com/v2/wvw/timers/teamAssignment` y `.../lockout`.
  - `GET https://tiles.gw2mists.com/mist_tiles/{z}/{x}/{y}.jpg` — **imágenes de mapa reales** (por teselas), el mismo tipo de fuente que ADR-011 deja como fondo opcional desactivado por defecto (RF-37).
  - `POST https://api.gw2mists.com/monitoring`, `/community-news/`, `/blog/...` — telemetría y contenido, irrelevantes.
- **No hay ninguna petición de posición del jugador.** Una página web no puede leer MumbleLink (memoria compartida del cliente), y no aparece ningún endpoint que suba/reciba coordenadas del jugador. Lo que la web "sabe de ti" es tu **equipo** (por la cuenta vinculada o por selección manual), y de ahí deduce tu color y qué objetivos son tuyos. `pendiente`: la vista con sesión iniciada no se ha podido inspeccionar (el `POST /auth/v2/session` devolvió 403 sin login).

### Notas de verificación: qué expone la API oficial sobre equipos y qué no (2026-10-31)

Comprobado `[API]` contra el índice de endpoints (`https://api.guildwars2.com/v2`). Esto **resuelve el punto que ADR-010 deja abierto como "no verificado"**.

- **La API oficial NO expone nombres de equipo ni de alianza.** No existe `/v2/wvw/teams` ni `/v2/wvw/alliances` (**404** en ambos) y el índice de `/v2` no lista nada equivalente. Los nombres tipo "Great House Aviary" solo están en bases de datos de terceros (gw2mists). ⇒ Confirma el contexto de ADR-010: la UI debe apoyarse en `id` + alias (RF-03a, RF-03f).
- **Sí existe la pertenencia gremio → equipo:** `GET /v2/wvw/guilds/eu` y `/na` devuelven `{ "<guild_id UUID>": "<team_id>" }` (282 KB para toda EU). Es la fuente oficial para saber a qué equipo pertenece un gremio, útil en RF-20/RF-23 (seguir guilds); `/v2/guild/:id` no lo informa.
- **`/v2/account/wvw`** existe y requiere API key (`[a]` en el índice): permitiría leer el equipo de la cuenta automáticamente, pero ADR-010 descarta pedir clave. Se anota como alternativa disponible, no como decisión.
- **`/v2/wvw/matches/overview`** devuelve solo la lista de ids de match en curso (83 bytes, los 9 actuales): endpoint barato para detectar el cambio de match sin descargar el JSON completo.
- **Orden de región:** `1-x` = NA y `2-x` = EU (deducido de los mundos de cada `all_worlds`; no viene indicado en el JSON).

### Notas de verificación: `claimed_by` es un id de gremio usable y `/v2/guild/:id` lo resuelve (2026-10-31)
Comprobado `[API]` con datos en vivo (`/v2/wvw/matches?ids=all`, 403 622 bytes, y dos gremios reales).

- **`claimed_by` llega como UUID con guiones y en mayúsculas**, p. ej.
  `"9FDB4A86-1588-E611-80D3-AC162DC0847D"`. Es el id que hay que enseñar **resuelto**: en crudo
  no lo reconoce nadie (era el bug de «sale un ID en vez del nombre del clan»).
- **`GET /v2/guild/<ese uuid con guiones>` → 200** con `{"id","name","tag","emblem":{…}}`;
  ejemplo medido: `[AMX] Le Coq Sportif`. Hay campos de más (`emblem`) que el parseo ignora.
- **Sin guiones NO funciona**: los 32 caracteres seguidos devuelven **HTTP 500**
  `{"text":"unknown error"}`. El id se usa tal cual viene en el matchup; no hay que normalizarlo.
- Un id que ya no existe devuelve **404** `{"text":"guild not found"}`: es el caso normal de un
  gremio disuelto, así que la UI debe caer al id y no reintentarlo en cada sondeo.
- **Coste:** una llamada por gremio. Con un tope de 10 por sondeo, un matchup de veinte gremios
  queda con nombre completo en ~1 minuto y luego vive en memoria toda la sesión.

### Notas de verificación: coordenadas y cajas del mapa (T-14, 2026-10-31)

Medido contra la API en vivo (`/v2/maps?ids=38,95,96,1099` y `/v2/wvw/objectives?ids=all`).

- **`coord` de los objetivos está ya en coordenadas de continente**, el mismo espacio que
  `continent_rect` de `/v2/maps`: Mendon's Gap (Eternos) llega como `10256.6, 13514.4` y la caja de
  Eternos es X[8958..12030] Y[12798..15870]. **No hay que pasar por `map_rect`** (que además es
  simétrico, ±36864 o ±30720/±43008) para dibujar: la proyección es normalizar dentro de
  `continent_rect` y voltear la Y. Test: `rf30_every_real_objective_of_eternals_falls_inside_the_box`.
- **`continent_rect` siempre viene con la esquina menor primero**: Eternos `[[8958,12798],
  [12030,15870]]`, Alpino verde/azul `[[5630,11518],[8190,15102]]` / `[[12798,10878],[15358,14462]]`,
  Desierto `[[9214,8958],[12286,12030]]`.
- **`points_tick` NO viene en `/v2/wvw/objectives`** (los 178 objetivos llegan sin ese campo): el
  umbral de «alto valor» de RF-30b solo se puede calcular desde los objetivos del **matchup**
  (`/v2/wvw/matches`), que sí lo traen. La vista de mapa usa por ahora el criterio por tipo.
- **Cobertura de datos**: 178 objetivos en total, **81 con `coord: null`** (spawns y algunos
  genéricos), así que omitirlos (RF-30) no es un caso raro. Eternos trae 28, de ellos 22 con
  coordenada. Tipos: 42 campamentos, 39 torres, 24 fortalezas, 1 castillo, 30 ruinas, 24 spawns,
  9 genéricos, 6 recursos, 3 mercenarios.
- **`points_tick` real por tipo** (match `2-2`): campamento **2**; torre **4, 6 u 8**; fortaleza
  **8, 12, 16 o 20**; castillo **12**; ruinas, mercenarios y spawns **0**. Con el umbral 8 de
  RF-30b quedan marcados 17 objetivos y ningún campamento.
- `/v2/maps` con los cuatro ids son ~2 KB: se piden una vez por sesión.
### Notas de verificación: buscar clanes y saber su equipo (2026-10-31)

Comprobado `[API]` en vivo, para el rediseño del selector de clanes (T-44).

- **`GET /v2/guild/search?name=<texto>` existe, no pide clave y devuelve ids de clan.** Medido:
  `name=lord` → `["114D20B6-6E89-4432-982C-C77B58F57460"]`. Aparece en el índice de `/v2` como
  `/v2/guild/search` sin marca `[a]` ni `[l]`.
- **Lo que no sirve:** `/v2/guild/:id/log` (el historial real del clan) **requiere clave `[a]`**, así
  que el «historial del clan» solo puede salir de lo que veamos nosotros.
- **`GET /v2/wvw/guilds/:region`** (`eu`/`na`) devuelve `{ "<guild_id>": "<team_id>" }`; 282 644
  bytes para EU. Es la única fuente oficial para saber en qué equipo está un clan.
- La región del jugador no se expone: se deduce del equipo configurado (`1-x` = NA, `2-x` = EU).
### Notas de verificación: servidor de teselas del mapa (T-29, 2026-10-31)

Medido descargando imágenes reales con `curl` (una a una, comparando tamaños y contenido).

- **`GET https://tiles.guildwars2.com/{continente}/{piso}/{zoom}/{x}/{y}.jpg`** existe, no pide clave
  y no necesita `User-Agent` especial.
- **Los mapas de MvM están en el continente `2`, piso `1`.** El continente **1** devuelve una imagen
  de océano para *cualquier* coordenada: no da 404, así que una ruta mal montada **falla en
  silencio** con un fondo azul plausible.
- **El zoom máximo útil es 5** (6 y 7 dan 404). A zoom 5 una tesela cubre **1024 unidades de
  continente**, no 2048: la fórmula es `256 · 2^(7 − zoom)`.
- **Calibración con una tesela conocida**: `2/1/5/10/14.jpg` es Eternos con el Castillo de
  Piedraniebla en el centro. Con 1024 unidades por tesela, la caja de Eternos
  X[8958..12030] Y[12798..15870] cae en x 8..11, y 11..15 → **4×4 teselas**.
- **Cobertura comprobada**: 59 teselas entre los cuatro mapas (Eternos 16, Alpino verde 12, Alpino
  azul 15, Desierto 16), todas descargables. Es un orden de magnitud razonable para llevarlas a
  `<addon>/tiles/` la primera vez.
- Se descargan en el **worker** (`catalog::ensure_tiles`, hilo propio) y el render solo pide texturas
  ya presentes (4 por frame como máximo), con `add_image(..).col([1,1,1,opacidad])` para heredar la
  opacidad de la ventana.
### Notas de verificación: nombres de equipo desde gw2mists con el navegador (T-45, 2026-10-31)

La API de gw2mists da **403** a `curl` y la web es una SPA, así que `curl` sobre el HTML no ve nada.
Con el **navegador integrado** (que ejecuta el JavaScript) sí: las cuatro páginas se leen enteras y
cada tabla trae el nombre, y el ID de la fila da el orden alfabético de cada idioma.

- `https://gw2mists.com/en/teams/eu` → 15 equipos EU; `/de/teams/eu` → los mismos 15 **en alemán**
  (el orden cambia, así que se emparejan por la fila: alianzas, gremios, jugadores y comandantes).
- `/en/teams/na` → 12 equipos NA; `/de/teams/na` → los mismos 12 en alemán.
- La web **solo lista los equipos que están jugando**: nuestra tabla tiene 33 ids (los de
  `all_worlds`) y la web enseña 27, así que los 6 restantes (Mithric Cliffs, Lagula's Kraal, De
  Molish Post, Temple of Febe, Gyala Hatchery, Grekvelnn Burrows) se quedan con su nombre inglés.
- **El alemán de la web no es el del Excel**: `Grenths Tor` → `Grenths Tür`, `Frost-Zitadelle` →
  `Frostzitadelle`, `Ettin-Rücken` → `Ettinbuckel`, `Voliere des Großen Hauses` → `Vogelhalle des
  Großen Hauses`. Manda la web, que es lo que el jugador ve en el juego.
- **La nación coincide con la del API**: `Lutgardis Conservatory` (11011) aparece en la tabla **NA**
  de gw2mists, igual que en `all_worlds` (`1-1`). Un equipo del otro continente se reconoce por la
  primera cifra del id (`11xxx` NA, `12xxx` EU).
- Resultado: `data/team-names.json` se rehace con los 33 nombres (EN de `data/wvw-teams.json`, DE de
  la web, ES/FR del Excel cuando el nombre está en la hoja y si no el inglés), y el jugador puede
  pisar cualquiera desde los ajustes (`team_translations`), que siempre gana.

### Notas de verificación: `/v2/guild/search` (T-44, 2026-10-31)

Medido con `curl` contra la API en vivo, porque el buscador de clanes depende de esto.

- **`GET /v2/guild/search?name=<palabra>`** no pide clave y devuelve ids de clan.
- **Busca por prefijo, sin distinguir mayúsculas y solo una palabra**:
  `name=Raven` → 1 id (de *Legends Of Ravenous…*), `name=raven` → el mismo, `name=avenous` → **[]**
  y `name=legends+of` → **[]**. Con dos palabras no encuentra nada, así que la UI pide una.
- `%20` y `+` dan el mismo resultado: el problema no es la codificación, es que solo acepta un
  término.
- **`name=dolyak` devuelve un solo id y no es el clan del jugador** (medido 2026-10-31):
  devuelve `F0CE8616-8AD5-4D30-91E4-A1B54C3EC1CE` = `[DYAK] Dolyak`, mientras el suyo es
  `6C45F3C8-F21C-EF11-81AB-8B4A188BDE13` = `[YaK] Dolyak Sanctuary`. Con «dolyak» no sale
  *Dolyak Sanctuary*: la búsqueda no es por subcadena. Por eso el buscador del addon mira también
  los clanes que el jugador ya sigue, sin red.
- `/v2/wvw/guilds/:region` sigue siendo la forma de saber de qué equipo es cada clan: 282 644 bytes
  para EU, así que se cachea por región. Un clan sin reclamar en el emparejamiento actual **no
  aparece** en ese mapa (`F0CE8616` no está ni en `eu` ni en `na`).

### Notas de verificación: `claimed_by` sobrevive al cambio de color (2026-10-31)

Medido en `/v2/wvw/matches/1-1` y en la fixture `match_2-2.json`.

- Una estructura reclamada trae `claimed_by`, `claimed_at` (cuándo la reclamó el clan) y
  `last_flipped` (cuándo cambió de dueño). En la fixture, los 41 reclamos tienen
  `claimed_at` **posterior** a `last_flipped` (el clan reclama después de capturar).
- La API **no borra** `claimed_by` cuando la estructura cambia de bando: solo un **nuevo reclamo**
  lo sustituye. Quien se fíe de `claimed_by` a secas pinta el anillo de un clan en un campa de otro
  color, que es lo que vio el jugador el 2026-10-31. La regla que se aplica: un reclamo vale
  mientras `last_flipped <= claimed_at`; si `last_flipped` es posterior, el clan ya no la tiene.

### Notas de verificación: MumbleLink en WvW (`name`, `world_id`, `shard_id`, `team_color_id`, 2026-10-31)

Leído en vivo de la memoria compartida `MumbleLink` con el juego abierto (`map_id` 96, blue
borderlands), para decidir si se puede saber la partida sin preguntar al jugador.

- **`name` = «Guild Wars 2»**, no el personaje. El personaje está en `identity`:
  `{"name":"Jhoin","profession":6,"spec":48,"race":2,"map_id":96,"world_id":2147483649,
  "team_color_id":9,"commander":false,"map":96,"fov":1.222,"uisz":1}`. Indexar el roster por
  `name` guardaba el equipo del personaje bajo una clave que no era el personaje.
- **`shard_id` = 2147483649 (`0x80000001`)**, `instance` = 0, igual que `world_id` (`0x80000001`).
  Ese valor **no aparece en `all_worlds` de ninguna de las nueve partidas** (que mezclan mundos
  viejos como 1015 y equipos nuevos como 11012), así que **no identifica la partida**: buscar el
  shard en `/v2/wvw/matches` no puede funcionar. La detección de 0.22.9 no llegó a funcionar nunca.
- **`team_color_id` = 9** en WvW (y `0` fuera): no es 1/2/3, así que tampoco sirve como color de
  equipo con el mapeo que se suponía. Queda sin verificar qué significa.
- Lo que sí identifica la partida es el **equipo guardado**: el clan del jugador (`6C45F3C8`,
  YaK) tenía un reclamo en `1-1` (`96-38`, azul) y su equipo (`11011`) juega en `1-1` como azul.
- `ui_state` = 24 en WvW (`GAME_HAS_FOCUS | IS_IN_COMPETITIVE_MODE`).

### Notas de verificación: orientación de las coordenadas y del mapa real (2026-10-31)

Medido porque el jugador avisó de que «se ven 3 trozos: norte, centro, sur; el norte y el sur están
intercambiados» (el mapa llevaba al revés desde la calibración de teselas de 0.16.1).

- **El continente crece hacia el SUR.** En `/v2/continents/1/floors/1` los `label_coord` son
  `Ruins of Orr` y≈42112, `Kryta` y≈30080, `Janthir` y≈16252 y `Cantha` y≈101100. Orr está al sur de
  Kryta y Cantha más al sur todavía, así que **más Y = más al sur**. `continent_rect` va de la esquina
  noroeste a la sureste, igual que la pirámide de teselas (la fila 0 es el norte del continente).
- **Comprobado dibujando el mapa**: se bajan las 16 teselas de Eternos (`2/1/5/{x}/{y}.jpg`, columnas
  8–11, filas 12–15) y se pegan en orden de fila creciente. El resultado es Eternos **al derecho**, y
  cada objetivo de `/v2/wvw/objectives` cae sobre su estructura: *Stonemist Castle* en el castillo del
  centro, *Overlook* en la fortaleza del norte (y≈13656), *Lowlands* y *Valley* en las esquinas del sur
  (y≈15130), *Speldan Clearcut*, *Mendon's Gap* y *Veloka Slope* en el borde norte.
- Consecuencia para el código: `project::to_screen` **no debe invertir la Y**. Continente e imgui
  crecen hacia abajo, así que la proyección es escala y desplazamiento. Invertirla dejaba el norte
  abajo y cada fila de teselas en el sitio de otra.
