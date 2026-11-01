/* Lógica pura de los bocetos (sin DOM, probable con Node): tiempo, modelo del caso, regla de origen
   de hoy (crates/core/src/route.rs, locate_origin), red de caminos, escuadras y prisma espacio-tiempo.
   Todas las coordenadas son de continente (la Y crece hacia el sur). */
window.LK = window.LK || {};
(function (LK) {
  'use strict';

  // UNVERIFIED: velocidad de carrera a pie (333 pulgadas/s del juego ÷ 24 pulgadas por unidad) y carga del waypoint.
  LK.V_DEFECTO = 14;
  LK.WP_ESPERA = 20;
  LK.HUECO = 20 * 60;
  LK.ALCANCE_MAX = 5000;
  LK.EQUIPO = { Red: 'Rojo', Green: 'Verde', Blue: 'Azul' };
  LK.HOGAR = { Red: 1099, Green: 95, Blue: 96 };
  LK.GARRI = { 95: '95-37', 96: '96-37', 1099: '1099-113' };
  LK.LATERALES = { 95: ['95-32', '95-33'], 96: ['96-32', '96-33'] };

  const dos = (n) => String(n).padStart(2, '0');
  LK.seg = (hms) => { const p = hms.split(':').map(Number); return p[0] * 3600 + p[1] * 60 + (p[2] || 0); };
  LK.hms = (s) => `${dos(Math.floor(s / 3600) % 24)}:${dos(Math.floor(s / 60) % 60)}:${dos(Math.floor(s) % 60)}`;
  LK.hm = (s) => `${dos(Math.floor(s / 3600) % 24)}:${dos(Math.floor(s / 60) % 60)}`;
  LK.dur = (s) => {
    s = Math.round(s);
    if (s < 60) return `${s} s`;
    const m = Math.floor(s / 60), r = s % 60;
    if (m < 60) return r ? `${m} min ${r} s` : `${m} min`;
    const h = Math.floor(m / 60), mm = m % 60;
    return mm ? `${h} h ${dos(mm)} min` : `${h} h`;
  };
  LK.miles = (n) => String(Math.round(n)).replace(/\B(?=(\d{3})+(?!\d))/g, '\u00a0');
  LK.dist = (a, b) => Math.hypot(a[0] - b[0], a[1] - b[1]);
  LK.pos = (id) => { const o = LK.OBJ[id]; return o ? (o.c || o.l || null) : null; };
  LK.corto = (n) => (n.length <= 14 ? n : n.split(' ')[0]);

  LK.spawnPropio = (equipo, mapa) => {
    const e = Object.entries(LK.OBJ).find(([, o]) => o.m === mapa && o.t === 'Spawn' && o.o === equipo);
    return e ? { id: e[0], o: e[1], pos: e[1].c || e[1].l } : null;
  };

  // Un caso listo para pintar: pasos numerados, marcos por mapa (orden de aparición) y tramos consecutivos.
  LK.modelo = (sc) => {
    const pasos = sc.pasos.map((p, k) => {
      const o = LK.OBJ[p.id];
      const pos = p.sinPos || !o.c ? null : o.c;
      return { i: k + 1, id: p.id, o, mapa: o.m, t: LK.seg(p.t), hora: p.t, pos, sinPos: !pos };
    });
    const orden = [];
    pasos.forEach((p) => { if (!orden.includes(p.mapa)) orden.push(p.mapa); });
    const m = {
      sc, equipo: sc.equipo, pasos,
      marcos: orden.map((mapa) => ({ mapa, pasos: pasos.filter((p) => p.mapa === mapa) })),
      vigentes: (sc.vigentes || []).map((v) => ({ id: v.id, o: LK.OBJ[v.id], t: LK.seg(v.t), hora: v.t })),
      propias: sc.propias || [],
      t0: pasos[0].t, t1: pasos[pasos.length - 1].t,
    };
    m.tramos = pasos.slice(1).map((b, k) => {
      const a = pasos[k], dt = b.t - a.t;
      let tipo = 'normal', d = null;
      if (!a.pos || !b.pos) tipo = 'sinpos';
      else if (a.mapa !== b.mapa) tipo = 'mapa';
      else d = LK.dist(a.pos, b.pos);
      return { n: k + 1, a, b, dt, d, tipo, hueco: dt >= LK.HUECO, vreq: d === null ? null : d / Math.max(dt, 1) };
    });
    return m;
  };

  // Dueño que se pinta: lo que el equipo reclama o posee en el caso es suyo; el resto, la captura real.
  LK.dueno = (m, id) => {
    const mio = m.pasos.some((p) => p.id === id) || m.vigentes.some((v) => v.id === id) || m.propias.includes(id);
    return mio ? m.equipo : ((LK.OBJ[id] && LK.OBJ[id].o) || 'Neutral');
  };

  // Regla de origen de hoy: garri (mapa propio) > fortaleza lateral (alpinos) > spawn aproximado > desconocido.
  LK.origenHoy = (m, marco) => {
    const loc = marco.pasos.filter((p) => p.pos);
    if (!loc.length) return { tipo: 'desconocido' };
    const first = loc[0], last = loc[loc.length - 1];
    const antes = (id) => LK.OBJ[id].t === 'Keep' && [...m.vigentes, ...m.pasos].some((x) => x.id === id && x.t <= first.t);
    const garri = LK.GARRI[marco.mapa];
    if (LK.HOGAR[m.equipo] === marco.mapa && garri && antes(garri)) return { tipo: 'garri', id: garri, pos: LK.pos(garri) };
    let mejor = null;
    for (const id of LK.LATERALES[marco.mapa] || []) {
      const p = LK.pos(id);
      if (!p || !antes(id) || !(first.pos[1] < p[1] && last.pos[1] < p[1])) continue;
      const d = LK.dist(first.pos, p);
      if (!mejor || d < mejor.d) mejor = { tipo: 'lateral', id, pos: p, d };
    }
    if (mejor) return mejor;
    const sp = LK.spawnPropio(m.equipo, marco.mapa);
    return sp ? { tipo: 'spawn', id: sp.id, pos: sp.pos, aprox: true } : { tipo: 'desconocido' };
  };

  // Todos los orígenes que la regla podría haber elegido, con el motivo de aceptar o descartar cada uno.
  LK.candidatos = (m, marco) => {
    const loc = marco.pasos.filter((p) => p.pos);
    if (!loc.length) return [];
    const first = loc[0], last = loc[loc.length - 1];
    const elegido = LK.origenHoy(m, marco);
    const claim = (id) => [...m.vigentes, ...m.pasos].find((x) => x.id === id);
    const out = [];
    const garri = LK.GARRI[marco.mapa];
    if (garri) {
      const c = claim(garri);
      let why = 'El clan no la ha reclamado';
      let ok = false;
      if (LK.HOGAR[m.equipo] !== marco.mapa) why = 'No es el mapa propio del equipo';
      else if (c && c.t > first.t) why = `Reclamada después (${LK.hm(c.t)}), no pudo ser la salida`;
      else if (c) { ok = true; why = `Reclamada antes (${LK.hm(c.t)}) y aún en pie`; }
      out.push({ id: garri, tipo: 'garri', pos: LK.pos(garri), ok, why });
    }
    for (const id of LK.LATERALES[marco.mapa] || []) {
      const c = claim(id), p = LK.pos(id);
      let why = 'El clan no la ha reclamado';
      let ok = false;
      if (c && c.t > first.t) why = `Reclamada después (${LK.hm(c.t)})`;
      else if (c && !(first.pos[1] < p[1] && last.pos[1] < p[1])) why = 'Los claims no quedan al norte de ella';
      else if (c) { ok = true; why = `Reclamada antes (${LK.hm(c.t)}) y los claims quedan al norte`; }
      out.push({ id, tipo: 'lateral', pos: p, ok, why });
    }
    const sp = LK.spawnPropio(m.equipo, marco.mapa);
    if (sp) out.push({ id: sp.id, tipo: 'spawn', pos: sp.pos, ok: true, aprox: true, why: 'Punto de reaparición propio (posición aproximada)' });
    out.forEach((c) => { c.elegido = c.id === elegido.id; });
    // Si ninguna fortaleza gana, el spawn es el elegido; si gana una, el spawn queda como alternativa.
    return out.sort((a, b) => (b.elegido - a.elegido) || (b.ok - a.ok));
  };

  // --- Red de caminos (boceto A) ---
  const redes = {};
  LK.red = (mapa) => {
    if (redes[mapa]) return redes[mapa];
    const nodos = {}, aristas = [], ady = {};
    const lineas = (LK.CAMINOS[mapa] || '').split('\n').map((l) => l.trim()).filter(Boolean);
    for (const l of lineas) {
      const p = l.split(/\s+/);
      if (p[0] === 'nodo') nodos[p[1]] = [+p[2], +p[3]];
    }
    const pos = (id) => nodos[id] || LK.pos(id);
    for (const l of lineas) {
      const p = l.split(/\s+/);
      if (p[0] === 'nodo') continue;
      const via = p.slice(2).map((s) => s.split(',').map(Number));
      const e = { a: p[0], b: p[1] };
      e.pts = [pos(e.a), ...via, pos(e.b)];
      e.len = 0;
      for (let i = 1; i < e.pts.length; i++) e.len += LK.dist(e.pts[i - 1], e.pts[i]);
      e.k = e.a < e.b ? `${e.a}|${e.b}` : `${e.b}|${e.a}`;
      aristas.push(e);
      (ady[e.a] = ady[e.a] || []).push({ to: e.b, e, dir: 1 });
      (ady[e.b] = ady[e.b] || []).push({ to: e.a, e, dir: -1 });
    }
    return (redes[mapa] = { nodos, aristas, ady, pos });
  };

  LK.camino = (mapa, a, b) => {
    const r = LK.red(mapa);
    if (a === b) return { tramos: [], len: 0 };
    if (!r.ady[a] || !r.ady[b]) return null;
    const dist = { [a]: 0 }, prev = {}, visto = new Set();
    for (;;) {
      let u = null;
      for (const k in dist) if (!visto.has(k) && (u === null || dist[k] < dist[u])) u = k;
      if (u === null) return null;
      if (u === b) break;
      visto.add(u);
      for (const { to, e, dir } of r.ady[u]) {
        const nd = dist[u] + e.len;
        if (dist[to] === undefined || nd < dist[to]) { dist[to] = nd; prev[to] = { de: u, e, dir }; }
      }
    }
    const tramos = [];
    for (let u = b; u !== a;) { const p = prev[u]; tramos.unshift({ e: p.e, dir: p.dir }); u = p.de; }
    return { tramos, len: dist[b] };
  };

  // Puntos de una arista en el sentido del recorrido, suavizados (Chaikin) con los extremos fijos.
  LK.chaikin = (pts, n = 2) => {
    let p = pts;
    for (let k = 0; k < n && p.length > 2; k++) {
      const q = [p[0]];
      for (let i = 0; i < p.length - 1; i++) {
        const a = p[i], b = p[i + 1];
        q.push([0.75 * a[0] + 0.25 * b[0], 0.75 * a[1] + 0.25 * b[1]], [0.25 * a[0] + 0.75 * b[0], 0.25 * a[1] + 0.75 * b[1]]);
      }
      q.push(p[p.length - 1]);
      p = q;
    }
    return p;
  };
  LK.sentido = (t) => (t.dir === 1 ? t.e.pts : t.e.pts.slice().reverse());

  LK.puntoEn = (pts, f) => {
    let total = 0;
    for (let i = 1; i < pts.length; i++) total += LK.dist(pts[i - 1], pts[i]);
    const meta = total * f;
    let acc = 0;
    for (let i = 1; i < pts.length; i++) {
      const a = pts[i - 1], b = pts[i], l = LK.dist(a, b);
      if (l > 0 && acc + l >= meta) {
        const r = (meta - acc) / l;
        return { x: a[0] + (b[0] - a[0]) * r, y: a[1] + (b[1] - a[1]) * r, ang: (Math.atan2(b[1] - a[1], b[0] - a[0]) * 180) / Math.PI };
      }
      acc += l;
    }
    const a = pts[pts.length - 2] || pts[0], b = pts[pts.length - 1];
    return { x: b[0], y: b[1], ang: (Math.atan2(b[1] - a[1], b[0] - a[0]) * 180) / Math.PI };
  };

  // Para cada tramo consecutivo del mismo mapa: camino por la red, o «salto» si no se recorre a v.
  LK.rutaCaminos = (m, v) => {
    const legs = [], uso = {};
    for (const t of m.tramos) {
      if (t.tipo !== 'normal') { legs.push({ t, modo: t.tipo }); continue; }
      if (t.a.id === t.b.id) { legs.push({ t, modo: 'quieto' }); continue; }
      const c = LK.camino(t.a.mapa, t.a.id, t.b.id);
      if (!c) { legs.push({ t, modo: 'salto', len: t.d, vcam: t.d / Math.max(t.dt, 1) }); continue; }
      const vcam = c.len / Math.max(t.dt, 1);
      const leg = { t, c, len: c.len, vcam, modo: vcam > v ? 'salto' : 'camino' };
      legs.push(leg);
      if (leg.modo === 'camino') {
        c.tramos.forEach((x) => {
          const u = (uso[x.e.k] = uso[x.e.k] || { e: x.e, n: 0, dirs: { 1: 0, '-1': 0 }, mapa: t.a.mapa });
          u.n += 1;
          u.dirs[x.dir] += 1;
        });
      }
    }
    return { legs, uso };
  };

  // --- Escuadras (boceto C) ---
  LK.hubs = (m, mapa, desde, excluir) => {
    const out = [];
    const sp = LK.spawnPropio(m.equipo, mapa);
    if (sp) out.push({ id: sp.id, pos: sp.pos, tipo: 'Spawn' });
    const ids = new Set([...m.propias, ...m.vigentes.filter((x) => x.t <= desde).map((x) => x.id), ...m.pasos.filter((x) => x.t < desde).map((x) => x.id)]);
    ids.forEach((id) => {
      const o = LK.OBJ[id];
      if (o && o.m === mapa && id !== excluir && o.c && (o.t === 'Keep' || o.t === 'Castle')) out.push({ id, pos: o.c, tipo: o.t });
    });
    return out;
  };

  // Cada claim va a la escuadra que lo explica con menos prisa: correr si cabe, waypoint si no, escuadra nueva si ninguna.
  LK.pistas = (m, v) => {
    const pistas = [], tramos = [], pistaDe = {};
    for (const p of m.pasos) {
      let pista = null, modo = 'inicio', hub = null, d = null;
      const ant0 = m.pasos[p.i - 2];
      if (p.sinPos && ant0) { pista = pistas[pistaDe[ant0.i]]; modo = 'sinpos'; }
      else if (!p.sinPos) {
        const cands = [];
        for (const pi of pistas) {
          const u = pi.ult;
          if (!u) continue;
          if (u.mapa !== p.mapa) { cands.push({ pi, modo: 'mapa', req: 0, orden: 0, t: u.t }); continue; }
          const dd = LK.dist(u.pos, p.pos), dt = Math.max(p.t - u.t, 1);
          if (dd / dt <= v) { cands.push({ pi, modo: 'correr', req: dd / dt, orden: 0, d: dd, t: u.t }); continue; }
          let mejor = null;
          LK.hubs(m, p.mapa, u.t, p.id).forEach((h) => {
            const dh = LK.dist(h.pos, p.pos);
            if (dh / v + LK.WP_ESPERA <= dt && (!mejor || dh < mejor.dh)) mejor = { h, dh };
          });
          if (mejor) cands.push({ pi, modo: 'wp', req: dd / dt, orden: 1, d: dd, hub: mejor.h, dh: mejor.dh, t: u.t });
        }
        cands.sort((a, b) => a.orden - b.orden || a.req - b.req || b.t - a.t);
        if (cands[0]) { pista = cands[0].pi; modo = cands[0].modo; hub = cands[0].hub || null; d = cands[0].d ?? null; }
      }
      if (!pista) { pista = { k: pistas.length, pasos: [], tramos: [] }; pistas.push(pista); modo = 'inicio'; }
      pistaDe[p.i] = pista.k;
      const ant = pista.pasos[pista.pasos.length - 1] || null;
      let tipo = modo;
      if (ant && (!ant.pos || !p.pos)) tipo = 'sinpos';
      const tr = { pista: pista.k, a: ant, b: p, modo: tipo, hub, d: tipo === 'sinpos' ? null : d, dt: ant ? p.t - ant.t : null };
      if (ant && ant.pos && p.pos && ant.mapa === p.mapa) tr.d = LK.dist(ant.pos, p.pos);
      tr.vreq = tr.d !== null && tr.dt ? tr.d / Math.max(tr.dt, 1) : null;
      pista.pasos.push(p);
      pista.tramos.push(tr);
      tramos.push(tr);
      if (p.pos) pista.ult = p;
    }
    return { pistas, tramos, pistaDe };
  };

  // --- Prisma espacio-tiempo (boceto D): dónde pudo estar entre dos claims a velocidad máxima v ---
  LK.prisma = (a, b, dt, v) => {
    const d = LK.dist(a, b), L = v * dt;
    const A = Math.max(L, d) / 2, C = d / 2;
    return {
      cx: (a[0] + b[0]) / 2, cy: (a[1] + b[1]) / 2, rx: A, ry: Math.sqrt(Math.max(A * A - C * C, 0)),
      ang: (Math.atan2(b[1] - a[1], b[0] - a[0]) * 180) / Math.PI, d, L, imposible: L < d, falta: Math.max(d - L, 0),
    };
  };
})(window.LK);
