#![allow(dead_code)]

use raylib::prelude::*;
use raylib::texture::Image;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    color_pixels: Vec<u8>,
    depth_buffer: Vec<f32>,
    background_color: Color,
    pub current_color: Color,
    texture: Option<Texture2D>,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32, background_color: Color) -> Self {
        let len = (width * height) as usize;
        let mut color_pixels = vec![0u8; len * 4];
        let bg = [
            background_color.r,
            background_color.g,
            background_color.b,
            background_color.a,
        ];
        for px in color_pixels.as_chunks_mut::<4>().0 {
            *px = bg;
        }

        Self {
            width,
            height,
            color_pixels,
            depth_buffer: vec![f32::INFINITY; len],
            background_color,
            current_color: Color::WHITE,
            texture: None,
        }
    }

    pub fn clear(&mut self) {
        let bg = [
            self.background_color.r,
            self.background_color.g,
            self.background_color.b,
            self.background_color.a,
        ];
        for px in self.color_pixels.as_chunks_mut::<4>().0 {
            *px = bg;
        }
        self.depth_buffer.fill(f32::INFINITY);
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32 {
            let idx = (y as usize * self.width as usize + x as usize) * 4;
            self.color_pixels[idx] = color.r;
            self.color_pixels[idx + 1] = color.g;
            self.color_pixels[idx + 2] = color.b;
            self.color_pixels[idx + 3] = color.a;
        }
    }

    pub fn set_pixel_depth(&mut self, x: i32, y: i32, depth: f32, color: Color) -> bool {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return false;
        }

        let index = y as usize * self.width as usize + x as usize;
        if depth >= self.depth_buffer[index] {
            return false;
        }

        self.depth_buffer[index] = depth;
        let p = index * 4;
        self.color_pixels[p] = color.r;
        self.color_pixels[p + 1] = color.g;
        self.color_pixels[p + 2] = color.b;
        self.color_pixels[p + 3] = color.a;
        true
    }

    pub fn pixel_color(&self, x: i32, y: i32) -> Color {
        let idx = (y as usize * self.width as usize + x as usize) * 4;
        Color::new(
            self.color_pixels[idx],
            self.color_pixels[idx + 1],
            self.color_pixels[idx + 2],
            self.color_pixels[idx + 3],
        )
    }

    pub fn visible_pixel_count(&self) -> usize {
        self.depth_buffer
            .iter()
            .filter(|&&depth| depth < f32::INFINITY)
            .count()
    }

    pub fn point(&mut self, point: Vector2) {
        let x = point.x.round() as i32;
        let y = point.y.round() as i32;
        self.set_pixel(x, y, self.current_color);
    }

    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    pub fn render_to_file(&self, filename: &str) {
        if Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("bmp"))
        {
            self.render_to_bmp_file(filename);
        } else {
            let mut image = Image::gen_image_color(
                self.width as i32,
                self.height as i32,
                self.background_color,
            );
            for y in 0..self.height as i32 {
                for x in 0..self.width as i32 {
                    image.draw_pixel(x, y, self.pixel_color(x, y));
                }
            }
            image.export_image(filename);
        }
    }

    pub fn draw_in_window(
        &mut self,
        window: &mut RaylibHandle,
        thread: &RaylibThread,
        label: &str,
        overlay: &[String],
    ) {
        if self.texture.is_none() {
            let image = Image::gen_image_color(
                self.width as i32,
                self.height as i32,
                self.background_color,
            );
            if let Ok(tex) = window.load_texture_from_image(thread, &image) {
                tex.set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_POINT);
                self.texture = Some(tex);
            }
        }
        let win_w = window.get_screen_width() as f32;
        let win_h = window.get_screen_height() as f32;
        let scale = (win_w / self.width as f32).min(win_h / self.height as f32);
        let dst_w = self.width as f32 * scale;
        let dst_h = self.height as f32 * scale;
        let dst = Rectangle::new((win_w - dst_w) * 0.5, (win_h - dst_h) * 0.5, dst_w, dst_h);
        let src = Rectangle::new(0.0, 0.0, self.width as f32, self.height as f32);

        if let Some(texture) = self.texture.as_mut() {
            let _ = texture.update_texture(&self.color_pixels);
            let mut drawing = window.begin_drawing(thread);
            drawing.clear_background(Color::BLACK);
            drawing.draw_texture_pro(texture, src, dst, Vector2::zero(), 0.0, Color::WHITE);
            drawing.draw_text(label, 16, 16, 20, Color::WHITE);
            if !overlay.is_empty() {
                let box_h = 16 + overlay.len() as i32 * 20;
                drawing.draw_rectangle(12, 44, 460, box_h, Color::new(0, 0, 0, 175));
                for (i, line) in overlay.iter().enumerate() {
                    drawing.draw_text(line, 20, 52 + i as i32 * 20, 16, Color::GREEN);
                }
            }
        }
    }

    fn render_to_bmp_file(&self, filename: &str) {
        let width = self.width as usize;
        let height = self.height as usize;
        let row_stride = width * 3;
        let row_padding = (4 - (row_stride % 4)) % 4;
        let image_size = (row_stride + row_padding) * height;
        let file_size = 14 + 40 + image_size;

        let file = File::create(filename).expect("failed to create BMP file");
        let mut writer = BufWriter::new(file);

        // BMP file header.
        writer
            .write_all(&[0x42, 0x4D])
            .expect("failed to write BMP signature");
        writer
            .write_all(&(file_size as u32).to_le_bytes())
            .expect("failed to write BMP file size");
        writer
            .write_all(&[0u8; 4])
            .expect("failed to write BMP reserved fields");
        writer
            .write_all(&(54u32).to_le_bytes())
            .expect("failed to write BMP pixel offset");

        // DIB header (BITMAPINFOHEADER).
        writer
            .write_all(&(40u32).to_le_bytes())
            .expect("failed to write BMP DIB header size");
        writer
            .write_all(&(self.width as i32).to_le_bytes())
            .expect("failed to write BMP width");
        writer
            .write_all(&(self.height as i32).to_le_bytes())
            .expect("failed to write BMP height");
        writer
            .write_all(&(1u16).to_le_bytes())
            .expect("failed to write BMP planes");
        writer
            .write_all(&(24u16).to_le_bytes())
            .expect("failed to write BMP bpp");
        writer
            .write_all(&(0u32).to_le_bytes())
            .expect("failed to write BMP compression");
        writer
            .write_all(&(image_size as u32).to_le_bytes())
            .expect("failed to write BMP image size");
        writer
            .write_all(&(2835u32).to_le_bytes())
            .expect("failed to write BMP x ppm");
        writer
            .write_all(&(2835u32).to_le_bytes())
            .expect("failed to write BMP y ppm");
        writer
            .write_all(&(0u32).to_le_bytes())
            .expect("failed to write BMP palette colors");
        writer
            .write_all(&(0u32).to_le_bytes())
            .expect("failed to write BMP important colors");

        let padding = [0u8; 3];
        for y in (0..height).rev() {
            for x in 0..width {
                let color = self.pixel_color(x as i32, y as i32);
                writer
                    .write_all(&[color.b, color.g, color.r])
                    .expect("failed to write BMP pixel data");
            }
            writer
                .write_all(&padding[..row_padding])
                .expect("failed to write BMP row padding");
        }

        writer.flush().expect("failed to flush BMP file");
    }
}

pub fn point(framebuffer: &mut Framebuffer, point: Vector2) {
    framebuffer.point(point);
}
