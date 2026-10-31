# Changelog
Formato [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/), versionado [SemVer](https://semver.org/lang/es/).

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
