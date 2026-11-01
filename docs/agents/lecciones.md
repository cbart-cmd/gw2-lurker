# Lecciones con contexto (referencia de AGENTS.md)

Texto completo de las reglas que en `AGENTS.md` van en una línea. Cada punto sale de un fallo
real. Léelo cuando la tarea toque tests, imgui, el candado `RUNTIME` o `settings.json`;
no hace falta cargarlo en cada sesión.

## Candados y guardado de ajustes

- **Nunca** llames a una función que bloquee `RUNTIME` (`runtime()`, `team::selected()`,
  `team::remember()`, `request_save`) mientras ya tengas el candado: `std::sync::Mutex` no es
  reentrante y el juego se congela (pasó el 2026-10-31 al dibujar los ajustes). Usa las
  variantes `*_in(&Config)` desde dentro del render o de cualquier código que ya lo tenga.
- La escritura de `settings.json` vive en su propio hilo (`addon::writer`); el render solo
  llama a `persist_if_changed()`, que está limitado a dos guardados por segundo.

## Tests: reglas que ya nos han costado un fallo

Cada punto de esta lista sale de un fallo real de este proyecto, no de una buena intención.

1. **El valor esperado sale de los datos, no de la memoria.** Antes de escribir un `assert_eq!`,
   lee la fixture o llama a la API. Fallos reales: supuse que `NA` iba antes que `EU`
   (alfabéticamente `E` < `N`); supuse `[Red, Blue, Green]` cuando `TeamColor::ALL` es
   `[Red, Green, Blue]`; supuse tres ids por color en `all_worlds` cuando también lleva ids de
   mundo antiguos (`2014`, `2101`).
2. **Mira lo que hay dentro de la fixture.** `crates/api/tests/fixtures/matches_all.json` solo
   contiene el matchup `2-2`: un test que asuma nueve falla. Si necesitas más, regenérala con
   `scripts/capture_fixtures.sh`.
3. **Un test del addon no puede tocar Nexus.** `AddonApi::get()` revienta fuera del juego
   (`nexus/src/globals.rs:83`), así que llamar a `get_mumble_link`, `runtime()` o
   `character_name()` dentro de un test es un pánico. La regla va en una función pura y se prueba
   esa: `team::resolve(manual, roster, account, character)`, `i18n::resolve(...)`,
   `window::opacity_percent(...)`.
4. **Nada de tests que reimplementen la lógica.** Repetir el `match` de producción en el test no
   prueba nada; hay que llamar al código real. Y si el candado (`Mutex`) es lo que no se puede
   ejercitar, se prueba la función pura y el candado queda en un envoltorio trivial: un
   interbloqueo de `Mutex` no reentrante ya congeló el juego el 2026-10-31.
5. **Antes de asumir una firma o un valor de una API externa, abre el fuente.** `~/.cargo/git/checkouts/…`
   o `~/.cargo/registry/src/…`, y cita el fichero en el comentario. Ejemplos medidos:
   `GameData::language` es `Result<GameLanguage, u32>` con `English=0, French=2, German=3,
   Spanish=4`; `Agent::team` es un `u16` **sin** mapeo verificado a los colores;
   `event_consume!` exige una `fn`, no una closure.
6. **Invariantes antes que números mágicos** cuando el dato viene de la API: «ordenado», «sin
   duplicados», «no vacío», «cada matchup en un bloque», «la suma cuadra». Los números exactos
   envejecen y rompen el CI sin que haya un fallo real.
7. **Red solo con `#[ignore]`**, y se lanzan a mano antes de publicar una versión:
   `cargo test -p gw2lurker-addon -- --ignored`. Existen `live_get_returns_the_match_json` y
   `live_pipeline_builds_the_dashboard` justo para eso.
8. **El nombre lleva el id del requisito** (`rf03a_…`, `rnf04_…`); si el requisito no existe,
   primero se escribe en la spec: `scripts/check_traceability.sh` falla si citas un id que no
   está.
9. **Si un test falla, decide primero si está mal el test o el código.** Si el test codificaba
   una suposición mía, se corrige el test y la suposición se escribe en el mensaje del `assert`;
   no se toca el código para contentar al test.
10. **Antes de decir «está probado»**: `cargo fmt --check`,
    `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`,
    `./scripts/check_loc.sh` y `./scripts/check_traceability.sh`; y si el cambio toca red o el
    juego, además la prueba `--ignored` correspondiente.
11. **Escribir de vuelta solo lo que el usuario tocó**, no la copia entera del formulario. Un
    `Edits` que se vuelca completo cada frame deshace lo que hizo el propio widget en ese mismo
    frame: con las casillas de *Ventanas*, la `×` de los ajustes se reabría sola. Marca
    `changed` en el `Edits` y aplica solo eso. Y cuidado con `config.window(key)`: **crea** la
    entrada si no existe, con `visible = true`, así que pedirla por una ventana cerrada la
    resucita.
12. **Los draw lists de imgui son singletons por frame.** `get_window_draw_list()` **panics**
    (`arcdps-imgui` `draw_list.rs:103`: «You can only load one instance of it!») si ya hay un
    `DrawListMut` de la ventana vivo. Se pide **una vez** y se pasa por referencia a quien dibuje;
    pedirlo dentro de un bucle cierra el juego en cuanto hay más de un elemento. Costó un cierre
    real al abrir la pestaña Mapa (2026-10-31); el `catch_unwind` del render lo dejó en una línea
    de `Nexus.log` en vez de una pantalla negra.
