/* Propuesta D · Incertidumbre: en vez de afirmar un camino, dibuja dónde pudo estar el grupo entre dos claims
   a la velocidad elegida (un prisma espacio-tiempo visto desde arriba: una elipse con los dos claims como focos). */
window.LK = window.LK || {};
(function (LK) {
  'use strict';

  const f1 = (n) => Math.round(n * 10) / 10;
  const TINTA = LK.TINTA, esc = LK.esc, AM = LK.AMBAR, HUECO = 'rgba(10,14,19,.8)', ROJO = '#ee7a70', GRIS = '#c4d0da', CLARO = '#dbe6ee';
  const UMBRAL = { ajustado: 0.6, holgado: 0.25 };
  const NOMBRE = { no: 'No cabe', ajustado: 'Ajustado', holgado: 'Holgado', libre: 'Libre', hueco: 'Hueco', quieto: 'Mismo objetivo', sinpos: 'Sin posición', mapa: 'Cambio de mapa' };
  const PLURAL = { no: ['no cabe', 'no caben'], ajustado: ['ajustado', 'ajustados'], holgado: ['holgado', 'holgados'], libre: ['libre', 'libres'], hueco: ['hueco', 'huecos'] };
  const letra = (k) => String.fromCharCode(65 + k);
  const lim = (n) => Math.min(Math.max(n, 14), LK.L - 14);

  // ¿Cuánto explica el tiempo que hubo? r = distancia / (velocidad × tiempo): >1 no cabe, ≥0,6 ajustado, ≥0,25 holgado, menos libre.
  function evaluar(t, v) {
    if (t.tipo !== 'normal') return { t, cat: t.tipo };
    if (t.a.id === t.b.id) return { t, cat: 'quieto' };
    if (t.hueco) return { t, cat: 'hueco' };
    const p = LK.prisma(t.a.pos, t.b.pos, t.dt, v), r = p.d / p.L;
    const cat = p.imposible ? 'no' : r >= UMBRAL.ajustado ? 'ajustado' : r >= UMBRAL.holgado ? 'holgado' : 'libre';
    return { t, p, r, cat, sobra: p.imposible ? 0 : t.dt - p.d / v };
  }
  const textoPastilla = (e) => (e.cat === 'no' ? `No cabe · faltan ${LK.miles(e.p.falta)} u`
    : e.cat === 'hueco' ? `Hueco · ${LK.dur(e.t.dt)} sin datos` : `${NOMBRE[e.cat]} · sobran ${LK.dur(e.sobra)}`);
  const margen = (e) => (e.cat === 'no' ? `faltan ${LK.miles(e.p.falta)} u` : e.p ? `sobran ${LK.dur(e.sobra)}` : '—');

  const insignia = (v, c, k, i) => {
    const [x, y] = v.P(c.pos), bx = lim(x + 18), by = lim(y - 18);
    const estado = c.elegido ? 'Elegido hoy' : c.ok ? 'Posible' : 'Descartado';
    const rect = (r, a) => `<rect x="${f1(bx - r)}" y="${f1(by - r)}" width="${2 * r}" height="${2 * r}" rx="3" ${a}/>`;
    let s = c.elegido ? rect(11.5, `fill="none" stroke="${AM}" stroke-width="1.5"`) : '';
    s += rect(8.5, c.elegido ? `fill="${AM}" stroke="${TINTA}" stroke-width="1.6"` : `fill="${HUECO}" stroke="${CLARO}" stroke-width="1.8"${c.ok ? '' : ' stroke-dasharray="3 2"'}`);
    s += `<text x="${f1(bx)}" y="${f1(by)}" dy=".35em" text-anchor="middle" font-size="10.5" font-weight="700" fill="${c.elegido ? TINTA : CLARO}">${letra(k)}</text>`;
    return `<g class="mk d" data-n="o${i}${k}"><title>${esc(`${letra(k)} · ${LK.nombre(c.id)} · ${estado}: ${c.why}`)}</title>${s}</g>`;
  };

  // Lo que dibuja un tramo: elipse (o círculo de alcance si no cabe), línea recta y pastilla. La pastilla se devuelve
  // como función (cajas) => texto, para colocarla cuando ya se conocen las etiquetas de nombre.
  function tramo(v, e, o) {
    const t = e.t, p = e.p, a = v.P(t.a.pos), b = v.P(t.b.pos);
    const ids = `data-n="${t.n}" data-i="${t.a.i} ${t.b.i}"`, sel = t.n === o.sel;
    const recta = (x, y, w, col, dash, op) => `<path d="M${f1(a[0])},${f1(a[1])} L${f1(x)},${f1(y)}" fill="none" stroke="${TINTA}" stroke-width="${w + 3.2}" stroke-linecap="round" opacity=".7"/>`
      + `<path d="M${f1(a[0])},${f1(a[1])} L${f1(x)},${f1(y)}" fill="none" stroke="${col}" stroke-width="${w}"${dash ? ` stroke-dasharray="${dash}"` : ''} stroke-linecap="round" opacity="${op}"/>`;
    let prisma = '', linea = '', ancla = [(a[0] + b[0]) / 2, (a[1] + b[1]) / 2], col = AM;
    if (e.cat === 'hueco') {
      linea = recta(b[0], b[1], 2.2, GRIS, '0.1 6', 0.9);
      col = GRIS;
    } else if (e.cat === 'no') {
      const f = p.L / p.d, q = [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f];
      prisma = `<circle cx="${f1(a[0])}" cy="${f1(a[1])}" r="${f1(p.L * v.k)}" fill="none" stroke="${AM}" stroke-width="${sel ? 2.6 : 1.8}" stroke-dasharray="5 5"/>`;
      linea = recta(q[0], q[1], 2.6, AM, '', 1)
        + `<path d="M${f1(q[0])},${f1(q[1])} L${f1(b[0])},${f1(b[1])}" fill="none" stroke="${TINTA}" stroke-width="6.6" stroke-linecap="round" opacity=".7"/>`
        + `<path d="M${f1(q[0])},${f1(q[1])} L${f1(b[0])},${f1(b[1])}" fill="none" stroke="${ROJO}" stroke-width="3.4" stroke-dasharray="0.1 7" stroke-linecap="round"/>`;
      ancla = [(q[0] + b[0]) / 2, (q[1] + b[1]) / 2];
      col = ROJO;
    } else {
      if (o.prisma(e)) {
        const [cx, cy] = v.P([p.cx, p.cy]);
        const fill = e.cat === 'ajustado' ? 0.2 : e.cat === 'holgado' ? 0.13 : 0.07;
        const dash = e.cat === 'holgado' ? ' stroke-dasharray="9 6"' : e.cat === 'libre' ? ' stroke-dasharray="0.1 6" stroke-linecap="round"' : '';
        prisma = `<ellipse cx="${f1(cx)}" cy="${f1(cy)}" rx="${f1(p.rx * v.k)}" ry="${f1(p.ry * v.k)}" transform="rotate(${f1(p.ang)} ${f1(cx)} ${f1(cy)})" fill="rgba(255,191,77,${fill})" stroke="${AM}" stroke-width="${sel ? 2.6 : 1.8}"${dash}/>`;
      }
      linea = e.cat === 'libre' ? recta(b[0], b[1], 2.2, AM, '0.1 6', 0.85) : recta(b[0], b[1], 1.8, AM, '', 0.95);
    }
    let disco = '', caja = null, pastilla = null;
    if (e.cat === 'hueco') {
      disco = LK.disco(ancla[0], ancla[1], '?', { r: 9, hueco: true, dash: '3 2', stroke: GRIS, fs: 10, i: `${t.a.i} ${t.b.i}`, cls: 'd', title: `Entre el paso ${t.a.i} y el ${t.b.i} pasaron ${LK.dur(t.dt)}: no se pinta camino.` });
      caja = { x: ancla[0] - 12, y: ancla[1] - 12, w: 24, h: 24 };
    }
    if (o.pastilla(e)) {
      pastilla = (cajas) => `<g class="d" ${ids}>${LK.pastillaLibre(cajas, ancla[0], ancla[1], textoPastilla(e), { col: e.cat === 'no' ? ROJO : col, txt: e.cat === 'no' ? ROJO : undefined }, e.cat === 'hueco' ? [0, 28] : undefined)}</g>`;
    }
    return { prisma: prisma ? `<g class="d" ${ids} clip-path="url(#cp${o.idx})">${prisma}</g>` : '', linea: `<g class="d" ${ids}>${linea}</g>`, disco, caja, pastilla };
  }

  function marco(m, mf, i, st) {
    const v = LK.vista(mf, LK.extras(m, mf), st.completo);
    const origen = LK.origenHoy(m, mf);
    const cands = LK.candidatos(m, mf).filter((c) => c.pos);
    const evs = m.tramos.filter((t) => t.a.mapa === mf.mapa && t.b.mapa === mf.mapa).map((t) => evaluar(t, st.v));
    const justos = evs.filter((e) => e.p && e.cat !== 'libre').length;
    const nLibres = evs.filter((e) => e.cat === 'libre').length;
    const cajas = [];
    const caja = (x, y, r) => cajas.push({ x: x - r, y: y - r, w: 2 * r, h: 2 * r });
    LK.grupos(mf).forEach((g) => { const [x, y] = v.P(g.pos); caja(x, y, 16); });
    cands.forEach((c) => { const [x, y] = v.P(c.pos); caja(x, y, 14); caja(lim(x + 18), lim(y - 18), 12); });

    const o = {
      idx: i, sel: st.tramo,
      prisma: (e) => e.t.n === st.tramo || st.todas || (e.cat !== 'libre' && justos <= 8),
      pastilla: (e) => e.t.n === st.tramo || e.cat === 'hueco' || (e.p && (e.cat !== 'libre' || nLibres <= 6)),
    };
    const sal = LK.salida(v, mf, origen);
    let prismas = '', lineas = '', alto = '';
    const pastillas = [];
    evs.forEach((e) => {
      if (!e.p && e.cat !== 'hueco') return;
      const r = tramo(v, e, o);
      prismas += r.prisma;
      lineas += r.linea;
      alto += r.disco;
      if (r.caja) cajas.push(r.caja);
      if (r.pastilla) pastillas.push(r.pastilla);
    });
    LK.huerfanos(m, mf).forEach((h) => { const s = LK.marcaSinPos(v, h); lineas += s.bajo; alto += s.alto; });
    LK.grupos(mf).forEach((g) => { alto += LK.discoGrupo(v, g, { ultimo: m.pasos.length }); });
    cands.forEach((c, k) => { alto += insignia(v, c, k, i); });

    const resumen = ['no', 'ajustado', 'holgado', 'libre', 'hueco'].map((c) => [c, evs.filter((e) => e.cat === c).length]).filter(([, n]) => n)
      .map(([c, n]) => `${n} ${PLURAL[c][n === 1 ? 0 : 1]}`).join(' · ');
    const pie = `<span>${LK.textoOrigen(origen)}</span>` + (resumen ? `<span>${resumen}</span>` : '') + LK.bandeja(mf);
    return LK.figura(m, mf, i, v, {
      bajo: prismas + sal.bajo + lineas, alto: (cs) => alto + pastillas.map((f) => f(cs)).join('') + LK.cruces(m, mf, v, cs), cajas, pie,
      ids: origen.id ? [origen.id] : [], aria: `${LK.mapaNombre(mf.mapa)}: ruta inferida como zonas donde pudo estar el grupo`,
    });
  }

  const chipLetra = (c, k) => `<svg class="ico-eq" viewBox="0 0 18 18" width="18" height="18" aria-hidden="true"><rect x="1.5" y="1.5" width="15" height="15" rx="3" fill="${c.elegido ? AM : HUECO}" stroke="${c.elegido ? TINTA : CLARO}" stroke-width="1.4"${c.ok ? '' : ' stroke-dasharray="3 2"'}/>`
    + `<text x="9" y="9.5" dy=".35em" text-anchor="middle" font-size="10" font-weight="700" fill="${c.elegido ? TINTA : CLARO}">${letra(k)}</text></svg>`;

  function extra(m, st) {
    let h = '';
    const evs = m.tramos.map((t) => evaluar(t, st.v));
    if (evs.length) {
      h += LK.tabla('Tramos del recorrido', [{ t: 'Tramo' }, { t: 'En recta (u)', n: 1 }, { t: 'Tiempo', n: 1 }, { t: 'u/s necesarias', n: 1 }, { t: 'Margen', n: 1 }, { t: 'Lectura' }],
        evs.map((e) => {
          const t = e.t, n = `${t.a.i} → ${t.b.i}`;
          return {
            attrs: `data-n="${t.n}" data-i="${t.a.i} ${t.b.i}"`, cls: t.n === st.tramo ? 'activa' : '',
            celdas: [e.p ? `<button type="button" class="fila-btn" data-act="tramo" data-n="${t.n}" data-i="${t.a.i} ${t.b.i}" aria-pressed="${t.n === st.tramo}">${n}</button>` : n,
              t.d === null ? '—' : LK.miles(t.d), LK.dur(t.dt), t.vreq === null ? '—' : t.vreq.toFixed(1), margen(e), NOMBRE[e.cat]],
          };
        }),
        `Con una velocidad máxima de ${st.v} u/s. Pulsa un tramo para ver su elipse, aunque sea libre.`);
    }
    m.marcos.forEach((mf, i) => {
      const cs = LK.candidatos(m, mf);
      if (!cs.length) return;
      h += LK.tabla(`Orígenes posibles del paso 0${m.marcos.length > 1 ? ` · ${LK.mapaNombre(mf.mapa)}` : ''}`, [{ t: 'Origen' }, { t: 'Estado' }, { t: 'Por qué' }],
        cs.map((c, k) => ({ attrs: `data-n="o${i}${k}" tabindex="0"`, celdas: [`${chipLetra(c, k)}<span>${esc(LK.nombre(c.id))}</span>`, c.elegido ? 'Elegido hoy' : c.ok ? 'Posible' : 'Descartado', esc(c.why)] })),
        'Los orígenes salen de la regla de hoy (garri, fortaleza lateral, punto de reaparición). Sin porcentajes: solo se dice si cumple la regla.');
    });
    return h;
  }

  const ico = {
    ajustado: `<ellipse cx="17" cy="9" rx="14.5" ry="5.2" transform="rotate(-14 17 9)" fill="rgba(255,191,77,.2)" stroke="${AM}" stroke-width="1.8"/>`,
    holgado: `<ellipse cx="17" cy="9" rx="14.5" ry="7" fill="rgba(255,191,77,.13)" stroke="${AM}" stroke-width="1.8" stroke-dasharray="6 3.5"/>`,
    no: `<circle cx="9" cy="9" r="7" fill="none" stroke="${AM}" stroke-width="1.6" stroke-dasharray="3 3"/><path d="M16,9 L31,9" stroke="${ROJO}" stroke-width="3" stroke-dasharray="0.1 5" stroke-linecap="round"/>`,
    libre: `<path d="M3,9 L31,9" stroke="${AM}" stroke-width="2.2" stroke-dasharray="0.1 6" stroke-linecap="round"/>`,
    hueco: `<path d="M2,9 L13,9 M21,9 L32,9" stroke="${GRIS}" stroke-width="2.2" stroke-dasharray="0.1 6" stroke-linecap="round"/>${LK.disco(17, 9, '?', { r: 6, hueco: true, dash: '3 2', stroke: GRIS, fs: 9 })}`,
    origen: `<rect x="3" y="3" width="12" height="12" rx="2.5" fill="${AM}" stroke="${TINTA}" stroke-width="1.4"/><text x="9" y="9" dy=".35em" text-anchor="middle" font-size="9" font-weight="700" fill="${TINTA}">A</text>`
      + `<rect x="19" y="3" width="12" height="12" rx="2.5" fill="${HUECO}" stroke="${CLARO}" stroke-width="1.4" stroke-dasharray="3 2"/><text x="25" y="9" dy=".35em" text-anchor="middle" font-size="9" font-weight="700" fill="${CLARO}">B</text>`,
    sinpos: LK.disco(17, 9, '3?', { r: 8, hueco: true, dash: '3 2', stroke: GRIS, fs: 8.5 }),
    cruce: '<rect x="1" y="2" width="32" height="14" rx="7" fill="rgba(10,14,19,.93)" stroke="#9fb3c4" stroke-width="1.2"/><text x="17" y="9" dy=".35em" text-anchor="middle" font-size="8" fill="#f4eadb">viene de</text>',
  };

  LK.P = {
    id: 'd', nombre: 'Incertidumbre', marco, extra,
    cambia: [
      'No afirma un camino: dibuja la zona donde el grupo pudo estar entre dos claims a la velocidad elegida (una elipse con los dos claims como focos). Estrecha, el camino es casi seguro; ancha, hay margen.',
      'Si sobra mucho tiempo no pinta nada que parezca un camino; si no cabe, dice cuánto falta; si hay un hueco de 20 min o más, lo deja en blanco.',
      'El paso 0 deja de ser una sola flecha: muestra los orígenes posibles, cuál elige hoy la regla y por qué descarta los demás.',
    ],
    leer: [
      [ico.ajustado, 'Ajustado: la elipse es todo lo que alcanzó entre dos claims. Cuanto más estrecha, más seguro el camino.'],
      [ico.holgado, 'Holgado: sobra tiempo y hay margen para desviarse (elipse de rayas).'],
      [ico.no, 'No cabe: el círculo es lo que se alcanza a pie; el tramo punteado marca lo que falta.'],
      [ico.libre, 'Libre: sobra tanto tiempo que la elipse sería enorme. Solo línea de puntos y cuánto sobra.'],
      [ico.hueco, 'Hueco de 20 min o más: no se pinta camino.'],
      [ico.origen, 'Letra: origen posible del paso 0. Relleno, el que elige hoy; hueco, alternativa; discontinuo, descartado.'],
      [ico.sinpos, 'Paso sin posición.'],
      [ico.cruce, 'Etiqueta gris: el recorrido entra o sale de este mapa.'],
    ],
    controles: [
      { k: 'v', tipo: 'rango', et: 'Velocidad máxima', min: 8, max: 30, u: ' u/s', ayuda: 'Valor de partida estimado (UNVERIFIED). Cuanto más baja, más tramos se vuelven ajustados o no caben.' },
      { k: 'todas', tipo: 'check', et: 'Dibujar también las elipses de los tramos libres', ayuda: 'Sirve para ver por qué no se dibujan: abarcan mucho más que el propio tramo.' },
    ],
    defecto: { v: LK.V_DEFECTO, todas: false, tramo: null },
    alCaso(m, st) {
      const evs = m.tramos.map((t) => evaluar(t, st.v)).filter((e) => e.p);
      const e = evs.find((x) => x.cat === 'no') || evs.find((x) => x.cat !== 'libre') || evs[0];
      st.tramo = e ? e.t.n : null;
    },
    act: { tramo({ st, el }) { st.tramo = Number(el.dataset.n); } },
    notas: {
      n1: 'Los cinco tramos son «libres»: a 14 u/s sobran entre 2 min 47 s y 7 min 49 s en cada uno, así que cualquier camino razonable vale y la elipse no aporta. La del tramo 1→2, seleccionada, mide 2 870 u de largo.',
      n2: 'Solo el 2→3 no cabe: faltan 215 u a 14 u/s (1 335 u en 80 s). Los otros tres sobran casi 6 min cada uno.',
      n3: 'Sin tramos: aquí solo hay origen. La garri es la elegida (reclamada a las 19:40) y el punto de reaparición queda como alternativa posible.',
      e1: 'Tres tramos no caben (faltan 916, 575 y 170 u) y dos son holgados: el 2→3 con 58 s de sobra y el 4→5 con 23 s. Con 30 u/s ya no hay ninguno imposible.',
      e2: 'De 27 tramos, 23 son libres. Uno no cabe (el 21→22: faltan 793 u), otro es holgado (el 16→17), el 17→18 es un hueco de 45 min que se deja en blanco y el 15→16 repite objetivo.',
      e3: 'No se evalúan el 2→3 ni el 3→4 (el paso 3 no tiene posición) ni el 6→7 (cambio de mapa); los otros tres son libres. En el alpino se elige la fortaleza lateral porque los claims quedan al norte; la garri se descarta porque se reclama a las 21:02, ya después de salir.',
    },
    necesita: [
      'Una velocidad máxima de referencia (UNVERIFIED): decide el tamaño de cada elipse y qué tramos no caben.',
      'Sin llamadas nuevas a la API: salen de los claims que ya se guardan (con su `claimed_at`) y de la distancia entre objetivos.',
      'La elipse dice dónde pudo estar, no dónde estuvo ni adónde irá (ADR-001); sigue rotulada «ruta inferida» (ADR-005).',
      'Spec o ADR nuevo antes de implementar (AGENTS §2.4): cambia qué se dibuja y cómo se explica.',
    ],
  };
})(window.LK);
