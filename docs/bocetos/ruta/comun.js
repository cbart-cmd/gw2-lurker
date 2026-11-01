/* Piezas que comparten las cuatro propuestas: pasos agrupados por objetivo, cruces de mapa,
   pasos sin posición, la salida (paso 0) y las tablas. Solo construyen texto SVG/HTML. */
window.LK = window.LK || {};
(function (LK) {
  'use strict';

  const f1 = (n) => Math.round(n * 10) / 10;
  const esc = LK.esc;
  LK.mapaNombre = (id) => LK.MAPAS[id].nombre;

  // Los pasos con posición de un marco, agrupados por objetivo: las revisitas comparten disco.
  LK.grupos = (mf) => {
    const g = new Map();
    mf.pasos.filter((p) => p.pos).forEach((p) => {
      if (!g.has(p.id)) g.set(p.id, { id: p.id, pos: p.pos, pasos: [] });
      g.get(p.id).pasos.push(p);
    });
    return [...g.values()];
  };
  LK.titulo = (g) => `${g.pasos[0].o.n}: ${g.pasos.map((p) => `paso ${p.i} a las ${p.hora}`).join(', ')}`;

  // Disco numerado de un grupo. o.ultimo = nº de paso que lleva el anillo naranja; o.disco = opciones de LK.disco;
  // o.marca = otra función de dibujo con la misma firma que LK.disco (la propuesta C pinta formas distintas).
  LK.discoGrupo = (v, g, o = {}) => {
    const [x, y] = v.P(g.pos), r = (o.disco && o.disco.r) || 9;
    let s = '';
    if (o.ultimo && g.pasos.some((p) => p.i === o.ultimo)) s += `<circle cx="${f1(x)}" cy="${f1(y)}" r="${r + 4.5}" fill="none" stroke="${LK.NARANJA}" stroke-width="2"/>`;
    s += (o.marca || LK.disco)(x, y, g.pasos[0].i, Object.assign({ r, i: g.pasos.map((p) => p.i).join(' '), title: LK.titulo(g), cls: 'd' }, o.disco));
    if (g.pasos.length > 1) {
      const t = `×${g.pasos.length}`;
      s += `<g class="ins" pointer-events="none"><rect x="${f1(x + 5)}" y="${f1(y - r - 10)}" width="${t.length * 6 + 6}" height="13" rx="6.5" fill="${LK.TINTA}" stroke="${LK.AMBAR}" stroke-width="1"/>`
        + `<text x="${f1(x + 8 + t.length * 3)}" y="${f1(y - r - 3.5)}" text-anchor="middle" font-size="10" font-weight="700" fill="#f4eadb">${t}</text></g>`;
    }
    return s;
  };

  const dentro = (n, a, b) => Math.min(Math.max(n, a), b);
  // Etiquetas «viene de» / «sigue en» en cada punto donde el recorrido sale de este mapa o vuelve a él.
  // Con `cajas` (las ya ocupadas) buscan hueco; sin ellas se quedan en el desplazamiento fijo.
  LK.cruces = (m, mf, v, cajas) => {
    let s = '';
    const loc = mf.pasos.filter((p) => p.pos);
    const eti = (a, dy, txt) => {
      const [x, y] = v.P(a.pos);
      const px = dentro(x, 100, LK.L - 100), py = dentro(y + dy, 18, LK.L - 18);
      return cajas ? LK.pastillaLibre(cajas, px, py, txt, { col: '#9fb3c4' }) : LK.pastilla(px, py, txt, { col: '#9fb3c4' });
    };
    m.pasos.forEach((p, k) => {
      const n = m.pasos[k + 1];
      if (!n || p.mapa === n.mapa) return;
      if (p.mapa === mf.mapa) {
        const a = loc.filter((q) => q.i <= p.i).pop();
        if (a) s += eti(a, 32, `sigue en ${LK.mapaNombre(n.mapa)} · ${LK.hm(n.t)}`);
      } else if (n.mapa === mf.mapa) {
        const a = loc.find((q) => q.i >= n.i);
        if (a) s += eti(a, -32, `viene de ${LK.mapaNombre(p.mapa)} · ${LK.hm(p.t)}`);
      }
    });
    return s;
  };

  // Pastilla que busca hueco: prueba desplazamientos cada vez más lejos del preferido (por defecto, ninguno) hasta
  // no pisar nada de `cajas` ({x,y,w,h}), y anota su propia caja. Si no hay hueco, elige el que menos pisa.
  const choca = (a, b) => !(a.x + a.w < b.x || b.x + b.w < a.x || a.y + a.h < b.y || b.y + b.h < a.y);
  const solape = (a, b) => Math.max(0, Math.min(a.x + a.w, b.x + b.w) - Math.max(a.x, b.x)) * Math.max(0, Math.min(a.y + a.h, b.y + b.h) - Math.max(a.y, b.y));
  LK.pastillaLibre = (cajas, x, y, txt, o = {}, pref) => {
    const w = txt.length * 5.8 + 14, h = 18, p0 = pref || [0, 0];
    const sx = w / 2 + 14;
    const ops = [p0];
    [0, -22, 22, -44, 44, -66, 66, -88, 88].forEach((dy) => [0, -sx, sx, -2 * sx, 2 * sx].forEach((dx) => ops.push([dx, dy])));
    ops.sort((a, b) => Math.hypot(a[0] - p0[0], a[1] - p0[1]) - Math.hypot(b[0] - p0[0], b[1] - p0[1]));
    const caja = (q) => ({ x: x + q[0] - w / 2, y: y + q[1] - h / 2, w, h });
    const dentroMapa = (c) => c.x > 3 && c.x + c.w < LK.L - 3 && c.y > 3 && c.y + c.h < LK.L - 3;
    const libre = (c) => dentroMapa(c) && !cajas.some((b) => choca(b, c));
    const coste = (c) => cajas.reduce((s, b) => s + solape(b, c), 0) + (dentroMapa(c) ? 0 : 1e6);
    let q = ops.find((p) => libre(caja(p)));
    if (!q) q = ops.map((p) => [coste(caja(p)), p]).reduce((mejor, c) => (c[0] < mejor[0] ? c : mejor))[1];
    cajas.push(caja(q));
    return LK.pastilla(x + q[0], y + q[1], txt, o);
  };

  // Pasos sin posición (el catálogo no trae coordenada): vecinos con posición en el mismo mapa.
  LK.huerfanos = (m, mf) => mf.pasos.filter((p) => p.sinPos).map((p) => {
    let prev = null, next = null;
    for (let k = p.i - 2; k >= 0 && !prev; k--) if (m.pasos[k].mapa === mf.mapa && m.pasos[k].pos) prev = m.pasos[k];
    for (let k = p.i; k < m.pasos.length && !next; k++) if (m.pasos[k].mapa === mf.mapa && m.pasos[k].pos) next = m.pasos[k];
    return { p, prev, next };
  });
  LK.marcaSinPos = (v, h, col) => {
    if (!h.prev || !h.next) return { bajo: '', alto: '' };
    const a = v.P(h.prev.pos), b = v.P(h.next.pos), x = (a[0] + b[0]) / 2, y = (a[1] + b[1]) / 2;
    return {
      bajo: `<polyline points="${LK.pts([a, [x, y], b])}" fill="none" stroke="${col || '#c4d0da'}" stroke-width="2.4" stroke-dasharray="1 6" stroke-linecap="round" opacity=".9"/>`,
      alto: LK.disco(x, y, `${h.p.i}?`, { r: 11, hueco: true, dash: '3 2', stroke: col || '#c4d0da', fs: 9.5, i: h.p.i, cls: 'd', title: `Paso ${h.p.i}: ${h.p.o.n} (${h.p.hora}). El catálogo no trae su posición.` }),
    };
  };
  LK.bandeja = (mf) => {
    const sp = mf.pasos.filter((p) => p.sinPos);
    return sp.length ? `<span class="bandeja"><b>Sin posición</b> ${sp.map((p) => `<span data-i="${p.i}" tabindex="0">${p.i} · ${esc(p.o.n)}</span>`).join('')}</span>` : '';
  };

  // El paso 0: salida inferida hasta el primer claim, recta o por la red de caminos (o.red).
  LK.salida = (v, mf, origen, o = {}) => {
    const primero = mf.pasos.find((p) => p.pos);
    if (!origen.pos || !primero) return { bajo: '', alto: '' };
    const a = v.P(origen.pos), b = v.P(primero.pos);
    let trazo = `M${f1(a[0])},${f1(a[1])} L${f1(b[0])},${f1(b[1])}`;
    if (o.red && origen.id !== primero.id) {
      const c = LK.camino(mf.mapa, origen.id, primero.id);
      if (c && c.tramos.length) {
        let pts = [];
        c.tramos.forEach((t) => {
          const s = LK.chaikin(LK.sentido(t).map((p) => v.P(p)));
          pts = pts.concat(pts.length ? s.slice(1) : s);
        });
        trazo = 'M' + pts.map((p) => `${f1(p[0])},${f1(p[1])}`).join(' L');
      }
    }
    const col = o.col || LK.AMBAR;
    return {
      bajo: `<path d="${trazo}" fill="none" stroke="${col}" stroke-width="${o.w || 2.4}" stroke-dasharray="7 5" stroke-linecap="round" stroke-opacity=".9"/>`,
      alto: `<circle cx="${f1(a[0])}" cy="${f1(a[1])}" r="13" fill="none" stroke="${col}" stroke-opacity=".6" stroke-width="1"/>`
        + LK.disco(a[0], a[1], '0', { r: 8.5, hueco: true, stroke: col, title: `Paso 0: ${LK.ORIGEN[origen.tipo]} (inferido)` }),
    };
  };
  LK.textoOrigen = (origen) => (origen.tipo === 'desconocido' ? 'Origen desconocido' : `Salida inferida ${LK.ORIGEN[origen.tipo]}${origen.aprox ? ' (posición aproximada)' : ''}`);

  // Tabla de datos. cols: [{t, n}] (n: columna numérica); filas: [{celdas[], attrs, cls}]; la primera celda es el encabezado de fila.
  LK.tabla = (titulo, cols, filas, nota) => `<section class="tabla-caja"><h2>${titulo}</h2>${nota ? `<p class="nota">${nota}</p>` : ''}<div class="tabla-env"><table class="tabla"><thead><tr>${cols.map((c) => `<th scope="col"${c.n ? ' class="num"' : ''}>${c.t}</th>`).join('')}</tr></thead><tbody>`
    + filas.map((f) => `<tr${f.attrs ? ' ' + f.attrs : ''}${f.cls ? ` class="${f.cls}"` : ''}>${f.celdas.map((c, k) => (k === 0 ? `<th scope="row">${c}</th>` : `<td${cols[k].n ? ' class="num"' : ''}>${c}</td>`)).join('')}</tr>`).join('')
    + '</tbody></table></div></section>';
})(window.LK);
