# 03 – Tareas

Cada tarea es pequeña y verificable. Orden recomendado; las dependencias se indican entre paréntesis.

## Fase 0 – Spikes (reducen riesgo)

- **T-01** Spike Nexus: addon mínimo con `export!` que dibuja una ventana imgui, lee `map_id` de Mumble y envía una alerta con `send_alert`. *Hecho cuando:* se carga en el juego y muestra el `map_id` en vivo. Confirma R-1 y R-6.
- **T-02** Spike API: script que consulta `/v2/wvw/matches?world=<id>` cada 10 s durante 30 min y registra cuándo cambian los datos. *Hecho cuando:* hay una cifra medida de caché real y de errores; se fija el intervalo por defecto (R-2).

## Fase 0b – Fundamentos de ingeniería

- **T-03 ✔ (2026-10-31)** Workspace Rust con `rustfmt.toml`, `[workspace.lints]` y `rust-version`. *Hecho:* `cargo fmt --check`, `clippy -D warnings` y `cargo test` verdes en un workspace vacío. Nota: no existe `tooling/` en el repo; la configuración se creó a mano (`rustfmt.toml`, `rust-toolchain.toml`, `deny.toml` ya estaban). `license` sigue sin decidir: **bloquea `cargo deny` en T-03a**.
- **T-03a ✔ (2026-10-31)** CI en `.github/workflows/ci.yml`: `cargo fmt --check`, `clippy -D warnings`, `cargo test`, `check_loc.sh`, `cargo deny` + `cargo audit`, build release `x86_64-pc-windows-msvc` y `check_size.sh`. *Pendiente por depender de otras tareas:* test de humo PE/export (T-27) y check de trazabilidad RF → tests (T-03b). Verificado que `check_loc.sh` falla con un fichero de 801 líneas.
- **T-03b ✔ (2026-10-31)** Check de trazabilidad RF → tests y `manual-tests.md`: `scripts/check_traceability.sh`, en CI. Falla si un test cita un requisito inexistente; informa de los que aún no tienen prueba; `--strict` falla por cobertura para usarlo antes de publicar. (T-03a)
- **T-02b** Spike de tamaño: medir DLL mínima con 2–3 opciones de cliente HTTP/TLS y fijar presupuesto real (ADR-008). (T-01)
- **T-24** Revisión de bocetos `05-wireframes.md` y cierre de sus 3 pendientes.

## Fase 1 – Núcleo (sin juego)

- **T-03** Workspace + CI local (`fmt`, `clippy`, `test`). *Hecho:* `cargo test` verde. **(Duplicado de la T-03 de la Fase 0b: la parte de workspace está hecha y la de CI es T-03a.)**
- **T-04 ✔ (2026-10-31)** `core::model` y deserialización serde de fixtures reales del match y de objetivos. *Hecho:* 11 tests contra `crates/api/tests/fixtures/` (match `2-2`, objetivos, mapas). (T-03)
- **T-05 ✔ (2026-10-31)** `match_resolver`: equipo → color propio/enemigos buscando el `team_id` en `all_worlds`; nombres desde `data/wvw-teams.json` embebido (`include_str!`). *Hecho:* 6 tests con equipo, mundo enlazado, id desconocido, tabla corrupta e id sin nombre. (T-04) — RF-03a, RF-03b, RF-03g, RF-03i
- **T-06 ✔ (2026-10-31)** Baseline y cambio de match en `diff`: `Snapshot`, `Differ` con `Event::{Baseline, MatchChanged, Gap}`. *Hecho:* 8 tests (baseline de sesión, cambio de match, hueco > 3×intervalo y su borde exacto). (T-04) — RF-03d, RF-04, RF-26
- **T-07 ✔ (2026-10-31)** `diff` de flips y claims/unclaims con orden determinista. *Hecho:* `Event::{Flip, Claim, Unclaim}`; recorre los objetivos por id y emite `Unclaim` antes de `Claim`; el baseline no emite ninguno de los tres. 6 tests. (T-06) — RF-10, RF-21, RF-22
- **T-08** `rules`: ámbito, equipo propio, cooldown, guilds seguidas. (T-07) — RF-12, RF-13
- **T-09 ✔ (2026-10-31)** Claims tardíos y `Gap`: `Event::Claim` lleva `late`; un baseline (sesión, cambio de match o hueco) registra **todos** los claims vigentes con su `claimed_at` real y `late=true` (RF-25). El `Gap` se emite antes del baseline. (T-07) — RF-25, RF-26
- **T-10 ✔ (2026-10-31, parcial)** `store` JSON-lines: append, lectura, línea corrupta, rotación y descarte de la sesión anterior (ADR-012). 6 tests. Falta la consulta por guild/rango, que es de T-11. (T-04) — RF-24, RNF-03
- **T-11** `follow` + `history`: estado actual por guild y consulta por rango. (T-09, T-10) — RF-23

## Fase 2 – API y worker

- **T-11b ✔ (2026-10-31)** `api`: `trait Transport` + `Api<T>` tipado (`matches_all`, `match_for_team`, `objectives(lang)`, `guild`), `ApiError` y `Backoff` exponencial con tope. 9 tests con las fixtures reales, incluidos 503, JSON truncado y equipo inexistente. Añadida la caché en disco (`cache::Cache`: objetivos por idioma + guilds, tolerante a ficheros corruptos, 4 tests). Cliente HTTP real en el addon (`net::WinHttpTransport` sobre WinHTTP del sistema, medidas en `07-spikes.md`: HTTP 200 en 270 ms sin crecer la DLL). (T-04) — RF-20, RF-44, RNF-04
- **T-11c ✔ (2026-10-31, parcial)** `worker`: hilo propio con parada limpia (`stop()` + sondeo
  de 250 ms), `MAY_POLL` desde la compuerta del render (nunca red ni disco en el hilo de render),
  `ids=all` mientras no hay equipo y `/wvw/matches/{id}` (nuevo `Api::match_by_id`) cuando lo
  hay, backoff hasta 5 min, `Health::{Idle,Ok,Degraded}`, histórico en `SessionStore` y
  `Dashboard` (claims, clanes, nombres, últimos 50 eventos) que la ventana de estado ya lee.
  Probado de verdad con `live_pipeline_builds_the_dashboard` (HTTP real, 1,2 s). Falta: dejar de
  escribir `settings.json` en mitad de un poll largo y separar el sondeo de objetivos del de
  estado. (T-11b, T-07) — RF-20, RF-26, RNF-04

## Fase 3 – Integración Nexus

- **T-31 ✔ (2026-10-31, parcial)** `i18n`: tabla es/en/fr/de en `core::i18n` (36 claves),
  funciones `t`/`w`/`f` con `###clave` para ids estables de imgui, fallback a inglés y mapa de
  `GameLanguage` verificado en `nexus-rs` (0=en, 1=coreano, 2=fr, 3=de, 4=es, 5=chino).
  `addon::i18n` resuelve preferencia → idioma del juego → inglés, aplica el override de Ctrl
  derecho (`ffi::keys`, `GetAsyncKeyState` + proceso en primer plano) y registra las claves en
  Nexus con `set_translation`. Tests: 9 en `core` (incluida la completitud de las cuatro
  columnas), 3 en el addon. Falta la prueba en juego (T-35) y que T-41 confirme la
  terminología de tipos con nativos. (T-03) — RF-44, RF-45, RNF-08

- **T-12 ✔ (2026-10-31, parcial)** `context` + `config`: activación solo en WvW, persistencia y export real. Hecho: `export!` con `Direct` + `DisableHotloading` + URL de distribución (verificada dentro de la DLL), lectura de `NexusLink::is_gameplay` y del mapa de Mumble en `ffi/`, `config.json` con valores por defecto, saneo y copia `.corrupt-*` ante JSON dañado (5 tests), y ventana de estado mínima que respeta `core::gate`. Falta: selector de equipo (T-30), ventana de configuración (T-17) y la prueba dentro del juego (T-35). (T-01, T-05) — RF-01, RF-02, RF-02a, RF-03a/c, RF-40, RF-41

- **T-30 ✔ (2026-10-31, parcial)** `addon::ui::settings`: selector en **tres pasos** — región
  (EU/NA, solo las que tienen matchups vivos) → color (rojo/verde/azul, con su color real en la
  lista) → equipo, con `nombre (id) · matchup`. Con los datos del 2026-10-31 son 5 filas en EU y
  4 en NA, en vez de 27. El desplegable arranca en la región y color del equipo ya guardado. Si
  no hay primer sondeo, lo dice en vez de mostrar una lista vacía; si la combinación no tiene
  equipos, también. Alias editable con aviso de que es local, y campo de idioma, intervalo y
  alcance. Falta comprobar los dos rivales contra el matchup y el resto de la ventana. (T-05)
  — RF-03a, RF-03f, RF-03g
- **T-13 ✔ (2026-10-31, parcial)** Widget de avisos con **filtro de origen** (RF-19a):
  casillas para Eternos, Alpino azul, Alpino verde, Desierto rojo, «el mapa en el que estoy» y
  «los clanes que sigo», todas marcadas por defecto. Un clan seguido avisa en **cualquier** mapa
  del emparejamiento; el mapa actual se recalcula con el `map_id` de Mumble (38/95/96/1099). El
  **pop-up** dura 0–5 s (2 s por defecto, `popup_secs`), se dibuja arriba en el centro y
  desaparece sin quitar nada de la lista; los claims tardíos del baseline no hacen pop-up. La
  lista de clanes a seguir sale de los que tienen algo reclamado en el matchup. (T-08, T-12)
  — RF-10, RF-16, RF-17, RF-19, RF-19a, RF-73
- **T-13b ✔ (2026-10-31)** Modo bloqueado click-through: una ventana bloqueada usa
  `WindowFlags::NO_INPUTS`, así que sigue dibujándose pero todos los clics llegan al juego.
  `Alt+Shift+L` (keybind de Nexus, reasignable allí) bloquea o desbloquea **todas** las ventanas;
  el handler solo levanta un `AtomicBool` y el render lo aplica, para no coger el candado de la
  configuración desde el hilo de entrada. Los ajustes tienen una casilla de click-through por
  ventana y una **ventana de registro de avisos** con la lista completa y scroll. (T-13)
  — RF-14, RF-18
- **T-42 ✔ (2026-10-31)** Una sola ventana con pestañas, como `docs/boceto.html` (ADR-014):
  **Avisos · Mapa · Clanes · Registro** y la **última, Ajustes** (equipo, idioma, intervalo,
  filtros de aviso y clanes seguidos dentro de la propia ventana). Un único `WindowState` para
  toda la ventana. El **pop-up** se activa y desactiva desde ahí (`popup_enabled` + `popup_secs`)
  y desde las opciones de Nexus, que además ofrecen **Ver la ventana**; el pop-up se dibuja
  aunque la ventana esté cerrada. (T-13, T-17) — RF-14, RF-16, RF-19a, RF-46
- **T-17 ✔ (2026-10-31, parcial)** Contrato de ventana compartido en `addon::ui::window`:
  barra de título propia con `×`, plegado con doble clic, mover/redimensionar solo si no está
  bloqueada, opacidad sobre el fondo, menú contextual (bloquear, opacidad, restablecer) y
  estado por ventana (`visible`, `collapsed`, `locked`, `opacity`, `position`, `size`) en
  `config.json` bajo `windows.<clave>`, saneado y con mínimo de tamaño. La ventana de estado ya
  lo usa. Contenido de ajustes hecho: equipo + alias (T-30), idioma, intervalo de sondeo
  (15–600 s, saneado al guardar) y alcance de los avisos. Falta el indicador de obsoleto y el
  resto de opciones de sonido/atajos. (T-12) — RF-43, RF-46

## Fase 4 – Mapas

- **T-14** Proyección `world_to_screen` + mapa simplificado con colores y formas. (T-04, T-12) — RF-30
- **T-15** Capa de claims y vista de estado por guild. (T-14, T-11) — RF-31, RF-23
- **T-16** Vista Histórico: ruta simulada numerada, flechas, selector de rango, banner de "inferida", estilo para `late`/`gap`, tooltip. (T-15) — RF-32, RF-33, RF-34

- **T-29 (S)** `map::background` y `map::icons`: fondo opcional con teselas oficiales e iconos de objetivo del campo `marker`, con caché en disco y sin empaquetar imágenes (ADR-011). Desactivado por defecto. *Hecho:* el mapa se ve igual con el fondo apagado. (T-14) — RF-37, RF-38

## Fase 4b – Actualización y fallo seguro

- **T-25** Spike y aviso de actualización: comprobar cómo deja Nexus la DLL nueva y construir el banner W7 (ADR-007). (T-01, T-12) — RF-50, RF-53
- **T-26** Modo seguro: config/historial corruptos → `.corrupt`, valores por defecto, aviso W8; `catch_unwind` en la frontera FFI. (T-10, T-12) — RF-51, RNF-06
- **T-27** Pipeline de release: DLL en CI, SHA-256, test de humo PE/export con crate `object`, registro de tamaño en CHANGELOG. (T-03a, T-02b) — RNF-09, RNF-12

## Fase 5 – Aceptación y pulido

- **T-18** Medición de frame time con mapa abierto. *Hecho:* < 0.5 ms (RNF-01).
- **T-19** Checklist manual en el juego: widget bloqueado sin capturar clics, activar/desactivar por mapa, reset semanal, sin red, API 503, guild con claims reales.
- **T-20** README, CHANGELOG, `docs/` de usuario, instalación, limitaciones conocidas (RNF-05), empaquetado de la DLL.
- **T-28** Módulo `theme`: herencia del estilo de Nexus, tema propio opcional aplicado solo a ventanas del addon (stack de estilo push/pop), colores de equipo fijos y fondo del widget configurable. (T-13, T-17) — RF-60..63
- **T-21 (S)** RF-11, RF-15, RF-19b, RF-19c, RF-35, atajo/icono de Quick Access.
- **T-22 (C)** RF-19d, RF-36, reproducción animada de la ruta.

## Fase 6 – MVP de validación (backlog priorizado)

Orden pensado para llegar cuanto antes a "lo veo en el juego y me sirve para decidir". Cada
tarea indica de qué depende.

1. **T-32 (M)** Bocetos y `docs/boceto.html` navegable para validar la UI antes de escribirla.
   *Hecho:* el HTML enseña widget, mapa, lista de guilds e historial con datos reales de
   ejemplo, y se puede abrir sin el juego. Requiere `PRODUCT.md` (flujo `impeccable init`).
   — RF-16, RF-30, RF-70..73
2. **T-33 (M)** No dibujar nada ni pollear en carga, selección/creación de personaje: señal
   `NexusLink::is_gameplay` con respaldo por `map_type`. (T-12) — RF-02, RF-02a
   - **Parcial (2026-10-31):** la decisión pura está hecha en `core::gate` (6 tests). Falta
     conectar `NexusLink::is_gameplay` y `Context` cuando exista T-12.
3. **T-34 ✔ (2026-10-31)** Publicar la DLL en `..\gw2addonsreleases` siguiendo `docs/RELEASE.md`
   (MD5, excepción LFS, tag `gw2-lurker-v*`, retención actual+anterior). Automatizado en
   `scripts/release.ps1` (`-DryRun` / normal / `-VerifyOnly`), que aborta en vez de publicar
   un puntero LFS, un `.md5` que no mida 32 bytes o una descarga que no coincida. El fichero
   pasa a llamarse `gw2_lurker.dll` por el `[lib] name` del addon. (T-27) — RF-53
4. **T-35 (M)** Arrancar el addon dentro del juego con este repo: `export!` real, versión
   visible y traza en `addons/Nexus/Nexus.log`. Es la T-01 ejecutada de verdad. (T-33, T-34)
5. **T-36 ✔ (2026-10-31)** Vista **"Claims del matchup"**: los cuatro mapas agrupados y
   plegables, cada fila `hora · objetivo · clan` con el color del dueño y el mapa actual marcado
   con «estás aquí». Filtros de sesión, todos activos al empezar: campamentos, torres,
   fortalezas/castillos y «solo alto valor». (T-11, T-15) — RF-70
6. **T-37 ✔ (2026-10-31)** Lista de clanes con claim ordenada por la más reciente, con número
   de claims, hora y mapa de la última; estrella para **seguir** el clan, que es lo que hace que
   avise en cualquier mapa del matchup. (T-36) — RF-71
7. **T-38 ✔ (2026-10-31)** Historial por clan al seleccionarlo, **separado entre el mapa en el que
   estás y los demás**, en orden cronológico. Se alimenta de los `Claim` de la sesión
   (`Dashboard::history`, tope 2000; el `store` guarda todo). (T-16, T-37) — RF-72
8. **T-39 ✔ (2026-10-31)** Favoritos: la propia lista de clanes tiene la estrella de
   seguir/dejar de seguir y se guarda en `settings.json` (`followed`), además de en los ajustes.
   `FollowList` ya impedía duplicados y `favourites_first` mantiene el orden por recencia.
   (T-37) — RF-73
9. **T-40 ✔ (2026-10-31, parcial)** `core::route`: ruta de un clan en un mapa a partir de sus
   claims, en orden cronológico, con **origen inferido**: fortaleza (guarnición) → fortaleza
   lateral (torres/castillo) → punto de reaparición. El origen no se repite como paso: la UI lo
   dibuja como inicio. La ventana de clanes ya enseña la ruta en texto, con el aviso
   «Ruta inferida a partir de las reclamaciones» (ADR-005). **Falta**: pintarla sobre el mapa
   simplificado, que llega con T-14 (proyección `world_to_screen` y `continent_rect`), y afinar
   qué objetivo es la guarnición de cada mapa (los `Keep` de la API incluyen colinas y bahía).
   (T-38) — RF-32a
10. **T-41 (S)** Cerrar i18n: terminología de tipos con hablantes nativos de fr/de. (T-31)
    — RF-44, RNF-08

> Nota: la matriz de trazabilidad de `01-requirements.md` cita **T-29** y **T-30**, que no
> existen en este fichero. Hay que crear ambas o corregir la matriz.
