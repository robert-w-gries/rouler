// rouler - A container-based system for generating die rolls
// Copyright (C) 2016 by John Berry
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use rand::{distributions::{Distribution, Uniform}, thread_rng};

pub fn roll_dice_raw(num: u64, sides: u64) -> u64 {
    if sides == 0 {
        return 0;
    }

    // The `rand` docs recommend constructing `Uniform` distribution to make
    // sampling of multiple values faster.
    let between = Uniform::from(1..(sides + 1));
    let mut rng = thread_rng();
    (0..num).map(|_| between.sample(&mut rng)).fold(0, |acc, x| acc + x)
}

pub fn roll_custom_dice_raw(num: u64, sides: &[i64]) -> i64 {
    if sides.is_empty() {
        return 0;
    }

    use rand::seq::SliceRandom;
    let mut rng = thread_rng();

    (0..num).map(|_| sides.choose(&mut rng).unwrap()).fold(0, |acc, x| acc + *x)
}

#[cfg(test)]
mod tests {
    mod normal {
        use super::super::roll_dice_raw;

        #[test]
        fn zero_d_zero() {
            assert_eq!(roll_dice_raw(0,0), 0);
        }

        #[test]
        fn one_d_zero() {
            assert_eq!(roll_dice_raw(1,0), 0);
        }

        #[test]
        fn zero_d_one() {
            assert_eq!(roll_dice_raw(0,1), 0);
        }

        #[test]
        fn x_d_one() {
            for x in 1..100 {
                assert_eq!(roll_dice_raw(x,1), x);
            }
        }

        #[test]
        fn one_d_x() {
            for x in 1..100 {
                let roll = roll_dice_raw(1,x);
                assert!(1 <= roll && roll <= x);
            }
        }
    }

    mod custom {
        use super::super::roll_custom_dice_raw;

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
    }
}
