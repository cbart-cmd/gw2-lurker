# PRODUCT.md

Contexto de producto para las herramientas de diseño (`impeccable`). Escrito a partir del
brief del usuario y de la spec; **las suposiciones están marcadas**.

## Qué es

**gw2-lurker**: addon para Guild Wars 2 (WvW) que avisa de capturas y sigue guilds enemigas
mostrando su historial de claims. Se carga a través de Raidcore Nexus.

## Para quién

Jugadores de Mundo contra Mundo que quieren saber **qué está pasando en su matchup** sin
mirar el mapa del juego, y **qué guilds** están moviéndose. Uso principal: escuadrón en
combate, pantalla completa, atención repartida.

## Trabajo a resolver (por orden)

1. Saber al instante que han perdido/tomado algo importante, sin mirar.
2. Ver qué está reclamado en los cuatro mapas del matchup.
3. Identificar qué guilds están activas y cuál se mueve más.
4. Reconstruir por dónde ha pasado una guild.

## Superficies

- **Ventana única con pestañas** (ADR-014): Avisos · Mapa · Clanes · Registro · Ajustes.
- **Widget de alertas** (pestaña Avisos): siempre en pantalla, 2–5 líneas, bloqueable; en
  click-through los clics pasan al juego y se sale con el botón derecho.
- **Mapa**: esquemático, con filtros por tipo y marca de alto valor.
- **Clanes**: los que sigues (10 como mucho) y un buscador por subcadena sobre un índice propio.
- **Historial de la sesión**: ruta inferida con origen deducido.

## Restricciones reales

- Vive dentro del juego: imgui de Nexus, no HTML. El HTML del boceto es **solo validación**.
- El mapa se dibuja con primitivas; las teselas son opcionales y se cachean (ADR-002, ADR-011).
- Textos en es/en/fr/de (RF-44), con fr y de de primera clase, y **solo ASCII + latino-1**: la fuente
  de Nexus no tiene más glifos (lo que no tiene, sale como «?»).
- Nada de info que la API no dé: sin «bajo ataque», sin posición de jugadores ajenos.
- El color nunca es la única señal (RF-19, RF-62).

## Éxito del boceto

Que el usuario pueda decir **"así sí, así no"** sobre cuatro decisiones: jerarquía del mapa,
forma de marcar alto valor, cómo se lee la lista de guilds y cómo se entiende la ruta. Si
hay que explicar el boceto, ha fallado.

## Suposiciones declaradas

- *Suposición:* el boceto se valida en escritorio (navegador del PC donde se juega).
- *Suposición:* tono oscuro sobrio tipo GW2 (elegido por el usuario), no tema claro.
- *Suposición:* los datos del boceto son reales del match `2-2` capturado el 2026-10-31
  (mapa EBG), no inventados.
- *Suposición:* el boceto no implementa persistencia ni red; los controles solo demuestran
  el comportamiento visual.
