# Changelog
Formato [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/), versionado [SemVer](https://semver.org/lang/es/).

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
