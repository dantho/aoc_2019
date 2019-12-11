/// https://adventofcode.com/2019/day/10
use std::collections::HashSet;
use num_integer::gcd;

fn main() {
    // orig is a list of coord pairs containing asteroids
    let orig_skymap: HashSet<(isize,isize)> = INPUT.lines()
    .filter(|line|{line.trim().len() > 0})
    .enumerate()
    .map::<Vec<(isize,isize)>,_>(|(row,line)|{
        line.chars().enumerate()
        .filter(|(_col, ch)|{ ch == &'#' })
        .map(|(col, _)| { (col as isize, row as isize) })
        .collect()
    })
    .flatten()
    .collect();
    // orig is now constructed and will never change

    // Plan:
    // Map each asteroid on the orig map to count of visible targets as follows:
    //    make a copy of orig
    //    remove subject asteroid
    //    In a loop, pick any target,
    //       move closest asteroid on path to target into visible list
    //       (alternately, just bump a count)
    //       remove all other asteroids on path through target to edge of grid
    //       ...until all targets are gone
    //    result is count of visible asteroids

    // find upper bounds (lower bounds are 0 by definition)
    let max = orig_skymap.iter().fold((0isize,0isize),|(mx,my), (x,y)| {
        (if *x>mx {*x} else {mx}, if *y>my {*y} else {my})
    });
    let best = orig_skymap.iter().fold((0,0,0),|(prior_max, prior_x, prior_y), subject| {
        let mut targets = orig_skymap.clone();
        targets.remove(subject);
        let mut visible_cnt = 0;
        loop {
            let target = match targets.iter().next() {
                Some(t) => *t,
                None => break,
            };
            let path = reduce_path((target.0-subject.0,target.1-subject.1));
            let mut x = subject.0;
            let mut y = subject.1;
            // Find/count/remove first target
            loop {
                x += path.0;
                y += path.1;
                if x < 0 || y < 0 || x > max.0 || y > max.1 {
                    panic!("Target on which path was based was not found on path!");
                }
                if targets.remove(&(x,y)) {
                    visible_cnt += 1;
                    break; // done - first target found
                }
            }
            // Remove all other targets on path
            while x >= 0 && y >= 0 && x <= max.0 && y <= max.1 {
                targets.remove(&(x,y));
                x += path.0;
                y += path.1;
            }
            if targets.len() == 0 {break}; // done -- a target-free environment
        }
        if visible_cnt > prior_max {
            (visible_cnt, subject.0, subject.1)
        } else {
            (prior_max, prior_x, prior_y)
        }
    });
    println!("Part 1: {} asteroids are visible from ({},{})", best.0, best.1, best.2 );

    // Part 2

    let laser = (best.1,best.2);
    let mut sortable: Vec<_> = depthorize(orig_skymap.clone(), &laser).into_iter().map(|(depth,x,y)| {
        let qs = quadrant_slope(&(x-laser.0), &(y-laser.1)); // laser angle to target as sortable tuple
        (depth, qs.0, qs.1, x, y)
    }).collect();

    sortable.sort();
    assert_eq!({match sortable[0] {(d,_,_,x,y) => (d,x,y)}}, (0,laser.0,laser.1));
    // for item in &sortable { println!("{:?}", *item)};
    match sortable.iter().skip(200).next() { // Includes self, so is skipping 199 target asteroids
        Some((_,_,_,x,y)) => println!("Part 2: The 200th element is ({},{}), so answer is {}",
            x, y, x*100+y),
        None => println!("Weird. Didn't find 200 asteroids."),
    };
}
fn reduce_path(tup: (isize,isize)) -> (isize,isize) {
    let (x,y) = tup;
    let d = gcd(x,y);
    (x/d,y/d)
}
// adds a prefix to tuple indicating depth from perspective of subject
fn depthorize(mut input_map: HashSet<(isize,isize)>,subject: &(isize,isize)) -> HashSet<(usize,isize,isize)> {
    let mut with_depth: HashSet<(usize,isize,isize)> = HashSet::new();
    let max = input_map.iter().fold((0isize,0isize),|(mx,my), (x,y)| {
        (if *x>mx {*x} else {mx}, if *y>my {*y} else {my})
    });
    match input_map.take(subject) {
        Some((x,y)) => if !with_depth.insert((0,x,y)) {panic!("depthorize(): Already added subject.")},
        None => panic!(format!("depthorize(): Subject ({},{}) not found in map!", subject.0, subject.1)),
    }
    loop {
        let target = match input_map.iter().next() {
            Some(t) => *t,
            None => break, // input_map is not empty. We're done.
        };
        let path = reduce_path((target.0-subject.0,target.1-subject.1));
        let mut x = subject.0;
        let mut y = subject.1;
        let mut depth = 1;
        // Find all targets along path
        while x >= 0 && y >= 0 && x <= max.0 && y <= max.1 {
            if let Some(_) = input_map.take(&(x,y)) {
                if !with_depth.insert((depth,x,y)) {panic!("depthorize(): Already added target.")};
                depth += 1;
            };
            x += path.0;
            y += path.1;
        }
        if depth == 1 {panic!("Target on which path was based was not found on path!")};
        if input_map.len() == 0 {break}; // done -- a target-free environment
    };
    with_depth
}
// The point of this routine is ONLY to create something with a sort order consistant with
// the angle of the direction starting a 0 for straight up, and increasing going clockwise.
// HOWEVER, the puzzle defines pos x and y to be right and down, respectively.
// Since the laser starts by pointing up, quadrant 1 is right and up (negative y's). 
// Monotonicity is important.  Allow div-by-zero if inf is a value.  Nothing else matters.
fn quadrant_slope(x: &isize, y: &isize)-> (u8,u32) {
    const FIXED_PT_RESOLUTION: usize = 10000;
    let quadrant = match (*x>=0, *y>=0) {
        (true,false) => 1,
        (true,true) => 2,
        (false,true) => 3,
        (false,false) => 4,
    };
    let slope_in_fixed_pt = match quadrant {
        1|3 => ((*x as f32 / *y as f32).abs() * FIXED_PT_RESOLUTION as f32).round() as u32,
        2|4 => ((*y as f32 / *x as f32).abs() * FIXED_PT_RESOLUTION as f32).round() as u32,
        _ => panic!("Just wrong."),
    };
    (quadrant, slope_in_fixed_pt)
}

const EX1: &'static str =
"
.#..#
.....
#####
....#
...##
";
// Best is 5,8 with 33 other asteroids detected:
const EX2: &'static str =
"
......#.#.
#..#.#....
..#######.
.#.#.###..
.#..#.....
..#....#.#
#..#....#.
.##.#..###
##...#..#.
.#....####
";
// Best is 1,2 with 35 other asteroids detected:
const EX3: &'static str =
"
#.#...#.#.
.###....#.
.#....#...
##.#.#.#.#
....#.#.#.
.##..###.#
..#...##..
..##....##
......#...
.####.###.
";
// Best is 6,3 with 41 other asteroids detected:
const EX4: &'static str =
"
.#..#..###
####.###.#
....###.#.
..###.##.#
##.##.#.#.
....###..#
..#.#..#.#
#..#.#.###
.##...##.#
.....#.#..
";
// Best is 11,13 with 210 other asteroids detected:
const EX5: &'static str =
"
.#..##.###...#######
##.############..##.
.#.######.########.#
.###.#######.####.#.
#####.##.#.##.###.##
..#####..#.#########
####################
#.####....###.#.#.##
##.#################
#####.##.###..####..
..######..##.#######
####.##.####...##..#
.#####..#.######.###
##...#.##########...
#.##########.#######
.####.#.###.###.#.##
....##.##.###..#####
.#.#.###########.###
#.#.#.#####.####.###
###.##.####.##.#..##
";
// Part 1 input data:
const INPUT: &'static str =
"
.###..#......###..#...#
#.#..#.##..###..#...#.#
#.#.#.##.#..##.#.###.##
.#..#...####.#.##..##..
#.###.#.####.##.#######
..#######..##..##.#.###
.##.#...##.##.####..###
....####.####.#########
#.########.#...##.####.
.#.#..#.#.#.#.##.###.##
#..#.#..##...#..#.####.
.###.#.#...###....###..
###..#.###..###.#.###.#
...###.##.#.##.#...#..#
#......#.#.##..#...#.#.
###.##.#..##...#..#.#.#
###..###..##.##..##.###
###.###.####....######.
.###.#####.#.#.#.#####.
##.#.###.###.##.##..##.
##.#..#..#..#.####.#.#.
.#.#.#.##.##########..#
#####.##......#.#.####.
";