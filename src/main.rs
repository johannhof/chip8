//#![deny(warnings)]

#![feature(link_args)]

#[link_args = "-s USE_SDL=2 -s ASYNCIFY=1"]
extern "C" {
    pub fn emscripten_sleep(dur: c_int);
}

extern crate sdl2;
extern crate chip8;

use std::os::raw::c_int;
use std::io::Cursor;
use sdl2::rect::Rect;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::EventPump;
use sdl2::keyboard::Scancode;
use chip8::Chip8;

fn set_keys(event_pump: &mut EventPump) {
    let is_pressed = event_pump.keyboard_state().is_scancode_pressed(Scancode::A);
}

fn draw(canvas: &mut Canvas<Window>, pixels: &[u8; 2048]) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for (i, pixel) in pixels.iter().enumerate() {
        let y = i / 64;
        let x = i - 64 * y;
        if *pixel != 0 {
            canvas.fill_rect(Rect::new(x as i32, y as i32, 1, 1)).unwrap();
        }
    }

    canvas.present();
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let window = video_subsystem.window("Example", 64, 32).build().unwrap();

    let mut game = Chip8::new();

    let mut buff = Cursor::new(vec![
                               0xA2,
                               0x06,
                               0xD0,
                               0x0A,
                               0xF1,
                               0x0A,

        0xF0, 0x90, 0x90, 0x90, 0xF0,
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
    ]);

    //let mut buff = Cursor::new(vec![0xF1, 0x0A]);
    game.load(&mut buff);

    // Let's create a Canvas which we will use to draw in our Window
    let mut canvas: Canvas<Window> = window.into_canvas()
        .present_vsync() //< this means the screen cannot
        // render faster than your display rate (usually 60Hz or 144Hz)
        .build().unwrap();

    loop {
        game.cycle();
        draw(&mut canvas, &game.gfx);
        set_keys(&mut event_pump);
        unsafe {
            emscripten_sleep(1000 / 60);
        }
    }
}
