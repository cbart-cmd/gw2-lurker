/* Cáscara común de los cuatro bocetos: cabecera con las propuestas, carril de casos, escenario, panel y
   estado en el hash (#e2, #e2-hoy). Cada propuesta define LK.P y llama a LK.app(LK.P). */
window.LK = window.LK || {};
(function (LK) {
  'use strict';

  const PROPS = [
    { id: 'a', f: 'boceto-a-caminos.html', n: 'Caminos' },
    { id: 'b', f: 'boceto-b-estela.html', n: 'Estela' },
    { id: 'c', f: 'boceto-c-escuadras.html', n: 'Escuadras' },
    { id: 'd', f: 'boceto-d-incertidumbre.html', n: 'Incertidumbre' },
  ];
  const EQ = { Red: 'rojo', Green: 'verde', Blue: 'azul' };
  const esc = LK.esc;
  const AM = LK.AMBAR;
  const cap = (s) => s.charAt(0).toUpperCase() + s.slice(1);

  const HOY = {
    reglas: [
      'Une los claims con rectas, en el orden de la hora.',
      'Pinta el paso 0 desde el garri, la fortaleza lateral o el punto de reaparición.',
      'Descarta en silencio los pasos sin posición y numera por mapa.',
      'No avisa del cambio de mapa ni de los huecos largos entre claims.',
    ],
    leer: [
      [`<line x1="3" y1="9" x2="31" y2="9" stroke="${AM}" stroke-width="2"/>`, 'Recta ámbar entre dos claims seguidos.'],
      [LK.disco(17, 9, '2', { r: 7.5 }), 'Número: orden del claim en este mapa.'],
      [LK.disco(17, 9, '0', { r: 7.5, hueco: true }), 'El 0 hueco es la salida inferida.'],
      [`<line x1="3" y1="9" x2="31" y2="9" stroke="${AM}" stroke-width="1.6" stroke-dasharray="5 4"/>`, 'Discontinua: de la salida al primer claim.'],
      [`<circle cx="17" cy="9" r="7" fill="none" stroke="${LK.NARANJA}" stroke-width="2"/>`, 'Anillo naranja: el último claim.'],
    ],
  };

  const ESQUELETO = `<a class="salto" href="#escena">Saltar al escenario</a>
<header class="top">
  <div class="marca"><svg viewBox="0 0 28 28" width="28" height="28" aria-hidden="true"><rect width="28" height="28" rx="7" fill="#141c25" stroke="#22303d"/><path d="M6 20 C10 20 9 9 14 10 S19 8 22 8" fill="none" stroke="${AM}" stroke-width="2" stroke-linecap="round"/><circle cx="6" cy="20" r="2.4" fill="${AM}"/><circle cx="22" cy="8" r="2.4" fill="${AM}"/></svg><div><strong>WvW Lurker</strong><span>Ruta inferida · bocetos</span></div></div>
  <nav id="props" class="props" aria-label="Propuestas"></nav>
  <div id="conm" class="conm" role="group" aria-label="Qué ruta ver"></div>
</header>
<div class="cuerpo">
  <nav id="carril" class="carril" aria-label="Casos"></nav>
  <main id="escena" class="escena" tabindex="-1"><header id="caso" class="caso"></header><div id="marcos" class="marcos"></div><div id="extra" class="extra"></div></main>
  <aside id="panel" class="panel" aria-label="Detalle de la propuesta"></aside>
</div>
<footer class="pie-pag"><p>Datos sintéticos sobre objetivos reales (partida 2-2) y gremios ficticios. Los caminos y la velocidad de carrera están por verificar (UNVERIFIED): el boceto sirve para validar la idea, no la medida.</p></footer>`;

  const objetivos = () => {
    const g = (t) => `<svg class="ico" viewBox="0 0 24 24" width="22" height="22" aria-hidden="true">${LK.glifo(t, 12, 12, 7, '#9aa8b5', 0.9)}</svg>`;
    return `<ul class="leyenda base">${['Camp', 'Tower', 'Keep', 'Castle', 'Ruins', 'Spawn'].map((t) => `<li>${g(t)}<span>${cap(LK.TIPO[t])}</span></li>`).join('')}</ul>`
      + '<p class="nota">El relleno es el dueño (rojo, verde, azul o gris sin dueño); el anillo ámbar marca lo reclamado por el clan.</p>';
  };

  const control = (c, st) => {
    if (c.act) return `<button type="button" class="btn" data-act="${c.act}">${c.et}</button>`;
    const val = st[c.k];
    const ayuda = c.ayuda ? `<p class="ayuda">${c.ayuda}</p>` : '';
    if (c.tipo === 'rango') {
      return `<div class="ctl"><label for="c-${c.k}">${c.et}</label><output id="o-${c.k}" for="c-${c.k}">${val}${c.u || ''}</output>`
        + `<input id="c-${c.k}" type="range" data-k="${c.k}" data-u="${c.u || ''}" min="${c.min}" max="${c.max}" step="${c.paso || 1}" value="${val}"></div>${ayuda}`;
    }
    if (c.tipo === 'check') return `<div class="ctl chk"><input id="c-${c.k}" type="checkbox" data-k="${c.k}"${val ? ' checked' : ''}><label for="c-${c.k}">${c.et}</label></div>${ayuda}`;
    if (c.tipo === 'sel') {
      return `<div class="ctl"><label for="c-${c.k}">${c.et}</label><select id="c-${c.k}" data-k="${c.k}">${c.op.map(([v, t]) => `<option value="${v}"${String(val) === String(v) ? ' selected' : ''}>${t}</option>`).join('')}</select></div>${ayuda}`;
    }
    return '';
  };

  LK.app = function (P) {
    const st = Object.assign({ caso: 'n1', vista: 'prop', v: LK.V_DEFECTO, completo: false }, P.defecto || {});
    const modelos = {};
    LK.ESC.forEach((sc) => { modelos[sc.id] = LK.modelo(sc); });
    document.title = `${P.id.toUpperCase()} · ${P.nombre} · Ruta inferida · bocetos`;
    document.getElementById('app').innerHTML = ESQUELETO;
    const q = (id) => document.getElementById(id);
    const el = { nav: q('props'), conm: q('conm'), carril: q('carril'), escena: q('escena'), caso: q('caso'), marcos: q('marcos'), extra: q('extra'), panel: q('panel') };
    const actual = () => { const sc = LK.ESC.find((s) => s.id === st.caso); return { sc, m: modelos[sc.id] }; };
    const hash = () => `#${st.caso}${st.vista === 'hoy' ? '-hoy' : ''}`;
    const leerHash = () => {
      const [c, v] = location.hash.slice(1).split('-');
      st.caso = LK.ESC.some((s) => s.id === c) ? c : 'n1';
      st.vista = v === 'hoy' ? 'hoy' : 'prop';
    };

    const cabecera = () => {
      el.nav.innerHTML = PROPS.map((p) => `<a href="${p.f}${hash()}"${p.id === P.id ? ' aria-current="page"' : ''}><b>${p.id.toUpperCase()}</b>${p.n}</a>`).join('');
      el.conm.innerHTML = [['hoy', 'Hoy'], ['prop', 'Propuesta']].map(([k, t]) => `<button type="button" data-vista="${k}" aria-pressed="${st.vista === k}">${t}</button>`).join('');
    };
    const carril = () => {
      const grupo = (clase, titulo) => `<h2>${titulo}</h2><ul>${LK.ESC.filter((s) => s.clase === clase).map((s) => `<li><a href="#${s.id}${st.vista === 'hoy' ? '-hoy' : ''}"${s.id === st.caso ? ' aria-current="true"' : ''}><span class="cod">${s.cod}</span><span class="tit">${esc(s.titulo)}</span><span class="sub">${esc(s.sub)}</span></a></li>`).join('')}</ul>`;
      el.carril.innerHTML = grupo('normal', 'Casos normales') + grupo('extremo', 'Casos extremos');
    };
    const cabeceraCaso = () => {
      const { sc, m } = actual();
      const mapas = m.marcos.map((f) => LK.mapaNombre(f.mapa)).join(' y ');
      const rango = m.pasos.length > 1 ? `${LK.hm(m.t0)} a ${LK.hm(m.t1)} · ${LK.dur(m.t1 - m.t0)}` : `a las ${LK.hm(m.t0)}`;
      el.caso.innerHTML = `<h1><span class="cod">${sc.cod}</span>${esc(sc.titulo)}</h1>`
        + `<p class="meta"><span class="clan eq-${m.equipo}">${esc(sc.gremio)} · equipo ${EQ[m.equipo]}</span><span>${mapas}</span><span>${m.pasos.length} ${m.pasos.length === 1 ? 'claim' : 'claims'}</span><span>${rango}</span></p>`;
    };
    const panel = () => {
      const { sc } = actual();
      const sec = (t, c) => `<section class="sec"><h2>${t}</h2>${c}</section>`;
      const lista = (a) => `<ul class="lista">${a.map((x) => `<li>${x}</li>`).join('')}</ul>`;
      const leyenda = (a) => `<ul class="leyenda">${a.map(([svg, txt]) => `<li>${LK.ico(svg)}<span>${txt}</span></li>`).join('')}</ul>`;
      const comun = { k: 'completo', tipo: 'check', et: 'Ver el mapa completo' };
      if (st.vista === 'hoy') {
        el.panel.innerHTML = sec('Qué dibuja hoy', lista(HOY.reglas)) + sec('Controles', control(comun, st))
          + sec('Cómo se lee', leyenda(HOY.leer) + objetivos()) + sec('Este caso', `<p>${esc(sc.hoy)}</p>`);
        return;
      }
      // Los controles van arriba: el panel es fijo y con scroll propio, y abajo quedarían fuera de la vista.
      el.panel.innerHTML = sec('Qué cambia', lista(P.cambia))
        + sec('Controles', (P.controles || []).concat([comun]).map((c) => control(c, st)).join(''))
        + sec('Cómo se lee', leyenda(P.leer) + objetivos())
        + sec('Este caso', `<p>${(P.notas && P.notas[sc.id]) || ''}</p>`) + sec('Qué necesita', lista(P.necesita));
    };
    const escenario = (entra) => {
      const { m } = actual();
      const hoy = st.vista === 'hoy';
      const scroll = [...el.marcos.querySelectorAll('.lienzo')].map((l) => l.scrollLeft);
      el.marcos.innerHTML = m.marcos.map((mf, k) => (hoy ? LK.hoy(m, mf, k, st) : P.marco(m, mf, k, st))).join('');
      el.extra.innerHTML = !hoy && P.extra ? P.extra(m, st) : '';
      el.escena.classList.toggle('entra', !!entra);
      el.escena.classList.remove('hov');
      // En pantallas estrechas el mapa se desplaza dentro de su marco: al cambiar de caso arranca centrado y al
      // redibujar (controles, animación) se queda donde estaba, porque el DOM nuevo vuelve al borde izquierdo.
      el.marcos.querySelectorAll('.lienzo').forEach((l, k) => {
        const sobra = l.scrollWidth - l.clientWidth;
        if (sobra > 0) l.scrollLeft = entra || scroll[k] === undefined ? sobra / 2 : scroll[k];
      });
    };
    const todo = (entra) => { cabecera(); carril(); cabeceraCaso(); panel(); escenario(entra); };

    // Redibuja solo el escenario y devuelve el foco al control que lo tenía (el DOM se reemplaza).
    const redibujar = () => {
      const a = document.activeElement;
      const clave = a && a !== document.body && el.escena.contains(a)
        ? ['data-act', 'data-n', 'data-i'].map((k) => (a.hasAttribute(k) ? `[${k}="${a.getAttribute(k)}"]` : '')).join('') : '';
      escenario(false);
      const n = clave && el.escena.querySelector(clave);
      if (n) n.focus({ preventScroll: true });
    };

    const limpiar = () => { el.escena.classList.remove('hov'); el.escena.querySelectorAll('.sel').forEach((x) => x.classList.remove('sel')); };
    const marcar = (ev) => {
      const t = ev.target.closest ? ev.target.closest('[data-i],[data-n]') : null;
      limpiar();
      if (!t) return;
      el.escena.classList.add('hov');
      ['i', 'n'].forEach((a) => {
        const val = t.getAttribute('data-' + a);
        if (val) val.split(' ').forEach((tok) => el.escena.querySelectorAll(`[data-${a}~="${tok}"]`).forEach((x) => x.classList.add('sel')));
      });
    };
    el.escena.addEventListener('pointerover', marcar);
    el.escena.addEventListener('pointerleave', limpiar);
    el.escena.addEventListener('focusin', marcar);
    el.escena.addEventListener('focusout', limpiar);

    const alClick = (ev) => {
      const b = ev.target.closest && ev.target.closest('[data-act]');
      if (!b || !P.act || !P.act[b.dataset.act]) return;
      if (P.act[b.dataset.act]({ st, m: actual().m, el: b, ev, redibujar }) !== false) redibujar();
    };
    el.escena.addEventListener('click', alClick);
    el.panel.addEventListener('click', alClick);
    el.panel.addEventListener('input', (ev) => {
      const t = ev.target, k = t.getAttribute && t.getAttribute('data-k');
      if (!k) return;
      st[k] = t.type === 'checkbox' ? t.checked : t.type === 'range' ? Number(t.value) : t.value;
      const o = document.getElementById('o-' + k);
      if (o) o.textContent = t.value + (t.dataset.u || '');
      if (P.alControl) P.alControl(k, st, actual().m);
      escenario(false);
    });
    el.conm.addEventListener('click', (ev) => {
      const b = ev.target.closest('[data-vista]');
      if (b) location.hash = `#${st.caso}${b.dataset.vista === 'hoy' ? '-hoy' : ''}`;
    });
    window.addEventListener('hashchange', () => {
      leerHash();
      if (P.parar) P.parar();
      if (P.alCaso) P.alCaso(actual().m, st);
      todo(true);
    });

    leerHash();
    if (P.alCaso) P.alCaso(actual().m, st);
    if (P.iniciar) P.iniciar({ el: el.escena, st, redibujar, modelo: () => actual().m });
    todo(true);
  };
})(window.LK);
