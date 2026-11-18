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
