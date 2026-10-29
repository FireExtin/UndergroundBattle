//! Physical marker placement boundary for WM031's pending implementation.
//! A placement is observed at the write, never inferred from an action's net delta.
use crate::{batch4_markers::MarkerKind, engine::RuleResult, model::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkerPlacement {
    pub turn: u32,
    pub carrier: Card,
    pub kind: MarkerKind,
    pub before: u32,
    /// Actual increase, including the existing saturating-write boundary.
    pub placed: u32,
}
impl Game {
    /// Existing board/attachment callers retain their own target and shield checks.
    /// No event is created for zero, a missing carrier, or saturation at MAX.
    /// Returns every real placement and whether it is this instance's first this turn.
    pub(crate) fn place_board_markers(&mut self, id: &str, kind: MarkerKind, amount: u32)
        -> Option<(MarkerPlacement, bool)>
    {
        let turn = self.turn;
        let c = self.board_mut(id)?;
        let before = kind.count(c);
        let after = before.saturating_add(amount);
        if after == before { return None; }
        *kind.field(c) = after;
        let event = MarkerPlacement { turn, carrier: c.clone(), kind, before, placed: after-before };
        // The receipt is deliberately independent of the carrier's current count:
        // removing the marker later in this same command must not erase the event.
        let first = self.record_marker_placement(event.clone());
        Some((event, first))
    }
    /// Called only after an atomic marker batch has finished writing all carriers.
    pub(crate) fn record_marker_placement(&mut self, event: MarkerPlacement) -> bool {
        self.first_marker_placements.retain(|e| e.turn == self.turn);
        let first = !self.first_marker_placements.iter().any(|e| e.carrier.id == event.carrier.id);
        if first { self.first_marker_placements.push(event); }
        first
    }
    pub(crate) fn validate_marker_placements(&self) -> RuleResult<()> {
        let mut ids = BTreeSet::new();
        for e in &self.first_marker_placements {
            if e.turn != self.turn || e.placed == 0
                || e.before.checked_add(e.placed) != Some(e.kind.count(&e.carrier))
                || e.carrier.owner >= self.players.len() || e.carrier.controller >= self.players.len()
                || !self.mystic_instance_valid(&e.carrier.id) || !ids.insert(&e.carrier.id)
            { return Err("逐次标志放置的首次回执、回合或实体身份损坏".into()); }
            self.mobility_immutable_identity(&e.carrier)?;
        }
        Ok(())
    }
}
