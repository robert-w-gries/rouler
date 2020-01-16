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
        Rule::uint => pair.as_str().parse::<u64>().unwrap() as i64,
        Rule::int => pair.as_str().parse::<i64>().unwrap().into(),
        Rule::expr => compute(pair.into_inner()),
        Rule::roll => {
            let mut inner = pair.into_inner();
            let num_rolls: u64 = {
                let rolls = inner.next().unwrap();
                rolls.as_str().parse::<u64>().expect("Could not parse number of rolls")
            };
            let die_type = inner.next().unwrap();
            let keep_highest: u64 = match inner.next() {
                None => num_rolls,
                Some(rule) => {
                    let parsed = {
                        let parsed = inner.next().unwrap().as_str().parse::<u64>().expect("Could not parse keep/drop number");
                        // Ensure we aren't dropping/keeping more than specified number of rolls
                        if parsed > num_rolls {
                            num_rolls
                        } else {
                            parsed
                        }
                    };

                    match rule.as_rule() {
                        Rule::keep => parsed,
                        Rule::drop => num_rolls - parsed,
                        _ => unreachable!(),
                    }
                },
            };

            match die_type.as_rule() {
                Rule::uint => {
                    let num_sides = die_type.as_str().parse::<u64>().expect("Could not parse number of sides");
                    roll_dice_raw(num_rolls, num_sides, keep_highest) as i64
                },
                Rule::custom_die => {
                    let mut inner = die_type.clone().into_inner();
                    let mut sides = vec![];
                    while let Some(side) = inner.next() {
                        sides.push(side.as_str().parse::<i64>().expect("Could not parse custom side"));
                    }
                    roll_custom_dice_raw(num_rolls, &sides)
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
