# Changelog
Formato [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/), versionado [SemVer](https://semver.org/lang/es/).

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
