# ADR-003 – Historial en JSON-lines

- **Estado:** Aceptado (decisión del usuario). La **retención** ya no es de 30 días: pasa a por sesión según ADR-012; el formato y el `trait EventStore` no cambian.
- **Decisión:** eventos en ficheros `history/AAAA-MM.jsonl`, append-only, con campo de versión `v`. Detrás de `trait EventStore`.
- **Motivo:** sin dependencias nativas en la DLL, fácil de inspeccionar y depurar, tolerante a líneas corruptas.
- **Consecuencias:** las consultas por rango se resuelven con índice en memoria cargado al abrir la vista. Si el volumen o las consultas lo exigen, se sustituye por SQLite implementando el mismo trait.
