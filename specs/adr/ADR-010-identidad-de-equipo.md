# ADR-010 – Identidad de equipo tras la reestructuración de mundos

- **Estado:** Aceptado. El punto "la API no expone nombres de equipo" queda **verificado el 2026-10-31** (ver `07-spikes.md`); solo queda abierto el uso del `world_id` de Mumble, que se prueba en T-01.
- **Contexto:** la reestructuración sustituyó los servidores por equipos que se recomponen por temporada o relink. El usuario observa que los nombres de equipo ya no se ven en el juego y `/v2/worlds` muestra nombres antiguos.
  - **Verificado (2026-10-31):** `/v2/wvw/matches?ids=all` devuelve, por color, `worlds` (mundo heredado) y `all_worlds`, que **contiene los ids de equipo** (11001–11015 NA, 12001–12018 EU). La API oficial **no** expone nombres: `/v2/wvw/teams` y `/v2/wvw/alliances` devuelven 404 y el índice de `/v2` no lista nada equivalente.
  - **Verificado:** `/v2/wvw/guilds/eu` y `/na` sí dan `{ guild_id: team_id }`, así que la pertenencia de un gremio a un equipo sí es oficial.
  - **Verificado:** el prefijo del id de match indica región (`1-x` NA, `2-x` EU).
- **Decisión:**
  1. El `team_id` es la única clave de identidad; nombres, alias y colores son presentación (RF-03h).
  2. El selector muestra `nombre (id)`; tras elegir se muestran los dos rivales del match para verificar (RF-03g).
  3. El usuario puede definir un alias (RF-03f).
  4. **Los nombres viven en `data/wvw-teams.json`**, tabla versionada id → nombre → nación, y se regeneran con `scripts/update_wvw_teams.ps1`. El script toma los ids en uso de la API oficial y solo los nombres de la fuente curada; si la fuente falla, conserva lo ya conocido y avisa de los ids sin nombre (RF-03i). El addon funciona sin nombres, mostrando ids.
  5. El `world_id` de Mumble es un *shard id* y no identifica el equipo (documentado en `gw2-mumble-rs`: "not usable since the switch to the megaserver system"). Solo se usará como sugerencia si T-01 demuestra lo contrario (RF-03e).
- **Alternativas descartadas:** empaquetar una lista de nombres sin script de refresco (se desactualiza en silencio); exigir API key para leer el equipo de la cuenta (contradice ADR-004, aunque `/v2/account/wvw` lo permitiría); copiar la base de datos de un tercero (gw2mists) incluyendo sus métricas agregadas — se descarta por ser compilación ajena, cambiar constantemente y no ser necesaria: solo se curan los 33 pares id → nombre.
- **Consecuencias:** la UI nunca depende de un tercero en ejecución; actualizar nombres tras un reset es una orden manual (`scripts/update_wvw_teams.ps1`) y queda registrada en el diff. Si aparece una fuente oficial de nombres, basta cambiar el origen de la tabla.
