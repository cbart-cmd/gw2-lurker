# Progreso: gw2-lurker

Bitácora para continuar el trabajo, incluso si quien lo retoma no es la misma IA ni la
misma sesión. **Lo más reciente va arriba.** Cada entrada dice qué se pretendía, qué se
hizo, cómo se verificó y qué quedó pendiente.

## HECHO – 2026-11-10 · Corregir la carga de texturas de teselas

- El `Nexus.log` de 0.26.15 muestra que `get_texture_or_create_from_file` devuelve `None` con las
  JPG locales. `ensure_tiles` ahora precarga caché y descargas en un hilo de fondo; el render pasa
  instantáneas inmutables de bytes a `get_texture_or_create_from_memory`, sin E/S de disco.
- Test `rf37_renderer_keeps_a_safe_snapshot_while_worker_replaces_tile_bytes`: pasa. La validación
  completa detectó solo formato, corregido y repetido; fmt, clippy, LOC, trazabilidad, versión y
  tests en verde (392 pasados, 4 ignorados). Build release: DLL de 1 641 472 bytes. `check_bugs.ps1`
  y `git diff --check` limpios.
- No probado dentro del juego: falta confirmar que Nexus crea las texturas JPG desde memoria.
  Se mantiene el CDN oficial; sus teselas aún muestran el fondo compartido/alpino, no hay una
  fuente oficial identificada para imágenes beta distintas. El jugador confirma que gw2mists
  también muestra el mapa alpino y no autoriza cambiar a esa fuente.

## HECHO – 2026-11-10 · Detectar las tres fronteras Seared y renovar sus fondos

- Diagnóstico confirmado: `Nexus.log` registra `map 1651 tipo 11`; `/v2/maps/1651` es Seared
  Borderlands, `GreenHome`, con `continent_rect` [[5630,11518],[8190,15102]] y 21 objetivos
  oficiales (18 con coordenadas). Al faltar 1651 en el catálogo, la UI caía al mapa verde 95.
- Añadidos 1651, geometría y objetivos a la consulta/fixtures; rol de Mumble 10/11/12 y nombre
  de `/v2/maps` deciden rol y etiqueta, no una asociación permanente del ID beta. Así puede volver
  a Alpine/Desert o renombrarse sin conservar una etiqueta Seared fija.
- Las teselas oficiales son globales por coordenada, no por ID: se renuevan en segundo plano para
  mapas que la API llame Seared, las demás caducan a las 24 h y una generación de caché recarga
  las texturas sin comprobar metadatos de disco cada fotograma.
- `scripts/validate.ps1`: fmt, clippy, LOC, trazabilidad, versión y tests en verde (391 pasados,
  4 ignorados). `-Release`: DLL de 1 641 984 bytes, dentro del límite. `modelo.md` regenerado;
  sin descripciones pendientes. `check_bugs.ps1` y `git diff --check` limpios.
- Versión 0.26.15 subida a `main` (`df8ec4b`). Release `gw2-lurker-v0.26.15` publicada;
  commit de distribución `fa8731d`. Descarga real: 1 641 984 bytes, `MZ`, MD5
  `3cc4f761e5098b59191184bdbc8d1f18` coincidente.
- La consulta de CI por el commit no devolvió ejecuciones. No probado dentro del juego.

## HECHO – 2026-11-10 · Incluir Seared Borderlands 1654 en el selector

- El `Nexus.log` del jugador informa `map 1654 tipo 10`; la API oficial tiene geometría propia
  para 1654 (`continent_rect` [[12798,10878],[15358,14462]]) y 21 objetivos, 18 con coordenadas;
  `/v2/maps` del addon solo pedía IDs hasta 1652.
- Añadido 1654 al catálogo consultado y a las fixtures; el selector lo etiqueta Seared Borderlands
  con el color de Mumble. La prueba cubre la opción azul y el rol verde beta, geometría y objetivos.
- `scripts/validate.ps1`: fmt, clippy, LOC, trazabilidad, versión y tests en verde (388 pasados,
  4 ignorados). Build release: 1 638 400 bytes, MD5 `7caa425e43b2b3f968ed516f12dc68df`.
- Versión 0.26.14 subida en `main` (`5b132e2`). Release `gw2-lurker-v0.26.14`
  publicada; commit de distribución `cdaaf08`. Descarga real verificada: 1 638 400 bytes,
  cabecera `MZ`, MD5 coincidente `7caa425e43b2b3f968ed516f12dc68df`.
- La consulta de CI por el commit no devolvió ejecuciones.
- No probado dentro del juego.

## HECHO – 2026-11-10 · Restaurar UI en Armistice Bastion (0.26.13)

- Corrección del diagnóstico por el jugador: el cierre era por ArcDPS, no por el addon ni por
  `map_type` 18. El `arcdps.log` nuevo muestra `received incompatible imgui context, using
  standalone` y carga `ImguiLegacy`; la coincidencia temporal con la activación del widget fue
  insuficiente para atribuir la causa.
- Decisión: retirar la excepción de 0.26.12, restaurar el comportamiento de 0.26.11 y publicar
  0.26.13. La investigación de ArcDPS queda fuera de este hotfix.
- Test `rf01_the_lounge_shows_and_polls_like_other_wvw_maps`: falló antes de revertir la excepción
  y pasó después. `scripts/validate.ps1`: todo en verde (387 pasados, 4 ignorados). Build release:
  1 638 400 bytes, MD5 `5a39a0d15db1bfca4adf0b3e9fce4cec`.
- Código `9312fb7`; release `gw2-lurker-v0.26.13` publicada, distribución `584625b`. Descarga
  real verificada: 1 638 400 bytes, cabecera `MZ`, MD5 coincidente
  `5a39a0d15db1bfca4adf0b3e9fce4cec`.
- `arcdps.log` del 20:54 muestra `received incompatible imgui context, using standalone`, carga
  de `ImguiLegacy` (style delta +32) y varias extensiones. Este turno revierte solo la mitigación
  errónea de tipo 18; no se ha aislado ni corregido ArcDPS.
- No probado dentro del juego; la consulta de CI no devolvió ejecuciones para el commit.

## HECHO – 2026-11-10 · Mitigación del cierre en Armistice Bastion (0.26.12)

- Diagnóstico inicial incorrecto: se atribuyó relevancia causal a la coincidencia entre la UI
  en tipo 18 y el cierre. El jugador aclaró que el culpable era ArcDPS; se revierte en 0.26.13.
- Test `rf01_the_lounge_hides_overlay_and_polling_but_keeps_settings_available`: falla antes del
  cambio, pasa después; `rf01_playable_wvw_maps` confirma que 13/14 siguen activos. Excepción
  implementada en `core::gate`, documentada en RF-01.
- `scripts/validate.ps1`: todo en verde (387 pasados, 4 ignorados). Build release 0.26.12: 1 638 912
  bytes, dentro del límite, MD5 local `4981cfce08d21ffc1d929db34ce6e5da`.
- Código `5da6573`; release `gw2-lurker-v0.26.12` publicada. Distribución `3205bc2`; descarga
  real: 1 638 912 bytes, `MZ`, MD5 coincidente `4981cfce08d21ffc1d929db34ce6e5da`.
- Diagnóstico no concluyente: Windows reportó `0xc0000409` y eventos GPU 117/141; ArcDPS no tiene
  una traza actual en el `lastcrash` (archivo del 30 de octubre); el log reciente sí señala
  contexto ImGui incompatible y carga de `ImguiLegacy`.
- No probado dentro del juego; la CI de GitHub no devolvió ejecuciones para el commit consultado.

## HECHO – 2026-11-10 · Detección ampliada y Seared Borderlands (1652)

- Plan acordado con el jugador: admitir también `map_type` 13, 14 y 18 para UI y polling,
  siguiendo la clasificación de `gw2-timers`; dibujar los objetivos estáticos de Seared
  Borderlands (1652) sin inventar estado mientras los matches activos no lo publiquen.
- Aclaración del jugador: en la beta el mismo mapa 1652 sirve de frontera azul, verde y roja; el
  rol actual se deriva de `map_type` 10/11/12, nunca de `map_id` ni del tipo `RedHome` del catálogo.
- Hechos verificados contra la API oficial: `/v2/maps/1652` da la geometría y lo cataloga como
  `RedHome`; `/v2/wvw/objectives?ids=all` incluye 21 objetivos, 18 con coordenadas; las teselas oficiales
  responden. Ninguno de los 9 matches activos consultados contiene el mapa.
- RF-01/RF-30 y P-05 implementados: puerta igual a `gw2-timers`, rol por `map_type`,
  geometría/objetivos estáticos por ID, teselas únicas y aviso explícito cuando no hay estado live.
- `scripts/validate.ps1`: fmt, clippy, LOC, trazabilidad, versión y tests en verde (387 pasados,
  4 ignorados). Versión 0.26.11; build release de 1 638 912 bytes con `MZ` y tamaño dentro del límite.
- Código subido a `main` (`8b25f87`, integrado con los cambios remotos de README en `ab61dbb`).
  Release `gw2-lurker-v0.26.11` publicada; commit de distribución `adbbc5b`. Descarga real
  verificada: 1 638 912 bytes, MD5 `48d2e874c1fa809e024ceb5ea6d9dafd`, cabecera `MZ`.
- Pendiente de verificar dentro del juego. La API de los 9 matches consultados no expuso estado
  live para 1652; no se infieren propietarios ni reclamos. Tampoco se verificó si el API futuro
  distinguirá las tres variantes de mapa en los payloads de match.

## HECHO – 2026-11-02 · Buscador de clanes: índice protegido, tamaño visible y etiqueta primero (0.26.10)

- Informe: «AIS» solo mostraba *[AiS] Asian In Spirit* en una sesión anterior; el índice local
  tenía entonces muy pocos nombres (con el completo hay 36 coincidencias).
- Plan (RF-20b/RF-20d, T-44): `names::append_at` no escribe si el índice existe pero no se puede
  leer o está mal formado (antes lo tomaba por vacío y lo sobrescribía con un solo clan; la
  carpeta está en OneDrive). El buscador dice cuántos clanes tiene el índice. Orden: etiqueta
  exacta, etiqueta que empieza por el texto, nombre que empieza, resto; luego longitud.
- `[AIS] All In Squadron` no figura en `/v2/wvw/guilds/eu`: añadido a mano al índice publicado
  (distribución `cd32bed`, código `1d39819`).
- Hecho en `4c364f0`: `names::read_strict` (fichero ausente = vacío; ilegible o mal formado =
  error y `append_at` no escribe), `names::rank` y el recuento bajo el buscador (reutiliza la
  clave i18n de los ajustes; aparece tras la primera búsqueda). Tests
  `rf20d_an_unreadable_index_is_never_overwritten` y
  `t44_an_exact_tag_comes_before_names_that_contain_the_text`. `validate.ps1 -Release` en verde:
  385 tests, DLL 1 634 816 bytes. Publicado con `release.ps1` (MD5 `4c86d048…`), 0.26.8 retirada.
- No verificado en el juego: el orden con «ais» y el recuento visible.

## HECHO – 2026-11-02 · Publicación 0.26.9 (T-54 y T-56)

- Código `6a8eb9e`; distribución `6f3066f`, tag `gw2-lurker-v0.26.9`.
- `scripts/release.ps1`: descarga real verificada, 1 634 816 bytes, MD5
  `e0e7659073901558c0d3da75d322e2ac`.
- Hallazgo previo a publicar: el emergente guardaba la posición después de cerrar su ventana
  imgui, cuando ya era actual otra ventana. Corregido y reproducido con imgui nativo sin Nexus:
  `rf19d_placement_remembers_the_popup_position_not_the_parent`.
- Hallazgo previo a publicar: el filtro antiguo del emergente también descartaba los reclamos
  tardíos recientes de clanes seguidos, excepción explícita de RF-19a/RF-25. Corregido: el test
  `rf25_r07_only_recent_followed_late_claims_reach_popup` pasa los eventos por el filtrado real
  y comprueba la excepción, los descartes y la caducidad; antes fallaba con cero líneas.
- Medición aislada optimizada de pila y dibujo imgui, con 200 objetivos, 50 avisos y la lista de
  clanes llena: p95 0,058–0,097 ms en régimen y 0,198–0,473 ms en ráfaga (cuatro pasadas).
  No mide widget y mapa completos ni sustituye la comprobación de RNF-01 dentro del juego.
- Validación tras la corrección: fmt, clippy, LOC, trazabilidad, tests y release en verde;
  383 tests pasados, cuatro de red ignorados y pasados aparte.
- **Sin verificar en GW2:** click-through, colocación del emergente y coste del frame completo.
- Consulta por reclamo ausente (21:11–21:30): la API de la partida `2-2` mantenía el objetivo
  `1099-105` con `claimed_at` 20:05:16 local; no reflejaba un reclamo reciente. El addon seguía
  sondeando cada 5 s, reconocía el mapa 1099 y tenía activados torres y reclamos; el sondeo
  registrado a las 21:11 informó cero eventos nuevos. No se conoce la causa del dato antiguo.

## HECHO – 2026-11-02 · T-54 → T-56 – Formato y emergente

- Clasificación pura desde el equipo propio y formato RF-19f conectado al widget, registro y
  emergente; antigüedad viva, alto valor, mapas completos y hora exacta en registro/tooltip.
- Pila T-56 con caducidad individual, prioridad, máximo cinco líneas, contador de descartadas y
  `NO_INPUTS` salvo al colocarla. Tests RF-19f/RF-19d, incluido reemplazar el «tomado» más antiguo.
- `scripts/validate.ps1`: fmt, clippy, loc, trace, version y tests en verde (381 pasados, 4
  ignorados); `check_bugs.ps1`: sin cambios. No hay commit.
- **No verificado en el juego:** interacción `NO_INPUTS`/colocación; no se publicó ni se probó la DLL.

Reglas:
- Al **empezar** una tarea: añade una entrada `EN CURSO` con el plan en 3–5 líneas.
- **Durante**: actualiza esa entrada con hallazgos, decisiones y callejones sin salida.
- Al **terminar**: cámbiala a `HECHO`, pon los commits y lo que no se pudo verificar.
- No borres entradas antiguas. Si algo se revierte, dilo aquí y por qué.
- Lo que contradiga la spec se propone y se decide, no se entierra aquí.

---

## Trampas ya conocidas (no volver a tropezar)

- `owner` viaja como `"Red"`/`"Green"`/`"Blue"` (PascalCase); los contenedores usan claves
  minúsculas. Mezclarlo rompe la deserialización.
- `claimed_by` es **una** guild o `null`, nunca una lista. Las claves siempre vienen.
- Torre totalmente mejorada llega a `points_tick` 8: "alto valor" no puede ser por tipo.
- Ids de equipo 11001–11015 (NA) y 12001–12018 (EU). Prefijo del match: `1-x` NA, `2-x` EU.
- Nexus expone `NexusLink::is_gameplay`: sirve para no dibujar en carga/selección.
- `crate::config()` es la copia **del arranque** y no cambia nunca; lo que el jugador toca en la
  ventana está en `crate::runtime()`. Leer `config()` desde el render hizo que el clic derecho no
  viera el click-through, que las listas de clanes no se refrescaran y que el clic derecho no
  deshiciera nada (0.22.10).
- **La DLL instalada no se queda**: Nexus (`UpdateProvider::Direct`) la sobrescribe en cada arranque
  con la que haya publicada en `gw2addonsreleases`. Probar una versión local exige publicarla
  (`scripts/release.ps1`) o desactivar la comprobación de actualización en Nexus (0.23.1).
- **`raw.githubusercontent.com` cachea ~3 min**: el `.md5` se sirve antes que la DLL, así que una
  release recién subida parece buena si solo se mira el HTTP 200. Comprobar **bytes y MD5** de la
  descarga real (`scripts/release.ps1 -VerifyOnly`) antes de decirle al jugador que abra el juego.
- MumbleLink `name` = «Guild Wars 2», **no** el personaje: el personaje está en `identity`
  (`{"name":"Jhoin",…}`). Y `shard_id` en WvW vale `0x80000001` (medido 2026-10-31): no es el mundo
  ni la partida, así que no sirve para detectar el emparejamiento.
- `map_id` de Mumble puede cambiar durante una beta: usa `map_type` para el rol y el nombre/geometría
  vigentes de `/v2/maps` para identificar la variante. Las teselas son globales por coordenada; la
  caché no debe quedar fijada para siempre a la imagen obtenida en la primera instalación.
- Un reclamo del API sigue ahí aunque el objetivo cambie de color: hay que comparar `last_flipped`
  con `claimed_at` o el mapa pinta un anillo en un campa de otro color (0.22.10).
- Publicar: `UpdateProvider::Direct` + `DisableHotloading`, MD5 de 32 hex sin BOM, excepción
  de LFS para la DLL, tag propio con `--latest=false`. Ver `docs/RELEASE.md`.
- `InputBinds.json` cambia con el juego abierto: no cachear el atajo para siempre. Lo relee el
  worker según la fecha de modificación y la tecla se nombra en el hilo que pide el texto
  (`GetKeyNameTextW` usa la distribución de teclado de ese hilo).
- Los bocetos de `docs/bocetos/ruta/` cargan los tiles por `file:///` con una ruta absoluta del PC
  del jugador (`LK.TILES`, `escena.js`). El navegador integrado de VS Code no sirve para
  verificarlos (403 entre carpetas, no redimensiona el viewport): se probaron con Edge del sistema
  y `playwright-core` (hay que reinstalarlo; no queda nada en el repo).

## HECHO – 2026-11-02 · Propuestas de UX (solo bocetos, nada implementado)

- Crítica con `impeccable` (revisión de diseño + detector) de `crates/addon/src/ui`: heurísticas
  22/40. Bocetos en `docs/bocetos/ux/propuestas.html`: aviso desde tu equipo
  (PERDIDO/TOMADO + antigüedad), HUD de combate separado (choca con ADR-014), emergente agrupado
  de 5 s, tira de cuatro mapas con leyenda, bloque único «Qué me avisa» y selector de equipo que
  solo guarda al confirmar.
- **Decisión del jugador (2026-11-02):** pasan a la spec las propuestas 1, 2, 4, 5 y 6, y la 3
  cambiada (emergente apilado, cada línea dura `popup_secs`, 15 s por defecto; prioridad clan
  seguido > alto valor > perdido > tomado). Escrito en RF-19d (reescrito), RF-19f, RF-03j, RF-30c,
  RF-40a, RF-47, ADR-017 (enmienda ADR-014) y tareas T-54..T-59 (Fase 8). Nada implementado.

## HECHO – 2026-11-02 · DLL de 2,28 MB a 1,62 MB sin tocar funcionalidad

- `cargo bloat --crates`: `regex` + `regex_automata` + `aho_corasick` + `regex_syntax` ≈ 390 KiB.
  Entraban **solo** por la feature `log_filter` de `nexus` (→ `env_filter`), para parsear
  `log_filter: "info"`, que es un nivel sin directivas por módulo.
- Cambio: se quita `log_filter` (feature y campo de `export!`) y se pide `log` con
  `max_level_info`. Es el mismo filtro, pero se aplica al compilar: los `debug!`/`trace!` no
  llegan a la DLL. Fuente: `nexus/src/logger.rs` (sin la feature, `NexusLogger` deja pasar todo
  y `set_max_level(Trace)`; lo corta el límite estático de `log`).
- 2 284 032 → **1 616 896 bytes** (−29 %). `validate.ps1 -Release` en verde (373 tests).
- Lo que queda (`std` con `panic="unwind"` por ADR-008, imgui, serde_json, nuestro código) es
  funcional; no hay otro recorte grande sin tocar un ADR.
- **No verificado en el juego:** que el log de Nexus siga mostrando info/warn/error.
- Publicado como **0.26.8** (1 616 384 bytes, MD5 `8fa8cf9c…`, descarga verificada).

## HECHO – 2026-11-02 · Aviso de API parada a 5 min y selector de equipo en ventana propia

- Aviso de API congelada: el umbral **ya era 5 min** (ahora constante `FROZEN_AFTER_MIN` en
  `ui/status.rs`). `matchup::digest` se deja como estaba, **con puntos**: mide si la API publica
  algo. Quitar los puntos se probó y se descartó: avisaba con el marcador aún moviéndose. Si el
  jugador lo vio a los «30 min», era el contador de minutos de una congelación larga o la copia
  sí cambiaba; no hay ningún umbral de 30 min en el código.
- Selector en ventana propia centrada `ui/team_picker.rs` (RF-03j, nuevo en la spec; excepción
  anotada en ADR-014 y RF-46). Ajustes solo tiene «Cambiar de equipo». Lógica pura en
  `core::prompt` (`matchup_key` = `match:ids ordenados`, `prompt` → NoTeam/Stale/NewMatchup);
  `confirmed_matchups` por equipo en `settings.json`. Se comprueba una vez por entrada, 5 s
  después de cargar el mapa (la cuenta de arcdps llega tarde). Sin clave guardada se apunta la
  actual sin preguntar (actualizaciones y equipos recién elegidos). Cerrar con × no confirma.
- Tests: `rf03j_*` en core y en `team_picker/tests.rs`; los de estrechamiento se movieron de
  `settings/tests.rs`. `validate.ps1` completo en verde (373 tests).
- **No verificado (requiere juego):** posición centrada, que se abra al entrar y el retraso de
  la cuenta de arcdps. Commit `b026eef`, publicado como **0.26.7** (MD5 `73fa0baa…`, descarga
  verificada por `release.ps1`).
- `gen_modelo.ps1` usa `git ls-files`: los ficheros nuevos se marcaron con `git add -N`.

## HECHO – 2026-11-02 · Deprecar T-21 y T-41

- El jugador retira T-21 y T-41 del listado activo; se conservan como deprecadas en el historial.
- RF-11/RF-15 y D-13 alineados; avisos y traducciones existentes sin cambios. MT-21 sigue pendiente.
- Trazabilidad y `git diff --check` correctos; bugs sin cambios. Solo documentación, sin versión ni commit.

## HECHO – 2026-11-02 · B — Estela publicada en 0.26.6 para probar

- Petición: retirar el pendiente de audio, aceptar las pruebas anteriores en juego por indicación
  del jugador y completar T-52/T-53. La aceptación previa no sustituye la prueba del nuevo mapa.
- T-52: dibujo preparado y cacheado, curvas/flechas, revisitas, intervalos, origen y marcas `late`/`gap`.
- T-53: cronología temporal con ratón/teclado, ventanas inclusivas y reproducción a 700 ms por claim;
  pausa al ocultar Mapa/cambiar clan. Tests puros y medición CPU del render preparado.
- Validación y documentación completadas; publicada con `release.ps1`, incluida descarga real.
- Hallazgos corregidos: 2.000 claims agotaban los índices de 16 bits de imgui; se comparten arcos
  coincidentes y la teselación se adapta a la densidad. La preparación temporal superaba 0,5 ms:
  pasa al worker fuera del candado y se publica en una instantánea coherente con historia/eventos.
- No se usa el intervalo entre claims como duración de un hueco de sondeo. Se conservan número
  actual/último en revisitas densas. Medición nativa local, no simulación del juego ni medida GPU.
- Gate en verde: fmt, clippy, loc, trazabilidad, versiones y 366 tests (4 ignorados de red).
  Preparación de geometría con 2.000 claims p95 0,056 ms; cronología/objetivos/ruta nativos
  p95 0,311 ms (20.860 vértices en el caso denso). DLL 0.26.6: 2.264.576 B, dentro del objetivo.
  Un test preexistente de búsqueda con estado global compartido falló una vez en paralelo;
  dos ejecuciones posteriores de la suite completa pasaron. No se altera su comportamiento.
- `modelo.md` regenerado sin items sin descripción. Audio retirado y pruebas previas aceptadas
  como decisión del jugador; no se atribuye una nueva ejecución en GW2. Pendiente MT-21 del mapa nuevo.
- Código subido: `c5b34de`. Distribución: `f9ddb14`, release `gw2-lurker-v0.26.6`.
  La URL exacta de Nexus devuelve 2.264.576 B, MZ y MD5 `c05a61df2476b97bdfe6bd27b9b2a67a`.

## HECHO – 2026-11-02 · RF-03a: aviso de equipo pendiente en todas las pestañas

- Petición: sin equipo elegido, cualquier pestaña de datos debe explicar que no hay datos
  hasta elegirlo en Ajustes; Ajustes debe seguir funcionando.
- Regla compartida `navigation::tab_available` antes de dibujar cada cuerpo, probada sin Nexus,
  y texto i18n en los cuatro idiomas. La cuenta actual se resuelve con `selected_in`, sin volver
  a bloquear `RUNTIME`. Sin equipo se ocultan estadísticas y avisos de navegación anteriores,
  conservando avisos generales (click-through, actualización, modo seguro).
- Dos tests nuevos: todas las pestañas con/sin equipo y cuenta distinta sin equipo. RF-03a,
  M-07/M-08, MT-03, T-42, CHANGELOG y `modelo.md` actualizados.
- Validación completa en verde: fmt, clippy, loc, trazabilidad, versión y 355 tests pasados,
  0 fallidos, 4 ignorados. `modelo.md`: 0 items sin descripción.
- No verificado: dibujo dentro del juego (MT-03). Sin publicar, subir versión ni crear commit.
  T-52/T-53 y los cambios pendientes de Estela se mantienen intactos.

## HECHO – 2026-11-02 · T-51: modelo puro de B — Estela

- `core::route::trail`: observaciones prestadas en orden temporal, cronología proporcional,
  ventanas Todo/1 h/15 min inclusivas, intervalos, revisitas agrupadas y selección sin renumerar.
- Cada par consecutivo conserva su identidad: salto de mapa, falta de posición, revisita o curva.
  No se inventan posiciones/horas ni se conectan vecinos saltando un claim sin coordenadas.
- `late` se cruza por clan/objetivo/hora con eventos originales; `gap` se conserva aparte del
  silencio entre claims ≥20 min. Gaps solapados se fusionan para consultas logarítmicas.
  `ClaimRow` no incluye esas marcas: T-52/T-53 deberán pasar los eventos retenidos al nuevo modelo.
- Reutiliza la ordenación de `route` y deja el origen en `with_origin`; ni el dibujo ni los
  controles actuales cambian. Sin dependencias nuevas, red, E/S ni reloj en la lógica.
- 11 tests nuevos: límites 900/3600 s (también precisión subsegundo), umbral 1200 s, empates,
  revisitas, clanes distintos, pasos rápidos/densos, mapas, coordenadas oficiales/ausentes/no
  finitas, horas desconocidas, cero/un paso y selección fuera de rango acotada explícitamente.
- Validación completa: fmt, clippy, loc, trazabilidad, versiones y tests en verde (353 pasados,
  0 fallidos, 4 ignorados). `modelo.md` regenerado; 0 items sin descripción.
- Pendiente: T-52 (dibujo), T-53 (controles/reproducción), seis casos visuales, medir render y
  validar en el juego. No se publica ni se sube versión. Sin commit en este turno.

## HECHO – 2026-11-02 · Spec aprobada de B — Estela; implementación pendiente

- El jugador eligió B y aprobó la propuesta de spec antes de implementar. Solo se modifican docs;
  no se cambia el addon, los bocetos ni la versión.
- RF-32/RF-33/RF-36: curvas inferidas, recencia, cronología, ventanas Todo/1 h/15 min (Todo por
  defecto), reproducción a 700 ms por claim, pausas, anillos de intervalo y huecos ≥20 min.
- Se conservan origen 0, `late`, `gap`, saltos de mapa, revisitas y claims sin coordenadas.
  Sin nuevas consultas, caminos reales, separación de escuadras ni historial en disco.
- Diseño §8 alineado; escenarios H-15..H-27 y tareas pendientes T-51..T-53 añadidos.
- Verificación documental: coherencia de IDs, límites inclusivos y distinción entre intervalo
  sin claims y `gap` de sondeo. No procede compilar ni ejecutar tests Rust: no hay cambios de código.
- Pendiente: implementar T-51..T-53, medir render y validar en el juego. Sin commit en este turno.

## HECHO – 2026-11-01 · El equipo se guarda solo por cuenta y no se migra nada (RF-03a)

- Petición: `settings.json` «parece guardar el equipo por personaje»; la cuenta ya se recupera (arcdps),
  así que debe ser la clave. Decisión del jugador: **no migrar datos; a partir de ahora, por cuenta**.
- Diagnóstico con el `Nexus.log` de 0.26.5: la cuenta llegó a las 21:57:13 y el equipo se resolvió por la
  clave heredada del personaje 1 (21:58:27). Al cambiar de personaje (22:00:49) el 2 no tenía entrada ni
  la cuenta tampoco, y el selector estuvo ~80 min hasta que el jugador eligió a mano (23:20), que sí se
  guardó por cuenta. El guardado ya era por cuenta; confundían dos cosas: `team::resolve` seguía leyendo
  las entradas por personaje y el campo del fichero se llama `by_character`.
- Hecho: (1) `Roster` se serializa como `by_account` **sin alias**: el `by_character` de ≤ 0.26.5 se ignora
  y desaparece en el siguiente guardado (cada cuenta elige su equipo una vez); (2) `team::resolve` lee
  solo la clave actual (la cuenta; el personaje únicamente si arcdps no ha dado cuenta, es decir, sin
  arcdps) y `forget` borra esa misma clave; (3) specs, docs y CHANGELOG. Tests `rf03a_…` en `select.rs`,
  `team.rs` y `config/tests.rs`.
- Coste conocido: al actualizar, cada cuenta elige su equipo **una vez**, también la que 0.26.5 ya había
  guardado a las 23:20 (el campo cambia de nombre y no hay alias). La basura de `by_character` (el
  `"Guild Wars 2"` de un bug del 2026-10-31 y personajes sueltos) desaparece al guardar. Si molesta, el
  atajo es `#[serde(alias = "by_character")]` en `Roster::by_account`: conserva las cuentas y deja inertes
  las claves de personaje.
- Sin arcdps no hay cuenta: la clave sigue siendo el personaje, como antes. Sin cuenta ni personaje
  (pantalla de selección) `remember` no guarda nada.
- Specs tocadas **a petición del jugador** («no migres datos»): RF-03a, `02-design.md` (roster por cuenta)
  y `06-diff-scenarios.md` (M-05 reescrito; M-05b y M-05c nuevos).
- Tests nuevos o sustituidos: `rf03a_roster_is_saved_by_account_and_the_old_map_is_not_read` (`select.rs`),
  `rf03a_a_character_entry_is_not_read_once_the_account_is_known`,
  `rf03a_picking_a_team_saves_it_under_the_account_only`,
  `rf03a_forgetting_clears_the_key_that_was_read_and_nothing_else` (`team.rs`) y
  `rf03a_teams_saved_per_character_by_older_versions_are_not_carried_over` (`config/tests.rs`).
- Verificado: `validate.ps1` completo en verde (fmt, clippy, loc, trazabilidad, versión 0.26.5 en 6
  sitios, 342 tests pasados, 0 fallidos, 4 ignorados); `modelo.md` regenerado, 0 sin descripción.
- **No verificado:** el comportamiento dentro del juego. Nexus sobrescribe la DLL con la publicada, así
  que hay que publicar (0.26.6; llevaría también el recorte de logs, `130dabd`, aún sin publicar).
- Commit: `c780f04`.

## HECHO – 2026-11-01 · Bocetos de mejora de la ruta inferida (solo `docs/`, sin tocar código)

- Petición: ideas para mejorar la inferencia de ruta (casos extremos, caminos en vez de rectas) y **4
  bocetos HTML** con impeccable, cada uno con 3 casos normales y 3 extremos sobre los tiles reales.
- Los tiles **no los crea el addon: los descarga** (`catalog::ensure_tiles`, `tiles.guildwars2.com`,
  zoom 5, una vez, en un hilo propio). Los bocetos los leen por `file:///` desde la carpeta del addon
  (`LK.TILES`, `escena.js` l.15: ruta absoluta de este PC, cámbiala si se abre en otro) y **no se
  commitean** (ADR-011); sin ellos, caen a un tablero oscuro.
- Qué hay en `docs/bocetos/ruta/` (todo cuelga de `window.LK`, sin dependencias ni build; doble clic en
  un `boceto-*.html`): datos (`datos-objetivos.js` con objetivos reales de `objectives_all.json` y
  `match_2-2.json`; `datos-caminos.js`, red de caminos **sintética**; `escenarios.js`, los 6 casos, los
  mismos para las cuatro), lógica pura (`logica.js`: agrupar revisitas, huecos ≥ 20 min, factibilidad
  por velocidad, orígenes candidatos, caminos, escuadras, prisma), dibujo (`escena.js`: mapa, objetivos,
  etiquetas, vista «Hoy»; `comun.js`: discos agrupados, cruces de mapa, pastillas, salida, tablas),
  `app.js` (panel, hover enlazado, hash `#n1…#e3` y `-hoy`) y `ruta.css`. Una página por propuesta:
  A caminos (`propuesta-a.js`), B estela con línea de tiempo y reproducción (`-b`), C escuadras y saltos
  de waypoint (`-c`), D incertidumbre con elipses y tablas (`-d`). Cada una alterna «Hoy / Propuesta».
- Base común: los pasos sin posición se muestran como `N?` en vez de descartarse, numeración continua,
  revisitas agrupadas (`×N`), huecos de 20 min o más sin camino, etiquetas `viene de` / `sigue en` en los
  cambios de mapa y lista de orígenes posibles del paso 0. Datos nuevos que exige cada una: red de
  caminos curada (A); waypoints utilizables y velocidad (C, D). Ninguna necesita llamadas nuevas a la
  API: todo sale del `claimed_at` que ya se guarda.
- Verificación con Edge real (1440×900 y 390×844, 4 propuestas × 6 casos): 0 errores de consola, tiles
  100 % cargados, sin desborde horizontal y 0 solapes entre etiquetas, pastillas y marcas. La primera
  ronda encontró pastillas pisando etiquetas y chips de origen; se arregló en `LK.figura` (acepta
  `cajas` y `alto`/`sobre` como función: las pastillas se colocan con las etiquetas ya puestas) y en
  `LK.pastillaLibre` (más candidatos y, sin hueco, el de menor solape). Además: controles arriba del
  panel, scroll móvil del mapa centrado y conservado al redibujar, el rótulo «sin datos» de B por
  encima de la barra de avance y, por el detector de impeccable, dos `repeating-linear-gradient`
  sustituidos por borde discontinuo y relleno plano.
- **No verificado / UNVERIFIED**: la velocidad de carrera (14 u/s), que haya waypoint en
  Spawn/Keep/Castle y los caminos de A (a ojo) son parámetros del boceto, no datos. Los tiles por
  `file://` solo se probaron en Edge (Chrome y Firefox, no). Todo se revisó con
  `prefers-reduced-motion: reduce`, así que la entrada animada del trazo no se vio en movimiento (la
  reproducción de B sí se probó).
- Honestidad (ADR-001, ADR-005): todo va rotulado «ruta inferida»; no se predice el siguiente objetivo.
- Si el jugador elige una propuesta, antes de tocar código hace falta spec/ADR (AGENTS §2.4).
- `PROGRESO.md` superaba ~30 KB (31,0 KB): las entradas de 0.25.14 a 0.26.0 se movieron tal cual a
  `docs/historial/progreso-hasta-0.26.0.md` (20,4 KB; comprobado línea a línea contra `HEAD`).
- Commits: `948f48e` (los 16 ficheros de `docs/bocetos/ruta/` y `modelo.md`) y `59ebc1e` (esta entrada y
  el archivo histórico). Sin cambios en Rust: no se ejecutó `validate.ps1`; solo el CI.

## HECHO – 2026-11-01 · Menos ruido en `Nexus.log` (sin publicar)

- Resultado con su propio log (95 líneas del addon en 5 min): 59 eran `evento: Claim/Flip/Unclaim`
  (→ `debug`, no llegan al log porque `log_filter` es `info`), 8 `deteccion:` (2 textos distintos),
  5 `vigilando` (3 distintos, por el nº de emparejamientos), 1 `arcdps: cuenta`. Quedan ≈ 25 (arranque
  y `gate:`, que ya eran cambios de estado). En régimen estable pasa de ≈ 3 líneas/min (vigilando +
  deteccion + latido) a 1 cada 5 min más lo que cambie de verdad. Estimación, no medida durante una hora.
- Cambios: `diagnostics.rs` (`changed`, `watched_key` sin el nº de emparejamientos, `heartbeat_due` por
  tiempo con `HEARTBEAT_EVERY` = 300 s), `detect.rs` (`detection_line` pura: calla si el juego y el
  equipo guardado coinciden; el resto se escribe una vez por resultado), `arc_team.rs` (sin log de la
  cuenta; sigue guardándola). `worker.rs` intacto. Tests: `t35_*` (3, en `diagnostics.rs`) y
  `rf03a_a_detection_*` / `rf03a_a_shard_*` (3, en `detect.rs`). `validate.ps1` completo en verde.
- **Verificado en el juego**: `AgentUpdate::account` llega relleno, con los **dos puntos** de arcdps
  (`:Lord Rematador.3056`). La clave del equipo por cuenta usa ese texto tal cual; no se normaliza para
  no dejar huérfano lo guardado con la 0.26.5. El plan B con `EV_ACCOUNT_NAME` no hace falta.
- Lo que no se puede ver sin publicar: Nexus sobrescribe la DLL con la publicada, así que el recorte se
  comprueba en el siguiente `Nexus.log` tras una 0.26.6.
- Petición del jugador: «ya no hace falta crear tantos logs». Con su `Nexus.log` delante (90 líneas
  del addon en 4 minutos de juego) las fuentes de ruido son: una línea por cada `Claim` (~50 `late: true`
  al arrancar) y `Flip`/`Unclaim`; `worker: vigilando` que **alterna** cada minuto porque su clave
  lleva el número de emparejamientos (9 con `ids=all`, 1 con la partida sola); `deteccion:` repetida
  cada 60 s con el mismo texto; el latido cada 10 sondeos (≈1 min con 5 s, no los «cinco minutos» que
  decía el comentario); y `arcdps: cuenta <nombre>`, que ya cumplió (**verificado en el juego**: llega
  `:Lord Rematador.3056`, con los dos puntos de arcdps) y además metía el nombre de cuenta en un fichero
  que se pega en los informes.
- Plan: quitar la línea de la cuenta; `Claim`/`Flip`/`Unclaim` a `debug` (el addon filtra a `info`);
  `vigilando` y `deteccion` solo al cambiar de verdad; latido por tiempo (5 min). Sin tocar `worker.rs`
  (777 líneas).

## HECHO – 2026-11-01 · Publicada la 0.26.5

- Release `gw2-lurker-v0.26.5`, MD5 `5db876cb1ce08641388a44cc08b28d8e` (2 202 112 B), verificado con
  la descarga real. Retención: quedan 0.26.5 y 0.26.4; borrada la 0.26.3.
- Contenido: el equipo se guarda por cuenta (`AgentUpdate::account` de arcdps).
- Pendiente de probar en GW2: ~~que `Nexus.log` diga `arcdps: cuenta <nombre>`~~ (verificado el
  2026-11-01: `:Lord Rematador.3056`; esa línea ya no se escribe desde el recorte de logs) y que
  el selector salga una sola vez por cuenta, no por personaje.

## HECHO – 2026-11-01 · El equipo se guarda por cuenta (RF-03a)

- Petición del jugador: con dos cuentas, el equipo/zona debe guardarse por **cuenta**, no por
  personaje, para que todos los personajes de una cuenta lo compartan y la otra cuenta arranque
  vacía.
- **No hace falta detectar nada nuevo**: `arc_team.rs` ya se suscribe a `EV_ARCDPS_SELF_JOIN`, y el
  `AgentUpdate` que trae el equipo lleva también `account()` (nombre de cuenta) y `character()`
  (`nexus/src/api/event/arc.rs`). El campo se tiraba; ahora se guarda en un `Mutex` y se expone en
  `arc_team::self_account()`. Cero dependencias nuevas, cero `unsafe` nuevo (el accesor es seguro).
- `team.rs`: `key(account, character)` decide la clave (cuenta primero; personaje como respaldo y
  como clave heredada de ≤ 0.26.4) y `resolve` la usa, así que una entrada vieja por personaje sigue
  resolviendo. `remember_in` y `forget_in` guardan/borran por cuenta.
- `worker.rs`: el barrido de los nueve emparejamientos se rearma cuando cambia la **cuenta**
  (`identity_changed` con `team::current_key()`), no con cada personaje.
- Spec actualizada (RF-03a) y CHANGELOG. Tests: `rf03a_the_team_is_remembered_per_account_and_shared_by_its_characters`
  y `rf03a_a_team_saved_per_character_still_resolves_after_the_upgrade`.
- ~~UNVERIFIED: que `AgentUpdate::account` llegue con `Jhoin.1234` en el juego.~~ **Verificado el
  2026-11-01**: llega `:Lord Rematador.3056` (con los dos puntos de arcdps). El log de la cuenta se
  quitó después (ver «Menos ruido en `Nexus.log`»).

## HECHO – 2026-11-01 · Publicada la 0.26.4

- Release `gw2-lurker-v0.26.4`, MD5 `b4536bef9dae5b6f0c38732493bda61d` (2 199 552 B), verificado
  con la descarga real. Retención: quedan 0.26.4 y 0.26.3; borrada la 0.26.2.
- Entra lo de la pestaña *Mapa*: abre en el mapa del jugador y sin clan elegido, la entrada
  «No guild» en el desplegable y el salto arreglado al volver a mano.
- Pendiente de probar en GW2 (es lo que el jugador va a hacer con esta versión).

## HECHO – 2026-11-01 · La pestaña Mapa abre sin clan elegido (RF-71)

- Petición del jugador: al entrar a mano en *Mapa* (sin venir de clicar un clan) debe ponerse en su
  mapa, con el desplegable de clan vacío, y verse todo lo que reclaman los clanes que sigue.
- El mapa ya se ponía en el del jugador (`snap_to_current` + `resolved_map_role`); lo que fallaba
  era el desplegable: tomaba el clan de la pestaña *Clanes* (`guilds::selected_guild()`), así que
  al volver seguía elegido y dibujaba su ruta.
- `navigation.rs`: estado propio del mapa (`guild_choice`/`set_guild_choice`) y
  `guild_from_pick`, que convierte la entrada «No guild» (cadena vacía) en «sin ruta». Test
  `rf71_entering_the_map_tab_by_hand_leaves_no_guild_chosen`.
- `map.rs`: el desplegable estrena entrada «No guild» (antes no había forma de vaciarlo) y aplica
  la elección a ese estado. `mod.rs`: el snap de entrada vacía la elección y un clic de la pestaña
  *Clanes* o de un aviso la fija, así los dos sitios siguen en sintonía.
- `validate.ps1 -Release`: todo OK, 329 tests, DLL 2 199 552 B.
- **Trampa (informe del jugador, 2026-11-01):** «vengo de un clic de clan» se decidía con
  `WANT_TAB != TAB_MAP`, y esa marca vive hasta que la pestaña se dibuja. Si imgui no cambia de
  pestaña en un fotograma, al volver a *Mapa* a mano parecía que se venía de un clan y no se
  saltaba al mapa del jugador. Ahora manda un fotograma: `FRAME`/`GUILD_JUMP_AT` y
  `navigation::entering_by_hand`, con ventana de un fotograma. Test
  `rf30_returning_to_the_map_tab_by_hand_snaps_to_the_players_map`.

## Bloqueos

- **T-01, T-02b, T-25**: requieren ejecutar dentro de GW2 (carga, click-through, tamaño de
  DLL con TLS, comportamiento de actualización). No se simulan.
- El soporte Seared 1651/1652/1654 necesita publicación antes de validar en GW2; el código local no
  sustituye la DLL que Nexus descarga al arrancar.
- **T-29 y T-30**: la matriz de trazabilidad las cita y no existen en `03-tasks.md`.

---

## HECHO – 2026-11-01 · Atajo con «Ñ» y botón *Grabar* (sin publicar)

- `keybinds.rs` traducía el código de escaneo con la tabla US (0x27 = «;»); ahora pide el nombre a
  Windows con la distribución del jugador (`ffi::keys::layout_key_name`, `GetKeyNameTextW`) y deja la
  tabla como respaldo. Tests `rf18_*` con teclado español falso y una llamada real con la L.
- Ajustes: fuera el botón *Grabar* y `TeamChange::SaveNow`; `persist_if_changed` ya guardaba cada
  cambio. En su lugar, «Los cambios se guardan solos.».
- No verificado: teclas extendidas (flechas, Insert) en `InputBinds.json` (`UNVERIFIED(RF-18)`), y la
  «Ñ» en el juego.

> Entradas de 0.26.1 a 0.26.3: [docs/historial/progreso-0.26.1-0.26.3.md](docs/historial/progreso-0.26.1-0.26.3.md).

> Entradas de 0.25.14 a 0.26.0: [docs/historial/progreso-hasta-0.26.0.md](docs/historial/progreso-hasta-0.26.0.md).

> Entradas anteriores a 0.25.13: [docs/historial/progreso-hasta-0.25.13.md](docs/historial/progreso-hasta-0.25.13.md).
