//! Closed BQ095 foundation. Not yet wired to Game, admission or card play.
//! A headquarters is a distinct zone, never an alias for the physical grave.
//! Entry dispatch, repeated creation and whole-engine consumers must be wired
//! before this module can be used by the live rules interpreter.
#![allow(dead_code)]
use crate::{engine::RuleResult, model::{Card, Game}};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub(crate) const ID: &str = "BQ095";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Headquarters {
    pub id: String,
    pub seat: usize,
    pub source_instance: String,
    pub cards: Vec<Card>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Destination {
    Graveyard(usize),
    Headquarters(String),
}

impl Headquarters {
    /// Caller must dispatch the actual resolved BQ095 entry. This primitive
    /// only implements the first-created zone;
    /// it makes no ruling about repeated entries creating same-named zones.
    pub(crate) fn first_entry(
        game: &mut Game,
        source: &Card,
        host: &str,
    ) -> RuleResult<Self> {
        if source.definition != ID
            || source.face_down
            || host != format!("society:p{}", source.controller)
            || source.id.is_empty()
            || source.controller >= game.players.len()
        {
            return Err("BQ095总部创建须有实际正面来源、本方秘社与独立区域身份".into());
        }
        let mut zone = Self {
            // Replaced by a fresh engine identity after all input validation.
            id: String::new(),
            seat: source.controller,
            source_instance: source.id.clone(),
            cards: vec![],
        };
        // Validate before moving anything: failure must leave the grave intact.
        zone.validate_cards(&game.players[source.controller].graveyard)?;
        zone.id = game.id();
        let mut cards=vec![];while !game.players[source.controller].graveyard.is_empty(){cards.push(game.take_grave_observed(source.controller,0));}
        zone.cards = cards.into_iter().map(|c| game.reset_zone_card(c)).collect();
        Ok(zone)
    }

    pub(crate) fn validate(&self) -> RuleResult<()> {
        if self.id.is_empty() || self.source_instance.is_empty() || self.id == self.source_instance {
            return Err("BQ095总部缺少独立区域或原来源身份".into());
        }
        self.validate_cards(&self.cards)
    }

    fn validate_cards(&self, cards: &[Card]) -> RuleResult<()> {
        let mut ids = BTreeSet::new();
        if cards.iter().any(|c| {
            c.owner != self.seat || c.id.is_empty() || c.id == self.id
                || c.id == self.source_instance || !ids.insert(&c.id)
        }) {
            return Err("BQ095总部迁移须保持原墓地牌的拥有者和唯一实体".into());
        }
        Ok(())
    }

    /// Only an owned card's printed *own graveyard* reference is rewritten.
    /// Controller changes cannot transfer ownership of the lasting modifier.
    pub(crate) fn own_printed_reference(&self, source: &Card) -> Destination {
        if source.owner == self.seat {
            Destination::Headquarters(self.id.clone())
        } else {
            Destination::Graveyard(source.controller)
        }
    }

    /// Generic/opponent/all-grave references remain physical grave references.
    pub(crate) fn ordinary_grave_reference(seat: usize) -> Destination {
        Destination::Graveyard(seat)
    }

    /// Destination replacement applies independently of the effect's owner.
    /// The caller retains death/discard events, but must not emit a physical
    /// grave-arrival event when the resolved destination is headquarters.
    pub(crate) fn incoming_destination(&self, grave_seat: usize) -> Destination {
        if grave_seat == self.seat {
            Destination::Headquarters(self.id.clone())
        } else {
            Destination::Graveyard(grave_seat)
        }
    }

    /// Receive a card after the existing caller has applied its zone reset.
    /// This never copies a card into a second container or changes its identity.
    pub(crate) fn receive(&mut self, card: Card) -> RuleResult<()> {
        self.validate()?;
        self.validate_cards(std::slice::from_ref(&card))?;
        if self.cards.iter().any(|old| old.id == card.id) {
            return Err("BQ095总部不能重复接收同一实体".into());
        }
        self.cards.push(card);
        Ok(())
    }

    /// The caller still checks the printed target's kind/subtype/cost and any
    /// bound-frame guard. This removes only the resolved zone's actual card.
    pub(crate) fn take_for_own_printed_reference(
        &mut self,
        source: &Card,
        id: &str,
    ) -> RuleResult<Card> {
        self.validate()?;
        if self.own_printed_reference(source) != Destination::Headquarters(self.id.clone()) {
            return Err("非本方拥有的牌不能借墓地引用访问集团总部".into());
        }
        let index = self.cards.iter().position(|c| c.id == id)
            .ok_or("BQ095总部原目标已失效")?;
        Ok(self.cards.remove(index))
    }
}
