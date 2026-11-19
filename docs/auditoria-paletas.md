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
