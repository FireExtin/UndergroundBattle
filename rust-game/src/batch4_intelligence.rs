//! WM044 minimum slice: society attachment and global hidden-entry slow.
//! Contest inspection and Sentinel entry lock are NOT implemented yet.
//! Keep this native WIP out of deck/public admission and whole-card counts.
use crate::{model::*, rules::*};

pub(crate) fn definition() -> Definition {
    attachment_definition(target(
        Zone::Society, EntityKind::Any, Relation::ControlledByActor, Range::Anywhere,
    ))
}

impl Game {
    /// Call only when a concealed character actually enters the board. This
    /// requirement does not reveal its print, emit printed entry triggers, or
    /// repeatedly exhaust an existing hidden card on movement/ready/hide.
    pub(crate) fn apply_hidden_entry_slow(&self, entrant: &mut Card) {
        if entrant.face_down && self.attachments.iter().any(|a| {
            a.card.definition == "WM044"
                && !a.card.face_down
                && self.players.get(a.card.controller).is_some_and(|p| !p.eliminated)
                && self.attachment_host_valid(a)
        }) {
            entrant.exhausted = true;
        }
    }
}
