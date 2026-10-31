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

## [x] clanes seguidos no dice el nombre (2026-10-31)
![ver imagen](image-1.png)
- En «Clanes seguidos» y en la lista de «Añadir clan» los clanes salían como el id crudo
  (`260D021E-F008-…`), imposible de reconocer.
- Causa: los nombres se pedían a `/v2/guild/:id` **solo para los clanes que habían reclamado algo
  en el emparejamiento**, así que un clan seguido que aún no había reclamado nunca se resolvía.
- **Hecho (0.17.1):** `catalog::resolve_guild_names` pide primero los **clanes seguidos** y después
  los del emparejamiento, con el mismo presupuesto por sondeo y la misma caché de sesión. Los que
  la API rechaza se recuerdan para no repetir la petición.

## [x] wdiget - ajustes nombres pequeños (2026-10-31)
![alt text](image-2.png)
- En la pestaña de ajustes los rótulos de la derecha se cortaban («INTERVALO DE SON…»,
  «ALCANCE DE LOS AVISOS»): el control se comía el ancho y el texto quedaba fuera de la ventana.
- Causa: el ancho reservado para los rótulos se calculaba con una **frase larga** (un aviso), no
  con los rótulos que de verdad se dibujan, así que la columna de los controles era enorme.
- **Hecho (0.17.1):** `ui::settings::label_column` mide la columna sobre la lista real de rótulos en
  el idioma activo, y los controles se quedan con el resto (`push_item_width`).

## [ ] avisos no funciona (2026-10-31)
- «En mapa azul he reclamado y no ha saltado el aviso de que he reclamado, aunque tengo marcado
  seguir reclamaciones de mi mapa.»
- **Diagnóstico (0.17.1):** el registro del juego dice que el addon estaba vigilando el
  emparejamiento **`1-1`, que es de Norteamérica**, con el equipo guardado `11011`
  («Lutgardis Conservatory», un equipo NA). Si el jugador está en Europa, el addon nunca ve sus
  capturas: son de otro emparejamiento. Los ids de equipo son de la temporada actual y el nombre
  del equipo no tiene por qué coincidir con el mundo antiguo que uno recuerda (T-45).
- **Hecho (0.17.1) en la parte que se puede arreglar sin el jugador:** la cabecera del widget
  enseña ahora la **región del emparejamiento** («Matchup 1-1 · NA»), se avisa en naranja cuando el
  equipo guardado ya no juega, y el worker escribe una línea por sondeo cuando cambia el
  emparejamiento vigilado (`worker: vigilando …`) más una por evento detectado
  (`evento: Claim … (late: …)`), que es lo que faltaba para distinguir «no llega» de «llega y se
  filtra».
- **Pendiente de confirmar por el jugador:** volver a elegir equipo (Ajustes → Región → Equipo) y,
  si sigue sin saltar, mandar las líneas `evento:` y `worker: vigilando` del `Nexus.log`.

## [x] wdiget - buscador de clanes no muestra nada (2026-10-31)
- El desplegable de equipo no desplegaba nada y escribir una palabra y pulsar «Buscar» no hacía
  nada.
- Causa doble: (1) el bloque exigía **equipo y región**, y los dos se sacaban de la lista de
  equipos vivos, así que con la lista vacía no había ni desplegable ni búsqueda; y (2) dentro de
  WvW la lista de equipos solo se pide si alguien la pide, y este bloque no la pedía.
- **Hecho (0.17.1):** la **región** (la del equipo del jugador) es lo único obligatorio y el equipo
  pasa a ser un filtro opcional («Equipo (cualquiera)», y los resultados enseñan su equipo). El
  bloque pide la lista de equipos al worker mientras está en pantalla y, si aún no ha llegado, lo
  dice en vez de quedarse mudo. El campo de texto va antes del botón.

## [x] Botones de menú escondidos (2026-10-31)
- Con la ventana estrecha las últimas pestañas se quedaban fuera y no había forma de llegar a
  ellas.
- **Hecho (0.17.1):** la barra de pestañas usa `FITTING_POLICY_SCROLL`, así que imgui dibuja las
  flechas de desplazamiento en la propia barra (el «…» que se pedía), como el proyecto hermano.
