/// tps://adventofcode.com/2019/day/19
const ESC_CLS: &'static str = "\x1B[2J";
const ESC_CURSOR_OFF: &'static str = "\x1B[?25l";
// const ESC_CURSOR_ON: &'static str = "\x1B[?25h";

mod intcode;

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
use futures::future::BoxFuture; // https://rust-lang.github.io/async-book/07_workarounds/05_recursion.html
use DroidStatus::*;
use DroidMovement::*;
use MapData::*;
use Error::*;
use std::time::Duration;

fn main() -> Result<(),Error> {
    const PROG_MEM_SIZE: usize = 1000;
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
    let (affected_points, part2) = match block_on(boot_intcode_and_droid(prog_orig.clone())) {
        Ok(result) => result,
        Err(e) => return Err(e),
    };
    println!("");
    println!("Part 1: {} Points are affected by tracter beam in 50x50 area", affected_points );
    println!("Part 2: TBD {}", part2 );
    Ok(())
}
async fn boot_intcode_and_droid(prog: Vec<isize>) -> Result<(usize,usize),Error> {
    const BUFFER_SIZE: usize = 10;
    let (droid_tx, computer_rx) = channel::<isize>(BUFFER_SIZE);
    let (computer_tx, droid_rx) = channel::<isize>(BUFFER_SIZE);
    let mut computer = intcode::Intcode::new(prog, computer_rx, computer_tx);
    let droid = droid_run(droid_rx, droid_tx);
    let (_computer_return,droid_response) = join!(computer.run(), droid); // , computer_snooper.monitor(), droid_snooper.monitor()
    droid_response
}
async fn droid_run(rx: Receiver<isize>, tx: Sender<isize>) -> Result<(usize,usize),Error> {
    let mut droid = Droid::new(rx, tx);
    // droid.explored_world.redraw_screen()?;
    droid.explore().await?;
    let tractor_beam_count = droid.explored_world.data.iter().fold(0, |cnt, ((_,_), m)| {
        cnt + if m == &TractorBeam {1} else {0}
    });
    Ok((tractor_beam_count,0))
}
#[derive(Debug)]
enum Error {
    IllegalStatus {val: isize},
    DroidComms {msg: String},
    MapAssertFail {msg: String},
    MapOriginWrong {msg: String},
}
#[derive(Debug,Copy,Clone,Eq,PartialEq)]
enum MapData {
    Empty,
    TractorBeam,
}
// See https://jrgraphix.net/r/Unicode/2700-27BF for Dingbats in unicode
impl MapData {
    fn to_str(&self) -> &'static str {
        print("\x1B[56m"); // SIDE EFFECT - print white control chars (default color)
        match *self {
            Empty => ".",
            TractorBeam => "#",
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
    fn move_from(&self, loc: (usize,usize)) -> (usize,usize) {
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
#[derive(Debug,Copy,Clone,Eq,PartialEq)]
enum DroidStatus {
    Nothing=0,
    BeamDetected=1,
}
impl TryFrom<isize> for DroidStatus {
    type Error = Error;
    fn try_from(val: isize) -> Result<Self, Self::Error> {
        use DroidStatus::*;
        let status = match val {
            n if n == Nothing as isize => Nothing,
            n if n == BeamDetected as isize => BeamDetected,
            _ => return Err(Error::IllegalStatus { val }),
        };
        Ok(status)
    }
}
struct WorldMap {
    origin: (usize,usize),
    data: BTreeMap<(usize,usize), MapData>,
}
impl WorldMap {
    fn new() -> Self {
        let origin = (0,0);
        let data = BTreeMap::new();
        WorldMap {origin, data}
    }
    fn is_known(&self, loc: &(usize,usize)) -> bool {
        self.data.contains_key(loc)
    }
    fn modify_data(&mut self, location: (usize,usize), data: MapData) -> Result<(),Error> {
        match self.data.get_mut(&location) {
            None => {
                self.data.insert(location, data);
            },
            Some(map_data_here) => {
                *map_data_here = data;
            }
        }
        Ok(())
    }
    fn draw_location(&self, loc: (usize,usize)) -> Result<(),Error> {
        if loc.0 < self.origin.0 || loc.1 < self.origin.1 {
            return Err(MapOriginWrong {
                msg: format!("Map loc {:?} is lower than origin at {:?}", loc, self.origin)})}
        set_cursor_loc(loc.0 - self.origin.0, loc.1 - self.origin.1);
        let map_item = match self.data.get(&loc) {
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
        for (loc, _) in &self.data {
            self.draw_location(*loc)?;
        }
        Ok(())
    }
    fn update_origin(&mut self, location:(usize,usize)) -> Result<(),Error> {
        let mut redraw_required = false;
        if location.0 < self.origin.0 {
            self.origin = (self.origin.0-5, self.origin.1);
            redraw_required = true;
        }
        if location.1 < self.origin.1 {
            self.origin = (self.origin.0, self.origin.1-5);
            redraw_required = true;
        }
        if redraw_required {
            self.redraw_screen()?;
            set_cursor_loc(20, 20);
        }
        Ok(())
    }
    fn lower_right_corner(&self) -> (usize,usize) {
        self.data.iter().fold((std::usize::MIN,std::usize::MIN),|(max_y,max_x), ((y,x),_)| {
            (
                if *y > max_y {*y} else {max_y},
                if *x > max_x {*x} else {max_x}
            )
        })
    }
    fn lower_right_corner_on_screen(&self) -> (usize,usize) {
        let signed_location = self.lower_right_corner();
        let on_screen = (signed_location.0-self.origin.0, signed_location.1-self.origin.1);
        on_screen
    }
}
fn print(s: &str) {
    print!("{}",s);
    stdout().flush().unwrap();
}
// fn print_ch(ch: char) {
//     print!("{}",ch);
//     stdout().flush().unwrap();
// }
fn set_cursor_loc(y:usize,x:usize) {
    print!("\x1B[{};{}H", y+1, x+1);
    stdout().flush().unwrap();
}
// fn set_color(color:u8) {
//     print(
//         &format!("\x1B[{}m", 41 + color)
//     );
// }
struct Droid {
    explored_world: WorldMap,
    droid_location: (usize,usize),
    rx: Receiver<isize>,
    tx: Sender<isize>,
}
impl Droid {
    fn new(rx: Receiver<isize>, tx: Sender<isize>) -> Self {
        let mut explored_world = WorldMap::new();
        let droid_location: (usize,usize) = (0,0);
        Droid { explored_world, droid_location, rx, tx }
    }
    // explore() is a trivial grid march
    // See https://rust-lang.github.io/async-book/07_workarounds/05_recursion.html 
    //    for explanation of fn syntax and async block usage
    fn explore<'a>(&'a mut self) -> BoxFuture<'a, Result<(),Error>> {
        async move {
            for y in 0..50 {
                for x in 0..50 {
                    match self.move_droid(y,x).await? {
                        BeamDetected => {
                            println!("Modifying self.explored_world");
                            self.explored_world.modify_data((y,x), TractorBeam)?},
                        Nothing => {
                            println!("Modifying self.explored_world");
                            self.explored_world.modify_data((y,x), Empty)?
                        },
                    };        
                };
            };
            Ok(())
        }.boxed()
    }
    async fn move_droid(&mut self, y: usize, x: usize) -> Result<DroidStatus,Error> {
        // Slow things down for debug or visualization
        // ESPECIALLY at start
        let delay = Duration::from_millis(0);
        std::thread::sleep(delay);
        // Send x of movement command to Droid's Intcode Computer
        if let Err(_) = self.tx.send(x as isize).await {
            return Err(Error::DroidComms { msg:format!("Droid output channel failure.  The following data is being discarded:\n   {:?}", x) });
        }
        // Send y of movement command to Droid's Intcode Computer
        if let Err(_) = self.tx.send(y as isize).await {
            return Err(Error::DroidComms { msg:format!("Droid output channel failure.  The following data is being discarded:\n   {:?}", y) });
        }
        // And fetch a response
        println!("Attempting to fetch move response...");
        let status = match self.rx.next().await {
            Some(st) => {
                println!("Status read from Intcode: {}", st);
                let ds = DroidStatus::try_from(st)?;
                println!("DroidStatus is {:?}", ds);
                ds
            },
            None => return Err(DroidComms {msg: "Intcode computer stopped transmitting.".to_string()}),
        };
        Ok(status)
    }
}