# 06 – Catálogo de escenarios (diff, filtros de aviso y mapa)

Contrato de comportamiento. **Ojo con la nomenclatura:** los tests se llaman por el **requisito**
(`rf10_owner_change_emits_a_flip_with_the_real_time`), no por el ID `d07` que se usó al planificar;
la columna «Test» de cada tabla dice cuál es el nombre real o «—» si aún no lo hay. Si un escenario
cambia, se cambia aquí primero.

Convenciones: `S0` = snapshot anterior, `S1` = actual. Tiempos en UTC. «Seguida» = guild en la lista
de seguimiento. No hay módulo `rules` ni crate `store`: los filtros viven en `core::alerts::Filter`
y se aplican en `addon::alerts::Dashboard::recent_alerts`; el historial es de la sesión y está en
memoria (ADR-015).

## D – `core::diff` (Snapshot, Snapshot) → Vec<Event>

| ID | Dado | Cuando | Entonces |
|---|---|---|---|
| D-01 | S0 == S1 | diff | Sin eventos (propiedad: `diff(s,s)=∅`) |
| D-02 | Torre X: owner Verde → Rojo | diff | 1 `Flip{from:Verde,to:Rojo}` con `at = last_flipped` |
| D-03 | Campamento: Neutral → Azul | diff | 1 `Flip{from:Neutral,to:Azul}` |
| D-04 | Objetivo sin `claimed_by` pasa a guild G | diff | 1 `Claim{guild:G, at:claimed_at, late:false}` |
| D-05 | `claimed_by` G → None | diff | 1 `Unclaim{guild:G}` con `detected_at = S1.taken_at` |
| D-06 | `claimed_by` G → H | diff | `Unclaim(G)` y después `Claim(H)` (orden fijo) |
| D-07 | Mismo `claimed_by` G pero `claimed_at` posterior | diff | `Unclaim(G)` + `Claim(G)` (reclaim entre polls) |
| D-08 | Flip y claim del mismo objetivo en un intervalo | diff | `Flip` antes que `Claim`; orden por `at`, luego por id |
| D-09 | Varios objetivos cambian a la vez | diff | Salida ordenada de forma determinista (tiempo, luego id) |
| D-10 | Objetivo presente en S0 y ausente en S1 | diff | Sin panic; se ignora y se registra aviso de anomalía |
| D-11 | Objetivo nuevo en S1 que no estaba en S0 | diff | Sin `Flip`; se trata como línea base de ese objetivo |
| D-12 | Objetivo no reclamable (sin campos de claim) | diff | Sin eventos de claim; no falla el parseo |
| D-13 | Cambia solo `yaks_delivered` o `guild_upgrades` | diff | Sin `Flip`/`Claim` (solo candidato a RF-11) |
| D-14 | `claimed_at` anterior a `S0.taken_at` y `claimed_by` nuevo | diff | `Claim` con `late:true` (reloj o caché desfasados) |

## B – Línea base, cambio de match y huecos

| ID | Dado | Cuando | Entonces |
|---|---|---|---|
| B-01 | Sin S0 | primer diff | Sin `Flip`; ninguna alerta de flip (RF-04) |
| B-02 | Sin S0, objetivo claimed por guild seguida G | primer diff | `Claim{late:true, at:claimed_at}` (RF-25) |
| B-03 | Sin S0, objetivo claimed por guild no seguida | primer diff | `Claim{late:true}` como observación en la sesión (no avisa, alimenta las sugeridas de RF-20a) |
| B-04 | `match_id` distinto entre S0 y S1 | diff | `MatchChanged` y nueva línea base; sin flips por la diferencia (RF-03d) |
| B-05 | `S1.taken_at - S0.taken_at` > umbral | diff | `Gap{from,to}` (RF-26) |
| B-06 | Hueco y claim de guild seguida con `claimed_at` dentro del hueco | diff | `Gap` + `Claim{late:true}` |
| B-07 | Hueco y claim con `claimed_at` anterior al hueco | diff | Sin claim duplicado si ya estaba registrado |
| B-08 | Mismo snapshot repetido por API atascada (huella idéntica, tiempo avanza) | diff + worker | Sin eventos; `matchup::digest` + `payload_since` marcan la partida como congelada y el widget lo dice (RF-43) |
| B-09 | `taken_at` no monótono (S1 anterior a S0) | diff | Sin panic; S1 se descarta y se registra |
| B-10 | `claimed_by` puesto, `last_flipped` **posterior** a `claimed_at` | diff | Claim **caducado**: se descarta y no se pinta (`matchup::claim_holds`; bug de 2026-10-31, «un clan en un campa de otro color») |

## R – Filtros de aviso (`core::alerts::Filter` aplicado en `addon::alerts`)

No hay regla por «equipo propio» ni cooldown: se avisa de lo que pase en los mapas marcados, o de los
clanes seguidos en cualquier mapa, y **solo de estructuras reclamables** (RF-19e).

| ID | Dado | Cuando | Entonces | Test |
|---|---|---|---|---|
| R-01 | `Flip` en un mapa marcado, cualquier dueño | filtro | Avisa (RF-10) | `rf19_by_default_everything_is_told` |
| R-02 | `Flip` en un mapa **no** marcado y sin clan seguido | filtro | No avisa (RF-12) | `rf19_unticking_a_map_silences_only_that_map` |
| R-03 | `Flip` de unas **ruinas** con todas las casillas marcadas | filtro | No avisa (RF-19e) | `rf19e_ruins_never_alert` |
| R-04 | `Claim` de un clan seguido, en cualquier mapa | filtro | Avisa con clan, objetivo y mapa (RF-21) | `rf73_a_followed_guild_is_interesting_on_every_map` |
| R-05 | `Claim` de un clan no seguido y su mapa sin marcar | filtro | No avisa | `rf73_…` (caso contrario) |
| R-06 | «Solo los clanes que sigo» marcado | filtro | Manda sobre las casillas de mapa | `rf19a_only_the_followed_guilds_is_one_switch` |
| R-07 | `Claim{late:true}` de un clan seguido, hace minutos | filtro | Avisa (es lo que se perdió al cargar) | `rf25_late_claims_carry_the_real_claimed_at` |
| R-08 | `Claim{late:true}` de hace horas | filtro | No avisa; queda en el historial de la sesión | — (regla en `LATE_GRACE`) |
| R-09 | `Gap` | UI | Línea de estado «puede haber eventos perdidos», no alerta de objetivo | — |
| R-10 | El jugador cambia los filtros | render | Se aplica en el acto a la lista ya construida | `rf19_unticking_a_map_silences_only_that_map` |
| R-11 | Cooldown por objetivo | — | **No implementado**: se retiró de la UI y no está en el código | — |

## M – Resolución de equipo (`core::resolver` + `addon::team`)

| ID | Dado | Cuando | Entonces |
|---|---|---|---|
| M-01 | Id de equipo en `all_worlds` del match, color verde | resolver | Equipo propio = Verde |
| M-02 | Id de equipo que solo aparece en `all_worlds.blue` (junto a ids de mundo antiguos) | resolver | Equipo propio = Azul (RF-03b) |
| M-03 | Id de equipo que no aparece en ningún match vivo | resolver | Sin elección válida: se reabre el selector |
| M-04 | Mismo equipo, match distinto la semana siguiente, color distinto | resolver | Color nuevo; el color no se persiste (RF-03d) |
| M-05 | Sin id manual, personaje conocido | `team::resolve` | Manda el roster del personaje |
| M-06 | Id manual distinto del roster | `team::resolve` | Manda el manual |

## S – Historial de la sesión (antes `store`, hoy memoria: ADR-015)

| ID | Dado | Cuando | Entonces |
|---|---|---|---|
| S-01 | Eventos empujados | consultar por clan | Mismos eventos, en orden |
| S-02 | Más de 2 000 filas | push | Se descartan las más antiguas (`MAX_HISTORY`) |
| S-03 | Reinicio del juego | `load` | El historial empieza vacío: no hay fichero que leer |
| S-04 | Claim de un clan cualquiera | `guild_history` | Aparece como observación (alimenta RF-20a) |
| S-05 | Consulta de un clan sin historial | `guild_history` | Lista vacía, sin panic |
| ~~S-06..S-09~~ | Rotación, `prune`, permisos, path traversal | — | **Ya no existen**: no hay fichero de historial. El formato JSON-lines se retiró con `crates/store` |

## H – Ruta inferida (`core::route`)

| ID | Dado | Cuando | Entonces |
|---|---|---|---|
| H-01 | 5 claims de G en 2 h | ruta | 5 nodos numerados 1..5 en orden temporal (RF-32) |
| H-02 | Claims en dos mapas | ruta | Tramos separados por mapa con el texto «salto a <mapa>» (RF-35) |
| H-03 | Un solo claim | ruta | Un nodo y una flecha con la dirección (`heading`) |
| H-04 | Claim `late` | ruta | Nodo marcado `late` |
| H-05 | Dos claims con el mismo `at` | ruta | Orden estable (por id de objetivo) |
| H-06 | Rango sin eventos | ruta | Lista vacía, sin error |
| H-07 | Unclaim sin claim previo registrado | ruta/duración | Duración desconocida, sin panic |
| H-08 | Duración de claim | ruta | `unclaim.at - claim.at`; si sigue vigente, "en curso" |

## P – Proyección de mapa

| ID | Dado | Cuando | Entonces |
|---|---|---|---|
| P-01 | Coordenadas de objetivos de un mapa | límites | min/max + margen contiene todos los objetivos |
| P-02 | Punto proyectado y des-proyectado | ida y vuelta | Igual con tolerancia (propiedad) |
| P-03 | Mapa con un solo objetivo | límites | Caja mínima no degenerada, sin división por cero |
| P-04 | Ventana de tamaño cero | proyección | Sin panic, devuelve punto fuera de pantalla |

## Cómo se usan

1. La IA convierte cada fila en un test (`#[test] fn d07_same_guild_reclaim_emits_unclaim_then_claim()`).
2. Las filas que dependan de una decisión aún abierta se marcan `#[ignore = "..."]` con el motivo, nunca se omiten.
3. Escenarios nuevos que aparezcan en producción se añaden aquí **antes** de corregir el código.
