# ADR-016 – El índice de nombres de clan se distribuye aparte y no vive en memoria

- **Estado:** Aceptado (petición del jugador, 2026-11-01). Completa ADR-015 (los nombres sí van a
  disco; el historial no) y RF-20b/RF-20c/RF-20d.
- **Contexto:** el buscador de clanes necesitaba un índice propio porque
  `/v2/guild/search?name=` devuelve **un solo** clan y solo si el texto empieza su nombre
  (medido: «die» → *Die*, «a ne» → nada). Se construyó `guild-names.json` con **9 886** nombres
  (675 KB) y en la 0.24.0 se cargaba **entero en memoria al arrancar**, se clonaba en cada sondeo
  para publicarlo con el dashboard y se volvía a escribir cuando aparecía un nombre nuevo. El
  jugador lo notó: «importante no estar leyendo guild-names.json todo el rato».
- **Decisión:**
  1. **`settings.json` es la fuente de los nombres que se dibujan.** `Config::guild_names`
     (`id → "[TAG] Nombre"`) guarda el nombre la primera vez que se resuelve y no se vuelve a
     pedir: los nombres de clan no cambian. Es un mapa pequeño (unos 60 bytes por clan seguido o
     visto) en un fichero que ya se reescribe solo.
  2. **`guild-names.json` es solo el índice de búsqueda.** No se lee al arrancar, no se tiene en
     memoria y no se toca en el render: lo lee el hilo de la búsqueda, lo filtra por subcadena y
     lo suelta.
  3. **Se distribuye como ZIP** (`gw2_lurker_guilds.zip` + `.md5`) en el repositorio de
     distribución, y el addon lo descarga **la primera vez** que no lo encuentra y **a petición**
     desde los ajustes, con URL configurable. Antes de escribir nada se comprueba el MD5
     publicado. Mismo mecanismo que el `data.zip` del addon hermano `gw2-rotation-builds`.
  4. **Al empezar a seguir un clan que no está en el índice**, su nombre se resuelve por API y se
     añade al índice (en un hilo corto, lectura-mezcla-escritura) para que la próxima búsqueda lo
     encuentre.
- **Consecuencias:**
  - El render no paga nada por el índice: ni una lectura, ni un clon de 9 900 entradas.
  - Un clan seguido se ve por su nombre **también fuera de WvW** (Bastión del Armisticio), porque
    el nombre está en los ajustes y no depende de que haya sondeo.
  - El índice puede quedarse viejo sin que nada se rompa: la búsqueda sigue funcionando con lo que
    haya y el API aporta un resultado más.
  - Se añade la dependencia `zip` (2.x, `default-features = false, features = ["deflate"]`), la
    misma caja y la misma feature que el addon hermano. El MD5 se implementa en el propio addon
    (`md5.rs`, ~40 líneas, probado con los vectores del RFC 1321) para no añadir otra caja.
- **Alternativas descartadas:**
  - **Meter el índice en la DLL** (`include_str!`): 675 KB más de DLL para un dato que cambia sin
    recompilar, y el límite de tamaño de RNF-01 lo hace inviable.
  - **Pedirlo al API cada vez**: no existe endpoint de listado masivo (`/v2/guild?ids=` → 404) y
    serían miles de peticiones.
  - **Comprimirlo con LZMA/Zstd**: el ZIP con deflate ya baja de 675 KB a ~300 KB y es lo que
    cualquier herramienta abre.
