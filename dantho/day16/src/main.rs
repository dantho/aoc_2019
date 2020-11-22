/// https://adventofcode.com/2019/day/16
use std::iter::repeat;

const _BASE_REPEAT: usize = 10_000;

fn main() {
    let _avoid_unused_warning = [SIGNAL, EX1, EX2, EX3, EX4, EX5, EX6];
    let _avoid_unused_answers = [
        PART1_ANSWER,
        EX0_ANSWER1,
        EX0_ANSWER2,
        EX0_ANSWER3,
        EX0_ANSWER4,
        EX1_ANSWER,
        EX2_ANSWER,
        EX3_ANSWER,
        EX4_ANSWER,
        EX5_ANSWER,
        EX6_ANSWER,
    ];
    println!("{} {}", part1(EX0, 1), EX0_ANSWER1);
}
trait ToDigits {
    fn to_digits(&self) -> Vec<u8>;
}
impl ToDigits for str {
    fn to_digits(&self) -> Vec<u8> {
        self.chars().map(|c|c.to_digit(10).unwrap() as u8).collect()
    }
}
pub fn part1(signal: &str, phase_cnt: u8) -> u64 {
    let input_digits = signal.to_digits();
    (1..=phase_cnt).fold(input_digits,|phase_input_digits, _|{
        do_phase(&phase_input_digits)
    }).iter().take(8).fold(0,|num,d|{num*10 + *d as u64})
}

pub fn do_phase(input: &Vec<u8>) -> Vec<u8> {
    const BASE_PATTERN: [isize; 4] = [0, 1, 0, -1];
    input.iter()
        .zip(1..)
        .map(|(_,cycle)|{
        let tmp_sum =
            BASE_PATTERN
            .iter()
            .flat_map(|d| repeat(*d).take(cycle))
            .cycle()
            .skip(1)
            .zip(input)
            .map(|(a,b)| a * *b as isize)
            .sum();
        last_digit(tmp_sum)
    }).collect()
}

fn last_digit(x: isize) -> u8 {
    (x % 10).abs() as u8
}

#[test]
fn test_last_digit() {
    assert_eq!(last_digit(21), 1);
    assert_eq!(last_digit(20), 0);
    assert_eq!(last_digit(19), 9);
    assert_eq!(last_digit(11), 1);
    assert_eq!(last_digit(10), 0);
    assert_eq!(last_digit(9), 9);
    assert_eq!(last_digit(2), 2);
    assert_eq!(last_digit(1), 1);
    assert_eq!(last_digit(0), 0);
    assert_eq!(last_digit(-1), 1);
    assert_eq!(last_digit(-2), 2);
    assert_eq!(last_digit(-9), 9);
    assert_eq!(last_digit(-10), 0);
    assert_eq!(last_digit(-11), 1);
    assert_eq!(last_digit(-19), 9);
    assert_eq!(last_digit(-20), 0);
    assert_eq!(last_digit(-21), 1);
}

const SIGNAL: &'static str =
"59787832768373756387231168493208357132958685401595722881580547807942982606755215622050260150447434057354351694831693219006743316964757503791265077635087624100920933728566402553345683177887856750286696687049868280429551096246424753455988979991314240464573024671106349865911282028233691096263590173174821612903373057506657412723502892841355947605851392899875273008845072145252173808893257256280602945947694349746967468068181317115464342687490991674021875199960420015509224944411706393854801616653278719131946181597488270591684407220339023716074951397669948364079227701367746309535060821396127254992669346065361442252620041911746738651422249005412940728";
const PART1_ANSWER: u64 = 42945143;
const EX0: &'static str = "12345678";
const EX0_ANSWER1: u64 = 48226158;
const EX0_ANSWER2: u64 = 34040438;
const EX0_ANSWER3: u64 = 03415518;
const EX0_ANSWER4: u64 = 01029498;
const EX1: &'static str = "80871224585914546619083218645595";
const EX1_ANSWER: u64 = 24176176;
const EX2: &'static str = "19617804207202209144916044189917";
const EX2_ANSWER: u64 = 73745418;
const EX3: &'static str = "69317163492948606335995924319873";
const EX3_ANSWER: u64 = 52432133;
const EX4: &'static str = "03036732577212944063491565474664";
const EX4_ANSWER: u64 = 84462026;
const EX5: &'static str = "02935109699940807407585447034323";
const EX5_ANSWER: u64 = 78725270;
const EX6: &'static str = "03081770884921959731165446850517";
const EX6_ANSWER: u64 = 53553731;

#[test]
fn test_part1() {
    assert_eq!(part1(SIGNAL, 100), PART1_ANSWER);
}
#[test]
fn test_ex0() {
    assert_eq!(part1(EX0, 1), EX0_ANSWER1);
    assert_eq!(part1(EX0, 2), EX0_ANSWER2);
    assert_eq!(part1(EX0, 3), EX0_ANSWER3);
    assert_eq!(part1(EX0, 4), EX0_ANSWER4);
}
#[test]
fn test_ex1() {
    assert_eq!(part1(EX1, 100), EX1_ANSWER);
}
#[test]
fn test_ex2() {
    assert_eq!(part1(EX2, 100), EX2_ANSWER);
}
#[test]
fn test_ex3() {
    assert_eq!(part1(EX3, 100), EX3_ANSWER);
}
// Part2
// #[test]
// fn test_ex4() {
//     assert_eq!(part2(EX4, 100), EX4_ANSWER);
// }
// #[test]
// fn test_ex5() {
//     assert_eq!(part2(EX5, 100), EX5_ANSWER);
// }
// #[test]
// fn test_ex6() {
//     assert_eq!(part2(EX6, 100), EX6_ANSWER);
// }
