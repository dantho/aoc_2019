/// https://adventofcode.com/2019/day/15
const ESC_CLS: &'static str = "\x1B[2J";
// const ESC_CURSOR_ON: &'static str = "\x1B[?25h";
const ESC_CURSOR_OFF: &'static str = "\x1B[?25l";
const DBG: bool = false;

use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, Write, stdout};
use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;
use std::collections::BTreeMap;
use ExplorerMovement::*;
use MapData::*;
use Error::*;

type Location = (usize,usize);

fn main() -> Result<(),Error> {
    let filename = "ex1.txt";
    initiate_search(filename)?;
    Ok(())
}
fn initiate_search(filename: &'static str) -> Result<usize,Error> {
    let mut room_map = WorldMap::new(filename)?;
    room_map.redraw_screen()?;
    let entrance_loc = room_map.find_entrance()?;
    room_map.clear_location(entrance_loc)?; // Remove the Entrance.  We don't need it anymore.
    let part1 = pick_up_all_keys(room_map.clone(), entrance_loc)?;
    let part2 = 0;
    println!("");
    println!("Part 1: Fewest steps to pickup all the keys is {}", part1 );
    println!("Part 2: TBD is {}", part2 );
    Ok(part1)
}
#[derive(Debug)]
enum Error {
    IllegalMapData {ch: char},
    MapAssertFail {msg: String},
    KeyNotFoundAt {loc: Location},
    BadKeyName {name: char},
    UnlockFail {msg: String},
    ItemNotFound {msg: String},
}
// Return value is step count required to find all FURTHER keys remaining on this copy of the map,
// or a MAX value to indicate failure (only locked doors found)
fn pick_up_all_keys(mut my_own_map: WorldMap, starting_loc: Location) -> Result<usize, Error> {
    // Algo: 
    // 1) Find (min distance to all) Keys and Doors for which we presently have keys
    //    (treat doors we don't have the key for as Walls -- ignore them)
    //    1.1) If None found, we're done! TERMINATE RECURSION and return total steps to date
    // 2) For each of the above: 
    //    2.1) Jump to target location (accumulate total steps)
    //    2.2) Pick up key or unlock door
    //    2.3) Recurse by cloning current map and calling self
    let keys = find_accessible_keys(&mut my_own_map, starting_loc, 0);
    my_own_map.clear_distances();
    if keys.len() == 0 {
        if DBG {println!("No Keys found!");}
        Ok(0) // END TERMINATION -- SUCCESS - ALL DONE
    } else {
        // // NOW HERE'S THE TRICKY PART!
        // // RECURSE INTO ALL POSSIBLE CHOICES FOR NEXT TARGET AND TAKE THE
        // // Lowest TOTAL step size which is able to remove the rest of the stuff
        // // ToDo: Got work to do on accumulating STEPS properly.
        // // ToDo: iterate through targets and recurse into (cloning map for each) and accumulate-in only the solution with the MIN TOTAL STEPS
        // // if DBG {println!("Recursing for Doors and Keys: {:?}", keys);}
        let result_step_count = keys.into_iter().fold(Ok(std::usize::MAX), |result_min, (loc, dist)| {
            let mut multiverse_n = my_own_map.clone();
            match result_min {
                // pickup key (and unlock door) then continue search by recursing
                Ok(min) => {
                    match multiverse_n.data.get(&loc) {
                        Some(Key(k)) => {
                            let key_name = *k;
                            let d = k.to_ascii_uppercase();
                            if DBG {print!("Picking up Key({})", k);}
                            multiverse_n.pick_up_key(loc).ok().unwrap();
                            if DBG {println!(" and unlocking Door({}) {}", d, dist);}
                            multiverse_n.unlock_door(key_name).ok().unwrap();
                        },
                        _ => panic!("Something other than a key found in keys!"),
                    };
                    // Recurse from this location
                    // Cloning map 'cause we're choosing among alternate (recursive) universes
                    // and discarding the rest like trash.
                    match pick_up_all_keys(multiverse_n, loc) {
                        Ok(v) => if v+dist < min {Ok(v+dist)} else {Ok(min)},
                        e => e,
                    }
                }
                // manually pass error along the fold closure path
                e => e,
            }
        });
        if DBG {println!("Min steps to clear remaining: {:?}", result_step_count );}
        result_step_count
    }
}
fn find_accessible_keys(shared_map: &mut WorldMap, present_loc: Location, present_distance: usize) -> Vec<(Location, usize)> {
    let mut keys = Vec::new();
    match shared_map.data.get_mut(&present_loc) {
        Some(Empty(d)) if *d > present_distance => {
            *d = present_distance; // label present location with distance marker -- critical step!
            // recurse in cardinal directions here using present_distance + 1
            keys.append(&mut find_accessible_keys(shared_map, (present_loc.0+1,present_loc.1), present_distance+1));
            keys.append(&mut find_accessible_keys(shared_map, (present_loc.0-1,present_loc.1), present_distance+1));
            keys.append(&mut find_accessible_keys(shared_map, (present_loc.0,present_loc.1+1), present_distance+1));
            keys.append(&mut find_accessible_keys(shared_map, (present_loc.0,present_loc.1-1), present_distance+1));
        },
        Some(Key(_)) => {
            keys.push((present_loc, present_distance)); // FOUND a target.  END RECURSION
        },
        Some(Empty(_)) => (), // *d <= present_distance, so been here, done that. END RECURSION
        Some(Door(_))|Some(Wall) => (), // Hit a locked door or wall.  END RECURSION
        Some(Entrance) => panic!("This algorithm requires the Entrance be 'cleared'."),
        None => panic!{"We stumbled off the map, somehow!"},
    }
    keys
}
// fn droid_run() -> Result<(usize,usize),Error> {
//     map_distance(&mut distance_map, (0, 0), 0)?;

//     let lower_right_corner = droid.known_map.lower_right_corner_on_screen();
//     set_cursor_loc(lower_right_corner.0+1,0);
//     let distance_to_oxygen_sensor: usize = match distance_map.get(&droid.oxygen_location_if_known.unwrap()) {
//         Some(d) => *d,
//         None => return Err(Error::MapAssertFail {msg: format!("Can't find oxygen sensor at {:?} !", droid.oxygen_location_if_known.unwrap())}),
//     };

//     // reset distance map
//     for (_,dist) in &mut distance_map {
//         *dist = std::usize::MAX;
//     }

//     map_distance(&mut distance_map, droid.oxygen_location_if_known.unwrap(), 0)?;
//     let minutes_to_fill_with_oxygen = distance_map.iter().fold(0,|most_minutes, ((_,_), minutes)| {
//         if *minutes > most_minutes {*minutes} else {most_minutes}
//     });
//     Ok((distance_to_oxygen_sensor, minutes_to_fill_with_oxygen))
// }
fn map_distance(map: &mut BTreeMap<Location, usize>, loc: Location, distance: usize) -> Result<(),Error> {
    let this_loc = match map.get_mut(&loc) {
        Some(dist) => dist,
        None => return Ok(()), // END RECURSION (Wall or unknown found)
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
#[derive(Debug,Copy,Clone,Eq,PartialEq)]
enum MapData {
    Empty(usize),
    Wall,
    Entrance,
    Door(char),
    Key(char),
}
// See https://jrgraphix.net/r/Unicode/2700-27BF for Dingbats in unicode
impl MapData {
    fn to_char(&self) -> char {
        match *self {
            Empty(_) => '.',
            Wall => '#',
            Entrance => '@',
            Door(d) => d,
            Key(k) => k,
        }
    }
    fn to_string(&self) -> String {
        self.to_char().to_string()
    }
}
impl TryFrom<char> for MapData {
    type Error = Error;
    fn try_from(ch: char) -> Result<Self, Self::Error> {
        use MapData::*;
        let status = match ch {
            en if en == Entrance.to_char() => Entrance,
            mt if mt == Empty(0).to_char() => Empty(std::usize::MAX),
            w if w == Wall.to_char() => Wall,
            d if d.is_alphabetic() && d.is_uppercase() => Door(d),
            k if k.is_alphabetic() && k.is_lowercase() => Key(k),
            _ => return Err(Error::IllegalMapData { ch }),
        };
        Ok(status)
    }
}
#[derive(Debug,Copy,Clone,Eq,PartialEq)]
enum ExplorerMovement {
    North=1,
    South=2,
    West=3,
    East=4,
}
impl ExplorerMovement {
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
#[derive(Clone)]
struct WorldMap {
    data: BTreeMap<Location, MapData>,
}
impl WorldMap {
    fn clear_distances(&mut self) {
        for item in &mut self.data {
            if let (_,Empty(dist)) = item {
                *dist = std::usize::MAX;
            }
        }
    }
    fn pick_up_key(&mut self, loc: Location) -> Result<char,Error> {
        let key_found = match self.data.get_mut(&loc) {
            Some(Key(k)) => *k,
            _ => return Err(KeyNotFoundAt {loc: loc}),
        };
        self.clear_location(loc);
        Ok(key_found)
    }
    fn unlock_door(&mut self, key_name: char) -> Result<(),Error> {
        let door_name = if key_name.is_lowercase() {
            key_name.to_ascii_uppercase()
        } else {
            return Err(BadKeyName {name: key_name})
        };
        if let Some(door_loc) = self.find_door(door_name)? {
            match self.data.get_mut(&door_loc) {
                Some(Door(d)) if *d == door_name => (),
                Some(Door(d)) => return Err(UnlockFail {msg: format!("Can't unlock door '{}' using key '{}'.", d, key_name)}),
                _ => return Err(ItemNotFound {msg: format!("Door '{}' not found at {:?}", door_name, door_loc)}),
            }
            self.clear_location(door_loc)?;
        }
        Ok(())
    }
    fn clear_location(&mut self, loc: Location) -> Result<(),Error> {
        let target = match self.data.get_mut(&loc) {
            Some(item) => item,
            None => return Err(MapAssertFail {msg: format!("Can't clear what's not there! {:?}",loc)}),
        };
        match target {
            Entrance|Door(_)|Key(_) => *target = Empty(std::usize::MAX), // CLEARED AN ITEM OFF MAP
            Wall => return Err(MapAssertFail {msg: format!("Can't clear a Wall at {:?} !",loc)}),
            Empty(_) => return Err(MapAssertFail {msg: format!("Already Empty at {:?} !",loc)}),
        };
        Ok(())
    }
    fn find_door(&self, door_name: char) -> Result<Option<Location>,Error> {
        let door = self.data.iter().fold(None,|maybe_found, (loc,item)| {
            if *item == Door(door_name) {Some(*loc)} else {maybe_found}
        });
        Ok(door)
    }
    fn find_entrance(&self) -> Result<Location,Error> {
        let entrance = self.data.iter().fold(None,|found_e, (loc,item)| {
            if *item == Entrance {Some(loc)} else {found_e}
        });
        let entrance = match entrance {
            Some(e) => *e,
            None => return Err(ItemNotFound {msg: format!("{:?}", Entrance)}),
        };
        Ok(entrance)
    }
    fn new(filename: &'static str) -> Result<Self,Error> {
        let data = WorldMap::read_initial_map(filename)?;
        Ok(WorldMap {data})
    }
    fn read_initial_map(filename: &'static str) -> Result<BTreeMap<Location,MapData>,Error> {
        let mut new_world = BTreeMap::new();
        let fd = File::open(filename).expect(&format!("Failure opening {}", filename));
        let buf = BufReader::new(fd);
        buf.lines().enumerate().for_each(|(y, line)| {
            line.unwrap().chars().enumerate().for_each(|(x,ch)| {
                let map_item: MapData = match ch.try_into() {
                    Ok(map_data) => map_data,
                    Err(e) => panic!(format!("Error inside closure: '{:?}'", e)),
                };
                if let Some(_) = new_world.insert((y,x), map_item) {
                    assert!(false, "Overwritting data while reading.  Not locsible given code design.");
                };
            });
        });
        Ok(new_world)
    }
    fn is_known(&self, loc: &Location) -> bool {
        self.data.contains_key(loc)
    }
    fn modify_data(&mut self, location: Location, data: MapData) -> Result<(),Error> {
        match self.data.get_mut(&location) {
            None => {
                self.data.insert(location, data);
            },
            Some(&mut Wall) => {
                if data != Wall {
                    return Err(Error::MapAssertFail {msg: format!("Placing {:?} on Wall at {:?}", data, location)});
                }
            },
            Some(map_data_here) => {
                *map_data_here = data;
            }
        }
        self.draw_location(location)?;
        Ok(())
    }
    fn draw_location(&self, loc: Location) -> Result<(),Error> {
        set_cursor_loc(loc.0, loc.1);
        let map_item = match self.data.get(&loc) {
            None => ' ',
            Some(data) => data.to_char(),
        };
        print_char(map_item);
        Ok(())
    }
    fn redraw_screen(&self) -> Result<(),Error> {
        print(ESC_CLS); // clear screen, reset cursor
        print(ESC_CURSOR_OFF); // Turn OFF cursor
        // print(ESC_CURSOR_ON); // Turn ON cursor
        for (loc, _) in &self.data {
            self.draw_location(*loc)?;
        }
        println!("");
        Ok(())
    }
    fn lower_right_corner(&self) -> Location {
        self.data.iter().fold((std::usize::MIN,std::usize::MIN),|(max_y,max_x), ((y,x),_)| {
            (
                if *y > max_y {*y} else {max_y},
                if *x > max_x {*x} else {max_x}
            )
        })
    }
}
fn print(s: &str) {
    print!("{}",s);
    stdout().flush().unwrap();
}
fn print_char(ch: char) {
    print!("{}",ch);
    stdout().flush().unwrap();
}
fn set_cursor_loc(y:usize,x:usize) {
    print!("\x1B[{};{}H", y+1, x+1);
    stdout().flush().unwrap();
}
fn set_color(color:u8) {
    print(
        &format!("\x1B[{}m", 41 + color)
    );
}
// struct Explorer<'a> {
//     known_map: &'a mut WorldMap,
//     explorer_location: Location,
// }
// impl Explorer<'a> {
//     fn new() -> Self {
//         let mut known_map = WorldMap::new();
//         let explorer_location: Location = (0,0);
//         let oxygen_location_if_known: Option<Location> = None;  // Unknown as yet
//         known_map.data.insert(explorer_location, MapData::Explorer);
//         Explorer { known_map, explorer_location, oxygen_location_if_known}
//     }
//     // explore() is a recursive algorithm (4-way) to visit all UNVISITED squares to determine the contents.
//     // A previously visited square of any kind (preemptively) ENDS the (leg of the 4-way) recursion.
//     fn explore<'a>(&'a mut self) -> Result<(),Error> {
//         // Explore cardinal directions, returning to center each time
//         for dir in &[North, South, West, East] {
//             let move_dir = *dir;
//             if !self.known_map.is_known(&move_dir.move_from(self.explorer_location)) {
//                 if self.move_droid(move_dir)? {
//                     // Then explore there
//                     self.explore()?;
//                     // and move back to continue more local exploration
//                     self.move_droid(move_dir.reverse())?;
//                 }
//             } 
//         }
//         Ok(())
//     }
//     fn move_droid(&mut self, move_dir: ExplorerMovement) -> Result<bool,Error> {
//         let move_succeeded: bool;
//         // Slow things down for debug or visualization
//         // ESPECIALLY at start
//         let delay = Duration::from_millis(0);
//         std::thread::sleep(delay);
//         // Send a movement command to Explorer's Intcode Computer
//         // self.tx.send(move_dir as isize)
//         // And fetch a response
//         let st = '#' as isize;
//         let status = ExplorerStatus::try_from(st)?;
//         // Interpret response
//         match status {
//             HitWall => {
//                 move_succeeded = false;
//                 let wall_location = move_dir.move_from(self.explorer_location);
//                 self.known_map.modify_data(wall_location, Wall)?;
//             },
//             Moved => {
//                 move_succeeded = true;
//                 // clear up old droid location
//                 self.known_map.modify_data(self.explorer_location, Empty)?; // Empty unless...
//                 if let Some(ox) = self.oxygen_location_if_known {
//                     if ox == self.explorer_location {
//                         self.known_map.modify_data(self.explorer_location, OxygenSystem)?;
//                     }
//                 }
//                 // move droid
//                 self.explorer_location = move_dir.move_from(self.explorer_location);
//                 self.known_map.modify_data(self.explorer_location, Explorer)?;
//             },
//             OxygenSystemDetected => {
//                 move_succeeded = true;
//                 // clear up old droid location
//                 self.known_map.modify_data(self.explorer_location, Empty)?; // definitely Empty
//                 // move droid
//                 self.explorer_location = move_dir.move_from(self.explorer_location);
//                 self.known_map.modify_data(self.explorer_location, Explorer)?; // Or Explorer_Oxygen combo?
//                 // and udate crucial information
//                 self.oxygen_location_if_known = Some(self.explorer_location);
//             },
//         }
//         Ok(move_succeeded)
//     }
// }

#[test]
fn test_ex1() -> Result<(),Error> {
    assert_eq!(initiate_search("ex1.txt")?, 8);
    Ok(())
}
#[test]
fn test_ex2() -> Result<(),Error> {
    assert_eq!(initiate_search("ex2.txt")?, 86);
    Ok(())
}
#[test]
fn test_ex3() -> Result<(),Error> {
    assert_eq!(initiate_search("ex3.txt")?, 132);
    Ok(())
}
#[test]
fn test_ex4() -> Result<(),Error> {
    assert_eq!(initiate_search("ex4.txt")?, 136);
    Ok(())
}
#[test]
fn test_ex5() -> Result<(),Error> {
    assert_eq!(initiate_search("ex5.txt")?, 81);
    Ok(())
}