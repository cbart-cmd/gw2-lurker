# 03 – Tareas

> **Esto es el registro histórico del proyecto**, no la descripción del código. Recoge las tareas
> con las que se construyó, en orden, y lo que se pidió en cada momento; algunas dicen cosas que hoy
> ya no son ciertas (por ejemplo, hablan de `crates/store`, de `data/equipos.xlsx` o del
> comportamiento de la UI en 0.10). **La verdad del código está en `02-design.md` y
> `01-requirements.md`**, y el día a día en `PROGRESO.md`. Si algo de aquí contradice al código, manda
> el código y esta lista se actualiza o se marca como superada.

Cada tarea es pequeña y verificable. Orden recomendado; las dependencias se indican entre paréntesis.

## Estado a 2026-11-02 (0.26.6)

Lo que **queda vivo**, con por qué:

| Tarea | Qué falta | Necesita |
|---|---|---|
| **Prueba B** | Probar el nuevo mapa 0.26.6 (MT-21); implementación T-51..T-53 terminada | El jugador |

Pruebas previas en juego aceptadas por indicación del jugador el 2026-11-02, sin atribuir nuevas
mediciones a la IA. T-21 y T-41 deprecadas por decisión del jugador; se conserva lo ya implementado.
Audio retirado del alcance; T-24 ya estaba cerrado. El estado fino está en `PROGRESO.md`.

## Estados que la lista original describía y hoy no existen

- **`crates/store` (S-01…S-09 de `06-diff-scenarios.md`)**: eliminado en 0.24.0, el historial es
  de la sesión y vive en memoria (ADR-015).
- **`data/equipos.xlsx` y `scripts/merge_team_names.ps1`**: retirados el 2026-11-01; las
  traducciones de equipo se editan a mano en `data/team-names.json` y salen de la lista de la wiki.
- **Cooldown por objetivo y sonido**: nunca se implementaron; se retiraron de la UI y de los
  requisitos (ver RF-40 y `06-diff-scenarios.md`, R-11).
- **Caché en disco de objetivos/mapas/guild→equipo**: hoy es memoria por sesión (y 30 min para el
  mapa guild→equipo del buscador).
- **Ventanas separadas** (ajustes, log, mapa, histórico): desde ADR-014 hay **una** ventana con
  pestañas.

## Fase 0 – Spikes (reducen riesgo)

- **T-01 ✔ (2026-10-31)** Spike Nexus: addon mínimo con `export!` que dibuja una ventana imgui, lee `map_id` de Mumble y envía una alerta con `send_alert`. *Hecho cuando:* se carga en el juego y muestra el `map_id` en vivo. Confirma R-1 y R-6.
- **T-02 ✔ (2026-10-31)** Spike API: script que consulta `/v2/wvw/matches?world=<id>` cada 10 s durante 30 min y registra cuándo cambian los datos. *Hecho cuando:* hay una cifra medida de caché real y de errores; se fija el intervalo por defecto (R-2).

## Fase 0b – Fundamentos de ingeniería

- **T-03 ✔ (2026-10-31)** Workspace Rust con `rustfmt.toml`, `[workspace.lints]` y `rust-version`. *Hecho:* `cargo fmt --check`, `clippy -D warnings` y `cargo test` verdes en un workspace vacío. Nota: no existe `tooling/` en el repo; la configuración se creó a mano (`rustfmt.toml`, `rust-toolchain.toml`, `deny.toml` ya estaban). `license` sigue sin decidir: **bloquea `cargo deny` en T-03a**.
- **T-03a ✔ (2026-10-31)** CI en `.github/workflows/ci.yml`: `cargo fmt --check`, `clippy -D warnings`, `cargo test`, `check_loc.sh`, `cargo deny` + `cargo audit`, build release `x86_64-pc-windows-msvc` y `check_size.sh`. *Pendiente por depender de otras tareas:* test de humo PE/export (T-27) y check de trazabilidad RF → tests (T-03b). Verificado que `check_loc.sh` falla con un fichero de 801 líneas.
- **T-03b ✔ (2026-10-31)** Check de trazabilidad RF → tests y `manual-tests.md`: `scripts/check_traceability.sh`, en CI. Falla si un test cita un requisito inexistente; informa de los que aún no tienen prueba; `--strict` falla por cobertura para usarlo antes de publicar. (T-03a)
- **T-02b ✔ (2026-10-31)** Spike de tamaño: medir DLL mínima con 2–3 opciones de cliente HTTP/TLS y fijar presupuesto real (ADR-008). (T-01)
- **T-24** Revisión de bocetos `05-wireframes.md` y cierre de sus 3 pendientes.

## Fase 1 – Núcleo (sin juego)

- **T-03** Workspace + CI local (`fmt`, `clippy`, `test`). *Hecho:* `cargo test` verde. **(Duplicado de la T-03 de la Fase 0b: la parte de workspace está hecha y la de CI es T-03a.)**
- **T-04 ✔ (2026-10-31)** `core::model` y deserialización serde de fixtures reales del match y de objetivos. *Hecho:* 11 tests contra `crates/api/tests/fixtures/` (match `2-2`, objetivos, mapas). (T-03)
- **T-05 ✔ (2026-10-31)** `match_resolver`: equipo → color propio/enemigos buscando el `team_id` en `all_worlds`; nombres desde `data/wvw-teams.json` embebido (`include_str!`). *Hecho:* 6 tests con equipo, mundo enlazado, id desconocido, tabla corrupta e id sin nombre. (T-04) — RF-03a, RF-03b, RF-03g, RF-03i
- **T-06 ✔ (2026-10-31)** Baseline y cambio de match en `diff`: `Snapshot`, `Differ` con `Event::{Baseline, MatchChanged, Gap}`. *Hecho:* 8 tests (baseline de sesión, cambio de match, hueco > 3×intervalo y su borde exacto). (T-04) — RF-03d, RF-04, RF-26
- **T-07 ✔ (2026-10-31)** `diff` de flips y claims/unclaims con orden determinista. *Hecho:* `Event::{Flip, Claim, Unclaim}`; recorre los objetivos por id y emite `Unclaim` antes de `Claim`; el baseline no emite ninguno de los tres. 6 tests. (T-06) — RF-10, RF-21, RF-22
- **T-08 ✔ (2026-10-31)** `rules`: ámbito, equipo propio, cooldown, guilds seguidas. (T-07) — RF-12, RF-13
- **T-09 ✔ (2026-10-31)** Claims tardíos y `Gap`: `Event::Claim` lleva `late`; un baseline (sesión, cambio de match o hueco) registra **todos** los claims vigentes con su `claimed_at` real y `late=true` (RF-25). El `Gap` se emite antes del baseline. (T-07) — RF-25, RF-26
- **T-10 ✔ (2026-10-31, parcial)** `store` JSON-lines: append, lectura, línea corrupta, rotación y descarte de la sesión anterior (ADR-012). 6 tests. Falta la consulta por guild/rango, que es de T-11. (T-04) — RF-24, RNF-03
- **T-11 ✔ (2026-10-31)** `follow` + `history`: estado actual por guild y consulta por rango. (T-09, T-10) — RF-23

## Fase 2 – API y worker

- **T-11b ✔ (2026-10-31)** `api`: `trait Transport` + `Api<T>` tipado (`matches_all`, `match_for_team`, `objectives(lang)`, `guild`), `ApiError` y `Backoff` exponencial con tope. 9 tests con fixtures reales, incluidos 503, JSON truncado y equipo inexistente. La caché en disco (`cache::Cache`, objetivos por idioma + guilds; 4 tests) se retiró el 2026-11-01: no tenía consumidores y contradecía el diseño actual, donde estos catálogos se cachean en memoria por sesión. Cliente HTTP real en el addon (`net::WinHttpTransport` sobre WinHTTP del sistema, medidas en `07-spikes.md`: HTTP 200 en 270 ms sin crecer la DLL). (T-04) — RF-20, RF-44, RNF-04
- **T-11c ✔ (2026-10-31, parcial)** `worker`: hilo propio con parada limpia (`stop()` + sondeo
  de 250 ms), `MAY_POLL` desde la compuerta del render (nunca red ni disco en el hilo de render),
  `ids=all` mientras no hay equipo y `/wvw/matches/{id}` (nuevo `Api::match_by_id`) cuando lo
  hay, backoff hasta 5 min, `Health::{Idle,Ok,Degraded}` e histórico del ciclo en memoria en
  `Dashboard` (claims, clanes y nombres) que la ventana de estado ya lee. El monitor de
  `teamAssignment` reinicia ese ciclo con un baseline al confirmar una reasignación.
  Probado de verdad con `live_pipeline_builds_the_dashboard` (HTTP real, 1,2 s). Falta: dejar de
  escribir `settings.json` en mitad de un poll largo y separar el sondeo de objetivos del de
  estado. (T-11b, T-07) — RF-20, RF-26, RNF-04

## Fase 3 – Integración Nexus

- **T-31 ✔ (2026-10-31, parcial)** `i18n`: tabla es/en/fr/de en `core::i18n` (36 claves),
  funciones `t`/`w`/`f` con `###clave` para ids estables de imgui, fallback a inglés y mapa de
  `GameLanguage` verificado en `nexus-rs` (0=en, 1=coreano, 2=fr, 3=de, 4=es, 5=chino).
  `addon::i18n` resuelve preferencia → idioma del juego → inglés, aplica el override de Ctrl
  derecho (`ffi::keys`, `GetAsyncKeyState` + proceso en primer plano) y registra las claves en
  Nexus con `set_translation`. Tests: 9 en `core` (incluida la completitud de las cuatro
  columnas), 3 en el addon. La prueba previa en juego T-35 se aceptó y la revisión por nativos
  T-41 se deprecó; se mantienen los cuatro idiomas. (T-03) — RF-44, RF-45, RNF-08

- **T-12 ✔ (2026-10-31, parcial)** `context` + `config`: activación solo en WvW, persistencia y export real. Hecho: `export!` con `Direct` + `DisableHotloading` + URL de distribución (verificada dentro de la DLL), lectura de `NexusLink::is_gameplay` y del mapa de Mumble en `ffi/`, `config.json` con valores por defecto, saneo y copia `.corrupt-*` ante JSON dañado (5 tests), y ventana de estado mínima que respeta `core::gate`. Falta: selector de equipo (T-30), ventana de configuración (T-17) y la prueba dentro del juego (T-35). (T-01, T-05) — RF-01, RF-02, RF-02a, RF-03a/c, RF-40, RF-41

- **T-30 ✔ (2026-10-31, parcial)** `addon::ui::settings`: selector en **tres pasos** — región
  (EU/NA, solo las que tienen matchups vivos) → color (rojo/verde/azul, con su color real en la
  lista) → equipo, con `nombre (id) · matchup`. Con los datos del 2026-10-31 son 5 filas en EU y
  4 en NA, en vez de 27. El desplegable arranca en la región y color del equipo ya guardado. Si
  no hay primer sondeo, lo dice en vez de mostrar una lista vacía; si la combinación no tiene
  equipos, también. Alias editable con aviso de que es local, y campo de idioma, intervalo y
  alcance. Falta comprobar los dos rivales contra el matchup y el resto de la ventana. (T-05)
  — RF-03a, RF-03f, RF-03g
- **T-13 ✔ (2026-10-31, parcial)** Widget de avisos con **filtro de origen** (RF-19a):
  casillas para Eternos, Alpino azul, Alpino verde, Desierto rojo, «el mapa en el que estoy» y
  «los clanes que sigo», todas marcadas por defecto. Un clan seguido avisa en **cualquier** mapa
  del emparejamiento; el mapa actual se recalcula con el `map_id` de Mumble (38/95/96/1099). El
  **pop-up** dura 0–5 s (2 s por defecto, `popup_secs`), se dibuja arriba en el centro y
  desaparece sin quitar nada de la lista; los claims tardíos del baseline no hacen pop-up. La
  lista de clanes a seguir sale de los que tienen algo reclamado en el matchup. (T-08, T-12)
  — RF-10, RF-16, RF-17, RF-19, RF-19a, RF-73
- **T-13b ✔ (2026-10-31)** Modo bloqueado click-through: una ventana bloqueada usa
  `WindowFlags::NO_INPUTS`, así que sigue dibujándose pero todos los clics llegan al juego.
  `Alt+Shift+L` (keybind de Nexus, reasignable allí) bloquea o desbloquea **todas** las ventanas;
  el handler solo levanta un `AtomicBool` y el render lo aplica, para no coger el candado de la
  configuración desde el hilo de entrada. Los ajustes tienen una casilla de click-through por
  ventana y una **ventana de registro de avisos** con la lista completa y scroll. (T-13)
  — RF-14, RF-18
- **T-42 ✔ (2026-10-31)** Una sola ventana con pestañas, como `docs/boceto.html` (ADR-014):
  **Avisos · Mapa · Clanes · Registro** y la **última, Ajustes** (equipo, idioma, intervalo,
  filtros de aviso y clanes seguidos dentro de la propia ventana). Un único `WindowState` para
  toda la ventana. El **pop-up** se activa y desactiva desde ahí (`popup_enabled` + `popup_secs`)
  y desde las opciones de Nexus, que además ofrecen **Ver la ventana**; el pop-up se dibuja
  aunque la ventana esté cerrada. (T-13, T-17) — RF-14, RF-16, RF-19a, RF-46
  - **Ampliación ✔ (2026-11-02), RF-03a:** sin equipo en la cuenta actual, los cuerpos de Avisos,
    Mapa, Clanes y Registro muestran el aviso de elegir equipo en Ajustes; Ajustes queda operativo.
    No se enseñan estadísticas ni avisos de navegación de la partida anterior. Regla compartida
    con dos tests puros (todas las pestañas y cuenta distinta); texto es/en/fr/de.
- **T-43 ✔ (2026-10-31)** Enseñar en la lista de clanes el **nombre en inglés y la traducción** cuando exista.
  Falta decidir el origen de la traducción: el juego no traduce nombres de clan, así que solo puede
  salir de una tabla propia (como `data/team-names.json` para equipos) o de un campo que escriba el
  usuario, reutilizando el patrón de `team_translations`. (T-37)
- **T-44 ✔ (2026-10-31)** Selector de clanes a seguir rediseñado (pedido del usuario, 2026-10-31). Hoy la lista de
  «clanes seguidos» de los ajustes vuelca **todos** los clanes del matchup, así que crece sin freno y
  acabará pesando. Debe quedar así:
  1. La sección enseña **solo los clanes que ya sigo**, con su `[TAG] Nombre` y un botón para dejar
     de seguir cada uno. **Máximo 10** (`MAX_FOLLOWED`, ya aplicado en 0.14.4).
  2. Botón **Añadir** con dos vías:
     - **Los que han reclamado algo desde que arrancó el addon** (los que tienen «logs», es decir,
       los que aparecen en el historial de la sesión): un clic lo añade a seguidos.
     - **Buscador**: obligatorio indicar **el equipo** (y que sea de la región en la que estoy, EU o
       NA), escribir parte del nombre y buscar.
  Verificado contra la API el 2026-10-31 y **no hace falta inventar nada**:
  - `GET /v2/guild/search?name=<texto>` → lista de ids de clan (sin clave, ya probado: `name=lord`
    devolvió un id). Con esos ids, `/v2/guild/:id` da `name` y `tag`, y `/v2/guild/:id/log` no vale
    porque **pide clave**.
  - `GET /v2/wvw/guilds/:region` (`eu`/`na`) → `{ "<guild_id>": "<team_id>" }` (282 KB para EU):
    es la forma de saber en qué equipo está un clan y de filtrar por el equipo elegido.
  - La región sale de la pertenencia del equipo ya configurado (`1-x` NA, `2-x` EU), que es lo que
    hace que «obligatorio que sea de mi zona» no necesite configuración extra.
  Rendimiento: la búsqueda se lanza a mano (nunca por frame), con resultados acotados y cacheados
  como los nombres de clan (`catalog::resolve_guild_names`). (T-37, T-13) — RF-73, RF-31
- **T-45 ✔ (2026-10-31)** Nombres de equipo decentes. `data/team-names.json` se rehace desde las
  **cuatro páginas de gw2mists** (`/en/teams/eu`, `/de/teams/eu`, `/en/teams/na`, `/de/teams/na`):
  - El acceso se resolvió con el **navegador integrado** de VS Code, que ejecuta el JavaScript de la
    SPA; `curl` sigue viendo una página vacía y `api.gw2mists.com` sigue dando 403 a los scripts
    (ver `07-spikes.md`). Las cuatro tablas se emparejan por fila (alianzas, gremios, jugadores,
    comandantes) porque el orden cambia con el idioma.
  - **Inglés**: el de `data/wvw-teams.json` (33 ids de `all_worlds`). **Alemán**: el de la web, que
    no es el del Excel (`Grenths Tor` → `Grenths Tür`, `Ettin-Rücken` → `Ettinbuckel`…).
    **Español y francés**: los del Excel cuando el nombre está en la hoja; si no, el inglés, para que
    el jugador los traduzca en los ajustes. La web solo lista los 27 equipos en juego, así que los 6
    restantes se quedan en inglés.
  - Las traducciones propias del jugador (`team_translations`) siguen mandando sobre la tabla.
  - Tests: `rf44_german_comes_from_gw2mists_and_not_from_the_spreadsheet` y
    `rf44_the_pool_covers_every_team_in_use`. (T-30) — RF-03f
    (ya lo hizo una vez con un Excel). **Bloqueada por eso.** (T-30) — RF-03f
- **T-46 ✔ (2026-11-01)** Cuatro peticiones del jugador en una tanda:
  1. **Avisos solo de estructuras reclamables** (RF-19e): campamentos, torres, fortalezas/garris y
     castillos, con casilla por tipo. Las ruinas y los mercenarios dejan de avisar.
  2. **El índice de nombres no se lee todo el rato** (RF-20b): los nombres resueltos se guardan en
     `settings.json` (`guild_names`) y `guild-names.json` pasa a ser solo el índice de búsqueda, que
     se lee en el hilo del buscador y se suelta (ADR-016).
  3. **El índice se distribuye** (RF-20c) como `gw2_lurker_guilds.zip` + `.md5` en el repositorio
     compartido, con descarga la primera vez y a petición (`scripts/pack-guild-index.ps1`).
  4. **Al añadir un clan que no está en el índice** se resuelve su nombre y se añade (RF-20d).
  Además: fuera la hoja `data/equipos.xlsx` y su script; `data/team-names.json` se edita a mano.
  (T-44, T-13) — RF-19e, RF-20b, RF-20c, RF-20d
- **T-17 ✔ (2026-10-31, parcial)** Contrato de ventana compartido en `addon::ui::window`:
  barra de título propia con `×`, plegado con doble clic, mover/redimensionar solo si no está
  bloqueada, opacidad sobre el fondo, menú contextual (bloquear, opacidad, restablecer) y
  estado por ventana (`visible`, `collapsed`, `locked`, `opacity`, `position`, `size`) en
  `config.json` bajo `windows.<clave>`, saneado y con mínimo de tamaño. La ventana de estado ya
  lo usa. Contenido de ajustes hecho: equipo + alias (T-30), idioma, intervalo de sondeo
  (15–600 s, saneado al guardar) y alcance de los avisos. Falta el indicador de obsoleto y el
  resto de opciones de sonido/atajos. (T-12) — RF-43, RF-46

## Fase 4 – Mapas

- **T-14 ✔ (2026-10-31)** Proyección `world_to_screen` y mapa simplificado: `core::project`
  (`Frame::of`, `normalise`, `to_screen`, `project`) con tests contra las fixtures reales, `Api::maps`
  cacheado una vez por sesión y `ui::map` dibujando objetivos con `DrawList` (forma por tipo, color
  por equipo, contorno doble en alto valor, anillo en guilds seguidas) más selector de mapa.
  Verificado que `coord` está en coordenadas de continente y que `points_tick` no viene en
  `/v2/wvw/objectives`; el «alto valor» sale del `points_tick` del snapshot en `ClaimRow`
  (RF-30b, ver 0.13.2). (T-04, T-12) — RF-30
- **T-16 (parcial) ✔ (2026-10-31)** La ruta del clan elegido se dibuja sobre el mapa: nodos numerados en orden
  temporal, línea entre ellos, gris si el claim no trae hora, y anillo en el más reciente (RF-32).
  Falta el selector de rango temporal, el trazo discontinuo para `late`/`gap` (RF-33) y el tooltip
  por nodo (RF-34). (T-15) — RF-32, RF-33, RF-34
- **T-15 ✔ (2026-10-31, parcial)** Capa de claims sobre el mapa: los objetivos de las guilds seguidas
  llevan anillo dorado **y su tag** (RF-31), y el hover enseña dueño, alto valor, clan y hora. Falta
  la vista de estado por guild (RF-23). (T-14, T-11) — RF-31, RF-23
- **T-16 ✔ (2026-11-02)** Vista Histórico: ruta simulada numerada, flechas, selector de rango, banner de "inferida", estilo para `late`/`gap`, tooltip. Completada con T-52/T-53. (T-15) — RF-32, RF-33, RF-34

- **T-26 ✔ (2026-10-31)** Modo seguro: un `settings.json` ilegible se renombra a
  `settings.json.corrupt-<fecha>`, el addon sigue con los valores por defecto y **lo dice en rojo**
  encima de la ventana (`lib::safe_mode`, aviso permanente durante la sesión). El registro de la
  sesión ya toleraba líneas corruptas (`adr003_corrupt_line_is_skipped_and_counted`). (T-10, T-12)
  — RF-51, RNF-06
- **T-47 ✔ (2026-10-31)** **Icono del tipo de objetivo en los avisos**: cada línea del widget y del pop-up
  lleva delante el dibujo del objetivo (campamento, torre, fortaleza o piedraniebla) en el color de la
  línea; los objetivos que no son de esos cuatro (ruinas, spawns) no llevan dibujo y no reservan hueco.
  El espacio se reserva con un `dummy` y el dibujo se pinta en **una sola pasada** con la lista tomada una
  vez (AGENTS §11.12). Módulo `ui::icons`, compartido con T-46. (T-13) — RF-19, RF-38

- **T-29 ✔ (2026-10-31, parcial)** Fondo con las **teselas oficiales**: `tiles.guildwars2.com`, **continente 2, piso 1**, a zoom 5 (**1024 unidades por tesela**; 4 teselas por lado y mapa, 59 en total para los cuatro), descargadas por el worker a `<addon>/tiles/` y dibujadas con `add_image` heredando la **opacidad de la ventana**. Las que aún no están en disco no se dibujan y se van rellenando. Interruptor **Mapa real** en los ajustes. Los iconos van en T-46. (T-14) — RF-37, RF-38

- **T-46 ✔ (2026-10-31, sin las capturas)** Iconos del mapa como en el juego: círculo del color del equipo
  con el dibujo **dentro**, dibujado con primitivas (`ui::icons`), porque las capturas del usuario (tienda y
  torre) no están disponibles como ficheros en el repo y el dibujo no depende de un asset que pueda faltar.
  Los cuatro que puntúan tienen dibujo propio (tienda, torre, fortaleza con tres almenas y castillo con dos
  torres), más el tamaño por tipo; el de piedraniebla se dibuja más ancho que la fortaleza para distinguirse.
  Pendiente solo si el usuario aporta los PNG reales: cambiarlos es sustituir `icons::parts`. (T-29) —
  RF-30, RF-31, RF-38

- **T-48 ✔ (2026-10-31)** **El widget también en el Bastión del Armisticio**: `map_type` 18 no tiene
  objetivos que vigilar, pero es donde el jugador quiere tocar los ajustes. `Gate::CONFIG_ONLY` deja
  `show_ui` en alto y `poll` en bajo (no se gasta red), la lista de equipos se sigue pidiendo y la
  cabecera lo explica («en este mapa no hay nada que vigilar, pero los ajustes funcionan»). La Fortaleza
  del Valor y el Sagrario de Obsidiana siguen ocultos: son una recompensa y un puzle, no un mapa de MvM
  al que uno vaya a configurar. Cambia el GWT de RF-01 (lo pidió el usuario). — RF-01, RF-02

- **T-49 ✔ (2026-10-31)** **El clic derecho saca del click-through**: con `NO_INPUTS` la ventana no recibe
  nada del ratón, así que la única salida era el atajo o el panel de Nexus. El callback de render mira el
  ratón por su cuenta (`ui::right_click_on_widget`, solo en el flanco de bajada) y, si el clic derecho cae
  dentro de la caja guardada de la ventana (`window::covers`, pura y con test), apaga el click-through y
  guarda. El aviso naranja del widget lo dice y usa el atajo real (`LOCK_SHORTCUT`), que estaba escrito a
  mano con el `Alt+Shift+L` viejo. — RF-02, RF-18

- **T-50 ✔ (2026-10-31)** **Clanes en su pestaña, horas locales y registro limpio**: `follow::body` se
  parte en `followed_section` y `add_section` y las dibuja la pestaña *Clanes* (los ajustes solo dicen
  dónde están); la lista de «han reclamado» se corta en **10**; el buscador empieza por la **región** y
  los equipos se enseñan con nombre; un id sin forma de id de clan se descarta antes de pedirlo; la
  barra de título entera responde al ratón (doble clic contrae/despliega, un clic abre si está contrada);
  y `addon::clock` pinta todas las horas en la hora local. Un `late` solo avisa si es de un clan seguido
  y de los últimos 10 minutos. (T-13, T-44) — RF-19, RF-21, RF-25, RF-44, RF-46

## Fase 4b – Actualización y fallo seguro

- **T-25 ✔ (2026-10-31, parcial)** Spike y aviso de actualización: comprobar cómo deja Nexus la DLL nueva y construir el banner W7 (ADR-007). (T-01, T-12) — RF-50, RF-53
- **T-26** Modo seguro: config/historial corruptos → `.corrupt`, valores por defecto, aviso W8; `catch_unwind` en la frontera FFI. (T-10, T-12) — RF-51, RNF-06
- **T-27 ✔ (2026-10-31)** Pipeline de release: DLL en CI, SHA-256, test de humo PE/export con crate `object`, registro de tamaño en CHANGELOG. (T-03a, T-02b) — RNF-09, RNF-12

## Fase 5 – Aceptación y pulido

- **T-18 ✔ (2026-10-31)** Medición de frame time con mapa abierto. *Hecho:* < 0.5 ms (RNF-01).
- **T-19 ✔ (2026-11-02)** Checklist previa en juego aceptada por indicación expresa del jugador.
  No implica una medición nueva por la IA ni la aceptación del nuevo seguimiento B (MT-21).
- **T-20 ✔ (2026-10-31, parcial)** README, CHANGELOG, `docs/` de usuario, instalación, limitaciones conocidas (RNF-05), empaquetado de la DLL.
- **T-28 ✔ (2026-10-31)** Módulo `theme`: herencia del estilo de Nexus, tema propio opcional aplicado solo a ventanas del addon (stack de estilo push/pop), colores de equipo fijos y fondo del widget configurable. (T-13, T-17) — RF-60..63
- **T-21 (S) — deprecada (2026-11-02)** El jugador retira la ampliación pendiente de RF-11 y
  RF-15. Las partes previamente implementadas de RF-19b, RF-19c, RF-35 y Quick Access se mantienen.
- **T-22 (C) ✔ (2026-10-31, parcial)** RF-19d, RF-36, reproducción animada de la ruta.
  Ampliación B — Estela completada en T-51..T-53 (0.26.6). Audio retirado por el jugador.

## Fase 6 – MVP de validación (backlog priorizado)

Orden pensado para llegar cuanto antes a "lo veo en el juego y me sirve para decidir". Cada
tarea indica de qué depende.

1. **T-32 (M)** Bocetos y `docs/boceto.html` navegable para validar la UI antes de escribirla.
   *Hecho:* el HTML enseña widget, mapa, lista de guilds e historial con datos reales de
   ejemplo, y se puede abrir sin el juego. Requiere `PRODUCT.md` (flujo `impeccable init`).
   — RF-16, RF-30, RF-70..73
2. **T-33 (M) ✔ (2026-11-02)** No dibujar nada ni pollear en carga, selección/creación de personaje: señal
   `NexusLink::is_gameplay` con respaldo por `map_type`. (T-12) — RF-02, RF-02a
   - Integrado en `ffi/context` y el gate de UI/worker; prueba previa aceptada por el jugador.
3. **T-34 ✔ (2026-10-31)** Publicar la DLL en `..\gw2addonsreleases` siguiendo `docs/RELEASE.md`
   (MD5, excepción LFS, tag `gw2-lurker-v*`, retención actual+anterior). Automatizado en
   `scripts/release.ps1` (`-DryRun` / normal / `-VerifyOnly`), que aborta en vez de publicar
   un puntero LFS, un `.md5` que no mida 32 bytes o una descarga que no coincida. El fichero
   pasa a llamarse `gw2_lurker.dll` por el `[lib] name` del addon. (T-27) — RF-53
4. **T-35 (M) ✔ (2026-11-02)** Arrancar el addon dentro del juego con este repo: `export!` real, versión
   visible y traza en `addons/Nexus/Nexus.log`. Es la T-01 ejecutada de verdad. (T-33, T-34)
5. **T-36 ✔ (2026-10-31)** Vista **"Claims del matchup"**: los cuatro mapas agrupados y
   plegables, cada fila `hora · objetivo · clan` con el color del dueño y el mapa actual marcado
   con «estás aquí». Filtros de sesión, todos activos al empezar: campamentos, torres,
   fortalezas/castillos y «solo alto valor». (T-11, T-15) — RF-70
6. **T-37 ✔ (2026-10-31)** Lista de clanes con claim ordenada por la más reciente, con número
   de claims, hora y mapa de la última; estrella para **seguir** el clan, que es lo que hace que
   avise en cualquier mapa del matchup. (T-36) — RF-71
7. **T-38 ✔ (2026-10-31)** Historial por clan al seleccionarlo, **separado entre el mapa en el que
   estás y los demás**, en orden cronológico. Se alimenta de los `Claim` de la sesión
   (`Dashboard::history`, tope 2000; el `store` guarda todo). (T-16, T-37) — RF-72
8. **T-39 ✔ (2026-10-31)** Favoritos: la propia lista de clanes tiene la estrella de
   seguir/dejar de seguir y se guarda en `settings.json` (`followed`), además de en los ajustes.
   `FollowList` ya impedía duplicados y `favourites_first` mantiene el orden por recencia.
   (T-37) — RF-73
9. **T-40 ✔ (2026-11-01)** `core::route::with_origin` y `ui::map::route`: origen localizado **0**
   (garri propia → fortaleza lateral alpina válida → respawn del equipo del clan), con
   condiciones temporales/geográficas, coordenadas oficiales y aviso si son aproximadas o faltan.
   Marcador hueco, enlace discontinuo 0→1 y 0/1 si coinciden; no se inventa un evento ni una hora.
   Texto de rutas entre mapas junto al dibujo, con «salto a <mapa>» y aviso de inferencia (ADR-005).
   Tests con fixtures reales y medición CPU manual; el frame completo requiere GW2.
   (T-38) — RF-32a
10. **T-41 (S) — deprecada (2026-11-02)** Se retira la revisión pendiente de terminología fr/de
    con hablantes nativos; las traducciones existentes y RF-44/RNF-08 se mantienen. (T-31)

## Fase 7 – B: Estela (implementada en 0.26.6; prueba nueva en juego MT-21)

- **T-51 ✔ (2026-11-02)** Modelo puro y tests primero: pasos/intervalos, selección temporal, ventanas inclusivas
  Todo/1 h/15 min, revisitas, datos incompletos y marcas de huecos ≥20 min. Conservar origen 0,
  orden estable, `late`, `gap` y saltos de mapa; sin red ni persistencia del historial.
  (T-16, T-40) — RF-24, RF-32, RF-32a, RF-33, RF-35, RF-36; H-15..H-20, H-23..H-25, H-27.
  Implementado en `core::route::trail`: observaciones prestadas, índices estables, grupos de
  revisitas y pares consecutivos incluso sin posición. Conserva `late`/`gap` de eventos
  proporcionados, sin deducirlos de la separación entre claims; el origen sigue en `with_origin`.
  11 tests nuevos (incluidos datos de coordenadas de la fixture oficial).
  Solo modelo en esta tarea; dibujo e interacción completados posteriormente en T-52/T-53.
- **T-52 ✔ (2026-11-02)** Dibujar B en Mapa: curvas alternadas con flechas y recencia por grosor/opacidad,
  marcadores de revisitas, paso elegido y posteriores, anillos de intervalo, leyenda y aviso
  de inferencia. Mantener el comportamiento sin clan seleccionado y las capas actuales.
  (T-51) — RF-30, RF-31, RF-32, RF-32a, RF-33, RF-35; H-15, H-16, H-20, H-23..H-25, H-27.
- **T-53 ✔ (2026-11-02)** Cronología interactiva e i18n: clic/arrastre/teclado, ventana Todo por defecto,
  play/pausa a 700 ms por claim, parada al final, pausa al salir de Mapa y al cambiar clan.
  Tests con reloj controlado y aceptación de los seis escenarios del boceto B. Medir coste
  del render (RNF-01), validar en el juego y actualizar docs/CHANGELOG antes de cerrar;
  no dar por verificado el rendimiento por el HTML.
  (T-51, T-52) — RF-36, RF-44, RNF-01; H-16..H-19, H-21, H-22, H-24..H-27.
  Tests con fixtures para tres casos normales y tres extremos, reloj controlado, pausa oculta,
  refresco sin perder selección, reutilización de geometría y retención/reset de anotaciones.
  Medición optimizada con el draw list real de imgui y preparación de geometría; no sustituye
  el frame completo de GW2. MT-21 queda para probar esta versión, como pide el jugador.

## Fase 8 – Revisión de UX (aprobada el 2026-11-02, sin implementar)

Bocetos en `docs/bocetos/ux/propuestas.html`. Cada tarea va sola y con tests primero.

- **T-54 (M)** Línea de aviso desde mi equipo: verbo PERDIDO/TOMADO/cambio/reclamo, ALTO por
  `points_tick` ≥ 8, antigüedad relativa (`hh:mm` desde 60 min) y nombres de mapa únicos.
  Función pura en `core` que clasifica la alerta con el equipo propio; tests `rf19f_…`.
  — RF-19f, RF-30b
- **T-55 (L)** HUD de combate y consola: segunda superficie con su `WindowState`, línea de estado
  con API parada, 2–5 avisos, interruptor **Ver el HUD** en Nexus; la consola pierde la pestaña
  Avisos y pliega la cabecera. (T-54) — RF-47, RF-46, RF-43, ADR-017
- **T-56 (M)** Emergente en pila: una línea por aviso, cada una dura `popup_secs` desde que entra,
  máximo 5 y «y N más», prioridad clan seguido > alto valor > perdido > tomado, `NO_INPUTS` y
  fundido del texto. Modelo puro de la pila con reloj controlado; tests `rf19d_…`. (T-54) — RF-19d
- **T-57 (M)** Mapa: tira de cuatro mapas con `G4 A3 R5` y última novedad, leyenda fija, controles
  de ruta bajo el tablero y dueño en texto en «Claims del matchup». — RF-30c, RF-70
- **T-58 (M)** Ajustes: bloque «Qué me avisa» con frase de resumen (función pura, tests
  `rf40a_…`) y el resto en secciones plegables, con la URL del índice en Avanzado. — RF-40a, RF-19a
- **T-59 (M)** Selector de equipo: guardar solo al confirmar, propuesta de un clic con el equipo de
  arcdps y lista agrupada por emparejamiento; unificar los colores de equipo con `ui::theme`.
  — RF-03j
