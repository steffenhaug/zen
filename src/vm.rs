use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Instant, Duration};

use display::{self, BufferSend, ControllerHandle};

#[derive(Debug)]
pub enum Instruction {
    Halt,
    Illegal,
    LoadI,
    LoadR,
    Add,
    Print,
    Nop,
    Jump,
    Jeq,
    Jneq,
    Comp,
    Color,
    Inc,
    Dec,
    Call,
    Ret,
    Input,
    And,
    Xor,
    Or,
    Sub,
    Div,
    Mul,
    Jz,
    Jnz,
    PrintB,
    AndI,
    ColorI,
    Draw,
    Push,
    Pop,
    JumpDt,
}

impl From<u8> for Instruction {
    fn from(x: u8) -> Self {
        use self::Instruction::*;

        match x {
            0 => Halt,
            1 => LoadI,
            2 => Add,
            3 => Print,
            4 => Nop,
            5 => Jump,
            6 => LoadR,
            7 => Jeq,
            8 => Comp,
            9 => Jneq,
            10 => Color,
            11 => Inc,
            12 => Dec,
            13 => Call,
            14 => Ret,
            15 => Input,
            16 => And,
            17 => Xor,
            18 => Or,
            19 => Sub,
            20 => Div,
            21 => Mul,
            22 => Jz,
            23 => Jnz,
            24 => PrintB,
            25 => AndI,
            26 => ColorI,
            27 => Draw,
            28 => Push,
            29 => Pop,
            30 => JumpDt,
            _ => Illegal,
        }
    }
}

pub struct VM {
    program: Vec<u8>,
    screen: BufferSend,
    // frequency [1]
    cpu_freq: u8,
    // Pixel buffer to render graphics
    buf: [u8; ::N_BYTES],
    controller: ControllerHandle,
    stack:      Vec<u8>,
    call_stack: Vec<Option<usize>>,
    // general purpose registers
    reg: [u8; 64],
    // special purpose registers and flags
    ip: usize,
    ret: Option<usize>,  // retptr
    e: bool,    // equal
    z: bool,    // zero
}

impl VM {
    pub fn new(scr: BufferSend, con: ControllerHandle) -> VM {
        VM {
            program: vec![],
            screen: scr,
            cpu_freq: 10,
            buf: [0; ::N_BYTES],
            controller: con,
            stack:      Vec::with_capacity(32),
            call_stack: Vec::with_capacity(32),
            reg: [0; 64],
            ip:  0,
            ret: None,
            e:   false,
            z:   false
        }
    }

    pub fn interpret(&mut self, program: Vec<u8>) {
        self.program = program;
        self.run();
    }

    fn fetch(&mut self) -> u8 {
        let val = self.program[self.ip];
        self.ip += 1;
        val
    }

    pub fn run(&mut self) {
        use self::Instruction::*;

        let t = 1000 as f64 * (1 as f64  / self.cpu_freq as f64);
        let lambda = Duration::from_millis(t as u64);

        let mut last_tick = Instant::now();

        'fetch_execute: loop {
            match Instruction::from(self.fetch()) {
                Halt    => break 'fetch_execute,
                Illegal => break, // handle error
                Nop     => continue,
                Draw => {
                    self.screen.send(self.buf.to_vec());
                },
                LoadI => {
                    let r = self.fetch();
                    let x = self.fetch();
                    self.reg[usize::from(r)] = x;
                },
                LoadR => {
                    let r1 = self.fetch();
                    let r2 = self.fetch();
                    let x = self.reg[usize::from(r2)];
                    self.reg[usize::from(r1)] = x;
                },
                Add => {
                    let r1 = self.fetch();
                    let r2 = self.fetch();
                    let r3 = self.fetch();
                    let x = self.reg[usize::from(r2)];
                    let y = self.reg[usize::from(r3)];
                    self.reg[usize::from(r1)] = x + y;
                },
                Jump => {
                    let ip = self.fetch();
                    self.ip = ip as usize;
                },
                Jeq => {
                    let ip = self.fetch();
                    if self.e {
                        self.ip = ip as usize;
                    }
                },
                Jneq => {
                    let ip = self.fetch();
                    if !self.e {
                        self.ip = ip as usize;
                    }
                },
                Comp => {
                    let r1 = self.fetch();
                    let r2 = self.fetch();
                    let x = self.reg[usize::from(r1)];
                    let y = self.reg[usize::from(r2)];
                    self.e = x == y;
                },
                Color => {
                    let r1 = self.fetch(); // x coord
                    let r2 = self.fetch(); // y coord
                    let r3 = self.fetch(); // 8-bit color
                    let x = self.reg[usize::from(r1)] as usize;
                    let y = self.reg[usize::from(r2)] as usize;
                    let rgb = self.reg[usize::from(r3)];
                    display::set(&mut self.buf, x, y, rgb);
                },
                ColorI => {
                    let r1 = self.fetch(); // x coord
                    let r2 = self.fetch(); // y coord
                    let rgb = self.fetch(); // 8-bit color
                    let x = self.reg[usize::from(r1)] as usize;
                    let y = self.reg[usize::from(r2)] as usize;
                    display::set(&mut self.buf, x, y, rgb);
                },
                Inc => {
                    let r = self.fetch(); // x coord
                    self.reg[usize::from(r)] += 1;
                },
                Dec => {
                    let r = self.fetch(); // x coord
                    self.reg[usize::from(r)] -= 1;
                },
                Print => {
                    let r = self.fetch();
                    println!("PRINT     {}",
                             self.reg[usize::from(r)]);
                },
                PrintB => {
                    let r = self.fetch();
                    println!("PRINT     {:b}",
                             self.reg[usize::from(r)]);
                },
                Call => {
                    let ip = self.fetch();
                    self.call_stack.push(self.ret);
                    self.ret = Some(self.ip);
                    self.ip = ip as usize;
                },
                Ret => {
                    if let Some(caller) = self.ret {
                        self.ip = caller;
                        self.ret = self.call_stack.pop().unwrap();
                    } else {
                        break // error
                    }
                },
                Input => {
                    let r1 = self.fetch(); // x coord
                    let g = self.controller.lock().unwrap();
                    self.reg[usize::from(r1)] = *g;
                }
                And => {
                    let r1 = self.fetch();
                    let r2 = self.fetch();
                    let r3 = self.fetch();
                    let x = self.reg[usize::from(r2)];
                    let y = self.reg[usize::from(r3)];
                    self.reg[usize::from(r1)] = x & y;
                    self.z = 0 == x & y;
                },
                AndI => {
                    let r1 = self.fetch();
                    let r2 = self.fetch();
                    let x = self.reg[usize::from(r2)];
                    let y = self.fetch();
                    self.reg[usize::from(r1)] = x & y;
                    self.z = 0 == x & y;
                },
                Xor => {
                    let r1 = self.fetch();
                    let r2 = self.fetch();
                    let r3 = self.fetch();
                    let x = self.reg[usize::from(r2)];
                    let y = self.reg[usize::from(r3)];
                    self.reg[usize::from(r1)] = x ^ y;
                    self.z = 0 == x ^ y;
                },
                Or => {
                    let r1 = self.fetch();
                    let r2 = self.fetch();
                    let r3 = self.fetch();
                    let x = self.reg[usize::from(r2)];
                    let y = self.reg[usize::from(r3)];
                    self.reg[usize::from(r1)] = x | y;
                    self.z = 0 == x | y;
                },
                Sub => {
                    let r1 = self.fetch();
                    let r2 = self.fetch();
                    let r3 = self.fetch();
                    let x = self.reg[usize::from(r2)];
                    let y = self.reg[usize::from(r3)];
                    self.reg[usize::from(r1)] = x - y;
                },
                Div => {
                    let r1 = self.fetch();
                    let r2 = self.fetch();
                    let r3 = self.fetch();
                    let x = self.reg[usize::from(r2)];
                    let y = self.reg[usize::from(r3)];
                    self.reg[usize::from(r1)] = x / y;
                },
                Mul => {
                    let r1 = self.fetch();
                    let r2 = self.fetch();
                    let r3 = self.fetch();
                    let x = self.reg[usize::from(r2)];
                    let y = self.reg[usize::from(r3)];
                    self.reg[usize::from(r1)] = x * y;
                },
                Jz => {
                    let ip = self.fetch();
                    if self.z {
                        self.ip = ip as usize;
                    }
                },
                Jnz => {
                    let ip = self.fetch();
                    if !self.z {
                        self.ip = ip as usize;
                    }
                },
                Push => {
                    let r1 = self.fetch();
                    let x = self.reg[usize::from(r1)];
                    self.stack.push(x);
                },
                Pop => {
                    let r1 = self.fetch();
                    if let Some(x) =  self.stack.pop() {
                        self.reg[r1 as usize] = x;
                    } else {
                        self.reg[r1 as usize] = 0;
                    }
                },
                JumpDt => {
                    let ip = self.fetch();
                    if last_tick.elapsed() < lambda {
                        self.ip = ip as usize;
                    } else {
                        last_tick = Instant::now();
                    }
                }
            };
        }
    }
}

/* Footnotes
 * ~~~~~~~~~
 * [1] The fectch-execute cycle runs as fast as possible still,
 * to render at high framerates. You can skip things when needed,
 * i.e. the game logic, with JMPDT (jump delta time), and there
 * instructions will be executed at the cpu freq.
 * */
