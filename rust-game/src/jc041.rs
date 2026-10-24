//! Complete JC041: its paid hidden action stays in play until resolution;
//! its compulsory Reveal then uses the existing resumable private discards.
use crate::{catalog, engine::RuleResult, model::*, rules::{self, *}};
use std::collections::BTreeSet;

pub(crate) const HIDDEN: &str = "jc041-water-hidden-reveal";
pub(crate) const REVEAL: &str = "jc041-reveal-each-discard";
pub(crate) const DISCARD_TITLE: &str = "禁婆：弃 1 张手牌";
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
fn uses(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::JC041RevealOriginalHidden => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => uses(body),
        Op::IfTargetExhausted { exhausted, ready, .. } =>
            uses(std::slice::from_ref(exhausted)) || uses(std::slice::from_ref(ready)),
        _ => false,
    })
}
fn own(a: &AbilitySpec) -> bool {
    matches!(a.key.as_str(), HIDDEN | REVEAL) || uses(&a.ops) || a.modes.iter().any(|m| uses(&m.ops))
}
fn related(f: &ResolutionFrame) -> bool {
    f.source.card.definition == "JC041" || matches!(f.ability_key.as_str(), HIDDEN | REVEAL)
        || f.steps.iter().any(|s| uses(std::slice::from_ref(&s.op)))
}
pub(crate) fn definition() -> Definition {
    let mut action = rules::ability(HIDDEN, "快速暗藏者行动2：水域现身", Timing::Fast,
        vec![Cost::Assets(2)], vec![], vec![Op::JC041RevealOriginalHidden], None);
    action.activation_only = true;
    rules::with_abilities(vec![action, rules::ability(REVEAL, "现身强制触发：每位玩家各弃一张手牌",
        Timing::Fast, vec![], vec![], vec![Op::ForEachLivingPlayer(vec![Op::Discard {
            player: PlayerRef::Context, count: Some(1), redraw: false, optional: false,
        }])], Some(Event::Reveal))])
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if own(a) && (id != "JC041" || !definition().abilities.iter().any(|v| same(v, a))) {
        return Err("JC041只准入完整水域暗藏者行动和强制全玩家弃牌".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if (id == "JC041" && !same(d, &definition()))
        || (id != "JC041" && d.abilities.iter().any(own)) {
        return Err("JC041完整定义不能改写或移植".into());
    }
    Ok(())
}
pub(crate) fn validate_character_action(id: &str, a: &Action) -> RuleResult<()> {
    if id == "JC041" && *a != (Action {
        card_id: a.card_id.clone(), region: if a.kind == "reveal" { None } else { a.region },
        ..Action::new(&a.kind)
    }) { return Err("JC041派遣或普通现身不附带能力、目标或额外选择".into()); }
    Ok(())
}
pub(crate) fn hidden_action(c: &Card, a: &Action) -> bool {
    c.definition == "JC041" && c.face_down && a.kind == "activate" && a.ability_id.as_deref() == Some(HIDDEN)
}
impl Game {
    pub(crate) fn jc041_check_activation(&self, actor: usize, region: Option<usize>, c: &Card, a: &Action) -> RuleResult<()> {
        if c.definition != "JC041" { return Ok(()); }
        if !hidden_action(c, a) || c.controller != actor || a.card_id.as_deref() != Some(c.id.as_str())
            || *a != (Action { card_id: Some(c.id.clone()), ability_id: Some(HIDDEN.into()), ..Action::new("activate") })
            || !region.is_some_and(|r| self.region_live(r) && catalog::card(&self.regions[r].card.definition).subtypes.iter().any(|s| s == "水域")) {
            return Err("JC041快速暗藏者行动只能由操控者在水域地区发动".into());
        }
        Ok(())
    }
    fn jc041_source(&self, s: &SourceSnapshot, actor: usize) -> RuleResult<()> {
        if s.card.definition != "JC041" || s.card.id.is_empty() || actor >= self.players.len()
            || s.card.owner >= self.players.len() || s.card.controller != actor
            || s.region.is_none_or(|r| r >= self.regions.len())
            || s.source_region_instance.as_ref().is_none_or(|v| v.is_empty() || *v == s.card.id)
            || s.observed_death.is_some() || s.observed_mobility_grant.is_some() || s.attachment_host_instance.is_some() {
            return Err("JC041冻结来源、行动者或原地区身份无效".into());
        }
        let region_uid = s.source_region_instance.as_deref().unwrap();
        if self.regions.iter().enumerate().any(|(index,r)| r.card.id == region_uid && Some(index) != s.region) {
            return Err("JC041原地区UID仍可辨认但位于错误地区槽".into());
        }
        let cards = self.regions.iter().flat_map(|r| std::iter::once(&r.card).chain(&r.cards))
            .chain(self.players.iter().flat_map(|p| p.hand.iter().chain(&p.deck).chain(&p.assets).chain(&p.graveyard).chain(&p.score_cards).chain(p.society_zone.card.iter())))
            .chain(&self.world).chain(self.attachments.iter().map(|a| &a.card))
            .chain(self.sealed_cards.iter().map(|s| &s.card)).chain(self.stack.iter().filter_map(|s| s.card.as_ref()));
        for c in cards {
            if c.id == region_uid && catalog::card(&c.definition).kind != "region" {
                return Err("JC041原地区UID与可辨认的非地区实体冲突".into());
            }
            if c.id == s.card.id && (c.definition != s.card.definition || c.owner != s.card.owner || c.face_down != s.card.face_down) {
                return Err("JC041冻结UID与既有实体的定义、拥有者或明暗不符".into());
            }
        }
        Ok(())
    }
    fn jc041_trigger(&self, key: &str, s: &SourceSnapshot) -> RuleResult<AbilitySpec> {
        match key {
            REVEAL => Ok(definition().abilities[1].clone()),
            "renown" | "jc089-combat-glory" => {
                let r = s.source_region_instance.as_deref().filter(|s| !s.is_empty()).ok_or("JC041获授奖励缺少原地区身份")?;
                Ok(if key == "renown" { crate::renown::renown_ability(r) } else { rules::jc089_combat_glory_ability(r) })
            }
            _ => Err("JC041未知触发程序".into()),
        }
    }
    fn jc041_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if self.validated_cross_card_declaration(d)? { return Ok(()); }
        if d.source.card.definition != "JC041" && !own(&d.ability) { return Ok(()); }
        self.jc041_source(&d.source, d.actor)?;
        if d.source.card.face_down || d.source.play_source.is_some()
            || !same(&d.ability, &self.jc041_trigger(&d.ability.key, &d.source)?) {
            return Err("JC041现身强制声明或获授奖励不符".into());
        }
        Ok(())
    }
    pub(crate) fn jc041_declare(&mut self, d: Declaration) -> RuleResult<bool> {
        self.jc041_declaration(&d)?;
        if d.ability.key != REVEAL { return Ok(false); }
        if self.players[d.actor].eliminated { return Ok(true); }
        let label = format!("{}：{}", catalog::card("JC041").name, d.ability.label);
        let f = self.make_frame(d.actor, d.source, &d.ability, vec![], vec![], None);
        self.dispatch_frame(f, label, None, ResponsePolicy::Respondable);
        Ok(true)
    }
    pub(crate) fn jc041_reject_optional(&self, d: &Declaration) -> RuleResult<()> {
        self.jc041_declaration(d)?;
        if d.ability.key == REVEAL { return Err("JC041现身强制触发不能可选或放弃".into()); }
        Ok(())
    }
    pub(crate) fn validate_jc041_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if self.validated_cross_card_frame(f)? || !related(f) { return Ok(()); }
        self.jc041_source(&f.source, f.actor)?;
        if f.frame_id.is_empty() || !f.targets.is_empty() { return Err("JC041帧身份或多余目标无效".into()); }
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            if f.cursor != 0 || !f.steps.is_empty() || !matches!(f.guard, GuardState::Unchecked)
                || f.chosen_region != f.source.region
                || (f.ability_key == "deploy" && (f.source.card.face_down || !f.source.valid_character_play()))
                || (f.ability_key == "reveal" && (!f.source.card.face_down || f.source.play_source.is_some()))
                || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if f.money_units(0) <= 4 && ids.iter().all(|i| !i.is_empty()) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len()) {
                return Err("JC041普通付费派遣或现身空帧不符".into());
            }
            return Ok(());
        }
        if f.ability_key == HIDDEN {
            if !f.source.card.face_down || f.source.play_source.is_some() || f.chosen_region.is_some()
                || f.steps.len() != 1 || f.steps[0].context != f.actor || !matches!(f.steps[0].op, Op::JC041RevealOriginalHidden)
                || f.cursor > 1 || (f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked))
                || (f.cursor == 1 && !matches!(f.guard, GuardState::Accepted))
                || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if f.money_units(0) == 2 && ids.iter().all(|i| !i.is_empty()) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len()) {
                return Err("JC041水域暗藏者付费帧或游标不符".into());
            }
            let r = &self.regions[f.source.region.unwrap()];
            if !r.vacant && Some(&r.card.id) == f.source.source_region_instance.as_ref()
                && !catalog::card(&r.card.definition).subtypes.iter().any(|s| s == "水域") {
                return Err("JC041冻结发动地区仍在场但不是水域".into());
            }
            return Ok(());
        }
        let expected = self.jc041_trigger(&f.ability_key, &f.source)?;
        if f.source.card.face_down || f.source.play_source.is_some() || !f.already_paid.is_empty() || f.chosen_region.is_some() {
            return Err("JC041强制弃牌及获授奖励没有费用或地区选择".into());
        }
        if f.ability_key == REVEAL {
            let seats = f.steps.iter().map(|s| s.context).collect::<Vec<_>>();
            if seats.is_empty() || !seats.contains(&f.actor) || seats.iter().any(|s| *s >= self.players.len())
                || seats.windows(2).any(|v| v[0] >= v[1])
                || self.players.iter().any(|p| !p.eliminated && !seats.contains(&p.seat))
                || f.steps.iter().any(|s| !matches!(s.op, Op::Discard { player: PlayerRef::Context, count: Some(1), redraw: false, optional: false }))
                || f.cursor > f.steps.len() || (f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked))
                || (f.cursor > 0 && !matches!(f.guard, GuardState::Accepted)) {
                return Err("JC041必须保留全玩家一次弃牌、冻结顺序及真实游标".into());
            }
        } else if f.cursor != 0 || !same(&f.steps, &expected.ops.iter().map(|op| Step { context: f.actor, op: op.clone() }).collect()) {
            return Err("JC041既有获授奖励不能改写".into());
        }
        Ok(())
    }
    pub(crate) fn jc041_reveal_hidden(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        self.validate_jc041_frame(f)?;
        if f.ability_key != HIDDEN || f.cursor != 1 || !matches!(f.guard, GuardState::Accepted) {
            return Err("JC041暗藏者现身必须经过已支付的响应堆栈".into());
        }
        let id = &f.source.card.id;
        if !self.board(id).is_some_and(|(r,c)| self.region_live(r) && c.face_down && c.definition == "JC041") { return Ok(()); }
        // Water is an activation condition, not a resolution target predicate.
        // Reset the original UID even if it moved; a replacement is never used.
        let controller = self.controller_after_reset(id).ok_or("JC041原暗藏者已失效")?;
        let (r, c) = self.leave_board(id).ok_or("JC041原暗藏者已失效")?;
        let mut c = self.fresh(c); c.face_down = false; c.controller = controller;
        c.damage = 0; c.wounds = 0; c.shield = 0;
        let next = c.id.clone(); self.regions[r].cards.push(c);
        self.note("禁婆通过水域暗藏者行动现身".into());
        self.enter_triggers(controller, "JC041", &next, true);
        Ok(())
    }
    fn jc041_continuation(&self, f: &ResolutionFrame, choice: &FrameChoice, seat: usize) -> RuleResult<()> {
        self.validate_jc041_frame(f)?;
        if f.ability_key != REVEAL || f.cursor == 0 || f.cursor > f.steps.len() || !matches!(f.guard, GuardState::Accepted)
            || !matches!(choice, FrameChoice::Discard { seat: s, redraw: false } if *s == seat && *s == f.steps[f.cursor-1].context) {
            return Err("JC041弃牌续体的选择者、游标或程序不符".into());
        }
        Ok(())
    }
    pub(crate) fn validate_jc041_choice(&self, f: &ResolutionFrame, choice: &FrameChoice, seat: usize, a: &Action, selected: &[String]) -> RuleResult<()> {
        if !related(f) { return Ok(()); }
        self.jc041_continuation(f, choice, seat)?;
        if *a != (Action { choice_id: a.choice_id.clone(), selected: Some(selected.to_vec()), ..Action::new("choose") })
            || selected.len() != 1 || !self.players[seat].hand.iter().any(|c| c.id == selected[0]) {
            return Err("JC041必须选择自己的一张当前手牌弃掉".into());
        }
        Ok(())
    }
    pub(crate) fn validate_jc041_state(&self) -> RuleResult<()> {
        let mut ids = BTreeSet::new();
        for item in &self.stack {
            if let Some(f) = &item.frame {
                self.validate_jc041_frame(f)?;
                if related(f) && !self.validated_cross_card_frame(f)? {
                    if !ids.insert(f.frame_id.clone()) || item.id != f.frame_id || item.controller != f.actor
                        || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked) || item.target.is_some() {
                        return Err("JC041堆栈身份或外壳无效".into());
                    }
                    if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
                        if item.deploy_region != f.source.region || item.reveal != (f.ability_key == "reveal")
                            || item.card.as_ref().is_none_or(|c| { let mut expected=f.source.card.clone(); expected.id=c.id.clone(); expected.face_down=false; expected.time_markers=0; expected.lock_markers=0;
                                c.id.is_empty() || c.id==f.source.card.id || !same(c,&expected) }) {
                            return Err("JC041普通派遣/现身容器与冻结来源不符".into());
                        }
                    } else if item.card.is_some() || item.deploy_region.is_some() || item.reveal {
                        return Err("JC041能力不能夹带进场卡或重定向地区".into());
                    }
                } else if item.card.as_ref().is_some_and(|c| c.definition == "JC041") {
                    return Err("JC041进场卡不能借用其他来源程序".into());
                }
            } else if item.card.as_ref().is_some_and(|c| c.definition == "JC041")
                || definition().abilities.iter().any(|a| item.label == format!("{}：{}", catalog::card("JC041").name, a.label)) {
                return Err("JC041进场堆栈缺少冻结帧".into());
            }
        }
        for e in &self.effects {
            match e {
                Effect::Declare { declaration: d } => self.jc041_declaration(d)?,
                Effect::Frame { frame: f } => {
                    self.validate_jc041_frame(f)?;
                    if related(f) && (!ids.insert(f.frame_id.clone())
                        || (matches!(f.ability_key.as_str(), "deploy" | "reveal" | HIDDEN | REVEAL)
                            && !self.rumor_frame_continuation_valid(f))) {
                        return Err("JC041进场/水域行动/强制弃牌不能复制或绕过响应堆栈".into());
                    }
                }
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Frame { frame: f, choice } if related(f) => {
                    self.jc041_continuation(f, choice, p.seat)?;
                    let options = self.players[p.seat].hand.iter().map(|c| self.option(c, p.seat, None, None)).collect::<Vec<_>>();
                    let expected = Choice { id:p.choice.id.clone(), kind:"discard".into(), title:DISCARD_TITLE.into(), description:"选择符合数量限制的选项，然后确认。".into(), player_id:player_id(p.seat), options,
                        preview_cards:vec![], min:Some(1), max:Some(1), amount:None, allow_decline:Some(false) };
                    if !ids.insert(f.frame_id.clone()) || p.choice.id.is_empty() || expected.options.is_empty() || !same(&expected, &p.choice) {
                        return Err("JC041私有强制弃牌选项或元数据不符".into());
                    }
                }
                ChoiceResolution::Declare { declaration: d, stage } if d.source.card.definition == "JC041" || own(&d.ability) => {
                    if self.validated_cross_card_declaration(d)? { return Ok(()); }
                    self.jc041_reject_optional(d)?;
                    let expected = Choice { id:p.choice.id.clone(), kind:"trigger".into(), title:format!("{}：{}", catalog::card("JC041").name, d.ability.label), description:"选择符合数量限制的选项，然后确认。".into(), player_id:player_id(d.actor),
                        options:vec![ChoiceOption { id:"accept".into(),label:"发动触发能力".into(),card:None }], preview_cards:vec![], min:Some(0),max:Some(1),amount:None,allow_decline:Some(true) };
                    if !matches!(stage, DeclareChoice::Accept) || p.seat != d.actor || p.choice.id.is_empty() || !same(&expected,&p.choice) {
                        return Err("JC041获授奖励待选不符".into());
                    }
                }
                _ if p.choice.title == DISCARD_TITLE => return Err("JC041强制弃牌不能脱离其冻结帧续体".into()),
                _ => {}
            }
        }
        Ok(())
    }
}
