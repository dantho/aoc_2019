/// https://adventofcode.com/2019/day/17
const ESC_CLS: &'static str = "\x1B[2J";
// const ESC_CURSOR_ON: &'static str = "\x1B[?25h";
const ESC_CURSOR_OFF: &'static str = "\x1B[?25l";

mod intcode;

use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, Write, stdout};
use std::convert::TryFrom;
use std::fmt::Debug;
use std::collections::BTreeMap;
use futures::prelude::*;
use futures::channel::mpsc::{channel,Sender,Receiver};
use futures::executor::block_on;
use futures::join;
use DroidMovement::*;
use DroidCommand::*;
use MapData::*;
use Error::*;

type Location = (isize,isize);

fn main() -> Result<(),Error> {
    const PROG_MEM_SIZE: usize = 6000;
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
    let (part1,part2) = match block_on(boot_intcode_and_droid(prog_orig.clone())) {
        Ok(result) => result,
        Err(e) => return Err(e),
    };
    println!("");
    println!("Part 1: Sum of alignment parameters is {}", part1);
    println!("Part 2: Dust collected {}", part2);
    // println!("Part 2: xxx is {}", xxx);
    Ok(())
}
async fn boot_intcode_and_droid(prog: Vec<isize>) -> Result<(isize,isize),Error> {
    const BUFFER_SIZE: usize = 10;
    let (droid_tx, computer_rx) = channel::<isize>(BUFFER_SIZE);
    let (computer_tx, droid_rx) = channel::<isize>(BUFFER_SIZE);
    let unhacked_program = prog.clone();
    // No hacks for Part 1
    let computer = intcode::intcode_run(unhacked_program, computer_rx, computer_tx);
    let droid = droid_run_part1(droid_rx, droid_tx);
    let (_computer_return,droid_response_part1) = join!(computer, droid); // , computer_snooper.monitor(), droid_snooper.monitor()

    Ok((droid_response_part1.unwrap(), 0))
}
async fn droid_run_part1(rx: Receiver<isize>, tx: Sender<isize>) -> Result<isize,Error> {
    let mut droid = Droid::new(rx, tx);
    let response = droid.read_response().await?;
    println!("Initial response: {}", response);
    let response = droid.read_response().await?;
    println!("Arbitrary 2nd response: {}", response);
    droid.send_command(Move{dir: North}).await?;
    let response = droid.read_response().await?;
    println!("Response to Move command: {}", response);
    // droid.santas_ship.redraw_screen()?;
    // println!("\nIntersections: {:?}", intersections);
    Ok(0)
}
#[derive(Debug)]
enum Error {
    ComputerError {internal: intcode::Error},
    IllegalDroidResponse {val: isize},
    DroidComms {msg: String},
    MapOriginWrong {msg: String},
    MapAssertFail {msg: String},
}
#[derive(Debug,Copy,Clone,Eq,PartialEq)]
enum MapData {
    Empty=46,        // '.'
}
// See https://jrgraphix.net/r/Unicode/2700-27BF for Dingbats in unicode
impl MapData {
    fn to_str(&self) -> &'static str {
        print("\x1B[56m"); // SIDE EFFECT - print white control chars (default color)
        match *self {
            Empty => "•",
        }
    }
}
impl TryFrom<isize> for MapData {
    type Error = Error;
    fn try_from(val: isize) -> Result<Self, Self::Error> {
        use MapData::*;
        let status = match val {
            n if n == Empty as isize => Empty,
            _ => return Err(Error::IllegalDroidResponse { val }),
        };
        Ok(status)
    }
}
enum DroidCommand {
    Move {dir: DroidMovement},
    Inventory,
    TakeItem {item: String},
    DropItem {item: String},
}
impl DroidCommand {
    fn to_str(&self) -> String {
        let tmp: String;
        match self {
            Move{dir} => match dir {
                North => "north",
                South => "south",
                East => "east",
                West => "west",
            },
            Inventory => "inv",
            TakeItem{item} => {
                tmp = format!("take {}", item);
                &tmp
            },
            DropItem{item} => {
                tmp = format!("drop {}", item);
                &tmp
            },
        }.to_string()
    }
}
struct Droid {
    santas_ship: WorldMap,
    present_location: Location,
    rx: Receiver<isize>,
    tx: Sender<isize>,
}
impl Droid {
    fn new(rx: Receiver<isize>, tx: Sender<isize>) -> Self {
        let santas_ship = WorldMap::new();
        let present_location = santas_ship.origin;
        Droid { santas_ship, present_location, rx, tx }
    }
    async fn send_command(&mut self, command: DroidCommand) -> Result<(),Error> {
        for ch in command.to_str().chars().chain(vec!['\n'].into_iter()) {
            // Send a command to Droid's Intcode Computer
            if let Err(_) = self.tx.send(ch as isize).await {
                return Err(Error::DroidComms { msg:format!("Droid output channel failure.  The following data is being discarded:\n   {:?}", ch) });
            }
        }
        Ok(())
    }
    async fn read_response(&mut self) -> Result<String,Error> {
        let mut response = String::new();
        loop {
            // Fetch prompt
            match self.rx.next().await {
                Some(10) => {break;},
                Some(ans) => response.push(ans as u8 as char),
                None => return Err(DroidComms {msg: "Incode computer stopped transmitting.".to_string()}),
            };
        }
        Ok(response)
    }
}
#[derive(Debug,Copy,Clone,Eq,PartialEq)]
enum DroidMovement {
    North=1,
    East=2,
    South=3,
    West=4,
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
            _ => return Err(Error::IllegalDroidResponse { val }),
        };
        Ok(status)
    }
}
impl DroidMovement {
    fn move_from(&self, loc: Location) -> Location {
        match self {
            North => (loc.0-1, loc.1),
            South => (loc.0+1, loc.1),
            West => (loc.0, loc.1-1),
            East => (loc.0, loc.1+1),
        }
    }
    fn reverse(&self) -> Self {
        match self {
            North => South,
            South => North,
            West => East,
            East => West,            
        }
    }
}
struct WorldMap {
    origin: Location,
    data: BTreeMap<Location, MapData>,
}
impl WorldMap {
    fn new() -> Self {
        let origin = (-2,-2);
        let data = BTreeMap::new();
        WorldMap {origin, data}
    }
    fn draw_position(&self, pos: Location) -> Result<(),Error> {
        if pos.0 < self.origin.0 || pos.1 < self.origin.1 {
            return Err(MapOriginWrong {
                msg: format!("Map pos {:?} is lower than origin at {:?}", pos, self.origin)})}
        set_cursor_pos(pos.0 - self.origin.0, pos.1 - self.origin.1);
        let map_item = match self.data.get(&pos) {
            None => " ",
            Some(data) => data.to_str(),
        };
        print(map_item);
        Ok(())
    }
    fn redraw_screen(&self) -> Result<(),Error> {
        print(ESC_CLS); // clear screen, reset cursor
        print(ESC_CURSOR_OFF); // Turn OFF cursor
        // print(ESC_CURSOR_ON); // Turn ON cursor
        for (pos, _) in &self.data {
            self.draw_position(*pos)?;
        }
        println!("");
        Ok(())
    }
}
fn print(s: &str) {
    print!("{}",s);
    stdout().flush().unwrap();
}
fn set_cursor_pos(y:isize,x:isize) {
    print!("\x1B[{};{}H", y+1, x+1);
    stdout().flush().unwrap();
}
// fn print_ch(ch: char) {
//     print!("{}",ch);
//     stdout().flush().unwrap();
// }
// fn set_color(color:u8) {
//     print(
//         &format!("\x1B[{}m", 41 + color)
//     );
// }
