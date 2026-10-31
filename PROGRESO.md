# Progreso: gw2-lurker

Bitácora para continuar el trabajo, incluso si quien lo retoma no es la misma IA ni la
misma sesión. **Lo más reciente va arriba.** Cada entrada dice qué se pretendía, qué se
hizo, cómo se verificó y qué quedó pendiente.

Reglas:
- Al **empezar** una tarea: añade una entrada `EN CURSO` con el plan en 3–5 líneas.
- **Durante**: actualiza esa entrada con hallazgos, decisiones y callejones sin salida.
- Al **terminar**: cámbiala a `HECHO`, pon los commits y lo que no se pudo verificar.
- No borres entradas antiguas. Si algo se revierte, dilo aquí y por qué.
- Lo que contradiga la spec se propone y se decide, no se entierra aquí.

---

## EN CURSO – 2026-10-31 · Bocetos HTML (T-32)

- Objetivo: `docs/boceto.html` navegable con widget, mapa, lista de guilds e historial, con
  datos de ejemplo reales, para validar la UI antes de escribirla en imgui.
- Bloqueado por: la skill `impeccable` exige `PRODUCT.md` (`PRODUCT_INIT_REQUIRED`); hay
  preguntas al usuario sin responder (superficie a validar, tono visual).
- Siguiente: escribir `PRODUCT.md` con las respuestas y construir el HTML.

## HECHO – 2026-10-31 · Cimiento: T-03, T-03a, T-04, T-05, T-06, T-07

- **Workspace** `x86_64-pc-windows-msvc`, 4 crates, lints del workspace (`unwrap_used` y
  `expect_used` en deny), perfil release de ADR-008, licencia **GPL-3.0-only**.
- **CI** (`.github/workflows/ci.yml`): fmt, clippy, test, `check_loc.sh`, `cargo deny`,
  `cargo audit`, build release msvc y `check_size.sh`. ~1 min por push.
  - Trampa encontrada: los `.sh` estaban en modo `100644` → `exit 126`. Arreglado con
    `git update-index --chmod=+x` y llamando con `bash`.
  - Trampa encontrada: `rustsec/audit-check` compila `cargo-audit` y costaba **339 s**.
    Sustituido por `taiki-e/install-action` + `cargo audit`.
- **`core::model`** (T-04): tipos serde de `/v2/wvw/matches`, `/v2/wvw/objectives`,
  `/v2/maps`, con fixtures reales en `crates/api/tests/fixtures/`.
- **`core::teams` + `core::resolver`** (T-05): tabla `data/wvw-teams.json` embebida con
  `include_str!`, `label()` con caída a `(id)`, `resolve()` sobre `all_worlds`.
- **`core::diff`** (T-06, T-07): `Differ` con `Baseline`/`MatchChanged`/`Gap` y, cuando no
  hay baseline, `Flip`/`Claim`/`Unclaim` ordenados por id de objetivo (`Unclaim` antes de
  `Claim`).
- Verificación: `cargo fmt --check`, `clippy -D warnings` y **31 tests** en verde; CI verde.
- Commits: `73b1eaa` (T-03), `add0bf2` (T-04 + licencia), `4195f85` (T-05), `69fa39b` (T-06),
  `36506ef` (T-07), `cd6441a`/`e7a30ad` (CI), `c03e0fb` (specs + backlog).

## HECHO – 2026-10-31 · Spikes T-02 y notas verificadas

- T-02 medido de verdad: 227 peticiones, 0 errores. Cambios de propiedad con mediana 19–20 s;
  un emparejamiento inactivo pasó **22 min 40 s sin un solo cambio**. Se mantiene 30 s.
- Hallazgo: la API **no** expone nombres de equipo (404 en `/v2/wvw/teams` y
  `/v2/wvw/alliances`). Sí da `guild → equipo` en `/v2/wvw/guilds/:region`.
- Hallazgo: `coord` está en coordenadas de continente; la caja correcta es `continent_rect`.
- Todo en `specs/07-spikes.md`, sección *Resultados*.

---

## Trampas ya conocidas (no volver a tropezar)

- `owner` viaja como `"Red"`/`"Green"`/`"Blue"` (PascalCase); los contenedores usan claves
  minúsculas. Mezclarlo rompe la deserialización.
- `claimed_by` es **una** guild o `null`, nunca una lista. Las claves siempre vienen.
- Torre totalmente mejorada llega a `points_tick` 8: "alto valor" no puede ser por tipo.
- Ids de equipo 11001–11015 (NA) y 12001–12018 (EU). Prefijo del match: `1-x` NA, `2-x` EU.
- Nexus expone `NexusLink::is_gameplay`: sirve para no dibujar en carga/selección.
- Publicar: `UpdateProvider::Direct` + `DisableHotloading`, MD5 de 32 hex sin BOM, excepción
  de LFS para la DLL, tag propio con `--latest=false`. Ver `docs/RELEASE.md`.

## Bloqueos

- **T-01, T-02b, T-25**: requieren ejecutar dentro de GW2 (carga, click-through, tamaño de
  DLL con TLS, comportamiento de actualización). No se simulan.
- **T-29 y T-30**: la matriz de trazabilidad las cita y no existen en `03-tasks.md`.
