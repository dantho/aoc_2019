/// https://adventofcode.com/2019/day/8
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::collections::HashMap;

fn main() {
    let (filename, cols, rows) = ("input.txt", 25, 6);
    // let (filename, cols, rows) = ("day08_example.txt", 3, 2);
    // let filename = "day08_example1.txt"; 
    let fd = File::open(filename).expect(&format!("Failure opening {}", filename));
    let buf = BufReader::new(fd);
    let input = buf.lines().next().expect(&format!("Input error reading file '{}'", filename)).ok().unwrap();
    let mut digits = input.chars().map(|d| {
        let num = d.to_string().parse::<u8>();
        num.ok().expect(&format!("Illegal char in input: '{}'", d))
    });
    let mut all_layers: Vec<Vec<Vec<u8>>> = Vec::new();
    'layers: for layer in 1.. {
        let mut new_layer: Vec<Vec<u8>> = Vec::new();
        for row in 0..rows {
            let mut new_row: Vec<u8> = Vec::new();
            for col in 0..cols {
                match digits.next() {
                    Some(d) => new_row.push(d),
                    None => {
                        if col == 0 && row == 0 {
                            break 'layers;
                        } else {
                            panic!("Input stopped in middle of ({} x {}) layer #{}.", cols, rows, layer);
                        }
                    }
                };
            }
            new_layer.push(new_row);
        }
        all_layers.push(new_layer);
    }
    println!("Found {} layers, each is {} cols by {} rows.",
        all_layers.len(), all_layers[0][0].len(), all_layers[0].len());

    // find count of 1's and 2's in layer with fewest zeros
    let (_layer_num, _cnt_0s, cnt_1s, cnt_2s) = all_layers.iter().fold((0,std::u8::MAX,0,0), |(layer_num, cnt_0s, cnt_1s, cnt_2s), layer| {
        let mut digihash = HashMap::new();
        layer.iter().flatten().for_each(|d| {
            *digihash.entry(d).or_default() += 1;
        });
        println!("HashMap: {:?}", digihash);
        let zeros = if let Some(cnt) = digihash.get(&0) {*cnt} else {0};
        // Decide what to return (fold), old layer or this new one
        let layer_num = layer_num + 1;
        if zeros < cnt_0s {
            let ones = if let Some(cnt) = digihash.get(&1) {*cnt} else {0};
            let twos = if let Some(cnt) = digihash.get(&2) {*cnt} else {0};
            (layer_num, zeros, ones, twos)
        } else {
            (layer_num,cnt_0s,cnt_1s,cnt_2s)
        }
    });
    println!("Part 1 answer is {}.", cnt_1s as i32 * cnt_2s as i32);
}
