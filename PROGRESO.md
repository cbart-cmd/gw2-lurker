# Progreso: gw2-lurker

Bitácora para continuar el trabajo, incluso si quien lo retoma no es la misma IA ni la
misma sesión. **Lo más reciente va arriba.** Cada entrada dice qué se pretendía, qué se
hizo, cómo se verificó y qué quedó pendiente.

Reglas:
- Al **empezar** una tarea: añade una entrada `EN CURSO` con el plan en 3–5 líneas.
- **Durante**: actualiza esa entrada con hallazgos, decisiones y callejones sin salida.
- Al **terminar**: cámbiala a `HECHO`, pon los commits y lo que no se pudo verificar.
- No borres entradas antiguas. Si algo se revierte, dilo aquí y por qué.
- Lo que contradiga la spec se propone y se decide, no se entierra aquí.

---

## HECHO – 2026-11-01 · El mapa coloreaba solo los objetivos reclamados (0.25.13)

- Informe del jugador: «Puesto avanzado abrasado» pasó a rojo (aviso `AZUL -> ROJO`, 11:01:41) y el
  mapa seguía en «sin reclamar».
- **No era caché ni retraso del sondeo.** `map::body` deducía el dueño de la fila de `matchup::claims`,
  y `claims` solo emite objetivos con claim vivo (`claim_holds` + `claimed_by`). Un objetivo capturado
  **sin** claim no tiene fila: se pintaba con `UNCLAIMED` y el hover caía al texto «Sin reclamar». En
  el registro, 8 s después del vuelco, llega `[aT] … · Sin reclamar` (`Event::Unclaim`), así que el
  caso reportado era exactamente dueño rojo + claim vacío.
- Arreglo: nuevo `core::matchup::ObjectiveState` + `states(&Match)` — **todos** los objetivos del match
  con `owner`, y `guild`/`claimed_at` solo si el claim sigue vivo. `Dashboard::states`, `Tabbed::states`
  y `map::View::states` lo llevan hasta el dibujo; `detail_line` pasa a recibir `Option<&ObjectiveState>`
  y dice «Rojo · Sin reclamar». `claims` y `history` no cambian: siguen alimentando la pestaña Clanes,
  el ranking y las rutas.
- Tests: `rf30b_an_objective_without_claim_keeps_its_owner` (dueño sí, clan no, y `claims` no cambia),
  `rf30b_every_objective_of_the_fixture_has_an_owner_state` (invariante contra la fixture: 44 objetivos
  y el mismo número de reclamados que `claims`) y
  `rf30b_an_objective_owned_without_claim_keeps_its_colour_on_the_hover`.
- Spec actualizada (RF-30b): el color sale del `owner` del objetivo, **no** del reclamo.
- Publicado en **0.25.13**: commit `7d4f7a8`, DLL MD5 `bfd73176da612973cc4945042a4cb12a`, release
  `gw2-lurker-v0.25.13` en `gw2addonsreleases`. `install.ps1` **no pudo copiarla**: GW2 estaba abierto
  y bloquea la DLL (la instalada era la 0.25.8). Se instalará al cerrar el juego, o Nexus la sustituye
  en el siguiente arranque.
- **No verificado:** falta verlo en el juego con una captura sin reclamar.

## HECHO – 2026-11-01 · Ajustes por defecto de una instalación nueva (0.25.12)

- Petición del jugador: en «avisar de», de fábrica solo **«el mapa en el que estoy»** y **«los clanes
  que sigo»** (con las cuatro estructuras), y el aviso emergente a **15 s**.
- `AlertFilter::default()` deja los cuatro mapas fijos en `false`; `current_map` y `followed` siguen
  en `true`, igual que las cuatro estructuras (`Structures::default`).
- Nuevo `config::POPUP_SECS_DEFAULT = 15.0`; `Config::default()` y el «al encenderlo» de la ventana
  lo usan. Antes el de la ventana era 5 y el de fábrica 2: dos números distintos para lo mismo.
- **Lo guardado no se toca**: `#[serde(default)]` solo rellena lo que falta, así que el fichero del
  jugador conserva sus casillas y sus segundos.
- Tests ajustados a la intención nueva: `rf19a_by_default_only_my_map_and_the_guilds_i_follow` (core)
  y `rf19a_by_default_my_map_and_the_guilds_i_follow` (config). Los tests del worker que no van de
  filtros piden ahora «todos los mapas» con un helper `any_map()`, para no depender del defecto.

## HECHO – 2026-11-01 · Desplegable de clan en la pestaña Mapa (0.25.11)

- Petición del jugador: el desplegable de mapa ocupaba todo el ancho «y no hace falta tanto»; al
  lado quería uno de clan con los que sigue y «justo después los últimos 5 que han reclamado algo»,
  para no tener que ir a la pestaña *Clanes* a buscarles.
- `map::View` lleva ahora `guild` (el elegido, de `guilds::selected_guild`) y `session_guilds`
  (`Dashboard::session_guilds`, ya ordenado por reclamo más reciente). `draw_guild_choice` dibuja el
  combo a 210 px, con `*` en los seguidos; `guild_choices` es la regla pura y tiene test.
- Elegir un clan hace lo mismo que clicarlo en la pestaña *Clanes*: `guilds::select` + llevar el mapa
  al último sitio donde reclamó (`last_map_of`), que es lo que se está pidiendo al elegirlo.
- El combo de mapa pasa a 120 px con `ui.push_item_width`: `arcdps-imgui` 0.8 no tiene
  `ComboBox::width` (leído en `combo_box.rs`), así que el ancho se pone por el item.
- Claves nuevas en la tabla: `Guild` y `No guild`.
- Tests `rf31_the_guild_menu_is_the_followed_ones_and_then_the_latest_five` y
  `rf31_the_guild_menu_without_anything_to_offer_is_empty`.

## HECHO – 2026-11-01 · «Con algo reclamado ahora» enseñaba ids (0.25.10)

- Informe del jugador con captura: la lista de la pestaña *Clanes* mostraba
  `C3484B97-EC4A-E411-AA1F-AC162DAE275E` en vez de `[PEST] Please excuse the silly tactics`, mientras
  la lista de abajo sí los enseñaba.
- **Causa:** a `session_block` se le pasaba `config.guild_names` (solo los clanes **seguidos**, por
  RF-20b) en vez del mapa que se dibuja —ajustes **+ `temp.json`**—. Los de la sesión no están en los
  ajustes, así que `label()` caía al id. No hacía falta leer el índice `guild-names.json` en el
  render: el nombre ya estaba en `temp.json`.
- Arreglo: `followed_section`/`add_section`/`session_block` reciben **dos** mapas, `known` (lo que se
  dibuja) y `saved` (lo que se escribe). Al añadir desde la lista, `remember_name` guarda el nombre en
  los ajustes y devuelve la entrada para el índice. Test `rf20_the_session_list_shows_names_and_not_ids`.
- `ui/mod.rs` se había ido a 803 líneas con el cambio: la línea de estado pasa a `ui/status.rs`
  (`draw_health`, `colour_label`, `region_of*`), 530 + 218.

## HECHO – 2026-11-01 · El hover del mapa, sin nombre y con el alto valor al final (0.25.9)

- Petición del jugador: el nombre del objetivo en el hover del mapa «tapa el resto» y el alto valor
  no hacía falta. Orden pedido: `azul · * yak… · 12:31 · alto valor`.
- `ui::map::detail_line` ya no recibe el `ObjectiveInfo` (era solo para el nombre) y el idioma pasa a
  ser **parámetro**: `i18n::text` del addon llama a `english_override()` → `ffi::keys` → Nexus, y
  **revienta fuera del juego** (`addon api not initialized`). Es la trampa de AGENTS §11.3 y el mismo
  patrón que `alerts::spans`. Los tests nuevos usan `core::i18n::t(Language::Es, …)`.
- Tests `rf30_the_hover_says_who_and_when_and_leaves_the_name_out` (comprueba el orden de las cuatro
  partes y que el nombre no está) y `rf30_the_hover_without_a_claim_only_says_unclaimed`.
- **Susto y lección:** al añadir esta entrada descubrí que las seis anteriores (0.25.4 a 0.25.8 y la
  del espejo `specs/adr/`) **no se habían escrito**: usé `$t.Replace($anchor, …)` con un ancla que
  llevaba backticks dentro de una cadena de PowerShell **de doble comilla**, así que el ancla no
  existía y `Replace` devolvió el texto sin tocar, en silencio. Y con `bugs.md` algo parecido lo
  dejó en **3 entradas de 17**: recuperado de `bbb124e` y repuestas las que faltaban (20 ahora).
  Regla nueva: **en estos dos ficheros se escribe con `edit` (que falla si el ancla no existe) y se
  comprueba el número de `## ` después**, nunca con `Replace` a ciegas.

## HECHO – 2026-11-01 · Fuera el espejo `specs/adr/`

- Petición del jugador: «borra las copias duplicadas de `specs/adr`». Eran **14 ficheros repetidos**
  de `specs/ADR-*.md`, y dos (003 y 012) se habían quedado atrás: un espejo son dos fuentes de
  verdad. Se quedan los canónicos ADR-001…016 en `specs/`.

## HECHO – 2026-11-01 · CI en rojo: advisories de `zip` y `config.rs` con 813 líneas (0.25.8)

- `fmt · clippy · test · loc` fallaba en `check_loc.sh`: `config.rs` con **813 líneas**. Partido en
  `config/mod.rs` (500) + `config/tests.rs` (255).
- `deny · audit` fallaba por **tres RUSTSEC sobre `zip` 2.6.1** (SUID/SGID al extraer y enlaces
  simbólicos fuera del destino). El addon no llama a `extract()`, pero un aviso no se silencia:
  **`zip` 9.2.0**. Coste: MSRV **1.88** y +57 856 B de DLL (2 141 696 B).
- Verificado en local con `cargo deny check` (lo tengo instalado) antes de publicar, y con la prueba
  de red `live_the_published_zip_unpacks_into_the_index` (>9 000 nombres sacados del ZIP publicado).

## HECHO – 2026-11-01 · La descarga del índice fallaba por una errata en la URL (0.25.7)

- `DEFAULT_URL` decía `gw2addonreleases` y el repositorio es `gw2addonsreleases`: 404 de GitHub,
  traducido a «no existe». **Solo se veía con el cliente del addon**, no con `curl` (escribía la
  buena a mano). Ahora la errata se repara sola en `settings.json`, el 404 se explica y hay prueba
  de red permanente.

## HECHO – 2026-11-01 · Los nombres de la sesión van a `temp.json` (0.25.6)

- Reparto definitivo: `settings.json` → solo los clanes seguidos; `temp.json` → los de la sesión, que
  **se borra en cada arranque** (tope 500); `guild-names.json` → índice de búsqueda, leído solo al
  buscar. Módulo `session_names.rs`, con `reset`/`merge`/`save`.

## HECHO – 2026-11-01 · `settings.json` solo con los nombres de los clanes seguidos (0.25.5)

- `Config::guild_names` se queda con los seguidos (`≤ MAX_FOLLOWED`); al dejar de seguir se borra el
  nombre y `sanitize` limpia al cargar. Su fichero: **51 → 8**.

## HECHO – 2026-11-01 · El primer clic del buscador caía en el aire (0.25.4)

- `is_item_deactivated_after_edit()` valía `true` al clicar una fila, lanzaba otra búsqueda en el
  mismo fotograma y la lista se sustituía por «Buscando...». Ahora perder el foco no busca y
  `State::Searching` arrastra los resultados anteriores.

## HECHO – 2026-11-01 · El índice vuelve a `data/` (y el ZIP con él)

- **`data/equipos.xlsx` está borrado A PROPÓSITO** (el jugador lo usó una sola vez para rellenar
  `team-names.json` y no lo quiere; lo dijo el 2026-10-31 y otra vez el 2026-11-01, enfadado porque
  lo repuse). **No se repone**: está en `.gitignore` con esa nota. Si alguna vez hiciera falta, se
  pregunta antes de recuperarlo.
- **Error mío, y queda apuntado como trampa:** al pasar el índice a "solo caché de búsqueda"
  quité del repo `data/guild-names.json` —que había generado el jugador— y lo añadí a
  `.gitignore`. Nadie me lo pidió: al contrario, AGENTS §9 dice que no se borra lo del jugador sin
  preguntar. También se había quedado fuera `data/equipos.xlsx` (dos veces) y las capturas
  `image*.png` del jugador. Todo repuesto desde el historial de git y desde la carpeta del addon.
- Estado actual: `data/guild-names.json` (9 886 nombres), `data/gw2_lurker_guilds.zip` y
  `data/gw2_lurker_guilds.zip.md5` **versionados**, que es lo que el jugador venía haciendo. El ZIP
  regenerado da el mismo MD5 que el publicado (`c8941df1…`), así que la distribución y `data/`
  coinciden.
- `scripts/pack-guild-index.ps1` empaqueta por defecto **desde y hacia `data/`** (antes escribía en
  `dist/`, que está ignorado, y leía de Documentos).
- **Trampa para el que venga:** ficheros que crea el jugador (`data/guild-names.json`,
  `data/equipos.xlsx`, capturas) **no se borran ni se desversionan sin permiso explícito**. Si algo
  parece sobrante, se pregunta.
## HECHO – 2026-11-01 · Español y francés de los equipos desde la lista de la wiki (0.25.3)

- El jugador pasó una instantánea de `wiki.guildwars2.com/wiki/API:2/wvw/guilds/:region`
  (2025-11-15) que, al final, trae la **lista estática de los 27 equipos con `{en, de, es, fr}`**:
  es la primera fuente con traducciones oficiales que aparece (la API devuelve 404 y gw2mists solo
  tiene inglés).
- Se completaron **15 valores en 13 equipos** que tenían el inglés de relleno, y se **mantuvieron**
  los que ya tenían traducción propia. Dos de la wiki traen errata y **no** se copiaron:
  `Prisón de Abaddon` (falta la i) y `Ravn de la pierre de sang` (falta la i de *Ravin*).
- Se añadió `Cathedral of Blood` (11010 en la wiki; hoy *Ruined Cathedral of Blood*) para que el
  nombre resuelva si vuelve.
- **Sin tocar a propósito:** el alemán difiere en 7 equipos entre gw2mists y la wiki; el jugador
  pidió solo español y francés. Queda anotado en `07-spikes.md` para decidirlo.
- Verificación: `data/team-names.json` parsea, 246 tests en verde, 0.25.3 publicada.
## HECHO – 2026-11-01 · El «?» de la ventana de avisos: glifos fuera de latino-1 (0.25.2)

- El jugador vio un «?» donde iba una flecha en algunas líneas de aviso. No era un fallo de datos:
  el atlas de fuentes de Nexus solo trae latín básico y latino-1, así que `→` (U+2192) se dibuja como
  «?». Lo delata que `·` (U+00B7) y `×` (U+00D7), que el addon ya usaba, sí se ven: el corte está
  exactamente en latino-1.
- Afectaba a todo lo que quedaba fuera: flechas de los avisos y de la ruta (`→`, `←`), puntos
  suspensivos tipográficos (`…` en «Buscando…» y «Descargando el índice…»), estrellas de la pestaña
  Clanes (`★`/`☆`) y las comillas alemanas (`„ “`) de una ayuda del buscador.
- Arreglo: `->`, `<-`→ texto «estás aquí», `...`, `*`/`-`, « ». Y un **test de invariante**
  (`rf44_the_table_stays_inside_latin_1`) que recorre toda la tabla de textos y falla con cualquier
  carácter por encima de U+00FF: es la forma de que esto no vuelva en la siguiente clave que alguien
  escriba.
- Límite conocido: los nombres de clan vienen del API y pueden traer cirílico o emoji; esos seguirán
  saliendo como «?» salvo que metamos una fuente propia en la DLL (descartado por tamaño, §2.9).
- Verificación: 245 tests, clippy/fmt limpios.
## HECHO – 2026-11-01 · Cuelgue mío (0.24.2), avisos solo de estructuras y el índice fuera de memoria (0.25.0)

- **0.24.2 — el cuelgue del jugador a las 00:40 era mío.** El aviso naranja de «partida congelada»
  que añadí en la 0.24.1 llamaba a `crate::runtime()` **dentro** de `draw_health`, que ya recibe la
  configuración bloqueada: `Mutex` no reentrante → el hilo de render se espera a sí mismo y el juego
  se congela. El log acababa en `sondeo 50` sin pánico (el worker seguía vivo) y el `arcdps_crash`
  del directorio era del 30/10, así que no había excepción: era un bloqueo. **Misma trampa del
  2026-10-31, en otro sitio.** Revisadas las demás llamadas a `runtime()` de `ui/mod.rs`: todas
  están fuera del candado (callbacks del menú, pestaña Clanes y `draw_openers`).
- **0.25.0 — cuatro peticiones**, con ADR-016:
  1. **Avisos solo de estructuras** (RF-19e): `AlertFilter::structures` con una casilla por tipo
     reclamable; ruinas, mercenarios y *bloodlust* nunca avisan. Un objetivo de tipo **desconocido
     sí** avisa (la tabla de objetivos llega una vez por sesión; descartarlo dejaba el addon mudo).
  2. **Nombres fuera del índice** (RF-20b): `Config::guild_names` en `settings.json`; `names.rs`
     pasa a ser solo el índice, sin `load`/`all`/`get`. El worker mezcla (no sustituye) para no
     pisar lo que el render acaba de añadir.
  3. **El índice se distribuye** (RF-20c): `scripts/pack-guild-index.ps1` genera
     `gw2_lurker_guilds.zip` (9 886 nombres, 297 532 B) + `.md5` (`c8941df1…`) y lo publica en
     `gw2addonsreleases`. Verificado bajando la URL real (bytes y MD5). Añadida la dependencia
     `zip` 2.x (`deflate`), la misma que usa `gw2-rotation-builds`; el MD5 va en `md5.rs` con los
     vectores del RFC 1321 para no añadir otra caja.
  4. **Al añadir un clan** que no está en el índice se pide su nombre y se añade (`names::append`
     en un hilo corto, `publish` por lectura-mezcla-escritura con `.tmp` + rename).
- **Trampa nueva (apúntala):** `ui::settings::body` y `ui::guilds::body` reciben `&mut Config`, o
  sea que **el render ya tiene el candado**. Cualquier cosa que llame a `crate::runtime()` desde
  ahí (o desde `follow::add_section`) cuelga el juego. Para escribir nombres desde la ventana se
  pasa `&mut config.guild_names` y deja que `persist_if_changed()` guarde.
- **Callejón sin salida:** `git push` al repo compartido fue rechazado porque alguien más había
  publicado (`2e52ec2 Update README…`). El script nuevo comprueba el código de salida de
  `commit`/`push` y ahora aborta en vez de decir «ok» (antes lo decía con el push fallido).
- Verificación: 241 tests, clippy/fmt limpios, `check_loc.sh` y `check_traceability.sh` en verde.
  Publicadas 0.24.2 y 0.25.0; ZIP del índice verificado en su URL.
## HECHO – 2026-11-01 · La causa de «deja de registrar eventos»: la API congela la partida (0.24.1)

- El jugador reapuntó los «~10 minutos» y aportó captura (`image-6.png`): a las 00:15 la cabecera
  decía «última actualización 00:15:20» y «último reclamo hace 7 min», estando en alpino azul con el
  equipo Invernadero de Lutgardis. Es decir: el addon **no** estaba parado.
- **Medido con `curl`, al margen del addon:** la partida `1-1` devolvía la misma copia durante
  al menos 20 minutos —reclamo más nuevo 23:07:58 UTC y **puntos congelados**
  (`67568/52264/35741` en cuatro consultas separadas)— mientras `2-3`, `1-2`, `1-3` y `1-4`
  avanzaban (23:20–23:25). Ni CDN (mismo resultado con `?cb=`) ni WinHTTP: es el backend de
  ArenaNet. Comprobado también que el equipo `11011` solo está en `1-1` y que el clan `DDK`
  (`ACBC90A5`) no aparece en ninguna de las nueve, porque su partida es esa y su copia está vieja.
- Arreglo posible: `core::matchup::digest` (huella de puntos + objetivos), `payload_digest` y
  `payload_since` en el Dashboard, y aviso naranja a partir de 5 minutos con los minutos y el
  intervalo. Así «no pasa nada» no se confunde con «el addon está parado».
- **Ojo:** `bugs.md` se había quedado sin las entradas anteriores al añadir las de la 0.24.0 (fallo
  mío al reescribir el fichero con PowerShell). Restauradas desde el commit `349bbef`.
- Verificación: 226 tests, clippy/fmt limpios, 0.24.1 publicada (MD5 `92ec8b87…`) e instalada.
## HECHO – 2026-11-01 · Fuera el historial en disco, nombres de clan de verdad y buscador con caché (0.24.0)

- Seis peticiones del jugador, todas con causa medida:
  1. **Fuera `crates/store`**: el `session-000.jsonl` no lo leía nadie (la UI usa el historial en
     memoria y ADR-012 borra el fichero al arrancar). Se borra el crate, el volcado del worker y
     el fichero que hubiera.
  2. **Nombres de clan siempre, y para siempre**: `names.rs` + `guild-names.json`. Se pidió «en el
     Bastión salen ids»: allí no se sondea nada y los nombres solo venían del sondeo. Ahora la
     caché se carga al arrancar, se publica en el dashboard y se rellena también sin WvW.
  3. **Segundos del pop-up**: el saneado recortaba a 5 s y el control ofrecía 20 → se guardaba 5.
     Mismo tope en los dos sitios (`POPUP_SECS_MAX`).
  4. **El atajo ya no es un literal**: `keybinds.rs` lee `Nexus/InputBinds.json` (Nexus no expone
     lector de atajos). El fichero real tenía `Ctrl+Shift+Code 39`, no la «L» que decía el texto.
  5. **Click-through**: fuera la casilla de Ajustes → Ventanas; queda el clic derecho.
  6. **Buscador**: `/v2/guild/search` devuelve **un** clan y solo por el principio del nombre
     (medido: `die` → *Die*; `a new` → *A New*). La búsqueda real es por subcadena sobre
     `guild-names.json`; el API solo añade una coincidencia. `scripts/guild-search.ps1` permite
     buscar y construir el índice sin abrir el juego.
- Verificación: 225 tests, clippy/fmt limpios, sin ficheros >800 líneas. Publicada la **0.24.0**
  (MD5 `446ea91f…`, 1 930 752 B) e instalada en local. El índice EU+NA se construye con
  `scripts/guild-search.ps1 -BuildIndex` (~10 000 nombres; con `-DelayMs 30` tarda un rato largo,
  y el fichero `guild-names.json` se puede copiar entre instalaciones).
- **Trampa nueva:** el índice hay que construirlo con el script (o esperar a que el addon vaya
  viendo clanes); el API **no** permite listar «todos los que contienen X».

## HECHO – 2026-10-31 · Live map de verdad: 5 s, ruta desde el primer minuto y avisos «solo los míos» (0.23.1)

- El jugador pidió lo que hace gw2mists: lista de favoritos, seguimiento de sus reclamos mientras no
  se reinicie el juego, ruta al clicar un favorito y avisos configurados. Tres de las cuatro ya
  estaban (RF-73, RF-70…72, RF-19a); lo que faltaba era el **sondeo rápido**, el **historial desde el
  baseline** y dos comodidades.
- **Sondeo cada 5 s** (RNF-02 actualizado): el coste se arregla pidiendo **una** partida por id
  (`match_by_id`, 44 096 B con sus cuatro mapas) en vez de `match_for_team` —que por dentro pide
  `ids=all`, 400 KB—. Con el id conocido el gasto baja de 13 KB/s a 9 KB/s. `ids=all` queda para el
  selector en pantalla y la validación al entrar (una vez por carga y al cambiar de personaje).
- **Historial desde el baseline** (ADR-012 punto 2, que el código incumplía): `history_row` ya no
  tira los reclamos `late`; entran con su `claimed_at` real. Los avisos siguen ignorándolos
  (`AlertLine::late`), que era el motivo del filtro.
- **Clic en favorito o en un aviso → pestaña Mapa** (`TabItemFlags::SET_SELECTED` en el fotograma
  siguiente). **Flecha de dirección** en el último tramo y un solo reclamo ya dibuja punto.
- **«Solo los clanes que sigo»**: `AlertFilter::only_followed`, una casilla.
- Verificación: 226 tests en verde, clippy y fmt limpios, sin ficheros >800 líneas.
- **0.23.1 (mismo día):** el sondeo pedía `ids=all` en **cada** sondeo, porque «el selector está
  en pantalla» se encendía con `gate.settings_available()` (cierto casi siempre): a 5 s eran
  4,8 MB por minuto. Ahora el aviso caduca a los 3 s, los nueve se piden como mucho una vez por
  minuto y el resto de sondeos van a una partida (44 KB). Publicada 0.23.1 el 2026-10-31 23:5x.
- **Trampa nueva:** Nexus **sustituye** nuestra DLL en cada arranque si difiere de la publicada
  (`UpdateProvider::Direct`). Instalar en local una versión que no esté publicada no sirve: el
  `install.ps1` copia, pero al abrir el juego el updater vuelve a bajar la del repo. Para probar,
  publicar. Y `raw.githubusercontent.com` tarda ~3 min en servir el blob nuevo: el `.md5` sale
  antes que la DLL, así que hay que comprobar la **descarga**, no el HTTP 200.

## HECHO – 2026-10-31 · Los cuatro bugs de la bandeja, con el juego abierto como testigo (0.22.10)

- Se miró `bugs.md` (el jugador lo reescribió mientras trabajábamos: «a los 10 minutos deja de
  funcionar» y «tengo el clan en na y en eu» llegaron a mitad). Esta vez había **evidencia en
  vivo**: el juego estaba abierto, así que se leyeron `Nexus.log`, `settings.json` y el propio
  MumbleLink (memoria compartida) en vez de suponer.
- **MumbleLink medido:** `name` = «Guild Wars 2» (no el personaje), `identity` =
  `{"name":"Jhoin",…,"world_id":2147483649,"team_color_id":9}`, `shard_id` = `2147483649`
  (`0x80000001`), `instance` = 0. De ahí salen dos cosas: el `shard_id` **no** es la partida (la
  detección de 0.22.9 no puede funcionar, y el log ya lo dice), y el roster se indexaba con el
  nombre del juego.
- **Arreglos (detalle en `bugs.md`):** reclamo caducado fuera (`last_flipped > claimed_at`), el
  `×` de dejar de seguir dentro de la ventana, `click_through` copiado de vuelta al configurar y
  leído del runtime para el clic derecho, el buscador enseña los clanes que ya se siguen, el
  nombre del personaje sale de `identity`, y el sondeo pide la partida del equipo que va a vigilar.
- Verificación: 222 tests en verde (96 addon · 16 api · 104 core · 6 store), `clippy -D warnings` y
  `cargo fmt --check` limpios, ningún fichero por encima de 800 líneas (se partió
  `worker.rs` → `worker/tests.rs` y los ayudantes de clic a `ui/window.rs`), DLL de 1.904.640 B
  (objetivo 2,2 MB).
- **No verificado en el juego**: nada de esto se ha podido pulsar dentro de GW2. En particular el
  «a los 10 minutos deja de funcionar» sigue abierto: el log dirá si el addon sigue sondeando
  (`sondeo N: … último reclamo hace X min` cada 10 sondeos).
- **Trampa nueva apuntada:** `crate::config()` es la copia **del arranque** y nunca cambia; todo lo
  que el jugador toca en la ventana (clanes seguidos, filtros, click-through) vive en
  `crate::runtime()`. Leer `config()` desde el render fue el origen de tres de los cuatro bugs.

## HECHO – 2026-10-31 · La partida la dice el juego, no los ajustes (0.22.9)

- El jugador corrigió el diagnóstico: **entró al juego con 0.22.6, reclamó campa y el addon no lo
  detectó**; minutos después otro clan reclamó otro campa y tampoco. No era «la partida estaba
  tranquila»: era que se vigilaba la partida del **equipo guardado**, y ese equipo no era el suyo
  (la cabecera decía `PARTIDA EN NORTEAMÉRICA` con un equipo NA mientras él jugaba en Europa).
  Todo lo que ocurre en su partida real es invisible para el addon.
- Arreglo de raíz: `core::detect` (`of_shard`) + `addon::detect` (caché de 15 min). En cada arranque
  y cada 15 minutos se piden los nueve emparejamientos, se lee el `shard_id` de Mumble y se busca en
  `all_worlds` de cada uno (que mezcla mundos viejos y equipos nuevos); el que aparezca es la partida
  del jugador, y de ahí sale también su equipo. La cabecera lo marca: `EQUIPO … (lo dice el juego)`.
  Sin detección se sigue usando el equipo guardado, así que un shard raro no rompe nada.
- Tests: `rf03a_the_shard_says_which_matchup_the_player_is_in`, `…_a_shard_nobody_knows_detects_nothing`,
  `…_the_colour_is_reported_even_when_it_does_not_match`. El segundo es el que habría cazado el bug:
  un mundo de EU **no** puede caer en `1-1`.
- Refactor: la detección a `addon/src/detect.rs` (worker.rs había llegado a 850 líneas).
- **UNVERIFIED(T-19)**: que `shard_id` sea el mundo/equipo de WvW no está confirmado en el juego.
  Por eso el log escribe `deteccion: …` y `donde: mumble shard …`: la próxima sesión lo confirma o lo
  desmiente, y mientras tanto se cae al equipo guardado.
- Verificación: 215 tests en verde (91 addon · 16 api · 102 core · 6 store), `clippy -D warnings` y
  `cargo fmt --check` limpios, ningún fichero por encima de 800 líneas.

## HECHO – 2026-10-31 · Cuatro bugs de la 0.22.6 y el aviso de equipo equivocado (0.22.8)

Del informe de las 22:35, con el API y el log como evidencia.

- **Click-through no hacía nada (regresión mía de 0.22.0).** `Window::no_inputs()` no lo hereda la
  `ChildWindow` donde vive el cuerpo desde que el cuerpo está en una hija: el cuerpo seguía
  capturando el ratón aunque el ajuste estuviera marcado. Ahora la hija se crea con `no_inputs`.
- **El `×` de dejar de seguir no se podía confirmar.** La pregunta y los dos botones iban en la misma
  línea y los botones se salían por la derecha de la ventana. Ahora la pregunta va en una línea
  (`text_wrapped`) y los botones en la siguiente, con textos explícitos.
- **Buscador de clanes.** Dos causas medidas: (1) el equipo elegido **descartaba** los resultados,
  así que buscar el clan propio con otro equipo decía «no existe»; ahora se enseña todo lo de la
  región con su equipo y los del equipo elegido primero (`Hit::matches_team`). (2)
  `/v2/guild/search?name=yak` devuelve **otro** clan: el API busca por el principio de una palabra
  del nombre, no por el tag, así que *Dolyak Sanctuary* no sale con «yak». El mensaje lo explica.
- **«Ha dejado de recoger quién reclama»: sigue abierto, pero con evidencia nueva.** El API decía que
  en `1-1` no había habido **ningún** reclamo entre las 22:02 y las 22:36, mientras otras partidas
  se movían. Como no se puede distinguir «partida tranquila» de «estoy en otra partida», se añade:
  - `diagnostics::log_where`: una línea por cambio con `mumble shard`, `color`, equipo configurado y
    el equipo que el juego dice (`Match::team_of_shard`).
  - `Dashboard::played_team` y un aviso en naranja en la cabecera cuando no coinciden.
  - `ffi::mumble_where` + `ffi::identity_team_colour` (puro y con test: 1/2/3 = rojo/verde/azul).
  - **UNVERIFIED(T-19)**: el significado de `shard_id` en WvW no está confirmado en el juego; la
    línea del log es justo lo que lo resuelve, y `team_of_shard` cae al id nuevo del color.
- Convención nueva a petición del jugador: **no se borran los bugs**, se marcan `[x]` con la versión
  (AGENTS.md §9 reescrito; `bugs.md` mantiene los cuatro con su estado).
- Verificación: 212 tests en verde (91 addon · 16 api · 99 core · 6 store), `clippy -D warnings` y
  `cargo fmt --check` limpios, ningún fichero por encima de 800 líneas.
- **No verificado en el juego**: click-through, la confirmación del `×`, el buscador y el aviso de
  equipo.

## HECHO – 2026-10-31 · Solo dos ventanas: fuera el aviso de Nexus (0.22.7)

- El jugador lo dejó claro: «hay dos ventanas, el widget y la ventana de arriba de avisos; no hay
  avisos de Nexus». Yo estaba explicando un **tercer** mecanismo (`nexus::alert::send_alert`, RF-19d
  original) que existía en el código, estaba apagado por defecto y nunca se vio. Un mecanismo que no
  se ve y que hay que explicar es ruido: se elimina.
- Se van `Config::critical_toast`, su casilla, `ui::toast_critical`, `alerts::plain` y las dos claves
  de la tabla que lo explicaban. `settings.json` sigue cargando: el campo es `#[serde(default)]` y
  una clave de más no molesta.
- `specs/01-requirements.md` RF-19d se reescribe: el aviso emergente es el nuestro, enseña lo que
  pase los filtros, de uno en uno, 0–20 s cada uno, y **no** se usa el sistema de avisos de Nexus.
  El sonido sigue fuera: `nexus-rs` 0.12 no tiene API de audio.
- Verificación: 211 tests en verde, `clippy -D warnings` y `cargo fmt --check` limpios, ningún
  fichero por encima de 800 líneas.

## HECHO – 2026-10-31 · Nada de «1-1» en la interfaz (0.22.6)

- El jugador lo había pedido cinco veces y tenía razón: `1-1` es un identificador del API y no
  significa nada para una persona. Cabecera: `PARTIDA EN NORTEAMÉRICA` + `Verde · <equipo>   Azul ·
  <equipo>   Rojo · <equipo>` (los tres del emparejamiento, con `theme::team_colour`). El id sigue
  en el log, que es donde sirve para diagnosticar.
- El selector de equipo también lo enseñaba (`Invernadero de Lutgardis · 1-1 · Europa`): ahora
  `Invernadero de Lutgardis · EU · Lutgardis, Vabbi`. `ui::region_of` pasa a ser la fuente única de
  región a partir del id.
- Verificación: 211 tests en verde, `clippy -D warnings` y `cargo fmt --check` limpios.
- La tabla i18n llegó a 806 líneas: se parte en `i18n/table.rs` y `i18n/table/more.rs`, con
  `i18n::entries()` recorriendo las dos y un test que comprueba que una clave de cada mitad traduce.

## HECHO – 2026-10-31 · «Aviso emergente», todo lo que sigo, último reclamo y teselas a medias (0.22.5)

Tres cosas del informe de las 22:09, con la evidencia del `Nexus.log` y del API.

- **«Ha dejado de recoger quién reclama».** El log dice `sondeo 20: partida Some("1-1"), 44
  reclamados de 178 objetivos, **0 eventos nuevos**`, y el API confirmó lo mismo: en `1-1` el último
  `claimed_at` era 21:02 UTC (22:02 local) mientras otras partidas se movían a las 21:09/21:10. O la
  partida estaba tranquila o el equipo configurado no es el que se juega. Como eso no se puede
  distinguir desde el widget, ahora **el widget y el latido dicen cuándo fue el último reclamo de la
  partida** (`Dashboard::last_claim_at`, `diagnostics::last_claim_of`). No se toca la detección: no
  había nada que detectar.
- **«Toast» → «aviso emergente».** El ajuste pasa a «Aviso emergente (Nexus) de todo lo que sigo» y
  su ayuda dice que lo dibuja Nexus encima del juego. Y ya no se limita a fortalezas y castillo:
  manda `send_alert` para **cualquier** aviso que pase los filtros de tipo y alcance, como el pop-up
  del addon. `alerts::is_critical` desaparece con su test y entra
  `rf19d_every_alert_the_player_follows_can_be_interrupted`.
- **Huecos en el mapa (bug de verdad, encontrado en el log).** Decenas de `tiles: no se pudo cargar
  la textura`: la descarga escribía directo en el nombre final, así que el render leía JPEG a medias;
  esas cuatro fallaban y **gastaban el presupuesto de texturas de cada frame**, con lo que el resto
  del mapa no se cargaba nunca. Arreglo: descarga a `.part` + renombrado, `MIN_TILE_BYTES`, y una
  tesela que falla se salta 5 s (`TILE_RETRY`) sin repetir el aviso.
- Refactor: `heartbeat` y `last_claim_of` a `diagnostics.rs` (worker.rs había llegado a 819 líneas).
- Verificación: 210 tests en verde, `clippy -D warnings` limpio, `cargo fmt --check` limpio, ningún
  fichero por encima de 800 líneas.
- **No verificado en el juego.**

## HECHO – 2026-10-31 · El buscador de clanes y el detalle del mapa (0.22.4)

Los dos bugs que quedaban en `bugs.md`, con captura (`image-2.png`).

- **`invalid type: string "12008"` en el buscador, la causa de verdad.** El API cambió la forma de
  `/v2/wvw/guilds/:region`: ahora manda `{ "id-de-clan": "12008" }`, es decir el **equipo como
  cadena**, y `api::wvw_guilds` parseaba a `BTreeMap<String, u32>`, así que el paso «equipos» de la
  búsqueda moría siempre. Medido con `curl`: 282 644 bytes en EU, 5 542 clanes, **todos** los valores
  cadenas numéricas, y `0006EDDA-…` resuelve a *Les Ringardes Dansants* `[LrD]`. Arreglo: `TeamRef`
  (`#[serde(untagged)]`) que acepta texto o número y convierte, con test
  `rf03a_the_team_of_a_guild_is_read_whatever_the_api_uses`. Lo de 0.18.0 (comprobar la forma del id)
  ya estaba bien; el fallo no venía de ahí.
- **El detalle del objetivo, encima del ratón.** Se dibujaba como texto debajo del mapa y con el mapa
  ya a tamaño real había que hacer scroll. Ahora es una etiqueta con fondo junto al cursor, dentro
  del recorte del mapa y sin salirse del recuadro (`ui.calc_text_size`). Debajo queda solo la pista,
  y solo cuando no hay nada bajo el ratón.
- Refactor: los tests de `ui::map` a `ui/map/tests.rs` (el fichero había llegado a 803 líneas).
- Verificación: 210 tests en verde (90 addon · 16 api · 98 core · 6 store), `clippy -D warnings`
  limpio, `cargo fmt --check` limpio, ningún fichero por encima de 800 líneas.
- **No verificado en el juego**: la etiqueta del ratón y la búsqueda completa (el buscador necesita
  pulsar «Buscar» con el juego abierto).

## HECHO – 2026-10-31 · El pop-up va de uno en uno y el mapa con su proporción (0.22.3)

Tres bugs nuevos del jugador, apuntados de golpe en `bugs.md`:

- **Pop-up de uno en uno** (`alerts::PopupQueue`): antes se enseñaba todo lo llegado en la ventana de
  `popup_secs`, así que dos reclamaciones seguidas se leían como una y desaparecían juntas. Ahora hay
  cola (`pending` + `current` + `shown_at` + `seen`), cada aviso dura su tiempo y entra el siguiente;
  `0 s` la vacía. El reloj entra por parámetro para poder probarlo: dos tests nuevos
  (`rf19a_the_popup_shows_one_alert_at_a_time`, `rf19a_the_popup_fades_in_its_last_half_second`) y el
  de `late` reescrito. El desvanecido de medio segundo se calcula desde `shown_at`, no desde cuándo
  llegó la línea.
- **Selector de segundos de 0 a 20** (antes 0 a 5) con `POPUP_MAX_SECS`, y el valor al activar el
  pop-up pasa a 5 s (`POPUP_DEFAULT_SECS`). El texto de ayuda dice lo nuevo.
- **El mapa con su proporción real** (`map::map_size`, con test): se dibujaba `ancho × 280`.
  Medido en `/v2/maps`: Eternos 3072×3072 (cuadrada), frontera alpina 2560×3584 (1,4 más alta que
  ancha). Ahora el alto sale de `continent_rect`, se recorta entre 160 y 620 px y, si hay que
  estrechar el ancho para que quepa, se mantiene la proporción y se centra.
- Refactor: `ui::popup` sale de `ui/mod.rs`, que había llegado a 808 líneas.
- **`install.ps1` dice la versión**: la lee del `Cargo.toml` (no del nombre del fichero) y comprueba
  que esa cadena esté dentro de la DLL (`env!("CARGO_PKG_VERSION")` la mete en el binario), así que
  una DLL compilada antes de subir la versión sale avisada con «usa -Build». También dice qué había
  instalado antes (MD5 y fecha: la versión de una DLL ajena no se puede leer, no lleva recurso de
  versión) y avisa si ya estaba la misma. Probado con PowerShell 5.1, que es el que lanza
  `install.bat`, y con el manifiesto subido a 0.22.4 a propósito para ver el aviso.
- Verificación: 209 tests en verde (90 addon · 15 api · 98 core · 6 store), `clippy -D warnings`
  limpio, `cargo fmt --check` limpio, ningún fichero por encima de 800 líneas.
- **Pendiente de comprobar en el juego**: el pop-up de uno en uno, la proporción del mapa y —del
  informe anterior— que la ruta inferida se pinte sobre el mapa (no se ha tocado: `draw_route` ya la
  dibuja cuando hay dos claims del mismo mapa y `history` los tiene).

## HECHO – 2026-10-31 · El mapa se sale de la ventana (0.22.2)

- Bug nuevo del jugador, con captura: «el mapa lo pinta fuera de sitio, pinta encima de las pestañas
  de arriba y por debajo». Regresión de 0.22.0: al meter el cuerpo en una ventana hija, el mapa y los
  iconos del registro siguen pidiendo `get_window_draw_list()`, y en imgui la lista de dibujo de una
  hija **es la del padre**; el recorte que sí hace la ventana hija solo se aplica a los widgets, no a
  los trazos a mano.
- Arreglo: `DrawListMut::with_clip_rect_intersect` en los dos sitios que dibujan a mano
  (`ui::map::draw` con el rectángulo del mapa, `ui::alerts::draw_lines` con el de la ventana). Al
  intersecar con el recorte actual, el dibujo nunca se sale de la ventana aunque el mapa sea más
  grande que ella.
- De paso, la captura confirma que **la orientación ya es correcta**: Eternos sale al derecho y cada
  anillo sobre su estructura.
- **Cabecera legible** (de la misma captura): «MAPA 1099 (TIPO 12)» pasa a «Estás en: Desierto rojo»
  (`MapRole::from_map_id` + `map::map_label`, ahora `pub(super)`); el id solo sale cuando el mapa no
  es de WvW. Y «LAST UPDATE» en inglés era una clave mal escrita: el código pedía `Last update` y la
  tabla tiene `Last update {0}`, así que caía al inglés en los cuatro idiomas.
- Verificación: 207 tests en verde, `clippy -D warnings` limpio, `cargo fmt --check` limpio.
- **No verificado en el juego** (la DLL estaba bloqueada por el juego al instalar; hay que cerrarlo).

## HECHO – 2026-10-31 · Dejar de seguir un clan pide confirmación (0.22.1)

- El `×` de «Clanes seguidos» quitaba el clan sin vuelta atrás. Ahora el `×` **arma** la fila y la
  pregunta se hace en el mismo sitio («¿Quitar *nombre* de los que sigues?» con **Sí** / **No**),
  en vez de un modal de dos botones para una acción de una fila. El estado vive en `PENDING_REMOVE`
  (una sola fila pendiente a la vez, que es lo que se ve).
- Claves nuevas en la tabla i18n: `Stop following {0}?`, `Yes`, `No` (es/en/fr/de).
- Verificación: 207 tests en verde, `clippy -D warnings` limpio, `cargo fmt --check` limpio.
- **No verificado en el juego.**

## HECHO – 2026-10-31 · Tema, avisos configurables, toast crítico, reproducción y banner (0.22.0)

Todas las tareas que quedaban sin depender del jugador ni de terceros.

- **T-28 · `ui::theme` (RF-60 a RF-63)**: `Config::theme` (`Inherited` por defecto, `Own`), aplicado
  con la pila de estilo de imgui **solo dentro de nuestras ventanas** (los `push` devuelven un token
  que saca lo empujado al caer: en esta versión no hay `pop_style_color`, y llevar la cuenta a mano
  dejaba la pila descuadrada). `Config::widget_background` (`Dark`/`Transparent`, RF-63) y los colores
  de equipo en **un solo sitio** (`theme::team_colour`), usados por widget, mapa e historial (RF-62).
  Dos tests: colores distintos y tabla de estilo sin duplicados.
- **RF-19c**: `font_scale` (0,8–1,6, aplicado con `set_window_font_scale`), `max_lines` y `alert_kinds`
  (capturas/reclamaciones/liberaciones) filtran el widget, el registro y el pop-up.
- **RF-19d**: `critical_toast` (desactivado por defecto) manda `nexus::alert::send_alert` para
  fortalezas y castillo (`alerts::is_critical`, con test). **El sonido no se puede**: `nexus-rs` 0.12
  no expone ninguna API de audio.
- **RF-36**: `route::progress` y `route::visible_steps` (puros, con tests) y play/pausa en la pestaña
  Mapa; `draw_route` recorta los pasos ya «reproducidos».
- **RF-35**: `guilds::route_steps` inserta «salto a <mapa>» cuando la ruta cruza de mapa, emparejando
  los pasos por `objective_id`. Test `rf35_a_route_that_crosses_maps_says_jump_to` (por eso el helper
  recibe el idioma: leer el activo pasa por la API de Nexus y revienta fuera del juego).
- **RF-50 · banner de versión nueva**: al cargar se apunta tamaño y fecha de
  `<juego>/addons/gw2_lurker.dll`; si cambian, el widget lo dice en naranja con la ruta al pasar el
  ratón. Advertí de que la versión nueva **no** se puede leer del fichero (la DLL no lleva recurso de
  versión), así que el aviso nombra la que está corriendo.
- Verificación: 206 tests en verde (88 addon · 15 api · 97 core · 6 store), `clippy -D warnings`
  limpio, ningún fichero por encima de 800 líneas.

### Añadido después, en la misma 0.22.0 (ronda del 2026-10-31, «borra los bugs resueltos»)

- **El mapa estaba del revés (bug de verdad, no solo de teselas).** `project::to_screen` invertía la
  Y «porque el API dibuja con Y arriba», y eso era una suposición, no un dato. Medido:
  `/v2/continents/1/floors/1` da `Orr y≈42112`, `Kryta y≈30080`, `Janthir y≈16252`,
  `Cantha y≈101100`, y Orr está al **sur** de Kryta, así que **más Y = más al sur**. Comprobado
  bajando las 16 teselas de Eternos y pegándolas en orden de fila: el mapa sale al derecho y cada
  objetivo cae sobre su estructura (*Stonemist Castle* en el castillo central, *Overlook* en el norte,
  *Lowlands*/*Valley* en las esquinas del sur). Con la inversión, el norte salía abajo y cada fila de
  teselas en el sitio de otra: eso era el «se ven 3 trozos, el norte y el sur intercambiados». Ahora la
  proyección es escala y desplazamiento, sin espejo, con el invariante fijado en el test
  `rf30_the_real_orientation_matches_the_continent_data`. Detalle en `07-spikes.md`.
- **La cabecera del widget no se va con el scroll.** El cuerpo se dibuja en un `ChildWindow` con barra
  propia y tamaño = lo que queda de ventana, así que la pestaña de Ajustes scrollea dentro y el `×` y
  el menú del clic derecho siguen donde estaban. Ojo: con el cuerpo en una ventana hija,
  `ui.is_window_hovered()` a secas deja de ser cierto sobre el cuerpo, así que el menú del clic derecho
  pasa a `is_window_hovered_with_flags(CHILD_WINDOWS)`; sin eso solo abriría en el borde.
- **Tests de nombres de equipo que se rompían al traducirlos.** Tres tests fijaban el español del
  Excel (`Pajar de la Gran Casa`); el jugador lo cambió a `Gran Aviario` en `data/team-names.json` y
  los tests se cayeron. Ahora comprueban el **mecanismo** (la tabla manda si no hay nombre propio, el
  nombre propio gana, vaciar vuelve a la tabla) tomando el texto del fichero, sin fijarlo.
- **`bugs.md` se limpia.** A petición del jugador, los bugs arreglados **se borran** del fichero (el
  histórico vive en esta bitácora y en `CHANGELOG.md`); quedan solo cuatro pendientes y §9 de
  `AGENTS.md` ya lo dice así.
- Verificación: 207 tests en verde (88 addon · 15 api · 98 core · 6 store), `clippy -D warnings`
  limpio, `cargo fmt --check` limpio, ningún fichero por encima de 800 líneas.
- **No verificado**: nada de esto se ha visto dentro del juego todavía (orientación del mapa, scroll de
  la cabecera, menú del clic derecho sobre el cuerpo).

## HECHO – 2026-10-31 · La ventana vuelve a moverse, el registro va ligero y el equipo se reconoce (0.21.0)

- **La ventana no se movía y el `×` plegaba**: regresión de 0.18.0. La barra de título se había
  convertido en un `invisible_button` a toda la anchura (para que el doble clic fuera fácil de acertar)
  y se quedaba con el clic, así que el arrastre no llegaba a la ventana y el `×` contaba como clic en la
  barra. Vuelta al `ui.text` del título: `×` cierra, doble clic contrae/despliega, un clic abre si está
  contrada, y el arrastre funciona porque el texto no captura el ratón.
- **Registro ligero**: `log_body` usa **`ListClipper`** y un tope de **50** líneas
  (`alerts::MAX_LOGGED`); antes pintaba todo lo de la sesión y cada línea son varios trozos de texto.
- **Nombres de clan en los avisos**: 20 búsquedas por sondeo en vez de 10 (`GUILD_LOOKUPS_PER_POLL`).
- **T-52 · los mundos antiguos en el selector de equipo**: `api::worlds()` (51 entradas, una vez por
  sesión, `catalog::world_names`), `Candidate::worlds` y `settings::worlds_label`, así que el desplegable
  enseña `Equipo (id) · Partida · Mundos antiguos`. Es la forma de reconocer un equipo nuevo: el jugador
  eligió un equipo **NA** llamado *Lutgardis Conservatory* teniendo el mundo Lutgardis en EU, y por eso
  sus capturas no aparecían (el widget lo decía: `EMPAREJAMIENTO 1-1 · NA`).
- Se **respetan** las traducciones que se añadieron a mano en `data/team-names.json`
  (`Invernadero de Lutgardis`, `Bosquemusgoso`); el test ya no fija un nombre concreto para eso.
- Verificación: 200 tests en verde, `clippy -D warnings` limpio, ningún fichero por encima de 800 líneas.

## HECHO – 2026-10-31 · El mapa real se dibuja entero y el sondeo se puede ver (0.20.1)

- **«El mapa no es real», encontrado en el código**: `tile_texture` gastaba un presupuesto de 4
  teselas **por sesión** en lugar de 4 **por frame** (el contador nunca se reiniciaba), así que de las
  16 teselas de un mapa solo se cargaban las 4 primeras y el resto no aparecía nunca. Ahora el
  presupuesto se reinicia en `draw_tiles`, y una textura que no se pueda crear deja un `warn` con la
  ruta.
- Comprobado además en la instalación del jugador que la caché ya es la correcta: **59 ficheros
  `2-1-*`** y el log dice «las del mapa ya estaban en caché», así que el continente y la escala están
  bien desde 0.16.1.
- **Latido del worker** (`worker::heartbeat`): cada diez sondeos, una línea con la partida, los
  reclamados de los objetivos y los eventos nuevos. «Desde las 20:22 no recoge nada» pasa a ser
  comprobable: si el latido avanza, el sondeo va; si `eventos nuevos` se queda en 0 y no hay
  `evento:` detrás, el problema está en el diferencial o en el filtro, no en la red.
- Dato del log del jugador: `worker: vigilando 1-1 (NA) · Confirmed { team_id: 11011 }` y gw2mists
  lista **Lutgardis Conservatory en la tabla NA**, así que el equipo está bien elegido y la partida
  vigilada es la suya.
- Verificación: 200 tests en verde, `clippy -D warnings` limpio, ningún fichero por encima de 800
  líneas.

## HECHO – 2026-10-31 · Nombres de equipo desde gw2mists, con el navegador (0.20.0)

- **T-45, desbloqueada**: el jugador tenía razón — el navegador integrado de VS Code ejecuta el
  JavaScript de la SPA y las cuatro páginas se leen enteras. `curl` sigue viendo una página vacía y
  `api.gw2mists.com` sigue dando 403 a los scripts, así que lo verificado es **el navegador**
  (`07-spikes.md`).
- `data/team-names.json` rehecho con **33 entradas** (una por id de `all_worlds`): inglés de
  `wvw-teams.json`, **alemán de la web**, español y francés del Excel cuando el nombre inglés está en
  la hoja y si no el inglés. La web solo lista los **27 equipos en juego**, así que los 6 restantes se
  quedan en inglés.
- **El alemán de la web no es el del Excel**, y manda la web porque es lo que se ve en el juego:
  `Grenths Tor` → `Grenths Tür`, `Frost-Zitadelle` → `Frostzitadelle`, `Ettin-Rücken` → `Ettinbuckel`,
  `Voliere des Großen Hauses` → `Vogelhalle des Großen Hauses`, `Skrittsburgh` → `Skrittsburg`.
- Emparejado por fila (alianzas, gremios, jugadores, comandantes) porque el orden de la tabla cambia
  con el idioma; comprobado que `Lutgardis Conservatory` (11011) sale en la tabla **NA**, igual que en
  la API (`1-1`).
- Las traducciones del jugador (`team_translations`) siguen mandando: la tabla es solo el valor por
  defecto.
- Tests nuevos: `rf44_german_comes_from_gw2mists_and_not_from_the_spreadsheet` y
  `rf44_the_pool_covers_every_team_in_use` (falla si un equipo en uso se queda sin entrada).
- Verificación: 200 tests en verde (84 addon · 15 api · 95 core · 6 store), `clippy -D warnings`
  limpio.

## HECHO – 2026-10-31 · Avisos legibles, ruta al clicar un clan y hover con nombre (0.19.0)

- **Formato del aviso** (lo pidió el jugador): la línea se compone de trozos con color propio
  (`alerts::spans`): hora en gris, **mapa en su color** («Eternos» gris, «Azul»/«Verde»/«Rojo» con el
  color de su frontera), y en gris el **sitio**, un guion y el **clan actual**. Antes la línea entera
  iba del color del capturador, que escondía el mapa y el clan. Tests
  `rf19_the_map_is_drawn_in_its_own_colour_and_the_rest_in_grey`, `rf19_a_flip_names_both_colours`.
  La fila se dibuja dentro de un `ui.group` para poder colorearla por trozos y seguir siendo clicable.
- **T-51 · clicar un clan lleva a su ruta**: líneas de aviso y clanes seguidos son clicables; se
  selecciona el clan (`guilds::select`) y el mapa salta al último donde reclamó
  (`Dashboard::last_map_of`, que elige por `claimed_at`). Test `t51_clicking_a_guild_knows_its_last_map`.
- **Hover del mapa con nombre**: `map::detail_line` recibe el mapa de nombres y enseña `[TAG] Nombre`
  en vez del id (era el mismo problema que los clanes seguidos).
- `diagnostics.rs`: las líneas de diagnóstico del worker se mudan allí porque `worker.rs` había
  llegado a 808 líneas.
- Verificación: 199 tests en verde (84 addon · 15 api · 94 core · 6 store), `clippy -D warnings`
  limpio, ningún fichero por encima de 800 líneas.

## HECHO – 2026-10-31 · Clicar un clan lleva a su ruta (0.19.0)

- **T-51**: una línea de aviso (widget y registro) y cada clan de *Clanes seguidos* son ahora
  `Selectable`: al clicar se selecciona ese clan (`guilds::select`) y el mapa salta al **último donde
  reclamó algo** (`Dashboard::last_map_of`), así que la pestaña Mapa dibuja su ruta sin buscarla.
- `last_map_of` elige por **`claimed_at`** y no por orden de inserción: un mismo sondeo puede traer
  varios claims y el orden dentro de él es el que mandó la API. Test `t51_clicking_a_guild_knows_its_last_map`.
- El mapa y el registro se pueden clicar sin dejar de ser legibles: el `Selectable` respeta el color
  de la línea y el tooltip dice qué hace.
- Verificación: 198 tests en verde (83 addon · 15 api · 94 core · 6 store), `clippy -D warnings` limpio.

## HECHO – 2026-10-31 · El atajo trae la ventana y se registra por código de tecla (0.18.1)

- El atajo «no hacía nada». Dos causas a la vez: (1) solo alternaba click-through, así que con la
  ventana cerrada (`×`) no se veía ningún cambio; y (2) se registraba **por cadena**
  (`register_keybind_with_string("CTRL+SHIFT+L")`), y una cadena que el parser de Nexus no acepta se
  registra en silencio — no hay error ni aviso.
- Ahora se registra con `register_keybind_with_struct` y un `Keybind { key: 0x4C /* L */, ctrl, shift }`,
  que no pasa por ningún parser, y lo que hace es **traer la ventana**: `ui::reveal` la muestra, la
  despliega y le quita el click-through. Test `rf18_the_shortcut_brings_the_window_back`.
- Se registra en el log el atajo al cargar y cada pulsación, que es lo que faltaba para poder decir
  «no llega» o «llega y no hace nada».
- Verificación: 197 tests en verde (82 addon · 15 api · 94 core · 6 store), `clippy -D warnings` limpio.

## HECHO – 2026-10-31 · Clanes en su pestaña, horas locales y registro limpio (0.18.0)

Cinco fallos nuevos de `bugs.md`, diagnosticados con el código y con la API en vivo.

- **Registro que no se limpiaba al reiniciar**: la línea base marca como `late` todo lo ya
  reclamado, y esas líneas se enseñaban si el clan era seguido: hasta 50 filas de ayer. Ahora un
  `late` solo se enseña si es de un clan seguido **y** de los últimos 10 minutos (`LATE_GRACE`);
  lo viejo se queda en el historial y en la ruta. Tests `rf21_a_followed_guild_is_told_when_the_claim_is_fresh`
  y `rf25_a_restart_does_not_refill_the_log_with_yesterdays_claims`.
- **Clanes en su pestaña**: `follow::body` se parte en `followed_section` y `add_section`, y las
  dibuja la pestaña **Clanes** (que ya tiene el contexto de la partida); los ajustes solo dicen
  dónde están. La lista de «han reclamado» se corta en **10** filas.
- **Buscador**: la **región** pasa a ser el primer control (con la del jugador por defecto, y
  cambiable), los equipos se enseñan con **nombre** (`team_label` usa `data/wvw-teams.json`, no el
  id) y los resultados dicen a qué equipo pertenecen cuando se busca sin filtrar. Un id que no
  tenga forma de id de clan se descarta antes de pedirlo (`looks_like_guild_id`, con test): de ahí
  venía el `invalid type string …`. El error ahora dice qué se buscó y en qué paso falló.
- **Doble clic en la barra de título**: la barra entera es un `invisible_button` (antes solo las
  letras) y con la ventana contrada un solo clic la abre.
- **Horas en UTC**: nuevo `addon::clock` (`clock`, `clock_seconds`, `local`) y todas las horas del
  addon pasan por ahí; el almacenamiento sigue en UTC. Tests
  `rf19_the_clock_uses_the_players_timezone_and_not_utc`.
- Verificación: 196 tests en verde (81 addon · 15 api · 94 core · 6 store), `clippy -D warnings`
  limpio, ningún fichero por encima de 800 líneas.

## HECHO – 2026-10-31 · La caja de la ventana se guarda también en click-through (0.17.4)

- El clic derecho de T-49 se prueba contra la caja guardada de la ventana, y esa caja **no se
  escribía con click-through**: hasta haber movido el widget alguna vez, el clic derecho no acertaba.
  Ahora la caja se apunta siempre que la ventana no esté bloqueada (en click-through no se puede
  arrastrar, así que apuntarla no le quita nada al jugador; bloqueada se sigue respetando, que era el
  bug original de 0.14.2).
- Verificación: 192 tests en verde, `clippy -D warnings` limpio.

## HECHO – 2026-10-31 · El widget en el Bastión y el clic derecho para salir del click-through (0.17.3)

- **T-48 · el widget en el Bastión del Armisticio**: `map_type` 18 se ocultaba entero por no tener
  objetivos. Nuevo `Gate::CONFIG_ONLY` (`show_ui` alto, `poll` bajo): la ventana se dibuja —es donde se
  configuran los ajustes— y no se gasta una sola petición. La lista de equipos se sigue pidiendo
  (`settings_available`), así que el selector funciona allí. Fortaleza del Valor (13) y Sagrario de
  Obsidiana (14) siguen ocultos. Cambia el GWT de RF-01, pedido por el usuario.
- La cabecera dice en gris «en este mapa no hay nada que vigilar, pero los ajustes funcionan» en vez de
  enseñar pestañas vacías sin explicación.
- **T-49 · clic derecho para salir del click-through**: con `NO_INPUTS` la ventana no recibe el ratón, así
  que `ui::right_click_on_widget` lo mira desde el callback de render, **solo en el flanco de bajada** (con
  `mouse_down` a secas se alternaría sesenta veces por segundo) y contra la caja guardada de la ventana
  (`window::covers`, pura y con test). Al acertar se apaga el click-through y se guarda.
- De paso: el aviso naranja decía `Alt+Shift+L` **escrito a mano** (el atajo viejo que se queda arcDPS) y
  se enseñaba según `locked` en vez de según `click_through`. Ahora se construye con `LOCK_SHORTCUT` y sale
  cuando toca.
- Verificación: 192 tests en verde (77 addon · 15 api · 94 core · 6 store), `clippy -D warnings` limpio.

## HECHO – 2026-10-31 · La descarga de teselas no se da por hecha sin mapas (0.17.2)

- Al revisar el arreglo de las teselas apareció una regresión que yo mismo había metido: `ensure_tiles`
  marcaba la sesión como «ya descargado» en la **primera** llamada, y esa llamada puede llegar antes
  que `/v2/maps` (el registro de 19:40:45 lo enseñaba: «mapa: 0 mapas, 0 objetivos»). Sin mapas no hay
  teselas que pedir, así que el mapa real se habría quedado esquemático toda la sesión.
- Ahora solo se marca cuando hay cajas de mapa, y se registra cuántas teselas se piden y a qué
  continente/piso.
- Verificación: 190 tests en verde, `clippy -D warnings` limpio.

## HECHO – 2026-10-31 · Los cinco fallos del widget (0.17.1)

Leídos de `bugs.md` y diagnosticados con el `Nexus.log` y el `settings.json` reales.

- **Clanes seguidos sin nombre**: `resolve_guild_names` solo pedía `/v2/guild/:id` para los clanes
  que habían reclamado algo en el emparejamiento, así que los seguidos que aún no habían reclamado se
  quedaban como id. Ahora se piden **primero los seguidos**.
- **Rótulos cortados en los ajustes**: el ancho reservado a los rótulos se calculaba con una frase
  larga (un aviso) en lugar de con los rótulos que se dibujan, así que el control se comía la
  ventana. `label_column` mide la lista real de rótulos en el idioma activo.
- **Buscador de clanes mudo**: exigía equipo *y* región, y los dos salían de la lista de equipos
  vivos; con la lista vacía el desplegable no tenía nada y el botón no hacía nada. Además, dentro de
  WvW esa lista solo se pide si alguien la pide y este bloque no lo hacía. Ahora la **región** (la
  del equipo del jugador) es lo único obligatorio, el equipo es un filtro opcional que muestra el de
  cada resultado, el bloque pide la lista al worker y el campo de texto va antes del botón.
- **Pestañas escondidas**: `FITTING_POLICY_SCROLL` en la barra, que es el «…» que se pedía.
- **«Avisos no funciona»**: el log dice que se vigilaba el emparejamiento **`1-1` (NA)** con el equipo
  `11011` («Lutgardis Conservatory», NA). Si el jugador está en EU, sus capturas son de otro
  emparejamiento y no hay nada que avisar. Se añade lo que faltaba para verlo: la cabecera enseña la
  **región** del emparejamiento, avisa en naranja si el equipo guardado ya no juega, y el worker
  escribe `worker: vigilando …` al cambiar de emparejamiento y `evento: …` por cada evento. Queda
  pendiente de que el jugador vuelva a elegir equipo y mire esas líneas.
- **Teselas**: el nombre del fichero no llevaba el continente, así que una caché de otra versión
  podía reutilizarse; ahora es `2-1-{zoom}-{x}-{y}.jpg` en fichero y URL, y las viejas se borran.
- Para no pasar de 800 líneas, las líneas de aviso (`AlertKind`, `AlertLine`, `Received`,
  `push_alert`, `recent_alerts`, `alert_line`) se mudan a `crates/addon/src/alerts.rs`.
- Verificación: 190 tests en verde (76 addon · 15 api · 93 core · 6 store), `clippy -D warnings`
  limpio, ningún fichero por encima de 800 líneas.

## HECHO – 2026-10-31 · Iconos del juego en el mapa y en los avisos (0.17.0)

- **T-46**: los objetivos del mapa ya no son formas geométricas: son el **círculo del color del equipo
  con el dibujo dentro**, como en el juego. Módulo nuevo `ui::icons` que construye cada dibujo como una
  lista de primitivas puras (`Rect`/`Tri` con tinta y «papel» para la puerta recortada) y las pinta aparte,
  así que la geometría **se puede testear sin juego**: que los cuatro tipos que puntúan tengan dibujo, que
  ninguno se dibuje igual que otro (forma y número de piezas), que la torre sea alta y la fortaleza ancha, y
  que el dibujo quede centrado donde se le pide.
- Los dibujos no son assets: el usuario pasó capturas, pero un PNG en el repo puede faltar en una
  instalación y esto no. Si algún día llegan los PNG reales, se sustituye `icons::parts` y nada más.
- **T-47**: `AlertLine` gana `objective_kind` (lo resuelve el worker con la tabla de objetivos) y cada
  línea del widget y del pop-up lleva delante su dibujo en el color de la línea.
- **Trampa evitada**: el dibujo de las líneas se pinta en **una sola pasada**, con las posiciones
  apuntadas mientras se maquetan los textos y la lista tomada una vez (AGENTS §11.12); pedirla por línea
  habría vuelto a tumbar el juego.
- Verificación: 187 tests en verde (73 addon · 15 api · 93 core · 6 store), `clippy -D warnings` limpio.

## HECHO – 2026-10-31 · El mapa real se pide al continente correcto y con la escala correcta (0.16.1)

- **Bug real de 0.16.0**: las teselas se pedían al **continente 1**, y ese continente devuelve una
  imagen de océano para cualquier coordenada (falla en silencio, sin 404). Los mapas de MvM viven en
  **continente 2, piso 1**.
- Y la escala estaba mal: se suponía 2048 unidades por tesela a zoom 5 cuando son **1024**. Fórmula
  corregida: `units_per_tile(zoom) = 256 · 2^(7 − zoom)` (`core::project`), así que un mapa son
  **4×4 teselas** y los cuatro mapas **59 imágenes** en total.
- Verificación: descargadas las 59 URLs (`2/1/5/{x}/{y}.jpg` para Eternos, Alpino verde, Alpino azul
  y Desierto) y **todas responden imagen real** (>5 KB). El test de `tiles_of` ahora espera 1024
  unidades por tesela y como mucho 16 teselas.
- Si una tesela no está en disco todavía, se sigue viendo el tablero esquemático; y apagando **Mapa
  real** vuelve el esquemático siempre.

## HECHO – 2026-10-31 · Mapa real con teselas, rango de la ruta y pop-up arreglado (0.16.0)

- **T-29 (parcial)**: verificado que `tiles.guildwars2.com` llega hasta **zoom 5** (zoom 6 y 7 dan
  404) y que a ese nivel una tesela son **2048 unidades de continente** (⚠ **falso, corregido en
  0.16.1**: son **1024**, y el continente es el 2): un mapa son 2–3 teselas
  por lado, ~20 en total para los cuatro. `core::project::{units_per_tile, tile_box, tiles_of}` es
  puro y tiene dos tests (cobertura de esquinas y «degenerado = ninguna»).
  - El **worker** descarga las que falten a `<addon>/tiles/` en su propio hilo (`ensure_tiles`), así
    que el render nunca toca la red.
  - El **render** pide 4 teselas por frame como máximo (`get_texture_or_create_from_file`) y dibuja
    con `add_image(..).col([1,1,1,opacity])`: el mapa **hereda la opacidad** de la ventana, que es
    lo que pidió el usuario.
  - Interruptor **Mapa real** (`Config::map_tiles`, por defecto **sí**): si se apaga vuelve el
    tablero con rejilla. RF-37 decía «desactivado por defecto»; se cambia porque el usuario lo pidió
    así (2026-10-31).
- **T-16 (más cerca)**: selector de **rango** de la ruta (última hora / 2 horas / toda la sesión).
  El trazo discontinuo para `late`/`gap` queda sin objeto: los `late` ya no entran en el historial
  desde 0.12.1, y los `gap` no son filas. El tooltip es el hover del mapa.
- Backlog nuevo: **T-46** (iconos del mapa como en el juego, con las capturas de campamento y torre;
  falta el de fortaleza) y **T-45** anotada como bloqueada: gw2mists es una SPA y su API da 403.
## HECHO – 2026-10-31 · Pop-ups que se van, mapa con fondo y selector de clanes nuevo (0.15.0)

- **Pop-up que no se iba**: `alert_line` ponía `received = Utc::now()` **cada vez que se formateaba**
  la línea, así que `popup_lines` comparaba "ahora" con "ahora" y nunca caducaba. Ahora el instante
  se guarda con el evento (`worker::Received`) y el pop-up dura lo que dice el ajuste, con
  desvanecido en el último medio segundo.
- Pop-up sin marco (`no_decoration`), arrastrable y con la posición guardada (`popup_pos`), y botón
  **Probar pop-up** en los ajustes para colocarlo sin esperar una captura.
- El texto del aviso empieza por el **mapa**: «Eternos · Zanja de Mendon: Azul → Verde».
- **Mapa**: tablero con rejilla cada 512 unidades, borde y el nombre del mapa; hover por objetivo
  con `nearest`/`inside` (puras y con test) que enseña dueño, alto valor, clan y hora. Las teselas
  reales siguen siendo T-29.
- **T-44**: `ui::follow` sustituye a la lista gigante. Solo los seguidos, × para quitar, y dos vías
  de añadir: los que han reclamado esta sesión (`Dashboard::session_guilds`) y un buscador por
  equipo + palabra con `search` (hilo propio, sin red en el render). Límite de 10.
- **T-43**: `Config::guild_labels` guarda el nombre que escribe el jugador para un clan y se enseña
  detrás del nombre real; nunca lo pisa la API.
- **T-15**: el tag del clan seguido se dibuja junto a su anillo.
- Verificado que `/v2/guild/search` busca **una sola palabra por prefijo** (`Raven` sí, `avenous`
  no, `legends of` no): anotado en `07-spikes.md` y dicho en la UI.
## HECHO – 2026-10-31 · Cierre al abrir el mapa: el draw list es un singleton (0.14.5)
- **Causa encontrada en `Nexus.log` del usuario**, no adivinando:
  `panicked at arcdps-imgui-0.8.0/src/draw_list.rs:103: The DrawListMut instance for the window
  draw list is already loaded! You can only load one instance of it!` con la línea previa
  `mapa: 4 mapas, 178 objetivos (97 con coord), rol Unknown, en Some(RedHome)`.
- `ui::map` pedía `get_window_draw_list()` **una vez por objetivo**. La segunda llamada panica
  (es un singleton por frame con un flag atómico y `Drop`). Ahora se pide una sola vez en `body` y
  se pasa `&DrawListMut<'_>` a `draw_objective` y `draw_route`.
- El `catch_unwind` de 0.14.3 evitó que la excepción cruzara el FFI y dejó la traza en el log:
  sin él esto habría sido otra pantalla negra sin diagnóstico. Merece la pena tenerlo.
- Regla nueva en `AGENTS.md` §11.12 para no repetirlo.
- Ojo para la próxima: tras un pánico el guard deja de dibujar el widget **durante esa sesión**;
  con el fallo arreglado eso ya no se dispara, pero si se vuelve a ver «no aparece nada», es eso.

## HECHO – 2026-10-31 · Bloquear ≠ click-through, atajo libre y sin cierres (0.14.2)

- **Tres reportes del usuario a la vez:**
  1. «La ventana tiene opción de bloquear, con o sin click-through»: en 0.11.2 junté los dos modos
     en un solo `locked` y eso dejó sin nombre al bloqueo de posición. Ahora `WindowState` tiene
     `locked` (no mover ni redimensionar, **sí** acepta clics) y `click_through` (`NO_INPUTS`). El
     menú contextual, los ajustes y las opciones de Nexus tienen los dos por separado. `locked` ya
     no quita el ratón, así que quien lo tuviera activo recupera la ventana al actualizar.
  2. «Alt+Shift+L no abre el menú, abre el legacy de arcDPS»: el atajo por defecto chocaba y el
     juego se lo da a arcDPS. Pasa a **`Ctrl+Shift+L`** y las tres menciones del atajo en la UI se
     construyen con la constante (`LOCK_SHORTCUT`), así que no puede volver a desincronizarse.
     RF-18 actualizado con el motivo.
  3. «Al clicar Mapa ha hecho crash»: no he podido reproducirlo (haría falta el juego) y el
     `catch_unwind` de RNF-06 no existía. Añadido `ui::guard` envolviendo los dos callbacks de
     render: un pánico se anota en `nexus.log` como «pánico dibujando «widget»» y el overlay sigue
     vivo. Además la pestaña Mapa deja una línea `mapa: N mapas, M objetivos (K con coord), rol…`
     la primera vez que se dibuja, para que el próximo informe diga con qué datos murió.
- Pendiente: si vuelve a cerrarse, el `nexus.log` dirá si fue un pánico nuestro o una aserción de
  imgui (esas no se pueden capturar).

## HECHO – 2026-10-31 · La ventana no respondía al ratón (0.14.1)

- **Diagnóstico con el fichero real del usuario**: su `settings.json` tenía
  `"WvW Lurker": { "locked": true }`. Click-through activo y sin ninguna señal visible: los clics
  se van al juego y el addon parece colgado. Nada en pantalla decía cómo salir.
- Arreglos: (1) el estado se anuncia en naranja bajo el estado del worker, con el atajo y la
  mención a las opciones de Nexus; (2) las opciones de Nexus ganan la casilla **Click-through**,
  que es la salida sin teclado; (3) la lista de **clanes seguidos** de los ajustes enseña
  `[TAG] Nombre` en vez del UUID (era el último sitio donde quedaba el id).
- Un clan **seguido** vuelve a avisar aunque el claim ya estuviera al arrancar (RF-21, RF-25): mi
  filtro de 0.12.1 los quitaba todos, y seguir un clan es justo para enterarte. Test
  `rf21_a_followed_guild_is_told_even_if_the_claim_was_already_there`.
- Sin verificar: si el claim de Eternos que reportó el usuario era suyo con su propio clan; si no
  lo sigue, no avisará (RF-04: el baseline no genera avisos). Queda como duda en `bugs.md`.

## HECHO – 2026-10-31 · Aviso de configuración dañada (T-26; 0.14.0)

- `config::load` ya renombraba el fichero ilegible y devolvía un informe, pero ese informe solo iba
  al registro. Ahora `lib::safe_mode` lo guarda para la sesión y `ui::draw_tabs` lo pinta en rojo
  encima de todo (RF-51): volver a los valores por defecto en silencio parece que el addon olvidó
  la configuración.
- La verificación de 0.13.2 en la CDN de `raw/main` terminó en verde al reintentar (bytes y MD5
  coinciden); la trampa de siempre, no un fallo de la publicación.

## HECHO – 2026-10-31 · «Alto valor» por dato y no por tipo (0.13.2)

- `ClaimRow` gana `points_tick`, copiado del objetivo del snapshot en `matchup::claims` y en
  `history_row`, e `is_high_value()` pasa a ser `points_tick >= HIGH_VALUE_TICK` (8). Antes era
  «es Keep o Castle», que dejaba fuera las torres mejoradas y metía una fortaleza recién capturada.
- Valores reales del match `2-2` (fixture): campamento 2; torre 4, 6 u 8; fortaleza 8, 12, 16 o
  20; castillo 12; ruinas, mercenarios y spawns 0. El umbral de 8 marca 17 objetivos y deja fuera
  los 24 campamentos, como dice RF-30b.
- Test nuevo `rf30b_high_value_is_the_tick_of_this_snapshot_not_the_kind`, y el fixture de
  `ui::matchup` pasa a usar los ticks reales por tipo.

## HECHO – 2026-10-31 · Ruta numerada sobre el mapa (T-16 parcial; 0.13.1)

- `ui::map` gana `View` (filas, objetivos, mapas, clanes seguidos e historial) y `draw_route`:
  proyecta las reclamaciones del clan elegido en el mapa que se está viendo, las ordena por
  `claimed_at`, las une con una línea y numera cada nodo; el más reciente lleva un anillo naranja.
  Sin hora, el nodo va en gris.
- La captura de «Ruta inferida a partir de las reclamaciones; no es el recorrido real» (ADR-005)
  se pinta bajo el mapa cuando hay historial, reutilizando la clave de i18n que ya existía.
- Falta para cerrar T-16: selector de rango temporal, trazo discontinuo para `late`/`gap` y
  tooltip por nodo.
- De paso: `worker.rs` había pasado de 800 líneas con la llegada de los catálogos, así que
  objetivos, mapas y nombres de clan se mueven a `catalog.rs` (AGENTS §2.8).

## HECHO – 2026-10-31 · Pestaña Mapa con proyección real (T-14; 0.13.0)

- **`core::project`**: `Frame` (caja de `continent_rect`) y `project` (coordenada de continente →
  píxeles con la Y volteada). Cuatro tests contra las fixtures reales, incluido
  `rf30_every_real_objective_of_eternals_falls_inside_the_box`.
- **Verificado en vivo**: `coord` de los objetivos **ya está en coordenadas de continente** (no hay
  que pasar por `map_rect`), `continent_rect` viene siempre con la esquina menor primero y
  **`points_tick` no existe en `/v2/wvw/objectives`** — el umbral de RF-30b solo se puede calcular
  desde los objetivos del matchup. Todo en `specs/07-spikes.md`.
- **`Api::maps`** pide los cuatro ids (≈2 KB) y el worker lo guarda una vez por sesión
  (`maps_for`), publicado en `Dashboard.maps`.
- **`ui::map`**: dibuja con el `DrawList` de imgui (sin texturas): círculo campamento, cuadrado
  torre, rombo fortaleza (dos triángulos), triángulo castillo; color por equipo o gris si no tiene
  dueño; contorno doble en alto valor; anillo dorado en los de guilds seguidas (RF-31). Los tipos
  sin puntuación no se dibujan y los `coord: null` se omiten.
- La pestaña «Mapa» sustituye en el mismo sitio a la lista de objetivos, que queda plegada debajo
  (`CollapsingHeader`): el texto se lee más rápido que el dibujo en mitad de una pelea.
- Pendiente para la próxima: usar `points_tick` del matchup en `ClaimRow` para que RF-30b sea por
  datos y no por tipo, y la ruta numerada de T-16 sobre este mismo mapa.

## HECHO – 2026-10-31 · El historial empieza vacío al arrancar (0.12.1)

- **Lo reportó el usuario**: «el historial de claims no se borra al reiniciar; lo que se guarda es
  el de clanes que sigues». Causa: el baseline marca `late=true` los claims que **ya estaban** al
  arrancar, y esos se apuntaban con su hora real, así que la lista de avisos y el historial de
  clanes nacían con claims de ayer.
- `recent_alerts` descarta los `late`: lo que ya estaba no es noticia. `history_row` también, así
  que el historial de la sesión empieza vacío.
- En disco solo se escriben los claims de las guilds seguidas (`keeps_in_the_store`, con test):
  el resto es el estado del matchup y la API lo devuelve en cada sondeo. RF-25 queda actualizado.
- 62 tests en el addon (2 nuevos), resto en verde, `clippy -D warnings` limpio.

## HECHO – 2026-10-31 · Una sola ventana con pestañas (T-42; 0.12.0)

- **ADR-014**: el addon pasa de cinco ventanas sueltas a **una** con pestañas —Avisos · Mapa ·
  Clanes · Registro · Ajustes (la última)—, como `docs/boceto.html`. Un solo `WindowState` para
  toda la ventana; los estados viejos se quedan en `settings.json` sin usar, no se borran.
- Refactor de las vistas: `alerts::body`/`log_body`, `matchup::body`, `guilds::body` y
  `settings::body` ya no dibujan cromo; `ui::window` solo envuelve la ventana única. `draw_inline`
  (opciones de Nexus) pasa a ser «coger el candado y llamar a `body`».
- **Opciones de Nexus** ganan al principio **Ver la ventana** y **Avisos emergentes**; siguen
  dibujando los ajustes completos, así que son la salida cuando la ventana está cerrada.
- **`popup_enabled`** (nuevo, por defecto `true`) separa «enseñar pop-ups» de «cuánto duran»: al
  apagarlos y encenderlos se conserva la duración. El pop-up se dibuja **fuera** de la ventana,
  así que funciona con el widget cerrado —que es justo para lo que sirve el interruptor—.
- Trampa evitada: el cromo y la pestaña de ajustes escriben los mismos campos el mismo frame. El
  estado se copia y se vuelca **campo a campo**, y el cromo solo gana en los campos que nadie más
  tocó; si no, mover/bloquear desde ajustes se deshacía cada frame.
- Verificado: 60 tests en el addon (incluye `t42_the_settings_are_the_last_tab`), 86 en `core`,
  14 en `api`, 6 en `store`; `fmt` y `clippy -D warnings` limpios; trazabilidad y LOC en verde.
  **Sin verificar en el juego**: la barra de pestañas real, el tamaño por defecto y que la pestaña
  de ajustes quepa sin scroll incómodo.

## HECHO – 2026-10-31 · Los cinco fallos del widget (0.11.3)

- **Regla nueva y permanente en `AGENTS.md`**: `bugs.md` se lee **antes de empezar, después de
  cada paso y al terminar** (§1.0, §3.0, §9). "Cada paso" = cada test en verde, cada arreglo,
  cada versión.
- **Los clanes salían como UUID.** `claimed_by` viene con guiones y mayúsculas
  (`9FDB4A86-1588-E611-80D3-AC162DC0847D`) y `/v2/guild/<id>` lo resuelve a `[AMX] Le Coq
  Sportif`: medido en vivo, y la forma sin guiones da **500**, así que el id se usa tal cual (ver
  `specs/07-spikes.md`). `worker::resolve_guild_names` los pide de 10 en 10 por sondeo, guarda
  los aciertos para la sesión y **no reintenta** los 404 (gremio disuelto). `Dashboard::guild_names`
  los publica y lo usan las tres vistas; `AlertLine::guild` sigue siendo el id porque es contra
  lo que compara «clanes que sigo», y el nombre va en `guild_name`.
- **La lista de clanes no dejaba elegir**: la historia se dibujaba *debajo* de una lista más
  larga que la ventana, así que elegir parecía no hacer nada. La lista vive ahora en una
  `ChildWindow` con scroll de 150 px y la historia queda siempre a la vista.
- **Los avisos no desaparecían**: el widget compacto enseña los últimos 10 minutos
  (`alerts::WIDGET_MINUTES`, función pura `within`) y el registro completo se queda en la ventana
  nueva. El pop-up no cambia.
- **El selector de equipos dentro de MvM enseñaba un equipo por color**: en WvW el worker pedía
  solo el matchup del jugador (~44 KB). `wants_full_list(team_id, selector_en_pantalla)` pide los
  nueve cuando el selector está abierto.
- **Filtro nuevo en objetivos del matchup**: casilla «Clanes que sigo», que es lo que faltaba
  para ver solo lo tuyo sin perder los demás mapas.
- Verificado: 60 tests en el addon (4 nuevos), 86 en `core`, 14 en `api`, 6 en `store`; `fmt` y
  `clippy -D warnings` limpios. Sin probar en el juego: los clics reales en la lista (la causa
  era de maquetación y el cambio se ve, pero conviene confirmarlo).

## HECHO – 2026-10-31 · Click-through con atajo y registro de avisos (T-13b; 0.11.2)

- **Bloqueo = click-through**: `ui::window::draw` añade `Window::no_inputs()` cuando la ventana
  está bloqueada, así que imgui no le da ni un clic y durante una pelea todo llega al juego. El
  menú contextual sigue teniendo la opción (renombrada *Click-through*), pero bloqueada no se
  puede abrir: la vuelta es el atajo o los ajustes.
- **Atajo `Alt+Shift+L`** con la API de keybinds de Nexus (verificada en
  `nexus/src/api/keybind.rs`, tag 0.12.0): `register_keybind_with_string` +
  `keybind_handler!`, revertido al descargar. El handler corre en el hilo de entrada de Nexus y
  **solo levanta un `AtomicBool`**; el render lo consume y aplica el cambio con el candado de la
  configuración ya cogido, sin invertir el orden de candados. `ui::toggle_locked` es pura y
  tiene tres tests: bloquea todas, con una bloqueada bloquea todas, y **no resucita** una
  ventana cerrada.
- Trampa cara, ya en `AGENTS.md` §11.11: `config.window(key)` **crea** la entrada con
  `visible = true`. La primera versión del atajo resucitaba las ventanas cerradas. Y volcar el
  `Edits` entero cada frame deshacía la `×` de los propios ajustes; ahora cada fila lleva
  `changed` y solo se escribe lo que el usuario tocó.
- **Ventana «Registro de avisos»** (`alerts::LOG_KEY`): la lista completa que el worker guarda
  (50), con scroll, en una ventana más grande. La lista de ventanas de los ajustes ya no es una
  enum a mano: `ui::window_keys()` y una casilla de visible + otra de click-through por ventana.
- Bug de paso: `settings::draw_inline` (el panel de opciones de Nexus) no escribía de vuelta
  avisos, pop-up ni clanes seguidos; ahora copia todo.
- Verificado: 56 tests en el addon (3 nuevos), 86 en `core`, 14 en `api`, 6 en `store`; `fmt` y
  `clippy -D warnings` limpios; trazabilidad y LOC en verde. Sin probar dentro del juego: que
  `Alt+Shift+L` no choque con un atajo del jugador — se puede reasignar en Nexus.

## HECHO – 2026-10-31 · Ventanas cerradas que no se podían reabrir (0.11.1)

- **Lo reportó el usuario en Bastión de la Armisticia** («no veo el widget de avisos»). El gate
  hace bien en ocultar el overlay ahí: el Bastión es `map_type 18` (salón), y solo se pinta en
  9/10/11/12/15. El problema real era otro: su `settings.json` tenía `visible = false` y
  **no había ninguna forma de volver a abrir una ventana cerrada** con la ×.
- `ui/settings.rs` gana una sección **Ventanas** con una casilla por ventana (Avisos, Claims
  del matchup, Clanes) y una línea que explica que el overlay solo se dibuja en los mapas de
  Mundo contra Mundo. Las claves nuevas están en los cuatro idiomas.
- Verificado: 53 tests en el addon, 86 en `core`, 14 en `api`, 6 en `store`; `fmt` y
  `clippy -D warnings` limpios; trazabilidad y LOC en verde (avisos en 569/679/584 líneas);
  DLL de 1 600 512 bytes. Ninguna de estas comprobaciones necesita el juego.

## ARCHIVADO – 2026-10-31 · Snapshot de la fase T-31 (se deja como histórico)

- Hecho sin necesitar al usuario: publicación automatizada (T-34), caché en disco (T-11b),
  gate de visibilidad, modelos de las tres vistas y el check de trazabilidad (ver abajo).
- Siguiente sin el juego: T-11c (hilo worker con parada limpia y estado de salud) y favoritos
  (RF-73). Cuidado con RF-32a: identificar garrison/colinas/bahía requiere criterio que aún no
  está verificado.
- **Requiere GW2**: T-12 (leer `NexusLink`/`Context`), T-35, T-01, T-02b, T-25.
- Decidido: **sí** a la feature `rtapi` (RF-44); su coste en DLL se mide en T-02b.
  *(Revisado después: se eliminó en 0.9.1 — no aportaba tamaño. Esta línea se conserva como
  histórico del plan de aquel momento; ver «Fuera RTAPI» más abajo.)*
- CI: el job `deny` seguía en curso al cerrar esta entrada; `license-files = []` es la
  primera prueba para las licencias de `nexus`. Si vuelve a fallar, hay que calcular el hash
  real de los ficheros de licencia, no seguir probando variantes.

- Trampa para quien siga: **la URL `raw/main` tarda en apuntar al commit nuevo**. Tras
  publicar 0.3.0, `raw.githubusercontent.com/.../main/gw2_lurker.dll` seguía sirviendo la DLL
  de 0.2.0 (1 161 728 bytes) mientras el commit (`/419e857/...`) y el asset de la release ya
  devolvían los 1 200 640 correctos, incluso añadiendo `?v=0.3.0`. No es LFS ni un fallo del
  push: es caché del endpoint. Por eso `release.ps1` verifica y **falla** en vez de dar por
  bueno un HTTP 200. Se reintenta con `-VerifyOnly` pasados unos minutos; no se cambia la URL
  ni la versión para taparlo.

## HECHO – 2026-10-31 · Traducción por equipo, botón Grabar y ruta inferida (T-40; 0.10.1–0.11.0)

- **Bug de `bugs.md`**: la traducción se guardaba por idioma, no por equipo, así que se arrastraba
  a otros servidores y no volvía al original. `Config::team_translations` indexa por **id de
  equipo → idioma**; el campo propone la traducción de la tabla cuando no hay una propia, y las
  formas antiguas (`team_names`, `team_alias`) se leen y se migran al guardar. Añadido botón
  **Grabar** que fuerza la escritura de `settings.json`.
- **T-40 parcial**: `core::route` arma la ruta de un clan en un mapa (claims en orden) con origen
  inferido por precedencia **fortaleza → fortaleza lateral → reaparición**, y su motivo se
  escribe siempre en la UI (ADR-005: «Ruta inferida a partir de las reclamaciones; no es el
  recorrido real»). Seis tests puros. La ventana de clanes ya la muestra en texto.
- **Decisión pendiente de spec**: la API marca colinas y bahía como `Keep`, igual que la
  guarnición, así que «desde su fortaleza» es honesto pero no distingue cuál. Afinarlo requiere
  identificar el keep de cada mapa por id, y eso no está verificado: queda anotado en T-40.
- Verificado: 53 tests en el addon, **86** en `core` (6 nuevos), `clippy -D warnings` limpio,
  trazabilidad en verde. DLL de 1 590 272 bytes (objetivo 2,2 MB).

## HECHO – 2026-10-31 · Presupuesto a 2,2 MB, lista de clanes e historial (T-37, T-38; 0.10.0)

- **ADR-013**: el objetivo de `check_size.sh` pasa a **2 306 867 bytes (2,2 MB)**; el límite
  sigue en 2,5 MB. Actualizados CI, `04-engineering-standards.md`, RNF-09, ADR-008 (nota de
  recalibrado) y el espejo `specs/adr/`. Con 1,57 MB quedan ~700 KB de aire.
- `worker::Dashboard::history`: los `Claim` de la sesión, con nombre, tipo y dueño resueltos, en
  un `Vec<ClaimRow>` con tope de 2000 (el `store` guarda el registro completo). Se construye con
  la función libre `history_row`, **fuera** del candado del dashboard, porque en ese bucle
  también se escribe en disco.
- `ui::guilds` (T-37 + T-38 en una ventana): clanes ordenados por la reclamación más reciente,
  con el número de claims, la hora y el mapa de la última; **estrella para seguir** (RF-73), que
  es lo que hace que ese clan avise en todos los mapas del matchup. Al pinchar uno se abre su
  historial, **separado entre «mapa actual» y «otros mapas»**, en orden cronológico.
- Verificado: 52 tests en el addon (2 nuevos), 80 en `core`, `clippy -D warnings` limpio,
  `check_loc` y `check_traceability` en verde. DLL de 1 571 328 bytes, muy por debajo del nuevo
  objetivo.

## HECHO – 2026-10-31 · Fuera RTAPI, y `bugs.md` como bandeja obligatoria (0.9.1)

- **`rtapi` eliminado** de las features de `nexus-rs` y todo su código (`i18n::game_language`).
  El idioma real del juego llega por Unofficial Extras, así que RTAPI era un respaldo que nunca
  se usaba. Aviso honesto: **no adelgazó la DLL** (1 547 776 → 1 548 288 bytes; la variación es
  de disposición del código con LTO). El problema de tamaño sigue abierto y habrá que atacarlo
  de otra forma (subir el objetivo o recortar de verdad).
- **Bug del usuario**: el selector de equipo mostraba «EU (0)» / «NA (0)» y ningún equipo.
  Causa real: el worker solo pedía matchups **dentro de WvW**, así que con los ajustes abiertos
  fuera de un mapa no había equipos; y el contador se pintaba aunque no hubiera datos.
  Arreglado: `worker::set_teams_needed` (lo activa el render mientras los ajustes están
  disponibles) refresca la lista cada 10 min sin sondear el matchup, y el `(0)` solo aparece
  cuando hay datos.
- **Nuevo flujo obligatorio**: `bugs.md` es la bandeja de fallos del usuario; se lee al empezar y
  al terminar, **no se borra** y cada entrada se marca `[x]` con la versión. Escrito en
  `AGENTS.md` §9 y en la lectura obligatoria §1.
- Verificado: `clippy -D warnings` limpio, 50 tests en el addon, 80 en `core`.

## HECHO – 2026-10-31 · Limpieza y vista de claims del matchup (T-36, 0.9.0)

**Limpieza**
- Borrado `bugs.md` (el fallo de la región quedó arreglado y registrado en esta bitácora).
- `arc_team` tenía 90 líneas que nadie usaba (una `Calibration` para un futuro detector de
  cambio de equipo): fuera. Queda lo que sí sirve, y ahora `load` registra en el log el
  `team` que reporta arcdps (`arcdps team N` o `no arcdps team`), que es lo que hace
  comprobable un informe de «estaba en el verde».
- Quitados dos `#[allow(dead_code)]` que ya no hacían falta (`net::with_timeout`,
  `team::roster`) y el de `ffi::http`, que ya usa el worker.

**T-36**
- `ui::matchup`: ventana con los cuatro mapas plegables, cada fila `hora · objetivo · clan`
  pintada con el color del dueño, y el mapa actual marcado con «estás aquí».
- Filtros (estado de sesión, todos activos al empezar): campamentos, torres,
  fortalezas/castillos y «solo alto valor». `Filters::accepts` y `rows_of` son puros y se
  prueban sin Nexus.
- Verificado: 50 tests en el addon (4 nuevos), 80 en `core`, `clippy -D warnings` limpio.
  DLL de 1 547 776 bytes: **25 KB por debajo del objetivo de 1,5 MB**.

## HECHO – 2026-10-31 · Filtro de avisos por mapa y clanes, con pop-up temporizado (0.8.0)

- `core::alerts::Filter` (puro, 6 tests): casillas Eternos / Alpino azul / Alpino verde /
  Desierto rojo / «el mapa en el que estoy» / «los clanes que sigo», **todas activas por
  defecto**. Un clan seguido avisa en cualquier mapa del matchup; si no, manda el mapa.
  `MapRole::from_map_id` (38/95/96/1099, comprobados) decide qué es «mi mapa».
- El **filtro se aplica en el worker** (`Dashboard::recent_alerts(&filter, &follow, current,
  limit)`), no en el render: así el hilo de dibujo sigue sin recorrer datos.
- **Pop-up** aparte de la lista: `alerts::draw_popup` dibuja arriba en el centro lo recibido
  hace menos de `popup_secs` (por defecto **2 s**, rango 0–5, `0` = desactivado) y desaparece
  sin quitar nada de la lista. `popup_lines` es pura y se prueba con horas fijas; los claims
  `late` del baseline no hacen pop-up.
- `AlertLine` lleva ahora `received` (cuándo lo supo el addon, no cuándo pasó) y `map_role`.
- En ajustes: las seis casillas, el deslizador de segundos y la **lista de clanes a seguir** con
  los que tienen algo reclamado en el matchup, persistida en `followed`.
- Verificado: 47 tests en el addon (5 nuevos), 80 en `core` (6 nuevos), `clippy -D warnings`
  limpio. DLL de 1 532 928 bytes.

## HECHO – 2026-10-31 · Widget de avisos (T-13 parcial, 0.7.0)

- `ui::alerts`: ventana con los últimos avisos, **el más reciente arriba**, con hora, nombre del
  objetivo y color del nuevo dueño (`Rojo → Azul`), reclamaciones (`Zanja de Mendon reclamado
  por YaK`) y liberaciones. Pasa por el contrato de ventana, así que se mueve, se pliega, se
  bloquea y tiene opacidad como las demás.
- `worker::Dashboard::recent_alerts(limit)` resuelve los ids a nombres **en el worker** y devuelve
  líneas ya listas (`AlertLine`), para que el render no busque en la tabla de objetivos ni
  mantenga el candado del dashboard mientras dibuja.
- Verificado: 43 tests en el addon (4 nuevos), 73 en `core`, `clippy -D warnings` limpio.
- Trampa (otra vez la regla 3): `describe` llamaba a `i18n::text`, que lee el idioma activo por
  RTAPI y revienta en un test. Ahora el idioma entra como parámetro y se prueba con `Language::Es`.
- Pendiente de T-13: fade de los avisos antiguos, modo bloqueado click-through con atajo
  (`Alt+Shift+L`) y la ventana de log ampliada (T-13b).

## HECHO – 2026-10-31 · «Alias» pasa a «Traducción», prellenada (0.6.0)

- El campo se llama **Traducción** y viene **prellenado** con el nombre en el idioma del juego
  (el efectivo, no el de Ctrl derecho, para que mantenerlo pulsado no reescriba nada).
- Se guarda **por idioma** en `team_names: {"es": "…"}`: cambiar el idioma del juego no muestra
  el nombre español encima del francés. `Config::custom_name` / `set_custom_name` son las dos
  únicas puertas; escribir exactamente el nombre automático **no** crea entrada, así que el
  campo sigue las traducciones incluidas cuando el jugador no corrige nada.
- El `team_alias` único de 0.5.x se sigue leyendo (`custom_name` cae a él) y se migra a
  `team_names` en cuanto se guarda, para no perder lo que ya estaba escrito.
- `team::display_name` es ahora la única forma de pintar el nombre del equipo (ventana de estado
  y desplegable), así que no hay dos criterios.
- Verificado: 39 tests en el addon (4 nuevos), 73 en `core`, `clippy -D warnings` limpio.
- Trampa: al leer el idioma activo se toca `RealTimeApi` y eso revienta en un test
  (`AddonApi::get()`), así que la regla recibe el idioma como parámetro; es la regla 3 del
  §10 otra vez.

## HECHO – 2026-10-31 · Nombres de equipo desde `data/equipos.xlsx` (0.5.1)

- `scripts/merge_team_names.ps1`: lee el `.xlsx` sin dependencias (es un ZIP con XML,
  `sharedStrings` + `sheet1`), detecta las columnas por la cabecera, normaliza erratas y escribe
  `data/team-names.json`. Re-ejecutable cuando cambie el Excel; `-DryRun` para ver el resumen.
- **Indexado por nombre inglés, no por id**: los nombres de equipo rotan cada temporada y los
  ids no, así que una traducción debe sobrevivir a que un id cambie de nombre y volver si el
  nombre regresa. `core::teams::translations()` es el pool y `name_in` lo consulta **después**
  del mapa `names` por id (que es donde irían los nombres oficiales del juego).
- Resultado: **21 nombres extraídos**, **12 coinciden** con los 33 vivos y 9 son de una
  temporada anterior (se conservan igual). Errata corregida: `Modremoth's → Mordremoth's`.
- Aviso importante para el usuario: el Excel son traducciones **literales**
  (`Great House Aviary → Pajar de la Gran Casa`), y el juego en español dice *Bosquemugoso*;
  `Lutgardis Conservatory` (*Invernadero de Lutgardis*) ni aparece. Sirven de etiqueta, y cada
  entrada lleva `source: "spreadsheet"` para distinguirlas de las oficiales cuando las haya.
- Trampa: el pool es global, así que un test que antes esperaba inglés para un nombre sin
  traducción alemana ahora recibe la del Excel; los tests usan a propósito un nombre ausente
  (`Moogooloo`) para probar la caída a inglés.
- Verificado: 73 tests en `core` (2 nuevos), 36 en el addon, `clippy -D warnings` limpio.

## HECHO – 2026-10-31 · Selector de equipo en tres pasos (0.5.0)

- `ui::settings` pide **región → color → equipo**. La lista sale de `worker::dashboard().candidates`
  (los matchups vivos), filtrada con `core::select::of_region_and_color`: con los datos del
  2026-10-31 son **5 filas en EU y 4 en NA** en vez de 27. Cada fila muestra `nombre (id) · 2-2`.
- La región se ofrece solo si tiene matchups vivos; el color va con su color real en la lista,
  como el panel de WvW del juego. Arranca en la región y el color del equipo guardado; lo que el
  jugador elija se recuerda durante la sesión (`Narrowing`, no se persiste: es una forma de
  navegar, no una preferencia).
- Si el worker aún no ha sondeado, la ventana lo dice; si la combinación no tiene equipos,
  también, en vez de un desplegable vacío sin explicación.
- **Orden de candados**: los candidatos se copian del dashboard **antes** de bloquear la
  configuración, y el worker nunca tiene el dashboard mientras pide la configuración. Así no se
  repite el interbloqueo de 0.4.2.
- Verificado: 36 tests en el addon (4 nuevos del selector), 71 en `core`, `clippy -D warnings`
  limpio, DLL de 1 469 952 bytes. Datos reales comprobados con `curl`: 5 matchups EU, 4 NA.

## HECHO – 2026-10-31 · Ajustes en Documentos (como los vecinos) y nombre de personaje

- `addon::ffi::paths::documents()`: `SHGetKnownFolderPath(FOLDERID_Documents)` + `CoTaskMemFree`
  por `extern` (sin el crate `windows`), porque Documentos está redirigido a OneDrive y leer el
  registro daría otra ruta. Comprobado en esta máquina con un test que exige que la carpeta
  exista.
- Los ajustes pasan a `<Documentos>\Guild Wars 2\addons\gw2_lurker\settings.json`, igual que
  `timers_wvw` y `gw2_builds_wvw`. Ventaja añadida: escribir ahí **no necesita administrador**,
  al contrario que `Program Files`. Si Documentos no se pudiera resolver, se cae a la carpeta de
  Nexus para no perder los ajustes.
- Migración automática una sola vez desde la carpeta antigua (`settings.json` o `config.json`),
  sin borrar el original; y `install.ps1` ya informa de la ruta correcta.
- Migrada a mano la carpeta de esta máquina: `settings.json` + `.bak`.
- **Segundo bug encontrado al leer el fichero migrado**: la clave del roster era el JSON de
  identidad completo (`{"name":"Jhoin",...}`), no el nombre, porque `read_identity_string()`
  devuelve ese blob. Ahora se usa el campo `name` de Mumble con el JSON solo como respaldo
  (`ffi::identity_name`, función pura con tests). El roster migrado quedó corregido a
  `{"Jhoin": 11011}`.
- Verificado: 35 tests en el addon (2 nuevos), 71 en `core`, `clippy -D warnings` limpio.

## HECHO – 2026-10-31 · Arreglado el cuelgue al entrar en un mapa (0.4.2)

- **Síntoma**: el juego se congelaba al entrar en un mapa después de elegir personaje (reportado
  por el usuario en 0.4.1).
- **Causa**: `ui::settings::draw` pedía el equipo con `team::selected()`, que hace
  `runtime()` (bloquea la configuración) mientras el render ya tenía ese mismo candado tomado.
  `std::sync::Mutex` no es reentrante ⇒ interbloqueo en el propio hilo. Se disparaba en el
  primer frame tras la selección de personaje porque es cuando `settings_available()` pasa a
  verdadero y el selector se dibuja. Mismo problema en el panel de opciones de Nexus.
- **Arreglo**: `team` tiene ahora variantes `selected_in(&Config)`, `remember_in(&mut Config)` y
  `forget_in(&mut Config)`, y los sitios que ya tienen el candado usan esas. La regla queda
  escrita en el módulo: *nunca llamar a la variante que bloquea con un guard vivo*.
- **Test**: la precedencia (override manual → roster del personaje) se comprobó con una función
  pura `resolve(manual, roster, character)`, porque en un test no hay Nexus y `character_name()`
  revienta al llamar a `AddonApi::get()`.
- Verificado: `clippy -D warnings` limpio, 30 tests en el addon, 71 en `core`; DLL 0.4.2 de
  1 461 760 bytes.

## HECHO – 2026-10-31 · Worker de sondeo y datos en vivo en la ventana de estado (T-11c)

- `addon::worker`: hilo propio, `MAY_POLL` como `AtomicBool` que escribe el render (la compuerta
  manda), parada con `stop()` y espera en tramos de 250 ms, backoff exponencial con tope de
  5 min. Red y disco **solo** en ese hilo; el render lee un `Dashboard` pequeño bajo un mutex.
- Primera vuelta con `ids=all` (~400 KB) porque aún no se sabe el equipo; después
  `/wvw/matches/{id}` (~44 KB) gracias al nuevo `Api::match_by_id`. Si el equipo guardado ya no
  juega, vuelve a `ids=all` para que el selector ofrezca los equipos vivos.
- Eventos → `SessionStore` (histórico de la sesión) y últimos 50 al dashboard; claims y clanes
  salen de `core::matchup`. Nombre de objetivos cacheado por idioma.
- La ventana de estado enseña emparejamiento, `N objetivos reclamados`, `N clanes` y hora.
- El equipo elegido se guarda **por personaje** (`team::remember`/`forget` sobre `Roster`), con
  `config.team_id` como override manual que gana siempre. Ojo: el cambio se aplica **después**
  de soltar el candado de la config, porque `team::*` lo vuelve a tomar (si no, interbloqueo).
- Verificado: `live_pipeline_builds_the_dashboard` (ignorado por defecto) recorre WinHTTP →
  `matches?ids=all` → objetivos en español → `match_by_id` → claims → ranking y pasa en 1,2 s;
  27 tests en el addon, 70 en `core`, 14 en `api`. DLL 0.4.0: **1 455 616 bytes**, todavía por
  debajo del objetivo de 1,5 MB pero con poco margen.

## HECHO – 2026-10-31 · Instalación local con install.bat / install.ps1

- Copiados del patrón de los vecinos (`gw2-timers`, `fight_analysis_danger`):
  [install.bat](/C:/Users/Usuario/OneDrive/proyectos/gw2-guild-lurker/install.bat) lanza
  [install.ps1](/C:/Users/Usuario/OneDrive/proyectos/gw2-guild-lurker/install.ps1) con
  `-NoProfile -ExecutionPolicy Bypass`, así que sirve el doble clic.
- `install.ps1`: `-Build` (tests + `cargo build --release`), `-DryRun` y `-Dest <ruta>`.
  Detecta la carpeta del juego por registro (`Install64Path`/`InstallPath`, comprobando que
  exista `Gw2-64.exe`) con `C:\Program Files\Guild Wars 2\addons` como respaldo; verifica
  tamaño (una DLL de 130 bytes sería un puntero LFS), cabecera `MZ` y MD5 antes **y después**
  de copiar, y pide elevación solo si Program Files la exige.
- Se niega a copiar con GW2 abierto (el fichero está bloqueado) y lo dice sin traza de
  excepción, que es lo que ve alguien que ha hecho doble clic.
- Trampa: lanzado desde `cmd` con `powershell` (5.1), `Get-FileHash` no se resuelve en esta
  máquina; el MD5 se calcula con `System.Security.Cryptography.MD5` y funciona en 5.1 y en 7.
- Verificado: `-DryRun`, copia real a una carpeta temporal (bytes y MD5 idénticos), rechazo
  con GW2 abierto y error claro con un destino inexistente.

## HECHO – 2026-10-31 · Nombres traducidos, herramienta idempotente y etiqueta de equipo de arcdps

- Pregunta del usuario: «saca las traducciones de gw2mists». Medido: gw2mists **solo tiene
  inglés** (`?lang=es`/`?locale=es` siguen devolviendo `Moogooloo`) y la API oficial responde
  404 en `/v2/wvw/teams`, así que no hay fuente automática. Se deja la estructura
  `names: {es, fr, de}` en `data/wvw-teams.json` para curación humana y
  `scripts/update_wvw_teams.ps1` ahora **preserva** esas traducciones (probado con `-DryRun`,
  33 equipos). `core::TeamTable::label_in("es", id)` cae al inglés si falta.
- Pregunta: «fight_analysis_danger sí sabe qué color soy». Comprobado: no lo lee del juego, lo
  saca de **arcDPS** (`Agent::is_self` + `Agent::team`; Nexus lo sirve como
  `EV_ARCDPS_SELF_JOIN`). Su propio enum etiqueta el caso vivo como `Team::Own`
  (*"without a verified color mapping"*).
- Implementado aquí `addon::arc_team`: se suscribe a `SELF_JOIN` y guarda el `team` de arcdps
  como **etiqueta opaca**, con `Calibration` para aprender el color cuando el jugador elige
  equipo y para detectar que está en otro equipo (alt o sorteo mensual). Sin dependencias
  nuevas: feature `arc` de `nexus-rs`.
- Verificado: `clippy -D warnings` limpio; 26 tests en el addon (3 nuevos), 70 en `core`
  (incluye `rf44_translated_team_names_fall_back_to_english`).

## HECHO – 2026-10-31 · Selector de equipo en dos pasos y memoria por personaje

- `core::select` (69 tests en `core`, todos con datos reales de `/v2/wvw/matches?ids=all`):
  `Selection::{Missing, Confirmed, Stale}`, `candidates`, `teams_of`, `matchups_of`,
  `rivals`, `is_team` y `Roster` por personaje.
- Respuesta a las dudas del usuario, comprobadas en el código: **el juego no puede decirte tu
  equipo** (Mumble no lo tiene; `Identity::team_color_id` es un id de tinte, no el color de
  WvW; Extras no incluye equipo; Nexus no lo expone), y **la API no traduce nombres de
  equipo** (`/v2/wvw/teams` → 404, mientras que `/v2/wvw/objectives?lang=es` sí traduce).
- Diseño aceptado: región → matchup → 3 equipos con color; solo ids de equipo (los `2xxx` de
  mundo antiguo quedan fuera); si el equipo guardado ya no juega, el selector se abre solo; y
  memoria por personaje para que la cuenta alt tenga la suya.
- Trampa: `TeamColor::ALL` está en orden `[Red, Green, Blue]` (el de la UI), no
  `[Red, Blue, Green]`; el test lo fija para que nadie lo desordene sin querer.

## HECHO – 2026-10-31 · DLL en `target\release` como los vecinos

- Duda del usuario: los proyectos hermanos dejan la DLL en `target\release\` y aquí aparecía en
  `target\x86_64-pc-windows-msvc\release\`. La causa era mía: compilaba siempre con
  `--target x86_64-pc-windows-msvc`, y Cargo anida la salida bajo el triple cuando se pasa ese
  flag. El host **ya es** ese triple, así que el flag no aportaba nada.
- Cambiado a `cargo build --release` en `scripts/release.ps1` (`-Target` queda como opción
  explícita), en el job `build-windows` de `ci.yml` y en `docs/RELEASE.md` / `AGENTS.md` §7.
- Segunda duda: había **dos** DLL, `gw2_lurker.dll` y `gw2lurker_addon.dll`. La segunda era un
  resto de antes de fijar `[lib] name = "gw2_lurker"`: Cargo nombra el cdylib como el paquete
  (`gw2lurker-addon`) si no se le dice otra cosa. El árbol viejo se ha borrado y solo queda
  `target\release\gw2_lurker.dll`.
- La build sin `--target` pesa 1 270 784 bytes (1 536 menos, misma funcionalidad: cambia la
  metadata, no el código) y es la que se ha vuelto a publicar como 0.3.2 para que el asset
  coincida con lo que produce el script.

## HECHO – 2026-10-31 · Idioma real del juego vía Extras y Nexus (T-31 cerrada en lo posible)

- Pregunta del usuario: «Nexus sabe el idioma que usas, ¿cómo lo hace?». Respuesta medida:
  Nexus **no** lo expone por API; lo guarda en `Nexus/Settings.json` como nombre visible y
  carga `Nexus/Locales/<id>_Main.json` (que sí lleva `"Identifier": "es"`).
- Implementado `addon::lang` con las dos fuentes de verdad:
  1. `nexus::event::extras::LANGUAGE_CHANGED` (feature `extras` de `nexus-rs`), que el plugin
     Unofficial Extras emite **también al inicializar** con el idioma actual: idioma real del
     juego, en vivo y sin sondeo. `revert_on_unload()` deja la desuscripción a Nexus.
  2. `Nexus/Settings.json` + `Locales/*_Main.json` como respaldo (dos lecturas de fichero en
     `load`, memorizadas; nunca en el hilo de render).
- Cadena final: preferencia → Extras → RTAPI → Nexus → idioma de Windows → inglés.
- Coste: **+15 872 bytes** de DLL (1 256 448 → 1 272 320). Nada de dependencias nuevas.
- Trampa: `event_consume!` sólo acepta una función, no una closure; con closure infiere `<()>`
  y falla con un error de tipos que no dice nada del macro.
- Verificado: `clippy -D warnings` limpio; 23 tests en el addon (5 nuevos de `lang`).

## HECHO – 2026-10-31 · Cliente HTTP real sobre WinHTTP (T-11b cerrada)

- `addon::ffi::http`: GET con WinHTTP del sistema (`WinHttpOpen/Connect/OpenRequest/Send/Receive/
  ReadData`), handles con `Drop`, timeouts de 10 s, tope de 8 MB y `SAFETY` en cada bloque.
- `addon::net::WinHttpTransport` implementa `gw2lurker_api::Transport` y traduce estados:
  2xx → cuerpo, 404 → `NotFound`, 429/5xx → `Status`.
- Se descartó un cliente con TLS embebido: habría sumado 1,5–2 MB a una DLL de 1,26 MB.
- Verificado de verdad: `cargo test -p gw2lurker-addon live_get -- --ignored` → **HTTP 200** con
  el JSON de `2-2` en 270 ms. Los tests de `split_url` cubren https/http, puerto explícito y
  URLs inválidas.
- Trampa: en una `cdylib`, `dead_code` marca como muerto todo lo que aún no se usa, y el
  transport sólo lo usará el worker (T-11c); de ahí el `#[allow(dead_code)]` temporal con
  motivo escrito al lado.

## HECHO – 2026-10-31 · Ajustes como los vecinos y escritura fuera del render

- Fichero `settings.json` en `<addons>/gw2_lurker/` (antes `config.json`), con
  `settings.json.bak`, escritura atómica y lectura del fichero antiguo para migrar.
- `addon::writer`: hilo propio que recibe copias de la config por canal, coalesce y escribe;
  la UI llama a `persist_if_changed()` como mucho cada 500 ms. En `unload` hay guardado final
  y `join`. Cumple la regla 6 sin perder ajustes si el juego se cierra de golpe.
- Ajustes también en el panel de opciones de Nexus (`RenderType::OptionsRender`), visible
  fuera de WvW: la configuración deja de depender de la compuerta.
- Diagnóstico para el caso «carga pero no dibuja nada»: en `load` se registra la ruta de
  ajustes y `mumble map/tipo/gameplay`; cada cambio de compuerta deja una línea
  `gate: map .. tipo .. gameplay .. -> show_ui ..`; y `debug_show_everywhere` (por defecto
  `false`) permite saltarse la compuerta sin tocar código.
- Verificado: `clippy -D warnings` limpio, 16 tests en el addon (uno nuevo del escritor).
- **Primera prueba real en juego (2026-10-31 15:38, versión 0.3.0)**: `Nexus.log` dice
  `Loaded addon: gw2_lurker.dll` en 695 µs con `Signature: -1282765419` y
  `WvW Lurker cargado (idioma en)`. Se creó `addons/gw2_lurker/config.json`, pero con
  `"windows": {}` y sin entrada en `Nexus/imgui.ini` ⇒ **el render no llegó a dibujar la
  ventana ni una vez**: o la compuerta la ocultaba (el usuario cerró el juego 25 s después de
  cargar, probablemente en carga/selección de personaje) o no estaba en WvW. La build 0.3.1
  deja la respuesta en el log (`gate: map .. tipo .. gameplay .. -> show_ui ..`) y añade los
  ajustes en el panel de opciones de Nexus, que no depende de la compuerta.
- Dato del mismo log: **RealTime API no está instalado** en esa máquina (no aparece en
  `Nexus.log`), de ahí el `idioma en` con el cliente en español. Añadido el respaldo por
  idioma de Windows; queda propuesto como cambio de RF-44.
- La carpeta de datos real del addon es `C:\Program Files\Guild Wars 2\addons\gw2_lurker\`
  (junto a `ControlOptions`, `WvWFightAnalysis`…), no la de Documentos: ahí `addons/` es sólo
  datos de arcdps. La DLL va a `<juego>\addons\gw2_lurker.dll`.

## HECHO – 2026-10-31 · Ventana de ajustes con selector de equipo (T-30, T-17 parcial)

- `addon::ui::settings`: ComboBox de equipo con `nombre (id)` sobre la tabla incluida, opción
  «sin equipo», alias editable, idioma, intervalo de sondeo (`InputInt`, saneado a 15–600) y
  alcance de los avisos. Se abre desde el botón *Ajustes* de la ventana de estado.
- Las ediciones se acumulan en una copia local y se escriben al terminar de dibujar, porque
  `window::draw` ya tiene prestado el estado de la ventana.
- `core::teams::sorted()` devuelve la lista ordenada por región y nombre; sin orden, el
  `HashMap` hacía bailar el desplegable entre frames. 6 claves nuevas de traducción.
- Trampa: el orden alfabético deja **EU antes que NA** (`E` < `N`), no al revés como asumí en
  el primer test; el test ahora comprueba bloques y orden, no cuántos NA hay al principio.
- Verificado: `clippy -D warnings` limpio; 15 tests en el addon, 60 en `core`, 13 en `api`,
  6 en `store`; DLL 0.3.0 publicada y verificada por bytes y MD5.

## HECHO – 2026-10-31 · Contrato de ventana compartido (T-17 parcial)

- `addon::ui::window::draw` es el único sitio que dibuja cromo: barra de título propia (texto
  traducido + id estable), `×` a la derecha, plegado con doble clic, mover/redimensionar solo
  si no está bloqueada, `bg_alpha` con la opacidad, menú contextual con *Bloquear posición*,
  *Opacidad* y *Restablecer posición y tamaño*, y tamaño mínimo 180×90.
- Estado por ventana en `config.json` (`windows.<clave>`): `visible`, `collapsed`, `locked`,
  `opacity`, `position`, `size`, con saneo (NaN, infinitos, tamaños imposibles) y prueba de
  ida y vuelta.
- La UI escribe en una copia mutable (`lib::runtime()`); el fichero se guarda en `unload`
  porque el hilo de render no puede hacer E/S. Se pierde lo movido si el juego revienta; el
  guardado diferido llega con el worker (T-11c).
- Decisión: el plegado **no** usa el `collapsed` de imgui. Con `title_bar(false)` una ventana
  plegada de imgui no dibuja nada y no habría forma de desplegarla; se controla desde nuestro
  estado y se autoajusta el tamaño.
- Verificado: `clippy -D warnings` limpio; 12 tests en el addon (4 nuevos de RF-46), 59 en
  `core`, 13 en `api`, 6 en `store`; `check_loc.sh` y `check_traceability.sh` en verde.

## HECHO – 2026-10-31 · i18n es/en/fr/de con override de Ctrl derecho (T-31)

- `core::i18n`: 36 claves indexadas por el texto inglés, columnas `[es, en, fr, de]`, `t`/`w`
  (`###clave` para que el id de imgui no cambie al traducir) y `f` con `{0}`/`{1}`. Columna
  vacía → inglés; clave desconocida → la propia clave. 9 tests, entre ellos el de completitud
  de las cuatro columnas (falla si alguien añade una clave solo en un idioma).
- `addon::i18n`: preferencia del usuario (si no es *Auto*) → `RealTimeApi::read_game().language`
  → inglés; override a inglés mientras se mantiene Ctrl derecho con el juego en primer plano.
  Registra las claves en Nexus con `set_translation` desde `load`.
- `ffi::keys`: `GetAsyncKeyState(VK_RCONTROL)` y foco por proceso en primer plano, con
  `extern "system"` contra `user32` (no se añade el crate `windows`).
- Verificado: `clippy -D warnings` limpio, 9 tests en `core`, 8 en el addon (3 nuevos), DLL de
  release **1 062 400 bytes** (+8,7 KB sobre la anterior) dentro del objetivo.
- Trampa para quien siga: la spec citaba `UiState::GAME_HAS_FOCUS`, que **no existe** en
  `nexus-rs` 0.12; lo he dejado documentado en `07-spikes.md` y corregido en `02-design.md`
  §7c. Tampoco imgui distingue Ctrl izquierdo de derecho: de ahí Win32.
- Pendiente: probar en juego que el cambio de idioma y el override funcionan (T-35) y la
  revisión de terminología por nativos (T-41).

## HECHO – 2026-10-31 · Publicación automatizada de la DLL (T-34)

- `scripts/release.ps1`: compila, comprueba `MZ` y tamaño (`check_size.sh`), escribe el `.md5`
  con `UTF8Encoding($false)` (32 bytes exactos, sin BOM ni salto), añade la excepción de LFS a
  `.gitattributes` del repo compartido, **aborta** si `git cat-file -s :gw2_lurker.dll` no
  coincide con el tamaño local (puntero LFS) o si `check-attr` no devuelve los cuatro atributos
  en `unset`, hace commit **solo** de `gw2_lurker.*`, publica la release con `--latest=false`,
  borra las releases nuestras más allá de la anterior y descarga la URL exacta de `update_link`
  para comparar bytes y MD5.
- Opciones: `-DryRun` (compila y valida, no toca lo remoto) y `-VerifyOnly` (solo verifica lo
  ya publicado, útil tras un aviso de caché).
- Corregido un fallo propio que esto destapó: la DLL se llamaba `gw2lurker_addon.dll` porque
  el `[lib]` no tenía `name`. Ahora `crates/addon/Cargo.toml` fija `name = "gw2_lurker"` y el
  `ci.yml` apunta al nombre nuevo.
- Verificado: `-DryRun` completo (DLL 1 053 696 bytes, `MZ`, MD5 calculado, tamaño dentro del
  objetivo). La publicación real y la descarga se comprueban en el mismo comando.
- Trampa para quien siga: en PowerShell, `"$size:"` dentro de una cadena se parsea como
  unidad/scope y rompe el fichero; hay que escribir `${size}`.

## HECHO – 2026-10-31 · Caché en disco de objetivos y guilds (T-11b)

- `crates/api/src/cache.rs`: `Cache` sobre el directorio del addon. Objetivos **por idioma**
  (`objectives-<lang>.json`) para que cambiar de idioma o mantener Ctrl derecho pulsado
  (RF-45) no toque la red, y `guild-<id>.json` para no repetir `/v2/guild/:id` cada sesión.
- Una entrada corrupta o ausente es un *cache miss*, nunca un error que aborte la llamada.
- `GuildInfo` ahora también implementa `Serialize` (antes solo `Deserialize`).
- Verificado: `cargo clippy --workspace --all-targets --exclude gw2lurker-addon -D warnings`
  limpio; 13 tests en `api` (4 nuevos), 50 en `core`, 6 en `store`.
- Ojo para quien siga: `write` necesita `T: Serialize + ?Sized` porque se le pasa `&[T]`.

## HECHO – 2026-10-31 · Capa de API con transporte intercambiable (T-11b parcial)

- `crates/api`: `trait Transport` (solo baja bytes) y `Api<T>` con `matches_all`,
  `match_for_team(team_id)`, `objectives(lang)` y `guild(id)`. El cliente HTTP real queda
  fuera a propósito: lo elige la medición de T-02b.
- `match_for_team` **escanea `ids=all`** en vez de usar `?world=<id>`, porque ese parámetro
  espera un id de mundo antiguo y nosotros identificamos equipos por 11xxx/12xxx.
- `ApiError` (transporte, estado, parseo, no existe) y `Backoff` exponencial con tope y
  criterio de reintento (5xx y 429 sí, 4xx no).
- 9 tests con fixtures reales: equipo 12014 → match `2-2` azul, 503, JSON truncado, idioma
  en la URL, guild con tag.
- 65 tests en total (50 core + 9 api + 6 store).

## HECHO – 2026-10-31 · Addon real: export, contexto y configuración (T-12 parcial)

- `nexus` 0.12.0 (tag fijo) con `log`, `log_filter`, `mumble` y `rtapi`; añadido `log` y
  `clock` de chrono. `deny.toml` gana `allow-git` para los tres repos de `zerthox`.
- `export!` con **`UpdateProvider::Direct`**, `DisableHotloading` y la URL de
  `gw2addonsreleases`. Comprobado dentro del binario: contiene `GetAddonDef`, "WvW Lurker",
  `gw2_lurker.dll` y `raw.githubusercontent.com`.
- `ffi/mod.rs` (único `unsafe`, con `SAFETY`): `is_gameplay()` por `read_volatile` y
  `mumble_map()`. `context.rs` los convierte en el `GameState` de `core::gate`.
- `config.rs`: JSON en `addons/gw2_lurker/config.json`, defaults, saneo de rangos (incluido
  NaN), renombrado a `config.json.corrupt-<fecha>` si está dañado, `save()` listo para T-17.
- Ventana de estado mínima, gateada por `core::gate`: solo se dibuja en WvW jugable.
- **Tamaño medido: 1.053.696 bytes (1,00 MB)**, `MZ`, bajo el objetivo de 1,5 MB de ADR-008
  y aún sin cliente HTTP. Apuntado en T-02b.
- 61 tests (50 core + 6 store + 5 addon). Clippy en verde.
- Trampa: `Utc::now()` exige la feature `clock` de chrono; y `log` es dependencia directa,
  no basta con la feature de nexus.

## HECHO – 2026-10-31 · Claims tardíos con bandera `late` (T-09)

- `Event::Claim` gana `late: bool`. Un baseline emite `Baseline` y después **un claim por
  cada objetivo reclamado**, con su `claimed_at` real y `late = true` (RF-25). Los claims
  observados en vivo llevan `late = false`.
- Consecuencia para quien consuma: el baseline de una sesión son ~41 eventos, no uno. El
  repositorio de eventos ya los guarda tal cual (T-10).
- Tests: los de baseline ahora separan eventos de ciclo de vida de los claims tardíos, y hay
  uno específico que comprueba guild y hora reales de un claim tardío.
- 50 tests de core + 6 de store.

## HECHO – 2026-10-31 · Almacenamiento por sesión (T-10 parcial)

- `crates/store`: `SessionStore` con `Record { v, at, match_id, event }` (los eventos son
  los de `core::diff`, serializados tal cual). `open()` **borra los `.jsonl` anteriores**
  (ADR-012), `append()` rota a los 2 MB (RNF-03) y `read_all()` salta líneas corruptas o de
  versión futura y las cuenta.
- Dos bugs propios cazados por los tests: la rotación reutilizaba el nombre de fichero y
  truncaba registros (ahora `sesion-000.jsonl`, `sesion-001.jsonl`…), y un test comparaba
  una fecha literal con otra calculada. Ambos corregidos.
- 6 tests de store + 49 de core. Falta la consulta por guild y rango (T-11).

## HECHO – 2026-10-31 · Favoritos y limpieza de módulos (T-39 parcial)

- `core::follow`: `FollowList` con `add`/`remove`/`toggle` idempotentes, sin duplicados y
  serializable como array JSON; `favourites_first` sube las seguidas sin desordenar el resto.
- `diff.rs` llegó a 501 líneas (aviso de RNF-10) al crecer los tests: partido en
  `diff/mod.rs` (196) + `diff/tests.rs` (244). El fichero más grande ahora es `model.rs` (375).
- 49 tests en verde; trazabilidad: 21 requisitos con test, 52 sin cubrir.
- Trampa: `cargo fmt | Select-Object -First 5` corta el proceso y deja el formateo a medias.
- Trampa: una dependencia entre crates del workspace **necesita `version`** además de
  `path`; sin ella `cargo deny` la ve como wildcard y falla el job de supply-chain.

## HECHO – 2026-10-31 · Vistas de claims y trazabilidad (T-36/37/38 parcial, T-03b)

- `core::matchup`: `claims()` (41 filas reales), `by_map()` en orden fijo, `with_kinds()`
  para los filtros, `rank_guilds()` por claim más reciente con desempate estable,
  `split_by_map()` y `guild_timeline()`. 7 tests.
- `scripts/check_traceability.sh`: 83 requisitos, 20 con test, 18 con prueba manual, 53 sin
  cubrir. Falla si un test cita un RF inexistente (probado con `rf999`), informa de los no
  cubiertos y `--strict` falla por cobertura. Añadido a la CI.
- Trampa: los comodines de la matriz (`RF-03*`) se colaban como requisito; ahora se filtran.
- 44 tests en verde.

## HECHO – 2026-10-31 · Bocetos HTML (T-32)

- `docs/boceto.html`: una página, sin red, con cuatro pestañas (alertas, mapa, guilds,
  historial) y una columna "en el juego" con el widget sobre un fondo simulado.
- **Datos reales**: 76 objetivos con coordenadas de los cuatro mapas y los 41 claims del
  match `2-2`, capturados de las fixtures. Los 39 tags de guild salen de `/v2/guild/:id`.
- Decisiones visibles que hay que validar: tamaño y contorno doble por `points_tick`,
  anillo discontinuo para "sin reclamar", lista de guilds por claim más reciente,
  separación "este mapa / otros mapas" y ruta dibujada en el último mapa con su nombre.
- `PRODUCT.md` escrito para que la skill `impeccable` tenga contexto (suposiciones marcadas).
- Detector `impeccable detect`: corregidos texto en mayúsculas, sombra difusa con borde fino
  y contraste del texto atenuado. Quedan avisos de tipo *slop* (radial-halo, grid) y tres
  de contraste que son falsos positivos: el detector no resuelve el fondo oscuro que se
  aplica con `background` + degradados; el texto real sobre `#0f151c` pasa AA de sobra.
- No verificado: que el usuario lo encuentre útil. Es justo lo que toca preguntar.

## HECHO – 2026-10-31 · Cimiento: T-03, T-03a, T-04, T-05, T-06, T-07

- **Workspace** `x86_64-pc-windows-msvc`, 4 crates, lints del workspace (`unwrap_used` y
  `expect_used` en deny), perfil release de ADR-008, licencia **GPL-3.0-only**.
- **CI** (`.github/workflows/ci.yml`): fmt, clippy, test, `check_loc.sh`, `cargo deny`,
  `cargo audit`, build release msvc y `check_size.sh`. ~1 min por push.
  - Trampa encontrada: los `.sh` estaban en modo `100644` → `exit 126`. Arreglado con
    `git update-index --chmod=+x` y llamando con `bash`.
  - Trampa encontrada: `rustsec/audit-check` compila `cargo-audit` y costaba **339 s**.
    Sustituido por `taiki-e/install-action` + `cargo audit`.
- **`core::model`** (T-04): tipos serde de `/v2/wvw/matches`, `/v2/wvw/objectives`,
  `/v2/maps`, con fixtures reales en `crates/api/tests/fixtures/`.
- **`core::teams` + `core::resolver`** (T-05): tabla `data/wvw-teams.json` embebida con
  `include_str!`, `label()` con caída a `(id)`, `resolve()` sobre `all_worlds`.
- **`core::diff`** (T-06, T-07): `Differ` con `Baseline`/`MatchChanged`/`Gap` y, cuando no
  hay baseline, `Flip`/`Claim`/`Unclaim` ordenados por id de objetivo (`Unclaim` antes de
  `Claim`).
- Verificación: `cargo fmt --check`, `clippy -D warnings` y **31 tests** en verde; CI verde.
- Commits: `73b1eaa` (T-03), `add0bf2` (T-04 + licencia), `4195f85` (T-05), `69fa39b` (T-06),
  `36506ef` (T-07), `cd6441a`/`e7a30ad` (CI), `c03e0fb` (specs + backlog).

## HECHO – 2026-10-31 · Spikes T-02 y notas verificadas

- T-02 medido de verdad: 227 peticiones, 0 errores. Cambios de propiedad con mediana 19–20 s;
  un emparejamiento inactivo pasó **22 min 40 s sin un solo cambio**. Se mantiene 30 s.
- Hallazgo: la API **no** expone nombres de equipo (404 en `/v2/wvw/teams` y
  `/v2/wvw/alliances`). Sí da `guild → equipo` en `/v2/wvw/guilds/:region`.
- Hallazgo: `coord` está en coordenadas de continente; la caja correcta es `continent_rect`.
- Todo en `specs/07-spikes.md`, sección *Resultados*.

---

## Trampas ya conocidas (no volver a tropezar)

- `owner` viaja como `"Red"`/`"Green"`/`"Blue"` (PascalCase); los contenedores usan claves
  minúsculas. Mezclarlo rompe la deserialización.
- `claimed_by` es **una** guild o `null`, nunca una lista. Las claves siempre vienen.
- Torre totalmente mejorada llega a `points_tick` 8: "alto valor" no puede ser por tipo.
- Ids de equipo 11001–11015 (NA) y 12001–12018 (EU). Prefijo del match: `1-x` NA, `2-x` EU.
- Nexus expone `NexusLink::is_gameplay`: sirve para no dibujar en carga/selección.
- `crate::config()` es la copia **del arranque** y no cambia nunca; lo que el jugador toca en la
  ventana está en `crate::runtime()`. Leer `config()` desde el render hizo que el clic derecho no
  viera el click-through, que las listas de clanes no se refrescaran y que el clic derecho no
  deshiciera nada (0.22.10).
- **La DLL instalada no se queda**: Nexus (`UpdateProvider::Direct`) la sobrescribe en cada arranque
  con la que haya publicada en `gw2addonsreleases`. Probar una versión local exige publicarla
  (`scripts/release.ps1`) o desactivar la comprobación de actualización en Nexus (0.23.1).
- **`raw.githubusercontent.com` cachea ~3 min**: el `.md5` se sirve antes que la DLL, así que una
  release recién subida parece buena si solo se mira el HTTP 200. Comprobar **bytes y MD5** de la
  descarga real (`scripts/release.ps1 -VerifyOnly`) antes de decirle al jugador que abra el juego.
- MumbleLink `name` = «Guild Wars 2», **no** el personaje: el personaje está en `identity`
  (`{"name":"Jhoin",…}`). Y `shard_id` en WvW vale `0x80000001` (medido 2026-10-31): no es el mundo
  ni la partida, así que no sirve para detectar el emparejamiento.
- Un reclamo del API sigue ahí aunque el objetivo cambie de color: hay que comparar `last_flipped`
  con `claimed_at` o el mapa pinta un anillo en un campa de otro color (0.22.10).
- Publicar: `UpdateProvider::Direct` + `DisableHotloading`, MD5 de 32 hex sin BOM, excepción
  de LFS para la DLL, tag propio con `--latest=false`. Ver `docs/RELEASE.md`.

## Bloqueos

- **T-01, T-02b, T-25**: requieren ejecutar dentro de GW2 (carga, click-through, tamaño de
  DLL con TLS, comportamiento de actualización). No se simulan.
- **T-29 y T-30**: la matriz de trazabilidad las cita y no existen en `03-tasks.md`.
