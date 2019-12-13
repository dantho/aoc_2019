/// Day03 code stolen from https://github.com/kodsnack/advent_of_code_2019/blob/master/tomasskare-rust/day2/src/main.rs
extern crate num;

use std::fmt::Debug;
use std::cmp::Ordering::*;
use num::integer::gcd;

#[derive(Eq,PartialEq,Hash,Debug,Clone,Copy)]
struct ThreeD {
    x: isize,
    y: isize,
    z: isize,
}
type Pos = ThreeD;
type Vel = ThreeD;
#[derive(Eq,PartialEq,Hash,Debug,Clone)]
struct Moon {
    pos: Pos,
    vel: Vel,
}
impl Moon {
    fn apply_gravity(&mut self, other: &Moon) {
        self.vel.x += match (self.pos.x).cmp(&other.pos.x) {
            Less => 1,
            Equal => 0,
            Greater => -1,            
        };
        self.vel.y += match (self.pos.y).cmp(&other.pos.y) {
            Less => 1,
            Equal => 0,
            Greater => -1,            
        };
        self.vel.z += match (self.pos.z).cmp(&other.pos.z) {
            Less => 1,
            Equal => 0,
            Greater => -1,            
        };
    }
    fn apply_velocity(&mut self) {
        self.pos.x += self.vel.x;
        self.pos.y += self.vel.y;
        self.pos.z += self.vel.z;
    }
    fn axes_are_same(&self, other: &Moon) -> (bool,bool,bool) {
        (
            self.vel.x == other.vel.x && self.vel.x == other.vel.x,
            self.vel.y == other.vel.y && self.vel.y == other.vel.y,
            self.vel.z == other.vel.z && self.vel.z == other.vel.z,
        )
    }
}
fn main() {
    let example1 = vec![
        Pos {x:-1, y:0,   z:2 },
        Pos {x:2,  y:-10, z:-7},
        Pos {x:4,  y:-8,  z:8 },
        Pos {x:3,  y:5,   z:-1},
    ];
    let example2 = vec![
        Pos {x:-8, y:-10,z:0 },
        Pos {x:5,  y:5,  z:10},
        Pos {x:2,  y:-7, z:3 },
        Pos {x:9,  y:-8, z:-3},
    ];
    let input = vec![
        Pos {x:-6,  y:-5, z:-8 },
        Pos {x:0,   y:-3, z:-13},
        Pos {x:-15, y:10, z:-11},
        Pos {x:-3,  y:-8, z:3  },
    ];
    let initial_pos = input;
    let initial_vel = vec![Vel {x:0, y:0, z:0}; 4];
    let mut moons: Vec<_> = initial_pos.clone().into_iter().zip(initial_vel.into_iter())
        .map(|(pos,vel)|{ Moon {pos,vel} }).collect();
    println!("Initial Moons: ", );
    for moon in &mut moons {
        println!("   {:?}", moon);
    }
    let initial_state = moons.clone();
    let mut repeat_found_at = (None,None,None);
    for step in 1.. {
        // apply gravity
        for _ in 0..moons.len() {
            let mut this = moons.pop().unwrap();
            for other in &moons {
                    this.apply_gravity(other);
            }
            moons.reverse();
            moons.push(this);
            moons.reverse();
        }
        // apply velocity
        for moon in &mut moons {
            moon.apply_velocity();
        }
        // compare new state vs initial state ON A PER AXIS BASIS
        let are_axes_same = moons.iter().zip(initial_state.iter())
            .fold((true,true,true), |(bx,by,bz),(moon,other)| {
                let (mx,my,mz) = moon.axes_are_same(other);
                (bx&&mx,by&&my,bz&&mz)
        });
        if let None = repeat_found_at.0 {
            if are_axes_same.0 {
                repeat_found_at = (Some(step), repeat_found_at.1, repeat_found_at.2);
                println!("X Axis repeats at Step {}", step);
            }
        }
        if let None = repeat_found_at.1 {
            if are_axes_same.1 {
                repeat_found_at = (repeat_found_at.0, Some(step), repeat_found_at.2);
                println!("Y Axis repeats at Step {}", step);
            }
        }
        if let None = repeat_found_at.2 {
            if are_axes_same.2 {
                repeat_found_at = (repeat_found_at.0, repeat_found_at.1, Some(step));
                println!("Z Axis repeats at Step {}", step);
            }
        }
        if let (Some(_), Some(_), Some(_)) = repeat_found_at {
            break;
        }
    }
    let first_repeat_on_all_axes = lcf_via_gcd(repeat_found_at.0.unwrap(),repeat_found_at.1.unwrap(),repeat_found_at.2.unwrap());
    let total_energy: isize = moons.iter()
    .map(|moon|{
        (moon.pos.x.abs() + moon.pos.y.abs() + moon.pos.z.abs()) *
        (moon.vel.x.abs() + moon.vel.y.abs() + moon.vel.z.abs())
    })
    .sum();
    println!("Part 1: Total Energy is {}", total_energy);
    println!("Part 2: First repeat occurs at Step {}", first_repeat_on_all_axes);
}

fn lcf_via_gcd(a:i32,b:i32,c:i32) -> u64 {
    let ab = gcd(a, b);
    let bc = gcd(b, c);
    let gcd_3_way = gcd(ab, bc) as u64;
    (a as u64 / gcd_3_way) * (b as u64 / gcd_3_way) * (c as u64 / gcd_3_way)
}