# Changelog
Formato [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/), versionado [SemVer](https://semver.org/lang/es/).

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
