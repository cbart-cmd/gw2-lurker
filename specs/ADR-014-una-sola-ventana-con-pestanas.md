# ADR-014 · Una sola ventana con pestañas

- **Estado**: aceptado (2026-10-31); enmendado el 2026-11-02: el selector de equipo pasa a ser una
  ventana aparte, centrada, que se abre sola al entrar en MvM o en el Bastión (RF-03j), por
  petición expresa del jugador. Es la única excepción; el resto sigue en la ventana con pestañas.
- **Decide**: el usuario, tras ver el addon funcionando y compararlo con `docs/boceto.html`
- **Afecta a**: RF-17, RF-18, RF-46; T-42

## Contexto

El addon acabó con cinco ventanas sueltas (estado, avisos, objetivos del matchup, clanes y
registro), cada una con su propio estado guardado. Funciona, pero no se parece al boceto
aprobado —una sola ventana con pestañas— y obliga a colocar y bloquear cinco ventanas para
jugar cómodo: en mitad de una pelea eso es ruido, y el estado por ventana se multiplica en
`settings.json`.

## Decisión

1. **Una única ventana** (`WvW Lurker`) con barra de pestañas: **Avisos · Mapa · Clanes ·
   Registro · Ajustes**. Los ajustes son siempre la **última** pestaña.
2. Un **único estado** de ventana (`visible`, `collapsed`, `locked`, `opacity`, `position`,
   `size`) para toda la ventana. Se pierde el estado por vista; nadie lo usaba y era lo que
   hacía que «cerrar una ventana» no se entendiera.
3. Los **ajustes** dejan de ser una ventana: se dibujan como pestaña dentro del widget y siguen
   apareciendo en las opciones de Nexus (`RenderType::OptionsRender`), que es la vía de vuelta
   cuando la ventana está oculta.
4. Las **opciones de Nexus** ganan dos interruptores al principio: **Ver la ventana** y
   **Avisos emergentes**. El segundo es un campo nuevo, `popup_enabled`, independiente de
   `popup_secs`, para apagar y encender los pop-ups sin perder la duración elegida.
5. El **pop-up** se dibuja fuera de la ventana y no depende de que la ventana esté visible: es
   justo lo que se quiere cuando se juega con el overlay cerrado.

## Consecuencias

- `WindowState` sigue existiendo, con una sola entrada. Los estados antiguos de las ventanas
  retiradas se quedan en el fichero sin usarse; no se borran (no molestan y borrarlos es
  arriesgar datos del usuario por nada).
- `ui::window_keys()` desaparece: solo hay una clave (`ui::WIDGET`).
- El atajo `Alt+Shift+L` (RF-18) sigue alternando el click-through, ahora de la única ventana.
- Las vistas puras (`alerts`, `matchup`, `guilds`, `settings`) siguen separadas en módulos y
  exponen un `body(…)` sin cromo: la ventana es una, el código no tiene por qué serlo.
