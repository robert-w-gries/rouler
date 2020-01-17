// rouler - A container-based system for generating die rolls
// Copyright (C) 2016 by John Berry
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use pest::{
    prec_climber::*,
    iterators::*,
};
use roll::{Roll, TargetRoll};

lazy_static! {
    static ref PREC_CLIMBER: PrecClimber<Rule> = {
        use self::Assoc::*;
        use self::Rule::*;

        // Order of precedence: "+-" is less than "*/"
        PrecClimber::new(vec![
            Operator::new(plus, Left) | Operator::new(minus, Left),
            Operator::new(times, Left) | Operator::new(slash, Left),
        ])
    };
}

#[derive(Parser)]
#[grammar = "rouler.pest"]
pub struct RollParser;

pub fn compute(expr: Pairs<Rule>) -> i64 {
    let primary = |pair: Pair<Rule>| match pair.as_rule() {
        Rule::uint => pair.as_str().parse::<u64>().unwrap() as i64,
        Rule::int => pair.as_str().parse::<i64>().unwrap().into(),
        Rule::expr => compute(pair.into_inner()),
        Rule::roll => {
            let mut inner = pair.into_inner();

            let mut roll = Roll::new();

            let num_rolls: u64 = {
                let num_rolls_str = inner.next().unwrap().as_str();
                num_rolls_str.parse::<u64>().expect("Could not parse number of rolls")
            };
            roll.count(num_rolls);

            let die_type = inner.next().unwrap();

            // Invariant: This while loop should execute twice at most, once if there's a keep/drop and once for target roll
            while let Some(pair) = inner.next() {
                let uint = inner.next().unwrap().as_str().parse::<u64>().expect("Could not parse uint");
                match pair.as_rule() {
                    Rule::keep => roll.keep_highest(uint),
                    Rule::drop => roll.drop_lowest(uint),
                    Rule::gt => roll.target_roll(TargetRoll::GT(uint)),
                    Rule::gte => roll.target_roll(TargetRoll::GTE(uint)),
                    Rule::lt => roll.target_roll(TargetRoll::LT(uint)),
                    Rule::lte => roll.target_roll(TargetRoll::LTE(uint)),
                    _ => unreachable!(),
                };
            }

            match die_type.as_rule() {
                Rule::normal_die => {
                    roll.sides(die_type.as_str().parse::<u64>().expect("Could not parse number of sides"));
                    roll.roll_dice() as i64
                },
                Rule::custom_die => {
                    let mut inner = die_type.clone().into_inner();
                    let mut sides = vec![];
                    while let Some(side) = inner.next() {
                        sides.push(side.as_str().parse::<i64>().expect("Could not parse custom side"));
                    }
                    crate::roll::roll_custom_dice(num_rolls, &sides)
                },
                _ => unreachable!(),
            }
        },
        _ => unreachable!(),
    };

    let infix = |lhs: i64, op: Pair<Rule>, rhs: i64| match op.as_rule() {
        Rule::plus => lhs + rhs,
        Rule::minus => lhs - rhs,
        Rule::times => lhs * rhs,
        Rule::slash => lhs / rhs,
        _ => unreachable!(),
    };

    PREC_CLIMBER.climb(
        expr,
        primary,
        infix,
    )
}
