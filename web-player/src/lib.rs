mod utils;

use image::{GenericImage, RgbaImage};
use js_sys::ArrayBuffer;
use std::f32::consts;
use std::io::Cursor;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn init() {
    utils::set_panic_hook();
    wasm_logger::init(wasm_logger::Config::new(log::Level::Debug));
}

#[wasm_bindgen]
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[wasm_bindgen]
pub struct Cdg {
    scsi: cdg::SubchannelStreamIter<Cursor<Vec<u8>>>,
    interpreter: cdg_renderer::CdgInterpreter,
    image: RgbaImage,
    frame: Vec<u8>,
    i: f32,
}

#[wasm_bindgen]
impl Cdg {
    pub fn new(source: ArrayBuffer) -> Cdg {
        let typebuf = js_sys::Uint8Array::new(&source);
        let mut body = vec![0; typebuf.length() as usize];
        typebuf.copy_to(&mut body[..]);

        log::info!("CDG length: {}", body.len());

        let scsi = cdg::SubchannelStreamIter::new(Cursor::new(body));
        let interpreter = cdg_renderer::CdgInterpreter::new();
        let image = RgbaImage::new(300, 216);
        let frame = Vec::new();

        Cdg {
            scsi,
            interpreter,
            image,
            frame,
            i: 0.0,
        }
    }

    pub fn next_frame(&mut self, sectors_since: isize) {
        for _ in 0..sectors_since {
            let sector = self.scsi.next();

            //Break the loop once no more sectors exist to render
            if let Some(s) = sector {
                for cmd in s {
                    self.interpreter.handle_cmd(cmd);
                }
            }
        }
        self.image.copy_from(&self.interpreter, 0, 0);
        self.frame = self.image.clone().into_raw();
    }

    pub fn frame(&self) -> *const u8 {
        self.frame.as_ptr()
    }

    //Sine wave formula for rainbow cycling background color
    pub fn rainbow_cycle(&mut self) -> Rgba {
        self.i = if (self.i + 1.0) % 4096.0 == 0.0 {
            0.0
        } else {
            self.i + 1.0
        };
        let r = ((consts::PI / 4096.0 * 2.0 * self.i + 0.0 * consts::PI / 3.0).sin() * 127.0)
            .floor()
            + 128.0;
        let g = ((consts::PI / 4096.0 * 2.0 * self.i + 4.0 * consts::PI / 3.0).sin() * 127.0)
            .floor()
            + 128.0;
        let b = ((consts::PI / 4096.0 * 2.0 * self.i + 8.0 * consts::PI / 3.0).sin() * 127.0)
            .floor()
            + 128.0;
        let a = 1.0;

        Rgba { r, g, b, a }
    }
}
