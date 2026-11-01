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


