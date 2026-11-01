/* Propuesta A · Caminos: la ruta sigue la red de caminos del mapa en lugar de la recta. */
window.LK = window.LK || {};
(function (LK) {
  'use strict';

  const f1 = (n) => Math.round(n * 10) / 10;
  const AM = LK.AMBAR, TINTA = LK.TINTA;
  const trazo = (pts) => 'M' + pts.map((p) => `${f1(p[0])},${f1(p[1])}`).join(' L');
  const largo = (pts) => pts.reduce((s, p, k) => (k ? s + LK.dist(pts[k - 1], p) : 0), 0);

  // Red completa en gris tenue, para ver por dónde podría ir cada tramo.
  function red(v, mapa) {
    const r = LK.red(mapa);
    let s = '';
    r.aristas.forEach((e) => { s += `<path d="${trazo(LK.chaikin(e.pts.map((p) => v.P(p))))}" fill="none" stroke="#c5d3df" stroke-opacity=".4" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>`; });
    Object.values(r.nodos).forEach((p) => { const [x, y] = v.P(p); s += `<circle cx="${f1(x)}" cy="${f1(y)}" r="2.4" fill="#c5d3df" fill-opacity=".6"/>`; });
    return s;
  }

  // Una cinta por arista usada (orden del primer uso, para que el dibujo siga la marcha), con puntas de sentido.
  function cintas(v, rc, mapa) {
    const vistos = new Set();
    let k = 0, funda = '', cinta = '', puntas = '';
    rc.legs.forEach((lg) => {
      if (lg.modo !== 'camino' || lg.t.a.mapa !== mapa) return;
      lg.c.tramos.forEach((t) => {
        if (vistos.has(t.e.k)) return;
        vistos.add(t.e.k);
        const u = rc.uso[t.e.k];
        const suave = LK.chaikin(t.e.pts.map((p) => v.P(p)));
        const w = 4.2 + 1.5 * Math.min(u.n - 1, 4);
        const d = trazo(t.dir === 1 ? suave : suave.slice().reverse());
        funda += `<path d="${d}" fill="none" stroke="${TINTA}" stroke-width="${f1(w + 3)}" stroke-linecap="round" stroke-linejoin="round" opacity=".88"/>`;
        cinta += `<path class="traza" pathLength="1" style="--d:${k * 70}ms" d="${d}" fill="none" stroke="${AM}" stroke-width="${f1(w)}" stroke-linecap="round" stroke-linejoin="round"/>`;
        const n = Math.max(1, Math.floor(largo(suave) / 85));
        for (let j = 0; j < n; j++) {
          if (u.dirs[1]) { const q = LK.puntoEn(suave, (j + (u.dirs[-1] ? 0.25 : 0.5)) / n); puntas += LK.flecha(q.x, q.y, q.ang, 3.6, TINTA); }
          if (u.dirs[-1]) { const q = LK.puntoEn(suave, (j + (u.dirs[1] ? 0.75 : 0.5)) / n); puntas += LK.flecha(q.x, q.y, q.ang + 180, 3.6, TINTA); }
        }
        k++;
      });
    });
    return { funda, cinta, puntas };
  }

  function marco(m, mf, i, st) {
    const v = LK.vista(mf, LK.extras(m, mf), st.completo);
    const rc = LK.rutaCaminos(m, st.v);
    const origen = LK.origenHoy(m, mf);
    const c = cintas(v, rc, mf.mapa);
    const sal = LK.salida(v, mf, origen, { red: true });
    let bajo = (st.red ? red(v, mf.mapa) : '') + c.funda + sal.bajo + c.cinta + c.puntas;
    let alto = '';
    let nSalto = 0, total = 0;
    rc.legs.forEach((lg) => {
      const t = lg.t;
      if (t.a.mapa !== mf.mapa || t.b.mapa !== mf.mapa) return;
      if (lg.modo === 'camino') total += lg.len;
      if (lg.modo !== 'salto') return;
      nSalto++;
      const a = v.P(t.a.pos), b = v.P(t.b.pos);
      bajo += `<line class="d" data-n="${t.n}" x1="${f1(a[0])}" y1="${f1(a[1])}" x2="${f1(b[0])}" y2="${f1(b[1])}" stroke="${AM}" stroke-width="3.4" stroke-dasharray="1 8" stroke-linecap="round"/>`;
      alto += `<g class="d" data-n="${t.n}">${LK.pastilla((a[0] + b[0]) / 2, (a[1] + b[1]) / 2, `Salto · ${LK.dur(t.dt)}`)}</g>`;
    });
    LK.huerfanos(m, mf).forEach((h) => { const s = LK.marcaSinPos(v, h); bajo += s.bajo; alto += s.alto; });
    LK.grupos(mf).forEach((g) => { alto += LK.discoGrupo(v, g, { ultimo: m.pasos.length }); });
    const pie = `<span>${LK.textoOrigen(origen)}</span><span>${mf.pasos.length} ${mf.pasos.length === 1 ? 'paso' : 'pasos'}</span>`
      + (total ? `<span>${LK.miles(total)} u por caminos</span>` : '') + (nSalto ? `<span>${nSalto} ${nSalto === 1 ? 'salto' : 'saltos'}</span>` : '') + LK.bandeja(mf);
    return LK.figura(m, mf, i, v, { bajo, alto: (cajas) => alto + sal.alto + LK.cruces(m, mf, v, cajas), pie, ids: origen.id ? [origen.id] : [], aria: `${LK.mapaNombre(mf.mapa)}: ruta inferida por la red de caminos` });
  }

  function extra(m, st) {
    const rc = LK.rutaCaminos(m, st.v);
    const modo = { camino: 'A pie por la red', salto: 'No cabe a pie', quieto: 'Mismo objetivo', sinpos: 'Sin posición', mapa: 'Cambio de mapa' };
    if (!rc.legs.length) return '';
    const filas = rc.legs.map((lg) => ({
      attrs: `data-n="${lg.t.n}" data-i="${lg.t.b.i}" tabindex="0"`,
      celdas: [`${lg.t.a.i} → ${lg.t.b.i}`, lg.t.d === null ? '—' : LK.miles(lg.t.d), lg.len ? LK.miles(lg.len) : '—', LK.dur(lg.t.dt), lg.vcam ? lg.vcam.toFixed(1) : '—', modo[lg.modo]],
    }));
    return LK.tabla('Tramos del recorrido', [{ t: 'Tramo' }, { t: 'En recta (u)', n: 1 }, { t: 'Por caminos (u)', n: 1 }, { t: 'Tiempo', n: 1 }, { t: 'u/s por caminos', n: 1 }, { t: 'Lectura' }], filas,
      `Con una velocidad de carrera de ${st.v} u/s. «No cabe a pie» significa que el camino pide más velocidad que esa.`);
  }

  const ico = {
    cinta: `<path d="M3,9 L31,9" stroke="${TINTA}" stroke-width="8" stroke-linecap="round"/><path d="M3,9 L31,9" stroke="${AM}" stroke-width="5" stroke-linecap="round"/>${LK.flecha(12, 9, 0, 3.6, TINTA)}${LK.flecha(24, 9, 0, 3.6, TINTA)}`,
    salto: `<path d="M3,9 L31,9" stroke="${AM}" stroke-width="3.4" stroke-dasharray="1 8" stroke-linecap="round"/>`,
    salida: `<path d="M3,9 L31,9" stroke="${AM}" stroke-width="2.4" stroke-dasharray="7 5" stroke-linecap="round"/>${LK.disco(5, 9, '0', { r: 5.5, hueco: true, fs: 8 })}`,
    sinpos: LK.disco(17, 9, '3?', { r: 8, hueco: true, dash: '3 2', stroke: '#c4d0da', fs: 8.5 }),
    revisita: `${LK.disco(15, 10, '4', { r: 7 })}<rect x="19" y="0.5" width="14" height="10" rx="5" fill="${TINTA}" stroke="${AM}"/><text x="26" y="6" dy=".35em" text-anchor="middle" font-size="8" font-weight="700" fill="#f4eadb">×2</text>`,
    cruce: `<rect x="1" y="2" width="32" height="14" rx="7" fill="rgba(10,14,19,.93)" stroke="#9fb3c4" stroke-width="1.2"/><text x="17" y="9" dy=".35em" text-anchor="middle" font-size="8" fill="#f4eadb">viene de</text>`,
  };

  LK.P = {
    id: 'a', nombre: 'Caminos', marco, extra,
    cambia: [
      'En vez de una recta entre claims, la ruta sigue la red de caminos del mapa (el camino más corto entre los dos objetivos).',
      'Si un tramo no se puede correr en el tiempo que hubo, no se disfraza de camino: sale como salto punteado.',
      'Marca los pasos sin posición, las revisitas, los cambios de mapa y la salida (paso 0) por la red.',
    ],
    leer: [
      [ico.cinta, 'Cinta por los caminos. Más gruesa si se repite; las puntas oscuras dan el sentido.'],
      [ico.salto, 'Puntos grandes: salto. El tramo pide más velocidad que la elegida.'],
      [ico.salida, 'Salida inferida (paso 0), por la red hasta el primer claim.'],
      [ico.sinpos, 'Paso sin posición: el catálogo no trae su coordenada.'],
      [ico.revisita, 'Insignia ×N: el clan vuelve al mismo objetivo.'],
      [ico.cruce, 'Etiqueta gris: el recorrido viene de otro mapa o sigue en otro.'],
    ],
    controles: [
      { k: 'v', tipo: 'rango', et: 'Velocidad de carrera', min: 8, max: 30, u: ' u/s', ayuda: 'Valor de partida estimado (UNVERIFIED). Súbelo para ver qué saltos serían posibles con montura o waypoint.' },
      { k: 'red', tipo: 'check', et: 'Ver la red de caminos' },
    ],
    defecto: { v: LK.V_DEFECTO, red: false },
    notas: {
      n1: 'Los cinco tramos se hacen a pie con mucho margen (hasta 2,8 u/s de 14). El 4→5 por camino son 839 u en vez de los 650 de la recta.',
      n2: 'El tramo 2→3 son 80 s y por la red hay 2 708 u (34 u/s): no se corre, y la cinta se corta en un salto. El 4→5 rodea: 1 214 u por camino contra 798 en recta.',
      n3: 'No hay tramos, pero sí paso 0: la salida desde el garri baja por la red (628 u) en lugar de una recta suelta.',
      e1: 'Los tramos 1→2, 3→4 y 5→6 son saltos (80, 90 y 100 s para 1 943 a 2 435 u por camino). Con 14 u/s es imposible que sea un solo grupo; el 2→3 y el 4→5 sí se corren.',
      e2: 'Veintisiete tramos: casi todo cabe a pie. Las cintas gruesas son caminos repetidos; el 15→16 es el mismo objetivo dos veces (×2), el 17→18 tiene 45 min de hueco y el 21→22 es un salto (2 527 u en 90 s).',
      e3: 'El paso 3 no tiene posición: sale un «3?» sobre un punteado entre el 2 y el 7, los vecinos con posición en ese mapa. El 6→7 cambia de mapa, y el alpino sale de la fortaleza lateral por la red (754 u).',
    },
    necesita: [
      'Una red de caminos por mapa (nodos y aristas) hecha a mano o extraída del juego. La API no la da; que se pueda extraer es UNVERIFIED.',
      'Una velocidad de carrera de referencia (UNVERIFIED): decide qué tramos son saltos.',
      'Sin llamadas nuevas a la API: se calcula con los claims que ya se guardan, y se sigue rotulando como «ruta inferida» (ADR-005).',
      'Spec o ADR nuevo antes de implementar (AGENTS §2.4): cambia la inferencia de ruta.',
    ],
  };
})(window.LK);
