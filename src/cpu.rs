use rand::Rng;
use rand::rngs::ThreadRng;

use std::fs::File;
use std::io::{Read, Error};

use crate::display::Display;
use crate::keypad::Keypad;

// Few examples to show how fontset work:
// DEC   HEX    BIN         RESULT    DEC   HEX    BIN         RESULT
// 240   0xF0   1111 0000    ****     240   0xF0   1111 0000    ****
// 144   0x90   1001 0000    *  *      16   0x10   0001 0000       *
// 144   0x90   1001 0000    *  *      32   0x20   0010 0000      *
// 144   0x90   1001 0000    *  *      64   0x40   0100 0000     *
// 240   0xF0   1111 0000    ****      64   0x40   0100 0000     *
const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Cpu {
    // Memory map:
    // 0x000-0x1FF - Chip 8 interpreter (contains font set in emu)
    // 0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
    // 0x200-0xFFF - Program ROM and work RAM
    memory: [u8; 4096],

    // Stack and stack pointer (sp)
    stack: [usize; 16],
    sp: usize,

    // Registers
    registers: [u8; 16],
    index: usize,
    pc: usize,
    delay_timer: u8,
    sound_timer: u8,

    rand: ThreadRng,
}

impl Cpu {
    pub fn new(filename: &str) -> Result<Cpu, Error> {
        let mut memory = [0; 4096];

        // First 80 cells of memory are reserved for fontset
        for (i, &font_value) in FONTSET.iter().enumerate() {
            memory[i] = font_value;
        }

        // Read program into memory
        let mut file = File::open(filename)?;
        file.read(&mut memory[0x200..])?;

        rand::thread_rng();

        Ok(Cpu {
            memory,
            stack: [0; 16],
            sp: 0,
            registers: [0; 16],
            index: 0,
            pc: 0x200,
            delay_timer: 0,
            sound_timer: 0,
            rand: rand::thread_rng(),
        })
    }

    fn read_opcode(&self) -> u16 {
        ((self.memory[self.pc] as u16) << 8) | (self.memory[(self.pc + 1) as usize] as u16)
    }

    pub fn run_cycle(&mut self, display: &mut Display, keypad: &Keypad) -> Option<String> {
        let opcode = self.read_opcode();
        self.pc += 2;

        // Split opcode in 4 parts to easily match arguments
        let op1 = (opcode & 0xF000) >> 12;
        let op2 = (opcode & 0x0F00) >> 8;
        let op3 = (opcode & 0x00F0) >> 4;
        let op4 = opcode & 0x000F;

        let x = op2 as usize;
        let y = op3 as usize;

        let last = (opcode & 0x000F) as u8;
        let last_two = (opcode & 0x00FF) as u8;
        let last_three = (opcode & 0x0FFF) as u16;

        match (op1, op2, op3, op4) {
            (0, 0, 0xE, 0xE) => {
                self.sp -= 1;
                self.pc = self.stack[self.sp];
            },
            (0, 0, 0xE, 0) => {
                display.clear();
            },
            (0, _, _, _) => {
                return Some(String::from("Call to RCA 1802 programs is not supported"));
            },
            (1, _, _, _) => {
                self.pc = last_three as usize;
            },
            (2, _, _, _) => {
                self.stack[self.sp] = self.pc;
                self.sp += 1;
                self.pc = last_three as usize;
            },
            (3, _, _, _) => {
                if self.registers[x] == last_two {
                    self.pc += 2;
                }
            },
            (4, _, _, _) => {
                if self.registers[x] != last_two {
                    self.pc += 2;
                }
            },
            (5, _, _, 0) => {
                if self.registers[x] == self.registers[y] {
                    self.pc += 2;
                }
            },
            (6, _, _, _) => {
                self.registers[x] = last_two;
            },
            (7, _, _, _) => {
                self.registers[x] = self.registers[x].wrapping_add(last_two);
            },
            (8, _, _, 0) => {
                self.registers[x] = self.registers[y];
            },
            (8, _, _, 1) => {
                self.registers[x] |= self.registers[y];
            },
            (8, _, _, 2) => {
                self.registers[x] &= self.registers[y];
            },
            (8, _, _, 3) => {
                self.registers[x] ^= self.registers[y];
            },
            (8, _, _, 4) => {
                let (result, overflow) = self.registers[x].overflowing_add(self.registers[y]);
                self.registers[0xF] = overflow as u8;
                self.registers[x] = result;
            },
            (8, _, _, 5) => {
                let (result, overflow) = self.registers[x].overflowing_sub(self.registers[y]);
                self.registers[0xF] = !overflow as u8;
                self.registers[x] = result;
            },
            (8, _, _, 6) => {
                self.registers[0xF] = self.registers[x] & 1;
                self.registers[x] >>= 1;
            },
            (8, _, _, 7) => {
                let (result, overflow) = self.registers[y].overflowing_sub(self.registers[x]);
                self.registers[0xF] = !overflow as u8;
                self.registers[x] = result;
            },
            (8, _, _, 0xE) => {
                self.registers[0xF] = (self.registers[x] & 0x80) >> 7;
                self.registers[x] <<= 1;
            },
            (9, _, _, 0) => {
                if self.registers[x] != self.registers[y] {
                    self.pc += 2;
                }
            },
            (0xA, _, _, _) => {
                self.index = last_three as usize;
            },
            (0xB, _, _, _) => {
                self.pc = (self.registers[0] as u16 + last_three) as usize;
            },
            (0xC, _, _, _) => {
                self.registers[x] = self.rand.gen::<u8>() & last_two;
            }
            (0xD, _, _, _) => {
                let collision = display.draw_sprite(self.registers[x] as usize, self.registers[y] as usize, &self.memory[self.index..self.index + last as usize]);
                self.registers[0xF] = collision as u8;
            },
            (0xE, _, 9, 0xE) => {
                if keypad.is_pressed(self.registers[x] as usize) {
                    self.pc += 2;
                }
            },
            (0xE, _, 0xA, 1) => {
                if !keypad.is_pressed(self.registers[x] as usize) {
                    self.pc += 2;
                }
            },
            (0xF, _, 0, 7) => {
                self.registers[x] = self.delay_timer;
            },
            (0xF, _, 0, 0xA) => {
                self.pc -= 2;
                if let Some(key) = keypad.which_pressed() {
                    self.registers[x] = key as u8;
                    self.pc += 2;
                }
            },
            (0xF, _, 1, 5) => {
                self.delay_timer = self.registers[x];
            },
            (0xF, _, 1, 8) => {
                self.sound_timer = self.registers[x];
            },
            (0xF, _, 1, 0xE) => {
                self.index += self.registers[x] as usize;
                if self.index > 0xFFF {
                    self.registers[0xF] = 1;
                    self.index -= 0xFFF;
                } else {
                    self.registers[0xF] = 0;
                }
            },
            (0xF, _, 2, 9) => {
                // Each symbol is represented with 5 bytes. To get start of
                // the index for character x, you need to multiply it by 5
                self.index = self.registers[x] as usize * 5;
            },
            (0xF, _, 3, 3) => {
                self.memory[self.index] = self.registers[x] / 100;
                self.memory[self.index + 1] = (self.registers[x] / 10) % 10;
                self.memory[self.index + 2] = self.registers[x] % 10;
            },
            (0xF, _, 5, 5) => {
                self.memory[self.index..=(self.index + x)].copy_from_slice(&self.registers[..=x]);
            },
            (0xF, _, 6, 5) => {
                self.registers[..=x].copy_from_slice(&self.memory[self.index..=(self.index + x)]);
            },
            _ => {
                return Some(format!("Unknown operation {:#x}", opcode));
            },
        };

        // Update timers
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // TODO: beep something
            }
            self.sound_timer -= 1;
        }

        None
    }
}