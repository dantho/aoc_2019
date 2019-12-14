/// https://adventofcode.com/2019/day/13#part2
extern crate crossterm;
const ESC_CLS:&'static str = "\x1B[2J";

use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, Write, stdout};
use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;
use std::collections::BTreeMap;
use futures::prelude::*;
use futures::channel::mpsc::{channel,Sender,Receiver};
use futures::executor::block_on;
use futures::join;
use TileID::*;
use JoystickPosition::*;
use std::time::Duration;

#[derive(Debug)]
enum Error {
    IllegalOpcode { code: isize },
    IllegalTileID { val: isize },
    ArcadeComms { msg: String },
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
#[derive(Debug,Copy,Clone,Eq,PartialEq)]
enum JoystickPosition {
    Left=-1,
    Neutral=0,
    Right=1,
}
#[derive(Debug,Copy,Clone,Eq,PartialEq)]
enum TileID {
    Empty=0,
    Wall=1,
    Block=2,
    HorizontalPaddle=3,
    Ball=4,
}
impl TryFrom<isize> for TileID {
    type Error = Error;
    fn try_from(val: isize) -> Result<Self, Self::Error> {
        use TileID::*;
        let tile_id = match val {
            n if n == Empty as isize => Empty,
            n if n == Wall as isize => Wall,
            n if n == Block as isize => Block,
            n if n == HorizontalPaddle as isize => HorizontalPaddle,
            n if n == Ball as isize => Ball,
            _ => return Err(Error::IllegalTileID { val }),
        };
        Ok(tile_id)
    }
}
// See https://jrgraphix.net/r/Unicode/2700-27BF for Dingbats in unicode
impl TileID {
    fn to_char(&self) -> char {
        match *self {
            Empty => ' ',
            Wall => '■',
            Block => '□',
            HorizontalPaddle => '═',
            Ball => '●',
        }
    }
}
fn print(s: &str) {
    print!("{}",s);
    stdout().flush().unwrap();
}
fn print_ch(ch: char) {
    print!("{}",ch);
    stdout().flush().unwrap();
}
fn set_cursor_pos(y:isize,x:isize) {
    print!("\x1B[{};{}H", y+1, x+1);
    stdout().flush().unwrap();
}
fn set_color(color:u8) {
    print(
        &format!("\x1B[{}m", 41 + color)
    );
}
async fn arcade_run(mut rx: Receiver<isize>, mut tx: Sender<isize>) -> Result<isize,Error> {
    const COLLISION_ROW: isize = 23;

    let mut arcade_screen = BTreeMap::new(); // grid of tiles
    let mut score: isize = 0;
    let mut ball_position: (isize,isize) = (0,0);
    let mut prior_ball_position: (isize,isize) = (0,0);
    let mut paddle_position: (isize,isize) = (0,0);

    // Do Not Print out WHOLE SCREEN on every character change: (too slow?)
    // print!("\u{001Bc}"); // clear screen, reset cursor
    print(ESC_CLS); // clear screen, reset cursor

    // process all messages
    loop {
        // Intcode Output
        let x = match rx.next().await {
            Some(x) => x,
            None => break,
        };
        let y = match rx.next().await {
            Some(y) => y,
            None => break,
        };
        if (0,-1) == (y,x) {
            score = match rx.next().await {
                Some(score) => {
                    set_cursor_pos(24,0);
                    println!("\nScore: {}\n\n", score);
                    score
                },
                None => break,
            };    
        } else {
            let tile_id = match rx.next().await {
                Some(tile_val) => TileID::try_from(tile_val)?,
                None => break,
            };
            set_cursor_pos(y,x);
            print_ch(tile_id.to_char());
            match tile_id {
                Ball => {
                    ball_position = (y,x);
                    // Control Joystick via Intcode Input
                    let joystick_position = if paddle_position.1 == ball_position.1 {
                        Neutral
                    } else if paddle_position.1 < ball_position.1 {
                        Right
                    } else {
                        Left
                    };
                    if let Err(_) = tx.send(joystick_position as isize).await {
                        return Err(Error::ArcadeComms { msg:format!("Arcade output channel failure.  The following data is being discarded:\n   {:?}", joystick_position) });
                    }
                },
                HorizontalPaddle => {
                    paddle_position = (y,x);
                },
                _ => (),
            }
            if let Some(_prior_tile) = arcade_screen.insert((y,x),tile_id) {};    
        }
        // // DEBUG PRINT WHOLE SCREEN
        // print(ESC_CLS); // clear screen, reset cursor
        // if arcade_screen.len() >= 960 {
        //     let (min_y, min_x, max_y, max_x) = arcade_screen.iter().fold((0,0,0,0), |(min_y, min_x, max_y, max_x), ((y,x),_tile_id)| {
        //         (
        //             if *x < min_x {*x} else { min_x },
        //             if *y < min_y {*y} else { min_y },
        //             if *x > max_x {*x} else { max_x },
        //             if *y > max_y {*y} else { max_y },
        //         )
        //     });
        //     for ((y,x), tile) in &arcade_screen {
        //         let ch = tile.to_char();
        //         set_cursor_pos(*y, *x);
        //         print_ch(ch);
        //     }
        //     // for y in min_y..=max_y {
        //     //     for x in min_x..=max_x {
        //     //         let ch = match arcade_screen.get(&(y,x)) {
        //     //             Some(tile) => tile.to_char(),
        //     //             None => ' ',
        //     //         };
        //     //         print!("{}", ch);
        //     //     }
        //     //     println!("");
        //     // }
        //     set_cursor_pos(24,0);
        //     println!("\nScore: {}\n\n", score);
        //     println!("Screen contains {} unique characters.", arcade_screen.len());
        //     println!("With {} of them Empty.", arcade_screen.iter().filter(|((_,_),tile)|{tile==&&Empty}).count());
        //     println!("And  {} of them Wall.", arcade_screen.iter().filter(|((_,_),tile)|{tile==&&Wall}).count());
        //     println!("And  {} of them Blocks.", arcade_screen.iter().filter(|((_,_),tile)|{tile==&&Block}).count());
        //     println!("Only {} is a Ball.", arcade_screen.iter().filter(|((_,_),tile)|{tile==&&Ball}).count());
        //     println!("And  {} is a Paddle.", arcade_screen.iter().filter(|((_,_),tile)|{tile==&&HorizontalPaddle}).count());
        //     println!("Screen goes from {}, {} to {}, {}", min_x, min_y, max_x, max_y);
        //     let delay = Duration::from_millis(0);
        //     std::thread::sleep(delay);
        // }
    }
    Ok(score)
}
async fn boot_intcode_and_arcade(prog: Vec<isize>) -> Result<isize,Error> {
    const BUFFER_SIZE: usize = 10;
    let (arcade_tx, computer_rx) = channel::<isize>(BUFFER_SIZE);
    let (computer_tx, arcade_rx) = channel::<isize>(BUFFER_SIZE);
    let mut hacked_program = prog.clone();
    hacked_program[0] = 2;
    let computer = intcode_run(hacked_program, computer_rx, computer_tx);
    let arcade = arcade_run(arcade_rx, arcade_tx);
    let (_computer_return,final_score) = join!(computer, arcade); // , computer_snooper.monitor(), arcade_snooper.monitor()
    final_score
}
fn main() -> Result<(),Error> {
    const PROG_MEM_SIZE: usize = 3000;
    let filename = "input.txt";
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
    let final_score = match block_on(boot_intcode_and_arcade(prog_orig.clone())) {
        Ok(score) => score,
        Err(e) => return Err(e),
    };
    println!("Part 2: Final score is {}", final_score );
    Ok(())
}
