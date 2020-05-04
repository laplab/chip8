use crate::display::{SCREEN_HEIGHT, SCREEN_WIDTH, Display};
use crate::keypad::Keypad;
use crate::cpu::Cpu;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::rect::Rect;
use sdl2::keyboard::Keycode;
use sdl2::render::WindowCanvas;
use sdl2::EventPump;

use maplit::hashmap;

use std::time::{Duration, Instant};

const SCALE: usize = 8;
const BACKGROUND: Color = Color::RGB(10, 61, 98);
const CELL: Color = Color::RGB(130, 204, 221);

pub enum EmulatorError {
    ReadProgramError(std::io::Error),
    RuntimeError(String),
}

pub struct Emulator {
    canvas: WindowCanvas,
    events: EventPump,
}

impl Emulator {
    pub fn new() -> Emulator {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("chip8", (SCREEN_WIDTH * SCALE) as u32, (SCREEN_HEIGHT * SCALE) as u32)
            .position_centered()
            .build()
            .unwrap();

        let canvas = window.into_canvas().build().unwrap();
        let events = sdl_context.event_pump().unwrap();

        Emulator { canvas, events }
    }

    pub fn run(&mut self, filename: &str) -> Result<(), EmulatorError> {
        let mut keypad = Keypad::new();
        let mut display = Display::new();
        let mut cpu = match Cpu::new(filename) {
            Ok(c) => c,
            Err(e) => return Err(EmulatorError::ReadProgramError(e)),
        };

        let keymap = hashmap!{
            Keycode::Num1 => 1,
            Keycode::Num2 => 2,
            Keycode::Num3 => 3,
            Keycode::Num4 => 0xC,
            Keycode::Q => 4,
            Keycode::W => 5,
            Keycode::E => 6,
            Keycode::R => 0xD,
            Keycode::A => 7,
            Keycode::S => 8,
            Keycode::D => 9,
            Keycode::F => 0xE,
            Keycode::Z => 0xA,
            Keycode::X => 0,
            Keycode::C => 0xB,
            Keycode::V => 0xF,
        };

        let mut last_rendered = Instant::now();
        let render_frequency = Duration::from_nanos(10u64.pow(9) / 60);

        let mut last_computed = Instant::now();
        let compute_frequency = Duration::from_nanos(10u64.pow(9) / 500);

        'running: loop {
            // Read keyboard events
            for event in self.events.poll_iter() {
                match event {
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running
                    },
                    Event::KeyDown { keycode: Some(keycode), .. } => {
                        if let Some(&index) = keymap.get(&keycode) {
                            keypad.press(index);
                        }
                    },
                    Event::KeyUp { keycode: Some(keycode), .. } => {
                        if let Some(&index) = keymap.get(&keycode) {
                            keypad.release(index);
                        }
                    },
                    _ => {}
                }
            }

            let now = Instant::now();
            if (now - last_computed) > compute_frequency {
                // Run one cycle of game
                match cpu.run_cycle(&mut display, &keypad) {
                    None => (),
                    Some(e) => return Err(EmulatorError::RuntimeError(e)),
                };
                last_computed = now;
            }

            let now = Instant::now();
            if (now - last_rendered) > render_frequency {
                // Compute rectangles to draw
                let mut rects = vec![];
                for y in 0..SCREEN_HEIGHT {
                    for x in 0..SCREEN_WIDTH {
                        if display.get_pixel(x, y) == 0 {
                            continue;
                        }

                        let rect_x = (x * SCALE) as i32;
                        let rect_y = (y * SCALE) as i32;
                        let dimension = SCALE as u32;
                        let rect = Rect::new(rect_x, rect_y, dimension, dimension);
                        rects.push(rect);
                    }
                }

                // Clear everything from canvas
                self.canvas.set_draw_color(BACKGROUND);
                self.canvas.clear();

                // Draw computed rectangles
                self.canvas.set_draw_color(CELL);
                self.canvas.fill_rects(rects.as_slice()).unwrap();

                // Render result
                self.canvas.present();
                last_rendered = now;
            }
        }

        Ok(())
    }
}