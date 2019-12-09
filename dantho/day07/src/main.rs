/// Day03 code stolen from https://github.com/kodsnack/advent_of_code_2019/blob/master/tomasskare-rust/day2/src/main.rs
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
fn intcode_run(v: &mut Vec<isize>, machine_input: Vec<isize>, machine_output: &mut Vec<isize>){
    let mut i: usize = 0;
    let mut input = machine_input.clone();
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
                v[p1 as usize] = match input.pop() {
                    Some(v) => v,
                    None => panic!("Input buffer empty."),
                };
                println!("Input: {}", v[p1 as usize]);
                i += 2;
            }
            4 => {
                let p1 = v[i + 1];
                let v1 = if m1 == 0 {v[p1 as usize]} else {p1};
                machine_output.push(v1);
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
fn amplifier_x5(v: &mut Vec<isize>, phase_x5: Vec<isize>, machine_input: isize, machine_output: &mut isize){
    let mut input = machine_input;
    let mut output = Vec::new();
    assert_eq!(phase_x5.len(), 5);
    for phase in phase_x5.into_iter() {
        let mut prog = v.clone();
        let mut input_vec = Vec::new();
        input_vec.push(input);
        input_vec.push(phase);
        intcode_run(&mut prog, input_vec, &mut output);
        input = output.pop().unwrap();
    }
    *machine_output = input;
}
fn main() {
    let filename = "day07_example1.txt";
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
    let input = 0;
    let mut out: isize = 999999;
    let phase = vec![4,3,2,1,0];
    amplifier_x5(&mut v, phase, input, &mut out);
    println!("Output: {}", out);
}
