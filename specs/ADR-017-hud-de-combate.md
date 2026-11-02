# ADR-017 · HUD de combate separado de la consola

- **Estado**: aceptado (2026-11-02), pendiente de implementar (T-55)
- **Decide**: el jugador, tras la revisión de UX de `docs/bocetos/ux/propuestas.html` (propuesta 2)
- **Afecta a**: RF-16, RF-17, RF-43, RF-46, RF-47; ADR-014 (enmienda parcial)

## Contexto

ADR-014 juntó cinco ventanas en una sola con pestañas. Eso arregló el desorden, pero la revisión
de UX (Nielsen 22/40) mostró otro problema: durante una pelea lo único que importa son los
avisos, y la ventana completa mete 6–10 líneas de cabecera (equipo, región, sondeo, estado de la
API, índice) antes de las pestañas. Para ver los avisos hay que tener abierta la ventana grande
o confiar en el emergente, que dura unos segundos. Tampoco había un sitio fijo y pequeño donde
ver que la API lleva minutos parada (RF-43).

## Decisión

1. Dos superficies: un **HUD de combate** pequeño y fijo (RF-47) y la ventana con pestañas, que
   pasa a llamarse **consola**.
2. El HUD tiene una línea de estado (equipo en texto y color, región, mapa, antigüedad de los
   datos, aviso de API parada ≥ 5 min) y las últimas 2–5 líneas de aviso con el formato de RF-19f.
3. La consola conserva Mapa, Clanes, Registro y Ajustes. Pierde la pestaña Avisos, porque su
   contenido es el HUD, y la cabecera de estado se pliega en una línea.
4. Cada superficie tiene su propio estado de ventana con el contrato de RF-46 (posición, tamaño,
   opacidad, bloqueo, click-through con `Alt+Shift+L`).
5. El emergente (RF-19d) sigue aparte: es el aviso que llama la atención; el HUD es lo que se
   consulta de un vistazo.

## Alternativas descartadas

- **Mantener una sola ventana y compactar la cabecera.** Ayuda, pero sigue obligando a elegir
  entre ver el mapa o ver los avisos, y a tener una ventana grande abierta en la pelea.
- **Usar solo el emergente.** Desaparece a los `popup_secs` y no deja ver el estado de la API.
- **Volver a las ventanas sueltas de antes de ADR-014.** Era justo lo que el jugador rechazó.

## Consecuencias

- ADR-014 queda enmendado: «una sola ventana» pasa a ser «una consola con pestañas más el HUD
  y el selector de equipo (RF-03j)».
- `settings.json` gana una segunda entrada de `WindowState` (la del HUD). La primera vez se crea
  arriba a la izquierda, con la opacidad de la consola. Los ajustes antiguos siguen siendo válidos.
- Las opciones de Nexus ganan el interruptor **Ver el HUD**, junto a **Ver la ventana**.
- La lista de avisos (`ui::alerts::body`) se reutiliza en el HUD; no se duplica la lógica.
