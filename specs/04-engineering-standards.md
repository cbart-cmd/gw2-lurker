# 04 – Estándares de ingeniería

Cumplimiento verificado por CI siempre que sea posible. Lo que no se pueda automatizar se revisa en PR con la checklist del final.

## 1. Formato y estilo

- **rustfmt** obligatorio, configuración en `rustfmt.toml` (ver `tooling/`). `cargo fmt --check` en CI.
- **clippy** con `-D warnings`. Lints del workspace en `[workspace.lints]`: `clippy::all` y `clippy::pedantic` como warn, con excepciones justificadas en línea (`#[allow(..., reason = "...")]`).
- Nomenclatura estándar de Rust (RFC 430): `snake_case` funciones/módulos, `CamelCase` tipos, `SCREAMING_SNAKE_CASE` constantes.
- Edición 2021 o superior, MSRV fijada en `rust-version` y comprobada en CI.
- Sin código comentado ni `TODO` sin identificador de tarea (`TODO(T-14): ...`).

## 2. Tamaño de ficheros y funciones

- **Máximo 800 líneas por fichero `.rs`** (duro, CI falla). Aviso a las 500.
- Funciones: objetivo ≤ 60 líneas; `clippy::too_many_lines` y `cognitive_complexity` en warn.
- Un módulo, una responsabilidad: si un fichero se acerca al límite, se divide en un directorio con `mod.rs` y submódulos (ver diseño §1).
- Los tests extensos van en `tests/` o en ficheros `*_tests.rs` aparte para no inflar el módulo bajo prueba (cuentan igual para el límite).
- Script: `tooling/scripts/check_loc.sh`.

## 3. Errores, seguridad y robustez

- Prohibido `unwrap()`/`expect()` fuera de tests y de invariantes documentadas (`clippy::unwrap_used` en deny para el crate `addon` y `api`).
- Errores tipados con `thiserror` en librerías; sin `anyhow` en `core`.
- `#![forbid(unsafe_code)]` en `core`, `api` y `store`. El `unsafe` solo se permite en `addon`, aislado en un módulo `ffi` con comentario `// SAFETY:` en cada bloque.
- **Frontera FFI:** todo callback que invoque Nexus envuelve su cuerpo en `catch_unwind` y registra el error en lugar de propagarlo. `panic = "unwind"` en el perfil release (ver §5, ADR-008).
- Sin bloqueo en el hilo de render: ni E/S, ni red, ni locks largos. Revisión explícita en PR.
- Entradas externas (JSON de API, ficheros de historial y config) se tratan como no confiables: límites de tamaño, validación y descarte de lo corrupto, nunca `panic`.

## 4. Dependencias

- Política de mínimos: cada dependencia nueva requiere justificarse en el PR (coste en tamaño de DLL, mantenimiento, licencia).
- `cargo-deny` (licencias, avisos de seguridad, fuentes) y `cargo audit` en CI. Configuración en `tooling/deny.toml`.
- Sin runtime asíncrono (`tokio`) salvo que se demuestre necesario; el worker usa un hilo y cliente HTTP bloqueante.
- Versiones fijadas con `Cargo.lock` versionado.
- Elección del cliente HTTP y de TLS por **medición de tamaño** en el spike T-02b, no por preferencia.

## 5. Tamaño de la DLL (ADR-008)

Presupuesto **provisional**, a calibrar con el addon mínimo del spike T-01:

| Medida | Objetivo | Límite CI |
|---|---|---|
| DLL release | ≤ 1.5 MB | 2.5 MB |

Perfil release recomendado:

```toml
[profile.release]
opt-level = "z"        # optimiza tamaño; medir también "s" y 3 si afecta al rendimiento
lto = "fat"
codegen-units = 1
strip = "symbols"
panic = "unwind"       # necesario para catch_unwind en la frontera FFI
debug = false
```

- `panic = "abort"` reduce tamaño, pero convertiría cualquier panic en un cierre del juego. Se descarta por la constitución (principio 7).
- Evitar `regex`, `chrono` con todas las features, `reqwest`/`tokio`. Usar `time` o un parser ISO-8601 mínimo.
- Revisar con `cargo bloat --release --crates` en cada release; el tamaño se registra en el CHANGELOG.
- Script de comprobación: `tooling/scripts/check_size.sh`.

## 6. Estrategia de pruebas

| Tipo | Herramienta | Alcance |
|---|---|---|
| Unitarias | `cargo test` | `core`, `store`, `api` (parseo) |
| Propiedades | `proptest` | `diff` (idempotencia: `diff(s,s)=∅`), `world_to_screen` (inversa), rotación de `store` |
| Instantáneas | `insta` | Salida de `diff`/`rules` sobre fixtures reales |
| Fixtures | JSON reales guardados en `crates/api/tests/fixtures/` | Respuestas de match, objetivos, guild, errores 503 y JSON truncado |
| Cobertura | `cargo llvm-cov` | `core` ≥ 85 %, `store` ≥ 80 % |
| Mutación (opcional) | `cargo mutants` | `core::diff` y `core::rules` |
| Humo de DLL | test con crate `object` | La DLL generada exporta `GetAddonDef` y es un PE válido x64 |
| Aceptación manual | Checklist T-19 | Todo lo que dependa del juego |

**Trazabilidad:** cada test de comportamiento lleva el ID del requisito en el nombre (`rf10_flip_of_own_objective_emits_alert`). Un check de CI lista los requisitos M sin ningún test que los referencie; los que solo pueden probarse a mano se enumeran en `specs/manual-tests.md`.

## 7. Documentación

- `#![warn(missing_docs)]` en `core`, `api` y `store`; rustdoc con ejemplos ejecutables (doctests) en la API pública.
- `README.md` (instalación, límites conocidos, capturas), `CHANGELOG.md` (Keep a Changelog + SemVer), `docs/` para el usuario final.
- **Spec-driven:** ningún cambio de comportamiento sin actualizar antes `01-requirements.md` y, si procede, un ADR. El PR enlaza los RF/T afectados.
- ADRs inmutables una vez aceptados; los cambios se hacen con un ADR nuevo que sustituye al anterior.

## 8. Control de versiones y releases

- Commits convencionales (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`).
- Ramas cortas y PR obligatorio; CI verde para fusionar.
- Release: etiqueta SemVer, DLL compilada en CI, **SHA-256 publicado** junto al artefacto, notas del CHANGELOG, tamaño registrado.

## 9. CI mínima (orden)

1. `cargo fmt --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `check_loc.sh` (≤ 800 líneas)
4. `cargo test --workspace`
5. `cargo deny check` y `cargo audit`
6. Build release de `addon` para `x86_64-pc-windows-msvc`
7. `check_size.sh` y test de humo de la DLL
8. Check de trazabilidad RF → tests

## 10. Checklist de PR

- [ ] Enlaza RF/T/ADR afectados y la spec está actualizada.
- [ ] Sin E/S ni red en el hilo de render.
- [ ] Sin `unwrap` nuevos fuera de tests; `unsafe` solo con `SAFETY`.
- [ ] Tests nuevos con IDs de requisito en el nombre.
- [ ] Ningún fichero > 800 líneas; DLL dentro del presupuesto.
- [ ] Dependencias nuevas justificadas.
- [ ] CHANGELOG actualizado si cambia algo visible.
