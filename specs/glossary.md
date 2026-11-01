# Glosario

| Término | Significado |
|---|---|
| **WvW / MCM** | Mundo contra Mundo. |
| **Match** | Enfrentamiento de una semana entre tres equipos (rojo, azul, verde). ID tipo `2-5`. |
| **Equipo (team)** | Sustituye al mundo como identidad del jugador. Ids `11xxx` (NA) y `12xxx` (EU). El nombre no está en la API: se resuelve con `data/wvw-teams.json` (inglés) y `data/team-names.json` (es/fr/de). |
| **Mundo (world)** | Servidor antiguo. Aparece en `all_worlds` del match mezclado con los ids de equipo; ya **no** identifica a nadie (ADR-010). No se debe usar para elegir equipo. |
| **Objetivo (objective)** | Campamento, torre, fortaleza, castillo, ruina, etc. ID tipo `95-36` (`mapId-objectiveId`). |
| **Snapshot** | Estado completo de todos los objetivos del match en un instante. |
| **Flip** | Cambio de dueño (`owner`) de un objetivo (`last_flipped` cambia). |
| **Claim** | Una guild reclama un objetivo (`claimed_by`, `claimed_at`). Un objetivo admite **un solo** claim. |
| **Claim caducado** | `claimed_by` que la API deja puesto tras un flip. Se descarta cuando `last_flipped` es posterior a `claimed_at` (`core::matchup::claim_holds`). |
| **Claim tardío (`late`)** | Claim detectado en el primer snapshot o tras un hueco; su hora viene de `claimed_at`, no del momento de detección. No genera aviso, salvo clan seguido reciente. |
| **Hueco (`gap`)** | Periodo sin snapshots válidos (red caída, API atascada, juego cerrado). Pueden haberse perdido eventos. |
| **Partida congelada** | La API devuelve la misma copia de un match (puntos incluidos) durante minutos. El widget lo dice en naranja; `core::matchup::digest` es la huella que lo detecta. |
| **Ruta inferida** | Polilínea que une, en orden cronológico, los objetivos con claim de una guild. Es una inferencia, no un rastreo (ADR-005). |
| **Índice de nombres** | `guild-names.json`: nombres de clan para buscar por subcadena. Se distribuye en ZIP (`gw2_lurker_guilds.zip` + `.md5`) y **no** se tiene en memoria. |
| **`temp.json`** | Nombres de clan resueltos **en la sesión**. Se genera al arrancar y se borra en el siguiente. |
| **ADR** | Architecture Decision Record. |
| **EARS** | Easy Approach to Requirements Syntax: "Cuando <evento>, el sistema deberá <respuesta>". |
