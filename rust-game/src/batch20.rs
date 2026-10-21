//! Closed, printed programs for the current twenty-card release batch.
//! Printed facts remain disabled in deck construction until their entire card is ready.
use crate::{catalog, engine::RuleResult, model::*, rules::{self, *}};
use std::collections::{BTreeMap, BTreeSet};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttachmentEntryObservation {
    pub event_id: String,
    pub attachment: Attachment,
}

pub(crate) const IMPLEMENTING: &[&str] = &["JZ33", "JZ37", "JC043", "BQ054", "JC048", "WM034", "BQ091", "JC068", "BQ094", "JZ52", "BQ080", "JZ57"];
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unit_support::*;

    #[test]
    fn jz33_face_up_reduction_still_requires_loyalty_and_never_reduces_reveal() {
        let mut g = game();
        let id = held(&mut g, "JZ33", 0);
        g.destroy_all_regions();
        let c = g.players[0].hand[0].clone();
        assert_eq!(g.effective_cost(0, &c), 0);
        assert_eq!(g.paid_reveal_cost(0, &c), 6);
        let action = Action { card_id: Some(id), region: Some(0), ..Action::new("deploy") };
        reject(&mut g, 0, action.clone());
        fund(&mut g, 0, "JC047", 1);
        act(&mut g, 0, action);
        assert_eq!(resources(&g, 0), 1);
        let mut bad = g.clone();
        bad.stack[0].frame.as_mut().unwrap().steps.push(Step { context: 0, op: Op::JZ37DestroyWorld });
        assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    }

    #[test]
    fn jz33_forced_entry_hits_only_face_up_characters_in_destroyed_regions() {
        let mut g = game();
        g.regions[0].destroyed = true;
        let victim = field(&mut g, "JC125", 1);
        let hidden = field(&mut g, "JC125", 2);
        g.board_mut(&hidden).unwrap().face_down = true;
        let safe = g.make_card("JC125", 1);
        let safe_id = safe.id.clone();
        g.regions[1].cards.push(safe);
        fund(&mut g, 0, "JC047", 4);
        let id = held(&mut g, "JZ33", 0);
        play_card(&mut g, 0, &id);
        for _ in 0..12 {
            if g.stack.last().is_some_and(|s| s.frame.as_ref().unwrap().ability_key == "batch20-jz33-scorch") { break; }
            assert!(g.pending.is_none(), "mandatory entry has no optional acceptance");
            pass(&mut g);
        }
        assert_eq!(g.stack.last().unwrap().frame.as_ref().unwrap().ability_key, "batch20-jz33-scorch");
        restore(&mut g);
        top(&mut g);
        assert!(g.board(&victim).is_none());
        assert!(g.board(&hidden).is_some());
        assert_eq!(g.board(&safe_id).unwrap().1.damage, 0);
        let demon = g.regions[0].cards.iter().find(|c| c.definition == "JZ33").unwrap();
        assert_eq!(demon.damage, 1);
    }

    #[test]
    fn jz37_destroys_all_entities_together_but_preserves_assets_and_region_identity() {
        let mut g = game();
        let ids = g.regions.iter().map(|r| r.card.id.clone()).collect::<Vec<_>>();
        let host = field(&mut g, "JC125", 0);
        let hidden = field(&mut g, "JC125", 1);
        g.board_mut(&hidden).unwrap().face_down = true;
        let attachment = g.make_card("BQ022", 1);
        g.attachments.push(Attachment { card: attachment, host_id: host });
        let region_attachment = g.make_card("JC090", 0);
        g.attachments.push(Attachment { card: region_attachment, host_id: ids[0].clone() });
        fund(&mut g, 0, "JC047", 9);
        let id = held(&mut g, "JZ37", 0);
        act(&mut g, 0, Action { card_id: Some(id), ..Action::new("play") });
        while !g.stack.is_empty() { pass(&mut g); }
        assert!(g.regions.iter().all(|r| r.cards.is_empty() && r.destroyed && !r.vacant));
        assert!(g.attachments.is_empty());
        assert_eq!(g.players[0].assets.len(), 9);
        assert_eq!(g.regions.iter().map(|r| r.card.id.clone()).collect::<Vec<_>>(), ids);
        // The destroyed observer still saw its owner's one public character;
        // the hidden opponent did not create a second death event.
        assert!(g.pending.as_ref().is_some_and(|p| matches!(&p.resolution,
            ChoiceResolution::Declare { declaration, .. } if declaration.source.card.definition == "JC090")));
        choose(&mut g, vec!["accept".into()]);
        top(&mut g);
        assert_eq!(g.regions[0].influence, [1, 0]);
        restore(&mut g);
    }
}
fn owns(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::JZ33ScorchDestroyedRegions | Op::JZ37DestroyWorld | Op::BQ054LockTarget | Op::JC043DestroyAttachment
            | Op::Batch20Prepare { .. } | Op::WM034AddOneTime | Op::WM034DestroyOtherCharacters
            | Op::BQ080HideForAttachment { .. } | Op::JZ57MillAndOptionalDraw => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => owns(body),
        Op::IfTargetExhausted { exhausted, ready, .. } =>
            owns(std::slice::from_ref(exhausted)) || owns(std::slice::from_ref(ready)),
        _ => false,
    })
}
fn owned_ability(a: &AbilitySpec) -> bool {
    a.key.starts_with("batch20-") || owns(&a.ops) || a.modes.iter().any(|m| owns(&m.ops))
        || a.costs.iter().any(|c| matches!(c, Cost::RemoveOneTimeFromSource | Cost::SacrificeControlledSubtype(_)))
}
fn related(f: &ResolutionFrame) -> bool {
    IMPLEMENTING.contains(&f.source.card.definition.as_str()) || f.ability_key.starts_with("batch20-")
        || f.steps.iter().any(|s| owns(std::slice::from_ref(&s.op)))
        || f.already_paid.iter().any(|p| matches!(p, PaidCost::SourceRemovedTime(_)))
}
pub(crate) fn definition(id: &str) -> Option<Definition> {
    if id == "BQ080" {
        let mut d = rules::with_abilities(vec![
            rules::ability("batch20-bq080-return", "进场触发：回收自己的墓地目标角色", Timing::Fast,
                vec![], vec![rules::target(Zone::Graveyard, EntityKind::Character, Relation::OwnedByActor, Range::Anywhere)],
                vec![Op::Move(EntityRef::Target(0), Destination::ActorHand)], Some(Event::Enter)),
            rules::ability("batch20-bq080-hide", "本方圣甲虫产业附属进场：潜伏", Timing::Fast,
                vec![], vec![], vec![Op::BQ080HideForAttachment { observation: None }], Some(Event::AttachmentEntered)),
        ]);
        d.traits.public = true; d.traits.retreat = true;
        return Some(d);
    }
    if id == "JZ57" {
        let mut host = rules::target(Zone::Board, EntityKind::Character, Relation::Any, Range::Anywhere);
        host.subtype = Some("人类".into());
        let mut d = rules::attachment_definition(host);
        d.attachment.as_mut().unwrap().host_defense_bonus = 1;
        let mut a = rules::ability("batch20-jz57-mill", "行动：横置、牺牲本方不死生物；目标玩家磨2，骷髅可抓1", Timing::Standard,
            vec![Cost::ExhaustSource, Cost::SacrificeControlledSubtype(vec!["不死生物".into()])],
            vec![rules::target(Zone::Player, EntityKind::Any, Relation::Any, Range::Anywhere)],
            vec![Op::JZ57MillAndOptionalDraw], None);
        a.activation_only = true; d.abilities.push(a);
        return Some(d);
    }
    if id == "BQ094" { return rules::death_observer_definition(id); }
    if id == "JZ52" { return Some(Definition::default()); }
    if matches!(id, "JC048" | "WM034" | "BQ091" | "JC068") {
        let label = match id {
            "JC048" => "准备强制触发：时间标志、原地区伤害及重新计数",
            "WM034" => "准备强制触发：若没有时间标志则牺牲",
            "BQ091" => "准备触发：将自己的牌库顶牌置墓",
            _ => "准备强制触发：支付1，否则横置",
        };
        let prep = rules::ability(&format!("batch20-prepare-{id}"), label, Timing::Fast,
            vec![], vec![], vec![Op::Batch20Prepare { event_id: None }], Some(Event::Prepare));
        if id == "WM034" {
            let mut action = rules::ability("batch20-wm034-destroy", "行动：横置并移除自身1个时间标志，消灭所有其他角色",
                Timing::Standard, vec![Cost::ExhaustSource, Cost::RemoveOneTimeFromSource], vec![],
                vec![Op::WM034DestroyOtherCharacters], None);
            action.activation_only = true;
            return Some(rules::with_abilities(vec![rules::ability("batch20-wm034-entry", "时间标志1",
                Timing::Fast, vec![], vec![], vec![Op::WM034AddOneTime], Some(Event::Enter)), action, prep]));
        }
        let host = rules::target(if id == "JC068" { Zone::Society } else { Zone::Region },
            EntityKind::Any, if id == "JC068" { Relation::ControlledByActor } else { Relation::Any }, Range::Anywhere);
        let mut d = rules::attachment_definition(host);
        d.abilities.push(prep);
        return Some(d);
    }
    if id == "JC043" {
        let mut slot = rules::target(Zone::Board, EntityKind::Attachment, Relation::Any, Range::Anywhere);
        slot.predicate = Some(TargetPredicate::JC043SocietyOrOriginalRegion);
        return Some(rules::with_abilities(vec![rules::ability("batch20-jc043-enter",
            "进场触发：消灭结附于秘社或原地区的目标附属", Timing::Fast, vec![], vec![slot],
            vec![Op::JC043DestroyAttachment], Some(Event::Enter))]));
    }
    if id == "BQ054" {
        return Some(rules::with_abilities(vec![
            rules::ability("batch20-bq054-enter", "进场触发：横置原地区的目标暗藏者", Timing::Fast,
                vec![], vec![rules::target(Zone::Board, EntityKind::Hidden, Relation::Any, Range::SourceRegion)],
                vec![Op::Exhaust(EntityRef::Target(0))], Some(Event::Enter)),
            rules::ability("batch20-bq054-discard", "弃牌触发：锁定目标角色或暗藏者", Timing::Fast,
                vec![], vec![rules::target(Zone::Board, EntityKind::CharacterOrHidden, Relation::Any, Range::Anywhere)],
                vec![Op::BQ054LockTarget], Some(Event::HandDiscard)),
        ]));
    }
    let mut a = match id {
        "JZ33" => rules::ability("batch20-jz33-scorch", "进场强制触发：对已毁灭地区的所有角色造成1点伤害",
            Timing::Fast, vec![], vec![], vec![Op::JZ33ScorchDestroyedRegions], Some(Event::Enter)),
        "JZ37" => rules::ability("batch20-jz37-world", "行动：消灭所有角色、暗藏者和附属，毁灭所有地区",
            Timing::Standard, vec![], vec![], vec![Op::JZ37DestroyWorld], None),
        _ => return None,
    };
    a.play_only = id == "JZ37";
    Some(rules::with_abilities(vec![a]))
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    let mut normalized = a.clone();
    if let [Op::Batch20Prepare { event_id }] = normalized.ops.as_mut_slice() { *event_id = None; }
    if let [Op::BQ080HideForAttachment { observation }] = normalized.ops.as_mut_slice() { *observation = None; }
    if owned_ability(a) && definition(id).is_none_or(|d| !d.abilities.iter().any(|e| same(&normalized, e))) {
        return Err("本批卡牌只准入已核原图的完整固定程序".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if definition(id).is_some_and(|expected| !same(d, &expected))
        || (!IMPLEMENTING.contains(&id) && d.abilities.iter().any(owned_ability)) {
        return Err("本批完整定义不可改写或移植".into());
    }
    Ok(())
}
pub(crate) fn validate_action(id: &str, a: &Action) -> RuleResult<()> {
    if matches!(id, "JZ33" | "JZ37" | "JC043" | "BQ054" | "WM034") && a.kind != "activate" && (a.target_id.is_some() || a.option.is_some()
        || a.choice_id.is_some() || a.selected.is_some() || a.top.is_some()
        || a.bottom.is_some() || a.allocations.is_some() || a.cost_selected.is_some()
        || a.deck_draft.is_some() || (matches!(a.kind.as_str(), "reveal" | "play") && a.region.is_some())
        || (a.ability_id.is_some() && a.kind != "play")) {
        return Err("该程序没有额外目标、费用选择或地区选择".into());
    }
    Ok(())
}

impl Game {
    pub(crate) fn observe_bq080_attachment_entry(&mut self, id: &str) {
        let Some(a) = self.attachments.iter().find(|a| a.card.id == id && !a.card.face_down && self.attachment_host_valid(a)
            && catalog::card(&a.card.definition).subtitle.as_deref() == Some("圣甲虫产业")).cloned() else { return; };
        let actor = a.card.controller;
        let observers = self.regions.iter().enumerate().flat_map(|(r, reg)| reg.cards.iter().map(move |c| (r,c)))
            .filter(|(_,c)| c.definition == "BQ080" && !c.face_down && c.controller == actor)
            .map(|(r,c)| self.source_snapshot(c, Some(r))).collect::<Vec<_>>();
        if observers.is_empty() { return; }
        let observation = AttachmentEntryObservation { event_id: self.id(), attachment: a };
        for source in observers {
            let mut ability = definition("BQ080").unwrap().abilities[1].clone();
            ability.ops = vec![Op::BQ080HideForAttachment { observation: Some(observation.clone()) }];
            self.effects.push_back(Effect::Declare { declaration: Declaration { actor, source, ability } });
        }
    }
    fn validate_bq080_entry(&self, s: &SourceSnapshot, observation: &Option<AttachmentEntryObservation>) -> RuleResult<()> {
        let o = observation.as_ref().ok_or("回收专员潜伏缺少真实附属进场观察")?;
        let a = &o.attachment;
        if s.card.definition != "BQ080" || o.event_id.is_empty() || a.card.id.is_empty() || a.host_id.is_empty()
            || a.card.id == s.card.id || a.card.face_down || a.card.controller != s.card.controller
            || a.card.owner >= self.players.len() || catalog::card(&a.card.definition).kind != "attachment"
            || catalog::card(&a.card.definition).subtitle.as_deref() != Some("圣甲虫产业") {
            return Err("回收专员附属进场观察的身份、副名或操控者不符".into());
        }
        Ok(())
    }
    pub(crate) fn pay_subtype_sacrifice(&mut self, actor: usize, subtypes: &[String], a: &Action) -> RuleResult<PaidCost> {
        let ids = a.cost_selected.as_ref().filter(|ids| ids.len() == 1).ok_or("请选择一名符合类别的本方角色牺牲")?;
        let (_,c) = self.board(&ids[0]).filter(|(_,c)| !c.face_down && c.controller == actor
            && catalog::card(&c.definition).kind == "character").ok_or("牺牲费用须为本方明置角色")?;
        let actual = self.current_subtypes(c);
        if !subtypes.iter().any(|s| actual.contains(s)) { return Err("牺牲角色不符合费用类别".into()); }
        let receipt = PaidCost::SacrificedSubtype { card: c.clone(), subtypes: actual };
        self.remove_dead(&ids[0], RemovalCause::Sacrifice);
        Ok(receipt)
    }
    fn validate_subtype_payment(&self, f: &ResolutionFrame, expected: &[Cost]) -> RuleResult<()> {
        if expected.len() != f.already_paid.len() { return Err("类别牺牲费用数量不符".into()); }
        for (cost, receipt) in expected.iter().zip(&f.already_paid) {
            match (cost, receipt) {
                (Cost::ExhaustSource, PaidCost::Exhausted(id)) if id == &f.source.card.id && !f.source.card.exhausted => {},
                (Cost::SacrificeControlledSubtype(required), PaidCost::SacrificedSubtype { card:c, subtypes })
                    if !c.id.is_empty() && !c.face_down && c.controller == f.actor && c.owner < self.players.len()
                        && catalog::card(&c.definition).kind == "character" && required.iter().any(|s| subtypes.contains(s)) => {},
                _ => return Err("类别牺牲费用收据不符".into()),
            }
        }
        Ok(())
    }
    pub(crate) fn jz57_resolve(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        let seat = f.targets[0].id.strip_prefix('p').and_then(|s| s.parse::<usize>().ok()).filter(|s| *s < self.players.len()).ok_or("磨牌目标玩家无效")?;
        self.mill_cards(seat, 2);
        if f.already_paid.iter().any(|p| matches!(p, PaidCost::SacrificedSubtype { subtypes, .. } if subtypes.iter().any(|s| s == "骷髅"))) {
            self.choice(f.actor, "trigger", "骷髅纹身：可以抓一张牌".into(), vec![ChoiceOption { id: "draw".into(), label: "抓一张牌".into(), card: None }],
                0, 1, None, ChoiceResolution::Frame { frame: Box::new(f), choice: FrameChoice::Batch20OptionalDraw });
        }
        Ok(())
    }
    pub(crate) fn batch20_optional_draw(&mut self, f: &ResolutionFrame, chooser: usize, selected: &[String]) -> RuleResult<()> {
        self.validate_batch20_frame(f)?;
        if f.ability_key != "batch20-jz57-mill" || chooser != f.actor || f.cursor != 1
            || !f.already_paid.iter().any(|p| matches!(p, PaidCost::SacrificedSubtype { subtypes, .. } if subtypes.iter().any(|s| s == "骷髅")))
            || selected.len() > 1 || selected.first().is_some_and(|s| s != "draw") { return Err("骷髅抓牌续体无效".into()); }
        if !selected.is_empty() { self.draw(f.actor, 1)?; }
        Ok(())
    }
    pub(crate) fn original_region(&self, s: &SourceSnapshot) -> Option<usize> {
        s.region.filter(|r| self.regions.get(*r).is_some_and(|region|
            !region.vacant && s.source_region_instance.as_ref() == Some(&region.card.id)))
    }
    fn region_time(&self, r: usize) -> u32 {
        self.regions[r].cards.iter().map(|c| c.time_markers)
            .chain(self.region_attachment_cards(r).map(|c| c.time_markers))
            .fold(0u32, u32::saturating_add)
    }
    pub(crate) fn batch20_prepare_resolve(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        match f.source.card.definition.as_str() {
            "JC048" => {
                if let Some(c) = self.board_mut(&f.source.card.id).filter(|c| !c.face_down) {
                    c.time_markers = c.time_markers.saturating_add(1);
                }
                if let Some(r) = self.original_region(&f.source) {
                    let n = self.region_time(r);
                    self.damage(self.regions[r].cards.iter().filter(|c| !c.face_down
                        && catalog::card(&c.definition).kind == "character").map(|c| (c.id.clone(), n)).collect())?;
                    if self.region_time(r) >= 3 { self.remove_dead(&f.source.card.id, RemovalCause::Sacrifice); }
                }
            }
            "WM034" => {
                if self.board(&f.source.card.id).is_some_and(|(_,c)| !c.face_down && self.effective_time_markers(c) == 0) {
                    self.remove_dead(&f.source.card.id, RemovalCause::Sacrifice);
                }
            }
            "BQ091" => self.mill_cards(f.actor, 1),
            "JC068" => {
                if self.resources(f.actor) == 0 { self.jc068_exhaust(&f.source.card.id); }
                else {
                    self.choice(f.actor, "upkeep", "阿格斯IV：支付1，否则横置".into(), self.jc068_upkeep_options(f.actor),
                        1, 1, None, ChoiceResolution::Frame { frame: Box::new(f.clone()), choice: FrameChoice::JC068Upkeep });
                }
            }
            _ => return Err("非准入准备效果".into()),
        }
        Ok(())
    }
    pub(crate) fn mill_cards(&mut self, seat: usize, count: usize) {
        for _ in 0..count.min(self.players[seat].deck.len()) {
            let c = self.players[seat].deck.remove(0);
            let c = self.reset_zone_card(c);
            self.players[c.owner].graveyard.push(c);
        }
    }
    fn jc068_exhaust(&mut self, id: &str) {
        if let Some(c) = self.board_mut(id) { c.exhausted = true; }
    }
    fn jc068_upkeep_options(&self, seat: usize) -> Vec<ChoiceOption> {
        let mut options = vec![ChoiceOption { id: "exhaust".into(), label: "横置阿格斯IV".into(), card: None }];
        if self.resources(seat) > 0 { options.insert(0, ChoiceOption { id: "pay".into(), label: "支付1".into(), card: None }); }
        options
    }
    pub(crate) fn validate_jc068_upkeep(&self, p: &Pending, f: &ResolutionFrame) -> RuleResult<()> {
        if f.source.card.definition != "JC068" || f.cursor != 1 || !matches!(f.guard, GuardState::Accepted)
            || p.seat != f.actor || p.choice.player_id != player_id(f.actor) || p.choice.kind != "upkeep"
            || p.choice.id.is_empty() || p.choice.title != "阿格斯IV：支付1，否则横置"
            || p.choice.min != Some(1) || p.choice.max != Some(1) || p.choice.allow_decline != Some(false)
            || p.choice.amount.is_some() || !p.choice.preview_cards.is_empty()
            || !same(&p.choice.options, &self.jc068_upkeep_options(f.actor)) {
            return Err("阿格斯IV维护费用待选无效".into());
        }
        Ok(())
    }
    pub(crate) fn jc068_upkeep_complete(&mut self, f: &ResolutionFrame, chooser: usize, selected: &[String]) -> RuleResult<()> {
        self.validate_jc035_frame(f)?;
        if chooser != f.actor || f.source.card.definition != "JC068" || f.cursor != 1 { return Err("维护费用来源或选择者不符".into()); }
        match selected.first().map(String::as_str) {
            Some("pay") if selected.len() == 1 => self.pay(f.actor, 1)?,
            Some("exhaust") if selected.len() == 1 => self.jc068_exhaust(&f.source.card.id),
            _ => return Err("须支付或横置".into()),
        }
        Ok(())
    }
    pub(crate) fn jc068_blocks_conceal(&self, seat: usize) -> bool {
        self.attachments.iter().any(|a| a.card.definition == "JC068" && !a.card.face_down && !a.card.exhausted
            && self.attachment_host_valid(a) && self.team(a.card.controller) != self.team(seat))
    }
    pub(crate) fn wm034_time_payments(&self, source: &SourceSnapshot) -> Vec<TimeMarkerPayment> {
        self.in_play_cards().into_iter().filter(|(_, c)| !c.face_down && c.time_markers > 0
            && (c.id == source.card.id || self.attachments.iter().any(|a| a.card.id == c.id && a.host_id == source.card.id)))
            .map(|(region,c)| TimeMarkerPayment { carrier: c.clone(), region,
                attachment_host: self.attachments.iter().find(|a| a.card.id == c.id).map(|a| a.host_id.clone()),
                remaining: c.time_markers - 1 }).collect()
    }
    pub(crate) fn wm034_pay_time(&mut self, source: &SourceSnapshot, action: &Action) -> RuleResult<PaidCost> {
        let ids = action.cost_selected.as_ref().filter(|ids| ids.len() == 1).ok_or("请选择来源自身或其附属的时间标志")?;
        let p = self.wm034_time_payments(source).into_iter().find(|p| p.carrier.id == ids[0]).ok_or("来源没有该时间标志")?;
        self.board_mut(&p.carrier.id).ok_or("载体离场")?.time_markers = p.remaining;
        Ok(PaidCost::SourceRemovedTime(p))
    }
    fn validate_wm034_payment(&self, f: &ResolutionFrame) -> RuleResult<()> {
        let [PaidCost::Exhausted(id), PaidCost::SourceRemovedTime(p)] = f.already_paid.as_slice() else { return Err("乌尔德须横置并移除一枚自身有效时间标志".into()); };
        let mut source = f.source.card.clone(); source.exhausted = true;
        if id != &source.id || f.source.card.exhausted || p.carrier.time_markers.checked_sub(1) != Some(p.remaining)
            || p.region != f.source.region || p.carrier.face_down || p.carrier.owner >= self.players.len()
            || p.carrier.controller >= self.players.len() || p.carrier.id.is_empty()
            || (p.carrier.id == source.id && (p.attachment_host.is_some() || !same(&p.carrier, &source)))
            || (p.carrier.id != source.id && (p.attachment_host.as_ref() != Some(&source.id)
                || catalog::card(&p.carrier.definition).kind != "attachment")) {
            return Err("乌尔德横置及真实时间载体费用收据不符".into());
        }
        Ok(())
    }
    pub(crate) fn wm034_destroy_others(&mut self, f: &ResolutionFrame) {
        let deaths = self.regions.iter().enumerate().flat_map(|(r, reg)| reg.cards.iter().map(move |c| (r,c)))
            .filter(|(_,c)| !c.face_down && c.id != f.source.card.id && catalog::card(&c.definition).kind == "character")
            .map(|(r,c)| (c.id.clone(), RemovalCause::Destroy, self.source_snapshot(c, Some(r)))).collect();
        self.remove_death_batch(deaths);
    }
    pub(crate) fn jz33_scorch(&mut self) -> RuleResult<()> {
        let allocations = self.regions.iter().filter(|r| !r.vacant && r.destroyed)
            .flat_map(|r| &r.cards)
            .filter(|c| !c.face_down && catalog::card(&c.definition).kind == "character")
            .map(|c| (c.id.clone(), 1)).collect::<BTreeMap<_, _>>();
        self.damage(allocations)
    }
    pub(crate) fn jz37_destroy_world(&mut self) {
        // Freeze one simultaneous wave before losing any observers or control grants.
        let characters = self.regions.iter().enumerate().flat_map(|(r, region)|
            region.cards.iter().filter(|c| c.face_down || catalog::card(&c.definition).kind == "character")
                .map(move |c| (r, c)))
            .map(|(r,c)| (c.id.clone(), RemovalCause::Destroy, self.source_snapshot(c, Some(r))));
        // Remove explicitly destroyed attachments first, so a host-leaves-to-hand
        // clause cannot rescue an attachment from this same destroy wave.
        let deaths = self.attachments.iter().map(|a| (a.card.id.clone(), RemovalCause::Destroy,
                self.source_snapshot(&a.card, self.attachment_region(a))))
            .chain(characters)
            .collect();
        self.remove_death_batch(deaths);
        self.destroy_all_regions();
    }
    fn validate_batch20_source(&self, actor: usize, source: &SourceSnapshot) -> RuleResult<()> {
        let c = &source.card;
        if !IMPLEMENTING.contains(&c.definition.as_str()) || c.id.is_empty()
            || actor >= self.players.len() || actor != c.controller || c.owner >= self.players.len()
            || source.region.is_some_and(|r| r >= self.regions.len())
            || source.observed_death.is_some()
            || (catalog::card(&c.definition).kind != "attachment" && source.attachment_host_instance.is_some()) {
            return Err("本批来源身份、操控者或上下文不符".into());
        }
        Ok(())
    }
    fn batch20_expected(&self, f: &ResolutionFrame) -> RuleResult<AbilitySpec> {
        if let Some(mut a) = definition(&f.source.card.definition).and_then(|d|
            d.abilities.into_iter().find(|a| a.key == f.ability_key)) {
            if a.key == "batch20-bq080-hide" {
                let [Step { op: Op::BQ080HideForAttachment { observation }, .. }] = f.steps.as_slice() else { return Err("附属进场程序不符".into()); };
                self.validate_bq080_entry(&f.source, observation)?;
                a.ops = vec![Op::BQ080HideForAttachment { observation: observation.clone() }];
            }
            return Ok(a);
        }
        let region = f.source.source_region_instance.as_deref().ok_or("奖励缺少原地区实例")?;
        match f.ability_key.as_str() {
            "renown" => Ok(crate::renown::renown_ability(region)),
            "jc089-combat-glory" => Ok(rules::jc089_combat_glory_ability(region)),
            _ => Err("本批来源上的未知程序".into()),
        }
    }
    pub(crate) fn validate_batch20_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if let Some(PlaySource::GraveyardByJZ52(grant)) = &f.source.play_source {
            if f.ability_key != "deploy" || f.actor >= self.players.len() || f.actor != f.source.card.controller
                || grant.owner >= self.players.len() || f.source.card.face_down || !f.source.valid_character_play()
                || self.in_play(&grant.id).is_some_and(|(_,c)| c.definition != grant.definition || c.owner != grant.owner) {
                return Err("墓地出牌缺少有效的事件时死亡领域许可".into());
            }
        }
        if self.validated_mobility_frame(f)? || !related(f) { return Ok(()); }
        if f.source.card.definition == "BQ094" && f.source.observed_death.is_some() { return self.validate_observer_frame(f, None); }
        if crate::jc035::preparation_frame(f) { return self.validate_jc035_frame(f); }
        self.validate_batch20_source(f.actor, &f.source)?;
        if f.frame_id.is_empty() || f.cursor > f.steps.len() {
            return Err("本批原子程序帧身份、游标或目标不符".into());
        }
        let paid_printed = matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)]
            if ids.len() <= catalog::card(&f.source.card.definition).cost as usize
                && ids.iter().all(|id| !id.is_empty())
                && ids.iter().collect::<BTreeSet<_>>().len() == ids.len());
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            if catalog::card(&f.source.card.definition).kind != "character" || !f.steps.is_empty() || !f.targets.is_empty()
                || f.source.region.is_none() || f.chosen_region != f.source.region || !paid_printed
                || (f.ability_key == "deploy" && (f.source.card.face_down || !f.source.valid_character_play()))
                || (f.ability_key == "reveal" && (!f.source.card.face_down || f.source.play_source.is_some())) {
                return Err("本批角色进场帧不符".into());
            }
        } else {
            let a = self.batch20_expected(f)?;
            let steps = a.ops.iter().map(|op| Step { context: f.actor, op: op.clone() }).collect::<Vec<_>>();
            if !same(&steps, &f.steps) || f.source.card.face_down || f.source.play_source.is_some()
                || (f.chosen_region.is_some() && !a.targets.iter().any(|s| s.zone == Zone::Region))
                || (a.event.is_some() && !f.already_paid.is_empty())
                || (a.event.is_none() && !a.costs.iter().any(|c| matches!(c, Cost::SacrificeControlledSubtype(_) | Cost::RemoveOneTimeFromSource)) && !paid_printed) {
                return Err("本批程序、费用收据或声明地区不符".into());
            }
            if f.targets.len() != a.targets.len() || f.targets.iter().zip(&a.targets).any(|(t, s)|
                !same(&t.spec, s) || t.id.is_empty() || t.public.instance_id != t.id
                || (t.spec.zone != Zone::Region && t.region_instance.is_some()) || !t.public.valid || t.public.status != "valid"
                || t.public.invalid_reason.is_some()) {
                return Err("本批目标绑定与固定程序不符".into());
            }
            if f.ability_key == "batch20-wm034-destroy" { self.validate_wm034_payment(f)?; }
            if a.costs.iter().any(|c| matches!(c, Cost::SacrificeControlledSubtype(_))) { self.validate_subtype_payment(f, &a.costs)?; }
        }
        Ok(())
    }
    pub(crate) fn validate_batch20_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if self.validated_mobility_declaration(d)? { return Ok(()); }
        if d.source.card.definition == "BQ094" && d.source.observed_death.is_some() { return self.validate_observer_declaration(d); }
        if d.ability.event == Some(Event::Prepare) { return crate::jc035::validate_ability(&d.source.card.definition, &d.ability); }
        if !IMPLEMENTING.contains(&d.source.card.definition.as_str()) && !owned_ability(&d.ability) { return Ok(()); }
        self.validate_batch20_source(d.actor, &d.source)?;
        let f = self.make_frame(d.actor, d.source.clone(), &d.ability, vec![], vec![], None);
        let expected = self.batch20_expected(&f)?;
        if expected.event.is_none() || !same(&d.ability, &expected) || d.source.card.face_down
            || d.source.play_source.is_some() { return Err("本批声明不是完整触发程序".into()); }
        Ok(())
    }
    pub(crate) fn batch20_reject_optional(&self, d: &Declaration) -> RuleResult<()> {
        if matches!(d.ability.key.as_str(), "batch20-jz33-scorch" | "death-observer-BQ094") { return Err("该能力是强制触发".into()); }
        Ok(())
    }
    pub(crate) fn batch20_declare_forced(&mut self, d: &Declaration) -> RuleResult<bool> {
        if !matches!(d.ability.key.as_str(), "batch20-jz33-scorch" | "death-observer-BQ094") { return Ok(false); }
        self.validate_batch20_declaration(d)?;
        if !self.players[d.actor].eliminated {
            let f = self.make_frame(d.actor, d.source.clone(), &d.ability, vec![], vec![], None);
            self.dispatch_frame(f, d.ability.label.clone(), None, ResponsePolicy::Respondable);
        }
        Ok(true)
    }
    pub(crate) fn validate_batch20_state(&self) -> RuleResult<()> {
        self.validate_destruction_state()?;
        for item in &self.stack {
            if let Some(f) = &item.frame {
                self.validate_batch20_frame(f)?;
                if related(f) && !crate::mobility::related_frame(f) {
                    if item.id != f.frame_id || item.controller != f.actor || item.target != f.targets.first().map(|t| t.id.clone())
                        || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked) {
                        return Err("本批堆叠外壳不符".into());
                    }
                    let transaction = matches!(f.ability_key.as_str(), "deploy" | "reveal" | "attach" | "batch20-jz37-world");
                    if transaction != item.card.is_some() || item.card.as_ref().is_some_and(|c|
                        c.definition != f.source.card.definition || c.owner != f.source.card.owner || c.controller != f.actor || c.face_down)
                        || item.deploy_region != (if matches!(f.ability_key.as_str(), "deploy" | "reveal") { f.chosen_region } else { None })
                        || item.reveal != (f.ability_key == "reveal") {
                        return Err("本批堆叠实体、地区或现身身份不符".into());
                    }
                }
            } else if item.card.as_ref().is_some_and(|c| IMPLEMENTING.contains(&c.definition.as_str())) {
                return Err("本批打出实体缺少帧".into());
            }
        }
        for e in &self.effects {
            match e {
                Effect::Declare { declaration } => self.validate_batch20_declaration(declaration)?,
                Effect::Frame { frame } if related(frame) => return Err("本批原子程序不能绕过响应堆叠保存".into()),
                _ => {},
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Declare { declaration, .. } => {
                    self.validate_batch20_declaration(declaration)?;
                    self.batch20_reject_optional(declaration)?;
                }
                ChoiceResolution::Frame { frame, choice: FrameChoice::JC068Upkeep } if crate::jc035::preparation_frame(frame) => {},
                ChoiceResolution::Frame { frame: f, choice: FrameChoice::Batch20OptionalDraw } => {
                    self.validate_batch20_frame(f)?;
                    if f.ability_key != "batch20-jz57-mill" || f.cursor != 1 || !matches!(f.guard, GuardState::Accepted)
                        || p.seat != f.actor || p.choice.player_id != player_id(f.actor) || p.choice.kind != "trigger"
                        || p.choice.min != Some(0) || p.choice.max != Some(1) || p.choice.allow_decline != Some(true)
                        || !same(&p.choice.options, &vec![ChoiceOption { id: "draw".into(), label: "抓一张牌".into(), card: None }])
                        || !f.already_paid.iter().any(|r| matches!(r, PaidCost::SacrificedSubtype { subtypes, .. } if subtypes.iter().any(|s| s == "骷髅"))) {
                        return Err("骷髅抓牌待选无效".into());
                    }
                }
                ChoiceResolution::Frame { frame, .. } if related(frame) => return Err("本批原子程序没有帧内待选".into()),
                _ => {},
            }
        }
        Ok(())
    }
}
