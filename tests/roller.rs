// rouler - A container-based system for generating die rolls
// Copyright (C) 2016 by John Berry
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

extern crate rouler;

use rouler::*;

macro_rules! assert_range {
    ($begin:expr => $val:expr => $end:expr) => {
        assert!(($begin <= $val) && ($val <= $end));
    };
}

#[test]
fn roll_dice_within_range() {
    for _ in 0..100 {
        assert_range!(4 => roll_dice("4d6") => 24);
    }
}

#[test]
fn roller_object_within_range() {
    let test_roll = Roller::new("2d6 + 4");

    assert_range!(6 => test_roll.total() => 16);
}

#[test]
fn reroll_changes_value() {
    let mut test_roll = Roller::new("100d100");

    assert_ne!(test_roll.total(), test_roll.reroll())
}

#[test]
fn roll_custom_dice_within_range() {
    for _ in 0..100 {
        assert_range!(10 => roll_dice("2d[5, 6, 7]") => 14)
    }
}

#[test]
fn roll_negative_custom_dice_within_range() {
    for _ in 0..100 {
        assert_range!(-14 => roll_dice("2d[-5, -6, -7]") => -10)
    }
}

#[test]
fn custom_dice_spaces_optional() {
    for _ in 0..100 {
        assert_range!(10 => roll_dice("2d[ 5,6,7 ]") => 14)
    }
}

#[test]
fn negative_dice_negates_roll_value() {
    assert_range!(-18 => Roller::new("-3d6").total() => -3);
}

#[test]
fn num_of_dice_nonzero() {
    assert!(Roller::new("0d6").total() == 0);
}

#[test]
fn non_uint_sides_ignored() {
    assert!(Roller::new("3d-6").total() == 3);
}

#[test]
fn d_is_case_insensitive() {
    assert_range!(1 => Roller::new("1D6").total() => 6);
    assert_range!(1 => Roller::new("1d6").total() => 6);
}

#[test]
fn spaces_not_allowed_in_die_codes() {
    assert!(Roller::new("1 d 6").total() == 1);
}

#[test]
fn rollers_are_iterators() {
    let mut d20 = Roller::new("1d20");
    let mut roll5 = d20.iter().take(5).collect::<Vec<i64>>();

    assert_eq!(5, roll5.len());
    assert_eq!(d20.total(), roll5.pop().unwrap());
}

#[test]
fn roll20_ref_roll_cmd() {
    assert_eq!(Roller::new("/roll 1 + 1").total(), 2);
    assert_eq!(Roller::new("/r 1 + 1").total(), 2);
}

#[test]
#[should_panic(expected = "Failed to parse")]
fn roll20_ref_bad_roll_cmd() {
    assert_ne!(Roller::new("/ro 1 + 1").total(), 2);
}

#[test]
fn roll20_ref_including_addtional_information() {
    assert_range!(6 => Roller::new("/roll 1d20+5 \\ +5 Roll for Initiative").total() => 25);
}

#[test]
fn roll20_ref_drop_keep() {
    assert_range!(5 => Roller::new("/roll 8d10d3").total() => 50); // modified to test smaller range
}

#[test]
fn drop_two() {
    assert_eq!(Roller::new("5d1d2").total(), 3);
    assert_eq!(Roller::new("5d1dl2").total(), 3);

    assert_eq!(Roller::new("5d1D2").total(), 3);
    assert_eq!(Roller::new("5d1DL2").total(), 3);
}

#[test]
fn keep_two() {
    assert_eq!(Roller::new("5d1k2").total(), 2);
    assert_eq!(Roller::new("5d1kh2").total(), 2);

    assert_eq!(Roller::new("5d1K2").total(), 2);
    assert_eq!(Roller::new("5d1KH2").total(), 2);
}

#[test]
fn target_roll() {
    assert_eq!(Roller::new("5d1>=1").total(), 5);
    assert_eq!(Roller::new("5d1>1").total(), 0);
    assert_eq!(Roller::new("5d1>0").total(), 5);

    assert_eq!(Roller::new("5d1<=1").total(), 5);
    assert_eq!(Roller::new("5d1<1").total(), 0);
    assert_eq!(Roller::new("5d1<2").total(), 5);

    assert_range!(0 => Roller::new("10d10kh8>=8").total() => 8);
    assert_eq!(Roller::new("10d10kh8>=1").total(), 8);
}

#[test]
fn gm_commands() {
    assert_range!(35 => Roller::new("12d6 + 10d8kh8 + 15").total() => 151);
    assert_range!(0 => Roller::new("18d20>16").total() => 18);
}

#[test]
fn parens() {
    assert_eq!(Roller::new("(1)").total(), 1);
    assert_eq!(Roller::new("2 * (1+1)").total(), 4);
    assert_eq!(Roller::new("2 * (100d1)").total(), 200);
}