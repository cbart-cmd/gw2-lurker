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

## EN CURSO – 2026-10-31 · Siguiente: T-31 (i18n) y las vistas del MVP

- Hecho sin necesitar al usuario: publicación automatizada (T-34), caché en disco (T-11b),
  gate de visibilidad, modelos de las tres vistas y el check de trazabilidad (ver abajo).
- Siguiente sin el juego: T-11c (hilo worker con parada limpia y estado de salud) y favoritos
  (RF-73). Cuidado con RF-32a: identificar garrison/colinas/bahía requiere criterio que aún no
  está verificado.
- **Requiere GW2**: T-12 (leer `NexusLink`/`Context`), T-35, T-01, T-02b, T-25.
- Decidido: **sí** a la feature `rtapi` (RF-44); su coste en DLL se mide en T-02b.
- CI: el job `deny` seguía en curso al cerrar esta entrada; `license-files = []` es la
  primera prueba para las licencias de `nexus`. Si vuelve a fallar, hay que calcular el hash
  real de los ficheros de licencia, no seguir probando variantes.

## HECHO – 2026-10-31 · Publicación automatizada de la DLL (T-34)

- `scripts/release.ps1`: compila, comprueba `MZ` y tamaño (`check_size.sh`), escribe el `.md5`
  con `UTF8Encoding($false)` (32 bytes exactos, sin BOM ni salto), añade la excepción de LFS a
  `.gitattributes` del repo compartido, **aborta** si `git cat-file -s :gw2_lurker.dll` no
  coincide con el tamaño local (puntero LFS) o si `check-attr` no devuelve los cuatro atributos
  en `unset`, hace commit **solo** de `gw2_lurker.*`, publica la release con `--latest=false`,
  borra las releases nuestras más allá de la anterior y descarga la URL exacta de `update_link`
  para comparar bytes y MD5.
- Opciones: `-DryRun` (compila y valida, no toca lo remoto) y `-VerifyOnly` (solo verifica lo
  ya publicado, útil tras un aviso de caché).
- Corregido un fallo propio que esto destapó: la DLL se llamaba `gw2lurker_addon.dll` porque
  el `[lib]` no tenía `name`. Ahora `crates/addon/Cargo.toml` fija `name = "gw2_lurker"` y el
  `ci.yml` apunta al nombre nuevo.
- Verificado: `-DryRun` completo (DLL 1 053 696 bytes, `MZ`, MD5 calculado, tamaño dentro del
  objetivo). La publicación real y la descarga se comprueban en el mismo comando.
- Trampa para quien siga: en PowerShell, `"$size:"` dentro de una cadena se parsea como
  unidad/scope y rompe el fichero; hay que escribir `${size}`.

## HECHO – 2026-10-31 · Caché en disco de objetivos y guilds (T-11b)

- `crates/api/src/cache.rs`: `Cache` sobre el directorio del addon. Objetivos **por idioma**
  (`objectives-<lang>.json`) para que cambiar de idioma o mantener Ctrl derecho pulsado
  (RF-45) no toque la red, y `guild-<id>.json` para no repetir `/v2/guild/:id` cada sesión.
- Una entrada corrupta o ausente es un *cache miss*, nunca un error que aborte la llamada.
- `GuildInfo` ahora también implementa `Serialize` (antes solo `Deserialize`).
- Verificado: `cargo clippy --workspace --all-targets --exclude gw2lurker-addon -D warnings`
  limpio; 13 tests en `api` (4 nuevos), 50 en `core`, 6 en `store`.
- Ojo para quien siga: `write` necesita `T: Serialize + ?Sized` porque se le pasa `&[T]`.

## HECHO – 2026-10-31 · Capa de API con transporte intercambiable (T-11b parcial)

- `crates/api`: `trait Transport` (solo baja bytes) y `Api<T>` con `matches_all`,
  `match_for_team(team_id)`, `objectives(lang)` y `guild(id)`. El cliente HTTP real queda
  fuera a propósito: lo elige la medición de T-02b.
- `match_for_team` **escanea `ids=all`** en vez de usar `?world=<id>`, porque ese parámetro
  espera un id de mundo antiguo y nosotros identificamos equipos por 11xxx/12xxx.
- `ApiError` (transporte, estado, parseo, no existe) y `Backoff` exponencial con tope y
  criterio de reintento (5xx y 429 sí, 4xx no).
- 9 tests con fixtures reales: equipo 12014 → match `2-2` azul, 503, JSON truncado, idioma
  en la URL, guild con tag.
- 65 tests en total (50 core + 9 api + 6 store).

## HECHO – 2026-10-31 · Addon real: export, contexto y configuración (T-12 parcial)

- `nexus` 0.12.0 (tag fijo) con `log`, `log_filter`, `mumble` y `rtapi`; añadido `log` y
  `clock` de chrono. `deny.toml` gana `allow-git` para los tres repos de `zerthox`.
- `export!` con **`UpdateProvider::Direct`**, `DisableHotloading` y la URL de
  `gw2addonsreleases`. Comprobado dentro del binario: contiene `GetAddonDef`, "WvW Lurker",
  `gw2_lurker.dll` y `raw.githubusercontent.com`.
- `ffi/mod.rs` (único `unsafe`, con `SAFETY`): `is_gameplay()` por `read_volatile` y
  `mumble_map()`. `context.rs` los convierte en el `GameState` de `core::gate`.
- `config.rs`: JSON en `addons/gw2_lurker/config.json`, defaults, saneo de rangos (incluido
  NaN), renombrado a `config.json.corrupt-<fecha>` si está dañado, `save()` listo para T-17.
- Ventana de estado mínima, gateada por `core::gate`: solo se dibuja en WvW jugable.
- **Tamaño medido: 1.053.696 bytes (1,00 MB)**, `MZ`, bajo el objetivo de 1,5 MB de ADR-008
  y aún sin cliente HTTP. Apuntado en T-02b.
- 61 tests (50 core + 6 store + 5 addon). Clippy en verde.
- Trampa: `Utc::now()` exige la feature `clock` de chrono; y `log` es dependencia directa,
  no basta con la feature de nexus.

## HECHO – 2026-10-31 · Claims tardíos con bandera `late` (T-09)

- `Event::Claim` gana `late: bool`. Un baseline emite `Baseline` y después **un claim por
  cada objetivo reclamado**, con su `claimed_at` real y `late = true` (RF-25). Los claims
  observados en vivo llevan `late = false`.
- Consecuencia para quien consuma: el baseline de una sesión son ~41 eventos, no uno. El
  repositorio de eventos ya los guarda tal cual (T-10).
- Tests: los de baseline ahora separan eventos de ciclo de vida de los claims tardíos, y hay
  uno específico que comprueba guild y hora reales de un claim tardío.
- 50 tests de core + 6 de store.

## HECHO – 2026-10-31 · Almacenamiento por sesión (T-10 parcial)

- `crates/store`: `SessionStore` con `Record { v, at, match_id, event }` (los eventos son
  los de `core::diff`, serializados tal cual). `open()` **borra los `.jsonl` anteriores**
  (ADR-012), `append()` rota a los 2 MB (RNF-03) y `read_all()` salta líneas corruptas o de
  versión futura y las cuenta.
- Dos bugs propios cazados por los tests: la rotación reutilizaba el nombre de fichero y
  truncaba registros (ahora `sesion-000.jsonl`, `sesion-001.jsonl`…), y un test comparaba
  una fecha literal con otra calculada. Ambos corregidos.
- 6 tests de store + 49 de core. Falta la consulta por guild y rango (T-11).

## HECHO – 2026-10-31 · Favoritos y limpieza de módulos (T-39 parcial)

- `core::follow`: `FollowList` con `add`/`remove`/`toggle` idempotentes, sin duplicados y
  serializable como array JSON; `favourites_first` sube las seguidas sin desordenar el resto.
- `diff.rs` llegó a 501 líneas (aviso de RNF-10) al crecer los tests: partido en
  `diff/mod.rs` (196) + `diff/tests.rs` (244). El fichero más grande ahora es `model.rs` (375).
- 49 tests en verde; trazabilidad: 21 requisitos con test, 52 sin cubrir.
- Trampa: `cargo fmt | Select-Object -First 5` corta el proceso y deja el formateo a medias.
- Trampa: una dependencia entre crates del workspace **necesita `version`** además de
  `path`; sin ella `cargo deny` la ve como wildcard y falla el job de supply-chain.

## HECHO – 2026-10-31 · Vistas de claims y trazabilidad (T-36/37/38 parcial, T-03b)

- `core::matchup`: `claims()` (41 filas reales), `by_map()` en orden fijo, `with_kinds()`
  para los filtros, `rank_guilds()` por claim más reciente con desempate estable,
  `split_by_map()` y `guild_timeline()`. 7 tests.
- `scripts/check_traceability.sh`: 83 requisitos, 20 con test, 18 con prueba manual, 53 sin
  cubrir. Falla si un test cita un RF inexistente (probado con `rf999`), informa de los no
  cubiertos y `--strict` falla por cobertura. Añadido a la CI.
- Trampa: los comodines de la matriz (`RF-03*`) se colaban como requisito; ahora se filtran.
- 44 tests en verde.

## HECHO – 2026-10-31 · Bocetos HTML (T-32)

- `docs/boceto.html`: una página, sin red, con cuatro pestañas (alertas, mapa, guilds,
  historial) y una columna "en el juego" con el widget sobre un fondo simulado.
- **Datos reales**: 76 objetivos con coordenadas de los cuatro mapas y los 41 claims del
  match `2-2`, capturados de las fixtures. Los 39 tags de guild salen de `/v2/guild/:id`.
- Decisiones visibles que hay que validar: tamaño y contorno doble por `points_tick`,
  anillo discontinuo para "sin reclamar", lista de guilds por claim más reciente,
  separación "este mapa / otros mapas" y ruta dibujada en el último mapa con su nombre.
- `PRODUCT.md` escrito para que la skill `impeccable` tenga contexto (suposiciones marcadas).
- Detector `impeccable detect`: corregidos texto en mayúsculas, sombra difusa con borde fino
  y contraste del texto atenuado. Quedan avisos de tipo *slop* (radial-halo, grid) y tres
  de contraste que son falsos positivos: el detector no resuelve el fondo oscuro que se
  aplica con `background` + degradados; el texto real sobre `#0f151c` pasa AA de sobra.
- No verificado: que el usuario lo encuentre útil. Es justo lo que toca preguntar.

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
