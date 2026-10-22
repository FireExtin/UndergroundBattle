mod chaos;
mod batch4;
mod batch4_conference;
#[cfg(test)]
mod batch4_conference_tests;
mod batch4_plans;
#[cfg(test)]
mod batch4_plans_tests;
mod batch4_tattoo;
mod batch4_shape;
mod batch4_dahlia;
mod batch4_feast;
mod batch4_statue;
#[cfg(test)]
mod batch4_statue_tests;
#[cfg(test)]
mod batch4_feast_tests;
#[cfg(test)]
mod batch4_dahlia_tests;
#[cfg(test)]
mod batch4_shape_tests;
#[cfg(test)]
mod batch4_tattoo_tests;
#[cfg(test)]
mod batch4_tests;
mod batch3;
mod batch3_phoebe;
#[cfg(test)]
mod batch3_phoebe_tests;
#[cfg(test)]
mod batch3_abi94_tests;
mod batch3_bats;
mod batch3_trap;
#[cfg(test)]
mod batch3_trap_tests;
#[cfg(test)]
mod batch3_bats_tests;
mod batch3_entry_order;
#[cfg(test)]
mod batch3_guard_tests;
mod batch3_blood;
#[cfg(test)]
mod batch3_blood_tests;
mod batch3_warrant;
#[cfg(test)]
mod batch3_warrant_tests;
mod batch3_alpha;
#[cfg(test)]
mod batch3_alpha_tests;
mod batch3_ghoul;
#[cfg(test)]
mod batch3_ghoul_tests;
mod batch3_red;
mod batch3_black;
mod batch3_host;
mod batch3_grave;
mod batch3_inspect;
mod batch3_politician;
mod batch3_law;
#[cfg(test)]
mod batch3_serpent_tests;
#[cfg(test)]
mod batch3_inspect_tests;
#[cfg(test)]
mod batch3_grave_tests;
mod exposure;
mod rumor;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod next20_cross_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod exposure_tests;
mod attachment;
mod region_destruction;
mod batch20;
mod next20_red;
#[cfg(test)]
mod next20_red_tests;
mod jc065;
mod extra_influence;
mod batch20_observers;
mod raid;
mod next20_blue_gray;
#[cfg(test)]
mod next20_blue_gray_tests;
#[cfg(test)]
mod batch20_tests;
mod bq035;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod bq035_tests;
mod mobility;
mod bq030;
mod red_time;
mod blue_expansion;
mod black_expansion;
mod wm056;
mod wm062;
mod jc054;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod jc054_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod wm062_tests;
mod wm023;
mod xq25;
mod jz46;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod jz46_tests;
mod wm038;
mod jz35;
mod jc052;
mod jc046;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod jc046_tests;
mod jc037;
mod jc060;
mod jc057;
mod jc044;
mod jc066;
mod jc041;
mod jc035;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod jc035_tests;
mod xq23;
#[cfg(test)]
mod xq23_tests;
#[cfg(test)]
mod jc041_tests;
#[cfg(test)]
mod jc066_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod jc044_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod jc057_tests;
mod jc095;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod jc095_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod jc060_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod jc037_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod jc052_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod jz35_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod wm038_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod xq25_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod wm023_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod wm056_tests;
mod gray_expansion;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod red_time_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod xq18_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod blue_expansion_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod black_expansion_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod gray_expansion_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod jc061_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod jc087_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod jc094_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod xq24_tests;
mod attributes;
mod jc050;
mod xq27;
mod xq37;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod xq37_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod win_flow_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod xq27_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod jc050_tests;
pub mod catalog;
mod control;
mod death_observers;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod death_observer_tests;
#[cfg(test)]
mod control_tests;
pub mod deck;
#[cfg(test)]
mod unit_support;
#[cfg(test)]
mod defence_equipment_tests;
pub mod engine;
#[cfg(test)]
mod jc004_tests;
#[cfg(test)]
mod jc005_tests;
#[cfg(test)]
mod jc005_society_regression_tests;
#[cfg(test)]
mod jc008_tests;
pub mod model;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod msjc01_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod msjc06_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod msjc07_tests;
mod play_sources;
#[cfg(test)]
mod protection_tests;
mod renown;
#[cfg(test)]
mod renown_tests;
mod resolution;
mod sealing;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod sealing_tests;
mod jz50_search;
mod bounded_search;
mod deck_seal_search;
mod gray_lock;
mod jz22;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod jz22_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod gray_lock_tests;
// Retain the historical test-only positive seal preparation alongside actual-card tests.
#[cfg(all(test, not(feature = "society-fixtures")))]
mod deck_seal_search_draft;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod deck_seal_search_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod bounded_search_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod bounded_entry_fresh_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod jz50_search_tests;
pub mod room;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod response_preview_tests;
pub mod rules;
#[cfg(feature = "native")]
pub mod service;
pub mod society;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod society_zone_tests;
#[cfg(test)]
mod white_wound_tests;
mod world;

#[cfg(all(test, not(feature = "society-fixtures")))]
mod yellow_search_batch_tests;

#[cfg(test)]
mod equipment_abilities_tests;

#[cfg(test)]
mod hand_deck_tests;

#[cfg(test)]
mod hand_interactions_tests;

#[cfg(test)]
mod purple_tools_tests;
#[cfg(test)]
mod wound_defence_tests;

#[cfg(test)]
mod dream_reveal_tests;

#[cfg(test)]
mod repress_assets_tests;

#[cfg(all(test, not(feature = "society-fixtures")))]
mod region_aura_tests;

#[cfg(all(test, not(feature = "society-fixtures")))]
mod msjc08_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod msjc11_tests;

#[cfg(all(test, not(feature = "society-fixtures")))]
mod msjc0405_tests;

#[cfg(all(test, not(feature = "society-fixtures")))]
mod jc029_tests;

#[cfg(all(test, not(feature = "society-fixtures")))]
mod green_minimum_tests;

#[cfg(all(test, not(feature = "society-fixtures")))]
mod jc030_tests;
mod blue_minimum;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod blue_minimum_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod mill_public_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod jz31_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod jz55_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod jz49_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod jz48_tests;
#[cfg(all(test, not(feature = "society-fixtures")))]
mod jc089_tests;

mod black_next20;
#[cfg(test)]
mod black_next20_tests;
