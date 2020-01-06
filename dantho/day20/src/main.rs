/// https://adventofcode.com/2019/day/15
const ESC_CLS: &'static str = "\x1B[2J";
// const ESC_CURSOR_ON: &'static str = "\x1B[?25h";
const ESC_CURSOR_OFF: &'static str = "\x1B[?25l";
const DBG: bool = false;
const INFINITY: usize = std::usize::MAX/1_000_000_000_000_000_000 * 1_000_000_000_000_000_000 + 123_456; // BIG number! But also recognizable as "special"
const MAX_DISTANCE: usize = 20000;
const DEEPEST_LEVEL: usize = 60;

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

const STACK_SIZE: usize = 4 * 1024 * 1024;

fn run() -> Result<(),Error> {
    let filename = "ex3_part2.txt";
    let part2 = process_part2(filename)?;
    // println!("Part 1: Fewest steps from AA to ZZ is {}", if part1 == INFINITY {"INFINITY".to_string()} else {part1.to_string()});
    println!("Part 2: Fewest steps from AA to ZZ in recursive Donut Space is {}", if part2 == INFINITY {"INFINITY".to_string()} else {part2.to_string()});
    Ok(())
}

fn main() {
    // Spawn thread with explicit stack size
    let child = thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(run)
        .unwrap();

    // Wait for thread to join
    child.join().unwrap();
}

type Location = (usize,usize);

// How many steps does it take to get from the open tile marked AA to the open tile marked ZZ?
fn process_part2(filename: &'static str) -> Result<usize,Error> {
    let mut donut_map = DonutMap::new(filename)?;
    donut_map.redraw_screen()?;
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
    let aa_portal = donut_map.find_portals_by_name("AA").pop().unwrap();
    let starting_loc = aa_portal.location;
    let part2 = donut_map.shortest_path_to_end(starting_loc, 0)?;
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
    fn go_up_one_level(&mut self) -> usize {
        if self.stack_depth == 0 {panic!("We're raising the roof here!")}
        if DBG {println!("         ...up");}
        if DBG {println!("   ...up... ");}
        if DBG {println!("Up... ");}
        self.stack_depth -= 1;
        self.stack_depth
    }
    fn go_down_one_level(&mut self) -> usize {
        if DBG {println!("Down...");}
        if self.stack_depth == self.maps.len()-1 {
            if DBG {println!("     ...down...");}
            // We need to create a new (lower) level!
            let mut new_level = self.map_data().clone();
            // clear distances on new layer
            for (_,item) in &mut new_level {
                match item {
                    Empty(d) => *d = INFINITY,
                    _ => (),
                }
            };
            self.maps.push(new_level);
        }
        if DBG {println!("             ...down");}
        self.stack_depth += 1;
        self.stack_depth
    }
    fn map_data(&self) -> &BTreeMap<Location,MapData> {
        &self.maps[self.stack_depth]
    }
    fn map_data_mut(&mut self) -> &mut BTreeMap<Location,MapData> {
        &mut self.maps[self.stack_depth]
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
    fn shortest_path_to_end(&mut self, you_are_here: Location, distance_to_here: usize) -> Result<usize,Error> {
        if DBG {println!("In shortest_path_to_end(), on level {}, map contains {} INFINITIES", self.stack_depth,
            self.map_data().iter().fold(0,|count_of_infinities, (_loc,item)| {
                count_of_infinities + match *item {
                    Empty(dist) if dist == INFINITY => 1,
                    _ => 0,
                }
            })
        );}
        if distance_to_here > MAX_DISTANCE {
            println!("    >> ABORT -- Too far! This can't be right?", );
            return Ok(INFINITY)
        } // ABORT
        if self.stack_depth > DEEPEST_LEVEL {
            println!("    >> ABORT -- Too deep! This can't be right?", );
            return Ok(INFINITY)
        } // ABORT
        // End recursion with any path (of many!) or FINAL GOAL achieved.
        let debug_maybe_portal = match self.find_portal_by_loc(&you_are_here) {
            Some(p) => Some(p.clone()),
            _ => None,
        };
        // match whats_underfoot
        match self.map_data_mut().get_mut(&you_are_here) {
            None => Err(MapAssertFail {msg: format!("We somehow walked off the map to {:?}", you_are_here)}),
            Some(Wall) => Ok(INFINITY), // Hit a Wall
            Some(Empty(distance)) if *distance <= distance_to_here => Ok(INFINITY), // Crossed paths. Been here done that.
            Some(Empty(distance)) => {
                *distance = distance_to_here; // Mark your trail -- prevent back-tracking
                // recurse into cardinal directions
                let saved_depth = self.stack_depth;
                let n = self.shortest_path_to_end(North.move_from(&you_are_here), distance_to_here+1)?;
                if self.stack_depth != saved_depth {
                    if true {println!("{} at level {}, changing back to level {}",
                        if n == INFINITY {"Dead end"} else {"Found ZZ"}, self.stack_depth, saved_depth);}
                    self.stack_depth = saved_depth;
                }
                let s = self.shortest_path_to_end(South.move_from(&you_are_here), distance_to_here+1)?;
                if self.stack_depth != saved_depth {
                    if true {println!("{} at level {}, changing back to level {}",
                        if s == INFINITY {"Dead end"} else {"Found ZZ"}, self.stack_depth, saved_depth);}
                    self.stack_depth = saved_depth;
                }
                let e = self.shortest_path_to_end( East.move_from(&you_are_here), distance_to_here+1)?;
                if self.stack_depth != saved_depth {
                    if true {println!("{} at level {}, changing back to level {}",
                        if e == INFINITY {"Dead end"} else {"Found ZZ"}, self.stack_depth, saved_depth);}
                    self.stack_depth = saved_depth;
                }
                let w = self.shortest_path_to_end( West.move_from(&you_are_here), distance_to_here+1)?;
                if self.stack_depth != saved_depth {
                    if true {println!("{} at level {}, changing back to level {}",
                        if w == INFINITY {"Dead end"} else {"Found ZZ"}, self.stack_depth, saved_depth);}
                    self.stack_depth = saved_depth;
                }
                Ok(*[n,s,e,w].iter().min().unwrap())
            },
            Some(PortalChar(_,distance)) => {
                if *distance <= distance_to_here {return Ok(INFINITY);} // Crossed paths. Been here done that.
                // SHOULD WE DO THIS?.. PORTALS must be REENTRANT in the other direction
                // Saying "no" might be too aggressive. We certainly MUST make the other direction allowable once, at least.
                *distance = distance_to_here; // Mark your trail
                // Prevent reentering the same portal IN THE SAME DIRECTION more than once
                // Determine if this Portal is THE END!  If so, return distance_to_here!!
                // If not, PASS THROUGH the portal
                // COMPLICATED LOGIC FOLLOWS -- We don't want to block this Portal in the reverse direction,
                // but we also DON'T want the Dijkstra algo immediately reversing back into the portal.
                // SO... we check if THIS portal is one we just popped through from the other side just one step ago
                // And if so, reject the transport.  No warp for you!  No immediate round-trip warp, that is.
                // 
                // 
                // todo
                let destination_location = match self.transport.get(&you_are_here) {
                    Some(maybe_destination) => *maybe_destination,
                    None => return Err(MapAssertFail {msg: "Portal not located???".to_string()}),
                };
                match destination_location {
                    None => {
                        if 0 == distance_to_here {
                            // Portal AA -- We're just starting out
                            let starting_loc = self.find_portal_by_loc(&you_are_here).unwrap().ejection_direction.move_from(&you_are_here);
                            assert_eq!(self.stack_depth, 0);
                            self.shortest_path_to_end(starting_loc, 0)
                        } else {
                            if true {println!("Reached Portal {} on level {}, distance so far is {}",
                                debug_maybe_portal.clone().unwrap().name,
                                self.stack_depth,
                                if distance_to_here == INFINITY {"INFINITY".to_string()} else {distance_to_here.to_string()});}
                            if 0 == self.stack_depth {
                                // Portal ZZ -- DESTINATION REACHED.  Return distance_to_here to end recursion finally.
                                if true {println!("    >> DONE -- This branch finished successfully! <<    Total distance: {}", distance_to_here - 1);}
                                Ok(distance_to_here - 1) // We're done! (subtracting one because we are NOT stepping into the final portal)
                            } else {
                                // inner portals AA or ZZ
                                if true {println!("    >> TERMINATED -- Portal {} is a wall at level {}.", debug_maybe_portal.unwrap().name, self.stack_depth);}
                                Ok(INFINITY) // [Effectively] Hit a wall! AA and ZZ act like walls on all but outer layer
                            }
                        }
                    },
                    Some(other_side) => {
                        let (destination_is_outer,ejection_direction) = match self.find_portal_by_loc(&other_side).unwrap() {
                            p => (p.is_outer, p.ejection_direction)
                        };
                        // Outer portals act like walls on top layer
                        if 0 == self.stack_depth && !destination_is_outer {
                            if DBG {println!("    >> TERMINATED -- Can't recurse higher than level {}.", self.stack_depth);}
                            return Ok(INFINITY);
                        }
                        let output_depth = if destination_is_outer {self.go_down_one_level()} else {self.go_up_one_level()};
                        if DBG {println!("----------- to LEVEL {} -------------", output_depth);}
                        let output_dest = ejection_direction.move_from(&other_side);
                        if true {println!("Using Portal {} to warp {} to level {} at {:?} heading {:?}, distance so far is {}",
                            debug_maybe_portal.unwrap().name,
                            if destination_is_outer {"down .."} else {" up  ^^"},
                            output_depth,
                            destination_location.unwrap(),
                            ejection_direction,
                            if distance_to_here == INFINITY {"INFINITY".to_string()} else {distance_to_here.to_string()});}
                        let shortest = self.shortest_path_to_end(output_dest, distance_to_here+0);
                        shortest
                    },
                }
            },
        }
    }
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
            match me.map_data().get(corner) {
                None => (),
                Some(unexpected) => return Err(MapAssertFail {msg: format!("Expected nothing at {:?}, found {:?}", *corner, unexpected)}),
            }
        }
        match me.map_data().get(&(2,2)) {
            Some(Wall) => (),
            _ => return Err(MapAssertFail {msg: format!("Expected corner at {:?}", (2,2))}),
        }
        match me.map_data().get(&(y-2,x-2)) {
            Some(Wall) => (),
            _ => return Err(MapAssertFail {msg: format!("Expected corner at {:?}", (y-2,x-2))}),
        }
        Ok(me)
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
        println!("center: {:?}", center);
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
            println!("Stop when you hit donut: {:?}", stop_when_you_hit_donut);
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
                        if let Some(PortalChar(ch,_)) = self.map_data().get(&portal_start) {
                            let loc = dir.reverse_from(portal_start);
                            let ch2 = match self.map_data().get(&loc) {
                                Some(PortalChar(c,_)) => c,
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
                        if let Some(PortalChar(ch,_)) = self.map_data().get(&portal_start) {
                            let loc = dir.reverse_from(portal_start);
                            let ch2 = match self.map_data().get(&loc) {
                                Some(PortalChar(c,_)) => c,
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
                        if let Some(PortalChar(ch,_)) = self.map_data().get(&portal_start) {
                            let loc = dir.reverse_from(portal_start);
                            let ch2 = match self.map_data().get(&loc) {
                                Some(PortalChar(c,_)) => c,
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
                        if let Some(PortalChar(ch,_)) = self.map_data().get(&portal_start) {
                            let loc = dir.reverse_from(portal_start);
                            let ch2 = match self.map_data().get(&loc) {
                                Some(PortalChar(c,_)) => c,
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
                        if let Some(PortalChar(ch,_)) = self.map_data().get(&portal_start) {
                            let loc = dir.reverse_from(portal_start);
                            let ch2 = match self.map_data().get(&loc) {
                                Some(PortalChar(c,_)) => c,
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
                        if let Some(PortalChar(ch,_)) = self.map_data().get(&portal_start) {
                            let loc = dir.reverse_from(portal_start);
                            let ch2 = match self.map_data().get(&loc) {
                                Some(PortalChar(c,_)) => c,
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
                        if let Some(PortalChar(ch,_)) = self.map_data().get(&portal_start) {
                            let loc = dir.reverse_from(portal_start);
                            let ch2 = match self.map_data().get(&loc) {
                                Some(PortalChar(c,_)) => c,
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
                        if let Some(PortalChar(ch,_)) = self.map_data().get(&portal_start) {
                            let loc = dir.reverse_from(portal_start);
                            println!("Reverse from {:?} is {:?} for portal char {}", portal_start, loc, ch);
                            let ch2 = match self.map_data().get(&loc) {
                                Some(PortalChar(c,_)) => c,
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
        let mut unique_portal_names = self.portals.iter().map(|((s,_),_)|{s}).collect::<HashSet<&String>>();
        println!("Found {} unique names in {} portals", unique_portal_names.len(), self.portals.len() );
        for name in unique_portal_names {
            let mut maybe_two: Vec<_> = self.portals.iter().filter_map(|((n,loc),_)|if n==name {Some(loc)} else {None}).collect();
            let one = *maybe_two.pop().unwrap();
            let two = match maybe_two.pop() {Some(loc) => Some(*loc), None=>None,};
            println!("Portal {}: {:?} to {:?}", name, one, two );
            self.transport.insert(one, two);
            if let Some(second) = two {
                self.transport.insert(second, Some(one));
            }
        }
        Ok(())
    }
    fn draw_location(&self, loc: Location) -> Result<(),Error> {
        set_cursor_loc(loc.0, loc.1);
        let map_item = match self.map_data().get(&loc) {
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
        for (loc, _) in &self.maps[0] {
            self.draw_location(*loc)?;
        }
        println!("");
        Ok(())
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
    PortalChar(char,usize),
}
// See https://jrgraphix.net/r/Unicode/2700-27BF for Dingbats in unicode
impl MapData {
    fn to_char(&self) -> char {
        match *self {
            Empty(_) => '.',
            Wall => '#',
            PortalChar(ch,_) => ch,
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
            p if p.is_alphabetic() && p.is_uppercase() => PortalChar(p,INFINITY),
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
fn test_input() -> Result<(),Error> {
    assert_eq!(process_part1("input.txt")?, 0);
    Ok(())
}
#[test]
fn test_ex1() -> Result<(),Error> {
    assert_eq!(process_part1("ex1.txt")?, 23);
    Ok(())
}
#[test]
fn test_ex2() -> Result<(),Error> {
    assert_eq!(process_part1("ex2.txt")?, 58);
    Ok(())
}
#[test]
fn test_input_2() -> Result<(),Error> {
    assert_eq!(process_part1("input.txt")?, 0);
    Ok(())
}
#[test]
fn test_ex1_2() -> Result<(),Error> {
    assert_eq!(process_part1("ex1.txt")?, 26);
    Ok(())
}
#[test]
fn test_ex2_2() -> Result<(),Error> {
    assert_eq!(process_part1("ex2.txt")?, 396);
    Ok(())
}
