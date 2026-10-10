//! Real MSJC07 program; shared bodies live in `msjc06_tests`.
use crate::msjc06_tests::*;
const MSJC07: Spec = Spec {
    id: "MSJC07",
    search: "search-black-unique",
    color: "黑",
    name: ("方碑序列", "恐怖同盟"),
    subtypes: &["法师结社"],
    cards: &[
        "JC086", "JC092", "JC091", "JZ54", "JC084", "JC085", "JC088", "BQ083", "JC093", "JC096",
        "XQ38",
    ],
    full: 33,
    color_count: 19,
    uniques: &["JC096", "XQ38"],
    hits: ["JC096", "XQ38", "JC091", "LC23"],
};
#[test]
fn msjc07_original_whole_card_and_finite_existing_search_program() {
    original(&MSJC07);
}
#[test]
fn msjc07_construction_actual_black_24_25_33_and_society_outside_50() {
    construction(&MSJC07);
}
#[test]
fn msjc07_four_same_societies_real_black_hits_own_deck_private_choice_public_reveal_fresh_hand() {
    four_searches(&MSJC07);
}
#[test]
fn msjc07_rejections_are_atomic_and_cannot_pay_or_activate_with_teammate() {
    rejections(&MSJC07);
}
#[test]
fn msjc07_initiative_draw_and_rear_paid_no_draw_are_personal_and_separate() {
    initiative_draw(&MSJC07);
}
#[test]
fn msjc07_empty_search_shuffles_and_empty_deck_is_not_draw_elimination() {
    empty_search(&MSJC07);
}
#[test]
fn msjc07_cancelled_paid_frame_retains_payment_exhaustion_and_quota() {
    cancelled_frame(&MSJC07);
}
#[test]
fn msjc07_ready_turn_and_new_instance_keep_quota_restart_clears() {
    quota_lifecycle(&MSJC07);
}
