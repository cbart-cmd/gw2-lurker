# Constitución del proyecto: WvW Watch (nombre provisional)

Addon de Nexus para Guild Wars 2, escrito en Rust sobre `zerthox/nexus-rs`, activo solo en Mundo contra Mundo (WvW/MCM).

Estos principios son **no negociables**. Cualquier requisito, diseño o tarea que los contradiga debe rechazarse o cambiar la constitución mediante un ADR.

## Principios

1. **Solo fuentes legítimas.** Datos de la API pública de GW2 (sin API key) y de lo que Nexus/MumbleLink expone. Prohibido leer memoria del juego, automatizar input o usar fuentes no oficiales.
2. **Sin API key.** El addon no pide ni almacena claves. El mundo del jugador se elige manualmente (ADR-004).
3. **El render nunca espera.** Red y disco viven en hilos propios. El hilo de render solo lee estado ya calculado.
4. **Coste cero fuera de WvW.** Fuera de un mapa de WvW no hay polling ni dibujado.
5. **Honestidad de los datos.** Lo observado se distingue de lo inferido. Las rutas son simuladas y se etiquetan siempre como tal (ADR-005). No se promete detectar "ataques en curso" (ADR-001).
6. **Cortesía con la API.** Respeta caché y rate limits, con backoff exponencial ante errores.
7. **Nunca tumbar el juego.** Sin `panic` que cruce la frontera FFI; los errores se registran y se muestran como estado, no como cierre.
8. **Lógica pura y testeable.** Todo lo que no sea Nexus/imgui vive en un crate sin dependencias del juego, con tests automáticos.
9. **Datos locales acotados.** Nada crece sin freno (ADR-015, ADR-016): el historial vive **solo en memoria** (2 000 filas), se reinicia al confirmar una reasignación de equipos y se pierde al cerrar; `settings.json` guarda los ajustes y **solo los nombres de los clanes que sigues**; `temp.json` guarda los nombres de la sesión y **se borra en cada arranque** (tope de 500); el índice `guild-names.json` no se tiene en memoria y las teselas del mapa van a una caché que se puede borrar. **No hay historial en disco.**
10. **Degradación elegante.** Sin red o con la API atascada, se muestra el último estado conocido marcado como "obsoleto".
11. **Código mantenible por norma.** Ficheros de código ≤ 800 líneas, `rustfmt` y `clippy` sin avisos, sin `unwrap` en producción. Detalle en `04-engineering-standards.md`.
12. **DLL pequeña.** Presupuesto de tamaño con comprobación en CI (ADR-008).
13. **La carga la gobierna Nexus.** El addon no valida ni sustituye su propia DLL; falla de forma segura y deja que Nexus gestione actualizaciones y carga (ADR-007).

## Definición de "hecho" para cualquier tarea

- Tests del crate `core` pasan (`cargo test`).
- `cargo clippy -- -D warnings` y `cargo fmt --check` limpios.
- Cada requisito tocado tiene al menos un criterio de aceptación verificable.
- No se añade trabajo al hilo de render sin medirlo (RNF-01).
- CI completa en verde (ver `04-engineering-standards.md` §9).
