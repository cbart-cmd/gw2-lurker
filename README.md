# WvW Watch (nombre provisional)

Addon de [Raidcore Nexus](https://raidcore.gg/nexus) para Guild Wars 2, escrito en Rust. Solo funciona en Mundo contra Mundo.

- Avisa en un widget fijo cuando una estructura de tu equipo cambia de dueño.
- Sigue guilds enemigas: avisa de sus claims y guarda un historial local.
- Mapa simplificado y vista Histórico con la **ruta inferida** de claims por guild.

> Limitaciones: la API pública de GW2 no expone el estado "bajo ataque" y no guarda historial. Los avisos son de capturas, y el historial solo cubre el tiempo en que el addon estuvo activo (ver `specs/adr/ADR-001` y `specs/01-requirements.md`).

## Instalación
_Pendiente (T-20)._

## Desarrollo
Lee `START-HERE.md` y `specs/`. Agentes de IA: `AGENTS.md`.

## Licencia

GPL-3.0-only. Ver [LICENSE](./LICENSE).
