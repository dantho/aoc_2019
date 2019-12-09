/// Day03 code stolen from https://github.com/kodsnack/advent_of_code_2019/blob/master/tomasskare-rust/day2/src/main.rs
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::collections::HashMap;

fn main() {
    let filename = "input.txt";
    let fd = File::open(filename).expect(&format!("Failure opening {}", filename));
    let buf = BufReader::new(fd);
    let COM = Body::new("COM"); // Center of Mass
    let mut known_bodies = HashMap::new(); // Empty universe.  Deep.
    known_bodies.insert(COM.name.clone(), COM);
    buf.lines().for_each(|line| {
        let line = line.unwrap();
        let mut input_pair:Vec<&str> = line.split(')').collect();
        let new_satelite_name = input_pair.pop().unwrap().to_string();
        assert_ne!(&new_satelite_name,"COM");
        let orbitee_name = input_pair.pop().unwrap().to_string();
        assert_eq!(input_pair.len(), 0);
        known_bodies.entry(new_satelite_name.clone())
            .or_insert(Body::new(&new_satelite_name));
        known_bodies.entry(orbitee_name.clone())
            .or_insert(Body::new(&orbitee_name)).satelites.push(new_satelite_name);
    });
    println!("Part 1) Total orbit count: {}", deep_sat_count("COM", &known_bodies, 0));
    let path_between = path_between("YOU", "SAN", &known_bodies);
    // println!("Orbital path from me to Santa: {:?}", path_between);
    println!("Part 2) Orbital path to Santa: {}", path_between.len() as isize - 3);
}
fn path_between<'a>(body1: &'a str, body2: &'a str, bodies: &'a HashMap<String, Body>) -> Vec<&'a str> {
    let mut root_path1 = path_to_COM(body1, bodies);
    root_path1.reverse();
    let mut root_path2 = path_to_COM(body2, bodies);
    root_path2.reverse();
    // find intersection
    // println!("Path to YOU {:?}", &root_path1);
    // println!("Path to SAN {:?}", &root_path2);
    let mut highest_common = "none found";
    loop {
        let pop1 = root_path1.pop();
        let pop2 = root_path2.pop();
        if pop1 == pop2 && pop1 != None {
            highest_common = pop1.unwrap();
        } else {
            if let Some(b) = pop1 { root_path1.push(b); }
            if let Some(b) = pop2 { root_path2.push(b); }
            break;
        }
    }
    let mut path = root_path1;
    path.push(highest_common);
    root_path2.reverse();
    path.append(&mut root_path2);
    path
}
fn path_to_COM<'a>(body_name: &'a str, bodies: &'a HashMap<String, Body>) -> Vec<&'a str> {
    let mut v = if body_name == "COM" {
        Vec::new()
    } else {
        let orbitee_name = find_orbitee(body_name, bodies);
        path_to_COM(orbitee_name,bodies)
    };
    v.push(body_name);
    v
}
fn find_orbitee<'a>(orbiter_name: &'a str, bodies: &'a HashMap<String, Body>) -> &'a str {
    for (name, body) in bodies {
        if body.satelites.contains(&orbiter_name.to_string()) {
            return name;
        }
    }
    panic!("'{}' is not orbiting ANY body!", orbiter_name);
}
fn deep_sat_count(body_name: &str, bodies: &HashMap<String, Body>, depth: usize) -> usize {
    match bodies[body_name].sat_count() {
        0 => {
            // println!("{} at depth {} has no satelites", body_name, depth);
            depth
        },
        n => {
            // println!("found {} satelites of {} at depth {}", n, body_name, depth);
            bodies[body_name].satelites.iter().fold(depth, |acc, sat_name| { acc + deep_sat_count(sat_name, bodies, depth+1) })
        },
    }
}
struct Body {
    name: String,
    satelites: Vec<String>,
}
impl Body {
    fn new(name: &str) -> Self {
        let name = name.to_string();
        let satelites: Vec<String> = Vec::new();
        Body {name, satelites}
    }
    fn sat_count(&self) -> usize {
        self.satelites.len()
    }
}