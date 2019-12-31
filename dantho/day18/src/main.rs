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
    let filename = "Ex2.txt";
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
    let key_index = room_map.data.iter().filter_map(|(loc, item)| {
        match item {
            Key(k,_) => Some((*k,*loc)),
            _ => None,
        }
    }).collect::<BTreeMap<_,_>>();
    let paths: Vec<String> = find_paths(room_map.clone(), entrance_loc)?;
    let min_step_count = paths.iter().fold(None,|min_so_far, path| {
        let steps = match count_steps(path.to_string(), room_map.clone(), &key_index, entrance_loc) {
            Ok(s) => s,
            Err(_) => panic!("Stupid Result Err() in closure."),
        };
        match min_so_far {
            None => Some(steps),
            Some(min) => if steps < min {
                println!("New min path found: {}", steps);
                Some(steps)
            } else {
                Some(min)
            },
        }
    });
    let min_step_count = min_step_count.unwrap();
    // let part1 = pick_up_all_keys(room_map.clone(), entrance_loc)?;
    // let part2 = 0;
    // println!("");
    // println!("Part 1: Fewest steps to pickup all the keys is {}", part1 );
    // println!("Part 2: TBD is {}", part2 );
    Ok(min_step_count)
}
fn count_steps(mut walk_path: String, mut my_own_map: WorldMap, key_index: &BTreeMap<char, (usize, usize)>, current_loc: Location) -> Result<usize,Error> {
    my_own_map.clear_distances();
    map_distance(&mut my_own_map, current_loc, 0)?;
    match walk_path.pop() {
        Some(key_name) => {
            let key_loc = *key_index.get(&key_name).unwrap();
            let steps_to_key = match my_own_map.data.get(&key_loc) {
                Some(Key(k,dist)) => {
                    assert_eq!(key_name,*k);
                    *dist
                },
                _ => return Err(MapAssertFail {msg: "Didn't find key.".to_string()}),
            };
            println!("Taking {} steps to {:?} ", steps_to_key, key_name);
            my_own_map.pick_up_key(key_loc)?;
            Ok(steps_to_key + count_steps(walk_path, my_own_map, key_index, key_loc)?)
        },
        None => Ok(0), // END Recursion, walk is complete, no more steps
    }
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
    // build dependency tree (bush?)
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
    // map to deep dependencies
    let deep_dependencies: BTreeMap<_,_> = key_dependencies.iter().map(|(k,_)| {
        (*k,dependency_dive(k, &key_dependencies))
    }).collect();
    println!("Deep Dependencies:");
    for pair in &deep_dependencies {
        println!("{:?}", pair);
    }
    // count deep dependency depth
    let deep_depth: BTreeMap<_,_> = key_dependencies.iter().map(|(k,_)| {
        (*k,dependency_depth(k, &key_dependencies))
    }).collect();
    println!("Dependency Depth:");
    for pair in &deep_depth {
        println!("{:?}", pair);
    }
    // concatenate deep dependancies and depth of same
    let deep_depth_deps = deep_depth.into_iter().zip(deep_dependencies.into_iter())
    .map(|((ch1,depth),(ch1prime,deps))| {
        assert_eq!(ch1,ch1prime);
        (ch1, (depth,deps))
    }).collect();
    // now make it rain
    let walk_paths = generate_all_walk_paths(&orig_paths_with_doors, &key_dependencies, &deep_depth_deps);
    println!("Walking paths:");
    for walk in &walk_paths {
        println!("> {:?}", walk);
    }
    Ok(walk_paths)
}
fn is_full_path(path: &str, deep_depth_deps: &BTreeMap<char, (usize, HashSet<char>)> ) -> bool {
    let mut at_least_one_key_was_found = false;
    for k in path.chars() {
        if is_key(k) {
            at_least_one_key_was_found = true;
            if let Some((_,v)) = deep_depth_deps.get(&k) {
                if v.len() > 0 {
                    return false;
                }
            }
        }
    }
    at_least_one_key_was_found
}
fn generate_all_walk_paths(paths: &HashSet<String>, deps: &BTreeMap<char, HashSet<char>>, deep_depth_deps: &BTreeMap<char, (usize, HashSet<char>)>) -> Vec<String> {
    // end recursion when all keys are gone from all paths
    if 0==paths.len() {return Vec::new()};
    // just double-checking for a set of empty paths (unnecessary?)
    if paths.iter().fold(true,|b,s| {b && 0==s.len()}) {return Vec::new()};
    // println!("Path count: {}, paths: {:?}", paths.len(), paths);
    // PLAN:
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
    let mut full_path_only = false;
    // be GREEDY about full paths, regardless of distance, to exclusion of all others
    for path in paths {
        if is_full_path(path,deep_depth_deps) {
            full_path_only = true;  // If any path is full, then exlude all partial paths and process ONLY full paths
            break;
        }
    }
    for path in paths {
        if full_path_only {
            if !is_full_path(path, deep_depth_deps) {continue};  // abort this path (for now) if not full
        }
        let mut keys_removed_on_this_path = Vec::new();
        for key_or_door in path.chars().rev() {
            if is_key(key_or_door) { // it's a key... without any door in the way... so let's remove it (aka "pick it up")
                keys_removed_on_this_path.push(key_or_door);
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
            let mut ddd = deep_depth_deps.clone();
            for key in &keys_removed_on_this_path {
                remove_key_from_paths(key, &mut reduced_paths);
                remove_key_from_ddd(key, &mut ddd)
            }
            let sub_paths = generate_all_walk_paths(&reduced_paths, &deps, &ddd);
            let keys_removed_string = keys_removed_on_this_path.into_iter().collect::<Vec<char>>().iter().rev().collect::<String>();
            if sub_paths.len() == 0 {
                walking_paths.push(keys_removed_string);
            } else {
                for sub_string in sub_paths {
                    walking_paths.push(format!("{}{}", sub_string, keys_removed_string));
                }
            }
            // println!("path_in_paths: {:?}, keys_removed: {:?}", path, keys_removed_on_this_path);
        }
    }
    walking_paths
}
fn remove_key_from_ddd(key: &char, deep_depth_deps: &mut BTreeMap<char, (usize, HashSet<char>)>) {
    deep_depth_deps.remove(key);
    for (_,deps) in deep_depth_deps.values_mut() {
        deps.remove(key);
    };
}
fn remove_key_from_paths(key: &char, paths: &mut HashSet<String>) {
    let mut modified = HashSet::new();
    for path in paths.iter() {
        let tmp = path.chars().filter_map(
            |ch| { if ch == key.to_ascii_lowercase() || ch == key.to_ascii_uppercase() {None} else {Some(ch)}
        }).collect::<String>();
        if tmp.len() > 0 {
            modified.insert(tmp);
        }
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
// fn map_distance_to_keys(shared_map: &mut WorldMap, loc: Location, distance: usize) -> Result<(),Error> {
//     let mut is_key = false;
//     let this_loc = match shared_map.data.get_mut(&loc) {
//         Some(Empty(dist)) => dist,
//         Some(Key(_,dist)) => {
//             is_key = true;
//             dist
//         },
//         _ => return Ok(()), // END RECURSION (Door, Wall or unknown found)
//     };
//     if *this_loc <= distance {
//         return Ok(()) // END RECURSION (crossed [equiv or superior] path)
//     }
//     *this_loc = distance; // Set present location
//     if is_key {return Ok(())} // END REcURSION (early 'cause key found here)
//     // Recurse into cardinal directions
//     map_distance_to_keys(shared_map, (loc.0-1,loc.1), distance+1)?; // North
//     map_distance_to_keys(shared_map, (loc.0+1,loc.1), distance+1)?; // South
//     map_distance_to_keys(shared_map, (loc.0,loc.1-1), distance+1)?; // West
//     map_distance_to_keys(shared_map, (loc.0,loc.1+1), distance+1)?; // East
//     Ok(())
// }
fn map_distance(shared_map: &mut WorldMap, loc: Location, distance: usize) -> Result<(),Error> {
    let this_loc = match shared_map.data.get_mut(&loc) {
        Some(Empty(dist)) => dist,
        Some(Key(_,dist)) => dist,
        Some(Door(_,dist)) => dist,
        _ => return Ok(()), // END RECURSION (Wall or unknown found)
    };
    if *this_loc <= distance {
        return Ok(()) // END RECURSION (crossed [equiv or superior] path)
    }
    *this_loc = distance; // Set present location
    // Recurse into cardinal directions
    map_distance(shared_map, (loc.0-1,loc.1), distance+1)?; // North
    map_distance(shared_map, (loc.0+1,loc.1), distance+1)?; // South
    map_distance(shared_map, (loc.0,loc.1-1), distance+1)?; // West
    map_distance(shared_map, (loc.0,loc.1+1), distance+1)?; // East
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
