/// Day03 code stolen from https://github.com/kodsnack/advent_of_code_2019/blob/master/tomasskare-rust/day2/src/main.rs
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;
use std::collections::HashMap;
use futures::prelude::*;
use futures::channel::mpsc::{channel,Sender,Receiver};
use futures::executor::block_on;
use futures::join;
use PaintColor::*;
use TurnDirection::*;
use Orientation::*;

#[derive(Debug)]
enum Error {
    IllegalOpcode { code: isize },
    IllegalColor { val: isize },
    IllegalTurnDirection { val: isize },
    RobotComms { msg: String },
    ComputerComms { msg: String },
}
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
                    None => return Err(Error::ComputerComms{msg:"Expecting input, but stream has terminated.".to_string()}),
                };
                pc += 2;
            }
            Write => {
                let p1 = v[pc + 1];
                let v1 = match m1 { 0=>v[p1 as usize], 1=>p1, 2=>v[(p1+relative_base) as usize], _ => panic!("Bad Mode"),};
                if let Err(_) = output.send(v1).await {
                    return Err(Error::ComputerComms{msg:"Problem sending output data. Has receiver been dropped?".to_string()});
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
#[derive(Debug,Copy,Clone)]
enum PaintColor {
    Black = 0,
    White = 1,
}
impl TryFrom<isize> for PaintColor {
    type Error = Error;
    fn try_from(val: isize) -> Result<Self, Self::Error> {
        use PaintColor::*;
        let color = match val {
            n if n == Black as isize => Black,
            n if n == White as isize => White,
            _ => return Err(Error::IllegalColor { val }),
        };
        Ok(color)
    }
}
#[derive(Debug,Copy,Clone)]
enum TurnDirection {
    Left = 0,
    Right = 1,
}
impl TryFrom<isize> for TurnDirection {
    type Error = Error;
    fn try_from(val: isize) -> Result<Self, Self::Error> {
        use TurnDirection::*;
        let turn = match val {
            n if n == Left as isize => Left,
            n if n == Right as isize => Right,
            _ => return Err(Error::IllegalTurnDirection { val }),
        };
        Ok(turn)
    }
}
#[derive(Debug,Copy,Clone)]
enum Orientation {
    North,
    South,
    East,
    West,
}
impl Orientation {
    fn turn(&self, dir: TurnDirection) -> Orientation {
        match dir {
            Left => 
            match *self {
                North => West,
                East => North,
                South => East,
                West => South,
            },
        Right => 
            match *self {
                North => East,
                East => South,
                South => West,
                West => North,
            },
        }
    }
    fn step(&self, coord: (isize,isize)) -> (isize,isize) {
        let x = match self {
            North => coord.0,
            South => coord.0,
            East => coord.0+1,
            West => coord.0-1,
        };
        let y = match self {
            East => coord.1,
            West => coord.1,
            North => coord.1+1,
            South => coord.1-1,
        };
        (x,y)
    }
}
async fn robot_run(mut rx: Receiver<isize>, mut tx: Sender<isize>) -> Result<HashMap<(isize,isize),PaintColor>,Error> {
    let mut paint_map = HashMap::new(); // map of paint colors by coords
    let mut robot_location = (0,0); // starting location (arbitrary)
    let mut robot_orientation = North; // initial orientation
    paint_map.entry(robot_location).or_insert(White); // Initialize starting location color;
    // Now process all messages
    loop {
        let this_panel_color = paint_map.entry(robot_location).or_insert(Black);
        if let Err(_) = tx.send(*this_panel_color as isize).await {
            return Err(Error::RobotComms { msg:format!("Robot output channel failure.  The following data is being discarded:\n   {:?}", this_panel_color) });
        }
        if let Some(color_v) = rx.next().await {
            *this_panel_color = PaintColor::try_from(color_v)?;
            println!("Robot: Painting {:?}", *this_panel_color);
        } else { break; }
        if let Some(turn_v) = rx.next().await {
            robot_orientation = robot_orientation.turn(TurnDirection::try_from(turn_v)?);
            robot_location = robot_orientation.step(robot_location);
            println!("Robot: At {:?} facing {:?}", robot_location, robot_orientation);
                // pass it through blindly...
        } else { break; }
    }
    Ok(paint_map)
}
// pub struct ManInTheMiddle<'a, T: Debug> {
//     original_tx: &'a mut Sender<T>,
//     tx: Sender<T>,
//     rx: Receiver<T>,
//     debug_prefix: Option<String>,
// }
// impl<'a, T> ManInTheMiddle<'a, T> 
//     where T: Debug {
//     fn new(original_tx: &'a mut Sender<T>, debug_prefix: &str) -> Self {
//         const BUFFER_SIZE: usize = 100;
//         let (tx, rx) = channel::<T>(BUFFER_SIZE);
//         let debug_prefix = if 0 == debug_prefix.len() {
//             None
//         } else {
//             Some(debug_prefix.to_owned())
//         };
//         ManInTheMiddle { original_tx, tx, rx, debug_prefix }
//     }
//     pub fn tx_ptr(&'a mut self) -> &'a mut Sender<T> {
//         &mut self.tx
//     }
//     async fn monitor(&mut self)
//         where T: Debug {
//         loop {
//             // fetch package
//             let msg: T = match self.rx.next().await {
//                 Some(m) => m,
//                 None => {
//                     if let Some(prefix) = self.debug_prefix {
//                         println!("{}: Terminating now due to input channel termination.", prefix);
//                     };
//                     return;
//                 }
//             };
//             // [optional] monitor output
//             if let Some(prefix) = self.debug_prefix {
//                 println!("{}: {:?}", prefix, msg);
//             };
//             // send package
//             if let Err(_) = self.tx.send(msg).await {
//                 if let Some(prefix) = self.debug_prefix {
//                     println!("{}: Output channel failure.  The following data is being discarded:\n   {:?}", prefix, msg);
//                 };    
//             }
//         }
//     }    
// }
async fn boot_intcode_and_robot(prog: Vec<isize>) -> Result<HashMap<(isize,isize),PaintColor>,Error> {
    const BUFFER_SIZE: usize = 100;
    let (robot_tx, computer_rx) = channel::<isize>(BUFFER_SIZE);
    let (computer_tx, robot_rx) = channel::<isize>(BUFFER_SIZE);
    let computer = intcode_run(prog.clone(), computer_rx, computer_tx);
    let robot = robot_run(robot_rx, robot_tx);
    let (_computer_return,robot_return) = join!(computer, robot); // , computer_snooper.monitor(), robot_snooper.monitor()
    robot_return
}
fn main() -> Result<(),Error> {
    const PROG_MEM_SIZE: usize = 2000;
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
    };
    let list_of_paint_color_by_location = match block_on(boot_intcode_and_robot(prog_orig.clone())) {
        Ok(list) => list,
        Err(e) => return Err(e),
    };
    println!("Part 1: {} locations painted at least once", list_of_paint_color_by_location.len());
    // Print out painting result:
    let (min_x, min_y, max_x, max_y) = list_of_paint_color_by_location.iter().fold((0,0,0,0), |(min_x, min_y, max_x, max_y), ((x,y),_color)| {
        (
            if *x < min_x {*x} else { min_x },
            if *y < min_y {*y} else { min_y },
            if *x > max_x {*x} else { max_x },
            if *y > max_y {*y} else { max_y },
        )
    });
    for y in (min_y..=max_y).rev() {
        for x in min_x..=max_x {
            let ch = match list_of_paint_color_by_location.get(&(x,y)) {
                Some(color) => match color {
                    Black => ' ',
                    White => '#',
                },
                None => ' ',
            };
            print!("{}", ch);
        }
        println!("");
    }
    Ok(())
}
