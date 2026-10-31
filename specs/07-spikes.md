# 07 – Spikes: guía y resultados

Un spike es un experimento corto y desechable para convertir una suposición en un hecho. **Los ejecuta una persona** (necesitan el juego o la red real). La IA solo debe confiar en lo que aparezca en la sección *Resultados*.

Regla: si un resultado contradice la spec, no se "arregla" el código; se abre un ADR.

---

## T-01 – Addon mínimo en Nexus (≈ 2–3 h)

**Objetivo:** confirmar la API real de `nexus-rs` y qué se puede hacer dentro del juego.

**Preparación**
1. Instala Nexus en una copia de GW2 y comprueba que abre con `Ctrl+O`.
2. Clona `https://github.com/zerthox/nexus-rs` y lee su README y los ejemplos que incluya.
3. Crea un crate `cdylib` aparte (no en este repo) que use `nexus::export!` con un nombre y una firma negativa aleatoria.

**Qué debe hacer el addon de prueba**
- Dibujar una ventana imgui con el `map_id` actual leído de MumbleLink, el tipo de mapa y el nombre del personaje si está disponible.
- Enviar una alerta con `send_alert` al pulsar un botón.
- Registrar un keybind gestionado por Nexus (`Alt+Shift+L`) que cambie un contador visible.
- Probar una segunda ventana con los flags de imgui para **no capturar ratón/teclado** (click-through) y comprobar si los clics llegan al juego.
- Guardar y leer un fichero en el directorio de addon (`paths`).

**Qué anotar (rellena Resultados):**
- Versión de Nexus, de `nexus-rs` y de Rust usadas.
- Firma exacta para registrar el callback de render y el código que funciona.
- `map_id` en cada mapa de WvW: Ojos de Aguas Eternas, Frontera Verde, Azul, Roja, y también el de Bastión del Armisticio y una ciudad (para excluirlos).
- ¿Mumble da `world_id` en WvW? ¿Coincide con tu mundo?
- Flags de imgui que logran el click-through y si hay que reaplicarlos cada frame.
- ¿El keybind es reasignable desde Nexus?
- Tamaño de la DLL release del addon mínimo (`ls -l`).
- Dónde escribe Nexus sus logs y cómo ver los del addon.

**Hecho cuando:** hay un addon que carga, dibuja, avisa, responde al atajo y el click-through está confirmado o descartado.

---

## T-02 – Frecuencia real de la API (≈ 40 min, mayormente esperando)

**Objetivo:** saber cada cuánto cambian de verdad los datos y qué caché declara la API.

1. Averigua el ID de tu mundo (`https://api.guildwars2.com/v2/worlds?ids=all&lang=es`).
2. Ejecuta, **en horario de actividad de WvW**:
   - Windows: `.\scripts\poll_match.ps1 -World <ID> -IntervalSec 10 -DurationMin 30`
   - Linux/macOS/Git Bash: `./scripts/poll_match.sh <ID> 10 30`
3. Se genera `spike-logs/poll_<fecha>.csv` y los cuerpos distintos en `spike-logs/bodies/`.
4. Con `./scripts/capture_fixtures.sh` (o `.ps1`) guarda además las fixtures estáticas. Repítelo 5–10 min después para tener dos snapshots del mismo match.

**Qué anotar:** intervalo mínimo entre cambios consecutivos, mediana, cabecera `Cache-Control`/`Age`, porcentaje de respuestas no-200 y duración de los huecos.

**Hecho cuando:** hay un intervalo de polling por defecto basado en datos (sustituye el provisional de 30 s).

---

## T-02b – Tamaño con distintos clientes HTTP (≈ 2 h)

**Objetivo:** fijar el presupuesto real de la DLL (ADR-008).

1. Usa el addon mínimo de T-01 con el perfil release de `04-engineering-standards.md` §5.
2. Añade una petición HTTPS simple con cada opción y anota el tamaño de la DLL: (a) `ureq` con `rustls`, (b) `ureq` con TLS nativo del sistema, (c) otra alternativa ligera que encuentres.
3. Verifica que cada una funciona dentro del juego (la petición a la API responde).

**Hecho cuando:** hay una tabla opción → tamaño → funciona sí/no, y una elección razonada.

---

## T-25 – Cómo actualiza Nexus un addon (≈ 1–2 h)

**Objetivo:** saber qué hace Nexus con la DLL nueva (ADR-007).

1. Publica dos versiones de prueba del addon mínimo (`0.0.1`, `0.0.2`) en un repo de GitHub con releases, usando `UpdateProvider::GitHub`.
2. Instala la `0.0.1`, abre el juego, anota el listado de la carpeta `addons/` (nombres, tamaños, fechas).
3. Con el juego abierto, fuerza la actualización desde Nexus y vuelve a listar la carpeta.
4. Cierra el juego, ábrelo de nuevo y comprueba qué versión carga.

**Qué anotar:** ¿Nexus sustituye el fichero, crea uno temporal o espera al reinicio? ¿Qué mensaje enseña Nexus? ¿Se puede detectar el cambio desde el addon (fichero nuevo, hash, fecha)?

**Hecho cuando:** ADR-007 pasa de "Propuesto" a "Aceptado" o se sustituye por otro.

---

## Resultados (se rellenan a mano)

### T-01
- Fecha / versiones: _
- Firma de render verificada: _
- `map_id` WvW: EBG=_, Verde=_, Azul=_, Roja=_, Armisticio=_, ciudad=_
- Mumble `world_id` fiable: _
- Click-through: _
- Keybind reasignable: _
- Tamaño DLL mínima: _
- Logs: _

### T-02
- Fecha / hora local y mundo: _
- Intervalo mínimo / mediana de cambio: _
- Cache-Control / Age: _
- % de errores y huecos máximos: _
- Intervalo por defecto elegido: _

### T-02b
| Opción | Tamaño DLL | Funciona en juego |
|---|---|---|
| ureq + rustls | _ | _ |
| ureq + TLS nativo | _ | _ |
| otra | _ | _ |
- Elección y motivo: _

### T-25
- Comportamiento de Nexus ante la actualización: _
- Método de detección viable: _
