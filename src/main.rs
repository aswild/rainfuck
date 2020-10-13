//! rainfuck: A simple Rust brainfuck interpreter.

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, Read, Write};

use anyhow::{anyhow, Context, Result};
use clap::{App, Arg, ArgGroup};

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
    pub fn parse(code: &[u8]) -> Result<Self> {
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
                    None => return Err(anyhow!("unmatched ]")),
                };
            }
        }
        if !startstack.is_empty() {
            return Err(anyhow!("unmatched ["));
        }
        Ok(prog)
    }

    pub fn load<R: Read>(mut file: R) -> Result<Self> {
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        Self::parse(&data)
    }

    #[inline(always)]
    fn cell(&self) -> u8 {
        self.data[self.ptr]
    }

    #[inline(always)]
    fn set_cell(&mut self, val: u8) {
        self.data[self.ptr] = val;
    }

    /// Execute the instruction at self.pc.
    /// Returns Ok(true) if the program continues, or Ok(false) if the program has halted,
    /// Returns Err if there's an IO error, only for '.' or ',' commands.
    /// cin and cout are read/write streams for IO.
    pub fn step<R: Read, W: Write>(
        &mut self,
        cin: &mut R,
        cout: &mut W,
    ) -> Result<bool, io::Error> {
        if self.pc >= self.cmds.len() {
            return Ok(false);
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
                    return Ok(false);
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
                cout.write_all(&[self.cell()])?;
            }
            Cmd::In => {
                let mut b = [0u8];
                match cin.read_exact(&mut b) {
                    Ok(_) => self.set_cell(b[0]),
                    Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => (),
                    Err(err) => return Err(err),
                }
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
        Ok(true)
    }

    /// Run the program to completion, exiting early with Err if an IO error is encountered
    pub fn run_stdio(&mut self) -> Result<(), io::Error> {
        let mut cin = io::stdin();
        let mut cout = io::stdout();
        loop {
            match self.step(&mut cin, &mut cout) {
                Ok(true) => (),
                Ok(false) => break Ok(()),
                Err(e) => break Err(e),
            }
        }
    }
}

fn run() -> Result<()> {
    let args = App::new("rainfuck")
        .about("A brainfuck interpreter. Input and output is connected to stdin/stdout")
        .usage("rainfuck [OPTIONS] {-e PROGRAM | FILE}")
        .arg(
            Arg::with_name("code")
                .short("e")
                .takes_value(true)
                .value_name("PROGRAM")
                .help("Provide the program on the command line rather than as an input file"),
        )
        .arg(
            Arg::with_name("file")
                .required(false)
                .value_name("FILE")
                .help("Source file to run. Required unless -e is used."),
        )
        .group(
            ArgGroup::with_name("input")
                .arg("code")
                .arg("file")
                .required(true),
        )
        .get_matches();

    let mut prog = if args.is_present("code") {
        Program::parse(args.value_of("code").unwrap().as_bytes())
    } else {
        Program::load(BufReader::new(
            File::open(args.value_of("file").unwrap()).context("error opening input file")?,
        ))
    }
    .context("failed to parse program")?;

    prog.run_stdio().context("IO Error")?;
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{:#}", e);
        std::process::exit(1);
    }
}
