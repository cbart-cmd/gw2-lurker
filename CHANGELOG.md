# Changelog
Formato [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/), versionado [SemVer](https://semver.org/lang/es/).

## [Unreleased]

## [0.26.9] - 2026-11-02
### Añadido
- Avisos con el formato de RF-19f: verbo desde tu equipo, mapa con su nombre completo, marca de
  alto valor, antigüedad en vivo y hora exacta en el registro y el tooltip.
- Emergente apilado: hasta cinco avisos simultáneos, ordenados por prioridad y con caducidad
  individual desde su llegada; las líneas descartadas se cuentan sin reproducirse después.
### Corregido
- El emergente guarda su propia posición al colocarlo, en vez de la posición de otra ventana.
- Los reclamos recientes de clanes seguidos recuperados al entrar también salen en el emergente;
  los reclamos antiguos y de clanes no seguidos siguen descartándose.
### Técnico
- DLL release: 1 634 816 bytes (1 616 384 bytes en la versión anterior).

## [0.26.8] - 2026-11-02
### Cambiado
- La DLL pasa de 2,28 MB a 1,62 MB: se ha quitado una librería de expresiones regulares que
  solo servía para filtrar el log por nivel. Sin cambios de funcionamiento.

## [0.26.7] - 2026-11-02
### Añadido
- **Selector de equipo en su propia ventana**, centrada en pantalla. Se abre sola al entrar en
  MvM o en el Bastión del Armisticio si aún no has elegido equipo, si tu equipo ya no juega o si
  ha cambiado el emparejamiento (te lo avisa y te pide confirmar o cambiar). En *Ajustes* queda
  el botón **Cambiar de equipo**, que abre esa ventana.

## [0.26.6] - 2026-11-02
### Añadido
- **B — Estela** en Mapa: curvas inferidas con flechas, recencia, revisitas agrupadas,
  selección de paso y anillos de intervalo. Cronología proporcional a las horas reales,
  ventanas Todo/1 h/15 min y play/pausa a 700 ms por claim; pausa al ocultar el mapa.
- Los claims tardíos y los huecos de sondeo conservan sus marcas. Saltos de mapa y posiciones
  ausentes no se conectan; los intervalos largos sin claims no se presentan como gaps de red.
### Corregido
- Sin equipo elegido para la cuenta actual, todas las pestañas de datos del widget indican
  que hay que elegirlo en Ajustes, que sigue accesible. No se muestran estadísticas de la
  partida anterior mientras falta esa elección.
### Cambiado
- Modelos temporales preparados en el worker y geometría cacheada; arcos coincidentes y marcas
  ligeras evitan agotar los índices de imgui con 2.000 claims. Sin consultas de API nuevas.
- Audio retirado del alcance por petición del jugador; no había una integración implementada
  que eliminar ni queda un pendiente esperando una API de Nexus.
- El equipo se guarda **solo por cuenta** y no se migra nada de lo anterior: `settings.json` lo
  guarda ahora en `roster.by_account` y el `roster.by_character` de versiones anteriores se ignora
  (desaparece al guardar), así que cada cuenta elige su equipo **una vez** tras actualizar y todos
  sus personajes lo comparten. Antes, una entrada vieja por personaje seguía resolviendo y el
  selector volvía a salir al cambiar de personaje. Sin arcdps no hay cuenta, y entonces la clave
  sigue siendo el personaje.
- `Nexus.log` escribe bastante menos: los reclamos, cambios de dueño y abandonos de cada sondeo ya
  no se anotan (eran ~60 líneas en los primeros minutos), y `vigilando` y `deteccion` solo salen
  cuando algo cambia en vez de repetirse cada minuto. El latido (`sondeo N`) sale cada cinco
  minutos aunque el intervalo de sondeo sea de 5 s. Se mantienen el arranque, los cambios de
  contexto (`gate`), los avisos y los errores.
- El log ya no guarda el nombre de tu cuenta de GW2: se pega en los informes.
### Técnico
- DLL release: 2 264 576 B, dentro del objetivo; sin dependencias añadidas.
- Preparación de geometría con 2.000 claims: p95 0,056 ms. Cronología, objetivos y ruta con
  imgui nativo local: p95 0,311 ms en el caso denso. No es una medición del frame completo de GW2.

## [0.26.5] - 2026-11-01
### Añadido
- El equipo se guarda **por cuenta**, no por personaje: todos los personajes de una cuenta comparten
  equipo y zona, y una segunda cuenta arranca vacía y se pregunta una vez. La cuenta sale del nombre
  que arcdps ya entregaba en el evento de carga de mapa y que el addon no usaba. Las entradas
  guardadas por personaje en versiones anteriores se siguen leyendo.

## [0.26.4] - 2026-11-01
### Corregido
- Volver a la pestaña *Mapa* a mano abre el mapa en el que estás. Antes, después de abrir el mapa
  de un clan, se quedaba en el mapa del clan: la marca de «vengo de un clic de clan» seguía viva
  mientras imgui tardaba en cambiar de pestaña.
### Añadido
- La pestaña *Mapa* abre en el mapa en el que estás y con el desplegable de clan vacío: se ven los
  reclamos de todos los clanes que sigues (anillo y TAG) y la ruta solo si eliges un clan.
- El desplegable de clan del mapa tiene una entrada «No guild» para quitar la ruta sin salir de la
  pestaña.

## [0.26.3] - 2026-11-01
### Corregido
- El aviso del click-through muestra el atajo actual aunque se cambie en Nexus con el juego
  abierto (antes se quedaba con el que había al cargar el addon).
- El historial de un clan se pinta de más reciente a más antiguo, en lugar de al revés.

## [0.26.2] - 2026-11-01
### Corregido
- Abrir la ruta de un clan lleva al último mapa donde reclamó, también si no es favorito, sin
  sustituirlo por el mapa del jugador durante el cambio de pestaña.
- Si no hay eventos registrados para el clan, se conserva la pestaña y se muestra un aviso con
  el inicio del ciclo de seguimiento. El selector de clan del mapa usa la misma validación.
- Los favoritos separan nombre (ver historial) y botón `i` (ir a Mapa), y muestran el historial
  aunque ya no haya reclamos vigentes. El botón de mapa coincide con el último claim de la fila;
  si solo quedan eventos históricos, permite ver su ruta.
- La ruta pinta el origen inferido **0** desde garri, una fortaleza lateral alpina válida o el
  respawn del equipo del clan, con condiciones temporales y geográficas; distingue posiciones
  aproximadas, no cuenta el origen como claim y usa **0/1** si coincide con el primer evento.
- La preparación de rutas indexa coordenadas y evita copiar textos por nodo para mantener el
  presupuesto de CPU con el historial máximo.
### Técnico
- DLL release: 2 195 456 B, dentro del objetivo de tamaño; sin dependencias añadidas.

## [0.26.1] - 2026-11-01
### Corregido
- El atajo del click-through se nombra con la distribución de teclado del jugador: en un teclado
  español sale `Ctrl+Shift+Ñ` y no `Ctrl+Shift+;`.
- Ajustes: se quita el botón *Grabar*, que no hacía nada visible; los cambios ya se guardaban solos y
  ahora una línea lo dice.

## [0.26.0] - 2026-11-01
### Corregido
- El diff registra el fin de un claim cuando el objetivo cambia de dueño, descarta claims caducados
  en los baselines y detecta un nuevo claim del mismo clan por su hora de reclamación (RF-22, RF-25).
- El índice de clanes ya no se instala si falta el MD5 publicado, no se puede leer o no coincide
  (RF-20c).
- La detección de una DLL actualizada sigue comprobándose durante la sesión y el render solo lee el
  resultado en caché (RF-50).
- Si no se puede iniciar el hilo de ajustes, el error queda registrado; al cerrar se conserva el
  guardado final incluso si el hilo terminó inesperadamente.
- Al recuperar el historial tras un hueco, no se vuelven a emitir claims que ya estaban registrados;
  sí se conservan los claims nuevos ocurridos durante el hueco (D-B07).
- El worker reutiliza la respuesta global de partidas cuando busca por equipo y deja de repetir
  peticiones sin datos Mumble; al descargar el addon espera a que terminen sus tareas.
- Las descargas de catálogos e imágenes aplican esperas tras errores temporales; el índice valida el
  tamaño real del ZIP y conserva los nombres añadidos localmente durante la descarga.
- El idioma y el tema seleccionados se aplican correctamente en la UI; el popup de prueba se puede
  mostrar aunque no haya avisos reales y el mapa corrige el anillo de objetivos de alto valor.
- Mientras no se conoce la partida del equipo, la búsqueda de los nueve emparejamientos (~400 KB)
  se hace como mucho una vez por minuto, no cada 5 s: el intervalo de sondeo es solo para el estado
  de los mapas del emparejamiento elegido (RNF-02).
- Los ids de clan y de partida se comprueban antes de pedirlos a la API: un id mal formado del
  índice de nombres ya no puede cambiar la petición.
- Los UUID de clan válidos se normalizan a una forma canónica en las búsquedas, los mapas guild→equipo,
  los claims y la lista de seguimiento; los errores de API se muestran localizados en el estado y el
  buscador.
- Al confirmarse una reasignación de equipos tras `teamAssignment`, el worker vuelve a resolver el
  match, crea un baseline y limpia el historial/las alertas anteriores sin tocar la configuración
  ni los clanes seguidos. La pestaña *Clanes* muestra desde cuándo corre el ciclo actual.

## [0.25.15] - 2026-11-01
### Cambiado
- **La pestaña *Mapa* se abre en el mapa en el que estás** (petición del jugador): al entrar desde
  otra pestaña se dibuja el mapa que da el Mumble, así que si cambias de mapa en el juego lo ves al
  abrirla sin tocar el desplegable.
- La excepción sigue siendo clicar un clan (su fila en *Clanes* o el botón `i`): entonces se abre el
  mapa donde ese clan reclamó, que es lo que se quiere ver de su ruta.
- Quedarse en la pestaña no cambia el mapa elegido a mano en el desplegable.

## [0.25.14] - 2026-11-01
### Cambiado
- **La lista de clanes del widget lleva dos botones por fila** (petición del jugador): `+` para seguir
  el clan —solo aparece si no lo sigues ya— e `i` para ver su ruta. Clicar la fila no decía qué hacía:
  ahora cada acción tiene su botón, y clicar la fila sigue enseñando el historial (RF-72).
- **La ruta inferida ya no se escribe en texto debajo de la lista**: `i` lleva el mapa al último sitio
  donde reclamó el clan y allí está el dibujo. El resumen en texto («desde su fortaleza -> A -> salto a
  X -> B», con el salto de mapa de RF-35) se pinta junto al mapa, no en la pestaña *Clanes*.
- Si ya sigues al clan, en el sitio del `+` aparece `*`: es estado, no un botón. Quitar se hace con la
  `×` de la lista de seguidos, que pregunta antes.

## [0.25.13] - 2026-11-01
### Arreglado
- **El mapa pintaba «sin reclamar» objetivos que sí tenían dueño** (informe del jugador): el color de
  cada objetivo salía del reclamo, y un objetivo capturado **sin claim** no tiene reclamo, así que se
  dibujaba en gris y su hover no decía el dueño. Ahora `matchup::states` da **todos** los objetivos del
  match con su dueño y el mapa lo usa: un puesto rojo sin reclamar se dibuja rojo y se lee
  «Rojo · Sin reclamar». La lista de reclamos y las rutas no cambian.
- Tests nuevos: un objetivo sin claim conserva dueño y hora vacía; el hover dice el color; los estados
  y los reclamos coinciden en qué objetivos están reclamados.

## [0.25.12] - 2026-11-01
### Cambiado
- **Ajustes por defecto de una instalación nueva** (petición del jugador): en «avisar de» vienen
  marcados **«el mapa en el que estoy»** y **«los clanes que sigo»**, con campamentos, torres,
  fortalezas/garitas y castillos; los **cuatro mapas fijos ya no vienen marcados** (llenaban la lista
  con cosas de mapas en los que el jugador no estaba).
- **El aviso emergente dura 15 s por defecto** en vez de 2, que se iba antes de poder leerlo.
- Los ajustes guardados no cambian: esto solo afecta a quien no los tenga todavía.

## [0.25.11] - 2026-11-01
### Añadido
- **Desplegable de clan en la pestaña *Mapa***, al lado del de mapa: primero los clanes que sigues
  (con `*`) y después los **cinco últimos** que han reclamado algo en la sesión, sin repetir. Elegir
  uno dibuja su ruta y lleva el mapa al último sitio donde reclamó, igual que clicarlo en la pestaña
  *Clanes* — así no hay que cambiar de pestaña para buscarle.

### Cambiado
- El desplegable de mapa ocupa **120 px** en vez de todo el ancho: solo enseña cuatro palabras y
  dejaba sin sitio al de clan.

## [0.25.10] - 2026-11-01
### Corregido
- **«Con algo reclamado ahora» enseñaba ids de clan en vez de nombres.** A esa lista se le pasaba
  solo el mapa de `settings.json`, donde únicamente están los clanes que ya sigues; los de la sesión
  viven en `temp.json`. Ahora recibe el mapa que se dibuja (ajustes + `temp.json`), así que sale
  `[PEST] Please excuse the silly tactics` y no `C3484B97-EC4A-…`. Al añadir uno desde esa lista, su
  nombre se guarda en los ajustes y se le enseña al índice.
- `ui/mod.rs` llegó a 803 líneas: la línea de estado pasa a `ui/status.rs`.

## [0.25.9] - 2026-11-01
### Cambiado
- **El hover del mapa ya no dice el nombre del objetivo** y el alto valor pasa al final de la línea:
  `Azul · * [YaK] Dolyak Sanctuary · 12:31 · Alto valor`. El nombre alargaba la caja hasta tapar los
  objetivos vecinos y el jugador ya sabe dónde está. Sin claim, la línea dice solo «Sin reclamar».

## [0.25.8] - 2026-11-01
### Corregido
- **CI en rojo por tres avisos de seguridad de `zip` 2.6.1** (RUSTSEC-2026-0367, -0368 y -0369: bits
  SUID/SGID y enlaces simbólicos al extraer, todos corregidos en `zip` 9). Actualizado a `zip` 9.2.0.
  El addon nunca llama a `extract()` —lee las entradas una a una y escribe en una ruta fija—, pero
  no se silencia un aviso: se actualiza.
- Consecuencias del salto: **MSRV 1.88** (la 9.x usa edición 2024) y **+58 KB** de DLL (2 083 840 →
  2 141 696 B, aún bajo el objetivo de 2,2 MB de ADR-013).
- `CI`: el job de `check_loc.sh` fallaba por `config.rs` con 813 líneas; ya estaba partido en
  `config/mod.rs` + `config/tests.rs` en el commit anterior.

## [0.25.7] - 2026-11-01
### Corregido
- **La descarga del índice de nombres fallaba con «no existe».** La URL por defecto tenía una errata
  (`gw2addonreleases` en vez de `gw2addonsreleases`), así que GitHub respondía 404. Corregida, y la
  errata se repara sola en `settings.json` al cargar. El error 404 ahora se explica en la ventana, y
  la descarga lleva un sello `?v=` para saltarse la caché del CDN.

## [0.25.6] - 2026-11-01
### Cambiado
- **Los nombres de la sesión van a `temp.json`**, no a memoria: el jugador prefiere un fichero
  temporal que **se borra y regenera en cada arranque** (tope de 500 entradas) a tenerlos en el
  proceso. `settings.json` sigue guardando solo los de los clanes que sigue.
- `temp.json` está en la carpeta del addon, junto a `settings.json`, y no se versiona en ningún
  sitio: es del juego, no del repo.

## [0.25.5] - 2026-11-01
### Cambiado
- **`settings.json` ya no guarda todos los nombres de clan.** Solo los de los clanes que siguen
  (10 como mucho, `MAX_FOLLOWED`); los de los clanes que han reclamado algo en la sesión viven en
  memoria y se van al reiniciar el juego. Al dejar de seguir un clan se borra su nombre, y al
  cargar se descarta cualquiera que no sea de un clan seguido (limpieza automática del fichero:
  el del jugador pasa de 51 nombres a 7).

## [0.25.4] - 2026-11-01
### Corregido
- **El primer clic en un resultado del buscador de clanes no añadía el clan.** Clicar la fila
  desactivaba el campo de texto, eso lanzaba otra búsqueda en el mismo fotograma y la lista se
  sustituía por «Buscando...»: el clic caía en el vacío. Ahora perder el foco no busca, y los
  resultados siguen en pantalla (y clicables) mientras se busca.

## [0.25.3] - 2026-11-01
### Cambiado
- **Nombres de equipo en español y francés completados** con la lista estática de la wiki
  (`API:2/wvw/guilds/:region`, instantánea del 2025-11-15). 15 valores en 13 equipos que estaban
  en inglés de relleno: *Descanso de Rall*, *Sala del Juicio*, *Puerto de Yohlon*, *Skrittsburgo*,
  *Dominio de la Angustia*, *Repos de Rall*, *Hall de jugement*, *Forêt silencieuse*,
  *Havre de Yohlon*, *Domaine du tourment*, *Trône de Balthazar*, *Tombes de Drascir*,
  *Bois moussu*, *Domaine de l'angoisse* y *Conservatoire de Lutgardis*.
- Añadido `Cathedral of Blood` (es *Catedral de la Sangre*, fr *Cathédrale sanglante*), el nombre
  que la wiki da al equipo `11010`; hoy se llama *Ruined Cathedral of Blood* y el pool conserva los
  dos para que resuelva si el nombre vuelve.
## [0.25.2] - 2026-11-01
### Corregido
- **Un «?» donde iba una flecha en la ventana de avisos.** La fuente de Nexus solo trae latín básico
  y latino-1: las flechas pasan a `->`, los puntos suspensivos a `...`, las estrellas a `*`/`-` y las
  comillas alemanas a « ». Un test impide que vuelva a colarse un carácter fuera de latino-1 en los
  textos del addon.

## [0.25.1] - 2026-11-01

### Cambiado
- El buscador de clanes pone **primero los nombres que empiezan** por lo escrito (sin contar el
  [TAG]): con «DIE» salen antes *Die Gäng*, *Die Wiener* o *Die Garde* que *Never Die*.

## [0.25.0] - 2026-11-01
### Añadido
- **Avisos solo de estructuras** (RF-19e): casillas para campamentos, torres, fortalezas/garris y
  castillos. Las ruinas y los campamentos mercenarios **nunca** avisan, aunque esté marcado «el mapa
  en el que estoy» (petición del jugador).
- **El índice de nombres de clan se distribuye aparte** (RF-20c): `gw2_lurker_guilds.zip` + `.md5`
  en el repositorio de distribución. El addon lo descarga la primera vez que no lo encuentra y a
  petición, con la URL configurable en Ajustes → *Índice de nombres de clan*.
- Al empezar a seguir un clan que no estaba en el índice, su nombre se resuelve y se añade (RF-20d),
  así que la próxima búsqueda por texto lo encuentra.

### Cambiado
- **Los nombres de clan ya resueltos se guardan en `settings.json`** (RF-20b) y `guild-names.json`
  pasa a ser **solo** el índice de búsqueda: se lee en el hilo del buscador, se filtra y se suelta.
  Antes se cargaba entero al arrancar y se clonaba en cada sondeo (9 900 entradas cada 5 s).
- El buscador de clanes busca mientras escribes (300 ms de espera) contra el índice y añade el API
  al pulsar Enter o *Buscar*.

### Quitado
- `data/equipos.xlsx` y `scripts/merge_team_names.ps1`: `data/team-names.json` se edita a mano.

### Corregido
- **Cuelgue del juego con el widget abierto** (0.24.1): el aviso naranja de «partida congelada»
  pedía el candado de la configuración desde dentro de una función que ya lo tenía, y el hilo de
  render se quedaba esperándose a sí mismo. El log acababa en `sondeo 50` sin pánico (crash del
  jugador a las 00:40). Ahora lee el intervalo de la configuración que ya trae la función.

## [0.24.2] - 2026-11-01

### Corregido
- **Cuelgue del juego al abrir el widget** (crash del jugador, 00:40). El aviso de «partida
  congelada» de la 0.24.1 pedía el candado de la configuración dentro de una función que **ya lo
  tenía**: `std::sync::Mutex` no es reentrante y el hilo de render se quedaba esperándose a sí
  mismo. Ahora lee el intervalo de la configuración que ya trae la función. Es el mismo fallo que
  congeló el juego el 2026-10-31, esta vez dentro de `draw_health`.

## [0.24.1] - 2026-11-01

### Añadido
- **Aviso de partida congelada**: cuando la API de ArenaNet repite la misma copia de la partida
  durante 5 minutos o más, el widget lo dice en naranja con los minutos y el intervalo de sondeo.
  El 2026-11-01 la partida `1-1` devolvió durante **20 minutos** exactamente los mismos puntos y
  los mismos reclamos mientras las otras ocho se movían; el jugador lo leyó como «ha dejado de
  funcionar a los 11 minutos». Se compara una huella de la partida (puntos + dueño de cada
  objetivo), así que el aviso distingue «no pasa nada» de «el addon se ha parado».
### Corregido
- `bugs.md`: se restauran las entradas que se perdieron al añadir las de la 0.24.0 (se habían
  quedado fuera el parte anterior y el reporte nuevo de los 11 minutos). El fichero vuelve a
  tenerlas todas.

## [0.24.0] - 2026-11-01

### Añadido
- **Buscador de clanes que busca de verdad.** El API (`/v2/guild/search`) devuelve **un solo**
  clan y solo cuando el texto empieza su nombre, así que «DIE» encontraba *Die* y ninguna más.
  Ahora el addon busca por **subcadena** en su propia caché de nombres (`guild-names.json`), que
  se rellena sola con lo que va viendo y se puede llenar de golpe con
  `scripts/guild-search.ps1 -BuildIndex`. El API queda como extra. Sin región ni equipo que
  filtrar: escribes y la lista sale.
- **Los nombres de los clanes se guardan para siempre.** Se piden una vez y no se vuelven a pedir
  (no cambian). Fuera de WvW —el Bastión del Armisticio— ya no hay ids de 36 caracteres: era
  justo el sitio donde nada sondeaba y la lista salía sin nombres.
### Corregido
- **Los segundos del aviso emergente se guardaban como 5.** El control de Ajustes ofrecía hasta
  20 s pero el saneado del fichero recortaba a 5, así que cualquier valor mayor aparecía como 5
  tras reiniciar. Ahora el tope es el mismo en los dos sitios (20 s).
- **El atajo del click-through ya no es un literal.** El addon lee `addons/Nexus/InputBinds.json`
  y enseña la combinación que Nexus tiene **guardada** (Nexus no expone un lector de atajos a los
  addons). Si no se puede leer, dice el identificador `GW2LURKER_CLICK_THROUGH` en vez de inventar.
- **Click-through**: se activa con el **clic derecho** sobre el widget; la casilla de
  Ajustes → Ventanas desaparece (era la forma de quedarse sin ratón sin saber por qué).
### Eliminado
- **El historial en disco (`history/session-000.jsonl`) y toda su maquinaria** (`crates/store`).
  Nadie lo leía: la UI usa el historial en memoria y ADR-012 borra el fichero al arrancar, así que
  solo era un registro de depuración que se tiraba. Se borra el fichero que hubiera.

## [0.23.2] - 2026-10-31
### Cambiado
- **Los nueve emparejamientos se piden una vez al entrar**, no cada cuarto de hora. Hasta ahora el
  addon barría las nueve partidas cada 15 minutos «para localizarte», y eso no servía de nada: el
  `shard_id` de Mumble no identifica la partida (medido: `0x80000001`, no aparece en ninguna). Ese
  barrido queda para lo que sí hace falta —**validar tu equipo y llenar el selector**— al cargar el
  addon y cuando **cambias de personaje** (cada uno tiene su equipo). A partir de ahí, todos los
  sondeos van a la partida que tengas elegida y a ninguna otra.

## [0.23.1] - 2026-10-31
### Corregido
- **Coste del sondeo a 5 s**: el addon pedía los nueve emparejamientos (`ids=all`, ~400 KB) en
  **cada** sondeo, porque el indicador «el selector de equipo está en pantalla» estaba encendido
  casi siempre. A 5 s eso serían 4,8 MB por minuto. Ahora se piden los nueve solo cuando hace
  falta de verdad (sin equipo, al entrar —una vez— para validar tu equipo, o el selector abierto con la
  lista vieja) y como mucho una vez por minuto; el resto de sondeos van a una partida (~44 KB).

## [0.23.0] - 2026-10-31
### Añadido
- **Sondeo cada 5 segundos**, como la live map de gw2mists. El sondeo pide ahora **una** partida por
  id (`/v2/wvw/matches/{id}`, ~44 KB con sus **cuatro mapas**) en vez de resolver el equipo contra
  `ids=all` (~400 KB): a 5 s mueve menos datos por segundo que antes a 30 s. El intervalo se puede
  subir en Ajustes (5–600 s).
- **La ruta y el historial empiezan con lo que ya está reclamado** (ADR-012): los reclamos que
  existen al cargar entran en el historial con su hora real, así que un clan seguido enseña su
  ruta desde el primer minuto en vez de quedarse vacío hasta que vuelva a reclamar.
- **Clic en un clan seguido (o en un aviso) → pestaña Mapa** con su ruta dibujada. Antes
  seleccionaba el clan pero había que cambiar de pestaña a mano y parecía que no hacía nada.
- **Flecha de dirección** en el último tramo de la ruta y **un solo reclamo ya se dibuja** (punto
  numerado): es la única pista honesta de «hacia dónde va» sin inventar el recorrido (ADR-005).
- **«Solo los clanes que sigo»**: un interruptor único en Avisos. Antes había que destildar los
  cuatro mapas y «el mapa en el que estoy» uno a uno.
### Cambiado
- El atajo de *Añadir clan* se llama ahora «Con algo reclamado ahora», que es lo que enseña.
- Incluye todo lo de la 0.22.10, que no llegó a publicarse.

## [0.22.10] - 2026-10-31
### Corregido
- **Un reclamo caducado ya no pinta el anillo de un clan en un campa de otro color.** La API no
  borra `claimed_by` cuando el objetivo cambia de manos: solo un **nuevo reclamo** lo sustituye. Ahora
  un reclamo cuenta solo mientras el objetivo no haya dado la vuelta después (`last_flipped` anterior
  a `claimed_at`). Era lo que se veía como «YaK tiene reclamado un campa que en verdad era de otro
  color».
- **El click-through del menú del clic derecho ya hace algo.** `click_through` era el único campo que
  el cromo de la ventana no escribía de vuelta, así que la marca del menú se tiraba al final del
  fotograma.
- **El clic derecho devuelve el ratón aunque el click-through se activara en Ajustes.** La
  comprobación leía la configuración del arranque, que nunca ve los cambios hechos después.
- **Dejar de seguir un clan, de verdad.** La `×` se dibujaba después de una fila que ocupa todo el
  ancho, así que caía fuera de la ventana; ahora el hueco se reserva antes (como el `×` de la
  cabecera). Además la pestaña *Clanes* y el filtro de avisos leen la configuración viva y no la del
  arranque, así que la estrella y las listas cambian en el momento.
- **El buscador enseña los clanes que ya sigues.** Buscar «dolyak» en EU devuelve **otro** clan
  (`DYAK · Dolyak`), no el tuyo (`YaK · Dolyak Sanctuary`), así que la ventana decía «no existe» de un
  clan que ya estaba en tu lista. Ahora se buscan también los seguidos, sin red.
- **El equipo del personaje se guarda con su nombre.** Mumble publica `"Guild Wars 2"` en el campo
  `name` (el del juego, no el del personaje), así que el equipo quedaba guardado bajo una clave que
  no era el personaje. Ahora se lee del JSON de identidad.
- **El sondeo pide la partida del equipo que se va a vigilar.** Con la detección cacheada 15 minutos,
  se pedía la partida del equipo guardado y luego se evaluaba el equipo detectado: el addon miraba la
  partida equivocada casi todo el rato.
### Nota
- El `shard_id` de Mumble **no** identifica la partida: en el juego vale `2147483649`
  (`0x80000001`) y no aparece en ninguna de las nueve. La detección por Mumble no puede funcionar
  hasta encontrar otro dato; el log ya lo deja escrito (`deteccion: el shard … no está en ninguna
  partida`).

## [0.22.9] - 2026-10-31
### Corregido
- **El addon mira la partida en la que estás, no la del equipo guardado.** Hasta ahora vigilaba el
  emparejamiento del equipo que tuvieras en Ajustes; si no era el tuyo, las capturas y las
  reclamaciones del juego **no aparecían por ningún lado**, aunque el sondeo funcionara. Ahora, en
  cada arranque y cada 15 minutos, se pregunta a Mumble en qué mundo/equipo estás y se busca esa
  partida entre las nueve; la que diga el juego es la que se vigila. La cabecera lo dice:
  `EQUIPO … (lo dice el juego)`.
- Si el equipo guardado no era el tuyo, el log deja una línea `deteccion: el juego dice …; el equipo
  guardado era …` para no volver a adivinarlo.

## [0.22.8] - 2026-10-31
### Corregido
- **Click-through vuelve a funcionar**: la ventana hija del cuerpo no heredaba `no_inputs`, así que
  seguía capturando los clics aunque el ajuste estuviera activo.
- **Dejar de seguir un clan se puede confirmar**: la pregunta y sus dos botones (`Sí, quítalo` /
  `No, seguir siguiéndolo`) van en líneas separadas; en una sola se salían por la derecha.
- **El buscador de clanes enseña todo lo de la región**: el equipo elegido ya no descarta resultados
  (solo los ordena), y el mensaje de «no hay resultados» explica que la API busca por el principio de
  una palabra del nombre, no por el tag.
### Añadido
- **Aviso de equipo equivocado**: el addon compara el equipo guardado con lo que el juego publica por
  Mumble (`shard_id` y `team_color_id`) y, si no coinciden, lo dice en naranja en la cabecera y en el
  log (`donde: mumble shard …, color …`). Es la causa que ha roto los avisos todas las veces.
  El significado de `shard_id` en WvW queda por confirmar en el juego (T-19).

## [0.22.7] - 2026-10-31
### Cambiado
- **Fuera el aviso de Nexus**: el addon ya solo tiene **dos** cosas, el widget y el **aviso
  emergente** (la ventana de arriba). Tenía además un tercer mecanismo que pedía a Nexus que pintara
  su propio aviso; nunca se vio y solo servía para confundir, así que desaparece con su casilla.
- El aviso emergente ya enseñaba todo lo que sigues (tipo de aviso y alcance) y sigue igual: de uno
  en uno, con los segundos que pongas (0–20).

## [0.22.6] - 2026-10-31
### Cambiado
- **Se acabaron los códigos en la cabecera**: donde decía `EMPAREJAMIENTO 1-1 · NA` ahora pone
  **`PARTIDA EN NORTEAMÉRICA`** y debajo **quién juega**, con sus colores:
  `Verde · Invernadero de Lutgardis   Azul · …   Rojo · …`. El id interno (`1-1`) ya solo aparece en
  el log. El selector de equipo tampoco enseña el código: `Nombre · EU · mundos antiguos`.
- Mismo criterio que el mapa: la cabecera dice `ESTÁS EN: DESIERTO ROJO`, nunca `MAPA 1099 (TIPO 12)`.

## [0.22.5] - 2026-10-31
### Cambiado
- **«Aviso emergente» en vez de «toast»**: en los ajustes se llama «Aviso emergente (Nexus) de todo
  lo que sigo», con ayuda al pasar el ratón diciendo que lo dibuja Nexus encima del juego, aunque el
  widget esté cerrado.
- **Ese aviso ya no es solo de fortalezas y castillo**: sale cualquier aviso que pase tus filtros de
  tipo (capturas, reclamaciones, liberaciones) y de alcance, igual que el pop-up del addon.
### Añadido
- **«Último reclamo: hace N min»** en el widget, y lo mismo en el latido del log: es la marca del
  **juego**, no la nuestra, así que distingue «en esta partida no ha reclamado nadie» de «el addon
  se ha quedado parado» (informe de 2026-10-31).
### Corregido
- **El mapa ya no se queda con huecos**: las teselas se escribían directamente en su nombre final y
  el render podía leer un JPEG a medias; esas cuatro fallaban, gastaban el presupuesto de texturas de
  cada frame y las demás no llegaban a cargarse nunca. Ahora se descargan a un `.part` y se renombran,
  una tesela que falla se deja tranquila 5 s y el aviso sale una sola vez por tesela.

## [0.22.4] - 2026-10-31
### Corregido
- **El buscador de clanes vuelve a funcionar**: el API manda el equipo de cada clan como **cadena**
  (`{ "id-de-clan": "12008" }`), y el addon lo leía como número, así que fallaba con
  `LA BÚSQUEDA FALLÓ … INVALID TYPE: STRING "12008"`. Ahora se lee el texto y se convierte (y un
  número pelado sigue valiendo, por si el API vuelve a mandarlo así). Comprobado contra el API real:
  5 542 clanes en EU, todos con equipo válido.
- **El detalle de un objetivo sale encima del ratón**: antes se escribía debajo del mapa y había que
  hacer scroll para leerlo. Ahora es una etiqueta pegada al cursor, con fondo para que se lea sobre
  el mapa, y no se sale del recuadro.

## [0.22.3] - 2026-10-31
### Corregido
- **El pop-up enseña los avisos de uno en uno**: antes salían todos los de la ventana a la vez, así
  que dos reclamaciones seguidas se leían como una sola y el pop-up se iba de golpe. Ahora hay una
  cola: cada aviso dura lo que diga el ajuste y luego entra el siguiente.
- **El selector de segundos llega a 20** (antes 5), y el texto de ayuda explica que el pop-up va uno
  a uno y que la lista los guarda todos.
- **El mapa mantiene su proporción real**: se dibujaba `ancho × 280` y salía chafado. La frontera
  alpina es 2560×3584 y Eternos 3072×3072 (medido en `/v2/maps`), así que ahora el alto sale de
  `continent_rect` y el mapa se centra si tiene que estrecharse para caber.

## [0.22.2] - 2026-10-31
### Corregido
- **El mapa se salía de la ventana**: se pintaba encima de las pestañas y por debajo del widget. Al
  meter el cuerpo en una ventana hija (0.22.0) el mapa siguió usando la lista de dibujo del padre,
  que en imgui es la misma para la hija, así que le faltaba su propio recorte. Ahora el mapa y los
  iconos del registro se dibujan con `with_clip_rect_intersect` sobre su rectángulo.
- **La cabecera dice dónde estás con nombre**: «Estás en: Desierto rojo» en vez de «MAPA 1099
  (TIPO 12)». El id solo se enseña cuando no es un mapa de WvW, que es cuando sirve para algo.
- **«LAST UPDATE» estaba en inglés**: el código pedía la clave `Last update` y en la tabla solo
  estaba `Last update {0}`, así que caía al inglés en todos los idiomas.

## [0.22.1] - 2026-10-31
### Corregido
- **Dejar de seguir un clan pide confirmación**: el `×` de «Clanes seguidos» pregunta en la misma
  fila («¿Quitar *nombre* de los que sigues?» · Sí / No) antes de quitarlo, porque no hay deshacer
  (petición del jugador).

## [0.22.0] - 2026-10-31
### Corregido
- **El mapa ya no está del revés**: la proyección invertía la Y, así que el norte salía abajo y las
  teselas reales se colocaban en la fila de otra (eso era lo de «se ven 3 trozos, el norte y el sur
  intercambiados»). Medido contra `/v2/continents/1/floors/1` y comprobado dibujando las 16 teselas de
  Eternos, con cada objetivo sobre su estructura. Ver `07-spikes.md`.
- **La cabecera del widget no se va al hacer scroll**: el cuerpo se dibuja en su propia ventana con
  barra, así que el `×` y el menú del clic derecho (bloquear, click-through, opacidad) están siempre a
  mano aunque la pestaña de Ajustes sea larga.
### Añadido
- **Estilo de las ventanas (T-28)**: se puede elegir entre **heredar el de Nexus** (por defecto) o el
  **tema oscuro del addon**, que se aplica solo a nuestras ventanas. El fondo del widget puede ser
  oscuro semitransparente o **transparente**. Los colores de equipo salen ahora de un único sitio, así
  que son los mismos en el widget, el mapa y el historial.
- **Tamaño del texto y líneas del widget (RF-19c)**: escala de 0,8× a 1,6×, cuántas líneas enseña el
  widget y qué tipos de aviso se muestran (capturas, reclamaciones y liberaciones).
- **Aviso por toast para lo crítico (RF-19d)**: fortalezas y castillo pueden saltar también en el
  toast de Nexus. **Desactivado por defecto.** El sonido opcional no se puede: `nexus-rs` 0.12 no
  expone ninguna API de audio (comprobado en su código).
- **Reproducción de la ruta (RF-36)**: play/pausa y progreso en la pestaña Mapa; parada, la ruta se
  dibuja entera.
- **«Salto a <mapa>» (RF-35)**: cuando la ruta de un clan cruza de mapa, el texto lo dice.
- **Aviso de versión nueva (RF-50)**: si Nexus deja otra DLL en disco, el widget lo dice en naranja y
  pide reiniciar, con la ruta del fichero al pasar el ratón.

## [0.21.0] - 2026-10-31
### Corregido
- **La ventana se mueve y el `×` cierra**: en 0.18.0 la barra de título se había vuelto un botón
  invisible a toda la anchura que se quedaba con el clic, así que no se podía arrastrar y el `×`
  parecía plegar la ventana. Vuelta al título de siempre: `×` cierra, doble clic contrae y con la
  ventana contrada un clic la abre.
- **El registro de avisos va ligero**: solo se dibujan las filas visibles (`ListClipper`) y como mucho
  50, las más nuevas. Antes se pintaba todo lo de la sesión y cada línea son varios trozos de texto.
- **Los avisos llevan el nombre del clan**: se resuelven 20 nombres por sondeo en vez de 10.
### Añadido
- **El selector de equipo enseña los mundos antiguos de cada equipo**, que es el nombre que uno
  reconoce: los equipos nuevos se llaman como sitios donde nadie ha jugado y era fácil elegir el de
  otra región (pasó: un equipo NA llamado *Lutgardis Conservatory* teniendo el mundo Lutgardis en EU).

## [0.20.1] - 2026-10-31
### Corregido
- **El mapa real ya puede dibujarse entero**: el presupuesto de teselas era «4 en toda la sesión» en
  lugar de «4 por frame», así que el mapa se quedaba con las cuatro primeras y nada más (era el «el
  mapa no es real»). Si una textura no se puede crear, ahora queda dicho en el log.
- **Se puede saber si el sondeo funciona**: cada diez sondeos el registro escribe una línea con la
  partida, los objetivos reclamados y los eventos nuevos, que es lo que faltaba para distinguir «no
  llega nada» de «llega y se filtra».

## [0.20.0] - 2026-10-31
### Añadido
- **Nombres de equipo de verdad (T-45)**: `data/team-names.json` se rehace con las cuatro páginas de
  **gw2mists**. El **alemán** es el de la web, que no es el del Excel (`Grenths Tür`, `Ettinbuckel`,
  `Vogelhalle des Großen Hauses`, `Lutgardis-Konservatorium`…), el **inglés** el de la tabla de ids y
  el **español y francés** los del Excel cuando el nombre está en la hoja; los que no, se quedan en
  inglés para que los traduzcas tú en los ajustes. Tus traducciones propias siguen mandando.
- La web solo lista los 27 equipos que están jugando; los 6 restantes se quedan en inglés.

## [0.19.0] - 2026-10-31
### Añadido
- **Clicar un clan enseña su ruta**: una línea de aviso o un clan de *Clanes seguidos* se puede
  clicar; se selecciona ese clan y el mapa salta al **último donde reclamó algo**, así que en la
  pestaña Mapa aparece su ruta sin buscarla. Un aviso lo dice al pasar el ratón.
- **Los avisos se leen de un vistazo**, como pediste: icono del objetivo, el **mapa en su color**
  («Eternos» en gris, «Azul» en azul, «Verde» en verde, «Rojo» en rojo) y, ya en gris, el **sitio**,
  un guion y el **clan que lo tiene reclamado ahora**. Antes toda la línea iba del color del que
  había capturado, que escondía justo lo que interesa.
- **El hover del mapa dice el clan**, no su id.

### Cambiado
- El diagnóstico del worker se muda a `diagnostics.rs` (el fichero estaba en el límite de líneas).

## [0.18.1] - 2026-10-31
### Corregido
- **El atajo de teclado ya abre el widget**: se registraba por cadena (y una cadena que Nexus no
  acepta se registra en silencio) y además solo alternaba click-through, así que con la ventana
  cerrada no pasaba nada visible. Ahora se registra por código de tecla y lo que hace es **traer la
  ventana**: la muestra, la despliega y le quita el click-through. El registro deja constancia en el
  log del atajo y de cada pulsación.
- El aviso de los ajustes dice lo que hace el atajo de verdad.

## [0.18.0] - 2026-10-31
### Añadido
- **La pestaña Clanes es donde se sigue a los clanes**: empieza por **Clanes seguidos** (con la ×
  para dejar de seguir) y debajo **Añadir clan**, con los **10 últimos que han reclamado algo** en
  esta sesión y el buscador. En los ajustes queda una línea que dice dónde está.
- **El buscador de clanes empieza por la región** (EU/NA, con la tuya por defecto y cambiable) y los
  equipos se enseñan con su **nombre**, no con el número.
### Corregido
- **Las horas son las tuyas**: se pintaban en UTC, así que todo salía dos horas antes en España. El
  almacenamiento sigue en UTC (es lo correcto para comparar) y la conversión se hace al dibujar, en
  avisos, registro, clanes, emparejamiento y mapa.
- **Al reiniciar, el registro empieza limpio**: lo que ya estaba reclamado solo avisa si es de un
  clan seguido **y** de los últimos diez minutos.
- **La barra de título entera responde**: doble clic contrae y despliega, y con la ventana contrada
  un solo clic la abre.
- **Un id que no es de clan ya no se pide a la API** (era el origen del `invalid type string …`), y
  si la búsqueda falla el mensaje dice qué se buscó y en qué paso.

## [0.17.4] - 2026-10-31
### Corregido
- **El clic derecho funciona a la primera**: con click-through la ventana no apuntaba su caja, así que
  hasta haberla movido alguna vez no había dónde acertar. Ahora la caja se guarda también en
  click-through (no se puede arrastrar allí, así que no hay nada que discutir); bloqueada se sigue
  respetando.

## [0.17.3] - 2026-10-31
### Añadido
- **El widget funciona en el Bastión del Armisticio**: antes se ocultaba por no tener objetivos. Ahora se
  dibuja (es donde están los ajustes) y no se sondea nada; la cabecera lo explica. La Fortaleza del Valor
  y el Sagrario de Obsidiana siguen ocultos.
- **El clic derecho saca del click-through**: con click-through la ventana no recibe nada del ratón, así
  que la única salida era el atajo o el panel de Nexus. Un clic derecho dentro del widget lo desactiva y
  lo guarda.
### Corregido
- El aviso del click-through decía `Alt+Shift+L` (escrito a mano) y se enseñaba según el interruptor de
  «bloquear posición» en vez de según el de click-through. Ahora usa el atajo real y sale cuando toca.

## [0.17.2] - 2026-10-31
### Corregido
- **El mapa real podía quedarse vacío toda la sesión**: la descarga de teselas se daba por hecha en
  la primera llamada, y esa llamada puede llegar antes que `/v2/maps` (sin mapas no hay nada que
  pedir). Ahora solo se da por hecha cuando ya hay cajas de mapa, y el registro dice cuántas teselas
  se piden y a qué continente.

## [0.17.1] - 2026-10-31
### Corregido
- **Los clanes seguidos ya dicen su nombre**: se pedían a la API solo los clanes que habían reclamado
  algo en el emparejamiento, así que un clan seguido sin reclamos se quedaba como id crudo en los
  ajustes y en la lista de añadir.
- **Los rótulos de los ajustes ya no se cortan**: la columna de los controles se calculaba con una
  frase larga y no con los rótulos reales, así que «Intervalo de sondeo» acababa en
  «INTERVALO DE SON…».
- **El buscador de clanes funciona**: la región es lo único obligatorio (el equipo pasa a ser un
  filtro opcional), el bloque pide la lista de equipos mientras está en pantalla y, si aún no ha
  llegado, lo dice en vez de no responder. El campo de texto va antes del botón.
- **Las pestañas ya no se pierden**: la barra usa desplazamiento, así que con la ventana estrecha
  aparecen las flechas para llegar a las últimas.
- **Se ve qué emparejamiento se está vigilando**: la cabecera dice la región («Matchup 1-1 · NA») y
  avisa en naranja si el equipo guardado ya no juega. El registro escribe una línea por evento
  detectado y otra cuando cambia el emparejamiento, que es lo que faltaba para saber si un aviso no
  llega o si llega y se filtra.
- **El mapa real ya no puede dibujar océano**: el nombre del fichero de cada tesela lleva el
  continente (`2-1-…`) y las teselas de versiones anteriores se borran al arrancar.

## [0.17.0] - 2026-10-31
### Añadido
- **Iconos del mapa como en el juego (T-46)**: cada objetivo es un **círculo del color del equipo con el
  dibujo dentro** (tienda de campaña, torre, fortaleza con almenas y castillo con dos torres), dibujado
  con primitivas y sin assets. El tamaño también cambia por tipo, así que se distingue sin depender del
  color. El marcador de «alto valor» y el anillo del clan seguido se mantienen.
- **Icono del tipo en los avisos (T-47)**: cada línea del widget y del pop-up lleva delante el dibujo de
  lo que se ha tomado, en el color de la línea.

## [0.16.1] - 2026-10-31
### Corregido
- **El mapa real salía mal encuadrado (zoom y continente)**: se pedían las teselas del **continente 1**,
  que responde con una imagen de océano para cualquier coordenada, y se suponía que a zoom 5 una
  tesela cubre 2048 unidades de continente cuando son **1024**. Ahora se piden al **continente 2,
  piso 1** y `units_per_tile(zoom) = 256 · 2^(7 − zoom)`, así que cada mapa son 4×4 teselas (59 en
  total para los cuatro) y el fondo cae donde toca. Verificado descargando las 59.

## [0.16.0] - 2026-10-31
### Añadido
- **Mapa real (T-29)**: las teselas oficiales de `tiles.guildwars2.com` a zoom 5 (~20 imágenes para
  los cuatro mapas) descargadas por el worker a `<addon>/tiles/` y dibujadas **con la opacidad de
  la ventana**. Las que faltan se van rellenando solas, sin bloquear el render. Interruptor
  **Mapa real** en los ajustes; apagado, vuelve el tablero esquemático.
- **Rango temporal de la ruta (T-16)**: *Última hora*, *Últimas 2 horas* o *Toda la sesión*.
- **Probar pop-up** en los ajustes y pop-up sin marco, arrastrable y con posición guardada.
### Corregido
- El pop-up no caducaba nunca (el instante de recepción se recalculaba cada frame).
- El aviso empieza por el mapa.

## [0.15.0] - 2026-10-31
### Corregido
- **El pop-up ya se va solo**: el instante de recepción se recalculaba en cada frame, así que
  ninguna alerta envejecía nunca. Ahora se guarda con el evento, el pop-up se apaga a los segundos
  elegidos y se desvanece en el último medio segundo.
- **El pop-up parece un pop-up**: sin marco ni barra de título, y se puede arrastrar a donde
  quieras (la posición se guarda). Botón **Probar pop-up** en los ajustes para colocarlo sin
  esperar a una captura.
- **El pop-up dice primero el mapa** de la alerta: es lo que decide si te importa.
- **El mapa tiene fondo**: tablero oscuro con rejilla, borde y el nombre del mapa, y **al pasar el
  ratón por un objetivo** se ve su dueño, si es de alto valor, el clan y la hora. Las teselas
  reales siguen siendo T-29 (necesitan texturas y caché).
### Añadido
- **Selector de clanes a seguir rediseñado (T-44)**: la sección enseña solo los que sigues, con una
  × para dejar de seguir; *Añadir clan* ofrece primero **los que han reclamado desde que arrancó**
  (un clic, sin red) y un **buscador por equipo y nombre** (el equipo, obligatorio y de tu región).
  Máximo 10.
- **Tu nombre para cada clan (T-43)**: en la pestaña de clanes, el clan elegido tiene un campo para
  escribir su nombre o traducción; se enseña detrás del nombre real y manda sobre la API.
- **El tag del clan seguido** se dibuja junto a su anillo en el mapa (T-15).

## [0.14.6] - 2026-10-31
### Cambiado
- La línea de carga del registro dice la **versión** (`WvW Lurker v0.14.6 cargado…`). Sin eso no habia
  forma de saber que build estaba corriendo al mirar `Nexus.log`.

## [0.14.5] - 2026-10-31
### Corregido
- **Cierre al abrir la pestaña Mapa, por fin con causa**: `get_window_draw_list()` solo se puede
  pedir **una vez por frame** — la segunda llamada lanza un pánico de `arcdps-imgui`
  («You can only load one instance of it!») y se llevaba el addon. Se pedía una vez por objetivo,
  así que el mapa cerraba el juego en cuanto había algo que dibujar. Ahora se pide una vez y se
  pasa por referencia.
- El `catch_unwind` de 0.14.3 hizo su trabajo: el pánico quedó en `Nexus.log` en vez de una
  pantalla negra, y por eso se ha podido identificar.

## [0.14.4] - 2026-10-31
### Añadido
- **Máximo 10 clanes seguidos** (`MAX_FOLLOWED`): la lista deja de poder crecer sin freno y avisa
  cuando está llena, tanto en los ajustes como en la pestaña de clanes. El rediseño del selector
  con buscador está apuntado como T-44.

## [0.14.3] - 2026-10-31
### Corregido
- Tras un pánico dibujando el widget, el addon deja de dibujarlo en esa sesión en vez de repetir el
  fallo cada frame. El motivo queda en `nexus.log`.

## [0.14.2] - 2026-10-31
### Corregido
- **«Bloquear» y «click-through» vuelven a ser dos cosas distintas**: bloquear impide mover o
  redimensionar pero la ventana sigue aceptando clics; click-through deja pasar todos los clics al
  juego. Antes eran el mismo interruptor y por eso no se podían apuntar los dos estados.
- **El atajo ya no es `Alt+Shift+L`**: ese es el menú legacy de arcDPS y el juego se lo queda.
  Ahora es **`Ctrl+Shift+L`**, y sigue siendo reasignable en Nexus.
- Un pánico dibujando una pestaña ya no cierra el juego: se captura, se anota en `nexus.log` y el
  overlay sigue vivo (RNF-06). La pestaña *Mapa* se cerró una vez al abrirla.

## [0.14.1] - 2026-10-31
### Corregido
- **La ventana no respondía al ratón**: el modo click-through se había quedado guardado y no había
  ninguna señal visible, así que parecía un addon colgado. Ahora, con click-through activo, la
  ventana lo dice en naranja con el atajo, y las opciones de Nexus tienen su propia casilla, de
  modo que siempre hay una salida sin depender del teclado.
- **La lista de clanes seguidos enseñaba identificadores**: ahora muestra `[TAG] Nombre`.
- Un clan **seguido** vuelve a avisar de sus claims aunque el objetivo ya estuviera reclamado al
  arrancar (RF-21, RF-25). Los de clanes que no sigues siguen sin aparecer.

## [0.14.0] - 2026-10-31
### Añadido
- **Aviso de configuración dañada** (T-26, RF-51): si `settings.json` no se puede leer, el addon
  arranca con los valores por defecto y ahora lo dice en rojo encima de la ventana, en vez de
  dejarlo solo en el registro. El fichero dañado se conserva como `settings.json.corrupt-<fecha>`.

## [0.13.2] - 2026-10-31
### Corregido
- **«Alto valor» ahora sale del dato, no del tipo**: `ClaimRow` guarda el `points_tick` del
  snapshot, así que una torre totalmente mejorada (tick 8) cuenta como alto valor y un campamento
  no. Antes se decidía solo por tipo y las torres mejoradas se quedaban fuera (RF-30b).

## [0.13.1] - 2026-10-31
### Añadido
- **Ruta numerada sobre el mapa** (T-16): al elegir un clan en la pestaña Clanes, el mapa dibuja
  sus reclamaciones de la sesión en orden temporal, unidas por una línea, con el número de cada
  paso y un anillo en la más reciente. El aviso de «ruta inferida» (ADR-005) se pinta debajo.
### Corregido
- `worker.rs` pasaba de 800 líneas: los catálogos de la API (objetivos, mapas y nombres de clan)
  se mueven a `catalog.rs`.

## [0.13.0] - 2026-10-31
### Añadido
- **Pestaña Mapa** (T-14): mapa simplificado de cada uno de los cuatro mapas de Mundo contra Mundo,
  con los objetivos como figuras coloreadas por equipo —círculo campamento, cuadrado torre, rombo
  fortaleza, triángulo castillo—, contorno doble en los de alto valor y anillo dorado en los de las
  guilds que sigues. Selector de mapa, con el mapa en el que estás marcado y elegido por defecto.
  Sin texturas ni teselas (ADR-002). La lista de objetivos del matchup sigue ahí, plegada.
- `Api::maps` y `core::project` (`Frame`, `project`): proyección pura desde `continent_rect`, con
  tests contra las fixtures reales.

## [0.12.1] - 2026-10-31
### Corregido
- Al reiniciar el juego, la lista de avisos y el historial de clanes seguían mostrando claims de
  antes: lo que ya estaba reclamado al arrancar se apuntaba como si acabara de pasar. Ahora eso
  no entra ni en los avisos ni en el historial de la sesión.
- En disco solo se guardan los claims de las guilds que sigues; el resto es el estado del
  matchup, que la API devuelve en cada sondeo.

## [0.12.0] - 2026-10-31
### Cambiado
- **Una sola ventana con pestañas**, como el boceto: **Avisos · Mapa · Clanes · Registro** y, al
  final, **Ajustes**. Desaparecen las cinco ventanas sueltas y su estado por ventana: ahora la
  ventana se coloca, se bloquea y se opaca una vez.
- Las **opciones de Nexus** empiezan con **Ver la ventana** y **Avisos emergentes**, y siguen
  mostrando los ajustes completos: es la vía de vuelta aunque la ventana esté cerrada.
### Añadido
- Interruptor de **avisos emergentes** (el pop-up) dentro de la pestaña de ajustes, con su
  duración de 0 a 5 s. El pop-up se dibuja aunque la ventana esté cerrada.

## [0.11.3] - 2026-10-31
### Corregido
- Los clanes se veían como un identificador (`9FDB4A86-1588-…`) en vez de su nombre. Ahora se
  resuelven con la API y salen como `[TAG] Nombre` en las tres ventanas.
- Elegir un clan en la lista parecía no hacer nada: la historia se dibujaba debajo de una lista
  más alta que la ventana. La lista tiene su propia zona con scroll y la historia queda a la vista.
- El widget de avisos se quedaba con todo lo de la sesión. Ahora enseña los últimos 10 minutos;
  el resto está en el nuevo registro de avisos.
- Dentro de Mundo contra Mundo el selector de equipo solo ofrecía el equipo actual: el addon
  pedía únicamente el matchup del jugador. Ahora pide los nueve mientras el selector está abierto.
### Añadido
- Casilla **Clanes que sigo** en la ventana de objetivos del matchup.

## [0.11.2] - 2026-10-31
### Añadido
- **Modo click-through**: una ventana bloqueada sigue dibujándose pero deja pasar todos los clics
  al juego, que es lo que se necesita en mitad de una pelea. Se bloquea y desbloquea todo el
  overlay con **Alt+Mayús+L** (atajo de Nexus, reasignable allí) o con la casilla de cada ventana
  en Ajustes → Ventanas.
- **Ventana de registro de avisos**, con la lista completa y scroll, para leer después lo que el
  widget compacto solo deja entrever. Se abre desde Ajustes → Ventanas.
### Corregido
- Los interruptores de la ventana de Ajustes de Nexus (su panel de opciones) no guardaban los
  avisos, el pop-up ni los clanes seguidos.

## [0.11.1] - 2026-10-31
### Corregido
- Una ventana cerrada con la × no se podía volver a abrir. Ahora los ajustes tienen una sección
  **Ventanas** con casillas para Avisos, Objetivos del matchup y Clanes.

## [0.11.0] - 2026-10-31
### Añadido
- **Ruta inferida** de un clan: al elegirlo en la ventana de clanes se ve de dónde se supone que
  vino (su fortaleza, una fortaleza lateral o el punto de reaparición) y los objetivos que fue
  reclamando en orden. Va siempre etiquetada como inferida, nunca como recorrido real.

## [0.10.1] - 2026-10-31
### Corregido
- La traducción del equipo se guardaba por idioma y no por equipo, así que al elegir otro servidor
  aparecía la del anterior y al volver al primero se había perdido. Ahora cada equipo conserva
  la suya y el campo propone la traducción incluida cuando no hay una propia.
### Añadido
- Botón **Grabar** en los ajustes, que escribe settings.json en el momento.

## [0.10.0] - 2026-10-31
### Añadido
- **Ventana de clanes**: los que tienen algo reclamado, ordenados por lo más reciente, con su
  número de claims y el mapa de la última. La estrella sigue a un clan, de modo que avisa en
  cualquier mapa del emparejamiento.
- **Historial por clan**: al elegir uno, sus reclamaciones de la sesión separadas entre el mapa
  en el que estás y los demás.
### Cambiado
- El objetivo de tamaño de la DLL pasa a 2,2 MB (ADR-013); el límite sigue en 2,5 MB.

## [0.9.1] - 2026-10-31
### Corregido
- El selector de equipo ya no muestra «EU (0)» / «NA (0)» sin datos: la lista de equipos se
  pide también con los ajustes abiertos fuera de Mundo contra Mundo, y el número solo aparece
  cuando se conoce.
### Cambiado
- Quitada la dependencia del RealTime API de Nexus, que no se usaba (el idioma llega por
  Unofficial Extras).

## [0.9.0] - 2026-10-31
### Añadido
- **Claims del matchup**: ventana con los cuatro mapas de tu emparejamiento, cada objetivo
  reclamado con su clan y la hora, y el mapa en el que estás marcado. Filtros por campamentos,
  torres, fortalezas y «solo alto valor».
### Cambiado
- El log de arranque dice ahora qué equipo ve arcdps, útil para diagnosticar.

## [0.8.1] - 2026-10-31
### Corregido
- El desplegable **Región** del selector de equipo salía vacío hasta que llegaba el primer
  sondeo. Ahora EU y NA están siempre, con cuántos equipos tiene cada una.
- La ventana de ajustes nace más alta para que quepan los nuevos filtros de avisos.

## [0.8.0] - 2026-10-31
### Añadido
- **Selector de avisos**: qué mapas te avisan (Eternos, Alpino azul, Alpino verde, Desierto rojo),
  «el mapa en el que estoy» y «los clanes que sigo». Todo marcado por defecto. Un clan seguido
  avisa en cualquier mapa del emparejamiento.
- **Aviso emergente** con duración configurable entre 0 y 5 segundos (2 por defecto): desaparece
  solo, pero el aviso sigue en la lista.
- Lista de **clanes a seguir** en los ajustes, sacada de los que tienen algo reclamado.

## [0.7.0] - 2026-10-31
### Añadido
- **Widget de avisos**: lista de las últimas capturas, reclamaciones y liberaciones, con la hora,
  el nombre del objetivo y el color del nuevo dueño. Se mueve, se pliega y se le ajusta la
  opacidad como al resto de ventanas.

## [0.6.0] - 2026-10-31
### Cambiado
- El campo «Alias» pasa a llamarse **«Traducción»** y viene relleno con el nombre del equipo en
  el idioma del juego. Puedes corregirlo y se guarda por idioma, para que cambiar el idioma del
  juego no te muestre el nombre de otro. Si lo dejas como está (o lo vacías), se sigue usando la
  traducción incluida en el addon.

## [0.5.1] - 2026-10-31
### Añadido
- Nombres de equipo traducidos a español, francés y alemán, extraídos de `data/equipos.xlsx`
  con `scripts/merge_team_names.ps1`. Se resuelven por el nombre inglés, así que siguen
  sirviendo cuando un equipo cambia de nombre la temporada siguiente.
  Aviso: son traducciones literales; donde no coinciden con el nombre del cliente se sigue
  viendo el inglés.

## [0.5.0] - 2026-10-31
### Añadido
- **Selector de equipo en tres pasos**: región (EU/NA) → color (rojo/verde/azul) → equipo. Con
  los datos actuales pasa de 27 equipos a 5 en EU y 4 en NA, y cada fila dice su matchup.
  Arranca en la región y color del equipo ya guardado, y avisa si aún no hay datos o si esa
  combinación no tiene equipos.
- El alias del equipo se explica en la propia ventana (solo afecta a tu instalación).

## [0.4.3] - 2026-10-31
### Cambiado
- Los ajustes se guardan en `<Documentos>\Guild Wars 2\addons\gw2_lurker\settings.json`, como
  los addons vecinos, en vez de junto a la DLL (que está en Program Files y exigía permisos).
  Se migra solo el fichero antiguo la primera vez.
### Corregido
- El equipo recordado por personaje usaba como clave el JSON de identidad completo de Mumble en
  lugar del nombre del personaje.

## [0.4.2] - 2026-10-31
### Corregido
- **Cuelgue al entrar en un mapa.** La ventana de ajustes pedía el equipo con una función
  que bloquea la configuración mientras el render ya la tenía bloqueada: std::sync::Mutex
  no es reentrante y el juego se congelaba al dibujarse la primera vez tras la pantalla de
  selección. Ahora esa consulta se hace sobre la referencia que ya se tiene (*_in).

## [0.4.1] - 2026-10-31
### Cambiado
- La opacidad se ajusta en **porcentaje (0–100 %)** en el menú de cada ventana, no en
  fracciones de 0 a 1. Se sigue guardando como 0.0–1.0, que es lo que espera imgui.

## [0.4.0] - 2026-10-31
### Añadido
- **Sondeo real del matchup**: un hilo aparte consulta la API (400 KB con `ids=all` la primera
  vez, 44 KB por matchup cuando ya hay equipo) cada 30 s, aplica la detección de cambios, guarda
  el histórico de la sesión y publica claims y clanes para la UI. Nunca toca la red ni el disco
  desde el hilo de render.
- La ventana de estado enseña emparejamiento, objetivos reclamados, clanes y hora de la última
  actualización, o el error si la API falla.
- El equipo elegido se recuerda **por personaje**, así que una cuenta alt no hereda el equipo
  del main.

## [0.3.2] - 2026-10-31
### Cambiado
- El idioma sale ahora del juego de verdad: se escucha el evento de idioma de Unofficial Extras
  (que también llega al arrancar) y, si no está, se lee el idioma que usa Nexus de sus propios
  ficheros. El idioma de Windows queda como último recurso antes del inglés.

## [0.3.1] - 2026-10-31
### Cambiado
- Los ajustes se guardan como `settings.json` dentro de la carpeta del addon, igual que los
  proyectos vecinos, con `settings.json.bak` y escritura atómica. Se sigue leyendo el antiguo
  `config.json` para no perder nada.
### Añadido
- Los ajustes también aparecen en el panel de opciones de Nexus, accesible fuera de WvW.
- Escritura de ajustes fuera del hilo de render (hilo propio, como mucho dos guardados por
  segundo) y `debug_show_everywhere` para diagnosticar por qué no se dibuja nada.
- Si Nexus no tiene instalado el RealTime API, el idioma se toma del idioma de Windows en vez
  de caer directamente a inglés (la preferencia explícita del usuario sigue mandando).

## [0.3.0] - 2026-10-31
### Añadido
- Ventana de ajustes: selector de equipo con los 33 equipos reales (`nombre (id)`), alias,
  idioma (Auto/es/en/fr/de), intervalo de sondeo y alcance de los avisos. Se abre desde el
  botón *Ajustes* de la ventana de estado y todo se guarda en `config.json`.

## [0.2.0] - 2026-10-31
### Añadido
- Addon cargable en Nexus (`gw2_lurker.dll`) que solo se muestra en Mundo contra Mundo y se
  oculta en pantallas de carga y selección de personaje.
- Ventana de estado con el contrato de ventana compartido: mover, redimensionar, plegar,
  bloquear, opacidad y menú contextual; todo se guarda en `config.json`.
- Traducciones a español, inglés, francés y alemán; el idioma por defecto es el del juego y
  mantener pulsado **Ctrl derecho** lo muestra todo en inglés.
- Publicación automática de la DLL y su `.md5` en el repositorio de distribución.
- Núcleo puro con detección de cambios (`diff`), equipos, matchup, favoritos y caché en disco
  de objetivos por idioma.

## [Sin publicar]
### Añadido
- Especificación inicial del proyecto.
