//! First serial reference slice. Reuse ordinary payments, target guards and the
//! existing mystic-contest continuation; no second apply or contest kernel.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use std::collections::BTreeSet;

pub(crate) const CARDS: &[&str] = &["BQ090", "BQ064"];
pub(crate) const DESTROY: &str = "import20-bq090-destroy";
pub(crate) const HAND: &str = "import20-bq090-hand-contest";
pub(crate) const EYE_HAND: &str = "import20-bq039-hand-contest";
pub(crate) const SAMPLES: &str = "import20-bq064-take-samples";

pub(crate) fn is_hand_key(key: &str) -> bool { matches!(key, HAND | EYE_HAND) }
pub(crate) fn hand_ability(id: &str) -> Option<AbilitySpec> {
    let key = match id { "BQ090" => HAND, "BQ039" => EYE_HAND, _ => return None };
    let mut hand = ability(key, "手牌行动：支付2并弃自身，进行隐秘对抗", Timing::Standard,
        vec![Cost::Assets(2), Cost::DiscardSelectedHandCard], vec![],
        vec![Op::MysticContest { actor_bonus: 0 }], None);
    hand.activation_only = true;
    Some(hand)
}

fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
pub(crate) fn definition(id: &str) -> Option<Definition> {
    let abilities = match id {
        "BQ090" => {
            let mut target = target(Zone::Board, EntityKind::Character, Relation::Any, Range::Anywhere);
            target.printed_cost_max = Some(3);
            let mut play = ability(DESTROY, "快速行动：消灭印刷费用不大于3的目标角色",
                Timing::Fast, vec![], vec![target], vec![Op::Destroy(EntityRef::Target(0))], None);
            play.play_only = true;
            vec![play, hand_ability(id).unwrap()]
        }
        "BQ064" => {
            let mut play = ability(SAMPLES, "行动：移除本方秘社所有失控标志，每枚抓一张",
                Timing::Standard, vec![], vec![], vec![Op::BQ064TakeSamples], None);
            play.play_only = true;
            vec![play]
        }
        _ => return None,
    };
    Some(with_abilities(abilities))
}
pub(crate) fn owns(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::BQ064TakeSamples => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => owns(body),
        Op::IfTargetExhausted { exhausted, ready, .. } =>
            owns(std::slice::from_ref(exhausted)) || owns(std::slice::from_ref(ready)),
        _ => false,
    })
}
pub(crate) fn related(id: &str, a: &AbilitySpec) -> bool {
    CARDS.contains(&id) || a.key.starts_with("import20-") || owns(&a.ops)
        || a.modes.iter().any(|m| owns(&m.ops))
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if definition(id).and_then(|d| d.abilities.into_iter().find(|b| b.key == a.key))
        .is_some_and(|expected| same(&expected, a)) { Ok(()) }
    else { Err("首批隐秘卡能力须保留原卡完整费用、目标与程序".into()) }
}
pub(crate) fn validate_action(id: &str, a: &Action) -> RuleResult<()> {
    if id == "BQ039" { return crate::import20_observers::validate_action(id, a); }
    if !CARDS.contains(&id) { return Ok(()); }
    let expected = match (id, a.kind.as_str()) {
        ("BQ090", "play") => Action { card_id: a.card_id.clone(), ability_id: a.ability_id.clone(),
            target_id: a.target_id.clone(), ..Action::new("play") },
        ("BQ090", "activate") if a.ability_id.as_deref() == Some(HAND) =>
            Action { card_id: a.card_id.clone(), ability_id: a.ability_id.clone(), ..Action::new("activate") },
        ("BQ064", "play") => Action { card_id: a.card_id.clone(), ability_id: a.ability_id.clone(), ..Action::new("play") },
        _ => return Err("本卡没有该行动程序".into()),
    };
    if *a == expected { Ok(()) } else { Err("本卡行动不可夹带额外目标、费用或选择".into()) }
}
pub(crate) fn admitted_start(id: &str, key: &str, bonus: u32) -> bool {
    matches!((id, key, bonus), ("BQ090", HAND, 0) | ("BQ039", EYE_HAND, 0))
}

impl Game {
    pub(crate) fn import20_begin_hand(&mut self, actor: usize, a: &Action) -> RuleResult<()> {
        let id = match a.ability_id.as_deref() { Some(HAND) => "BQ090", Some(EYE_HAND) => "BQ039", _ => return Err("非准入神秘学博弈手牌能力".into()) };
        validate_action(id, a)?;
        let player = self.players.get(actor).filter(|p| !p.eliminated).ok_or("手牌行动玩家无效")?;
        let c = player.hand.iter().find(|c| Some(c.id.as_str()) == a.card_id.as_deref()
            && c.definition == id && !c.face_down && c.controller == actor)
            .ok_or("神秘学博弈须弃置本人的原能力手牌")?;
        self.mobility_immutable_identity(c)?;
        let source = self.source_snapshot(c, None);
        let spec = hand_ability(id).unwrap();
        self.check_timing(actor, &spec)?;
        let payment = Action { cost_selected: Some(vec![source.card.id.clone()]), ..a.clone() };
        let paid = self.pay_ability_costs(actor, &source, &spec, &payment)?;
        let frame = self.make_frame(actor, source, &spec, vec![], paid, None);
        self.dispatch_frame(frame, spec.label, None, spec.response_policy);
        Ok(())
    }
    pub(crate) fn validate_import20_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        let id = f.source.card.definition.as_str();
        let spec = definition(id).and_then(|d| d.abilities.into_iter().find(|a| a.key == f.ability_key))
            .or_else(|| hand_ability(id).filter(|a| a.key == f.ability_key))
            .ok_or("首批隐秘卡未知程序或来源")?;
        let s = &f.source;
        if f.actor >= self.players.len() || s.card.controller != f.actor || s.card.owner >= self.players.len()
            || s.card.face_down || s.region.is_some() || s.source_region_instance.is_some()
            || s.attachment_host_instance.is_some() || s.play_source.is_some() || s.death_echo.is_some()
            || s.observed_death.is_some() || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some()
            || f.chosen_region.is_some() || f.cursor > 1
            || f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked)
            || f.cursor == 1 && !matches!(f.guard, GuardState::Accepted)
            || !self.mystic_instance_number(&s.card.id).zip(self.mystic_instance_number(&f.frame_id))
                .is_some_and(|(source, frame)| source < frame)
            || f.steps.len() != 1 || f.steps[0].context != f.actor || !same(&f.steps[0].op, &spec.ops[0])
            || f.targets.len() != spec.targets.len()
            || f.targets.iter().zip(&spec.targets).any(|(t, spec)| !same(&t.spec, spec)) {
            return Err("首批隐秘卡须保留原来源、阶段、目标及完整程序".into());
        }
        self.mobility_immutable_identity(&s.card)?;
        let assets = match f.already_paid.as_slice() {
            [PaidCost::Assets(ids), PaidCost::Discarded { old_instance, holder, owner }]
                if is_hand_key(&f.ability_key) && old_instance == &s.card.id && *holder == f.actor
                    && *owner == s.card.owner && ids.len() == 2
                    && !self.batch3_physical_cards().any(|c| c.id == s.card.id) => ids,
            [PaidCost::Assets(ids)] if !is_hand_key(&f.ability_key) && ids.len() <= catalog::card(id).cost as usize => ids,
            _ => return Err("首批隐秘卡须保留实际出牌或支付2并弃自身的费用收据".into()),
        };
        if assets.iter().collect::<BTreeSet<_>>().len() != assets.len()
            || assets.iter().any(|id| *id == s.card.id || !self.mystic_instance_number(id)
                .zip(self.mystic_instance_number(&f.frame_id)).is_some_and(|(asset, frame)| asset < frame)) {
            return Err("首批隐秘卡费用实体重复或无效".into());
        }
        Ok(())
    }
    pub(crate) fn import20_take_samples(&mut self, actor: usize) -> RuleResult<()> {
        // This is the existing physical loss-of-control counter, distinct from
        // exposure, secrecy, destruction or temporary influence. Count removals
        // at resolution, not declaration; clearing zero never draws a card.
        let removed = std::mem::take(&mut self.players[actor].society_zone.loss_control);
        self.draw(actor, removed as usize)
    }
}
