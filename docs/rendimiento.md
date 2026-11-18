# Rendimiento del renderizador por software

## Mediciones base (800×600, perfil `release` inicial)

Medido en modo `release` con `std::time::Instant` (promedio de 10 cuadros sin tope de FPS).
`Lejos` = esfera cubriendo `15.3%` de la pantalla (`73,580` px visibles); `Cerca` = `100.0%` (`480,000` px visibles).

| Shader | Distancia | FPS | Total (ms) | Vertex (ms) | Raster (ms) | Shade (ms) | FB + Z-test (ms) | Present (ms) |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| Star | Lejos | 8.41 | 118.85 | 0.021 | 4.37 | 103.31 | 4.33 | 1.26 |
| Star | Cerca | 1.42 | 703.92 | 0.022 | 29.44 | 612.81 | 24.22 | 2.21 |
| Rocky | Lejos | 1.90 | 526.25 | 0.025 | 4.31 | 508.25 | 5.19 | 3.06 |
| Rocky | Cerca | 0.31 | 3266.74 | 0.022 | 30.42 | 3170.78 | 27.34 | 3.01 |
| GasGiant | Lejos | 2.38 | 420.84 | 0.020 | 4.28 | 402.48 | 5.28 | 2.98 |
| GasGiant | Cerca | 0.39 | 2567.86 | 0.021 | 30.64 | 2469.83 | 27.33 | 3.07 |

## Hallazgos principales

1. **Sombreado dominante (87%–97% del frame)** y **ruido `OpenSimplex` (>79% de los ciclos de CPU)**: el profiling con `perf` muestra `open_simplex_4d::surflet` (25.3%), `OpenSimplex::get` 3D (22.9%) y 4D (17.4%), `noise::ridged` (8.1%) y `noise::fbm` (5.7%) como el cuello de botella principal.
2. **Overdraw de 2.0×**: sin backface culling y ejecutando `fragment_shader` antes del z-test, la cara trasera oculta también se rasteriza y sombrea (`147,734` frags para `73,580` px visibles lejos; `960,379` para `480,000` cerca).
3. **Asignaciones por cuadro**: `Vec<Fragment>` (48 bytes/elemento) reserva hasta `~58 MiB` por frame, y `Framebuffer` recrea `Image` y `Texture` en cada cuadro.

## Presupuesto objetivo de frame

- **Mínimo**: **30 FPS** (`<= 33.33 ms/frame`) con el sistema completo a resolución interna por defecto.
- **Deseable**: **60 FPS** (`<= 16.67 ms/frame`).

## Mediciones tras el rasterizador por tiles (PR B)

Máquina de prueba: **16 hilos lógicos** (`std::thread::available_parallelism() = 16`).
Promedio de 15 cuadros por caso en modo `release` (`lto = true`, `codegen-units = 1`, backface culling activo, buffer de visibilidad 64×64 por tiles, ruido propio `f32`, mapas estáticos horneados `512×256`, textura persistente).

| Escala | Hilos | Shader | Distancia | FPS | Total (ms) | Raster (ms) | Shade (ms) | Pres (ms) |
|---|---:|---|---|---:|---:|---:|---:|---:|
| 1× (800×600) | 1 | Star | Lejos | 35.9 | 27.88 | 0.63 | 24.38 | 1.02 |
| 1× (800×600) | 1 | Star | Cerca | 8.3 | 120.76 | 2.81 | 115.78 | 0.67 |
| 1× (800×600) | 1 | Rocky | Lejos | 24.8 | 40.25 | 0.63 | 37.22 | 0.48 |
| 1× (800×600) | 1 | Rocky | Cerca | 4.4 | 229.02 | 2.74 | 223.99 | 0.64 |
| 1× (800×600) | 1 | GasGiant | Lejos | 59.8 | 16.73 | 0.61 | 14.20 | 0.58 |
| 1× (800×600) | 1 | GasGiant | Cerca | 11.3 | 88.31 | 2.72 | 83.53 | 0.57 |
| 1× (800×600) | 16 | Star | Lejos | 102.3 | 9.77 | 0.40 | 7.16 | 0.54 |
| 1× (800×600) | 16 | Star | Cerca | 37.6 | 26.59 | 0.78 | 23.59 | 0.64 |
| 1× (800×600) | 16 | Rocky | Lejos | 79.6 | 12.56 | 0.36 | 10.09 | 0.56 |
| 1× (800×600) | 16 | Rocky | Cerca | 22.6 | 44.30 | 0.77 | 41.51 | 0.66 |
| 1× (800×600) | 16 | GasGiant | Lejos | 158.4 | 6.31 | 0.36 | 3.84 | 0.49 |
| 1× (800×600) | 16 | GasGiant | Cerca | 47.7 | 20.96 | 0.81 | 17.63 | 0.72 |
| 1/2 (400×300) | 1 | Star | Lejos | 100.5 | 9.95 | 0.23 | 7.81 | 0.56 |
| 1/2 (400×300) | 1 | Star | Cerca | 29.5 | 33.95 | 0.79 | 31.43 | 0.55 |
| 1/2 (400×300) | 1 | Rocky | Lejos | 87.9 | 11.38 | 0.21 | 9.84 | 0.34 |
| 1/2 (400×300) | 1 | Rocky | Cerca | 16.5 | 60.72 | 0.75 | 58.39 | 0.49 |
| 1/2 (400×300) | 1 | GasGiant | Lejos | 212.9 | 4.70 | 0.18 | 3.64 | 0.20 |
| 1/2 (400×300) | 1 | GasGiant | Cerca | 42.9 | 23.28 | 0.75 | 21.10 | 0.39 |
| 1/2 (400×300) | 16 | Star | Lejos | 195.7 | 5.11 | 0.22 | 3.96 | 0.22 |
| 1/2 (400×300) | 16 | Star | Cerca | 113.0 | 8.85 | 0.32 | 7.45 | 0.30 |
| 1/2 (400×300) | 16 | Rocky | Lejos | 152.1 | 6.57 | 0.26 | 4.94 | 0.31 |
| 1/2 (400×300) | 16 | Rocky | Cerca | 69.0 | 14.50 | 0.34 | 12.95 | 0.35 |
| 1/2 (400×300) | 16 | GasGiant | Lejos | 308.8 | 3.24 | 0.18 | 2.05 | 0.20 |
| 1/2 (400×300) | 16 | GasGiant | Cerca | 162.4 | 6.16 | 0.29 | 4.86 | 0.25 |
| 1/3 (266×200) | 1 | Star | Lejos | 232.2 | 4.31 | 0.11 | 3.61 | 0.13 |
| 1/3 (266×200) | 1 | Star | Cerca | 60.8 | 16.45 | 0.34 | 15.13 | 0.31 |
| 1/3 (266×200) | 1 | Rocky | Lejos | 202.8 | 4.93 | 0.10 | 4.24 | 0.11 |
| 1/3 (266×200) | 1 | Rocky | Cerca | 38.7 | 25.84 | 0.30 | 24.51 | 0.35 |
| 1/3 (266×200) | 1 | GasGiant | Lejos | 343.9 | 2.91 | 0.10 | 1.84 | 0.19 |
| 1/3 (266×200) | 1 | GasGiant | Cerca | 91.8 | 10.90 | 0.33 | 9.51 | 0.28 |
| 1/3 (266×200) | 16 | Star | Lejos | 238.2 | 4.20 | 0.22 | 2.89 | 0.26 |
| 1/3 (266×200) | 16 | Star | Cerca | 143.7 | 6.96 | 0.28 | 5.23 | 0.34 |
| 1/3 (266×200) | 16 | Rocky | Lejos | 186.1 | 5.37 | 0.24 | 3.53 | 0.37 |
| 1/3 (266×200) | 16 | Rocky | Cerca | 103.5 | 9.66 | 0.28 | 8.27 | 0.28 |
| 1/3 (266×200) | 16 | GasGiant | Lejos | 492.0 | 2.03 | 0.15 | 1.31 | 0.10 |
| 1/3 (266×200) | 16 | GasGiant | Cerca | 255.1 | 3.92 | 0.23 | 3.10 | 0.16 |

### Comparativa y cuello de botella restante

- **Cumplimiento de presupuesto**: a la resolución interna por defecto (`1/2`, `400×300`) con `16` hilos se supera tanto el mínimo de **30 FPS** como el deseable de **60 FPS** incluso con un planeta ocupando el 100% de la pantalla (`Star` `113.0 FPS`, `Rocky` `69.0 FPS`, `GasGiant` `162.4 FPS`).
- **Aceleración a igual resolución (`800×600`, `16` hilos vs. base)**: `Rocky` cerca pasa de `0.31` a `22.6 FPS` (**72.9×**); `GasGiant` cerca pasa de `0.39` a `47.7 FPS` (**122.3×**); `Star` cerca pasa de `1.42` a `37.6 FPS` (**26.5×**).
- **Cuello de botella restante**: la etapa de **sombreado** sigue representando `75%–94%` del tiempo de frame (especialmente en `Rocky` cerca a `800×600`, donde la capa viva de nubes `cloud_mask` evalúa `fbm` de 3 octavas por píxel junto con el muestreo bilineal triple para el bump).

## Costo por cuerpo con el sistema completo en pantalla (PR H1)

Medido en modo `release` (`400×300`, `16` hilos, promedio de `15` cuadros) con el pipeline completo activo: icosfera deformada con LOD por tamaño proyectado, frustum culling, horneado `512×256`, mapeo de normales en espacio tangente (`Rocky`), instancias en superficie con culling y LOD, nave y fondo estelar por píxel vacío. Acercamiento por cuerpo a `d = 2.4 × radio`.

| Vista | Triángulos | FPS | Total (ms) | Vértices (ms) | Raster (ms) | Shade (ms) |
|---|---:|---:|---:|---:|---:|---:|
| Sistema general (`d = 24`) | 5 424 | 135.5 | 7.38 | 0.73 | 0.70 | 5.89 |
| `Sol` cerca (`Star`) | 5 200 | 117.3 | 8.53 | 0.77 | 0.79 | 6.90 |
| `Zebes` cerca (`Rocky` + 253 instancias) | 34 412 | 58.6 | 17.06 | 4.51 | 1.71 | 10.79 |
| `Tau Ceti` cerca (`GasGiant`) | 5 200 | 126.7 | 7.89 | 0.62 | 0.58 | 6.64 |

- **Criterio de 30 FPS**: **cumplido en todos los cuerpos** (`58.6–135.5 FPS`, margen `1.95×–4.5×` sobre los 30 FPS mínimos).
- **Cuello de botella en `Zebes` cerca**: al subir la icosfera deformada a `L5` (`20 480` tris) junto con las `253` instancias en superficie (`34 412` tris totales), la etapa de vértices sube a `4.51 ms` y el sombreado (`cloud_mask` + `normal_mapping` + fondo estelar) a `10.79 ms`.
