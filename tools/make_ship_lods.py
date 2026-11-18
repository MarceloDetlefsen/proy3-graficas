"""Script de Blender (bpy) para generar niveles de detalle de assets/ship.obj.

Uso desde la raíz de proy3-graficas/:
    blender --background --python tools/make_ship_lods.py

Genera:
    assets/ship_lod0.obj (~3000 triángulos)
    assets/ship_lod1.obj (~800 triángulos)
    assets/ship_lod2.obj (~150 triángulos)
"""

import os
import bpy

TARGETS = [
    ("assets/ship_lod0.obj", 3000),
    ("assets/ship_lod1.obj", 800),
    ("assets/ship_lod2.obj", 150),
]


def clear_scene():
    bpy.ops.wm.read_factory_settings(use_empty=True)


def import_ship(path):
    bpy.ops.wm.obj_import(filepath=path)
    meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
    if not meshes:
        raise RuntimeError(f"No se encontraron mallas en {path}")
    bpy.context.view_layer.objects.active = meshes[0]
    for obj in meshes:
        obj.select_set(True)
    if len(meshes) > 1:
        bpy.ops.object.join()
    return bpy.context.view_layer.objects.active


def triangle_count(obj):
    depsgraph = bpy.context.evaluated_depsgraph_get()
    eval_obj = obj.evaluated_get(depsgraph)
    mesh = eval_obj.to_mesh()
    mesh.calc_loop_triangles()
    count = len(mesh.loop_triangles)
    eval_obj.to_mesh_clear()
    return count


def main():
    src_path = os.path.abspath("assets/ship.obj")
    for out_rel, target_tris in TARGETS:
        clear_scene()
        ship = import_ship(src_path)
        current = max(1, triangle_count(ship))
        ratio = min(1.0, max(0.001, target_tris / float(current)))

        mod = ship.modifiers.new(name="DecimateLOD", type="DECIMATE")
        mod.decimate_type = "COLLAPSE"
        mod.ratio = ratio
        mod.use_collapse_triangulate = True

        bpy.ops.object.modifier_apply(modifier=mod.name)
        out_path = os.path.abspath(out_rel)
        bpy.ops.wm.obj_export(
            filepath=out_path,
            export_selected_objects=True,
            export_triangulated_mesh=True,
            export_normals=True,
            export_uv=True,
            forward_axis="NEGATIVE_Z",
            up_axis="Y",
        )
        print(f"Exportado {out_rel} (~{target_tris} tris objetivo, ratio={ratio:.4f})")


if __name__ == "__main__":
    main()
