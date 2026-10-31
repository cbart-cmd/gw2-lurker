# ADR-011 – Límites del mapa por `continent_rect` y fondo opcional con teselas

- **Estado:** Propuesto (pendiente de T-01 y de revisar la política de ArenaNet)
- **Contexto:** `coord` de los objetivos está en coordenadas de continente (Las Brumas, 16384 × 16384), las mismas que usa el servicio de teselas oficial. `/v2/maps` aporta datos por mapa. Las teselas no se regeneran desde Living World Season 4, aunque los mapas de WvW son contenido antiguo.
- **Decisión:**
  1. Límites del mapa a partir del rectángulo de continente de `/v2/maps`, con respaldo por mínimo/máximo de objetivos.
  2. Fondo con teselas y iconos oficiales, **opcional y desactivado por defecto** (RF-37, RF-38): descarga en ejecución, caché en disco con límite, nunca dentro de la DLL.
  3. La v1 sigue sin texturas obligatorias (ADR-002).
- **Alternativas descartadas:** empaquetar imágenes en la DLL (tamaño y licencia); copiar el enfoque de terceros sin conocer su código ni su licencia.
- **Consecuencias:** exige un cargador de texturas operativo en Nexus y una caché; si T-01 o la política lo impiden, el fondo se descarta sin afectar al resto.
