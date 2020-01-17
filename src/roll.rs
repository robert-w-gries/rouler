// rouler - A container-based system for generating die rolls
// Copyright (C) 2016 by John Berry
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use rand::{distributions::{Distribution, Uniform}, thread_rng};

const MAX_ROLLS: u64 = 1000;
const MAX_SIDES: u64 = u32::max_value() as u64;
const MAX_CUSTOM_SIDES: usize = 1000;

pub enum DieType {
    Custom,
    Normal,
}

#[derive(Clone, Copy)]
pub enum Take {
    KeepHighest(u64),
    DropLowest(u64),
}

#[derive(Clone, Copy)]
pub enum TargetRoll {
    GT(u64),
    GTE(u64),
    LT(u64),
    LTE(u64),
}

pub struct Roll {
    count: u64,
    custom_sides: Vec<i64>,
    die_type: DieType,
    sides: u64,
    take: Option<Take>,
    target_roll: Option<TargetRoll>,
}

impl Roll {
    pub fn new() -> Self {
        Self {
            count: 0,
            custom_sides: Vec::new(),
            die_type: DieType::Normal,
            sides: 0,
            take: None,
            target_roll: None,
        }
    }

    pub fn add_custom_sides<'a>(&'a mut self, custom_sides: &[i64]) -> &'a mut Self {
        self.custom_sides.extend(custom_sides);
        self
    }

    pub fn count<'a>(&'a mut self, count: u64) -> &'a mut Self {
        let count = if count > MAX_ROLLS {
            MAX_ROLLS
        } else {
            count
        };
        self.count = count;
        self
    }

    pub fn die_type<'a>(&'a mut self, die_type: DieType) -> &'a mut Self {
        self.die_type = die_type;
        self
    }

    pub fn drop_lowest<'a>(&'a mut self, drop_lowest: u64) -> &'a mut Self {
        self.take = Some(Take::DropLowest(drop_lowest));
        self
    }

    pub fn keep_highest<'a>(&'a mut self, keep_highest: u64) -> &'a mut Self {
        self.take = Some(Take::KeepHighest(keep_highest));
        self
    }

    pub fn sides<'a>(&'a mut self, sides: u64) -> &'a mut Self {
        let sides = if sides > MAX_SIDES {
            MAX_SIDES
        } else {
            sides
        };
        self.sides = sides;
        self
    }

    pub fn target_roll<'a>(&'a mut self, target_roll: TargetRoll) -> &'a mut Self {
        self.target_roll = Some(target_roll);
        self
    }

    pub fn roll_dice(&self) -> i64 {
        match self.die_type {
            DieType::Custom => self.roll_custom_dice(),
            DieType::Normal => self.roll_normal_dice() as i64,
        }
    }

    fn roll_normal_dice(&self) -> u64 {
        let mut results: Vec<u64> = if self.sides > 0 {
            // The `rand` docs recommend constructing `Uniform` distribution to make
            // sampling of multiple values faster.
            let between = Uniform::from(1..(self.sides + 1));
            let mut rng = thread_rng();
            (0..self.count).map(|_| between.sample(&mut rng)).collect()
        } else {
            // zero-sided dice will always roll zero
            vec![0; self.count as usize]
        };

        if let Some(take) = self.take {
            let limit_take = |take_num: u64| -> u64 {
                if take_num > self.count {
                    self.count
                } else {
                    take_num
                }
            };

            results.sort_by(|a, b| a.cmp(b)); // sort by ascending
            results = match take {
                Take::KeepHighest(kh) => results[..limit_take(kh) as usize].to_vec(),
                Take::DropLowest(dl) => results[limit_take(dl) as usize..].to_vec(),
            };
        }

        if let Some(target_roll) = self.target_roll {
            let success_count = match target_roll {
                TargetRoll::GT(target_number) => results.iter().filter(|&roll| *roll > target_number).count(),
                TargetRoll::GTE(target_number) => results.iter().filter(|&roll| *roll >= target_number).count(),
                TargetRoll::LT(target_number) => results.iter().filter(|&roll| *roll < target_number).count(),
                TargetRoll::LTE(target_number) => results.iter().filter(|&roll| *roll <= target_number).count(),
            };
            success_count as u64
        } else {
            results.iter().fold(0, |acc, x| acc + x)
        }
    }

    fn roll_custom_dice(&self) -> i64 {
        if self.custom_sides.is_empty() {
            return 0;
        }

        let custom_sides = if self.custom_sides.len() > MAX_CUSTOM_SIDES {
            &self.custom_sides[..MAX_CUSTOM_SIDES]
        } else {
            &self.custom_sides[..]
        };

        use rand::seq::SliceRandom;
        let mut rng = thread_rng();
        (0..self.count).map(|_| custom_sides.choose(&mut rng).unwrap()).fold(0, |acc, x| acc + *x)
    }
}

#[cfg(test)]
mod tests {
    mod normal {
        use super::super::{MAX_ROLLS, MAX_SIDES, Roll, TargetRoll};

        #[test]
        fn zero_d_zero() {
            assert_eq!(Roll::new().count(0).sides(0).roll_dice(), 0);
        }

        #[test]
        fn one_d_zero() {
            assert_eq!(Roll::new().count(1).sides(0).roll_dice(), 0);
        }

        #[test]
        fn zero_d_one() {
            assert_eq!(Roll::new().count(0).sides(1).roll_dice(), 0);
        }

        #[test]
        fn x_d_one() {
            for x in 1..100 {
                assert_eq!(Roll::new().count(x).sides(1).roll_dice(), x as i64);
            }
        }

        #[test]
        fn one_d_x() {
            for x in 1..100 {
                let roll = Roll::new().count(1).sides(x).roll_dice();
                assert!(1 <= roll && roll <= x as i64);
            }
        }

        #[test]
        fn max() {
            let roll = Roll::new().count(u64::max_value()).sides(u64::max_value()).roll_dice();
            let max = MAX_ROLLS * MAX_SIDES;
            assert!(1 <= roll && roll <= max as i64);
        }

        #[test]
        fn keep_two() {
            assert_eq!(Roll::new().count(5).sides(1).keep_highest(2).roll_dice(), 2);
        }

        #[test]
        fn keep_more() {
            assert_eq!(Roll::new().count(5).sides(1).keep_highest(6).roll_dice(), 5);
        }


        #[test]
        fn keep_zero() {
            assert_eq!(Roll::new().count(5).sides(1).keep_highest(0).roll_dice(), 0);
        }

        #[test]
        fn keep_max() {
            assert_eq!(Roll::new().count(5).sides(1).keep_highest(u64::max_value()).roll_dice(), 5);
        }

        #[test]
        fn drop_none() {
            assert_eq!(Roll::new().drop_lowest(0).roll_dice(), 0);
        }

        #[test]
        fn drop_two() {
            assert_eq!(Roll::new().count(5).sides(1).drop_lowest(2).roll_dice(), 3);
        }

        #[test]
        fn drop_more() {
            assert_eq!(Roll::new().count(5).sides(1).drop_lowest(6).roll_dice(), 0);
        }


        #[test]
        fn drop_zero() {
            assert_eq!(Roll::new().count(5).sides(1).drop_lowest(0).roll_dice(), 5);
        }

        #[test]
        fn drop_max() {
            assert_eq!(Roll::new().count(5).sides(1).drop_lowest(u64::max_value()).roll_dice(), 0);
        }

        #[test]
        fn target_gt() {
            assert_eq!(Roll::new().count(100).sides(1).target_roll(TargetRoll::GT(1)).roll_dice(), 0);
            assert_eq!(Roll::new().count(100).sides(100).target_roll(TargetRoll::GT(0)).roll_dice(), 100);
            assert_eq!(Roll::new().count(u64::max_value()).sides(u64::max_value()).target_roll(TargetRoll::GT(u64::max_value())).roll_dice(), 0);
        }

        #[test]
        fn target_gte() {
            assert_eq!(Roll::new().count(100).sides(1).target_roll(TargetRoll::GTE(1)).roll_dice(), 100);
            assert_eq!(Roll::new().count(100).sides(100).target_roll(TargetRoll::GTE(0)).roll_dice(), 100);
            assert_eq!(Roll::new().count(u64::max_value()).sides(u64::max_value()).target_roll(TargetRoll::GTE(u64::max_value())).roll_dice(), 0);

            assert_eq!(Roll::new().count(10).sides(0).target_roll(TargetRoll::GTE(0)).roll_dice(), 10);
            assert_eq!(Roll::new().count(0).sides(10).target_roll(TargetRoll::GTE(0)).roll_dice(), 0);
            assert_eq!(Roll::new().count(100).sides(1).target_roll(TargetRoll::GTE(1)).roll_dice(), 100);
        }

        #[test]
        fn target_lt() {
            assert_eq!(Roll::new().count(100).sides(100).target_roll(TargetRoll::LT(101)).roll_dice(), 100);
            assert_eq!(Roll::new().count(100).sides(0).target_roll(TargetRoll::LT(1)).roll_dice(), 100);
            assert_eq!(Roll::new().count(100).sides(1).target_roll(TargetRoll::LT(1)).roll_dice(), 0);
            assert_eq!(Roll::new().count(0).sides(100).target_roll(TargetRoll::LT(1)).roll_dice(), 0);
            assert_eq!(Roll::new().count(100).sides(0).target_roll(TargetRoll::LT(0)).roll_dice(), 0);
        }

        #[test]
        fn target_lte() {
            assert_eq!(Roll::new().count(100).sides(100).target_roll(TargetRoll::LTE(100)).roll_dice(), 100);
            assert_eq!(Roll::new().count(100).target_roll(TargetRoll::LTE(0)).roll_dice(), 100);
            assert_eq!(Roll::new().count(100).sides(1).target_roll(TargetRoll::LTE(1)).roll_dice(), 100);
            assert_eq!(Roll::new().sides(100).target_roll(TargetRoll::LTE(1)).roll_dice(), 0);
        }
    }

    mod custom {
        use super::super::{MAX_ROLLS, MAX_CUSTOM_SIDES, DieType, Roll};

        #[test]
        fn zero_d_empty() {
            assert_eq!(Roll::new().die_type(DieType::Custom).add_custom_sides(&[]).roll_dice(), 0);
        }

        #[test]
        fn one_d_empty() {
            assert_eq!(Roll::new().die_type(DieType::Custom).count(1).roll_dice(), 0);
        }

        #[test]
        fn zero_d_one() {
            assert_eq!(Roll::new().die_type(DieType::Custom).add_custom_sides(&[42]).roll_dice(), 0);
        }

        #[test]
        fn one_d_one() {
            assert_eq!(Roll::new().die_type(DieType::Custom).count(1).add_custom_sides(&[42]).roll_dice(), 42);
        }

        #[test]
        fn one_d_many() {
            let sequence: Vec<i64> = (-25..25).collect();
            let roll = Roll::new().die_type(DieType::Custom).count(1).add_custom_sides(&sequence[..]).roll_dice();
            assert!(-25 <= roll && roll <= 25);
        }

        #[test]
        fn many_d_one() {
            assert_eq!(Roll::new().die_type(DieType::Custom).count(100).add_custom_sides(&[42]).roll_dice(), 100*42);
        }

        #[test]
        fn max() {
            let custom_sides: Vec<i64> = (1..(MAX_CUSTOM_SIDES * 2) as i64).collect();
            let roll = Roll::new().die_type(DieType::Custom).count(u64::max_value()).add_custom_sides(&custom_sides[..]).roll_dice();
            let max = (MAX_ROLLS as u64) * (MAX_CUSTOM_SIDES as u64);
            assert!(MAX_ROLLS as i64 <= roll && roll <= max as i64);
        }
    }
}
