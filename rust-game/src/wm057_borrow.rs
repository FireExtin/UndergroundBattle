//! The user's personal-marker ruling, on the existing broker payment/frame.
//! Borrowing the printed action never changes control or lends its win trigger.
use crate::{
    engine::RuleResult, import20_broker as b, model::*, mystic_contest::ContestOutcome, rules::*,
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BorrowPermission {
    pub source_instance: String,
    pub source_controller: usize,
    pub actor: usize,
    pub turn: u32,
    pub personal_secrecy: u32,
}
fn same<T: Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
pub(crate) fn permission(f: &ResolutionFrame) -> Option<BorrowPermission> {
    f.already_paid.last().and_then(|p| {
        if let PaidCost::WM057Borrowed(p) = p {
            Some(p.clone())
        } else {
            None
        }
    })
}
impl Game {
    pub(crate) fn wm057_borrow_available(&self, actor: usize, c: &Card) -> bool {
        c.definition == "WM057"
            && !c.face_down
            && !c.exhausted
            && self
                .players
                .get(actor)
                .is_some_and(|p| !p.eliminated && p.society_zone.secrecy > 0)
            && self
                .players
                .get(c.controller)
                .is_some_and(|p| !p.eliminated)
            && self.team(actor) != self.team(c.controller)
            && self.board(&c.id).is_some_and(|(_, live)| live == c)
    }
    pub(crate) fn wm057_borrow_command(&self, actor: usize, c: &Card, a: &Action) -> bool {
        a.kind == "activate"
            && a.card_id.as_deref() == Some(c.id.as_str())
            && a.ability_id.as_deref().is_none_or(|k| k == b::ACT)
            && self.wm057_borrow_available(actor, c)
    }
    pub(crate) fn wm057_capture_borrow(
        &self,
        actor: usize,
        s: &SourceSnapshot,
        spec: &AbilitySpec,
        a: &Action,
    ) -> RuleResult<Option<BorrowPermission>> {
        if s.card.definition != "WM057" || spec.key != b::ACT || s.card.controller == actor {
            return Ok(None);
        }
        b::validate_action("WM057", a)?;
        b::validate_ability("WM057", spec)?;
        if !self.wm057_borrow_command(actor, &s.card, a) {
            return Err("借用信息掮客须由发动者本人秘社持有隐秘标志".into());
        }
        Ok(Some(BorrowPermission {
            source_instance: s.card.id.clone(),
            source_controller: s.card.controller,
            actor,
            turn: self.turn,
            personal_secrecy: self.players[actor].society_zone.secrecy,
        }))
    }
    pub(crate) fn wm057_validate_borrow(
        &self,
        f: &ResolutionFrame,
        p: &BorrowPermission,
    ) -> RuleResult<()> {
        let c = &f.source.card;
        if c.definition != "WM057"
            || f.ability_key != b::ACT
            || f.actor >= self.players.len()
            || c.controller >= self.players.len()
            || self.team(f.actor) == self.team(c.controller)
            || c.face_down
            || c.exhausted
            || p.source_instance != c.id
            || p.source_controller != c.controller
            || p.actor != f.actor
            || p.turn != self.turn
            || p.personal_secrecy == 0
        {
            return Err("信息掮客借用须保留原来源、发动者本人隐秘资格及原费用见证".into());
        }
        Ok(())
    }
    pub(crate) fn wm057_outcome_authority(&self, o: &ContestOutcome) -> RuleResult<()> {
        if o.source.card.controller == o.actor {
            if o.borrow_permission.is_some() {
                return Err("本人隐秘对抗不能夹带敌方借用凭据".into());
            }
            return Ok(());
        }
        let p = o
            .borrow_permission
            .as_ref()
            .ok_or("借用对抗结果须保留原本人隐秘资格")?;
        if o.source.card.definition != "WM057"
            || o.ability_key != b::ACT
            || o.actor_bonus != 0
            || p.actor != o.actor
            || p.source_controller != o.source.card.controller
            || p.source_instance != o.source.card.id
            || p.turn != o.turn
            || p.personal_secrecy == 0
            || o.source.card.exhausted
            || o.source.card.face_down
            || p.source_controller >= self.players.len()
            || self.team(o.actor) == self.team(p.source_controller)
        {
            return Err("借用对抗结果不能改本人资格、原操控者或程序".into());
        }
        Ok(())
    }
    pub(crate) fn wm057_bind_outcome(
        &self,
        f: &ResolutionFrame,
        o: &ContestOutcome,
    ) -> RuleResult<()> {
        if !same(&permission(f), &o.borrow_permission) {
            return Err("对抗结果须保留同一实际付费帧的借用资格".into());
        }
        Ok(())
    }
}
