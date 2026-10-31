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
- `[API]` `Cache-Control: public, max-age=1`, sin cabecera `Age`, `X-Rate-Limit-Limit: 600` (600 peticiones/hora).
- `[medido]` Con polling a 10 s: los cambios de **propiedad de objetivos** tienen mediana 19–20 s (máx 110 s observado) y hay emparejamientos muertos con **22 min sin un solo cambio**. 227 peticiones, 0 errores. Conclusión: **30 s por defecto** (120 req/h).
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
- GWT: *Given* el jugador entra en el Bastión del Armisticio (`map_type` 18, `map_id` 1315), *When* el addon lee el contexto, *Then* **no** activa nada.

**RF-02 (M)** Cuando el jugador salga de un mapa de WvW, el addon deberá detener el polling y ocultar toda su UI (excepto la ventana de configuración si está abierta).
- GWT: *Given* el polling activo, *When* el jugador viaja a una ciudad, *Then* no se emiten peticiones de red y no se dibuja nada.

**RF-03a (M)** Si no hay equipo configurado, el addon deberá pedir al usuario que elija su **equipo** (no su mundo) de la lista de equipos del match en curso, obtenida de `all_worlds` de `/v2/wvw/matches?ids=all` (ids 11xxx/12xxx). Se mostrará siempre `nombre (id)`, leyendo el nombre de la tabla versionada `data/wvw-teams.json`, porque la API oficial no expone nombres de equipo. Si un id no está en la tabla, se mostrará solo el id y se avisará de que falta el nombre (ADR-010).
- **Selector en dos pasos (2026-10-31):** región (EU/NA, deducida del prefijo del id: `2-x`/`1-x`) → matchup (`2-1`…`2-5`) → uno de sus **tres** equipos, con su color. Se ofrecen solo ids de equipo: `all_worlds` mezcla ids de mundo antiguos (`2xxx`) que nadie puede elegir.
- **No se puede deducir el color del jugador:** Mumble solo expone `server_address`, `map_id`, `map_type`, `shard_id`, `instance` y `build_id`; `Identity::team_color_id` es un id de *tinte* (`/v2/colors`), no el color de WvW; `UserInfo` de Unofficial Extras no tiene campo de equipo; Nexus no lo expone. Por eso el color se conoce **después** de elegir equipo, y sirve para confirmar rivales (RF-03g), no para filtrar la primera elección.
- **Validación automática:** cada vez que se entra en WvW se comprueba el equipo guardado contra los matchups vivos. Si sigue jugando, no se pregunta nada; si no aparece en ninguno (sorteo mensual), **el selector vuelve a abrirse solo**.
- **Memoria por personaje** (`Roster`, indexado por `Identity::name` de Mumble): una segunda cuenta tiene otros personajes, así que el alt no hereda el equipo del main y se le pregunta una vez. El botón manual de ajustes sigue siendo la salida para el resto de casos.
- La **API no traduce nombres de equipo** (`/v2/wvw/teams` responde 404; `/v2/wvw/objectives?lang=es` sí traduce objetivos). Traducir los 33 nombres requiere una columna curada en `data/wvw-teams.json`.

**RF-03b (M)** El addon deberá resolver el match buscando el **id de equipo** en `all_worlds` (no en `worlds`, que solo trae el mundo heredado) para deducir el color propio y los de los enemigos. Usará el prefijo del id de match para descartar la región contraria (`1-x` = NA, `2-x` = EU).

**RF-03c (M)** El usuario podrá cambiar de equipo en cualquier momento desde la configuración.

**RF-03d (M)** Cuando cambie el ID del match, el addon deberá descartar el baseline anterior, resolver de nuevo el color y marcar el historial con el nuevo `match_id`. Consultará `/v2/wvw/matches/overview` para detectar el cambio de forma barata y `/v2/wvw/timers/teamAssignment` para anticipar la reasignación de equipos.
- GWT: *Given* un match `2-5` en curso, *When* la API devuelve `2-6` para el mismo equipo, *Then* no se emiten alertas por la diferencia y se crea un baseline nuevo.

**RF-03e (S)** El `world_id` de MumbleLink es un *shard id* y **no** identifica el equipo (verificado en el código de `gw2-mumble-rs`: "not usable since the switch to the megaserver system"). Por tanto la selección es manual por defecto. Solo si el spike T-01 demuestra que coincide con el id de equipo se ofrecerá como sugerencia aceptable.

**RF-03f (S)** El usuario podrá asignar un **alias** propio a su equipo, que se usará en la UI en lugar del nombre de la tabla.

**RF-03g (M)** Tras elegir equipo, el addon deberá mostrar los otros dos equipos del match actual (nombre e id) para que el usuario compruebe que es el suyo.
- GWT: *Given* un equipo recién elegido, *When* se resuelve el match, *Then* se listan sus dos rivales con nombre e id y el color asignado, con opción de corregir la elección.

**RF-03h (M)** El id de equipo es la única clave de identidad; los nombres, alias y colores son solo de presentación y nunca se usan para comparar ni persistir relaciones.

**RF-03i (M)** El addon deberá llevar la tabla `data/wvw-teams.json` (id → nombre → nación) **dentro del binario o junto a él**, versionada en el repositorio. La tabla se regenera con `scripts/update_wvw_teams.ps1`, que contrasta los equipos en uso según `all_worlds` de la API oficial y conserva los nombres ya conocidos si la fuente de nombres no responde. Sin esa tabla el addon sigue funcionando mostrando ids.

**RF-04 (M)** El primer snapshot tras activar, cambiar de match o recuperar tras un hueco largo se tratará como **baseline** y no generará alertas de flip. Al arrancar el addon el historial anterior **se descarta** (ADR-012) y el baseline reconstruye el estado actual con todos los claims vigentes (RF-25); se registra el `Gap` correspondiente y la UI indica "datos desde <hora de inicio de sesión>".

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

**RF-18 (M)** El widget deberá tener modo **bloqueado**: sin cabecera ni bordes interactivos y sin capturar el ratón (los clics pasan al juego). Se alterna desde la configuración o con el atajo por defecto `Alt+Shift+L` (keybind gestionado por Nexus, reasignable).
- GWT: *Given* el widget bloqueado, *When* el usuario hace clic sobre él, *Then* el clic llega al juego.

**RF-19 (M)** Cada línea deberá mostrar: icono/forma de tipo de objetivo, texto, mapa y antigüedad ("hace 2 min"), con color según el equipo implicado y distinción por tipo de alerta (pérdida, claim de guild seguida, informativa). No se usará solo el color para distinguirlas.

**RF-19a (M)** Las líneas deberán desvanecerse y desaparecer tras un tiempo configurable (por defecto 60 s), con opción de mantenerlas hasta descartarlas. Las alertas de guilds seguidas podrán configurarse como persistentes.

**RF-19b (S)** Clic en una línea (widget desbloqueado) abrirá el mapa simplificado centrado en ese objetivo.

**RF-19c (S)** El usuario podrá configurar opacidad, tamaño de fuente, máximo de líneas visibles (por defecto 5) y qué tipos de alerta se muestran en el widget.

**RF-19d (C)** Toast emergente adicional y sonido opcional para alertas críticas, desactivados por defecto salvo el sonido ya cubierto por RF-13.

## C. Objetivo 2 – Seguimiento de guilds enemigas

**RF-20 (M)** El usuario podrá añadir guilds a seguir por nombre (`/v2/guild/search?name=`) o por ID, y quitarlas. Se guardará ID, nombre y tag (`/v2/guild/:id`). Para saber a qué **equipo** pertenece cada guild se usará `/v2/wvw/guilds/:region` (mapa guild → equipo de la API oficial), cacheado en disco y refrescado al cambiar de match.
- GWT: *Given* un nombre ambiguo, *When* la búsqueda devuelve varias guilds, *Then* el usuario elige entre candidatos con nombre y tag.
- GWT: *Given* una guild seguida, *When* se resuelve el match, *Then* la UI muestra el equipo al que pertenece según `/v2/wvw/guilds/:region` y si es propio o enemigo.

**RF-20a (M)** Además de buscar por nombre, el addon ofrecerá una **lista de guilds sugeridas** para seguir, construida con los claims ya observados en el historial local: ordenada por número de claims en el mapa actual, después en el resto de mapas del match y después en total; cada fila muestra tag, nombre si se conoce, nº de claims y la hora del último. Un clic la añade a seguidas.
- GWT: *Given* el addon lleva 2 h observando el mapa actual, *When* el usuario abre la pestaña Guilds, *Then* la lista empieza por las guilds con más claims vistos **en ese mapa**, no por orden alfabético.
- La lista nace **vacía** en cada sesión y crece solo con lo observado en ella (ADR-012); la UI debe indicarlo ("claims vistos desde 12:00") para no aparentar un ranking global. No se obtiene de ninguna fuente externa: la API no publica estadísticas de claims por guild.

**RF-21 (M)** Cuando un objetivo pase a estar reclamado por una guild seguida, el addon deberá avisar con guild, objetivo, mapa y hora del claim.

**RF-22 (M)** El addon deberá registrar cada claim y unclaim observado: timestamp, `match_id`, objetivo, mapa, guild, tipo de evento y flag `late`. Como un objetivo admite **un solo claim**, cuando pasa de reclamado por X a reclamado por Y se registran `Unclaim(X)` y `Claim(Y)` con el mismo instante de observación.
- GWT: *Given* una torre reclamada por X, *When* un snapshot la muestra reclamada por Y, *Then* el historial contiene `Unclaim(X)` seguido de `Claim(Y)` y el widget solo avisa de lo que toque según las guilds seguidas.

**RF-23 (M)** El usuario podrá ver el estado actual: qué objetivos tiene reclamados cada guild seguida ahora mismo.

**RF-24 (M)** El historial cubrirá **la sesión actual** (ADR-012): al arrancar el addon se descarta el de la sesión anterior y se empieza uno nuevo. Dentro de la sesión podrá consultarse por guild y rango de tiempo. La UI mostrará siempre la hora de inicio de sesión como límite inferior de lo que se puede ver.

**RF-25 (M)** En baseline o tras un hueco, para **cada** objetivo con `claimed_by` —de cualquier guild, no solo las seguidas— el addon deberá registrar un claim marcado `late=true` usando `claimed_at` como hora real. Los claims de guilds no seguidas se guardan como observación: no generan aviso, pero alimentan el ranking de RF-20a.

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

**RF-71 (M)** El addon mostrará una **lista de guilds con claim** ordenada por el claim **más reciente** primero, con el número de objetivos que tienen, el mapa del último claim y su hora. Un clic selecciona la guild (RF-72).
- GWT: *Given* una guild con 5 claims antiguos y otra con 1 claim de hace 30 s, *When* se abre la lista, *Then* la segunda aparece primera.

**RF-72 (M)** Al seleccionar una guild se verá su **historial**, separando lo reclamado en el mapa en el que está el jugador **ahora** de lo reclamado en los **demás mapas**, con el rango temporal de RF-32.

**RF-73 (M)** El usuario podrá marcar guilds como **favoritas** ("seguir"): persisten entre sesiones, aparecen siempre arriba en la lista de RF-71 y alimentan las alertas de RF-21.

**RF-32a (C)** La ruta virtual (RF-32) se dibuja en el **último mapa donde la guild reclamó algo**, con el origen inferido por esta precedencia:
1. Si es su **mapa principal** y tienen el **garrison** reclamado → se empieza en el garrison.
2. Si el objetivo está **al norte** de colinas/bahía (fronteras alpinas; el equivalente en la desértica) → se empieza en colinas/bahía, más cerca que el respawn.
3. En cualquier otro caso → se empieza en el **respawn** del equipo.
La ruta se etiqueta como **inferida** (RF-33): es una reconstrucción, no el recorrido real.

## D. Vistas de mapa

**RF-30 (M)** Mapa simplificado por mapa de WvW: objetivos como figuras coloreadas por equipo (forma por tipo) sobre un fondo esquemático, sin texturas del juego (ADR-002). Selector de mapa. Los límites se toman de `continent_rect` de `/v2/maps` (ADR-011) y los objetivos con `coord: null` se omiten del dibujo en lugar de dibujarse en (0,0).

**RF-30a (M)** El mapa tendrá filtros por tipo de objetivo (campamento, torre, fortaleza, castillo) activables individualmente y **persistentes** entre sesiones. Los tipos sin valor de puntuación (`Ruins`, `Mercenary`, `Spawn`, `Resource`, `Generic`) estarán ocultos por defecto.
- GWT: *Given* solo "torre" y "castillo" activos, *When* se repinta el mapa, *Then* no se dibuja ningún campamento, pero la capa de claims seguidos sigue mostrando los suyos.

**RF-30b (M)** El mapa marcará los objetivos de **alto valor** con un contorno doble, además de la forma y el color. El criterio es **de datos, no una lista cableada ni el tipo**: alto valor = `points_tick` ≥ 8. Medido en el match `2-2`: las 12 fortalezas y el castillo siempre lo superan, y **4 de 24 torres** también porque están totalmente mejoradas (tick 8); ningún campamento (tick 2) ni Ruinas/Mercenarios/Spawn (tick 0). Un cambio de puntuación en la API se refleja sin tocar código.
- GWT: *Given* el match `2-2`, *When* se aplica el umbral, *Then* se marcan **17** objetivos (12 fortalezas, el castillo y 4 torres mejoradas) y ninguno de los 24 campamentos.
- GWT: *Given* un snapshot, *When* se abre la vista, *Then* cada objetivo aparece en su posición relativa con el color de su dueño.

**RF-31 (M)** Capa de claims: marca visualmente los objetivos reclamados por guilds seguidas (anillo/etiqueta con tag de guild).

**RF-32 (M)** Vista **Histórico** por guild: dibuja los claims en orden cronológico como polilínea numerada y direccional (ruta simulada), con selector de guild y de rango temporal. El claim **más reciente** se resaltará de forma inequívoca (relleno distinto y número en negrita), porque es el que el usuario quiere ver primero.
- GWT: *Given* 5 claims de la guild G en 2 h, *When* se selecciona G y el rango "últimas 2 h", *Then* se dibujan 5 nodos numerados unidos por flechas en orden temporal, con el 5 resaltado como más reciente.

**RF-33 (M)** La vista Histórico deberá indicar de forma visible que la ruta es **inferida**, y los claims `late` y los tramos con `gap` se dibujarán con estilo distinto (trazo discontinuo).

**RF-34 (M)** Al pasar el ratón por un nodo: objetivo, hora, duración del claim y si es `late`.

**RF-35 (S)** Si los nodos cruzan mapas distintos, la ruta se divide en tramos por mapa con un indicador "salto a <mapa>".

**RF-36 (C)** Reproducción animada de la ruta (play/pausa) sobre el tiempo.

**RF-37 (C)** Fondo opcional del mapa con las teselas oficiales (servicio de teselas, continente de Las Brumas), descargadas en ejecución y guardadas en caché local; nunca empaquetadas en la DLL (ADR-011). Desactivado por defecto.

**RF-38 (C)** Iconos oficiales de los objetivos (campo `marker` de `/v2/wvw/objectives`) como alternativa a las formas dibujadas, con las formas como respaldo si el icono no está disponible.

## E. Configuración y UX

**RF-40 (M)** Ventana de configuración: equipo, ámbito de alertas, opciones del widget (posición, bloqueo, opacidad, líneas, tiempo de vida), sonido, cooldown, intervalo de polling, retención de historial, guilds seguidas.

**RF-41 (M)** Persistencia de la configuración en el directorio de addon de Nexus.

**RF-42 (S)** Atajo de teclado y/o icono de acceso rápido de Nexus para abrir/cerrar la ventana principal.

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

**RF-46 (M)** Todas las ventanas (opciones, mapa, histórico, guilds, log) y el widget compartirán el mismo **contrato de ventana**: cerrar con `×` (oculta, no destruye), plegar/desplegar con doble clic en la cabecera, mover arrastrando la cabecera, redimensionar por el borde y el grip inferior derecho, **bloquear** (impide mover y redimensionar; el widget bloqueado además no captura clics, RF-18) y **opacidad 0–100 %** aplicada al fondo. El estado por ventana (`visible`, `position`, `size`, `opacity`, `locked`, `collapsed`) se guarda en la configuración y se restaura al arrancar.
- Menú contextual con **clic derecho** sobre cualquier ventana: *Bloquear posición*, *Opacidad* (slider), *Restablecer posición y tamaño*, *Ajustes…*.
- GWT: *Given* opacidad 60 % y ventana bloqueada, *When* se reinicia el juego, *Then* aparece en la misma posición, tamaño, opacidad y bloqueada.

## E2. Actualización, integridad y fallo seguro

**RF-50 (M)** Cuando el fichero de la DLL en disco cambie respecto a la versión cargada (ADR-007), el addon deberá mostrar un aviso visible y persistente: "Actualizado a vX.Y.Z, reinicia el juego para aplicar", sin intentar recargarse en caliente.
- GWT: *Given* v0.1.0 cargada, *When* Nexus deja v0.2.0 en disco, *Then* el banner aparece en ≤ 2 min y permanece hasta reiniciar.

**RF-51 (M)** Si la inicialización o la lectura de `config.json` o del historial falla, el addon deberá arrancar en **modo seguro**: renombrar el fichero dañado a `*.corrupt-<fecha>`, usar valores por defecto, avisar en UI (W8) y no cerrar el juego.
- GWT: *Given* un `config.json` con JSON truncado, *When* carga el addon, *Then* se conserva una copia `.corrupt`, se crea una configuración por defecto y se muestra el aviso.

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
| RNF-02 | Intervalo de polling configurable; mínimo 15 s; por defecto **30 s**, fijado con datos de T-02 (cambios de propiedad con mediana 19–20 s; 120 de las 600 peticiones/h) | Test + spike |
| RNF-03 | Historial acotado **por sesión** (ADR-012): tope de 2 MB con rotación dentro de la sesión; al arrancar se descarta la anterior | Test de rotación |
| RNF-04 | Sin red: UI funcional con último estado y marca "obsoleto"; backoff exponencial con tope | Test con cliente simulado |
| RNF-05 | El historial es "mejor esfuerzo": no se promete completitud | Documentado en UI y README |
| RNF-06 | Sin `panic` a través de FFI; errores internos a log | Revisión + `catch_unwind` en puntos de entrada |
| RNF-07 | Compatible con las versiones de Nexus/GW2 objetivo; el addon se desactiva sin romper si falla la carga | Prueba manual |
| RNF-08 | Todo texto propio pasa por la tabla de traducción (es/en/fr/de, con caída a inglés); sin literales en la UI. Un test recorre las claves y detecta columnas vacías | Revisión + test |
| RNF-09 | DLL release ≤ 1.5 MB objetivo, 2.5 MB límite (ADR-008) | `check_size.sh` en CI |
| RNF-10 | Ningún fichero de código > 800 líneas | `check_loc.sh` en CI |
| RNF-11 | Formato y lints: `rustfmt` y `clippy -D warnings` | CI |
| RNF-12 | DLL generada es PE x64 válido y exporta el punto de entrada de Nexus | Test de humo en CI |

## G. Fuera de alcance (v1)

Backend compartido entre usuarios, posición real de jugadores, alertas fuera del juego (Discord, móvil), detección de "bajo ataque", lectura de memoria del juego, mapas con texturas oficiales.

## Matriz de trazabilidad (resumen)

| Requisito | Módulo de diseño | Tarea |
|---|---|---|
| RF-01/02 | `nexus_ui::context` | T-12 |
| RF-03*, RF-04 | `core::match_resolver`, `core::baseline`, `addon::ui::world_picker`, `data/wvw-teams.json` | T-05, T-06, T-30 |
| RF-10..15 | `core::diff`, `core::rules`, `nexus_ui::alerts` | T-07, T-08, T-13 |
| RF-16..19d | `addon::ui::alert_widget` | T-13, T-13b |
| RF-20..26 | `core::follow`, `store`, `api::guild` | T-09, T-10, T-11 |
| RF-30..36 | `map::*` | T-14, T-15, T-16 |
| RF-37..38 | `map::background`, `map::icons` | T-29 |
| RF-40..43 | `config`, `nexus_ui::settings` | T-12, T-17 |
| RF-44..46 | `addon::i18n`, `addon::ui::window`, `ffi::keys` | T-31, T-13, T-17 |
| RF-70..73 | `addon::ui::matchup`, `core::history` | T-36, T-37, T-38, T-39 |
| RF-32a | `map::route` | T-40 |
| RF-60..63 | `addon::ui::theme` | T-28 |
| RF-50..53 | `addon::update`, `addon::safe_mode` | T-25, T-26, T-27 |
