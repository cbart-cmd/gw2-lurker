# Bugs

Bandeja de entrada de fallos. **No se borra**: cada bug se queda con su marca. `[x]` cuando creo que
está arreglado, con la versión en la que sale el arreglo, para que el jugador pueda comprobarlo y
tacharlo del todo si quiere.

Formato:

```
## [x] Título corto (fecha en que se reporta)
- Qué pasa, cómo se reproduce, qué se esperaba.
- **Hecho (x.y.z):** qué se cambió y dónde.
- **Pendiente:** qué falta (si falta algo).
```

Estado: `[ ]` pendiente · `[x]` arreglado según el addon (a falta de que el jugador lo confirme).

> **2026-10-31, 23:40:** los arreglos de la 0.22.10 **no llegaron al juego**: Nexus sustituye la DLL
> instalada en cada arranque por la publicada en `gw2addonsreleases`, que era la 0.22.9. Todo esto
> sale junto en la **0.23.1**, que sí se publica.

## [ ] «ha dejado de recoger quién reclama» 
cuando pasa un rato de varios munutos, no recoge información de quien reclama. además, parece tener una cache porque entré en el juiego más taerde y el mapa pomia que YaK tenia reclamado un campa  que en verdad era de otro color. a los 10 mintos deja de funcionar
- **Evidencia (2026-10-31, 23:20, con la 0.22.9 y el juego abierto):** el MumbleLink dice
  `shard_id = 2147483649` (`0x80000001`), `instance = 0`, `team_color_id = 9`, y el campo `name`
  lleva «Guild Wars 2» mientras el personaje (`Jhoin`) está en `identity`. Ese `shard_id` no
  aparece en `all_worlds` de ninguna de las nueve partidas, así que la detección por Mumble **no
  puede funcionar**; el log lo deja escrito cada 15 minutos. La partida vigilada (`1-1`, NA) sí era
  la suya: su clan (`YaK · Dolyak Sanctuary`) tiene un reclamo ahí (`96-38`, azul).
- **Hecho (0.23.1):** el reclamo caducado desaparece del mapa y de las listas: la API **no** borra
  `claimed_by` al cambiar el color, solo un reclamo nuevo lo sustituye, así que ahora se descarta
  cuando `last_flipped` es posterior a `claimed_at`. Eso era el «clan en un campa de otro color».
- **Hecho (0.23.1):** el sondeo pide la partida del equipo que va a vigilar, no la del guardado
  (con la detección cacheada un cuarto de hora, miraba otra partida 14 de cada 15 minutos). Y el
  equipo del personaje se guarda con su nombre: con «Guild Wars 2» como clave, el equipo se
  buscaba bajo una entrada que no era el personaje.
- **Causa encontrada (2026-11-01, 0.24.1):** la partida se queda **congelada en la API de ArenaNet**.
  Medido con `curl`: `1-1` devolvió 20 minutos la misma copia, puntos incluidos, mientras las otras
  ocho se movían. El addon sondeaba cada 5,2 s. Ahora el widget lo avisa en naranja a partir de 5
  minutos (`ArenaNet's API has not published any change…`), así que «no pasa nada» deja de parecer
  «el addon está parado». Ver la entrada «deja de registrar eventos a los ~11 minutos».
- **Pendiente:** los `sondeo N: … último reclamo hace X min` del log dicen que entre las 22:02 y las
  23:04 la API de esa partida no publicó **ningún** reclamo y luego llegaron todos de golpe. Si con
  la 0.22.10 vuelve a pararse a los 10 minutos, esas líneas (cada 10 sondeos) dicen si el addon
  sigue sondeando; pegarlas aquí resuelve la duda en una lectura.

## [x] no puiedo eliminar clanes 
pulsar "x" al lado de un clan al que sigo no lo borra. deberia preguntar si estas seguro, y eliminarlo de clanes que sigues
- **Hecho (0.23.1):** la `×` se dibujaba **después** de una fila que ocupa todo el ancho de la
  pestaña, así que caía fuera de la ventana y no se podía pulsar. Ahora el hueco se reserva primero,
  como el `×` de la cabecera. La pregunta sigue siendo «¿Quitar a *nombre* de los clanes que sigues?»
  con `Sí, quítalo` / `No, seguir siguiéndolo`. Además la pestaña *Clanes* y el filtro de avisos
  leen la configuración viva: la estrella y las listas cambian en el mismo momento.
- **Pendiente:** que el jugador lo pruebe en el juego.

## [x] click thoufgh no hace nada 
version 0.22.6. bnoton derecho + click trghough en el widget no hace nada. deberia poderse usar aunque se haya puesto cliq thoufgh en opciones (que sí fubciona)
- **Hecho (0.23.1):** dos fallos. (1) `click_through` era el único campo que el cromo de la ventana
  no escribía de vuelta, así que la marca del menú del clic derecho se tiraba al final del
  fotograma. (2) La comprobación del clic derecho leía la configuración **del arranque**, no la
  viva: activar el click-through en Ajustes no lo veía nunca. Ahora lee la copia editable y el
  rectángulo se resuelve igual que al dibujar.
- **Pendiente:** que el jugador lo pruebe (clic derecho sobre el widget con click-through activado
  desde Ajustes).

## [x] bujscador de clanes no funciopna 
ejemplo, pongo europa y busco dolyak  y dice uqe no existe. pero sí exxiste, es mi clan y de hehco lo estoy siguiendo como favorito. tengo el clan en na y en eu
- **Comprobado con el API (2026-10-31):** `/v2/guild/search?name=dolyak` devuelve **un** clan, y no
  es el del jugador: devuelve `F0CE8616 · [DYAK] Dolyak`, mientras el suyo es
  `6C45F3C8 · [YaK] Dolyak Sanctuary`. El buscador del API va por el principio de una palabra del
  nombre, así que encontrar «Dolyak» no es encontrar «Dolyak Sanctuary».
- **Hecho (0.23.1):** además de los resultados del API, el buscador mira **los clanes que ya
  sigues** por nombre y tag, sin red: «dolyak» o «yak» enseñan `[YaK] Dolyak Sanctuary · ya lo
  sigues` encima de los resultados, y los resultados que ya sigues llevan ★ (con el clan en NA y en
  EU salen las dos entradas, que es lo que se pidió). Si el API no encuentra nada más, el mensaje lo
  dice en vez de «no existe».
- **Pendiente:** que el jugador lo pruebe en el juego.

## [x] borrar el json que nadie lee (2026-11-01)
1.- borra el json aquel que nadie lee y elimina su codigo.
- **Hecho (0.24.0):** se va `crates/store` entero: el `session-000.jsonl` no lo leía nadie (la UI
  usa el historial en memoria y ADR-012 borra el fichero al arrancar). También se borra el fichero
  que hubiera en la carpeta del addon.

## [x] nombres de clan en el Bastión y en todas partes (2026-11-01)
2.- en el Bastión los clanes salen con su id largo; si no sabes el nombre, búscalo y caches al settings.
- **Hecho (0.24.0):** `names.rs` guarda `id → [TAG] Nombre` en `guild-names.json` (fuera de
  `settings.json` a propósito: puede tener miles de entradas y `settings.json` se reescribe cada
  vez que se mueve una ventana). Se carga al arrancar y se publica en el dashboard, así que el
  Bastión (donde no se sondea) enseña nombres desde el primer fotograma. Un nombre pedido una vez
  no se vuelve a pedir nunca.

## [x] los segundos del aviso se pierden al reiniciar (2026-11-01)
3.- los segundos de avisos emergentes se pierden en cada reinicio.
- **Hecho (0.24.0):** el saneado del fichero recortaba a **5 s** mientras el control ofrecía 20:
  cualquier valor mayor se guardaba como 5. Ahora el tope es el mismo (`POPUP_SECS_MAX` = 20) y hay
  test con 12 s.

## [x] el atajo del click-through está mal (2026-11-01)
4.- si pulso clickthrough dice que pulse una combinación que no es la correcta.
- **Hecho (0.24.0):** el texto era el literal «CTRL+SHIFT+L». Ahora `keybinds.rs` lee
  `addons/Nexus/InputBinds.json` y enseña lo que Nexus tiene guardado (en tu fichero:
  `Ctrl+Shift` + código 39, que es «;»). Si no se puede leer, se dice el identificador
  `GW2LURKER_CLICK_THROUGH` en vez de inventar una tecla.

## [x] quitar la casilla de click-through de la ventana (2026-11-01)
5.- en widget, ajustes, «ventanas» elimina la opción de click-through; solo el botón derecho.
- **Hecho (0.24.0):** la casilla desaparece de Ajustes → Ventanas; queda el clic derecho sobre el
  widget (y el atajo que tengas en Nexus).

## [x] buscador de clanes: hace falta un índice (2026-11-01)
6.- el buscador no funciona; extrae a un script para probar: «a new» debe encontrar «A new name…»;
    «DIE» debe encontrar varios.
- **Medido (2026-11-01):** `/v2/guild/search?name=` devuelve **un solo** clan y solo cuando el
  texto **empieza** su nombre: `die` → *Die*, `a new` → *A New*, `a ne` → nada, `DIE anmutig` →
  nada. No hay forma de pedirle «todos los que contienen X».
- **Hecho (0.24.0):** la búsqueda real es por **subcadena** sobre la caché de nombres
  (`guild-names.json`), instantánea y sin red; el API añade una coincidencia más. `scripts/guild-search.ps1`
  busca fuera del juego y **construye** el índice (`-BuildIndex -Region eu`), que es lo que hace
  que «DIE» encuentre *Die Quaggans*, *Die Garde*, etc.
- **Pendiente:** el índice hay que construirlo una vez (el script tarda ~10-20 min para una
  región); después se reutiliza para siempre.

## [ ] deja de registrar eventos a los ~11 minutos (2026-11-01)
«como verás en image-6.png, a los 11 minutos deja de registrar eventos. puedes ver en la pantalla que lleva 7 minutos sin encontrar eventos, que he reclamado con DDK el campamento y que no sale pese a que tengo puesto que siga clanes favoritos y lo que pase en el mapa en el que estoy. y se ve que detecta correctamente que estoy en alpino azul, en el equipo invernadero de lutgardis.»
- **Medido (2026-11-01, 00:23-00:30, con el juego cerrado y `curl`):** el addon **sí** sondea: el
  log tiene `sondeo 180` a las 00:13:50 y `sondeo 190` a las 00:14:42 (5,2 s por vuelta) y la
  cabecera decía «ÚLTIMA ACTUALIZACIÓN 00:15:20». Lo que está parado es **la partida `1-1` en la
  API de ArenaNet**: entre las 00:23 y las 00:30 devolvía siempre lo mismo, con el reclamo más
  nuevo de las 23:07:58 UTC (hace 20 min) **y los puntos también congelados**
  (`67568/52264/35741` en cuatro consultas separadas). Las otras ocho partidas sí se movían
  (`2-3` a 23:21, `1-4` a 23:25). Ni la caché del CDN (misma respuesta con `?cb=`) ni el cliente
  HTTP: es el backend.
- Comprobado además: el equipo `11011` (Invernadero de Lutgardis, «Controlado por» en el panel del
  juego) **solo** está en `1-1`, y el clan `DDK` (`ACBC90A5`) no aparece reclamando en **ninguna**
  de las nueve, porque la partida en la que reclamó es esa y su copia está congelada.
- **Hecho (0.24.1):** el widget lo dice en naranja cuando el dato no cambia:
  «La API de ArenaNet no publica cambios de esta partida desde hace N min; el addon sigue
  sondeando cada X s». Se calcula con una huella de la partida (puntos + objetivos), así que
  distingue «no pasa nada» de «el addon está parado».
- **Pendiente:** que el jugador lo compruebe la próxima vez; si el aviso sale, la culpa es de la
  API y se puede ver a mano en `https://api.guildwars2.com/v2/wvw/matches/1-1`.
