use std::convert::{TryFrom, TryInto};
use std::collections::HashMap;

fn main() -> Result<(),Error> {
    for i in 0..1 {
        let mut ff = FuelFactory::new()?;
        // What is the minimum amount of ORE required to produce exactly 1 FUEL?
        let ore_per_fuel = ff.ore_required(1.0, "FUEL")?;
        println!("1 FUEL requires {} ORE", ore_per_fuel);
        println!("1 Trillion ORE can make {} FUEL", 1e12/ore_per_fuel);
    }    
    Ok(())
}
struct FuelFactory {
    reactions: HashMap<&'static str, Reaction>,
}
impl FuelFactory {
    fn new() -> Result<Self,Error> {
        let input = INPUT;
        let _unused = (EX1, EX2, EX3, EX4, EX5, INPUT);
        let reactions: Vec<Reaction> = input.into_iter().map(|s|
            (*s).try_into() 
        ).collect::<Result<Vec<_>,_>>()?;
        let reactions = reactions.into_iter().map(|r|{(r.output.symbol,r)}).collect::<HashMap<_,_>>();    
        Ok(FuelFactory { reactions })
    }
    fn ore_required(&mut self, how_many: f64, to_make: &'static str) -> Result<f64,Error> {
        // println!("Making {} {}:", how_many, to_make);
        // END RECURSION
        if to_make == "ORE" {
            return Ok(how_many);
        };
        // How many (fractional) reactions are required?
        let this_reaction = self.reactions.get(to_make).ok_or(Error::UnknownChem{s:to_make})?;
        let reaction_count = how_many / this_reaction.output.qty;
        // Produce more by recursing down all paths to ORE
        let mut total_ore = 0.0f64;
        for input in this_reaction.inputs.clone() {
            total_ore += self.ore_required(input.qty*reaction_count, input.symbol)?;
        }
        Ok(total_ore)
    }
}
#[derive(Debug)]
enum Error {
    ChemParse { s: &'static str },
    ReactionParse { s: &'static str },
    UnknownChem { s: &'static str },
}
#[derive(Debug,Clone,Copy)]
struct Chem {
    symbol: &'static str,
    qty: f64,
}
impl TryFrom<&'static str> for Chem {
    type Error = Error;
    fn try_from(s: &'static str) -> Result<Self, Self::Error> {
        let mut qty_chem: Vec<_> = s.split_ascii_whitespace().collect();
        if qty_chem.len()!=2 {return Err(Error::ChemParse {s});}
        let symbol:&'static str = qty_chem.pop().unwrap();
        let qty = match qty_chem.pop().unwrap().parse() {
            Ok(q) => q,
            Err(_) => return Err(Error::ChemParse{s}),
        };
        Ok(Chem{symbol,qty})
    }
}
struct Reaction {
    inputs: Vec<Chem>,
    output: Chem,
}
impl TryFrom<&'static str> for Reaction {
    type Error = Error;
    fn try_from(s: &'static str) -> Result<Self, Self::Error> {
        let mut left_right: Vec<&'static str> = s.split(" => ").collect();
        let right = left_right.pop().unwrap();
        let left = left_right.pop().unwrap();
        if left_right.len()!=0 {return Err(Error::ReactionParse { s });}
        let left: Vec<_> = left.split(",").collect();
        if left.len()==0 {return Err(Error::ReactionParse { s });}
        let chem: Chem = right.try_into()?;
        Ok(Reaction {
            output: chem,
            inputs: left.into_iter().map(|s| {s.try_into().unwrap()}).collect()
        })
    }
}
const EX1: [&'static str;6] =[
    "10 ORE => 10 A",
    "1 ORE => 1 B",
    "7 A, 1 B => 1 C",
    "7 A, 1 C => 1 D",
    "7 A, 1 D => 1 E",
    "7 A, 1 E => 1 FUEL",
];
const EX2: [&'static str;7] =[
    "9 ORE => 2 A",
    "8 ORE => 3 B",
    "7 ORE => 5 C",
    "3 A, 4 B => 1 AB",
    "5 B, 7 C => 1 BC",
    "4 C, 1 A => 1 CA",
    "2 AB, 3 BC, 4 CA => 1 FUEL",
];
const EX3: [&'static str;9] =[
    "157 ORE => 5 NZVS",
    "165 ORE => 6 DCFZ",
    "44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL",
    "12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ",
    "179 ORE => 7 PSHF",
    "177 ORE => 5 HKGWZ",
    "7 DCFZ, 7 PSHF => 2 XJWVT",
    "165 ORE => 2 GPVTF",
    "3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT",
];
const EX4: [&'static str;12] =[
    "2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG",
    "17 NVRVD, 3 JNWZP => 8 VPVL",
    "53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL",
    "22 VJHF, 37 MNCFX => 5 FWMGM",
    "139 ORE => 4 NVRVD",
    "144 ORE => 7 JNWZP",
    "5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC",
    "5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV",
    "145 ORE => 6 MNCFX",
    "1 NVRVD => 8 CXFTF",
    "1 VJHF, 6 MNCFX => 4 RFSQX",
    "176 ORE => 6 VJHF",
];
const EX5: [&'static str;17] =[
    "171 ORE => 8 CNZTR",
    "7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL",
    "114 ORE => 4 BHXH",
    "14 VRPVC => 6 BMBT",
    "6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL",
    "6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT",
    "15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW",
    "13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW",
    "5 BMBT => 4 WPTQ",
    "189 ORE => 9 KTJDG",
    "1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP",
    "12 VRPVC, 27 CNZTR => 2 XDBXC",
    "15 KTJDG, 12 BHXH => 5 XCVML",
    "3 BHXH, 2 VRPVC => 7 MZWV",
    "121 ORE => 7 VRPVC",
    "7 XCVML => 6 RJRHP",
    "5 BHXH, 4 VRPVC => 5 LTCX",
];
const INPUT: [&'static str;63] =[
    "1 JNDQ, 11 PHNC => 7 LBJSB",
    "1 BFKR => 9 VGJG",
    "11 VLXQL => 5 KSLFD",
    "117 ORE => 6 DMSLX",
    "2 VGJG, 23 MHQGW => 6 HLVR",
    "2 QBJLJ => 6 DBJZ",
    "1 CZDM, 21 ZVPJT, 1 HLVR => 5 VHGQP",
    "1 RVKX => 1 FKMQD",
    "38 PHNC, 10 MHQGW => 5 GMVJX",
    "4 CZDM, 26 ZVHX => 7 QBGQB",
    "5 LBJSB, 2 DFZRS => 4 QBJLJ",
    "4 TJXZM, 11 DWXW, 14 VHGQP => 9 ZBHXN",
    "20 VHGQP => 8 SLXQ",
    "1 VQKM => 9 BDZBN",
    "115 ORE => 4 BFKR",
    "1 VGJG, 1 SCSXF => 5 PHNC",
    "10 NXZXH, 7 ZFXP, 7 ZCBM, 7 MHNLM, 1 BDKZM, 3 VQKM => 5 RMZS",
    "147 ORE => 2 WHRD",
    "16 CQMKW, 8 BNJK => 5 MHNLM",
    "1 HLVR => 5 TJQDC",
    "9 GSLTP, 15 PHNC => 5 SFZTF",
    "2 MJCD, 2 RVKX, 4 TJXZM => 1 MTJSD",
    "1 DBJZ, 3 SLXQ, 1 GMSB => 9 MGXS",
    "1 WZFK => 8 XCMX",
    "1 DFZRS => 9 GSLTP",
    "17 PWGXR => 2 DFZRS",
    "4 BFKR => 7 JNDQ",
    "2 VKHN, 1 SFZTF, 2 PWGXR => 4 JDBS",
    "2 ZVPJT, 1 PHNC => 6 VQKM",
    "18 GMSB, 2 MGXS, 5 CQMKW => 3 XGPXN",
    "4 JWCH => 3 BNJK",
    "1 BFKR => 2 PWGXR",
    "12 PHNC => 2 GMSB",
    "5 XGPXN, 3 VQKM, 4 QBJLJ => 9 GXJBW",
    "4 MHQGW => 9 DWXW",
    "1 GMSB, 1 BFKR => 5 DBKC",
    "1 VLXQL, 10 KSLFD, 3 JWCH, 7 DBKC, 1 MTJSD, 2 WZFK => 9 GMZB",
    "4 JDBS => 8 BRNWZ",
    "2 ZBHXN => 7 HMNRT",
    "4 LBJSB => 7 BCXGX",
    "4 MTJSD, 1 SFZTF => 8 ZCBM",
    "12 BRNWZ, 4 TJXZM, 1 ZBHXN => 7 WZFK",
    "10 HLVR, 5 LBJSB, 1 VKHN => 9 TJXZM",
    "10 BRNWZ, 1 MTJSD => 6 CMKW",
    "7 ZWHT => 7 VKHN",
    "5 CQMKW, 2 DBKC => 6 ZFXP",
    "1 CMKW, 5 JNDQ, 12 FKMQD, 72 BXZP, 28 GMVJX, 15 BDZBN, 8 GMZB, 8 RMZS, 9 QRPQB, 7 ZVHX => 1 FUEL",
    "10 MGXS => 9 JWCH",
    "1 BFKR => 8 SCSXF",
    "4 SFZTF, 13 CZDM => 3 RVKX",
    "1 JDBS, 1 SFZTF => 9 TSWV",
    "2 GMVJX, 1 PHNC => 1 CZDM",
    "6 JDBS => 1 BXZP",
    "9 TSWV, 5 TJXZM => 8 NXZXH",
    "1 HMNRT, 5 TSWV => 4 VLXQL",
    "16 WZFK, 11 XCMX, 1 GXJBW, 16 NXZXH, 1 QBGQB, 1 ZCBM, 10 JWCH => 3 QRPQB",
    "12 SCSXF, 6 VGJG => 4 ZVPJT",
    "10 JNDQ => 3 ZWHT",
    "1 DBJZ, 9 BCXGX => 2 CQMKW",
    "1 WHRD, 14 DMSLX => 8 MHQGW",
    "3 VKHN, 8 TJQDC => 4 MJCD",
    "1 QBJLJ => 4 ZVHX",
    "1 MHQGW, 4 ZVHX => 3 BDKZM",
];
