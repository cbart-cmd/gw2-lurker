/* Escena compartida de los bocetos: marco cuadrado de mapa con teselas reales, glifos de objetivo,
   etiquetas sin solaparse y la vista «Hoy». Todo se dibuja en px de un lienzo de 820x820. */
window.LK = window.LK || {};
(function (LK) {
  'use strict';

  const L = 820;
  const TINTA = '#0a0e13';
  const f1 = (n) => Math.round(n * 10) / 10;
  LK.L = L;
  LK.TINTA = TINTA;
  LK.AMBAR = '#ffbf4d';
  LK.NARANJA = '#ff9b3d';
  // Las teselas las baja el addon a su carpeta; no se commitean (ADR-011). Sin ellas queda el tablero oscuro.
  LK.TILES = 'file:///C:/Users/Usuario/OneDrive/Documentos/Guild%20Wars%202/addons/gw2_lurker/tiles/';
  LK.COL = { Red: '#d0453b', Green: '#4fa564', Blue: '#3f86d6', Neutral: '#7f8c98' };
  LK.TIPO = { Camp: 'campamento', Tower: 'torre', Keep: 'fortaleza', Castle: 'castillo', Ruins: 'ruinas', Spawn: 'punto de reaparición' };
  LK.ORIGEN = { garri: 'desde su fortaleza', lateral: 'desde su fortaleza lateral', spawn: 'desde el punto de reaparición', desconocido: 'Origen desconocido' };
  LK.esc = (s) => String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/"/g, '&quot;');
  const esc = LK.esc;
  LK.nombre = (id) => {
    const o = LK.OBJ[id];
    return o.t === 'Spawn' ? 'Reaparición' : LK.corto(o.n);
  };
  const choca = (a, b) => !(a.x + a.w < b.x || b.x + b.w < a.x || a.y + a.h < b.y || b.y + b.h < a.y);

  // Puntos que deben caber en el encuadre además de los pasos: origen, candidatos y hubs de waypoint.
  LK.extras = (m, marco) => {
    const pts = [];
    const o = LK.origenHoy(m, marco);
    if (o.pos) pts.push(o.pos);
    LK.candidatos(m, marco).forEach((c) => c.pos && pts.push(c.pos));
    LK.pistas(m, LK.V_DEFECTO).tramos.forEach((t) => t.hub && t.b.mapa === marco.mapa && pts.push(t.hub.pos));
    return pts;
  };

  LK.vista = (marco, extra, completo) => {
    const [[rx0, ry0], [rx1, ry1]] = LK.MAPAS[marco.mapa].rect;
    const W = rx1 - rx0, H = ry1 - ry0;
    let x0, y0, lado;
    if (completo) {
      lado = Math.max(W, H);
      x0 = (rx0 + rx1) / 2 - lado / 2;
      y0 = (ry0 + ry1) / 2 - lado / 2;
    } else {
      const pts = marco.pasos.filter((p) => p.pos).map((p) => p.pos).concat(extra || []);
      const a = [Infinity, Infinity, -Infinity, -Infinity];
      pts.forEach((p) => { a[0] = Math.min(a[0], p[0]); a[1] = Math.min(a[1], p[1]); a[2] = Math.max(a[2], p[0]); a[3] = Math.max(a[3], p[1]); });
      lado = Math.min(Math.max(Math.max(a[2] - a[0], a[3] - a[1]) * 1.3 + 260, 1500), Math.max(W, H));
      const cx = (a[0] + a[2]) / 2, cy = (a[1] + a[3]) / 2;
      x0 = lado >= W ? (rx0 + rx1) / 2 - lado / 2 : Math.min(Math.max(cx - lado / 2, rx0), rx1 - lado);
      y0 = lado >= H ? (ry0 + ry1) / 2 - lado / 2 : Math.min(Math.max(cy - lado / 2, ry0), ry1 - lado);
    }
    const k = L / lado;
    return { mapa: marco.mapa, x0, y0, lado, k, rect: [[rx0, ry0], [rx1, ry1]], P: (p) => [(p[0] - x0) * k, (p[1] - y0) * k] };
  };

  const teselas = (v) => {
    const [[rx0, ry0], [rx1, ry1]] = v.rect;
    let s = '';
    for (let ty = Math.floor(ry0 / 1024); ty <= Math.floor((ry1 - 1) / 1024); ty++) {
      for (let tx = Math.floor(rx0 / 1024); tx <= Math.floor((rx1 - 1) / 1024); tx++) {
        const [x, y] = v.P([tx * 1024, ty * 1024]), w = 1024 * v.k;
        if (x > L || y > L || x + w < 0 || y + w < 0) continue;
        s += `<image href="${LK.TILES}2-1-5-${tx}-${ty}.jpg" x="${f1(x)}" y="${f1(y)}" width="${f1(w + 0.8)}" height="${f1(w + 0.8)}" preserveAspectRatio="none"/>`;
      }
    }
    return s;
  };

  const rejilla = (v) => {
    let s = '';
    for (let gx = Math.ceil(v.x0 / 512) * 512; gx < v.x0 + v.lado; gx += 512) s += `<line x1="${f1(v.P([gx, 0])[0])}" y1="0" x2="${f1(v.P([gx, 0])[0])}" y2="${L}"/>`;
    for (let gy = Math.ceil(v.y0 / 512) * 512; gy < v.y0 + v.lado; gy += 512) s += `<line x1="0" y1="${f1(v.P([0, gy])[1])}" x2="${L}" y2="${f1(v.P([0, gy])[1])}"/>`;
    return `<g stroke="#18232e" stroke-width="1">${s}</g>`;
  };

  LK.glifo = (tipo, x, y, r, col, fo) => {
    const st = `fill="${col}" fill-opacity="${fo}" stroke="${TINTA}" stroke-width="1.6"`;
    x = f1(x); y = f1(y);
    switch (tipo) {
      case 'Camp': return `<circle cx="${x}" cy="${y}" r="${r}" ${st}/>`;
      case 'Tower': return `<rect x="${f1(x - r)}" y="${f1(y - r)}" width="${2 * r}" height="${2 * r}" rx="2" ${st}/>`;
      case 'Keep': { const q = f1(r * 1.3); return `<polygon points="${x},${f1(y - q)} ${f1(x + q)},${y} ${x},${f1(y + q)} ${f1(x - q)},${y}" ${st}/>`; }
      case 'Castle': {
        let p = '';
        for (let i = 0; i < 8; i++) { const a = (Math.PI / 4) * i + Math.PI / 8; p += `${f1(x + r * 1.4 * Math.cos(a))},${f1(y + r * 1.4 * Math.sin(a))} `; }
        return `<polygon points="${p}" ${st}/>`;
      }
      case 'Ruins': return `<polygon points="${x},${f1(y - r)} ${f1(x + r * 0.95)},${f1(y + r * 0.7)} ${f1(x - r * 0.95)},${f1(y + r * 0.7)}" fill="none" stroke="#8294a5" stroke-opacity="${fo}" stroke-width="1.4"/>`;
      case 'Spawn': return `<path d="M${f1(x - 4)},${f1(y + r)} V${f1(y - r)} L${f1(x + r)},${f1(y - r * 0.45)} L${f1(x - 4)},${f1(y + r * 0.1)}" ${st} stroke-linejoin="round"/>`;
      default: return '';
    }
  };

  LK.involucrados = (m, marco, extra) => new Set([...marco.pasos.filter((p) => !p.sinPos).map((p) => p.id), ...m.vigentes.filter((x) => x.o.m === marco.mapa).map((x) => x.id), ...(extra || [])]);
  LK.reclamados = (m, marco) => new Set([...marco.pasos.filter((p) => !p.sinPos).map((p) => p.id), ...m.vigentes.filter((x) => x.o.m === marco.mapa).map((x) => x.id)]);

  const capaObjetivos = (m, v, inv, rec, ocultos) => {
    let svg = '';
    const cajas = [];
    for (const [id, o] of Object.entries(LK.OBJ)) {
      if (o.m !== v.mapa || o.t === 'Mercenary' || ocultos.has(id)) continue;
      const p = o.c || (o.t === 'Spawn' ? o.l : null);
      if (!p) continue;
      const [x, y] = v.P(p);
      if (x < -20 || y < -20 || x > L + 20 || y > L + 20) continue;
      const on = inv.has(id), r = on ? 11 : 8;
      const col = LK.COL[LK.dueno(m, id)] || LK.COL.Neutral;
      svg += `<g class="obj${on ? ' on' : ''}"><title>${esc(o.n)} · ${LK.TIPO[o.t]}</title>`;
      if (rec.has(id)) svg += `<circle cx="${f1(x)}" cy="${f1(y)}" r="${f1(r * 1.6)}" fill="none" stroke="${LK.AMBAR}" stroke-width="2"/>`;
      svg += `${LK.glifo(o.t, x, y, r, col, on ? 0.95 : 0.5)}</g>`;
      cajas.push({ x: x - r * 1.6, y: y - r * 1.6, w: r * 3.2, h: r * 3.2 });
    }
    return { svg, cajas };
  };

  const etiquetas = (v, ids, cajas) => {
    let svg = '';
    ids.forEach((id) => {
      const o = LK.OBJ[id], p = o && (o.c || (o.t === 'Spawn' ? o.l : null));
      if (!p) return;
      const [x, y] = v.P(p);
      const txt = LK.nombre(id);
      const w = txt.length * 6 + 8, h = 14, r = 20;
      const op = [
        { a: 'start', tx: x + r, ty: y + 4, bx: x + r - 2, by: y - 8 },
        { a: 'end', tx: x - r, ty: y + 4, bx: x - r + 2 - w, by: y - 8 },
        { a: 'middle', tx: x, ty: y - r - 3, bx: x - w / 2, by: y - r - 14 },
        { a: 'middle', tx: x, ty: y + r + 12, bx: x + 0 - w / 2, by: y + r },
      ];
      const dentro = (q) => q.bx > 3 && q.bx + w < L - 3 && q.by > 3 && q.by + h < L - 3;
      const pick = op.find((q) => dentro(q) && !cajas.some((c) => choca(c, { x: q.bx, y: q.by, w, h }))) || op.find(dentro) || op[0];
      cajas.push({ x: pick.bx, y: pick.by, w, h });
      svg += `<text x="${f1(pick.tx)}" y="${f1(pick.ty)}" text-anchor="${pick.a}">${esc(txt)}</text>`;
    });
    return svg;
  };

  // c: { bajo, alto, sobre, pie, ids (extra a destacar), aria, etiquetar (por defecto, todo lo destacado), cajas }
  // `cajas` son las {x,y,w,h} que ya ocupa la propuesta (chips, etc.): las etiquetas de nombre las esquivan.
  // `alto` y `sobre` pueden ser una función (cajas) => texto SVG: se evalúa con las cajas finales (objetivos, cajas
  // de la propuesta y etiquetas ya colocadas), para que las pastillas no pisen nada; la función puede añadir las suyas.
  LK.figura = (m, marco, idx, v, c) => {
    const ocultos = new Set(m.pasos.filter((p) => p.sinPos).map((p) => p.id));
    const inv = LK.involucrados(m, marco, c.ids);
    const obj = capaObjetivos(m, v, inv, LK.reclamados(m, marco), ocultos);
    const orden = [...inv].filter((id) => !ocultos.has(id));
    const cajas = obj.cajas.concat(c.cajas || []);
    const etq = etiquetas(v, c.etiquetar || orden, cajas);
    const capa = (x) => (typeof x === 'function' ? x(cajas) : x || '');
    const alto = capa(c.alto), sobre = capa(c.sobre);
    const [[rx0, ry0], [rx1, ry1]] = v.rect;
    const [ax, ay] = v.P([rx0, ry0]), [bx, by] = v.P([rx1, ry1]);
    const area = `x="${f1(ax)}" y="${f1(ay)}" width="${f1(bx - ax)}" height="${f1(by - ay)}"`;
    const nombre = LK.MAPAS[marco.mapa].nombre;
    return `<figure class="marco" data-mapa="${marco.mapa}">
  <figcaption class="cab"><span class="mapa">${nombre}</span>${c.cab || ''}<span class="chip">Ruta inferida</span></figcaption>
  <div class="lienzo"><svg viewBox="0 0 ${L} ${L}" role="img" aria-label="${esc(c.aria || nombre)}">
    <defs><clipPath id="cp${idx}"><rect ${area}/></clipPath>${c.defs || ''}</defs>
    <rect width="${L}" height="${L}" fill="#05080b"/><rect ${area} fill="#0c1218"/>${rejilla(v)}
    <g clip-path="url(#cp${idx})">${teselas(v)}</g>
    <rect width="${L}" height="${L}" fill="#070a0e" opacity=".42"/><rect ${area} fill="none" stroke="#3a4f62" stroke-width="1.2"/>
    <g class="bajo">${c.bajo || ''}</g>${obj.svg}<g class="alto">${alto}</g>
    <g class="etq">${etq}</g><g class="sobre">${sobre}</g>
  </svg></div>${c.pie ? `<div class="pie">${c.pie}</div>` : ''}
</figure>`;
  };

  // Primitivas de dibujo
  LK.disco = (x, y, txt, o = {}) => {
    const r = o.r || 9, hueco = !!o.hueco;
    const fill = hueco ? 'rgba(10,14,19,.8)' : (o.fill || LK.AMBAR);
    const stroke = hueco ? (o.stroke || LK.AMBAR) : TINTA;
    const tcol = hueco ? (o.stroke || LK.AMBAR) : (o.txt || TINTA);
    return `<g class="mk${o.cls ? ' ' + o.cls : ''}"${o.i ? ` data-i="${o.i}"` : ''}>${o.title ? `<title>${esc(o.title)}</title>` : ''}`
      + `<circle cx="${f1(x)}" cy="${f1(y)}" r="${r}" fill="${fill}" stroke="${stroke}" stroke-width="${hueco ? 1.8 : 1.6}"${o.dash ? ` stroke-dasharray="${o.dash}"` : ''}/>`
      + `<text x="${f1(x)}" y="${f1(y)}" dy=".35em" text-anchor="middle" font-size="${o.fs || 10.5}" font-weight="700" fill="${tcol}">${txt}</text></g>`;
  };
  LK.flecha = (x, y, ang, s, col, extra) => `<path d="M${-s},${f1(-s * 0.62)} L${s},0 L${-s},${f1(s * 0.62)} Z" transform="translate(${f1(x)} ${f1(y)}) rotate(${f1(ang)})" fill="${col}" ${extra || ''}/>`;
  LK.pastilla = (x, y, txt, o = {}) => {
    const w = txt.length * 5.8 + 14;
    return `<g class="pastilla${o.cls ? ' ' + o.cls : ''}"${o.i ? ` data-i="${o.i}"` : ''}><rect x="${f1(x - w / 2)}" y="${f1(y - 9)}" width="${f1(w)}" height="18" rx="9" fill="rgba(10,14,19,.93)" stroke="${o.col || LK.AMBAR}" stroke-width="1.2"/>`
      + `<text x="${f1(x)}" y="${f1(y)}" dy=".35em" text-anchor="middle" font-size="10.5" font-weight="600" fill="${o.txt || '#f4eadb'}">${esc(txt)}</text></g>`;
  };
  LK.pts = (arr) => arr.map((p) => `${f1(p[0])},${f1(p[1])}`).join(' ');
  LK.ico = (inner) => `<svg class="ico" viewBox="0 0 34 18" width="34" height="18" aria-hidden="true">${inner}</svg>`;
  LK.angulo = (a, b) => (Math.atan2(b[1] - a[1], b[0] - a[0]) * 180) / Math.PI;

  // Vista «Hoy»: lo que dibuja el addon ahora (rectas, 0 hueco, anillo en el último, sin pasos sin coordenada).
  LK.hoy = (m, marco, idx, st) => {
    const origen = LK.origenHoy(m, marco);
    const v = LK.vista(marco, LK.extras(m, marco), st.completo);
    const loc = marco.pasos.filter((p) => p.pos);
    const num = new Map(marco.pasos.map((p, k) => [p.i, k + 1]));
    const pts = loc.map((p) => v.P(p.pos));
    const o = origen.pos ? v.P(origen.pos) : null;
    let bajo = '', alto = '';
    if (o && pts[0]) bajo += `<line x1="${f1(o[0])}" y1="${f1(o[1])}" x2="${f1(pts[0][0])}" y2="${f1(pts[0][1])}" stroke="${LK.AMBAR}" stroke-width="1.6" stroke-dasharray="5 4" opacity=".85"/>`;
    if (pts.length > 1) {
      bajo += `<polyline class="traza" pathLength="1" points="${LK.pts(pts)}" fill="none" stroke="${LK.AMBAR}" stroke-width="2" stroke-linejoin="round"/>`;
      const a = pts[pts.length - 2], b = pts[pts.length - 1], ang = LK.angulo(a, b), d = Math.hypot(b[0] - a[0], b[1] - a[1]);
      if (d > 34) alto += LK.flecha(b[0] - Math.cos((ang * Math.PI) / 180) * 17, b[1] - Math.sin((ang * Math.PI) / 180) * 17, ang, 6, LK.AMBAR);
    }
    if (pts.length) alto += `<circle cx="${f1(pts[pts.length - 1][0])}" cy="${f1(pts[pts.length - 1][1])}" r="13.5" fill="none" stroke="${LK.NARANJA}" stroke-width="2"/>`;
    loc.forEach((p, k) => { alto += LK.disco(pts[k][0], pts[k][1], num.get(p.i), { r: 8.5, title: `${p.o.n} · ${p.hora}` }); });
    if (o) alto += `<circle cx="${f1(o[0])}" cy="${f1(o[1])}" r="12" fill="none" stroke="${LK.AMBAR}" stroke-opacity=".7" stroke-width="1"/>` + LK.disco(o[0], o[1], '0', { r: 8, hueco: true });
    const pie = `${LK.ORIGEN[origen.tipo]}${origen.aprox ? ' · posición aproximada' : ''} · ${marco.pasos.length} ${marco.pasos.length === 1 ? 'paso' : 'pasos'}`;
    return LK.figura(m, marco, idx, v, { bajo, alto, pie, ids: origen.id ? [origen.id] : [], aria: `${LK.MAPAS[marco.mapa].nombre}: ruta inferida de hoy` });
  };
})(window.LK);
