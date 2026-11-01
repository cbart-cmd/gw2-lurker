# 01 – Requisitos

Formato EARS + criterios de aceptación Given/When/Then (GWT). Prioridad: **M** = must (v1), **S** = should, **C** = could.

## Hechos verificados que condicionan los requisitos

Comprobado el **2026-10-31** contra la API real, el código fuente de `nexus-rs` y una medición propia (`07-spikes.md`, sección *Resultados*). `[API]` = respuesta real; `[src]` = código fuente citado; `[medido]` = experimento.

**Datos de WvW**

- `[API]` `/v2/wvw/matches?world=<id>` devuelve el match con todos los objetivos. Por objetivo: `id`, `type`, `owner`, `last_flipped`, `claimed_by` (ID de guild), `claimed_at`, `points_tick`, `points_capture`, `yaks_delivered`, `guild_upgrades`. **Las claves siempre están presentes**; sin claim valen `null`. **El claim es único**: `claimed_by` es una cadena, no una lista (verificado en el match `2-2`: 41 de 91 objetivos con claim, 50 con `null`). La API no expone la ventana de reclamación ni qué otras guilds lo intentaron.
- `[API]` **Los equipos sustituyen a los mundos.** El match trae `worlds` (mundo heredado por color) y `all_worlds`, que **incluye los ids de equipo**: 11001–11015 (NA) y 12001–12018 (EU). Un jugador hoy se identifica por equipo (p. ej. `12014` "Great House Aviary"), no por mundo.
- `[API]` **La API no expone nombres de equipo ni de alianza**: `/v2/wvw/teams` y `/v2/wvw/alliances` devuelven 404 y el índice de `/v2` no lista nada equivalente. Los nombres solo están en fuentes de terceros ⇒ tabla propia versionada (`data/wvw-teams.json`, ADR-010).
- `[API]` El prefijo del id de match indica región: **`1-x` = NA, `2-x` = EU**.
- `[API]` `/v2/wvw/timers/teamAssignment` y `/v2/wvw/timers/lockout` dan las fechas de reasignación de equipos y de bloqueo; no hay que cablear el calendario.
- `[API]` `/v2/wvw/matches/overview` devuelve solo la lista de ids de match en curso (83 bytes): endpoint barato para detectar el cambio de match.
- `[API]` `/v2/wvw/guilds/eu` y `/na` devuelven `{ guild_id: team_id }` (282 KB para toda EU): es la fuente oficial guild → equipo. `/v2/guild/:id` no lo informa.
- `[API]` `/v2/wvw/objectives?ids=all`: `coord` está en **coordenadas de continente** y **81 de 178** entradas tienen `coord: null`. `/v2/maps` aporta `continent_rect` (caja real) y `map_rect` (ADR-011).
- `[API]` `Cache-Control: public, max-age=1`, sin cabecera `Age`, `X-Rate-Limit-Limit: 600`. **Límite real** (wiki *API:Best practices*, copia de archive.org del 2025-12-16): cubo de **300** peticiones por IP que se recarga a **5 por segundo** (300/min). La cifra 600 de la cabecera **no** es «600 por hora»; superar el cubo devuelve `429`.
- `[medido]` Con polling a 10 s: los cambios de **propiedad de objetivos** tienen mediana 19–20 s (máx 110 s observado) y hay emparejamientos muertos con **22 min sin un solo cambio**. 227 peticiones, 0 errores. Conclusión inicial: 30 s. **Decisión posterior (2026-10-31): 5 s por defecto** —el jugador pidió acercarse a gw2mists y el sondeo pide **una** partida por id, no los nueve—, con recorte por backoff (RNF-02).
- `[medido]` **La API congela una partida entera**: el 2026-11-01 la partida `1-1` devolvió 20 minutos la misma copia, **puntos incluidos**, mientras las otras ocho se movían. No es CDN (`?cb=` da lo mismo) ni el cliente HTTP: es el backend. Por eso el widget avisa en naranja cuando la huella de la partida no cambia en ≥5 min, y no se puede confundir «no pasa nada» con «el addon está parado».
- `[medido]` **El atlas de fuentes de Nexus solo trae latín básico y latino-1**: `·` (U+00B7) y `×` (U+00D7) se dibujan; `→` (U+2192), `…` (U+2026) y `★` (U+2605) salen como «?». Todo texto de la UI se queda en ASCII + latino-1 (test `rf44_the_table_stays_inside_latin_1`). Los nombres de clan que llegan del API sí pueden traer otros alfabetos: esos no tienen glifo.
- `[API]` `/v2/guild/search?name=` devuelve **un solo** clan y solo si el texto **empieza** su nombre (`die` → *Die*; `a ne` → nada). No hay búsqueda por subcadena ni listado masivo (`/v2/guild?ids=` → 404). De ahí el índice propio (`guild-names.json`, 9 886 nombres) que el addon distribuye y consulta al buscar.
- La API **no** expone estado "bajo ataque" (ADR-001) ni historial: solo el último claim vigente.
- La API a veces se queda atascada o devuelve 503/"unknown error" durante minutos u horas (reportado por la comunidad). Hay que tolerarlo (RNF-04).
- `[API]` `/v2/account/wvw` existe y requiere API key: permitiría leer el equipo de la cuenta, pero ADR-010 descarta pedir clave.

**Mumble / Nexus**

- `[src]` `Identity::world_id` es un *shard id* y está documentado como **no usable** desde el megaservidor: **no** identifica el equipo (`gw2-mumble-rs`, `src/identity.rs`).
- `[src]` Sí son fiables `Context::map_id`, `Context::map_type`, `Context::shard_id` y `UiState::IS_IN_COMPETITIVE_MODE`. `map_type` vale 9 (EBG), 10/11/12 (fronteras), 14 (Obsidian Sanctum), 15 (EotM), 13 (Fortune's Vale) y **18 (Armisticio)** (`gw2-mumble-rs`, `src/map_type.rs`).
- `[src]` `Context::player_x/y` y `map_center_x/y` están documentados como *"not updated in competitive modes"*: no se puede contar con la posición del jugador sin verificar en WvW.
- `[src]` `nexus-rs` **0.12.0** (`rust-version = 1.87.0`, ImGui vía `arcdps-imgui` 0.8.0). Logs en `<juego>/addons/Nexus/Nexus.log` y datos del addon en `<juego>/addons/<nombre>/`.
- `[src]` `AddonFlags::DisableHotloading` implica que la actualización exige reiniciar (ADR-007).
- Hay prior art con los mismos endpoints (Realm Report, C++/Nexus). Sirve de referencia, no de dependencia.

## A. Activación y contexto

**RF-01 (M)** Cuando el MumbleLink indique que el jugador está en un mapa de WvW, el addon deberá activar polling y UI. La detección se hará por **`Context::map_type`** ∈ {9 EBG, 10 frontera azul, 11 verde, 12 roja, 15 EotM}, no por una lista de `map_id`, para no romperse si ArenaNet añade mapas.
- GWT: *Given* el jugador está en Ojos de Aguas Eternas (`map_type` 9), *When* el addon recibe el contexto por Mumble, *Then* arranca el polling en ≤ 1 intervalo.
- GWT *(cambio del 2026-10-31, a petición del usuario)*: *Given* el jugador entra en el Bastión del Armisticio (`map_type` 18, `map_id` 1315), *When* el addon lee el contexto, *Then* **dibuja la ventana** (es donde se configuran los ajustes) y **no** activa el polling. La Fortaleza del Valor (13) y el Sagrario de Obsidiana (14) sí se siguen ocultando.

**RF-02 (M)** Cuando el jugador salga de un mapa de WvW, el addon deberá detener el polling y ocultar toda su UI (excepto la ventana de configuración si está abierta).
- GWT: *Given* el polling activo, *When* el jugador viaja a una ciudad, *Then* no se emiten peticiones de red y no se dibuja nada.

**RF-03a (M)** Si no hay equipo configurado, el addon deberá pedir al usuario que elija su **equipo** (no su mundo) de la lista de equipos del match en curso, obtenida de `all_worlds` de `/v2/wvw/matches?ids=all` (ids 11xxx/12xxx). Se mostrará siempre `nombre (id)`, leyendo el nombre de la tabla versionada `data/wvw-teams.json`, porque la API oficial no expone nombres de equipo. Si un id no está en la tabla, se mostrará solo el id y se avisará de que falta el nombre (ADR-010).
- **Selector en dos pasos (2026-10-31):** región (EU/NA, deducida del prefijo del id: `2-x`/`1-x`) → matchup (`2-1`…`2-5`) → uno de sus **tres** equipos, con su color. Se ofrecen solo ids de equipo: `all_worlds` mezcla ids de mundo antiguos (`2xxx`) que nadie puede elegir.
- **No se puede deducir el color del jugador:** Mumble solo expone `server_address`, `map_id`, `map_type`, `shard_id`, `instance` y `build_id`; `Identity::team_color_id` es un id de *tinte* (`/v2/colors`), no el color de WvW; `UserInfo` de Unofficial Extras no tiene campo de equipo; Nexus no lo expone. Por eso el color se conoce **después** de elegir equipo, y sirve para confirmar rivales (RF-03g), no para filtrar la primera elección.
- **Validación automática:** cada vez que se entra en WvW se comprueba el equipo guardado contra los matchups vivos. Si sigue jugando, no se pregunta nada; si no aparece en ninguno (sorteo mensual), **el selector vuelve a abrirse solo**.
- **Memoria por cuenta** (2026-11-01, decisión del jugador): `Roster` se indexa por el **nombre de cuenta** que arcdps entrega en `EV_ARCDPS_SELF_JOIN` (`AgentUpdate::account`, ya suscrito en `arc_team.rs`, con los dos puntos delante: `:Nombre.1234`), así que todos los personajes de una cuenta comparten equipo y una segunda cuenta no lo hereda: se le pregunta una vez. El nombre de personaje de Mumble solo es la clave **mientras arcdps no ha dado la cuenta** (o no está instalado); la clave con la que se guarda es siempre la que se lee, y con la cuenta conocida una entrada por personaje **no se lee nunca**. **Sin migración** (decisión del jugador, 2026-11-01): `settings.json` guarda `roster.by_account`, y el `roster.by_character` de las versiones ≤ 0.26.5 se ignora y desaparece en el siguiente guardado, así que cada cuenta elige su equipo una vez al actualizar. El botón manual de ajustes sigue siendo la salida para el resto de casos.
- La **API no traduce nombres de equipo** (`/v2/wvw/teams` responde 404; `/v2/wvw/objectives?lang=es` sí traduce objetivos). Las traducciones viven en `data/team-names.json` **indexadas por nombre inglés** (los nombres rotan cada temporada, los ids no) y se mantienen **a mano** desde el 2026-11-01 —la hoja `data/equipos.xlsx` y el script que la convertía se retiraron—: son literales (`Great House Aviary → Pajar de la Gran Casa`), no los del cliente (*Bosquemugoso*), así que el mapa `names` por id de `wvw-teams.json` tiene prioridad para meter los oficiales.

**RF-03b (M)** El addon deberá resolver el match buscando el **id de equipo** en `all_worlds` (no en `worlds`, que solo trae el mundo heredado) para deducir el color propio y los de los enemigos. Usará el prefijo del id de match para descartar la región contraria (`1-x` = NA, `2-x` = EU).

**RF-03c (M)** El usuario podrá cambiar de equipo en cualquier momento desde la configuración.

**RF-03d (M)** Cuando cambie el ID del match, el addon deberá descartar el baseline anterior y resolver de nuevo el color. Consultará `/v2/wvw/timers/teamAssignment`; al vencer el timer comprobará `/v2/wvw/matches/overview` y, si el match observado ya no está activo, volverá a consultar `ids=all` para resolver el equipo. Si la selección cambia o queda obsoleta por la reasignación, borrará el historial y las alertas del ciclo anterior, reiniciará el diff y creará un baseline nuevo. El equipo configurado, la lista de clanes seguidos y el resto de ajustes se conservan. Si el match sigue activo tras vencer el timer, no se borra nada.
- GWT: *Given* un match `2-5` en curso, *When* la API devuelve `2-6` para el mismo equipo, *Then* no se emiten alertas por la diferencia y se crea un baseline nuevo.
- GWT: *Given* que venció `teamAssignment` y el match observado desapareció de `overview`, *When* `ids=all` confirma otra selección o que el equipo ya no juega, *Then* se vacían historial y alertas anteriores, se conserva la configuración y el siguiente snapshot crea un baseline.

**RF-03e (S)** El `world_id` de MumbleLink es un *shard id* y **no** identifica el equipo (verificado en el código de `gw2-mumble-rs`: "not usable since the switch to the megaserver system"). Por tanto la selección es manual por defecto. Solo si el spike T-01 demuestra que coincide con el id de equipo se ofrecerá como sugerencia aceptable.

**RF-03f (S)** El usuario podrá **corregir la traducción** del nombre de su equipo, que se usará en la UI en lugar del nombre de la tabla.
- El campo viene **prellenado** con el nombre en el idioma del juego (el efectivo, no el override de Ctrl derecho) y se guarda **por idioma** (`team_names: { "es": "…" }`), para que cambiar el idioma del juego no muestre el nombre de otro idioma.
- Escribir exactamente el nombre automático **no** crea una entrada: el campo vuelve a seguir las traducciones incluidas. Vaciar el campo hace lo mismo.
- La corrección es **local**: vive en su `settings.json` y no se comparte. Para que *todos* vean un nombre concreto hay que promoverlo a `data/wvw-teams.json` (`names` por id) o a `data/team-names.json` (por nombre inglés), que van dentro de la DLL: se estrena con la siguiente versión, no en caliente.
- El alias único de 0.5.x se sigue leyendo y se migra a `team_names` en cuanto se guarda.

**RF-03g (M)** Tras elegir equipo, el addon deberá mostrar los otros dos equipos del match actual (nombre e id) para que el usuario compruebe que es el suyo.
- GWT: *Given* un equipo recién elegido, *When* se resuelve el match, *Then* se listan sus dos rivales con nombre e id y el color asignado, con opción de corregir la elección.

**RF-03h (M)** El id de equipo es la única clave de identidad; los nombres, alias y colores son solo de presentación y nunca se usan para comparar ni persistir relaciones.

**RF-03i (M)** El addon deberá llevar la tabla `data/wvw-teams.json` (id → nombre → nación) **dentro del binario o junto a él**, versionada en el repositorio. La tabla se regenera con `scripts/update_wvw_teams.ps1`, que contrasta los equipos en uso según `all_worlds` de la API oficial y conserva los nombres ya conocidos si la fuente de nombres no responde. Sin esa tabla el addon sigue funcionando mostrando ids.

**RF-04 (M)** El primer snapshot tras activar, cambiar de match, confirmar una reasignación o recuperar tras un hueco largo se tratará como **baseline** y no generará alertas de flip. Al arrancar el addon el historial anterior **se descarta** (ADR-012) y el baseline reconstruye el estado actual con todos los claims vigentes (RF-25); se registra el `Gap` correspondiente y la UI indica "datos desde <inicio de sesión o última reasignación>".

## B. Objetivo 1 – Alertas de estructuras

> Alcance real según ADR-001: avisos de **flip/captura** y de **cambios observables**, no de "ataque en curso".

**RF-10 (M)** Cuando un objetivo propio pase a manos de un equipo enemigo, el addon deberá mostrar una alerta en el widget fijo (RF-16) con nombre, tipo, mapa y hora.
- GWT: *Given* "Torre X" con `owner=Green` y mi equipo Green, *When* un snapshot la muestra `owner=Red`, *Then* aparece en el widget la línea "Torre X perdida ante Rojo (mapa)".

**RF-11 (S)** Cuando cambie un estado observable de un objetivo propio (descenso de `yaks_delivered`, cambio de `guild_upgrades`, pérdida de claim), el addon deberá emitir un aviso de prioridad baja, configurable.

**RF-12 (M)** El usuario podrá elegir el ámbito de las alertas: *solo mi mapa actual* o *todos los mapas del match*.

**RF-13 (M)** El addon deberá permitir activar/desactivar sonido y configurar un cooldown anti-spam por objetivo (por defecto 60 s).
- GWT: *Given* cooldown 60 s, *When* un objetivo cambia de dueño dos veces en 30 s, *Then* solo se muestra una alerta (la segunda se agrupa en el log).

**RF-14 (M)** El addon deberá mantener un log de alertas recientes (últimas N configurables), accesible desde el widget en una ventana ampliada.

**RF-15 (S)** Se podrán emitir también avisos de captura enemiga de objetivos neutrales o de objetivos enemigos tomados por mi equipo (informativo), desactivados por defecto.

### B2. Widget fijo de alertas (ADR-006)

**RF-16 (M)** El addon deberá mostrar las alertas en un **widget fijo** en pantalla: una lista compacta de las últimas alertas, la más reciente arriba, visible solo en mapas de WvW. Si no hay alertas vigentes y el widget está bloqueado, se oculta por completo (opción para mostrar "Sin novedades").
- GWT: *Given* el jugador en WvW con el widget visible, *When* se genera una alerta, *Then* aparece como nueva primera línea en ≤ 1 frame tras llegar al estado de UI.

**RF-17 (M)** El widget deberá poder moverse y redimensionarse libremente, y su posición y tamaño deberán persistir entre sesiones (mismo contrato de ventana que RF-46).

**RF-18 (M)** El widget tendrá **dos interruptores independientes** (ADR-014; antes eran el mismo y por eso «bloquear» dejaba la ventana sin ratón):

- **Bloquear posición**: no se puede mover ni redimensionar, pero **sí** acepta clics.
- **Click-through**: sigue dibujándose y **no captura el ratón** (los clics pasan al juego).

El click-through **no tiene casilla en los ajustes** (se quitó el 2026-11-01: era la forma de quedarse sin ratón sin saber por qué). Se alterna desde el **menú del clic derecho sobre el widget** y con el **keybind de Nexus** `GW2LURKER_CLICK_THROUGH`, que se registra por defecto como `Ctrl+Shift+L` pero el jugador puede reasignar: la ventana muestra el atajo **que Nexus tiene guardado**, leído de `Nexus/InputBinds.json` (`keybinds.rs`), no un literal. `Alt+Shift+L` no vale: es el menú legacy de arcDPS y el juego se lo queda (comprobado en el juego, 2026-10-31).
- GWT: *Given* el widget en click-through, *When* el usuario hace clic sobre él, *Then* el clic llega al juego.
- GWT *(2026-10-31)*: *Given* el widget en click-through, *When* el usuario hace **clic derecho**
  dentro de la caja del widget, *Then* el click-through se apaga y se guarda, sin abrir los ajustes. Con
  `NO_INPUTS` la ventana no recibe el ratón, así que el callback de render lo mira por su cuenta
  (`ui::window::right_click_on`, solo en el flanco de bajada, contra la caja guardada).
- GWT: *Given* el widget con la posición bloqueada, *When* el usuario hace clic en una pestaña, *Then* cambia de pestaña.

**RF-19 (M)** Cada línea deberá mostrar: icono/forma de tipo de objetivo, texto, mapa y antigüedad ("hace 2 min"), con color según el equipo implicado y distinción por tipo de alerta (pérdida, claim de guild seguida, informativa). No se usará solo el color para distinguirlas.

**RF-19a (M)** El usuario elegirá **de qué se le avisa**, con casillas independientes: Eternos, Alpino azul, Alpino verde, Desierto rojo, «el mapa en el que estoy» y «los clanes que sigo».
- **Por defecto (2026-11-01, petición del jugador):** marcados **«el mapa en el que estoy»** y **«los clanes que sigo»**, y **desmarcados los cuatro mapas fijos**. Antes venían marcados los cuatro, y eso llenaba la lista de cosas de mapas en los que el jugador no estaba.
- Un **clan seguido** avisa en **cualquier** mapa del emparejamiento, esté el jugador donde esté.
- Además hay un **aviso emergente** que desaparece tras `popup_secs` segundos (por defecto **15**, rango **0–20**; `0` lo desactiva), mientras el aviso **sigue en la lista** del widget, que conserva hasta 50 líneas (`max_lines` configura cuántas se ven). Los claims descubiertos en el baseline (`late`) no hacen pop-up, salvo que sean de un clan seguido y de los últimos 10 minutos.
- Las cuatro estructuras (campamento, torre, fortaleza/garita y castillo) vienen marcadas; ruinas y mercenarios nunca avisan (RF-19e).
- Hay un interruptor «**solo los clanes que sigo**» (`only_followed`) que manda sobre las casillas de mapa: con él marcado, lo demás no cuenta.
- El mapa actual sale del `map_id` de Mumble: 38 Eternos, 95 alpino verde, 96 alpino azul, 1099 desierto. Cualquier otro (Cañón, salón, PvE) no cuenta como «mapa actual».

**RF-19b (S)** Clic en una línea (widget desbloqueado) abrirá el mapa simplificado centrado en ese objetivo.

**RF-19c (S)** El usuario podrá configurar opacidad, tamaño de fuente, máximo de líneas visibles (por defecto 5) y qué tipos de alerta se muestran en el widget.

**RF-19d (C)** El **aviso emergente** (la ventana de arriba, la que dibuja el addon) muestra cualquier alerta que pase los filtros del jugador —tipo de aviso y alcance—, de una en una y durante los segundos que se configuren (0–20). No se usa el sistema de avisos de Nexus: `nexus-rs` 0.12 solo expone `send_alert(texto)`, sin título ni sonido, y duplicaba esta ventana (decisión de 2026-10-31 tras la confusión del jugador entre «toast» y «aviso emergente»).
- GWT: *Given* dos reclamaciones seguidas, *When* llegan en el mismo sondeo, *Then* el pop-up enseña primero la más antigua y la siguiente cuando pasa su tiempo, nunca las dos a la vez.
- GWT: *Given* una reclamación de un clan que no se sigue con «Guilds I follow» activo, *When* se detecta, *Then* no sale en el aviso emergente.
- **El sonido sigue fuera**: no hay API de audio en `nexus-rs` 0.12 (comprobado en su código).

**RF-19e (M)** Solo avisan las **estructuras que un clan puede reclamar**: campamentos, torres, fortalezas/garris y castillos, con una casilla por tipo (todas marcadas por defecto). Las ruinas, los campamentos mercenarios, los spawns y el resto del decorado **nunca** generan aviso, aunque esté marcado «el mapa en el que estoy» (petición del jugador, 2026-11-01: «no me avises cuando alguien capture ruinas»).
- Un objetivo de tipo desconocido **sí** avisa: la tabla de objetivos llega una vez por sesión y hasta entonces todo es `Unknown`, así que descartarlo dejaría el addon mudo en vez de filtrar.
- GWT: *Given* todas las casillas marcadas, *When* cambia el dueño de unas ruinas, *Then* no hay aviso ni en la lista ni en el pop-up.

## C. Objetivo 2 – Seguimiento de guilds enemigas

**RF-20 (M)** El usuario podrá añadir guilds a seguir **buscándolas por un trozo del nombre** (índice local + `/v2/guild/search?name=` como extra) o desde la lista de las que han reclamado algo en el ciclo de seguimiento actual, y quitarlas. Se guarda el id en `followed` y su `[TAG] Nombre` en `settings.json` (RF-20b). Los UUID válidos se normalizan a mayúsculas y al formato `8-4-4-4-12` antes de compararlos o guardarlos. Para saber a qué **equipo** pertenece cada guild se usa `/v2/wvw/guilds/:region` (mapa guild → equipo de la API oficial), **cacheado en memoria 30 minutos** (`search.rs`), no en disco.
- GWT: *Given* un nombre ambiguo, *When* la búsqueda devuelve varias guilds, *Then* el usuario elige entre candidatos con nombre y tag.
- GWT: *Given* una guild seguida, *When* se resuelve el match, *Then* la UI muestra el equipo al que pertenece según `/v2/wvw/guilds/:region` y si es propio o enemigo.

**RF-20b (M)** Los nombres de clan se guardan en **dos sitios distintos**, y ninguno de ellos crece sin freno (petición del jugador, 2026-11-01: «tienen que estar los de los clanes que sigo, y los que han reclamado en la sesión, pero no puede crecer indefinidamente»):
- **`settings.json`** (`guild_names`): **solo los clanes que se siguen** (`≤ MAX_FOLLOWED`). Son los que hay que poder nombrar fuera de WvW, cuando no hay sondeo (Bastión del Armisticio). Al dejar de seguir un clan, su nombre sale del fichero; al cargar se descarta cualquier nombre que no sea de un clan seguido.
- **`temp.json`** (en la carpeta del addon): los nombres de los clanes que han reclamado algo en **esta sesión**. Se **regenera en cada arranque** (se borra al cargar el addon) y tiene un tope de 500 entradas, así que no se guarda «para siempre» ni crece sin freno. El jugador lo pidió así el 2026-11-01: «llévalo a un temp.json que se regenere cada vez que entras en el juego».
- **`guild-names.json`** es **solo** el índice de búsqueda por texto, no se tiene en memoria: se lee en el hilo de la búsqueda, se filtra y se suelta. El hilo de render nunca lo lee ni lo clona.
- GWT: *Given* siete clanes seguidos y cincuenta nombres resueltos en la sesión, *When* se cierra el juego, *Then* `settings.json` guarda siete nombres, no cincuenta.

**RF-20c (M)** El índice de nombres se **distribuye** como ZIP (`gw2_lurker_guilds.zip`) junto a su `.md5` en el repositorio de distribución, y el addon lo descarga **la primera vez que arranca** (si no está en disco) y **a petición**, con la URL configurable en los ajustes. Antes de escribir nada se comprueba el MD5 publicado; si no coincide, no se instala.
- Es el mismo mecanismo que el addon hermano `gw2-rotation-builds` usa para su `data.zip` (descargar → verificar MD5 → descomprimir).

**RF-20d (M)** Al empezar a seguir un clan cuyo nombre no está en el índice, el addon pide su nombre al API y lo **añade al índice**, para que la próxima búsqueda por texto lo encuentre.

**RF-20a (M)** Además de buscar, el addon ofrecerá una **lista de guilds sugeridas** para seguir, construida con los claims observados en el ciclo de seguimiento actual: ordenada por número de claims en el mapa actual, después en el resto de mapas del match y después en total; cada fila muestra tag, nombre si se conoce, nº de claims y la hora del último. Un clic la añade a seguidas.
- GWT: *Given* el addon lleva 2 h observando el mapa actual, *When* el usuario abre la pestaña Guilds, *Then* la lista empieza por las guilds con más claims vistos **en ese mapa**, no por orden alfabético.
- La lista nace **vacía** al iniciar el addon o confirmar una reasignación y crece solo con lo observado en el ciclo actual (ADR-012); la UI debe indicarlo ("claims vistos desde 12:00") para no aparentar un ranking global. No se obtiene de ninguna fuente externa: la API no publica estadísticas de claims por guild.

**RF-21 (M)** Cuando un objetivo pase a estar reclamado por una guild seguida, el addon deberá avisar con guild, objetivo, mapa y hora del claim.

**RF-22 (M)** El addon deberá registrar cada claim y unclaim observado: timestamp, `match_id`, objetivo, mapa, guild, tipo de evento y flag `late`. Como un objetivo admite **un solo claim**, cuando pasa de reclamado por X a reclamado por Y se registran `Unclaim(X)` y `Claim(Y)` con el mismo instante de observación.
- GWT: *Given* una torre reclamada por X, *When* un snapshot la muestra reclamada por Y, *Then* el historial contiene `Unclaim(X)` seguido de `Claim(Y)` y el widget solo avisa de lo que toque según las guilds seguidas.

**RF-23 (M)** El usuario podrá ver el estado actual: qué objetivos tiene reclamados cada guild seguida ahora mismo.

**RF-24 (M)** El historial cubrirá el ciclo de seguimiento actual (ADR-012): al arrancar el addon se descarta el de la sesión anterior; tras una reasignación de equipos confirmada (RF-03d) se descartan el historial y las alertas del ciclo anterior y se empieza otro. Dentro del ciclo podrá consultarse por guild y rango de tiempo. La UI mostrará como límite inferior la hora de inicio de sesión o la última reasignación confirmada.

**RF-25 (M)** En baseline o tras un hueco, para **cada** objetivo con `claimed_by` —de cualquier guild, no solo las seguidas— el addon deberá registrar un claim marcado `late=true` usando `claimed_at` como hora real. Los claims de guilds no seguidas se guardan como observación: no generan aviso, pero alimentan el ranking de RF-20a.
- Los `late` **no entran en la lista de avisos** ni en el pop-up: al reiniciar el juego no aparece de golpe todo lo que ya estaba reclamado (bug de 2026-10-31). La única excepción es un claim de un **clan seguido** de los últimos 10 minutos, que es justo lo que se perdió al cargar (RF-21).
- No hay nada de esto en disco (ADR-015): el historial `late` vive en memoria y se limpia al iniciar un nuevo ciclo de seguimiento.

**RF-26 (M)** Cuando el hueco entre snapshots válidos supere el umbral (por defecto 3 × intervalo), el addon deberá registrar un evento `gap` para que la UI advierta de que pueden faltar claims.

**RF-27 (M)** La UI nunca atribuirá una captura a una guild si no hay claim: `owner` solo dice el **color** del equipo, no quién lo cogió. Los textos usarán "reclamado por <guild>" y nunca "capturado por <guild>", para no prometer un dato que la API no da.
- GWT: *Given* un campamento que pasa a `owner=Red` sin `claimed_by`, *When* el widget muestra el flip, *Then* dice "Campamento X perdido ante Rojo" y no menciona ninguna guild.

**RF-27a (C)** Aviso opcional de "capturado sin reclamar": cuando un objetivo reclamable cambia de dueño y sigue con `claimed_by = null` (RF-27b), el addon *podrá* avisar de que está disponible para reclamar. Desactivado por defecto: es una pista, no una captura.

**RF-27d (C)** Opcional posterior: si una guild seguida reclama objetivos en una zona y pocos minutos después un objetivo vecino cambia de dueño sin claim, la UI *podría* sugerir "posible captura de <guild>" marcada explícitamente como **inferida**. Desactivado por defecto.

**RF-27b (M)** El addon deberá distinguir y mostrar **tres estados de claim**, todos derivables de un snapshot diffeado:
1. **Sin reclamar**: `claimed_by = null` en un tipo reclamable (`Camp`, `Tower`, `Keep`, `Castle`). Si además `last_flipped` es posterior al snapshot anterior, se mostrará "capturado hace N s · sin reclamar", que es el caso de un objetivo recién tomado antes de que nadie reclame. En tipos no reclamables (`Ruins`, `Mercenary`, `Spawn`, `Resource`, `Generic`) no se muestra ese estado.
2. **Reclamado**: `claimed_by` con valor; se muestra la guild, la hora del claim (`claimed_at`) y el tiempo que lleva.
3. **Reclamo liberado**: antes había guild y ahora `claimed_by = null` con el mismo `owner`; se muestra "claim liberado" y se registra como `Unclaim` (RF-22).
- GWT: *Given* un campamento con `owner=Blue`, `claimed_by=null` y `last_flipped` de hace 20 s, *When* el usuario abre su tooltip, *Then* lee "sin reclamar · capturado hace 20 s" y no se atribuye a ninguna guild.

**RF-27c (M)** La UI no deberá representar más de un claim por objetivo ni insinuar que varias guilds reclaman a la vez: el modelo de la API es de un único claim por objetivo y cualquier otra lectura sería falsa.

**Limitación conocida:** la API solo da el último claim vigente. Un claim que ocurra y se libere entre dos polls, o dos claims consecutivos de guilds distintas entre dos polls, no se pueden observar. La spec no promete completitud (RNF-05).

### C2. Estado del matchup y rutas inferidas (MVP de validación)

**RF-70 (M)** El addon ofrecerá una vista **"Claims del matchup"** con lo reclamado en **los cuatro mapas** del match, agrupado por mapa y con filtros por tipo y por equipo. Cada fila muestra objetivo, guild (tag), dueño y antigüedad del claim.
- GWT: *Given* el match `2-2` con 41 claims, *When* se abre la vista, *Then* aparecen los 41 agrupados por mapa, incluidos los de mapas donde no está el jugador.

**RF-71 (M)** El addon mostrará una **lista de guilds con claim** ordenada por el claim **más reciente** primero, con el número de objetivos que tienen, el mapa del último claim y su hora. Un clic selecciona la guild (RF-72). Cada fila lleva además dos botones, porque clicar la fila no decía qué hacía (petición del jugador, 2026-11-01): `+` para **seguir** la guild (RF-73), que solo aparece si no se sigue ya, e `i` para ver su **ruta inferida en la pestaña *Mapa*** (RF-32, RF-33), donde se dibuja. La ruta **no** se escribe en texto en esta pestaña; el resumen en texto y el «salto a `<mapa>`» de RF-35 viven junto al dibujo del mapa.
- GWT: *Given* una guild con 5 claims antiguos y otra con 1 claim de hace 30 s, *When* se abre la lista, *Then* la segunda aparece primera.
- GWT: *Given* una guild de la lista que no se sigue, *When* se pulsa `+`, *Then* pasa a seguida, su nombre se guarda en `settings.json` y en la lista de seguidos aparece con su `×`; si ya se seguía, la fila enseña `*` en el sitio del `+` y no hay botón que quite desde aquí.
- GWT: *Given* una guild con reclamos en la sesión, *When* se pulsa `i`, *Then* se abre la pestaña *Mapa* en el último mapa donde reclamó, con su ruta dibujada y el resumen debajo.
- El destino coincide con el **claim vigente más reciente mostrado en la fila**, no con un evento histórico posterior de otro mapa. Si no quedan reclamos vigentes pero sí historial, `i` permite ver el último mapa histórico. Sin ninguna observación registrada, no cambia de pestaña y muestra un aviso con el inicio del ciclo de seguimiento (decisión del jugador, 2026-11-01).

**RF-72 (M)** Al seleccionar una guild se verá su **historial**, separando lo reclamado en el mapa en el que está el jugador **ahora** de lo reclamado en los **demás mapas**, con el rango temporal de RF-32.

**RF-73 (M)** El usuario podrá marcar guilds como **favoritas** ("seguir"): persisten entre sesiones usando UUIDs válidos en forma canónica (mayúsculas, guiones `8-4-4-4-12`), aparecen siempre arriba en la lista de RF-71 y alimentan las alertas de RF-21.
- Igual que en la lista de reclamos, pulsar el **nombre** selecciona el historial debajo, sin navegar; el botón `i` abre la ruta en Mapa según RF-71. El botón `×` sigue reservado a dejar de seguir, con confirmación. El historial del favorito se muestra aunque la lista de reclamos vigentes esté vacía.

**RF-32a (C)** La ruta virtual (RF-32) abre el mapa de RF-71 y dibuja un **paso 0 de origen inferido**, separado de los claims observados 1..N (propuesta aprobada por el jugador, 2026-11-01):
1. **Garri del mapa principal del equipo del clan**, si sigue reclamada por ese clan y su claim es anterior o igual al primer paso del tramo visible. Identidades verificadas en la fixture: `95-37`, `96-37` y la fortaleza central desértica `1099-113`.
2. **Bahía/Colinas en un alpino** (`95/96-32/33`), si siguen reclamadas por ese clan desde antes del primer paso y **tanto el primer claim como el último** del tramo están al norte de la fortaleza (Y de continente menor). Si hay varias candidatas válidas, se elige la más cercana al primer paso, con desempate estable por id. No se aplica esta regla lateral en desierto ni en Eternos.
3. **Respawn del equipo del clan**, identificado por el `owner` del objetivo `Spawn` del matchup, nunca por el equipo del jugador. Si `coord` es nulo, se usa `label_coord` oficial y se indica que la posición es aproximada, no la puerta exacta de salida.
- El origen no tiene timestamp inventado ni cuenta como un claim. Se pinta hueco, con número **0**, leyenda y conexión discontinua hacia **1**; si coincide con el primer claim, el marcador es **0/1**, sin línea de longitud cero. Permanece visible durante la reproducción.
- Los cambios de mapa no se unen gráficamente; el resumen conserva «salto a <mapa>» (RF-35). Los filtros de tiempo se aplican antes de elegir el origen, para no usar una fortaleza reclamada después de la salida.
- Sin claims en el tramo no se dibuja un 0 aislado. Si faltan equipo o coordenadas defendibles, se muestra «Origen desconocido» en lugar de inventar una posición. La ruta y su origen se etiquetan como **inferidos** (RF-33), no como recorrido real.

## D. Vistas de mapa

**RF-30 (M)** Mapa simplificado por mapa de WvW: objetivos como figuras coloreadas por equipo (forma por tipo) sobre un fondo esquemático, sin texturas del juego (ADR-002). Selector de mapa. Los límites se toman de `continent_rect` de `/v2/maps` (ADR-011) y los objetivos con `coord: null` se omiten del dibujo en lugar de dibujarse en (0,0).
- GWT: *Given* el jugador cambia de mapa en el juego, *When* entra en la pestaña *Mapa* desde otra, *Then* se abre en el mapa en el que está (petición del jugador, 2026-11-01). Quedarse en la pestaña no cambia el mapa elegido a mano, y llegar desde un clic de clan (RF-32a) abre el mapa de ese clan, no el del jugador.

**RF-30a (M)** El mapa tendrá filtros por tipo de objetivo (campamento, torre, fortaleza, castillo) activables individualmente y **persistentes** entre sesiones. Los tipos sin valor de puntuación (`Ruins`, `Mercenary`, `Spawn`, `Resource`, `Generic`) estarán ocultos por defecto.
- GWT: *Given* solo "torre" y "castillo" activos, *When* se repinta el mapa, *Then* no se dibuja ningún campamento, pero la capa de claims seguidos sigue mostrando los suyos.

**RF-30b (M)** El mapa marcará los objetivos de **alto valor** con un contorno doble, además de la forma y el color. El criterio es **de datos, no una lista cableada ni el tipo**: alto valor = `points_tick` ≥ 8. Medido en el match `2-2`: las 12 fortalezas y el castillo siempre lo superan, y **4 de 24 torres** también porque están totalmente mejoradas (tick 8); ningún campamento (tick 2) ni Ruinas/Mercenarios/Spawn (tick 0). Un cambio de puntuación en la API se refleja sin tocar código.
- GWT: *Given* el match `2-2`, *When* se aplica el umbral, *Then* se marcan **17** objetivos (12 fortalezas, el castillo y 4 torres mejoradas) y ninguno de los 24 campamentos.
- GWT: *Given* un snapshot, *When* se abre la vista, *Then* cada objetivo aparece en su posición relativa con el color de su dueño.
- El color sale del `owner` **del propio objetivo**, no del reclamo: `matchup::claims` solo devuelve objetivos reclamados, así que deducir el color de ahí pintaba de gris un objetivo capturado sin claim (fallo de 2026-11-01: «Puesto avanzado abrasado» pasó a rojo y el mapa seguía en gris). El mapa se alimenta de `matchup::states`, que devuelve **todos** los objetivos del match con su dueño y, si lo hay, clan y hora.
- GWT: *Given* un objetivo rojo cuyo `claimed_by` está vacío, *When* se pinta el mapa y se pasa el ratón, *Then* se dibuja rojo y el detalle dice «Rojo · Sin reclamar», nunca solo «Sin reclamar».
- GWT: *Given* las coordenadas del API, *When* se proyectan, *Then* el **norte queda arriba**: `continent_rect` y la pirámide de teselas crecen hacia el sur (medido en `/v2/continents/1/floors/1`: Orr, al sur de Kryta, tiene más Y), así que la proyección no invierte la Y. Un mapa del revés pone cada fila de teselas en el sitio de otra (fallo de 2026-10-31).

**RF-31 (M)** Capa de claims: marca visualmente los objetivos reclamados por guilds seguidas (anillo/etiqueta con tag de guild).

**RF-32 (M)** Vista **Histórico** por guild: dibuja los claims en orden cronológico como polilínea numerada y direccional (ruta simulada), con selector de guild y de rango temporal. El claim **más reciente** se resaltará de forma inequívoca (relleno distinto y número en negrita), porque es el que el usuario quiere ver primero.
- GWT: *Given* 5 claims de la guild G en 2 h, *When* se selecciona G y el rango "últimas 2 h", *Then* se dibujan 5 nodos numerados unidos por flechas en orden temporal, con el 5 resaltado como más reciente.

**RF-33 (M)** La vista Histórico deberá indicar de forma visible que la ruta es **inferida**, y los claims `late` y los tramos con `gap` se dibujarán con estilo distinto (trazo discontinuo).

**RF-34 (M)** Al pasar el ratón por un objetivo del mapa se enseña **una línea** con dueño, clan, hora del claim y, al final, si es de alto valor: `Azul  ·  * [YaK] Dolyak Sanctuary  ·  12:31  ·  Alto valor`. El **nombre del objetivo no aparece**: el jugador ya sabe dónde está y alargaba la caja hasta tapar los objetivos vecinos (petición del jugador, 2026-11-01). El `*` marca los clanes que se siguen y sin claim la línea dice solo `Sin reclamar`.
- El orden es deliberado: primero quién y cuándo, y el alto valor como matiz al final (petición del jugador, 2026-11-01).
- Los nombres siguen estando donde no estorban: en la pestaña *Clanes* (historial y ruta) y en la lista de claims.
- GWT: *Given* un castillo azul reclamado por un clan seguido a las 12:31 y de alto valor, *When* el ratón se posa encima, *Then* la línea es la de arriba, sin el nombre del sitio.

**RF-35 (S)** Si los nodos cruzan mapas distintos, la ruta se divide en tramos por mapa con un indicador "salto a <mapa>".

**RF-36 (C)** Reproducción animada de la ruta (play/pausa) sobre el tiempo.

**RF-37 (C)** Fondo opcional del mapa con las teselas oficiales (servicio de teselas, continente de Las Brumas), descargadas en ejecución y guardadas en caché local; nunca empaquetadas en la DLL (ADR-011). Desactivado por defecto.

**RF-38 (C)** Iconos oficiales de los objetivos (campo `marker` de `/v2/wvw/objectives`) como alternativa a las formas dibujadas, con las formas como respaldo si el icono no está disponible.

## E. Configuración y UX

**RF-40 (M)** Ventana de configuración (pestaña *Ajustes*, la última): equipo (selector región → color → equipo), traducción del nombre del equipo, idioma, intervalo de sondeo, ámbito y filtros de avisos (mapas, clanes seguidos, tipos de estructura), segundos y activación del aviso emergente, tipos de aviso del widget, fondo del mapa, tema, fondo y opacidad del widget, tamaño de texto, líneas visibles, ventana visible y posición bloqueada, y la **URL del índice de nombres** con su botón de descarga. Las guilds seguidas se gestionan en la pestaña *Clanes*, no aquí.

**RF-41 (M)** Persistencia de la configuración en el directorio de addon de Nexus.

**RF-42 (S)** El widget se abre y se cierra desde las opciones de Nexus (*Ver la ventana*), desde el menú del clic derecho y con el keybind `GW2LURKER_CLICK_THROUGH` (que alterna el click-through, no la visibilidad); el atajo que se muestra es el que Nexus tiene guardado.

**RF-43 (M)** Indicador de estado: último refresco correcto, "obsoleto" si supera N intervalos, error de red resumido.

### E1. Idioma (i18n)

**RF-44 (M)** Todo texto propio del addon saldrá de una **tabla de traducción** indexada por el texto original en inglés (nunca literales sueltos en la UI). Idiomas de primera clase: **es, en, fr, de** — hay usuarios reales en los cuatro, así que **ninguna columna puede quedar vacía** (lo comprueba un test); una clave sin traducir cae a inglés. El idioma efectivo se resuelve en este orden: (1) preferencia del usuario si no es *Auto*; (2) **idioma del juego** leído de Nexus RTAPI (`read_game().language`: 0=en, 2=fr, 3=de, 4=es); (3) idioma de Nexus / evento de Unofficial Extras `EV_UNOFFICIAL_EXTRAS_LANGUAGE_CHANGED`; (4) inglés. Por defecto: **Auto**.
- GWT: *Given* el juego en francés y preferencia *Auto*, *When* se abre la configuración, *Then* los textos salen en francés sin tocar nada.
- GWT: *Given* una clave nueva añadida solo en español e inglés, *When* corre `cargo test`, *Then* falla el test de completitud de columnas.
- El addon registrará además sus claves en Nexus (`nexus::localization::set_translation`) para que Nexus muestre sus textos en el idioma del usuario.
- Los nombres que vienen de la API (objetivos, mapas) se piden con `?lang=<código>` y se cachean **por idioma** (los cuatro), para poder cambiar de idioma al instante sin red.
- **Terminología de tipos de objetivo** (`Camp`, `Tower`, `Keep`, `Castle`, `Ruins`…): se traducen en la tabla y deben coincidir con el término que usa el juego en ese idioma, no con una traducción libre. Se verifican con el cliente en `fr` y `de` antes de dar la traducción por buena.
- **Decidido (2026-10-31):** se asume el coste de la feature `rtapi` de `nexus-rs` para leer el idioma real del juego; su efecto en el tamaño de la DLL se mide en T-02b. Si la medición lo hace inviable, la cadena de reserva (Extras → Nexus → inglés) ya está definida y no cambia el resto.

**RF-45 (M)** Mientras se mantenga pulsado **Ctrl derecho** con el juego en primer plano, **todo** el texto del addon se renderizará en **inglés**; al soltar vuelve al idioma efectivo. No modifica la preferencia guardada.
- La lectura del teclado estará en `ffi/` con `// SAFETY` (`GetAsyncKeyState(VK_RCONTROL)`), y solo se aplicará si `UiState::GAME_HAS_FOCUS` o el cursor está sobre una ventana del addon, para no dispararse al escribir en otra aplicación.
- GWT: *Given* idioma efectivo español, *When* el usuario mantiene Ctrl derecho con GW2 en primer plano, *Then* widget y ventanas pasan a inglés y vuelven al soltar en ≤ 1 frame.

**RF-46 (M)** El addon tendrá **una sola ventana**, con pestañas (Avisos, Mapa, Clanes, Registro y
Ajustes, esta última siempre al final; ADR-014), que compartirá el **contrato de ventana**: cerrar
con `×` (oculta, no destruye), plegar/desplegar con doble clic en la cabecera, mover arrastrando la
cabecera, redimensionar por el borde y el grip inferior derecho, **bloquear** (impide mover y
redimensionar; bloqueada además no captura clics, RF-18) y **opacidad 0–100 %** aplicada al fondo.
El estado (`visible`, `position`, `size`, `opacity`, `locked`, `collapsed`) se guarda en la
configuración y se restaura al arrancar.
- Las opciones de Nexus ofrecen siempre **Ver la ventana** y **Avisos emergentes**, aunque la
  ventana esté oculta o el overlay no se dibuje fuera de MvM, y dibujan también los ajustes.
- Menú contextual con **clic derecho** sobre cualquier ventana: *Bloquear posición*, *Opacidad* (slider), *Restablecer posición y tamaño*, *Ajustes…*.
- GWT: *Given* opacidad 60 % y ventana bloqueada, *When* se reinicia el juego, *Then* aparece en la misma posición, tamaño, opacidad y bloqueada.

## E2. Actualización, integridad y fallo seguro

**RF-50 (M)** Cuando el fichero de la DLL en disco cambie respecto a la versión cargada (ADR-007), el addon deberá mostrar un aviso visible y persistente: "Actualizado a vX.Y.Z, reinicia el juego para aplicar", sin intentar recargarse en caliente.
- GWT: *Given* v0.1.0 cargada, *When* Nexus deja v0.2.0 en disco, *Then* el banner aparece en ≤ 2 min y permanece hasta reiniciar.

**RF-51 (M)** Si la lectura de `settings.json` falla, el addon deberá arrancar en **modo seguro**: renombrar el fichero dañado a `*.corrupt-<fecha>`, usar valores por defecto, avisar en UI y no cerrar el juego.
- GWT: *Given* un `settings.json` con JSON truncado, *When* carga el addon, *Then* se conserva una copia `.corrupt`, se crea una configuración por defecto y se muestra el aviso.

**RF-52 (S)** La ventana de configuración mostrará la versión actual y el estado de actualización.

**RF-53 (M)** El addon declarará `UpdateProvider::Direct` (no GitHub) y `DisableHotloading` (ADR-007), con `update_link` = `https://raw.githubusercontent.com/cbart-cmd/gw2addonsreleases/main/gw2_lurker.dll`. Publicará junto a la DLL su **MD5** (lo que parsea Nexus) y un **SHA-256** propio para que el usuario verifique la descarga. El procedimiento completo está en `docs/RELEASE.md`.
- GWT: *Given* una versión nueva publicada, *When* Nexus consulta el `update_link`, *Then* descarga una DLL que empieza por `MZ` y cuyo MD5 coincide con el publicado.

**RF-02a (M)** En pantallas de carga, selección de personaje y creación de personaje el addon **no** dibujará nada ni emitirá peticiones. La señal primaria es `NexusLink::is_gameplay == false` (verificado en `nexus-rs`, `src/api/data_link/nexus.rs`); como respaldo, `map_type` ∈ {`AUTO_REDIRECT` (0), `CHARACTER_CREATION` (1)} o `map_id == 0`.
- GWT: *Given* el jugador cierra sesión y vuelve a la selección de personaje, *When* Nexus marca `is_gameplay = false`, *Then* el widget desaparece y el worker deja de pedir datos.

**Nota de alcance:** la validación de DLL corruptas al cargarlas es responsabilidad de Nexus. El proyecto aporta lo que está a su alcance: publicar checksums, probar en CI que la DLL generada es un PE x64 válido que exporta el símbolo de entrada, y fallar de forma segura (RF-51).

## E3. Apariencia (ADR-009)

**RF-60 (M)** Las ventanas del addon (configuración, log, mapa, histórico) deberán heredar por defecto el estilo de imgui que aplica Nexus.

**RF-61 (S)** La configuración ofrecerá un selector de estilo de ventanas: *Heredado de Nexus* (por defecto) o *Propio del addon* (tema oscuro definido por el addon).
- GWT: *Given* el estilo "Propio", *When* se abre cualquier ventana del addon, *Then* usa el tema del addon sin afectar a las ventanas de Nexus ni de otros addons.

**RF-62 (M)** Los colores de equipo serán fijos y clásicos: rojo, azul y verde del juego, y los mismos en widget, mapa e histórico. No hay modo accesible en v1, por lo que la forma de cada objetivo y el texto de cada alerta deben transmitir la información sin depender del color (refuerza RF-19).

**RF-63 (M)** El widget de alertas tendrá fondo configurable (transparente con texto con sombra, o semitransparente oscuro). Por defecto, semitransparente; la opacidad es ajustable (RF-19c).

## F. No funcionales

| ID | Requisito | Verificación |
|---|---|---|
| RNF-01 | Frame time añadido < 0.5 ms en render (widget y mapa abiertos) | Medición con temporizador en dev |
| RNF-02 | Intervalo de polling configurable, **5–600 s**; por defecto **5 s** (decisión del jugador, 2026-10-31, para acercarse a gw2mists). Ese intervalo es **solo** para el estado de los mapas del emparejamiento elegido: **una** partida por id (`/v2/wvw/matches/{id}`, ~44 KB con sus cuatro mapas), 0,2 peticiones/s frente a la recarga de 5/s del cubo de la API. Todo lo demás va limitado aparte (decisión del jugador, 2026-11-01): los nueve emparejamientos (`ids=all`, ~400 KB) como mucho una vez por minuto, también cuando aún no se conoce la partida del equipo; la lista de equipos y los nombres fuera de MvM cada 10 min; objetivos, mapas y mundos una vez por sesión; nombres de clan con un presupuesto por sondeo. Ante un `429` el worker retrocede | Test + spike |
| RNF-03 | Historial acotado **por sesión** (ADR-012/ADR-015): **solo memoria**, tope de 2 000 filas; no hay fichero de historial y al arrancar se empieza de cero | Test con datos sintéticos |
| RNF-04 | Sin red: UI funcional con último estado y marca "obsoleto"; backoff exponencial con tope | Test con cliente simulado |
| RNF-05 | El historial es "mejor esfuerzo": no se promete completitud | Documentado en UI y README |
| RNF-06 | Sin `panic` a través de FFI; errores internos a log | Revisión + `catch_unwind` en puntos de entrada |
| RNF-07 | Compatible con las versiones de Nexus/GW2 objetivo; el addon se desactiva sin romper si falla la carga | Prueba manual |
| RNF-08 | Todo texto propio pasa por la tabla de traducción (es/en/fr/de, con caída a inglés); sin literales en la UI. Un test recorre las claves y detecta columnas vacías | Revisión + test |
| RNF-09 | DLL release ≤ 2,2 MB objetivo, 2,5 MB límite (ADR-013). Medida el 2026-11-01: **2 083 840 B** con las vistas, el índice en ZIP (`zip` 2.x) y la tabla de textos | `check_size.sh` en CI |
| RNF-10 | Ningún fichero de código > 800 líneas | `check_loc.sh` en CI |
| RNF-11 | Formato y lints: `rustfmt` y `clippy -D warnings` | CI |
| RNF-12 | DLL generada es PE x64 válido y exporta el punto de entrada de Nexus | Test de humo en CI |

## G. Fuera de alcance (v1)

Backend compartido entre usuarios, posición real de jugadores, alertas fuera del juego (Discord, móvil), detección de "bajo ataque", lectura de memoria del juego, mapas con texturas oficiales.

## Matriz de trazabilidad (resumen)

Módulos **reales** del workspace actual (`crates/core`, `crates/api`, `crates/addon`). La columna de tareas remite a `03-tasks.md`, que es histórico.

| Requisito | Módulo | Dónde |
|---|---|---|
| RF-01/02/02a | `addon::context`, `addon::detect`, `addon::ffi::mumble` | `ui::mod`, `worker` |
| RF-03* | `addon::team`, `addon::arc_team`, `core::detect`, `core::resolver`, `core::select`, `core::teams` + `data/wvw-teams.json` y `data/team-names.json` | `ui::settings`, `worker` |
| RF-10..15, RF-22..27 | `core::diff`, `core::gate`, `core::matchup` (`claim_holds`, `digest`) | `worker` |
| RF-16..19e | `addon::alerts`, `addon::ui::alerts`, `addon::ui::popup`, `core::alerts` (filtros) | `ui::mod` |
| RF-20..20d, RF-73 | `addon::catalog` (nombres), `addon::names` (índice), `addon::session_names` (`temp.json`), `addon::guild_index` (ZIP), `addon::search` (búsqueda), `core::follow` | `ui::follow`, `ui::guilds` |
| RF-30..38 | `core::project`, `core::model`, `core::route`, `addon::ui::map`, `addon::ui::map::route`, `addon::ui::icons` | `ui::map` |
| RF-40..43, RF-46 | `addon::config`, `addon::writer`, `addon::ui::settings`, `addon::ui::window` | `ui::mod` |
| RF-44..45 | `core::i18n` (tabla), `addon::i18n`, `addon::lang`, `addon::keybinds`, `ffi::keys` | todo el render |
| RF-50..53 | `addon::lib` (DLL en disco, versión), `addon::config` (modo seguro), `scripts/release.ps1`, `scripts/pack-guild-index.ps1` | CI + `docs/RELEASE.md` |
| RF-60..63 | `addon::ui::theme` | `ui::mod` |
