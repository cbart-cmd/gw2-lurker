# AGENTS.md – Instrucciones para agentes de IA

Proyecto: addon de Nexus (Guild Wars 2) en Rust para Mundo contra Mundo. Avisa de capturas de estructuras, sigue guilds enemigas y dibuja su historial. **Spec-driven**: la spec manda, el código la obedece.

## 1. Lectura obligatoria, en este orden

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

- Target de la DLL: `x86_64-pc-windows-msvc`. Si no puedes compilar ese target, indícalo y limita la verificación a `core`, `api` y `store`.
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

## 9. Progreso y continuidad (`PROGRESO.md`)

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

## 10. CI: lanzar y seguir

- Después de `git push`, **no se espera al pipeline**. Se anuncia ("CI lanzada; puede
  fallar algo") y se continúa con la siguiente tarea.
- Prohibido `gh run watch` en primer plano salvo que el usuario lo pida. Un `gh run list`
  suelto al final del turno basta para saber cómo fue.
- Cuando llegue el resultado en rojo, se diagnostica con `gh run view --log-failed` y se
  arregla como una tarea más, sin dramatizar y sin bloquear lo que esté en curso.
- Si el pipeline tarda más de ~2 minutos es normal: compila Nexus e ImGui desde cero.
