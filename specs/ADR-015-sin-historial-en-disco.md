# ADR-015 – No hay historial en disco

- **Estado:** Aceptado (decisión del usuario, 2026-11-01). **Sustituye** el almacenamiento descrito
  en ADR-003 y complementa ADR-012.
- **Contexto:** desde ADR-012 el historial es **de la sesión**: al cargar el addon se descarta el de
  la sesión anterior. Aun así se seguían escribiendo los reclamos de los clanes seguidos en
  `history/session-000.jsonl` (tope de 2 MB) y **nadie los leía**: la interfaz dibuja el historial
  que tiene en memoria, y leer el fichero habría sido una segunda fuente de verdad que además se
  borra en el arranque siguiente. El jugador lo zanjó: «borra el json aquel que nadie lee y elimina
  su código».
- **Decisión:** se elimina `crates/store` (formato JSON-lines, rotación y el rastreador de eventos) y
  todo su uso desde el worker. El historial vive **solo en memoria** (`Dashboard::history`, tope de
  2000 filas) mientras el addon está cargado.
- **Consecuencias:**
  - RNF-03 (tope por sesión) y RF-24 (consulta del histórico) quedan cubiertos por la memoria, no
    por el disco.
  - Se pierde poder inspeccionar una sesión pasada. Si hiciera falta, se añadirá un botón de
    *exportar* que escriba a petición del jugador, no un registro permanente.
  - Los nombres de clan **sí** se guardan en disco (`guild-names.json`, 0.24.0): son datos estáticos
    que cambian poco y ahorran peticiones, no un historial.
- **Alternativas descartadas:** mantener el fichero «por si acaso» (código muerto que confunde y
  ocupa disco); leerlo al arrancar (contradice ADR-012).
