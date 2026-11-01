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

## [ ] Abrir la ruta de un clan lleva al mapa del jugador (2026-11-01)
- Al abrir un clan no favorito, Mapa mostraba el mapa del jugador en vez del último mapa con
  reclamos del clan. Sin eventos, también cambiaba de pestaña sin dar explicación.
- **Pendiente:** arreglo implementado en `ui/navigation.rs`, `ui/mod.rs` y `ui/map.rs`: conservar
  el salto hasta dibujar Mapa y validar que haya eventos; de lo contrario, mostrar un aviso.
  Falta publicar y comprobar la navegación y el aviso dentro de GW2.
