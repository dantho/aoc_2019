/// https://adventofcode.com/2019/day/17
const ESC_CLS: &'static str = "\x1B[2J";
// const ESC_CURSOR_ON: &'static str = "\x1B[?25h";
const ESC_CURSOR_OFF: &'static str = "\x1B[?25l";

use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, Write, stdout};
use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;
use std::collections::{BTreeMap, HashSet, HashMap};
use futures::prelude::*;
use futures::channel::mpsc::{channel,Sender,Receiver};
use futures::executor::block_on;
use futures::join;
use RobotMovement::*;
use MapData::*;
use Error::*;
use std::time::Duration;

type Location = (isize,isize);

fn main() -> Result<(),Error> {
    const PROG_MEM_SIZE: usize = 4000;
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
    let (part1,part2) = match block_on(boot_intcode_and_robot(prog_orig.clone())) {
        Ok(result) => result,
        Err(e) => return Err(e),
    };
    println!("");
    println!("Part 1: Sum of alignment parameters is {}", part1);
    println!("Part 2: Dust collected {}", part2);
    // println!("Part 2: xxx is {}", xxx);
    Ok(())
}
async fn boot_intcode_and_robot(prog: Vec<isize>) -> Result<(isize,isize),Error> {
    const BUFFER_SIZE: usize = 10;
    let (robot_tx, computer_rx) = channel::<isize>(BUFFER_SIZE);
    let (computer_tx, robot_rx) = channel::<isize>(BUFFER_SIZE);
    let unhacked_program = prog.clone();
    // No hacks for Part 1
    let computer = intcode_run(unhacked_program, computer_rx, computer_tx);
    let robot = robot_run_part1(robot_rx, robot_tx);
    let (_computer_return,robot_response_part1) = join!(computer, robot); // , computer_snooper.monitor(), robot_snooper.monitor()
    // Part 2 **************
    let (robot_tx, computer_rx) = channel::<isize>(BUFFER_SIZE);
    let (computer_tx, robot_rx) = channel::<isize>(BUFFER_SIZE);
    // Hacks required for Part 2
    let mut hacked_program = prog.clone();
    hacked_program[0] = 2;
    let computer = intcode_run(hacked_program, computer_rx, computer_tx);
    let robot = robot_run_part2(robot_rx, robot_tx);
    let (_computer_return,robot_response_part2) = join!(computer, robot); // , computer_snooper.monitor(), robot_snooper.monitor()

    Ok((robot_response_part1.unwrap(), robot_response_part2.unwrap()))
}
async fn robot_run_part1(rx: Receiver<isize>, tx: Sender<isize>) -> Result<isize,Error> {
    let mut robot = Robot::new(rx, tx);
    robot.download_camera_view().await?;
    robot.camera_view.redraw_screen()?;
    let intersections = robot.find_intersections()?;
    let sum_of_alignment_params = intersections.iter()
        .fold(0,|sum, (y,x)| {
            sum + *y**x
        });
    // println!("\nIntersections: {:?}", intersections);
    Ok(sum_of_alignment_params)
}
async fn robot_run_part2(rx: Receiver<isize>, tx: Sender<isize>) -> Result<isize,Error> {
    let mut robot = Robot::new(rx, tx);
    robot.download_camera_view().await?;
    // robot.camera_view.redraw_screen()?;
    let path_to_end = robot.find_path_to_end()?;
    let _path_to_end = path_to_end.split(",").map(|s|{s.chars()}).flatten().collect::<String>();
    let main = "A,C,C,A,C,B,A,B,C,B";
    let a = "L,4,L,6,L,8,L,12";
    let b = "R,12,L,6,L,6,L,8";
    let c = "L,8,R,12,L,12";

    let dust = robot.execute_path(main,a,b,c).await?;

    Ok(dust)
}
#[derive(Debug)]
enum Error {
    IllegalOpcode {code: isize},
    IllegalRobotResponse {val: isize},
    RobotComms {msg: String},
    ComputerComms {msg: String},
    MapAssertFail {msg: String},
    MapOriginWrong {msg: String},
    ImpossibleTurn {msg: String},
    CameraNotFound,
}
#[derive(Debug,Copy,Clone,Eq,PartialEq)]
enum MapData {
    Empty=46,        // '.'
    Scaffold=35,     // '#'
    Intersection=79, // 'O'
    Up=94,           // '^'
    Down=118,        // 'v'
    Left=60,         // '<'
    Right=62,        // '>'
    TumblingThroughSpace=88, // 'X'
    NewLine=10,      // Used for control flow (changing row #, marking end of output) only. Not saved in map.
}
// See https://jrgraphix.net/r/Unicode/2700-27BF for Dingbats in unicode
impl MapData {
    fn to_str(&self) -> &'static str {
        print("\x1B[56m"); // SIDE EFFECT - print white control chars (default color)
        match *self {
            Scaffold => "■",
            Empty => "•",
            Intersection => "+",
            Up => "^",
            Down =>"v",
            Left =>"<",
            Right =>">",
            TumblingThroughSpace =>"☻",
            NewLine => "Not a REAL Map Item",
        }
    }
}
impl TryFrom<isize> for MapData {
    type Error = Error;
    fn try_from(val: isize) -> Result<Self, Self::Error> {
        use MapData::*;
        let status = match val {
            n if n == Scaffold as isize => Scaffold,
            n if n == Empty as isize => Empty,
            n if n == Intersection as isize => Intersection,
            n if n == Up as isize => Up,
            n if n == Down as isize => Down,
            n if n == Left as isize => Left,
            n if n == Right as isize => Right,
            n if n == TumblingThroughSpace as isize => TumblingThroughSpace,
            n if n == NewLine as isize => NewLine,
            _ => return Err(Error::IllegalRobotResponse { val }),
        };
        Ok(status)
    }
}
#[derive(Debug,Copy,Clone,Eq,PartialEq)]
enum RobotMovement {
    North=1,
    East=2,
    South=3,
    West=4,
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
            _ => return Err(Error::IllegalRobotResponse { val }),
        };
        Ok(status)
    }
}
impl RobotMovement {
    fn move_from(&self, loc: Location) -> Location {
        match self {
            North => (loc.0-1, loc.1),
            South => (loc.0+1, loc.1),
            West => (loc.0, loc.1-1),
            East => (loc.0, loc.1+1),
        }
    }
    fn turn_required(&self, new_dir: RobotMovement) -> Result<char,Error> {
        let this = *self as isize;
        let next = new_dir as isize;
        match next-this {
            1|-3 => Ok('R'),
            -1|3 => Ok('L'),
            _ => Err(ImpossibleTurn {msg: format!("Can't turn from {:?} to {:?} !",this,new_dir)}),
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
        let origin = (-5,-5);
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
    rx: Receiver<isize>,
    tx: Sender<isize>,
}
impl Robot {
    fn new(rx: Receiver<isize>, tx: Sender<isize>) -> Self {
        let camera_view = WorldMap::new();
        Robot { camera_view, rx, tx: tx }
    }
    fn find_robot(&self) -> Result<Location,Error> {
        match self.camera_view.data.iter().fold(None,|cam_loc, ((y,x), item)| {
            match item {
                Up|Down|Left|Right => Some((*y,*x)),
                _ => cam_loc,
            }
        }) {
            Some(cam_loc) => Ok(cam_loc),
            None => Err( CameraNotFound ),
        }
    }
    fn surrounding_space_is_scaffold(&self, loc: Location) -> Result<(bool,bool,bool,bool),Error> {
        Ok((
            self.camera_view.data.get(&North.move_from(loc)) == Some(&Scaffold),
            self.camera_view.data.get(&South.move_from(loc)) == Some(&Scaffold),
            self.camera_view.data.get(&West.move_from(loc)) == Some(&Scaffold),
            self.camera_view.data.get(&East.move_from(loc)) == Some(&Scaffold),
        ))
    }
    fn find_path_to_end(&self) -> Result<String,Error> {
        let robot_loc = self.find_robot()?;
        let starting_direction = match self.camera_view.data.get(&robot_loc) {
            Some(Up) => North,
            Some(Down) => South,
            Some(Left) => West,
            Some(Right) => East,
            _ => return Err(MapAssertFail {msg: format!("Robot not found at robot location {:?}!",robot_loc)})
        };
        let mut list_of_move_commands = String::new();
        let mut this_location = robot_loc;
        let mut move_direction = starting_direction;
        let mut move_cnt_since_last_turn = 0;
        loop {
            match self.surrounding_space_is_scaffold(this_location)? {
                (true,true,true,true) => {
                    // Ignore intersection -- just passing through
                },
                (n,s,w,e) => {
                    // remove direction we just came from
                    let dir_to_move = match move_direction.reverse() {
                        North => (false,s,w,e),
                        South => (n,false,w,e),
                        West => (n,s,false,e),
                        East => (n,s,w,false),
                    };
                    // should be only one direction remaining -- the direction to move in
                    let next_move_direction = match dir_to_move {
                        ( true, false, false, false) => North,
                        (false,  true, false, false) => South,
                        (false, false,  true, false) => West,
                        (false, false, false,  true) => East,
                        (false, false, false, false) => {
                            // Found end of path! Report length of final stretch
                            list_of_move_commands.push(',');
                            list_of_move_commands.push_str(&move_cnt_since_last_turn.to_string());                            
                            break
                        }, 
                        _ => return Err(MapAssertFail {msg: format!("Can't follow scaffolding path at {:?}", this_location)}),
                    };
                    // Do we need to turn?
                    if next_move_direction != move_direction {
                        if move_cnt_since_last_turn > 0 {
                            list_of_move_commands.push(',');
                            list_of_move_commands.push_str(&move_cnt_since_last_turn.to_string());
                            list_of_move_commands.push(',');
                        }
                        list_of_move_commands.push(move_direction.turn_required(next_move_direction)?);
                        move_direction = next_move_direction;
                        move_cnt_since_last_turn = 0;
                    }
                },
            }
            this_location = move_direction.move_from(this_location);
            move_cnt_since_last_turn += 1;
        }
        Ok(list_of_move_commands)
    }
    fn find_intersections(&self) -> Result<HashSet<Location>,Error> {
        let robot_loc = self.find_robot()?;
        let starting_direction = match self.surrounding_space_is_scaffold(robot_loc)? {
            ( true, false, false, false) => North,
            (false,  true, false, false) => South,
            (false, false,  true, false) => West,
            (false, false, false,  true) => East,
            _ => return Err(MapAssertFail {msg: "Robot not on start of Scaffold!".to_string()})
        };
        let mut intersections = HashSet::new();
        let mut this_location = robot_loc;
        let mut search_direction = starting_direction;
        loop {
            match self.surrounding_space_is_scaffold(this_location)? {
                (true,true,true,true) => {
                    // Intersection FOUND
                    intersections.insert(this_location);
                    this_location = search_direction.move_from(this_location);
                }
                (n,s,w,e) => {
                    // remove direction we just came from
                    let dir_to_move = match search_direction.reverse() {
                        North => (false,s,w,e),
                        South => (n,false,w,e),
                        West => (n,s,false,e),
                        East => (n,s,w,false),
                    };
                    search_direction = match dir_to_move {
                        ( true, false, false, false) => North,
                        (false,  true, false, false) => South,
                        (false, false,  true, false) => West,
                        (false, false, false,  true) => East,
                        (false, false, false, false) => break, // Found end of path!
                        _ => return Err(MapAssertFail {msg: format!("Can't follow scaffolding path at {:?}", this_location)}),
                    };
                    this_location = search_direction.move_from(this_location);
                }
            }
        }
        Ok(intersections)
    }
    async fn execute_path(&mut self, main: &str, a: &str, b: &str, c: &str) -> Result<isize,Error> {
        for sub_program in &[main, a, b, c] {
            println!("Sending sub-program");
            let ascii_stream: Vec<_> = sub_program.chars().map(|ch|ch as isize).chain((10..).take(1)).collect();
            for i in ascii_stream {
                // Send a program data to Robot's Intcode Computer
                if let Err(_) = self.tx.send(i).await {
                    return Err(Error::RobotComms { msg:format!("Robot output channel failure.  The following data is being discarded:\n   {:?}", i) });
                }
            }
            loop {
                // Fetch confirmation
                let arbitrary = match self.rx.next().await {
                    Some(ans) => ans,
                    None => return Err(RobotComms {msg: "Incode computer stopped transmitting.".to_string()}),
                };
                if arbitrary == 10 {
                    break;
                } else {
                    println!("Got arb response: {}", arbitrary);
                }
            }
                // Fetch confirmation
            let arbitrary = match self.rx.next().await {
                Some(ans) => ans,
                None => return Err(RobotComms {msg: "Incode computer stopped transmitting.".to_string()}),
            };
            println!("Got arb response: {}", arbitrary);
        }
        // Send a y/n answer to Robot's Intcode Computer
        if let Err(_) = self.tx.send('n' as isize).await {
            return Err(Error::RobotComms { msg:format!("Robot output channel failure.  The following data is being discarded:\n   {:?}", 'n') });
        }
        println!("Waiting on dust response");
        // Fetch the response of total dust collected...
        let dust_collected = match self.rx.next().await {
            Some(ans) => ans,
            None => return Err(RobotComms {msg: "Incode computer stopped transmitting.".to_string()}),
        };
        Ok(dust_collected)
    }
    async fn download_camera_view(&mut self) -> Result<(),Error> {
        // Slow things down for debug or visualization
        // ESPECIALLY at start
        let delay = Duration::from_millis(0);
        std::thread::sleep(delay);
        let mut next_coord = (0,0);
        loop {
            // Fetch the next image datapoint
            let next_item = match self.rx.next().await {
                Some(item) => MapData::try_from(item)?,
                None => return Err(RobotComms {msg: "Incode computer stopped transmitting.".to_string()}),
            };
            if next_item == NewLine {
                if next_coord.1 == 0 {break;} // \n\n signifies the end of map image
                next_coord = (next_coord.0+1,0);
            } else {
                self.camera_view.data.insert(next_coord, next_item);
                next_coord = (next_coord.0, next_coord.1+1);
            };
        }
        // Validate response
        let row_endpoints: Vec<_> = self.camera_view.data.iter().map(|((y,x),_)|{(y,x)})
            .skip(1).zip(self.camera_view.data.iter().map(|((y,x),_)|{(y,x)}))
            .filter_map(|((y,_),(old_y,old_x))| {
                if *y == *old_y + 1 {Some((*old_y,*old_x))} else {None}
            }).collect();
        println!("Row Endpoints: {:?}", row_endpoints);
        if row_endpoints.iter().map(|(y,_)|{y})
            .skip(1).zip(row_endpoints.iter().map(|(y,_)|{y}))
            .fold(false,|b,(y,old_y)|{b || *y != *old_y + 1}) {
                Err(MapAssertFail {msg: "Bad Map: Rows not contiguous!".to_string()})
            } else if row_endpoints.iter().map(|(_,x)|{x})
                .skip(1).zip(row_endpoints.iter().map(|(_,x)|{x})).fold(false, |b,(x,old_x)| {b || x != old_x}) {
                Err(MapAssertFail {msg: "Bad Map: Rows are not the same length!".to_string()})
            } else {
                Ok(())
            }
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
