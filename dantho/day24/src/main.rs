/// https://adventofcode.com/2019/day/24
const ESC_CLS: &'static str = "\x1B[2J";
const ESC_CURSOR_OFF: &'static str = "\x1B[?25l";

use std::collections::BTreeMap;
use std::io::{stdout,Write};

fn main() {
    // let initial_map = EXAMPLE;
    let initial_map = MY_LIFE;
    let mut lifemaps: Vec<BTreeMap<_,_>> = Vec::new();
    lifemaps.push(BTreeMap::new());
    {
        let lifemap = &mut lifemaps[0];
        for (y,line) in initial_map.lines().filter(|l|{l.len()==7}).enumerate() {
            for (x,ch) in line.chars().enumerate() {
                lifemap.insert((y,x), if ch == '#' {1} else {0});
            }
        }
    }
    print(ESC_CURSOR_OFF);
    for min in 0..=200 {
        print(ESC_CLS);
        println!("\nMin: {}", min);
        print_screen(&lifemaps);
        // create new life!
        let mut blanklayer = lifemaps[0].clone();
        clear_layer(&mut blanklayer);
        lifemaps.insert(0, blanklayer.clone()); // add blank to beginning
        lifemaps.push(blanklayer); // add blank to end
        preprocess(&mut lifemaps);
        reproduce(&mut lifemaps);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
fn digit(v: u32, d: u32) -> u32 {
    if d == 0 {
        0
    } else {
        let v = v/d;
        let digit = v-v/10*10;
        digit
    }
}
fn reproduce(lifemaps: &mut Vec<BTreeMap<(usize,usize),u32>>) {
    for lifemap in lifemaps {
        // map() decodes center tile as digits in order: Marker, Top, Bottom, Left, Right
        // where Marker == 10_000
        let new_life = lifemap.values().map(|v|{if *v >= 10_000 {digit(*v,1000)} else {*v}}) // above
        .zip(lifemap.values().skip(7).map(|v|{if *v >= 10_000 {digit(*v,0)} else {*v}}))  // center
        .zip(lifemap.values().skip(14).map(|v|{if *v >= 10_000 {digit(*v,100)} else {*v}})) // below
        .zip(lifemap.values().skip(6).map(|v|{if *v >= 10_000 {digit(*v,10)} else {*v}}))  // left
        .zip(lifemap.values().skip(8).map(|v|{if *v >= 10_000 {digit(*v,1)} else {*v}}))  // right
        .map(|((((above,center),below),left),right)|{(center,
            above +
            below +
            left  +
            right         )})
        .map(|(center,neighbors)|{
            if center == 1 {
                if neighbors == 1 {1} else {0}
            } else {
                if neighbors == 1 || neighbors == 2 {1} else {0}
            }
        }).collect::<Vec<_>>();
        // replace old data with new (skips are vital for correct placement)
        for (n, (old, new)) in lifemap.values_mut().skip(7).zip(new_life).enumerate() {
            if n % 7 > 0 && n % 7 < 6 {*old = new;}
        }                
    }
}
fn preprocess(lifemaps: &mut Vec<BTreeMap<(usize,usize),u32>>) {
    // outside values sum to center on next layer
    let mut top = 0;
    let mut bottom = 0;
    let mut left = 0;
    let mut right = 0;
    for ndx in 0..lifemaps.len() {
        // encode outside values from prior (inner) layer into this layer's center cell
        // encodes center tile as digits in order: Marker, Top, Bottom, Left, Right
        // where Marker == 10_000
        *lifemaps[ndx].get_mut(&(3,3)).unwrap() = 10_000+top*1000+bottom*100+left*10+right;
        // calculate values for next layer from this layer's perimeter
        top    = lifemaps[ndx].iter().filter_map(|((y,x),v)|{if *y==1&&*x>0&&*x<6 {Some(v)} else {None}}).sum();
        bottom = lifemaps[ndx].iter().filter_map(|((y,x),v)|{if *y==5&&*x>0&&*x<6 {Some(v)} else {None}}).sum();
        left   = lifemaps[ndx].iter().filter_map(|((y,x),v)|{if *x==1&&*y>0&&*y<6 {Some(v)} else {None}}).sum();
        right  = lifemaps[ndx].iter().filter_map(|((y,x),v)|{if *x==5&&*y>0&&*y<6 {Some(v)} else {None}}).sum();
    }
    // inside values (surrounding 3,3) fan-out to outside perimeter on "previous" layer (we're stepping backwards)
    let mut top = 0;
    let mut bottom = 0;
    let mut left = 0;
    let mut right = 0;
    for rev_ndx in 0..lifemaps.len() {
        let ndx = lifemaps.len()-1-rev_ndx;
        for x in 1..=5 {
            *lifemaps[ndx].get_mut(&(0,x)).unwrap() = bottom;
            *lifemaps[ndx].get_mut(&(6,x)).unwrap() = top;
        }
        for y in 1..=5 {
            *lifemaps[ndx].get_mut(&(y,0)).unwrap() = right;
            *lifemaps[ndx].get_mut(&(y,6)).unwrap() = left;
        }
        top    = *lifemaps[ndx].get(&(2,3)).unwrap();
        bottom = *lifemaps[ndx].get(&(4,3)).unwrap();
        left   = *lifemaps[ndx].get(&(3,2)).unwrap();
        right  = *lifemaps[ndx].get(&(3,4)).unwrap();
    }
    println!("Life count is {}", life_count(&lifemaps));
}
fn clear_layer(lifemap: &mut BTreeMap<(usize,usize),u32>) {
    for val in lifemap.values_mut() {
        *val = 0;
    }
}
fn life_count(lifemaps: &Vec<BTreeMap<(usize,usize),u32>>) -> u32 {
    let mut total = 0;
    for lifemap in lifemaps {
        for ((row,col), life_cnt) in lifemap {
            if *row == 0 || *row == 6 {continue;} // do not include borders
            if *col == 0 || *col == 6 {continue;} // do not include borders
            if *col == 3 && *row == 3 {continue;} // do not include center
            assert!(*life_cnt == 0 || *life_cnt == 1);
            total += *life_cnt;
        }
    }
    total
}
fn print_screen(lifemaps: &Vec<BTreeMap<(usize,usize),u32>>) {
    let middle = (lifemaps.len()-1)/2;
    let lifemap = &lifemaps[middle];
    {
    // for lifemap in lifemaps {
        let mut last_row = 1;
        let mut center_value = -1;
        for ((row,col), life_cnt) in lifemap {
            // if *row == 0 || *row == 6 {continue;} // do not print border
            // if *col == 0 || *col == 6 {continue;} // do not print border
            if *row != last_row {
                println!("");
            }
            if (3,3) == (*row,*col) {
                center_value = *life_cnt as isize;
                print!("?");
            } else {
                print!("{}", if *life_cnt > 0 {'#'} else {'.'});
            }
            last_row = *row;
        }
        println!("");
        println!("Center value is {}", center_value);
    }
}
fn print(s: &str) {
    print!("{}",s);
    stdout().flush().unwrap();
}

const EXAMPLE: &'static str = r#" 
.......
.....#.
.#..#..
.#..##.
...#...
.#.....
.......
"#;
const MY_LIFE: &'static str = r#"
.......
..#.##.
..#.#..
.##.#..
.####..
.#.###.
.......
"#;

#[test]
fn test_digit() {
    assert_eq!(digit(1234, 0), 0);
    assert_eq!(digit(1234, 1), 4);
    assert_eq!(digit(1234, 10), 3);
    assert_eq!(digit(1234, 100), 2);
    assert_eq!(digit(1234, 1000), 1);
    assert_eq!(digit(1234, 10_000), 0);
}