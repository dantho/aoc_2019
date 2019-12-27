/// tps://adventofcode.com/2019/day/19

use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;
use futures::prelude::*;
use futures::channel::mpsc::{Sender,Receiver};

#[derive(Debug)]
pub enum Error {
    IllegalOpcode {code: isize},
    ComputerComms {msg: String},
}
// Intcode Computer
#[derive(Debug)]
enum OpCode {
    Add = 1,
    Multiply = 2,
    Read = 3,
    Write = 4,
    BranchNE = 5,
    BranchEQ = 6,
    CompareLT = 7,
    CompareEQ = 8,
    AdjustBase = 9,
    Halt = 99,
}
impl TryFrom<isize> for OpCode {
    type Error = Error;
    fn try_from(code: isize) -> Result<Self, Self::Error> {
        use OpCode::*;
        let opcode = match code {
            1 => Add,
            2 => Multiply,
            3 => Read,
            4 => Write,
            5 => BranchNE,
            6 => BranchEQ,
            7 => CompareLT,
            8 => CompareEQ,
            9 => AdjustBase,
            99 => Halt,
            _ => return Err(Error::IllegalOpcode { code }),
        };
        Ok(opcode)
    }
}
pub struct Intcode {
    prog: Vec<isize>,
    input: Receiver<isize>,
    output: Sender<(isize,isize,isize)>,
    pc: usize,
    relative_base: isize,
    write_tuple_buffer: Vec<isize>,
}
impl Intcode {
    pub fn new(prog: Vec<isize>, input: Receiver<isize>, output: Sender<(isize,isize,isize)>) -> Self {
        let pc = 0;
        let relative_base = 0;
        let write_tuple_buffer = Vec::new();
        println!("Intcode Model 2019_23.1 booting...", );
        Intcode { prog, input, output, pc, relative_base, write_tuple_buffer }
    }
    pub async fn run(&mut self) -> Result<(), Error> {
        loop {
            use OpCode::*;
            let mode = self.prog[self.pc] / 100;
            let op = (self.prog[self.pc] - mode * 100).try_into()?;
            let m1 = mode - mode / 10 * 10;  let mode = mode / 10;
            let m2 = mode - mode / 10 * 10;  let mode = mode / 10;
            let m3 = mode - mode / 10 * 10;  let mode = mode / 10;
            assert_eq!(mode, 0);
            match op {
                Add => {
                    let p1 = self.prog[self.pc + 1];
                    let p2 = self.prog[self.pc + 2];
                    let p3 = self.prog[self.pc + 3];
                    let v1 = match m1 { 0=>self.prog[p1 as usize], 1=>p1, 2=>self.prog[(p1+self.relative_base) as usize], _ => panic!("Bad Mode"),};
                    let v2 = match m2 { 0=>self.prog[p2 as usize], 1=>p2, 2=>self.prog[(p2+self.relative_base) as usize], _ => panic!("Bad Mode"),};
                    assert_ne!(m3, 1);
                    self.prog[ if m3 ==2 {p3+self.relative_base} else {p3} as usize] = v1 + v2;
                    self.pc += 4;
                }
                Multiply => {
                    let p1 = self.prog[self.pc + 1];
                    let p2 = self.prog[self.pc + 2];
                    let p3 = self.prog[self.pc + 3];
                    let v1 = match m1 { 0=>self.prog[p1 as usize], 1=>p1, 2=>self.prog[(p1+self.relative_base) as usize], _ => panic!("Bad Mode"),};
                    let v2 = match m2 { 0=>self.prog[p2 as usize], 1=>p2, 2=>self.prog[(p2+self.relative_base) as usize], _ => panic!("Bad Mode"),};
                    assert_ne!(m3, 1);
                    self.prog[ if m3 ==2 {p3+self.relative_base} else {p3} as usize] = v1 * v2;
                    self.pc += 4;
                }
                Read => {
                    let p1 = self.prog[self.pc + 1];
                    assert_ne!(m1, 1);
                    self.prog[ if m1 ==2 {p1+self.relative_base} else {p1} as usize] = match self.input.next().await {
                        Some(v) => {
                            // println!("Intcode READing i32: {}", v);
                            v
                        },
                        None => return Err(Error::ComputerComms{msg:"Expecting input, but stream has terminated.".to_string()}),
                    };
                    self.pc += 2;
                }
                Write => {
                    let p1 = self.prog[self.pc + 1];
                    let v1 = match m1 { 0=>self.prog[p1 as usize], 1=>p1, 2=>self.prog[(p1+self.relative_base) as usize], _ => panic!("Bad Mode"),};
                    // println!("Intcode WRITEing i32: {}", v1);
                    self.write_tuple_buffer.push(v1);
                    if self.write_tuple_buffer.len() == 3 {
                        let tuple = (self.write_tuple_buffer[0], self.write_tuple_buffer[1], self.write_tuple_buffer[2]);
                        if let Err(_) = self.output.send(tuple).await {
                            // println!("Intcode Reporting WRITE error");
                            return Err(Error::ComputerComms{msg:"Problem sending output data. Has receiver been dropped?".to_string()});
                        };
                        self.write_tuple_buffer = Vec::new();
                    }
                    self.pc += 2;
                }
                BranchNE => {
                    let p1 = self.prog[self.pc + 1];
                    let p2 = self.prog[self.pc + 2];
                    let v1 = match m1 { 0=>self.prog[p1 as usize], 1=>p1, 2=>self.prog[(p1+self.relative_base) as usize], _ => panic!("Bad Mode"),};
                    let v2 = match m2 { 0=>self.prog[p2 as usize], 1=>p2, 2=>self.prog[(p2+self.relative_base) as usize], _ => panic!("Bad Mode"),};
                    if v1 != 0 {
                        self.pc = v2 as usize
                    } else {
                        self.pc += 3
                    };
                }
                BranchEQ => {
                    let p1 = self.prog[self.pc + 1];
                    let p2 = self.prog[self.pc + 2];
                    let v1 = match m1 { 0=>self.prog[p1 as usize], 1=>p1, 2=>self.prog[(p1+self.relative_base) as usize], _ => panic!("Bad Mode"),};
                    let v2 = match m2 { 0=>self.prog[p2 as usize], 1=>p2, 2=>self.prog[(p2+self.relative_base) as usize], _ => panic!("Bad Mode"),};
                    if v1 == 0 {
                        self.pc = v2 as usize
                    } else {
                        self.pc += 3
                    };
                }
                CompareLT => {
                    let p1 = self.prog[self.pc + 1];
                    let p2 = self.prog[self.pc + 2];
                    let p3 = self.prog[self.pc + 3];
                    let v1 = match m1 { 0=>self.prog[p1 as usize], 1=>p1, 2=>self.prog[(p1+self.relative_base) as usize], _ => panic!("Bad Mode"),};
                    let v2 = match m2 { 0=>self.prog[p2 as usize], 1=>p2, 2=>self.prog[(p2+self.relative_base) as usize], _ => panic!("Bad Mode"),};
                    assert_ne!(m3, 1);
                    self.prog[ if m3 ==2 {p3+self.relative_base} else {p3} as usize] = if v1 < v2 {1} else {0};
                    self.pc += 4;
                }
                CompareEQ => {
                    let p1 = self.prog[self.pc + 1];
                    let p2 = self.prog[self.pc + 2];
                    let p3 = self.prog[self.pc + 3];
                    let v1 = match m1 { 0=>self.prog[p1 as usize], 1=>p1, 2=>self.prog[(p1+self.relative_base) as usize], _ => panic!("Bad Mode"),};
                    let v2 = match m2 { 0=>self.prog[p2 as usize], 1=>p2, 2=>self.prog[(p2+self.relative_base) as usize], _ => panic!("Bad Mode"),};
                    assert_ne!(m3, 1);
                    self.prog[ if m3 ==2 {p3+self.relative_base} else {p3} as usize] = if v1 == v2 {1} else {0};
                    self.pc += 4;
                }
                AdjustBase => {
                    let p1 = self.prog[self.pc + 1];
                    let v1 = match m1 { 0=>self.prog[p1 as usize], 1=>p1, 2=>self.prog[(p1+self.relative_base) as usize], _ => panic!("Bad Mode"),};
                    self.relative_base += v1;
                    self.pc += 2;
                }
                Halt => {
                    // println!("Intcode: Received 'Halt' command");
                    break;
                },
            }
        }
        Ok(()) // Drop the input Receiver, this allow downstream fetching of values we are not going to process ('cause we're done)
    }
}
