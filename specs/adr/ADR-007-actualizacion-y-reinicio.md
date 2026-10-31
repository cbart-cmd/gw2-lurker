# ADR-007 – Actualizaciones gestionadas por Nexus, con aviso de reinicio

- **Estado:** Propuesto (verificar en T-01 y T-25)
- **Contexto:** Nexus ofrece `UpdateProvider` (None, Raidcore, GitHub, Direct, manual) y el flag `DisableHotloading`, que impide descargar el addon en caliente y exige reinicio si se actualiza. La carga, descarga y validación de DLL las hace Nexus; el addon no controla ese proceso.
- **Decisión:**
  1. Publicar en GitHub Releases y declarar `UpdateProvider::GitHub` con el enlace del repositorio, para que Nexus gestione la actualización.
  2. Activar `AddonFlags::DisableHotloading`: no se descarga el addon en caliente.
  3. Aviso propio: al cargar, el addon guarda la huella (SHA-256) y la versión de su propia DLL; un hilo comprueba periódicamente (cada 60 s, fuera del render) si el fichero en disco cambió. Si cambió, muestra el banner W7 "Actualizado a vX.Y.Z, reinicia el juego".
- **Alternativas descartadas:** hot-reload (más superficie de fallo, sin beneficio claro); comprobar versiones contra GitHub desde el addon (añade dependencia de red ajena a la API de GW2; requeriría enmendar la constitución).
- **A verificar:** cómo y dónde deja Nexus la DLL nueva (¿sustituye el fichero o usa un fichero temporal?). Si no sustituye mientras el juego la tiene cargada, el aviso debe basarse en el fichero que Nexus deje para el siguiente arranque. Spike T-25.
- **Consecuencias:** el aviso de reinicio es una cortesía y no una garantía; la actualización efectiva sigue siendo de Nexus.
