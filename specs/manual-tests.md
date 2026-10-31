# Pruebas manuales en el juego

Requisitos que solo se pueden comprobar con GW2 + Nexus (referenciados por la trazabilidad). Se ejecutan en T-19 y antes de cada release. Anota fecha, versión, resultado.

| ID | Requisito | Prueba | Resultado esperado |
|---|---|---|---|
| MT-01 | RF-01 | Entrar a cada mapa de WvW | El addon se activa en ≤ 1 intervalo |
| MT-02 | RF-02 | Salir a una ciudad | Sin peticiones de red, sin UI |
| MT-03 | RF-03a/c | Primer uso y cambio de mundo | Selector aparece; el cambio se aplica |
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
