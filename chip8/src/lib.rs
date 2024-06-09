use rand::random;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200;
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
0xF0, 0x80, 0xF0, 0x80, 0x80 // F
];

pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    // general program registers
    v_reg: [u8; NUM_REGS],
    // index register - indexes into RAM for r/w
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    // delay timer
    dt: u8,
    // sound timer
    st: u8,
}

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };
        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        new_emu
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

    pub fn tick(&mut self) {
        // Fetch
        let opcode = self.fetch();
        // Decode
        self.execute(opcode);
        // Execute
    }

    fn fetch(&mut self) -> u16 {
        let high = self.ram[self.pc as usize] as u16;
        let low = self.ram[(self.pc + 1) as usize] as u16;
        let opcode = (high << 8) | low;
        self.pc += 2;
        opcode
    }

    fn execute (&mut self, opcode: u16) {
        let digit1 = (opcode & 0xF000) >> 12;
        let digit2 = (opcode & 0x0F00) >> 8;
        let digit3 = (opcode & 0x00F0) >> 4;
        let digit4 = (opcode & 0x000F);

        match (digit1, digit2, digit3, digit4) {
            // NOP
            (0, 0, 0, 0) => return,
            // CLS - Clear Screen
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            },
            // RET
            (0, 0, 0xE, 0xE) => {
                let ret_addr = self.pop();
                self.pc = ret_addr;
            },
            // JMP
            (1, _, _, _) => {
                let addr = opcode & 0xFFF;
                self.pc = addr;
            },
            // Call
            (2, _, _, _) => {
                let addr = opcode & 0xFFF;
                self.push(self.pc);
                self.pc = addr;
            },
            // SKIP VX == NN
            (3, _, _, _) => {
                let x = digit2 as usize;
                let nn = (opcode & 0xFF) as u8;
                if self.v_reg[x] == nn {
                    self.pc +=2;
                }
            },
            // SKIP VX != NN
            (4, _, _, _) => {
                let x = digit2 as usize;
                let nn = (opcode & 0xFF) as u8;
                if self.v_reg[x] != nn {
                    self.pc +=2;
                }
            },
            // SKIP VX = VY
            (5, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc +=2;
                }
            },
            // VX = NN
            (6, _, _, _) => {
                let x = digit2 as usize;
                let nn = (opcode & 0xFF) as u8;
                self.v_reg[x] = nn;
            },
            // VX += NN
            (7, _, _, _) => {
                let x = digit2 as usize;
                let nn = (opcode & 0xFF) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            },
            // VY = NN
            (8, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] = self.v_reg[y];
            },
            // VX |= VY
            (8, _, _, 1) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] |= self.v_reg[y];
            },
            // VX &= VY
            (8, _, _, 2) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] &= self.v_reg[y];
            },
            // VX ^= VY
            (8, _, _, 3) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] ^= self.v_reg[y];
            },
            // VX += VY
            (8, _, _, 4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                let new_vf = if carry { 1 } else { 0 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            // VX -= VY
            (8, _, _, 5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            // VX >> 1
            (8, _, _, 6) => {
                let x = digit2 as usize;
                let lsb = self.v_reg[x] & 1;
                self.v_reg[x] >>= 1;
                self.v_reg[0xF] = lsb;
            },
            // VY - VX
            (8, _, _, 7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            // VX <<= 1
            (8, _, _, 0xE) => {
                let x = digit2 as usize;
                let msb = (self.v_reg[x] >> 7) & 1;
                self.v_reg[x] <<= 1;
                self.v_reg[0xF] = msb;
            },
            // SKIP if VX != VY
            (9, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            },
            // I = NNN
            (0xA, _, _, _) => {
                let nnn = opcode & 0xFFF;
                self.i_reg = nnn;
            },
            // JMP to v0 + NNN
            (0xB, _, _, _) => {
                let nnn = opcode & 0xFFF;
                self.pc = (self.v_reg[0] as u16) + nnn;
            },
            // VX = rand & NN
            (0xC, _, _, _) => {
                let x = digit2 as usize;
                let nn = (opcode & 0xFF) as u8;
                let rng: u8 = random();
                self.v_reg[x] = rng & nn;
            },
            // DRAW
            (0xD, _, _, _) => {
                // get the (x, y) for our sprite
                let x_coord = self.v_reg[digit2 as usize] as u16;
                let y_coord = self.v_reg[digit3 as usize] as u16;
                // the last digit determines how many rows high our sprite is
                let num_rows = digit4;
                let mut flipped = false;
                // iterate over each row of our sprite
                for y_line in 0..num_rows {
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];
                    // iterate over each column in our row
                    for x_line in 0..8 {
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            // sprites should wrap around screen
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                            // get pixel's index for our 1D screen array
                            let idx = x + SCREEN_WIDTH * y;
                            // check if we're about to flip the pixel and set
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                    if flipped {
                        self.v_reg[0xF] = 1;
                    } else {
                        self.v_reg[0xF] = 0;
                    }
                }
            },
            // Skip if key pressed
            (0xE, _, 9, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if key {
                    self.pc += 2;
                }
            },
            // SKIP if key released
            (0xE, _, 0xA, 1) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if !key {
                    self.pc += 2;
                }
            },
            // VX = DT
            (0xF, _, 0, 7) => {
                let x = digit2 as usize;
                self.v_reg[x] = self.dt;
            },
            // Wait for key press
            (0xF, _, 0, 0xA) => {
                let x = digit2 as usize;
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_reg[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }
                if !pressed {
                    self.pc -= 2;
                }
            },
            // DT = VX
            (0xF, _, 1, 5)  => {
                let x = digit2 as usize;
                self.dt = self.v_reg[x];
            },
            // ST = VX
            (0xF, _, 1, 8) => {
                let x = digit2 as usize;
                self.st = self.v_reg[x];
            },
            // I += VX
            (0xF, _, 1, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x] as u16;
                self.i_reg = self.i_reg.wrapping_add(vx);
            }
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", opcode),
        }
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

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -=1;
        self.stack[self.sp as usize]
    }

}