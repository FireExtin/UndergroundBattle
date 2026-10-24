//! Complete XQ23: a paid, responsive asset return followed by the original
//! clerk becoming an exhausted asset. Shared zone cleanup owns both resets.
use crate::{catalog, engine::RuleResult, model::*, rules::{self, *}};
use std::collections::BTreeSet;

pub(crate) const KEY: &str = "xq23-return-asset-become-asset";
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
fn uses(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::XQ23ReturnAssetThenBecomeAsset => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => uses(body),
        Op::IfTargetExhausted { exhausted, ready, .. } => uses(std::slice::from_ref(exhausted)) || uses(std::slice::from_ref(ready)),
        _ => false,
    })
}
fn own(a: &AbilitySpec) -> bool {
    a.key == KEY || uses(&a.ops) || a.modes.iter().any(|m| uses(&m.ops))
        || a.targets.iter().chain(a.modes.iter().flat_map(|m| &m.targets))
            .any(|t| t.predicate == Some(TargetPredicate::XQ23ActorAsset))
}
fn related(f: &ResolutionFrame) -> bool {
    f.source.card.definition == "XQ23" || f.ability_key == KEY
        || f.steps.iter().any(|s| uses(std::slice::from_ref(&s.op)))
        || f.targets.iter().any(|t| t.spec.predicate == Some(TargetPredicate::XQ23ActorAsset))
}
pub(crate) fn definition() -> Definition {
    let mut target = rules::target(Zone::AttachmentOrAsset, EntityKind::Any, Relation::ControlledByActor, Range::Anywhere);
    target.predicate = Some(TargetPredicate::XQ23ActorAsset);
    let mut action = rules::ability(KEY, "快速行动2，横置：退回本方资产，然后自身横置成为资产", Timing::Fast,
        vec![Cost::Assets(2), Cost::ExhaustSource], vec![target], vec![Op::XQ23ReturnAssetThenBecomeAsset], None);
    action.activation_only = true;
    let mut d = rules::with_abilities(vec![action]);
    d.traits.public = true;
    d.traits.retreat = true;
    d
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if own(a) && (id != "XQ23" || !same(a, &definition().abilities[0])) {
        return Err("XQ23只准入完整付2横置、退回资产与自身成为资产程序".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if id == "XQ23" && !same(d, &definition()) || id != "XQ23" && d.abilities.iter().any(own) {
        return Err("XQ23必须同时保留公开、撤回及完整快速行动，不能移植".into());
    }
    Ok(())
}
pub(crate) fn validate_character_action(id: &str, a: &Action) -> RuleResult<()> {
    if id == "XQ23" && *a != (Action { card_id: a.card_id.clone(), region: a.region, ..Action::new(&a.kind) }) {
        return Err("XQ23派遣或现身不接受额外目标、选项或费用声明".into());
    }
    Ok(())
}
impl Game {
    fn xq23_source(&self, s: &SourceSnapshot, actor: usize) -> RuleResult<()> {
        if s.card.definition != "XQ23" || s.card.id.is_empty() || actor >= self.players.len()
            || s.card.owner >= self.players.len() || s.card.controller != actor
            || s.region.is_none_or(|r| r >= self.regions.len()) || s.observed_death.is_some()
            || s.attachment_host_instance.is_some() || s.observed_mobility_grant.is_some()
            || s.source_region_instance.as_ref().is_none_or(|id| id.is_empty() || *id == s.card.id) {
            return Err("XQ23冻结来源、行动者或原地区身份无效".into());
        }
        let region_uid = s.source_region_instance.as_deref().unwrap();
        if self.regions.iter().enumerate().any(|(r, reg)| reg.card.id == region_uid && Some(r) != s.region) {
            return Err("XQ23原地区UID与既有地区槽冲突".into());
        }
        for c in self.regions.iter().flat_map(|r| std::iter::once(&r.card).chain(&r.cards))
            .chain(self.players.iter().flat_map(|p| p.hand.iter().chain(&p.deck).chain(&p.assets)
                .chain(&p.graveyard).chain(&p.score_cards).chain(p.society_zone.card.iter())))
            .chain(&self.world).chain(self.attachments.iter().map(|a| &a.card))
            .chain(self.sealed_cards.iter().map(|s| &s.card)).chain(self.stack.iter().filter_map(|s| s.card.as_ref())) {
            if c.id == region_uid && catalog::card(&c.definition).kind != "region" {
                return Err("XQ23原地区UID不能引用非地区实体".into());
            }
            if c.id == s.card.id && (c.definition != s.card.definition || c.owner != s.card.owner || c.face_down != s.card.face_down) {
                return Err("XQ23冻结来源与可辨认的既有实体身份冲突".into());
            }
        }
        Ok(())
    }
    fn xq23_granted(&self, key: &str, s: &SourceSnapshot) -> RuleResult<AbilitySpec> {
        let region = s.source_region_instance.as_deref().filter(|s| !s.is_empty()).ok_or("XQ23获授奖励缺少原地区身份")?;
        match key {
            "renown" => Ok(crate::renown::renown_ability(region)),
            "jc089-combat-glory" => Ok(rules::jc089_combat_glory_ability(region)),
            _ => Err("XQ23没有该触发程序".into()),
        }
    }
    pub(crate) fn validate_xq23_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if self.validated_cross_card_declaration(d)? || d.source.card.definition != "XQ23" && !own(&d.ability) { return Ok(()); }
        self.xq23_source(&d.source, d.actor)?;
        if d.source.card.face_down || d.source.play_source.is_some()
            || !same(&d.ability, &self.xq23_granted(&d.ability.key, &d.source)?) {
            return Err("XQ23主动行动不能伪装成免费或可选触发".into());
        }
        Ok(())
    }
    pub(crate) fn validate_xq23_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if self.validated_cross_card_frame(f)? || !related(f) { return Ok(()); }
        self.xq23_source(&f.source, f.actor)?;
        if f.frame_id.is_empty() { return Err("XQ23缺少冻结帧身份".into()); }
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            if f.cursor != 0 || !f.steps.is_empty() || !f.targets.is_empty() || !matches!(f.guard, GuardState::Unchecked)
                || f.chosen_region != f.source.region
                || f.ability_key == "deploy" && (f.source.card.face_down || !f.source.valid_character_play())
                || f.ability_key == "reveal" && (!f.source.card.face_down || f.source.play_source.is_some())
                || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if f.money_units(0) <= 1 && ids.iter().all(|id| !id.is_empty())) {
                return Err("XQ23普通付费派遣/现身容器不符".into());
            }
            return Ok(());
        }
        if f.source.card.face_down || f.source.play_source.is_some() || f.chosen_region.is_some() {
            return Err("XQ23快速行动或获授奖励来源不符".into());
        }
        if f.ability_key == KEY {
            if f.source.card.exhausted || f.targets.len() != 1 || f.targets[0].id.is_empty()
                || f.targets[0].id == f.source.card.id || !same(&f.targets[0].spec, &definition().abilities[0].targets[0])
                || f.targets[0].public.instance_id != f.targets[0].id || f.targets[0].public.kind != "asset"
                || f.steps.len() != 1 || f.steps[0].context != f.actor || !matches!(f.steps[0].op, Op::XQ23ReturnAssetThenBecomeAsset)
                || f.cursor > 1 || f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked)
                || f.cursor == 1 && !matches!(f.guard, GuardState::Accepted)
                || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids), PaidCost::Exhausted(id)]
                    if f.money_units(0) == 2 && id == &f.source.card.id && ids.iter().all(|i| !i.is_empty() && i != id)
                    && ids.iter().collect::<BTreeSet<_>>().len() == ids.len()) {
                return Err("XQ23必须保留完整目标、两份资产支付、原来源横置和真实游标".into());
            }
            return Ok(());
        }
        let a = self.xq23_granted(&f.ability_key, &f.source)?;
        if f.cursor != 0 || !f.already_paid.is_empty() || !f.targets.is_empty()
            || !same(&f.steps, &a.ops.iter().map(|op| Step { context: f.actor, op: op.clone() }).collect::<Vec<_>>()) {
            return Err("XQ23既有获授奖励不能改写".into());
        }
        Ok(())
    }
    pub(crate) fn xq23_return_and_become_asset(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        let target = &f.targets[0].id;
        let i = self.players[f.actor].assets.iter().position(|c| c.id == *target).ok_or("XQ23本方原资产已失效")?;
        let asset = self.players[f.actor].assets.remove(i);
        let owner = asset.owner;
        let asset = self.reset_zone_card(asset);
        self.players[owner].hand.push(asset);
        self.note("谨慎的办事员：原资产移回其拥有者手中".into());
        // The bound target must survive, but the paid ability is independent of
        // its source. Only the still-live original source can perform part two.
        if self.board(&f.source.card.id).is_some_and(|(_, c)| c.definition == "XQ23" && !c.face_down) {
            let (_, clerk) = self.leave_board(&f.source.card.id).unwrap();
            let mut clerk = self.reset_zone_card(clerk);
            clerk.controller = f.actor;
            clerk.exhausted = true;
            self.players[f.actor].assets.push(clerk);
            self.note("谨慎的办事员横置成为行动者的资产".into());
        }
        Ok(())
    }
    pub(crate) fn validate_xq23_state(&self) -> RuleResult<()> {
        // Asset targets expose owner labels before a paid frame exists. Reject
        // invalid seat indexes before projection as well as before resolution.
        if self.players.iter().flat_map(|p| &p.assets).any(|c| c.owner >= self.players.len()) {
            return Err("资产区拥有者座位无效".into());
        }
        let mut ids = BTreeSet::new();
        for item in &self.stack {
            if let Some(f) = &item.frame {
                self.validate_xq23_frame(f)?;
                if related(f) && !self.validated_cross_card_frame(f)? {
                    if !ids.insert(f.frame_id.clone()) || item.id != f.frame_id || item.controller != f.actor
                        || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked)
                        || item.target != f.targets.first().map(|t| t.id.clone()) {
                        return Err("XQ23堆栈身份或响应外壳不符".into());
                    }
                    if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
                        if item.deploy_region != f.source.region || item.reveal != (f.ability_key == "reveal")
                            || item.card.as_ref().is_none_or(|c| {
                                let mut expected = f.source.card.clone(); expected.id = c.id.clone(); expected.face_down = false; expected.time_markers = 0; expected.lock_markers = 0;
                                c.id.is_empty() || c.id == f.source.card.id || !same(c, &expected)
                            }) { return Err("XQ23进场卡与原冻结来源不符".into()); }
                    } else if item.card.is_some() || item.deploy_region.is_some() || item.reveal {
                        return Err("XQ23行动不能夹带进场卡或地区重定向".into());
                    }
                } else if item.card.as_ref().is_some_and(|c| c.definition == "XQ23") {
                    return Err("XQ23进场卡不能借用其他来源".into());
                }
            } else if item.card.as_ref().is_some_and(|c| c.definition == "XQ23")
                || item.label == format!("{}：{}", catalog::card("XQ23").name, definition().abilities[0].label) {
                return Err("XQ23堆栈缺少冻结帧".into());
            }
        }
        for e in &self.effects {
            match e {
                Effect::Declare { declaration } => self.validate_xq23_declaration(declaration)?,
                Effect::Frame { frame: f } => {
                    self.validate_xq23_frame(f)?;
                    if related(f) && (!ids.insert(f.frame_id.clone()) || matches!(f.ability_key.as_str(), "deploy" | "reveal" | KEY)) {
                        return Err("XQ23进场/快速行动不能复制或绕过响应堆栈".into());
                    }
                }
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Frame { frame, .. } if related(frame) => return Err("XQ23原行动不产生待选续体".into()),
                ChoiceResolution::Declare { declaration: d, stage } if d.source.card.definition == "XQ23" || own(&d.ability) => {
                    if self.validated_cross_card_declaration(d)? { return Ok(()); }
                    self.validate_xq23_declaration(d)?;
                    let expected = Choice { id: p.choice.id.clone(), kind: "trigger".into(), title: format!("{}：{}", catalog::card("XQ23").name, d.ability.label), description: "选择符合数量限制的选项，然后确认。".into(), player_id: player_id(d.actor), options: vec![ChoiceOption { id: "accept".into(), label: "发动触发能力".into(), card: None }], preview_cards: vec![], min: Some(0), max: Some(1), amount: None, allow_decline: Some(true) };
                    if !matches!(stage, DeclareChoice::Accept) || p.seat != d.actor || p.choice.id.is_empty() || !same(&p.choice, &expected) {
                        return Err("XQ23获授奖励待选不符".into());
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
