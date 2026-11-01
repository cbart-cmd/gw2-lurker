# Pruebas manuales en el juego

Requisitos que solo se pueden comprobar con GW2 + Nexus (referenciados por la trazabilidad). Se ejecutan en T-19 y antes de cada release. Anota fecha, versión, resultado.

**2026-11-02:** el jugador da por buenas las comprobaciones previas MT-01…MT-20 y las validaciones
anteriores de carga/arranque (T-33/T-35). Se registra su aceptación, no una ejecución ni medición
nueva por la IA. El nuevo seguimiento B — Estela de 0.26.6 se entrega para su prueba MT-21.

| ID | Requisito | Prueba | Resultado esperado |
|---|---|---|---|
| MT-01 | RF-01 | Entrar a cada mapa de WvW | El addon se activa en ≤ 1 intervalo |
| MT-02 | RF-02 | Salir a una ciudad | Sin peticiones de red, sin UI |
| MT-03 | RF-03a/c | Sin equipo en la cuenta actual, recorrer Avisos/Mapa/Clanes/Registro; elegirlo en Ajustes y volver; repetir tras olvidar el equipo con datos en memoria | Sin equipo, cada pestaña de datos dice «No hay datos hasta que elijas un equipo en Ajustes», sin contenido ni estadísticas anteriores; Ajustes funciona. Tras elegirlo vuelve el contenido normal |
| MT-04 | RF-16, RF-17 | Mover y redimensionar el widget; reiniciar el juego | Posición y tamaño persisten |
| MT-05 | RF-18 | Bloquear el widget y hacer clic encima / `Alt+Shift+L` | El clic llega al juego; el atajo alterna el bloqueo |
| MT-06 | RF-16 | Widget bloqueado sin alertas | Totalmente oculto |
| MT-07 | RF-10 | Esperar una pérdida real de estructura propia | Aparece en el widget con datos correctos |
| MT-08 | RF-21, RF-22 | Seguir una guild con claims reales | Aviso, y entrada en el historial |
| MT-09 | RF-30..34 | Abrir mapa e histórico | Posiciones plausibles; ruta con banner "inferida" |
| MT-10 | RF-43, RNF-04 | Cortar la red durante 5 min | Marca "obsoleto"; el juego sigue fluido; recupera al volver |
| MT-11 | RF-51 | Corromper `config.json` a mano | Modo seguro, copia `.corrupt`, sin cierre |
| MT-12 | RF-50 | Actualizar el addon desde Nexus con el juego abierto | Banner "reinicia el juego"; tras reiniciar, versión nueva |
| MT-13 | RNF-01 | Medir frame time con widget y mapa abiertos | < 0.5 ms añadido |
| MT-14 | RF-60/61 | Cambiar estilo Nexus/propio | Solo cambian las ventanas del addon |
| MT-15 | RF-03d | Primer día tras el reset semanal | Color de equipo correcto, sin avalancha de alertas |
| MT-16 | RF-03a/g | Elegir mundo y comparar con el panel de WvW del juego | El selector muestra `nombre (id)`; los rivales mostrados coinciden con los del juego |
| MT-17 | RF-37 | Activar fondo con teselas | El fondo carga sin bloquear el render; se puede desactivar |
| MT-18 | RF-19e | Todas las casillas de estructuras marcadas, capturar ruinas y un campamento | Solo sale el campamento; las ruinas no avisan ni en la lista ni en el pop-up |
| MT-19 | RF-20c | Borrar `guild-names.json` de la carpeta del addon y arrancar el juego | El índice se descarga solo (Ajustes dice cuántos nombres) y el buscador encuentra «a new» y «DIE» |
| MT-20 | RF-20b/d | Seguir un clan que no esté en el índice, reiniciar y mirar `settings.json` | El nombre está en `guild_names` (no en el índice) y no se vuelve a pedir |
| MT-21 | RF-32/32a/33/36, RNF-01 | Abrir la ruta de un clan con uno/varios claims, revisitas, pausas largas y cambios de mapa; arrastrar cronología, usar flechas/Home/End y Todo/1 h/15 min; reproducir y cambiar de pestaña, cerrar o colapsar widget | Curvas inferidas con flechas, último claim y selección claros, intervalos distintos de gaps, sin puentes entre mapas ni coordenadas ausentes. Play avanza cada 700 ms y pausa al ocultarse. Sin red/E/S extra ni tirones; medir también frame completo <0,5 ms |
