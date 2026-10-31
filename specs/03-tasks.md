# 03 – Tareas

Cada tarea es pequeña y verificable. Orden recomendado; las dependencias se indican entre paréntesis.

## Fase 0 – Spikes (reducen riesgo)

- **T-01** Spike Nexus: addon mínimo con `export!` que dibuja una ventana imgui, lee `map_id` de Mumble y envía una alerta con `send_alert`. *Hecho cuando:* se carga en el juego y muestra el `map_id` en vivo. Confirma R-1 y R-6.
- **T-02** Spike API: script que consulta `/v2/wvw/matches?world=<id>` cada 10 s durante 30 min y registra cuándo cambian los datos. *Hecho cuando:* hay una cifra medida de caché real y de errores; se fija el intervalo por defecto (R-2).

## Fase 0b – Fundamentos de ingeniería

- **T-03 ✔ (2026-10-31)** Workspace Rust con `rustfmt.toml`, `[workspace.lints]` y `rust-version`. *Hecho:* `cargo fmt --check`, `clippy -D warnings` y `cargo test` verdes en un workspace vacío. Nota: no existe `tooling/` en el repo; la configuración se creó a mano (`rustfmt.toml`, `rust-toolchain.toml`, `deny.toml` ya estaban). `license` sigue sin decidir: **bloquea `cargo deny` en T-03a**.
- **T-03a ✔ (2026-10-31)** CI en `.github/workflows/ci.yml`: `cargo fmt --check`, `clippy -D warnings`, `cargo test`, `check_loc.sh`, `cargo deny` + `cargo audit`, build release `x86_64-pc-windows-msvc` y `check_size.sh`. *Pendiente por depender de otras tareas:* test de humo PE/export (T-27) y check de trazabilidad RF → tests (T-03b). Verificado que `check_loc.sh` falla con un fichero de 801 líneas.
- **T-03b** Check de trazabilidad RF → tests y `manual-tests.md`. (T-03a)
- **T-02b** Spike de tamaño: medir DLL mínima con 2–3 opciones de cliente HTTP/TLS y fijar presupuesto real (ADR-008). (T-01)
- **T-24** Revisión de bocetos `05-wireframes.md` y cierre de sus 3 pendientes.

## Fase 1 – Núcleo (sin juego)

- **T-03** Workspace + CI local (`fmt`, `clippy`, `test`). *Hecho:* `cargo test` verde. **(Duplicado de la T-03 de la Fase 0b: la parte de workspace está hecha y la de CI es T-03a.)**
- **T-04 ✔ (2026-10-31)** `core::model` y deserialización serde de fixtures reales del match y de objetivos. *Hecho:* 11 tests contra `crates/api/tests/fixtures/` (match `2-2`, objetivos, mapas). (T-03)
- **T-05 ✔ (2026-10-31)** `match_resolver`: equipo → color propio/enemigos buscando el `team_id` en `all_worlds`; nombres desde `data/wvw-teams.json` embebido (`include_str!`). *Hecho:* 6 tests con equipo, mundo enlazado, id desconocido, tabla corrupta e id sin nombre. (T-04) — RF-03a, RF-03b, RF-03g, RF-03i
- **T-06 ✔ (2026-10-31)** Baseline y cambio de match en `diff`: `Snapshot`, `Differ` con `Event::{Baseline, MatchChanged, Gap}`. *Hecho:* 8 tests (baseline de sesión, cambio de match, hueco > 3×intervalo y su borde exacto). (T-04) — RF-03d, RF-04, RF-26
- **T-07 ✔ (2026-10-31)** `diff` de flips y claims/unclaims con orden determinista. *Hecho:* `Event::{Flip, Claim, Unclaim}`; recorre los objetivos por id y emite `Unclaim` antes de `Claim`; el baseline no emite ninguno de los tres. 6 tests. (T-06) — RF-10, RF-21, RF-22
- **T-08** `rules`: ámbito, equipo propio, cooldown, guilds seguidas. (T-07) — RF-12, RF-13
- **T-09** Claims tardíos y `Gap`. (T-07) — RF-25, RF-26
- **T-10** `store` JSON-lines: append, query, línea corrupta, rotación/retención. (T-04) — RF-24, RNF-03
- **T-11** `follow` + `history`: estado actual por guild y consulta por rango. (T-09, T-10) — RF-23

## Fase 2 – API y worker

- **T-11b** `api`: cliente con `trait ApiClient`, backoff exponencial, búsqueda de guild, caché de nombres/objetivos en disco. (T-04) — RF-20, RNF-04
- **T-11c** `worker`: hilo de polling con parada limpia y estado de salud. (T-11b, T-07)

## Fase 3 – Integración Nexus

- **T-31** `i18n`: tabla es/en/fr/de, detección del idioma del juego (RTAPI + fallbacks) y override con Ctrl derecho tras `ffi/`. *Hecho:* test que recorre las claves y falla si falta una columna; el idioma cambia con Ctrl der. en el juego. (T-03) — RF-44, RF-45, RNF-08

- **T-12** `context` + `config`: activación solo en WvW, persistencia, selector de mundo. (T-01, T-05) — RF-01, RF-02, RF-03a/c, RF-40, RF-41
- **T-13** Widget fijo de alertas: lista, fade, colores/formas por tipo, mover/redimensionar y persistir posición. (T-08, T-12) — RF-10, RF-16, RF-17, RF-19, RF-19a
- **T-13b** Modo bloqueado click-through, atajo de bloqueo/desbloqueo y ventana de log ampliada. (T-13) — RF-14, RF-18
- **T-17** Ventana de configuración completa e indicador de estado/obsoleto. (T-12) — RF-43

## Fase 4 – Mapas

- **T-14** Proyección `world_to_screen` + mapa simplificado con colores y formas. (T-04, T-12) — RF-30
- **T-15** Capa de claims y vista de estado por guild. (T-14, T-11) — RF-31, RF-23
- **T-16** Vista Histórico: ruta simulada numerada, flechas, selector de rango, banner de "inferida", estilo para `late`/`gap`, tooltip. (T-15) — RF-32, RF-33, RF-34

## Fase 4b – Actualización y fallo seguro

- **T-25** Spike y aviso de actualización: comprobar cómo deja Nexus la DLL nueva y construir el banner W7 (ADR-007). (T-01, T-12) — RF-50, RF-53
- **T-26** Modo seguro: config/historial corruptos → `.corrupt`, valores por defecto, aviso W8; `catch_unwind` en la frontera FFI. (T-10, T-12) — RF-51, RNF-06
- **T-27** Pipeline de release: DLL en CI, SHA-256, test de humo PE/export con crate `object`, registro de tamaño en CHANGELOG. (T-03a, T-02b) — RNF-09, RNF-12

## Fase 5 – Aceptación y pulido

- **T-18** Medición de frame time con mapa abierto. *Hecho:* < 0.5 ms (RNF-01).
- **T-19** Checklist manual en el juego: widget bloqueado sin capturar clics, activar/desactivar por mapa, reset semanal, sin red, API 503, guild con claims reales.
- **T-20** README, CHANGELOG, `docs/` de usuario, instalación, limitaciones conocidas (RNF-05), empaquetado de la DLL.
- **T-28** Módulo `theme`: herencia del estilo de Nexus, tema propio opcional aplicado solo a ventanas del addon (stack de estilo push/pop), colores de equipo fijos y fondo del widget configurable. (T-13, T-17) — RF-60..63
- **T-21 (S)** RF-11, RF-15, RF-19b, RF-19c, RF-35, atajo/icono de Quick Access.
- **T-22 (C)** RF-19d, RF-36, reproducción animada de la ruta.
