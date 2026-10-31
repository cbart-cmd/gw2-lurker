# 05 – Bocetos de interfaz (wireframes)

Boceto de estructura, no de estilo visual. Leyenda: `[R]` rojo, `[A]` azul, `[V]` verde; formas: `●` campamento, `■` torre, `◆` fortaleza, `★` castillo; **contorno doble** `(◆)` = objetivo de alto valor. El color nunca es la única señal (RF-19).

## W1 – Widget fijo de alertas (RF-16..19)

Desbloqueado (movible/redimensionable):
```
┌ WvW Watch ───────────── 🔒 ≡ ┐   🔒 bloquear   ≡ abrir log
│ ■ Torre Durios perdida [R]   │
│   Ojos de Aguas · hace 12 s  │
│ ◆ Claim <GUILD> en Fortaleza │
│   Frontera Roja · hace 3 min │
│ ● Campamento X perdido [A]   │
│   Frontera Verde · hace 9 min│
└──────────────────────────────┘
```
Bloqueado (sin cabecera, sin capturar clics, opacidad configurable):
```
 ■ Torre Durios perdida [R]  · 12 s
 ◆ Claim <GUILD> Fortaleza   · 3 min
```
Estados: sin alertas y bloqueado → widget oculto (opcional: texto tenue "Sin novedades"); datos obsoletos → línea fija `⚠ Datos obsoletos desde 14:02`.

## W2 – Primer uso: elegir equipo (RF-03a, RF-03g)
```
┌ WvW Watch – Configuración inicial ───────────────────┐
│ Elige tu equipo (match 2-2 · EU):                    │
│ [ buscar equipo…                              🔍 ]   │
│  ◉ Great House Aviary (12014)                        │
│  ○ Bava Nisos      (12015)                           │
│  ○ Bloodstone Gulch (12007)                          │
│                                                      │
│ Comprobación: eres AZUL; tus rivales son             │
│ Bava Nisos (rojo) y Bloodstone Gulch (verde).        │
│                        [ Correcto ]   [ Cambiar ]    │
└──────────────────────────────────────────────────────┘
```
El nombre sale de `data/wvw-teams.json`; si el id no está en la tabla se muestra solo `(12014)`. La identidad es el id: los nombres y los colores cambian cada temporada.

## W3 – Configuración (RF-40)
```
┌ WvW Watch ─ Configuración ────────────────────────┐
│ [General] [Alertas] [Widget] [Guilds] [Datos]     │
├───────────────────────────────────────────────────┤
│ Alertas:  ◉ Mi mapa  ○ Todos los mapas            │
│ Sonido: [x]   Cooldown por objetivo: [ 60 ] s     │
│ Widget: Líneas [5]  Vida [60] s  Opacidad [80]%   │
│         Posición/tamaño: (arrastra el widget)     │
│         [x] Bloqueado    Atajo: [ ... ]           │
│ Polling: [30] s     Historial: [30] días          │
│ Estilo ventanas: ◉ Nexus  ○ Propio                │
│ Fondo widget: ○ Transparente ◉ Semitransparente   │
│ Versión 0.1.0  ·  Estado: ✔ actualizado           │
└───────────────────────────────────────────────────┘
```

## W4 – Guilds seguidas y sugeridas (RF-20, RF-20a, RF-23)
```
┌ Guilds seguidas ─────────────────────────────────────┐
│ Añadir: [ nombre o ID…          ] [Buscar]           │
│  Resultados: ○ Nombre [TAG]   ○ Nombre2 [TG2]        │
├──────────────────────────────────────────────────────┤
│ Sugeridas (claims vistos desde 31/10 12:00)          │
│  YAK [YAK]  6 claims · este mapa  · últ. 14:41  [+]  │
│  OTR [OTR]  3 claims · este mapa  · últ. 14:02  [+]  │
│  XYZ [XYZ]  9 claims · otros mapas · últ. 13:50  [+] │
├──────────────────────────────────────────────────────┤
│ ▸ Nombre [TAG]            2 objetivos ahora     [✕]  │
│     ◆ Fortaleza X – Frontera Roja – desde 14:20      │
│     ■ Torre Y     – Ojos de Aguas – desde 14:41      │
│ ▸ Otra [OTR]              0 objetivos ahora     [✕]  │
└──────────────────────────────────────────────────────┘
```
La lista de sugeridas sale **solo** de claims que el addon ha visto (RF-20a): nace vacía y crece. Se ordena por claims en el mapa actual y luego por el resto, con la fecha de inicio de observación siempre visible para no aparentar un ranking global.

## W5 – Mapa simplificado (RF-30, RF-30a, RF-30b, RF-31)
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
Nodos tardíos con contorno punteado; saltos entre mapas con indicador `→ Ojos de Aguas`. El nodo **más reciente** va relleno y con el número en negrita (RF-32): es la respuesta a "¿qué ha reclamado por última vez?".

## W7 – Banner de actualización (RF-50)
```
┌ WvW Watch ────────────────────────────────────────────┐
│ ⟳ Actualizado a v0.2.0. Reinicia el juego para aplicar.│
│                                              [ Cerrar ]│
└────────────────────────────────────────────────────────┘
```
Se muestra fijo y visible hasta reiniciar, aunque el jugador no esté en WvW (solo este aviso; ver RF-50).

## W8 – Fallo de carga / modo seguro (RF-51)
```
 ⚠ WvW Watch en modo seguro: configuración dañada, se usan valores por defecto.
   Copia conservada en config.json.corrupt-20261031-1415   [ Detalles ]
```

## Pendientes de cierre del boceto

1. ~~Atajo por defecto del widget~~ **Resuelto:** `Alt+Shift+L` (RF-18). Debe registrarse como keybind gestionado por Nexus, para que el usuario pueda reasignarlo si choca con otro.
2. ~~Widget vacío~~ **Resuelto:** se oculta por completo cuando no hay novedades (RF-16). Opción "Mostrar «Sin novedades»" disponible en configuración, desactivada por defecto. Mientras esté desbloqueado se muestra siempre, para poder colocarlo.
3. ~~Paleta y tema~~ **Resuelto (ADR-009):** ventanas heredan el estilo de Nexus con opción de estilo propio; colores de equipo clásicos del juego; fondo del widget configurable, semitransparente por defecto.

**Todos los pendientes de bocetos están cerrados.**
