# 05 – Bocetos de interfaz (wireframes)

Boceto de estructura, no de estilo visual. Leyenda: `[R]` rojo, `[A]` azul, `[V]` verde; formas: `●` campamento, `■` torre, `◆` fortaleza, `★` castillo. El color nunca es la única señal (RF-19).

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

## W2 – Primer uso: elegir mundo (RF-03a)
```
┌ WvW Watch – Configuración inicial ────┐
│ Elige tu mundo:                       │
│ [ buscar…                        🔍 ] │
│  ○ Mundo A      ○ Mundo B             │
│  ○ Mundo C      ○ …                   │
│                       [ Guardar ]     │
│ Tu equipo se detecta cada semana.     │
└───────────────────────────────────────┘
```

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

## W4 – Guilds seguidas (RF-20, RF-23)
```
┌ Guilds seguidas ──────────────────────────────────┐
│ Añadir: [ nombre o ID…        ] [Buscar]          │
│  Resultados: ○ Nombre [TAG]  ○ Nombre2 [TG2]      │
├───────────────────────────────────────────────────┤
│ ▸ Nombre [TAG]            2 objetivos ahora   [✕] │
│     ◆ Fortaleza X – Frontera Roja – desde 14:20   │
│     ■ Torre Y     – Ojos de Aguas – desde 14:41   │
│ ▸ Otra [OTR]              0 objetivos ahora   [✕] │
└───────────────────────────────────────────────────┘
```

## W5 – Mapa simplificado (RF-30, RF-31)
```
┌ Mapa ─ [Ojos ▾] [Frontera R] [Frontera A] [Frontera V] ┐
│                      ■[R]                              │
│        ●[V]                ◆[R]★                       │
│                ●[A]             <TAG>                  │
│   ■[V]                   ●[R]                          │
│ Capas: [x] Dueños  [x] Claims seguidos  [ ] Flips 10m  │
│ Actualizado 14:45:10 (hace 8 s)                        │
└────────────────────────────────────────────────────────┘
```
Tooltip al pasar el ratón: nombre, tipo, dueño, último flip, claim (guild + hora).

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
Nodos tardíos con contorno punteado; saltos entre mapas con indicador `→ Ojos de Aguas`.

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
