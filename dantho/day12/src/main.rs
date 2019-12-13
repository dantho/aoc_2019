/// Day03 code stolen from https://github.com/kodsnack/advent_of_code_2019/blob/master/tomasskare-rust/day2/src/main.rs
use std::fmt::Debug;
use std::cmp::Ordering::*;

#[derive(Debug)]
enum Error {
    IllegalOpcode { code: isize },
}
#[derive(Debug,Clone,Copy)]
struct ThreeD {
    x: isize,
    y: isize,
    z: isize,
}
type Pos = ThreeD;
type Vel = ThreeD;
#[derive(Debug)]
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
}
fn main() -> Result<(),Error> {
    // let example1 = vec![
    //     Pos {x:-1, y:0,   z:2 },
    //     Pos {x:2,  y:-10, z:-7},
    //     Pos {x:4,  y:-8,  z:8 },
    //     Pos {x:3,  y:5,   z:-1},
    // ];
    // let example2 = vec![
    //     Pos {x:-1, y:0,   z:2 },
    //     Pos {x:2,  y:-10, z:-7},
    //     Pos {x:4,  y:-8,  z:8 },
    //     Pos {x:3,  y:5,   z:-1},
    // ];
    let input = vec![
        Pos {x:-6,  y:-5, z:-8 },
        Pos {x:0,   y:-3, z:-13},
        Pos {x:-15, y:10, z:-11},
        Pos {x:-3,  y:-8, z:3  },
    ];
    let initial_pos = input;

    let initial_vel = vec![Vel {x:0, y:0, z:0}; 4];
    let mut moons: Vec<_> = initial_pos.into_iter().zip(initial_vel.into_iter())
        .map(|(pos,vel)|{ Moon {pos,vel} }).collect();
    println!("Initial Moons: ", );
    for moon in &mut moons {
        println!("   {:?}", moon);
    }
    for _step in 0..1000 {
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
        // // debug print
        // println!("Moons:");
        // for moon in &mut moons {
        //     println!("   {:?}", moon);
        // }
    }
    let total_energy: isize = moons.iter()
    .map(|moon|{
        (moon.pos.x.abs() + moon.pos.y.abs() + moon.pos.z.abs()) *
        (moon.vel.x.abs() + moon.vel.y.abs() + moon.vel.z.abs())
    })
    .sum();
    println!("Part 1: Total Energy is {}", total_energy);
    Ok(())
}
