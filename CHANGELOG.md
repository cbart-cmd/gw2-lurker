# Changelog
Formato [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/), versionado [SemVer](https://semver.org/lang/es/).

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
