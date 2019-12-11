use std::convert::TryInto;
// One change is, I'm using Result instead of panicking. That propogates all the
// way up through main returning Result. If there is an illegal opcode, main
// will print that error out instead of panicking. Generally, panicking should
// be only due to bugs and not due to input data value.
// For the error type, I'm using an enum. I derive Debug on it so that if there
// is an illegal opcode, it gets printed nicely. That's also the reason that I
// used a struct-like declaration for the IllegalOpcode parameter, instead of a
// tuple-like declaration, so that the automatically-generated Debug error
// message is more helpful.
#[derive(Debug)]
enum Error {
    IllegalOpcode { code: usize },
}
const PROGRAM: [usize; 165] = [
    1, 0, 0, 3, 1, 1, 2, 3, 1, 3, 4, 3, 1, 5, 0, 3, 2, 1, 6, 19, 1, 9, 19, 23, 1, 6, 23, 27, 1, 10,
    27, 31, 1, 5, 31, 35, 2, 6, 35, 39, 1, 5, 39, 43, 1, 5, 43, 47, 2, 47, 6, 51, 1, 51, 5, 55, 1,
    13, 55, 59, 2, 9, 59, 63, 1, 5, 63, 67, 2, 67, 9, 71, 1, 5, 71, 75, 2, 10, 75, 79, 1, 6, 79,
    83, 1, 13, 83, 87, 1, 10, 87, 91, 1, 91, 5, 95, 2, 95, 10, 99, 2, 9, 99, 103, 1, 103, 6, 107,
    1, 107, 10, 111, 2, 111, 10, 115, 1, 115, 6, 119, 2, 119, 9, 123, 1, 123, 6, 127, 2, 127, 10,
    131, 1, 131, 6, 135, 2, 6, 135, 139, 1, 139, 5, 143, 1, 9, 143, 147, 1, 13, 147, 151, 1, 2,
    151, 155, 1, 10, 155, 0, 99, 2, 14, 0, 0,
];
fn main() -> Result<(), Error> {
    for noun in 0..=99 {
        for verb in 0..=99 {
            // I used a Vec instead of an array for two reasons. First, because
            // it is flexible about the size of the contained program. Second,
            // because arrays are on the stack and Vecs are on the heap. Since
            // this is "big", I think it's better on the heap.
            //
            // I was unsure if using vec![with the program here] would create an
            // array on the stack from which to initialize the Vec, so I moved
            // the program into the global constant PROGRAM and use to_vec here
            // to get a fresh memory initialized from that each iteration.
            let mut mem = PROGRAM.to_vec();
            mem[1] = noun;
            mem[2] = verb;
            if run(mem)? == 19_690_720 {
                println!("{}", 100 * noun + verb);
                return Ok(());
            };
        }
    }
    println!("No answer.");
    Ok(())
}
fn run(mut mem: Vec<usize>) -> Result<usize, Error> {
    // Instead of mutating a pc variable with increment, I step the pc by 4
    // through the program...
    for pc in (0..mem.len()).step_by(4) {
        // ...and then take a 4-element slice from the memory, which is
        // converted into an array by TryFrom. That can't fail, so I unwrap the
        // result.
        let instruction: [usize; 4] = (&mem[pc..pc + 4]).try_into().unwrap();
        // I am implementing TryFrom instead of From because conversion from a
        // numeric value to an Opcode can fail. (Note that implementing TryFrom
        // implicitly implements TryInto.) I am using the '?' operator to bail
        // early with an error if the conversion fails.
        let opcode: Opcode = instruction.try_into()?;
        // I moved execution logic for the opcodes to their own impl. The return
        // value is whether or not the instruction executed. Only Halt returns false.
        if !opcode.execute(&mut mem) {
            break;
        }
    }
    Ok(mem[0])
}
// For Add and Multiply, I store the operands (which are addresses) within. I
// use an array because the pattern match on an array is exhaustive, since an
// array has a fixed size. Slices on the other hand, because they have unknown
// size, don't have exhaustive pattern matching for a known-length slice.
enum Opcode {
    Add([usize; 3]),
    Multiply([usize; 3]),
    Halt,
}
impl Opcode {
    fn execute(self, mem: &mut [usize]) -> bool {
        match self {
            // Here you can see the nice pattern match on the operands.
            Opcode::Add([a, b, c]) => mem[c] = mem[a] + mem[b],
            Opcode::Multiply([a, b, c]) => mem[c] = mem[a] * mem[b],
            Opcode::Halt => return false,
        }
        true
    }
}
impl std::convert::TryFrom<[usize; 4]> for Opcode {
    type Error = Error;
    fn try_from([code, a, b, c]: [usize; 4]) -> Result<Self, Error> {
        let opcode = match code {
            1 => Opcode::Add([a, b, c]),
            2 => Opcode::Multiply([a, b, c]),
            99 => Opcode::Halt,
            _ => return Err(Error::IllegalOpcode { code }),
        };
        Ok(opcode)
    }
}

#[test]
fn test_add() {
    let op = Add;
    assert_eq!(op.exec(1,2), 3);
}

#[test]
fn test_mult() {
    let op = Mult;
    assert_eq!(op.exec(1,2), 2);
}