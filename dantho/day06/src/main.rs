/// Day03 code stolen from https://github.com/kodsnack/advent_of_code_2019/blob/master/tomasskare-rust/day2/src/main.rs
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::collections::HashSet;

fn main() {
    let filename = "input.txt";
    let fd = File::open(filename).expect(&format!("Failure opening {}", filename));
    let buf = BufReader::new(fd);
    let mut CoM = Body::new("COM".to_string()); // Center of Mass
    buf.lines().for_each(|line| {
        let mut input_pair:Vec<&str> = line.unwrap().split(')').collect();
        let known_body_name = input_pair.pop().unwrap();
        let new_satelite_name = input_pair.pop().unwrap();
        assert_eq!(input_pair.len(), 0);
    });
}
struct Body {
    name: String,
    satelites: HashSet<String>,
}
impl Body {
    fn new(name: String) -> Self {
        let satelites: HashSet<String> = HashSet::new();
        Body {name, satelites}
    }
    fn sat_count(&self) -> usize {
        self.satelites.iter().fold(1,|count, (_)| {count+body.sat_count()})
    }
    fn entry(&mut self, name: String) -> Entry<String, Body> {
        let direct_satelite = {self.satelites.contains_key(&name)};
        let indirect_satelite = if direct_satelite { false } else {
            self.satelites.iter().fold(false, |found, (_, sat)| { found || sat.satelites.contains_key(&name) })
        };
        if indirect_satelite {
            for (_,sat) in &mut self.satelites {
                if sat.satelites.contains_key(&name) {
                    return sat.satelites.entry(name);
                }
            }
        }
        self.satelites.entry(name)
    }
}