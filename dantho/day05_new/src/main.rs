/// Day03 code stolen from https://github.com/kodsnack/advent_of_code_2019/blob/master/tomasskare-rust/day2/src/main.rs
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
fn intcode_run(v: &mut Vec<isize>, machine_input: Option<isize>, machine_output: &mut Option<isize>){
    let mut i: usize = 0;
    loop {
        let mode = v[i] / 100;
        let op = v[i] - mode * 100;
        let m1 = mode - mode / 10 * 10;  let mode = mode / 10;
        let m2 = mode - mode / 10 * 10;  let mode = mode / 10;
        let m3 = mode - mode / 10 * 10;  let mode = mode / 10;
        assert_eq!(mode, 0);
        match op {
            1 => {
                let p1 = v[i + 1];
                let p2 = v[i + 2];
                let p3 = v[i + 3];
                let v1 = if m1 == 0 {v[p1 as usize]} else {p1};
                let v2 = if m2 == 0 {v[p2 as usize]} else {p2};
                assert_eq!(m3, 0);
                v[p3 as usize] = v1 + v2;
                i += 4;
            }
            2 => {
                let p1 = v[i + 1];
                let p2 = v[i + 2];
                let p3 = v[i + 3];
                let v1 = if m1 == 0 {v[p1 as usize]} else {p1};
                let v2 = if m2 == 0 {v[p2 as usize]} else {p2};
                assert_eq!(m3, 0);
                v[p3 as usize] = v1 * v2;
                i += 4;
            }
            3 => {
                let p1 = v[i + 1];
                assert_eq!(m1, 0);
                v[p1 as usize] = match machine_input {
                    Some(v) => v,
                    None => panic!("Input expected. Found None."),
                };
                i += 2;
            }
            4 => {
                let p1 = v[i + 1];
                let v1 = if m1 == 0 {v[p1 as usize]} else {p1};
                match machine_output {
                    Some(_) => *machine_output = Some(v1),
                    None => panic!("Machine output not provided."),
                }
                i += 2;
            }
            5 => {
                let p1 = v[i + 1];
                let p2 = v[i + 2];
                let v1 = if m1 == 0 {v[p1 as usize]} else {p1};
                let v2 = if m2 == 0 {v[p2 as usize]} else {p2};
                if v1 != 0 {
                    i = v2 as usize
                } else {
                    i += 3
                };
            }
            6 => {
                let p1 = v[i + 1];
                let p2 = v[i + 2];
                let v1 = if m1 == 0 {v[p1 as usize]} else {p1};
                let v2 = if m2 == 0 {v[p2 as usize]} else {p2};
                if v1 == 0 {
                    i = v2 as usize
                } else {
                    i += 3
                };
            }
            7 => {
                let p1 = v[i + 1];
                let p2 = v[i + 2];
                let p3 = v[i + 3];
                let v1 = if m1 == 0 {v[p1 as usize]} else {p1};
                let v2 = if m2 == 0 {v[p2 as usize]} else {p2};
                assert_eq!(m3, 0);
                v[p3 as usize] = if v1 < v2 {1} else {0};
                i += 4;
            }
            8 => {
                let p1 = v[i + 1];
                let p2 = v[i + 2];
                let p3 = v[i + 3];
                let v1 = if m1 == 0 {v[p1 as usize]} else {p1};
                let v2 = if m2 == 0 {v[p2 as usize]} else {p2};
                assert_eq!(m3, 0);
                v[p3 as usize] = if v1 == v2 {1} else {0};
                i += 4;
            }
            99 => break,
            x => panic!("Unexpected opcode {}", x),
        }
    }
}
// fn get_input() -> isize {
//     println!("Input an integer:");
//     let mut input_text = String::new();
//     io::stdin()
//         .read_line(&mut input_text)
//         .expect("failed to read from stdin");

//     let trimmed = input_text.trim();
//     match trimmed.parse::<isize>() {
//         Ok(i) => i,
//         Err(..) => {println!("this was not an integer: {}", trimmed); 0},
//     }
// }
// fn set_output(v: isize) {
//     println!("Output: {}", v);
// }
fn main() {
    let filename = "day05_input.txt";
    let fd = File::open(filename).expect(&format!("Failure opening {}", filename));
    let buf = BufReader::new(fd);
    let mut v_orig = Vec::new();
    buf.lines().for_each(|line| {
        line.unwrap().split(',').for_each(|numstr| {
            let num = numstr.parse::<isize>().unwrap();
            v_orig.push(num);
        });
    });
    let mut v = v_orig.clone();
    let mut out: Option<isize> = Some(0);
    intcode_run(&mut v, Some(5), &mut out);
    match out {
        Some(v) => println!("Output: {}", v),
        None => println!("Output: None"),
    }
}
