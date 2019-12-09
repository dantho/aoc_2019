/// https://adventofcode.com/2019/day/3
use std::convert::TryInto;
use std::collections::HashMap;
#[derive(Debug)]
enum Error {
    InputParseError { unexpected: String },
}
fn main() -> Result<(), Error> {
    let wire_a_input = "R998,D934,L448,U443,R583,U398,R763,U98,R435,U984,L196,U410,L475,D163,R776,D796,R175,U640,R805,D857,R935,D768,L99,D75,R354,U551,L986,D592,R51,U648,L108,U8,R44,U298,L578,U710,R745,U60,L536,D62,R620,D454,L143,U407,R465,U606,L367,U107,L581,U900,R495,D12,R763,D244,R946,D424,R367,D696,L534,U452,R274,D942,L813,U336,L742,U134,R571,U703,R941,D532,L903,D833,L821,D577,L598,D83,R858,U798,L802,D852,R913,U309,L784,D235,L446,D571,R222,D714,R6,D379,R130,D313,R276,U632,L474,U11,L551,U257,R239,U218,R592,U901,L596,D367,L34,D397,R520,U547,L795,U192,R960,U77,L825,U954,R307,D399,R958,U239,R514,D863,L162,U266,R705,U731,R458,D514,R42,U314,R700,D651,L626,U555,R774,U773,R553,D107,L404,D100,R149,U845,L58,U674,R695,U255,R816,D884,R568,U618,R510,D566,L388,D947,L851,U127,L116,U143,L744,D361,L336,U903,L202,U683,R287,D174,L229,U371,L298,U839,L27,U462,R443,D39,R411,U788,L197,D160,L289,U840,L78,D262,R352,U83,R20,U109,R657,D225,R587,D968,R576,D791,R493,U805,R139,D699,R783,U140,L371,D170,L635,U257,R331,D311,R725,D970,R57,D986,L222,D760,L830,D960,L901,D367,R469,D560,L593,D940,L71,D384,R603,D689,R250,D859,L156,U499,L850,U166,R726,D210,L36,D584,R672,U47,L713,U985,R551,U22,L499,D575,R210,D829,L186,U340,R696,D939,L744,D46,L896,U467,L214,D71,R376,D379,L1,U870,R785,D779,L94,U723,L199,D185,R210,U937,R645,U25,R116,D821,R964,U959,R569,U496,R809,U112,R712,D315,L747,U754,L66,U614,L454,D945,R214,U965,L248,U702,L287,D863,R700,U768,R139,D242,R914,D818,R340,D60,L400,D924,R69,U73,L449,U393,L906";
    let wire_b_input = "L1005,D207,R487,U831,R81,U507,R701,D855,R978,U790,R856,U517,R693,D726,L857,D442,L13,U441,R184,D42,R27,D773,R797,D242,L689,D958,R981,D279,L635,D881,L907,U716,L90,U142,R618,D188,L725,U329,R717,D857,L583,U851,L140,U903,R363,U226,L413,U240,R772,U523,L860,U596,L861,D198,L44,U956,R862,U683,L542,U581,L346,U376,L568,D488,L254,D565,R480,D418,L567,U73,R397,U265,R632,U87,R803,D85,L100,D12,L989,U886,R279,U507,R274,U17,L36,U309,L189,D145,R50,U408,L451,D37,R930,D566,R96,U673,L302,U859,R814,U478,R218,U494,R177,D85,L376,U545,L106,U551,L469,U333,R685,U625,L933,U99,R817,D473,R412,D203,R912,U460,L527,D730,L193,U894,L256,D209,L868,D942,L8,U532,L270,U147,R919,U899,R256,U124,R204,D199,L170,D844,R974,U16,R722,U12,L470,D51,R821,U730,L498,U311,R587,D570,R981,D917,R440,D485,R179,U874,R26,D310,R302,U260,R446,D241,R694,D138,L400,D852,L194,U598,R73,U387,R660,D597,L803,D571,L956,D89,L394,U564,L287,U668,L9,D103,R152,D318,L215,U460,L870,U997,L595,D479,R262,U531,R609,U50,L165,U704,L826,D527,L901,D857,L914,U623,R432,D988,R562,D301,L277,U274,R39,D177,L827,U944,R64,U560,R801,D83,R388,U978,R387,U435,L759,U200,L760,U403,L218,D399,L178,U700,L75,U749,R85,U368,R538,U3,L172,D634,R518,D435,L542,U347,L745,U353,L178,D133,L475,U459,L522,U354,R184,U339,R845,D145,L44,U61,L603,U256,R534,U558,L998,D36,R42,U379,R813,D412,R878,D370,R629,U883,L490,D674,L863,U506,L961,D882,R436,D984,L229,D78,L779,D117,L674,U850,L494,D205,L988,D202,L368,U955,L662,U647,R774,D575,L753,D294,R131,U318,R873,U114,L30";
    // let wire_a_input = "R8,U5,L5,D3";
    // let wire_b_input = "U7,R6,D4,L4";
    // let wire_a_input = "R75,D30,R83,U83,L12,D49,R71,U7,L72";
    // let wire_b_input = "U62,R66,U55,R34,D71,R55,D58,R83";
    // let wire_a_input = "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51";
    // let wire_b_input = "U98,R91,D20,R16,D67,R40,U7,R15,U6,R7";
    let wire_a = process_input(wire_a_input);
    let wire_b = process_input(wire_b_input);
    let list_of_intersections: Vec<Coord> =
        wire_a.iter()
        .zip(wire_a.iter().skip(1))
        .map::<Vec<Coord>,_>(|(a0,a1)|{
            wire_b.iter()
                .zip(wire_b.iter().skip(1))
                .filter_map(|(b0,b1)|{
                    find_intersection(a0,a1,b0,b1)
                }).collect()
        }).flatten().collect();
    let mut distances_to_intersections:HashMap<Coord,i32> = HashMap::new();
    wire_a.iter().zip(wire_a.iter().skip(1)).fold(0, |dist_to_a0, (a0, a1) | {
        for intersection in &list_of_intersections {
            if crosses_point(a0,a1,intersection) {
                let new_dist = distances_to_intersections.entry(*intersection).or_insert(0);
                if *new_dist == 0 {
                    *new_dist = dist_to_a0 + (intersection.x-a0.x).abs() + (intersection.y-a0.y).abs();
                };
            };
        };
        dist_to_a0 + (a1.x-a0.x).abs() + (a1.y-a0.y).abs() // a1 will be next a0 (by construction)
    });
    wire_b.iter().zip(wire_b.iter().skip(1)).fold(0, |dist_to_a0, (a0, a1) | {
        for intersection in &list_of_intersections {
            if crosses_point(a0,a1,intersection) {
                let new_dist = distances_to_intersections.entry(*intersection).or_insert(0);
                if *new_dist == 0 {
                    panic!("b intersects, but a never did??");
                } else {
                    *new_dist += dist_to_a0 + (intersection.x-a0.x).abs() + (intersection.y-a0.y).abs();
                };
            };
        };
        dist_to_a0 + (a1.x-a0.x).abs() + (a1.y-a0.y).abs() // a1 will be next a0 (by construction)
    });
    // println!("Intersections:\n{:?}", list_of_intersections);
    let nearest_intersection = list_of_intersections.iter().map(|c|{c.x.abs()+c.y.abs()}).min().unwrap();
    let shortest_path = distances_to_intersections.iter().map(|(key,val)|{val}).min().unwrap();
    println!("The shortest a/b total path is {}", shortest_path);
    Ok(())
}
#[derive(Clone,Copy,Debug,Hash,PartialEq,Eq)]
struct Coord {
    x: i32,
    y: i32,
}
fn crosses_point(a: &Coord, b: &Coord, point: &Coord) -> bool {
    let is_vertical = a.x == b.x;
    if is_vertical {
        let (top,bottom) = if a.y > b.y {(a.y, b.y)} else {(b.y,a.y)};
        point.x == a.x && top >= point.y && point.y >= bottom
    } else {
        let (right,left) = if a.x > b.x {(a.x, b.x)} else {(b.x,a.x)};
        point.y == a.y && right >= point.x && point.x >= left
    }
}
fn find_intersection(a0: &Coord, a1: &Coord, b0: &Coord, b1: &Coord) -> Option<Coord> {
    let a_is_vertical = a0.x == a1.x;
    let b_is_vertical = b0.x == b1.x;
    let (intersect,top,bottom,right,left) = match (a_is_vertical, b_is_vertical) {
        (true,true)|(false,false) => return None,
        (true,false) => {
            let (t,b) = normalize(a0,a1);
            let (r,l) = normalize(b0,b1);
            let i = Coord {x: t.x, y: r.y};
            (i,t,b,r,l)
        },
        (false,true) => {
            let (t,b) = normalize(b0,b1);
            let (r,l) = normalize(a0,a1);
            let i = Coord {x: t.x, y: r.y};
            (i,t,b,r,l)
        },
    };
    if intersect.y < top.y &&
       intersect.y > bottom.y &&
       intersect.x < right.x &&
       intersect.x > left.x
    {
        Some(intersect)
    } else {
        None
    }
}
fn normalize<'a>(a: &'a Coord, b: &'a Coord) -> (&'a Coord, &'a Coord) {
    if a.x > b.x || a.y > b.y {
        (a,b)
    } else {
        (b,a)
    }
}
fn process_input(input: &str) -> Vec<Coord> {
    let wires: Vec<(Dir, i32)> = input.split(',').map(|s|{
        let dir = s.chars().nth(0).unwrap().try_into().unwrap();
        let dist = s[1..].parse().unwrap();
        (dir,dist)
    }).collect();
    let starting_vec = vec!(Coord {x: 0, y: 0});
    let wire_vertices: Vec<Coord> = wires.into_iter().fold(starting_vec, |mut v,(dir,dist)| {
        let last = v.pop().unwrap();
        let x = last.x;
        let y = last.y;
        v.push(last);
        v.push(match dir {
            Up => Coord {x, y: y+dist},
            Down => Coord {x, y: y-dist},
            Left => Coord {x: x-dist, y},
            Right => Coord {x: x+dist, y},
        });
        v
    });
    wire_vertices
}
use Dir::*;
#[derive(Debug)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}
impl std::convert::TryFrom<char> for Dir {
    type Error = Error;
    fn try_from(ch: char) -> Result<Self, Self::Error> {
        match ch {
            'U' => Ok(Up),
            'D' => Ok(Down),
            'L' => Ok(Left),
            'R' => Ok(Right),
            bad => Err(Error::InputParseError { unexpected: bad.to_string() }),
        }
    }
}
