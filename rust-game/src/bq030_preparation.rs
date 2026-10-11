//! Preparation only: snapshot collection, without admission or event dispatch.
//! The caller's eventual commit point, control/death ordering and simultaneous
//! declaration ordering require the parent-coordinated attachment hook.
use crate::{catalog, model::*};

#[allow(dead_code)]
pub(crate) struct AttachmentObservation {
    pub attachment_instance: String,
    pub host_instance: String,
    pub host_controller: usize,
    // Discovery order is not declaration/stack order. The controller must be
    // allowed to order simultaneous triggers before these become declarations.
    pub observers: Vec<SourceSnapshot>,
}

impl Game {
    #[allow(dead_code)]
    pub(crate) fn snapshot_attachment_observers(
        &self,
        attachment_id: &str,
    ) -> Option<AttachmentObservation> {
        let attachment = self
            .attachments
            .iter()
            .find(|a| a.card.id == attachment_id)?;
        if attachment.card.face_down || !self.attachment_host_valid(attachment) {
            return None;
        }
        let (_, host) = self.board(&attachment.host_id)?;
        if host.face_down || catalog::card(&host.definition).kind != "character" {
            return None;
        }
        let actor = host.controller;
        if self.players.get(actor).is_none_or(|p| p.eliminated) {
            return None;
        }
        let observers = self
            .regions
            .iter()
            .enumerate()
            .flat_map(|(r, region)| {
                region
                    .cards
                    .iter()
                    .filter(move |c| {
                        c.definition == "BQ030" && !c.face_down && c.controller == actor
                    })
                    // BQ030 has no region-bound, attachment-host, death or play
                    // context. Avoid the admitted registry's source_snapshot
                    // while this collector is still an unregistered proposal.
                    .map(move |c| SourceSnapshot {
                        card: c.clone(),
                        region: Some(r),
                        observed_death: None,
                        source_region_instance: None,
                        attachment_host_instance: None,
                        play_source: None,
                    })
            })
            .collect();
        Some(AttachmentObservation {
            attachment_instance: attachment.card.id.clone(),
            host_instance: host.id.clone(),
            host_controller: actor,
            observers,
        })
    }
}

#[cfg(all(test, not(feature = "society-fixtures")))]
mod tests {
    use super::*;
    use crate::jc029_tests::{board, game};

    fn prepared() -> (Game, String, String) {
        let mut g = game(0);
        let host = board(&mut g, "XQ12", 0, 0);
        let card = g.make_card("BQ040", 2);
        let attachment = card.id.clone();
        g.attachments.push(Attachment {
            card,
            host_id: host.clone(),
        });
        (g, host, attachment)
    }
    fn observer(g: &mut Game, owner: usize, controller: usize, region: usize) -> String {
        // Explicit collector-contract fixture, not an admitted card/Room state.
        // Only the identity, controller and visibility are under this contract.
        let mut card = g.make_card("JZ22", owner);
        card.definition = "BQ030".into();
        card.controller = controller;
        let id = card.id.clone();
        g.regions[region].cards.push(card);
        id
    }

    #[test]
    fn controller_not_owner_team_or_attachment_controller_and_no_region_limit() {
        let (mut g, host, attachment) = prepared();
        let qualified = observer(&mut g, 1, 0, 4);
        observer(&mut g, 2, 1, 0); // Same team does not mean 本方.
        observer(&mut g, 0, 2, 0); // Ownership does not mean control.
        let event = g.snapshot_attachment_observers(&attachment).unwrap();
        assert_eq!(event.attachment_instance, attachment);
        assert_eq!(event.host_instance, host);
        assert_eq!(event.host_controller, 0);
        assert_eq!(event.observers.len(), 1);
        assert_eq!(event.observers[0].card.id, qualified);
        assert_eq!(event.observers[0].region, Some(4));
    }

    #[test]
    fn hidden_observers_excluded_exhausted_observers_included_and_actor_frozen() {
        let (mut g, _, attachment) = prepared();
        let hidden = observer(&mut g, 0, 0, 1);
        g.board_mut(&hidden).unwrap().face_down = true;
        let visible = observer(&mut g, 0, 0, 2);
        g.board_mut(&visible).unwrap().exhausted = true;
        let event = g.snapshot_attachment_observers(&attachment).unwrap();
        assert_eq!(event.observers.len(), 1);
        assert_eq!(event.observers[0].card.id, visible);
        g.board_mut(&visible).unwrap().controller = 2;
        assert_eq!(event.observers[0].card.controller, 0);
    }

    #[test]
    fn missing_attachment_missing_host_or_hidden_host_has_no_snapshot() {
        let (mut g, host, attachment) = prepared();
        assert!(g.snapshot_attachment_observers("missing").is_none());
        g.board_mut(&host).unwrap().face_down = true;
        assert!(g.snapshot_attachment_observers(&attachment).is_none());
        g.regions[0].cards.clear();
        assert!(g.snapshot_attachment_observers(&attachment).is_none());
    }

    #[test]
    fn multiple_observers_collected_without_prescribing_stack_order() {
        let (mut g, _, attachment) = prepared();
        let a = observer(&mut g, 0, 0, 4);
        let b = observer(&mut g, 0, 0, 1);
        let event = g.snapshot_attachment_observers(&attachment).unwrap();
        let ids: std::collections::BTreeSet<_> =
            event.observers.iter().map(|s| s.card.id.clone()).collect();
        assert_eq!(ids, [a, b].into_iter().collect());
        assert!(g.effects.is_empty()); // Collection never declares/resolves.
    }
}
