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
- `InputBinds.json` cambia con el juego abierto: no cachear el atajo para siempre. Lo relee el
  worker según la fecha de modificación y la tecla se nombra en el hilo que pide el texto
  (`GetKeyNameTextW` usa la distribución de teclado de ese hilo).

## HECHO – 2026-11-01 · Menos ruido en `Nexus.log` (sin publicar)

- Resultado con su propio log (95 líneas del addon en 5 min): 59 eran `evento: Claim/Flip/Unclaim`
  (→ `debug`, no llegan al log porque `log_filter` es `info`), 8 `deteccion:` (2 textos distintos),
  5 `vigilando` (3 distintos, por el nº de emparejamientos), 1 `arcdps: cuenta`. Quedan ≈ 25 (arranque
  y `gate:`, que ya eran cambios de estado). En régimen estable pasa de ≈ 3 líneas/min (vigilando +
  deteccion + latido) a 1 cada 5 min más lo que cambie de verdad. Estimación, no medida durante una hora.
- Cambios: `diagnostics.rs` (`changed`, `watched_key` sin el nº de emparejamientos, `heartbeat_due` por
  tiempo con `HEARTBEAT_EVERY` = 300 s), `detect.rs` (`detection_line` pura: calla si el juego y el
  equipo guardado coinciden; el resto se escribe una vez por resultado), `arc_team.rs` (sin log de la
  cuenta; sigue guardándola). `worker.rs` intacto. Tests: `t35_*` (3, en `diagnostics.rs`) y
  `rf03a_a_detection_*` / `rf03a_a_shard_*` (3, en `detect.rs`). `validate.ps1` completo en verde.
- **Verificado en el juego**: `AgentUpdate::account` llega relleno, con los **dos puntos** de arcdps
  (`:Lord Rematador.3056`). La clave del equipo por cuenta usa ese texto tal cual; no se normaliza para
  no dejar huérfano lo guardado con la 0.26.5. El plan B con `EV_ACCOUNT_NAME` no hace falta.
- Lo que no se puede ver sin publicar: Nexus sobrescribe la DLL con la publicada, así que el recorte se
  comprueba en el siguiente `Nexus.log` tras una 0.26.6.
- Petición del jugador: «ya no hace falta crear tantos logs». Con su `Nexus.log` delante (90 líneas
  del addon en 4 minutos de juego) las fuentes de ruido son: una línea por cada `Claim` (~50 `late: true`
  al arrancar) y `Flip`/`Unclaim`; `worker: vigilando` que **alterna** cada minuto porque su clave
  lleva el número de emparejamientos (9 con `ids=all`, 1 con la partida sola); `deteccion:` repetida
  cada 60 s con el mismo texto; el latido cada 10 sondeos (≈1 min con 5 s, no los «cinco minutos» que
  decía el comentario); y `arcdps: cuenta <nombre>`, que ya cumplió (**verificado en el juego**: llega
  `:Lord Rematador.3056`, con los dos puntos de arcdps) y además metía el nombre de cuenta en un fichero
  que se pega en los informes.
- Plan: quitar la línea de la cuenta; `Claim`/`Flip`/`Unclaim` a `debug` (el addon filtra a `info`);
  `vigilando` y `deteccion` solo al cambiar de verdad; latido por tiempo (5 min). Sin tocar `worker.rs`
  (777 líneas).

## HECHO – 2026-11-01 · Publicada la 0.26.5

- Release `gw2-lurker-v0.26.5`, MD5 `5db876cb1ce08641388a44cc08b28d8e` (2 202 112 B), verificado con
  la descarga real. Retención: quedan 0.26.5 y 0.26.4; borrada la 0.26.3.
- Contenido: el equipo se guarda por cuenta (`AgentUpdate::account` de arcdps).
- Pendiente de probar en GW2: ~~que `Nexus.log` diga `arcdps: cuenta <nombre>`~~ (verificado el
  2026-11-01: `:Lord Rematador.3056`; esa línea ya no se escribe desde el recorte de logs) y que
  el selector salga una sola vez por cuenta, no por personaje.

## HECHO – 2026-11-01 · El equipo se guarda por cuenta (RF-03a)

- Petición del jugador: con dos cuentas, el equipo/zona debe guardarse por **cuenta**, no por
  personaje, para que todos los personajes de una cuenta lo compartan y la otra cuenta arranque
  vacía.
- **No hace falta detectar nada nuevo**: `arc_team.rs` ya se suscribe a `EV_ARCDPS_SELF_JOIN`, y el
  `AgentUpdate` que trae el equipo lleva también `account()` (nombre de cuenta) y `character()`
  (`nexus/src/api/event/arc.rs`). El campo se tiraba; ahora se guarda en un `Mutex` y se expone en
  `arc_team::self_account()`. Cero dependencias nuevas, cero `unsafe` nuevo (el accesor es seguro).
- `team.rs`: `key(account, character)` decide la clave (cuenta primero; personaje como respaldo y
  como clave heredada de ≤ 0.26.4) y `resolve` la usa, así que una entrada vieja por personaje sigue
  resolviendo. `remember_in` y `forget_in` guardan/borran por cuenta.
- `worker.rs`: el barrido de los nueve emparejamientos se rearma cuando cambia la **cuenta**
  (`identity_changed` con `team::current_key()`), no con cada personaje.
- Spec actualizada (RF-03a) y CHANGELOG. Tests: `rf03a_the_team_is_remembered_per_account_and_shared_by_its_characters`
  y `rf03a_a_team_saved_per_character_still_resolves_after_the_upgrade`.
- ~~UNVERIFIED: que `AgentUpdate::account` llegue con `Jhoin.1234` en el juego.~~ **Verificado el
  2026-11-01**: llega `:Lord Rematador.3056` (con los dos puntos de arcdps). El log de la cuenta se
  quitó después (ver «Menos ruido en `Nexus.log`»).

## HECHO – 2026-11-01 · Publicada la 0.26.4

- Release `gw2-lurker-v0.26.4`, MD5 `b4536bef9dae5b6f0c38732493bda61d` (2 199 552 B), verificado
  con la descarga real. Retención: quedan 0.26.4 y 0.26.3; borrada la 0.26.2.
- Entra lo de la pestaña *Mapa*: abre en el mapa del jugador y sin clan elegido, la entrada
  «No guild» en el desplegable y el salto arreglado al volver a mano.
- Pendiente de probar en GW2 (es lo que el jugador va a hacer con esta versión).

## HECHO – 2026-11-01 · La pestaña Mapa abre sin clan elegido (RF-71)

- Petición del jugador: al entrar a mano en *Mapa* (sin venir de clicar un clan) debe ponerse en su
  mapa, con el desplegable de clan vacío, y verse todo lo que reclaman los clanes que sigue.
- El mapa ya se ponía en el del jugador (`snap_to_current` + `resolved_map_role`); lo que fallaba
  era el desplegable: tomaba el clan de la pestaña *Clanes* (`guilds::selected_guild()`), así que
  al volver seguía elegido y dibujaba su ruta.
- `navigation.rs`: estado propio del mapa (`guild_choice`/`set_guild_choice`) y
  `guild_from_pick`, que convierte la entrada «No guild» (cadena vacía) en «sin ruta». Test
  `rf71_entering_the_map_tab_by_hand_leaves_no_guild_chosen`.
- `map.rs`: el desplegable estrena entrada «No guild» (antes no había forma de vaciarlo) y aplica
  la elección a ese estado. `mod.rs`: el snap de entrada vacía la elección y un clic de la pestaña
  *Clanes* o de un aviso la fija, así los dos sitios siguen en sintonía.
- `validate.ps1 -Release`: todo OK, 329 tests, DLL 2 199 552 B.
- **Trampa (informe del jugador, 2026-11-01):** «vengo de un clic de clan» se decidía con
  `WANT_TAB != TAB_MAP`, y esa marca vive hasta que la pestaña se dibuja. Si imgui no cambia de
  pestaña en un fotograma, al volver a *Mapa* a mano parecía que se venía de un clan y no se
  saltaba al mapa del jugador. Ahora manda un fotograma: `FRAME`/`GUILD_JUMP_AT` y
  `navigation::entering_by_hand`, con ventana de un fotograma. Test
  `rf30_returning_to_the_map_tab_by_hand_snaps_to_the_players_map`.

## HECHO – 2026-11-01 · Publicada la 0.26.3

- Release `gw2-lurker-v0.26.3` en `cbart-cmd/gw2addonsreleases`, MD5
  `bd625f8c7ed73d5e607c40f69daecf35` (2 197 504 B), el mismo que la DLL de `cargo build --release`
  de `bcd7d2f`. `release.ps1 -VerifyOnly` descargó y comprobó el fichero publicado.
- Contenido: el aviso del click-through sigue al `InputBinds.json` real (commit `25dd2a5`) y el
  historial de un clan se pinta de más reciente a más antiguo (`8e2d8d8`, `ui/guilds.rs`).
- Retención: quedan 0.26.3 y 0.26.2; la 0.26.1 ya no está en el repo de distribución.
- Probado en GW2 (2026-11-01) y los tres bugs abiertos quedan `[x]` en `bugs.md`: el aviso nombra
  la tecla nueva, la ruta y el aviso de «sin eventos» funcionan y el historial del clan sale con lo
  más reciente arriba.

## HECHO – 2026-11-01 · Subir versión en un solo paso · 0.26.3

- Subir versión se hacía a mano en cuatro sitios y era fácil dejarse las dependencias internas:
  ahora `scripts/bump-version.ps1 -Version x.y.z` toca `Cargo.toml` del workspace, la `version` de
  las dependencias internas en `crates/*/Cargo.toml`, las entradas `gw2lurker-*` de `Cargo.lock`
  (por texto, sin llamar a cargo) y pasa `[Unreleased]` de `CHANGELOG.md` a la versión con fecha.
  Es idempotente y tiene `-DryRun`, `-Date` y `-SkipChangelog`.
- `scripts/check_version.sh` comprueba que los seis sitios coinciden; `validate.ps1` lo corre como
  paso `version` (también en la pasada por defecto).
- Trampa: en .NET las regex numeran los grupos **con nombre después** de los anónimos, así que
  `${2}` no era el cierre de comilla y duplicó bloques de `Cargo.lock`. Se usa `${head}` + literal.
- De paso, `-Only fmt,version` (la sintaxis que documenta la cabecera) fallaba porque `ValidateSet`
  no admite la lista con comas al invocar con `-File`; ahora se separa y se valida a mano.
- `validate.ps1 -Release`: todo OK, 328 tests, DLL 2 197 504 B. Orden del historial de un clan
  cambiado a más reciente primero en `ui/guilds.rs` (commit `8e2d8d8`, del jugador).

## HECHO – 2026-11-01 · Atajo del click-through desactualizado (RF-18, sin publicar)

- Bug: el jugador cambió el atajo a Alt+Shift+Ñ en Nexus y el aviso seguía diciendo Ctrl+Shift+Ñ.
  `InputBinds.json` (18:33:36) ya decía `Alt: true, Ctrl: false`; el addon lo leía solo al cargar
  en un `OnceLock`.
- `keybinds.rs`: `Watch` compara la fecha del fichero y lo relee solo si cambió; `refresh()` se
  llama en `worker::sleep` (cada 250 ms, nunca en el render). `shortcut()` devuelve `String`.
- Test `rf18_a_rebinding_made_while_playing_reaches_the_message` (fichero temporal reescrito).
  `validate.ps1 -Release`: todo OK, 328 tests. Pendiente: comprobarlo en GW2.

## HECHO – 2026-11-01 · Favoritos coherentes y paso 0 de la ruta · 0.26.2

- Primero: clic en favorito selecciona su historial sin saltar; botón `i` abre el mapa del último
  claim vigente de la fila, o del último histórico si ya no tiene reclamos (decisión del usuario).
  Sin historial ni reclamos, aviso y sin navegación.
- Después: aplicar la propuesta aprobada de origen 0 (garri, fortaleza lateral alpina o respawn
  del equipo del clan), con condiciones temporales y geográficas y coordenadas verificadas.
- Tests primero por hito, validación completa al estabilizar; sin inferir actividad real del clan.
- Regresiones de botones y mapa primero en rojo, después en verde; origen y plan de dibujo también.
  El historial conserva los claims vigentes si su observación antigua salió del tope de 2 000.
- Medición CPU con 2 000 claims: primer intento mediana 1,013 ms; índice de coordenadas bajó a
  0,467 ms pero no cumplió p95. Textos prestados (`Cow`) evitan copiar dos cadenas por paso:
  mediana 0,278 ms, p95 0,352 ms en release, por debajo de 0,5 ms. Frame completo pendiente en GW2.
- Validación completa correcta: 327 tests; los 4 tests de red ignorados también pasan al lanzarlos
  manualmente. Últimos ajustes de selección y destino revalidados; DLL 0.26.2 de 2 195 456 B.
  El destino usa la fila del fotograma para evitar cambios si el worker actualiza durante el clic.
- Publicada 0.26.2 con `scripts/release.ps1`: código `3c8cbcd`, distribución `5df5125`.
  Descarga real verificada: MZ, 2 195 456 B y MD5 `89b25637839b54a31dd848c83b51b57d`.
  Retención: 0.26.2 y 0.26.1; eliminada 0.26.0. `cargo bloat --release --crates` revisado.
- El benchmark CPU solo se compila en release; el publicador conserva su flujo y añade commit
  convencional con trailer de coautoría. Pendiente en GW2: clics, avisos, dibujo 0/1 y frame completo.

## HECHO – 2026-11-01 · Navegación al mapa de un clan (sin publicar; RF-30, RF-71, RF-32a)

- `ui/navigation.rs` conserva el salto pendiente hasta que imgui dibuja Mapa. El acceso manual
  aplica el mapa del jugador antes del dibujo, no después de sobrescribir el mapa del clan.
- Sin eventos, no se cambia la pestaña ni la selección: aviso traducido a es/en/fr/de con la hora
  de inicio del seguimiento y botón para cerrarlo. El selector de clan del mapa pasa por la misma
  validación que favoritos, avisos, registro e información del clan.
- Tres regresiones fallaron con las reglas anteriores; tras el arreglo, 315 tests correctos,
  4 ignorados. `validate.ps1`: fmt, clippy, LOC, trazabilidad y tests correctos.
- Prueba con `match_2-2.json`: clan no favorito, mapa del último claim y resumen de ruta disponibles.
  Sin red, E/S, nuevas dependencias ni consultas adicionales por fotograma al historial.
- Commit: `fix: conserva el mapa del clan al abrir su ruta`. Pendiente: publicar y comprobar la
  navegación y el aviso dentro de GW2. Anotado en `bugs.md`, README y CHANGELOG.

## HECHO – 2026-11-01 · Publicar 0.26.1

- Versión raíz y dependencias internas 0.26.1; CHANGELOG cierra los bugs del atajo con «Ñ» y del
  botón *Grabar*. Commits: `df5cf0e` (versión) y `435c454` (`Cargo.lock`).
- Validación `scripts/validate.ps1 -Release`: fmt, clippy, LOC, trazabilidad, 310 tests y build
  correctos; DLL de 2 189 312 B.
- Publicada con `scripts/release.ps1`: tag `gw2-lurker-v0.26.1`, MD5
  `1430a2dfc25378f3c4b1c9a5a36b23b4`. La descarga real coincidió en tamaño y MD5; retención eliminó
  `gw2-lurker-v0.25.15`.
- La confirmación del atajo «Ñ» requiere probarlo en GW2.

## Bloqueos

- **T-01, T-02b, T-25**: requieren ejecutar dentro de GW2 (carga, click-through, tamaño de
  DLL con TLS, comportamiento de actualización). No se simulan.
- **T-29 y T-30**: la matriz de trazabilidad las cita y no existen en `03-tasks.md`.

---

## HECHO – 2026-11-01 · Atajo con «Ñ» y botón *Grabar* (sin publicar)

- `keybinds.rs` traducía el código de escaneo con la tabla US (0x27 = «;»); ahora pide el nombre a
  Windows con la distribución del jugador (`ffi::keys::layout_key_name`, `GetKeyNameTextW`) y deja la
  tabla como respaldo. Tests `rf18_*` con teclado español falso y una llamada real con la L.
- Ajustes: fuera el botón *Grabar* y `TeamChange::SaveNow`; `persist_if_changed` ya guardaba cada
  cambio. En su lugar, «Los cambios se guardan solos.».
- No verificado: teclas extendidas (flechas, Insert) en `InputBinds.json` (`UNVERIFIED(RF-18)`), y la
  «Ñ» en el juego.

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
