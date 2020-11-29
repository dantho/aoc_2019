/// https://adventofcode.com/2019/day/22
extern crate regex;
use regex::Regex;
use std::collections::HashSet;
use Command::*;

const DECK_SIZE: usize = 119_315_717_514_047;
const SHUFFLE_REPEAT: usize = 101_741_582_076_661;

// const DECK_SIZE:u128 = 119_315_717_514_047;
fn main() -> Result<(), Error> {
    const CARD: usize = 2020;
    let mut pos_of_card = CARD; // Starting location of card '2019'
    let commands = command_parse(INPUT);
    //for c in &commands { println!("{:?}", *c); }
    let mut detect_repeats = HashSet::new();
    for _ in 0..SHUFFLE_REPEAT {
        for (ndx,cmd) in commands.iter().enumerate() {
            if !detect_repeats.insert((pos_of_card, ndx)) {
                println!("Repeat Found at {} at command index {}", pos_of_card, ndx);
            };
            pos_of_card = match cmd {
                Cut(n) => (pos_of_card + DECK_SIZE - n) % DECK_SIZE,
                Increment(n) => pos_of_card * n % DECK_SIZE,
                NewStack => DECK_SIZE - pos_of_card - 1,
            }
        }
    }
    println!("Card {} located at {}", CARD, pos_of_card);
    Ok(())
}

#[derive(Debug)]
enum Error {
    _NotImplemented,
    // MapAssertFail {msg: String},
}

#[derive(Debug)]
enum Command {
    NewStack,
    Increment(usize),
    Cut(usize),
}

fn command_parse(input: &str) -> Vec<Command> {
    let set = Regex::new(
        r"deal with increment (?P<Increment>\d+)|cut (?P<Cut>-?\d+)|deal into new stack(?P<NewStack>)",
    ).unwrap();
    set.captures_iter(input)
        .map(|cap| {
            if let Some(cut) = cap.name("Cut") {
                Cut(match cut.as_str().parse::<isize>().unwrap() {
                    n if n >= 0 => n,
                    n => DECK_SIZE as isize + n,
                } as usize)
            } else if let Some(incr) = cap.name("Increment") {
                Increment((incr.as_str()).parse::<usize>().unwrap())
            } else if let Some(_new_stack) = cap.name("NewStack") {
                NewStack
            } else {
                // A capture we don't understand??  (Didn't parse, above.)
                // This will ONLY detect matched but not handled content
                // Any unmatched content in the string is ignored
                panic!(format!("Unknown: {}", &cap[0]));
            }
        })
        .collect()
}

const _EX1: &'static str = "
deal with increment 7
";
const _EX2: &'static str = "
cut 6
deal with increment 7
deal into new stack
";
const _EX3: &'static str = "
deal with increment 7
deal with increment 9
cut -2
";
const _EX4: &'static str = "
deal into new stack
cut -2
deal with increment 7
cut 8
cut -4
deal with increment 7
cut 3
deal with increment 9
deal with increment 3
cut -1
";
const INPUT: &'static str = "
cut 2257
deal with increment 18
cut -7620
deal with increment 13
cut 2616
deal into new stack
cut -3891
deal with increment 14
cut 2441
deal with increment 25
deal into new stack
cut -5543
deal with increment 70
cut 3718
deal with increment 26
cut -3987
deal with increment 64
cut -9087
deal with increment 54
cut -6062
deal with increment 12
cut 409
deal with increment 65
cut 9350
deal with increment 67
cut -194
deal into new stack
cut -5895
deal with increment 8
cut -9651
deal into new stack
cut -5859
deal into new stack
cut -5137
deal with increment 64
deal into new stack
deal with increment 51
cut -864
deal with increment 59
deal into new stack
deal with increment 8
deal into new stack
deal with increment 59
cut -2931
deal into new stack
deal with increment 68
cut 9670
deal with increment 3
cut 5096
deal with increment 45
cut -984
deal with increment 38
cut -9911
deal with increment 30
cut -4410
deal with increment 30
cut 3957
deal with increment 42
cut 1160
deal into new stack
deal with increment 16
cut 2753
deal with increment 21
cut 1089
deal with increment 12
cut -3818
deal with increment 11
cut -8579
deal with increment 22
deal into new stack
cut -2802
deal with increment 36
cut 7733
deal with increment 46
cut 8672
deal with increment 30
cut 7161
deal into new stack
deal with increment 11
cut -288
deal with increment 46
deal into new stack
cut 4565
deal with increment 4
cut -5330
deal with increment 41
deal into new stack
cut 6908
deal with increment 14
cut -6762
deal with increment 46
cut 3041
deal with increment 56
cut 1723
deal with increment 50
deal into new stack
deal with increment 52
cut -9189
deal with increment 58
deal into new stack
";

// Used for testing:
// fn _create_deck_and_shuffle(input: &'static str) -> Vec<u128> {
//     let deck: Vec<u128> = (0..DECK_SIZE).collect();
//     shuffle(&deck, input)
// }
//
// #[test]
// fn test_increment_n() {
//     assert_eq!(DECK_SIZE, 10);
//     let deck: Vec<u128> = (0..DECK_SIZE).collect();
//     let deck = increment_m(&deck, 3);
//     assert_eq!(deck, vec![0,7,4,1,8,5,2,9,6,3]);
//     let deck: Vec<u128> = (0..DECK_SIZE).collect();
//     let deck = increment_m(&deck, 7);
//     assert_eq!(deck, vec![0,3,6,9,2,5,8,1,4,7]);
// }
// #[test]
// fn test_cut_n() {
//     assert_eq!(DECK_SIZE, 10);
//     let deck: Vec<u128> = (0..DECK_SIZE).collect();
//     let deck = cut_n(&deck, 3);
//     assert_eq!(deck, vec![3,4,5,6,7,8,9,0,1,2]);
//     let deck: Vec<u128> = (0..DECK_SIZE).collect();
//     let deck = cut_n(&deck, -2);
//     assert_eq!(deck, vec![8,9,0,1,2,3,4,5,6,7]);
// }
// #[test]
// fn test_new_stack() {
//     assert_eq!(DECK_SIZE, 10);
//     let deck: Vec<u128> = (0..DECK_SIZE).collect();
//     let deck = new_stack(&deck);
//     assert_eq!(deck, vec![9,8,7,6,5,4,3,2,1,0]);
//     let deck = new_stack(&deck);
//     assert_eq!(deck, vec![0,1,2,3,4,5,6,7,8,9]);
// }

// #[test]
// fn test_big_new_stack() {
//     assert_eq!(DECK_SIZE, 10_007);
//     let deck: Vec<u128> = (0..DECK_SIZE).collect();
//     let deck: Vec<u128> = new_stack(&deck).into_iter().take(10).collect();
//     assert_eq!(deck, vec![10_006,10_005,10_004,10_003,10_002,10_001,10_000,9_999,9_998,9_997]);
// }
// #[test]
// fn test_big_cut_n() {
//     assert_eq!(DECK_SIZE, 10_007);
//     let size = DECK_SIZE;
//     let deck: Vec<u128> = (0..DECK_SIZE).collect();
//     let deck_start: Vec<u128> = cut_n(&deck, 3).iter().take(5).map(|c|{*c}).collect();
//     assert_eq!(deck_start, vec![3,4,5,6,7]);
//     let deck_end: Vec<u128> = cut_n(&deck, 3).into_iter().skip(size as usize-5).collect();
//     assert_eq!(deck_end, vec![10_005,10_006,0,1,2]);
//     // negative cut
//     let size = DECK_SIZE;
//     let deck: Vec<u128> = (0..DECK_SIZE).collect();
//     let deck_start: Vec<u128> = cut_n(&deck, -3).iter().take(5).map(|c|{*c}).collect();
//     assert_eq!(deck_start, vec![10_004,10_005,10_006,0,1]);
//     let deck_end: Vec<u128> = cut_n(&deck, -3).into_iter().skip(size as usize-5).collect();
//     assert_eq!(deck_end, vec![9_999,10_000,10_001,10_002,10_003]);
// }
// #[test]
// fn test_big_increment_n() {
//     assert_eq!(DECK_SIZE, 10_007);
//     let deck: Vec<u128> = (0..DECK_SIZE).collect();
//     let deck: Vec<u128> = increment_m(&deck, 2).into_iter().take(10).collect();
//     assert_eq!(deck, vec![0,5004,1,5005,2,5006,3,5007,4,5008]);
//     let deck: Vec<u128> = (0..DECK_SIZE).collect();
//     let deck: Vec<u128> = increment_m(&deck, 3).into_iter().take(10).collect();
//     assert_eq!(deck, vec![0,3336,6672,1,3337,6673,2,3338,6674,3]);
//     let deck: Vec<u128> = (0..DECK_SIZE).collect();
//     let deck: Vec<u128> = increment_m(&deck, 9).into_iter().take(10).collect();
//     assert_eq!(deck, vec![0,1112,2224,3336,4448,5560,6672,7784,8896,1]);
//     let deck: Vec<u128> = (0..DECK_SIZE).collect();
//     let deck: Vec<u128> = increment_m(&deck, 10_006).into_iter().take(10).collect();
//     assert_eq!(deck, vec![0,10_006,10_005,10_004,10_003,10_002,10_001,10_000,9_999,9_998,]);
// }

// #[test]
// fn test_big_shuffle() {
//     assert_eq!(DECK_SIZE, 10_007);
//     let deck0: Vec<_> = (0..DECK_SIZE).collect();
//     let deck1: Vec<_> = increment_m(&deck0, 2);
//     let deck2: Vec<_> = cut_n(&deck1, 9);
//     let _deck3: Vec<_> = new_stack(&deck2);
//     let deck_shuffled: Vec<_> = shuffle(&deck0,
//         &format!("deal with increment {}\ncut {}\ndeal into new stack\n",
//         2, 9));
//     assert_eq!(deck1.iter().take(10).cloned().collect::<Vec<_>>(),
//                 vec![0,5004,1,5005,2,5006,3,5007,4,5008]);
//                 let deck2: Vec<_> = cut_n(&deck1, 9);
//                 let deck3: Vec<_> = new_stack(&deck2);
//                 assert_eq!(deck2.iter().take(10).cloned().collect::<Vec<_>>(),
//                 vec![5008, 5, 5009, 6, 5010, 7, 5011, 8, 5012, 9]);
//     assert_eq!(deck3.iter().take(10).cloned().collect::<Vec<_>>(),
//                 vec![4, 5007, 3, 5006, 2, 5005, 1, 5004, 0, 5003]);
//     assert_eq!(deck_shuffled.iter().take(10).cloned().collect::<Vec<_>>(),
//                        _deck3.iter().take(10).cloned().collect::<Vec<_>>()
//               );
// }

// #[test]
// fn test_part1() {
//     assert_eq!(DECK_SIZE, 10_007);
//     let deck = _create_deck_and_shuffle(INPUT);
//     let ans = deck.iter().enumerate().fold(None,|pass_through,(n,card)|{if *card == 2019 {Some(n)} else {pass_through}}).unwrap();
//     assert_eq!(ans, 4703);
// }

// #[test]
// fn test_ex1() {
//     if DECK_SIZE == 10 {
//         assert_eq!(_create_deck_and_shuffle(_EX1), vec![0, 3, 6, 9, 2, 5, 8, 1, 4, 7]);
//     } else if DECK_SIZE == 10_007 {
//         assert_eq!(_create_deck_and_shuffle(_EX1).iter().take(10).cloned().collect::<Vec<_>>(),
//         vec![0, 7148, 4289, 1430, 8578, 5719, 2860, 1, 7149, 4290]);
//     } else {
//         assert!(false, "DECK_SIZE not 10 or 10_007")
//     }
// }
// #[test]
// fn test_ex2() {
//     if DECK_SIZE == 10 {
//         assert_eq!(_create_deck_and_shuffle(_EX2), vec![3, 0, 7, 4, 1, 8, 5, 2, 9, 6]);
//     } else if DECK_SIZE == 10_007 {
//         assert_eq!(_create_deck_and_shuffle(_EX2).iter().take(10).cloned().collect::<Vec<_>>(),
//         vec![2865, 5724, 8583, 1435, 4294, 7153, 5, 2864, 5723, 8582]);
//     } else {
//         assert!(false, "DECK_SIZE not 10 or 10_007")
//     }
// }
// #[test]
// fn test_ex3() {
//     if DECK_SIZE == 10 {
//         assert_eq!(_create_deck_and_shuffle(_EX3), vec![6, 3, 0, 7, 4, 1, 8, 5, 2, 9]);
//     } else if DECK_SIZE == 10_007 {
//         assert_eq!(_create_deck_and_shuffle(_EX3).iter().take(10).cloned().collect::<Vec<_>>(),
//         vec![3971, 6989, 0, 3018, 6036, 9054, 2065, 5083, 8101, 1112]);
//     } else {
//         assert!(false, "DECK_SIZE not 10 or 10_007")
//     }
// }
// #[test]
// fn test_ex4() {
//     if DECK_SIZE == 10 {
//         assert_eq!(_create_deck_and_shuffle(_EX4), vec![9, 2, 5, 8, 1, 4, 7, 0, 3, 6]);
//     } else if DECK_SIZE == 10_007 {
//         assert_eq!(_create_deck_and_shuffle(_EX4).iter().take(10).cloned().collect::<Vec<_>>(),
//         vec![2799, 6944, 1082, 5227, 9372, 3510, 7655, 1793, 5938, 76]);
//     } else {
//         assert!(false, "DECK_SIZE not 10 or 10_007")
//     }
// }
