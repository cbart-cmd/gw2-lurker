# ADR-009 – Apariencia: estilo heredado, colores de equipo fijos

- **Estado:** Aceptado (decisión del usuario)
- **Decisión:**
  1. Ventanas con el estilo de Nexus por defecto; opción de tema propio en configuración.
  2. Colores de equipo clásicos (rojo/azul/verde) fijos, sin modo accesible en v1.
  3. Fondo del widget configurable, semitransparente por defecto.
- **Motivo:** coherencia con el resto de addons sin coste, y colores reconocibles al instante para jugadores de WvW.
- **Consecuencias:** al no haber modo accesible, la información debe viajar también por forma y texto (RF-19, RF-62). Si se pide un modo daltónico más adelante, se añade como tema de colores sin tocar el resto. El tema propio debe aplicarse con push/pop de estilo acotado a las ventanas del addon, para no alterar el estilo global de imgui.
