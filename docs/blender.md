# Generación de LODs en Blender

Para generar las mallas de bajo detalle de `assets/ship.obj` desde Blender (sin interfaz gráfica), ejecuta desde `proy3-graficas/`:

```bash
blender --background --python tools/make_ship_lods.py
```

Salidas generadas con `blender --background --python tools/make_ship_lods.py` (Blender 5.2.2 LTS, Decimate `COLLAPSE` por bisección preservando caja envolvente `< 2.5%`):
- `assets/ship_lod0.obj`: `3000` triángulos (`ratio = 0.10102`)
- `assets/ship_lod1.obj`: `800` triángulos (`ratio = 0.02694`)
- `assets/ship_lod2.obj`: `610` triángulos (`ratio = 0.02060`; límite mínimo sin colapsar las islas desconectadas de las alas ni exceder el 3% de tolerancia en caja envolvente)

> **Nota**: `src/mesh_lod.rs::load_ship_lods` carga directamente estos tres archivos cuando existen y conserva la decimación por agrupación de vértices únicamente como respaldo automático si faltaran.

## Modelos de superficie (`assets/surface/`)

Archivos esperados:
- `assets/surface/tree.obj`
- `assets/surface/rock.obj`
- `assets/surface/building.obj`
- `assets/surface/monument.obj`

Especificaciones de modelado y exportación en Blender:
- **Conteo de triángulos**: máximo ~300 triángulos por modelo.
- **Origen**: en el centro de la base (`(0, 0, 0)`), de modo que el motor pueda apoyar y hundir ligeramente la base sobre la superficie deformada.
- **Orientación y escala**: eje `+Y` hacia arriba, **1.0 unidad de alto** (`y` en `[0.0, 1.0]`).
- **Opciones de exportación OBJ**:
  - `Forward Axis`: `-Z` / `Up Axis`: `+Y`
  - Marcar **Triangulate Mesh** (solo triángulos)
  - Marcar **Write Normals** (`vn`)
  - Opcional: **UV Coordinates** (`vt`) si se desea texturizado futuro

Cuando alguno de los `.obj` no existe en `assets/surface/`, el motor avisa en consola y utiliza automáticamente los marcadores de posición procedurales de `src/surface_props.rs`.
