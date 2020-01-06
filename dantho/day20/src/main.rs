/// https://adventofcode.com/2019/day/20
const ESC_CLS: &'static str = "\x1B[2J";
// const ESC_CURSOR_ON: &'static str = "\x1B[?25h";
const ESC_CURSOR_OFF: &'static str = "\x1B[?25l";
const DBG: bool = false;
const DBGG: bool = false; // More G's, more verbosity
const INFINITY: usize = std::usize::MAX/1_000_000_000_000_000_000 * 1_000_000_000_000_000_000 + 123_456; // BIG number! But also recognizable as "special"
const MAX_DISTANCE: usize = 20000;
const DEEPEST_LEVEL: usize = 50;
const STACK_SIZE: usize = 400 * 1024 * 1024;

use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, Write, stdout};
use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;
use std::collections::{BTreeMap, HashSet};
use CardinalDirection::*;
use MapData::*;
use Error::*;
use std::thread;

type Location = (usize,usize);

fn main() {
    // Spawn thread with explicit stack size
    let child = thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(run)
        .unwrap();

    // Wait for thread to join
    child.join().unwrap();
}

fn run() -> Result<(),Error> {
    let filename = "input.txt";
    let part2 = process_part2(filename)?;
    // println!("Part 1: Fewest steps from AA to ZZ is {}", if part1 == INFINITY {"INFINITY".to_string()} else {part1.to_string()});
    println!("Part 2: Fewest steps from AA to ZZ in recursive Donut Space is {}", if part2 == INFINITY {"INFINITY".to_string()} else {part2.to_string()});
    Ok(())
}

// How many steps does it take to get from the open tile marked AA to the open tile marked ZZ?
fn process_part2(filename: &'static str) -> Result<usize,Error> {
    let mut donut_map = DonutMap::new(filename)?;
    donut_map.redraw_screen()?;
    if DBG {
        for p in &donut_map.portals {
            match p {
                (_, p) => println!("{:?}", p),
            }
        }
        for t in &donut_map.transport {
            match t {
                (from, to) => {
                    let p_name = &donut_map.find_portal_by_loc(from).unwrap().name;
                    println!("{} at {:?} warps to {:?}", p_name, from, to );
                }
            }
        }
    };
    let aa_portal = donut_map.find_portals_by_name("AA").pop().unwrap();
    let starting_loc = aa_portal.location;
    let part2 = donut_map.shortest_path_to_end(starting_loc, 0, 3)?;
    Ok(part2)
}
#[derive(Debug)]
enum Error {
    IllegalMapData {ch: char},
    MapAssertFail {msg: String},
}
#[derive(Debug, Clone)]
struct Portal {
    name: String,
    location: Location,
    is_outer: bool,
    ejection_direction: CardinalDirection,
}
#[derive(Debug)]
struct DonutMap {
    portals: BTreeMap<(String,Location), Portal>,
    transport: BTreeMap<Location, Option<Location>>,
    maps: Vec<BTreeMap<Location,MapData>>,
    donut_range: (usize,usize,usize,usize),
    donut_hole_range: (usize,usize,usize,usize),
    stack_depth: usize,
}
impl DonutMap {
    fn new(filename: &'static str) -> Result<Self,Error> {
        let mut me = DonutMap::read_initial_map(filename)?;
        me.init_portals_from_map_data()?;
        // Validate DonutMap
        let (_,_,y,x) = me.donut_range;
        for corner in &[
            (0,0),(0,1),(1,0),(1,1),
            (0,x+1),(0,x+2),(1,x+1),(1,x+2),
            (y+1,0),(y+1,1),(y+2,0),(y+2,1),
            (y+1,x+1),(y+1,x+2),(y+2,x+1),(y+2,x+2),
        ] {
            match me.map_data(0).get(corner) {
                None => (),
                Some(unexpected) => return Err(MapAssertFail {msg: format!("Expected nothing at {:?}, found {:?}", *corner, unexpected)}),
            }
        }
        match me.map_data(0).get(&(2,2)) {
            Some(Wall) => (),
            _ => return Err(MapAssertFail {msg: format!("Expected corner at {:?}", (2,2))}),
        }
        match me.map_data(0).get(&(y-2,x-2)) {
            Some(Wall) => (),
            _ => return Err(MapAssertFail {msg: format!("Expected corner at {:?}", (y-2,x-2))}),
        }
        Ok(me)
    }
    fn clear_layer(&mut self, depth:usize) {
        for item in self.map_data_mut(depth).values_mut() {
            if let Empty(distance) = item {
                *distance = INFINITY;
            }
        }
    }
    fn map_data(&self, depth: usize) -> &BTreeMap<Location,MapData> {
        if depth >= self.maps.len() {panic!("Requested depth of {} is more layers lower than we've ever had.", depth)}
        &self.maps[depth]
    }
    fn map_data_mut(&mut self, depth: usize) -> &mut BTreeMap<Location,MapData> {
        if depth == self.maps.len() {
            self.maps.push(self.maps[depth-1].clone());
            self.clear_layer(depth);
        }
        else if depth > self.maps.len() {panic!("Requested depth of {} is two or more layers lower than we've ever been.", depth)}
        &mut self.maps[depth]
    }
    fn find_portal_by_loc(&self, loc: &Location) -> Option<&Portal> {
        for ((_,_),p) in &self.portals {
            if p.location == *loc {return Some(p)}
        }
        None
    }
    fn find_portals_by_name(&self, name: &str) -> Vec<&Portal> {
        self.portals.iter().filter_map(|((_,_),p)|{
            if p.name == name {Some(p)} else {None}
        }).collect()
    }
    fn draw_location(&self, loc: Location) -> Result<(),Error> {
        set_cursor_loc(loc.0, loc.1);
        let map_item = match self.map_data(0).get(&loc) {
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
        let mut prev_column = 0;
        for (loc, _) in &self.maps[0] {
            if loc.1 < prev_column {println!("");}
            prev_column = loc.1;
            self.draw_location(*loc)?;
        }
        println!("");
        Ok(())
    }
    fn read_initial_map(filename: &'static str) -> Result<DonutMap,Error> {
        let mut map_data = BTreeMap::new();
        let fd = File::open(filename).expect(&format!("Failure opening {}", filename));
        let buf = BufReader::new(fd);
        buf.lines().enumerate().for_each(|(y, line)| {
            line.unwrap().chars().enumerate().for_each(|(x,ch)| {
                if ch != ' ' {
                    let map_item: MapData = match ch.try_into() {
                        Ok(map_data) => map_data,
                        Err(e) => panic!(format!("Error inside closure: '{:?}'", e)),
                    };
                    if let Some(_) = map_data.insert((y,x), map_item) {
                        assert!(false, "Overwritting data while reading.  Not possible given code design.");
                    };
                }
            });
        });
        let top_left = (0,0);
        let bottom_right = map_data.iter().fold((std::usize::MIN,std::usize::MIN),|(max_y,max_x), ((y,x),_)| {
            (if *y > max_y {*y} else {max_y},
             if *x > max_x {*x} else {max_x})
        });
        // top, left, bottom, right
        let donut_range = (top_left.0+2, top_left.1+2, bottom_right.0-2, bottom_right.1-2);
        let center = ((donut_range.2-donut_range.0)/2+donut_range.0, (donut_range.3-donut_range.1)/2+donut_range.1);
        if DBG {println!("center: {:?}", center)};
        let _donut_hole_range = (6,6,14,14);
        let donut_hole_range = DonutMap::find_donut_hole_range(&map_data, center);
        let me = DonutMap {
            maps: vec![map_data],
            portals: BTreeMap::new(),
            transport: BTreeMap::new(),
            donut_range,
            donut_hole_range, 
            stack_depth: 0,
        };
        Ok(me)
    }
    // Search in 4 cardinal directions for donut material (a Wall or an Empty(_))
    fn find_donut_hole_range(
        map_data: &BTreeMap::<Location,MapData>,
        a_location_in_hole: (usize,usize)
    ) -> (usize,usize,usize,usize) {
        // find all 4 boundaries of the whole
        let mut boundaries = Vec::new();
        for heading in &[North, West, South, East] {
            let mut stop_when_you_hit_donut = a_location_in_hole;
            let mut timeout = 10_000;
            loop {
                timeout -= 1;
                if timeout == 0 {panic!("find_donut_hole_range() produced infinite loop.");}
                if let Some(Empty(_))|Some(Wall) = map_data.get(&stop_when_you_hit_donut) {break;}
                stop_when_you_hit_donut = heading.move_from(&stop_when_you_hit_donut);
            }
            boundaries.push(stop_when_you_hit_donut);
            if DBG {println!("Stop when you hit donut: {:?}", stop_when_you_hit_donut)};
        }
        let point_range = (a_location_in_hole.0,a_location_in_hole.1,a_location_in_hole.0,a_location_in_hole.1);
        boundaries.into_iter().fold(point_range, |range, (y,x)| {(
            if y< range.0 {y} else {range.0},
            if x< range.1 {x} else {range.1},
            if y> range.2 {y} else {range.2},
            if x> range.3 {x} else {range.3},
        )})
    }
    fn init_portals_from_map_data(&mut self) -> Result<(),Error> {
        // find outside corners of donut data
        let (top,left,bottom,right) = self.donut_range;
        for dir in &[North,West,South,East] {
            match dir {
                // Scan for Portals at top
                South => {
                    let start_row = top-1;
                    for x in left..=right {
                        let portal_start = (start_row, x);
                        if let Some(PortalChar(ch)) = self.map_data(0).get(&portal_start) {
                            let loc = dir.reverse_from(portal_start);
                            let ch2 = match self.map_data(0).get(&loc) {
                                Some(PortalChar(c)) => c,
                                _ => return Err(MapAssertFail {msg: format!("Can't find 2nd char in portal name at {:?}", loc)}),
                            };
                            let mut name = ch.to_string();
                            name.push(*ch2);
                            if *dir == South||*dir==East {name = name.chars().rev().collect::<String>();};
                            self.portals.insert((name.clone(),portal_start),
                                Portal {
                                    name,
                                    location: portal_start,
                                    is_outer: true,
                                    ejection_direction: *dir,
                                }
                            );
                        }
                    }
                },
                // Scan for Portals at bottom
                North => {
                    let start_row = bottom+1;
                    for x in left..=right {
                        let portal_start = (start_row, x);
                        if let Some(PortalChar(ch)) = self.map_data(0).get(&portal_start) {
                            let loc = dir.reverse_from(portal_start);
                            let ch2 = match self.map_data(0).get(&loc) {
                                Some(PortalChar(c)) => c,
                                _ => return Err(MapAssertFail {msg: format!("Can't find 2nd half of portal name at {:?}", loc)}),
                            };
                            let mut name = ch.to_string();
                            name.push(*ch2);
                            if *dir == South||*dir==East {name = name.chars().rev().collect::<String>();};
                            self.portals.insert((name.clone(),portal_start),
                                Portal {
                                    name,
                                    location: portal_start,
                                    is_outer: true,
                                    ejection_direction: *dir,
                                }
                            );
                        }
                    }
                },
                // Scan for Portals at left
                East => {
                    let start_col = left-1;
                    for y in top..=bottom {
                        let portal_start = (y, start_col);
                        if let Some(PortalChar(ch)) = self.map_data(0).get(&portal_start) {
                            let loc = dir.reverse_from(portal_start);
                            let ch2 = match self.map_data(0).get(&loc) {
                                Some(PortalChar(c)) => c,
                                _ => return Err(MapAssertFail {msg: format!("Can't find 2nd half of portal name at {:?}", loc)}),
                            };
                            let mut name = ch.to_string();
                            name.push(*ch2);
                            if *dir == South||*dir==East {name = name.chars().rev().collect::<String>();};
                            self.portals.insert((name.clone(),portal_start),
                                Portal {
                                    name,
                                    location: portal_start,
                                    is_outer: true,
                                    ejection_direction: *dir,
                                }
                            );
                        }
                    }
                },
                // Scan for Portals at right
                West => {
                    let start_col = right+1;
                    for y in top..=bottom {
                        let portal_start = (y, start_col);
                        if let Some(PortalChar(ch)) = self.map_data(0).get(&portal_start) {
                            let loc = dir.reverse_from(portal_start);
                            let ch2 = match self.map_data(0).get(&loc) {
                                Some(PortalChar(ch)) => ch,
                                _ => return Err(MapAssertFail {msg: format!("Can't find 2nd half of portal name at {:?}", loc)}),
                            };
                            let mut name = ch.to_string();
                            name.push(*ch2);
                            if *dir == South||*dir==East {name = name.chars().rev().collect::<String>();};
                            self.portals.insert((name.clone(),portal_start),
                                Portal {
                                    name,
                                    location: portal_start,
                                    is_outer: true,
                                    ejection_direction: *dir,
                                }
                            );
                        }
                    }
                },
            }
        }
        // find inside corners of donut data, next to donut hole
        let (top,left,bottom,right) = self.donut_hole_range;
        for dir in &[North,West,South,East] {
            match dir {
                // Scan for Portals at top of donut hole
                North => {
                    let start_row = top+1;
                    for x in left..=right {
                        let portal_start = (start_row, x);
                        if let Some(PortalChar(ch)) = self.map_data(0).get(&portal_start) {
                            let loc = dir.reverse_from(portal_start);
                            let ch2 = match self.map_data(0).get(&loc) {
                                Some(PortalChar(ch)) => ch,
                                _ => return Err(MapAssertFail {msg: format!("Can't find 2nd half of portal name at {:?}", loc)}),
                            };
                            let mut name = ch.to_string();
                            name.push(*ch2);
                            if *dir == South||*dir==East {name = name.chars().rev().collect::<String>();};
                            self.portals.insert((name.clone(),portal_start),
                                Portal {
                                    name,
                                    location: portal_start,
                                    is_outer: false,
                                    ejection_direction: *dir,
                                }
                            );
                        }
                    }
                },
                // Scan for Portals at bottom of donut hole
                South => {
                    let start_row = bottom-1;
                    for x in left..=right {
                        let portal_start = (start_row, x);
                        if let Some(PortalChar(ch)) = self.map_data(0).get(&portal_start) {
                            let loc = dir.reverse_from(portal_start);
                            let ch2 = match self.map_data(0).get(&loc) {
                                Some(PortalChar(ch)) => ch,
                                _ => return Err(MapAssertFail {msg: format!("Can't find 2nd half of portal name at {:?}", loc)}),
                            };
                            let mut name = ch.to_string();
                            name.push(*ch2);
                            if *dir == South||*dir==East {name = name.chars().rev().collect::<String>();};
                            self.portals.insert((name.clone(),portal_start),
                                Portal {
                                    name,
                                    location: portal_start,
                                    is_outer: false,
                                    ejection_direction: *dir,
                                }
                            );
                        }
                    }
                },
                // Scan for Portals at left of donut hole
                West => {
                    let start_col = left+1;
                    for y in top..=bottom {
                        let portal_start = (y, start_col);
                        if let Some(PortalChar(ch)) = self.map_data(0).get(&portal_start) {
                            let loc = dir.reverse_from(portal_start);
                            let ch2 = match self.map_data(0).get(&loc) {
                                Some(PortalChar(ch)) => ch,
                                _ => return Err(MapAssertFail {msg: format!("Can't find 2nd half of portal name at {:?}", loc)}),
                            };
                            let mut name = ch.to_string();
                            name.push(*ch2);
                            if *dir == South||*dir==East {name = name.chars().rev().collect::<String>();};
                            self.portals.insert((name.clone(),portal_start),
                                Portal {
                                    name,
                                    location: portal_start,
                                    is_outer: false,
                                    ejection_direction: *dir,
                                }
                            );
                        }
                    }
                },
                // Scan for Portals at right of donut hole
                East => {
                    let start_col = right-1;
                    for y in top..=bottom {
                        let portal_start = (y, start_col);
                        if let Some(PortalChar(ch)) = self.map_data(0).get(&portal_start) {
                            let loc = dir.reverse_from(portal_start);
                            if DBG {println!("Reverse from {:?} is {:?} for portal char {}", portal_start, loc, ch)};
                            let ch2 = match self.map_data(0).get(&loc) {
                                Some(PortalChar(ch)) => ch,
                                _ => return Err(MapAssertFail {msg: format!("Can't find 2nd half of portal name at {:?}", loc)}),
                            };
                            let mut name = ch.to_string();
                            name.push(*ch2);
                            if *dir == South||*dir==East {name = name.chars().rev().collect::<String>();};
                            self.portals.insert((name.clone(),portal_start),
                                Portal {
                                    name,
                                    location: portal_start,
                                    is_outer: false,
                                    ejection_direction: *dir,
                                }
                            );
                        }
                    }
                },
            }
        }
        let unique_portal_names = self.portals.iter().map(|((s,_),_)|{s}).collect::<HashSet<&String>>();
        if DBG {println!("Found {} unique names in {} portals", unique_portal_names.len(), self.portals.len())};
        for name in unique_portal_names {
            let mut maybe_two: Vec<_> = self.portals.iter().filter_map(|((n,loc),_)|if n==name {Some(loc)} else {None}).collect();
            let one = *maybe_two.pop().unwrap();
            let two = match maybe_two.pop() {Some(loc) => Some(*loc), None=>None,};
            if DBG {println!("Portal {}: {:?} to {:?}", name, one, two)};
            self.transport.insert(one, two);
            if let Some(second) = two {
                self.transport.insert(second, Some(one));
            }
        }
        Ok(())
    }
    fn shortest_path_to_end(&mut self, you_are_here: Location, layer_depth: usize, distance_to_here: usize) -> Result<usize,Error> {
        if DBGG {println!("In shortest_path_to_end(), on level {}, map contains {} INFINITIES", layer_depth,
            self.map_data(layer_depth).iter().fold(0,|count_of_infinities, (_loc,item)| {
                count_of_infinities + match *item {
                    Empty(dist) if dist == INFINITY => 1,
                    _ => 0,
                }
            })
        );}
        if distance_to_here > MAX_DISTANCE {
            if DBG {println!("    >> ABORT -- Too far! This can't be right?")};
            return Ok(INFINITY)
        } // ABORT
        if layer_depth > DEEPEST_LEVEL {
            if DBG {println!("    >> ABORT -- Too deep! This can't be right?")};
            return Ok(INFINITY)
        } // ABORT
        // End recursion with any path (of many!) or FINAL GOAL achieved.
        let debug_maybe_portal = match self.find_portal_by_loc(&you_are_here) {
            Some(p) => Some(p.clone()),
            _ => None,
        };
        // match whats_underfoot
        match self.map_data_mut(layer_depth).get_mut(&you_are_here) {
            None => Err(MapAssertFail {msg: format!("We somehow walked off the map to {:?}", you_are_here)}),
            Some(Wall) => Ok(INFINITY), // Hit a Wall
            Some(Empty(distance)) if *distance <= distance_to_here => Ok(INFINITY), // Crossed paths. Been here done that.
            Some(Empty(distance)) => {
                *distance = distance_to_here; // Mark your trail -- prevent back-tracking
                // recurse into cardinal directions
                let n = self.shortest_path_to_end(North.move_from(&you_are_here), layer_depth, distance_to_here+1)?;
                let s = self.shortest_path_to_end(South.move_from(&you_are_here), layer_depth, distance_to_here+1)?;
                let e = self.shortest_path_to_end( East.move_from(&you_are_here), layer_depth, distance_to_here+1)?;
                let w = self.shortest_path_to_end( West.move_from(&you_are_here), layer_depth, distance_to_here+1)?;
                Ok(*[n,s,e,w].iter().min().unwrap())
            },
            // CRITICAL LOGIC on portal transport.  This took DAYS to debug when done poorly -- and without
            // sufficient thought.
            // 
            // Here are the RULES FOR PORTAL REUSE:
            //
            // Reuse IN THE SAME DIRECTION on the SAME LAYER is impossible by design -- the path LEADING to
            // the portal is already blocked by prior use, so the portal is safe from re-use on the same level. 
            // However, reuse IN THE SAME DIRECTION on DIFFERENT LAYERS is allowed, and might be a concern for
            // infinite recursion.  Is a concern.  We will handle this by having a max depth counter.
            // Reuse IN THE OPPOSITE DIRECTION on the new level we transport/warp to is tricky. We must and
            // we do ALLOW REUSE IN REVERSE DIRECTION FROM ANOTHER LEVEL (a 3rd, and different level from
            // either end of the portal).
            // I thought we had to disallow immediate backtracking, which does happen with default Dijkstra.
            // Since the portal has no blocking mechanism, the Dijkstra flood algo pours right back in
            // immediately after we step out.  But then the perform the reverse transport and the Dijkstra
            // path fails completely and ends after returning to the layer we just transported from.
            // This fine. NO EXTRA COMPLEXITY REQUIRED.  (Maybe to make DBG messages a little cleaner?)
            
            Some(PortalChar(_)) => {
                let destination_location = match self.transport.get(&you_are_here) {
                    Some(maybe_destination) => *maybe_destination,
                    None => return Err(MapAssertFail {msg: "Portal not located???".to_string()}),
                };
                match destination_location {
                    None => {
                        if distance_to_here < 5 { // test was == 0, but I'm debugging an edge case...
                            // Portal AA -- We're just starting out
                            let starting_loc = self.find_portal_by_loc(&you_are_here).unwrap().ejection_direction.move_from(&you_are_here);
                            assert_eq!(layer_depth, 0);
                            self.shortest_path_to_end(starting_loc, 0, 0)
                        } else {
                            if DBG {println!("Reached Portal {} on level {}, distance so far is {}",
                                debug_maybe_portal.clone().unwrap().name,
                                layer_depth,
                                if distance_to_here == INFINITY {"INFINITY".to_string()} else {distance_to_here.to_string()});}
                            if 0 == layer_depth {
                                // Portal ZZ -- DESTINATION REACHED.  Return distance_to_here to end recursion finally.
                                if DBG {println!("    >> DONE -- This branch finished successfully! <<    Total distance: {}", distance_to_here - 1);}
                                Ok(distance_to_here - 1) // We're done! (subtracting one because we are NOT stepping into the final portal)
                            } else {
                                // inner portals AA or ZZ
                                if DBG {println!("    >> TERMINATED -- Portal {} is a wall at level {}.", debug_maybe_portal.unwrap().name, layer_depth);}
                                Ok(INFINITY) // [Effectively] Hit a wall! AA and ZZ act like walls on all but outer layer
                            }
                        }
                    },
                    Some(other_side) => {
                        let (destination_is_outer,ejection_direction) = match self.find_portal_by_loc(&other_side).unwrap() {
                            p => (p.is_outer, p.ejection_direction)
                        };
                        // Outer portals act like walls on top layer
                        if 0 == layer_depth && !destination_is_outer {
                            if DBGG {println!("    >> TERMINATED -- Can't recurse higher than level {}.", layer_depth);}
                            return Ok(INFINITY);
                        }
                        let destination_output_depth = if destination_is_outer {layer_depth+1} else {layer_depth-1};
                        if DBGG {println!("----------- to LEVEL {} -------------", layer_depth);}
                        let output_dest = ejection_direction.move_from(&other_side);
                        if DBG {println!("Using Portal {} to warp {} to level {} at {:?} heading {:?}, distance so far is {}",
                            debug_maybe_portal.unwrap().name,
                            if destination_is_outer {"down .."} else {" up  ^^"},
                            destination_output_depth,
                            other_side,
                            ejection_direction,
                            if distance_to_here == INFINITY {"INFINITY".to_string()} else {distance_to_here.to_string()});}
                        let shortest = self.shortest_path_to_end(output_dest, destination_output_depth, distance_to_here+0);
                        shortest
                    },
                }
            },
        }
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
#[derive(Debug,Copy,Clone,Eq,PartialEq)]
enum MapData {
    Wall,
    Empty(usize),
    PortalChar(char),
}
// See https://jrgraphix.net/r/Unicode/2700-27BF for Dingbats in unicode
impl MapData {
    fn to_char(&self) -> char {
        match *self {
            Empty(_) => '.',
            Wall => '#',
            PortalChar(ch) => ch,
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
            mt if mt == Empty(0).to_char() => Empty(INFINITY),
            w if w == Wall.to_char() => Wall,
            p if p.is_alphabetic() && p.is_uppercase() => PortalChar(p),
            _ => return Err(Error::IllegalMapData { ch }),
        };
        Ok(status)
    }
}
#[derive(Debug,Copy,Clone,Eq,PartialEq)]
enum CardinalDirection {
    North=1,
    South=2,
    West=3,
    East=4,
}
impl CardinalDirection {
    fn move_from(&self, loc: &Location) -> Location {
        match self {
            North => (loc.0-1, loc.1),
            South => (loc.0+1, loc.1),
            West => (loc.0, loc.1-1),
            East => (loc.0, loc.1+1),
        }
    }
    fn reverse_from(&self, loc: Location) -> Location {
        match self {
            South => (loc.0-1, loc.1),
            North => (loc.0+1, loc.1),
            East => (loc.0, loc.1-1),
            West => (loc.0, loc.1+1),
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

#[test]
fn test_ex3() -> Result<(),Error> {
    assert_eq!(process_part2("ex3_part2.txt")?, 396);
    Ok(())
}
#[test]
fn test_input() -> Result<(),Error> {
    assert_eq!(process_part2("input.txt")?, 999);
    Ok(())
}
