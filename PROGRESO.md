# Progreso: gw2-lurker

Bitácora para continuar el trabajo, incluso si quien lo retoma no es la misma IA ni la
misma sesión. **Lo más reciente va arriba.** Cada entrada dice qué se pretendía, qué se
hizo, cómo se verificó y qué quedó pendiente.

Reglas:
- Al **empezar** una tarea: añade una entrada `EN CURSO` con el plan en 3–5 líneas.
- **Durante**: actualiza esa entrada con hallazgos, decisiones y callejones sin salida.
- Al **terminar**: cámbiala a `HECHO`, pon los commits y lo que no se pudo verificar.
- No borres entradas antiguas. Si algo se revierte, dilo aquí y por qué.
- Lo que contradiga la spec se propone y se decide, no se entierra aquí.

---

## EN CURSO – 2026-10-31 · Siguiente: T-31 (i18n) y las vistas del MVP

- Hecho sin necesitar al usuario: publicación automatizada (T-34), caché en disco (T-11b),
  gate de visibilidad, modelos de las tres vistas y el check de trazabilidad (ver abajo).
- Siguiente sin el juego: T-11c (hilo worker con parada limpia y estado de salud) y favoritos
  (RF-73). Cuidado con RF-32a: identificar garrison/colinas/bahía requiere criterio que aún no
  está verificado.
- **Requiere GW2**: T-12 (leer `NexusLink`/`Context`), T-35, T-01, T-02b, T-25.
- Decidido: **sí** a la feature `rtapi` (RF-44); su coste en DLL se mide en T-02b.
- CI: el job `deny` seguía en curso al cerrar esta entrada; `license-files = []` es la
  primera prueba para las licencias de `nexus`. Si vuelve a fallar, hay que calcular el hash
  real de los ficheros de licencia, no seguir probando variantes.

- Trampa para quien siga: **la URL `raw/main` tarda en apuntar al commit nuevo**. Tras
  publicar 0.3.0, `raw.githubusercontent.com/.../main/gw2_lurker.dll` seguía sirviendo la DLL
  de 0.2.0 (1 161 728 bytes) mientras el commit (`/419e857/...`) y el asset de la release ya
  devolvían los 1 200 640 correctos, incluso añadiendo `?v=0.3.0`. No es LFS ni un fallo del
  push: es caché del endpoint. Por eso `release.ps1` verifica y **falla** en vez de dar por
  bueno un HTTP 200. Se reintenta con `-VerifyOnly` pasados unos minutos; no se cambia la URL
  ni la versión para taparlo.

## HECHO – 2026-10-31 · Selector de equipo en tres pasos (0.5.0)

- `ui::settings` pide **región → color → equipo**. La lista sale de `worker::dashboard().candidates`
  (los matchups vivos), filtrada con `core::select::of_region_and_color`: con los datos del
  2026-10-31 son **5 filas en EU y 4 en NA** en vez de 27. Cada fila muestra `nombre (id) · 2-2`.
- La región se ofrece solo si tiene matchups vivos; el color va con su color real en la lista,
  como el panel de WvW del juego. Arranca en la región y el color del equipo guardado; lo que el
  jugador elija se recuerda durante la sesión (`Narrowing`, no se persiste: es una forma de
  navegar, no una preferencia).
- Si el worker aún no ha sondeado, la ventana lo dice; si la combinación no tiene equipos,
  también, en vez de un desplegable vacío sin explicación.
- **Orden de candados**: los candidatos se copian del dashboard **antes** de bloquear la
  configuración, y el worker nunca tiene el dashboard mientras pide la configuración. Así no se
  repite el interbloqueo de 0.4.2.
- Verificado: 36 tests en el addon (4 nuevos del selector), 71 en `core`, `clippy -D warnings`
  limpio, DLL de 1 469 952 bytes. Datos reales comprobados con `curl`: 5 matchups EU, 4 NA.

## HECHO – 2026-10-31 · Ajustes en Documentos (como los vecinos) y nombre de personaje

- `addon::ffi::paths::documents()`: `SHGetKnownFolderPath(FOLDERID_Documents)` + `CoTaskMemFree`
  por `extern` (sin el crate `windows`), porque Documentos está redirigido a OneDrive y leer el
  registro daría otra ruta. Comprobado en esta máquina con un test que exige que la carpeta
  exista.
- Los ajustes pasan a `<Documentos>\Guild Wars 2\addons\gw2_lurker\settings.json`, igual que
  `timers_wvw` y `gw2_builds_wvw`. Ventaja añadida: escribir ahí **no necesita administrador**,
  al contrario que `Program Files`. Si Documentos no se pudiera resolver, se cae a la carpeta de
  Nexus para no perder los ajustes.
- Migración automática una sola vez desde la carpeta antigua (`settings.json` o `config.json`),
  sin borrar el original; y `install.ps1` ya informa de la ruta correcta.
- Migrada a mano la carpeta de esta máquina: `settings.json` + `.bak`.
- **Segundo bug encontrado al leer el fichero migrado**: la clave del roster era el JSON de
  identidad completo (`{"name":"Jhoin",...}`), no el nombre, porque `read_identity_string()`
  devuelve ese blob. Ahora se usa el campo `name` de Mumble con el JSON solo como respaldo
  (`ffi::identity_name`, función pura con tests). El roster migrado quedó corregido a
  `{"Jhoin": 11011}`.
- Verificado: 35 tests en el addon (2 nuevos), 71 en `core`, `clippy -D warnings` limpio.

## HECHO – 2026-10-31 · Arreglado el cuelgue al entrar en un mapa (0.4.2)

- **Síntoma**: el juego se congelaba al entrar en un mapa después de elegir personaje (reportado
  por el usuario en 0.4.1).
- **Causa**: `ui::settings::draw` pedía el equipo con `team::selected()`, que hace
  `runtime()` (bloquea la configuración) mientras el render ya tenía ese mismo candado tomado.
  `std::sync::Mutex` no es reentrante ⇒ interbloqueo en el propio hilo. Se disparaba en el
  primer frame tras la selección de personaje porque es cuando `settings_available()` pasa a
  verdadero y el selector se dibuja. Mismo problema en el panel de opciones de Nexus.
- **Arreglo**: `team` tiene ahora variantes `selected_in(&Config)`, `remember_in(&mut Config)` y
  `forget_in(&mut Config)`, y los sitios que ya tienen el candado usan esas. La regla queda
  escrita en el módulo: *nunca llamar a la variante que bloquea con un guard vivo*.
- **Test**: la precedencia (override manual → roster del personaje) se comprobó con una función
  pura `resolve(manual, roster, character)`, porque en un test no hay Nexus y `character_name()`
  revienta al llamar a `AddonApi::get()`.
- Verificado: `clippy -D warnings` limpio, 30 tests en el addon, 71 en `core`; DLL 0.4.2 de
  1 461 760 bytes.

## HECHO – 2026-10-31 · Worker de sondeo y datos en vivo en la ventana de estado (T-11c)

- `addon::worker`: hilo propio, `MAY_POLL` como `AtomicBool` que escribe el render (la compuerta
  manda), parada con `stop()` y espera en tramos de 250 ms, backoff exponencial con tope de
  5 min. Red y disco **solo** en ese hilo; el render lee un `Dashboard` pequeño bajo un mutex.
- Primera vuelta con `ids=all` (~400 KB) porque aún no se sabe el equipo; después
  `/wvw/matches/{id}` (~44 KB) gracias al nuevo `Api::match_by_id`. Si el equipo guardado ya no
  juega, vuelve a `ids=all` para que el selector ofrezca los equipos vivos.
- Eventos → `SessionStore` (histórico de la sesión) y últimos 50 al dashboard; claims y clanes
  salen de `core::matchup`. Nombre de objetivos cacheado por idioma.
- La ventana de estado enseña emparejamiento, `N objetivos reclamados`, `N clanes` y hora.
- El equipo elegido se guarda **por personaje** (`team::remember`/`forget` sobre `Roster`), con
  `config.team_id` como override manual que gana siempre. Ojo: el cambio se aplica **después**
  de soltar el candado de la config, porque `team::*` lo vuelve a tomar (si no, interbloqueo).
- Verificado: `live_pipeline_builds_the_dashboard` (ignorado por defecto) recorre WinHTTP →
  `matches?ids=all` → objetivos en español → `match_by_id` → claims → ranking y pasa en 1,2 s;
  27 tests en el addon, 70 en `core`, 14 en `api`. DLL 0.4.0: **1 455 616 bytes**, todavía por
  debajo del objetivo de 1,5 MB pero con poco margen.

## HECHO – 2026-10-31 · Instalación local con install.bat / install.ps1

- Copiados del patrón de los vecinos (`gw2-timers`, `fight_analysis_danger`):
  [install.bat](/C:/Users/Usuario/OneDrive/proyectos/gw2-guild-lurker/install.bat) lanza
  [install.ps1](/C:/Users/Usuario/OneDrive/proyectos/gw2-guild-lurker/install.ps1) con
  `-NoProfile -ExecutionPolicy Bypass`, así que sirve el doble clic.
- `install.ps1`: `-Build` (tests + `cargo build --release`), `-DryRun` y `-Dest <ruta>`.
  Detecta la carpeta del juego por registro (`Install64Path`/`InstallPath`, comprobando que
  exista `Gw2-64.exe`) con `C:\Program Files\Guild Wars 2\addons` como respaldo; verifica
  tamaño (una DLL de 130 bytes sería un puntero LFS), cabecera `MZ` y MD5 antes **y después**
  de copiar, y pide elevación solo si Program Files la exige.
- Se niega a copiar con GW2 abierto (el fichero está bloqueado) y lo dice sin traza de
  excepción, que es lo que ve alguien que ha hecho doble clic.
- Trampa: lanzado desde `cmd` con `powershell` (5.1), `Get-FileHash` no se resuelve en esta
  máquina; el MD5 se calcula con `System.Security.Cryptography.MD5` y funciona en 5.1 y en 7.
- Verificado: `-DryRun`, copia real a una carpeta temporal (bytes y MD5 idénticos), rechazo
  con GW2 abierto y error claro con un destino inexistente.

## HECHO – 2026-10-31 · Nombres traducidos, herramienta idempotente y etiqueta de equipo de arcdps

- Pregunta del usuario: «saca las traducciones de gw2mists». Medido: gw2mists **solo tiene
  inglés** (`?lang=es`/`?locale=es` siguen devolviendo `Moogooloo`) y la API oficial responde
  404 en `/v2/wvw/teams`, así que no hay fuente automática. Se deja la estructura
  `names: {es, fr, de}` en `data/wvw-teams.json` para curación humana y
  `scripts/update_wvw_teams.ps1` ahora **preserva** esas traducciones (probado con `-DryRun`,
  33 equipos). `core::TeamTable::label_in("es", id)` cae al inglés si falta.
- Pregunta: «fight_analysis_danger sí sabe qué color soy». Comprobado: no lo lee del juego, lo
  saca de **arcDPS** (`Agent::is_self` + `Agent::team`; Nexus lo sirve como
  `EV_ARCDPS_SELF_JOIN`). Su propio enum etiqueta el caso vivo como `Team::Own`
  (*"without a verified color mapping"*).
- Implementado aquí `addon::arc_team`: se suscribe a `SELF_JOIN` y guarda el `team` de arcdps
  como **etiqueta opaca**, con `Calibration` para aprender el color cuando el jugador elige
  equipo y para detectar que está en otro equipo (alt o sorteo mensual). Sin dependencias
  nuevas: feature `arc` de `nexus-rs`.
- Verificado: `clippy -D warnings` limpio; 26 tests en el addon (3 nuevos), 70 en `core`
  (incluye `rf44_translated_team_names_fall_back_to_english`).

## HECHO – 2026-10-31 · Selector de equipo en dos pasos y memoria por personaje

- `core::select` (69 tests en `core`, todos con datos reales de `/v2/wvw/matches?ids=all`):
  `Selection::{Missing, Confirmed, Stale}`, `candidates`, `teams_of`, `matchups_of`,
  `rivals`, `is_team` y `Roster` por personaje.
- Respuesta a las dudas del usuario, comprobadas en el código: **el juego no puede decirte tu
  equipo** (Mumble no lo tiene; `Identity::team_color_id` es un id de tinte, no el color de
  WvW; Extras no incluye equipo; Nexus no lo expone), y **la API no traduce nombres de
  equipo** (`/v2/wvw/teams` → 404, mientras que `/v2/wvw/objectives?lang=es` sí traduce).
- Diseño aceptado: región → matchup → 3 equipos con color; solo ids de equipo (los `2xxx` de
  mundo antiguo quedan fuera); si el equipo guardado ya no juega, el selector se abre solo; y
  memoria por personaje para que la cuenta alt tenga la suya.
- Trampa: `TeamColor::ALL` está en orden `[Red, Green, Blue]` (el de la UI), no
  `[Red, Blue, Green]`; el test lo fija para que nadie lo desordene sin querer.

## HECHO – 2026-10-31 · DLL en `target\release` como los vecinos

- Duda del usuario: los proyectos hermanos dejan la DLL en `target\release\` y aquí aparecía en
  `target\x86_64-pc-windows-msvc\release\`. La causa era mía: compilaba siempre con
  `--target x86_64-pc-windows-msvc`, y Cargo anida la salida bajo el triple cuando se pasa ese
  flag. El host **ya es** ese triple, así que el flag no aportaba nada.
- Cambiado a `cargo build --release` en `scripts/release.ps1` (`-Target` queda como opción
  explícita), en el job `build-windows` de `ci.yml` y en `docs/RELEASE.md` / `AGENTS.md` §7.
- Segunda duda: había **dos** DLL, `gw2_lurker.dll` y `gw2lurker_addon.dll`. La segunda era un
  resto de antes de fijar `[lib] name = "gw2_lurker"`: Cargo nombra el cdylib como el paquete
  (`gw2lurker-addon`) si no se le dice otra cosa. El árbol viejo se ha borrado y solo queda
  `target\release\gw2_lurker.dll`.
- La build sin `--target` pesa 1 270 784 bytes (1 536 menos, misma funcionalidad: cambia la
  metadata, no el código) y es la que se ha vuelto a publicar como 0.3.2 para que el asset
  coincida con lo que produce el script.

## HECHO – 2026-10-31 · Idioma real del juego vía Extras y Nexus (T-31 cerrada en lo posible)

- Pregunta del usuario: «Nexus sabe el idioma que usas, ¿cómo lo hace?». Respuesta medida:
  Nexus **no** lo expone por API; lo guarda en `Nexus/Settings.json` como nombre visible y
  carga `Nexus/Locales/<id>_Main.json` (que sí lleva `"Identifier": "es"`).
- Implementado `addon::lang` con las dos fuentes de verdad:
  1. `nexus::event::extras::LANGUAGE_CHANGED` (feature `extras` de `nexus-rs`), que el plugin
     Unofficial Extras emite **también al inicializar** con el idioma actual: idioma real del
     juego, en vivo y sin sondeo. `revert_on_unload()` deja la desuscripción a Nexus.
  2. `Nexus/Settings.json` + `Locales/*_Main.json` como respaldo (dos lecturas de fichero en
     `load`, memorizadas; nunca en el hilo de render).
- Cadena final: preferencia → Extras → RTAPI → Nexus → idioma de Windows → inglés.
- Coste: **+15 872 bytes** de DLL (1 256 448 → 1 272 320). Nada de dependencias nuevas.
- Trampa: `event_consume!` sólo acepta una función, no una closure; con closure infiere `<()>`
  y falla con un error de tipos que no dice nada del macro.
- Verificado: `clippy -D warnings` limpio; 23 tests en el addon (5 nuevos de `lang`).

## HECHO – 2026-10-31 · Cliente HTTP real sobre WinHTTP (T-11b cerrada)

- `addon::ffi::http`: GET con WinHTTP del sistema (`WinHttpOpen/Connect/OpenRequest/Send/Receive/
  ReadData`), handles con `Drop`, timeouts de 10 s, tope de 8 MB y `SAFETY` en cada bloque.
- `addon::net::WinHttpTransport` implementa `gw2lurker_api::Transport` y traduce estados:
  2xx → cuerpo, 404 → `NotFound`, 429/5xx → `Status`.
- Se descartó un cliente con TLS embebido: habría sumado 1,5–2 MB a una DLL de 1,26 MB.
- Verificado de verdad: `cargo test -p gw2lurker-addon live_get -- --ignored` → **HTTP 200** con
  el JSON de `2-2` en 270 ms. Los tests de `split_url` cubren https/http, puerto explícito y
  URLs inválidas.
- Trampa: en una `cdylib`, `dead_code` marca como muerto todo lo que aún no se usa, y el
  transport sólo lo usará el worker (T-11c); de ahí el `#[allow(dead_code)]` temporal con
  motivo escrito al lado.

## HECHO – 2026-10-31 · Ajustes como los vecinos y escritura fuera del render

- Fichero `settings.json` en `<addons>/gw2_lurker/` (antes `config.json`), con
  `settings.json.bak`, escritura atómica y lectura del fichero antiguo para migrar.
- `addon::writer`: hilo propio que recibe copias de la config por canal, coalesce y escribe;
  la UI llama a `persist_if_changed()` como mucho cada 500 ms. En `unload` hay guardado final
  y `join`. Cumple la regla 6 sin perder ajustes si el juego se cierra de golpe.
- Ajustes también en el panel de opciones de Nexus (`RenderType::OptionsRender`), visible
  fuera de WvW: la configuración deja de depender de la compuerta.
- Diagnóstico para el caso «carga pero no dibuja nada»: en `load` se registra la ruta de
  ajustes y `mumble map/tipo/gameplay`; cada cambio de compuerta deja una línea
  `gate: map .. tipo .. gameplay .. -> show_ui ..`; y `debug_show_everywhere` (por defecto
  `false`) permite saltarse la compuerta sin tocar código.
- Verificado: `clippy -D warnings` limpio, 16 tests en el addon (uno nuevo del escritor).
- **Primera prueba real en juego (2026-10-31 15:38, versión 0.3.0)**: `Nexus.log` dice
  `Loaded addon: gw2_lurker.dll` en 695 µs con `Signature: -1282765419` y
  `WvW Lurker cargado (idioma en)`. Se creó `addons/gw2_lurker/config.json`, pero con
  `"windows": {}` y sin entrada en `Nexus/imgui.ini` ⇒ **el render no llegó a dibujar la
  ventana ni una vez**: o la compuerta la ocultaba (el usuario cerró el juego 25 s después de
  cargar, probablemente en carga/selección de personaje) o no estaba en WvW. La build 0.3.1
  deja la respuesta en el log (`gate: map .. tipo .. gameplay .. -> show_ui ..`) y añade los
  ajustes en el panel de opciones de Nexus, que no depende de la compuerta.
- Dato del mismo log: **RealTime API no está instalado** en esa máquina (no aparece en
  `Nexus.log`), de ahí el `idioma en` con el cliente en español. Añadido el respaldo por
  idioma de Windows; queda propuesto como cambio de RF-44.
- La carpeta de datos real del addon es `C:\Program Files\Guild Wars 2\addons\gw2_lurker\`
  (junto a `ControlOptions`, `WvWFightAnalysis`…), no la de Documentos: ahí `addons/` es sólo
  datos de arcdps. La DLL va a `<juego>\addons\gw2_lurker.dll`.

## HECHO – 2026-10-31 · Ventana de ajustes con selector de equipo (T-30, T-17 parcial)

- `addon::ui::settings`: ComboBox de equipo con `nombre (id)` sobre la tabla incluida, opción
  «sin equipo», alias editable, idioma, intervalo de sondeo (`InputInt`, saneado a 15–600) y
  alcance de los avisos. Se abre desde el botón *Ajustes* de la ventana de estado.
- Las ediciones se acumulan en una copia local y se escriben al terminar de dibujar, porque
  `window::draw` ya tiene prestado el estado de la ventana.
- `core::teams::sorted()` devuelve la lista ordenada por región y nombre; sin orden, el
  `HashMap` hacía bailar el desplegable entre frames. 6 claves nuevas de traducción.
- Trampa: el orden alfabético deja **EU antes que NA** (`E` < `N`), no al revés como asumí en
  el primer test; el test ahora comprueba bloques y orden, no cuántos NA hay al principio.
- Verificado: `clippy -D warnings` limpio; 15 tests en el addon, 60 en `core`, 13 en `api`,
  6 en `store`; DLL 0.3.0 publicada y verificada por bytes y MD5.

## HECHO – 2026-10-31 · Contrato de ventana compartido (T-17 parcial)

- `addon::ui::window::draw` es el único sitio que dibuja cromo: barra de título propia (texto
  traducido + id estable), `×` a la derecha, plegado con doble clic, mover/redimensionar solo
  si no está bloqueada, `bg_alpha` con la opacidad, menú contextual con *Bloquear posición*,
  *Opacidad* y *Restablecer posición y tamaño*, y tamaño mínimo 180×90.
- Estado por ventana en `config.json` (`windows.<clave>`): `visible`, `collapsed`, `locked`,
  `opacity`, `position`, `size`, con saneo (NaN, infinitos, tamaños imposibles) y prueba de
  ida y vuelta.
- La UI escribe en una copia mutable (`lib::runtime()`); el fichero se guarda en `unload`
  porque el hilo de render no puede hacer E/S. Se pierde lo movido si el juego revienta; el
  guardado diferido llega con el worker (T-11c).
- Decisión: el plegado **no** usa el `collapsed` de imgui. Con `title_bar(false)` una ventana
  plegada de imgui no dibuja nada y no habría forma de desplegarla; se controla desde nuestro
  estado y se autoajusta el tamaño.
- Verificado: `clippy -D warnings` limpio; 12 tests en el addon (4 nuevos de RF-46), 59 en
  `core`, 13 en `api`, 6 en `store`; `check_loc.sh` y `check_traceability.sh` en verde.

## HECHO – 2026-10-31 · i18n es/en/fr/de con override de Ctrl derecho (T-31)

- `core::i18n`: 36 claves indexadas por el texto inglés, columnas `[es, en, fr, de]`, `t`/`w`
  (`###clave` para que el id de imgui no cambie al traducir) y `f` con `{0}`/`{1}`. Columna
  vacía → inglés; clave desconocida → la propia clave. 9 tests, entre ellos el de completitud
  de las cuatro columnas (falla si alguien añade una clave solo en un idioma).
- `addon::i18n`: preferencia del usuario (si no es *Auto*) → `RealTimeApi::read_game().language`
  → inglés; override a inglés mientras se mantiene Ctrl derecho con el juego en primer plano.
  Registra las claves en Nexus con `set_translation` desde `load`.
- `ffi::keys`: `GetAsyncKeyState(VK_RCONTROL)` y foco por proceso en primer plano, con
  `extern "system"` contra `user32` (no se añade el crate `windows`).
- Verificado: `clippy -D warnings` limpio, 9 tests en `core`, 8 en el addon (3 nuevos), DLL de
  release **1 062 400 bytes** (+8,7 KB sobre la anterior) dentro del objetivo.
- Trampa para quien siga: la spec citaba `UiState::GAME_HAS_FOCUS`, que **no existe** en
  `nexus-rs` 0.12; lo he dejado documentado en `07-spikes.md` y corregido en `02-design.md`
  §7c. Tampoco imgui distingue Ctrl izquierdo de derecho: de ahí Win32.
- Pendiente: probar en juego que el cambio de idioma y el override funcionan (T-35) y la
  revisión de terminología por nativos (T-41).

## HECHO – 2026-10-31 · Publicación automatizada de la DLL (T-34)

- `scripts/release.ps1`: compila, comprueba `MZ` y tamaño (`check_size.sh`), escribe el `.md5`
  con `UTF8Encoding($false)` (32 bytes exactos, sin BOM ni salto), añade la excepción de LFS a
  `.gitattributes` del repo compartido, **aborta** si `git cat-file -s :gw2_lurker.dll` no
  coincide con el tamaño local (puntero LFS) o si `check-attr` no devuelve los cuatro atributos
  en `unset`, hace commit **solo** de `gw2_lurker.*`, publica la release con `--latest=false`,
  borra las releases nuestras más allá de la anterior y descarga la URL exacta de `update_link`
  para comparar bytes y MD5.
- Opciones: `-DryRun` (compila y valida, no toca lo remoto) y `-VerifyOnly` (solo verifica lo
  ya publicado, útil tras un aviso de caché).
- Corregido un fallo propio que esto destapó: la DLL se llamaba `gw2lurker_addon.dll` porque
  el `[lib]` no tenía `name`. Ahora `crates/addon/Cargo.toml` fija `name = "gw2_lurker"` y el
  `ci.yml` apunta al nombre nuevo.
- Verificado: `-DryRun` completo (DLL 1 053 696 bytes, `MZ`, MD5 calculado, tamaño dentro del
  objetivo). La publicación real y la descarga se comprueban en el mismo comando.
- Trampa para quien siga: en PowerShell, `"$size:"` dentro de una cadena se parsea como
  unidad/scope y rompe el fichero; hay que escribir `${size}`.

## HECHO – 2026-10-31 · Caché en disco de objetivos y guilds (T-11b)

- `crates/api/src/cache.rs`: `Cache` sobre el directorio del addon. Objetivos **por idioma**
  (`objectives-<lang>.json`) para que cambiar de idioma o mantener Ctrl derecho pulsado
  (RF-45) no toque la red, y `guild-<id>.json` para no repetir `/v2/guild/:id` cada sesión.
- Una entrada corrupta o ausente es un *cache miss*, nunca un error que aborte la llamada.
- `GuildInfo` ahora también implementa `Serialize` (antes solo `Deserialize`).
- Verificado: `cargo clippy --workspace --all-targets --exclude gw2lurker-addon -D warnings`
  limpio; 13 tests en `api` (4 nuevos), 50 en `core`, 6 en `store`.
- Ojo para quien siga: `write` necesita `T: Serialize + ?Sized` porque se le pasa `&[T]`.

## HECHO – 2026-10-31 · Capa de API con transporte intercambiable (T-11b parcial)

- `crates/api`: `trait Transport` (solo baja bytes) y `Api<T>` con `matches_all`,
  `match_for_team(team_id)`, `objectives(lang)` y `guild(id)`. El cliente HTTP real queda
  fuera a propósito: lo elige la medición de T-02b.
- `match_for_team` **escanea `ids=all`** en vez de usar `?world=<id>`, porque ese parámetro
  espera un id de mundo antiguo y nosotros identificamos equipos por 11xxx/12xxx.
- `ApiError` (transporte, estado, parseo, no existe) y `Backoff` exponencial con tope y
  criterio de reintento (5xx y 429 sí, 4xx no).
- 9 tests con fixtures reales: equipo 12014 → match `2-2` azul, 503, JSON truncado, idioma
  en la URL, guild con tag.
- 65 tests en total (50 core + 9 api + 6 store).

## HECHO – 2026-10-31 · Addon real: export, contexto y configuración (T-12 parcial)

- `nexus` 0.12.0 (tag fijo) con `log`, `log_filter`, `mumble` y `rtapi`; añadido `log` y
  `clock` de chrono. `deny.toml` gana `allow-git` para los tres repos de `zerthox`.
- `export!` con **`UpdateProvider::Direct`**, `DisableHotloading` y la URL de
  `gw2addonsreleases`. Comprobado dentro del binario: contiene `GetAddonDef`, "WvW Lurker",
  `gw2_lurker.dll` y `raw.githubusercontent.com`.
- `ffi/mod.rs` (único `unsafe`, con `SAFETY`): `is_gameplay()` por `read_volatile` y
  `mumble_map()`. `context.rs` los convierte en el `GameState` de `core::gate`.
- `config.rs`: JSON en `addons/gw2_lurker/config.json`, defaults, saneo de rangos (incluido
  NaN), renombrado a `config.json.corrupt-<fecha>` si está dañado, `save()` listo para T-17.
- Ventana de estado mínima, gateada por `core::gate`: solo se dibuja en WvW jugable.
- **Tamaño medido: 1.053.696 bytes (1,00 MB)**, `MZ`, bajo el objetivo de 1,5 MB de ADR-008
  y aún sin cliente HTTP. Apuntado en T-02b.
- 61 tests (50 core + 6 store + 5 addon). Clippy en verde.
- Trampa: `Utc::now()` exige la feature `clock` de chrono; y `log` es dependencia directa,
  no basta con la feature de nexus.

## HECHO – 2026-10-31 · Claims tardíos con bandera `late` (T-09)

- `Event::Claim` gana `late: bool`. Un baseline emite `Baseline` y después **un claim por
  cada objetivo reclamado**, con su `claimed_at` real y `late = true` (RF-25). Los claims
  observados en vivo llevan `late = false`.
- Consecuencia para quien consuma: el baseline de una sesión son ~41 eventos, no uno. El
  repositorio de eventos ya los guarda tal cual (T-10).
- Tests: los de baseline ahora separan eventos de ciclo de vida de los claims tardíos, y hay
  uno específico que comprueba guild y hora reales de un claim tardío.
- 50 tests de core + 6 de store.

## HECHO – 2026-10-31 · Almacenamiento por sesión (T-10 parcial)

- `crates/store`: `SessionStore` con `Record { v, at, match_id, event }` (los eventos son
  los de `core::diff`, serializados tal cual). `open()` **borra los `.jsonl` anteriores**
  (ADR-012), `append()` rota a los 2 MB (RNF-03) y `read_all()` salta líneas corruptas o de
  versión futura y las cuenta.
- Dos bugs propios cazados por los tests: la rotación reutilizaba el nombre de fichero y
  truncaba registros (ahora `sesion-000.jsonl`, `sesion-001.jsonl`…), y un test comparaba
  una fecha literal con otra calculada. Ambos corregidos.
- 6 tests de store + 49 de core. Falta la consulta por guild y rango (T-11).

## HECHO – 2026-10-31 · Favoritos y limpieza de módulos (T-39 parcial)

- `core::follow`: `FollowList` con `add`/`remove`/`toggle` idempotentes, sin duplicados y
  serializable como array JSON; `favourites_first` sube las seguidas sin desordenar el resto.
- `diff.rs` llegó a 501 líneas (aviso de RNF-10) al crecer los tests: partido en
  `diff/mod.rs` (196) + `diff/tests.rs` (244). El fichero más grande ahora es `model.rs` (375).
- 49 tests en verde; trazabilidad: 21 requisitos con test, 52 sin cubrir.
- Trampa: `cargo fmt | Select-Object -First 5` corta el proceso y deja el formateo a medias.
- Trampa: una dependencia entre crates del workspace **necesita `version`** además de
  `path`; sin ella `cargo deny` la ve como wildcard y falla el job de supply-chain.

## HECHO – 2026-10-31 · Vistas de claims y trazabilidad (T-36/37/38 parcial, T-03b)

- `core::matchup`: `claims()` (41 filas reales), `by_map()` en orden fijo, `with_kinds()`
  para los filtros, `rank_guilds()` por claim más reciente con desempate estable,
  `split_by_map()` y `guild_timeline()`. 7 tests.
- `scripts/check_traceability.sh`: 83 requisitos, 20 con test, 18 con prueba manual, 53 sin
  cubrir. Falla si un test cita un RF inexistente (probado con `rf999`), informa de los no
  cubiertos y `--strict` falla por cobertura. Añadido a la CI.
- Trampa: los comodines de la matriz (`RF-03*`) se colaban como requisito; ahora se filtran.
- 44 tests en verde.

## HECHO – 2026-10-31 · Bocetos HTML (T-32)

- `docs/boceto.html`: una página, sin red, con cuatro pestañas (alertas, mapa, guilds,
  historial) y una columna "en el juego" con el widget sobre un fondo simulado.
- **Datos reales**: 76 objetivos con coordenadas de los cuatro mapas y los 41 claims del
  match `2-2`, capturados de las fixtures. Los 39 tags de guild salen de `/v2/guild/:id`.
- Decisiones visibles que hay que validar: tamaño y contorno doble por `points_tick`,
  anillo discontinuo para "sin reclamar", lista de guilds por claim más reciente,
  separación "este mapa / otros mapas" y ruta dibujada en el último mapa con su nombre.
- `PRODUCT.md` escrito para que la skill `impeccable` tenga contexto (suposiciones marcadas).
- Detector `impeccable detect`: corregidos texto en mayúsculas, sombra difusa con borde fino
  y contraste del texto atenuado. Quedan avisos de tipo *slop* (radial-halo, grid) y tres
  de contraste que son falsos positivos: el detector no resuelve el fondo oscuro que se
  aplica con `background` + degradados; el texto real sobre `#0f151c` pasa AA de sobra.
- No verificado: que el usuario lo encuentre útil. Es justo lo que toca preguntar.

## HECHO – 2026-10-31 · Cimiento: T-03, T-03a, T-04, T-05, T-06, T-07

- **Workspace** `x86_64-pc-windows-msvc`, 4 crates, lints del workspace (`unwrap_used` y
  `expect_used` en deny), perfil release de ADR-008, licencia **GPL-3.0-only**.
- **CI** (`.github/workflows/ci.yml`): fmt, clippy, test, `check_loc.sh`, `cargo deny`,
  `cargo audit`, build release msvc y `check_size.sh`. ~1 min por push.
  - Trampa encontrada: los `.sh` estaban en modo `100644` → `exit 126`. Arreglado con
    `git update-index --chmod=+x` y llamando con `bash`.
  - Trampa encontrada: `rustsec/audit-check` compila `cargo-audit` y costaba **339 s**.
    Sustituido por `taiki-e/install-action` + `cargo audit`.
- **`core::model`** (T-04): tipos serde de `/v2/wvw/matches`, `/v2/wvw/objectives`,
  `/v2/maps`, con fixtures reales en `crates/api/tests/fixtures/`.
- **`core::teams` + `core::resolver`** (T-05): tabla `data/wvw-teams.json` embebida con
  `include_str!`, `label()` con caída a `(id)`, `resolve()` sobre `all_worlds`.
- **`core::diff`** (T-06, T-07): `Differ` con `Baseline`/`MatchChanged`/`Gap` y, cuando no
  hay baseline, `Flip`/`Claim`/`Unclaim` ordenados por id de objetivo (`Unclaim` antes de
  `Claim`).
- Verificación: `cargo fmt --check`, `clippy -D warnings` y **31 tests** en verde; CI verde.
- Commits: `73b1eaa` (T-03), `add0bf2` (T-04 + licencia), `4195f85` (T-05), `69fa39b` (T-06),
  `36506ef` (T-07), `cd6441a`/`e7a30ad` (CI), `c03e0fb` (specs + backlog).

## HECHO – 2026-10-31 · Spikes T-02 y notas verificadas

- T-02 medido de verdad: 227 peticiones, 0 errores. Cambios de propiedad con mediana 19–20 s;
  un emparejamiento inactivo pasó **22 min 40 s sin un solo cambio**. Se mantiene 30 s.
- Hallazgo: la API **no** expone nombres de equipo (404 en `/v2/wvw/teams` y
  `/v2/wvw/alliances`). Sí da `guild → equipo` en `/v2/wvw/guilds/:region`.
- Hallazgo: `coord` está en coordenadas de continente; la caja correcta es `continent_rect`.
- Todo en `specs/07-spikes.md`, sección *Resultados*.

---

## Trampas ya conocidas (no volver a tropezar)

- `owner` viaja como `"Red"`/`"Green"`/`"Blue"` (PascalCase); los contenedores usan claves
  minúsculas. Mezclarlo rompe la deserialización.
- `claimed_by` es **una** guild o `null`, nunca una lista. Las claves siempre vienen.
- Torre totalmente mejorada llega a `points_tick` 8: "alto valor" no puede ser por tipo.
- Ids de equipo 11001–11015 (NA) y 12001–12018 (EU). Prefijo del match: `1-x` NA, `2-x` EU.
- Nexus expone `NexusLink::is_gameplay`: sirve para no dibujar en carga/selección.
- Publicar: `UpdateProvider::Direct` + `DisableHotloading`, MD5 de 32 hex sin BOM, excepción
  de LFS para la DLL, tag propio con `--latest=false`. Ver `docs/RELEASE.md`.

## Bloqueos

- **T-01, T-02b, T-25**: requieren ejecutar dentro de GW2 (carga, click-through, tamaño de
  DLL con TLS, comportamiento de actualización). No se simulan.
- **T-29 y T-30**: la matriz de trazabilidad las cita y no existen en `03-tasks.md`.
