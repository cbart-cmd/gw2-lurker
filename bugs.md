# Bugs

Bandeja de entrada de fallos. **No se borra**: cada bug se queda con su marca. `[x]` cuando creo que
está arreglado, con la versión en la que sale el arreglo, para que el jugador pueda comprobarlo y
tacharlo del todo si quiere.

**Fichero compartido** (2026-11-01). El jugador puede borrar filas cuando quiera: si falta alguna, es
cosa suya y no se restaura. Lo que no se hace nunca es borrar lo que él añada mientras se trabaja en
otro arreglo: se relee el fichero antes de escribirlo y cada uno toca solo sus líneas.

Formato:

```
## [x] Título corto (fecha en que se reporta)
- Qué pasa, cómo se reproduce, qué se esperaba.
- **Hecho (x.y.z):** qué se cambió y dónde.
- **Pendiente:** qué falta (si falta algo).
```

Estado: `[ ]` pendiente · `[x]` arreglado según el addon (a falta de que el jugador lo confirme).

## [x] lista de clanes: qué hace clicar una fila (2026-11-01)
«en widget, lista clanes, sale una lista de clanes según reclamado. no queda claro que hace pulsar en
él. debería haber dos botones: uno de "+" para añadir a clanes favoritos (si no lo está ya), y otro
tipo "i" de información, que haga la inferencia sobre el mapa y no en texto abajo»
- **Hecho (0.25.14):** cada fila lleva `+` (seguir el clan, solo si no se sigue; si ya se sigue, en su
  sitio va un `*` que es estado y no botón) e `i`, que selecciona el clan y salta a la pestaña *Mapa*
  con su ruta dibujada en el último mapa donde reclamó. La ruta **ya no se escribe en texto** debajo de
  la lista: el resumen («desde su fortaleza -> A -> salto a X -> B») se pinta junto al mapa, donde está
  el dibujo. Clicar la fila sigue enseñando el historial del clan (RF-72).
- **Pendiente:** verlo en el juego.


