# Medidas de la raqueta Wilson Ultra (`refs/raqueta.webp`)

## Método de medición
1. Se cargó `refs/raqueta.webp` (597×1024 px) con `tools/measure_racket.py`.
2. Se separaron las dos vistas por componentes conexas de píxeles del marco: perfil izquierdo (`x ∈ [6, 164]`) y frente derecho (`x ∈ [178, 590]`, altura total `1000 px`).
3. Todas las dimensiones lineales se normalizaron a `longitud total = 2.0` (`escala = 0.002000 u/px`) con origen en el centro vertical de la vista frontal (`y ∈ [-1.0, +1.0]`) y eje de simetría sobre la mediana horizontal del mango.
4. Se detectaron los agujeros internos de la vista frontal (componentes de fondo desconectadas del borde): el lecho de cuerdas y la abertura triangular de la garganta en Y.
5. Se remuestreó el aro en 64 ángulos desde el centro de la cabeza para obtener la línea media y el ancho local en el plano, y se midió la profundidad (eje Z) sobre el perfil izquierdo.
6. Se calculó la mediana RGB por zona (costado izquierdo turquesa, derecho azul, tercio superior/puente marino, mango en percentiles 32/68, cuerdas grises y 'W' roja) y se discretizó la 'W' en una máscara `32×40`.

## Tabla de medidas normalizadas

| Parámetro | Valor normalizado | Píxeles (aprox.) |
|---|---:|---:|
| Longitud total | `2.0000` | `1000` |
| Centro vertical de cabeza (`Y`) | `0.4620` | `281.0` |
| Ancho exterior de cabeza | `0.8240` | `412` |
| Alto exterior de cabeza | `1.0500` | `525` |
| Ancho interior de cabeza (lecho) | `0.7460` | `373` |
| Alto interior de cabeza (lecho) | `0.9880` | `494` |
| Grosor del aro (costado) | `0.0380` | `19.0` |
| Grosor del aro (arriba) | `0.0440` | `22` |
| Profundidad del marco (cabeza, Z) | `0.0840` | `42.0` |
| Profundidad del marco (garganta, Z) | `0.1080` | `54.0` |
| Profundidad del mango (Z) | `0.0960` | `48.0` |
| Largo de la garganta | `0.3360` | `168` |
| Apertura superior de la Y | `0.2580` | `129` |
| Unión inferior de brazos (`Y`) | `-0.3680` | `696` |
| Largo del mango | `0.5920` | `296` |
| Ancho del mango (arriba / abajo) | `0.0980` / `0.0980` | `49.0` / `49.0` |
| Ancho del tapón | `0.1240` | `62` |

## Paleta medida (mediana RGB)

| Zona | Hex | RGB `[0..1]` |
|---|---|---|
| Marco izquierdo (turquesa) | `#61A4C5` | `[0.3804, 0.6431, 0.7725]` |
| Marco derecho (azul) | `#4C789E` | `[0.2980, 0.4706, 0.6196]` |
| Tercio superior y puente (marino) | `#2D313C` | `[0.1765, 0.1922, 0.2353]` |
| Mango tono A / tono B | `#3D4048` / `#454754` | `[0.2392, 0.2510, 0.2824]` / `[0.2706, 0.2784, 0.3294]` |
| Cuerdas | `#D0C9CB` | `[0.8157, 0.7882, 0.7961]` |
| Logo W rojo | `#C8261E` | `[0.7843, 0.1490, 0.1176]` |
