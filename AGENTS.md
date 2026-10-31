# AGENTS.md – Instrucciones para agentes de IA

Proyecto: addon de Nexus (Guild Wars 2) en Rust para Mundo contra Mundo. Avisa de capturas de estructuras, sigue guilds enemigas y dibuja su historial. **Spec-driven**: la spec manda, el código la obedece.

## 1. Lectura obligatoria, en este orden

0. `bugs.md` (**siempre**: antes de empezar, **después de cada paso** y al terminar; ver §9).
1. `specs/00-constitution.md` – reglas no negociables.
2. `specs/04-engineering-standards.md` – estilo, límites y CI.
3. `specs/03-tasks.md` – localiza **la única tarea** que te han pedido.
4. Los requisitos (`specs/01-requirements.md`) y secciones de `specs/02-design.md` que la tarea cita.
5. Si la tarea toca `diff`, `rules`, `store` o `history`: `specs/06-diff-scenarios.md`.
6. `specs/07-spikes.md` → sección **Resultados**: es lo único verificado sobre Nexus y la API real. Lo que no esté ahí no está confirmado.

## 2. Reglas de oro

1. **Una tarea por vez.** No adelantes otras, no "aproveches" para refactorizar.
2. **Tests primero.** Cada test de comportamiento lleva el ID del requisito o escenario en el nombre (`rf10_...`, `d07_...`).
3. **No inventes la API de Nexus ni la de GW2.** Si necesitas una firma de `nexus-rs` que no está verificada en `07-spikes.md`, lee su código fuente (repositorio `zerthox/nexus-rs`) y cita el fichero. Si no puedes comprobarla, deja el código marcado `// UNVERIFIED(T-xx): motivo` y dilo en tu entrega.
4. **No cambies la spec por tu cuenta.** Si ves un error o un hueco en la spec, **para y propón** un cambio o un ADR en tu respuesta. Excepción: marcar tareas como hechas en `03-tasks.md`.
5. **No relajes los controles** (límites de líneas, lints, tamaño de DLL, CI) para que algo pase.
6. **Nada de red ni E/S en el hilo de render.**
7. **Sin `unwrap()`/`expect()` en código de producción**, ni `panic` que cruce la frontera FFI. `unsafe` solo en `crates/addon/src/ffi/`, con `// SAFETY:` en cada bloque.
8. **Ficheros ≤ 800 líneas** (aviso a las 500). Divide por módulos antes de llegar.
9. **Dependencias nuevas:** no las añadas sin justificarlas (coste en tamaño de DLL, licencia, mantenimiento) y pedir aprobación.
10. **Datos inferidos se etiquetan como inferidos** (rutas, claims tardíos). No prometas más de lo que la API da (ADR-001, ADR-005).

## 3. Flujo de trabajo por tarea

0. **Lee `bugs.md`.** Y vuelve a leerlo después de cada paso de esta lista (§9): el usuario
   apunta fallos mientras trabajas y una entrada nueva no puede esperar a la siguiente sesión.
1. Resume en 3–5 líneas qué vas a hacer y qué RF/escenarios cubres. Si algo es ambiguo, **pregunta antes de programar**.
2. Escribe los tests (deben fallar por la razón correcta).
3. Implementa lo mínimo para que pasen.
4. Ejecuta, desde la raíz del repo, y pega la salida resumida:
   ```
   cargo fmt --check
   cargo clippy --workspace --all-targets -- -D warnings
   ./scripts/check_loc.sh
   cargo test --workspace
   ```
5. Actualiza la documentación afectada (rustdoc, CHANGELOG si es visible para el usuario, `03-tasks.md`).
6. Entrega con el formato de la sección 5.

## 4. Mapa del repositorio

```
crates/core    lógica pura (sin Nexus, sin imgui, sin red). #![forbid(unsafe_code)]
crates/api     cliente HTTP y parseo. #![forbid(unsafe_code)]
crates/store   persistencia JSON-lines. #![forbid(unsafe_code)]
crates/addon   cdylib: integración Nexus + imgui (único sitio con unsafe, en ffi/)
specs/         documentación de diseño (no código)
scripts/       comprobaciones de CI
docs/          documentación de usuario final
```

## 5. Formato de entrega

```
## Tarea: T-xx – <título>
### Qué se hizo
### Requisitos/escenarios cubiertos: RF-.., D-..
### Cómo verificar (comandos y resultado)
### No verificado / UNVERIFIED (si lo hay)
### Propuestas de cambio de spec o ADR (si las hay)
### Dependencias añadidas (si las hay, con justificación)
```

## 6. Cuándo parar y preguntar

- La tarea contradice la constitución o un ADR.
- Hace falta una dependencia no aprobada o un `unsafe` fuera de `ffi/`.
- Necesitas ejecutar algo que solo se puede probar dentro del juego (dilo; no lo simules).
- Dos requisitos se contradicen o falta un dato que solo el humano puede dar (fixture, captura, medición).

## 7. Entorno

- Target de la DLL: `x86_64-pc-windows-msvc`, que es el del host en Windows. Se compila con `cargo build --release` **sin** `--target`, igual que los proyectos vecinos, para que la DLL quede en `target\release\gw2_lurker.dll`. Pasar `--target` anida la salida en `target\<triple>\release\` y duplica caché sin ganar nada. Si no puedes compilar el addon (p. ej. en Linux), indícalo y limita la verificación a `core`, `api` y `store`.
- El nombre del fichero lo fija `[lib] name = "gw2_lurker"` en `crates/addon/Cargo.toml`. Sin esa línea Cargo usa el nombre del paquete y produce `gw2lurker_addon.dll`, que Nexus no buscaría.
- Para probar en local: `install.bat` (doble clic) o `install.ps1 [-Build] [-DryRun] [-Dest <ruta>]`. Copia `gw2_lurker.dll` a `<juego>\addons` verificando tamaño, `MZ` y MD5, y se niega a copiar con GW2 abierto.
- Idioma: código y comentarios en inglés; textos de UI, specs y commits de documentación en español (con claves i18n, RNF-08).
- Commits convencionales (`feat:`, `fix:`, `test:`, `docs:`, `refactor:`, `chore:`).

## 8. Publicación (Nexus)

La DLL se distribuye desde **`https://github.com/cbart-cmd/gw2addonsreleases`**, un repositorio **compartido con otros addons**. Procedimiento completo y comprobaciones obligatorias en [`docs/RELEASE.md`](docs/RELEASE.md). Resumen:

- Ficheros propios: `gw2_lurker.dll` + `gw2_lurker.dll.md5` (32 hex en minúsculas, sin BOM ni salto de línea). `update_link` = `https://raw.githubusercontent.com/cbart-cmd/gw2addonsreleases/main/gw2_lurker.dll`.
- `UpdateProvider::Direct` + `AddonFlags::DisableHotloading` (ver RF-53 y ADR-007).
- **No tocar** los ficheros de los proyectos hermanos. Etiqueta `gw2-lurker-v<version>` y `--latest=false`.
- Retención: **release actual + la anterior**; se borra la de dos versiones atrás.
- Excepción de LFS para nuestra DLL en `.gitattributes` del repo de distribución **antes** de `git add`, o Nexus recibe un puntero de texto en vez de la DLL.
- Verificar la descarga real (bytes, `MZ`, MD5) desde la URL exacta, no fiarse de un HTTP 200.
- **Automatizado**: `pwsh -File scripts/release.ps1 -DryRun` (compila y valida sin tocar lo
  remoto), `pwsh -File scripts/release.ps1` (publica) y `-VerifyOnly` (solo comprueba la URL
  publicada). No repetir a mano los diez pasos: el script aborta si el blob es un puntero LFS,
  si el `.md5` no mide 32 bytes o si la descarga no coincide en bytes y MD5.
- El nombre del fichero sale del `[lib] name` de `crates/addon`: **`gw2_lurker`**, no
  `gw2lurker_addon`. Si alguien lo cambia, Nexus busca un nombre que ya no existe.

- **Nunca** llames a una función que bloquee `RUNTIME` (`runtime()`, `team::selected()`,
  `team::remember()`, `request_save`) mientras ya tengas el candado: `std::sync::Mutex` no es
  reentrante y el juego se congela (pasó el 2026-10-31 al dibujar los ajustes). Usa las
  variantes `*_in(&Config)` desde dentro del render o de cualquier código que ya lo tenga.
- La escritura de `settings.json` vive en su propio hilo (`addon::writer`); el render solo
  llama a `persist_if_changed()`, que está limitado a dos guardados por segundo.

## 9. `bugs.md`: la bandeja de fallos del usuario

`bugs.md`, en la raíz, es donde el usuario apunta fallos en una línea, sin formato.

- **Léelo antes de empezar, después de cada paso y al terminar la tanda.** "Cada paso" es cada
  test en verde, cada arreglo aplicado, cada versión compilada: se vuelve a abrir el fichero por
  si el usuario ha apuntado algo nuevo mientras trabajabas. Es obligatorio, no opcional.
- **No se borra nada.** Cuando creas que un bug está arreglado, **márcalo con `[x]`** y añade
  `- **Hecho (x.y.z):** qué se cambió y dónde` (petición del usuario, 2026-10-31). El jugador decide
  si se puede cerrar; borrar su reporte le quita la forma de comprobarlo. Nunca se vacía el fichero
  ni se recrea.
- Si el arreglo aún no ha salido en una versión, déjalo en `[ ]` con `- **Pendiente:** …`.
- Si un bug no se puede reproducir o necesita al usuario (p. ej. hay que probarlo en el juego),
  déjalo en `[ ]` y añade una línea `- **Pendiente:** …` explicando qué falta.
- Si el usuario reporta algo que ya está arreglado, dilo en la respuesta con la versión en la que
  salió; no hay nada que apuntar.
- Un bug arreglado también se apunta en `PROGRESO.md` (una línea) y, si es visible, en
  `CHANGELOG.md`.

## 10. Progreso y continuidad (`PROGRESO.md`)
`PROGRESO.md`, en la raíz, es la bitácora del trabajo. Existe para que **otra IA pueda
seguir** si la actual se corta a medias.

- Al empezar una tarea, escribe una entrada `EN CURSO` con el plan (3–5 líneas) y qué RF
  cubre **antes** de tocar código.
- Mientras trabajas, actualízala con hallazgos, decisiones y callejones sin salida: si un
  experimento falla, eso también se apunta.
- Al cerrar, pasa la entrada a `HECHO` con los commits y lo que no se pudo verificar.
- Lo más reciente arriba. No se borran entradas; si algo se revierte, se documenta.
- Mantén al día las secciones **Trampas ya conocidas** y **Bloqueos**: son lo primero que
  debe leer quien retome el trabajo.

## 11. Tests: reglas que ya nos han costado un fallo

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
   esa: `team::resolve(manual, roster, character)`, `i18n::resolve(...)`,
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

## 12. CI: lanzar y seguir

- Después de `git push`, **no se espera al pipeline**. Se anuncia ("CI lanzada; puede
  fallar algo") y se continúa con la siguiente tarea.
- Prohibido `gh run watch` en primer plano salvo que el usuario lo pida. Un `gh run list`
  suelto al final del turno basta para saber cómo fue.
- Cuando llegue el resultado en rojo, se diagnostica con `gh run view --log-failed` y se
  arregla como una tarea más, sin dramatizar y sin bloquear lo que esté en curso.
- Si el pipeline tarda más de ~2 minutos es normal: compila Nexus e ImGui desde cero.
