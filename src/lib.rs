#![allow(non_snake_case)]

const FONT_SET: [u8; 80] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Chip8 {
    memory: [u8; 4096],

    V: [u8; 16],

    I: u16,
    pc: u16,

    /// Array that holds the screen content in a 64x32 format.
    /// I.e. every 64th index marks the beginning of a new line.
    pub gfx: [u8; 64 * 32],

    delay_timer: u8,
    sound_timer: u8,

    stack: [u16; 16],
    sp: u16,

    key: [u8; 16],

    awaiting_key_press: bool,
}

impl Chip8 {
    pub fn new() -> Chip8 {
       let mut game = Chip8 {
            memory: [0; 4096],
            V: [0; 16],
            I: 0,
            pc: 0x200,
            gfx: [0; 64 * 32],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            key: [0; 16],
            awaiting_key_press: false,
       };

       for (i, val) in FONT_SET.iter().enumerate() {
           game.memory[i] = *val;
       }

       game
    }

    pub fn load<T: std::io::Read>(&mut self, reader: &mut T) -> std::io::Result<usize> {
        reader.read(&mut self.memory[0x200..])
    }

    pub fn cycle(&mut self) {
        let opcode: u16 = (self.memory[self.pc as usize] as u16) << 8 | (self.memory[(self.pc + 1) as usize] as u16);

        match opcode & 0xF000 {
            0x0000 => {
                match opcode {
                    // 00E0 - Clears the screen.
                    // TODO
                    0x00E0 => {},
                    // 00EE - Returns from a subroutine.
                    0x00EE => {
                        self.sp -= 1;
                        self.pc = self.stack[self.sp as usize] + 2;
                    },
                    _ => panic!("Opcode {} not matched.", opcode),
                };
            },
            // 1NNN - Jumps to address NNN.
            0x1000 => self.pc = opcode & 0x0FFF,

            // 2NNN - Calls subroutine at NNN.
            0x2000 => {
                // Put the current routine on the stack.
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = opcode & 0x0FFF;
            },

            // 3XNN - Skips the next instruction if VX equals NN. (Usually the next instruction is a jump to skip a code block)
            0x3000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;

                if self.V[x] == nn {
                    self.pc += 2;
                }
                self.pc += 2;
            },

            // 4XNN - Skips the next instruction if VX doesn't equal NN. (Usually the next instruction is a jump to skip a code block)
            0x4000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;

                if self.V[x] != nn {
                    self.pc += 2;
                }
                self.pc += 2;
            },

            // 5XY0 - Skips the next instruction if VX equals VY. (Usually the next instruction is a jump to skip a code block)
            0x5000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;

                if self.V[x] == self.V[y] {
                    self.pc += 2;
                }
                self.pc += 2;
            },

            // 6XNN - Sets VX to NN.
            0x6000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;

                self.V[x] = nn;
                self.pc += 2;
            },

            // 7XNN - Adds NN to VX. (Carry flag is not changed)
            0x7000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;

                self.V[x] = self.V[x].overflowing_add(nn).0;
                self.pc += 2;
            },

            0x8000 => {
                self.pc += 2;
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                match opcode & 0x000F {
                    // 8XY0 - Sets VX to the value of VY.
                    0 => self.V[x] = self.V[y],
                    // 8XY1 - Sets VX to VX or VY. (Bitwise OR operation).
                    1 => self.V[x] |= self.V[y],
                    // 8XY2 - Sets VX to VX and VY. (Bitwise AND operation)
                    2 => self.V[x] &= self.V[y],
                    // 8XY3 - Sets VX to VX xor VY.
                    3 => self.V[x] ^= self.V[y],
                    // 8XY4 - Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't.
                    4 => {
                        let (vx, carry) = self.V[x].overflowing_add(self.V[y]);
                        self.V[x] = vx;
                        self.V[0xF] = if carry { 1 } else { 0 };
                    },
                    // 8XY5 - VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there isn't.
                    5 => {
                        let (vx, carry) = self.V[x].overflowing_sub(self.V[y]);
                        self.V[x] = vx;
                        self.V[0xF] = if carry { 1 } else { 0 };
                    },
                    // 8XY6 - Shifts VY right by one and copies the result to VX. VF is set to the value of the least significant bit of VY before the shift.
                    6 => {
                        self.V[0xF] = self.V[y] & 1;
                        self.V[y] >>= 1;
                        self.V[x] = self.V[y];
                    },
                    // 8XY7 - Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there isn't.
                    7 => {
                        if self.V[y] > self.V[x] {
                            self.V[0xF] = 1;
                        } else {
                            self.V[0xF] = 0;
                        }

                        self.V[x] = self.V[y] - self.V[x];
                    },
                    // 8XYE - Shifts VY left by one and copies the result to VX. VF is set to the value of the most significant bit of VY before the shift.
                    0xE => {
                        self.V[0xF] = self.V[y] & 0b10000000;
                        self.V[y] <<= 1;
                        self.V[x] = self.V[y];
                    },
                    _ => panic!("Opcode {} not matched.", opcode),
                };
            },

            // 9XY0 - Skips the next instruction if VX doesn't equal VY. (Usually the next instruction is a jump to skip a code block)
            0x9000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;

                if self.V[x] != self.V[y] {
                    self.pc += 2;
                }
                self.pc += 2;
            },

            // ANNN - Sets I to the address NNN.
            0xA000 => {
                self.I = opcode & 0x0FFF;
                self.pc += 2;
            },

            // BNNN - Jumps to the address NNN plus V0.
            0xB000 => self.pc = opcode & 0x0FFF + self.V[0] as u16,

            // DXYN - Draws a sprite at coordinate (VX, VY) that has
            // a width of 8 pixels and a height of N pixels. Each row
            // of 8 pixels is read as bit-coded starting from memory
            // location I; I value doesn’t change after the execution
            // of this instruction. As described above, VF is set to 1
            // if any screen pixels are flipped from set to unset when
            // the sprite is drawn, and to 0 if that doesn’t happen
            0xD000 => {
                // Reset collision flag.
                self.V[0xF] = 0;

                let X = ((opcode & 0x0F00) >> 8) as u16;
                let Y = ((opcode & 0x00F0) >> 4) as u16;
                let N = (opcode & 0x000F) as u16;

                for y in 0..N {
                    // Gets the sprite, e.g. 0b00111100.
                    let sprite = self.memory[(self.I + y as u16) as usize];
                    for x in 0..8 {
                        // Gets the pixel by masking with a single bit shifted
                        // to the correct position.
                        // I.e. to find if the 5th pixel in 0b00111100 is set,
                        // we mask with 0b10000000 >> 4, so 0b00001000.
                        let pixel = sprite & (0x80 >> x);
                        // The gfx position we want to write to.
                        let pos = ((y + Y) * 64 + x + X) as usize;

                        // Set flag for collision detection.
                        if pixel != 0 && self.gfx[pos] != 0 {
                            self.V[0xF] = 1;
                        }

                        // Set the pixel.
                        if pixel == 0 {
                           self.gfx[pos] = 0;
                        } else {
                           self.gfx[pos] = 1;
                        };
                    }
                }
                self.pc += 2;
            },

            0xF000 => {
                // Extract the X from FX..
                let x = ((opcode & 0x0F00) >> 8) as u8;
                match opcode & 0x00FF {
                    // FX07 - Sets VX to the value of the delay timer.
                    0x07 => {
                        self.V[x as usize] = self.delay_timer;
                        self.pc += 2;
                    },
                    // FX0A - A key press is awaited, and then stored in VX.
                    // Blocking Operation. All instruction halted until
                    // next key event.
                    0x0A => {
                        self.awaiting_key_press = true;
                        for (i, key) in self.key.iter().enumerate() {
                            if *key != 0 {
                                self.awaiting_key_press = false;
                                self.pc += 2;
                                self.V[x as usize] = i as u8;
                                break;
                            }
                        }
                    },
                    _ => panic!("Opcode {} not matched.", opcode),
                };
            },
            _ => panic!("Opcode {} not matched.", opcode),
        };

        // Count down timers.
        if self.delay_timer > 0 && !self.awaiting_key_press {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 && !self.awaiting_key_press {
            self.sound_timer -= 1;
        }
    }
}

#[test]
#[ignore]
fn test_00E0() {
}

#[test]
#[ignore]
fn test_00EE() {
}

#[test]
#[ignore]
fn test_1NNN() {
}

#[test]
fn test_2NNN() {
    let mut game = Chip8::new();
    game.memory[0x200] = 0x21;
    game.memory[0x201] = 0x23;

    game.cycle();

    assert_eq!(game.stack[0], 0x200);
    assert_eq!(game.stack[1], 0);
    assert_eq!(game.sp, 1);
    assert_eq!(game.pc, 0x123);
}

#[test]
// 3XNN - Skips the next instruction if VX equals NN. (Usually the next instruction is a jump to skip a code block)
fn test_3XNN() {
    let mut game = Chip8::new();
    game.memory[0x200] = 0x31;
    game.memory[0x201] = 0x42;
    game.memory[0x202] = 0x00;
    game.memory[0x203] = 0x00;
    game.memory[0x204] = 0x32;
    game.memory[0x205] = 0x42;

    game.V[1] = 0x42;
    game.V[2] = 0x22;

    game.cycle();

    assert_eq!(game.pc, 0x204);

    game.cycle();

    assert_eq!(game.pc, 0x206);
}

#[test]
// 4XNN - Skips the next instruction if VX doesn't equal NN. (Usually the next instruction is a jump to skip a code block)
fn test_4XNN() {
    let mut game = Chip8::new();
    game.memory[0x200] = 0x41;
    game.memory[0x201] = 0x42;
    game.memory[0x202] = 0x00;
    game.memory[0x203] = 0x00;
    game.memory[0x204] = 0x42;
    game.memory[0x205] = 0x42;

    game.V[1] = 0x22;
    game.V[2] = 0x42;

    game.cycle();

    assert_eq!(game.pc, 0x204);

    game.cycle();

    assert_eq!(game.pc, 0x206);
}

#[test]
// 5XY0 - Skips the next instruction if VX equals VY. (Usually the next instruction is a jump to skip a code block)
fn test_5XY0() {
    let mut game = Chip8::new();
    game.memory[0x200] = 0x51;
    game.memory[0x201] = 0x20;
    game.memory[0x202] = 0x00;
    game.memory[0x203] = 0x00;
    game.memory[0x204] = 0x53;
    game.memory[0x205] = 0x40;

    game.V[1] = 0x42;
    game.V[2] = 0x42;
    game.V[3] = 0x22;
    game.V[4] = 0x42;

    game.cycle();

    assert_eq!(game.pc, 0x204);

    game.cycle();

    assert_eq!(game.pc, 0x206);
}

#[test]
// 6XNN - Sets VX to NN.
fn test_6XNN() {
    let mut game = Chip8::new();
    game.memory[0x200] = 0x61;
    game.memory[0x201] = 0x23;

    game.cycle();

    assert_eq!(game.V[1], 0x23);
    assert_eq!(game.pc, 0x202);
}

#[test]
// 7XNN - Adds NN to VX. (Carry flag is not changed).
fn test_7XNN() {
    let mut game = Chip8::new();
    game.memory[0x200] = 0x71;
    game.memory[0x201] = 0x23;
    game.memory[0x202] = 0x71;
    game.memory[0x203] = 0xFF;

    game.cycle();

    assert_eq!(game.V[0xF], 0);
    assert_eq!(game.V[1], 0x23);
    assert_eq!(game.pc, 0x202);

    game.cycle();

    assert_eq!(game.V[0xF], 0);
    assert_eq!(game.V[1], 0x22);
    assert_eq!(game.pc, 0x204);
}

#[test]
// 8XY0 - Sets VX to the value of VY.
fn test_8XY0() {
    let mut game = Chip8::new();
    game.memory[0x200] = 0x81;
    game.memory[0x201] = 0x20;
    game.V[0x2] = 0x42;

    game.cycle();

    assert_eq!(game.V[0x2], 0x42);
    assert_eq!(game.V[0x1], 0x42);
    assert_eq!(game.pc, 0x202);
}

#[test]
// 8XY4 - Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't.
fn test_8XY4() {
    let mut game = Chip8::new();
    game.memory[0x200] = 0x81;
    game.memory[0x201] = 0x24;
    game.memory[0x202] = 0x81;
    game.memory[0x203] = 0x34;
    game.V[0x1] = 0x42;
    game.V[0x2] = 0x16;
    game.V[0x3] = 0xFF;

    game.cycle();

    assert_eq!(game.V[0xF], 0);
    assert_eq!(game.V[0x1], 0x58);
    assert_eq!(game.pc, 0x202);

    game.cycle();

    assert_eq!(game.V[0xF], 1);
    assert_eq!(game.V[0x1], 0x57);
    assert_eq!(game.pc, 0x204);
}

#[test]
// 8XY5 - VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there isn't.
fn test_8XY5() {
    let mut game = Chip8::new();
    game.memory[0x200] = 0x81;
    game.memory[0x201] = 0x25;
    game.memory[0x202] = 0x81;
    game.memory[0x203] = 0x35;
    game.V[0x1] = 0x42;
    game.V[0x2] = 0x16;
    game.V[0x3] = 0xFF;

    game.cycle();

    assert_eq!(game.V[0xF], 0);
    assert_eq!(game.V[0x1], 0x26);
    assert_eq!(game.pc, 0x202);

    game.cycle();

    assert_eq!(game.V[0xF], 1);
    assert_eq!(game.V[0x1], 0x27);
    assert_eq!(game.pc, 0x204);
}

#[test]
// 9XY0 - Skips the next instruction if VX doesn't equal VY. (Usually the next instruction is a jump to skip a code block)
fn test_9XY0() {
    let mut game = Chip8::new();
    game.memory[0x200] = 0x91;
    game.memory[0x201] = 0x20;
    game.memory[0x202] = 0x00;
    game.memory[0x203] = 0x00;
    game.memory[0x204] = 0x93;
    game.memory[0x205] = 0x40;

    game.V[1] = 0x22;
    game.V[2] = 0x42;
    game.V[3] = 0x42;
    game.V[4] = 0x42;

    game.cycle();

    assert_eq!(game.pc, 0x204);

    game.cycle();

    assert_eq!(game.pc, 0x206);
}
#[test]
fn test_ANNN() {
    let mut game = Chip8::new();
    game.memory[0x200] = 0xA1;
    game.memory[0x201] = 0x23;

    game.cycle();

    assert_eq!(game.I, 0x123);
    assert_eq!(game.pc, 0x202);
}

#[test]
#[ignore]
fn test_BNNN() {
}

#[test]
fn test_DXYN() {
    let mut game = Chip8::new();
    game.memory[0x200] = 0xD0;
    game.memory[0x201] = 0x04;

    game.memory[0x210] = 0b00011000;
    game.memory[0x211] = 0b00111100;
    game.memory[0x212] = 0b01111110;
    game.memory[0x213] = 0b11111111;

    game.I = 0x210;

    game.cycle();

    assert_eq!(game.gfx[0..8], [
               0,0,0,1,1,0,0,0, ]);
    assert_eq!(game.gfx[64..64 + 8], [
               0,0,1,1,1,1,0,0, ]);
    assert_eq!(game.gfx[2 * 64..2 * 64 + 8], [
               0,1,1,1,1,1,1,0, ]);
    assert_eq!(game.gfx[3 * 64..3 * 64 + 8], [
               1,1,1,1,1,1,1,1, ]);
    assert_eq!(game.I, 0x210);
    assert_eq!(game.pc, 0x202);

    let mut game = Chip8::new();
    game.memory[0x200] = 0xD0;
    game.memory[0x201] = 0x05;

    game.memory[0x210] = 0xF0;
    game.memory[0x211] = 0x90;
    game.memory[0x212] = 0x90;
    game.memory[0x213] = 0x90;
    game.memory[0x214] = 0xF0;

    game.I = 0x210;

    game.cycle();

    assert_eq!(game.gfx[0..8], [
               1,1,1,1,0,0,0,0, ]);
    assert_eq!(game.gfx[64..64 + 8], [
               1,0,0,1,0,0,0,0, ]);
    assert_eq!(game.gfx[2 * 64..2 * 64 + 8], [
               1,0,0,1,0,0,0,0, ]);
    assert_eq!(game.gfx[3 * 64..3 * 64 + 8], [
               1,0,0,1,0,0,0,0, ]);
    assert_eq!(game.gfx[4 * 64..4 * 64 + 8], [
               1,1,1,1,0,0,0,0, ]);
    assert_eq!(game.I, 0x210);
    assert_eq!(game.pc, 0x202);
}

#[test]
// FX07 - Sets VX to the value of the delay timer.
fn test_FX07() {
    let mut game = Chip8::new();
    game.memory[0x200] = 0xF3;
    game.memory[0x201] = 0x07;
    game.delay_timer = 23;

    game.cycle();

    assert_eq!(game.V[0x3], 23);
    assert_eq!(game.pc, 0x202);
}

#[test]
// FX0A - A key press is awaited, and then stored in VX.
// Blocking Operation. All instruction halted until
// next key event.
fn test_FX0A() {
    let mut game = Chip8::new();
    game.memory[0x200] = 0xF4;
    game.memory[0x201] = 0x0A;
    game.delay_timer = 5;

    game.cycle();

    assert_eq!(game.delay_timer, 5);

    game.cycle();

    assert_eq!(game.delay_timer, 5);

    game.key[0x8] = 1;

    game.cycle();

    assert_eq!(game.V[0x4], 0x8);
    assert_eq!(game.pc, 0x202);
}
