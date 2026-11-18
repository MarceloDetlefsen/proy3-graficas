# Generación de LODs en Blender

Para generar las mallas de bajo detalle de `assets/ship.obj` desde Blender (sin interfaz gráfica), ejecuta desde `proy3-graficas/`:

```bash
blender --background --python tools/make_ship_lods.py
```

Salidas esperadas:
- `assets/ship_lod0.obj` (~3,000 triángulos)
- `assets/ship_lod1.obj` (~800 triángulos)
- `assets/ship_lod2.obj` (~150 triángulos)

> **Nota**: este script no fue ejecutado en el entorno de desarrollo (se requiere Blender instalado por Marcelo); el motor incluye decimación por agrupación de vértices en `src/mesh_lod.rs` como respaldo automático cuando estos archivos aún no existen.
