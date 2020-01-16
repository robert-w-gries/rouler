// rouler - A container-based system for generating die rolls
// Copyright (C) 2016 by John Berry
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use itertools::Itertools;
use rand::{distributions::{Distribution, Uniform}, thread_rng};

const MAX_ROLLS: u64 = 1000;
const MAX_SIDES: u64 = u32::max_value() as u64;
const MAX_CUSTOM_SIDES: usize = 1000;

pub fn roll_dice_raw(num: u64, sides: u64, keep_highest: u64) -> u64 {
    if sides == 0 {
        return 0;
    }

    let num = if num > MAX_ROLLS {
        MAX_ROLLS
    } else {
        num
    };

    let sides = if sides > MAX_SIDES {
        MAX_SIDES
    } else {
        sides
    };

    // The `rand` docs recommend constructing `Uniform` distribution to make
    // sampling of multiple values faster.
    let between = Uniform::from(1..(sides + 1));
    let mut rng = thread_rng();
    let rolls = (0..num).map(|_| between.sample(&mut rng)).sorted().rev().take(keep_highest as usize);
    rolls.fold(0, |acc, x| acc + x)
}

pub fn roll_custom_dice_raw(num: u64, sides: &[i64]) -> i64 {
    if sides.is_empty() {
        return 0;
    }

    let num = if num > MAX_ROLLS {
        MAX_ROLLS
    } else {
        num
    };

    let sides = if sides.len() > MAX_CUSTOM_SIDES {
        &sides[..MAX_CUSTOM_SIDES]
    } else {
        &sides[..]
    };

    use rand::seq::SliceRandom;
    let mut rng = thread_rng();
    (0..num).map(|_| sides.choose(&mut rng).unwrap()).fold(0, |acc, x| acc + *x)
}

#[cfg(test)]
mod tests {
    mod normal {
        use super::super::{MAX_ROLLS, MAX_SIDES, roll_dice_raw};

        #[test]
        fn zero_d_zero() {
            assert_eq!(roll_dice_raw(0,0,0), 0);
        }

        #[test]
        fn one_d_zero() {
            assert_eq!(roll_dice_raw(1,0,1), 0);
        }

        #[test]
        fn zero_d_one() {
            assert_eq!(roll_dice_raw(0,1,0), 0);
        }

        #[test]
        fn x_d_one() {
            for x in 1..100 {
                assert_eq!(roll_dice_raw(x,1, x), x);
            }
        }

        #[test]
        fn one_d_x() {
            for x in 1..100 {
                let roll = roll_dice_raw(1,x, 1);
                assert!(1 <= roll && roll <= x);
            }
        }

        #[test]
        fn max() {
            let roll = roll_dice_raw(u64::max_value(), u64::max_value(), u64::max_value());
            let max = MAX_ROLLS * MAX_SIDES;
            assert!(1 <= roll && roll <= max);
        }

        #[test]
        fn keep_two() {
            assert_eq!(roll_dice_raw(5, 1, 2), 2);
        }

        #[test]
        fn keep_more() {
            assert_eq!(roll_dice_raw(5, 1, 6), 5);
        }


        #[test]
        fn keep_zero() {
            assert_eq!(roll_dice_raw(5, 1, 0), 0);
        }

        #[test]
        fn keep_max() {
            assert_eq!(roll_dice_raw(5, 1, u64::max_value()), 5);
        }
    }

    mod custom {
        use super::super::{MAX_ROLLS, MAX_CUSTOM_SIDES, roll_custom_dice_raw};

        #[test]
        fn zero_d_empty() {
            assert_eq!(roll_custom_dice_raw(0, &[]), 0);
        }

        #[test]
        fn one_d_empty() {
            assert_eq!(roll_custom_dice_raw(1, &[]), 0);
        }

        #[test]
        fn zero_d_one() {
            assert_eq!(roll_custom_dice_raw(0, &[42]), 0);
        }

        #[test]
        fn one_d_one() {
            assert_eq!(roll_custom_dice_raw(1, &[42]), 42);
        }

        #[test]
        fn one_d_many() {
            let sequence: Vec<i64> = (-25..25).collect();
            let roll = roll_custom_dice_raw(1, &sequence[..]);
            assert!(-25 <= roll && roll <= 25);
        }

        #[test]
        fn many_d_one() {
            assert_eq!(roll_custom_dice_raw(100, &[42]), 100*42);
        }

        #[test]
        fn max() {
            let custom_sides: Vec<i64> = (1..(MAX_CUSTOM_SIDES * 2) as i64).collect();
            let roll = roll_custom_dice_raw(u64::max_value(), &custom_sides[..]);
            let max = (MAX_ROLLS as u64) * (MAX_CUSTOM_SIDES as u64);
            assert!(MAX_ROLLS as i64 <= roll && roll <= max as i64);
        }
    }
}
