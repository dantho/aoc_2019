fn main() {
    let (begin,end) = (231832,767346);
    // let (begin,end) = (111111,111111+10);
    let tens = [100000,10000,1000,100,10,1];
    let answer: Vec<i32> = (begin..=end).filter_map(|candidate| {
        let digits: Vec<i32> = tens.iter().map(|pow_ten|{
            candidate / pow_ten - candidate / pow_ten / 10 * 10
        }).collect();
        let test_increasing = digits.iter().zip(digits.iter().skip(1)).fold(true, |b, (prior, d)| {
            b && d >= prior
        });
        let (last_seq,partial_test_double) = digits.iter().zip(digits.iter().skip(1)).fold((1,false), |(seq,found), (prior, d)| {
            if d == prior {
                (seq + 1, found)
            } else {
                (1, found || seq == 2)
            }
        });
        let test_double = last_seq == 2 || partial_test_double;
        if test_increasing && test_double {
            Some(candidate)
        } else {
            None
        }
    }).collect();
    println!("Ans: {:?}", answer.len());
    // println!("Answer is {}", answer);
}
