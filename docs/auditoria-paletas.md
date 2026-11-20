# Auditoría de Paletas (PR I)

Análisis automático del tono dominante (espacio HSV, excluyendo el fondo estelar oscuro) medido sobre las capturas oficiales generadas con `--docs` en `docs/captures/*.png` (800×600).

## Tono dominante por cuerpo

| Cuerpo | Shader | Captura | Tono dominante (H°) | Color promedio del disco | Píxeles analizados |
|---|---|---|---:|---|---:|
| Sol | `Star` | [sol.png](captures/sol.png) | **315°** (magenta/violeta) | `#A431B3` | 230 932 |
| Zebes | `Rocky` | [zebes.png](captures/zebes.png) | **28°** (ámbar/tierra) | `#65511F` | 253 880 |
| Tau Ceti | `GasGiant` | [tau_ceti.png](captures/tau_ceti.png) | **97°** (verde bandas) | `#20400D` | 201 908 |
| Ardens | `Ardens` | [ardens.png](captures/ardens.png) | **60°** (dorado/naranja lava) | `#AB7D48` | 146 160 |
| Abismo | `Abismo` | [abismo.png](captures/abismo.png) | **193°** (azul cian/teal) | `#195261` | 221 624 |
| Basquia | `Basquia` | [basquia.png](captures/basquia.png) | **49°** (trazos amarillo sobre negro) | `#7E6049` | 74 088 |
| Vestigio | `Vestigio` | [vestigio.png](captures/vestigio.png) | **60°** (oliva/crema VHS) | `#6A6154` | 262 688 |
| Mural | `Mural` | [mural.png](captures/mural.png) | **8°** (parches rojo/crema sobre gris azulado) | `#70635E` | 246 476 |
| Fundido | `Fundido` | [fundido.png](captures/fundido.png) | **17°** (salmón/naranja magma) | `#9A6750` | 253 376 |

## Pares con distancia de tono menor a 25°

| Par | Tonos (H°) | Distancia angular | Nota visual |
|---|---:|---:|---|
| `Ardens` ↔ `Vestigio` | 60° ↔ 60° | **0°** | En el histograma ponderado por `S·V`, las grietas doradas de `Ardens` y las crestas oliva-crema de `Vestigio` caen en ~60°; visualmente `Ardens` es basalto violeta oscuro (`#2A0F2E`) con grietas incandescentes y `Vestigio` es desierto oliva/rosa con scanlines VHS. |
| `Mural` ↔ `Fundido` | 8° ↔ 17° | **9°** | `Mural` tiene 55% gris azulado desaturado (`#7C8794`, baja saturación `S`), por lo que el pico saturado de `S·V` cae en sus parches rojos (`#B8412F`, 8°), mientras `Fundido` es salmón/naranja en todo el disco. |
| `Zebes` ↔ `Fundido` | 28° ↔ 17° | **11°** | Ambos comparten tonos cálidos (roca ámbar vs. magma naranja/salmón); se distinguen por el mar turquesa, bioma verde y relieve rocoso de `Zebes` frente a las bandas onduladas de `Fundido`. |
| `Ardens` ↔ `Basquia` | 60° ↔ 49° | **11°** | El fondo negro de `Basquia` queda excluido por umbral de brillo y su trazo amarillo (`#F2C500`) domina el histograma junto al dorado de las grietas de `Ardens`; visualmente son inconfundibles (lienzo negro con crayón vs. placas de basalto). |
| `Basquia` ↔ `Vestigio` | 49° ↔ 60° | **11°** | Trazos amarillos sobre negro vs. dunas oliva-rosa con grano VHS. |
| `Zebes` ↔ `Mural` | 28° ↔ 8° | **20°** | Roca ámbar/turquesa vs. muro gris azulado descascarado. |
| `Zebes` ↔ `Basquia` | 28° ↔ 49° | **21°** | Superficie planetaria rocosa vs. lienzo negro con trazos primarios. |

> **Nota**: No se modificaron los colores de ningún shader en esta auditoría.

---

## Sistema de favoritos (13 cuerpos)

Medición automática del tono dominante (espacio HSV ponderado por `S·V`, excluyendo el fondo estelar y los paneles de interfaz) sobre las capturas de galería `docs/captures/galeria_*.png` (`800×600`).

| Cuerpo | Shader | Captura | Tono dominante (H°) | Color promedio del objeto | Píxeles analizados |
|---|---|---|---:|---|---:|
| Sol de los Phoenix Suns | `PhoenixSuns` | [galeria_suns.png](captures/galeria_suns.png) | **37°** (dorado/naranja solar) | `#9F5931` | 65 416 |
| Joy-Con izquierdo | `Chunky` | [galeria_joycon_izq.png](captures/galeria_joycon_izq.png) | **321°** (residuo neón/fondo)* | `#3F3D4E` | 119 322 |
| Joy-Con derecho | `Chunky` | [galeria_joycon_der.png](captures/galeria_joycon_der.png) | **321°** (residuo neón/fondo)* | `#4C3344` | 114 750 |
| Cubo de Rubik | `Chunky` | [galeria_rubik.png](captures/galeria_rubik.png) | **0°** (rojo dominante) | `#99392E` | 168 172 |
| Raqueta de tenis | `Chunky` | [galeria_raqueta.png](captures/galeria_raqueta.png) | **321°** (residuo/aro)* | `#6D4D65` | 125 146 |
| Cubo Skewb | `Chunky` | [galeria_skewb.png](captures/galeria_skewb.png) | **321°** (residuo/blanco)* | `#887376` | 108 234 |
| Meteoro de FF7 | `Meteor` | [galeria_meteoro_ff7.png](captures/galeria_meteoro_ff7.png) | **194°** (verde azulado / turquesa) | `#4D6976` | 81 862 |
| Casco de Samus | `Chunky` | [galeria_samus.png](captures/galeria_samus.png) | **14°** (terracota/salmón) | `#856A5C` | 122 818 |
| Trifuerza | `Triforce` | [galeria_trifuerza.png](captures/galeria_trifuerza.png) | **43°** (dorado metálico) | `#9C7756` | 123 962 |
| Megaminx | `Chunky` | [galeria_megaminx.png](captures/galeria_megaminx.png) | **0°** (cara roja/esquema 12 colores) | `#827371` | 110 490 |
| Cresta del Tottenham | `Spurs` | [galeria_spurs.png](captures/galeria_spurs.png) | **221°** (azul marino) | `#2C2A4F` | 92 318 |
| AirPods | `Chunky` | [galeria_airpods.png](captures/galeria_airpods.png) | **321°** (estuche blanco + brillo)* | `#8A7382` | 122 706 |
| La Bombonera | `Chunky` | [galeria_bombonera.png](captures/galeria_bombonera.png) | **224°** (azul gradas + gris palcos) | `#625C67` | 148 146 |

\* *En objetos predominantemente acromáticos (grises, blancos o negros: `AirPods`, `Joy-Con`, `Cubo Skewb`, `Raqueta de tenis`), la baja saturación del propio objeto hace que el pico de `S·V` caiga en el halo púrpura del cielo estelar (~321°) que rodea la silueta.*

### Pares con distancia de tono menor a 25° (Sistema de favoritos)

| Par | Tonos (H°) | Distancia angular | Nota visual |
|---|---:|---:|---|
| `Sol de los Phoenix Suns` ↔ `Trifuerza` | 37° ↔ 43° | **6°** | Ambos dominan en dorado/naranja cálido; se distinguen por la silueta plana con rayos de los Suns frente a los tres prismas huecos de la Trifuerza. |
| `Sol de los Phoenix Suns` ↔ `Casco de Samus` | 37° ↔ 14° | **23°** | Naranja/dorado solar frente a terracota rojizo con visor verde menta. |
| `Cubo de Rubik` ↔ `Megaminx` | 0° ↔ 0° | **0°** | Ambos comparten cara roja visible y cuerpo negro con pegatinas de colores; se distinguen por geometría (cubo de 27 piezas vs. dodecaedro de 12 caras). |
| `Cubo de Rubik` ↔ `Casco de Samus` | 0° ↔ 14° | **14°** | Rojo vivo de la cara frontal vs. terracota mate/satinado del casco. |
| `Casco de Samus` ↔ `Megaminx` | 14° ↔ 0° | **14°** | Terracota con visor menta vs. dodecaedro multicolor. |
| `Cresta del Tottenham` ↔ `La Bombonera` | 221° ↔ 224° | **3°** | Ambos comparten azul marino/oscuro como tono dominante; se distinguen claramente por silueta (gallo sobre balón vs. estadio en D con franja amarilla y cancha verde). |
| Grupo acromático (`Joy-Con izq/der`, `Raqueta`, `Skewb`, `AirPods`) | 321° ↔ 321° | **0°** | El pico medido refleja el halo de fondo por baja saturación promedio; visualmente sus acentos (riel azul/rojo neón, aro turquesa, estuche blanco) los separan entre sí. |

> **Nota**: No se modificaron los colores de ningún cuerpo ni shader; esta lista queda documentada para decisión del autor.
