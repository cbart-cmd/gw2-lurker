# 01 – Requisitos

Formato EARS + criterios de aceptación Given/When/Then (GWT). Prioridad: **M** = must (v1), **S** = should, **C** = could.

## Hechos verificados que condicionan los requisitos

- `/v2/wvw/matches?world=<id>` devuelve el match con todos los objetivos. Por objetivo: `owner`, `last_flipped`, `claimed_by` (ID de guild), `claimed_at`, `points_tick`, `yaks_delivered`, `guild_upgrades`. `claimed_by`/`claimed_at` no existen en objetivos no reclamables.
- El match incluye `worlds` (3 primarios) y `all_worlds` (con los mundos enlazados por color).
- La API **no** expone estado "bajo ataque" (ADR-001) ni historial: solo el último claim vigente.
- La API a veces se queda atascada o devuelve 503/"unknown error" durante minutos u horas (reportado por la comunidad). Hay que tolerarlo (RNF-04).
- Hay prior art con los mismos endpoints (Realm Report, C++/Nexus, polling 30 s por defecto). Sirve de referencia, no de dependencia.

## A. Activación y contexto

**RF-01 (M)** Cuando el MumbleLink indique que el jugador está en un mapa de WvW, el addon deberá activar polling y UI.
- GWT: *Given* el jugador está en Ojos de Aguas Eternas, *When* el addon recibe el mapa por Mumble, *Then* arranca el polling en ≤ 1 intervalo.

**RF-02 (M)** Cuando el jugador salga de un mapa de WvW, el addon deberá detener el polling y ocultar toda su UI (excepto la ventana de configuración si está abierta).
- GWT: *Given* el polling activo, *When* el jugador viaja a una ciudad, *Then* no se emiten peticiones de red y no se dibuja nada.

**RF-03a (M)** Si no hay mundo configurado, el addon deberá pedir al usuario que elija su mundo de una lista obtenida de `/v2/worlds?ids=all`.

**RF-03b (M)** El addon deberá resolver el match con `/v2/wvw/matches?world=<id>`, buscando el mundo en `all_worlds` (no solo en `worlds`) para deducir el color propio y los de los enemigos.

**RF-03c (M)** El usuario podrá cambiar de mundo en cualquier momento desde la configuración.

**RF-03d (M)** Cuando cambie el ID del match (reset semanal), el addon deberá descartar el baseline anterior, resolver de nuevo el color y marcar el historial con el nuevo `match_id`.
- GWT: *Given* un match `2-5` en curso, *When* la API devuelve `2-6` para el mismo mundo, *Then* no se emiten alertas por la diferencia y se crea un baseline nuevo.

**RF-03e (C)** Si MumbleLink aporta `world_id`, el addon podrá sugerirlo como valor por defecto, nunca como única fuente.

**RF-04 (M)** El primer snapshot tras activar, cambiar de match o recuperar tras un hueco largo se tratará como **baseline** y no generará alertas de flip.

## B. Objetivo 1 – Alertas de estructuras

> Alcance real según ADR-001: avisos de **flip/captura** y de **cambios observables**, no de "ataque en curso".

**RF-10 (M)** Cuando un objetivo propio pase a manos de un equipo enemigo, el addon deberá mostrar una alerta en el widget fijo (RF-16) con nombre, tipo, mapa y hora.
- GWT: *Given* "Torre X" con `owner=Green` y mi equipo Green, *When* un snapshot la muestra `owner=Red`, *Then* aparece en el widget la línea "Torre X perdida ante Rojo (mapa)".

**RF-11 (S)** Cuando cambie un estado observable de un objetivo propio (descenso de `yaks_delivered`, cambio de `guild_upgrades`, pérdida de claim), el addon deberá emitir un aviso de prioridad baja, configurable.

**RF-12 (M)** El usuario podrá elegir el ámbito de las alertas: *solo mi mapa actual* o *todos los mapas del match*.

**RF-13 (M)** El addon deberá permitir activar/desactivar sonido y configurar un cooldown anti-spam por objetivo (por defecto 60 s).
- GWT: *Given* cooldown 60 s, *When* un objetivo cambia de dueño dos veces en 30 s, *Then* solo se muestra una alerta (la segunda se agrupa en el log).

**RF-14 (M)** El addon deberá mantener un log de alertas recientes (últimas N configurables), accesible desde el widget en una ventana ampliada.

**RF-15 (S)** Se podrán emitir también avisos de captura enemiga de objetivos neutrales o de objetivos enemigos tomados por mi equipo (informativo), desactivados por defecto.

### B2. Widget fijo de alertas (ADR-006)

**RF-16 (M)** El addon deberá mostrar las alertas en un **widget fijo** en pantalla: una lista compacta de las últimas alertas, la más reciente arriba, visible solo en mapas de WvW. Si no hay alertas vigentes y el widget está bloqueado, se oculta por completo (opción para mostrar "Sin novedades").
- GWT: *Given* el jugador en WvW con el widget visible, *When* se genera una alerta, *Then* aparece como nueva primera línea en ≤ 1 frame tras llegar al estado de UI.

**RF-17 (M)** El widget deberá poder moverse y redimensionarse libremente, y su posición y tamaño deberán persistir entre sesiones.

**RF-18 (M)** El widget deberá tener modo **bloqueado**: sin cabecera ni bordes interactivos y sin capturar el ratón (los clics pasan al juego). Se alterna desde la configuración o con el atajo por defecto `Alt+Shift+L` (keybind gestionado por Nexus, reasignable).
- GWT: *Given* el widget bloqueado, *When* el usuario hace clic sobre él, *Then* el clic llega al juego.

**RF-19 (M)** Cada línea deberá mostrar: icono/forma de tipo de objetivo, texto, mapa y antigüedad ("hace 2 min"), con color según el equipo implicado y distinción por tipo de alerta (pérdida, claim de guild seguida, informativa). No se usará solo el color para distinguirlas.

**RF-19a (M)** Las líneas deberán desvanecerse y desaparecer tras un tiempo configurable (por defecto 60 s), con opción de mantenerlas hasta descartarlas. Las alertas de guilds seguidas podrán configurarse como persistentes.

**RF-19b (S)** Clic en una línea (widget desbloqueado) abrirá el mapa simplificado centrado en ese objetivo.

**RF-19c (S)** El usuario podrá configurar opacidad, tamaño de fuente, máximo de líneas visibles (por defecto 5) y qué tipos de alerta se muestran en el widget.

**RF-19d (C)** Toast emergente adicional y sonido opcional para alertas críticas, desactivados por defecto salvo el sonido ya cubierto por RF-13.

## C. Objetivo 2 – Seguimiento de guilds enemigas

**RF-20 (M)** El usuario podrá añadir guilds a seguir por nombre (`/v2/guild/search?name=`) o por ID, y quitarlas. Se guardará ID, nombre y tag (`/v2/guild/:id`).
- GWT: *Given* un nombre ambiguo, *When* la búsqueda devuelve varias guilds, *Then* el usuario elige entre candidatos con nombre y tag.

**RF-21 (M)** Cuando un objetivo pase a estar reclamado por una guild seguida, el addon deberá avisar con guild, objetivo, mapa y hora del claim.

**RF-22 (M)** El addon deberá registrar cada claim y unclaim observado de guilds seguidas: timestamp, `match_id`, objetivo, mapa, guild, tipo de evento, flag `late`.

**RF-23 (M)** El usuario podrá ver el estado actual: qué objetivos tiene reclamados cada guild seguida ahora mismo.

**RF-24 (M)** El historial deberá persistir entre sesiones y poder consultarse por guild y rango de tiempo.

**RF-25 (M)** En baseline o tras un hueco, para cada objetivo con `claimed_by` de una guild seguida, el addon deberá registrar un claim marcado `late=true` usando `claimed_at` como hora real.

**RF-26 (M)** Cuando el hueco entre snapshots válidos supere el umbral (por defecto 3 × intervalo), el addon deberá registrar un evento `gap` para que la UI advierta de que pueden faltar claims.

**Limitación conocida:** la API solo da el último claim vigente. Un claim que ocurra y se libere entre dos polls, o dos claims consecutivos de guilds distintas entre dos polls, no se pueden observar. La spec no promete completitud (RNF-05).

## D. Vistas de mapa

**RF-30 (M)** Mapa simplificado por mapa de WvW: objetivos como figuras coloreadas por equipo (forma por tipo) sobre un fondo esquemático, sin texturas del juego (ADR-002). Selector de mapa.
- GWT: *Given* un snapshot, *When* se abre la vista, *Then* cada objetivo aparece en su posición relativa con el color de su dueño.

**RF-31 (M)** Capa de claims: marca visualmente los objetivos reclamados por guilds seguidas (anillo/etiqueta con tag de guild).

**RF-32 (M)** Vista **Histórico** por guild: dibuja los claims en orden cronológico como polilínea numerada y direccional (ruta simulada), con selector de guild y de rango temporal.
- GWT: *Given* 5 claims de la guild G en 2 h, *When* se selecciona G y el rango "últimas 2 h", *Then* se dibujan 5 nodos numerados unidos por flechas en orden temporal.

**RF-33 (M)** La vista Histórico deberá indicar de forma visible que la ruta es **inferida**, y los claims `late` y los tramos con `gap` se dibujarán con estilo distinto (trazo discontinuo).

**RF-34 (M)** Al pasar el ratón por un nodo: objetivo, hora, duración del claim y si es `late`.

**RF-35 (S)** Si los nodos cruzan mapas distintos, la ruta se divide en tramos por mapa con un indicador "salto a <mapa>".

**RF-36 (C)** Reproducción animada de la ruta (play/pausa) sobre el tiempo.

## E. Configuración y UX

**RF-40 (M)** Ventana de configuración: mundo, ámbito de alertas, opciones del widget (posición, bloqueo, opacidad, líneas, tiempo de vida), sonido, cooldown, intervalo de polling, retención de historial, guilds seguidas.

**RF-41 (M)** Persistencia de la configuración en el directorio de addon de Nexus.

**RF-42 (S)** Atajo de teclado y/o icono de acceso rápido de Nexus para abrir/cerrar la ventana principal.

**RF-43 (M)** Indicador de estado: último refresco correcto, "obsoleto" si supera N intervalos, error de red resumido.

## E2. Actualización, integridad y fallo seguro

**RF-50 (M)** Cuando el fichero de la DLL en disco cambie respecto a la versión cargada (ADR-007), el addon deberá mostrar un aviso visible y persistente: "Actualizado a vX.Y.Z, reinicia el juego para aplicar", sin intentar recargarse en caliente.
- GWT: *Given* v0.1.0 cargada, *When* Nexus deja v0.2.0 en disco, *Then* el banner aparece en ≤ 2 min y permanece hasta reiniciar.

**RF-51 (M)** Si la inicialización o la lectura de `config.json` o del historial falla, el addon deberá arrancar en **modo seguro**: renombrar el fichero dañado a `*.corrupt-<fecha>`, usar valores por defecto, avisar en UI (W8) y no cerrar el juego.
- GWT: *Given* un `config.json` con JSON truncado, *When* carga el addon, *Then* se conserva una copia `.corrupt`, se crea una configuración por defecto y se muestra el aviso.

**RF-52 (S)** La ventana de configuración mostrará la versión actual y el estado de actualización.

**RF-53 (M)** El addon declarará `UpdateProvider::GitHub` y `DisableHotloading` (ADR-007), y publicará el SHA-256 de cada release.

**Nota de alcance:** la validación de DLL corruptas al cargarlas es responsabilidad de Nexus. El proyecto aporta lo que está a su alcance: publicar checksums, probar en CI que la DLL generada es un PE x64 válido que exporta el símbolo de entrada, y fallar de forma segura (RF-51).

## E3. Apariencia (ADR-009)

**RF-60 (M)** Las ventanas del addon (configuración, log, mapa, histórico) deberán heredar por defecto el estilo de imgui que aplica Nexus.

**RF-61 (S)** La configuración ofrecerá un selector de estilo de ventanas: *Heredado de Nexus* (por defecto) o *Propio del addon* (tema oscuro definido por el addon).
- GWT: *Given* el estilo "Propio", *When* se abre cualquier ventana del addon, *Then* usa el tema del addon sin afectar a las ventanas de Nexus ni de otros addons.

**RF-62 (M)** Los colores de equipo serán fijos y clásicos: rojo, azul y verde del juego, y los mismos en widget, mapa e histórico. No hay modo accesible en v1, por lo que la forma de cada objetivo y el texto de cada alerta deben transmitir la información sin depender del color (refuerza RF-19).

**RF-63 (M)** El widget de alertas tendrá fondo configurable (transparente con texto con sombra, o semitransparente oscuro). Por defecto, semitransparente; la opacidad es ajustable (RF-19c).

## F. No funcionales

| ID | Requisito | Verificación |
|---|---|---|
| RNF-01 | Frame time añadido < 0.5 ms en render (widget y mapa abiertos) | Medición con temporizador en dev |
| RNF-02 | Intervalo de polling configurable; mínimo 15 s; por defecto 30 s hasta medir la caché real (T-02) | Test + spike |
| RNF-03 | Historial acotado: retención en días y tamaño máx. (por defecto 30 días / 20 MB) con rotación | Test de rotación |
| RNF-04 | Sin red: UI funcional con último estado y marca "obsoleto"; backoff exponencial con tope | Test con cliente simulado |
| RNF-05 | El historial es "mejor esfuerzo": no se promete completitud | Documentado en UI y README |
| RNF-06 | Sin `panic` a través de FFI; errores internos a log | Revisión + `catch_unwind` en puntos de entrada |
| RNF-07 | Compatible con las versiones de Nexus/GW2 objetivo; el addon se desactiva sin romper si falla la carga | Prueba manual |
| RNF-08 | Textos de UI preparados para i18n (ES/EN) | Revisión |
| RNF-09 | DLL release ≤ 1.5 MB objetivo, 2.5 MB límite (ADR-008) | `check_size.sh` en CI |
| RNF-10 | Ningún fichero de código > 800 líneas | `check_loc.sh` en CI |
| RNF-11 | Formato y lints: `rustfmt` y `clippy -D warnings` | CI |
| RNF-12 | DLL generada es PE x64 válido y exporta el punto de entrada de Nexus | Test de humo en CI |

## G. Fuera de alcance (v1)

Backend compartido entre usuarios, posición real de jugadores, alertas fuera del juego (Discord, móvil), detección de "bajo ataque", lectura de memoria del juego, mapas con texturas oficiales.

## Matriz de trazabilidad (resumen)

| Requisito | Módulo de diseño | Tarea |
|---|---|---|
| RF-01/02 | `nexus_ui::context` | T-12 |
| RF-03*, RF-04 | `core::match_resolver`, `core::baseline` | T-05, T-06 |
| RF-10..15 | `core::diff`, `core::rules`, `nexus_ui::alerts` | T-07, T-08, T-13 |
| RF-16..19d | `addon::ui::alert_widget` | T-13, T-13b |
| RF-20..26 | `core::follow`, `store`, `api::guild` | T-09, T-10, T-11 |
| RF-30..36 | `map::*` | T-14, T-15, T-16 |
| RF-40..43 | `config`, `nexus_ui::settings` | T-12, T-17 |
| RF-60..63 | `addon::ui::theme` | T-28 |
| RF-50..53 | `addon::update`, `addon::safe_mode` | T-25, T-26, T-27 |
