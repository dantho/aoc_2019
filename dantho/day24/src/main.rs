const ESC_CLS: &'static str = "\x1B[2J";
const ESC_CURSOR_OFF: &'static str = "\x1B[?25l";

use std::collections::{BTreeMap,HashSet};
use std::io::{stdout,Write};

fn main() {
    // let initial_map = EXAMPLE;
    let initial_map = MY_LIFE;
    let mut lifemap = BTreeMap::new();
    let mut history = HashSet::new();
    for (y,line) in initial_map.lines().filter(|l|{l.len()==7}).enumerate() {
        for (x,ch) in line.chars().enumerate() {
            lifemap.insert((y,x), ch == '#');
        }
    }
    print(ESC_CURSOR_OFF);
    for min in 0.. {
        print(ESC_CLS);
        println!("\nMin: {}", min);
        print_screen(&lifemap);
        if !history.insert(lifemap.clone()) {break;}
        // create new life!
        let new_life = lifemap.values() // above
        .zip(lifemap.values().skip(7))  // center
        .zip(lifemap.values().skip(14)) // below
        .zip(lifemap.values().skip(6))  // left
        .zip(lifemap.values().skip(8))  // right
        .map(|((((above,center),below),left),right)|{(*center,
            if *above {1} else {0} +
            if *below {1} else {0} +
            if *left  {1} else {0} +
            if *right {1} else {0}
        )})
        .map(|(center,neighbors)|{
            if center {
                neighbors == 1
            } else {
                neighbors == 1 || neighbors == 2
            }
        }).collect::<Vec<_>>();
        // replace old data with new (skips are vital for correct placement)
        for (n, (old, new)) in lifemap.values_mut().skip(7).zip(new_life).enumerate() {
            if n % 7 > 0 && n % 7 < 6 {*old = new;}
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    let biodiversity_rating = lifemap.into_iter().filter_map(|((x,y), b)|{
        if y == 0 || y == 6 || x == 0 || x == 6 {None}
        else {Some(if b {1} else {0})}
    }).collect::<Vec<_>>().into_iter().rev().fold(0,|rating,b|{rating*2+b});
    println!("Biodiversity rating is {}", biodiversity_rating);
}
fn print_screen(lifemap: &BTreeMap<(usize,usize),bool>) {
    let mut last_row = 1;
    for ((row,col), is_life) in lifemap {
        // if *row == 0 || *row == 6 {continue;} // do not print border
        // if *col == 0 || *col == 6 {continue;} // do not print border
        if *row != last_row {
            println!("");
        }
        print!("{}", if *is_life {'#'} else {'.'});
        last_row = *row;
    }
    println!("");
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