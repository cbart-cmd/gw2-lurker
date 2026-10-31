# ADR-013 – Objetivo de tamaño de la DLL: 2,2 MB

- **Estado:** aceptado
- **Fecha:** 2026-10-31
- **Sustituye parcialmente a:** ADR-008 (que queda vigente en todo lo demás: perfil release y
  `panic="unwind"`).
- **Decisión:** el objetivo de la DLL release pasa de **1,5 MB a 2,2 MB** (`2 306 867` bytes). El
  límite duro de CI sigue en **2,5 MB** (`2 621 440`).

## Contexto

ADR-008 fijó 1,5 MB antes de conocer el coste real del addon, dejándolo «a recalibrar con la
medición del spike T-01». La medición llegó, y con ella el addon:

| Versión | Bytes | Qué entró |
|---|---|---|
| 0.1.0 | 1 053 696 | `nexus` (log, mumble, rtapi) + ImGui, sin HTTP |
| 0.4.0 | 1 455 616 | transporte HTTP, idioma, ajustes, ventana de estado |
| 0.9.0 | 1 547 776 | notas por arcdps/extras, vistas de claims y avisos |

Con 1,5 MB ya estábamos por encima en 0.8.x y quedaban por delante las vistas de clanes,
historial, mapa y ruta: no cabían.

## Por qué 2,2 y no más

- **2,5 MB es el techo práctico**: es lo que el límite duro ya aceptaba, y por encima se
  dispara el tiempo de carga del addon en Nexus, que compila/mapea la DLL al arrancar el juego.
- **1,9 MB no daba margen real**: las cuatro vistas que faltan (clanes, historial, mapa, ruta)
  más sus traducciones se estiman en 250–350 KB.
- **2,2 MB deja ~190 KB de aire** bajo el límite, suficiente para terminar el MVP sin volver a
  tocar el presupuesto.

## Alternativas descartadas

1. **Mantener 1,5 MB y recortar.** Se probó lo obvio: quitar la feature `rtapi` de `nexus-rs`
   (ya no se usaba, porque el idioma llega por Unofficial Extras) y borrar código muerto. Efecto
   medido: **ninguno** (1 547 776 → 1 548 288 bytes, variación de disposición con LTO). Lo que
   pesa es ImGui + `nexus` + el parser de JSON, que no son recortables.
2. **Cargar las vistas como datos externos** (glifos, textos) para adelgazar el binario. Añade
   E/S y un modo de fallo nuevo a cambio de unos pocos KB: peor negocio.
3. **Partir en dos DLLs.** Nexus carga un addon por DLL; partir obliga a un protocolo entre
   ambas y multiplica los modos de fallo en el juego.

## Consecuencias

- `scripts/check_size.sh` avisa por encima de **2 306 867** bytes y falla por encima de
  **2 621 440**.
- CI y `specs/04-engineering-standards.md` citan 2,2 MB.
- Sigue siendo obligatorio mirar el tamaño al publicar: con 2,2 MB el margen bajo el límite es
  del 8 %, no del 40 %.
