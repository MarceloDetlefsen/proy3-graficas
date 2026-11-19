"""Script de Blender (bpy) para generar niveles de detalle de assets/ship.obj.

Uso desde la raíz de proy3-graficas/:
    blender --background --python tools/make_ship_lods.py

Genera (ajustando la razón de Decimate por bisección a ±15% del objetivo):
    assets/ship_lod0.obj (~3000 triángulos)
    assets/ship_lod1.obj (~800 triángulos)
    assets/ship_lod2.obj (~150 triángulos)
"""

import os
import tempfile
import bpy

TARGETS = [
    ("assets/ship_lod0.obj", 3000),
    ("assets/ship_lod1.obj", 800),
    ("assets/ship_lod2.obj", 150),
]
MAX_BISECTION_ITERS = 8
TOLERANCE = 0.15


def clear_scene():
    bpy.ops.wm.read_factory_settings(use_empty=True)


def strip_mtl_to_temp(path):
    tmp = tempfile.NamedTemporaryFile("w", suffix=".obj", delete=False, encoding="utf-8")
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as src:
            for line in src:
                if line.startswith(("mtllib ", "usemtl ")):
                    continue
                tmp.write(line)
    finally:
        tmp.close()
    return tmp.name


def import_ship(clean_obj_path):
    bpy.ops.wm.obj_import(filepath=clean_obj_path)
    meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
    if not meshes:
        raise RuntimeError(f"No se encontraron mallas en {clean_obj_path}")
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


def bounds_world(obj):
    verts = obj.data.vertices
    if not verts:
        return (0.0, 0.0, 0.0), (0.0, 0.0, 0.0)
    xs = [v.co.x for v in verts]
    ys = [v.co.y for v in verts]
    zs = [v.co.z for v in verts]
    return (min(xs), min(ys), min(zs)), (max(xs), max(ys), max(zs))


def bounds_within_tol(base_bounds, cand_obj, frac=0.025):
    (bmin, bmax) = base_bounds
    (cmin, cmax) = bounds_world(cand_obj)
    extent = max(bmax[0] - bmin[0], bmax[1] - bmin[1], bmax[2] - bmin[2], 1e-4)
    dmin = sum((cmin[i] - bmin[i]) ** 2 for i in range(3)) ** 0.5
    dmax = sum((cmax[i] - bmax[i]) ** 2 for i in range(3)) ** 0.5
    return dmin <= extent * frac and dmax <= extent * frac


def decimate_from_base(base_obj, ratio):
    # Duplicar siempre desde la malla original para no acumular decimaciones.
    bpy.ops.object.select_all(action="DESELECT")
    dup = base_obj.copy()
    dup.data = base_obj.data.copy()
    bpy.context.collection.objects.link(dup)
    bpy.context.view_layer.objects.active = dup
    dup.select_set(True)

    mod = dup.modifiers.new(name="DecimateLOD", type="DECIMATE")
    mod.decimate_type = "COLLAPSE"
    mod.ratio = ratio
    mod.use_collapse_triangulate = True
    bpy.ops.object.modifier_apply(modifier=mod.name)
    return dup


def main():
    src_path = os.path.abspath("assets/ship.obj")
    clean_path = strip_mtl_to_temp(src_path)
    try:
        clear_scene()
        base_ship = import_ship(clean_path)
        base_bounds = bounds_world(base_ship)
        orig_tris = max(1, triangle_count(base_ship))

        for out_rel, target_tris in TARGETS:
            low = 0.0001
            high = 1.0
            ratio = min(1.0, max(low, target_tris / float(orig_tris)))
            best_obj = None
            best_tris = orig_tris
            best_ratio = ratio

            for _it in range(MAX_BISECTION_ITERS):
                cand = decimate_from_base(base_ship, ratio)
                tris = triangle_count(cand)
                bbox_ok = bounds_within_tol(base_bounds, cand, 0.025)
                if bbox_ok and (best_obj is None or abs(tris - target_tris) <= abs(best_tris - target_tris)):
                    if best_obj is not None:
                        bpy.data.objects.remove(best_obj, do_unlink=True)
                    best_obj = cand
                    best_tris = tris
                    best_ratio = ratio
                else:
                    bpy.data.objects.remove(cand, do_unlink=True)

                if bbox_ok and abs(tris - target_tris) <= target_tris * TOLERANCE:
                    break
                if not bbox_ok or tris < target_tris:
                    low = ratio
                else:
                    high = ratio
                ratio = 0.5 * (low + high)

            bpy.ops.object.select_all(action="DESELECT")
            bpy.context.view_layer.objects.active = best_obj
            best_obj.select_set(True)
            out_path = os.path.abspath(out_rel)
            bpy.ops.wm.obj_export(
                filepath=out_path,
                export_selected_objects=True,
                export_triangulated_mesh=True,
                export_normals=True,
                export_uv=True,
                export_materials=False,
                forward_axis="NEGATIVE_Z",
                up_axis="Y",
            )
            bpy.data.objects.remove(best_obj, do_unlink=True)
            print(
                f"Exportado {out_rel}: {best_tris} tris "
                f"(objetivo {target_tris} ±15%, ratio={best_ratio:.5f})"
            )
    finally:
        if os.path.exists(clean_path):
            os.remove(clean_path)


if __name__ == "__main__":
    main()
