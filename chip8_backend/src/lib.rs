use rand::random;


pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const START_ADDR: u16 = 0x200;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
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


pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt: u8,
    st: u8
}

impl Emu {
    pub fn new() -> Self {
        let mut new_emu: Emu = Self {
            pc: (START_ADDR),
            ram: ([0; RAM_SIZE]),
            screen: ([false; SCREEN_WIDTH * SCREEN_HEIGHT]),
            v_reg: ([0; NUM_REGS]),
            i_reg: (0),
            sp: (0),
            stack: ([0; STACK_SIZE]),
            keys: ([false; NUM_KEYS]),
            dt: (0),
            st: (0)
        };
        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        return new_emu;
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    fn push(&mut self, val: u16) {
        if self.sp as usize == STACK_SIZE {
            panic!("push");
        }
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        if self.sp == 0 {
            panic!("pop");
        }
        self.sp -= 1;
        return self.stack[self.sp as usize];
    }

    pub fn tick(&mut self) {
        let op = self.fetch();
        
        self.execute(op);
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        self.pc += 2;
        let op = lower_byte | (higher_byte << 8);

        return op;
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            if self.st == 1 {
                // BEEP
            }
            self.st -= 1;
        }
    }

    fn execute(&mut self, op: u16) {
        let digit1 = (0xF000 & op) >> 12;
        let digit2 = (0x0F00 & op) >> 8;
        let digit3 = (0x00F0 & op) >> 4;
        let digit4 = 0x000F & op;

        match (digit1, digit2, digit3, digit4) {
            (0, 0, 0, 0) => return,
            (0, 0, 0xE, 0) => {
                self.screen.fill(false);
            },
            (0, 0, 0xE, 0xE) => {
                self.pc = self.pop();
            },
            (1, _, _, _) => {
                self.pc = 0x0FFF & op; // self.pc = (digit2 << 8) | (digit3 << 4) | digit4;
            }
            (2, _, _, _) => {
                self.push(self.pc);
                self.pc = 0x0FFF & op; // self.pc = (digit2 << 8) | (digit3 << 4) | digit4;
            }
            (3, _, _, _) => {
                if self.v_reg[digit2 as usize] == (0x00FF & op) as u8 {
                    self.pc += 2;
                }
            },
            (4, _, _, _) => {
                if self.v_reg[digit2 as usize] != (0x00FF & op) as u8 {
                    self.pc += 2;
                }
            },
            (5, _, _, 0) => {
                if self.v_reg[digit2 as usize] == self.v_reg[digit3 as usize] {
                    self.pc += 2;
                }
            },
            (6, _, _, _) => {
                self.v_reg[digit2 as usize] = (0x00FF & op) as u8;
            },
            (7, _, _, _) => {
                self.v_reg[digit2 as usize] = self.v_reg[digit2 as usize].wrapping_add((0x00FF & op) as u8);
            },
            (8, _, _, 0) => {
                self.v_reg[digit2 as usize] = self.v_reg[digit3 as usize];
            },
            (8, _, _, 1) => {
                self.v_reg[digit2 as usize] |= self.v_reg[digit3 as usize];
            },
            (8, _, _, 2) => {
                self.v_reg[digit2 as usize] &= self.v_reg[digit3 as usize];
            },
            (8, _, _, 3) => {
                self.v_reg[digit2 as usize] ^= self.v_reg[digit3 as usize];
            },
            (8, _, _, 4) => {
                let (sum, carry) = self.v_reg[digit2 as usize].overflowing_add(self.v_reg[digit3 as usize]);
                self.v_reg[digit2 as usize] = sum;
                self.v_reg[0xF] = if carry {1} else {0};
            },
            (8, _, _, 5) => {
                let (diff, borrow) = self.v_reg[digit2 as usize].overflowing_sub(self.v_reg[digit3 as usize]);
                self.v_reg[digit2 as usize] = diff;
                self.v_reg[0xF] = if borrow {0} else {1};
            },
            (8, _, _, 6) => {
                self.v_reg[0xF] = self.v_reg[digit2 as usize] & 1;
                self.v_reg[digit2 as usize] >>= 1;
            },
            (8, _, _, 7) => {
                let (diff, borrow) = self.v_reg[digit3 as usize].overflowing_sub(self.v_reg[digit2 as usize]);
                self.v_reg[digit2 as usize] = diff;
                self.v_reg[0xF] = if borrow {0} else {1};
            },
            (8, _, _, 0xE) => {
                self.v_reg[0xF] = if self.v_reg[digit2 as usize] & 0x0080 > 0 {1} else {0};
                self.v_reg[digit2 as usize] <<= 1;
            },
            (9, _, _, 0) => {
                if self.v_reg[digit2 as usize] != self.v_reg[digit3 as usize] {
                    self.pc += 2;
                }
            },
            (0xA, _, _, _) => {
                self.i_reg = 0x0FFF & op;
            },
            (0xB, _, _, _) => {
                self.pc = self.v_reg[0] as u16 + (0x0FFF & op);
            },
            (0xC, _, _, _) => {
                self.v_reg[digit2 as usize] = random::<u8>() & (0x00FF & op) as u8;
            },
            (0xD, _, _, _) => {
                let x_coord = self.v_reg[digit2 as usize];
                let y_coord = self.v_reg[digit3 as usize];
                let n_rows = digit4;

                let mut flipped = false;
                for line in 0..n_rows {
                    let addr = self.i_reg + line;
                    let pixels = self.ram[addr as usize];

                    let mask = 0b1000_0000;
                    for i in 0..8 {
                        if pixels & (mask >> i) > 0 {
                            let x = (x_coord + i) as usize % SCREEN_WIDTH;
                            let y = (y_coord + line as u8) as usize % SCREEN_HEIGHT;

                            let idx = y * SCREEN_WIDTH + x;

                            flipped |= self.screen[idx];

                            self.screen[idx] ^= true;
                        }
                    }
                }

                self.v_reg[0xF] = if flipped {1} else {0};
            },
            (0xE, _, 9, 0xE) => {
                if self.keys[self.v_reg[digit2 as usize] as usize] {
                    self.pc += 2;
                }
            },
            (0xE, _, 0xA, 1) => {
                if !self.keys[self.v_reg[digit2 as usize] as usize] {
                    self.pc += 2;
                }
            },
            (0xF, _, 0, 7) => {
                self.v_reg[digit2 as usize] = self.dt;
            },
            (0xF, _, 0, 0xA) => {
                let mut pressed = false;
                for i in 0..NUM_KEYS {
                    if self.keys[i] {
                        self.v_reg[digit2 as usize] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                if !pressed {
                    self.pc -= 2;
                }
            },
            (0xF, _, 1, 5) => {
                self.dt = self.v_reg[digit2 as usize];
            },
            (0xF, _, 1, 8) => {
                self.st = self.v_reg[digit2 as usize];
            },
            (0xF, _, 1, 0xE) => {
                self.i_reg = self.i_reg.wrapping_add(self.v_reg[digit2 as usize] as u16);
            },
            (0xF, _, 2, 9) => {
                self.i_reg = 5 * self.v_reg[digit2 as usize] as u16;
            },
            (0xF, _, 3, 3) => {
                let x = self.v_reg[digit2 as usize];
                let h = x / 100;
                let t = (x / 10) % 10;
                let o = x % 10;

                self.ram[self.i_reg as usize] = h;
                self.ram[(self.i_reg + 1) as usize] = t;
                self.ram[(self.i_reg + 2) as usize] = o;
            },
            (0xF, _, 5, 5) => {
                let x = (digit2 + 1) as usize;
                for i in 0..x {
                    self.ram[self.i_reg as usize + i] = self.v_reg[i];
                }
            },
            (0xF, _, 6, 5) => {
                let x = (digit2 + 1) as usize;
                for i in 0..x {
                    self.v_reg[i] = self.ram[self.i_reg as usize + i];
                }
            },
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op)
        }
    }

    pub fn get_display(&self) -> &[bool] {
        return &self.screen;
    }

    pub fn get_sound(&self) -> bool {
        return self.st > 0;
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = start + data.len();
        self.ram[start..end].copy_from_slice(data);
    }
}
