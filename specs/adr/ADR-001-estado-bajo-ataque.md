# ADR-001 – El addon no detecta "bajo ataque"

- **Estado:** Aceptado
- **Contexto:** El usuario quiere un aviso cuando un enemigo empiece a atacar una estructura. La API pública `/v2/wvw/matches` expone `owner`, `last_flipped`, `claimed_by`, `claimed_at`, `yaks_delivered`, `guild_upgrades`, pero no el estado "contested" del juego.
- **Decisión:** v1 avisa de flips/capturas y de cambios observables en objetivos propios. No se promete detección de ataque en curso.
- **Alternativas descartadas:** heurísticas indirectas (poco fiables); lectura de memoria del juego (viola la constitución).
- **Consecuencias:** el aviso llega cuando la estructura ya cambió de dueño, con retraso de hasta un intervalo de polling. Si aparece una fuente legítima de estado "contested", se reabre este ADR.
