//! Real MSJC08 program; shared bodies live in `msjc06_tests`.
use crate::msjc06_tests::*;
const MSJC08: Spec = Spec {
    id: "MSJC08",
    search: "search-purple-unique",
    color: "紫",
    name: ("梦境行者", "幻梦呓语"),
    subtypes: &["群体", "梦境"],
    cards: &[
        "JC104", "JC102", "JZ67", "JC103", "JC107", "JZ59", "JZ58", "JZ61", "XQ43",
    ],
    full: 27,
    color_count: 12,
    uniques: &["XQ43"],
    hits: ["XQ43", "XQ43", "JC104", "LC23"],
};
#[test]
fn msjc08_original_whole_card_and_finite_existing_search_program() {
    original(&MSJC08);
}
#[test]
fn msjc08_construction_actual_purple_24_25_27_and_society_outside_50() {
    construction(&MSJC08);
}
#[test]
fn msjc08_four_same_societies_real_purple_hits_own_deck_private_choice_public_reveal_fresh_hand() {
    four_searches(&MSJC08);
}
#[test]
fn msjc08_rejections_are_atomic_and_cannot_pay_or_activate_with_teammate() {
    rejections(&MSJC08);
}
#[test]
fn msjc08_initiative_draw_and_rear_paid_no_draw_are_personal_and_separate() {
    initiative_draw(&MSJC08);
}
#[test]
fn msjc08_empty_search_shuffles_and_empty_deck_is_not_draw_elimination() {
    empty_search(&MSJC08);
}
#[test]
fn msjc08_cancelled_paid_frame_retains_payment_exhaustion_and_quota() {
    cancelled_frame(&MSJC08);
}
#[test]
fn msjc08_ready_turn_and_new_instance_keep_quota_restart_clears() {
    quota_lifecycle(&MSJC08);
}
