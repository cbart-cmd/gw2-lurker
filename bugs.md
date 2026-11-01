# Bugs

Bandeja de entrada de fallos. **No se borra**: cada bug se queda con su marca. `[x]` cuando creo que
está arreglado, con la versión en la que sale el arreglo, para que el jugador pueda comprobarlo y
tacharlo del todo si quiere.

Formato:

```
## [x] Título corto (fecha en que se reporta)
- Qué pasa, cómo se reproduce, qué se esperaba.
- **Hecho (x.y.z):** qué se cambió y dónde.
- **Pendiente:** qué falta (si falta algo).
```

Estado: `[ ]` pendiente · `[x]` arreglado según el addon (a falta de que el jugador lo confirme).

> **2026-10-31, 23:40:** los arreglos de la 0.22.10 **no llegaron al juego**: Nexus sustituye la DLL
> instalada en cada arranque por la publicada en `gw2addonsreleases`, que era la 0.22.9. Todo esto
> sale junto en la **0.23.1**, que sí se publica.

## [ ] «ha dejado de recoger quién reclama» 
cuando pasa un rato de varios munutos, no recoge información de quien reclama. además, parece tener una cache porque entré en el juiego más taerde y el mapa pomia que YaK tenia reclamado un campa  que en verdad era de otro color. a los 10 mintos deja de funcionar
- **Evidencia (2026-10-31, 23:20, con la 0.22.9 y el juego abierto):** el MumbleLink dice
  `shard_id = 2147483649` (`0x80000001`), `instance = 0`, `team_color_id = 9`, y el campo `name`
  lleva «Guild Wars 2» mientras el personaje (`Jhoin`) está en `identity`. Ese `shard_id` no
  aparece en `all_worlds` de ninguna de las nueve partidas, así que la detección por Mumble **no
  puede funcionar**; el log lo deja escrito cada 15 minutos. La partida vigilada (`1-1`, NA) sí era
  la suya: su clan (`YaK · Dolyak Sanctuary`) tiene un reclamo ahí (`96-38`, azul).
- **Hecho (0.23.1):** el reclamo caducado desaparece del mapa y de las listas: la API **no** borra
  `claimed_by` al cambiar el color, solo un reclamo nuevo lo sustituye, así que ahora se descarta
  cuando `last_flipped` es posterior a `claimed_at`. Eso era el «clan en un campa de otro color».
- **Hecho (0.23.1):** el sondeo pide la partida del equipo que va a vigilar, no la del guardado
  (con la detección cacheada un cuarto de hora, miraba otra partida 14 de cada 15 minutos). Y el
  equipo del personaje se guarda con su nombre: con «Guild Wars 2» como clave, el equipo se
  buscaba bajo una entrada que no era el personaje.
- **Causa encontrada (2026-11-01, 0.24.1):** la partida se queda **congelada en la API de ArenaNet**.
  Medido con `curl`: `1-1` devolvió 20 minutos la misma copia, puntos incluidos, mientras las otras
  ocho se movían. El addon sondeaba cada 5,2 s. Ahora el widget lo avisa en naranja a partir de 5
  minutos (`ArenaNet's API has not published any change…`), así que «no pasa nada» deja de parecer
  «el addon está parado». Ver la entrada «deja de registrar eventos a los ~11 minutos».
- **Pendiente:** los `sondeo N: … último reclamo hace X min` del log dicen que entre las 22:02 y las
  23:04 la API de esa partida no publicó **ningún** reclamo y luego llegaron todos de golpe. Si con
  la 0.22.10 vuelve a pararse a los 10 minutos, esas líneas (cada 10 sondeos) dicen si el addon
  sigue sondeando; pegarlas aquí resuelve la duda en una lectura.

## [x] no puiedo eliminar clanes 
pulsar "x" al lado de un clan al que sigo no lo borra. deberia preguntar si estas seguro, y eliminarlo de clanes que sigues
- **Hecho (0.23.1):** la `×` se dibujaba **después** de una fila que ocupa todo el ancho de la
  pestaña, así que caía fuera de la ventana y no se podía pulsar. Ahora el hueco se reserva primero,
  como el `×` de la cabecera. La pregunta sigue siendo «¿Quitar a *nombre* de los clanes que sigues?»
  con `Sí, quítalo` / `No, seguir siguiéndolo`. Además la pestaña *Clanes* y el filtro de avisos
  leen la configuración viva: la estrella y las listas cambian en el mismo momento.
- **Pendiente:** que el jugador lo pruebe en el juego.

## [x] click thoufgh no hace nada 
version 0.22.6. bnoton derecho + click trghough en el widget no hace nada. deberia poderse usar aunque se haya puesto cliq thoufgh en opciones (que sí fubciona)
- **Hecho (0.23.1):** dos fallos. (1) `click_through` era el único campo que el cromo de la ventana
  no escribía de vuelta, así que la marca del menú del clic derecho se tiraba al final del
  fotograma. (2) La comprobación del clic derecho leía la configuración **del arranque**, no la
  viva: activar el click-through en Ajustes no lo veía nunca. Ahora lee la copia editable y el
  rectángulo se resuelve igual que al dibujar.
- **Pendiente:** que el jugador lo pruebe (clic derecho sobre el widget con click-through activado
  desde Ajustes).

## [x] bujscador de clanes no funciopna 
ejemplo, pongo europa y busco dolyak  y dice uqe no existe. pero sí exxiste, es mi clan y de hehco lo estoy siguiendo como favorito. tengo el clan en na y en eu
- **Comprobado con el API (2026-10-31):** `/v2/guild/search?name=dolyak` devuelve **un** clan, y no
  es el del jugador: devuelve `F0CE8616 · [DYAK] Dolyak`, mientras el suyo es
  `6C45F3C8 · [YaK] Dolyak Sanctuary`. El buscador del API va por el principio de una palabra del
  nombre, así que encontrar «Dolyak» no es encontrar «Dolyak Sanctuary».
- **Hecho (0.23.1):** además de los resultados del API, el buscador mira **los clanes que ya
  sigues** por nombre y tag, sin red: «dolyak» o «yak» enseñan `[YaK] Dolyak Sanctuary · ya lo
  sigues` encima de los resultados, y los resultados que ya sigues llevan ★ (con el clan en NA y en
  EU salen las dos entradas, que es lo que se pidió). Si el API no encuentra nada más, el mensaje lo
  dice en vez de «no existe».
- **Pendiente:** que el jugador lo pruebe en el juego.

## [x] borrar el json que nadie lee (2026-11-01)
1.- borra el json aquel que nadie lee y elimina su codigo.
- **Hecho (0.24.0):** se va `crates/store` entero: el `session-000.jsonl` no lo leía nadie (la UI
  usa el historial en memoria y ADR-012 borra el fichero al arrancar). También se borra el fichero
  que hubiera en la carpeta del addon.

## [x] nombres de clan en el Bastión y en todas partes (2026-11-01)
2.- en el Bastión los clanes salen con su id largo; si no sabes el nombre, búscalo y caches al settings.
- **Hecho (0.24.0):** `names.rs` guarda `id → [TAG] Nombre` en `guild-names.json` (fuera de
  `settings.json` a propósito: puede tener miles de entradas y `settings.json` se reescribe cada
  vez que se mueve una ventana). Se carga al arrancar y se publica en el dashboard, así que el
  Bastión (donde no se sondea) enseña nombres desde el primer fotograma. Un nombre pedido una vez
  no se vuelve a pedir nunca.

## [x] los segundos del aviso se pierden al reiniciar (2026-11-01)
3.- los segundos de avisos emergentes se pierden en cada reinicio.
- **Hecho (0.24.0):** el saneado del fichero recortaba a **5 s** mientras el control ofrecía 20:
  cualquier valor mayor se guardaba como 5. Ahora el tope es el mismo (`POPUP_SECS_MAX` = 20) y hay
  test con 12 s.

## [x] el atajo del click-through está mal (2026-11-01)
4.- si pulso clickthrough dice que pulse una combinación que no es la correcta.
- **Hecho (0.24.0):** el texto era el literal «CTRL+SHIFT+L». Ahora `keybinds.rs` lee
  `addons/Nexus/InputBinds.json` y enseña lo que Nexus tiene guardado (en tu fichero:
  `Ctrl+Shift` + código 39, que es «;»). Si no se puede leer, se dice el identificador
  `GW2LURKER_CLICK_THROUGH` en vez de inventar una tecla.

## [x] quitar la casilla de click-through de la ventana (2026-11-01)
5.- en widget, ajustes, «ventanas» elimina la opción de click-through; solo el botón derecho.
- **Hecho (0.24.0):** la casilla desaparece de Ajustes → Ventanas; queda el clic derecho sobre el
  widget (y el atajo que tengas en Nexus).

## [x] buscador de clanes: hace falta un índice (2026-11-01)
6.- el buscador no funciona; extrae a un script para probar: «a new» debe encontrar «A new name…»;
    «DIE» debe encontrar varios.
- **Medido (2026-11-01):** `/v2/guild/search?name=` devuelve **un solo** clan y solo cuando el
  texto **empieza** su nombre: `die` → *Die*, `a new` → *A New*, `a ne` → nada, `DIE anmutig` →
  nada. No hay forma de pedirle «todos los que contienen X».
- **Hecho (0.24.0):** la búsqueda real es por **subcadena** sobre la caché de nombres
  (`guild-names.json`), instantánea y sin red; el API añade una coincidencia más. `scripts/guild-search.ps1`
  busca fuera del juego y **construye** el índice (`-BuildIndex -Region eu`), que es lo que hace
  que «DIE» encuentre *Die Quaggans*, *Die Garde*, etc.
- **Pendiente:** el índice hay que construirlo una vez (el script tarda ~10-20 min para una
  región); después se reutiliza para siempre.

## [ ] deja de registrar eventos a los ~11 minutos (2026-11-01)
«como verás en image-6.png, a los 11 minutos deja de registrar eventos. puedes ver en la pantalla que lleva 7 minutos sin encontrar eventos, que he reclamado con DDK el campamento y que no sale pese a que tengo puesto que siga clanes favoritos y lo que pase en el mapa en el que estoy. y se ve que detecta correctamente que estoy en alpino azul, en el equipo invernadero de lutgardis.»
- **Medido (2026-11-01, 00:23-00:30, con el juego cerrado y `curl`):** el addon **sí** sondea: el
  log tiene `sondeo 180` a las 00:13:50 y `sondeo 190` a las 00:14:42 (5,2 s por vuelta) y la
  cabecera decía «ÚLTIMA ACTUALIZACIÓN 00:15:20». Lo que está parado es **la partida `1-1` en la
  API de ArenaNet**: entre las 00:23 y las 00:30 devolvía siempre lo mismo, con el reclamo más
  nuevo de las 23:07:58 UTC (hace 20 min) **y los puntos también congelados**
  (`67568/52264/35741` en cuatro consultas separadas). Las otras ocho partidas sí se movían
  (`2-3` a 23:21, `1-4` a 23:25). Ni la caché del CDN (misma respuesta con `?cb=`) ni el cliente
  HTTP: es el backend.
- Comprobado además: el equipo `11011` (Invernadero de Lutgardis, «Controlado por» en el panel del
  juego) **solo** está en `1-1`, y el clan `DDK` (`ACBC90A5`) no aparece reclamando en **ninguna**
  de las nueve, porque la partida en la que reclamó es esa y su copia está congelada.
- **Hecho (0.24.1):** el widget lo dice en naranja cuando el dato no cambia:
  «La API de ArenaNet no publica cambios de esta partida desde hace N min; el addon sigue
  sondeando cada X s». Se calcula con una huella de la partida (puntos + objetivos), así que
  distingue «no pasa nada» de «el addon está parado».
- **Pendiente:** que el jugador lo compruebe la próxima vez; si el aviso sale, la culpa es de la
  API y se puede ver a mano en `https://api.guildwars2.com/v2/wvw/matches/1-1`.

## [x] avisos de ruinas, nombres de clan e índice de búsqueda (2026-11-01)
«no me avises cuando alguien capture ruinas, solo quiero campamentos, torres, fortalezas / garri / piedraniebla. el json de nombres de clan deberíamos incorporarlo a la distribución… que lo descargue la primera vez que ejecutes el addon, y a petición con la opción de poner otra URL… cuando dentro del addon se añada un nuevo clan, si ese clan no está en el json, que busque el nombre para ese clan concreto y lo añada. importante no estar leyendo guild names.json todo el rato: json de nombres es solo una caché para buscar clanes a añadir por texto; en followed, aparte del id, guardar el nombre.»
- **Hecho (0.25.0):** las ruinas (y los campamentos mercenarios) **nunca** avisan: `AlertFilter`
  lleva ahora una casilla por estructura reclamable (campamento, torre, fortaleza/garita, castillo)
  y descarta el resto. Un objetivo de tipo desconocido sí avisa, para no quedarse mudo mientras
  llega la tabla de objetivos.
- **Hecho (0.25.0):** los nombres resueltos se guardan en `settings.json` (`guild_names`) y
  `guild-names.json` deja de cargarse al arrancar: lo lee el hilo del buscador, lo filtra y lo
  suelta. Ya no se clonan 9 900 entradas en cada sondeo.
- **Hecho (0.25.0):** el índice se publica como `gw2_lurker_guilds.zip` + `.md5` en
  `gw2addonreleases` (9 886 nombres, 297 KB) y el addon lo descarga la primera vez o cuando pulsas
  *Descargar el índice* en Ajustes, con la URL configurable.
- **Hecho (0.25.0):** al empezar a seguir un clan que no estaba en el índice, su nombre se resuelve
  por API y se añade al fichero.
- **Pendiente:** que el jugador confirme en el juego que las ruinas ya no avisan y que el buscador
  encuentra «a new» y «DIE» sin esperar a nada.

## [x] el juego se ha congelado y ha hecho un crash (2026-11-01, 00:40-00:41)
«estaba probando y se me ha congelado el juego a las 00:40 o 00:41, ha hecho un crash»
- **Causa (0.24.1, mía):** el aviso naranja de «partida congelada» que añadí en la 0.24.1 pedía el
  candado de la configuración (`crate::runtime()`) **dentro** de una función que ya lo tenía
  (`draw_health` recibe `config: &mut Config`). `std::sync::Mutex` no es reentrante: el hilo de
  render se quedaba esperándose a sí mismo y el juego se congelaba. El log acaba en `sondeo 50` sin
  pánico y el `arcdps_crash` del directorio es del 30/10, así que no fue una excepción: fue un
  bloqueo. Es la misma trampa del 2026-10-31, esta vez en otro sitio.
- **Hecho (0.24.2):** el aviso lee el intervalo de la configuración que ya trae la función. Revisado
  el resto de `ui/mod.rs`: las demás llamadas a `crate::runtime()` están **fuera** del candado
  (callbacks del menú, pestaña «Clanes» y `draw_openers`).
- **Pendiente:** que el jugador confirme que ya no se cuelga al abrir el widget con la 0.24.2.

## [ ] no coge nombre de clan al añadir
busco clan, lo encuentro, clico para añadir en el bastion, y en vez de guardar id + nobmre y mostrar el nombre, guarda el id y hasta que no llegas a uin mapa de verdad y pasan unos segundos no vbusca le nombre. el comportamiento debe ser el mismo en cuakquier mapa, si añado un clan se guarda id + nombre y SIEMPRE en todas partes SE MUETERA EL NOMBRE DEL CLAN y no el id
![alt text](image.png)
## [x] un «?» en los avisos en lugar de una flecha (2026-11-01)
«en la ventana de avisos, a veces pone un “?” que creo que quería ser una flecha o similares. pasa en varios avisos»
- **Causa (0.25.2):** el atlas de fuentes de Nexus solo trae latín básico y latino-1. La línea del
  cambio de dueño usaba la flecha Unicode U+2192, así que **todas** las de tipo *flip* enseñaban «?»
  donde iba la flecha. En el mismo caso estaban los puntos suspensivos tipográficos (Buscando… /
  Descargando el índice…), las estrellas de la pestaña Clanes y las comillas alemanas „ “. Se
  comprobó que sí se dibujan `·` y `×` (latino-1), que acota el problema a lo que queda fuera de
  latino-1.
- **Hecho (0.25.2):** las flechas pasan a `->`, los puntos suspensivos a `...`, las estrellas a
  `*`/`-` y las comillas alemanas a « », y el «estás aquí» del mapa actual se dice con la clave
  `you are here` en vez de una flecha. Nuevo test `rf44_the_table_stays_inside_latin_1`: falla si
  alguien mete en la tabla de textos un carácter fuera de latino-1, para que no vuelva.
- **Pendiente:** verlo en el juego. Los nombres de clan que devuelve el API sí pueden traer cirílico
  o emoji: esos seguirán saliendo como «?» porque no hay glifo en la fuente, y el addon no puede
  añadir un atlas propio sin meter una fuente en la DLL.
## [x] en el buscador, el primer clic no añade el clan (2026-11-01)
«me da la sensación que tras buscar, el primer clic que hago al clan no lo añade, tengo que darle otra vez»
- **No era sensación.** Clicar una fila de resultados **desactiva el campo de texto**, y eso se
  estaba usando como señal de «busca ya»: en el mismo fotograma del clic se lanzaba otra búsqueda,
  el estado pasaba a *buscando* y la lista se sustituía por «Buscando...»; el clic caía en el vacío.
  El segundo clic funcionaba porque el campo ya estaba sin foco.
- **Hecho (0.25.4):** perder el foco ya no busca (busca el debounce mientras escribes, o Enter /
  botón). Y `State::Searching` **conserva los resultados anteriores**, así que la lista sigue en
  pantalla —y clicable— mientras llega la nueva. Test `t44_the_results_on_screen_survive_a_new_search`.
- **Pendiente:** confirmarlo en el juego.

## [x] `settings.json` guarda todos los nombres de clan y crece sin freno (2026-11-01)
«no me gusta que guardes todos los nombres en settings.json. tienen que estar los de los clanes que sigo, y los que han reclamado en la sesión, pero no puede crecer indefinidamente»
- **Hecho (0.25.6):** `settings.json` guarda **solo los clanes que sigues** (10 como mucho,
  `MAX_FOLLOWED`), que son los que hay que nombrar fuera de WvW. Los de los clanes que han reclamado
  algo **en esta sesión** van a **`temp.json`**, que se **borra y regenera en cada arranque** (tope de
  500 entradas): no se guardan «para siempre» ni se quedan en memoria. Al dejar de seguir un clan se
  borra su nombre de los ajustes, y al cargar se descarta cualquiera que no sea de un clan seguido
  (tu fichero pasó de **51 nombres a 8**).
- **Pendiente:** verlo tras reiniciar el juego.

## [x] la descarga del índice falla con «no existe» (2026-11-01)
«al intentar bajar el zip para probar que descarga, descomprime y machaca el guild-names.json, me da error»
- **Causa: una errata mía.** La URL por defecto apuntaba a `gw2addonreleases` en vez de
  `gw2addonsreleases` (le faltaba la «s» de *addons*). GitHub respondía **404** y el addon lo
  traducía como «no existe». Lo comprobé con el propio cliente HTTP del addon (`ffi::http::get`):
  la URL mala daba 404 y la buena 200 con 297 532 bytes; con `curl`/`Invoke-WebRequest` no se veía
  porque escribía la buena a mano.
- **Hecho (0.25.7):** URL corregida, y la errata se **repara sola** en `settings.json` al cargar.
  Tres tests nuevos (`rf20c_the_default_index_url_is_the_distribution_repo`,
  `rf20c_the_broken_default_url_repairs_itself` y la prueba de red `live_the_index_zip_is_downloadable`).
  El 404 ya se explica en la ventana, y la descarga lleva un `?v=<sello>` contra la caché del CDN.
- **Pendiente:** pulsar *Descargar el índice* en el juego.

## [x] en el mapa, el hover tapa con el nombre del sitio (2026-11-01)
«no necesito “Palacio del Águila Pescadora” en el mapa porque ya lo sé y me tapa el resto… lo de alto valor tampoco lo necesito, eso iría lo último: sería azul * yak… * 12:31 * alto valor»
- **Hecho (0.25.9):** la línea del hover de la pestaña *Mapa* ya no lleva el nombre del objetivo y el
  orden es dueño → clan → hora → **Alto valor al final**:
  `Azul  ·  * [YaK] Dolyak Sanctuary  ·  12:31  ·  Alto valor`. Sin claim, solo `Sin reclamar`.
  Está en `ui::map::detail_line`; los nombres siguen en la pestaña *Clanes* (historial y ruta).
- **Pendiente:** verlo en el juego.

## [x] «con algo reclamado ahora» enseña el id y no el nombre (2026-11-01)
«widget, "con algo reclamado ahora" no muestra nombres sino el id. debería ir al json a buscar los nombres.»
- **Causa:** a esa lista se le pasaba **solo** el mapa de `settings.json`, donde únicamente están los
  clanes que ya sigues; los de la sesión viven en `temp.json`. Como no estaban, `label()` caía al id.
  El nombre ya estaba cargado: **no hace falta leer el índice `guild-names.json` en el render**, y no
  se hace (RF-20b).
- **Hecho (0.25.10):** la lista recibe el mapa que se dibuja (ajustes **+ `temp.json`**) y se ve
  `[PEST] Please excuse the silly tactics`. Al añadir uno desde ahí, su nombre se guarda en los ajustes
  y se le enseña al índice, así que la próxima búsqueda por texto lo encuentra.
- **Pendiente:** verlo en el juego.
