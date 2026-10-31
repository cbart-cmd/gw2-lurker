# ADR-004 – Selección manual de mundo, sin API key

- **Estado:** Aceptado; completado por ADR-010 (identidad de equipo y nombres)
- **Decisión:** el usuario elige su mundo de la lista de `/v2/worlds`. Se guarda el `world_id`, no el color (cambia cada semana). El color se resuelve contra `all_worlds` del match actual.
- **Consecuencias:** no se necesita ninguna clave ni permisos. Si Mumble ofrece `world_id`, solo sirve como sugerencia.
