# gw2-lurker

Addon de [Raidcore Nexus](https://raidcore.gg/nexus) para Guild Wars 2, escrito en Rust. Solo
funciona en Mundo contra Mundo.

- **Avisos** de lo que pasa en tu matchup: campamentos, torres, fortalezas/garris y castillos
  (las ruinas nunca avisan), con un aviso emergente opcional.
- **Clanes**: sigues a los que te interesan (máximo 10) y se les ve por su nombre en el mapa, en los
  avisos y en la lista, también fuera de WvW.
- **Mapa** simplificado por mapa con los claims de tus clanes marcados, y **ruta inferida** de lo
  que ha reclamado cada uno.
- **Búsqueda de clanes** por un trozo del nombre, con un índice de ~9 900 nombres que el addon
  descarga solo la primera vez.
- **Una sola ventana con pestañas**: Avisos · Mapa · Clanes · Registro · Ajustes.

> Limitaciones honestas: la API pública de GW2 **no** expone el estado «bajo ataque» ni guarda
> historial, y a veces **congela una partida entera** durante minutos (el addon lo avisa en naranja).
> El historial es **de la sesión**: al reiniciar el juego empieza de cero (ADR-012, ADR-015). Las
> rutas son inferidas, no un rastreo, y el addon lo dice en pantalla.

## Instalación

Nexus instala y actualiza el addon solo desde el repositorio de distribución. Para probar una
compilación local, `install.bat` (doble clic) o `pwsh -File install.ps1 [-Build] [-DryRun]`; copia
`gw2_lurker.dll` a `<juego>\addons` comprobando tamaño, `MZ` y MD5, y se niega con GW2 abierto.

## Datos que escribe

- `settings.json` (en `<Documentos>\Guild Wars 2\addons\gw2_lurker\`): ajustes y los nombres de los
  clanes que sigues.
- `temp.json`: nombres de los clanes vistos en la sesión; **se borra en cada arranque**.
- `guild-names.json`: índice de búsqueda, descargado de su ZIP publicado.
- `tiles/`: caché de las teselas del mapa.

## Desarrollo

Lee `START-HERE.md` y `specs/` (empieza por `00-constitution.md`, `01-requirements.md` y
`02-design.md`). Agentes de IA: `AGENTS.md`. Bitácora: `PROGRESO.md`. Fallos del jugador: `bugs.md`.

## Licencia

GPL-3.0-only. Ver [LICENSE](./LICENSE).
