# Bugs

Bandeja de entrada de fallos. **No se borra este fichero**: cada bug se marca como `hecho`
cuando está resuelto, con la versión en la que salió el arreglo.

Formato:

```
## [ ] Título corto (fecha en que se reporta)
- Qué pasa, cómo se reproduce, qué se esperaba.
- **Hecho (versión):** qué se cambió y dónde.
```

Estado: `[ ]` pendiente · `[x]` hecho.

---

## [x] El traductor no guarda la traducción (2026-10-31)
- Al desplegar un servidor, permite escribir la traducción, pero no propone la que pusimos; no hay
  botón de grabar; al cambiar a otro servidor aparece la traducción del anterior; y al volver al
  primero no se había guardado.
- Causa: la traducción se guardaba **por idioma**, no por servidor, así que era la misma para
  todos los equipos. Y no había ninguna acción explícita de guardar.
- **Hecho (0.10.1):** `Config::team_translations` indexa por **id de equipo y luego idioma**;
  cada equipo recupera la suya y ninguno hereda la de otro. El campo se rellena con la
  traducción de la tabla para ese equipo cuando no hay una propia. Añadido botón **Grabar**, que
  fuerza la escritura inmediata de `settings.json` (los cambios ya se aplican al editarlos). Las
  traducciones antiguas (por idioma y el alias único) se siguen leyendo y se migran al guardar.

## [x] El selector de equipo muestra «EU (0)» / «NA (0)» y no aparece ningún equipo (2026-10-31)
- Al desplegar «Equipo» salen «EU (0)» y «NA (0)» y, escoja lo que escoja, la lista de equipos
  está vacía.
- **Hecho (0.9.1):** dos causas. (1) El worker solo pedía los matchups dentro de WvW, así que
  con la ventana de ajustes abierta fuera de un mapa no había equipos que ofrecer; ahora
  refresca la lista de equipos mientras el selector está en pantalla. (2) El número entre
  paréntesis se pintaba también sin datos, y un `(0)` parece un fallo; ahora solo aparece
  cuando hay datos y, si no hay ninguno, se explica.

## [x] El desplegable de región no ofrecía EU ni NA (2026-10-31)
- Al desplegar «Región» no salía nada que elegir.
- Causa: la lista de regiones se construía solo con las que tenían equipos del último sondeo,
  así que hasta el primer poll el desplegable estaba vacío.
- **Hecho (0.8.1):** `ui::settings::draw_region` ofrece siempre EU y NA; si no hay equipos, la
  lista de equipos lo explica.


## [x] Una ventana cerrada con la × no se podía volver a abrir (2026-10-31)
- En Bastión de la Armisticia no aparecía el widget de avisos y no había forma de sacarlo.
- Dos cosas distintas: (1) el gate oculta el overlay fuera de los mapas de Mundo contra Mundo
  a propósito — el Bastión es un salón (`map_type` 18) y ahí no se dibuja; (2) un fallo real:
  cerrar una ventana con la × la dejaba invisible para siempre, porque no existía ningún
  control para volver a abrirla.
- **Hecho (0.11.1):** los ajustes tienen una sección **Ventanas** con una casilla por ventana
  (Avisos, Claims del matchup, Clanes) y una línea que recuerda que el overlay solo se pinta en
  los mapas de Mundo contra Mundo.

## [x] El traductor no guarda la traducción
- Al desplegar un servidor, permite usar la traducción
- debería porponer la traducción por defecto que pusimos pero no lo hace
- Falta boton grabar
- Cuando voy a otro servidor, sigue poniendo la misma traducción que habia puesto a otro servidor
- Cuando vuelvo a uns erviudor que ya había traducido, no lo ha guardado
- **Hecho (0.10.1):** mismo fallo que la entrada de arriba, reportado dos veces. La traducción
  se guarda por equipo y el campo propone la de la tabla; hay botón **Grabar**.

## [x] widget: errores (2026-10-31)
- No sale nombre del clan sino un ID. adjunto imagen
- al intenraer seleccioanr un clan, no me deja. adjuto imagen de lista de clanes
- los avisos deberian desaparecer pero están siempre (a menos que no haya entendido bien la ventana esa)
- el selector de matchup claims no permite elegir " clanes que sigo (cualquier mapa)". adjunto imagen
- el selector d eequipo dentro de mcm: me deja seleccionar el equiopo acutal poero no salen otros, adjunto imagen
 ![alt text](image.png)
 ![alt text](image-1.png)
 ![alt text](image-2.png)
 ![alt text](image-3.png)
 ![alt text](image-4.png)
- **Hecho (0.11.3):**
  - **El ID era `claimed_by` sin resolver.** Viene como UUID con guiones
    (`9FDB4A86-1588-E611-80D3-AC162DC0847D`) y `/v2/guild/<id>` lo resuelve a
    `[AMX] Le Coq Sportif` (medido en vivo; sin guiones da 500, así que se usa tal cual). Se
    resuelven de 10 en 10 por sondeo y se guardan para la sesión; los gremios disueltos (404) no
    se reintentan. Ahora sale `[TAG] Nombre` en las tres vistas.
  - **Seleccionar un clan**: la historia se dibujaba debajo de una lista más alta que la ventana,
    así que elegir parecía no hacer nada. La lista va en su propia zona con scroll y la historia
    queda siempre visible. **Pendiente:** confirmarlo en el juego.
  - **Los avisos no desaparecían**: el widget compacto enseña los últimos 10 minutos y el
    registro completo está en la ventana nueva «Registro de avisos».
  - **Filtro «Clanes que sigo»** añadido a la ventana de objetivos del matchup.
  - **Selector de equipo dentro de MvM**: el worker solo pedía el matchup del jugador, así que
    salía un equipo por color. Con el selector abierto pide los nueve.


 ## [x] Historial de claims no se borra (2026-10-31)
 - El historia de claims se debe borrar cada vez que se reiniicar el juego
 - Lo que se debe guardar es el historial de clanes que se sigue
 - **Hecho (0.12.1):** el baseline (lo que ya estaba reclamado al arrancar) se apuntaba con su hora real,
   así que avisos e historial de clanes arrancaban con claims de ayer. Ahora los `late` no entran ni en
   avisos ni en historial (`recent_alerts`, `history_row`) y en disco solo se escribe lo que reclamaron
   las guilds que sigues (`keeps_in_the_store`).

 ## [x] widget no se parece a bocetos (2026-10-31)
 - En bocetos una sola ventana tiene pestañas para agrupar todo
 - Falta el boceto de la ventana de avisos pop-up que dura 'x' segundos según se haya seleccionado
 - **Hecho (0.12.0):** una sola ventana con pestañas **Avisos · Mapa · Clanes · Registro · Ajustes**
   (la última), como `docs/boceto.html` (ADR-014). Las opciones de Nexus ofrecen **Ver la ventana** y
   **Avisos emergentes**, y siguen mostrando los ajustes aunque la ventana esté cerrada. El pop-up tiene
   interruptor propio (`popup_enabled`) además de la duración 0–5 s, y se dibuja aunque el widget esté
   cerrado. **Pendiente:** mirar en el juego el boceto del pop-up (fade, barra de color, tipografía).
 - **Hecho (0.12.1):** el baseline (lo que ya estaba reclamado al arrancar) se seguía apuntando con su hora
   real, así que la lista de avisos y el historial de clanes arrancaban con claims de ayer. Ahora los
   `late` no entran ni en avisos ni en historial (`recent_alerts`, `history_row`) y en disco solo se
   escribe lo que reclamaron las guilds que sigues (`keeps_in_the_store`).

## [x] Ventana widget no admite click (2026-10-31)
- Click derecho no hace nada
- clicar pestañas no hace nada
- he reclamado e eternos taj ode la gaurdia del ogro (ogrewatch cut) y no  ha saltado  ningun aviso de reclamado
- clanes seguidos sigue con id, adjunto imagen
- quiero otro comportamiento: otro oel desplegable de clan a salir lo muestra en inglés, que muestre ingles / traduccion
 - **Hecho (0.14.1):** la ventana estaba en **click-through** (`"WvW Lurker": { "locked": true }` en
   el `settings.json`), así que los clics se iban al juego y nada lo decía. Ahora el estado se anuncia
   en naranja con el atajo, y las opciones de Nexus tienen la casilla **Click-through** para salir sin
   teclado. **Mientras tanto**: `Alt+Mayús+L`, o desmarcar la casilla en las opciones de Nexus.
 - La lista de **clanes seguidos** ya enseña `[TAG] Nombre` en vez del identificador.
 - Un clan **seguido** vuelve a avisar aunque el objetivo ya estuviera reclamado al arrancar (RF-21).
   **Pendiente de confirmar:** si el claim de Eternos era de un clan que no sigues, no avisará (RF-04);
   dime si quieres que avise también de tus propias capturas y con qué criterio.
 - **Pendiente:** enseñar en el desplegable de clan el nombre en inglés **y** la traducción. Falta
   decidir de dónde sale la segunda: los nombres de clan no los traduce el juego, así que habría que
   usar la tabla de equipos (`data/team-names.json`) solo cuando coincida, o dejar que el usuario
   escriba la suya. Lo dejo apuntado como T-43.
![alt text](image.png)

## [x] Bloquear con o sin click-through, atajo pillado y crash al abrir Mapa (2026-10-31)
- La ventana tiene la opcion de bloquear, con o sin click-through.
- Alt+Shift+L no abre nuestro menu, abre el legacy de arcDPS.
- Al clicar la pestana Mapa ha hecho crash.
- **Hecho (0.14.2):** (1) `locked` (bloquear posicion; la ventana sigue aceptando clics) y
  `click_through` (los clics van al juego) vuelven a ser interruptores separados: menu contextual,
  ajustes y opciones de Nexus. (2) El atajo por defecto pasa a `Ctrl+Shift+L` porque `Alt+Shift+L`
  es de arcDPS; las menciones en la UI se generan desde la constante. (3) El render va envuelto en
  `catch_unwind`: un panico queda en `nexus.log` y el overlay sigue vivo. **Pendiente:** si vuelve a
  cerrarse al abrir Mapa, manda esa linea del log.


## [x] Crash al abrir la pestaña Mapa (2026-10-31)
- Al clicar «Mapa» se cerraba el juego.
- **Hecho (0.14.5):** causa exacta en `Nexus.log`:
  `panicked at arcdps-imgui-0.8.0/src/draw_list.rs:103: The DrawListMut instance for the window
  draw list is already loaded! You can only load one instance of it!`. `ui::map` pedía
  `get_window_draw_list()` **una vez por objetivo** y esa llamada es un singleton por frame: la
  segunda panica. Ahora se pide una sola vez y se pasa por referencia. El `catch_unwind` de 0.14.3
  evitó la pantalla negra y dejó la traza, que es lo que ha permitido cerrarlo.
