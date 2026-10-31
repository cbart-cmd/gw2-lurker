# ADR-006 – Alertas en un widget fijo

- **Estado:** Aceptado (decisión del usuario; sustituye al valor por defecto "toast + log" propuesto antes)
- **Contexto:** las alertas deben ser visibles de un vistazo durante el combate sin tapar la pantalla ni exigir interacción.
- **Decisión:** el canal principal de alertas es un widget fijo, movible, redimensionable y bloqueable (click-through). El log ampliado se abre desde el widget. El toast y el sonido pasan a ser opcionales.
- **Alternativas descartadas:** solo toasts (desaparecen y tapan el centro de la pantalla); solo ventana de log (requiere abrirla).
- **Consecuencias:** hay que persistir geometría y ajustes del widget, y verificar que el modo bloqueado no interfiere con el input del juego (R-8).
