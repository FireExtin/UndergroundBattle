//! XQ19: printed dash compares as0 by explicit user ruling. Normal face-up
//! play and paid reveal stay forbidden; the hand action places, not plays.
use crate::{engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub(crate) const HAND: &str = "xq19-hand-entry";
pub(crate) const BATCH20: &[&str] = &[
    "BQ090", "BQ064", "BQ039", "WM028", "WM025", "WM024", "BQ081", "WM057", "XG13", "WM064",
    "CQ24", "BQ092", "BQ093", "CQ18", "CQ19", "XG20", "BQ057", "CQ15", "CQ28", "XQ19",
];
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimeWitness {
    pub instance: String,
    pub amount: u32,
}
fn same<T: Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
pub(crate) fn definition() -> Definition {
    let mut hand = ability(
        HAND,
        "手牌行动0：场上至少4时间标志时，将原手牌放置进地区一",
        Timing::Standard,
        vec![Cost::Assets(0)],
        vec![],
        vec![Op::XQ19FromHand { witnesses: None }],
        None,
    );
    hand.activation_only = true;
    let mut d = with_abilities(vec![hand]);
    d.traits.spirit = true;
    d.traits.kill = 3;
    d
}
fn uses(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::XQ19FromHand { .. } => true,
        Op::ForEachLivingPlayer(b) | Op::ForEachLivingPlayerFromActor(b) => uses(b),
        Op::IfTargetExhausted {
            exhausted, ready, ..
        } => uses(std::slice::from_ref(exhausted)) || uses(std::slice::from_ref(ready)),
        _ => false,
    })
}
pub(crate) fn related(id: &str, a: &AbilitySpec) -> bool {
    id == "XQ19" || a.key == HAND || uses(&a.ops) || a.modes.iter().any(|m| uses(&m.ops))
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    let mut expected = definition().abilities.remove(0);
    if let [Op::XQ19FromHand { witnesses }] = a.ops.as_slice() {
        expected.ops = vec![Op::XQ19FromHand {
            witnesses: witnesses.clone(),
        }];
    }
    if id != "XQ19" || !same(a, &expected) {
        return Err("熵力化身只准入完整手牌0费条件放置程序".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if (id == "XQ19" || d.abilities.iter().any(|a| related(id, a)))
        && (id != "XQ19" || !same(d, &definition()))
    {
        return Err("熵力化身手牌条件、灵体与杀伤3不能拆改或移植".into());
    }
    Ok(())
}
impl Game {
    fn xq19_time_witnesses(&self) -> Vec<TimeWitness> {
        // Count physical in-play cards exactly once. Inherited host counters are not
        // a second copy; hands/decks/graves/sealed/score/world are outside play.
        let cards = self
            .regions
            .iter()
            .filter(|r| !r.vacant)
            .flat_map(|r| std::iter::once(&r.card).chain(&r.cards))
            .chain(
                self.attachments
                    .iter()
                    .filter(|a| self.attachment_host_valid(a))
                    .map(|a| &a.card),
            )
            .chain(
                self.players
                    .iter()
                    .flat_map(|p| p.assets.iter().chain(p.society_zone.card.iter())),
            );
        let mut seen = BTreeSet::new();
        cards
            .filter(|c| c.time_markers > 0 && seen.insert(c.id.clone()))
            .map(|c| TimeWitness {
                instance: c.id.clone(),
                amount: c.time_markers,
            })
            .collect()
    }
    pub(crate) fn xq19_begin_hand(&mut self, actor: usize, a: &Action) -> RuleResult<()> {
        if *a
            != (Action {
                card_id: a.card_id.clone(),
                ability_id: Some(HAND.into()),
                ..Action::new("activate")
            })
        {
            return Err("熵力化身手牌行动不接受地区、额外目标或费用载荷".into());
        }
        let c = self
            .players
            .get(actor)
            .filter(|p| !p.eliminated)
            .and_then(|p| {
                p.hand.iter().find(|c| {
                    Some(c.id.as_str()) == a.card_id.as_deref()
                        && c.definition == "XQ19"
                        && c.controller == actor
                        && !c.face_down
                })
            })
            .ok_or("须由熵力化身原手牌持有者发动")?
            .clone();
        let source = self.source_snapshot(&c, None);
        let witnesses = self.xq19_time_witnesses();
        if witnesses.iter().map(|w| u64::from(w.amount)).sum::<u64>() < 4 {
            return Err("熵力化身只能在场上有四个或更多时间标志时发动手牌能力".into());
        }
        if !self.region_live(0) {
            return Err("地区一当前没有可放置的地区".into());
        }
        let mut spec = definition().abilities.remove(0);
        spec.ops = vec![Op::XQ19FromHand {
            witnesses: Some(witnesses),
        }];
        self.check_timing(actor, &spec)?;
        let paid = self.pay_ability_costs(actor, &source, &spec, a)?;
        let f = self.make_frame(actor, source, &spec, vec![], paid, None);
        self.dispatch_frame(f, spec.label, None, spec.response_policy);
        Ok(())
    }
    pub(crate) fn validate_xq19_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if self.validated_cross_card_frame(f)? {
            return Ok(());
        }
        let relevant = f.ability_key == HAND
            || f.steps.iter().any(|s| uses(std::slice::from_ref(&s.op)))
            || f.source.card.definition == "XQ19"
                && matches!(f.ability_key.as_str(), "deploy" | "reveal");
        if !relevant {
            return Ok(());
        }
        let [Step {
            context,
            op: Op::XQ19FromHand { witnesses: Some(w) },
        }] = f.steps.as_slice()
        else {
            return Err("熵力化身不能正面打出或付费现身；手牌放置须保留发动时条件".into());
        };
        let c = &f.source.card;
        let mut ids = BTreeSet::new();
        if f.actor >= self.players.len()
            || c.definition != "XQ19"
            || c.owner >= self.players.len()
            || c.controller != f.actor
            || c.face_down
            || f.source.region.is_some()
            || f.source.source_region_instance.is_some()
            || f.source.attachment_host_instance.is_some()
            || f.source.play_source.is_some()
            || f.ability_key != HAND
            || f.chosen_region.is_some()
            || !f.targets.is_empty()
            || *context != f.actor
            || f.frame_id.is_empty()
            || f.cursor > 1
            || f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked)
            || f.cursor == 1 && !matches!(f.guard, GuardState::Accepted)
            || !matches!(f.already_paid.as_slice(),[PaidCost::Assets(a)]if a.is_empty())
            || !f.resource_payments.is_empty()
            || w.is_empty()
            || w.iter().any(|v| {
                v.amount == 0
                    || v.instance == c.id
                    || !ids.insert(&v.instance)
                    || !self
                        .mystic_instance_number(&v.instance)
                        .zip(self.mystic_instance_number(&f.frame_id))
                        .is_some_and(|(id, frame)| id < frame)
            })
            || w.iter().map(|w| u64::from(w.amount)).sum::<u64>() < 4
        {
            return Err("熵力化身须保留原手牌、实际0费、原发动时四时间门槛与标准行动帧".into());
        }
        self.mobility_immutable_identity(c)
    }
    pub(crate) fn xq19_place_hand(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        self.validate_xq19_frame(f)?;
        if f.ability_key != HAND || f.cursor != 1 || !matches!(f.guard, GuardState::Accepted) {
            return Err("熵力化身必须经过正常响应堆叠".into());
        }
        if self.players[f.actor].eliminated || !self.region_live(0) {
            return Ok(());
        }
        let Some(i) = self.players[f.actor]
            .hand
            .iter()
            .position(|c| c.id == f.source.card.id && c.definition == "XQ19")
        else {
            return Ok(());
        };
        let c = self.players[f.actor].hand.remove(i);
        let mut c = self.reset_zone_card(c);
        c.controller = f.actor;
        let id = c.id.clone();
        self.regions[0].cards.push(c);
        self.enter_triggers(f.actor, "XQ19", &id, false);
        Ok(())
    }
    pub(crate) fn validate_xq19_state(&self) -> RuleResult<()> {
        for item in &self.stack {
            if let Some(f) = &item.frame {
                self.validate_xq19_frame(f)?;
                if f.ability_key == HAND
                    && (item.id != f.frame_id
                        || item.controller != f.actor
                        || item.card.is_some()
                        || item.reveal
                        || item.deploy_region.is_some()
                        || item.target.is_some()
                        || f.cursor != 0)
                {
                    return Err("熵力化身手牌行动响应外壳无效".into());
                }
            }
        }
        for e in &self.effects {
            if let Effect::Frame { frame } = e {
                self.validate_xq19_frame(frame)?;
            }
        }
        Ok(())
    }
}
