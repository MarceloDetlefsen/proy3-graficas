#![allow(dead_code)]
//! Parámetros geométricos, contorno medio, máscara de la 'W' y paleta medidos
//! desde `refs/raqueta.webp` (vista de perfil izquierda + vista frontal derecha).
//!
//! Cómo regenerar:
//!     python3 tools/measure_racket.py

// Longitud total normalizada: 2.0 (escala = 0.002000 u/px sobre 1000 px).
pub const RACKET_TOTAL_LENGTH: f32 = 2.0;
pub const RACKET_HEAD_CENTER_Y: f32 = 0.46200;
pub const RACKET_HEAD_OUTER_WIDTH: f32 = 0.82400;
pub const RACKET_HEAD_OUTER_HEIGHT: f32 = 1.05000;
pub const RACKET_HEAD_INNER_WIDTH: f32 = 0.74600;
pub const RACKET_HEAD_INNER_HEIGHT: f32 = 0.98800;
pub const RACKET_RIM_THICK_SIDE: f32 = 0.03800;
pub const RACKET_RIM_THICK_TOP: f32 = 0.04400;
pub const RACKET_BRIDGE_Y: f32 = -0.04100;
pub const RACKET_BRIDGE_HEIGHT: f32 = 0.01800;
pub const RACKET_THROAT_JOIN_Y: f32 = -0.36800;
pub const RACKET_THROAT_LENGTH: f32 = 0.33600;
pub const RACKET_THROAT_OPEN_WIDTH: f32 = 0.25800;
pub const RACKET_GRIP_TOP_Y: f32 = -0.40800;
pub const RACKET_GRIP_LENGTH: f32 = 0.59200;
pub const RACKET_GRIP_WIDTH_TOP: f32 = 0.09800;
pub const RACKET_GRIP_WIDTH_BOT: f32 = 0.09800;
pub const RACKET_BUTTCAP_WIDTH: f32 = 0.12400;

// Profundidades medidas en la vista de perfil (eje Z).
pub const RACKET_DEPTH_HEAD: f32 = 0.08400;
pub const RACKET_DEPTH_THROAT: f32 = 0.10800;
pub const RACKET_DEPTH_GRIP: f32 = 0.09600;

// Paleta medida por mediana de píxeles en zonas de `refs/raqueta.webp`.
pub const RACKET_TEAL_RGB: [f32; 3] = [0.3804, 0.6431, 0.7725]; // #61A4C5
pub const RACKET_BLUE_RGB: [f32; 3] = [0.2980, 0.4706, 0.6196]; // #4C789E
pub const RACKET_NAVY_RGB: [f32; 3] = [0.1765, 0.1922, 0.2353]; // #2D313C
pub const RACKET_GRIP_A_RGB: [f32; 3] = [0.2392, 0.2510, 0.2824]; // #3D4048
pub const RACKET_GRIP_B_RGB: [f32; 3] = [0.2706, 0.2784, 0.3294]; // #454754
pub const RACKET_STRING_RGB: [f32; 3] = [0.8157, 0.7882, 0.7961]; // #D0C9CB
pub const RACKET_LOGO_RED_RGB: [f32; 3] = [0.7843, 0.1490, 0.1176]; // #C8261E

/// Línea media del aro (x, y) y ancho local en el plano por ángulo desde el centro de la cabeza.
pub const RACKET_HOOP_MIDLINE: &[[f32; 3]] = &[
    [0.39400, 0.46200, 0.04000],
    [0.39409, 0.50081, 0.04000],
    [0.39329, 0.54023, 0.03800],
    [0.38947, 0.58015, 0.04200],
    [0.38341, 0.62081, 0.03800],
    [0.37305, 0.66140, 0.04200],
    [0.36252, 0.70423, 0.04800],
    [0.34554, 0.74557, 0.04600],
    [0.32527, 0.78727, 0.04400],
    [0.29943, 0.82686, 0.04400],
    [0.26723, 0.86194, 0.05000],
    [0.23240, 0.89679, 0.04200],
    [0.19172, 0.92486, 0.04200],
    [0.14746, 0.94813, 0.04000],
    [0.10008, 0.96514, 0.04200],
    [0.05038, 0.97352, 0.04400],
    [0.00000, 0.97800, 0.04400],
    [-0.05028, 0.97253, 0.04600],
    [-0.09969, 0.96318, 0.04200],
    [-0.14688, 0.94621, 0.04400],
    [-0.19134, 0.92394, 0.04400],
    [-0.23098, 0.89414, 0.04400],
    [-0.26501, 0.85861, 0.05000],
    [-0.29690, 0.82377, 0.04400],
    [-0.32244, 0.78444, 0.04400],
    [-0.34244, 0.74304, 0.05000],
    [-0.36003, 0.70256, 0.04600],
    [-0.37129, 0.66046, 0.04200],
    [-0.37879, 0.61890, 0.04000],
    [-0.38565, 0.57898, 0.04200],
    [-0.39035, 0.53965, 0.04000],
    [-0.39011, 0.50042, 0.04000],
    [-0.39000, 0.46200, 0.03600],
    [-0.38613, 0.42397, 0.04000],
    [-0.38153, 0.38611, 0.03800],
    [-0.37416, 0.34850, 0.03800],
    [-0.36401, 0.31122, 0.04000],
    [-0.35189, 0.27391, 0.04200],
    [-0.33841, 0.23588, 0.04600],
    [-0.32157, 0.19809, 0.04400],
    [-0.30476, 0.15724, 0.04200],
    [-0.27913, 0.12188, 0.04800],
    [-0.25223, 0.08451, 0.04800],
    [-0.22438, 0.04221, 0.04800],
    [-0.19058, 0.00191, 0.06000],
    [-0.15240, -0.04039, 0.09000],
    [-0.10964, -0.08920, 0.14800],
    [-0.06087, -0.15601, 0.26600],
    [-0.00000, -0.05500, 0.04600],
    [0.06087, -0.15601, 0.26600],
    [0.11003, -0.09116, 0.14800],
    [0.15298, -0.04231, 0.09000],
    [0.19134, 0.00006, 0.06000],
    [0.22533, 0.04044, 0.04400],
    [0.25445, 0.08119, 0.04800],
    [0.28104, 0.11956, 0.04600],
    [0.30547, 0.15653, 0.04000],
    [0.32312, 0.19682, 0.04800],
    [0.34007, 0.23477, 0.04600],
    [0.35453, 0.27250, 0.04400],
    [0.36678, 0.31007, 0.03800],
    [0.37608, 0.34792, 0.03800],
    [0.38447, 0.38552, 0.04000],
    [0.38912, 0.42368, 0.03800],
];

pub const RACKET_W_MASK_COLS: usize = 32;
pub const RACKET_W_MASK_ROWS: usize = 40;
/// Máscara 32×40 de la 'W' roja normalizada a la caja interior del lecho (fila 0 = arriba).
pub const RACKET_W_MASK: &[u32] = &[
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00F3E7C0,
    0x00E08100,
    0x00808300,
    0x00818300,
    0x00800100,
    0x00800100,
    0x00800100,
    0x00838700,
    0x03838700,
    0x00808700,
    0x00C08700,
    0x00CF9F00,
    0x004C7C00,
    0x004C7C00,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
];
