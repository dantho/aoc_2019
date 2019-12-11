/// Day03 code stolen from https://github.com/kodsnack/advent_of_code_2019/blob/master/tomasskare-rust/day2/src/main.rs
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use futures::prelude::*;
use futures::channel::mpsc::{channel,Sender,Receiver};
use futures::executor::block_on;
use futures::join;
use std::convert::{TryFrom, TryInto};

#[derive(Debug)]
enum Error {
    IllegalOpcode { code: isize },
}
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
impl OpCode {
    fn pcount(&self) -> usize {
        use OpCode::*;
        match self {
            Add => 3,
            Multiply => 3,
            Read => 1,
            Write => 1,
            BranchNE => 2,
            BranchEQ => 2,
            CompareLT => 3,
            CompareEQ => 3,
            AdjustBase => 1,
            Halt => 0,
        }
    }
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
async fn intcode_run(mut v: Vec<isize>, mut input: Receiver<isize>, mut output: Sender<isize>) -> Result<Receiver<isize>, Error> {
    let mut pc: usize = 0;
    let mut relative_base: isize = 0;
    loop {
        use OpCode::*;
        let mode = v[pc] / 100;
        let op = (v[pc] - mode * 100).try_into()?;
        let m1 = mode - mode / 10 * 10;  let mode = mode / 10;
        let m2 = mode - mode / 10 * 10;  let mode = mode / 10;
        let m3 = mode - mode / 10 * 10;  let mode = mode / 10;
        assert_eq!(mode, 0);
        // if !op.execute(&mut v, mode) { break; }
        match op {
            Add => {
                let p1 = v[pc + 1];
                let p2 = v[pc + 2];
                let p3 = v[pc + 3];
                let v1 = match m1 { 0=>v[p1 as usize], 1=>p1, 2=>v[(p1+relative_base) as usize], _ => panic!("Bad Mode"),};
                let v2 = match m2 { 0=>v[p2 as usize], 1=>p2, 2=>v[(p2+relative_base) as usize], _ => panic!("Bad Mode"),};
                assert_ne!(m3, 1);
                v[ if m3 ==2 {p3+relative_base} else {p3} as usize] = v1 + v2;
                pc += 4;
            }
            Multiply => {
                let p1 = v[pc + 1];
                let p2 = v[pc + 2];
                let p3 = v[pc + 3];
                let v1 = match m1 { 0=>v[p1 as usize], 1=>p1, 2=>v[(p1+relative_base) as usize], _ => panic!("Bad Mode"),};
                let v2 = match m2 { 0=>v[p2 as usize], 1=>p2, 2=>v[(p2+relative_base) as usize], _ => panic!("Bad Mode"),};
                assert_ne!(m3, 1);
                v[ if m3 ==2 {p3+relative_base} else {p3} as usize] = v1 * v2;
                pc += 4;
            }
            Read => {
                let p1 = v[pc + 1];
                assert_ne!(m1, 1);
                v[ if m1 ==2 {p1+relative_base} else {p1} as usize] = match input.next().await {
                    Some(v) => v,
                    None => panic!("Expecting input, but stream has terminated."),
                };
                pc += 2;
            }
            Write => {
                let p1 = v[pc + 1];
                let v1 = match m1 { 0=>v[p1 as usize], 1=>p1, 2=>v[(p1+relative_base) as usize], _ => panic!("Bad Mode"),};
                if let Err(_) = output.send(v1).await {
                    panic!("Problem sending output data, has receiver been dropped?")
                };
                pc += 2;
            }
            BranchNE => {
                let p1 = v[pc + 1];
                let p2 = v[pc + 2];
                let v1 = match m1 { 0=>v[p1 as usize], 1=>p1, 2=>v[(p1+relative_base) as usize], _ => panic!("Bad Mode"),};
                let v2 = match m2 { 0=>v[p2 as usize], 1=>p2, 2=>v[(p2+relative_base) as usize], _ => panic!("Bad Mode"),};
                if v1 != 0 {
                    pc = v2 as usize
                } else {
                    pc += 3
                };
            }
            BranchEQ => {
                let p1 = v[pc + 1];
                let p2 = v[pc + 2];
                let v1 = match m1 { 0=>v[p1 as usize], 1=>p1, 2=>v[(p1+relative_base) as usize], _ => panic!("Bad Mode"),};
                let v2 = match m2 { 0=>v[p2 as usize], 1=>p2, 2=>v[(p2+relative_base) as usize], _ => panic!("Bad Mode"),};
                if v1 == 0 {
                    pc = v2 as usize
                } else {
                    pc += 3
                };
            }
            CompareLT => {
                let p1 = v[pc + 1];
                let p2 = v[pc + 2];
                let p3 = v[pc + 3];
                let v1 = match m1 { 0=>v[p1 as usize], 1=>p1, 2=>v[(p1+relative_base) as usize], _ => panic!("Bad Mode"),};
                let v2 = match m2 { 0=>v[p2 as usize], 1=>p2, 2=>v[(p2+relative_base) as usize], _ => panic!("Bad Mode"),};
                assert_ne!(m3, 1);
                v[ if m3 ==2 {p3+relative_base} else {p3} as usize] = if v1 < v2 {1} else {0};
                pc += 4;
            }
            CompareEQ => {
                let p1 = v[pc + 1];
                let p2 = v[pc + 2];
                let p3 = v[pc + 3];
                let v1 = match m1 { 0=>v[p1 as usize], 1=>p1, 2=>v[(p1+relative_base) as usize], _ => panic!("Bad Mode"),};
                let v2 = match m2 { 0=>v[p2 as usize], 1=>p2, 2=>v[(p2+relative_base) as usize], _ => panic!("Bad Mode"),};
                assert_ne!(m3, 1);
                v[ if m3 ==2 {p3+relative_base} else {p3} as usize] = if v1 == v2 {1} else {0};
                pc += 4;
            }
            AdjustBase => {
                let p1 = v[pc + 1];
                let v1 = match m1 { 0=>v[p1 as usize], 1=>p1, 2=>v[(p1+relative_base) as usize], _ => panic!("Bad Mode"),};
                relative_base += v1;
                pc += 2;
            }
            Halt => break,
        }
    }
    Ok(input) // Drop the input Receiver, this allow downstream fetching of values we are not going to process ('cause we're done)
}
async fn dump_output(mut rx: Receiver<isize>) {
    // print all outputs
    loop {
        if let Some(v) = rx.next().await {
            println!("Output: {}", v);
        } else {
            break;
        }
    }
}
async fn boot_intcode(prog: Vec<isize>, initial_input: isize) -> Result<(),Error> {
    const BUFFER_SIZE: usize = 100;
    let (mut input_tx, input_rx) = channel::<isize>(BUFFER_SIZE);
    let (output_tx, output_rx) = channel::<isize>(BUFFER_SIZE);
    // send initial input
    input_tx.send(initial_input).await.unwrap();
    // run computer
    let computer = intcode_run(prog.clone(), input_rx, output_tx);
    let outputter = dump_output(output_rx);
    let (r,_) = join!(computer,outputter);
    match r {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
fn main() {
    const PROG_MEM_SIZE: usize = 2000;
    let initial_input: isize = 2;
    let filename = "input.txt";
    // let filename = "day09_example1.txt";
    // let filename = "day09_example2.txt";
    // let filename = "day09_example3.txt";
    let fd = File::open(filename).expect(&format!("Failure opening {}", filename));
    let buf = BufReader::new(fd);
    let mut prog_orig = Vec::new();
    buf.lines().for_each(|line| {
        line.unwrap().split(',').for_each(|numstr| {
            let num = numstr.parse::<isize>().unwrap();
            prog_orig.push(num);
        });
    });
    // Add some empty space for code growth
    if prog_orig.len() < PROG_MEM_SIZE {
        let mut extra_space = vec![0; PROG_MEM_SIZE - prog_orig.len()];
        prog_orig.append(&mut extra_space);
    }

    let _out = block_on(boot_intcode(prog_orig.clone(), initial_input));
}
