# Glosario

| Término | Significado |
|---|---|
| **WvW / MCM** | Mundo contra Mundo. |
| **Match** | Enfrentamiento semanal entre tres equipos (rojo, azul, verde). ID tipo `2-5`. |
| **Mundo (world)** | Servidor. Puede estar *enlazado* con otros; en el match aparecen en `all_worlds`. |
| **Equipo / color** | Rojo, azul o verde. Cambia cada semana; por eso se guarda el mundo, no el color. |
| **Objetivo (objective)** | Campamento, torre, fortaleza, castillo, ruina, etc. ID tipo `95-36` (`mapId-objectiveId`). |
| **Snapshot** | Estado completo de todos los objetivos del match en un instante. |
| **Flip** | Cambio de dueño (`owner`) de un objetivo (`last_flipped` cambia). |
| **Claim** | Una guild reclama un objetivo (`claimed_by`, `claimed_at`). |
| **Claim tardío** | Claim detectado en el primer snapshot o tras un hueco; su hora viene de `claimed_at`, no del momento de detección. |
| **Hueco (gap)** | Periodo sin snapshots válidos (red caída, API atascada, juego cerrado). Pueden haberse perdido eventos. |
| **Ruta simulada** | Polilínea que une, en orden cronológico, los objetivos con claim de una guild. Es una inferencia, no un rastreo. |
| **ADR** | Architecture Decision Record. |
| **EARS** | Easy Approach to Requirements Syntax: "Cuando <evento>, el sistema deberá <respuesta>". |
