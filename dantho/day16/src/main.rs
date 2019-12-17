fn main() {
    process(EX1, 4);
}
fn process(input: &'static str, phase_cnt: u32) -> String {
    let mut input = input.to_string();
    let base_pattern = vec![0,1,0,-1];
    for _phase in 0..phase_cnt {
        input = (1..=input.len()).into_iter().map(|i| {
            let input_iter = input.chars().map(|c|{c.to_string().parse::<i32>()}).filter_map(|d|{d.ok()});
            let pattern_iter = base_pattern.iter()
                    .map(|pat_d| {std::iter::repeat(pat_d).take(i)})
                    .flatten().cycle().skip(1);
            let calc = input_iter.zip(pattern_iter).fold(0,|acc, (num,pat)|{
                // print!("{}*{} + ", num, pat);
                acc + num * pat
            });
            let rightmost = calc.abs() % 10;
            // println!("= {}", rightmost);
            rightmost.to_string().pop().unwrap()
        }).collect();
    }
    let part1 = input.chars().take(8).collect::<String>();
    println!("{}",part1);
    part1
}
const INPUT: &'static str =
"59787832768373756387231168493208357132958685401595722881580547807942982606755215622050260150447434057354351694831693219006743316964757503791265077635087624100920933728566402553345683177887856750286696687049868280429551096246424753455988979991314240464573024671106349865911282028233691096263590173174821612903373057506657412723502892841355947605851392899875273008845072145252173808893257256280602945947694349746967468068181317115464342687490991674021875199960420015509224944411706393854801616653278719131946181597488270591684407220339023716074951397669948364079227701367746309535060821396127254992669346065361442252620041911746738651422249005412940728";
const EX1: &'static str = "12345678";
const EX2: &'static str = "80871224585914546619083218645595";
const EX3: &'static str = "19617804207202209144916044189917";
const EX4: &'static str = "69317163492948606335995924319873";
// const EX5: &'static str = "03036732577212944063491565474664";
// const EX6: &'static str = "02935109699940807407585447034323";
// const EX7: &'static str = "03081770884921959731165446850517";

#[test]
fn test_input() {
    assert_eq!(process(INPUT, 100), "42945143".to_string());
}
#[test]
fn test_ex1() {
    assert_eq!(process(EX1, 4), "01029498".to_string());
}
#[test]
fn test_ex2() {
    assert_eq!(process(EX2, 100), "24176176".to_string());
}
#[test]
fn test_ex3() {
    assert_eq!(process(EX3, 100), "73745418".to_string());
}
#[test]
fn test_ex4() {
    assert_eq!(process(EX4, 100), "52432133".to_string());
}