# ADR-012 – Historial por sesión: se descarta al arrancar

- **Estado:** Aceptado (decisión del usuario, 2026-10-31). **Actualizado el 2026-11-01:** el historial ya no se escribe en disco en absoluto (ADR-015); sigue siendo de la sesión y vive en memoria. **Modifica** ADR-003 (formato) y sustituye la política de retención de RF-24 y RNF-03.
- **Contexto:** el usuario quiere que cada vez que se abre el juego el seguimiento empiece de cero. Con el juego cerrado no se observa nada, así que un historial persistente acumula rutas llenas de huecos que aparentan continuidad sin tenerla. Al arrancar, el baseline puede reconstruir el estado **actual** completo (todos los claims vigentes con su `claimed_at` real), que es lo que hace falta para empezar a seguir.
- **Decisión:**
  1. El historial de eventos es **de la sesión**. Al cargar el addon se descarta el historial de la sesión anterior y se crea uno nuevo con el formato de ADR-003 (`history/<fecha-sesión>.jsonl`).
  2. El primer snapshot de la sesión es baseline (RF-04) y registra **todos** los claims vigentes marcados `late=true` con su `claimed_at` real (RF-25), de modo que la ruta empieza con el mapa tal y como está.
  3. Se registra un evento `SessionStart` con `match_id`, hora local y `map_type`.
  4. La UI muestra siempre "datos desde <hora de inicio de sesión>" y el histórico no promete nada anterior. El banner de ruta inferida (RF-33) se mantiene.
  5. El ranking de guilds sugeridas (RF-20a) se calcula con los claims de la sesión actual.
- **Alternativas descartadas:** persistir 30 días (RF-24 anterior): aporta volumen y rutas incompletas sin valor, y complica el "¿desde cuándo hay datos?"; conservar una copia de la sesión anterior por defecto: no aporta al producto y confunde qué se está viendo.
- **Consecuencias:**
  - RF-24 pasa a "consulta dentro de la sesión"; RNF-03 pasa a un tope por sesión (2 MB) en lugar de 30 días / 20 MB.
  - Se pierden las rutas de sesiones anteriores; es un coste aceptado a cambio de claridad.
  - ADR-003 sigue vigente como formato y `trait EventStore`; solo cambia cuándo se rota el fichero.
  - Si más adelante se quiere un ranking que sobreviva a la sesión, se hará con **contadores agregados** (guild, mapa, día) en un fichero aparte, no revirtiendo esta decisión.
