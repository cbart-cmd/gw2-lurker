# ADR-002 – Mapa simplificado sin texturas

- **Estado:** Aceptado; ampliado por ADR-011 (límites por `continent_rect` y fondo opcional)
- **Decisión:** dibujar los mapas con primitivas imgui. Posiciones a partir de `coord` de `/v2/wvw/objectives`; límites por mínimo/máximo de coordenadas más margen.
- **Motivo:** evita descargar y cachear teselas, reduce dependencias y coste de render, y cumple el requisito de "mapa simplificado".
- **Consecuencias:** sin geografía real. Se puede añadir un contorno estático por mapa más adelante.
