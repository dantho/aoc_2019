/// https://adventofcode.com/2019/day/16
use std::char::from_digit;

fn main() {

    println!("5978783 Mod 650 is {}", 5978783 % 650);
    println!("0303673 Mod 32 is {}", 0303673 % 32);
    println!("0293510 Mod 32 is {}", 0293510 % 32); // 6 (is less than 8) -- This will be a problem until we implement the edge case
    println!("0308177 Mod 32 is {}", 0308177 % 32);

    // println!("EX5: Sum of 03036732577212944063491565474664 is {}", 0+3+0+3+6+7+3+2+5+7+7+2+1+2+9+4+4+0+6+3+4+9+1+5+6+5+4+7+4+6+6+4);
    println!("EX7: Sum of 03081770884921959731165446850517 is {}", 0+3+0+8+1+7+7+0+8+8+4+9+2+1+9+5+9+7+3+1+1+6+5+4+4+6+8+5+0+5+1+7);
    println!("100 Phase Calc (Part 2): {}", process(EX7, 100));
    let _avoid_unused_warning = [INPUT, EX5, EX6, EX7];
}
// WARNING:  EDGE CASE NOT YET IMPLEMENTED! -- WHERE starting_ndx lands WITHIN 8 DIGITS OF END OF INPUT.  :(
fn process(input: &'static str, phase_cnt: u32) -> String {
    let input_len = input.len();
    let offset: usize = digits(input, 7, 0); // "first seven digits" is message offset
    let starting_ndx = offset % input_len;
    let mut in_digits: Vec<usize> = input.chars().map(|c|c.to_digit(10).unwrap() as usize).collect();
    for _phase in 0..phase_cnt {
        let sum_base_input_digits: usize = in_digits.iter().sum();
        // println!("Base input digits sum to {}", sum_base_input_digits);
        let sum_remaining_input_blocks = sum_base_input_digits * ((input_len*10_000 - offset)/input_len);
        // println!("Sum of remaining ({}) full blocks is {}", (input_len*10_000 - offset)/input_len, sum_remaining_input_blocks);
        let mut rhs_digits: Vec<usize> = Vec::new();
        for dig_ndx in 0..input_len {
            rhs_digits.push((in_digits.iter().skip(dig_ndx).sum::<usize>() + sum_remaining_input_blocks) % 10);
        }
        in_digits = rhs_digits;
        // println!("Phase {}:  {:?}", _phase, in_digits);
    }
    // return 1st 8 digits at starting at "message offset"
    in_digits.iter().skip(starting_ndx).take(8).fold(0, |prev, d|{d + 10*prev}).to_string()
}
fn digits(input: &'static str, size: usize, index: usize) -> usize {
    input.split_at(index).1.split_at(size).0.parse().unwrap()
}
fn digit(input: &'static str, index: usize) -> usize {
    input.split_at(index).1.split_at(1).0.parse().unwrap()
}
const INPUT: &'static str =
"59787832768373756387231168493208357132958685401595722881580547807942982606755215622050260150447434057354351694831693219006743316964757503791265077635087624100920933728566402553345683177887856750286696687049868280429551096246424753455988979991314240464573024671106349865911282028233691096263590173174821612903373057506657412723502892841355947605851392899875273008845072145252173808893257256280602945947694349746967468068181317115464342687490991674021875199960420015509224944411706393854801616653278719131946181597488270591684407220339023716074951397669948364079227701367746309535060821396127254992669346065361442252620041911746738651422249005412940728";
const EX5: &'static str = "03036732577212944063491565474664";
const EX6: &'static str = "02935109699940807407585447034323";
const EX7: &'static str = "03081770884921959731165446850517";

#[test]
fn test_part2_input() {
    assert_eq!(process(INPUT, 100), "TBD".to_string());
}
#[test]
fn test_part2_ex5() {
    assert_eq!(process(EX5, 100), "84462026".to_string());
}
#[test]
fn test_part2_ex6() {
    assert_eq!(process(EX6, 100), "78725270".to_string());
}
#[test]
fn test_part2_ex7() {
    assert_eq!(process(EX7, 100), "53553731".to_string());
}

#[test]
fn test_digits() {
    assert_eq!(digits("123456789", 3, 1), 234);
    assert_eq!(digits("123456789", 7, 0), 1234567);
}
#[test]
fn test_digit() {
    assert_eq!(digit("123456789", 0), 1);
    assert_eq!(digit("123456789", 1), 2);
    assert_eq!(digit("123456789", 7), 8);
    assert_eq!(digit("123456789", 8), 9);
}