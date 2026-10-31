# START-HERE – Guía para ti (la persona)

Esta guía te dice qué hacer tú y cómo trabajar con la IA. Los detalles de qué construir están en `specs/`; las instrucciones que lee la IA, en `AGENTS.md`.

## 1. Qué hay en el paquete

| Fichero | Para quién | Qué es |
|---|---|---|
| `START-HERE.md` | Tú | Esta guía |
| `AGENTS.md` / `CLAUDE.md` | La IA | Reglas, flujo por tarea y formato de entrega |
| `specs/` | Ambos | Constitución, requisitos, diseño, tareas, estándares, bocetos, escenarios, spikes, ADRs |
| `scripts/` | Ambos | Comprobaciones de CI y captura de datos reales |
| `rustfmt.toml`, `deny.toml`, `rust-toolchain.toml`, `.editorconfig`, `.gitignore` | Herramientas | Configuración en la raíz |
| `.github/` | CI | Workflow y plantilla de PR |

## 2. Antes de pedir nada a la IA (checklist)

**A. Cuenta y repositorio**
- [ ] Crear el repositorio en GitHub (privado al principio sirve; público si quieres que Nexus lo actualice con `UpdateProvider::GitHub`).
- [ ] Copiar todo este paquete a la raíz del repo y hacer el primer commit.
- [ ] Activar protección de la rama `main` (PR obligatorio y CI verde).
- [ ] Decidir **nombre final** y **licencia** (hasta entonces, el nombre es provisional). Añadir `LICENSE`.

**B. Entorno**
- [ ] Instalar Rust con `rustup` y las herramientas de compilación de Windows (Visual Studio Build Tools, carga "Desarrollo de escritorio con C++").
- [ ] Instalar `cargo-deny` (`cargo install cargo-deny`).
- [ ] Tener una copia de GW2 con Nexus funcionando para probar.

**C. Spikes (los haces tú; no hay atajo)** — guía en `specs/07-spikes.md`
- [ ] **T-01** addon mínimo en Nexus (incluye: Mumble `world_id` vs id del match, nombre API vs nombre en juego, texturas, `continent_rect`).
- [ ] **T-02** medir la API (y capturar fixtures con `capture_fixtures`).
- [ ] **T-02b** tamaño de DLL con distintos clientes HTTP.
- [ ] **T-25** cómo actualiza Nexus.
- [ ] Rellenar la sección *Resultados* de `07-spikes.md` y hacer commit.

**D. Pasada de coherencia**
- [ ] Leer `specs/01-requirements.md` y `specs/05-wireframes.md` y comprobar que dicen lo que quieres.
- [ ] Revisar las políticas de terceros de ArenaNet y de addons de Nexus (la API pública sin clave debería encajar, pero lo debes confirmar tú).

> Puedes empezar con T-03 (scaffold) **sin** los spikes, pero no con tareas que toquen Nexus o la API real (T-04 en adelante usa fixtures de T-02).

## 3. Cómo trabajar con la IA

**Una tarea por conversación**, en el orden de `specs/03-tasks.md`. Empieza por T-03.

**Prompt para la primera tarea**
```
Lee AGENTS.md y sigue sus instrucciones. Haz solo la tarea T-03 de specs/03-tasks.md.
Antes de programar, resúmeme en 5 líneas qué vas a hacer y pregúntame lo que sea ambiguo.
```

**Prompt para las siguientes**
```
Lee AGENTS.md. Haz solo la tarea T-XX de specs/03-tasks.md.
Escribe primero los tests (IDs en los nombres), luego el código, y entrega con el formato de AGENTS.md §5.
Los spikes verificados están en specs/07-spikes.md; no supongas nada de Nexus que no esté ahí.
```

**Prompt tras completar spikes**
```
Lee specs/07-spikes.md (sección Resultados). Actualiza specs/02-design.md y los ADR afectados
(ADR-007, ADR-008) marcando lo verificado. Si algún resultado contradice la spec,
propón los cambios sin aplicarlos y explícame el impacto.
```

**Prompt de revisión de una entrega**
```
Revisa este diff contra specs/00-constitution.md y specs/04-engineering-standards.md.
Lista incumplimientos (unwrap, unsafe fuera de ffi, E/S en render, ficheros > 800 líneas, dependencias sin justificar),
requisitos sin test y cualquier cosa marcada UNVERIFIED.
```

## 4. Cómo revisar cada entrega (5 minutos)

1. ¿Hace **solo** lo que pedía la tarea?
2. Ejecuta tú: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && ./scripts/check_loc.sh && cargo test --workspace`.
3. ¿Hay tests con IDs (`rf10_`, `d07_`)? ¿Fallaban antes de implementar?
4. ¿Hay `UNVERIFIED`? Es honesto; anótalo para probarlo en el juego (`specs/manual-tests.md`).
5. ¿Añadió dependencias o cambió la spec? Solo si lo justificó y lo apruebas.
6. Commit con mensaje convencional y PR.

## 5. Orden recomendado de tareas

1. T-03, T-03a (workspace + CI) → 2. T-04 → T-07 → T-06, T-08, T-09 (`core`, con `06-diff-scenarios.md`) → 3. T-10, T-11 (`store`, historial) → 4. T-11b, T-11c (API y worker) → 5. T-12, T-13, T-13b, T-17, T-28 (Nexus y UI) → 6. T-14..T-16 (mapas) → 7. T-25..T-27 (actualización, modo seguro, release) → 8. T-18..T-20 (aceptación y docs).

## 6. Si algo sale mal

| Síntoma | Qué hacer |
|---|---|
| La IA inventa funciones de `nexus-rs` | Recuérdale la regla 3 de `AGENTS.md` y pídele que cite el fichero fuente |
| La IA toca la spec sin avisar | Descarta ese cambio y pide el ADR primero |
| Un test es difícil de escribir | Probablemente falta un escenario: añádelo a `06-diff-scenarios.md` |
| La DLL crece mucho | `cargo bloat --release --crates` y revisar la última dependencia añadida |
| Contradicción entre dos requisitos | Se resuelve con un ADR, no en el código |

## 7. Limitaciones que conviene recordar

- No hay detección de "bajo ataque" (ADR-001): son avisos de captura.
- El historial solo cubre el tiempo con el addon activo y puede perder claims entre polls (RNF-05).
- Los scripts `.ps1` están sin probar; los `.sh` pasaron comprobación de sintaxis. Si fallan, pega el error a la IA y pídele que los corrija.
