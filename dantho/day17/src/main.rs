/// https://adventofcode.com/2019/day/17
const ESC_CLS: &'static str = "\x1B[2J";
// const ESC_CURSOR_ON: &'static str = "\x1B[?25h";
const ESC_CURSOR_OFF: &'static str = "\x1B[?25l";

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
use RobotStatus::*;
use RobotMovement::*;
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
    let (fewest_moves, most_minutes) = match block_on(boot_intcode_and_robot(prog_orig.clone())) {
        Ok(result) => result,
        Err(e) => return Err(e),
    };
    println!("");
    println!("Part 1: Fewest moves to find the oxygen system is {}", fewest_moves );
    println!("Part 2: Minutes to fill every corner with oxygen is {}", most_minutes );
    Ok(())
}
async fn boot_intcode_and_robot(prog: Vec<isize>) -> Result<isize,Error> {
    const BUFFER_SIZE: usize = 10;
    let (robot_tx, computer_rx) = channel::<isize>(BUFFER_SIZE);
    let (computer_tx, robot_rx) = channel::<isize>(BUFFER_SIZE);
    let hacked_program = prog.clone();
    // No hacks
    let computer = intcode_run(hacked_program, computer_rx, computer_tx);
    let robot = robot_run(robot_rx, robot_tx);
    let (_computer_return,robot_response) = join!(computer, robot); // , computer_snooper.monitor(), robot_snooper.monitor()
    robot_response
}
async fn robot_run(rx: Receiver<isize>, tx: Sender<isize>) -> Result<isize,Error> {
    let mut robot = Robot::new(rx, tx);
    robot.camera_view.redraw_screen()?;
    robot.download_camera_view().await?;

    let sum_of_alignment_params = robot.camera_view.data.iter()
        .filter(|(_,item)| {**item == Intersection})
        .fold(0,|sum, ((y,x), _) | {
            *y**x
        });

    Ok(sum_of_alignment_params)

    // // Now that the full view is known (by the robot)
    // let lower_right_corner = robot.camera_view.lower_right_corner_on_screen();
    // set_cursor_pos(lower_right_corner.0+1,0);
    // let distance_to_oxygen_sensor: usize = match distance_map.get(&robot.oxygen_position_if_known.unwrap()) {
    //     Some(d) => *d,
    //     None => return Err(Error::MapAssertFail {msg: format!("Can't find oxygen sensor at {:?} !", robot.oxygen_position_if_known.unwrap())}),
    // };

    // // reset distance map
    // for (_,dist) in &mut distance_map {
    //     *dist = std::usize::MAX;
    // }

    // map_distance(&mut distance_map, robot.oxygen_position_if_known.unwrap(), 0)?;
    // let minutes_to_fill_with_oxygen = distance_map.iter().fold(0,|most_minutes, ((_,_), minutes)| {
    //     if *minutes > most_minutes {*minutes} else {most_minutes}
    // });
    // Ok((distance_to_oxygen_sensor, minutes_to_fill_with_oxygen))
}
fn map_distance(map: &mut BTreeMap<(isize,isize), usize>, loc: (isize,isize), distance: usize) -> Result<(),Error> {
    let this_loc = match map.get_mut(&loc) {
        Some(dist) => dist,
        None => return Ok(()), // END RECURSION (Scaffold or unknown found)
    };
    if *this_loc <= distance {
        return Ok(()) // END RECURSION (crossed [equiv or superior] path)
    }
    *this_loc = distance; // Set present location
    // Recurse into cardinal directions
    map_distance(map, (loc.0-1,loc.1), distance+1)?; // North
    map_distance(map, (loc.0+1,loc.1), distance+1)?; // South
    map_distance(map, (loc.0,loc.1-1), distance+1)?; // West
    map_distance(map, (loc.0,loc.1+1), distance+1)?; // East
    Ok(())
}
#[derive(Debug)]
enum Error {
    IllegalOpcode {code: isize},
    IllegalStatus {val: isize},
    RobotComms {msg: String},
    ComputerComms {msg: String},
    MapAssertFail {msg: String},
    MapOriginWrong {msg: String},
}
#[derive(Debug,Copy,Clone,Eq,PartialEq)]
enum MapData {
    Empty=46,        // '.'
    Scaffold=35,     // '#'
    NewLine=10,      // '\n'
    Intersection=79, // 'O'
    Up=94,           // '^'
    Down=118,        // 'v'
    Left=60,         // '<'
    Right=62,        // '>'
    TumblingThroughSpace=88, // 'X'
}
// See https://jrgraphix.net/r/Unicode/2700-27BF for Dingbats in unicode
impl MapData {
    fn to_str(&self) -> &'static str {
        print("\x1B[56m"); // SIDE EFFECT - print white control chars (default color)
        match *self {
            Scaffold => "■",
            Empty => ".",
            Intersection => "☻",
            Up => "^",
            Down =>"v",
            Left =>"<",
            Right =>">",
        }
    }
}
#[derive(Debug,Copy,Clone,Eq,PartialEq)]
enum RobotMovement {
    North=1,
    South=2,
    West=3,
    East=4,
}
impl TryFrom<isize> for RobotMovement {
    type Error = Error;
    fn try_from(val: isize) -> Result<Self, Self::Error> {
        use RobotMovement::*;
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
impl RobotMovement {
    fn move_from(&self, loc: (isize,isize)) -> (isize,isize) {
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
enum RobotStatus {
    HitScaffold=0,
    Moved=1,
    OxygenSystemDetected=2,
}
impl TryFrom<isize> for RobotStatus {
    type Error = Error;
    fn try_from(val: isize) -> Result<Self, Self::Error> {
        use RobotStatus::*;
        let status = match val {
            n if n == HitScaffold as isize => HitScaffold,
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
    fn is_known(&self, pos: &(isize,isize)) -> bool {
        self.data.contains_key(pos)
    }
    fn modify_data(&mut self, position: (isize,isize), data: MapData) -> Result<(),Error> {
        self.update_origin(position)?;
        match self.data.get_mut(&position) {
            None => {
                self.data.insert(position, data);
            },
            Some(&mut Scaffold) => {
                if data != Scaffold {
                    return Err(Error::MapAssertFail {msg: format!("Placing {:?} on Scaffold at {:?}", data, position)});
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
        Ok(())
    }
    fn update_origin(&mut self, position:(isize,isize)) -> Result<(),Error> {
        let mut redraw_required = false;
        if position.0 < self.origin.0 {
            self.origin = (self.origin.0-5, self.origin.1);
            redraw_required = true;
        }
        if position.1 < self.origin.1 {
            self.origin = (self.origin.0, self.origin.1-5);
            redraw_required = true;
        }
        if redraw_required {
            self.redraw_screen()?;
            set_cursor_pos(20, 20);
        }
        Ok(())
    }
    fn lower_right_corner(&self) -> (isize,isize) {
        self.data.iter().fold((std::isize::MIN,std::isize::MIN),|(max_y,max_x), ((y,x),_)| {
            (
                if *y > max_y {*y} else {max_y},
                if *x > max_x {*x} else {max_x}
            )
        })
    }
    fn lower_right_corner_on_screen(&self) -> (isize,isize) {
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
fn set_cursor_pos(y:isize,x:isize) {
    print!("\x1B[{};{}H", y+1, x+1);
    stdout().flush().unwrap();
}
// fn set_color(color:u8) {
//     print(
//         &format!("\x1B[{}m", 41 + color)
//     );
// }
struct Robot {
    camera_view: WorldMap,
    robot_position: (isize,isize),
    oxygen_position_if_known: Option<(isize,isize)>,
    rx: Receiver<isize>,
    tx: Sender<isize>,
}
impl Robot {
    fn new(rx: Receiver<isize>, tx: Sender<isize>) -> Self {
        let mut camera_view = WorldMap::new();
        let robot_position: (isize,isize) = (0,0);
        let oxygen_position_if_known: Option<(isize,isize)> = None;  // Unknown as yet
        camera_view.data.insert(robot_position, MapData::Robot);
        Robot { camera_view, robot_position, oxygen_position_if_known, rx, tx }
    }
    // download_camera_view() is a recursive algorithm (4-way) to visit all UNVISITED squares to determine the contents.
    // A previously visited square of any kind (preemptively) ENDS the (leg of the 4-way) recursion.
    // See https://rust-lang.github.io/async-book/07_workarounds/05_recursion.html 
    //    for explanation of fn syntax and async block usage
    fn download_camera_view<'a>(&'a mut self) -> BoxFuture<'a, Result<(),Error>> {
        async move {
            // Explore cardinal directions, returning to center each time
            for dir in &[North, South, West, East] {
                let move_dir = *dir;
                if !self.camera_view.is_known(&move_dir.move_from(self.robot_position)) {
                    if self.move_robot(move_dir).await? {
                        // Then download_camera_view there
                        self.download_camera_view().await?;
                        // and move back to continue more local exploration
                        self.move_robot(move_dir.reverse()).await?;
                    }
                } 
            }
            Ok(())
        }.boxed()
    }
    async fn move_robot(&mut self, move_dir: RobotMovement) -> Result<bool,Error> {
        let move_succeeded: bool;
        // Slow things down for debug or visualization
        // ESPECIALLY at start
        let delay = Duration::from_millis(0);
        std::thread::sleep(delay);
        // Send a movement command to Robot's Intcode Computer
        if let Err(_) = self.tx.send(move_dir as isize).await {
            return Err(Error::RobotComms { msg:format!("Robot output channel failure.  The following data is being discarded:\n   {:?}", move_dir) });
        }
        // And fetch a response
        let status = match self.rx.next().await {
            Some(st) => RobotStatus::try_from(st)?,
            None => return Err(RobotComms {msg: "Incode computer stopped transmitting.".to_string()}),
        };
        // Interpret response
        match status {
            HitScaffold => {
                move_succeeded = false;
                let wall_position = move_dir.move_from(self.robot_position);
                self.camera_view.modify_data(wall_position, Scaffold)?;
            },
            Moved => {
                move_succeeded = true;
                // clear up old robot location
                self.camera_view.modify_data(self.robot_position, Empty)?; // Empty unless...
                if let Some(ox) = self.oxygen_position_if_known {
                    if ox == self.robot_position {
                        self.camera_view.modify_data(self.robot_position, OxygenSystem)?;
                    }
                }
                // move robot
                self.robot_position = move_dir.move_from(self.robot_position);
                self.camera_view.modify_data(self.robot_position, Robot)?;
            },
            OxygenSystemDetected => {
                move_succeeded = true;
                // clear up old robot location
                self.camera_view.modify_data(self.robot_position, Empty)?; // definitely Empty
                // move robot
                self.robot_position = move_dir.move_from(self.robot_position);
                self.camera_view.modify_data(self.robot_position, Robot)?; // Or Robot_Oxygen combo?
                // and udate crucial information
                self.oxygen_position_if_known = Some(self.robot_position);
            },
        }
        Ok(move_succeeded)
    }
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
