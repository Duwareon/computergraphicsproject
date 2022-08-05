extern crate minifb;
mod text;

use text::Font;
use std::time::Instant;

use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 512;
const HEIGHT: usize = 512;

fn from_u8_rgb(col: [u8; 3]) -> u32 {
    let (r, g, b) = (col[0] as u32, col[1] as u32, col[2] as u32);
    (r << 16) | (g << 8) | b
}

fn interpolate(i0: u32, d0: f32, i1: u32, d1: f32) -> Vec<u32> {
    if i0 == i1 {
        return vec![d0 as u32];
    }
    let mut values: Vec<u32> = vec![];
    let a = (d1 - d0) / (i1 - i0) as f32;
    let mut d = d0;
    for _ in i0..i1 {
        values.append(&mut vec![d as u32]);
        d = d + a;
    }
    return values;
}

struct Buffer {
    buf: Vec<u32>,
    defaultfont: Font,
}

impl Buffer {
    pub fn new(buf: Vec<u32>, defaultfont: Font) -> Self {
        Buffer { buf: buf, defaultfont: defaultfont }
    }

    #[allow(dead_code)]
    pub fn into_vec(&mut self) -> &Vec<u32> {
        return &self.buf;
    }

    pub fn into_vec_mut(&mut self) -> &mut Vec<u32> {
        return &mut self.buf;
    }

    pub fn put_pixel(&mut self, x: u32, y: u32, col: [u8; 3]) {
        let index: usize = (x + y * WIDTH as u32) as usize;
        self.buf[index] = from_u8_rgb(col);
    }

    pub fn clear(&mut self, col: [u8; 3]) {
        let mut clearedbuf: Vec<u32> = vec!();
        let rgbcol = from_u8_rgb(col);


        for _ in 0..self.buf.len() {
            clearedbuf.append(&mut vec![rgbcol]);
        }

        self.buf = clearedbuf;
    }

    pub fn draw_text(&mut self, text: &str, font: Font, x: u32, y:u32, col: [u8; 3]) {
        // Loop through the characters in the text
        for i in text.char_indices(){
            let glyph = font.clone().shape_char(i.1);

            // Loop through the lines in the character
            for j in glyph.clone().into_iter().enumerate() {

                // Loop through the pixels in the line
                for k in j.1.char_indices() {
                    if k.1 == '1' {
                        self.put_pixel(x+(k.0 + i.0*6) as u32, y+j.0 as u32, col);
                    }
                }
            }
        }
    }

    pub fn draw_func(
        &mut self,
        f: impl Fn(f32) -> f32,
        col: [u8; 3],
        xscale: f32,
        yscale: f32,
        xoff: f32,
        yoff: f32,
    ) {
        for i in 0..WIDTH as u32 {
            let xoff = xoff + (WIDTH / 2) as f32;
            let yoff = yoff + (HEIGHT / 2) as f32;

            let j = yoff - (f((i as f32 - xoff) / xscale) * yscale);

            self.put_pixel(i, (j as u32).min(HEIGHT as u32 - 1), col);
        }
    }

    

    pub fn draw_line(&mut self, p0: [u32; 2], p1: [u32; 2], col: [u8; 3]) {
        let mut p0 = p0;
        let mut p1 = p1;

        if (p1[0] as i32 - p0[0] as i32).abs() > (p1[1] as i32 - p0[1] as i32).abs() {
            // Line is horizontal-ish
            // Make sure x0 < x1
            if p0[0] > p1[0] {
                std::mem::swap(&mut p0, &mut p1);
            }

            let ys = interpolate(p0[0], p0[1] as f32, p1[0], p1[1] as f32);

            for x in p0[0]..p1[0] {
                self.put_pixel(x, ys[(x - p0[0]) as usize], col)
            }
        } else {
            // Line is vertical-ish
            // Make sure y0 < y1
            if p0[1] > p1[1] {
                std::mem::swap(&mut p0, &mut p1)
            }
            let xs = interpolate(p0[1], p0[0] as f32, p1[1], p1[0] as f32);
            for y in p0[1]..p1[1] {
                self.put_pixel(xs[(y - p0[1]) as usize], y, col)
            }
        }
    }

    pub fn draw_wire_tri(&mut self, p0: [u32; 2], p1: [u32; 2], p2: [u32; 2], col: [u8; 3]) {
        self.draw_line(p0, p1, col);
        self.draw_line(p1, p2, col);
        self.draw_line(p2, p0, col);
    }

    fn draw_test(&mut self) {
        self.clear([0x00; 3]);

        self.put_pixel(10, 10, [0xff, 0x80, 0x00]);

        let func = |i: f32| -> f32 {
            i.abs().powf(2.0 / 3.0)
                + (8.0 - i.abs().powf(2.0)).abs().powf(1.0 / 2.0) * (16.0 * 3.1415926 * i).sin()
        };
        self.draw_func(func, [0xff, 0x80, 0xff], 60.0, 60.0, 0.0, 0.0);

        let func = |i: f32| -> f32 { i.sin() };

        self.draw_func(func, [0x80, 0xff, 0x00], 20.0, 100.0, 0.0, 0.0);

        self.draw_line([150, 200], [300, 400], [0xff, 0x00, 0x80]);
        self.draw_line([150, 400], [400, 300], [0xff, 0x00, 0x80]);
        self.draw_wire_tri([200, 300], [400, 400], [200, 200], [0x80, 0x80, 0xff]);

        //self.draw_text("poggers! Holy F*@#ING sh1t.", self.defaultfont.clone(),100, 100, [0xff; 3]);
    }

    pub fn draw(&mut self) {
        self.draw_test()
    }
}

fn main() -> std::io::Result<()> {
    let mut buffer: Buffer = Buffer::new(vec![0; WIDTH * HEIGHT], Font::new(Font::format("scientifica-11.bdf")));

    let mut options: WindowOptions = WindowOptions::default();

    options.borderless = false;

    let mut window = Window::new("pogging my pants", WIDTH, HEIGHT, options).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut time = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        /*for (pixel, i) in buffer.into_vec_mut().iter_mut().enumerate() {
            *i = from_u8_rgb(0xff, 0x80, pixel as u8);
        }*/

        
        buffer.draw();        
        buffer.draw_text(&Instant::now().duration_since(time).as_millis().to_string(), buffer.defaultfont.clone(), 10, 10, [0xff; 3]);

        time = Instant::now();
        
        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer.into_vec_mut(), WIDTH, HEIGHT)
            .unwrap();
        
    }
    Ok(())
}
