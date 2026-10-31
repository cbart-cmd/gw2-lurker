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


 ## [ ] Historial de claims no se borra
 - El historia de claims se debe borrar cada vez que se reiniicar el juego
 - Lo que se debe guardar es el historial de clanes que se sigue

 ## [ ] widget no se parece a bocetos
 - En bocetos una sola ventana tiene pestañas para agrupar todo
 - Falta el boceto de la ventana de avisos pop-up que dura 'x' segundos según se haya seleccionado