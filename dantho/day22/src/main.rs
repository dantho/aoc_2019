/// https://adventofcode.com/2019/day/22
fn main() -> Result<(),Error> {
    let deck = 0..10;
    let deck = increment_n(deck, 7, 10);
    let deck = new_stack(deck);
    let deck = new_stack(deck);
    // let deck = cut_n(deck, 4, 10);
    // let deck = increment_n(deck, 3, 10);
    println!("deck: {:?}", deck);
    Ok(())
}
#[derive(Debug)]
enum Error {
    NotImplemented,
    // MapAssertFail {msg: String},
}
fn new_stack<T>(cards: T) -> std::iter::Rev<T>
where T: Iterator<Item=usize>+DoubleEndedIterator<Item=usize> {
    cards.rev()
}
fn cut_n<T>(cards: T, n: isize, size: usize) -> std::vec::IntoIter<usize>
where T: Clone+Iterator<Item=usize>+DoubleEndedIterator<Item=usize> {
    let n = if n < 0 {size as isize + n} else {n} as usize;
    cards.cycle().skip(n).take(size).collect::<Vec<usize>>().into_iter()
}
fn increment_n<T>(cards: T, n: usize, size: usize) -> std::vec::IntoIter<usize>
where T: Clone+Iterator<Item=usize>+DoubleEndedIterator<Item=usize> {
    cards.cycle().step_by(size-n).take(size).collect::<Vec<usize>>().into_iter()
}
fn shuffle<T>(cards: T, input: &str) -> Vec<usize>
where T: Clone+Iterator<Item=usize>+DoubleEndedIterator<Item=usize> {
    let deck_size = cards.clone().count();
    let mut new_deal = cards;
    for line in input.lines() {
        let line = line.trim();
        if line.len() == 0 {continue};
        new_deal =
            if line.contains("deal into new stack") {
                new_stack(new_deal)
            } else if line.contains("deal with increment "){
                let pieces = line.split("deal with increment ");
                if pieces.len() == 2 {
                    if pieces[0].len() == 0 {
                        let param = pieces[1].parse().unwrap();
                        increment_n(new_deal, param, deck_size)
                    }
                }
            } else if line.contains("cut ") {
                let pieces = line.split("cut ");
                if pieces.len() == 2 {
                    if pieces[0].len() == 0 {
                        let param = pieces[1].parse().unwrap();
                        cut_n(new_deal, param, deck_size)
                    }
                }
            } else {
                panic!(format!("Unknown input line: {}", line));
            };
    }
    new_deal.collect::<Vec<usize>>()
}

#[test]
fn test_increment_n() {
    let deck = 0..10;
    let deck = increment_n(deck, 3, 10).collect::<Vec<usize>>();
    assert_eq!(deck, vec![0,7,4,1,8,5,2,9,6,3]);
}
#[test]
fn test_cut_n_cards() {
    let deck = 0..10;
    let deck = cut_n(deck, 3, 10).collect::<Vec<usize>>();
    assert_eq!(deck, vec![3,4,5,6,7,8,9,0,1,2]);
    let deck = 0..10;
    let deck = cut_n(deck, -2, 10).collect::<Vec<usize>>();
    assert_eq!(deck, vec![8,9,0,1,2,3,4,5,6,7]);
}
#[test]
fn test_new_stack() {
    let deck = 0..10;
    let deck = new_stack(deck).collect::<Vec<usize>>();
    assert_eq!(deck, vec![9,8,7,6,5,4,3,2,1,0]);
}
// #[test]
// fn ex1() {example(EX1, vec![0, 3, 6, 9, 2, 5, 8, 1, 4, 7]);}
// #[test]
// fn ex2() {example(EX1, vec![3, 0, 7, 4, 1, 8, 5, 2, 9, 6]);}
// #[test]
// fn ex3() {example(EX1, vec![6, 3, 0, 7, 4, 1, 8, 5, 2, 9]);}
// #[test]
// fn ex4() {example(EX1, vec![9, 2, 5, 8, 1, 4, 7, 0, 3, 6]);}

// fn example(input: &'static str, result: Vec<usize>) -> Result<(),Error> {
//     let deck = Deck::new(0..10);
//     let instructions = input;
//     let shuffle_result:Vec<usize> = deck.shuffle(instructions)?.into_iter().collect();
//     let expected_result = result;
//     assert_eq!(shuffle_result, expected_result);
//     Ok(())
// }
const EX1: &'static str = r#"
deal with increment 7
deal into new stack
deal into new stack
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
