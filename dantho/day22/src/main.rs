/// https://adventofcode.com/2019/day/22
use modinverse::modinverse; // for increment_m

const DECK_SIZE:usize = 10;
// const DECK_SIZE:usize = 119_315_717_514_047;

fn main() -> Result<(),Error> {
    // for i in 1..DECK_SIZE {
    //     println!("Inverse Modulo({},{}) is {}", i, DECK_SIZE, modinverse(i as usize,DECK_SIZE as usize).unwrap())
    // }
    let deck = Box::new(0..DECK_SIZE);
    if DECK_SIZE > 25 {
        let i = DECK_SIZE-1;
        let deck2 = increment_m(deck, i);
        // println!("Deck increment_{} starts with {:?}", i, deck2.iter().take(20).collect::<Vec<_>>());
    } else {
        for i in 1..DECK_SIZE {
            let deck2 = increment_m(deck, i);
            // println!("Deck {} is {:?}", i, deck2 );
        }
    }
    Ok(())
}
#[derive(Debug)]
enum Error {
    _NotImplemented,
    // MapAssertFail {msg: String},
}
fn new_stack(cards: Box<dyn DoubleEndedIterator<Item=usize>>) -> Box<dyn DoubleEndedIterator<Item=usize>> {
    Box::new(cards.rev())
}
fn cut_n(cards: Box<dyn DoubleEndedIterator<Item=usize>>, n: isize) -> Box<dyn DoubleEndedIterator<Item=usize>> {
    let n = if n < 0 {DECK_SIZE as isize + n} else {n} as usize;
    let place_at_bottom: Vec<_> = cards.take(n).collect();
    Box::new(cards.chain(place_at_bottom.into_iter()))
}
fn increment_m(cards: Box<dyn DoubleEndedIterator<Item=usize>>, m: usize) -> Box<dyn DoubleEndedIterator<Item=usize>> {
    const MAX_CACHE: usize = 10_007;
    if m >= DECK_SIZE {panic!(format!("m ({}) is greater than or equal to size ({})",m,DECK_SIZE))}
    if m == 0 {panic!(format!("increment_m must be 1 or more"))}
    let f = modinverse(m as isize, DECK_SIZE as isize).unwrap() as usize;
    // println!("Modulo Inverse of ({},{}) is {}", m, DECK_SIZE, f);
    let unshuffled = 0..DECK_SIZE;
    let cache: Vec<usize> = (*cards).take(MAX_CACHE).cloned().collect::<Vec<_>>();
    Box::new((0..MAX_CACHE).into_iter().map(|i|{cache[i * f % DECK_SIZE]}).chain(cards).take(MAX_CACHE).collect::<Vec<_>>().into_iter())
}
fn shuffle(cards: Box<dyn DoubleEndedIterator<Item=usize>>, input: &str) -> Box<dyn DoubleEndedIterator<Item=usize>> {
    let mut new_deal = cards;
    for line in input.lines() {
        let line = line.trim();
        if line.len() == 0 {continue};
        new_deal =
            if line.contains("deal into new stack") {
                new_stack(new_deal)
            } else if line.contains("deal with increment ") {
                let pieces: Vec<String> = line.split("deal with increment ").map(|s|{s.to_string()}).collect();
                let param = if pieces.len() == 2 {
                    if pieces[0].len() == 0 {
                        pieces[1].parse().unwrap()
                    } else {panic!("Ack!")}
                } else {panic!("Ack!")};
                increment_m(new_deal, param)
            } else if line.contains("cut ") {
                let pieces: Vec<String> = line.split("cut ").map(|s|{s.to_string()}).collect();
                let param = if pieces.len() == 2 {
                    if pieces[0].len() == 0 {
                        pieces[1].parse().unwrap()
                    } else {panic!("Ack!")}
                } else {panic!("Ack!")};
                cut_n(new_deal, param)
            } else {
                panic!(format!("Unknown input line: {}", line));
            }
    }
    new_deal
}

#[test]
fn test_increment_n() {
    let deck = Box::new(0..DECK_SIZE);
    let deck = increment_m(deck, 3);
    let deck = deck.collect();
    assert_eq!(deck, vec![0,7,4,1,8,5,2,9,6,3]);
    let deck = Box::new(0..DECK_SIZE);
    let deck = increment_m(deck, 7);
    let deck = deck.collect();
    assert_eq!(deck, vec![0,3,6,9,2,5,8,1,4,7]);
}
#[test]
fn test_cut_n() {
    let deck = Box::new(0..DECK_SIZE);
    let deck = cut_n(deck, 3);
    let deck = deck.collect();
    assert_eq!(deck, vec![3,4,5,6,7,8,9,0,1,2]);
    let deck = Box::new(0..DECK_SIZE);
    let deck = cut_n(deck, -2);
    let deck = deck.collect();
    assert_eq!(deck, vec![8,9,0,1,2,3,4,5,6,7]);
}
#[test]
fn test_new_stack() {
    let deck = Box::new(0..DECK_SIZE);
    let deck = new_stack(deck);
    let deck = deck.collect();
    assert_eq!(deck, vec![9,8,7,6,5,4,3,2,1,0]);
}

// #[test]
// fn test_big_new_stack() {
//     let deck: Vec<usize> = (0..DECK_SIZE).collect();
//     let deck: Vec<usize> = new_stack(deck).into_iter().take(10).collect();
//     assert_eq!(deck, vec![10_006,10_005,10_004,10_003,10_002,10_001,10_000,9_999,9_998,9_997]);
// }
// #[test]
// fn test_big_cut_n() {
//     let size = DECK_SIZE;
//     let deck: Vec<usize> = (0..DECK_SIZE).collect();
//     let deck_start: Vec<usize> = cut_n(deck.clone(), 3).iter().take(5).map(|c|{*c}).collect();
//     assert_eq!(deck_start, vec![3,4,5,6,7]);
//     let deck_end: Vec<usize> = cut_n(deck, 3).into_iter().skip(size-5).collect();
//     assert_eq!(deck_end, vec![10_005,10_006,0,1,2]);
//     // negative cut
//     let size = DECK_SIZE;
//     let deck: Vec<usize> = (0..DECK_SIZE).collect();
//     let deck_start: Vec<usize> = cut_n(deck.clone(), -3).iter().take(5).map(|c|{*c}).collect();
//     assert_eq!(deck_start, vec![10_004,10_005,10_006,0,1]);
//     let deck_end: Vec<usize> = cut_n(deck, -3).into_iter().skip(size-5).collect();
//     assert_eq!(deck_end, vec![9_999,10_000,10_001,10_002,10_003]);
// }
// #[test]
// fn test_big_increment_n() {
//     let deck: Vec<usize> = (0..DECK_SIZE).collect();
//     let deck: Vec<usize> = increment_n(deck, 2).into_iter().take(10).collect();
//     assert_eq!(deck, vec![0,5004,1,5005,2,5006,3,5007,4,5008]);
//     let deck: Vec<usize> = (0..DECK_SIZE).collect();
//     let deck: Vec<usize> = increment_n(deck, 3).into_iter().take(10).collect();
//     assert_eq!(deck, vec![0,3336,6672,1,3337,6673,2,3338,6674,3]);
//     let deck: Vec<usize> = (0..DECK_SIZE).collect();
//     let deck: Vec<usize> = increment_n(deck, 9).into_iter().take(10).collect();
//     assert_eq!(deck, vec![0,1112,2224,3336,4448,5560,6672,7784,8896,1]);
//     let deck: Vec<usize> = (0..DECK_SIZE).collect();
//     let deck: Vec<usize> = increment_n(deck, 10_006).into_iter().take(10).collect();
//     assert_eq!(deck, vec![0,10_006,10_005,10_004,10_003,10_002,10_001,10_000,9_999,9_998,]);
// }
#[test]
fn test_part1() {
    let deck = Box::new(0..DECK_SIZE);
    let deck = shuffle(deck, INPUT);
    for (card_num,card) in deck.enumerate() {
        if card == 2019 {
            assert_eq!(card_num, 4703);
            break;
        }
    }
}

// #[test]
// fn test_ex1() {assert_eq!(example(EX1), vec![0, 3, 6, 9, 2, 5, 8, 1, 4, 7]);}
// #[test]
// fn test_ex2() {(example(EX1), vec![3, 0, 7, 4, 1, 8, 5, 2, 9, 6]);}
// #[test]
// fn test_ex3() {(example(EX1), vec![6, 3, 0, 7, 4, 1, 8, 5, 2, 9]);}
// #[test]
// fn test_ex4() {(example(EX1), vec![9, 2, 5, 8, 1, 4, 7, 0, 3, 6]);}

// fn example(input: &'static str) -> Vec<usize> {
//     let deck: Vec<usize> = (0..DECK_SIZE).collect();
//     shuffle(deck, input)
// }

const EX1: &'static str = r#"
deal with increment 7
"#;
const EX2: &'static str = r#"
cut 6
deal with increment 7
deal into new stack
"#;
const EX3: &'static str = r#"
deal with increment 7
deal with increment 9
cut -2
"#;
const EX4: &'static str = r#"
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
"#;
const INPUT: &'static str = r#"
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
"#;