# 06 – Catálogo de escenarios (diff, rules, store, history, mapa)

Contrato de pruebas **antes del código**. Cada escenario es un test; el nombre de la función empieza por el ID en minúsculas (`d07_...`). Si un escenario cambia, se cambia aquí primero.

Convenciones: `S0` = snapshot anterior, `S1` = actual. Tiempos en UTC. "Seguida" = guild en la lista de seguimiento. Equipo propio = Verde salvo que se indique.

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
| B-03 | Sin S0, objetivo claimed por guild no seguida | primer diff | Sin eventos persistibles |
| B-04 | `match_id` distinto entre S0 y S1 | diff | `MatchChanged` y nueva línea base; sin flips por la diferencia (RF-03d) |
| B-05 | `S1.taken_at - S0.taken_at` > umbral | diff | `Gap{from,to}` (RF-26) |
| B-06 | Hueco y claim de guild seguida con `claimed_at` dentro del hueco | diff | `Gap` + `Claim{late:true}` |
| B-07 | Hueco y claim con `claimed_at` anterior al hueco | diff | Sin claim duplicado si ya estaba registrado |
| B-08 | Mismo snapshot repetido por API atascada (hash idéntico, tiempo avanza) | diff | Sin eventos; la capa superior marca "obsoleto" (RF-43) |
| B-09 | `taken_at` no monótono (S1 anterior a S0) | diff | Sin panic; S1 se descarta y se registra |

## R – `core::rules` (Event → Alert), equipo propio = Verde

| ID | Dado | Cuando | Entonces |
|---|---|---|---|
| R-01 | `Flip` Verde → Rojo, ámbito "todos" | rules | Alerta de pérdida (RF-10) |
| R-02 | `Flip` Rojo → Verde (lo tomamos nosotros) | rules | Sin alerta (informativa solo si RF-15 activo) |
| R-03 | `Flip` Rojo → Azul (enemigos entre sí) | rules | Sin alerta |
| R-04 | Pérdida en otro mapa, ámbito "solo mi mapa" | rules | Sin alerta (RF-12) |
| R-05 | Dos flips del mismo objetivo en 30 s, cooldown 60 s | rules | 1 alerta; la 2.ª solo al log (RF-13) |
| R-06 | `Claim` de guild seguida | rules | Alerta de claim con guild, objetivo, mapa (RF-21) |
| R-07 | `Claim` de guild no seguida | rules | Sin alerta |
| R-08 | `Claim{late:true}` de guild seguida | rules | Se registra; alerta solo si es reciente (configurable) |
| R-09 | `Gap` | rules | Aviso de estado "puede haber eventos perdidos", no alerta de objetivo |
| R-10 | Ámbito cambia en caliente | rules | Aplica a eventos posteriores, no reescribe el log |

## M – Resolución de equipo (`match_resolver`)

| ID | Dado | Cuando | Entonces |
|---|---|---|---|
| M-01 | Mundo en `worlds.green` | resolver | Equipo propio = Verde |
| M-02 | Mundo solo en `all_worlds.blue` (enlazado) | resolver | Equipo propio = Azul (RF-03b) |
| M-03 | Mundo no aparece en el match | resolver | Error tipado `WorldNotInMatch` |
| M-04 | Mismo mundo, match distinto la semana siguiente, color distinto | resolver | Color nuevo; no se persiste el color (RF-03d) |

## S – `store` (JSON-lines)

| ID | Dado | Cuando | Entonces |
|---|---|---|---|
| S-01 | Eventos escritos | leer | Mismos eventos, mismo orden |
| S-02 | Línea corrupta en medio del fichero | leer | Se omite, se cuenta y registra; el resto se lee |
| S-03 | Fichero truncado a mitad de línea | leer | Última línea descartada sin error fatal |
| S-04 | Línea con `v` desconocido | leer | Se omite y se avisa; sin panic |
| S-05 | Consulta por guild y rango | query | Solo eventos de esa guild dentro del rango (inclusivo/exclusivo documentado) |
| S-06 | Retención 30 días con ficheros mensuales antiguos | prune | Se eliminan los anteriores al límite |
| S-07 | Tamaño total > máximo | prune | Se eliminan los más antiguos hasta cumplir |
| S-08 | Directorio sin permisos de escritura | append | Error tipado; el addon sigue funcionando sin historial |
| S-09 | Ruta/ID con caracteres raros | append/leer | Sin inyección ni path traversal |

## H – `core::history` (ruta simulada)

| ID | Dado | Cuando | Entonces |
|---|---|---|---|
| H-01 | 5 claims de G en 2 h | ruta | 5 nodos numerados 1..5 en orden temporal (RF-32) |
| H-02 | Claims en dos mapas | ruta | Tramos separados por mapa con marca de salto (RF-35) |
| H-03 | Hueco entre dos claims > umbral | ruta | Tramo marcado discontinuo (RF-33) |
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
