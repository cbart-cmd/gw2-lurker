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

## [ ] El selector de equipo muestra «EU (0)» / «NA (0)» y no aparece ningún equipo (2026-10-31)
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
