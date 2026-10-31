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
> sale junto en la **0.23.0**, que sí se publica.

## [ ] «ha dejado de recoger quién reclama» 
cuando pasa un rato de varios munutos, no recoge información de quien reclama. además, parece tener una cache porque entré en el juiego más taerde y el mapa pomia que YaK tenia reclamado un campa  que en verdad era de otro color. a los 10 mintos deja de funcionar
- **Evidencia (2026-10-31, 23:20, con la 0.22.9 y el juego abierto):** el MumbleLink dice
  `shard_id = 2147483649` (`0x80000001`), `instance = 0`, `team_color_id = 9`, y el campo `name`
  lleva «Guild Wars 2» mientras el personaje (`Jhoin`) está en `identity`. Ese `shard_id` no
  aparece en `all_worlds` de ninguna de las nueve partidas, así que la detección por Mumble **no
  puede funcionar**; el log lo deja escrito cada 15 minutos. La partida vigilada (`1-1`, NA) sí era
  la suya: su clan (`YaK · Dolyak Sanctuary`) tiene un reclamo ahí (`96-38`, azul).
- **Hecho (0.23.0):** el reclamo caducado desaparece del mapa y de las listas: la API **no** borra
  `claimed_by` al cambiar el color, solo un reclamo nuevo lo sustituye, así que ahora se descarta
  cuando `last_flipped` es posterior a `claimed_at`. Eso era el «clan en un campa de otro color».
- **Hecho (0.23.0):** el sondeo pide la partida del equipo que va a vigilar, no la del guardado
  (con la detección cacheada un cuarto de hora, miraba otra partida 14 de cada 15 minutos). Y el
  equipo del personaje se guarda con su nombre: con «Guild Wars 2» como clave, el equipo se
  buscaba bajo una entrada que no era el personaje.
- **Pendiente:** los `sondeo N: … último reclamo hace X min` del log dicen que entre las 22:02 y las
  23:04 la API de esa partida no publicó **ningún** reclamo y luego llegaron todos de golpe. Si con
  la 0.22.10 vuelve a pararse a los 10 minutos, esas líneas (cada 10 sondeos) dicen si el addon
  sigue sondeando; pegarlas aquí resuelve la duda en una lectura.

## [x] no puiedo eliminar clanes 
pulsar "x" al lado de un clan al que sigo no lo borra. deberia preguntar si estas seguro, y eliminarlo de clanes que sigues
- **Hecho (0.23.0):** la `×` se dibujaba **después** de una fila que ocupa todo el ancho de la
  pestaña, así que caía fuera de la ventana y no se podía pulsar. Ahora el hueco se reserva primero,
  como el `×` de la cabecera. La pregunta sigue siendo «¿Quitar a *nombre* de los clanes que sigues?»
  con `Sí, quítalo` / `No, seguir siguiéndolo`. Además la pestaña *Clanes* y el filtro de avisos
  leen la configuración viva: la estrella y las listas cambian en el mismo momento.
- **Pendiente:** que el jugador lo pruebe en el juego.

## [x] click thoufgh no hace nada 
version 0.22.6. bnoton derecho + click trghough en el widget no hace nada. deberia poderse usar aunque se haya puesto cliq thoufgh en opciones (que sí fubciona)
- **Hecho (0.23.0):** dos fallos. (1) `click_through` era el único campo que el cromo de la ventana
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
- **Hecho (0.23.0):** además de los resultados del API, el buscador mira **los clanes que ya
  sigues** por nombre y tag, sin red: «dolyak» o «yak» enseñan `[YaK] Dolyak Sanctuary · ya lo
  sigues` encima de los resultados, y los resultados que ya sigues llevan ★ (con el clan en NA y en
  EU salen las dos entradas, que es lo que se pidió). Si el API no encuentra nada más, el mensaje lo
  dice en vez de «no existe».
- **Pendiente:** que el jugador lo pruebe en el juego.
