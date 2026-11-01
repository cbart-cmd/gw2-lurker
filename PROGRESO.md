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

## Trampas ya conocidas (no volver a tropezar)

- `owner` viaja como `"Red"`/`"Green"`/`"Blue"` (PascalCase); los contenedores usan claves
  minúsculas. Mezclarlo rompe la deserialización.
- `claimed_by` es **una** guild o `null`, nunca una lista. Las claves siempre vienen.
- Torre totalmente mejorada llega a `points_tick` 8: "alto valor" no puede ser por tipo.
- Ids de equipo 11001–11015 (NA) y 12001–12018 (EU). Prefijo del match: `1-x` NA, `2-x` EU.
- Nexus expone `NexusLink::is_gameplay`: sirve para no dibujar en carga/selección.
- `crate::config()` es la copia **del arranque** y no cambia nunca; lo que el jugador toca en la
  ventana está en `crate::runtime()`. Leer `config()` desde el render hizo que el clic derecho no
  viera el click-through, que las listas de clanes no se refrescaran y que el clic derecho no
  deshiciera nada (0.22.10).
- **La DLL instalada no se queda**: Nexus (`UpdateProvider::Direct`) la sobrescribe en cada arranque
  con la que haya publicada en `gw2addonsreleases`. Probar una versión local exige publicarla
  (`scripts/release.ps1`) o desactivar la comprobación de actualización en Nexus (0.23.1).
- **`raw.githubusercontent.com` cachea ~3 min**: el `.md5` se sirve antes que la DLL, así que una
  release recién subida parece buena si solo se mira el HTTP 200. Comprobar **bytes y MD5** de la
  descarga real (`scripts/release.ps1 -VerifyOnly`) antes de decirle al jugador que abra el juego.
- MumbleLink `name` = «Guild Wars 2», **no** el personaje: el personaje está en `identity`
  (`{"name":"Jhoin",…}`). Y `shard_id` en WvW vale `0x80000001` (medido 2026-10-31): no es el mundo
  ni la partida, así que no sirve para detectar el emparejamiento.
- Un reclamo del API sigue ahí aunque el objetivo cambie de color: hay que comparar `last_flipped`
  con `claimed_at` o el mapa pinta un anillo en un campa de otro color (0.22.10).
- Publicar: `UpdateProvider::Direct` + `DisableHotloading`, MD5 de 32 hex sin BOM, excepción
  de LFS para la DLL, tag propio con `--latest=false`. Ver `docs/RELEASE.md`.

## Bloqueos

- **T-01, T-02b, T-25**: requieren ejecutar dentro de GW2 (carga, click-through, tamaño de
  DLL con TLS, comportamiento de actualización). No se simulan.
- **T-29 y T-30**: la matriz de trazabilidad las cita y no existen en `03-tasks.md`.

---

## HECHO – 2026-11-01 · Menos contexto por sesión sin perder controles

- Publicada **0.26.0** (MD5 `18175802c121f071bfd954452b7223fd`, 2 190 336 B), commit `20e5dda`. Al
  subir versión hay que cambiar también la `version` de las dependencias internas
  (`crates/addon`, `crates/api`): sin eso Cargo no resuelve el workspace.
- `AGENTS.md` de 16 KB a ~8 KB: mismas reglas en una línea; las explicaciones largas (tests,
  candados) se movieron tal cual a `docs/agents/lecciones.md`. Lectura por IDs/secciones, no
  ficheros enteros.
- Entradas de `PROGRESO.md` hasta 0.25.13 movidas tal cual a
  `docs/historial/progreso-hasta-0.25.13.md` (132 KB → 13 KB). Trampas y Bloqueos, arriba.
- `scripts/validate.ps1`: fmt, clippy, loc, trazabilidad y tests con una línea por paso; logs en
  `target\validate\`. `scripts/check_bugs.ps1`: una línea si `bugs.md` no cambió.
- No verificado: ahorro real de créditos; hay que compararlo en las próximas sesiones.

## HECHO – 2026-11-01 · Seis mejoras API/addon/core (0.26.0)

- Retirada la caché sin consumidores; los errores `ApiError` llegan tipados a la UI y se localizan; el worker comparte `api::Backoff` con catálogo y búsquedas.
- UUIDs de clan canonizados en seguimiento, configuración, nombres de sesión, búsquedas, mapa guild→equipo y claims; añadidas pruebas para entradas minúsculas/sin guiones. Capturada una respuesta real `ids=all` (9 partidas) como fixture independiente.
- Documentación de límite y cadencia corregida: los 5 s son solo para el estado de mapas del matchup elegido; `ids=all` se limita a una vez por minuto. Eliminados `revisioncopilot.md` y `addendaparacopilot.md`.
- Añadido el monitor de reasignación basado en `teamAssignment` y `matches/overview`: al confirmar el cambio, reinicia el baseline y limpia historial/alertas sin tocar ajustes ni clanes seguidos; la UI muestra el inicio del ciclo. Actualizadas constitución, requisitos, diseño, escenarios y changelog.
- Verificado: formato, Clippy, LOC (worker 791 líneas), trazabilidad (32 RF/RNF aún sin prueba), workspace (308 pasados, 4 ignorados), build release y DLL de 2 190 336 B (objetivo 2 306 867 B).
- No verificado: comportamiento de reasignación dentro del juego. El usuario eliminó las entradas anteriores de `bugs.md` durante el trabajo; no se restauraron ni se modificó ese fichero. Sin commit.

## HECHO – 2026-11-01 · Límite real de la API y addenda de Sonnet (api + addon)

- Límite de la API (wiki, copia de archive.org del 2025-12-16): cubo de **300** y recarga de **5 por segundo** por IP; la cabecera `X-Rate-Limit-Limit: 600` no es «por hora» (ni «por minuto», como decía la addenda). Corregidos RNF-02, el contexto de `01-requirements.md`, `07-spikes.md` (corrección fechada, el histórico se conserva) y los comentarios de `worker.rs`, `catalog.rs`, `config/mod.rs` y `net.rs`.
- Decisión del jugador: los 5 s son **solo** para el estado de los mapas del emparejamiento elegido. `should_skip_poll` ahora deja esperar al enfriamiento de `ids=all` (60 s) cuando aún no se conoce la partida del equipo: antes `Fetch::Team` pedía ~400 KB en cada sondeo. Test `rnf02_only_the_watched_match_is_polled_at_the_configured_cadence`.
- `api`: `match_by_id`, `guild` y `objectives` validan id/idioma antes de construir la URL (`NotFound` sin petición); `looks_like_guild_id` pasa al `api` y `search.rs` la reutiliza; el `Fake` responde `NotFound` como `net.rs`; docs corregidas (cabecera con `ureq`, «contiene» → prefijo, `ids=all` de las dos regiones, «30 s»). Tres tests nuevos.
- Addenda descartado tras comprobarlo: 7.1.3 (búsqueda vacía = `200 []`, medido), 7.2.4 (ya hay tope de 8 MB en `ffi/http.rs`), test de mapas «débil» (el `Fake` solo responde a la URL exacta) y 1.8 (el dato era incorrecto).
- Propuestas sin aplicar: borrar `api::cache` (muerto, T-11b lo cita), traducir `ApiError` en la UI, unificar el backoff del worker con `api::Backoff`, normalizar ids de clan a mayúsculas (toca `core`), usar los timers de reasignación y un `ids=all` real de nueve partidas como fixture.
- Verificado: fmt, clippy `-D warnings`, 293 tests (4 ignorados), `check_loc.sh`, trazabilidad (33 sin cubrir; RNF-02 ya tiene test), release 2 170 368 B. No verificado en juego. Sin commit.

## HECHO – 2026-11-01 · Verificación de revisioncopilot.md

- Corregidos defectos reproducibles en los tres crates: el diff ya no repite claims conocidos tras un hueco; el worker reutiliza sondeos globales y espera sus hilos al descargar; catálogos y teselas aplican cooldowns y cancelación.
- El índice valida el límite ZIP con los bytes realmente leídos, serializa escrituras y fusiona nombres añadidos durante la descarga. También se corrigieron Alt, la gracia de claims tardíos y la codificación de búsquedas.
- La UI aplica el tema antes de crear la ventana, usa el fondo y el idioma actuales, evita clonar el dashboard con la ventana cerrada y permite mostrar el popup de prueba sin alertas reales; se corrigieron detalles del mapa.
- Verificado: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (289 pasados, 4 ignorados), `check_loc.sh`, `check_traceability.sh`, `cargo build --release` y `check_size.sh` (2 169 856 B; objetivo 2 306 867 B).
- La trazabilidad mantiene 34 requisitos sin test o prueba manual; `check_loc.sh` informa nueve archivos entre 500 y 800 líneas. No se modificaron specs ni se añadieron dependencias.
- No verificado: comportamiento dentro del juego y las optimizaciones que requieren mediciones en vivo. Sin commit.

## HECHO – 2026-11-01 · Auditoría general de rendimiento y robustez

- En `core::diff`, los claims caducados ya no contaminan historial/baselines, los flips que invalidan un claim producen `Unclaim` y un reclaim del mismo clan produce el par `Unclaim`/`Claim` (RF-22, RF-25). Tres regresiones nuevas.
- En el índice, el checksum publicado es obligatorio y se informa cualquier descarga/validación fallida sin instalar datos sin verificar (RF-20c). Tres regresiones nuevas.
- La comprobación de DLL nueva pasa al worker, se repite cada 10 s y el render solo lee el estado en caché (RF-50). El writer informa de fallos al arrancar/enviar y hace un guardado alternativo al cerrar.
- Verificado: fmt, clippy, LOC, trazabilidad, workspace (271 passed, 4 ignored), build release y DLL de 2 160 128 B (bajo el objetivo de 2 306 867 B).
- La trazabilidad sigue mostrando 34 RF/RNF sin test o prueba manual; se dejaron como deuda fuera de esta auditoría correctiva.
- No verificado: comportamiento dentro del juego. Durante el trabajo el usuario quitó las entradas de `bugs.md`; se respetó y no se restauraron.

## HECHO – 2026-11-01 · La pestaña *Mapa* se abre en el mapa del jugador (0.25.15)

- Petición del jugador: al entrar en la pestaña *Mapa* desde otra, y si no se llega por clicar un
  clan, se abre en el mapa en el que está.
- `draw_tabs` guarda en `shown` la pestaña que imgui dibuja este fotograma (`TabItem::build` solo
  ejecuta la elegida, `arcdps-imgui` 0.8 `widget/tab.rs:98`) y la compara con la del fotograma
  anterior (`LAST_TAB`). La regla va en `snap_to_current(shown, last, jump)`, pura y con test, y
  excluye el salto programático: `jump` es la pestaña que pide `WANT_TAB`, que es justo lo que pone
  un clic de clan, así que la ruta de un clan sigue abriendo **su** mapa (RF-32a).
- `map_choice` se queda como estaba: es la elección de la sesión, y esto solo la mueve al entrar.
- Spec: RF-30 gana el GWT de este caso. `bugs.md` con la entrada de hoy.
- Publicado en **0.25.15** (MD5 `5456bb462d3d29b5fa85e8585b44dd4e`). `install.ps1` **no pudo
  copiarla**: GW2 estaba abierto y bloquea la DLL. Se instalará al cerrar el juego (o Nexus la
  sustituye en el siguiente arranque). `cargo fmt`, `clippy -D warnings`, 264 tests, `check_loc.sh` y
  `check_traceability.sh` en verde.
- **No verificado:** verlo cambiando de mapa en el juego.

## HECHO – 2026-11-01 · Lista de clanes: `+` para seguir e `i` para ver la ruta en el mapa (0.25.14)

- Petición del jugador: en la pestaña *Clanes*, la lista ordenada por reclamo no decía qué hacía
  clicar una fila. Quiere dos botones: `+` (seguir, si no lo está ya) e `i` (información), y que la
  inferencia se vea **en el mapa** y no en texto debajo de la lista.
- `guilds::Action` pasa de `ToggleFollow` a `Follow` (solo añade; quitar es la `×` de la lista de
  seguidos, que pregunta antes) y `ShowOnMap`, que reutiliza el camino que ya existía: seleccionar el
  clan, llevar el mapa al último donde reclamó y saltar a la pestaña *Mapa*.
- Si ya se sigue, en el sitio del `+` va un `*` atenuado: ocupa lo mismo (filas alineadas) y pulsarlo
  no hace nada a propósito.
- **La ruta en texto se muda**: `route_steps` y `route_reason` salen de `ui/guilds.rs` a
  `ui/map/route.rs`, con `summary(rows, guild, language)`; la pestaña *Mapa* pinta el resumen debajo
  del dibujo. RF-35 («salto a `<mapa>`») sigue cubierto y su test se ha movido a `ui/map/tests.rs`,
  junto al de `summary`. `map.rs` reexporta `summary` como `pub(crate)` para no engordar el fichero
  (está en 755 líneas).
- Nuevo `follow::add_followed`, que usan los dos `+` de la interfaz (lista de la sesión y lista de
  clanes): añade, guarda el nombre conocido en `settings.json` y se lo enseña al índice. Si el clan ya
  se seguía o la lista está llena, no toca nada.
- Spec: RF-71 con los dos botones y tres GWT nuevos. `bugs.md`: **el jugador había vaciado el
  fichero** (solo quedaba la cabecera); no se ha restaurado nada de git y solo se ha añadido la
  entrada de hoy.
- Publicado en **0.25.14** (MD5 `7e8b2c863e832a3287986498620f2ac2`) e **instalado** en el juego.
  `cargo fmt --check`, `clippy -D warnings`, 263 tests, `check_loc.sh` y `check_traceability.sh`: todo
  en verde.
- **No verificado:** verlo en el juego (los botones y el salto de pestaña no se pueden probar fuera).

## HECHO – 2026-11-01 · `bugs.md` como fichero compartido (regla nueva)

- Petición del jugador: si él borra filas de `bugs.md`, es cosa suya y **no se restauran**; lo que no se
  puede hacer es borrar lo que él añada mientras se trabaja en un arreglo.
- Escrito en tres sitios para que no se pierda: regla de oro **§2.11**, paso **§3.7** (sustituye la
  nota suelta que había ahí) y **§9** con las dos mitades de la regla y la técnica de escritura
  (`edit` con ancla existente, comprobar el número de entradas después; el reemplazo de PowerShell se
  comió 14 entradas el 2026-11-01 sin error). También un apunte en **§10** para `PROGRESO.md` y una
  cabecera nueva en el propio `bugs.md`.
- Esto **anula** la restauración desde git que se hizo el 2026-10-31: a partir de ahora, filas que
  falten = filas que él borró.

> Entradas anteriores a 0.25.13: [docs/historial/progreso-hasta-0.25.13.md](docs/historial/progreso-hasta-0.25.13.md).

