/* Seis casos SINTÉTICOS sobre objetivos reales (partida 2-2). Los gremios y las horas son inventados.
   Cada caso lleva los mismos datos para las cuatro propuestas.
   pasos     claims del gremio en orden (sinPos: el catálogo no trae coordenada para ese objetivo, simulado)
   vigentes  claims del gremio anteriores a la ventana y aún en pie (los usa la regla de origen)
   propias   estructuras del equipo con waypoint durante el caso (dato de la API: dueño del objetivo)
   hoy       qué hace hoy la ruta en este caso */
window.LK = window.LK || {};
LK.ESC = [
  {
    id: 'n1', clase: 'normal', cod: 'N1', titulo: 'Empuje por el norte', sub: 'Eternos · rojo · 6 claims',
    equipo: 'Red', gremio: 'Cuervos del Alba',
    pasos: [
      { id: '38-17', t: '20:12:40' }, { id: '38-1', t: '20:16:05' }, { id: '38-19', t: '20:22:30' },
      { id: '38-18', t: '20:27:15' }, { id: '38-9', t: '20:35:50' }, { id: '38-21', t: '20:41:20' },
    ],
    vigentes: [], propias: [],
    hoy: 'Las rectas cruzan el castillo y los barrancos: parece un recorrido, pero solo une puntos.',
  },
  {
    id: 'n2', clase: 'normal', cod: 'N2', titulo: 'Cuenca verde con un tramo apurado', sub: 'Alpino verde · verde · 5 claims',
    equipo: 'Green', gremio: 'Sombras de Jade',
    pasos: [
      { id: '95-40', t: '20:05:10' }, { id: '95-51', t: '20:11:30' }, { id: '95-50', t: '20:12:50' },
      { id: '95-36', t: '20:19:20' }, { id: '95-34', t: '20:26:00' },
    ],
    vigentes: [{ id: '95-37', t: '19:52:00' }], propias: ['95-32'],
    hoy: 'El tramo 2 a 3 son 80 segundos y la recta lo hace parecer un paseo sobre la montaña central.',
  },
  {
    id: 'n3', clase: 'normal', cod: 'N3', titulo: 'Un solo claim', sub: 'Desierto · rojo · 1 claim',
    equipo: 'Red', gremio: 'Cuervos del Alba',
    pasos: [{ id: '1099-99', t: '20:18:30' }],
    vigentes: [{ id: '1099-113', t: '19:40:00' }], propias: [],
    hoy: 'Solo hay una línea discontinua del 0 al 1: sin hora, sin tiempo y sin saber por qué ese origen.',
  },
  {
    id: 'e1', clase: 'extremo', cod: 'E1', titulo: 'Dos escuadras a la vez', sub: 'Eternos · azul · 6 claims',
    equipo: 'Blue', gremio: 'Mareas Azules',
    pasos: [
      { id: '38-15', t: '21:00:00' }, { id: '38-20', t: '21:01:20' }, { id: '38-5', t: '21:02:40' },
      { id: '38-7', t: '21:04:10' }, { id: '38-16', t: '21:05:00' }, { id: '38-1', t: '21:06:40' },
    ],
    vigentes: [], propias: [],
    hoy: 'Dos grupos se dibujan como uno solo que cruza el mapa de punta a punta cada pocos segundos.',
  },
  {
    id: 'e2', clase: 'extremo', cod: 'E2', titulo: 'Maratón de tres horas', sub: 'Eternos · verde · 28 claims',
    equipo: 'Green', gremio: 'Sombras de Jade',
    pasos: [
      { id: '38-10', t: '20:02:10' }, { id: '38-12', t: '20:07:40' }, { id: '38-11', t: '20:13:05' },
      { id: '38-3', t: '20:19:30' }, { id: '38-13', t: '20:25:15' }, { id: '38-14', t: '20:31:00' },
      { id: '38-4', t: '20:36:20' }, { id: '38-12', t: '20:43:10' }, { id: '38-10', t: '20:50:00' },
      { id: '38-18', t: '20:58:20' }, { id: '38-6', t: '21:03:40' }, { id: '38-17', t: '21:08:50' },
      { id: '38-9', t: '21:14:30' }, { id: '38-18', t: '21:19:10' }, { id: '38-9', t: '21:23:00' },
      { id: '38-9', t: '21:27:15' }, { id: '38-1', t: '21:31:00' },
      { id: '38-19', t: '22:16:00' }, { id: '38-21', t: '22:21:30' }, { id: '38-8', t: '22:27:10' },
      { id: '38-22', t: '22:33:00' }, { id: '38-13', t: '22:34:30' }, { id: '38-3', t: '22:40:10' },
      { id: '38-14', t: '22:46:00' }, { id: '38-4', t: '22:51:20' }, { id: '38-15', t: '22:57:40' },
      { id: '38-7', t: '23:03:00' }, { id: '38-16', t: '23:09:30' },
    ],
    vigentes: [], propias: [],
    hoy: 'Veintiocho pasos sobre diecinueve objetivos: la maraña esconde el orden, las repeticiones y el hueco de 45 minutos.',
  },
  {
    id: 'e3', clase: 'extremo', cod: 'E3', titulo: 'Cruce de mapas y un paso sin posición', sub: 'Eternos y Alpino verde · verde · 7 claims',
    equipo: 'Green', gremio: 'Sombras de Jade',
    pasos: [
      { id: '38-13', t: '20:12:00' }, { id: '38-3', t: '20:19:30' }, { id: '38-12', t: '20:27:10', sinPos: true },
      { id: '95-51', t: '20:49:20' }, { id: '95-40', t: '20:55:40' }, { id: '95-37', t: '21:02:00' },
      { id: '38-10', t: '21:31:00' },
    ],
    vigentes: [{ id: '95-32', t: '19:58:00' }], propias: [],
    hoy: 'El paso sin posición desaparece (1, 2, 4), el cambio de mapa no se nota y el origen del alpino se da por bueno sin avisar de la duda.',
  },
];
