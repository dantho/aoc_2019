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
    fn reduce(tup: (isize,isize)) -> (isize,isize) {
        let (x,y) = tup;
        let d = gcd(x,y);
        (x/d,y/d)
    }
    let best = orig_skymap.iter().fold((0,0,0),|(prior_max, prior_x, prior_y), subject| {
        let mut targets = orig_skymap.clone();
        targets.remove(subject);
        let mut visible_cnt = 0;
        loop {
            let target = match targets.iter().next() {
                Some(t) => *t,
                None => break,
            };
            let path = reduce((target.0-subject.0,target.1-subject.1));
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