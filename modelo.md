# Modelo del código: dónde tocar cuando algo falla

Mapa del repositorio para localizar el código sin leerlo entero. **Uso:** busca con `grep` el
síntoma en la tabla de abajo o el nombre del fichero o de la función (`### \`crates/...` abre la
ficha de cada fichero) y abre solo ese rango.

- La parte de arriba (hasta el marcador `GENERADO`) está escrita a mano: arquitectura, hilos, flujo
  de datos y síntomas.
- La de abajo se regenera con `pwsh -File scripts/gen_modelo.ps1` a partir de los `//!` y `///`
  del código; lo que no tiene `///` toma su descripción de `scripts/modelo-descripciones.txt`.
  Hazlo tras cambios de estructura (ficheros, funciones o tipos nuevos). Para cada
  fichero da: qué hace, sus items (función, tipo, método, constante) con su descripción, de qué
  módulos internos depende, qué APIs externas toca (Nexus, Win32, GW2 API, disco, hilos, imgui) y
  sus tests (los tests se listan por nombre, que ya dice qué comprueban).
- Todos los items tienen descripción. El generador dice cuántos quedan sin ella; si sale > 0,
  `pwsh -File scripts/gen_modelo.ps1 -ListMissing` da las claves y se añaden al `.txt` (o un `///`
  en el código, que tiene prioridad).
- La spec manda sobre este fichero (`specs/`); este solo dice **dónde** está cada cosa.

## Arquitectura en una pantalla

```
crates/core   lógica pura: modelo serde de la API, diff de instantáneas, avisos, rutas, i18n.
              Sin red, sin Nexus, sin imgui, sin unsafe. Todo lo probable vive aquí.
crates/api    cliente HTTP de la API pública de GW2 (transporte inyectable) + parseo.
crates/addon  la DLL (cdylib gw2_lurker.dll): Nexus, imgui, hilos, disco. unsafe solo en ffi/.
data/         wvw-teams.json y team-names.json (incrustados en la DLL), índice de nombres de clan.
```

Dependencias: `addon → api → core` y `addon → core`. `core` no depende de nadie del workspace.

## Hilos y quién escribe qué

| Hilo | Entra por | Hace | Nunca |
|---|---|---|---|
| Render (juego) | `ui::register` → `draw_widget`, `draw_options` | Dibuja con imgui, lee `worker::dashboard()` y `runtime()`, llama a `persist_if_changed()` | Red, disco, bloqueos largos |
| Worker | `worker::start` → `run` / `poll_once` | Sondea `/v2/wvw/matches/{id}` (y `ids=all` como mucho 1/60 s), aplica el `Differ`, rellena `Dashboard`, resuelve nombres, relee el atajo (`keybinds::refresh`) y comprueba actualización de la DLL | Dibujar |
| Writer | `writer::Writer::start` | Escribe `settings.json` cuando la config cambia | — |
| Background | `background::spawn` | Tareas sueltas: índice de nombres (`guild_index`), búsqueda de clanes (`search`) | — |

Estado global (en `addon/src/lib.rs` y `worker.rs`):
- `CONFIG`: la config **del arranque**; no cambia.
- `RUNTIME`: la editable, la que hay que leer en la UI. Se accede con `runtime()`.
- `DASHBOARD`: lo que el worker publica para la UI.
- `WRITER`: el escritor de `settings.json`.
- `SAFE_MODE`: por qué la config no se pudo leer, si falló.

`Mutex` no es reentrante: con el candado de `RUNTIME` cogido no llames a `runtime()`,
`team::selected()`, `team::remember()` ni `request_save`. Usa las variantes `*_in(&Config)`.

## Flujo de un sondeo

1. `context` y `ffi::mumble_*` dicen si estás en WvW y en qué mapa, y `core::gate` decide si se
   puede dibujar y sondear. El render lo comunica con `worker::set_may_poll`.
2. `worker::poll_once`:
   1. `fetch_plan` y `needs_all_now` deciden si se piden los nueve emparejamientos o uno solo.
   2. `gw2lurker_api::Api` hace la petición por `net` y `ffi::http` (WinHTTP).
   3. `core::detect` y `team` sacan tu equipo, con `arc_team` y `select` para los empates, y
      `assignment` detecta las reasignaciones de equipo.
   4. `core::diff::Differ` compara la nueva instantánea con la anterior y produce eventos
      (capturas y reclamos), siguiendo los escenarios de `specs/06-diff-scenarios.md`.
   5. `core::alerts` filtra qué eventos merecen aviso, y `Dashboard` acumula `history` (en memoria,
      ADR-015) y `claims`.
3. La UI, en `ui/mod.rs` (`draw_tabs`), pinta Avisos (`ui/alerts`, `ui/popup`), Mapa (`ui/map`,
   `ui/map/route`), Clanes (`ui/guilds`, `ui/follow`, `ui/matchup`), Registro y Ajustes
   (`ui/settings`). `ui/navigation` decide los saltos entre pestañas.

## Síntoma → dónde mirar

| Síntoma | Primero | Después |
|---|---|---|
| No avisa de una captura, o avisa de más | `core/src/alerts.rs` | `core/src/diff/mod.rs`, `addon/src/alerts.rs`, `ui/alerts.rs` |
| El emergente no sale o se repite | `ui/popup.rs` | `ui/popup_stack.rs` (`tick`) |
| Reclamos fantasma, tardíos o duplicados | `core/src/diff/mod.rs` + `specs/06-diff-scenarios.md` | `core/src/matchup.rs` |
| Equipo o emparejamiento equivocado | `addon/src/team.rs`, `core/src/detect.rs` | `core/src/resolver.rs`, `core/src/select.rs`, `arc_team.rs`, `assignment.rs`, `core/src/teams.rs` |
| No sondea, sondea de más o «Degradado» | `worker.rs` (`run`, `poll_once`, `retry_delay`, `should_skip_poll`) | `core/src/gate.rs`, `context.rs`, `api/src/lib.rs`, `ffi/http.rs` |
| Lista de clanes, orden o «último mapa» | `core/src/matchup.rs` | `ui/guilds.rs`, `worker.rs` (`session_guilds`, `last_map_of`) |
| Favoritos: añadir, quitar o mostrar | `ui/follow.rs` | `core/src/follow.rs`, `search.rs`, `config/mod.rs` |
| Nombre de clan sale como id | `names.rs`, `session_names.rs` | `guild_index.rs`, `catalog.rs` (`resolve_guild_names`) |
| Al pulsar un clan va a la pestaña o al mapa equivocado | `ui/navigation.rs` | `ui/mod.rs` (`draw_tabs`, `WANT_TAB`, `snap_to_current`, `map_notice`) |
| Mapa mal proyectado o con iconos raros | `core/src/project.rs` | `ui/map.rs`, `ui/icons.rs`, `catalog.rs` |
| Ruta inferida o paso 0 | `core/src/route.rs` | `ui/map/route.rs` |
| Atajo o click-through | `keybinds.rs`, `ui/mod.rs` (`on_lock_keybind`, `toggle_click_through`) | `ffi/keys.rs`, `ui/status.rs`, `ui/window.rs` |
| Ventana: tamaño, posición, opacidad o tema | `ui/window.rs` | `ui/theme.rs`, `config/mod.rs` |
| Textos o idioma | `core/src/i18n/table*.rs` | `addon/src/i18n.rs`, `lang.rs` |
| `settings.json` no se guarda o se corrompe | `writer.rs`, `config/mod.rs` | `lib.rs` (`persist_if_changed`, `SAFE_MODE`) |
| Hora mal mostrada | `clock.rs` | — |
| Se congela el juego | Un candado reentrante (`runtime()` con el candado cogido) o E/S en el render | `lib.rs`, `ui/*` |
| Actualización o descarga de la DLL | `lib.rs` (`check_dll_update`), `md5.rs` | `scripts/release.ps1`, `docs/RELEASE.md` |
| Log o diagnóstico | `diagnostics.rs` | `worker.rs` |

## Ficheros de datos

- `data/wvw-teams.json`, `data/team-names.json`: identidad y nombres de los equipos. Van
  incrustados en la DLL (`core/src/teams.rs`) y se actualizan con `scripts/update_wvw_teams.ps1`.
- `data/guild-names.json` y `data/gw2_lurker_guilds.zip` (+ `.md5`): índice de nombres de clan.
  Se distribuyen fuera de la DLL (ADR-016), se empaquetan con `scripts/pack-guild-index.ps1` y se
  descargan con `addon/src/guild_index.rs`.
- `crates/api/tests/fixtures/`: respuestas reales de la API (`match_2-2.json`, `maps_wvw.json`…)
  para los tests. Se capturan con `scripts/capture_fixtures.sh`.
- En el juego: `addons/gw2_lurker/settings.json` (config) y `temp.json` (nombres de la sesión).
  Nexus guarda los atajos en `addons/Nexus/InputBinds.json`.

<!-- GENERADO: no editar debajo; se rehace con scripts/gen_modelo.ps1 -->

Generado el 2026-11-10 21:58 desde el commit `14d674b`.

## Crate `core`

- `crates/core/Cargo.toml`

### `crates/core/src/alerts.rs` (551 líneas)

Which captures deserve an alert (RF-19, RF-19a).

**Contenido:**

- `pub enum FlipPerspective` — A capture described from the player's team's point of view.
- `pub fn flip_perspective(…)` — Classifies an ownership change without guessing when the player's team is unknown.
- `pub struct Structures` — Which structures deserve an alert.
- `impl Default for Structures`
  - `fn default() -> Self` — Por defecto avisa de los cuatro tipos de estructura: campamento, torre, fortaleza y castillo.
- `impl Structures`
  - `pub fn wants(self, kind: ObjectiveType) -> bool` — Whether a change of owner of this kind of objective is worth an alert.
- `pub struct Filter` — What the player wants to be told about.
  - `pub struct ures` — Which structures deserve an alert. Ruins never do (request of 2026-11-01).
- `impl Default for Filter`
  - `fn default() -> Self` — Por defecto se avisa de **lo del mapa en el que estás y de los clanes que sigues**, con las cuatro estructuras marcadas (petición del jugador, 2026-11-01). Antes venían marcados también los cuatro map…
- `impl Filter`
  - `pub fn wants_role(&self, role: MapRole) -> bool` — Whether the fixed-map option for this role is ticked.
  - `pub fn notify(…)` — Whether a capture on `role` by `guild` is worth an alert.

**Usa (interno):** `crate::follow`, `crate::model`

**Tests (13):** `rf19f_flip_from_my_team_is_lost`, `rf19f_flip_to_my_team_is_taken`, `rf19f_other_or_unknown_team_is_a_change`, `rf19a_by_default_only_my_map_and_the_guilds_i_follow`, `rf19_unticking_a_map_silences_only_that_map`, `rf19a_the_map_i_am_on_can_be_the_only_source`, `rf19a_only_the_followed_guilds_is_one_switch`, `rf73_a_followed_guild_is_interesting_on_every_map`, `rf73_unticking_followed_guilds_silences_them`, `rf19_unknown_map_never_alerts`, `rf19e_ruins_never_alert`, `rf19e_each_structure_can_be_turned_off`, `rf19e_an_unknown_kind_is_let_through`


### `crates/core/src/detect.rs` (143 líneas)

Which matchup is the player actually in, worked out from the game itself (RF-03a).

**Contenido:**

- `pub struct Detected` — Where the player is playing, as the game tells it.
- `pub fn of_shard(matches: &[Match], shard: u32, colour: Option<TeamColor>) -> Option<Detected>` — Finds the matchup of a shard, preferring the one where the colour also matches.

**Usa (interno):** `crate::model`

**Tests (3):** `rf03a_the_shard_says_which_matchup_the_player_is_in`, `rf03a_a_shard_nobody_knows_detects_nothing`, `rf03a_the_colour_is_reported_even_when_it_does_not_match`


### `crates/core/src/diff/mod.rs` (260 líneas)

Snapshot lifecycle: baseline and match change (RF-03d, RF-04, RF-26).

**Contenido:**

- `pub struct Snapshot` — One poll result, flattened so the diff does not care about map grouping.
- `impl Snapshot`
  - `pub fn from_match(m: &Match, taken_at: DateTime<Utc>) -> Self` — Builds a snapshot from an API match payload.
- `pub enum BaselineReason` — Why a snapshot became a baseline.
- `pub enum Event` — What the application needs to know after a poll.
- `pub struct DiffConfig` — Tuning for the differ.
- `impl Default for DiffConfig`
  - `fn default() -> Self` — Intervalo de sondeo por defecto de 30 s; de él sale el umbral de hueco (3 intervalos).
- `impl DiffConfig`
  - `pub const GAP_FACTOR` — Multiple of the interval above which a hole counts as a gap (RF-26).
  - `pub fn gap_threshold(&self) -> Duration` — Threshold as a duration.
- `pub struct Differ` — Stateful differ: keeps the previous snapshot and decides the lifecycle events.
- `impl Differ`
  - `pub fn new(cfg: DiffConfig) -> Self` — Creates a differ with the given tuning.
  - `pub fn prev(&self) -> Option<&Snapshot>` — Last accepted snapshot, if any.
  - `pub fn reset(&mut self)` — Forgets the previous snapshot; the next one is a session baseline again.
  - `pub fn push(&mut self, snapshot: Snapshot) -> Vec<Event>` — Feeds a new snapshot and returns the lifecycle events it triggers.
- `fn late_claims(snapshot: &Snapshot, previous: Option<&Snapshot>, out: &mut Vec<Event>)` — Every claim visible in a baseline snapshot, ordered by objective id (RF-25).
- `fn flips_and_claims(prev: &Snapshot, cur: &Snapshot, out: &mut Vec<Event>)` — Compares two comparable snapshots and appends the observed changes.
- `mod tests`

**Usa (interno):** `crate::matchup`, `crate::model`


### `crates/core/src/diff/tests.rs` (492 líneas)

**Contenido:**

- `const MATCH` — Fixture real `match_2-2.json` del crate api, incrustada con `include_str!`.
- `fn parsed() -> Match` — Deserializa la fixture `MATCH` a `Match`.
- `fn at(offset_secs: i64) -> DateTime<Utc>` — Instante fijo (época 1 800 000 000 s) más `offset_secs`; da tiempos deterministas a los tests.
- `fn snap(match_id: &str, offset_secs: i64) -> Snapshot` — `Snapshot` de la fixture con otro id de partida y tomada en `at(offset_secs)`.
- `fn differ() -> Differ` — `Differ` nuevo con la configuración por defecto.
- `fn lifecycle(events: &[Event]) -> Vec<Event>` — Lifecycle events only: the late claims a baseline carries are checked separately (RF-25).
- `fn late_claims(events: &[Event]) -> usize` — Cuenta los `Event::Claim` marcados `late` (reclamos ya existentes al hacer la línea base).
- `const FIXTURE_CLAIMS` — The fixture carries 41 claims, so every baseline must discover exactly that many.
- `fn objective<'a>(s: &'a mut Snapshot, id: &str) -> &'a mut MatchObjective` — Acceso mutable a un objetivo del snapshot por id, para preparar casos.

**APIs externas:** `serde_json::from_str`

**Tests (19):** `rf25_late_claims_carry_the_real_claimed_at`, `rf04_first_snapshot_is_a_session_baseline`, `rf04_snapshot_inside_the_interval_triggers_nothing`, `rf03d_match_change_resets_the_baseline`, `rf26_gap_above_three_intervals_is_reported`, `rf25_a_gap_does_not_duplicate_claims_already_in_history`, `rf26_exactly_three_intervals_is_not_a_gap`, `rf26_gap_threshold_follows_the_configured_interval`, `rf04_reset_makes_the_next_snapshot_a_baseline_again`, `t06_snapshot_from_match_flattens_every_objective`, `rf10_owner_change_emits_a_flip_with_the_real_time`, `rf21_new_claim_uses_claimed_at`, `rf22_replaced_claim_emits_unclaim_then_claim`, `rf22_reclaim_by_the_same_guild_emits_unclaim_then_claim`, `rf22_flip_after_claim_emits_unclaim_for_the_stale_claim`, `rf25_expired_claim_is_not_added_to_a_baseline`, `rf22_released_claim_emits_only_unclaim`, `rf04_baseline_after_a_gap_reports_no_flips`, `t07_changes_are_ordered_by_objective_then_by_kind`


### `crates/core/src/follow.rs` (275 líneas)

Guilds the user follows, treated as favourites (RF-73).

**Contenido:**

- `pub const MAX_FOLLOWED` — How many guilds can be followed at once.
- `pub struct FollowList` — Ordered, duplicate-free list of followed guild ids.
- `impl <'de> Deserialize<'de> for FollowList`
  - `fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>` — Lee la lista de clanes seguidos de settings.json normalizando cada id (mayúsculas/guiones) y quitando duplicados.
- `pub fn normalize_guild_id(id: &str) -> Option<String>` — Converts a guild UUID to uppercase canonical `8-4-4-4-12` form.
- `fn canonical_guild_id(id: &str) -> String` — Id de clan normalizado; si no tiene forma de GUID, lo deja tal cual.
- `impl FollowList`
  - `pub fn new() -> Self` — Empty list.
  - `pub fn contains(&self, guild: &str) -> bool` — Whether the guild is followed.
  - `pub fn add(&mut self, guild: &str) -> bool` — Follows a guild. Returns `false` if it already was followed or the list is full.
  - `pub fn is_full(&self) -> bool` — Whether the list already has [`MAX_FOLLOWED`] guilds.
  - `pub fn remove(&mut self, guild: &str) -> bool` — Stops following a guild. Returns `false` if it was not followed.
  - `pub fn toggle(&mut self, guild: &str) -> bool` — Adds or removes, returning the new state.
  - `pub fn iter(&self) -> impl Iterator<Item = &str>` — Followed guilds, in the order they were added.
  - `pub fn len(&self) -> usize` — Number of followed guilds.
  - `pub fn is_empty(&self) -> bool` — Whether nothing is followed.
- `pub fn favourites_first(ranked: &[GuildSummary], follow: &FollowList) -> Vec<GuildSummary>` — Puts followed guilds first, keeping the recency order inside each group (RF-73).

**Usa (interno):** `crate::matchup`

**Tests (7):** `rf73_the_list_stops_at_ten_guilds`, `rf73_add_is_idempotent_and_toggle_flips`, `rf73_remove_reports_whether_it_did_anything`, `rf73_list_survives_serde`, `rf73_full_guild_ids_are_canonicalized_on_add_and_deserialize`, `rf73_favourites_come_first_keeping_recency_inside_each_group`, `rf73_following_does_not_resurrect_unclaimed_guilds`


### `crates/core/src/gate.rs` (224 líneas)

When the addon may draw and poll (RF-01, RF-02, RF-02a).

**Contenido:**

- `pub mod map_type` — `map_type` values that identify a WvW map (RF-01).
  - `pub const ETERNAL_BATTLEGROUNDS` — Eternal Battlegrounds.
  - `pub const BLUE_BORDERLANDS` — Blue borderlands.
  - `pub const GREEN_BORDERLANDS` — Green borderlands.
  - `pub const RED_BORDERLANDS` — Red borderlands.
  - `pub const EDGE_OF_THE_MISTS` — Edge of the Mists.
  - `pub const AUTO_REDIRECT` — Not a real map yet; the client is redirecting.
  - `pub const CHARACTER_CREATION` — Character creation.
  - `pub const WVW_REWARD` — Fortune's Vale (reward map): has no objectives to watch.
  - `pub const WVW_OBSIDIAN_SANCTUM` — Obsidian Sanctum: jump puzzle, no objectives.
  - `pub const WVW_LOUNGE` — Armistice Bastion: lounge, no objectives.
- `pub struct GameState` — What the client currently reports.
- `pub enum Hidden` — Why the overlay is hidden.
- `pub struct Gate` — Decision for the current frame.
- `impl Gate`
  - `fn hidden(reason: Hidden) -> Self` — Estado de la puerta con la UI oculta y sin sondeo, guardando el motivo (`Hidden`).
  - `pub fn settings_available(&self) -> bool` — Whether the settings window may still be opened (RF-02 keeps it available).
- `pub fn is_wvw_map_type(kind: u32) -> bool` — Whether Mumble reports one of the WvW map types, including modes without matchup objectives.
- `pub fn evaluate(state: GameState) -> Gate` — Evaluates the gate. Order matters: the loading screen wins over any map data.

**Tests (7):** `rf01_playable_wvw_maps_show_and_poll`, `rf02a_loading_screen_wins_over_map_data`, `rf02a_character_select_and_redirect_are_transitions`, `rf02a_without_any_data_nothing_is_drawn`, `rf02_outside_wvw_hides_but_allows_settings`, `rf01_the_lounge_shows_and_polls_like_other_wvw_maps`, `rf01_beta_seared_borderlands_1652_is_watched_by_map_type`


### `crates/core/src/i18n.rs` (309 líneas)

Translation table (RF-44).

**Contenido:**

- `pub const COLUMNS` — Columns per entry, in this order.
- `const ES` — Position of each language inside an entry.
- `const EN` — Índice de la columna inglesa en la tabla de traducciones.
- `const FR` — Índice de la columna francesa en la tabla de traducciones.
- `const DE` — Índice de la columna alemana en la tabla de traducciones.
- `pub enum Language` — Language of the interface.
- `impl Language`
  - `pub const ALL` — All languages, in the table order.
  - `pub fn index(self) -> usize` — Index of this language inside a table entry.
  - `pub fn code(self) -> &'static str` — Two-letter code used by the API (`?lang=`), the config and Nexus.
  - `pub fn from_code(code: &str) -> Option<Self>` — Parses a two-letter code, as stored in the configuration.
  - `pub fn from_game_code(code: u32) -> Option<Self>` — Maps Nexus RTAPI `GameData::language` (`GameLanguage` in `nexus-rs`).
  - `pub fn from_windows_language(primary: u16) -> Option<Self>` — Maps a Windows primary language id (low 10 bits of `LANGID`).
- `mod table`
- `fn entries() -> impl Iterator<Item = (&'static str, [&'static str; COLUMNS])>` — All the entries of the table, in two files so neither grows past the line limit.
- `fn lookup(…)` — Looks a key up in an arbitrary table. Kept separate so the English fallback can be tested with a table that does have an empty cell, which the real table may never have.
- `pub fn t(language: Language, key: &'static str) -> &'static str` — Translation of `key`, falling back to the key itself when it is not in the table.
- `pub fn w(language: Language, key: &'static str) -> String` — Translation with a stable imgui id appended.
- `pub fn f(language: Language, key: &'static str, args: &[&str]) -> String` — Translation with `{0}`, `{1}`… replaced. A placeholder with no argument is left as is.

**Tests (12):** `rf44_every_column_is_filled`, `rf44_the_table_stays_inside_latin_1`, `rf44_keys_are_unique`, `rf44_unknown_key_is_returned_as_is`, `rf44_both_halves_of_the_table_are_looked_up`, `rf44_empty_cell_falls_back_to_english`, `rf44_format_substitutes_every_placeholder`, `rf44_widget_id_does_not_change_with_language`, `rf44_game_language_codes_match_nexus`, `rf44_windows_language_is_only_a_fallback`, `rf44_language_codes_round_trip`, `rf44_spanish_column_is_actually_spanish`


### `crates/core/src/i18n/table.rs` (454 líneas)

The translation table itself (RF-44).

**Contenido:**

- `pub mod more`
- `pub mod picker`
- `pub mod trail`
- `pub const TEXT` — The table: `(english key, [es, en, fr, de])`.

**Usa (interno):** `super::COLUMNS`, `super::more`


### `crates/core/src/i18n/table/more.rs` (735 líneas)

The second half of the translation table (RF-44).

**Contenido:**

- `pub const TEXT` — The rest of the table: `(english key, [es, en, fr, de])`.

**Usa (interno):** `super::COLUMNS`


### `crates/core/src/i18n/table/picker.rs` (70 líneas)

Team picker window (RF-03a, RF-03j).

**Contenido:**

- `pub const TEXT` — The picker window and why it opens by itself, with their complete translations.

**Usa (interno):** `super::COLUMNS`


### `crates/core/src/i18n/table/trail.rs` (51 líneas)

B - Estela messages (RF-32, RF-33, RF-36, RF-44).

**Contenido:**

- `pub const TEXT` — Route controls and their complete translations.

**Usa (interno):** `super::COLUMNS`


### `crates/core/src/lib.rs` (40 líneas)

Pure domain logic: snapshots, diff, rules, guild following and history queries.

**Contenido:**

- `pub mod alerts`
- `pub mod detect`
- `pub mod diff`
- `pub mod follow`
- `pub mod gate`
- `pub mod i18n`
- `pub mod matchup`
- `pub mod model`
- `pub mod project`
- `pub mod prompt`
- `pub mod resolver`
- `pub mod route`
- `pub mod select`
- `pub mod teams`


### `crates/core/src/matchup.rs` (680 líneas)

Claim-centric view models: what is claimed right now across the matchup (RF-70), which guilds are active and how recently (RF-71) and how a guild's history splits between the player's map and the others (RF-72).

**Contenido:**

- `pub struct ClaimRow` — One claimed objective, with everything the list views need.
- `impl ClaimRow`
  - `pub fn is_high_value(&self) -> bool` — Whether the objective scores enough to be marked (RF-30b, threshold in [`crate::model::HIGH_VALUE_TICK`]).
- `pub fn claim_holds(objective: &crate::model::MatchObjective) -> bool` — Whether the guild that claimed the objective still holds it.
- `pub fn digest(m: &Match) -> u64` — Huella de lo que se enseña de una partida: puntos, y por objetivo el dueño, el clan y la hora del último cambio.
- `pub fn claims(m: &Match, names: &[ObjectiveInfo]) -> Vec<ClaimRow>` — Flattens every claimed objective of the match (RF-70).
- `pub struct ObjectiveState` — Estado de un objetivo en el snapshot: **dueño siempre**, claim solo si lo hay (RF-30b).
- `impl ObjectiveState`
  - `pub fn is_high_value(&self) -> bool` — Whether the objective scores enough to be marked (RF-30b).
- `pub fn states(m: &Match) -> Vec<ObjectiveState>` — Flattens **every** objective of the match with its owner, claimed or not (RF-30b).
- `pub fn by_map(rows: &[ClaimRow]) -> Vec<(MapRole, Vec<ClaimRow>)>` — Rows grouped by map role, in a stable order (RF-70).
- `pub fn with_kinds(rows: &[ClaimRow], kinds: &[ObjectiveType]) -> Vec<ClaimRow>` — Rows of the given kind only (RF-30a filter, applied to the claim list).
- `pub struct GuildSummary` — One guild with its observed claims.
- `pub fn rank_guilds(rows: &[ClaimRow]) -> Vec<GuildSummary>` — Guilds ranked by most recent claim first (RF-71).
- `pub fn last_claim_map(current: &[ClaimRow], history: &[ClaimRow], guild: &str) -> Option<MapRole>` — Destination of a guild's map button (RF-71).
- `pub fn split_by_map(rows: &[ClaimRow], current: MapRole) -> (Vec<ClaimRow>, Vec<ClaimRow>)` — Splits a guild's claims into "the map the player is on" and "the rest" (RF-72).
- `pub fn guild_timeline(rows: &[ClaimRow], guild: &str) -> Vec<ClaimRow>` — Rows of one guild, oldest first (the order the route needs).
- `pub fn guild_timeline_with_current(…)` — Include still-visible claims even if their original history row was evicted (RF-71, RF-72).
- `fn sort_timeline(out: &mut [ClaimRow])` — Ordena filas de reclamo de la más antigua a la más reciente (desempate por id de objetivo) para el historial.

**Usa (interno):** `crate::model`

**Tests (14):** `rf71_the_map_button_matches_the_latest_current_claim_in_the_list`, `rf72_current_claims_remain_visible_after_history_eviction_without_duplicates`, `rf70_a_claim_that_flipped_away_is_not_listed`, `rf70_the_fixture_claims_are_all_current`, `rf70_every_claim_of_the_match_is_listed`, `rf30b_an_objective_without_claim_keeps_its_owner`, `rf30b_every_objective_of_the_fixture_has_an_owner_state`, `rf70_rows_are_grouped_by_map_in_a_stable_order`, `rf19_a_frozen_payload_is_told_apart_from_a_quiet_matchup`, `rf30a_filter_keeps_only_the_requested_kinds`, `rf71_guilds_are_ranked_by_most_recent_claim`, `rf71_ranking_is_deterministic_for_ties`, `rf72_history_splits_current_map_from_the_rest`, `rf72_timeline_is_oldest_first`


### `crates/core/src/model.rs` (549 líneas)

Serde model of the public WvW endpoints of the GW2 API.

**Contenido:**

- `pub enum TeamColor` — Side of a matchup. Appears as `"Red"`, `"Green"`, `"Blue"` in the payload; the per-colour containers use lowercase keys instead, which is why they are not modelled with this enum's serde attributes.
- `impl TeamColor`
  - `pub const ALL` — All colours, in the order the UI shows them.
- `pub enum Region` — Region of a match, derived from the id prefix (RF-03b).
- `pub struct TeamScores` — A per-colour field of the match payload.
- `impl <T> TeamScores<T>`
  - `pub fn get(&self, color: TeamColor) -> &T` — Value of the given colour.
- `pub enum ObjectiveType` — Kind of objective, as sent in the `type` field.
- `impl ObjectiveType`
  - `pub fn is_claimable(self) -> bool` — Whether a guild can claim it (RF-27b). Only these show the "unclaimed" state.
  - `pub fn scores_points(self) -> bool` — Whether the objective scores for its owner. The rest is scenery for the map (RF-30a).
- `pub const HIGH_VALUE_TICK` — `points_tick` at or above which an objective is "high value" (RF-30b).
- `pub enum MapRole` — Role of a map inside the matchup (`type` of the entry in `maps`).
- `impl MapRole`
  - `pub fn from_map_id(map_id: u32) -> Self` — Role of a WvW map by its Mumble `map_id`, where the id is unambiguous.
  - `pub fn from_map_type(map_type: u32) -> Self` — Role of a WvW map from Mumble `map_type`.
  - `pub fn from_mumble(map_id: u32, map_type: u32) -> Self` — Mumble's role is authoritative when a map id is shared by multiple borderlands.
- `pub struct MatchObjective` — One objective as it appears inside a match snapshot.
- `fn normalize_claimed_guild_id<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>` — Deserializador de `claimed_by`: normaliza el id del clan que manda la API para que coincida con el de los seguidos.
- `impl MatchObjective`
  - `pub fn is_high_value(&self) -> bool` — High value per RF-30b: derived from scoring data, never from a hard-coded list.
  - `pub fn is_unclaimed(&self) -> bool` — Whether it is claimable and currently unclaimed (RF-27b).
- `pub struct MatchMap` — One of the maps of the matchup.
- `pub struct Match` — Match snapshot from `/v2/wvw/matches/:id`.
- `impl Match`
  - `pub fn region(&self) -> Option<Region>` — Region derived from the id prefix: `1-x` is NA, `2-x` is EU (RF-03b).
  - `pub fn color_of(&self, team_id: u32) -> Option<TeamColor>` — Colour of the given team (or legacy world) id in this match (RF-03b).
  - `pub fn team_of_shard(&self, shard: u32) -> Option<(TeamColor, u32)>` — Team the game is putting a given Mumble `shard_id` in, with the **new-style** id of that colour (`11xxx`/`12xxx`).
  - `pub fn others(&self, me: TeamColor) -> [TeamColor; 2]` — The two rival colours, in match order (RF-03g).
  - `pub fn map(&self, role: MapRole) -> Option<&MatchMap>` — Objectives of a given role, if the map is in this match.
  - `pub fn objectives_total(&self) -> usize` — Total objectives across every map of the matchup.
- `pub struct ObjectiveInfo` — Static objective metadata from `/v2/wvw/objectives`.
- `pub struct WvwMap` — A WvW map from `/v2/maps`, with the boxes the projection needs (ADR-011).
- `impl WvwMap`
  - `pub fn is_seared_borderlands(&self) -> bool` — Whether the API currently identifies this map as the beta Seared Borderlands.

**Usa (interno):** `crate::normalize_guild_id`

**Tests (13):** `rf01_map_roles_come_from_the_verified_map_ids`, `rf03b_resolves_color_from_all_worlds_not_worlds`, `rf73_claim_ids_are_canonical_when_read_from_match`, `rf03b_region_comes_from_match_id_prefix`, `rf03g_returns_the_two_rivals`, `rf27b_claimed_and_unclaimed_are_distinguished`, `rf27b_non_claimable_types_are_never_unclaimed`, `rf30b_high_value_is_derived_from_points_tick`, `rf30a_only_capture_points_score`, `rf30a_unknown_objective_type_does_not_break_parsing`, `t04_objective_info_tolerates_null_coord`, `rf30_map_bounds_come_from_continent_rect`, `rf30_objective_coords_fall_inside_continent_rect`


### `crates/core/src/project.rs` (290 líneas)

Projection of WvW coordinates onto the rectangle of a window (T-14, RF-30).

**Contenido:**

- `pub struct Frame` — The box of one map in continent coordinates.
- `impl Frame`
  - `pub fn of(map: &WvwMap) -> Self` — Box of a map from `/v2/maps`. The corners may come in either order.
  - `pub fn is_usable(&self) -> bool` — Whether the box has a width and a height to divide by.
  - `pub fn normalise(&self, coord: [f64; 2]) -> Option<[f32; 2]>` — Continent coordinates → `0..1` in both axes, with **Y down**: the minimum is the north edge.
- `pub fn to_screen(normalised: [f32; 2], min: [f32; 2], size: [f32; 2]) -> [f32; 2]` — `0..1` (Y down, north first) → pixels inside a rectangle (Y down), which is how imgui thinks.
- `pub fn project(frame: &Frame, coord: [f64; 2], min: [f32; 2], size: [f32; 2]) -> Option<[f32; 2]>` — Continent coordinates → pixels, or `None` when there is nothing sensible to draw.
- `pub const TILE_ZOOM` — Zoom level used for the real map: the highest the tile server answers (verified 2026-10-31).
- `pub fn units_per_tile(zoom: u32) -> f64` — Continent units covered by one tile at `zoom`.
- `pub fn tile_box(x: u32, y: u32, zoom: u32) -> [f64; 4]` — Continent box of one tile: `[min_x, min_y, max_x, max_y]`.
- `pub fn tiles_of(frame: &Frame, zoom: u32) -> Vec<(u32, u32)>` — Tiles that cover a map.

**Usa (interno):** `crate::model`

**Tests (7):** `rf30_the_corners_and_the_centre_land_where_they_should`, `rf30_north_is_up_on_screen`, `rf30_the_real_orientation_matches_the_continent_data`, `rf30_a_degenerate_box_has_no_projection`, `rf30_the_tile_grid_covers_the_whole_map`, `rf30_a_degenerate_box_asks_for_no_tiles`, `rf30_every_real_objective_of_eternals_falls_inside_the_box`


### `crates/core/src/prompt.rs` (176 líneas)

When the team picker has to open by itself (RF-03j).

**Contenido:**

- `pub enum Prompt` — Why the picker opened by itself.
- `pub fn matchup_key(all: &[Candidate], team_id: u32) -> Option<String>` — Identity of the matchup a team plays: the match id and the three teams in it.
- `pub fn prompt(…)` — Whether the picker has to open, and why.

**Usa (interno):** `crate::select`

**Tests (4):** `rf03j_the_key_names_the_match_and_its_three_teams`, `rf03j_new_rivals_are_a_new_matchup_with_the_same_match_id`, `rf03j_nothing_confirmed_or_no_data_is_not_a_change`, `rf03j_a_new_player_and_a_relinked_team_are_asked`


### `crates/core/src/resolver.rs` (76 líneas)

Resolves the player's team inside a match snapshot (RF-03b, RF-03g).

**Contenido:**

- `pub struct Resolution` — Outcome of resolving one team id against a match.
- `impl Resolution`
  - `pub fn is_own(&self, color: TeamColor) -> bool` — Whether the given colour is the player's own team.
- `pub fn resolve(m: &Match, team_id: u32) -> Option<Resolution>` — Finds the colour of `team_id` inside the match.

**Usa (interno):** `crate::model`

**Tests (3):** `rf03b_resolves_team_color_and_rivals`, `rf03b_linked_legacy_world_resolves_to_the_same_side`, `rf03b_unknown_team_is_not_resolved`


### `crates/core/src/route.rs` (630 líneas)

Virtual route of a guild (T-40, RF-32a).

**Contenido:**

- `pub mod playback`
- `pub mod trail`
- `pub enum Origin` — Where the route starts, and why.
- `impl Origin`
  - `pub fn reason(self) -> &'static str` — Short reason, for the caption under the route.
- `pub struct Step` — One node of the drawn route.
- `pub struct Route` — Route of one guild on one map: the origin plus the claims in order.
- `pub struct OriginPoint` — Located origin, distinct from an observed claim and without a made-up timestamp.
- `pub fn with_origin<'a>(…)` — Builds the observed route on one map and locates its inferred departure (RF-32a).
- `pub fn visible_steps(steps: usize, progress: f32) -> usize` — Cuántos pasos de la ruta se han «reproducido» con una fracción `progress` de 0 a 1 (RF-36).
- `pub fn progress(elapsed: f32, seconds: f32) -> f32` — Fracción de la ruta reproducida tras `elapsed` segundos de una reproducción de `seconds`.
- `pub fn build<'a>(rows: &'a [ClaimRow], guild: &str) -> Option<Route<'a>>` — Builds chronological observations; locating an origin additionally requires [`with_origin`].
- `fn ordered<'a>(…)` — Reclamos de un clan, filtrados opcionalmente por mapa y por fecha mínima, ordenados por `claimed_at` (los pasos de la ruta).
- `fn build_claims<'a>(mine: &[&'a ClaimRow]) -> Option<Route<'a>>` — Convierte los reclamos ordenados en una `Route` sin origen (un paso por reclamo); `None` si no hay ninguno.
- `pub fn locate_origin(…)` — Infers an origin from chronological claims already restricted to one guild and map.
- `fn origin_point(…)` — Posición del paso 0 a partir del objetivo: `coord` exacta o, si se permite, `label_coord` marcada como aproximada (los spawns no tienen `coord`).

**Usa (interno):** `crate::matchup`, `crate::model`

**Tests (16):** `rf32a_step_zero_locates_a_previously_claimed_home_garrison`, `rf32a_an_alpine_side_keep_requires_the_whole_segment_to_start_north`, `rf32a_a_future_garrison_cannot_be_the_departure_for_earlier_claims`, `rf32a_respawn_uses_the_guild_colour_and_the_official_label_position`, `rf32a_time_ranges_and_map_segments_do_not_change_earlier_departures`, `rf32a_enemy_garrisons_and_eternal_keeps_are_not_home_departures`, `rf32a_desert_uses_its_home_garrison_but_not_the_alpine_side_keep_rule`, `rf32a_missing_colour_or_coordinates_never_fabricates_step_zero`, `rf32a_route_follows_the_claim_order`, `rf32a_a_guild_without_claims_has_no_route`, `rf32a_claim_types_without_context_cannot_identify_a_garrison`, `rf32a_towers_do_not_identify_a_side_keep`, `rf32a_spawn_without_coordinates_remains_unknown`, `rf32a_the_origin_is_always_labelled_as_inferred`, `rf36_playback_shows_the_steps_it_has_reached`, `rf36_progress_goes_from_zero_to_one`


### `crates/core/src/route/playback.rs` (117 líneas)

Clock-injected step playback for B - Estela (RF-36).

**Contenido:**

- `const STEP` — One observation every 700 ms, not a simulation of movement speed.
- `pub struct Playback` — Session-only player. The caller pauses it when the map is hidden or the guild changes.
- `impl Playback`
  - `pub fn reset(&mut self, count: usize)` — Selects the latest observation of a new guild and stops playback.
  - `pub fn pause(&mut self)` — Pauses without losing the selected claim or counting elapsed time while hidden.
  - `pub fn choose(&mut self, index: usize, count: usize)` — Selects an existing step, clamping after history eviction, and pauses.
  - `pub fn toggle(&mut self, now: Duration, count: usize)` — Starts/resumes, or pauses. Play at the end restarts at the first observation.
  - `pub fn tick(&mut self, now: Duration, count: usize)` — Advances with an injected monotonic clock, preserving sub-step remainder.

**Tests (2):** `rf36_estela_playback_advances_at_700ms_and_stops_at_the_end`, `rf36_estela_pause_resume_and_scrubbing_do_not_count_hidden_time`


### `crates/core/src/route/trail_tests.rs` (258 líneas)

**Contenido:**

- `fn at(seconds: i64) -> DateTime<Utc>` — Synthetic clock with explicit offsets for temporal boundary tests.
- `fn row(id: &str, map_id: u32, seconds: Option<i64>) -> ClaimRow` — Synthetic observation; tests needing official coordinates load the objective fixture.
- `fn names() -> Vec<ObjectiveInfo>` — Real objective catalogue shared with the existing origin tests.

**Usa (interno):** `crate::model`

**APIs externas:** `serde_json::from_str`

**Tests (11):** `rf32_trail_preserves_revisits_and_orders_time_ties_by_objective`, `rf36_trail_windows_include_the_exact_limit_without_renumbering`, `rf33_trail_claim_silence_is_not_a_polling_gap`, `rf33_trail_preserves_late_events_and_overlapping_poll_gaps`, `rf32_trail_missing_coordinates_break_both_links_without_losing_the_step`, `rf36_trail_empty_single_and_equal_times_have_finite_timelines`, `rf36_trail_does_not_invent_times_or_intervals`, `rf32_trail_filters_the_guild_and_keeps_fast_claims_as_one_sequence`, `rf36_trail_unknown_times_stay_explicit_and_do_not_create_silence`, `rf36_trail_window_uses_exact_durations_not_rounded_seconds`, `rf32_trail_cannot_use_another_maps_or_nonfinite_coordinates`


### `crates/core/src/route/trail.rs` (291 líneas)

Pure temporal model for B - Estela (T-51, RF-32, RF-33, RF-36).

**Contenido:**

- `pub enum Window` — Time span behind the selected claim; observations outside it are dimmed, not discarded.
- `impl Window`
  - `fn duration(self) -> Option<TimeDelta>` — Window length, or no limit for the available history.
- `pub enum Visibility` — How a claim is presented relative to the selected step.
- `pub enum Connection` — Whether two consecutive observations can have a curved connector.
- `pub struct Observation` — Claim with its official position and temporal information, without invented data.
- `pub struct Leg` — One adjacent pair of observations, including pairs that must not be joined on the map.
- `pub struct Group` — All visits to an objective on one map, in stable chronological order.
- `pub struct Trail` — All observations of one guild; borrowed history and copied metadata only.
- `pub struct View` — Selection and per-step visibility, with exactly one state per original observation.
- `impl <'a> Trail<'a>`
  - `pub fn new(…)` — Builds the guild timeline and preserves late/gap annotations from retained events.
  - `pub fn view(&self, selected: usize, window: Window) -> View` — Classifies every step without filtering the source history.
  - `pub fn into_owned(self) -> Trail<'static>` — Owns the observations for a UI cache without borrowing the live dashboard.
- `fn group_visits(steps: &[Observation<'_>]) -> Vec<Group>` — Groups by map and objective while preserving each visit's index.
- `fn merged_gaps(events: &[Event]) -> Vec<(DateTime<Utc>, DateTime<Utc>)>` — Merges polling gaps so interval lookups are logarithmic, even for overlapping input.
- `mod tests`

**Usa (interno):** `super::ordered`, `super::with_origin`


### `crates/core/src/select.rs` (383 líneas)

Team selection: when the addon has to ask the player which team is theirs (RF-03a).

**Contenido:**

- `pub enum Selection` — What the picker must do right now.
- `pub struct Candidate` — One team the player may pick, with the matchup it is playing.
- `impl Candidate`
  - `pub fn region(&self) -> Option<crate::model::Region>` — Region of the matchup, from its id prefix (`1-x` is NA, `2-x` is EU).
- `pub fn of_region_and_color(…)` — Teams of one region and one colour: **one per matchup**, which is what the third step of the picker offers (RF-03a).
- `pub struct Roster` — The team chosen for each **account**.
- `impl Roster`
  - `pub fn get(&self, key: &str) -> Option<u32>` — Team remembered under a key: the account name, or the character name without arcdps.
  - `pub fn set(&mut self, key: &str, team_id: u32)` — Remembers the team under a key. An empty key is ignored: without a name the selection could not be told apart from any other.
  - `pub fn forget(&mut self, key: &str)` — Forgets a key.
  - `pub fn len(&self) -> usize` — Keys remembered.
  - `pub fn is_empty(&self) -> bool` — Whether nobody has chosen a team yet.
- `pub fn evaluate(saved: Option<u32>, live: &[Match]) -> Selection` — Decides whether the saved team still makes sense.
- `pub fn evaluate_for(key: &str, roster: &Roster, live: &[Match]) -> Selection` — Same decision, but looking the key (account, or character as fallback) up in the roster first.
- `pub fn candidates(live: &[Match]) -> Vec<Candidate>` — Every team playing a live match, ordered by match and then by colour.
- `pub fn is_team(id: u32) -> bool` — Whether an id is a WvW team id rather than a legacy world id.
- `pub fn matchups_of(live: &[Match], region: crate::model::Region) -> Vec<&Match>` — Matchups of a region, `EU` for ids starting with `2-` and `NA` for `1-` (RF-03b).
- `pub fn teams_of(live: &[Match], match_id: &str) -> Vec<Candidate>` — The three teams of a matchup, in colour order.
- `pub fn rivals(live: &[Match], match_id: &str, me: u32) -> Vec<Candidate>` — Teams of the match the player is in, for the "pick again" case: the three rivals are far easier to recognise than a list of thirty.

**Usa (interno):** `crate::model`

**Tests (10):** `rf03a_without_saved_team_the_picker_opens`, `rf03a_saved_team_still_playing_is_confirmed`, `rf03a_relinked_team_opens_the_picker_again`, `rf03a_candidates_are_the_three_teams_of_each_match`, `rf03a_the_two_teams_of_a_colour_are_not_confused`, `rf03a_roster_keeps_one_team_per_key`, `rf03a_roster_is_saved_by_account_and_the_old_map_is_not_read`, `rf03b_two_step_picker_groups_by_region_and_matchup`, `rf03a_region_and_colour_narrow_the_picker_to_one_team_per_matchup`, `rf03g_rivals_are_the_other_two_of_my_match`


### `crates/core/src/teams.rs` (358 líneas)

Team identity and the bundled `data/wvw-teams.json` table (ADR-010).

**Contenido:**

- `pub const BUNDLED_JSON` — Table shipped with the addon, regenerated by `scripts/update_wvw_teams.ps1`.
- `pub const NAMES_JSON` — Curated translations, keyed by the **English** name.
- `pub struct TeamName` — One translation entry.
- `impl TeamName`
  - `pub fn get(&self, language: &str) -> Option<&str>` — Translation for a two-letter language code, if there is one.
- `struct NamesFile` — Forma del JSON `team-names.json` incrustado: nombres de equipo por id.
- `pub fn translations() -> &'static BTreeMap<String, TeamName>` — Translations by English name, parsed once.
- `pub struct TeamEntry` — One entry of the table.
- `struct File` — Forma del JSON `wvw-teams.json` incrustado: lista de equipos.
- `pub struct TeamTable` — Id -> name table for WvW teams.
- `impl TeamTable`
  - `pub fn parse(json: &str) -> Result<Self, serde_json::Error>` — Parses the bundled table format. Unknown fields are ignored on purpose so the script can add metadata without breaking older builds.
  - `pub fn name(&self, id: u32) -> Option<&str>` — Name of the team, if known.
  - `pub fn nation(&self, id: u32) -> Option<&str>` — Region of the team as curated in the table (`EU`/`NA`), if known.
  - `pub fn label(&self, id: u32) -> String` — How the UI shows a team: `Name (id)`, or `(id)` when the name is unknown (RF-03a).
  - `pub fn name_in(&self, language: &str, id: u32) -> Option<&str>` — Name of the team in a language, falling back to the curated English name.
  - `pub fn label_in(&self, language: &str, id: u32) -> String` — `nombre (id)` in the requested language (RF-44), same fallback as [`Self::name_in`].
  - `pub fn len(&self) -> usize` — Number of known teams.
  - `pub fn sorted(&self) -> Vec<&TeamEntry>` — Every team, ordered by region and then by name, for the picker (RF-03a).
  - `pub fn is_empty(&self) -> bool` — Whether the table is empty.
- `pub fn bundled() -> &'static TeamTable` — The bundled table, parsed once.

**APIs externas:** `serde_json::from_str`, `serde_json::Error`

**Tests (9):** `rf03i_bundled_table_holds_every_live_team`, `rf03a_label_falls_back_to_the_id_alone`, `rf44_translated_team_names_fall_back_to_english`, `rf44_spreadsheet_names_are_used_by_english_name`, `rf44_spanish_and_french_come_from_the_wiki_list`, `rf44_german_comes_from_gw2mists_and_not_from_the_spreadsheet`, `rf44_the_pool_covers_every_team_in_use`, `rf03a_picker_order_is_stable_and_regional`, `rf03i_broken_table_degrades_to_ids`


## Crate `api`

- `crates/api/Cargo.toml`
- Fixtures de tests (9): `guild.json`, `maps_wvw.json`, `match_2-2.json`, `matches_all.json`, `matches_all_20261102.json`, `matches_overview.json`, `objectives_all.json`, `objectives_seared_borderlands.json`, `timers_team_assignment.json`

### `crates/api/src/lib.rs` (653 líneas)

HTTP client for the public GW2 API and its response parsing.

**Contenido:**

- `pub const BASE` — Base URL of the public API.
- `pub struct GuildInfo` — Public guild data.
- `pub struct TeamAssignmentTimers` — Regional team-assignment deadlines returned by `/v2/wvw/timers/teamAssignment`.
- `pub enum ApiError` — What can go wrong while talking to the API.
- `impl std::fmt::Display for ApiError`
- `impl std::error::Error for ApiError`
- `pub trait Transport` — Minimal HTTP surface. Implementations only fetch bytes; parsing lives above.
  - `fn get(&self, url: &str) -> Result<Vec<u8>, ApiError>;` — Performs a GET and returns the raw body.
- `pub struct Api` — API client over any [`Transport`].
- `impl <T: Transport> Api<T>`
  - `pub fn new(transport: T) -> Self` — Wraps a transport.
  - `pub fn transport(&self) -> &T` — Access to the underlying transport (tests, diagnostics).
  - `pub fn matches_all(&self) -> Result<Vec<Match>, ApiError>` — Every live match of **both** regions (NA `1-x` and EU `2-x`), from `/v2/wvw/matches?ids=all` (~400 KB).
  - `pub fn matches_overview(&self) -> Result<Vec<String>, ApiError>` — Ids of the live matches, from the small `/v2/wvw/matches/overview` response.
  - `pub fn team_assignment_timers(&self) -> Result<TeamAssignmentTimers, ApiError>` — Team-assignment deadlines for NA and EU.
  - `pub fn match_for_team(&self, team_id: u32) -> Result<Match, ApiError>` — Match that contains `team_id`.
  - `pub fn match_by_id(&self, id: &str) -> Result<Match, ApiError>` — A single match by id, from `/v2/wvw/matches/{id}`.
  - `pub fn objectives(&self, lang: &str) -> Result<Vec<ObjectiveInfo>, ApiError>` — Static objective metadata in the given language (RF-44).
  - `pub fn guild(&self, id: &str) -> Result<GuildInfo, ApiError>` — Public guild data.
  - `pub fn guild_search(&self, text: &str) -> Result<Vec<String>, ApiError>` — Ids of the guilds whose name **starts with** `text` (T-44).
  - `pub fn wvw_guilds(…)` — Guild id → team id for a region, the official source of "which team is this guild on".
  - `pub fn maps(&self) -> Result<Vec<gw2lurker_core::WvwMap>, ApiError>` — WvW map geometries used by the matchup and beta borderlands (RF-30, ADR-011).
  - `pub fn worlds(&self) -> Result<std::collections::BTreeMap<u32, String>, ApiError>` — Legacy world id → name, to say which old worlds fight for each team (RF-03a).
- `struct World` — One entry of `/v2/worlds`: the old servers, still in `all_worlds` for compatibility.
- `enum TeamRef` — The team of a guild in `/v2/wvw/guilds/:region`, which arrives as text.
- `impl TeamRef`
  - `fn into_id(self) -> Option<u32>` — Id de equipo venga como número o como texto en el JSON de la API; `None` si el texto no es un número.
- `pub fn looks_like_guild_id(id: &str) -> bool` — Whether a string has the shape of a guild id: 32 hexadecimal digits, with or without dashes.
- `fn looks_like_match_id(id: &str) -> bool` — `<region>-<tier>`, like `2-2`.
- `fn looks_like_language(lang: &str) -> bool` — Language supported by the addon: `es`, `en`, `fr` or `de`.
- `fn url_encode(text: &str) -> String` — Percent-encodes a query value: guild names have spaces, accents and apostrophes.
- `fn parse<T: for<'de> Deserialize<'de>>(body: &[u8]) -> Result<T, ApiError>` — Deserializa el cuerpo de una respuesta; un JSON inválido se convierte en `ApiError::Parse`.
- `pub struct Backoff` — Exponential retry with a cap, shared by the polling worker (RNF-04).
- `impl Default for Backoff`
  - `fn default() -> Self` — Backoff exponencial de 2 s a 5 min.
- `impl Backoff`
  - `pub fn new(base_ms: u64, cap_ms: u64) -> Self` — Creates a backoff with the given base and cap, both in milliseconds.
  - `pub fn reset(&mut self)` — Resets after a successful request.
  - `pub fn attempts(&self) -> u32` — Current attempt number (0 after a reset).
  - `pub fn next_delay_ms(&mut self) -> u64` — Delay for the next retry, recording the failure.
  - `pub fn should_retry(error: &ApiError) -> bool` — Whether the error deserves a retry: network failures and 5xx do, 4xx do not.

**Usa (interno):** `gw2lurker_core::model`, `gw2lurker_core::normalize_guild_id`, `gw2lurker_core::Region`, `gw2lurker_core::WvwMap`

**APIs externas:** `https://api.guildwars2.com/v2`, `serde_json::from_slice`

**Tests (21):** `t11b_matches_are_parsed_from_the_real_payload`, `rf03a_live_ids_all_fixture_has_nine_unique_matches`, `adr010_team_is_found_inside_all_worlds`, `adr010_unknown_team_is_not_found`, `t11b_single_match_fixture_parses`, `t11b_single_match_is_fetched_by_id`, `rf44_objectives_are_requested_in_the_asked_language`, `t44_guild_search_encodes_spaces_ampersands_and_utf8`, `rf20_guild_search_returns_canonical_uuid`, `rf20_wvw_guild_team_keys_are_canonical`, `rf03a_the_team_of_a_guild_is_read_whatever_the_api_uses`, `rf20_guild_returns_name_and_tag`, `rf20_a_malformed_guild_id_never_reaches_the_network`, `rf03d_team_assignment_timer_parses_both_regions`, `rf03d_matches_overview_returns_the_live_match_ids`, `rf20_a_malformed_match_id_or_language_never_reaches_the_network`, `rf20_unknown_routes_answer_like_the_real_transport`, `rf30_maps_come_with_the_boxes_of_the_projection`, `rnf04_http_503_is_reported_as_status`, `rnf04_truncated_json_is_a_parse_error`, `rnf04_backoff_grows_and_caps`


## Crate `addon`

- `crates/addon/Cargo.toml` — `max_level_info` drops debug!/trace! at compile time. It replaces nexus' `log_filter: "info"`, whose `env_filter` pulled in `regex` (~390 KiB of the DLL) just to parse one level. Descomprime el ZIP del índice de nombres…

### `crates/addon/src/alerts.rs` (214 líneas)

Alert lines: the events of the session turned into something the widget can draw (T-13).

**Contenido:**

- `pub const MAX_ALERTS` — How many events the dashboard keeps. The widget only shows the last few minutes; the rest is so the log window can show the session.
- `const LATE_GRACE` — How recent a claim found at a baseline has to be to still deserve an alert.
- `impl Dashboard`
  - `pub fn push_alert(&mut self, event: Event)` — Añade un evento a la lista de avisos (salvo línea base y huecos) con la hora de recepción, recortando a `MAX_ALERTS`.
  - `pub fn recent_alerts(…)` — Recent events as ready-to-draw lines, **newest first** (RF-19), already filtered.
  - `fn alert_line(&self, event: &Event, received: DateTime<Utc>) -> Option<AlertLine>` — Traduce un `Event` del diff a la línea de aviso de la UI: nombre del objetivo, mapa (rol por el prefijo del id), tipo, clan y colores.
- `pub fn retain_route_event(events: &mut std::sync::Arc<Vec<Event>>, event: &Event)` — Shares the annotation retention rule with the worker's prepared route snapshot.
- `pub enum AlertKind` — What happened, for the alert widget.
- `pub struct AlertLine` — One line of the alert widget.
- `pub struct Received` — An event and the instant this addon learnt about it.

**Usa (interno):** `crate::worker`, `gw2lurker_core::AlertFilter`, `gw2lurker_core::diff`, `gw2lurker_core::FollowList`, `gw2lurker_core::model`, `gw2lurker_core::ObjectiveType`


### `crates/addon/src/arc_team.rs` (77 líneas)

Which team the player is on, as arcdps sees it (RF-03a).

**Contenido:**

- `static SELF_TEAM` — arcdps team of the local player; `-1` until arcdps reports it.
- `static ACCOUNT` — Account name arcdps reports for the local player, `None` until the first self-join.
- `pub fn subscribe()` — Subscribes to arcdps' self-join event. Must be called from `load`.
- `fn on_self_join(payload: Option<&nexus::event::arc::AgentUpdate>)` — Callback del evento de ArcDPS «self join»: guarda el equipo del jugador en `SELF_TEAM`.
- `pub fn self_team_tag() -> Option<u16>` — Opaque arcdps team tag of the local player, if arcdps has reported one.
- `pub fn self_account() -> Option<String>` — Account name arcdps reported for the local player (`:Jhoin.1234`, with the colon arcdps puts in front; verified in the game on 2026-11-01), which is the key the team is saved under (RF-03a).
- `pub fn describe() -> String` — Tag as text for the log, so a bug report can say which side arcdps thought we were on.

**APIs externas:** `nexus::event`, `nexus::event::arc::AgentUpdate`


### `crates/addon/src/assignment.rs` (195 líneas)

Detects scheduled WvW team reassignment without polling global match data every frame.

**Contenido:**

- `const TIMER_TTL` — Cada cuánto se vuelven a pedir los temporizadores de reasignación (30 min).
- `const RETRY_AFTER_ERROR` — Espera tras un fallo al pedir los temporizadores (60 s).
- `const OVERVIEW_TTL` — Mínimo entre dos consultas del listado de partidas activas tras pasar la hora de reasignación (60 s).
- `pub struct AssignmentMonitor` — Estado del vigilante de reasignación de equipos: horas límite NA/EU y cuándo puede volver a preguntar.
- `impl AssignmentMonitor`
  - `pub fn rotation_confirmed<T: Transport>(…)` — `true` si la reasignación de la región del match ya pasó y el match vigilado ya no está entre los activos (`/v2/wvw/matches` sin ids).
  - `pub fn reset_after_rotation(&mut self)` — Olvida horas límite y esperas para empezar de cero tras una reasignación.
  - `fn refresh_timers<T: Transport>(…)` — Pide los temporizadores de reasignación a la API (como mucho cada 30 min; 60 s tras un error) y guarda las horas NA/EU.
- `pub fn reset_tracking_for_assignment(…)` — Si cambió el match tras la reasignación, reinicia diff, vigilante y ciclo de seguimiento del Dashboard (historial y avisos). Devuelve si reinició.
- `fn parse_deadline(value: &str) -> Result<DateTime<Utc>, ApiError>` — Fecha RFC 3339 de un temporizador a UTC; error de parseo si no es válida.

**Usa (interno):** `crate::worker`, `gw2lurker_core::diff`

**Tests (3):** `rf03d_expired_assignment_confirms_rotation_when_match_leaves_overview`, `rf03d_expired_assignment_keeps_tracking_if_match_is_still_live`, `rf03d_future_assignment_does_not_poll_matches_overview`


### `crates/addon/src/background.rs` (50 líneas)

**Contenido:**

- `static STOPPING` — Bandera de apagado: una vez puesta no se lanzan más tareas.
- `static HANDLES` — Acceso perezoso a `HANDLES`.
- `fn handles() -> &'static Mutex<Vec<JoinHandle<()>>>` — Acceso perezoso a `HANDLES`.
- `pub fn spawn(name: &str, task: impl FnOnce() + Send + 'static) -> io::Result<()>` — Lanza una tarea con nombre en su propio hilo, la registra y limpia las terminadas; falla si el addon se está apagando.
- `pub fn is_stopping() -> bool` — Si el addon se está descargando; las tareas largas lo consultan para salir antes.
- `pub fn stop_all()` — Marca el apagado y espera (join) a todas las tareas registradas.
- `fn log_join_failure(handle: JoinHandle<()>)` — Hace join de un hilo y registra en el log si terminó con pánico.


### `crates/addon/src/catalog.rs` (521 líneas)

The static catalogues of the API: objectives, maps and guild names.

**Contenido:**

- `const GUILD_LOOKUPS_PER_POLL` — How many guild names are resolved per poll.
- `const STATIC_DATA_RETRY` — Espera mínima antes de reintentar mapas o mundos que fallaron (60 s).
- `struct GuildLookupFailures` — Registro de ids de clan cuya consulta falló: permanentes (404) y reintentables con su backoff.
- `impl GuildLookupFailures`
  - `fn should_skip(&self, id: &str, now: Instant) -> bool` — Si no hay que pedir ese clan ahora (fallo permanente o aún en espera).
  - `fn record(&mut self, id: &str, error: &ApiError, now: Instant)` — Apunta un fallo: los reintentables programan el siguiente intento con backoff; el resto queda permanente.
  - `fn resolved(&mut self, id: &str)` — Borra cualquier fallo de un clan que ya se resolvió.
- `fn guild_lookup_failures() -> &'static Mutex<GuildLookupFailures>` — Acceso perezoso al registro global de fallos de clanes.
- `fn static_data_retry_due(last_attempt: Option<Instant>, now: Instant) -> bool` — Si ya toca reintentar un catálogo estático (nunca intentado o hace ≥ 60 s).
- `pub fn guild_label(info: &gw2lurker_api::GuildInfo) -> String` — `[TAG] Nombre`, the way the game shows a guild.
- `pub fn resolve_guild_names(…)` — Names of the guilds in the matchup: from **this session's file** (`temp.json`), from the settings, then from the API.
- `pub fn world_names(api: &Api<WinHttpTransport>) -> BTreeMap<u32, String>` — Names of the old servers, fetched once per session.
  - `struct WorldCache` — Caché de nombres de mundo (`/v2/worlds`) y hora del último intento.
- `pub fn maps_for(api: &Api<WinHttpTransport>) -> Vec<gw2lurker_core::WvwMap>` — WvW map geometries, fetched once per session (they never change). `None` means the API has not answered yet: the map tab says so instead of drawing an empty box, and the next attempt follows a short r…
  - `struct MapCache` — Caché de las geometrías WvW clásicas y beta (`/v2/maps`) y hora del último intento.
- `pub fn objectives_for(api: &Api<WinHttpTransport>) -> Result<Vec<ObjectiveInfo>, String>` — Objetivos WvW (`/v2/wvw/objectives`) en el idioma activo; se piden una vez por sesión y al cambiar de idioma.
- `pub fn tiles_dir() -> Option<std::path::PathBuf>` — Directory of the map tiles inside the addon's folder.
- `const TILE_CONTINENT` — Continent and floor of the WvW maps on the tile server.
- `const TILE_FLOOR` — Planta del servidor de teselas usada para los mapas WvW («1»).
- `const TILE_CACHE_TTL` — Edad máxima de caché de una tesela normal antes de pedirla de nuevo (24 h).
- `pub const MIN_TILE_BYTES` — Tamaño mínimo de un fichero para tratarlo como tesela completa (1 KB).
- `static TILE_REVISION` — Generación actual de la caché de teselas para invalidar texturas sin E/S en cada fotograma.
- `pub fn tile_revision() -> u64` — Generación actual de la caché de teselas para invalidar texturas sin E/S en cada fotograma.
- `pub fn tile_path(x: u32, y: u32) -> Option<std::path::PathBuf>` — File of one tile. The name has every coordinate so a cache can be inspected by hand, and the continent is in it too: a tile of another continent is not the same tile.
- `pub fn tile_url(x: u32, y: u32) -> String` — URL of one tile on the official tile server.
- `fn forget_stale_tiles(keep_prefix: &str)` — Deletes tiles saved by older versions, whose names did not carry the continent.
- `pub fn ensure_tiles(maps: &[gw2lurker_core::WvwMap])` — Downloads the tiles of the maps that are not in the cache yet, on its own thread.
- `fn tile_needs_refresh(…)` — Si una tesela beta debe renovarse siempre o una normal alcanzó su TTL.
  - `mod ified`

**Usa (interno):** `crate::background`, `crate::config`, `crate::i18n`, `crate::net`, `crate::request_save`, `crate::runtime`, `crate::session_names`, `crate::worker`, `gw2lurker_api::GuildInfo`, `gw2lurker_api::Transport`, `gw2lurker_core::FollowList`, `gw2lurker_core::Frame`, `gw2lurker_core::matchup`, `gw2lurker_core::model`, `gw2lurker_core::project`, `gw2lurker_core::WvwMap`

**APIs externas:** `https://tiles.guildwars2.com/{TILE_CONTINENT}/{TILE_FLOOR}/{}/{x}/{y}.jpg`, `disco: fs::read_dir`, `disco: fs::remove_file`, `disco: fs::metadata`, `disco: fs::create_dir_all`, `disco: fs::write`, `disco: fs::rename`

**Tests (6):** `rf20_a_guild_is_named_with_its_tag`, `rf20_transient_guild_lookup_errors_are_retried_after_a_cooldown`, `rf20_non_retryable_guild_errors_are_not_requested_again`, `rf20_static_catalogues_retry_after_a_cooldown_not_every_poll`, `p05_seared_map_tiles_are_always_refreshed`, `p06_cached_tiles_expire_daily_but_recent_tiles_are_reused`


### `crates/addon/src/clock.rs` (62 líneas)

Clock times as the player sees them.

**Contenido:**

- `pub fn clock(at: DateTime<Utc>) -> String` — `HH:MM` in the player's local time.
- `pub fn clock_seconds(at: DateTime<Utc>) -> String` — `HH:MM:SS` in the player's local time, for the alert lines where seconds matter.
- `pub fn local(at: DateTime<Utc>) -> DateTime<Local>` — An instant from the API, moved to the machine's timezone.
- `fn from_utc(seconds: i64, nanos: u32) -> Option<DateTime<Utc>>` — The same instant built from a UTC timestamp, for the tests.

**Tests (2):** `rf19_the_clock_uses_the_players_timezone_and_not_utc`, `rf19_the_same_instant_is_formatted_the_same_way_twice`


### `crates/addon/src/config/mod.rs` (556 líneas)

Addon configuration, stored as JSON in the addon directory (RF-41).

**Contenido:**

- `pub enum AlertScope` — Where the alerts apply (RF-12).
- `pub enum LanguagePreference` — Language preference (RF-44). `Auto` follows the game language.
- `pub struct WindowState` — Per-window UI state (RF-46).
- `impl Default for WindowState`
  - `fn default() -> Self` — Ventana visible, sin plegar, sin bloquear, sin click-through, opacidad 0,9 y sin posición/tamaño guardados.
- `impl WindowState`
  - `fn sanitize(&mut self)` — Corrige valores imposibles de settings.json: opacidad a 0–1, posición o tamaño no finitos fuera y tamaño mínimo 120×60.
- `pub const POLL_SECS_MIN` — Fastest poll the addon allows, in seconds.
- `pub const POLL_SECS_DEFAULT` — Interval used when the player has not chosen one.
- `pub const POPUP_SECS_MAX` — Longest a pop-up may stay on screen, in seconds (RF-19a).
- `pub const POPUP_SECS_DEFAULT` — Cuánto dura el aviso emergente cuando el jugador no ha tocado nada.
- `pub struct Config` — Todo lo que se guarda en settings.json: equipo y traducciones, sondeo, avisos, popup, mapa, clanes seguidos y nombres, ventanas, idioma, tema.
- `pub enum Theme` — Estilo de las ventanas del addon (RF-60, RF-61).
- `pub enum WidgetBackground` — Fondo del widget (RF-63).
- `pub struct AlertKinds` — Qué se enseña en el widget y en el pop-up (RF-19c).
- `impl Default for AlertKinds`
  - `fn default() -> Self` — Por defecto avisa de cambios de color, reclamos y pérdidas de reclamo.
- `impl AlertKinds`
  - `pub fn shows(&self, kind: crate::worker::AlertKind) -> bool` — Whether a line of this kind is shown (RF-19c).
- `impl Default for Config`
  - `fn default() -> Self` — Configuración inicial: sin equipo, sondeo y popup por defecto, mapa real activado, índice de clanes por defecto y ventanas por defecto.
- `impl Config`
  - `pub fn window(&mut self, key: &str) -> &mut WindowState` — State of one window, created with defaults the first time it is asked for.
  - `pub fn custom_name(&self, team_id: u32, language: &str) -> Option<&str>` — Name the user gave `team_id` in this language, if any.
  - `pub fn set_custom_name(&mut self, team_id: u32, language: &str, name: &str, automatic: &str)` — Stores (or clears, when equal to the automatic name) the translation of one team.
  - `pub fn team_color(&self, m: &gw2lurker_core::Match) -> Option<TeamColor>` — Colour of the configured team inside the given match, if it can be resolved.
  - `fn sanitize(&mut self)` — Extra widgets: the pop-up time is a duration, not a unitless number.
- `pub fn addon_dir() -> io::Result<PathBuf>` — Directory where the addon may write, created if needed.
- `pub fn addon_dir_in(documents: &Path) -> PathBuf` — `<Documentos>\Guild Wars 2\addons\<addon>`, split out so it can be tested without the API.
- `fn legacy_addon_dir() -> Option<PathBuf>` — Folder Nexus used to hand us, where 0.1–0.4.2 kept the settings.
- `fn migrate_legacy(dir: &Path) -> Option<String>` — Copies an existing settings file from the old location the first time.
- `pub const SETTINGS_FILE` — Name of the settings file, like `settings.json` in the neighbouring addons.
- `pub const SETTINGS_BACKUP` — Previous good copy, also like the neighbours.
- `pub const LEGACY_FILE` — Name used by 0.1–0.3, still read so nobody loses what they configured.
- `pub fn config_path() -> io::Result<PathBuf>` — Path of the settings file.
- `pub fn backup_path() -> io::Result<PathBuf>` — Path of the previous good copy.
- `pub fn load() -> (Config, Option<String>)` — Loads the settings.
- `pub fn save(config: &Config) -> io::Result<()>` — Writes the settings atomically and keeps the previous file as `settings.json.bak`.
- `mod tests`

**Usa (interno):** `crate::ADDON_DIR`, `crate::ffi`, `crate::guild_index`, `crate::worker`, `gw2lurker_core::AlertFilter`, `gw2lurker_core::FollowList`, `gw2lurker_core::Match`, `gw2lurker_core::model`, `gw2lurker_core::normalize_guild_id`, `gw2lurker_core::Roster`

**APIs externas:** `nexus::paths::get_addon_dir`, `disco: fs::create_dir_all`, `disco: fs::copy`, `disco: fs::read_to_string`, `disco: fs::rename`, `disco: fs::write`, `serde_json::from_str`, `serde_json::to_string_pretty`


### `crates/addon/src/config/tests.rs` (317 líneas)

Tests de la configuración y del estado de las ventanas.

**Usa (interno):** `crate::ADDON_DIR`, `crate::guild_index`, `gw2lurker_core::Match`

**APIs externas:** `http://ejemplo/x.zip`, `https://ejemplo/x.zip`, `https://raw.githubusercontent.com/cbart-cmd/gw2addonreleases/main/gw2_lurker_guilds.zip`, `serde_json::from_str`, `serde_json::to_string`

**Tests (18):** `rf41_settings_live_under_documents_like_the_neighbours`, `rf19a_popup_time_defaults_to_two_seconds_and_clamps`, `rf19a_by_default_my_map_and_the_guilds_i_follow`, `rf41_defaults_are_usable`, `rf41_old_files_without_fields_still_load`, `rf03a_teams_saved_per_character_by_older_versions_are_not_carried_over`, `rf41_out_of_range_values_are_clamped`, `rf20c_an_index_url_without_https_falls_back_to_the_published_one`, `rf20c_the_broken_default_url_repairs_itself`, `rf20b_guild_names_are_stored_in_the_settings_file`, `rf20b_only_the_followed_guilds_keep_their_name_in_the_settings`, `rf20b_settings_canonicalize_old_lowercase_guild_ids`, `rf03b_configured_team_resolves_its_colour`, `rf73_followed_guilds_round_trip`, `rf46_window_state_defaults_are_visible_and_movable`, `rf46_window_state_survives_a_restart`, `rf46_impossible_window_values_are_discarded`, `rf46_unknown_window_key_does_not_panic`


### `crates/addon/src/context.rs` (52 líneas)

Glue between Nexus/Mumble and the pure gate in `core`.

**Contenido:**

- `pub fn snapshot() -> GameState` — Current game state as the core understands it.
- `pub fn gate() -> Gate` — Whether the UI may draw and the worker may poll (RF-01, RF-02, RF-02a).
- `pub fn map_role(state: GameState) -> Option<gw2lurker_core::MapRole>` — Map role in a Mumble snapshot, for the "only my map" alert option (RF-19a).
- `pub fn log_gate(gate: Gate)` — Logs the gate decision, but only when it changes.

**Usa (interno):** `crate::ffi`, `gw2lurker_core::gate`, `gw2lurker_core::MapRole`


### `crates/addon/src/detect.rs` (144 líneas)

Dónde está jugando el jugador, según el juego (RF-03a).

**Contenido:**

- `type Detection` — When the last detection ran and what came out of it.
- `const RETRY_AFTER_MISSING_DATA` — Espera antes de reintentar la detección del equipo cuando faltaban datos (60 s).
- `fn slot() -> &'static Mutex<Detection>` — Acceso perezoso a la última detección (instante y resultado).
- `pub fn due() -> bool` — Whether the nine matchups have to be asked for: **once per load**, not every quarter of an hour.
- `fn retry_due(at: Instant, now: Instant) -> bool` — Si han pasado 60 s desde el último intento de detección.
- `pub fn forget()` — Forgets the last sweep, so the next poll asks for the nine matchups again.
- `pub fn cached() -> Option<Detected>` — Último equipo/match detectado desde el juego, si lo hay, sin volver a calcularlo.
- `fn detection_line(…)` — What a detection has to say: a saved team that disagrees with the game, or a shard that is in no matchup. When the game and the saved team agree there is nothing to report.
- `pub fn detect_in(live: &[Match], saved: Option<u32>) -> Option<Detected>` — Asks Mumble which shard the player is on and looks it up in every matchup.

**Usa (interno):** `crate::diagnostics`, `crate::ffi`, `gw2lurker_core::detect`, `gw2lurker_core::model`, `gw2lurker_core::TeamColor`

**Tests (4):** `rf03a_missing_mumble_data_is_retried_after_a_cooldown`, `rf03a_a_detection_that_agrees_with_the_saved_team_says_nothing`, `rf03a_a_detection_that_disagrees_with_the_saved_team_says_both`, `rf03a_a_shard_in_no_matchup_is_reported_with_the_saved_team`


### `crates/addon/src/diagnostics.rs` (272 líneas)

What the worker writes to the log, apart from `worker` itself so that file stays within its size limit.

**Contenido:**

- `const HEARTBEAT_EVERY` — How often the heartbeat is written, whatever the poll interval is.
- `pub fn changed(last: &mut Option<String>, line: &str) -> bool` — Whether `line` is different from the last one written in this slot, remembering it if so.
- `fn watched_key(selection: &Selection, match_id: Option<&str>) -> String` — What identifies the watched matchup: id, region and selection, and nothing that moves.
- `pub fn log_watched_matchup(…)` — Says which matchup is being watched, and what came out of the poll.
- `pub fn last_claim_of(live: &[Match], match_id: Option<&str>) -> Option<DateTime<Utc>>` — `claimed_at` comes from the API and is the game's clock, not ours (measured: the newest claim of `1-1` was 21:02 UTC while other matchups were at 21:10, which is the difference between «the addon is s…
- `fn heartbeat_due(last: &mut Option<Instant>, now: Instant) -> bool` — Whether the heartbeat is due: the first poll writes it (it proves the worker is up) and then one every [`HEARTBEAT_EVERY`].
- `pub fn heartbeat(…)` — A line every five minutes with what the last poll saw.
- `pub fn log_where(configured: Option<u32>, match_id: Option<&str>, live: &[Match])` — Says, once per change, where the game says the player is and which team the addon watches.
  - `type Key` — (equipo configurado, shard de Mumble, color de Mumble)

**Usa (interno):** `crate::clock`, `crate::ffi`, `gw2lurker_core::diff`, `gw2lurker_core::model`, `gw2lurker_core::select`

**Tests (3):** `t35_a_state_line_is_written_once_until_it_changes`, `t35_the_watched_line_still_says_which_region_is_being_watched`, `t35_the_heartbeat_is_one_line_per_interval_whatever_the_poll_interval`


### `crates/addon/src/ffi/http.rs` (317 líneas)

Minimal HTTP GET over WinHTTP (no third-party client, no TLS inside the DLL).

**Contenido:**

- `type Handle` — Handle opaco de WinHTTP (`HINTERNET`).
- `fn wide(text: &str) -> Vec<u16>` — WinHTTP wants UTF-16 with a terminating NUL.
- `const ACCESS_TYPE_DEFAULT_PROXY` — `WINHTTP_ACCESS_TYPE_DEFAULT_PROXY`: usa el proxy configurado del sistema.
- `const FLAG_SECURE` — `WINHTTP_FLAG_SECURE`: la petición va por HTTPS.
- `const QUERY_STATUS_CODE` — `WINHTTP_QUERY_STATUS_CODE`: pide el código HTTP de la respuesta.
- `const QUERY_FLAG_NUMBER` — `WINHTTP_QUERY_FLAG_NUMBER`: devuelve la cabecera como número.
- `const MAX_BODY` — Tamaño máximo aceptado de una respuesta (8 MB); más se trata como error.
- `struct Owned` — RAII wrapper so no handle leaks on an early return.
- `impl Owned`
  - `fn new(handle: Handle, what: &str) -> Result<Self, ApiError>` — Envuelve un handle de WinHTTP recién creado; si es nulo devuelve error de transporte con el último error del sistema.
- `impl Drop for Owned`
  - `fn drop(&mut self)` — Cierra el handle con `WinHttpCloseHandle` una sola vez.
- `fn split_url(url: &str) -> Result<(bool, &str, u16, &str), ApiError>` — Splits a URL into `(secure, host, port, path)`.
- `pub fn get(url: &str, timeout_ms: i32) -> Result<(u16, Vec<u8>), ApiError>` — Performs the GET and returns `(status, body)`.
- `fn query_status(request: Handle) -> Result<u16, ApiError>` — Lee el código de estado HTTP de una petición respondida (`WinHttpQueryHeaders`).
- `fn error_message(call: &str) -> String` — Texto «<llamada> falló: <error del SO>» con `GetLastError`.

**Usa (interno):** `gw2lurker_api::ApiError`

**APIs externas:** `Win32: WinHttpOpen (winhttp)`, `Win32: WinHttpConnect (winhttp)`, `Win32: WinHttpOpenRequest (winhttp)`, `Win32: WinHttpSendRequest (winhttp)`, `Win32: WinHttpReceiveResponse (winhttp)`, `Win32: WinHttpQueryHeaders (winhttp)`, `Win32: WinHttpReadData (winhttp)`, `Win32: WinHttpSetTimeouts (winhttp)`, `Win32: WinHttpCloseHandle (winhttp)`

**Tests (3):** `urls_are_split_without_guessing`, `live_get_returns_the_match_json`, `live_the_index_zip_is_downloadable`


### `crates/addon/src/ffi/keys.rs` (82 líneas)

Keyboard and focus state for RF-45.

**Contenido:**

- `const VK_RCONTROL` — Virtual-key code of the right Control key.
- `pub fn os_primary_language() -> u16` — Primary language id of the Windows display language.
- `pub fn right_ctrl_down() -> bool` — Whether right Ctrl is being held.
- `pub fn layout_key_name(scan_code: u16) -> Option<String>` — Name of the key with this set-1 scan code **in the player's keyboard layout** (`Ñ` on a Spanish keyboard where the US table says `;`). `None` if Windows has no name for it.
- `pub fn game_has_focus() -> bool` — Whether the foreground window belongs to this process, that is, the game has focus.

**APIs externas:** `Win32: GetAsyncKeyState (user32)`, `Win32: GetForegroundWindow (user32)`, `Win32: GetWindowThreadProcessId (user32)`, `Win32: GetKeyNameTextW (user32)`, `Win32: GetUserDefaultUILanguage (kernel32)`


### `crates/addon/src/ffi/mod.rs` (155 líneas)

Reads what Nexus and Mumble expose. The only place with `unsafe`.

**Contenido:**

- `pub mod http`
- `pub mod keys`
- `pub mod paths`
- `pub fn is_gameplay() -> Option<bool>` — `NexusLink::is_gameplay`: `false` on loading screens and character select.
- `pub fn mumble_map() -> Option<(u32, u32)>` — `(map_id, map_type)` from MumbleLink, or `None` when the link is not available.
- `pub fn character_name() -> Option<String>` — Character name from MumbleLink. It is the **fallback key** of the team memory (RF-03a): the real key is the account name arcdps reports (`arc_team::self_account`), because the team is shared by every…
- `const GAME_NAME` — The game name MumbleLink publishes in its `name` field, which is not a character.
- `fn pick_character_name(mumble_name: &str, identity: &str) -> Option<String>` — Character name out of the two fields Mumble publishes. Pure, so it can be tested: calling the game pointers from a test panics (`AddonApi::get()` is not initialized outside the game).
- `pub fn identity_name(identity: &str) -> Option<String>` — Character name inside MumbleLink's `identity` JSON. Pure, so it can be tested: calling the game pointers from a test panics (`AddonApi::get()` is not initialized outside the game).
- `pub fn mumble_where() -> Option<(u32, Option<gw2lurker_core::TeamColor>)>` — What the game itself says about where the player fights: `(shard_id, team colour)`.
- `pub fn identity_team_colour(identity: &str) -> Option<gw2lurker_core::TeamColor>` — Team colour out of the `identity` JSON. Pure, so it can be tested.

**Usa (interno):** `gw2lurker_core::TeamColor`

**APIs externas:** `nexus::data_link::mumble::get_mumble_link`, `nexus::data_link::get_nexus_link`

**Tests (4):** `rf03a_character_name_comes_out_of_the_identity_json`, `rf03a_identity_without_name_is_none`, `rf03a_the_character_comes_from_the_identity_and_not_from_the_game_name`, `rf03a_the_team_colour_comes_out_of_the_identity_json`


### `crates/addon/src/ffi/paths.rs` (109 líneas)

Where the addon keeps its data.

**Contenido:**

- `struct Guid` — `FOLDERID_Documents` = `{FDD39AD0-238F-46AF-ADB4-6C85480369C7}`.
- `const FOLDERID_DOCUMENTS` — GUID de la carpeta Documentos del usuario para `SHGetKnownFolderPath`.
- `const KF_FLAG_DEFAULT` — Flags por defecto de `SHGetKnownFolderPath` (0).
- `pub fn documents() -> Option<PathBuf>` — The user's Documents folder, honouring a OneDrive redirect.

**APIs externas:** `Win32: SHGetKnownFolderPath (shell32)`, `Win32: CoTaskMemFree (ole32)`

**Tests (2):** `documents_is_an_existing_folder`, `the_guid_matches_the_documented_value`


### `crates/addon/src/guild_index.rs` (414 líneas)

Descarga del índice de nombres de clan (`guild-names.json`) desde el repositorio de distribución.

**Contenido:**

- `pub const DEFAULT_URL` — Dónde se publica el ZIP del índice. El jugador puede cambiarlo en los ajustes.
- `const MD5_SUFFIX` — Sufijo del hash publicado junto al ZIP (`x.zip` → `x.zip.md5`).
- `const ENTRY` — Nombre que se busca dentro del ZIP.
- `const MAX_ENTRIES` — Topes de lo que se acepta descomprimir de un ZIP que no controlamos.
- `const MAX_BYTES` — Tamaño máximo descomprimido del índice de clanes (64 MB), contra ZIP bomba.
- `pub enum Status` — What the settings window shows.
- `static STATUS` — Estado de la descarga del índice de clanes que muestra Ajustes.
- `fn slot() -> &'static Mutex<Status>` — Acceso perezoso a `STATUS`.
- `pub fn status() -> Status` — Current state, copied so the window never holds the lock while drawing.
- `fn set_status(status: Status)` — Cambia el estado de la descarga del índice.
- `pub fn ensure_present()` — Descarga el índice si no está en disco. Se llama una vez al cargar el addon.
- `pub fn download(url: String) -> bool` — Pide una descarga en su propio hilo. Devuelve `false` si ya hay una en marcha.
- `fn run(url: &str) -> Result<usize, String>` — Descarga, verifica y descomprime. Corre en el hilo de la descarga.
- `pub fn cache_busted(url: &str, stamp: i64) -> String` — URL con un parámetro de versión para **saltarse la caché del CDN**.
- `fn verify_published_md5(bytes: &[u8], published: &[u8]) -> Result<(), String>` — Compara el MD5 del ZIP descargado con el publicado; si no coincide o no se puede leer, no se instala.
- `fn describe(error: &gw2lurker_api::ApiError) -> String` — Mensaje del fallo de descarga, con el 404 explicado (es el que más veces se ve).
- `struct Limits` — Limits accepted while unpacking, so a hostile archive cannot fill the disk.
- `impl Default for Limits`
  - `fn default() -> Self` — Límites por defecto al descomprimir: 64 entradas y 64 MB.
- `fn unpack(bytes: &[u8], limits: Limits) -> Result<Vec<u8>, String>` — Bytes of `guild-names.json` inside the archive. Pure, so it can be tested with a fixture.

**Usa (interno):** `crate::background`, `crate::config`, `crate::md5`, `crate::names`, `crate::net`, `gw2lurker_api::ApiError`, `gw2lurker_api::Transport`

**APIs externas:** `https://raw.githubusercontent.com/cbart-cmd/gw2addonsreleases/main/gw2_lurker_guilds.zip`, `disco: fs::create_dir_all`, `serde_json::from_slice`

**Tests (14):** `rf20c_the_download_is_cache_busted`, `rf20c_the_default_index_url_is_the_distribution_repo`, `rf20c_an_unreadable_published_md5_rejects_the_index`, `rf20c_a_mismatched_published_md5_rejects_the_index`, `rf20c_a_matching_published_md5_accepts_the_index`, `rf20c_a_404_says_the_file_is_not_published`, `live_the_published_zip_unpacks_into_the_index`, `rf20_the_index_is_found_inside_the_zip`, `rf20_a_deflated_zip_is_read_too`, `rf20_the_index_is_found_inside_a_folder_of_the_zip`, `rf20_a_zip_without_the_index_is_an_error`, `rf20_a_near_match_name_is_not_the_index`, `rf20_a_zip_with_too_many_entries_is_refused`, `rf20_a_non_zip_is_refused`


### `crates/addon/src/i18n.rs` (213 líneas)

Language resolution for the UI (RF-44, RF-45).

**Contenido:**

- `static PREFERENCE` — Convierte `PREFERENCE` en `LanguagePreference`.
- `pub fn set_preference(preference: LanguagePreference)` — Updates the cached language preference without taking the runtime configuration lock.
- `fn preference() -> LanguagePreference` — Convierte `PREFERENCE` en `LanguagePreference`.
- `pub fn active() -> Language` — Language that must be used right now.
- `pub fn effective() -> Language` — Saved preference unless it is *Auto*, then the best source available, then English.
- `fn resolve(preference: LanguagePreference, sources: [Option<Language>; 3]) -> Language` — Precedence between the saved preference and the sources, most reliable first.
- `pub fn english_override() -> bool` — `true` while right Ctrl is held and the game is the foreground window.
- `pub fn text(key: &'static str) -> &'static str` — Translated text for the language in force.
- `pub fn widget(key: &'static str) -> String` — Translated text with a stable imgui id, for windows and labels whose state must survive a language change.
- `pub fn format(key: &'static str, args: &[&str]) -> String` — Translated text with `{0}`, `{1}`… replaced.
- `pub fn format_api_error(error: &ApiError) -> String` — API errors are translated when displayed, not when produced on a background thread.
- `fn format_api_error_for(language: Language, error: &ApiError) -> String` — Mensaje traducido de un `ApiError` (red, HTTP, parseo, no encontrado) en el idioma dado.
- `pub fn register_translations_once()` — Registers our strings in Nexus so its own UI can show them in the user's language.

**Usa (interno):** `crate::config`, `crate::ffi`, `crate::lang`, `gw2lurker_api::ApiError`, `gw2lurker_core::i18n`

**APIs externas:** `nexus::localization::set_translation`

**Tests (5):** `rf44_auto_takes_the_most_reliable_source`, `rf44_explicit_preference_beats_everything`, `rf44_language_preference_is_cached_without_a_config_lock`, `rf44_api_errors_are_localized_by_variant`, `rf45_english_override_uses_the_english_column`


### `crates/addon/src/keybinds.rs` (449 líneas)

La combinación de teclas que **Nexus tiene guardada** para el atajo del addon (RF-18).

**Contenido:**

- `pub const IDENTIFIER` — Identificador con el que se registra el atajo (Nexus lo lista con este nombre).
- `const UNKNOWN` — Lo que el jugador ve cuando el atajo no se puede leer: el identificador, que es inconfundible.
- `struct Bind` — Una entrada de `InputBinds.json`.
- `struct Shown` — Lo último leído del fichero y su descripción. `pending` es una lectura nueva que aún no se ha convertido en texto: se convierte en el hilo que llama a [`shortcut`].
- `impl Shown`
  - `fn new() -> Self` — Estado inicial vacío: sin descripción y sin lectura pendiente.
  - `fn current(&mut self, layout: impl Fn(u16) -> Option<String>) -> &str` — Aplica la lectura pendiente, si la hay, y devuelve la descripción vigente.
- `static SHOWN` — Descripción vigente del atajo y lectura pendiente de traducir a texto.
- `static WATCH` — Vigilante de `InputBinds.json` (ruta y fecha); lo crea `remember_shortcut` y lo usa `refresh`.
- `pub fn shortcut() -> String` — Descripción del atajo según lo último que Nexus ha guardado. No toca el disco.
- `pub fn remember_shortcut()` — Lee `InputBinds.json` por primera vez y deja la ruta para [`refresh`]. Se llama desde `load` y nunca desde el render.
- `pub fn refresh()` — Vuelve a leer el atajo si Nexus ha reescrito el fichero. La llama el worker, nunca el render.
- `fn store(read: Option<Bind>)` — Deja una lectura nueva del atajo como pendiente en `SHOWN`.
- `fn binds_path() -> Option<PathBuf>` — `addons/Nexus/InputBinds.json` dentro de la carpeta del juego.
- `struct Watch` — Vigila la fecha de modificación del fichero para leerlo solo cuando cambia.
- `impl Watch`
  - `fn new(path: PathBuf) -> Self` — Vigilante para una ruta, aún sin fecha conocida (la primera consulta siempre lee).
  - `fn poll(&mut self) -> Option<Option<Bind>>` — `None` si el fichero no ha cambiado desde la última vez; si no, nuestro atajo tal como está ahora (`Some(None)` si ya no está o no se puede leer).
- `fn find_bind(json: &str) -> Option<Bind>` — Nuestro atajo dentro del JSON de Nexus.
- `fn describe_identifier(json: &str, layout: impl Fn(u16) -> Option<String>) -> Option<String>` — Descripción del atajo de `identifier` dentro del JSON. Pura: se prueba con el fichero real.
- `fn describe(bind: &Bind, layout: impl Fn(u16) -> Option<String>) -> String` — `Ctrl+Shift+L` a partir de los modificadores y el código de escaneo.
- `fn key_name(code: u16) -> Option<&'static str>` — Nombre de la tecla en la distribución **US** a partir del código de escaneo (set 1): solo cuando Windows no da el de la distribución del jugador. `None` si no se conoce, que es lo que evita inventarse…

**Usa (interno):** `crate::ffi`

**APIs externas:** `nexus::paths::get_game_dir`, `disco: fs::metadata`, `disco: fs::read_to_string`, `serde_json::from_str`

**Tests (8):** `rf18_the_shortcut_is_read_from_what_nexus_has_saved`, `rf18_the_key_is_named_with_the_players_keyboard_layout`, `rf18_windows_names_a_key_in_the_current_layout`, `rf18_a_rebinding_made_while_playing_reaches_the_message`, `rf18_another_addons_binding_is_not_ours`, `rf18_a_key_we_do_not_know_is_not_invented`, `rf18_modifiers_appear_in_a_stable_order`, `rf18_alt_is_read_from_nexuss_saved_binding`


### `crates/addon/src/lang.rs` (160 líneas)

Where the interface language comes from (RF-44).

**Contenido:**

- `static EXTRAS_CODE` — Raw GW2 language code reported by Extras; `-1` until the first event arrives.
- `static NEXUS_LANGUAGE` — Nexus language, resolved once because it costs two file reads.
- `pub fn subscribe()` — Subscribes to the Extras language event.
- `fn on_language(payload: Option<&nexus::event::extras::Language>)` — Extras sends the current language right after initialization, so this also covers startup.
- `pub fn from_extras() -> Option<Language>` — Game client language as reported by Extras, if it has told us yet.
- `pub fn from_nexus_settings() -> Option<Language>` — Nexus language, read once from its own files.
- `fn read_nexus_language() -> Option<Language>` — Idioma de Nexus: lee `Nexus/Settings.json` y busca en `Nexus/Locales` el código de ese nombre visible.
- `struct NexusSettings` — Parte de `Nexus/Settings.json` que interesa: el idioma.
- `fn parse_settings_language(json: &str) -> Option<String>` — Display name Nexus has stored (`"Español"`), trimmed.
- `struct LocaleFile` — Fichero de `Nexus/Locales`: código del idioma y nombre visible.
- `fn locale_identifier(dir: &std::path::Path, display: &str) -> Option<String>` — Two-letter identifier of the locale whose display name matches, e.g. `Español` → `es`.

**Usa (interno):** `gw2lurker_core::i18n`

**APIs externas:** `nexus::event::extras::LANGUAGE_CHANGED`, `nexus::event`, `nexus::event::extras::Language`, `nexus::paths::get_game_dir`, `disco: fs::read_to_string`, `disco: fs::read_dir`, `serde_json::from_str`

**Tests (5):** `rf44_nexus_settings_language_is_read`, `rf44_nexus_settings_without_language_is_none`, `rf44_locale_identifier_matches_display_name`, `rf44_extras_codes_match_the_game_enum`, `rf44_no_extras_event_means_no_answer`


### `crates/addon/src/lib.rs` (332 líneas)

Raidcore Nexus addon (cdylib).

**Contenido:**

- `mod alerts`
- `mod arc_team`
- `mod assignment`
- `mod background`
- `mod catalog`
- `mod clock`
- `mod config`
- `mod context`
- `mod detect`
- `mod diagnostics`
- `mod ffi`
- `mod guild_index`
- `mod i18n`
- `mod keybinds`
- `mod lang`
- `mod md5`
- `mod names`
- `mod net`
- `mod search`
- `mod session_names`
- `mod team`
- `mod ui`
- `mod worker`
- `mod writer`
- `pub const ADDON_DIR` — Directory name used by Nexus for this addon (`addons/gw2_lurker/`).
- `static CONFIG` — Configuration, loaded once on load. If it cannot be read the addon runs with defaults; T-26 will surface that as the safe-mode notice (RF-51).
- `static RUNTIME` — Editable copy the UI mutates (window positions, opacity, lock…). Written by [`writer::Writer`], so the render callback never touches the disk (rule 6).
- `static WRITER` — Background writer for `settings.json`. `Option` so `unload` can take it and join.
- `static SAFE_MODE` — Why the configuration could not be read, if it could not (RF-51).
- `pub fn safe_mode() -> Option<String>` — The load report, `Some` when the addon is running on defaults after a broken file (RF-51).
- `pub fn runtime() -> MutexGuard<'static, config::Config>` — Editable configuration used by the UI.
- `pub fn request_save(config: &config::Config)` — Asks the writer to persist the configuration. Cheap enough for the render callback.
- `pub fn persist_if_changed()` — Persists the configuration when it changed, at most twice a second.
- `pub fn loaded_version() -> &'static str` — Version loaded in this session, from the crate metadata.
- `static LOADED_DLL` — Size and modification time of our own DLL, taken at load.
- `const DLL_CHECK_INTERVAL` — Cada cuánto se mira si la DLL en disco cambió (10 s).
- `struct DllWatch` — Vigilante de la DLL: última comprobación y ruta si ya se detectó que en disco hay otra versión.
- `impl DllWatch`
  - `fn should_check(&mut self, now: std::time::Instant) -> bool` — Si toca comprobar la DLL (han pasado 10 s y no se detectó antes un cambio).
  - `fn observe(…)` — Compara tamaño y fecha de la DLL cargada con la de disco; marca y devuelve `true` la primera vez que difieren.
- `static DLL_WATCH` — Instancia global del vigilante de la DLL.
- `fn own_dll_path() -> Option<std::path::PathBuf>` — Path of the DLL Nexus put on disk: `<juego>/addons/gw2_lurker.dll`.
- `fn dll_fingerprint(path: &std::path::Path) -> Option<(u64, std::time::SystemTime)>` — Huella de un fichero: tamaño y fecha de modificación.
- `fn remember_loaded_dll()` — Remembers the DLL as it was at load, so a later change can be noticed (RF-50).
- `pub fn check_dll_update()` — Checks for a replacement from the worker thread, at most once every ten seconds.
- `pub fn stale_dll() -> Option<String>` — Whether Nexus has replaced the file on disk since this session started.
- `fn load()` — Arranque del addon (Nexus): carga settings.json (modo seguro si está roto), idioma, huella de la DLL, atajo, índice de clanes, suscripciones, writer, UI y worker.
- `fn unload()` — Descarga: para el worker, guarda la configuración vigente (writer o guardado directo) y deja constancia en el log.
- `pub fn config() -> &'static config::Config` — Configuration as it was loaded: what the worker reads, never changes.

**Usa (interno):** `crate::load`, `crate::unload`, `gw2lurker_core::i18n`

**APIs externas:** `nexus::export`, `nexus::AddonFlags`, `nexus::UpdateProvider`, `nexus::paths::get_game_dir`, `https://raw.githubusercontent.com/cbart-cmd/gw2addonsreleases/main/gw2_lurker.dll`, `disco: fs::metadata`

**Tests (1):** `rf50_dll_replacement_is_detected_after_an_unchanged_check`


### `crates/addon/src/md5.rs` (190 líneas)

MD5 en hexadecimal, para verificar lo que se descarga.

**Contenido:**

- `pub fn hex(bytes: &[u8]) -> String` — MD5 de `bytes`, en 32 caracteres hexadecimales en minúsculas.
- `pub fn parse(text: &str) -> Option<String>` — Hash of some text written as a `.md5` file: 32 hex digits, with or without a line ending.
- `const SHIFTS` — Rotaciones por ronda del algoritmo MD5 (RFC 1321).
- `fn digest(bytes: &[u8]) -> [u8; 16]` — MD5 de unos bytes (RFC 1321, implementación propia para no añadir dependencias).

**Tests (2):** `rf20_md5_matches_the_published_vectors`, `rf20_a_published_md5_is_read_with_or_without_a_line_ending`


### `crates/addon/src/names.rs` (458 líneas)

Índice de nombres de clan (`guild-names.json`): una **caché de búsqueda**, no un dato vivo.

**Contenido:**

- `pub const FILE` — Fichero del índice dentro de la carpeta del addon.
- `static INDEX_LEN` — Cuántos nombres tiene el índice, para que la ventana lo diga sin leer el fichero.
- `static INDEX_IO` — Candado que serializa lecturas y escrituras de `guild-names.json` en disco.
- `pub fn path() -> Option<PathBuf>` — Ruta del fichero, o `None` si la carpeta del addon no está disponible.
- `pub fn exists() -> bool` — Whether the index is on disk. `false` means the download has to bring it.
- `pub fn index_len() -> usize` — Size of the index, as last seen by a reader or a writer. `0` means «not read yet».
- `pub fn read() -> BTreeMap<String, String>` — Reads the index. **Call it from a thread that is not the render one**, and drop the result when the search is done: it is several thousand entries.
- `fn read_at(path: &std::path::Path) -> BTreeMap<String, String>` — [`read`] from a given file.
- `fn read_strict(path: &std::path::Path) -> std::io::Result<BTreeMap<String, String>>` — Reads the index, telling «there is no file» (empty) apart from «there is one I cannot read».
- `fn normalize_names(names: BTreeMap<String, String>) -> BTreeMap<String, String>` — Normaliza los ids de un mapa id→nombre; si dos ids normalizan igual, gana el primero.
- `pub fn search_in(…)` — Clanes del índice cuya etiqueta (nombre o tag) contiene `needle`, sin distinguir mayúsculas.
- `fn rank(label: &str, needle: &str) -> u8` — How well an entry matches: `0` exact tag, `1` tag prefix, `2` name prefix, `3` anything else.
- `fn split_label(label: &str) -> (Option<&str>, &str)` — `[TAG] Name` → (`Some("TAG")`, `"Name"`); a label without a tag is all name.
- `pub fn matches_text(label: &str, id: &str, needle: &str) -> bool` — Whether an entry matches what the player typed. Pure, so the rule can be tested.
- `fn matches_normalized_text(label: &str, id: &str, needle: &str) -> bool` — Coincidencia de búsqueda: el nombre contiene el texto o el id empieza por él (en minúsculas); texto vacío no coincide.
- `pub fn append_async(entries: Vec<(String, String)>)` — Añade nombres que faltaban al índice, en su propio hilo.
- `pub fn append(entries: &[(String, String)]) -> std::io::Result<usize>` — Adds the entries the index did not have yet and writes it back. Returns how many were new.
- `pub fn append_at(path: &std::path::Path, entries: &[(String, String)]) -> std::io::Result<usize>` — [`append`] against a given file, so it can be tested without touching the player's index.
- `pub fn install_at(path: &std::path::Path, downloaded: &[u8]) -> std::io::Result<usize>` — Installs a downloaded index while preserving names appended during this session.

**Usa (interno):** `crate::background`, `crate::config`, `crate::guild_index`, `gw2lurker_core::normalize_guild_id`

**APIs externas:** `disco: fs::read`, `disco: fs::write`, `disco: fs::rename`, `serde_json::from_slice`, `serde_json::to_vec`

**Tests (11):** `t44_the_search_is_a_substring_and_ignores_case`, `t44_two_letters_are_the_minimum`, `t44_a_single_multibyte_character_is_still_one_character`, `t44_die_finds_the_three_and_the_shortest_come_first`, `t44_names_that_start_with_the_text_come_before_the_rest`, `t44_the_limit_is_respected`, `t44_an_exact_tag_comes_before_names_that_contain_the_text`, `rf20d_an_unreadable_index_is_never_overwritten`, `rf20d_a_new_guild_is_added_to_the_index_and_never_duplicated`, `rf20d_index_guild_ids_are_normalized_before_storage`, `rf20_concurrent_download_and_append_preserve_both_indexes`


### `crates/addon/src/net.rs` (54 líneas)

`gw2lurker_api::Transport` over WinHTTP.

**Contenido:**

- `pub struct WinHttpTransport` — Transport backed by the WinHTTP service that ships with Windows.
- `impl Default for WinHttpTransport`
  - `fn default() -> Self` — Transporte WinHTTP con 10 s de timeout.
- `impl Transport for WinHttpTransport`
  - `fn get(&self, url: &str) -> Result<Vec<u8>, ApiError>` — GET por WinHTTP: 2xx devuelve el cuerpo, 404 `NotFound` y el resto `ApiError::Status`.

**Usa (interno):** `crate::ffi`, `gw2lurker_api::Transport`

**Tests (1):** `rf20_http_404_maps_to_not_found`


### `crates/addon/src/search.rs` (308 líneas)

Guild search for the follow selector (T-44).

**Contenido:**

- `const MAX_LOOKUPS` — Ids looked up per search. One call each, and the answer rarely needs more than a handful.
- `const MAX_HITS` — Hits kept per search.
- `const TEAMS_TTL` — How long the guild → team map is considered fresh (it only changes at reset).
- `pub struct Hit` — One guild found by the search.
- `pub enum Failure` — Fallo de una búsqueda de clanes: de la API o interno.
- `impl From<ApiError> for Failure`
  - `fn from(error: ApiError) -> Self` — Convierte un `ApiError` en `Failure::Api`.
- `pub enum State` — What the window draws while the thread works.
- `impl State`
  - `pub fn hits(&self) -> &[Hit]` — Hits on screen right now, whatever the state: lo que se puede clicar mientras se busca.
- `static STATE` — Estado de la búsqueda en curso (inactiva, buscando, resultados o error).
- `fn slot() -> &'static Mutex<State>` — Acceso perezoso a `STATE`.
- `pub fn snapshot() -> State` — Current state, copied so the window never holds the lock while drawing.
- `pub fn start(region: Region, team: Option<u32>, text: String, mode: Mode) -> bool` — Starts a search on its own thread. Returns `false` if one is already running.
- `pub enum Mode` — What a search is allowed to do.
- `fn run(region: Region, team: Option<u32>, text: &str, mode: Mode) -> Result<Vec<Hit>, Failure>` — Busca un clan (en su hilo): primero en el índice local `guild-names.json`; en modo API además `/v2/guild/search`, `/v2/guild/:id` y `/v2/wvw/guilds/:region` para marcar si es del equipo.
- `type TeamCache` — Guild → team of a region, with the moment it was fetched.
- `fn teams_of(…)` — Guild → team of a region, cached because the answer is ~280 KB and only changes at reset.

**Usa (interno):** `crate::background`, `crate::catalog`, `crate::names`, `crate::net`, `gw2lurker_core::normalize_guild_id`, `gw2lurker_core::Region`

**Tests (3):** `t44_a_search_is_not_started_on_top_of_another`, `t44_the_results_on_screen_survive_a_new_search`, `t44_only_guild_ids_are_looked_up`


### `crates/addon/src/session_names.rs` (167 líneas)

Nombres de los clanes que han reclamado algo **en esta sesión**, en `temp.json` (RF-20b).

**Contenido:**

- `pub const FILE` — Fichero de la sesión dentro de la carpeta del addon.
- `pub const MAX_NAMES` — Tope de seguridad: una sesión muy larga no debe dejar un fichero enorme.
- `static SLOT` — Acceso perezoso a `SLOT`.
- `fn slot() -> &'static Mutex<BTreeMap<String, String>>` — Acceso perezoso a `SLOT`.
- `pub fn path() -> Option<PathBuf>` — Ruta del fichero, o `None` si la carpeta del addon no está disponible.
- `pub fn reset()` — Vacía la sesión y **borra** el fichero: se llama al cargar el addon, así que cada vez que se entra al juego se empieza de cero.
- `pub fn all() -> BTreeMap<String, String>` — Copia de lo que hay en la sesión, para publicarlo con el dashboard.
- `fn normalize_names(names: &BTreeMap<String, String>) -> BTreeMap<String, String>` — Copia un mapa id→nombre con los ids normalizados (gana el primero si chocan).
- `pub fn merge(names: &BTreeMap<String, String>) -> bool` — Añade los nombres que faltaban. Devuelve `true` si creció (y hay que escribir el fichero).
- `pub fn save() -> std::io::Result<usize>` — Escribe la sesión en `temp.json`. Devuelve cuántos nombres hay.
- `pub fn save_to(path: &std::path::Path, names: &BTreeMap<String, String>) -> std::io::Result<()>` — [`save`] contra una ruta dada, para poder probarlo sin tocar los ficheros del jugador.

**Usa (interno):** `crate::config`, `gw2lurker_core::normalize_guild_id`

**APIs externas:** `disco: fs::remove_file`, `disco: fs::write`, `disco: fs::rename`, `serde_json::to_vec`

**Tests (3):** `rf20b_the_session_file_is_written_and_read_back`, `rf20b_session_name_ids_are_normalized`, `rf20b_the_session_never_grows_past_the_cap`


### `crates/addon/src/team.rs` (359 líneas)

Which team the current **account** plays for (RF-03a).

**Contenido:**

- `pub fn key(account: Option<&str>, character: Option<&str>) -> Option<String>` — Key the team is saved under: the **account** wins; the character is the fallback while arcdps has not said which account we are in, or is not installed.
- `pub fn current_key() -> Option<String>` — Key of the player playing right now, or `None` if neither source has an answer yet.
- `pub fn selected_in(config: &Config) -> Option<u32>` — Team of the current account, reading an already-locked configuration.
- `pub fn resolve(…)` — The actual rule, pure so it can be tested without Nexus: the manual override wins, then the entry under the current key, which is the account (or the character only while arcdps has not said which acc…
- `pub fn remember_in(config: &mut Config, team_id: u32)` — Remembers the team for the current account and clears the manual override.
- `pub fn forget_in(config: &mut Config)` — Forgets the team of the current account.
- `fn remember_under(config: &mut Config, key: Option<&str>, team_id: u32)` — The rule behind [`remember_in`], pure so it can be tested: the pick is saved under the same key [`resolve`] reads, and it replaces a manual override, which would hide it otherwise.
- `fn forget_under(config: &mut Config, key: Option<&str>)` — The rule behind [`forget_in`]: it clears the key [`resolve`] reads and no other.
- `pub fn selected() -> Option<u32>` — Team of the current account. Locks the configuration: never call it with a guard alive.
- `pub fn remember(team_id: u32)` — Remembers the team for the current account. Locks the configuration.
- `pub fn forget()` — Forgets the team of the current account. Locks the configuration.
- `pub fn automatic_name(language: &str, team_id: u32) -> String` — Name the team has in the bundled tables for a language (no override).
- `pub fn display_name(config: &Config, language: &str, team_id: u32) -> String` — Name shown to the player: their own translation wins over the bundled one (RF-03f).

**Usa (interno):** `crate::arc_team`, `crate::config`, `crate::ffi`, `crate::runtime`, `gw2lurker_core::bundled`, `gw2lurker_core::Roster`

**Tests (9):** `rf03f_a_translation_belongs_to_one_team`, `rf03a_override_wins_over_the_roster`, `rf03a_the_team_is_remembered_per_account_and_shared_by_its_characters`, `rf03a_a_character_entry_is_not_read_once_the_account_is_known`, `rf03a_picking_a_team_saves_it_under_the_account_only`, `rf03a_forgetting_clears_the_key_that_was_read_and_nothing_else`, `rf03f_the_players_translation_wins_over_the_bundled_one`, `rf03f_writing_the_automatic_name_is_not_an_override`, `rf03f_the_old_single_alias_is_still_honoured`


### `crates/addon/src/ui/alerts.rs` (750 líneas)

Alert widget (T-13, RF-19): the recent captures and claims, newest first.

**Contenido:**

- `pub const KEY` — Translation key (and stable id) of this window.
- `pub const LOG_KEY` — Translation key (and stable id) of the expanded log window.
- `const POPUP_KEY` — Translation key of the pop-up: separate window, separate id, state never saved.
- `pub struct LineContext` — Hora, equipo propio y estados de objetivos para formatear una línea.
- `const MAX_LINES` — How many lines fit in the window without scrolling forever; the player can ask for fewer (RF-19c).
- `fn keeps(kind: AlertKind) -> bool` — Whether the player wants this kind of alert at all (RF-19c).
- `pub const WIDGET_MINUTES` — How long an alert stays in the compact widget, in minutes.
- `pub const MAX_LOGGED` — How many lines the *Alert log* draws at most, the newest first.
- `const GLYPH_WIDTH` — Space reserved ahead of each line for the drawing of the objective (T-47).
- `const GLYPH_HALF` — Half size of that drawing: it has to fit in a line of text.
- `pub fn body(ui: &Ui, alerts: &[AlertLine], context: &LineContext<'_>) -> Option<String>` — Body of the *Alerts* tab: what just happened, so it is readable during a fight.
- `pub fn log_body(ui: &Ui, alerts: &[AlertLine], context: &LineContext<'_>) -> Option<String>` — Body of the *Alert log* tab: the whole session, scrolling if needed.
- `fn draw_faded_lines(…)` — One line per alert, each preceded by the drawing of what was taken (T-47).
- `pub fn within(lines: &[AlertLine], now: DateTime<Utc>, minutes: i64) -> Vec<AlertLine>` — The lines still worth a place in the widget: those that arrived in the last `minutes`.
- `pub fn draw_popup(…)` — Dibuja el emergente sin cromo; solo captura el ratón durante la colocación.
- `fn build_popup_window<T: AsRef<str>>(…)` — Dibuja el emergente y guarda su posición antes de que imgui cierre la ventana.
- `fn draw_line(…)` — One line, in coloured pieces. Returns `true` when the player clicked it and it names a guild.
- `fn draw_line_faded(…)` — Dibuja una línea RF-19f con su fundido y detecta el clic en un clan.
- `fn spans(…)` — Construye los fragmentos traducidos y coloreados del aviso desde el equipo propio.
- `fn age_text(…)` — Formatea la antigüedad en segundos, minutos o horas y minutos.
- `const GREY` — Neutral grey: everything that is not the map or an owner colour.
- `const HIGH_VALUE` — Color de la marca de objetivo de alto valor.
- `fn map_piece(…)` — The map label matches the rest of the addon; borderlands keep their team colour.
- `fn team_colour(colour: TeamColor) -> [f32; 4]` — Color de equipo, el mismo que en el mapa y en el historial (RF-62).
- `fn colour_label(colour: TeamColor, language: gw2lurker_core::i18n::Language) -> &'static str` — Nombre traducido de un color de equipo (Rojo/Verde/Azul).

**Usa (interno):** `crate::clock`, `crate::config`, `crate::i18n`, `crate::worker`, `gw2lurker_core::alerts`, `gw2lurker_core::i18n`, `gw2lurker_core::MapRole`, `gw2lurker_core::matchup`, `gw2lurker_core::model`, `super::icons`, `super::popup_stack`, `super::theme`

**APIs externas:** `nexus::imgui`, `nexus::imgui::MouseButton`, `imgui::MouseButton`, `imgui (dibuja con &Ui)`

**Tests (11):** `rf19d_placement_remembers_the_popup_position_not_the_parent`, `rnf01_popup_stack_and_native_draw_budget`, `rf19_the_widget_forgets_what_is_old`, `rf19d_every_alert_the_player_follows_can_be_interrupted`, `rf19_a_flip_names_both_colours`, `rf21_claims_name_the_guild`, `rf19_unclaim_says_released`, `rf19_the_map_is_drawn_in_its_own_colour_and_the_rest_in_grey`, `rf19f_line_uses_my_team_and_marks_high_value`, `rf19f_relative_age_uses_seconds_minutes_and_hours`, `rf19_the_owner_colours_are_told_apart`


### `crates/addon/src/ui/follow.rs` (627 líneas)

Followed guilds selector (T-44).

**Contenido:**

- `const MAX_SESSION_ROWS` — How many guilds the current-tracking-cycle shortcut offers.
- `static SEARCH_TEXT` — What the player typed in the search box. Session state on purpose: it is a tool, not a setting.
- `fn text_slot() -> std::sync::MutexGuard<'static, String>` — Texto del buscador de clanes de la pestaña Clanes (persistente entre fotogramas).
- `static PENDING_REMOVE` — Guild waiting for a *yes* before it stops being followed.
- `fn pending_remove() -> std::sync::MutexGuard<'static, Option<String>>` — Clan pendiente de confirmar para dejar de seguir.
- `const REMOVE_WIDTH` — Width reserved on the right of each followed row for the `×`.
- `pub enum Action` — Qué pidió el jugador en un favorito: ver su historial (`Select`) o abrir su ruta en Mapa (`ShowOnMap`).
- `fn followed_action(guild: &str, row_clicked: bool, map_clicked: bool) -> Option<Action>` — Decide la acción de una fila de favorito: el botón `i` gana sobre el clic en el nombre.
- `pub fn followed_section(…)` — Guilds the player follows, with a `×` to stop following each one.
- `pub fn add_section(…)` — Ways of following a new guild: what claimed something this session, and the search.
- `fn label(names: &BTreeMap<String, String>, id: &str) -> String` — `[TAG] Nombre` when it is known, the raw id otherwise.
- `pub fn add_followed(…)` — Guarda en los ajustes el nombre que ya se conoce de un clan al que se empieza a seguir, y devuelve la entrada que hay que enseñarle al índice (`names::append_async`).
- `fn remember_name(…)` — Guarda en la config el nombre conocido de un clan al seguirlo y devuelve lo guardado.
- `fn session_block(…)` — Guilds that claimed something during this tracking cycle: one click and they are followed.
- `fn search_block(…)` — Buscador de clanes: **índice local primero** —subcadena, sin red— y el API como extra.
- `const LOCAL_DELAY` — Cuánto tiene que estar quieto el texto antes de buscar en el índice.
- `struct Typing` — Lo que el jugador lleva escrito y si ya se buscó: sin esto, el fotograma siguiente al de una búsqueda volvería a lanzarla, y otra vez cada [`LOCAL_DELAY`].
- `static KEYSTROKE` — Último texto tecleado en el buscador y cuándo, para buscar tras una pausa (debounce).
- `fn typing() -> std::sync::MutexGuard<'static, Typing>` — Acceso perezoso a `KEYSTROKE`.
- `fn typing_settled(needle: &str) -> bool` — Whether the text has stopped changing long enough to search it, and has not been searched yet.
- `fn mark_searched()` — Marca el texto como «ya buscado» para que no se repita hasta que cambie.
- `fn note_typing(needle: &str)` — Reinicia el reloj porque el jugador acaba de pedir una búsqueda con Enter o el botón.
- `fn team_label(team: u32) -> String` — Name of a team as the player knows it: their own translation first, then the bundled name from `data/wvw-teams.json`, and the id in brackets to disambiguate.
- `pub fn region_of(candidates: &[Candidate], team: Option<u32>) -> Option<Region>` — Region of the configured team, for the search thread.

**Usa (interno):** `crate::clock`, `crate::config`, `crate::i18n`, `crate::names`, `crate::search`, `crate::session_names`, `crate::team`, `crate::worker`, `gw2lurker_core::model`, `gw2lurker_core::normalize_guild_id`, `super::guilds`

**APIs externas:** `nexus::imgui`, `nexus::imgui::CollapsingHeader`, `imgui::CollapsingHeader`, `imgui (dibuja con &Ui)`

**Tests (6):** `rf73_favourite_labels_select_history_and_only_the_info_button_opens_map`, `t44_the_search_region_is_the_one_of_the_players_team`, `t44_without_a_team_there_is_no_region_and_the_ui_says_so`, `t44_the_guilds_you_follow_are_found_whatever_the_api_answers`, `rf20_the_session_list_shows_names_and_not_ids`, `t44_the_search_is_launched_once_per_text`


### `crates/addon/src/ui/guilds.rs` (264 líneas)

Guilds of the matchup and the history of the chosen one (T-37, T-38; RF-71, RF-72).

**Contenido:**

- `pub const KEY` — Translation key (and stable id) of this window.
- `static SELECTED` — Guild whose history is on screen. Session state, like the other view filters.
- `pub enum Action` — What the player did in the window this frame.
- `pub struct Context` — Contexto de la lista de clanes: mapa actual del jugador y desde cuándo hay historial.
- `fn selected() -> Option<String>` — Clan seleccionado cuyo historial se muestra debajo.
- `pub fn selected_guild() -> Option<String>` — Guild whose history is on screen, if any. The map tab draws its route (T-16).
- `pub fn select(guild: &str)` — Selects a guild from outside this tab: an alert line or the list of followed guilds was clicked, and the route of that guild is what the player wants to see (request of 2026-10-31).
- `fn set_selected(guild: Option<String>)` — Cambia (o quita) el clan seleccionado.
- `pub fn body(…)` — Body of the *Guilds* tab. `history` is the current tracking cycle filtered by guild.
- `fn draw_group(ui: &Ui, heading: &'static str, rows: &[gw2lurker_core::matchup::ClaimRow])` — Dibuja un bloque del historial con título: hora, objetivo y mapa de cada reclamo.
- `fn map_label(role: MapRole) -> &'static str` — Nombre traducido de un mapa por su rol (Eternos, alpinos, desierto).

**Usa (interno):** `crate::clock`, `crate::i18n`, `gw2lurker_core::matchup`

**APIs externas:** `nexus::imgui`, `nexus::imgui::ChildWindow`, `nexus::imgui::Selectable`, `nexus::imgui::InputText`, `imgui::ChildWindow`, `imgui::Selectable`, `imgui::InputText`, `imgui (dibuja con &Ui)`

**Tests (2):** `rf72_the_selection_survives_a_frame_and_can_be_cleared`, `rf71_guilds_are_listed_once_each`


### `crates/addon/src/ui/icons.rs` (300 líneas)

Objective glyphs drawn with primitives: the little drawing inside the team-coloured circle of the game (T-46) and the same glyph ahead of every alert line (T-47).

**Contenido:**

- `pub const INK` — Colour of the drawing inside the circle: white, like the game, so the team colour stays the only colour on screen and the shape carries the kind (RF-30).
- `const OUTLINE` — Dark outline around the circle, which is what makes it readable over the map (T-46).
- `pub enum Shape` — One primitive of a glyph.
- `pub enum Ink` — Which colour a [`Part`] is painted with.
- `pub struct Part` — One primitive of a glyph, in screen coordinates.
- `pub fn has_glyph(kind: ObjectiveType) -> bool` — Whether a kind has a drawing of its own.
- `pub fn parts(kind: ObjectiveType, centre: [f32; 2], half: f32) -> Vec<Part>` — The primitives of a glyph, centred on `centre` and fitting in a box of `2 · half` a side.
- `pub fn badge(…)` — The circle of the game with the drawing inside: team colour outside, white glyph inside.
- `pub fn glyph(…)` — Paints a glyph on its own, without the circle: used by the alert lines (T-47).
- `pub fn paint(list: &DrawListMut<'_>, parts: &[Part], ink: [f32; 4], paper: [f32; 4])` — Paints the primitives. `paper` is the colour of the holes.

**Usa (interno):** `gw2lurker_core::model`

**APIs externas:** `nexus::imgui::DrawListMut`, `imgui::DrawListMut`

**Tests (5):** `rf30_the_four_objectives_that_score_have_their_own_drawing`, `rf30_each_objective_is_told_apart_by_its_drawing_and_not_only_by_colour`, `rf30_a_tower_is_taller_than_a_camp_and_a_keep_wider_than_a_tower`, `rf38_a_generic_kind_still_gets_a_mark`, `rf38_the_glyph_is_centred_where_it_was_asked_to_be`


### `crates/addon/src/ui/map.rs` (612 líneas)

Simplified map of one WvW map (T-14, RF-30).

**Contenido:**

- `mod route`
- `mod scene`
- `mod tiles`
- `mod timeline`
- `mod trail_tests`
- `pub const KEY` — Translation key (and stable id) of this tab.
- `const UNCLAIMED` — Grey of an objective nobody holds.
- `const FOLLOWED_RING` — Ring drawn around the objectives of a followed guild (RF-31).
- `pub struct View` — Everything the map tab draws, gathered by the caller (`ui::draw_tabs`).
- `pub fn body(ui: &Ui, view: &View<'_>, role: &mut MapRole, here: Option<MapRole>) -> Option<String>` — Body of the *Map* tab.
- `fn tag_of(label: &str) -> Option<&str>` — `[TAG] Nombre` → `TAG`, which is what fits next to a ring.
- `fn inside(point: [f32; 2], origin: [f32; 2], size: [f32; 2]) -> bool` — Inside the drawing area?
- `fn nearest(spots: &[[f32; 2]], mouse: [f32; 2], max_distance: f32) -> Option<usize>` — Index of the spot closest to `mouse`, if there is one within `max_distance` pixels.
- `fn detail_line(…)` — Una línea con lo que dice el hover, **sin el nombre del sitio**: el jugador ya sabe dónde está y el nombre alargaba la caja hasta tapar los objetivos vecinos (petición del jugador, 2026-11-01).
- `const MAP_COMBO_WIDTH` — Ancho del desplegable de mapa. Antes ocupaba todo el ancho de la ventana para enseñar cuatro palabras («Eternos», «Alpino azul»…) y dejaba sin sitio al de clan (petición del jugador, 2026-11-01).
- `const GUILD_COMBO_WIDTH` — Ancho del desplegable de clan: `[TAG] Nombre` no cabe en menos.
- `fn draw_selector(…)` — Desplegable de mapa (Eternos/alpinos/desierto): omite los que aún no llegaron de la API y resalta en azul el mapa donde está el jugador.
- `fn guild_choices(…)` — Los clanes del desplegable: primero los que sigue el jugador (en el orden en que los añadió) y después los que han reclamado algo **en la sesión**, el más reciente primero, sin repetir los que ya sale…
- `fn draw_guild_choice(…)` — Desplegable de clan, al lado del de mapa. Devuelve el que el jugador acaba de elegir, si hay uno.
- `const GUILDS_IN_MENU` — Cuántos clanes **no seguidos** ofrece el desplegable del mapa.
- `fn label_of(names: &BTreeMap<String, String>, guild: &str) -> String` — `[TAG] Nombre` cuando se conoce, el id si no.
- `fn is_drawable(kind: ObjectiveType) -> bool` — Objectives worth drawing: the ones that score (RF-30a hides ruins, mercenaries and spawns).
- `fn has_high_value_ring(state: Option<&ObjectiveState>) -> bool` — Si el objetivo merece el anillo de alto valor.
- `fn resolved_map_role(role: MapRole, here: Option<MapRole>) -> MapRole` — Mapa a mostrar: el elegido o, si no hay, el del jugador o Eternos.
- `fn map_for_role<'a>(…)` — Prefiere la geometría del `map_id` actual y, si no, el mapa del rol según el estado live o el catálogo clásico.
- `fn has_live_state_for_map(states: &[ObjectiveState], map_id: u32) -> bool` — Indica si el emparejamiento publicó estado live para el mapa mostrado.
- `pub fn map_label_for_map(map: &WvwMap, role: MapRole) -> String` — Etiqueta traducida a partir del nombre de mapa vigente en la API y el rol Mumble.
- `fn draw_objective(…)` — Dibuja un objetivo en el mapa: círculo del color del dueño con tamaño por tipo, anillo de alto valor y marca de clan seguido.
- `fn colour_rgba(colour: TeamColor) -> [f32; 4]` — Los colores de equipo salen del mismo sitio en todo el addon (RF-62).
- `pub fn map_label(role: MapRole) -> &'static str` — El nombre del mapa en el idioma activo: lo comparten la pestaña y la cabecera del widget.
- `mod tests`

**Usa (interno):** `crate::clock`, `crate::i18n`, `gw2lurker_core::i18n`, `gw2lurker_core::matchup`, `gw2lurker_core::model`, `gw2lurker_core::project`, `gw2lurker_core::route`, `super::icons`, `super::navigation`, `super::theme`

**APIs externas:** `nexus::imgui`, `nexus::imgui::DrawListMut`, `imgui::DrawListMut`, `imgui (dibuja con &Ui)`


### `crates/addon/src/ui/map/route.rs` (273 líneas)

Native drawing of the cached B - Estela scene (T-52).

**Contenido:**

- `pub struct RouteCanvas` — Projection box and screen rectangle used by both geometry tests and native drawing.
- `pub fn draw_route(list: &DrawListMut<'_>, scene: &Scene)` — Draws prepared geometry; all expensive grouping/projection happens outside the steady frame.
- `fn draw_stroke(list: &DrawListMut<'_>, stroke: &Stroke, segments: u32)` — Uses the verified arcdps-imgui 0.8 draw_list.rs Bezier builder; fixed tessellation bounds cost.
- `fn dashed_ring(list: &DrawListMut<'_>, point: [f32; 2], radius: f32, colour: [f32; 4])` — Dashed outline distinguishes future and late observations without relying on colour.
- `pub fn caption(ui: &Ui, state: &Session, names: &[ObjectiveInfo])` — Origin explanation and legend remain beside the map, never presented as observed movement.
- `pub fn hover(ui: &Ui, state: &Session, canvas: &RouteCanvas<'_>)` — Hover metadata of route nodes, retaining all revisit numbers without an oversized tooltip.
- `pub fn summary(…)` — Text summary with map transitions, kept compatible with the Guilds navigation tests.
- `pub fn route_steps(…)` — Summarizes the inferred sequence with an explicit jump between maps.
- `fn map_label_in(role: MapRole, language: gw2lurker_core::i18n::Language) -> &'static str` — Map name in a supplied language, so tests never query Nexus for the active language.
- `pub fn heading(from: [f32; 2], to: [f32; 2]) -> Option<(f32, f32)>` — Direction of a non-degenerate connector; no inferred heading from coincident points.

**Usa (interno):** `crate::clock`, `gw2lurker_core::build_route`, `gw2lurker_core::i18n`, `gw2lurker_core::Route`, `super::inside`, `super::scene`, `super::timeline`

**APIs externas:** `nexus::imgui`, `imgui (dibuja con &Ui)`


### `crates/addon/src/ui/map/scene.rs` (276 líneas)

Prepared geometry for B - Estela (T-52), independent of Nexus.

**Contenido:**

- `pub struct Curve` — Quadratic visual connector; not a traversable path.
- `impl Curve`
  - `pub fn between(from: [f32; 2], to: [f32; 2], index: usize) -> Self` — Alternate sides in a canonical direction, so a return leg cannot retrace the same arc.
  - `pub fn point(&self, t: f32) -> [f32; 2]` — Point on the quadratic curve, with t in 0..=1.
  - `pub fn cubic(&self) -> ([f32; 2], [f32; 2])` — Equivalent cubic control points for imgui's verified Bezier API.
- `pub struct Stroke` — A precomputed connector and its style.
- `pub struct Marker` — One marker for every visited objective, including all its original step numbers.
- `pub struct Scene` — Prepared drawing for one map; rebuilt only when data, viewport or selection changes.
- `impl Scene`
  - `pub fn prepare(trail: &Trail<'_>, view: &View, map: u32, canvas: &RouteCanvas<'_>) -> Self` — Projects consecutive observations without bridging missing coordinates or map changes.
  - `pub fn attach_origin<'a>(…)` — Origin is separate from observations and has no invented timestamp.

**Usa (interno):** `gw2lurker_core::Route`, `super::ClaimRow`, `super::project`, `super::route`


### `crates/addon/src/ui/map/tests.rs` (690 líneas)

Tests of the map tab: projection, hover, tiles and proportions.

**Usa (interno):** `crate::clock`, `crate::worker`, `gw2lurker_core::i18n`, `gw2lurker_core::Match`, `gw2lurker_core::matchup`, `gw2lurker_core::project`, `gw2lurker_core::Route`, `gw2lurker_core::RouteOrigin`, `gw2lurker_core::RouteStep`, `super::route`, `super::scene`

**APIs externas:** `serde_json::from_str`

**Tests (19):** `rf32a_the_drawn_route_includes_zero_and_keeps_claim_numbering`, `rf30_the_map_keeps_the_real_proportions`, `p05_seared_borderlands_static_objectives_fit_the_verified_map_box`, `p05_seared_borderlands_map_id_supports_all_three_mumble_roles`, `p05_seared_borderlands_1654_is_offered_for_blue_and_green_roles`, `p05_seared_borderlands_1651_is_detected_as_green_from_api_name`, `rf32_the_heading_is_the_last_jump_and_only_when_it_means_something`, `rf30_the_closest_objective_wins_the_hover`, `rf30_only_the_inside_of_the_board_has_hover`, `rf30a_scoring_objectives_are_the_only_ones_drawn`, `rf30b_missing_objective_state_does_not_get_a_high_value_ring`, `rf30_the_unknown_map_defaults_to_center_before_drawing_the_selector`, `rf30_the_hover_says_who_and_when_and_leaves_the_name_out`, `rf30_the_hover_without_a_claim_only_says_unclaimed`, `rf30b_an_objective_owned_without_claim_keeps_its_colour_on_the_hover`, `rf31_the_guild_menu_is_the_followed_ones_and_then_the_latest_five`, `rf31_the_guild_menu_without_anything_to_offer_is_empty`, `rf35_a_route_that_crosses_maps_says_jump_to`, `rf32_the_cross_map_summary_lists_claims_without_inventing_an_origin`


### `crates/addon/src/ui/map/tiles.rs` (219 líneas)

**Contenido:**

- `const MAP_HEIGHT` — Height of the drawing area when the map has no usable proportions.
- `const MIN_MAP_HEIGHT` — Smallest and biggest height the map may take when it keeps its real proportions.
- `const MAX_MAP_HEIGHT` — Altura máxima del lienzo del mapa (620 px).
- `pub fn map_size(frame: &Frame, avail: [f32; 2]) -> [f32; 2]` — Tamaño en pantalla del mapa, con la **proporción real** de `continent_rect`.
- `pub fn draw_tiles(…)` — The real map: the official tiles, drawn one by one with the opacity of the window (T-29).
- `type TileTextureCache` — Loaded tile texture and the file version it represents.
- `static TILE_TEXTURES` — Texturas ImGui por coordenada y generación de los ficheros en caché.
- `const TILES_PER_FRAME` — How many tiles may be created in one frame. The first time the tab is opened the map fills in over a few frames instead of stalling the game.
- `static TILE_BUDGET` — What is left of that allowance in the current frame. Reset by [`draw_tiles`].
- `fn tile_texture(x: u32, y: u32) -> Option<nexus::imgui::TextureId>` — Texture of one tile, loading it from the disk cache the first time it is needed.
- `const TILE_RETRY` — How long a tile that failed is left alone before trying again.
- `fn failures() -> &'static std::sync::Mutex<BTreeMap<(u32, u32), std::time::Instant>>` — When each tile failed last, so neither the budget nor the log repeats it every frame.
- `fn failed_recently(x: u32, y: u32) -> bool` — Si la carga de esa tesela falló hace menos de `TILE_RETRY` (no reintentar aún).
- `fn remember_failure(x: u32, y: u32) -> bool` — Marks the tile as failed. Returns `true` when it was already failing (so the log says it once).
- `pub fn draw_frame(…)` — Border and name over whatever the background is.
- `pub fn draw_background(…)` — Dark board with a grid and the map name: no textures (ADR-002), but the objectives are not floating on nothing either (report of 2026-10-31). The real map is [`draw_tiles`].

**Usa (interno):** `crate::catalog`, `gw2lurker_core::project`

**APIs externas:** `nexus::imgui::DrawListMut`, `nexus::imgui::TextureId`, `nexus::texture::get_texture_or_create_from_file`, `disco: fs::metadata`, `imgui::DrawListMut`, `imgui::TextureId`


### `crates/addon/src/ui/map/timeline_tests.rs` (128 líneas)

Native timeline state tests without Nexus or a running game.

**Contenido:**

- `struct Data` — History and fixture-backed catalogues shared by the test views.
- `impl Data`
  - `fn new() -> Self` — Normal sequence with unequal intervals and an official map catalogue.
  - `fn refresh(&mut self)` — Emulates preparation on the worker, not inside the UI view.
  - `fn view<'a>(&'a self, guild: Option<&'a str>, revision: i64) -> View<'a>` — Gives the cache a deterministic worker revision.

**Usa (interno):** `crate::ui`, `super::super`

**Tests (3):** `rf36_estela_hidden_map_and_changed_guild_pause_playback`, `rf36_estela_mouse_selection_uses_time_and_keeps_unknown_times_separate`, `rf36_estela_refresh_preserves_the_inspected_claim_and_reuses_geometry`


### `crates/addon/src/ui/map/timeline.rs` (450 líneas)

Session cache and native timeline controls for B - Estela (T-53).

**Contenido:**

- `struct SceneKey` — Inputs that affect the prepared geometry but not its cached source observations.
- `pub struct Session` — Session-only state. No configuration mutex or worker lock is held by this cache.
- `impl Default for Session`
  - `fn default() -> Self` — Starts empty and paused, with no prepared geometry or persistent playback settings.
- `fn session() -> MutexGuard<'static, Session>` — Locks only the map's own session state, never RUNTIME.
- `pub fn pause_unless_drawn(frame: i32)` — Pauses even when the widget is collapsed, closed, outside WvW or not on the Map tab.
- `pub fn prepare<'a>(ui: &Ui, data: &View<'a>) -> MutexGuard<'static, Session>` — Updates the source cache only on a worker revision or guild/cycle change, then draws controls.
- `impl Session`
  - `fn pause_if_hidden(&mut self, frame: i32)` — Visibility rule shared by tab changes, collapsed windows and the WvW render gate.
  - `fn synchronize(&mut self, data: &View<'_>)` — Refreshes history without jumping away from a step the player is inspecting.
  - `pub fn prepare_scene(&mut self, data: &View<'_>, map: u32, canvas: &RouteCanvas<'_>)` — Caches projection and origin inference until data, viewport or selection changes.
- `fn position(trail: &Trail<'_>, index: usize) -> (f32, bool)` — Temporal position, or a separately labelled lane for observations with unknown times.
- `fn pick_at(trail: &Trail<'_>, x: f32, unknown: bool) -> Option<usize>` — Nearest observation in the chosen time lane; ties keep the stable step order.
- `fn controls(ui: &Ui, state: &mut Session)` — Native controls precede the map so their draw list is dropped before drawing the board.
- `fn window_label(window: Window) -> &'static str` — Translation key for each non-persistent trail window.
- `pub fn duration(dt: chrono::TimeDelta) -> String` — Interval formatting, shared by timeline and route tooltips.
- `pub fn track(ui: &Ui, state: &mut Session)` — Real-time axis plus an explicit separate lane for unknown timestamps. Draws the real temporal axis, kept separate from localized controls for native measurements.
- `mod tests`

**Usa (interno):** `crate::clock`, `gw2lurker_core::Route`, `super::map_label`

**APIs externas:** `nexus::imgui`, `imgui (dibuja con &Ui)`


### `crates/addon/src/ui/map/trail_tests.rs` (236 líneas)

Acceptance of the prepared B scene without calling Nexus.

**Contenido:**

- `pub fn fixtures() -> (Vec<ObjectiveInfo>, Vec<WvwMap>)` — Real catalogues used for map projection and objective coordinates.
- `pub fn claim(id: &str, seconds: i64) -> ClaimRow` — Synthetic claim whose coordinates come from the real objective table.

**Usa (interno):** `crate::ui`, `gw2lurker_core::Event`, `gw2lurker_core::route`, `super::scene`

**APIs externas:** `nexus::imgui`, `serde_json::from_str`

**Tests (4):** `rf32_estela_curves_preserve_endpoints_and_show_separate_return_paths`, `rf32_estela_six_cases_keep_claims_maps_and_holes`, `rf33_estela_keeps_late_gap_and_future_styles`, `rnf01_estela_native_draw_budget`


### `crates/addon/src/ui/matchup.rs` (316 líneas)

Matchup claims view (T-36, RF-70): what is claimed right now, grouped by map.

**Contenido:**

- `pub struct Filters` — Filters the player can toggle; session state, like the team narrowing.
- `impl Filters`
  - `pub fn all_kinds() -> Self` — All kinds visible at the start; the boxes exist to *remove* noise, not to hide by default.
  - `pub fn accepts(&self, row: &ClaimRow, follow: &gw2lurker_core::FollowList) -> bool` — Whether a row survives the filters. `follow` is only consulted when the box is ticked.
- `pub fn rows_of<'a>(…)` — Rows of one map, in the order the API gave them.
- `pub fn body(…)` — Body of the *Map* tab. `state` is gone with the window: it lives in the widget now (T-42).
- `fn draw_filters(ui: &Ui, filters: &mut Filters)` — Casillas de filtro de la lista de reclamos: alto valor, campamentos, torres, fortalezas y clanes seguidos.
- `fn draw_row(ui: &Ui, row: &ClaimRow, names: &std::collections::BTreeMap<String, String>)` — Una fila de reclamo: hora, objetivo y clan, teñida del color del dueño.
- `fn map_label(role: MapRole) -> &'static str` — Nombre traducido de un mapa por su rol.

**Usa (interno):** `crate::clock`, `crate::i18n`, `gw2lurker_core::FollowList`, `gw2lurker_core::matchup`, `gw2lurker_core::model`, `super::theme`

**APIs externas:** `nexus::imgui`, `nexus::imgui::CollapsingHeader`, `imgui::CollapsingHeader`, `imgui (dibuja con &Ui)`

**Tests (6):** `rf73_the_claims_view_can_show_only_followed_guilds`, `rf30b_high_value_is_the_tick_of_this_snapshot_not_the_kind`, `rf70_all_kinds_are_visible_by_default`, `rf70_kind_filters_remove_rows`, `rf70_high_value_hides_camps_but_keeps_castles`, `rf70_rows_are_grouped_by_map`


### `crates/addon/src/ui/mod.rs` (795 líneas)

Windows of the addon. Every one of them goes through [`window::draw`] (RF-46).

**Contenido:**

- `mod alerts`
- `mod follow`
- `mod guilds`
- `mod icons`
- `mod map`
- `mod matchup`
- `mod navigation`
- `mod popup`
- `mod popup_stack`
- `mod settings`
- `mod status`
- `mod team_picker`
- `mod theme`
- `mod window`
- `pub const WIDGET` — Translation key (and stable id) of the only window of the addon (T-42).
- `pub const WIDGET_DEFAULT_SIZE` — Size the widget opens with, before the player resizes it.
- `const TABS` — Tabs of the widget, in order. The last one is always the settings (T-42).
- `const LOCK_KEYBIND` — Identifier of the lock keybind. Nexus lists it with this name, so the user can rebind it.
- `pub const LOCK_SHORTCUT` — Shortcut registered by default for bringing the window back.
- `const VK_L` — Virtual-key code of `L`, which is what Nexus' keybind API takes.
- `static LOCK_TOGGLE` — Set by the keybind handler and consumed by the render callback.
- `pub static IMGUI_TEST_LOCK` — ImGui allows only one active context, so native UI tests share this lock.
- `fn on_lock_keybind(_identifier: &str, is_release: bool)` — Callback del atajo de Nexus: al pulsar (no al soltar) pide alternar el click-through.
- `fn take_lock_toggle() -> bool` — Consume la petición del atajo (la lee y la pone a falso).
- `fn reveal(config: &mut crate::config::Config) -> bool` — Brings the window back: shown, not click-through, and not collapsed.
- `pub fn toggle_click_through(config: &mut crate::config::Config) -> bool` — Turns click-through on or off, and returns the new state.
- `pub fn register()` — Registers the render callbacks. Must be called from `load`, never per frame.
- `fn draw_options(ui: &nexus::imgui::Ui)` — Nexus' options pane: the two openers, then the whole form (T-42).
- `fn guard(name: &'static str, draw: impl FnOnce())` — Runs one render callback, turning a panic into a log line instead of a dead game.
- `fn draw_widget(ui: &nexus::imgui::Ui)` — Draws the only window: a tab bar and the body of the selected tab (T-42).
- `pub struct Tabbed` — Everything the tabs draw, gathered once per frame.
- `fn draw_tabs(…)` — Draws the tab bar and the body of the tab the player picked. The last tab is the settings.
- `const TAB_MAP` — Tab index of the *Map* tab inside [`TABS`].
- `fn map_notice() -> std::sync::MutexGuard<'static, Option<String>>` — Clan del aviso «no hay eventos registrados», si hay uno abierto.
- `fn draw_map_notice(ui: &nexus::imgui::Ui, data: &Tabbed)` — Dibuja el aviso de clan sin eventos (con «desde cuándo» y botón para cerrarlo).
- `static LAST_TAB` — Tab drawn on the previous frame, `usize::MAX` for «none». Session state, like the map choice.
- `static FRAME` — Fotograma actual del widget. Sirve para que un clic de clan mande solo en el fotograma siguiente y no deje una marca viva que se coma los saltos posteriores al mapa del jugador.
- `static GUILD_JUMP_AT` — Fotograma del último clic de clan que abrió la pestaña *Mapa*, `u64::MAX` si no hubo.
- `static WANT_TAB` — Tab the widget has to switch to on this frame, `usize::MAX` for «stay where the player is».
- `fn map_choice() -> std::sync::MutexGuard<'static, MapRole>` — Which map the *Map* tab draws: the session's choice, `Unknown` until the player picks one.
- `fn draw_openers(ui: &nexus::imgui::Ui)` — One line with the state of the worker and the configured team (RF-02, RF-46).

**Usa (interno):** `crate::clock`, `crate::config`, `crate::context`, `crate::i18n`, `crate::persist_if_changed`, `crate::request_save`, `crate::runtime`, `crate::safe_mode`, `crate::team`, `crate::worker`, `gw2lurker_core::Candidate`, `gw2lurker_core::FollowList`, `gw2lurker_core::matchup`, `gw2lurker_core::model`, `gw2lurker_core::ObjectiveInfo`, `gw2lurker_core::WvwMap`

**APIs externas:** `nexus::gui`, `nexus::keybind::register_keybind_with_struct`, `nexus::keybind::keybind_handler`, `nexus::keybind::Keybind`, `nexus::imgui::Ui`, `nexus::imgui`, `nexus::imgui::StyleColor`, `nexus::imgui::CollapsingHeader`, `imgui::Ui`, `imgui::StyleColor`, `imgui::CollapsingHeader`, `imgui (dibuja con &Ui)`

**Tests (5):** `rf30_the_map_tab_opens_on_the_map_i_am_on`, `rf18_the_shortcut_turns_click_through_on_and_off`, `rf18_the_shortcut_brings_the_window_back`, `rf46_click_through_does_not_reopen_a_closed_widget`, `t42_the_settings_are_the_last_tab`


### `crates/addon/src/ui/navigation.rs` (275 líneas)

Map navigation rules, independent of Nexus and imgui (RF-30, RF-71).

**Contenido:**

- `pub fn tab_available(key: &str, team: Option<u32>) -> bool` — Data tabs require the current account's team; Settings always stays usable (RF-03a).
- `pub fn pending_tab_after_draw(shown: usize, requested: usize) -> usize` — Mantiene la pestaña pedida hasta que se ha dibujado; entonces la suelta (`usize::MAX`).
- `pub fn entering_by_hand(…)` — ¿Toca saltar al mapa del jugador al dibujar la pestaña *Mapa*?
- `fn guild_choice_state() -> std::sync::MutexGuard<'static, Option<String>>` — Clan cuya ruta dibuja la pestaña *Mapa*, si hay alguno elegido.
- `pub fn guild_choice() -> Option<String>` — Clan cuya ruta dibuja la pestaña *Mapa*, si hay alguno elegido.
- `pub fn set_guild_choice(guild: Option<&str>)` — Fija o vacía el clan de la pestaña *Mapa* (`None` = sin ruta, solo el anillo de los seguidos).
- `pub fn guild_from_pick(pick: &str) -> Option<String>` — Lo elegido en el desplegable de clan → clan a dibujar.
- `pub fn request_guild_map(…)` — Ir al mapa de un clan: con mapa conocido fija mapa y pestaña Mapa; sin él no cambia de pestaña y deja el aviso.

**Usa (interno):** `gw2lurker_core::MapRole`, `super::settings`, `super::TAB_MAP`

**Tests (9):** `rf03a_without_a_team_every_data_tab_requires_the_settings_step`, `rf03a_another_accounts_saved_team_does_not_unlock_the_tabs`, `rf30_a_guild_jump_waits_until_the_map_tab_is_drawn`, `rf30_returning_to_the_map_tab_by_hand_snaps_to_the_players_map`, `rf71_a_guild_without_events_keeps_the_tab_and_map`, `rf71_a_guild_jump_chooses_its_map_and_clears_the_notice`, `rf71_an_empty_guild_cancels_an_earlier_pending_jump`, `rf32a_a_non_favourite_guild_opens_its_last_claim_map_with_a_route`, `rf71_entering_the_map_tab_by_hand_leaves_no_guild_chosen`


### `crates/addon/src/ui/popup_stack.rs` (454 líneas)

**Contenido:**

- `const MAX_POPUP_LINES` — Máximo de avisos visibles simultáneamente en el emergente.
- `pub struct PopupLine` — Aviso visible con su factor de fundido.
- `pub struct PopupSnapshot` — Líneas visibles y número de avisos descartados.
- `struct ActiveLine` — Aviso activo con su vencimiento y prioridad.
- `struct Identity` — Identidad de un evento para no volver a mostrarlo en cada fotograma.
- `impl From<&AlertLine> for Identity`
  - `fn from(line: &AlertLine) -> Self` — Construye la identidad a partir de una línea de aviso.
- `pub struct PopupStack` — Estado de avisos activos, identidades vistas y descartes.
- `impl PopupStack`
  - `pub fn tick(…)` — Caduca, ordena y limita las líneas con un reloj dado, y calcula su fundido.
- `fn priority(…)` — Prioriza clan seguido, alto valor, perdido, tomado y el resto.

**Usa (interno):** `crate::alerts`, `crate::worker`, `gw2lurker_core::FollowList`, `gw2lurker_core::matchup`, `gw2lurker_core::model`

**Tests (6):** `rf19d_alerts_stack_and_expire_independently`, `rf19d_priority_is_followed_then_valuable_then_lost_then_taken`, `rf19d_high_value_lost_replaces_the_oldest_taken_line`, `rf19d_discarded_lines_are_counted_and_never_queued`, `rf19d_text_fades_during_the_last_half_second`, `rf25_r07_only_recent_followed_late_claims_reach_popup`


### `crates/addon/src/ui/popup.rs` (178 líneas)

The transient, stacked pop-up (RF-19d).

**Contenido:**

- `pub static TEST_POPUP_UNTIL` — When the *Test pop-up* button of the settings wants a fake alert on screen (milliseconds).
- `pub fn start_test_popup()` — Shows a fake pop-up for a few seconds, so it can be placed without waiting for a capture.
- `pub fn draw(…)` — Pop-up with what just happened, on top of the screen (RF-19a).
- `fn popup_lines(snapshot: PopupSnapshot, test: Option<crate::worker::AlertLine>) -> PopupSnapshot` — Muestra la línea de prueba en lugar de la pila real mientras se coloca el emergente.
- `fn queue_slot() -> std::sync::MutexGuard<'static, PopupStack>` — The pop-up stack is session state, not a setting.
- `fn test_popup_line(now: chrono::DateTime<chrono::Utc>) -> Option<crate::worker::AlertLine>` — A fake line while the settings ask for one, so the pop-up can be placed at will.

**Usa (interno):** `crate::i18n`, `crate::runtime`, `crate::team`, `crate::worker`, `gw2lurker_core::model`, `gw2lurker_core::ObjectiveType`, `super::alerts`, `super::popup_stack`, `super::WIDGET`

**APIs externas:** `nexus::imgui::Ui`, `imgui::Ui`, `imgui (dibuja con &Ui)`

**Tests (1):** `rf19a_a_test_popup_is_shown_without_a_real_alert`


### `crates/addon/src/ui/settings.rs` (466 líneas)

Settings window (T-17, T-30): language, polling interval, alert scope and the button that opens the team picker, which is a window of its own since 2026-11-02 (`team_picker`, RF-03j).

**Contenido:**

- `pub const KEY` — Translation key (and stable id) of this window.
- `struct Edits` — Working copy of everything this window can edit.
- `pub fn body(ui: &Ui, config: &mut crate::config::Config)` — Body of the *Settings* tab: the whole form, without window chrome (T-42).
- `pub fn draw_inline(ui: &Ui)` — Draws the settings inline, without window chrome.
- `const POPUP_MAX_SECS` — Everything the label of a control can say, so the control never eats the label.
- `const POPUP_DEFAULT_SECS` — Lo que dura cada aviso del pop-up cuando se activa (segundos). El mismo defecto que en `config`, para que encenderlo desde aquí no ponga una duración distinta a la de fábrica.
- `const LABELS` — Etiquetas de los controles de Ajustes; con ellas se mide la columna de etiquetas en el idioma activo.
- `fn label_column(ui: &Ui) -> f32` — Width of the widest label of the form, in the current language, plus a margin.
- `fn draw_body(ui: &Ui, edits: &mut Edits, team: Option<&str>)` — Cuerpo de Ajustes: región, color, equipo, traducción, idioma, sondeo, avisos, popup, ventana y mapa.
- `fn draw_theme(ui: &Ui, edits: &mut Edits)` — Estilo y tamaño del texto de las ventanas, y qué se enseña en el widget (RF-19c, RF-60 a RF-63).
- `fn draw_alert_filters(ui: &Ui, edits: &mut Edits)` — Which maps and guilds deserve an alert (RF-19a), plus how long the pop-up lasts.
- `fn draw_guild_index(ui: &Ui, edits: &mut Edits)` — Índice de nombres de clan: estado y descarga (RF-20).
- `fn initial_translation(…)` — The translation the field starts with: what the player saved, or the automatic name.
- `fn remember_translation(config: &mut crate::config::Config, written: &str, language: &str)` — Stores what the player left in the field. Writing the automatic name back is not an override: the configuration drops the entry so the bundled translations keep applying.
- `fn draw_language(ui: &Ui, edits: &mut Edits)` — Desplegable de idioma (automático, español, inglés, francés, alemán).
- `fn draw_scope(ui: &Ui, edits: &mut Edits)` — Desplegable de alcance de avisos: solo el mapa actual o todos los del emparejamiento.
- `mod tests`

**Usa (interno):** `crate::config`, `crate::guild_index`, `crate::i18n`, `crate::keybinds`, `crate::names`, `crate::runtime`, `crate::team`, `crate::ui`, `gw2lurker_core::bundled`, `super::team_picker`

**APIs externas:** `nexus::imgui`, `imgui (dibuja con &Ui)`


### `crates/addon/src/ui/settings/tests.rs` (33 líneas)

Tests of the settings window: the translated names (RF-03f).

**Usa (interno):** `crate::config`, `gw2lurker_core::bundled`

**Tests (1):** `rf03f_the_field_starts_with_the_automatic_translation`


### `crates/addon/src/ui/status.rs` (241 líneas)

La línea de estado de la ventana: salud del sondeo, partida, equipo y origen del empate.

**Contenido:**

- `const FROZEN_AFTER_MIN` — Minutos con la copia de la partida idéntica (puntos incluidos, `matchup::digest`) a partir de los cuales se avisa (RF-43). En MvM el marcador cambia cada pocos segundos; 5 min sin ningún cambio es la…
- `pub fn draw_health(…)` — Bloque de estado: salud del sondeo, mapa del jugador, equipo (del juego o configurado), región y equipos del match, última actualización.
- `pub fn colour_label(colour: TeamColor) -> &'static str` — Nombre del color del equipo, el mismo que usa el panel de WvW del juego.
- `pub fn region_of(match_id: &str) -> Option<gw2lurker_core::model::Region>` — Region of a matchup id: `1-x` is North America and `2-x` is Europe.
- `fn region_of_matchup(match_id: &str) -> &'static str` — «NA»/«EU»/«?» según el prefijo del id del match.

**Usa (interno):** `crate::clock`, `crate::config`, `crate::context`, `crate::i18n`, `crate::keybinds`, `crate::loaded_version`, `crate::runtime`, `crate::stale_dll`, `crate::team`, `crate::worker`, `gw2lurker_core::gate`, `gw2lurker_core::model`, `gw2lurker_core::select`

**APIs externas:** `nexus::imgui::Ui`, `nexus::imgui::StyleColor`, `imgui::Ui`, `imgui::StyleColor`, `imgui (dibuja con &Ui)`


### `crates/addon/src/ui/team_picker.rs` (536 líneas)

The team picker, a window of its own (RF-03a, RF-03j).

**Contenido:**

- `pub const KEY` — Translation key (and stable id) of the window.
- `const WIDTH` — Width the window opens with; it grows with its content.
- `const GRACE` — Wait after entering a map before deciding: arcdps says which **account** we are in a moment after the map loads, and until then the team is looked up by character and is not found, which would open th…
- `pub enum TeamChange` — A change to the team selection, decided while drawing and applied once the lock is released.
- `struct Narrowing` — Which region and colour the player has already narrowed down to.
- `struct Picker` — Whether the window is open and why it opened.
- `struct Entry` — The map the player entered last, and whether that entry was already checked.
- `impl Entry`
  - `fn due(&mut self, map_id: u32, in_wvw: bool, now: Instant) -> bool` — Whether this entry still has to be checked. A new map re-arms it; only maps where the window may open (WvW and Armistice Bastion) count, and only after [`GRACE`].
- `enum Step` — What to do with one entry once its grace is over.
- `fn step(…)` — Decide qué hacer al entrar en un mapa: abrir el selector, apuntar el emparejamiento sin preguntar, esperar datos o nada.
- `static PICKER` — Bloquea `PICKER` recuperándose de un mutex envenenado.
- `static ENTRY` — Último mapa en que se entró y si esa entrada ya se comprobó (RF-03j).
- `fn picker() -> std::sync::MutexGuard<'static, Picker>` — Bloquea `PICKER` recuperándose de un mutex envenenado.
- `pub fn open(reason: Option<Prompt>)` — Opens the window. `reason` is `None` when the player asked for it (the *Settings* button).
- `fn auto_open(gate: Gate)` — Opens the window by itself when entering WvW or Armistice Bastion has something to ask.
- `pub fn draw(ui: &Ui, gate: Gate)` — Draws the window when it is open, after deciding whether it has to open by itself.
- `fn draw_reason(ui: &Ui, reason: Option<Prompt>)` — Why the window opened by itself, in the colour of a warning.
- `fn team_change(before: Option<u32>, now: Option<u32>) -> Option<TeamChange>` — Qué cambió en el equipo elegido: quitarlo, elegir otro o nada.
- `fn initial_narrowing(…)` — Where the narrowing starts: the saved team's own region and colour when we know them, and otherwise the region of the first candidate, so the picker is never a blank wall.
- `fn draw_region(ui: &Ui, narrowing: &mut Narrowing, candidates: &[Candidate])` — Desplegable de región (Europa/Norteamérica); ambas siempre elegibles aunque aún no haya equipos.
- `fn draw_colour(ui: &Ui, narrowing: &mut Narrowing)` — Desplegable de color de equipo, cada opción teñida de su color.
- `fn draw_team(…)` — Desplegable de equipo filtrado por región y color, con mundos y traducción propia del jugador.
- `fn draw_rivals(…)` — The other two teams of the chosen team's matchup, to check it is the right one (RF-03g).
- `fn worlds_label(ids: &[u32], worlds: &std::collections::BTreeMap<u32, String>) -> String` — ` · Lutgardis, Vabbi` con los mundos antiguos del equipo, o vacío si no se saben.
- `fn region_label(region: Region) -> &'static str` — Nombre traducido de una región.
- `fn region_code(region: Region) -> &'static str` — Código corto de región («eu»/«na») para ids de widget.
- `fn colour_rgba(colour: TeamColor) -> [f32; 4]` — Same colours as the in-game WvW panel, so the list is recognisable at a glance.
- `mod tests`

**Usa (interno):** `crate::config`, `crate::context`, `crate::i18n`, `crate::runtime`, `crate::team`, `crate::worker`, `gw2lurker_core::bundled`, `gw2lurker_core::model`, `gw2lurker_core::prompt`, `super::status`, `super::theme`

**APIs externas:** `nexus::imgui`, `nexus::imgui::ComboBoxHeight`, `imgui::ComboBoxHeight`, `imgui (dibuja con &Ui)`


### `crates/addon/src/ui/team_picker/tests.rs` (179 líneas)

Tests of the team picker: the narrowing and when it opens by itself (RF-03a, RF-03j).

**Contenido:**

- `fn candidate(team_id: u32, match_id: &str, color: TeamColor) -> Candidate` — Crea un `Candidate` de prueba con equipo, match y color.

**Tests (8):** `rf03a_narrowing_starts_at_the_saved_team`, `rf03a_both_regions_are_always_offered`, `rf03a_the_colour_is_offered_before_any_data`, `rf03a_narrowing_is_remembered_between_frames`, `rf03a_three_steps_bring_the_list_down_to_a_handful`, `rf03a_picking_and_clearing_are_told_apart`, `rf03j_each_map_entry_is_checked_once_after_the_grace`, `rf03j_an_entry_waits_for_data_and_records_the_first_matchup_silently`


### `crates/addon/src/ui/theme.rs` (119 líneas)

Estilo de las ventanas del addon (T-28, RF-60 a RF-63).

**Contenido:**

- `const OWN_COLORS` — Colores del tema propio: gris azulado oscuro, texto claro y acento azul.
- `const OWN_VARS` — Estilos del tema propio: esquinas suaves y algo de aire. Nada global.
- `pub struct StyleGuard` — Pila de estilo viva mientras se dibuja una ventana del addon.
- `pub fn apply<'ui>(ui: &'ui Ui, theme: Theme) -> StyleGuard<'ui>` — Aplica el tema pedido a lo que se dibuje dentro de la ventana actual (RF-60, RF-61).
- `pub fn team_colour(colour: TeamColor) -> [f32; 4]` — Color de cada equipo, **el mismo en el widget, en el mapa y en el historial** (RF-62).

**Usa (interno):** `crate::config`, `gw2lurker_core::model`

**APIs externas:** `nexus::imgui`

**Tests (2):** `rf62_the_three_teams_have_distinct_colours`, `rf61_every_style_colour_is_named_once`


### `crates/addon/src/ui/window.rs` (390 líneas)

Shared window contract (RF-46).

**Contenido:**

- `const MIN_SIZE` — Smallest window the user can shrink.
- `const MIN_BODY_HEIGHT` — Smallest body region, so a window shrunk to nothing still has somewhere to draw.
- `const CLOSE_WIDTH` — Width reserved for the close button.
- `pub struct WindowAppearance` — Aspecto compartido de las ventanas: tema, escala de fuente y fondo.
- `fn default_position(slot: usize) -> [f32; 2]` — Posición inicial escalonada por ventana para que no se solapen.
- `pub fn draw(…)` — Draws one window and returns `false` when the user closed it this frame.
- `fn clamp_size(size: [f32; 2]) -> [f32; 2]` — Limita el tamaño de ventana entre `MIN_SIZE` y 4096 px.
- `fn bg_alpha(opacity: f32, background: crate::config::WidgetBackground) -> f32` — Opacidad del fondo de la ventana según el ajuste (RF-63).
- `pub fn covers(position: Option<[f32; 2]>, size: Option<[f32; 2]>, point: [f32; 2]) -> bool` — Whether a point falls inside a window drawn at `position` with that `size`.
- `pub fn frame(…)` — Rectangle a window occupies, resolved the same way [`draw`] resolves it.
- `pub fn right_click_on(key: &str, default_size: [f32; 2], ui: &Ui) -> bool` — Whether the right mouse button was just pressed on top of the window `key`.
- `pub fn right_click_undoes_click_through(click_through: bool, visible: bool, inside: bool) -> bool` — What the right click on a click-through window has to do. Pure, so the rule can be tested.
- `pub fn chrome_wins<T: PartialEq>(before: T, after: T) -> bool` — Whether the window chrome is the one that changed a field this frame.
- `pub fn opacity_percent(opacity: f32) -> f32` — Opacity as whole percent, which is how the user thinks about it (0–100).
- `pub fn opacity_from_percent(percent: f32) -> f32` — Back to the 0.0–1.0 stored in the configuration.

**Usa (interno):** `crate::config`, `crate::i18n`, `crate::runtime`, `super::theme`

**APIs externas:** `nexus::imgui`, `imgui (dibuja con &Ui)`

**Tests (6):** `rf18_the_right_click_is_the_way_back_from_click_through`, `rf46_the_chrome_only_writes_the_fields_nobody_else_touched`, `rf46_opacity_is_shown_as_percent`, `rf46_window_background_uses_the_current_setting`, `rf46_percent_round_trip_keeps_the_stored_value`, `rf18_a_right_click_on_the_widget_is_noticed_even_with_click_through`


### `crates/addon/src/worker.rs` (791 líneas)

Polling worker (T-11c, RF-20, RNF-04, RNF-05).

**Contenido:**

- `mod routes`
- `pub const MAX_HISTORY` — Claims kept in memory for the current tracking cycle (RF-72).
- `const MAX_BACKOFF` — Cap used by both the shared exponential backoff and the configured polling wait.
- `static MAY_POLL` — Whether the worker may use the network (set from the render callback's gate).
- `static NEEDS_TEAMS` — When the UI last asked for the list of live teams (the picker or the guild search).
- `const TEAMS_WANTED_FOR` — How long an ask for the team list is honoured, in seconds.
- `const ALL_MATCHUPS_TTL` — How often the nine matchups are fetched while the picker is on screen (~400 KB each time).
- `static LAST_ALL` — When the nine matchups were fetched last.
- `static STOP` — Set on unload so the thread leaves its loop.
- `pub enum Health` — Health of the polling, shown in the UI (RF-26).
- `pub struct Dashboard` — What the UI reads. Small on purpose: the render callback must never touch the raw match.
- `impl Dashboard`
  - `pub fn reset_tracking_cycle(&mut self, at: DateTime<Utc>)` — Empieza un ciclo de seguimiento nuevo: vacía match, reclamos, estados, clanes, historial y avisos, y apunta desde cuándo.
  - `pub fn guild_history(&self, guild: &str) -> Vec<ClaimRow>` — Claims of one guild during the current tracking cycle, in the order they happened (RF-72).
  - `pub fn last_map_of(&self, guild: &str) -> Option<MapRole>` — Latest current claim's map, or the last historical one when nothing is held (RF-71).
  - `pub fn session_guilds(&self) -> Vec<(String, usize, Option<DateTime<Utc>>)>` — Guilds that hold something right now: the ones that claimed during this tracking cycle plus the ones already holding an objective at its baseline (ADR-012).
  - `pub fn objective_name(&self, id: &str) -> Option<&str>` — Human name of an objective, for the alert lines (T-13).
  - `pub fn objective_kind(&self, id: &str) -> gw2lurker_core::ObjectiveType` — Kind of an objective, for the drawing of the alert lines (T-47).
- `static DASHBOARD` — El `Dashboard` global que escribe el worker y lee el render.
- `pub fn dashboard() -> MutexGuard<'static, Dashboard>` — Reads the dashboard. Hold the guard only while formatting what is needed.
- `pub fn set_may_poll(allowed: bool)` — Tells the worker whether it may poll this frame. Called from the render callback.
- `pub fn set_teams_needed(needed: bool)` — Tells the worker the team picker is on screen, so it fetches the list of live teams even when polling is not allowed (the settings window also opens outside WvW).
- `fn teams_needed() -> bool` — Whether the UI asked for the team list a moment ago.
- `pub fn start()` — Starts the worker. Idempotent: a second call does nothing.
- `pub fn stop()` — Asks the worker to stop and waits for it to leave.
- `fn run()` — Bucle del hilo worker: fuera de WvW solo refresca equipos y nombres; en WvW sondea, aplica backoff ante errores y duerme el intervalo.
- `const TEAMS_TTL` — How long the team list fetched outside WvW stays valid. Teams only change at reset.
- `fn teams_stale(last: &mut Option<Instant>) -> bool` — Si la lista de equipos ya caducó (`TEAMS_TTL`) o nunca se pidió.
- `const NAMES_TTL` — How long the guild names resolved outside WvW stay fresh (names do not change).
- `fn names_stale(last: &mut Option<Instant>) -> bool` — Si los nombres de clanes seguidos ya caducaron (`NAMES_TTL`) o nunca se pidieron.
- `fn publish_teams(live: &[Match], selected: Option<u32>)` — Fills the dashboard with the teams of the live matches, and nothing else.
- `fn started_degraded(health: &Health) -> DateTime<Utc>` — Desde cuándo está degradado: conserva la hora si ya lo estaba; si no, ahora.
- `fn retry_delay(backoff: &mut Backoff, error: &ApiError) -> Duration` — Espera tras un error: backoff si es reintentable; si no, lo reinicia y no espera.
- `fn sleep(total: Duration)` — Sleeps in slices so `stop()` and a gate change are noticed quickly.
- `fn poll_once(…)` — Un sondeo: comprueba la reasignación, decide qué partidas pedir, detecta el equipo, hace diff y vuelca reclamos, historial y avisos al Dashboard.
- `fn needs_all_now(picker: bool, detect_due: bool, all_stale: bool) -> bool` — Whether this poll has to ask for the nine matchups (~400 KB) or just for one match (~44 KB).
- `fn should_skip_poll(…)` — Si saltar este sondeo: sin match conocido solo se busca con `ids=all` cuando ha pasado su enfriamiento.
- `fn all_stale() -> bool` — Whether the nine matchups were fetched longer than [`ALL_MATCHUPS_TTL`] ago.
- `fn identity_changed() -> bool` — Whether the identity changed since the last poll, which re-arms the one sweep of the nine matchups: the team is remembered per account (RF-03a), so changing character inside the same account does not…
- `fn remember_all()` — Remembers that this poll fetched the nine matchups.
- `fn polled_team(saved: Option<u32>, detected: Option<u32>) -> Option<u32>` — Team whose match the poll asks for: the one the game gave, or the saved one as before.
- `enum Fetch` — What one poll has to ask the API for.
- `fn fetch_plan(needs_all: bool, known: Option<&str>, polled: Option<u32>) -> Fetch` — Which request a poll needs. Pure, so the rule can be tested without the network.
- `fn watched_match_id() -> Option<String>` — The match the poll is watching, from the last one published (cheap: one clone of an id).
- `fn fetch_live(…)` — Runs the plan. Every fallback ends in `ids=all`: it is the only request that can find a match whose id or team we no longer know.
- `fn fetch_by_team(api: &Api<WinHttpTransport>, polled: Option<u32>) -> Result<Vec<Match>, ApiError>` — The match of a team, or every matchup when that team no longer plays (the ladder reshuffled, so the picker has to offer the live teams again).
- `fn matching_team_or_all(live: Vec<Match>, polled: Option<u32>) -> Vec<Match>` — De todas las partidas, solo la del equipo sondeado si aparece; si no, todas.
- `fn history_row(…)` — Turns a claim event into a history row, resolving name, kind and owner from the snapshot.
- `fn current_match<'a>(live: &'a [Match], id: Option<&str>) -> Option<&'a Match>` — La partida con ese id dentro de las recibidas.
- `fn selected_match_id(selection: &Selection) -> Option<&str>` — Id del match si la selección de equipo está confirmada.
- `mod tests`

**Usa (interno):** `crate::alerts`, `crate::assignment`, `crate::background`, `crate::catalog`, `crate::check_dll_update`, `crate::config`, `crate::detect`, `crate::diagnostics`, `crate::keybinds`, `crate::net`, `crate::runtime`, `crate::session_names`, `crate::team`, `gw2lurker_core::Candidate`, `gw2lurker_core::diff`, `gw2lurker_core::DiffConfig`, `gw2lurker_core::matchup`, `gw2lurker_core::model`, `gw2lurker_core::ObjectiveType`, `gw2lurker_core::route`, `gw2lurker_core::select`, `gw2lurker_core::WvwMap`

**APIs externas:** `hilos: thread::sleep`


### `crates/addon/src/worker/routes.rs` (50 líneas)

Prepares immutable Estela models on the polling thread, never in the render callback.

**Contenido:**

- `pub struct Prepared` — Atomically published history, annotations and derived per-guild models.
  - `pub mod els`
- `pub fn prepare(…)` — Merges a successful poll with bounded history and includes still-current baseline claims.

**Usa (interno):** `crate::alerts`, `gw2lurker_core::route`


### `crates/addon/src/worker/tests.rs` (628 líneas)

Tests of the polling worker.

**Contenido:**

- `fn any_map() -> gw2lurker_core::AlertFilter` — Filtro de «todos los mapas», para los tests que no van de filtros.

**Usa (interno):** `crate::alerts`, `crate::assignment`, `gw2lurker_core::AlertFilter`, `gw2lurker_core::BaselineReason`, `gw2lurker_core::diff`, `gw2lurker_core::DiffConfig`, `gw2lurker_core::FollowList`, `gw2lurker_core::ObjectiveType`, `gw2lurker_core::TeamColor`

**APIs externas:** `serde_json::from_str`

**Tests (21):** `rf03a_the_picker_asks_for_every_matchup_but_not_on_every_poll`, `rf03a_a_poll_without_a_known_team_or_match_does_not_fetch_all`, `rnf02_only_the_watched_match_is_polled_at_the_configured_cadence`, `rf03a_the_poll_asks_for_the_team_the_game_gave`, `rf03a_only_a_confirmed_selection_has_a_match_to_watch`, `rf03a_a_team_lookup_reuses_the_full_response_when_the_team_is_missing`, `rf24_assignment_reset_clears_history_and_alerts_and_sets_new_boundary`, `rf03d_confirmed_assignment_change_resets_the_differ_and_tracking_state`, `rf03d_expired_timer_does_not_reset_tracking_while_match_is_active`, `rf25_a_restart_does_not_show_yesterday`, `rf21_a_followed_guild_is_told_when_the_claim_is_fresh`, `rf25_late_claim_grace_uses_when_the_addon_received_it`, `rf25_a_restart_does_not_refill_the_log_with_yesterdays_claims`, `t02_the_poll_asks_for_the_match_it_already_knows`, `rf25_the_history_starts_with_what_is_already_claimed`, `t51_clicking_a_guild_knows_its_last_map`, `rf19_alerts_are_bounded_and_skip_baselines`, `rf33_estela_retains_poll_gaps_without_turning_them_into_alerts`, `rf33_estela_worker_prepares_immutable_models_and_keeps_current_baselines`, `rf38_an_alert_says_which_kind_of_objective_it_is_about`, `rnf04_worker_uses_shared_api_backoff_only_for_retryable_errors`


### `crates/addon/src/writer.rs` (106 líneas)

Off-thread writer for the settings file.

**Contenido:**

- `pub struct Writer` — Handle to the writer thread.
- `impl Writer`
  - `pub fn start() -> io::Result<Self>` — Starts the thread. Its only job is to sit on `recv` until something changes.
  - `pub fn request(&self, config: &Config) -> bool` — Queues a save without blocking the caller.
  - `pub fn finish(self, last: &Config)` — Writes the last value, then stops the thread.
- `impl std::fmt::Debug for Writer`
- `fn run(rx: &Receiver<Config>)` — Bucle del hilo writer con el guardado real de settings.json.
- `fn run_with(rx: &Receiver<Config>, mut save: impl FnMut(&Config) -> io::Result<()>)` — Recibe configuraciones, se queda solo con la última pendiente (coalescencia) y la guarda; los fallos van al log.

**Usa (interno):** `crate::config`

**APIs externas:** `hilos: thread::Builder`

**Tests (1):** `rf41_writer_keeps_the_last_value`


## Scripts, specs, docs y configuración

- `.editorconfig`
- `.gitattributes`
- `.github/workflows/ci.yml` — El crate `addon` depende del crate `windows` y no compila fuera de Windows; su lint y sus tests se ejecutan en el job de Windows.
- `.gitignore`
- `AGENTS.md` — AGENTS.md – Instrucciones para agentes de IA — Addon de Nexus (Guild Wars 2) en Rust para Mundo contra Mundo: avisa de capturas, sigue guilds enemigas y dibuja su historial. **Spec-driven**: la spec manda, el código la obedece. El contexto cuesta dinero: **lee seccio…
- `bugs.md` — Bugs — Bandeja de entrada de fallos. **No se borra**: cada bug se queda con su marca. `[x]` cuando creo que está arreglado, con la versión en la que sale el arreglo, para que el jugador pueda comprobarlo y tacharlo del todo si…
- `Cargo.lock`
- `Cargo.toml`
- `CHANGELOG.md` — Changelog — Formato [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/), versionado [SemVer](https://semver.org/lang/es/). las demás teselas caducan a las 24 horas para adaptarse a cambios de mapa. objetivos estáticos se mue…
- `CLAUDE.md` —  — Lee y sigue `AGENTS.md` (instrucciones completas para agentes de IA de este repositorio).
- `deny.toml` — RUSTSEC-2024-0384 (instant) y RUSTSEC-2024-0436 (paste): avisos de "sin mantenimiento" en crates que arrastra `nexus-rs`. No son vulnerabilidades, están fuera de nuestro control y no podemos actualizarlos por separado: s…
- `docs/agents/lecciones.md` — Lecciones con contexto (referencia de AGENTS.md) — Texto completo de las reglas que en `AGENTS.md` van en una línea. Cada punto sale de un fallo real. Léelo cuando la tarea toque tests, imgui, el candado `RUNTIME` o `settings.json`; no hace falta cargarlo en cada sesión.
- `docs/boceto.html`
- `docs/bocetos/ruta/app.js`
- `docs/bocetos/ruta/boceto-a-caminos.html`
- `docs/bocetos/ruta/boceto-b-estela.html`
- `docs/bocetos/ruta/boceto-c-escuadras.html`
- `docs/bocetos/ruta/boceto-d-incertidumbre.html`
- `docs/bocetos/ruta/comun.js`
- `docs/bocetos/ruta/datos-caminos.js`
- `docs/bocetos/ruta/datos-objetivos.js`
- `docs/bocetos/ruta/escena.js`
- `docs/bocetos/ruta/escenarios.js`
- `docs/bocetos/ruta/logica.js`
- `docs/bocetos/ruta/propuesta-a.js`
- `docs/bocetos/ruta/propuesta-b.js`
- `docs/bocetos/ruta/propuesta-c.js`
- `docs/bocetos/ruta/propuesta-d.js`
- `docs/bocetos/ruta/ruta.css`
- `docs/bocetos/ux/propuestas.html`
- `docs/RELEASE.md` — Publicación de la DLL para Nexus — El addon se distribuye desde **un repositorio compartido con los demás addons**: Regla de oro: **este repositorio no es nuestro**. Solo se tocan los ficheros de dejaba dos instancias del addon con el juego abierto y reve…
- `install.bat` — Copia gw2_lurker.dll a la carpeta addons de Guild Wars 2 con doble clic. Argumentos opcionales: install.bat -Build (tests + compilacion antes de copiar) install.bat -DryRun (solo dice que haria)
- `install.ps1` — Compila (opcional) e instala el addon gw2_lurker.dll en la carpeta addons de Guild Wars 2.
- `LICENSE`
- `modelo.md` — Modelo del código: dónde tocar cuando algo falla — Mapa del repositorio para localizar el código sin leerlo entero. **Uso:** busca con `grep` el síntoma en la tabla de abajo o el nombre del fichero o de la función (`### \`crates/...` abre la ficha de cada fichero) y abre…
- `PRODUCT.md` — PRODUCT.md — Contexto de producto para las herramientas de diseño (`impeccable`). Escrito a partir del brief del usuario y de la spec; **las suposiciones están marcadas**. **gw2-lurker**: addon para Guild Wars 2 (WvW) que avisa de ca…
- `PROGRESO.md` — Progreso: gw2-lurker — Bitácora para continuar el trabajo, incluso si quien lo retoma no es la misma IA ni la misma sesión. **Lo más reciente va arriba.** Cada entrada dice qué se pretendía, qué se hizo, cómo se verificó y qué quedó pendiente.
- `README.md` — gw2-lurker — Releases: https://github.com/cbart-cmd/gw2addonsreleases/releases Known issues: GW2 matchup APIs do not work 50% of the time. Addon de [Raidcore Nexus](https://raidcore.gg/nexus) para Guild Wars 2, escrito en Rust. Solo
- `rust-toolchain.toml` — Se usa "stable" hasta que el spike T-01 compile con éxito. Entonces: fija aquí la versión exacta que funcionó (p. ej. channel = "1.xx.0") y copia ese número en `rust-version` del workspace, para builds reproducibles.
- `rustfmt.toml`
- `scripts/bump-version.ps1` — Sube la versión a la vez en Cargo.toml, las dependencias internas de los crates, Cargo.lock y CHANGELOG.md.
- `scripts/capture_fixtures.sh` — Guarda respuestas reales de la API como fixtures de test. Uso: ./scripts/capture_fixtures.sh <WORLD_ID> ["Nombre de guild a buscar"] Rectángulos de los mapas de WvW (ids tomados del propio match; ADR-011)
- `scripts/check_bugs.ps1` — Cheap check of the shared bug inbox (AGENTS.md §7).
- `scripts/check_loc.sh` — Falla si algún .rs supera MAX líneas; avisa a partir de WARN.
- `scripts/check_size.sh` — Uso: check_size.sh ruta/a/addon.dll (LIMIT_BYTES por defecto 2.5 MB) Objetivo 2.2 MB desde ADR-013 (2026-10-31): el addon no cabía en 1.5 MB con las vistas.
- `scripts/check_traceability.sh` — Trazabilidad requisito -> prueba (T-03b).
- `scripts/check_version.sh` — Falla si la versión del workspace, las dependencias internas de los crates y Cargo.lock no coinciden. Lo llama scripts/validate.ps1 como paso `version` (AGENTS.md §3).
- `scripts/gen_modelo.ps1` — Regenera la parte automática de modelo.md: mapa de ficheros, items, tests y APIs que usa cada uno.
- `scripts/guild-search.ps1` — Buscador de clanes fuera del juego (T-44, RF-73).
- `scripts/modelo-descripciones.txt` — Descripciones para modelo.md de los items que no tienen `///` en el código. Formato: ruta|Tipo::item => descripción (Tipo:: solo para métodos de un impl/trait). `pwsh -File scripts/gen_modelo.ps1 -ListMissing` lista lo q…
- `scripts/pack-guild-index.ps1` — Empaqueta el indice de nombres de clan y lo publica en el repositorio de distribucion (RF-20c).
- `scripts/poll_match.ps1` — Spike T-02 (Windows / PowerShell): mide la frecuencia real de cambio de /v2/wvw/matches. Uso: .\scripts\poll_match.ps1 -World 1234 [-IntervalSec 10] [-DurationMin 30]
- `scripts/poll_match.sh` — Spike T-02: mide con qué frecuencia cambia realmente /v2/wvw/matches y qué caché declara. Uso: ./scripts/poll_match.sh <WORLD_ID> [intervalo_s=10] [duracion_min=30]
- `scripts/release.ps1` — Publica gw2_lurker.dll en el repositorio de distribucion compartido (T-34, RF-53).
- `scripts/update_wvw_teams.ps1` — Updates data/wvw-teams.json: the team id -> name table the addon uses to label WvW teams. The official GW2 API exposes team ids (11xxx/12xxx in `all_worlds` and /v2/wvw/guilds/:region) but not their names, so names are c…
- `scripts/validate.ps1` — Runs the local quality gate and prints one line per step (AGENTS.md §3).
- `specs/00-constitution.md` — Constitución del proyecto: WvW Watch (nombre provisional) — Addon de Nexus para Guild Wars 2, escrito en Rust sobre `zerthox/nexus-rs`, activo solo en Mundo contra Mundo (WvW/MCM). Estos principios son **no negociables**. Cualquier requisito, diseño o tarea que los contradiga deb…
- `specs/01-requirements.md` — 01 – Requisitos — Formato EARS + criterios de aceptación Given/When/Then (GWT). Prioridad: **M** = must (v1), **S** = should, **C** = could. Comprobado el **2026-10-31** contra la API real, el código fuente de `nexus-rs` y una medición pr…
- `specs/02-design.md` — 02 – Diseño — gw2-lurker/ ← raíz del repo (el nombre del remoto: github.com/cbart-cmd/gw2-lurker) ├── Cargo.toml # workspace + [workspace.lints] + [profile.release] ├── Cargo.lock # versionado
- `specs/03-tasks.md` — 03 – Tareas — Cada tarea es pequeña y verificable. Orden recomendado; las dependencias se indican entre paréntesis. Lo que **queda vivo**, con por qué: Pruebas previas en juego aceptadas por indicación del jugador el 2026-11-02, sin a…
- `specs/04-engineering-standards.md` — 04 – Estándares de ingeniería — Cumplimiento verificado por CI siempre que sea posible. Lo que no se pueda automatizar se revisa en PR con la checklist del final. tests se los saltan con `#![cfg_attr(test, allow(...))]`), `missing_debug_implementations…
- `specs/05-wireframes.md` — 05 – Bocetos de interfaz (wireframes) — Boceto de **estructura**, no de estilo visual, y actualizado al código de la 0.25.x. En el juego los iconos se dibujan con primitivas (`ui::icons`), no con estos caracteres, y **todo el texto va en ASCII + latino-1**: la…
- `specs/06-diff-scenarios.md` — 06 – Catálogo de escenarios (diff, filtros de aviso y mapa) — Contrato de comportamiento. **Ojo con la nomenclatura:** los tests se llaman por el **requisito** (`rf10_owner_change_emits_a_flip_with_the_real_time`), no por el ID `d07` que se usó al planificar; la columna «Test» de c…
- `specs/07-spikes.md` — 07 – Spikes: guía y resultados — Un spike es un experimento corto y desechable para convertir una suposición en un hecho. **Los ejecuta una persona** (necesitan el juego o la red real). La IA solo debe confiar en lo que aparezca en la sección *Resultado…
- `specs/ADR-001-estado-bajo-ataque.md` — ADR-001 – El addon no detecta "bajo ataque" — 
- `specs/ADR-002-mapa-sin-texturas.md` — ADR-002 – Mapa simplificado sin texturas — 
- `specs/ADR-003-almacenamiento-jsonl.md` — ADR-003 – Historial en JSON-lines — 
- `specs/ADR-004-seleccion-manual-de-mundo.md` — ADR-004 – Selección manual de mundo, sin API key — 
- `specs/ADR-005-ruta-simulada.md` — ADR-005 – La ruta de la vista Histórico es una inferencia — 
- `specs/ADR-006-widget-fijo-de-alertas.md` — ADR-006 – Alertas en un widget fijo — 
- `specs/ADR-007-actualizacion-y-reinicio.md` — ADR-007 – Actualizaciones gestionadas por Nexus, con aviso de reinicio — 1. Publicar en GitHub Releases y declarar `UpdateProvider::GitHub` con el enlace del repositorio, para que Nexus gestione la actualización. 2. Activar `AddonFlags::DisableHotloading`: no se descarga el addon en caliente.…
- `specs/ADR-008-presupuesto-de-tamano-y-panic.md` — ADR-008 – Presupuesto de tamaño de DLL y estrategia de panic — 
- `specs/ADR-009-apariencia.md` — ADR-009 – Apariencia: estilo heredado, colores de equipo fijos — 1. Ventanas con el estilo de Nexus por defecto; opción de tema propio en configuración. 2. Colores de equipo clásicos (rojo/azul/verde) fijos, sin modo accesible en v1. 3. Fondo del widget configurable, semitransparente…
- `specs/ADR-010-identidad-de-equipo.md` — ADR-010 – Identidad de equipo tras la reestructuración de mundos — 1. El `team_id` es la única clave de identidad; nombres, alias y colores son presentación (RF-03h). 2. El selector muestra `nombre (id)`; tras elegir se muestran los dos rivales del match para verificar (RF-03g). 3. El u…
- `specs/ADR-011-limites-y-fondo-de-mapa.md` — ADR-011 – Límites del mapa por `continent_rect` y fondo opcional con teselas — 1. Límites del mapa a partir del rectángulo de continente de `/v2/maps`, con respaldo por mínimo/máximo de objetivos. 2. Fondo con teselas y iconos oficiales, **opcional y desactivado por defecto** (RF-37, RF-38): descar…
- `specs/ADR-012-historial-por-sesion.md` — ADR-012 – Historial por sesión: se descarta al arrancar — 1. El historial de eventos es **de la sesión**: al cargar el addon se empieza con el historial vacío y se construye en memoria. 2. El primer snapshot de la sesión es baseline (RF-04) y registra **todos** los claims vigen…
- `specs/ADR-013-presupuesto-de-tamano-2-2-mb.md` — ADR-013 – Objetivo de tamaño de la DLL: 2,2 MB — límite duro de CI sigue en **2,5 MB** (`2 621 440`). ADR-008 fijó 1,5 MB antes de conocer el coste real del addon, dejándolo «a recalibrar con la medición del spike T-01». La medición llegó, y con ella el addon:
- `specs/ADR-014-una-sola-ventana-con-pestanas.md` — ADR-014 · Una sola ventana con pestañas — ventana aparte, centrada, que se abre sola al entrar en MvM o en el Bastión (RF-03j), por petición expresa del jugador. Enmendado de nuevo el 2026-11-02 por **ADR-017**: los avisos salen a un HUD de combate aparte y la v…
- `specs/ADR-015-sin-historial-en-disco.md` — ADR-015 – No hay historial en disco — en ADR-003 y complementa ADR-012. la sesión anterior. Aun así se seguían escribiendo los reclamos de los clanes seguidos en que tiene en memoria, y leer el fichero habría sido una segunda fuente de verdad que además se
- `specs/ADR-016-indice-de-nombres-distribuido.md` — ADR-016 – El índice de nombres de clan se distribuye aparte y no vive en memoria — disco; el historial no) y RF-20b/RF-20c/RF-20d. (medido: «die» → *Die*, «a ne» → nada). Se construyó `guild-names.json` con **9 886** nombres (675 KB) y en la 0.24.0 se cargaba **entero en memoria al arrancar**, se clona…
- `specs/ADR-017-hud-de-combate.md` — ADR-017 · HUD de combate separado de la consola — ADR-014 juntó cinco ventanas en una sola con pestañas. Eso arregló el desorden, pero la revisión de UX (Nielsen 22/40) mostró otro problema: durante una pelea lo único que importa son los avisos, y la ventana completa me…
- `specs/glossary.md` — Glosario — 
- `specs/manual-tests.md` — Pruebas manuales en el juego — Requisitos que solo se pueden comprobar con GW2 + Nexus (referenciados por la trazabilidad). Se ejecutan en T-19 y antes de cada release. Anota fecha, versión, resultado. **2026-11-02:** el jugador da por buenas las comp…
- `specs/templates/adr-template.md` — ADR-NNN – Título — 
- `START-HERE.md` — START-HERE – Guía para ti (la persona) — Esta guía te dice qué hacer tú y cómo trabajar con la IA. Los detalles de qué construir están en `specs/`; las instrucciones que lee la IA, en `AGENTS.md`. **A. Cuenta y repositorio** **B. Entorno**
