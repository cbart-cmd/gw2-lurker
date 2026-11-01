# 04 – Estándares de ingeniería

Cumplimiento verificado por CI siempre que sea posible. Lo que no se pueda automatizar se revisa en PR con la checklist del final.

## 1. Formato y estilo

- **rustfmt** obligatorio, configuración en `rustfmt.toml`. `cargo fmt --all --check` en CI.
- **clippy** con `-D warnings` en CI. Lints del workspace en `Cargo.toml` →
  `[workspace.lints]`: `clippy::all` como warn, `unwrap_used` y `expect_used` en **deny** (los
  tests se los saltan con `#![cfg_attr(test, allow(...))]`), `missing_debug_implementations` y
  `unused_qualifications` warn, `unsafe_op_in_unsafe_fn` deny.
- Nomenclatura estándar de Rust (RFC 430): `snake_case` funciones/módulos, `CamelCase` tipos, `SCREAMING_SNAKE_CASE` constantes.
- Edición 2021, MSRV en `rust-version` (**1.88.0**, por `zip` 9) y toolchain `stable` en `rust-toolchain.toml`.
- Sin código comentado ni `TODO` sin identificador de tarea (`TODO(T-14): ...`).

## 2. Tamaño de ficheros y funciones

- **Máximo 800 líneas por fichero `.rs`** (duro, CI falla). Aviso a las 500.
- Funciones: objetivo ≤ 60 líneas; `clippy::too_many_lines` y `cognitive_complexity` en warn.
- Un módulo, una responsabilidad: si un fichero se acerca al límite, se divide en un directorio con `mod.rs` y submódulos (ver diseño §1).
- Los tests extensos van en `tests/` o en ficheros `*_tests.rs` aparte para no inflar el módulo bajo prueba (cuentan igual para el límite).
- Script: `scripts/check_loc.sh` (lo lanza la CI).

## 3. Errores, seguridad y robustez

- Prohibido `unwrap()`/`expect()` fuera de tests: `clippy::unwrap_used` y `clippy::expect_used` están en **deny** en todo el workspace.
- Errores con enums propios y `Display` escrito a mano (`gw2lurker_api::ApiError`, `Health` del worker). **No** se usan `thiserror` ni `anyhow`: no aportan y suman tamaño.
- `#![forbid(unsafe_code)]` en `core` y `api`. El `unsafe` solo se permite en `addon`, aislado en `ffi/` con `// SAFETY:` en cada bloque.
- **Frontera FFI:** todo callback que invoque Nexus envuelve su cuerpo en `catch_unwind` y registra el error en lugar de propagarlo. `panic = "unwind"` en el perfil release (ver §5, ADR-008).
- Sin bloqueo en el hilo de render: ni E/S, ni red, ni locks largos. Revisión explícita en PR.
- Entradas externas (JSON de API, `settings.json`, el ZIP del índice que se descarga) se tratan como no confiables: límites de tamaño y de entradas del ZIP, validación del MD5 publicado, y descarte de lo corrupto, nunca `panic`.

## 4. Dependencias

- Política de mínimos: cada dependencia nueva requiere justificarse en el PR (coste en tamaño de DLL, mantenimiento, licencia). Dependencias actuales del `addon`: `chrono`, `log`, `serde`, `serde_json`, `nexus`, `gw2lurker-core`, `gw2lurker-api` y **`zip` 9.x** (`default-features = false, features = ["deflate"]`, para descomprimir el índice de nombres). Se subió de 2.x a 9.x el 2026-11-01 por tres RUSTSEC sobre `extract()`; el addon no lo usa, pero un aviso no se silencia.
- `cargo-deny` (licencias, avisos de seguridad, fuentes) y `cargo-audit` en CI, con `deny.toml` en la raíz.
- Sin runtime asíncrono (`tokio`): el worker es un hilo y el HTTP es **WinHTTP** (servicio del sistema, sin TLS en la DLL).
- Versiones fijadas con `Cargo.lock` versionado.
- Cliente HTTP decidido **por medición de tamaño** (T-02b), no por preferencia: `windows`/WinHTTP frente a `ureq`/`reqwest`.

## 5. Tamaño de la DLL (ADR-013)

| Medida | Objetivo | Límite CI |
|---|---|---|
| DLL release | ≤ 2,2 MB | 2,5 MB |

Medido el **2026-11-01**: **2 083 840 B** (1,99 MiB / 2,08 MB) con todas las vistas, las tablas de
textos y el descompresor del índice. `scripts/check_size.sh` falla por encima de 2,5 MB y avisa por
encima del objetivo; la CI lo ejecuta en el job de Windows.

Perfil release (real, en `Cargo.toml`):

```toml
[profile.release]
opt-level = "z"
lto = "fat"
codegen-units = 1
strip = "symbols"
panic = "unwind"       # necesario para catch_unwind en la frontera FFI
debug = false
```

- `panic = "abort"` reduciría tamaño, pero convertiría cualquier pánico en un cierre del juego. Se descarta por la constitución (principio 7).
- Evitar `regex`, `reqwest`/`tokio` y cualquier TLS empaquetado.
- Revisar con `cargo bloat --release --crates` en cada release; el tamaño se registra en el CHANGELOG.

## 6. Estrategia de pruebas

Lo que hay de verdad hoy: **250 tests** de `cargo test` (unitarios, incluidos los de `addon` que no
tocan Nexus), fixtures JSON reales y tres pruebas de red `#[ignore]`.

| Tipo | Herramienta | Alcance |
|---|---|---|
| Unitarias | `cargo test` | `core` (diff, filtros, rutas, proyección, i18n, equipos), `api` (parseo), `addon` (funciones puras: `team::resolve`, `window::*`, `names::*`, `md5`, `guild_index::unpack`, `session_names`, `keybinds`) |
| Fixtures | JSON reales en `crates/api/tests/fixtures/` | Respuestas de match, objetivos, guild, 503 y JSON truncado |
| Red | `#[ignore]` | `live_get_returns_the_match_json`, `live_pipeline_builds_the_dashboard`, `live_the_index_zip_is_downloadable`; se lanzan a mano antes de publicar |
| Tamaño y forma | `scripts/check_size.sh` | La DLL no pasa de 2,5 MB y es un PE x64 válido que exporta el punto de entrada |
| Aceptación manual | `specs/manual-tests.md` | Todo lo que depende del juego |

**Reglas que ya nos han costado un fallo** (AGENTS §11): el valor esperado sale de los datos;
un test del `addon` no puede llamar a `AddonApi::get()`; nada de tests que reimplementen la lógica;
antes de asumir una firma externa, se abre su fuente.

**Trazabilidad:** cada test de comportamiento lleva el ID del requisito en el nombre (`rf19e_ruins_never_alert`). `scripts/check_traceability.sh` **falla** si un test cita un requisito que no existe en `01-requirements.md`, e informa de los que no tienen cobertura.

## 7. Documentación

- `rustdoc` en la API pública de `core` y `api`; los módulos del `addon` llevan comentario de
  cabecera explicando el porqué (no `missing_docs` a nivel de crate).
- `README.md` (qué hace, instalación, límites conocidos), `CHANGELOG.md` (Keep a Changelog + SemVer),
  `docs/` para el usuario y para publicar (`RELEASE.md`).
- `PROGRESO.md` es la bitácora (lo más reciente arriba) y `bugs.md` la bandeja de fallos del jugador,
  que **no se borra nunca**.
- **Spec-driven:** ningún cambio de comportamiento sin actualizar antes `01-requirements.md` y, si
  procede, un ADR. El PR enlaza los RF/T afectados.
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
