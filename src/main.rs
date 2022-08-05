extern crate minifb;

use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 512;
const HEIGHT: usize = 512;

fn from_u8_rgb(col: [u8; 3]) -> u32 {
    let (r, g, b) = (col[0] as u32, col[1] as u32, col[2] as u32);
    (r << 16) | (g << 8) | b
}

struct Buffer {
    buf: Vec<u32>,
}

impl Buffer {
    pub fn new(_buf: Vec<u32>) -> Self {
        return Buffer { buf: _buf };
    }

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

    fn draw_test(&mut self) {
        self.put_pixel(10, 10, [0xff; 3]);

        let func //= |i: f32| -> f32 { i.sin() };
            = |i: f32| -> f32 {
            i.abs().powf(2.0 / 3.0)
                + (8.0 - i.abs().powf(2.0)).abs().powf(1.0 / 2.0) * (16.0 * 3.1415926 * i).sin()
        };
        self.draw_func(func, [0xff; 3], 24.0, 24.0, 0.0, 0.0);
    }

    pub fn draw(&mut self) {
        self.draw_test()
    }
}

fn main() {
    let mut buffer: Buffer = Buffer::new(vec![0; WIDTH * HEIGHT]);

    let mut options: WindowOptions = WindowOptions::default();

    options.borderless = false;

    let mut window = Window::new("pogging my pants", WIDTH, HEIGHT, options).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        /*for (pixel, i) in buffer.into_vec_mut().iter_mut().enumerate() {
            *i = from_u8_rgb(0xff, 0x80, pixel as u8);
        }*/

        buffer.draw();

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer.into_vec_mut(), WIDTH, HEIGHT)
            .unwrap();
    }
}
