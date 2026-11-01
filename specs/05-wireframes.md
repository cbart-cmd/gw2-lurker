# 05 – Bocetos de interfaz (wireframes)

Boceto de **estructura**, no de estilo visual, y actualizado al código de la 0.25.x. En el juego los
iconos se dibujan con primitivas (`ui::icons`), no con estos caracteres, y **todo el texto va en
ASCII + latino-1**: la fuente de Nexus no tiene flechas ni estrellas (se dibujan como «?»).

Una sola ventana con pestañas (ADR-014): **Avisos · Mapa · Clanes · Registro · Ajustes**.

## W1 – Pestaña de avisos / widget (RF-16..19e)

Desbloqueada (movible/redimensionable, con cromo):
```
┌ WvW Watch ─ [Avisos|Mapa|Clanes|Registro|Ajustes] ─ 🔒 × ┐
│ 12:31  Azul  ·  Zanja de Mendon  -  [YaK] Dolyak  -  36 s│
│ 12:28  Verde ·  Torre Durios     -  Rojo -> Azul    3 min│
│ 12:20  Rojo  ·  Campamento X     -  sin reclamar    9 min│
└──────────────────────────────────────────────────────────┘
```
Cada línea: hora, mapa con su color (Eternos en gris, Azul/Verde/Rojo en el suyo), objetivo, y en
gris el clan o el cambio de dueño (`Rojo -> Azul`). El tipo de objetivo va con su dibujo delante.
Bloqueada y en click-through (sin cromo, los clics van al juego):
```
 12:31  Azul  ·  Zanja de Mendon  -  [YaK] Dolyak
 12:28  Verde ·  Torre Durios     -  Rojo -> Azul
```
Estados: sin alertas y bloqueado → no se dibuja nada (indicador de estado con «sin novedades»);
datos obsoletos → línea fija `Datos obsoletos desde 14:02`; API congelada → aviso naranja
`La API de ArenaNet no publica cambios de esta partida desde hace N min`.

## W2 – Elegir equipo (RF-03a, RF-03g), dentro de Ajustes
```
┌ Ajustes ─────────────────────────────────────────────────┐
│ Región:   [EU ▾]                                         │
│ Color:    (Rojo) (Verde) (Azul)                          │
│ Equipo:   [Great House Aviary (12014) ▾]  ← nombre (id)  │
│ Traducción: [ Invernadero de Lutgardis            ]      │
│ (Comprobación: eres AZUL; rivales Bava Nisos (rojo)…)    │
└──────────────────────────────────────────────────────────┘
```
El nombre viene de `data/wvw-teams.json` y su traducción de `data/team-names.json`; si falta, se
muestra el id. La identidad es el **id de equipo**: nombres y colores cambian cada temporada.

## W3 – Pestaña Ajustes (RF-40, RF-44, RF-46)
```
┌ Ajustes ─────────────────────────────────────────── 🔒 ─ × ┐
│ Idioma: ◉ Auto (juego: español) ○ EN ○ ES ○ FR ○ DE        │
│         mantén Ctrl der. para ver todo en inglés           │
│ Intervalo de sondeo: [5] s                                 │
│ Ámbito de alertas: ◉ mapa en el que estoy ○ …              │
│ Avisar de: [x] Campamentos [x] Torres [x] Fortalezas y     │
│            garitas [x] Castillos                           │
│            [x] Eternos [x] Alpino azul [x] Verde [x] Rojo  │
│            [x] el mapa en el que estoy [x] clanes que sigo │
│            [ ] solo los clanes que sigo                    │
│ Segundos del aviso emergente: [2.0 s]   [x] Avisos activos │
│ Widget: líneas [5]  opacidad [85]%  texto [1.0x]          │
│         [x] Ventana visible   [x] Posición bloqueada       │
│         (el click-through se pone con clic derecho)        │
│ Fondo del mapa: [x]   Estilo: ◉ Nexus ○ Propio             │
│ Fondo del widget: ◉ Oscuro ○ Transparente                  │
│ Índice de nombres de clan: 9 886 listos · [Descargar]      │
│ URL del índice: [ https://…/gw2_lurker_guilds.zip      ]   │
│ Versión 0.25.7 · Estado: ✔ actualizado                     │
└────────────────────────────────────────────────────────────┘
```
Menú contextual (clic derecho sobre la ventana, mismo orden siempre):
```
┌───────────────────────────┐
│ [x] Bloquear posición     │
│ [ ] Click-through         │
│ ───────────────────────── │
│ Opacidad   [=====|     ]  │
│ ───────────────────────── │
│ Restablecer posición…     │
│ Ajustes…                  │
└───────────────────────────┘
```
La `×` oculta (no destruye) y la cabecera arrastra. El click-through **no** tiene casilla en
Ajustes a propósito (se quitó el 2026-11-01): se sale con el clic derecho o con el keybind de Nexus,
cuyo atajo se muestra tal y como lo tiene guardado Nexus.

## W4 – Pestaña Clanes (RF-20, RF-20a, RF-23, RF-73)
```
┌ Clanes ───────────────────────────────────────────────────┐
│ Clanes que sigo (3/10)                                    │
│  * [YaK] Dolyak Sanctuary                               × │
│  * [DB] Die Barbaren                                    × │
│  - [DG] Die Garde                                       × │
│ ───────────────────────────────────────────────────────── │
│ ▸ Añadir clan                                             │
│    Con algo reclamado ahora:  [YaK] … 2 · 12:31      [+]  │
│    Buscar un clan: [ dolyak            ] [Buscar]         │
│      * [YaK] Dolyak Sanctuary                             │
│        [Dyak] Dolyak Attack                               │
│    9 886 clanes en el índice                              │
└───────────────────────────────────────────────────────────┘
```
`*` = lo sigues, `-` = no (antes eran estrellas, que la fuente no tiene). Al añadir uno, su nombre
se guarda en `settings.json` y el índice aprende el clan. Las sugeridas salen **solo** de claims
vistos en la sesión: nacen vacías y crecen; la cabecera dice desde cuándo se observa.

## W5 – Mapa simplificado (RF-30, RF-30a, RF-30b, RF-31, RF-37)
```
┌ Mapa ─ [Ojos de Aguas ▾] ────────────────── hace 8 s ┐
│ Tipo:  [● Camp] [■ Torre] [◆ Fort] [★ Cast]          │
│ Capas: [x] Dueños   [x] Claims seguidos   [ ] Flips  │
├──────────────────────────────────────────────────────┤
│                          ■[R]                        │
│      ●[V]                    (◆)[R]                  │
│               ●[A]              <TAG>                │
│  ■[V]                              ●[R]              │
│                                                      │
│ Actualizado 14:45:10 · próxima petición en 22 s      │
└──────────────────────────────────────────────────────┘
```
Lectura sin etiquetas: el **tamaño** de la figura crece con `points_tick` (campamento < torre < fortaleza ≈ castillo) y el **contorno doble** marca alto valor (RF-30b), así se distingue lo importante de un vistazo sin leer texto ni depender del color.

Tooltip al pasar el ratón: nombre, tipo, dueño, último flip, claim (guild + hora), puntos por tick y por captura.

Con el filtro por tipo (RF-30a) solo quedan las figuras marcadas; la capa de claims seguidos no se filtra, para no ocultar justo lo que se está siguiendo.

## W6 – Histórico por guild (RF-32..35)
```
┌ Histórico ─ Guild: [Nombre ▾]  Rango: [Últimas 2 h ▾] ┐
│ ⚠ Ruta INFERIDA a partir de claims; no es el recorrido │
│   real de la guild.                                    │
│  Mapa: [Frontera Roja ▾]                               │
│        ②──────▶③                                      │
│       ╱           ╲                                    │
│     ①              ④ ┄ ┄ ┄ ▶⑤   (┄ = hueco de datos)  │
│ ──────────────────────────────────────────────────────│
│ 1 Fortaleza X  14:02  (dur. 25 min)                    │
│ 2 Torre Y      14:27  (dur. 12 min)                    │
│ 5 Campamento Z 15:10  ⌚ tardío                         │
└────────────────────────────────────────────────────────┘
```
Nodos tardíos con contorno punteado; saltos entre mapas con el texto «salto a <mapa>». El nodo
**más reciente** va relleno y con el número en negrita (RF-32): es la respuesta a «¿qué ha reclamado
por última vez?». El historial es **de la sesión** (ADR-015): el rango empieza al arrancar el addon.

## W7 – Banner de actualización (RF-50)
```
┌ WvW Watch ────────────────────────────────────────────┐
│ Actualizado a v0.25.7. Reinicia el juego para aplicar. │
│                                              [ Cerrar ]│
└────────────────────────────────────────────────────────┘
```
Se muestra fijo y visible hasta reiniciar, aunque el jugador no esté en WvW (solo este aviso; ver RF-50).

## W8 – Fallo de carga / modo seguro (RF-51)
```
 WvW Watch en modo seguro: configuración dañada, se usan valores por defecto.
 Copia conservada en settings.json.corrupt-20261031-1415   [ Detalles ]
```

## Pendientes de cierre del boceto

Todos cerrados:

1. ~~Atajo por defecto del widget~~ **Resuelto (2026-11-01):** keybind `GW2LURKER_CLICK_THROUGH`,
   registrado como `Ctrl+Shift+L` pero reasignable; la ventana muestra **el que Nexus tiene guardado**
   (leído de `Nexus/InputBinds.json`), no un literal.
2. ~~Widget vacío~~ **Resuelto:** sin novedades y en click-through no se dibuja; desbloqueado se
   muestra siempre, para poder colocarlo.
3. ~~Paleta y tema~~ **Resuelto (ADR-009):** ventanas heredan el estilo de Nexus con opción de estilo
   propio; colores de equipo clásicos del juego; fondo del widget configurable, semitransparente por
   defecto.
4. ~~Sonido y cooldown por objetivo~~ **Retirados de la UI:** no hay API de audio en `nexus-rs` 0.12
   y el cooldown por objetivo nunca se implementó. Si vuelven, se documentan antes en `01-requirements.md`.
