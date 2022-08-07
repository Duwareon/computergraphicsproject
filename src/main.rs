mod text;

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use std::time::Instant;
use text::Font;

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;

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

        self.draw_text(
            "Poggers! Wowee!",
            self.defaultfont.clone(),
            100,
            100,
            [0xff; 3],
        );
        self.draw_text(
            "'.@\"#$^&*()",
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
        Font::new("tamzen-12.bdf"),
    );

    buffer.clone().defaultfont.shape_char('8');

    /*let mut window = Window::new("pogging my pants", WIDTH, HEIGHT, WindowOptions::default()).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));*/

    let mut time = Instant::now();

    //println!("{}, {}", buffer.buf.len(), pixels.get_frame().len());

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            buffer.draw();
            buffer.draw_text(&Instant::now().duration_since(time).as_millis().to_string(), buffer.defaultfont.clone(), 5, 5, [0xff; 3]);
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

        window.request_redraw();
    });
}
