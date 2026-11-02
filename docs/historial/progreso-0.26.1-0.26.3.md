# Progreso archivado (0.26.1 → 0.26.3)

Entradas movidas tal cual desde `PROGRESO.md` el 2026-11-02 para no cargarlas en cada sesión.
Lo anterior está en [progreso-hasta-0.26.0.md](progreso-hasta-0.26.0.md).

## HECHO – 2026-11-01 · Publicada la 0.26.3

- Release `gw2-lurker-v0.26.3` en `cbart-cmd/gw2addonsreleases`, MD5
  `bd625f8c7ed73d5e607c40f69daecf35` (2 197 504 B), el mismo que la DLL de `cargo build --release`
  de `bcd7d2f`. `release.ps1 -VerifyOnly` descargó y comprobó el fichero publicado.
- Contenido: el aviso del click-through sigue al `InputBinds.json` real (commit `25dd2a5`) y el
  historial de un clan se pinta de más reciente a más antiguo (`8e2d8d8`, `ui/guilds.rs`).
- Retención: quedan 0.26.3 y 0.26.2; la 0.26.1 ya no está en el repo de distribución.
- Probado en GW2 (2026-11-01) y los tres bugs abiertos quedan `[x]` en `bugs.md`: el aviso nombra
  la tecla nueva, la ruta y el aviso de «sin eventos» funcionan y el historial del clan sale con lo
  más reciente arriba.

## HECHO – 2026-11-01 · Subir versión en un solo paso · 0.26.3

- Subir versión se hacía a mano en cuatro sitios y era fácil dejarse las dependencias internas:
  ahora `scripts/bump-version.ps1 -Version x.y.z` toca `Cargo.toml` del workspace, la `version` de
  las dependencias internas en `crates/*/Cargo.toml`, las entradas `gw2lurker-*` de `Cargo.lock`
  (por texto, sin llamar a cargo) y pasa `[Unreleased]` de `CHANGELOG.md` a la versión con fecha.
  Es idempotente y tiene `-DryRun`, `-Date` y `-SkipChangelog`.
- `scripts/check_version.sh` comprueba que los seis sitios coinciden; `validate.ps1` lo corre como
  paso `version` (también en la pasada por defecto).
- Trampa: en .NET las regex numeran los grupos **con nombre después** de los anónimos, así que
  `${2}` no era el cierre de comilla y duplicó bloques de `Cargo.lock`. Se usa `${head}` + literal.
- De paso, `-Only fmt,version` (la sintaxis que documenta la cabecera) fallaba porque `ValidateSet`
  no admite la lista con comas al invocar con `-File`; ahora se separa y se valida a mano.
- `validate.ps1 -Release`: todo OK, 328 tests, DLL 2 197 504 B. Orden del historial de un clan
  cambiado a más reciente primero en `ui/guilds.rs` (commit `8e2d8d8`, del jugador).

## HECHO – 2026-11-01 · Atajo del click-through desactualizado (RF-18, sin publicar)

- Bug: el jugador cambió el atajo a Alt+Shift+Ñ en Nexus y el aviso seguía diciendo Ctrl+Shift+Ñ.
  `InputBinds.json` (18:33:36) ya decía `Alt: true, Ctrl: false`; el addon lo leía solo al cargar
  en un `OnceLock`.
- `keybinds.rs`: `Watch` compara la fecha del fichero y lo relee solo si cambió; `refresh()` se
  llama en `worker::sleep` (cada 250 ms, nunca en el render). `shortcut()` devuelve `String`.
- Test `rf18_a_rebinding_made_while_playing_reaches_the_message` (fichero temporal reescrito).
  `validate.ps1 -Release`: todo OK, 328 tests. Pendiente: comprobarlo en GW2.

## HECHO – 2026-11-01 · Favoritos coherentes y paso 0 de la ruta · 0.26.2

- Primero: clic en favorito selecciona su historial sin saltar; botón `i` abre el mapa del último
  claim vigente de la fila, o del último histórico si ya no tiene reclamos (decisión del usuario).
  Sin historial ni reclamos, aviso y sin navegación.
- Después: aplicar la propuesta aprobada de origen 0 (garri, fortaleza lateral alpina o respawn
  del equipo del clan), con condiciones temporales y geográficas y coordenadas verificadas.
- Tests primero por hito, validación completa al estabilizar; sin inferir actividad real del clan.
- Regresiones de botones y mapa primero en rojo, después en verde; origen y plan de dibujo también.
  El historial conserva los claims vigentes si su observación antigua salió del tope de 2 000.
- Medición CPU con 2 000 claims: primer intento mediana 1,013 ms; índice de coordenadas bajó a
  0,467 ms pero no cumplió p95. Textos prestados (`Cow`) evitan copiar dos cadenas por paso:
  mediana 0,278 ms, p95 0,352 ms en release, por debajo de 0,5 ms. Frame completo pendiente en GW2.
- Validación completa correcta: 327 tests; los 4 tests de red ignorados también pasan al lanzarlos
  manualmente. Últimos ajustes de selección y destino revalidados; DLL 0.26.2 de 2 195 456 B.
  El destino usa la fila del fotograma para evitar cambios si el worker actualiza durante el clic.
- Publicada 0.26.2 con `scripts/release.ps1`: código `3c8cbcd`, distribución `5df5125`.
  Descarga real verificada: MZ, 2 195 456 B y MD5 `89b25637839b54a31dd848c83b51b57d`.
  Retención: 0.26.2 y 0.26.1; eliminada 0.26.0. `cargo bloat --release --crates` revisado.
- El benchmark CPU solo se compila en release; el publicador conserva su flujo y añade commit
  convencional con trailer de coautoría. Pendiente en GW2: clics, avisos, dibujo 0/1 y frame completo.

## HECHO – 2026-11-01 · Navegación al mapa de un clan (sin publicar; RF-30, RF-71, RF-32a)

- `ui/navigation.rs` conserva el salto pendiente hasta que imgui dibuja Mapa. El acceso manual
  aplica el mapa del jugador antes del dibujo, no después de sobrescribir el mapa del clan.
- Sin eventos, no se cambia la pestaña ni la selección: aviso traducido a es/en/fr/de con la hora
  de inicio del seguimiento y botón para cerrarlo. El selector de clan del mapa pasa por la misma
  validación que favoritos, avisos, registro e información del clan.
- Tres regresiones fallaron con las reglas anteriores; tras el arreglo, 315 tests correctos,
  4 ignorados. `validate.ps1`: fmt, clippy, LOC, trazabilidad y tests correctos.
- Prueba con `match_2-2.json`: clan no favorito, mapa del último claim y resumen de ruta disponibles.
  Sin red, E/S, nuevas dependencias ni consultas adicionales por fotograma al historial.
- Commit: `fix: conserva el mapa del clan al abrir su ruta`. Pendiente: publicar y comprobar la
  navegación y el aviso dentro de GW2. Anotado en `bugs.md`, README y CHANGELOG.

## HECHO – 2026-11-01 · Publicar 0.26.1

- Versión raíz y dependencias internas 0.26.1; CHANGELOG cierra los bugs del atajo con «Ñ» y del
  botón *Grabar*. Commits: `df5cf0e` (versión) y `435c454` (`Cargo.lock`).
- Validación `scripts/validate.ps1 -Release`: fmt, clippy, LOC, trazabilidad, 310 tests y build
  correctos; DLL de 2 189 312 B.
- Publicada con `scripts/release.ps1`: tag `gw2-lurker-v0.26.1`, MD5
  `1430a2dfc25378f3c4b1c9a5a36b23b4`. La descarga real coincidió en tamaño y MD5; retención eliminó
  `gw2-lurker-v0.25.15`.
- La confirmación del atajo «Ñ» requiere probarlo en GW2.
