# AGENTS.md – Instrucciones para agentes de IA

Addon de Nexus (Guild Wars 2) en Rust para Mundo contra Mundo: avisa de capturas, sigue guilds
enemigas y dibuja su historial. **Spec-driven**: la spec manda, el código la obedece.

## 1. Qué leer (y cuánto)

El contexto cuesta dinero: **lee secciones, no ficheros enteros**. Busca por ID (`RF-24`, `T-11c`,
`D-B07`) con `grep` y abre solo el rango que lo contiene.

1. `pwsh -File scripts/check_bugs.ps1` al empezar (§7).
2. `specs/00-constitution.md` (corto: entero) y `specs/04-engineering-standards.md` si tocas
   estilo, dependencias, CI o tamaño.
3. De `specs/01-requirements.md`, `02-design.md` y `03-tasks.md`, **solo** los IDs que cita la tarea.
4. `specs/06-diff-scenarios.md` si tocas `diff`, filtros de aviso, historial, nombres de clan o mapa.
5. `specs/07-spikes.md` → solo **Resultados**: es lo único verificado sobre Nexus y la API real.
6. `PROGRESO.md`: Trampas, Bloqueos y la entrada más reciente. Lo antiguo está en `docs/historial/`.
7. `CHANGELOG.md`: solo `[Unreleased]`. `docs/agents/lecciones.md` cuando toques tests, imgui,
   el candado `RUNTIME` o `settings.json`.

## 2. Reglas de oro

1. **Una tarea por vez.** No adelantes otras ni "aproveches" para refactorizar.
2. **Tests primero**, con el ID del requisito o escenario en el nombre (`rf10_…`, `d07_…`).
3. **No inventes la API de Nexus ni la de GW2.** Firma no verificada en `07-spikes.md` → lee el
   fuente (`zerthox/nexus-rs`, `~/.cargo/registry/src/…`) y cítalo. Si no puedes:
   `// UNVERIFIED(T-xx): motivo` y dilo en la entrega.
4. **No cambies la spec por tu cuenta**: para y propón cambio o ADR. Excepción: marcar tareas hechas.
5. **No relajes controles** (líneas, lints, tamaño de DLL, CI) para que algo pase.
6. **Nada de red ni E/S en el hilo de render.**
7. **Sin `unwrap()`/`expect()` en producción** ni `panic` que cruce FFI. `unsafe` solo en
   `crates/addon/src/ffi/`, con `// SAFETY:` en cada bloque.
8. **Ficheros ≤ 800 líneas** (aviso a 500). Divide por módulos antes de llegar.
9. **Dependencias nuevas**: justificadas (tamaño, licencia, mantenimiento) y con aprobación.
10. **Lo inferido se etiqueta como inferido** (rutas, claims tardíos). ADR-001, ADR-005.
11. **Nunca** llames a algo que bloquee `RUNTIME` (`runtime()`, `team::selected()`,
    `team::remember()`, `request_save`) con el candado ya cogido: `Mutex` no es reentrante y el
    juego se congela. Usa las variantes `*_in(&Config)`. `settings.json` se escribe en
    `addon::writer`; el render solo llama a `persist_if_changed()`.
12. **Tests**: el valor esperado sale de la fixture o la API, no de memoria; un test nunca llama a
    Nexus (`AddonApi::get()` revienta fuera del juego: prueba la función pura); no reimplementes la
    lógica en el test; red solo con `#[ignore]`; invariantes antes que números mágicos.
    Detalle y casos reales en `docs/agents/lecciones.md`.

## 3. Flujo por tarea

1. Resume en 3–5 líneas qué harás y qué RF/escenarios cubres. Si algo es ambiguo, **pregunta**.
2. Tests que fallen por la razón correcta → implementación mínima.
3. Mientras iteras: `pwsh -File scripts/validate.ps1 -Only test -Package <crate> -Filter <id>`.
4. Al estabilizar, **una** pasada completa: `pwsh -File scripts/validate.ps1` (fmt, clippy, loc,
   trazabilidad, tests; `-Release` añade build y tamaño). Imprime una línea por paso y deja el log
   completo en `target\validate\`; abre el log solo si algo falla. Tras un arreglo, repite solo
   los pasos afectados.
5. Docs afectadas (rustdoc, `CHANGELOG` si es visible, `03-tasks.md`), entrega (§5) y
   `check_bugs.ps1` antes de responder.

## 4. Mapa del repositorio

```
crates/core    lógica pura (sin Nexus, imgui ni red). #![forbid(unsafe_code)]
crates/api     cliente HTTP (transporte inyectable) y parseo. #![forbid(unsafe_code)]
crates/addon   cdylib: Nexus + imgui (único unsafe, en ffi/)
data/          wvw-teams.json y team-names.json (en la DLL); guild-names.json + ZIP (fuera)
specs/         diseño (no código)          scripts/  CI, validación, publicación
docs/          usuario, RELEASE.md, agents/ (lecciones) e historial/ (bitácora archivada)
```

No hay crate `store`: el historial es del ciclo de seguimiento y vive en memoria (ADR-015).

## 5. Formato de entrega

```
## Tarea: T-xx – <título>
### Qué se hizo
### Requisitos/escenarios cubiertos: RF-.., D-..
### Cómo verificar (salida de validate.ps1)
### No verificado / UNVERIFIED (si lo hay)
### Propuestas de cambio de spec o ADR (si las hay)
### Dependencias añadidas (si las hay, con justificación)
```

**Para y pregunta** si la tarea contradice la constitución o un ADR, pide una dependencia no
aprobada o `unsafe` fuera de `ffi/`, solo se puede probar en el juego (dilo; no lo simules), o falta
un dato que solo da el humano (fixture, captura, medición).

## 6. Entorno, publicación y CI

- DLL `x86_64-pc-windows-msvc` (host): `cargo build --release` **sin** `--target` →
  `target\release\gw2_lurker.dll`. El nombre lo fija `[lib] name = "gw2_lurker"` en
  `crates/addon/Cargo.toml`; sin él Nexus no la encuentra.
- Probar en local: `install.bat` o `install.ps1 [-Build] [-DryRun] [-Dest <ruta>]`. Pero Nexus
  sobrescribe la DLL con la publicada al arrancar: para probar en el juego hay que publicar.
- Idioma: código y comentarios en inglés; UI (con claves i18n, RNF-08), specs y docs en español.
  Commits convencionales (`feat:`, `fix:`, `docs:`…). Al subir versión, cambia también la
  `version` de las dependencias internas en `crates/*/Cargo.toml`.
- **Publicar**: `pwsh -File scripts/release.ps1 [-DryRun|-VerifyOnly]` en el repo compartido
  `cbart-cmd/gw2addonsreleases`. No lo repitas a mano ni toques ficheros de otros addons; el script
  aplica LFS, MD5, retención y verifica la descarga real. Detalle en `docs/RELEASE.md`.
- **CI**: tras `git push` no se espera. Al final del turno, una consulta del commit exacto:
  `gh run list --commit <sha> --limit 1 --json status,conclusion,url`. Si falla:
  `gh run view <id> --log-failed`, y se arregla como una tarea más. Prohibido `gh run watch`
  salvo petición expresa.

## 7. Ficheros compartidos con el usuario: `bugs.md` y `PROGRESO.md`

El usuario los edita a mano, incluso mientras trabajas.

- **Si faltan filas, las ha borrado él**: se acepta, no se restauran desde git ni se pregunta.
- **Nunca borres lo que él añada.** Relee justo antes de escribir, toca solo tus líneas y escribe
  con la herramienta `edit` y un ancla existente (un reemplazo de PowerShell se comió 14 entradas).
- `bugs.md`: `scripts/check_bugs.ps1` **al empezar, tras cada hito** (test en verde, arreglo
  aplicado, versión compilada) **y antes de entregar**. Imprime una línea si no cambió; ábrelo solo
  si dice `CAMBIADO`. Un bug arreglado se marca `[x]` con `- **Hecho (x.y.z):** qué y dónde`; si no
  ha salido en versión o requiere el juego, `[ ]` con `- **Pendiente:** …`. Nada se borra. Si ya
  estaba arreglado, dilo con la versión. Apúntalo también en `PROGRESO.md` y, si es visible, en
  `CHANGELOG.md`.
- `PROGRESO.md` existe para que otra IA pueda seguir. Entrada `EN CURSO` con el plan **antes** de
  tocar código; hallazgos y callejones sin salida mientras trabajas; `HECHO` con commits y lo no
  verificado al cerrar. Lo más reciente arriba; mantén al día **Trampas** y **Bloqueos**. Cuando
  supere ~30 KB, mueve las entradas antiguas **tal cual** a `docs/historial/`.

## 8. Gastar menos contexto sin perder calidad

- No re-leas lo que ya tienes; usa `view_range` y `grep` acotado. No vuelques fixtures ni logs.
- Para localizar código, `grep` en `modelo.md` (síntoma → fichero, y ficha por fichero) antes de
  abrir fuentes. Tras cambios de estructura: `pwsh -File scripts/gen_modelo.ps1`; si dice items
  sin descripción, `-ListMissing` y añádelos a `scripts/modelo-descripciones.txt`.
- Agrupa cambios coherentes antes de validar; no compiles tras cada línea.
- Subagentes solo para trabajo grande e independiente, nunca para un comando o una búsqueda.
- `gh … --json <campos>` antes que herramientas MCP para consultas simples a GitHub.
- Respuestas al usuario: lo hecho, lo verificado y lo pendiente; sin repetir el plan.
