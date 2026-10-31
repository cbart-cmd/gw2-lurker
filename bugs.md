# Bugs

Bandeja de entrada de fallos. **Aquí solo se queda lo que falta**: cuando un bug está arreglado se
borra de este fichero. El qué se cambió y en qué versión sale queda en `PROGRESO.md` y, si se ve, en
`CHANGELOG.md`.

Formato:

```
## [ ] Título corto (fecha en que se reporta)
- Qué pasa, cómo se reproduce, qué se esperaba.
- **Pendiente:** qué falta para cerrarlo (una prueba en el juego, un log, una decisión).
```

Estado: `[ ]` pendiente · `[x]` arreglado en esta tanda (se borra al cerrarla).

## [ ] avisos y registro · no recoge bien la información (2026-10-31)
- «No sale siempre quién ha reclamado»; «desde las 20:22 no recoge nuevos resultados aunque ha habido
  reclamaciones»; «el registro va lentísimo»; y el formato de la línea.
- Arreglado hasta ahora: el formato con colores por trozo (0.19.0), el registro con `ListClipper` y 50
  líneas como mucho (0.21.0) y los nombres de clan resueltos a 20 por sondeo (0.21.0).
- **Pendiente:** saber si el sondeo avanza. Desde 0.20.1 el worker escribe cada diez sondeos
  `sondeo N: partida …, N reclamados de M objetivos, K eventos nuevos`, y cada evento una línea
  `evento: …`. Con esas líneas de `Nexus.log` se cierra o se ataca el «desde las 20:22 no recoge nada».

## [ ] buscador de clanes · `invalid type string 12008` (2026-10-31)
- Al buscar un clan saltó `invalid type string 12008`; `12008` es un **id de equipo** (`12xxx` = EU),
  no un id de clan.
- Mitigado en 0.18.0: antes de pedir `/v2/guild/:id` se comprueba que el id tenga forma de id de clan
  (32 hexágonos), y el mensaje de fallo dice qué se buscó y en qué paso falló.
- **Pendiente:** confirmar con el log que ya no sale. Si vuelve, la línea `clanes: …` dice de dónde
  viene.

## [ ] varios
- la ruta inferida de un clan deberia pintarla sobre el mapa
- **Pendiente:** comprobar con 0.22.3. `map::draw_route` ya pinta la ruta cuando hay **dos o más**
  claims del mismo mapa con coordinada conocida; si no sale, hace falta saber el clan elegido y qué
  líneas hay en `history` (el log las enseña).

