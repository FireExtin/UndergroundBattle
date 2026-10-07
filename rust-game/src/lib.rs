mod attachment;
mod attributes;
pub mod catalog;
mod control;
#[cfg(test)]
mod control_tests;
pub mod deck;
#[cfg(test)]
mod defence_equipment_tests;
pub mod engine;
#[cfg(test)]
mod jc004_tests;
#[cfg(test)]
mod jc005_tests;
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
pub mod room;
pub mod rules;
#[cfg(feature = "native")]
pub mod service;
pub mod society;
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
#[cfg(test)]
mod jz49_tests;
