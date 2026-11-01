/* Propuesta B · Estela: la ruta como una estela que se espesa hacia el presente, con línea de tiempo,
   huecos largos y reproducción paso a paso. */
window.LK = window.LK || {};
(function (LK) {
  'use strict';

  const f1 = (n) => Math.round(n * 10) / 10;
  const AM = LK.AMBAR, TINTA = LK.TINTA;
  const esc = LK.esc;
  const VENT = { '1h': 3600, '15m': 900 };
  let reloj = null;
  const parar = () => { if (reloj) { clearInterval(reloj); reloj = null; } };

  // futuro: después del paso elegido; fuera: más viejo que la ventana; actual: el paso elegido; vivo: el resto.
  const estadoDe = (m, st, p) => {
    const cur = m.pasos[st.t - 1];
    if (p.i > cur.i) return 'futuro';
    if (VENT[st.ventana] && cur.t - p.t > VENT[st.ventana]) return 'fuera';
    return p.i === cur.i ? 'actual' : 'vivo';
  };

  // Arco cuadrático que se abre a un lado u otro; devuelve también un punto y el ángulo para la punta.
  function arco(a, b, lado) {
    const dx = b[0] - a[0], dy = b[1] - a[1], L = Math.hypot(dx, dy) || 1;
    const off = Math.max(14, Math.min(0.22 * L, 95)) * lado;
    const c = [(a[0] + b[0]) / 2 - (dy / L) * off, (a[1] + b[1]) / 2 + (dx / L) * off];
    const t = 0.58, u = 1 - t;
    const p = [u * u * a[0] + 2 * u * t * c[0] + t * t * b[0], u * u * a[1] + 2 * u * t * c[1] + t * t * b[1]];
    const tx = 2 * u * (c[0] - a[0]) + 2 * t * (b[0] - c[0]), ty = 2 * u * (c[1] - a[1]) + 2 * t * (b[1] - c[1]);
    return { d: `M${f1(a[0])},${f1(a[1])} Q${f1(c[0])},${f1(c[1])} ${f1(b[0])},${f1(b[1])}`, p, ang: (Math.atan2(ty, tx) * 180) / Math.PI, L };
  }

  function marco(m, mf, i, st) {
    const v = LK.vista(mf, LK.extras(m, mf), st.completo);
    const n = m.pasos.length, cur = m.pasos[st.t - 1];
    const origen = LK.origenHoy(m, mf);
    const sal = LK.salida(v, mf, origen, {});
    let bajo = sal.bajo, alto = '';
    m.tramos.forEach((tr) => {
      if (tr.tipo !== 'normal' || tr.a.mapa !== mf.mapa || tr.a.id === tr.b.id) return;
      const e = estadoDe(m, st, tr.b);
      const f = n > 2 ? (tr.n - 1) / (n - 2) : 1;
      const r = arco(v.P(tr.a.pos), v.P(tr.b.pos), tr.n % 2 ? 1 : -1);
      if (e === 'futuro') {
        bajo += `<path class="d" data-n="${tr.n}" d="${r.d}" fill="none" stroke="#9fb3c4" stroke-opacity=".34" stroke-width="1.4" stroke-dasharray="2 6" stroke-linecap="round"/>`;
        return;
      }
      if (e === 'fuera') {
        bajo += `<path class="d" data-n="${tr.n}" d="${r.d}" fill="none" stroke="${AM}" stroke-opacity=".16" stroke-width="1.4"/>`;
        return;
      }
      const w = 1.8 + 3.2 * f + (e === 'actual' ? 1.4 : 0);
      bajo += `<path class="traza d" data-n="${tr.n}" pathLength="1" style="--d:${(tr.n - 1) * 70}ms" d="${r.d}" fill="none" stroke="${AM}" stroke-opacity="${f1(0.36 + 0.64 * f)}" stroke-width="${f1(w)}" stroke-linecap="round"/>`;
      if (r.L > 46) bajo += `<g class="d" data-n="${tr.n}">${LK.flecha(r.p[0], r.p[1], r.ang, 3.4 + 2.2 * f, AM)}</g>`;
    });
    // Anillo fino: cuánto pasó hasta el siguiente claim (no es un radio del mapa).
    mf.pasos.forEach((p) => {
      const sig = m.pasos[p.i];
      if (!p.pos || !sig || estadoDe(m, st, p) === 'futuro' || estadoDe(m, st, p) === 'fuera') return;
      const [x, y] = v.P(p.pos), dt = sig.t - p.t;
      bajo += `<circle class="d" data-i="${p.i}" cx="${f1(x)}" cy="${f1(y)}" r="${f1(13 + Math.min(30, 4.2 * Math.sqrt(dt / 60)))}" fill="none" stroke="${AM}" stroke-opacity=".55" stroke-width="1.2"><title>Paso ${p.i}: ${LK.dur(dt)} hasta el siguiente claim</title></circle>`;
    });
    LK.huerfanos(m, mf).forEach((h) => { if (h.p.i <= cur.i) { const s = LK.marcaSinPos(v, h); bajo += s.bajo; alto += s.alto; } });
    LK.grupos(mf).forEach((g) => {
      const es = g.pasos.map((p) => estadoDe(m, st, p));
      const e = es.includes('actual') ? 'actual' : es.includes('vivo') ? 'vivo' : es.includes('fuera') ? 'fuera' : 'futuro';
      const disco = { futuro: { hueco: true, dash: '3 2', stroke: '#9fb3c4', r: 8 }, fuera: { fill: '#5d6c79', r: 7 }, actual: { r: 10.5 }, vivo: {} }[e];
      alto += LK.discoGrupo(v, g, { ultimo: e === 'actual' ? cur.i : 0, disco });
    });
    const aqui = mf.pasos.filter((p) => p.i <= cur.i).length;
    const pie = `<span>${LK.textoOrigen(origen)}</span><span>${aqui} de ${mf.pasos.length} pasos mostrados</span>${LK.bandeja(mf)}`;
    return LK.figura(m, mf, i, v, { bajo, alto: (cajas) => alto + sal.alto + LK.cruces(m, mf, v, cajas), pie, ids: origen.id ? [origen.id] : [], aria: `${LK.mapaNombre(mf.mapa)}: estela de la ruta hasta el paso ${cur.i}` });
  }

  const pc = (m, t) => (m.t1 === m.t0 ? 50 : 2 + (96 * (t - m.t0)) / (m.t1 - m.t0));
  const IC = {
    play: '<svg viewBox="0 0 18 18" width="18" height="18" aria-hidden="true"><path d="M5 3 L15 9 L5 15 Z" fill="currentColor"/></svg>',
    pausa: '<svg viewBox="0 0 18 18" width="18" height="18" aria-hidden="true"><rect x="4" y="3" width="3.4" height="12" rx="1" fill="currentColor"/><rect x="10.6" y="3" width="3.4" height="12" rx="1" fill="currentColor"/></svg>',
  };

  function extra(m, st) {
    const n = m.pasos.length, cur = m.pasos[st.t - 1], prev = m.pasos[st.t - 2];
    const x = m.pasos.map((p) => pc(m, p.t));
    const ult = [-99, -99];
    let h = '';
    m.tramos.forEach((tr) => {
      const xa = x[tr.a.i - 1], xb = x[tr.b.i - 1];
      if (tr.hueco) h += `<div class="hueco" style="left:${f1(xa)}%;width:${f1(xb - xa)}%" title="Sin datos durante ${LK.dur(tr.dt)}">${xb - xa > 14 ? `<span>sin datos ${LK.dur(tr.dt)}</span>` : ''}</div>`;
      if (tr.a.mapa !== tr.b.mapa) h += `<div class="cruce" style="left:${f1((xa + xb) / 2)}%"><span>${LK.mapaNombre(tr.b.mapa)}</span></div>`;
    });
    h += `<div class="avance" style="width:${f1(x[st.t - 1] - 2)}%"></div>`;
    m.pasos.forEach((p, k) => {
      const r = x[k] - ult[0] >= 3.4 ? 0 : x[k] - ult[1] >= 3.4 ? 1 : (ult[0] <= ult[1] ? 0 : 1);
      ult[r] = x[k];
      const e = estadoDe(m, st, p);
      h += `<i class="tk ${e}${p.sinPos ? ' sp' : ''}" style="left:${f1(x[k])}%" data-i="${p.i}" title="${p.i} · ${p.hora} · ${esc(p.o.n)}"></i>`
        + `<b class="tn r${r} ${e}" style="left:${f1(x[k])}%" data-i="${p.i}">${p.i}${p.sinPos ? '?' : ''}</b>`;
    });
    h += `<div class="mango" style="left:${f1(x[st.t - 1])}%"></div>`;
    return `<section class="linea" aria-label="Línea de tiempo del caso"><h2>Línea de tiempo</h2>
<div class="linea-fila"><button type="button" class="btn-ico" data-act="play" aria-label="${st.play ? 'Pausar' : 'Reproducir paso a paso'}">${st.play ? IC.pausa : IC.play}</button>
<div class="pista-env"><div class="pista" role="slider" tabindex="0" data-act="scrub" aria-label="Paso del recorrido" aria-valuemin="1" aria-valuemax="${n}" aria-valuenow="${st.t}" aria-valuetext="Paso ${cur.i} de ${n}, ${cur.hora}, ${esc(cur.o.n)}">${h}</div>
<div class="linea-ejes"><span>${LK.hm(m.t0)}</span><span>${LK.hm(m.t1)}</span></div></div></div>
<p class="lectura"><b>Paso ${cur.i} de ${n}</b> · ${cur.hora} · ${esc(cur.o.n)}${prev ? ` · ${LK.dur(cur.t - prev.t)} después del anterior` : ' · primer claim'}</p></section>`;
  }

  const ico = {
    estela: `<path d="M3,14 Q10,2 18,11" fill="none" stroke="${AM}" stroke-opacity=".4" stroke-width="1.8" stroke-linecap="round"/><path d="M18,11 Q25,18 31,5" fill="none" stroke="${AM}" stroke-width="4.6" stroke-linecap="round"/>${LK.flecha(26, 12, -40, 3.6, AM)}`,
    anillo: `<circle cx="17" cy="9" r="8.6" fill="none" stroke="${AM}" stroke-opacity=".6" stroke-width="1.2"/>${LK.disco(17, 9, '4', { r: 5.6, fs: 8 })}`,
    futuro: LK.disco(17, 9, '5', { r: 7, hueco: true, dash: '3 2', stroke: '#9fb3c4', fs: 9 }),
    actual: `<circle cx="17" cy="9" r="8.4" fill="none" stroke="${LK.NARANJA}" stroke-width="2"/>${LK.disco(17, 9, '3', { r: 5.2, fs: 8 })}`,
    hueco: '<rect x="2" y="3" width="30" height="12" rx="2" fill="rgba(255,255,255,.06)" stroke="#6d7f8f" stroke-dasharray="3 2"/><text x="17" y="9" dy=".35em" text-anchor="middle" font-size="8" fill="#c4d0da">sin datos</text>',
  };

  LK.P = {
    id: 'b', nombre: 'Estela', marco, extra, parar,
    cambia: [
      'La ruta es una estela: más gruesa y más opaca cuanto más reciente, con arcos que se abren a un lado y a otro para que ir y volver no se tape.',
      'Una línea de tiempo con el reloj real: se ven los ritmos, los huecos largos sin datos y los cambios de mapa.',
      'Se puede arrastrar o reproducir paso a paso, y limitar la estela a la última hora o a los últimos 15 minutos.',
    ],
    leer: [
      [ico.estela, 'Arco que se espesa hacia el presente. Lo más viejo se apaga.'],
      [ico.anillo, 'Anillo fino: tiempo hasta el siguiente claim. Más grande, más tiempo (no es un radio del mapa).'],
      [ico.actual, 'Anillo naranja: el paso elegido en la línea de tiempo.'],
      [ico.futuro, 'Hueco y punteado: lo que aún no ha pasado en la reproducción.'],
      [ico.hueco, 'Banda de contorno discontinuo: 20 minutos o más entre dos claims.'],
    ],
    controles: [
      { k: 'ventana', tipo: 'sel', et: 'Ventana de la estela', op: [['todo', 'Todo el caso'], ['1h', 'La última hora'], ['15m', 'Los últimos 15 min']], ayuda: 'Cuenta hacia atrás desde el paso elegido en la línea de tiempo.' },
    ],
    defecto: { t: 1, play: false, ventana: 'todo' },
    alCaso(m, st) { st.t = m.pasos.length; st.play = false; },
    act: {
      play({ st, m, redibujar }) {
        if (reloj) { parar(); st.play = false; return true; }
        if (st.t >= m.pasos.length) st.t = 1;
        st.play = true;
        reloj = setInterval(() => {
          if (st.t >= m.pasos.length) { parar(); st.play = false; } else st.t += 1;
          redibujar();
        }, 700);
        return true;
      },
    },
    iniciar(ctx) {
      let arrastra = false;
      const elegir = (ev) => {
        const p = ctx.el.querySelector('.pista');
        if (!p) return;
        const r = p.getBoundingClientRect(), x = ((ev.clientX - r.left) / r.width) * 100;
        let mejor = ctx.st.t, dm = Infinity;
        p.querySelectorAll('.tk').forEach((tk) => { const d = Math.abs(parseFloat(tk.style.left) - x); if (d < dm) { dm = d; mejor = Number(tk.dataset.i); } });
        if (mejor !== ctx.st.t) { ctx.st.t = mejor; ctx.redibujar(); }
      };
      ctx.el.addEventListener('pointerdown', (ev) => {
        if (!ev.target.closest('.pista')) return;
        parar(); ctx.st.play = false; arrastra = true; elegir(ev);
      });
      ctx.el.addEventListener('pointermove', (ev) => { if (arrastra) elegir(ev); });
      window.addEventListener('pointerup', () => { arrastra = false; });
      window.addEventListener('pointercancel', () => { arrastra = false; });
      ctx.el.addEventListener('keydown', (ev) => {
        if (!ev.target.closest || !ev.target.closest('.pista')) return;
        const n = ctx.modelo().pasos.length;
        let t = ctx.st.t;
        if (ev.key === 'ArrowRight' || ev.key === 'ArrowUp') t = Math.min(n, t + 1);
        else if (ev.key === 'ArrowLeft' || ev.key === 'ArrowDown') t = Math.max(1, t - 1);
        else if (ev.key === 'Home') t = 1;
        else if (ev.key === 'End') t = n;
        else return;
        ev.preventDefault();
        parar(); ctx.st.play = false;
        if (t !== ctx.st.t) { ctx.st.t = t; ctx.redibujar(); }
      });
    },
    notas: {
      n1: 'Seis claims en 28 min 40 s. El anillo más grande es el del paso 4: 8 min 35 s hasta el siguiente claim. Arrastra la línea de tiempo y la estela se va construyendo.',
      n2: 'El anillo del paso 2 es pequeño: solo 80 s hasta el 3, el tramo apurado que la recta escondía. El paso 0 sigue saliendo del garri.',
      n3: 'Un solo claim: no hay estela ni anillos. Queda la salida y el reloj en la línea de tiempo.',
      e1: 'Seis claims en menos de 7 min: arcos largos cruzando el mapa y anillos pequeños (50 a 100 s). La estela enseña la prisa, pero no separa a los grupos (eso lo hace la propuesta C).',
      e2: 'Veintiocho claims en 3 h 07 min. Con la ventana de 15 min solo se ve lo reciente y el resto se apaga; el hueco de 45 min (21:31 a 22:16) sale como una banda de contorno discontinuo.',
      e3: 'El paso 3 va como «3?» solo en la línea de tiempo. Los dos huecos (22 min 10 s y 29 min) salen como bandas de contorno discontinuo y coinciden con los cambios de mapa, que se marcan con el nombre del destino: Alpino verde y, al volver, Eternos.',
    },
    necesita: [
      'Nada nuevo en la API: usa la hora de cada claim, que ya se guarda (el historial vive en memoria, ADR-015).',
      'Estado de interfaz para el paso elegido, la ventana y la reproducción. Sin red ni E/S en el hilo de render.',
      'Decidir con el producto cuál es la ventana por defecto (todo, 1 h o 15 min).',
      'Spec o ADR nuevo antes de implementar (AGENTS §2.4). Sigue rotulado como «ruta inferida» (ADR-005) y no predice el siguiente objetivo (ADR-001).',
    ],
  };
})(window.LK);
