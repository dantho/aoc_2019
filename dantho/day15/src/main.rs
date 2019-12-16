/// https://adventofcode.com/2019/day/13#part2
const ESC_CLS:&'static str = "\x1B[2J";
const ESC_CURSOR_ON:&'static str = "\x1B[2J";
const ESC_CURSOR_OFF:&'static str = "\x1B[2J";

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
use DroidStatus::*;
use DroidMovement::*;
use MapData::*;
use Error::*;
use std::time::Duration;

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
    let fewest_moves = match block_on(boot_intcode_and_droid(prog_orig.clone())) {
        Ok(score) => score,
        Err(e) => return Err(e),
    };
    println!("Part 1: Fewest moves to find the oxygen system is {}", fewest_moves );
    Ok(())
}
#[derive(Debug)]
enum Error {
    IllegalOpcode {code: isize},
    IllegalStatus {val: isize},
    DroidComms {msg: String},
    ComputerComms {msg: String},
    MapAssertFail {msg: String},
    MapOriginWrong {msg: String},
}
#[derive(Debug,Copy,Clone,Eq,PartialEq)]
enum MapData {
    Empty,
    Wall,
    OxygenSystem=3,
    Droid=4,
}
// See https://jrgraphix.net/r/Unicode/2700-27BF for Dingbats in unicode
impl MapData {
    fn to_char(&self) -> char {
        match *self {
            Wall => '■',
            Empty => '.',
            OxygenSystem => '☻', // '●',
            Droid => 'D',
        }
    }
}
#[derive(Debug,Copy,Clone,Eq,PartialEq)]
enum DroidMovement {
    North=1,
    South=2,
    West=3,
    East=4,
}
impl TryFrom<isize> for DroidMovement {
    type Error = Error;
    fn try_from(val: isize) -> Result<Self, Self::Error> {
        use DroidMovement::*;
        let status = match val {
            n if n == North as isize => North,
            n if n == South as isize => South,
            n if n == West as isize => West,
            n if n == East as isize => East,
            _ => return Err(Error::IllegalStatus { val }),
        };
        Ok(status)
    }
}
impl DroidMovement {
    fn move_from(&self, loc: (isize,isize)) -> (isize,isize) {
        match self {
            North => (loc.0-1, loc.1),
            South => (loc.0+1, loc.1),
            West => (loc.0, loc.1-1),
            East => (loc.0, loc.1+1),
        }
    }
}
#[derive(Debug,Copy,Clone,Eq,PartialEq)]
enum DroidStatus {
    HitWall=0,
    Moved=1,
    OxygenSystemDetected=2,
}
impl TryFrom<isize> for DroidStatus {
    type Error = Error;
    fn try_from(val: isize) -> Result<Self, Self::Error> {
        use DroidStatus::*;
        let status = match val {
            n if n == HitWall as isize => HitWall,
            n if n == Moved as isize => Moved,
            n if n == OxygenSystemDetected as isize => OxygenSystemDetected,
            _ => return Err(Error::IllegalStatus { val }),
        };
        Ok(status)
    }
}
struct WorldMap {
    origin: (isize,isize),
    data: BTreeMap<(isize,isize), MapData>,
}
impl WorldMap {
    fn new() -> Self {
        let origin = (-5,-5);
        let data = BTreeMap::new();
        WorldMap {origin, data}
    }
    fn modify_data(&mut self, position: (isize,isize), data: MapData) -> Result<(),Error> {
        self.update_origin(position)?;
        match self.data.get_mut(&position) {
            None => {
                self.data.insert(position, data);
            },
            Some(&mut Wall) => {
                if data != Wall {
                    return Err(Error::MapAssertFail {msg: format!("Placing {:?} on Wall at {:?}", data, position)});
                }
            },
            Some(map_data_here) => {
                *map_data_here = data;
            }
        }
        self.draw_position(position)?;
        Ok(())
    }
    fn draw_position(&self, pos: (isize,isize)) -> Result<(),Error> {
        if pos.0 < self.origin.0 || pos.1 < self.origin.1 {
            return Err(MapOriginWrong {
                msg: format!("Map pos {:?} is lower than origin at {:?}", pos, self.origin)})}
        set_cursor_pos(pos.0 - self.origin.0, pos.1 - self.origin.0);
        let ch = match self.data.get(&pos) {
            None => ' ',
            Some(data) => data.to_char(),
        };
        print_ch(ch);
        Ok(())
    }
    fn redraw_screen(&self) -> Result<(),Error> {
        print(ESC_CLS); // clear screen, reset cursor
        for (pos, _) in &self.data {
            self.draw_position(*pos)?;
        }
        Ok(())
        // // DEBUG PRINT WHOLE SCREEN
        // print(ESC_CLS); // clear screen, reset cursor
        // if droid_screen.len() >= 960 {
        //     let (min_y, min_x, max_y, max_x) = droid_screen.iter().fold((0,0,0,0), |(min_y, min_x, max_y, max_x), ((y,x),_tile_id)| {
        //         (
        //             if *x < min_x {*x} else { min_x },
        //             if *y < min_y {*y} else { min_y },
        //             if *x > max_x {*x} else { max_x },
        //             if *y > max_y {*y} else { max_y },
        //         )
        //     });
        //     for ((y,x), tile) in &droid_screen {
        //         let ch = tile.to_char();
        //         set_cursor_pos(*y, *x);
        //         print_ch(ch);
        //     }
        //     // for y in min_y..=max_y {
        //     //     for x in min_x..=max_x {
        //     //         let ch = match droid_screen.get(&(y,x)) {
        //     //             Some(tile) => tile.to_char(),
        //     //             None => ' ',
        //     //         };
        //     //         print!("{}", ch);
        //     //     }
        //     //     println!("");
        //     // }
        //     set_cursor_pos(24,0);
        //     println!("\nScore: {}\n\n", score);
        //     println!("Screen contains {} unique characters.", droid_screen.len());
        //     println!("With {} of them Empty.", droid_screen.iter().filter(|((_,_),tile)|{tile==&&Empty}).count());
        //     println!("And  {} of them Wall.", droid_screen.iter().filter(|((_,_),tile)|{tile==&&Wall}).count());
        //     println!("And  {} of them Blocks.", droid_screen.iter().filter(|((_,_),tile)|{tile==&&Block}).count());
        //     println!("Only {} is a Ball.", droid_screen.iter().filter(|((_,_),tile)|{tile==&&Ball}).count());
        //     println!("And  {} is a Paddle.", droid_screen.iter().filter(|((_,_),tile)|{tile==&&HorizontalPaddle}).count());
        //     println!("Screen goes from {}, {} to {}, {}", min_x, min_y, max_x, max_y);
        //     let delay = Duration::from_millis(0);
        //     std::thread::sleep(delay);
        // }
    }
    fn update_origin(&mut self, position:(isize,isize)) -> Result<(),Error> {
        let mut redraw_required = false;
        if position.0 < self.origin.0 {
            self.origin.0 -= 5;
            redraw_required = true;
        }
        if position.1 < self.origin.1 {
            self.origin.1 -= 5;
            redraw_required = true;
        }
        if redraw_required {
            self.redraw_screen()?;
        }
        Ok(())
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
struct Droid {
    explored_world: WorldMap,
    droid_position: (isize,isize),
    oxygen_position_if_known: Option<(isize,isize)>,
    rx: Receiver<isize>,
    tx: Sender<isize>,
}
impl Droid {
    fn new(rx: Receiver<isize>, tx: Sender<isize>) -> Self {
        let mut explored_world = WorldMap::new();
        let droid_position: (isize,isize) = (0,0);
        let oxygen_position_if_known: Option<(isize,isize)> = None;  // Unknown as yet
        explored_world.data.insert(droid_position, MapData::Droid);
        Droid { explored_world, droid_position, oxygen_position_if_known, rx, tx }  
    }
    async fn explore(&mut self) -> Result<(),Error> {
        for dir in &[North, South, West, East] {
            let move_dir = *dir;
            // Slow things down for debug or visualization
            let delay = Duration::from_millis(1000);
            std::thread::sleep(delay);
            // Send a movement command to Droid's Intcode Computer
            if let Err(_) = self.tx.send(move_dir as isize).await {
                return Err(Error::DroidComms { msg:format!("Droid output channel failure.  The following data is being discarded:\n   {:?}", move_dir) });
            }
            // And fetch a response
            let status = match self.rx.next().await {
                Some(st) => DroidStatus::try_from(st)?,
                None => break,
            };
            // Interpret response
            match status {
                HitWall => {
                    // Add new data to map; Or validate known (& identical) data in map
                    // Do not change location
                    // Print new data (for new location only)
                    let wall_position = move_dir.move_from(self.droid_position);
                    self.explored_world.modify_data(wall_position, Wall)?;
                },
                Moved => {
                    // clear up old droid location
                    self.explored_world.modify_data(self.droid_position, Empty)?; // probably Empty
                    if let Some(ox) = self.oxygen_position_if_known {
                        if ox == self.droid_position {
                            self.explored_world.modify_data(self.droid_position, OxygenSystem)?;
                        }
                    }
                    // move droid
                    self.droid_position = move_dir.move_from(self.droid_position);
                    self.explored_world.modify_data(self.droid_position, Droid)?;
                },
                OxygenSystemDetected => {
                    // clear up old droid location
                    self.explored_world.modify_data(self.droid_position, Empty)?; // definitely Empty
                    // move droid
                    self.droid_position = move_dir.move_from(self.droid_position);
                    self.explored_world.modify_data(self.droid_position, Droid)?; // Or Droid_Oxygen combo?
                    // and udate crucial information
                    self.oxygen_position_if_known = Some(self.droid_position);
                },
            }
        }
        Ok(())
    }
}
async fn droid_run(rx: Receiver<isize>, tx: Sender<isize>) -> Result<u32,Error> {
    let mut droid = Droid::new(rx, tx);
    print(ESC_CLS);
    print(ESC_CURSOR_OFF);
    // start the big loop
    loop {
        droid.explore().await?;
    }
    // print(ESC_CURSOR_ON);
    // Ok(0)
}
async fn boot_intcode_and_droid(prog: Vec<isize>) -> Result<u32,Error> {
    const BUFFER_SIZE: usize = 10;
    let (droid_tx, computer_rx) = channel::<isize>(BUFFER_SIZE);
    let (computer_tx, droid_rx) = channel::<isize>(BUFFER_SIZE);
    let hacked_program = prog.clone();
    // No hacks
    let computer = intcode_run(hacked_program, computer_rx, computer_tx);
    let droid = droid_run(droid_rx, droid_tx);
    let (_computer_return,droid_response) = join!(computer, droid); // , computer_snooper.monitor(), droid_snooper.monitor()
    droid_response
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
