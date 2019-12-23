/// https://adventofcode.com/2019/day/22
fn main() -> Result<(),Error> {
    let deck: Deck = Deck::new(0..10);
    Ok(())
}
#[derive(Debug)]
enum Error {
    IllegalMapData {ch: char},
    MapAssertFail {msg: String},
}
#[derive(Debug, PartialEq)]
struct Deck (Vec<usize>);
impl Deck {
    fn new(range: std::ops::Range<usize>) -> Self {
        Deck (range.collect())
    }
    fn new_stack(cards: Iterator<Item = usize> + DoubleEndedIterator) -> Self {
        cards.rev()
    }
    fn cut_n_cards(self, n: usize) -> Self {
        let size = self.0.len();
        Deck (self.into_iter().cycle().skip(n).take(size).collect::<Vec<_>>())
    }
    fn increment_n(self, n: isize) -> Self {
        let n_pos = if n >= 0 {n as usize} else {(self.0.len() as isize + n) as usize};
        Deck (self.0.iter().cycle().step_by(n_pos).map(|v|{*v}).take(self.0.len()).collect::<Vec<_>>())
    }
    fn shuffle(self, instructions: &str) -> Result<Self,Error> {
        Ok(self)
    }
}
impl IntoIterator for Deck {
    type Item = usize;
    type IntoIter = DeckIntoIterator;

    fn into_iter(self) -> Self::IntoIter {
        DeckIntoIterator {
            deck: self,
            index: 0,
        }
    }
}

struct DeckIntoIterator {
    deck: Deck,
    index: usize,
}
impl Iterator for DeckIntoIterator {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        let result = match self.index {
            self.pop().unwrap
        };
        self.index += 1 mod;
        Some(result)
    }
}
#[test]
fn test_increment_n() {
    let deck = Deck::new(0..10);
    let deck = deck.increment_n(3);
    assert_eq!(deck.0, vec![0,3,6,9,2,5,8,1,4,7]);
}
#[test]
fn test_cut_n_cards() {
    let deck = Deck::new(0..10);
    let deck = deck.cut_n_cards(3);
    assert_eq!(deck.0, vec![3,4,5,6,7,8,9,0,1,2]);
}
#[test]
fn test_new_stack() {
    let deck = Deck::new(0..10);
    let deck = deck.new_stack();
    assert_eq!(deck.0, vec![9,8,7,6,5,4,3,2,1,0]);
}
#[test]
fn ex1() {example(EX1, vec![0, 3, 6, 9, 2, 5, 8, 1, 4, 7]);}
#[test]
fn ex2() {example(EX1, vec![3, 0, 7, 4, 1, 8, 5, 2, 9, 6]);}
#[test]
fn ex3() {example(EX1, vec![6, 3, 0, 7, 4, 1, 8, 5, 2, 9]);}
#[test]
fn ex4() {example(EX1, vec![9, 2, 5, 8, 1, 4, 7, 0, 3, 6]);}

fn example(input: &'static str, result: Vec<usize>) -> Result<(),Error> {
    let deck = Deck::new(0..10);
    let instructions = input;
    let shuffle_result:Vec<usize> = deck.shuffle(instructions)?.into_iter().collect();
    let expected_result = result;
    assert_eq!(shuffle_result, expected_result);
    Ok(())
}
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
