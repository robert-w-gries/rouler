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
use roll::{DieType, Roll, TargetRoll};

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

            // Loop through the nested die rules
            let mut inner_die = inner.next().unwrap().clone().into_inner();
            while let Some(pair) = inner_die.next() {
                match pair.as_rule() {
                    Rule::num_rolls => {
                        roll.count(pair.as_str().parse::<u64>().expect("Could not parse number of rolls"));
                    },
                    Rule::normal_die => {
                        roll.sides(pair.as_str().parse::<u64>().expect("Could not parse number of sides"));
                        roll.die_type(DieType::Normal);
                    },
                    Rule::custom_die => {
                        let mut inner = pair.clone().into_inner();
                        let mut sides = vec![];
                        while let Some(side) = inner.next() {
                            sides.push(side.as_str().parse::<i64>().expect("Could not parse custom side"));
                        }
                        roll.add_custom_sides(&sides);
                        roll.die_type(DieType::Custom);
                    },
                    _ => unreachable!(),
                }
            }

            // Invariant: This while loop should execute twice at most
            // Once if there's a keep/drop and once if there's a target roll
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

            roll.roll_dice()
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
