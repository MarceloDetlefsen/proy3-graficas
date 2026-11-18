# Auditoría de reutilización

Evaluación del contenido disponible en la carpeta general del curso frente a
los requisitos del Proyecto 3.

## Selección principal

| Fuente | Contenido útil | Decisión |
|---|---|---|
| Lab 6 | Pipeline CPU, OBJ, triángulos, framebuffer, z-buffer, normales, ruido y tres shaders celestes | Base principal copiada completa |
| Lab 5 | Primer rasterizador de triángulos y benchmarks | No copiar por separado: Lab 6 ya es su evolución e incluye profundidad e interpolación 3D |
| Lab 4 | Nave OBJ creada durante el curso | Copiar como `assets/ship.obj` |

## Material útil como referencia, no copiado

| Fuente | Idea reutilizable | Motivo para no copiar directamente |
|---|---|---|
| Proyecto 2 | Cámara con vectores forward/right/up, normal maps, texturas, distribución procedural y colisiones | Es un raytracer de cubos; sus intersecciones, materiales y cámara están acoplados a ese pipeline |
| Proyecto 1 | Entrada y movimiento multiplicados por `delta_time`, mouse-look y presentación de UI | Es un raycaster 2.5D; se debe adaptar el patrón, no copiar el renderer |
| Foro raytracing | Cálculo de FOV/perspectiva por rayos | No implementa matrices MVP ni rasterización de triángulos |
| Labs 1 y 2 | Framebuffer 2D, líneas y autómata celular | Superados por el framebuffer y rasterizador del Lab 6 |
| Ejemplos `hello_world`, `ventanas`, `lineas` y `raylib_image_tutorial` | Inicialización de Rust/Raylib y dibujo 2D | Ya están incorporados en la base actual |
| Blender y capturas sueltas | Posibles referencias visuales | No forman parte del pipeline ejecutable ni cumplen un requisito por sí solas |

## Cobertura real de la base

### Ya disponible

- Pipeline de software y presentación del framebuffer con Raylib.
- Carga de OBJ y dibujo de triángulos.
- Prueba de profundidad por píxel.
- Shaders propios claramente diferentes; algunos cambian con el tiempo.
- Esfera y modelo propio de nave disponibles como recursos.
- Ruido procedural reproducible con semilla fija en la biblioteca de ruido.

### Parcial o adaptable

- El shader rocoso perturba normales mediante un campo de altura procedural,
  pero todavía no implementa un normal map con textura y base tangente.
- Las transformaciones actuales están combinadas en una matriz orientada a
  pantalla y usan proyección ortográfica; deben convertirse a MVP real.
- El tiempo ya proviene de `get_frame_time`, pero aún no gobierna órbitas,
  rotaciones de cuerpos ni física de la nave.
- La esfera de 192 triángulos puede servir como primer LOD, pero faltan mallas
  con distintos niveles de detalle y el selector por distancia.

### No disponible

- Vuelo local alrededor de planetas, aceleración/frenado progresivos, gravedad
  y colisión esfera-nave.
- Frustum culling y contadores comparativos de triángulos.
- Geometría planetaria deformada o subdividida antes de rasterizar.
- Objetos colocados y orientados sobre una superficie curva.
- Escena multiobjeto continua con escalas espaciales y de superficie.

## Riesgos técnicos detectados

1. El vertex shader del Lab 6 transforma directamente a píxeles. Extender la
   escena sin introducir vista, perspectiva, división por `w` y clipping hará
   inestable el acercamiento al planeta.
2. La rasterización genera un `Vec<Fragment>` completo antes de escribirlo;
   varios planetas grandes pueden elevar el consumo de memoria. Conviene
   rasterizar/escribir por triángulo o por fragmento durante la evolución.
3. El parser OBJ guarda índices de posición, pero no conserva por esquina los
   índices `vt`/`vn`. Para normal mapping y modelos complejos deberá ampliarse.
4. Una sola esfera de baja resolución se verá facetada al acercarse. Se
   necesitará más detalle o subdivisión/LOD, pero mantener esta malla baja será
   útil para objetos lejanos.

## Estrategia de puntos sugerida

Después de cumplir los cinco requisitos obligatorios, una ruta coherente hacia
100 puntos es: tres planetas distinguibles (30), cuatro tipos de modelos sobre
superficie (20), distribución procedural y orientación curva (15), tres shaders
distinguibles más animación (26) y presentación (hasta 10). La suma disponible
permite margen para que algún criterio reciba crédito parcial; el alcance final
debe ajustarse tras tener vuelo y acercamiento estables.
