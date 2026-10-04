mod attachment;
mod attributes;
pub mod catalog;
mod control;
#[cfg(test)]
mod control_tests;
pub mod deck;
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
mod play_sources;
#[cfg(test)]
mod protection_tests;
#[cfg(test)]
mod defence_equipment_tests;
mod renown;
#[cfg(test)]
mod renown_tests;
mod resolution;
pub mod room;
pub mod rules;
#[cfg(feature = "native")]
pub mod service;
pub mod society;
mod world;

#[cfg(all(test, not(feature = "society-fixtures")))]
mod yellow_search_batch_tests;
