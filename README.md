# Proyecto 3: sistema solar con renderer por software

Base inicial del proyecto final de Gráficas por Computadora. El código reutiliza
el renderer por software desarrollado en el Lab 6 y añade el modelo de nave
creado para el Lab 4. Raylib se usa únicamente para ventana, entrada y
presentación del framebuffer.

> Estado actual: esta es una base comprobable, no la entrega final. Renderiza
> una esfera con z-buffer y shaders procedurales, pero todavía no implementa la
> escena completa, la cámara en perspectiva ni el control de vuelo.

## Ejecutar

Requisitos: Rust/Cargo y las herramientas nativas necesarias para compilar
`raylib-sys`.

```bash
cargo run --release
```

Pruebas:

```bash
cargo test
```

## Controles actuales de la base

| Tecla | Acción |
|---|---|
| `1`, `2`, `3` | Cambiar entre shader de estrella, planeta rocoso y gigante gaseoso |
| Flechas | Rotar la esfera |
| Rueda del mouse | Zoom ortográfico |
| `B`, `W` | Relleno baricéntrico o wireframe |
| `J`, `L` | Rotar la luz |
| `D` | Vista de depuración de ruido |
| `V` | Cambiar vista de depuración del planeta rocoso |
| `[` / `]` | Cambiar frecuencia del ruido |
| `P` | Guardar captura |

## Base reutilizada

- `src/`: pipeline de rasterización por software del Lab 6.
- `assets/sphere.obj`: esfera de baja resolución del Lab 6.
- `assets/ship.obj`: nave modelada para el Lab 4, renombrada para el proyecto.
- `noise = 0.9`: OpenSimplex/FBM usado por los shaders procedurales.

La evaluación detallada de los trabajos anteriores y su compatibilidad está en
[`docs/reuse-audit.md`](docs/reuse-audit.md).

## Capacidades que ya existen

- Carga manual de modelos OBJ triangulados.
- Transformación de modelo y normales.
- Ensamblaje y rasterización de triángulos.
- Interpolación baricéntrica de posición y normal.
- Framebuffer y depth buffer propios.
- Shaders de fragmentos en CPU para estrella, planeta rocoso y gigante gaseoso.
- Ruido procedural 3D/4D y animación basada en tiempo transcurrido.

## Requisitos indispensables aún pendientes

- Matrices separadas de modelo, vista y proyección con perspectiva y división
  por `w`.
- Escena simultánea con sol, planeta en órbita y nave visible.
- Control de vuelo de la nave y cámara que la siga.
- Rotación/orbitación dependientes de `delta time` dentro de la escena.
- Acercamiento estable desde escala espacial hasta escala de superficie.
- Instrucciones finales, lista de características solicitadas y video.

## Orden recomendado de implementación

1. Generalizar el vertex shader a matrices `model`, `view`, `projection` y
   coordenadas de clip/NDC.
2. Crear una cámara libre en 3D y dibujar simultáneamente sol, planeta y nave.
3. Añadir movimiento orbital/rotación y controles de vuelo con `delta time`.
4. Resolver clipping/culling y estabilidad al acercarse al planeta.
5. Elegir puntos opcionales: varios planetas, objetos procedurales sobre la
   superficie, colisión/gravedad, LOD y normal mapping.

## Procedencia y atribución

El proyecto parte de trabajo previo propio del curso:

- Lab 6: renderer, esfera, ruido y shaders procedurales.
- Lab 4: modelo OBJ de la nave.

Antes de entregar se deben documentar integrantes, contribuciones, recursos
externos y asistencia de IA, como exige el enunciado.
