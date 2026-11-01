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
## [x] la descarga del índice falla con «no existe» (2026-11-01)
«al intentar bajar el zip para probar que descarga, descomprime y machaca el guild-names.json, me da error» — *no se pudo descargar …/gw2_lurker_guilds.zip: no existe*.
- **Causa: una errata mía.** La URL por defecto apuntaba a `gw2addonreleases` en vez de
  `gw2addonsreleases` (le faltaba la «s» de *addons*). GitHub respondía **404** y el addon lo
  traducía como «no existe». Lo comprobé con el propio cliente HTTP del addon (`ffi::http::get`):
  la URL mala daba 404 y la buena 200 con 297 532 bytes; con `curl`/`Invoke-WebRequest` no se veía
  porque escribía la buena a mano.
- **Hecho (0.25.7):** URL corregida, y la errata se **repara sola** en `settings.json` al cargar
  (quien la tuviera guardada se queda sin descarga). Tres tests nuevos:
  `rf20c_the_default_index_url_is_the_distribution_repo`, `rf20c_the_broken_default_url_repairs_itself`
  y la prueba de red `live_the_index_zip_is_downloadable` (200 y ≥1000 bytes con el cliente real).
- De paso: el mensaje de error ya no dice «no existe» a secas, dice que el fichero no está
  publicado en esa URL (404) y que se revise la dirección; y la descarga lleva un `?v=<sello>` para
  saltarse la caché del CDN, que también puede servir un 404 viejo.
- **Pendiente:** pulsar *Descargar el índice* en el juego.