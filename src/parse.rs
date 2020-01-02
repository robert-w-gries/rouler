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
use random::*;

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
        Rule::int => pair.as_str().parse::<u64>().unwrap() as i64,
        Rule::number => pair.as_str().parse::<i64>().unwrap().into(),
        Rule::expr => compute(pair.into_inner()),
        Rule::roll => {
            let mut inner = pair.into_inner();
            let num_rolls = {
                let rolls = inner.next().unwrap();
                rolls.as_str().parse::<i64>().expect("Could not parse number of rolls")
            };
            let die_type = inner.next().unwrap();
            match die_type.as_rule() {
                Rule::int => {
                    let num_sides = die_type.as_str().parse::<i64>().expect("Could not parse number of sides");
                    num_rolls.signum() * roll_dice_raw(num_rolls.abs(), num_sides as u64)
                },
                Rule::custom_die => {
                    let mut inner = die_type.clone().into_inner();
                    let mut sides = vec![];
                    while let Some(side) = inner.next() {
                        sides.push(side.as_str().parse::<u64>().expect("Could not parse custom side"));
                    }
                    num_rolls.signum() * roll_custom_dice_raw(num_rolls.abs(), &sides)
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
