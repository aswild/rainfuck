use std::collections::HashMap;
use std::io::{stdout, Write};

#[derive(Copy, Clone, Eq, PartialEq)]
enum Cmd {
    Right,
    Left,
    Inc,
    Dec,
    Out,
    In,
    Start,
    End,
}

impl Cmd {
    fn from_byte(b: u8) -> Option<Self> {
        match b {
            b'>' => Some(Self::Right),
            b'<' => Some(Self::Left),
            b'+' => Some(Self::Inc),
            b'-' => Some(Self::Dec),
            b'.' => Some(Self::Out),
            b',' => Some(Self::In),
            b'[' => Some(Self::Start),
            b']' => Some(Self::End),
            _ => None,
        }
    }
}

struct Program {
    cmds: Vec<Cmd>,
    data: Vec<u8>,
    pc: usize,
    ptr: usize,
    loops: HashMap<usize, usize>,
}

impl Program {
    pub fn parse(code: &[u8]) -> Result<Self, &'static str> {
        let mut prog = Program {
            cmds: Vec::new(),
            data: vec![0; 1024],
            pc: 0,
            ptr: 0,
            loops: HashMap::new(),
        };

        let mut startstack = Vec::new();
        for (i, cmd) in code.iter().filter_map(|b| Cmd::from_byte(*b)).enumerate() {
            prog.cmds.push(cmd);
            if cmd == Cmd::Start {
                startstack.push(i);
            } else if cmd == Cmd::End {
                match startstack.pop() {
                    Some(start) => {
                        prog.loops.insert(start, i);
                        prog.loops.insert(i, start);
                    }
                    None => return Err("unmatched ]"),
                };
            }
        }
        if !startstack.is_empty() {
            return Err("unmatched [");
        }
        Ok(prog)
    }

    #[inline(always)]
    fn cell(&self) -> u8 {
        self.data[self.ptr]
    }

    #[inline(always)]
    fn set_cell(&mut self, val: u8) {
        self.data[self.ptr] = val;
    }

    pub fn step(&mut self) -> bool {
        if self.pc >= self.cmds.len() {
            return false;
        }
        match self.cmds[self.pc] {
            Cmd::Right => {
                if self.ptr == self.data.len() {
                    self.data.resize(self.data.len() + 1024, 0);
                }
                self.ptr += 1;
            }
            Cmd::Left => {
                if self.ptr == 0 {
                    return false;
                }
                self.ptr -= 1;
            }
            Cmd::Inc => {
                self.set_cell(self.cell().wrapping_add(1));
            }
            Cmd::Dec => {
                self.set_cell(self.cell().wrapping_sub(1));
            }
            Cmd::Out => {
                stdout().write(&[self.cell()]).unwrap();
            }
            Cmd::In => {
                todo!();
            }
            Cmd::Start => {
                if self.data[self.ptr] == 0 {
                    self.pc = self.loops[&self.pc];
                }
            }
            Cmd::End => {
                if self.data[self.ptr] != 0 {
                    self.pc = self.loops[&self.pc];
                }
            }
        }
        self.pc += 1;
        true
    }
}

fn main() {
    let text = std::env::args().nth(1).unwrap();
    let mut prog = Program::parse(text.as_bytes()).unwrap();

    while prog.step() {}
}
