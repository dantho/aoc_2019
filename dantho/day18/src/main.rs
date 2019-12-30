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
use std::collections::{BTreeMap,HashSet};
use ExplorerMovement::*;
use MapData::*;
use Error::*;

type Location = (usize,usize);

fn main() -> Result<(),Error> {
    let filename = "ex3.txt";
    initiate_search(filename)?;
    Ok(())
}
fn initiate_search(filename: &'static str) -> Result<usize,Error> {
    let mut room_map = WorldMap::new(filename)?;
    room_map.redraw_screen()?;
    let entrance_loc = room_map.find_entrance()?;
    room_map.clear_location(entrance_loc)?; // Remove the Entrance.  We don't need it anymore.
    room_map.clear_distances();
    let keynames: String = room_map.data.iter().filter_map(|(_,item)|{match item
    {
        Key(k,_) => Some(k),
        _ => None
    }}).collect();
    let doornames: String = room_map.data.iter().filter_map(|(_,item)|{match item
    {
        Door(d,_) => Some(d),
        _ => None
    }}).collect();
    println!("Keys: {}", keynames);
    println!("Doors: {}", doornames);
    let paths: Vec<String> = find_paths(room_map.clone(), entrance_loc)?;

    // let part1 = pick_up_all_keys(room_map.clone(), entrance_loc)?;
    // let part2 = 0;
    // println!("");
    // println!("Part 1: Fewest steps to pickup all the keys is {}", part1 );
    // println!("Part 2: TBD is {}", part2 );
    Ok(0)
}
#[derive(Debug)]
enum Error {
    IllegalMapData {ch: char},
    MapAssertFail {msg: String},
    BadKeyName {name: char},
    UnlockFail {msg: String},
    ItemNotFound {msg: String},
}
#[derive(PartialEq, Debug)]
struct SearchPath {
    items: Vec<(char,usize,Location)>,
}
impl ToString for SearchPath {
    fn to_string(&self) -> String {
            self.items.iter().map(|(ch,_,_)|{*ch}).collect::<String>()
    }
}
impl SearchPath {
    fn new(first_key: (char,usize, Location)) -> Self {
        let items = vec![first_key];
        SearchPath {items}
    }
}
// find_paths uses recursive pathfinding via dijkstra to identify all possible paths
// from entrance to all items on the map (Doors and Keys)
// The same items may appear at the beginning of two or more paths if the paths diverge after some distance
fn find_paths(mut my_own_map: WorldMap, starting_loc: Location) -> Result<Vec<String>, Error> {
    let paths = match find_all_items(&mut my_own_map, 0, starting_loc) {
        None => return Ok(Vec::new()),
        Some(vec_of_vecs) => vec_of_vecs,
    };
    println!("Paths:");
    for path in &paths {
        println!("{:?}", path);
    }
    let orig_paths_with_doors: HashSet<String> = paths.iter().map(|path|{
        path.to_string()
    }).collect();
    println!("Paths (as string) with doors:");
    for path in &orig_paths_with_doors {
        println!("{}", path);
    }
    //build dependency tree (bush?)
    let mut key_dependencies: BTreeMap<char,HashSet<char>> = BTreeMap::new();
    for path_str in &orig_paths_with_doors {
        let mut doors:Vec<char> = Vec::new();
        for dk in path_str.chars().rev() {
            match dk {
                door if is_door(door) => doors.push(to_key(door)),
                key if is_key(key) => {
                    match key_dependencies.get_mut(&key) {
                        Some(deps) => for d in &doors {deps.insert(*d);},
                        None => {key_dependencies.insert(key, doors.iter().cloned().collect());},
                    }
                },
                _ => panic!("Should be a key or a door.  No?"),
            }
            if is_door(dk) {
                doors.push(to_key(dk));
            }
        }
    }
    println!("Dependency Tree:");
    for k in &key_dependencies {
        println!("{:?}", k);
    }
    let deep_dependencies: BTreeMap<_,_> = key_dependencies.iter().map(|(k,_)| {
        (*k,dependency_dive(k, &key_dependencies))
    }).collect();
    println!("Deep Dependencies:");
    for pair in &deep_dependencies {
        println!("{:?}", pair);
    }
    let deep_depth: BTreeMap<_,_> = key_dependencies.iter().map(|(k,_)| {
        (*k,dependency_depth(k, &key_dependencies))
    }).collect();
    println!("Dependency Depth:");
    for pair in &deep_depth {
        println!("{:?}", pair);
    }

    let walk_paths = generate_all_walk_paths(&orig_paths_with_doors, &key_dependencies, &deep_dependencies, &deep_depth);
    println!("Walking paths:");
    for walk in &walk_paths {
        println!("> {:?}", walk);
    }
    Ok(walk_paths)
}
fn generate_all_walk_paths(paths: &HashSet<String>, deps: &BTreeMap<char, HashSet<char>>, deep_deps: &BTreeMap<char, HashSet<char>>, deep_depth: &BTreeMap<char, usize>) -> Vec<String> {
    println!("Path count: {}, paths: {:?}", paths.len(), paths);
    // end recursion when all keys are gone from all paths
    if 0==paths.len() {return Vec::new()};
    // just double-checking for a set of empty paths (unnecessary?)
    if paths.iter().fold(true,|b,s| {b && 0==s.len()}) {return Vec::new()};
    // for each path in paths,
    //      examine starting char(s) for one or more keys which are not locked behind doors
    //      then (for each path with keys to pick up)
    //          "pick them up" by cloning all paths and removing it/them from all paths.
    //             remove associated doors, too.
    //          recurse with the modified/reduced clone
    //          append each walking_path returned above to key(s) removed here,
    //          Add walking_path(s) to list of cummulative walking paths to be returned
    // return list of all walking paths
    let mut walking_paths: Vec<String> = Vec::new();
    for path in paths {
        let mut keys_removed_on_this_path = HashSet::new();
        for key_or_door in path.chars().rev() {
            if is_key(key_or_door) { // it's a key... without any door in the way... so let's remove it (aka "pick it up")
                let new_key = keys_removed_on_this_path.insert(key_or_door);
                assert!(new_key);
            } else {
                let this_doors_key = key_or_door.to_ascii_lowercase();
                if keys_removed_on_this_path.contains(&this_doors_key) {
                    continue; // recently unlocked/removed, let's find more keys
                } else {
                    break; // locked, we've reached our limit here, let's process previously removed keys
                } 
            };
        }
        if keys_removed_on_this_path.len() > 0 {
            let mut reduced_paths = paths.clone();
            for key in &keys_removed_on_this_path {
                remove_key2(key, &mut reduced_paths)
            }
            let sub_paths = generate_all_walk_paths(&reduced_paths, &deps, &deep_deps, &deep_depth);
            for sub_string in sub_paths {
                walking_paths.push(format!("{}{}", keys_removed_on_this_path.iter().collect::<String>(), sub_string));
            }
        }
    }
    walking_paths
}
// There's GOT to be an easier way!
fn last_char(path: &str) -> Option<char> {
    path.chars().rev().take(1).fold(None,|_,ch|{Some(ch)}) 
}
// fn combinatorial_paths(deps: &BTreeMap<char, HashSet<char>>, paths: &Vec<String>) -> Vec<String> {
//     // println!("deps.len(): {}", deps.len());
//     if 0 == paths.len() {  // end of recursion
//         paths.push(String::new());
//         return paths;
//     }
//     let mut mod_paths: = paths.clone();
//     for path in &mut mod_paths {
//             // skip paths that start with doors
//             if !path.chars().rev().take(1).collect::<Vec<char>>()[0].is_ascii_lowercase() {continue;} 
//             let mut deps_less_n = deps.clone();
//             let mut paths_less_n = paths.clone();
//             // Done:  Make a key2 version of remove_key that works with paths.
//             // Extend removal logic here to remove more than one key if path permits.
//             // Remove Doors from paths before sending them in here.
//             //    Or, maybe, consider door removal logic here... -- might make finding accessible trivial again.
//             // The point of all of the above is to perform the NO-BRAINER of
//             // scooping up all accessible keys in a path before turning around to make another move.
//             remove_key(&accessible, &mut deps_less_n);
//             remove_key2(&accessible, &mut paths_less_n);
//             for sub_path in combinatorial_paths(&deps_less_n, &paths_less_n) {
//                 let new_path = format!{"{}{}", accessible, sub_path};
//                 paths.push(new_path);
//             }
//     }
//     paths
// }
fn remove_key(key: &char, deps: &mut BTreeMap<char, HashSet<char>>) {
    deps.remove(key);
    for (_, others) in deps {
        others.remove(key);
    }
}
fn remove_key2(key: &char, paths: &mut HashSet<String>) {
    let mut modified = HashSet::new();
    for path in paths.iter() {
        let tmp = path.chars().filter_map(
            |ch| { if ch == key.to_ascii_lowercase() || ch == key.to_ascii_uppercase() {None} else {Some(ch)}
        }).collect::<String>();
        modified.insert(tmp);
    };
    *paths = modified;
}
fn dependency_dive(key: &char, deps: &BTreeMap<char, HashSet<char>>) -> HashSet<char> {
    let dep_cnt = deps.get(key).unwrap().len();
    if dep_cnt == 0 {return HashSet::new();} // End recursion
    let mut sub_deps = HashSet::new();
    for dep_key in deps.get(key).unwrap() {
        sub_deps.insert(*dep_key);
        for deeper_dep_key in dependency_dive(dep_key, deps) {
            sub_deps.insert(deeper_dep_key);
        }
    }
    sub_deps
}
fn dependency_depth(key: &char, deps: &BTreeMap<char, HashSet<char>>) -> usize {
    let dep_cnt = deps.get(key).unwrap().len();
    if dep_cnt > 0 {
        1 + deps.get(key).unwrap().iter().fold(0,|max_depth, dep| {
                let sub_depth = dependency_depth(dep, deps);
                if sub_depth > max_depth {sub_depth} else {max_depth}
            })
    } else {
        0 // Ends recursion
    }
}
fn is_door(ch: char) -> bool {
    ch.is_ascii_uppercase()
}
fn is_key(ch: char) -> bool {
    ch.is_ascii_lowercase()
}
fn to_key(ch: char) -> char {
    ch.to_ascii_lowercase()
}
fn find_all_items(shared_map: &mut WorldMap, present_distance: usize, present_loc: Location) -> Option<Vec<SearchPath>> {
    let mut key_found_here = false;
    let mut door_found_here = false;
    let mut item_name: Option<char> = None;
    match shared_map.data.get_mut(&present_loc) {
        Some(Wall) => {return None}, // Hit a wall.  END RECURSION
        Some(Empty(dist)) => {
            if *dist <= present_distance {return None;} // *d <= present_distance, so been here, done that. END RECURSION
            *dist = present_distance; // label present location with distance marker -- critical step! Continue searching
        },
        Some(Door(d,dist)) => {
            if *dist <= present_distance {return None;} // *d <= present_distance, so been here, done that. END RECURSION
            if DBG {println!("Found locked {:?} at {:?}", Door(*d,present_distance), present_loc );}
            item_name = Some(*d);
            *dist = present_distance; // label present location with distance marker -- critical step! Continue searching
            door_found_here = true; // FOUND a target.  Continue searching for more.
        }, // Hit a locked door.  END RECURSION
        Some(Key(_,dist)) if *dist <= present_distance => return None, // Found this key already.  END RECURSION
        Some(Key(k,dist)) => {
            if *dist <= present_distance {return None;} // *d <= present_distance, so been here, done that. END RECURSION
            if DBG {println!("Found {:?} at {:?}", Key(*k,present_distance), present_loc );}
            item_name = Some(*k);
            *dist = present_distance; // label present location with distance marker -- critical step! Continue searching
            key_found_here = true; // FOUND a target.  Continue searching for more.
        },
        Some(Entrance) => panic!("This algorithm requires the Entrance be 'cleared'."),
        None => panic!{"We stumbled off the map, somehow!"},
    }
    // recurse in cardinal directions here using present_distance + 1
    let mut multiple_paths = Vec::new();
    for dir in vec![North, South, East, West] {
        if let Some(mut vec_of_paths) = find_all_items(shared_map, present_distance+1, dir.move_from(present_loc)) {
             multiple_paths.append(&mut vec_of_paths);
        }
    }
    if key_found_here||door_found_here {
        let this_item = (item_name.unwrap(),present_distance,present_loc);
        match multiple_paths.len() {
            0 => {
                multiple_paths.push(SearchPath::new(this_item));
                if DBG {println!("Sole key on this path: {:?}", multiple_paths);}
                Some(multiple_paths)    
            },
            1 => {
                for path in &mut multiple_paths {
                    path.items.push(this_item);
                }
                if DBG {println!("Key added to single path: {:?}", multiple_paths);}
                Some(multiple_paths)    
            },
            _ => {
                for path in &mut multiple_paths {
                    path.items.push(this_item);
                }
                if DBG {println!("Key added to multiple_paths: {:?}", multiple_paths);}
                Some(multiple_paths)    
            },
        }    
    } else {
        match multiple_paths.len() {
            0 => None,
            _ => {
                Some(multiple_paths)
            },
        }    
    }
}
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
    Door(char,usize),
    Key(char,usize),
}
// See https://jrgraphix.net/r/Unicode/2700-27BF for Dingbats in unicode
impl MapData {
    fn to_char(&self) -> char {
        match *self {
            Empty(_) => '.',
            Wall => '#',
            Entrance => '@',
            Door(d,_) => d,
            Key(k,_) => k,
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
            d if d.is_alphabetic() && d.is_uppercase() => Door(d,std::usize::MAX),
            k if k.is_alphabetic() && k.is_lowercase() => Key(k,std::usize::MAX),
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
#[derive(Debug,Clone)]
struct WorldMap {
    data: BTreeMap<Location, MapData>,
}
impl WorldMap {
    fn clear_distances(&mut self) {
        for item in &mut self.data {
            if let (_,Empty(dist)) = item {
                *dist = std::usize::MAX;
            }
            if let (_,Key(_,dist)) = item {
                *dist = std::usize::MAX;
            }
        }
    }
    fn pick_up_key(&mut self, loc: Location) -> Result<char,Error> {
        let (key_found, key_distance) = match self.data.get_mut(&loc) {
            Some(Key(k,dist)) => (*k,*dist),
            _ => return Err(ItemNotFound {msg: format!("No Key at {:?}", loc)}),
        };
        self.unlock_door(key_found)?;
        self.clear_location(loc)?;
        if let Some(Empty(d)) = self.data.get_mut(&loc) {
            *d = key_distance;
        }
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
                Some(Door(d,_)) if *d == door_name => (),
                Some(Door(d,_)) => return Err(UnlockFail {msg: format!("Can't unlock door '{}' using key '{}'.", d, key_name)}),
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
            Entrance|Door(_,_)|Key(_,_) => *target = Empty(std::usize::MAX), // CLEARED AN ITEM OFF MAP
            Wall => return Err(MapAssertFail {msg: format!("Can't clear a Wall at {:?} !",loc)}),
            Empty(_) => return Err(MapAssertFail {msg: format!("Already Empty at {:?} !",loc)}),
        };
        if let Some(Empty(_)) = self.data.get(&loc) {
            ();
        } else {
            assert!(false, format!("This spot, {:?} should be clear now.", loc));
        }
        Ok(())
    }
    fn find_door(&self, door_name: char) -> Result<Option<Location>,Error> {
        let door = self.data.iter().fold(None,|maybe_found, (loc,item)| {
            if *item == Door(door_name,std::usize::MAX) {Some(*loc)} else {maybe_found}
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
#[test]
fn test_input() -> Result<(),Error> {
    assert_eq!(initiate_search("input.txt")?, 0);
    Ok(())
}
