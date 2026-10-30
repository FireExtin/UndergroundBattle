//! Closed BQ095 persistent-zone WIP. Card entry and reference consumers remain unwired.
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
pub struct Headquarters {
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


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeadquartersView {
    pub id: String,
    pub player_id: String,
    pub name: String,
    pub cards: Vec<crate::model::CardView>,
}

impl Game {
    /// Internal resolved-entry primitive, not a public action. Until the full
    /// printed-reference integration lands, no card dispatch invokes it.
    /// Project ruling: repeated entry reuses this society's one lasting zone.
    pub(crate) fn bq095_create_or_reuse(&mut self, source: &Card, host: &str) -> RuleResult<String> {
        if source.definition != ID || source.face_down || source.controller >= self.players.len()
            || host != format!("society:p{}",source.controller)
            || !self.mystic_instance_valid(&source.id) {
            return Err("总部进场来源或本方秘社无效".into());
        }
        self.validate_bq095_state()?;
        if let Some(zone)=self.bq095_headquarters.iter().find(|z|z.seat==source.controller) {
            // Destination replacement will keep the physical grave empty.
            // Never destroy/merge unexplained physical contents on re-entry.
            if !self.players[source.controller].graveyard.is_empty() {
                return Err("已有总部仍存在物理墓地牌，不能重进场掩盖损坏".into());
            }
            return Ok(zone.id.clone());
        }
        if self.players[source.controller].graveyard.iter().any(|c|
            self.batch3_physical_cards().filter(|live|live.id==c.id).count()!=1) {
            return Err("总部迁移前实体已重复，不能用新实例掩盖原损坏".into());
        }
        let mut next=self.clone();
        let zone=Headquarters::first_entry(&mut next,source,host)?;
        let id=zone.id.clone();next.bq095_headquarters.push(zone);
        next.validate_bq095_state()?;
        *self=next;Ok(id)
    }

    /// Resolve owned printed "own graveyard" independently of generic/enemy
    /// graveyard access. The supplied source is the original frozen card.
    pub(crate) fn bq095_own_reference(&self, source:&Card) -> Destination {
        self.bq095_headquarters.iter().find(|z|z.seat==source.owner)
            .map_or(Destination::Graveyard(source.controller),|z|z.own_printed_reference(source))
    }
    pub(crate) fn bq095_destination(&self,seat:usize)->Destination {
        self.bq095_headquarters.iter().find(|z|z.seat==seat)
            .map_or(Destination::Graveyard(seat),|z|Destination::Headquarters(z.id.clone()))
    }
    pub(crate) fn bq095_reference_cards(&self,source:&Card)->&[Card] {
        match self.bq095_own_reference(source) {
            Destination::Headquarters(id)=>&self.bq095_headquarters.iter().find(|z|z.id==id).unwrap().cards,
            Destination::Graveyard(seat)=>&self.players[seat].graveyard,
        }
    }
    pub(crate) fn bq095_commit_destination(&mut self,card:Card) {
        if let Some(z)=self.bq095_headquarters.iter_mut().find(|z|z.seat==card.owner) {
            z.cards.push(card);
        } else {self.players[card.owner].graveyard.push(card);}
    }
    pub(crate) fn bq095_views(&self,viewer:usize)->Vec<HeadquartersView> {
        self.bq095_headquarters.iter().map(|z|HeadquartersView {
            id:z.id.clone(),player_id:crate::model::player_id(z.seat),name:"集团总部".into(),
            cards:z.cards.iter().map(|c|self.card_view(c,viewer,None,Some("headquarters"))).collect(),
        }).collect()
    }
    pub(crate) fn validate_bq095_state(&self)->RuleResult<()> {
        let mut seats=BTreeSet::new();let mut zones=BTreeSet::new();
        for z in &self.bq095_headquarters {
            z.validate()?;
            if z.seat>=self.players.len() || !self.players[z.seat].graveyard.is_empty() || !seats.insert(z.seat) || !zones.insert(&z.id)
                || !self.mystic_instance_number(&z.source_instance).zip(self.mystic_instance_number(&z.id)).is_some_and(|(s,i)|s<i)
                || !self.mystic_instance_valid(&z.id)
                || self.batch3_physical_cards().any(|c|c.id==z.id) {
                return Err("总部区域身份、创建来源或唯一归属损坏".into());
            }
            for c in &z.cards {
                if !self.mystic_instance_valid(&c.id) || c.controller!=z.seat || c.face_down || c.exhausted
                    || !crate::catalog::catalog().cards.iter().any(|d|d.id==c.definition)
                    || c.named_character.is_some() || c.chosen_number.is_some() || c.chosen_subtype.is_some() || c.advancement_markers!=0 || c.blood_origin.is_some()
                    || c.generated.is_some() || c.damage!=0 || c.wounds!=0 || c.shield!=0
                    || c.lock_markers!=0 || c.time_markers!=0 || c.charge_markers!=0
                    || c.permanent_combat_markers!=0 || c.permanent_investigation_markers!=0 || c.permanent_defense_markers!=0
                    || self.batch3_physical_cards().filter(|v|v.id==c.id).count()!=1 {
                    return Err("总部实体重复、生成物或跨区状态未清理".into());
                }
                self.mobility_immutable_identity(c)?;
            }
        }
        Ok(())
    }
}
