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

## [ ] «ha dejado de recoger quién reclama» (2026-10-31 22:09)
- «Desde hace 5 minutos ha dejado de recoger quién reclama.»
- Lo que decía la evidencia: el log tenía `sondeo 20: partida Some("1-1"), 44 reclamados de 178
  objetivos, 0 eventos nuevos`, y el API confirmó que en `1-1` el último `claimed_at` era de las
  22:02 locales mientras otras partidas se movían a las 22:09/22:10. O la partida estaba tranquila, o
  el equipo configurado no es en el que se juega.
- **Pendiente:** con 0.22.5 el widget y el latido del log dicen «Último reclamo: hace N min» (marca
  del **juego**). Si en el juego reclaman y ese número no baja, el problema es el emparejamiento o el
  equipo elegido, no la detección: hay que mirar la cabecera (equipo y partida) contra lo que se ve
  en el juego.
