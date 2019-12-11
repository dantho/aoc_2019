/// Day03 code stolen from https://github.com/kodsnack/advent_of_code_2019/blob/master/tomasskare-rust/day2/src/main.rs
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use futures::prelude::*;
use futures::channel::mpsc::{channel,Sender,Receiver};
use futures::executor::block_on;
use futures::join;

async fn intcode_run(mut v: Vec<isize>, mut input: Receiver<isize>, mut output: Sender<isize>) -> Receiver<isize> {
    let mut pc: usize = 0;
    loop {
        let mode = v[pc] / 100;
        let op = v[pc] - mode * 100;
        let m1 = mode - mode / 10 * 10;  let mode = mode / 10;
        let m2 = mode - mode / 10 * 10;  let mode = mode / 10;
        let m3 = mode - mode / 10 * 10;  let mode = mode / 10;
        assert_eq!(mode, 0);
        match op {
            1 => {
                let p1 = v[pc + 1];
                let p2 = v[pc + 2];
                let p3 = v[pc + 3];
                let v1 = if m1 == 0 {v[p1 as usize]} else {p1};
                let v2 = if m2 == 0 {v[p2 as usize]} else {p2};
                assert_eq!(m3, 0);
                v[p3 as usize] = v1 + v2;
                pc += 4;
            }
            2 => {
                let p1 = v[pc + 1];
                let p2 = v[pc + 2];
                let p3 = v[pc + 3];
                let v1 = if m1 == 0 {v[p1 as usize]} else {p1};
                let v2 = if m2 == 0 {v[p2 as usize]} else {p2};
                assert_eq!(m3, 0);
                v[p3 as usize] = v1 * v2;
                pc += 4;
            }
            3 => {
                let p1 = v[pc + 1];
                assert_eq!(m1, 0);
                v[p1 as usize] = match input.next().await {
                    Some(v) => v,
                    None => panic!("Expecting input, but stream has terminated."),
                };
                pc += 2;
            }
            4 => {
                let p1 = v[pc + 1];
                let v1 = if m1 == 0 {v[p1 as usize]} else {p1};
                if let Err(_) = output.send(v1).await {
                    panic!("Problem sending output data, has receiver been dropped?")
                };
                pc += 2;
            }
            5 => {
                let p1 = v[pc + 1];
                let p2 = v[pc + 2];
                let v1 = if m1 == 0 {v[p1 as usize]} else {p1};
                let v2 = if m2 == 0 {v[p2 as usize]} else {p2};
                if v1 != 0 {
                    pc = v2 as usize
                } else {
                    pc += 3
                };
            }
            6 => {
                let p1 = v[pc + 1];
                let p2 = v[pc + 2];
                let v1 = if m1 == 0 {v[p1 as usize]} else {p1};
                let v2 = if m2 == 0 {v[p2 as usize]} else {p2};
                if v1 == 0 {
                    pc = v2 as usize
                } else {
                    pc += 3
                };
            }
            7 => {
                let p1 = v[pc + 1];
                let p2 = v[pc + 2];
                let p3 = v[pc + 3];
                let v1 = if m1 == 0 {v[p1 as usize]} else {p1};
                let v2 = if m2 == 0 {v[p2 as usize]} else {p2};
                assert_eq!(m3, 0);
                v[p3 as usize] = if v1 < v2 {1} else {0};
                pc += 4;
            }
            8 => {
                let p1 = v[pc + 1];
                let p2 = v[pc + 2];
                let p3 = v[pc + 3];
                let v1 = if m1 == 0 {v[p1 as usize]} else {p1};
                let v2 = if m2 == 0 {v[p2 as usize]} else {p2};
                assert_eq!(m3, 0);
                v[p3 as usize] = if v1 == v2 {1} else {0};
                pc += 4;
            }
            99 => break,
            x => panic!("Unexpected opcode {}", x),
        }
    }
    input // Drop the input Receiver, this allow downstream fetching of values we are not going to process ('cause we're done)
}
async fn amplifier_x5(prog: Vec<isize>, phase_x5: Vec<isize>, initial_input: isize) -> isize {
    let mut amplifier_inputs: Vec<Receiver<isize>> = Vec::new();
    let mut amplifier_outputs: Vec<Sender<isize>> = Vec::new();
    assert_eq!(phase_x5.len(), 5);
    // Create input/output connections ("wires")
    for phase in phase_x5.into_iter() {
        const BUFFER_SIZE: usize = 10;
        let (mut output_tx, input_rx) = channel::<isize>(BUFFER_SIZE);
        output_tx.send(phase).await.unwrap();
        // add initial_input to first amplifier ONLY
        if 0 == amplifier_inputs.len()  {
            output_tx.send(initial_input).await.unwrap();
        };
        amplifier_inputs.push(input_rx);
        amplifier_outputs.push(output_tx);
    }
    // Shift output order such that inputs come from prior amplifier, (or LAST amplifier for first amplifier input)
    // In other words, move first output to last position and shift all the others down one.  Done.
    amplifier_outputs.reverse();
    let tmp = amplifier_outputs.pop().unwrap();
    amplifier_outputs.reverse();
    amplifier_outputs.push(tmp);
    // Create/connect all amplifiers to produce a Vec of futures
    let amplifier_future_results: Vec<_> =
        amplifier_inputs.into_iter()
        .zip(amplifier_outputs.into_iter())
        .map(|(input_stream,output_stream)| {    
            intcode_run(prog.clone(), input_stream, output_stream)
        }).collect();
    assert_eq!(amplifier_future_results.len(), 5);
    let mut a = amplifier_future_results;
    let mut result_streams = join!(
        a.pop().expect("Amplifier E result stream not found."),
        a.pop().expect("Amplifier D result stream not found."),
        a.pop().expect("Amplifier C result stream not found."),
        a.pop().expect("Amplifier B result stream not found."),
        a.pop().expect("Amplifier A result stream not found."),
    );
    // Based on dataflow, and assuming each amplifier terminates cleanly in order,
    // the last channel should have one final result "in the pipe".
    // BUT, the last amplifier's output receiver is connect to the first amplifier's input
    // SO, we need the first amplifier's returned receiver.
    (result_streams.4).next().await.expect("Amplifier Output channel EMPTY.")
}
fn main() {
    let filename = "day07_input.txt";
    // let filename = "day07_example1.txt";
    // let filename = "day07_example2.txt";
    // let filename = "day07_example3.txt";
    // let filename = "day07_example4.txt";
    // let filename = "day07_example5.txt";
    let fd = File::open(filename).expect(&format!("Failure opening {}", filename));
    let buf = BufReader::new(fd);
    let mut prog_orig = Vec::new();
    buf.lines().for_each(|line| {
        line.unwrap().split(',').for_each(|numstr| {
            let num = numstr.parse::<isize>().unwrap();
            prog_orig.push(num);
        });
    });
    let initial_input = 0;
    let mut max_out = std::isize::MIN;
    let phases0 = vec![4,3,2,1,0];
    for ph0 in 0..5 {
        let mut phases1 = phases0.clone();
        phases1.remove(ph0);
        for ph1 in 0..4 {
            let mut phases2 = phases1.clone();
            phases2.remove(ph1);
            for ph2 in 0..3 {
                let mut phases3 = phases2.clone();
                phases3.remove(ph2);
                for ph3 in 0..2 {
                    let mut phases4 = phases3.clone();
                    phases4.remove(ph3);
                    for ph4 in 0..1 {
                        let phase = vec![phases4[ph4],phases3[ph3],phases2[ph2],phases1[ph1],phases0[ph0]];
                        let phase_display = format!("{:?}", phase);
                        let out = block_on(amplifier_x5(prog_orig.clone(), phase, initial_input));
                        if out > max_out {
                            println!("Part 1) Max thruster signal {} (from phase setting sequence {:?})", out, phase_display);
                            max_out = out;
                        }
                    }
                }
            }
        }
    }
}
