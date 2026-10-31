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

## [ ] mapa · cambiar de mapa sigue enseñando el anterior (2026-10-31)
- «Salto a Eternos y sigue enseñando el mapa verde.»
- **Pendiente:** comprobarlo en el juego con 0.22.0. El mapa se elige en el desplegable de la pestaña y
  en el «salto a <mapa>» de la ruta; si al cambiar de mapa se queda el fondo anterior, el fallo está en
  qué `Frame` usan las teselas y no en la selección.
- mapa lo pinta fuera de sitio, pinta encima de las pestañs de arriba y por debajo, adjunto imagen
![alt text](image-4.png)
- el pop up de avisos salta varios aviso s de golpe cada mucho, en vez de ir sacando el aviso suelto cuando alguien reclama algo (pueden pasar unos pocos segundos, maximo 5).  el selector de segundos deberia ser mas larego, permitri entre 0 y 20 segunods
- ![ no soy un robotm, no se que es mapa 99 tipo 12 ni emparejamiento 1-1, esos numeros para mi no significan nada](image-6.png)
- la ruta inferida de un clan deberia pintarla sobre  el mapa
- el mapa sale chafado, es mucho mas cuadrado. mira el mapa de gw2mists para ver como se deberian ver los mapas