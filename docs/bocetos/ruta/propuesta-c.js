/* Propuesta C · Escuadras: reparte los claims entre grupos que sí pudieron hacer cada trayecto
   (a pie o con waypoint) y deja sin unir lo que ningún grupo alcanzaba. */
window.LK = window.LK || {};
(function (LK) {
  'use strict';

  const f1 = (n) => Math.round(n * 10) / 10;
  const TINTA = LK.TINTA, esc = LK.esc, AM = LK.AMBAR, HUECO = 'rgba(10,14,19,.8)';
  const W = 3;
  // Color, trazo y forma por escuadra: el color nunca es la única señal. Se repiten de cuatro en cuatro.
  const ESTILOS = [
    { col: AM, dash: '', cap: 'round', forma: 'circulo' },
    { col: '#8fd3ff', dash: '9 6', cap: 'butt', forma: 'cuadrado' },
    { col: '#c79bff', dash: '12 4 2 4', cap: 'butt', forma: 'rombo' },
    { col: '#f2f6fa', dash: '0.1 6.5', cap: 'round', forma: 'triangulo' },
  ];
  const est = (k) => ESTILOS[k % ESTILOS.length];

  const forma = (tipo, x, y, r, fill, stroke, sw) => {
    const a = `fill="${fill}" stroke="${stroke}" stroke-width="${sw}" stroke-linejoin="round"`;
    if (tipo === 'cuadrado') return `<rect x="${f1(x - r)}" y="${f1(y - r)}" width="${f1(2 * r)}" height="${f1(2 * r)}" rx="2" ${a}/>`;
    if (tipo === 'rombo') { const q = r * 1.3; return `<polygon points="${LK.pts([[x, y - q], [x + q, y], [x, y + q], [x - q, y]])}" ${a}/>`; }
    if (tipo === 'triangulo') { const q = r * 1.3; return `<polygon points="${LK.pts([[x, y - q], [x + q * 0.95, y + q * 0.7], [x - q * 0.95, y + q * 0.7]])}" ${a}/>`; }
    return `<circle cx="${f1(x)}" cy="${f1(y)}" r="${f1(r)}" ${a}/>`;
  };
  // Marcador numerado con la forma de su escuadra.
  const marca = (x, y, n, k, o = {}) => {
    const s = est(k), r = o.r || 10, ty = s.forma === 'triangulo' ? y + 3 : y;
    return `<g class="mk d"${o.i ? ` data-i="${o.i}"` : ''}>${o.title ? `<title>${esc(o.title)}</title>` : ''}`
      + forma(s.forma, x, y, r, o.hueco ? HUECO : s.col, o.hueco ? s.col : TINTA, o.hueco ? 1.8 : 1.6)
      + `<text x="${f1(x)}" y="${f1(ty)}" dy=".35em" text-anchor="middle" font-size="${o.fs || 10.5}" font-weight="700" fill="${o.hueco ? s.col : TINTA}">${n}</text></g>`;
  };
  const ico = (k) => `<svg class="ico-eq" viewBox="0 0 18 18" width="18" height="18" aria-hidden="true">${forma(est(k).forma, 9, 9.6, 5.6, est(k).col, TINTA, 1.2)}</svg>`;

  const trazoAttr = (s) => `stroke="${s.col}" stroke-width="${W}"${s.dash ? ` stroke-dasharray="${s.dash}"` : ''} stroke-linecap="${s.cap}"`;
  const retardo = (orden) => `--d:${Math.min(orden * 60, 1200)}ms`;
  // Carrera: funda oscura y línea con el trazo de la escuadra. Solo el trazo continuo se «dibuja»; los demás aparecen.
  const carrera = (a, b, k, orden) => {
    const s = est(k), d = `M${f1(a[0])},${f1(a[1])} L${f1(b[0])},${f1(b[1])}`;
    return `<path d="${d}" fill="none" stroke="${TINTA}" stroke-width="${W + 3.4}" stroke-linecap="round" opacity=".8"/>`
      + `<path class="${s.dash ? 'aparece' : 'traza'}"${s.dash ? '' : ' pathLength="1"'} style="${retardo(orden)}" d="${d}" fill="none" ${trazoAttr(s)}/>`;
  };
  const punta = (a, b, col) => (Math.hypot(b[0] - a[0], b[1] - a[1]) < 70 ? ''
    : LK.flecha((a[0] + b[0]) / 2, (a[1] + b[1]) / 2, LK.angulo(a, b), 5.2, col, `stroke="${TINTA}" stroke-width="1.3" stroke-linejoin="round"`));
  // Teletransporte: arco de puntos del último claim al waypoint.
  const arco = (a, b, k, orden) => {
    const s = est(k), dx = b[0] - a[0], dy = b[1] - a[1];
    const d = `M${f1(a[0])},${f1(a[1])} Q${f1((a[0] + b[0]) / 2 - dy * 0.2)},${f1((a[1] + b[1]) / 2 + dx * 0.2)} ${f1(b[0])},${f1(b[1])}`;
    return `<path d="${d}" fill="none" stroke="${TINTA}" stroke-width="6" stroke-linecap="round" opacity=".7"/>`
      + `<path class="aparece" style="${retardo(orden)}" d="${d}" fill="none" stroke="${s.col}" stroke-width="3.6" stroke-dasharray="0.1 8.5" stroke-linecap="round"/>`;
  };
  // Escuadra nueva: «?» hueco junto al primer claim, del lado contrario a donde sigue la escuadra.
  const nueva = (v, t, pi) => {
    const s = est(t.pista), [x, y] = v.P(t.b.pos);
    const sig = pi.pistas[t.pista].pasos.find((p) => p.i > t.b.i && p.pos && p.mapa === t.b.mapa);
    const [nx, ny] = sig ? v.P(sig.pos) : [x + 20, y + 20];
    const ang = Math.atan2(y - ny, x - nx);
    const lim = (n) => Math.min(Math.max(n, 18), LK.L - 18);
    const ox = lim(x + Math.cos(ang) * 36), oy = lim(y + Math.sin(ang) * 36);
    return {
      bajo: `<line x1="${f1(ox)}" y1="${f1(oy)}" x2="${f1(x)}" y2="${f1(y)}" stroke="${s.col}" stroke-width="2" stroke-dasharray="3 3"/>`,
      alto: marca(ox, oy, '?', t.pista, { r: 8.5, hueco: true, i: t.b.i, title: `Escuadra ${t.pista + 1}: empieza en el paso ${t.b.i}. Ninguna escuadra anterior llegaba a tiempo y no se sabe de dónde sale.` }),
    };
  };

  function marco(m, mf, i, st) {
    const pi = LK.pistas(m, st.v);
    const enMapa = pi.tramos.filter((t) => t.b.mapa === mf.mapa);
    const hubs = enMapa.filter((t) => t.modo === 'wp' && t.hub);
    const origen = LK.origenHoy(m, mf);
    const v = LK.vista(mf, LK.extras(m, mf).concat(hubs.map((t) => t.hub.pos)), st.completo);
    const primero = mf.pasos.find((p) => p.pos);
    const sal = LK.salida(v, mf, origen, { col: est(primero ? pi.pistaDe[primero.i] : 0).col });
    const cajas = [];
    const ocupa = (p) => { const [x, y] = v.P(p); cajas.push({ x: x - 16, y: y - 16, w: 32, h: 32 }); };
    LK.grupos(mf).forEach((g) => ocupa(g.pos));
    if (origen.pos) ocupa(origen.pos);
    hubs.forEach((t) => ocupa(t.hub.pos));

    let bajo = sal.bajo, alto = sal.alto, orden = 0;
    const waypoints = [];
    enMapa.forEach((t) => {
      const s = est(t.pista), ids = `data-n="${t.b.i}" data-i="${t.a ? `${t.a.i} ` : ''}${t.b.i}"`;
      if (t.modo === 'correr' && t.a && t.a.pos && t.b.pos && t.a.id !== t.b.id) {
        const a = v.P(t.a.pos), b = v.P(t.b.pos);
        bajo += `<g class="d" ${ids}>${carrera(a, b, t.pista, orden)}${punta(a, b, s.col)}</g>`;
        orden++;
      } else if (t.modo === 'wp' && t.hub && t.a && t.a.pos && t.b.pos) {
        const a = v.P(t.a.pos), h = v.P(t.hub.pos), b = v.P(t.b.pos);
        bajo += `<g class="d" ${ids}>${arco(a, h, t.pista, orden)}${carrera(h, b, t.pista, orden + 1)}${punta(h, b, s.col)}</g>`;
        alto += `<g class="d" ${ids}><circle cx="${f1(h[0])}" cy="${f1(h[1])}" r="15" fill="none" stroke="${s.col}" stroke-width="2"/></g>`;
        waypoints.push({ ids, h, s, t });
        orden += 2;
      } else if (t.modo === 'inicio' && t.b.pos && t.b !== primero) {
        const n = nueva(v, t, pi);
        bajo += n.bajo;
        alto += n.alto;
      }
    });
    LK.huerfanos(m, mf).forEach((h) => { const s = LK.marcaSinPos(v, h, est(pi.pistaDe[h.p.i]).col); bajo += s.bajo; alto += s.alto; });
    LK.grupos(mf).forEach((g) => {
      alto += LK.discoGrupo(v, g, { ultimo: m.pasos.length, disco: { r: 10 }, marca: (x, y, n, o) => marca(x, y, n, pi.pistaDe[g.pasos[0].i], o) });
    });
    // Las pastillas «WP» y los cruces se colocan al final, cuando ya se conocen las etiquetas de nombre.
    const pastillas = (cs) => waypoints.map(({ ids, h, s, t }) => `<g class="d" ${ids}>${LK.pastillaLibre(cs, h[0], h[1], `WP · ${LK.nombre(t.hub.id)}`, { col: s.col }, [0, -30])}</g>`).join('')
      + LK.cruces(m, mf, v, cs);

    const escs = [...new Set(mf.pasos.map((p) => pi.pistaDe[p.i]))].sort((a, b) => a - b);
    const lista = (k) => { const ps = pi.pistas[k].pasos.filter((p) => p.mapa === mf.mapa).map((p) => p.i); return ps.length > 6 ? `${ps.length} pasos` : `pasos ${ps.join(', ')}`; };
    const pie = `<span>${LK.textoOrigen(origen)}</span>` + escs.map((k) => `<span class="esq">${ico(k)}Escuadra ${k + 1} · ${lista(k)}</span>`).join('')
      + (hubs.length ? `<span>${hubs.length} ${hubs.length === 1 ? 'waypoint' : 'waypoints'}</span>` : '') + LK.bandeja(mf);
    return LK.figura(m, mf, i, v, { bajo, alto: (cs) => alto + pastillas(cs), cajas, pie, ids: (origen.id ? [origen.id] : []).concat(hubs.map((t) => t.hub.id)), aria: `${LK.mapaNombre(mf.mapa)}: ruta inferida repartida en escuadras` });
  }

  function extra(m, st) {
    const pi = LK.pistas(m, st.v);
    const filas = pi.tramos.filter((t) => t.a || t.pista > 0).map((t) => {
      const lect = t.modo === 'wp' ? `Waypoint en ${esc(LK.nombre(t.hub.id))} y ${LK.miles(LK.dist(t.hub.pos, t.b.pos))} u a pie`
        : { correr: 'A pie', inicio: 'Escuadra nueva: ninguna otra llegaba a tiempo', sinpos: 'Sin posición', mapa: 'Cambio de mapa' }[t.modo];
      return {
        attrs: `data-n="${t.b.i}" data-i="${t.a ? `${t.a.i} ` : ''}${t.b.i}" tabindex="0"`,
        celdas: [`${ico(t.pista)}<span>Escuadra ${t.pista + 1}</span>`, t.a ? `${t.a.i} → ${t.b.i}` : `→ ${t.b.i}`, t.d === null ? '—' : LK.miles(t.d), t.dt === null ? '—' : LK.dur(t.dt), t.vreq === null ? '—' : t.vreq.toFixed(1), lect],
      };
    });
    if (!filas.length) return '';
    return LK.tabla('Tramos por escuadra', [{ t: 'Escuadra' }, { t: 'Tramo' }, { t: 'En recta (u)', n: 1 }, { t: 'Tiempo', n: 1 }, { t: 'u/s necesarias', n: 1 }, { t: 'Lectura' }], filas,
      `Con una velocidad de carrera de ${st.v} u/s y 20 s de espera por waypoint. Cada claim va a la escuadra que lo explica con menos prisa.`);
  }

  const iconos = {
    formas: ESTILOS.map((s, k) => forma(s.forma, 4.5 + k * 8, 9.3, 3.2, s.col, TINTA, 1)).join(''),
    trazos: ESTILOS.map((s, k) => `<line x1="2" y1="${3 + k * 4}" x2="32" y2="${3 + k * 4}" stroke="${s.col}" stroke-width="2" ${s.dash ? `stroke-dasharray="${s.dash}"` : ''} stroke-linecap="${s.cap}"/>`).join(''),
    wp: `<path d="M3,15 Q10,0 23,8" fill="none" stroke="${AM}" stroke-width="2.8" stroke-dasharray="0.1 5.5" stroke-linecap="round"/><circle cx="27.5" cy="9" r="5" fill="none" stroke="${AM}" stroke-width="1.6"/>`,
    nueva: `<line x1="3" y1="9" x2="14" y2="9" stroke="#8fd3ff" stroke-width="2" stroke-dasharray="3 3"/>${marca(24, 9, '?', 1, { r: 6.5, hueco: true, fs: 9 })}`,
    sinpos: LK.disco(17, 9, '3?', { r: 8, hueco: true, dash: '3 2', stroke: '#c4d0da', fs: 8.5 }),
    salida: `<path d="M3,9 L31,9" stroke="${AM}" stroke-width="2.4" stroke-dasharray="7 5" stroke-linecap="round"/>${LK.disco(5, 9, '0', { r: 5.5, hueco: true, fs: 8 })}`,
    cruce: '<rect x="1" y="2" width="32" height="14" rx="7" fill="rgba(10,14,19,.93)" stroke="#9fb3c4" stroke-width="1.2"/><text x="17" y="9" dy=".35em" text-anchor="middle" font-size="8" fill="#f4eadb">viene de</text>',
  };

  LK.P = {
    id: 'c', nombre: 'Escuadras', marco, extra,
    cambia: [
      'Si un mismo grupo no puede hacer un tramo en el tiempo que hubo, ya no se dibuja una línea imposible: los claims se reparten en escuadras, cada una con su color, su forma y su trazo.',
      'Si una escuadra solo llega con un waypoint (el spawn, una fortaleza o un castillo propios), se dibuja: arco de puntos hasta el waypoint y carrera desde allí.',
      'Una escuadra que empieza sin origen conocido lo dice con un «?» en vez de inventarse un paso 0.',
    ],
    leer: [
      [iconos.formas, 'Cada escuadra, su forma y color: círculo, cuadrado, rombo y triángulo. El número es el del paso.'],
      [iconos.trazos, 'Y su trazo: continuo, rayas, raya y punto, puntos.'],
      [iconos.wp, 'Waypoint: arco de puntos hasta el anillo del waypoint y carrera desde allí.'],
      [iconos.nueva, 'Escuadra nueva: ningún grupo anterior llegaba a tiempo y no se sabe de dónde sale.'],
      [iconos.sinpos, 'Paso sin posición: sigue a su escuadra, en el punto medio de sus vecinos.'],
      [iconos.salida, 'Salida inferida (paso 0) para el primer claim del mapa.'],
      [iconos.cruce, 'Etiqueta gris: el recorrido entra o sale de este mapa.'],
    ],
    controles: [
      { k: 'v', tipo: 'rango', et: 'Velocidad de carrera', min: 8, max: 30, u: ' u/s', ayuda: 'Valor de partida estimado (UNVERIFIED). Súbelo para ver cuándo dos escuadras se funden en una; bájalo y se separan más.' },
    ],
    defecto: { v: LK.V_DEFECTO },
    notas: {
      n1: 'Una sola escuadra: los seis claims se encadenan a pie. El tramo más exigente (3→4: 776 u en 4 min 45 s) pide 2,7 u/s, muy por debajo de los 14 de partida.',
      n2: 'Con 14 u/s el 2→3 no se corre (1 335 u en 80 s piden 16,7 u/s), pero sí con el waypoint de Shadaran Hills: quedan 665 u a pie desde él. A partir de 17 u/s ya no hace falta waypoint.',
      n3: 'Un solo claim: una escuadra y ningún tramo que repartir. Queda el paso 0 desde la garri, como en las demás propuestas.',
      e1: 'Dos escuadras: 1·4·5 y 2·3·6. Un mismo grupo no hace los saltos 1→2, 3→4 y 5→6 (80, 90 y 100 s piden 25,5, 20,4 y 15,7 u/s), así que alternan dos. La escuadra 2 sale con «?»: nada dice de dónde viene. Con 30 u/s se funden en una.',
      e2: 'Una escuadra de 28 claims. El 21→22 (2 053 u en 90 s: 22,8 u/s) solo cabe con el waypoint de Lowlands (342 u a pie desde él). El 17→18 deja 45 min de hueco y C lo une igualmente, porque cabe: nada prueba que sea el mismo grupo.',
      e3: 'Una escuadra. El paso 3 no tiene posición y sale como «3?» entre el 2 y el 7, con el estilo de su escuadra. Los cambios de mapa (3→4 y 6→7) llevan «sigue en» y «viene de».',
    },
    necesita: [
      'Qué waypoints sirven: el spawn y las fortalezas y castillos que el equipo posee (la API da el dueño). Que estén disponibles, sin disputar, no lo da la API (ADR-001): UNVERIFIED.',
      'Una velocidad de carrera (14 u/s) y una espera de waypoint (20 s), ambas UNVERIFIED: deciden qué claims caben en una escuadra.',
      'Una regla de reparto (aquí, la escuadra que lo explica con menos prisa). Es una heurística: dos grupos reales muy cerca pueden salir como uno.',
      'Sin llamadas nuevas a la API; sigue rotulado como «ruta inferida» (ADR-005). Spec o ADR nuevo antes de implementar (AGENTS §2.4).',
    ],
  };
})(window.LK);
