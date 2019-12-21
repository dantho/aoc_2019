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
use CardinalDirection::*;
use MapData::*;
use Error::*;

type Location = (usize,usize);

fn main() -> Result<(),Error> {
    let filename = "input.txt";
    let part1 = process_part1(filename)?;
    let part2 = 0;
    println!("Part 1: Fewest steps from AA to ZZ is {}", part1 );
    println!("Part 2: TBD is {}", part2 );
    Ok(())
}
// How many steps does it take to get from the open tile marked AA to the open tile marked ZZ?
fn process_part1(filename: &'static str) -> Result<usize,Error> {
    let mut donut_map = DonutMap::new(filename)?;
    donut_map.redraw_screen()?;
    println!("Portals: {:?}", donut_map.portals);
    println!("Transport: {:?}", donut_map.transport);

    let aa_portal = donut_map.portals.get(&"AA".to_string()).unwrap();
    let starting_loc = aa_portal.outside;
    let part1 = donut_map.shortest_path_to_end(starting_loc, 0)?;
    Ok(part1)
}
#[derive(Debug)]
enum Error {
    IllegalMapData {ch: char},
    MapAssertFail {msg: String},
}
#[derive(Debug)]
struct Portal {
    name: String,
    outside: Location,
    inside: Option<Location>,
}
#[derive(Debug)]
struct DonutMap {
    portals: BTreeMap<String, Portal>,
    transport: BTreeMap<Location, Option<Location>>,
    map_data: BTreeMap<Location,MapData>,
    donut_range: (usize,usize,usize,usize),
    donut_hole_range: (usize,usize,usize,usize),
}
impl DonutMap {
    fn shortest_path_to_end(&mut self, where_are_you: Location, distance_to_here: usize) -> Result<usize,Error> {
        // End recursion with any bad path or FINAL GOAL achieved.
        let whats_underfoot = self.map_data.get_mut(&where_are_you);
        match whats_underfoot {
            None => Err(MapAssertFail {msg: format!("We somehow walked off the map to {:?}", where_are_you)}),
            Some(Wall) => Ok(std::usize::MAX), // This direction is NOT the shortest path
            Some(Empty(distance)) if *distance <= distance_to_here => Ok(std::usize::MAX), // Crossed paths. Been here done that.
            Some(Empty(distance)) => {
                *distance = distance_to_here; // Mark your trail -- prevent back-tracking
                // recurse into cardinal directions
                Ok(*[
                    self.shortest_path_to_end(North.move_from(where_are_you), distance_to_here+1)?,
                    self.shortest_path_to_end(South.move_from(where_are_you), distance_to_here+1)?,
                    self.shortest_path_to_end( East.move_from(where_are_you), distance_to_here+1)?,
                    self.shortest_path_to_end( West.move_from(where_are_you), distance_to_here+1)?,
                ].iter().min().unwrap())
            },
            Some(PortalChar(_,distance)) if *distance <= distance_to_here => Ok(std::usize::MAX), // Crossed paths. Been here done that.
            Some(PortalChar(_,distance)) => {
                println!("Portal found at {:?}, old distance was {}, present distance is {}", where_are_you, distance, distance_to_here);
                *distance = distance_to_here; // Mark your trail -- prevent back-tracking
                // Determine if this Portal is THE END!  If so, return distance_to_here!!
                // If not, PASS THROUGH the portal
                match self.transport.get(&where_are_you) {
                    None => Err(MapAssertFail {msg: "Portal not located???".to_string()}),
                    Some(None) => if distance_to_here == 0 {
                        // We're just starting out
                        let mut output_dest = None;
                        for dir in &[North,South,East,West] {
                            if let Some(Empty(_)) = self.map_data.get(&dir.move_from(where_are_you)) {
                                output_dest = Some(dir.move_from(where_are_you));
                                break;
                            }
                        }
                        match output_dest {
                            Some(portal_output) => self.shortest_path_to_end(portal_output, distance_to_here+0),
                            None => Err(MapAssertFail {msg: format!("We're starting in a weird place {:?}", where_are_you)}),
                        }
                    } else {
                        Ok(distance_to_here - 1) // END OF LINE DETECTED!!!  Return distance_to_here
                    }
                    Some(Some(other_side)) => {
                        let mut output_dest = None;
                        for dir in &[North,South,East,West] {
                            if let Some(Empty(_)) = self.map_data.get(&dir.move_from(*other_side)) {
                                output_dest = Some(dir.move_from(*other_side));
                                break;
                            }
                        }
                        match output_dest {
                            Some(portal_output) => self.shortest_path_to_end(portal_output, distance_to_here+0),
                            None => Err(MapAssertFail {msg: format!("Portal tossed us into void?? From {:?} to {:?}", where_are_you, output_dest)}),
                        }
                    }
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
            match me.map_data.get(corner) {
                None => (),
                Some(unexpected) => return Err(MapAssertFail {msg: format!("Expected nothing at {:?}, found {:?}", *corner, unexpected)}),
            }
        }
        match me.map_data.get(&(2,2)) {
            Some(Wall) => (),
            _ => return Err(MapAssertFail {msg: format!("Expected corner at {:?}", (2,2))}),
        }
        match me.map_data.get(&(y-2,x-2)) {
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
            map_data,
            portals: BTreeMap::new(),
            transport: BTreeMap::new(),
            donut_range,
            donut_hole_range, 
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
                stop_when_you_hit_donut = heading.move_from(stop_when_you_hit_donut);
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
                        if let Some(PortalChar(ch,_)) = self.map_data.get(&portal_start) {
                            let loc = dir.reverse_from(portal_start);
                            let ch2 = match self.map_data.get(&loc) {
                                Some(PortalChar(c,_)) => c,
                                _ => return Err(MapAssertFail {msg: format!("Can't find 2nd char in portal name at {:?}", loc)}),
                            };
                            let mut name = ch.to_string();
                            name.push(*ch2);
                            if *dir == South||*dir==East {name = name.chars().rev().collect::<String>();};
                            self.portals.insert(name.clone(),
                                Portal {
                                    name,
                                    outside: portal_start,
                                    inside: None,
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
                        if let Some(PortalChar(ch,_)) = self.map_data.get(&portal_start) {
                            let loc = dir.reverse_from(portal_start);
                            let ch2 = match self.map_data.get(&loc) {
                                Some(PortalChar(c,_)) => c,
                                _ => return Err(MapAssertFail {msg: format!("Can't find 2nd half of portal name at {:?}", loc)}),
                            };
                            let mut name = ch.to_string();
                            name.push(*ch2);
                            if *dir == South||*dir==East {name = name.chars().rev().collect::<String>();};
                            self.portals.insert(name.clone(),
                                Portal {
                                    name,
                                    outside: portal_start,
                                    inside: None,
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
                        if let Some(PortalChar(ch,_)) = self.map_data.get(&portal_start) {
                            let loc = dir.reverse_from(portal_start);
                            let ch2 = match self.map_data.get(&loc) {
                                Some(PortalChar(c,_)) => c,
                                _ => return Err(MapAssertFail {msg: format!("Can't find 2nd half of portal name at {:?}", loc)}),
                            };
                            let mut name = ch.to_string();
                            name.push(*ch2);
                            if *dir == South||*dir==East {name = name.chars().rev().collect::<String>();};
                            self.portals.insert(name.clone(),
                                Portal {
                                    name,
                                    outside: portal_start,
                                    inside: None,
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
                        if let Some(PortalChar(ch,_)) = self.map_data.get(&portal_start) {
                            let loc = dir.reverse_from(portal_start);
                            let ch2 = match self.map_data.get(&loc) {
                                Some(PortalChar(c,_)) => c,
                                _ => return Err(MapAssertFail {msg: format!("Can't find 2nd half of portal name at {:?}", loc)}),
                            };
                            let mut name = ch.to_string();
                            name.push(*ch2);
                            if *dir == South||*dir==East {name = name.chars().rev().collect::<String>();};
                            self.portals.insert(name.clone(),
                                Portal {
                                    name,
                                    outside: portal_start,
                                    inside: None,
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
                        if let Some(PortalChar(ch,_)) = self.map_data.get(&portal_start) {
                            let loc = dir.reverse_from(portal_start);
                            let ch2 = match self.map_data.get(&loc) {
                                Some(PortalChar(c,_)) => c,
                                _ => return Err(MapAssertFail {msg: format!("Can't find 2nd half of portal name at {:?}", loc)}),
                            };
                            let mut name = ch.to_string();
                            name.push(*ch2);
                            if *dir == South||*dir==East {name = name.chars().rev().collect::<String>();};
                            let p = match self.portals.get_mut(&name) {
                                Some(pp) => pp,
                                None => {
                                    println!("Portals just before error: {:?}", &self.portals);
                                    return Err(MapAssertFail {msg: format!("Found portal '{}' at {:?} inside donut, but can't find it on outside.", name, loc)})
                                }
                            };
                            p.inside = Some(portal_start);
                        }
                    }
                },
                // Scan for Portals at bottom of donut hole
                South => {
                    let start_row = bottom-1;
                    for x in left..=right {
                        let portal_start = (start_row, x);
                        if let Some(PortalChar(ch,_)) = self.map_data.get(&portal_start) {
                            let loc = dir.reverse_from(portal_start);
                            let ch2 = match self.map_data.get(&loc) {
                                Some(PortalChar(c,_)) => c,
                                _ => return Err(MapAssertFail {msg: format!("Can't find 2nd half of portal name at {:?}", loc)}),
                            };
                            let mut name = ch.to_string();
                            name.push(*ch2);
                            if *dir == South||*dir==East {name = name.chars().rev().collect::<String>();};
                            let p = match self.portals.get_mut(&name) {
                                Some(pp) => pp,
                                None => return Err(MapAssertFail {msg: format!("Found portal '{}' at {:?} inside donut, but can't it on outside.", name, loc)})
                            };
                            p.inside = Some(portal_start);
                        }
                    }
                },
                // Scan for Portals at left of donut hole
                West => {
                    let start_col = left+1;
                    for y in top..=bottom {
                        let portal_start = (y, start_col);
                        if let Some(PortalChar(ch,_)) = self.map_data.get(&portal_start) {
                            let loc = dir.reverse_from(portal_start);
                            let ch2 = match self.map_data.get(&loc) {
                                Some(PortalChar(c,_)) => c,
                                _ => return Err(MapAssertFail {msg: format!("Can't find 2nd half of portal name at {:?}", loc)}),
                            };
                            let mut name = ch.to_string();
                            name.push(*ch2);
                            if *dir == South||*dir==East {name = name.chars().rev().collect::<String>();};
                            let p = match self.portals.get_mut(&name) {
                                Some(pp) => pp,
                                None => return Err(MapAssertFail {msg: format!("Found portal '{}' at {:?} inside donut, but can't it on outside.", name, loc)})
                            };
                            p.inside = Some(portal_start);
                        }
                    }
                },
                // Scan for Portals at right of donut hole
                East => {
                    let start_col = right-1;
                    for y in top..=bottom {
                        let portal_start = (y, start_col);
                        if let Some(PortalChar(ch,_)) = self.map_data.get(&portal_start) {
                            let loc = dir.reverse_from(portal_start);
                            println!("Reverse from {:?} is {:?} for portal char {}", portal_start, loc, ch);
                            let ch2 = match self.map_data.get(&loc) {
                                Some(PortalChar(c,_)) => c,
                                _ => return Err(MapAssertFail {msg: format!("Can't find 2nd half of portal name at {:?}", loc)}),
                            };
                            let mut name = ch.to_string();
                            name.push(*ch2);
                            if *dir == South||*dir==East {name = name.chars().rev().collect::<String>();};
                            let p = match self.portals.get_mut(&name) {
                                Some(pp) => pp,
                                None => return Err(MapAssertFail {msg: format!("Found portal '{}' at {:?} inside donut, but can't it on outside.", name, loc)})
                            };
                            p.inside = Some(portal_start);
                        }
                    }
                },
            }
        }
        for (_, portal) in &mut self.portals {
            let loc1 = portal.outside;
            if let Some(loc2) = portal.inside {
                self.transport.insert(loc1,Some(loc2));
                self.transport.insert(loc2,Some(loc1));
            } else {
                self.transport.insert(loc1, None);
            }
        }
        Ok(())
    }
    fn draw_location(&self, loc: Location) -> Result<(),Error> {
        set_cursor_loc(loc.0, loc.1);
        let map_item = match self.map_data.get(&loc) {
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
        for (loc, _) in &self.map_data {
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
            mt if mt == Empty(0).to_char() => Empty(std::usize::MAX),
            w if w == Wall.to_char() => Wall,
            p if p.is_alphabetic() && p.is_uppercase() => PortalChar(p,std::usize::MAX),
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
    fn move_from(&self, loc: Location) -> Location {
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
