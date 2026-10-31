# ADR-008 – Presupuesto de tamaño de DLL y estrategia de panic

- **Estado:** Aceptado (cifras provisionales)
- **Decisión:** objetivo 1.5 MB y límite de CI 2.5 MB para la DLL release, a recalibrar con la medición del spike T-01. Perfil release con `opt-level="z"`, `lto="fat"`, `codegen-units=1`, `strip="symbols"`. Se mantiene `panic="unwind"`.
  - **Recalibrado (2026-10-31):** el objetivo pasa a **2,2 MB** y el límite sigue en 2,5 MB. Ver ADR-013; lo de arriba sobre el perfil release sigue vigente.
- **Motivo de no usar `panic="abort"`:** un panic cerraría el juego entero. Con `unwind` se puede capturar con `catch_unwind` en la frontera FFI (RNF-06). Se acepta el coste de tamaño.
- **Medidas:** sin `tokio`/`reqwest` por defecto, elegir HTTP/TLS por medición, `cargo bloat` en cada release.
- **Consecuencias:** si se supera el límite, se evalúa por este orden: quitar features de dependencias, cambiar de cliente HTTP, reducir `opt-level`, y solo al final revisar el límite.
