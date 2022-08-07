mod text;

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use std::mem::swap;
use std::time::Instant;
use text::Font;

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;

fn interpolate(i0: u32, d0: f32, i1: u32, d1: f32) -> Vec<f32> {
    if i0 == i1 {
        return vec![d0 as f32];
    }
    let mut values: Vec<f32> = vec![];
    let a = (d1 - d0) / (i1 as f32 - i0 as f32);
    let mut d = d0;
    for _ in i0..i1 {
        values.append(&mut vec![d]);
        d = d + a;
    }
    return values;
}

// struct Xorshift {
//     val: [u64; 2],
// }
// impl Xorshift {
//     pub fn new() -> Self {
//         return Xorshift{val: [1, 2]};
//     }
//     // Xorshift128+
//     pub fn next(&mut self) -> u64 {
//         let mut s1 = self.val[0];
//         let mut s0 = self.val[1];
//         let result = s0 + s1;
//         self.val[0] = s0;
//         s1 = s1 ^ (s1 << 23);
//         self.val[1] = s1 ^ s0 ^ (s1 >> 18) ^ (s0 >> 5);
//         return result;
//     }
// }

#[derive(Clone)]
struct Buffer {
    buf: Vec<u8>,
    defaultfont: Font,
}

impl Buffer {
    pub fn new(buf: Vec<u8>, defaultfont: Font) -> Self {
        Buffer {
            buf: buf,
            defaultfont: defaultfont,
        }
    }

    #[allow(dead_code)]
    pub fn into_vec(&mut self) -> &Vec<u8> {
        return &self.buf;
    }

    pub fn into_vec_mut(&mut self) -> &mut Vec<u8> {
        return &mut self.buf;
    }

    pub fn put_pixel(&mut self, x: u32, y: u32, col: [u8; 3]) {
        let index: usize = ((x + y * WIDTH) * 4) as usize;
        self.buf[index] = col[0];
        self.buf[index + 1] = col[1];
        self.buf[index + 2] = col[2];
        self.buf[index + 3] = 0xff;
    }

    pub fn clear(&mut self, col: [u8; 3]) {
        for pixel in self.buf.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[col[0], col[1], col[2], 0xff]);
        }
    }

    pub fn draw_text(&mut self, text: &str, font: Font, x: u32, y: u32, col: [u8; 3]) {
        // Loop through the characters in the text
        for (i, character) in text.char_indices() {
            // loop through pixels in glyph
            for (_, loc) in font.clone().shape_char(character).into_iter().enumerate() {
                self.put_pixel(loc[0] as u32 + x + i as u32 * 6, loc[1] as u32 + y, col);
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
        for i in 0..WIDTH {
            let xoff = xoff + (WIDTH / 2) as f32;
            let yoff = yoff + (HEIGHT / 2) as f32;

            let j = yoff - (f((i as f32 - xoff) / xscale) * yscale);

            self.put_pixel(i, (j as u32).min(HEIGHT - 1), col);
        }
    }

    pub fn draw_line(&mut self, p0: [u32; 2], p1: [u32; 2], col: [u8; 3]) {
        let mut p0 = p0;
        let mut p1 = p1;

        if (p1[0] as i32 - p0[0] as i32).abs() > (p1[1] as i32 - p0[1] as i32).abs() {
            // Line is horizontal-ish
            // Make sure x0 < x1
            if p0[0] > p1[0] {
                swap(&mut p0, &mut p1);
            }

            let ys = interpolate(p0[0], p0[1] as f32, p1[0], p1[1] as f32);

            for x in p0[0]..p1[0] {
                self.put_pixel(x, ys[(x - p0[0]) as usize] as u32, col)
            }
        } else {
            // Line is vertical-ish
            // Make sure y0 < y1
            if p0[1] > p1[1] {
                swap(&mut p0, &mut p1)
            }
            let xs = interpolate(p0[1], p0[0] as f32, p1[1], p1[0] as f32);
            for y in p0[1]..p1[1] {
                self.put_pixel(xs[(y - p0[1]) as usize]as u32, y, col)
            }
        }
    }

    pub fn draw_wire_tri(&mut self, p0: [u32; 2], p1: [u32; 2], p2: [u32; 2], col: [u8; 3]) {
        self.draw_line(p0, p1, col);
        self.draw_line(p1, p2, col);
        self.draw_line(p2, p0, col);
    }

    pub fn draw_filled_tri(&mut self, p0: [u32; 2], p1: [u32; 2], p2: [u32; 2], col: [u8; 3]) {
        let mut p0 = p0;
        let mut p1 = p1;
        let mut p2 = p2;

        if p1[1] < p0[1] {
            swap(&mut p1, &mut p0)
        }
        if p2[1] < p0[1] {
            swap(&mut p2, &mut p0)
        }
        if p2[1] < p1[1] {
            swap(&mut p2, &mut p1)
        }

        let x0 = p0[0];
        let y0 = p0[1];
        let x1 = p1[0];
        let y1 = p1[1];
        let x2 = p2[0];
        let y2 = p2[1];

        let mut x012 = interpolate(y0, x0 as f32, y1, x1 as f32);
        let mut x12 = interpolate(y1, x1 as f32, y2, x2 as f32);
        let x02 = interpolate(y0, x0 as f32, y2, x2 as f32);

        x012.pop();
        x012.append(&mut x12);

        let m = (x012.len() as f32 / 2.0).floor() as usize;
        let x_left: Vec<f32>;
        let x_right: Vec<f32>;
        if x02[m] < x012[m] {
            x_left = x02;
            x_right = x012;
        } else {
            x_left = x012;
            x_right = x02;
        }

        for y in y0..y2 - 1 {
            for x in x_left[(y - y0) as usize] as u32..x_right[(y - y0) as usize] as u32 {
                self.put_pixel(x, y, col);
            }
        }
    }

    pub fn draw_gradient_tri(
        &mut self,
        p0: [u32; 2],
        p1: [u32; 2],
        p2: [u32; 2],
        col: [u8; 3],
        h: [f32; 3],
    ) {
        let mut p0 = p0;
        let mut p1 = p1;
        let mut p2 = p2;

        if p1[1] < p0[1] {
            swap(&mut p1, &mut p0)
        }
        if p2[1] < p0[1] {
            swap(&mut p2, &mut p0)
        }
        if p2[1] < p1[1] {
            swap(&mut p2, &mut p1)
        }

        let x0 = p0[0];
        let y0 = p0[1];
        let h0 = h[0];
        let x1 = p1[0];
        let y1 = p1[1];
        let h1 = h[1];
        let x2 = p2[0];
        let y2 = p2[1];
        let h2 = h[2];

        let mut x012 = interpolate(y0, x0 as f32, y1, x1 as f32);
        let mut h012 = interpolate(y0, h0, y1, h1);
        let mut x12 = interpolate(y1, x1 as f32, y2, x2 as f32);
        let mut h12 = interpolate(y1, h1 as f32, y2, h2 as f32);
        let x02 = interpolate(y0, x0 as f32, y2, x2 as f32);
        let h02 = interpolate(y0, h0 as f32, y2, h2 as f32);

        x012.pop();
        x012.append(&mut x12);

        h012.pop();
        h012.append(&mut h12);

        let m = (x012.len() as f32 / 2.0).floor() as usize;
        let x_left: Vec<f32>;
        let h_left: Vec<f32>;
        let x_right: Vec<f32>;
        let h_right: Vec<f32>;
        if x02[m] < x012[m] {
            x_left = x02;
            h_left = h02;

            x_right = x012;
            h_right = h012;
        } else {
            x_left = x012;
            h_left = h012;

            x_right = x02;
            h_right = h02;
        }

        for y in y0..y2 - 1 {
            let x_l = x_left[(y - y0) as usize] as u32;
            let x_r = x_right[(y - y0) as usize] as u32;

            let h_segment = interpolate(
                x_l as u32,
                h_left[(y - y0) as usize] as f32,
                x_r as u32,
                h_right[(y - y0) as usize] as f32,
            );
            for x in x_left[(y - y0) as usize] as u32..x_right[(y - y0) as usize] as u32 {
                let shaded_col = [
                    (col[0] as f32 * h_segment[(x - x_l) as usize]) as u8,
                    (col[1] as f32 * h_segment[(x - x_l) as usize]) as u8,
                    (col[2] as f32 * h_segment[(x - x_l) as usize]) as u8,
                ];
                self.put_pixel(x, y, shaded_col);
            }
        }
    }

    fn draw_test(&mut self) {
        self.put_pixel(10, 10, [0xff, 0x80, 0x00]);

        let func = |i: f32| -> f32 {
            i.abs().powf(2.0 / 3.0)
                + (8.0 - i.abs().powf(2.0)).abs().powf(1.0 / 2.0) * (16.0 * 3.1415926 * i).sin()
        };
        self.draw_func(func, [0xff, 0x80, 0xff], 60.0, 60.0, 0.0, 0.0);

        let func = |i: f32| -> f32 { i.sin() };

        self.draw_func(func, [0x80, 0xff, 0x00], 20.0, 100.0, 0.0, 0.0);

        self.draw_line([150, 200], [300, 400], [0xff, 0x00, 0x80]);
        self.draw_wire_tri([200, 300], [400, 400], [200, 200], [0x80, 0x80, 0xff]);
        self.draw_line([150, 400], [400, 300], [0xff, 0x00, 0x80]);

        self.draw_filled_tri([100, 400], [150, 320], [70, 350], [0xff, 0x80, 0x80]);
        self.draw_gradient_tri(
            [100, 300],
            [150, 220],
            [70, 250],
            [0xff, 0x80, 0x80],
            [0.0, 0.1, 1.0],
        );

        self.draw_text(
            "Poggers! Wowee!",
            self.defaultfont.clone(),
            100,
            100,
            [0xff; 3],
        );
        self.draw_text(
            "='.@\"#$^&*()",
            self.defaultfont.clone(),
            100,
            120,
            [0xff; 3],
        );
    }

    pub fn draw(&mut self) {
        self.clear([0x00; 3]);
        self.draw_test();
    }
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Pogging my pants")
            .with_inner_size(size)
            .with_resizable(false)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let mut buffer: Buffer = Buffer::new(
        vec![0; (4 * WIDTH * HEIGHT) as usize],
        Font::new("tamzen.bdf"),
    );

    /*let mut window = Window::new("pogging my pants", WIDTH, HEIGHT, WindowOptions::default()).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));*/

    let mut time = Instant::now();


    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            buffer.draw();
            buffer.draw_text(
                &format!(
                    "{}",
                    Instant::now().duration_since(time).as_micros() as f32 / 1000.0
                ),
                buffer.defaultfont.clone(),
                5,
                5,
                [0xff; 3],
            );
            time = Instant::now();
            pixels
                .get_frame()
                .copy_from_slice(buffer.into_vec_mut().as_slice());

            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // window.request_redraw();
    });
}
