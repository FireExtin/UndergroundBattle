//! XG17: personal, global Renown defence and event-time entry observations.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub(crate) const KEY: &str = "batch4-xg17-renown-entry";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum RenownBasis {
    Printed,
    Exposure { count: u32 },
    DestroyedRegion { instance: String },
    Alpha { source: Box<SourceSnapshot>, exposure: u32 },
    TurnGrant { grant: TurnAttributeModifier },
    Politician { proof: Box<crate::batch4_politician::MarkerRenown> },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenownEntry { pub event_id: String, pub turn: u32, pub entered: SourceSnapshot, pub basis: RenownBasis }
fn same<T: Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
pub(crate) fn definition() -> Definition {
    let mut slot = target(Zone::Board, EntityKind::Attachment, Relation::ControlledByActor, Range::Anywhere);
    slot.predicate = Some(TargetPredicate::XG17PersonalPlan);
    let mut d = with_abilities(vec![ability(KEY, "本方声望角色进场：目标本方计划推进1", Timing::Fast, vec![], vec![slot], vec![Op::XG17AdvancePlan { observation: None }], Some(Event::XG17RenownEntered))]);
    d.traits.public = true;
    d.modifiers.push(StaticModifier::XG17RenownDefense);
    d
}
pub(crate) fn canonical(a: &AbilitySpec) -> Option<AbilitySpec> {
    let mut expected = definition().abilities.into_iter().find(|b| b.key == a.key)?;
    if let [Op::XG17AdvancePlan { observation }] = a.ops.as_slice() { expected.ops = vec![Op::XG17AdvancePlan { observation: observation.clone() }]; }
    Some(expected)
}
pub(crate) fn validate_action(a: &Action) -> RuleResult<()> {
    let expected = match a.kind.as_str() {
        "deploy" | "conceal" => Action { card_id: a.card_id.clone(), region: a.region, ..Action::new(&a.kind) },
        "reveal" => Action { card_id: a.card_id.clone(), ..Action::new("reveal") },
        _ => return Err("专家研讨会只有正常派遣与现身操作".into()),
    };
    if *a != expected { return Err("专家研讨会不能添加目标、额外支付或私有载荷".into()); } Ok(())
}
pub(crate) fn validate_choice_action(p: &Pending, a: &Action) -> RuleResult<()> {
    if matches!(&p.resolution, ChoiceResolution::Declare { declaration: d, .. } if d.source.card.definition == "XG17")
        && *a != (Action { choice_id: a.choice_id.clone(), selected: a.selected.clone(), ..Action::new("choose") }) {
        return Err("研讨会触发待选只接受原选择，不接受额外费用或载荷".into());
    } Ok(())
}
impl Game {
    pub(crate) fn xg17_conditional_renown(&self, c: &Card) -> bool { c.definition == "XG17" && self.society_exposure(c.controller) > 0 }
    pub(crate) fn xg17_defense_bonus(&self, c: &Card) -> u32 {
        let sources = self.regions.iter().filter(|r| !r.vacant).flat_map(|r| &r.cards)
            .filter(|s| s.definition == "XG17" && !s.face_down && s.id != c.id && s.controller == c.controller && !self.players[s.controller].eliminated).count() as u32;
        if sources == 0 || !self.has_renown(c) || !self.board(&c.id).is_some_and(|(_, live)| live == c) { 0 } else { sources }
    }
    fn xg17_renown_basis(&self, c: &Card) -> Option<RenownBasis> {
        if !self.has_renown(c) { return None; }
        if crate::rules::definition(&c.definition).traits.renown { return Some(RenownBasis::Printed); }
        let exposure = self.society_exposure(c.controller);
        if matches!(c.definition.as_str(), "WM041" | "XG17") && exposure > 0 { return Some(RenownBasis::Exposure { count: exposure }); }
        if self.jz32_destroyed(c) { return Some(RenownBasis::DestroyedRegion { instance: self.regions[self.board(&c.id)?.0].card.id.clone() }); }
        if let Some(proof) = self.wm029_renown_basis(c) { return Some(RenownBasis::Politician { proof: Box::new(proof) }); }
        if self.current_public(c, c.controller) {
            if let Some((r, s)) = self.regions.iter().enumerate().filter(|(_, r)| !r.vacant).flat_map(|(r, region)| region.cards.iter().map(move |s| (r, s)))
                .find(|(_, s)| s.definition == "WM041" && !s.face_down && s.controller == c.controller && !self.players[s.controller].eliminated) {
                return Some(RenownBasis::Alpha { source: Box::new(self.source_snapshot(s, Some(r))), exposure });
            }
        }
        self.turn_attribute_modifiers.iter().find(|m| m.target_instance == c.id && m.expires_turn == self.turn && m.grants_renown).cloned().map(|grant| RenownBasis::TurnGrant { grant })
    }
    pub(crate) fn xg17_observe_entry(&mut self, id: &str) {
        let Some((r, c)) = self.board(id).filter(|(_, c)| !c.face_down && catalog::card(&c.definition).kind == "character") else { return; };
        let sources = self.regions.iter().enumerate().filter(|(_, r)| !r.vacant).flat_map(|(r, region)| region.cards.iter().map(move |s| (r, s)))
            .filter(|(_, s)| s.definition == "XG17" && !s.face_down && s.controller == c.controller && !self.players[s.controller].eliminated)
            .map(|(r, s)| self.source_snapshot(s, Some(r))).collect::<Vec<_>>();
        if sources.is_empty() { return; }
        let Some(basis) = self.xg17_renown_basis(c) else { return; };
        let mut entered = self.source_snapshot(c, Some(r)); entered.source_region_instance = Some(self.regions[r].card.id.clone());
        let observation = RenownEntry { event_id: self.id(), turn: self.turn, entered, basis };
        for source in sources {
            let mut a = definition().abilities.remove(0); a.ops = vec![Op::XG17AdvancePlan { observation: Some(Box::new(observation.clone())) }];
            self.effects.push_back(Effect::Declare { declaration: Declaration { actor: source.card.controller, source, ability: a } });
        }
    }
    fn xg17_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        let c = &s.card;
        if c.definition != "XG17" || c.id.is_empty() || c.owner >= self.players.len() || actor >= self.players.len() || c.controller != actor
            || s.death_echo.is_some() || s.observed_death.is_some() || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some() || s.attachment_host_instance.is_some()
            || s.region.is_none_or(|r| r >= self.regions.len()) || s.source_region_instance.as_ref().is_none_or(|id| id.is_empty() || *id == c.id)
            || s.region.zip(s.source_region_instance.as_ref()).is_some_and(|(r, id)| self.regions.iter().enumerate().any(|(other, live)| live.card.id == *id && r != other))
            || s.play_source.is_none() && self.batch3_physical_cards().any(|live| live.id == c.id) && self.board(&c.id).is_none()
        { return Err("专家研讨会须保留原角色、玩家及地区身份".into()); } self.mobility_immutable_identity(c)
    }
    fn validate_xg17_observation(&self, actor: usize, ops: &[Op]) -> RuleResult<()> {
        let [Op::XG17AdvancePlan { observation: Some(o) }] = ops else { return Err("专家研讨会缺少实际声望角色进场收据".into()); };
        let s = &o.entered; let c = &s.card;
        if o.turn != self.turn || actor >= self.players.len() || c.controller != actor || c.owner >= self.players.len() || c.id.is_empty() || c.face_down || c.advancement_markers != 0
            || !catalog::catalog().cards.iter().any(|d| d.id == c.definition && d.kind == "character" && catalog::runtime_implemented(d))
            || s.region.is_none_or(|r| r >= self.regions.len()) || s.source_region_instance.as_ref().is_none_or(|id| id.is_empty() || *id == c.id)
            || s.region.zip(s.source_region_instance.as_ref()).is_some_and(|(r, id)| self.regions.iter().enumerate().any(|(other, live)| live.card.id == *id && r != other))
            || s.play_source.is_some() || s.attachment_host_instance.is_some() || s.death_echo.is_some() || s.observed_death.is_some() || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some()
            || o.event_id.strip_prefix('i').and_then(|v| v.parse::<u64>().ok()).is_none_or(|n| n == 0 || n > self.sequence) || o.event_id == c.id
            || self.batch3_physical_cards().any(|live| live.id == c.id) && self.board(&c.id).is_none()
        { return Err("声望进场收据的原角色、控制者、地区或事件不完整".into()); }
        self.mobility_immutable_identity(c)?;
        let valid = match &o.basis {
            RenownBasis::Printed => crate::rules::definition(&c.definition).traits.renown,
            RenownBasis::Exposure { count } => *count > 0 && matches!(c.definition.as_str(), "WM041" | "XG17"),
            RenownBasis::DestroyedRegion { instance } => c.definition == "JZ32" && Some(instance) == s.source_region_instance.as_ref(),
            RenownBasis::Alpha { source, exposure } => {
                let a = &source.card;
                let valid = a.definition == "WM041" && !a.face_down && a.controller == actor && a.owner < self.players.len() && !a.id.is_empty()
                    && source.region.is_some_and(|r| r < self.regions.len()) && source.source_region_instance.as_ref().is_some_and(|id| !id.is_empty() && *id != a.id)
                    && source.play_source.is_none() && source.attachment_host_instance.is_none() && source.death_echo.is_none() && source.observed_death.is_none() && source.observed_exposure.is_none() && source.observed_mobility_grant.is_none()
                    && (crate::rules::definition(&c.definition).traits.public || matches!(c.definition.as_str(), "WM040" | "WM060") && *exposure > 0);
                if valid { self.mobility_immutable_identity(a)?; } valid
            },
            RenownBasis::TurnGrant { grant } => grant.target_instance == c.id && grant.expires_turn == o.turn && grant.grants_renown,
            RenownBasis::Politician { proof } => { self.validate_marker_renown(c, proof)?; true },
        };
        if !valid { return Err("声望进场不能由事后条件或其他角色的领域/授予冒充".into()); } Ok(())
    }
    fn xg17_expected(&self, s: &SourceSnapshot, key: &str, ops: &[Op]) -> RuleResult<AbilitySpec> {
        if key == "renown" || key == "jc089-combat-glory" {
            let uid = s.source_region_instance.as_deref().ok_or("研讨会奖励缺少原地区")?;
            return Ok(if key == "renown" { crate::renown::renown_ability(uid) } else { crate::rules::jc089_combat_glory_ability(uid) });
        }
        let mut a = definition().abilities.remove(0); if a.key != key { return Err("研讨会未知程序".into()); } a.ops = ops.to_vec(); Ok(a)
    }
    pub(crate) fn validate_xg17_declaration(&self, d: &Declaration) -> RuleResult<()> {
        self.xg17_source(d.actor, &d.source)?;
        let a = self.xg17_expected(&d.source, &d.ability.key, &d.ability.ops)?;
        if d.source.card.face_down || d.source.play_source.is_some() || !same(&a, &d.ability) { return Err("研讨会声明必须保留完整触发或既有奖励".into()); }
        if a.key == KEY {
            self.validate_xg17_observation(d.actor, &a.ops)?;
            if canonical(&a).is_none_or(|b| !same(&a, &b)) { return Err("研讨会完整固定程序被改写".into()); }
            if let [Op::XG17AdvancePlan { observation: Some(o) }] = a.ops.as_slice() {
                if o.event_id == d.source.card.id || o.entered.card.id == d.source.card.id && !same(&o.entered, &d.source) { return Err("研讨会自身进场须保留同一份实际角色快照".into()); }
            }
        } Ok(())
    }
    pub(crate) fn validate_xg17_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        self.xg17_source(f.actor, &f.source)?;
        if f.frame_id.is_empty() || f.frame_id == f.source.card.id || f.cursor > f.steps.len() || (f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked)) || f.cursor > 0 && !matches!(f.guard, GuardState::Accepted) { return Err("研讨会帧、游标或守卫无效".into()); }
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            if !f.steps.is_empty() || !f.targets.is_empty() || f.chosen_region != f.source.region
                || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if ids.len() <= 4 && ids.iter().all(|id| !id.is_empty() && *id != f.source.card.id) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len())
                || f.ability_key == "deploy" && (f.source.card.face_down || !f.source.valid_character_play())
                || f.ability_key == "reveal" && (!f.source.card.face_down || f.source.play_source.is_some()) { return Err("研讨会派遣/现身必须保留地区及实际支付".into()); } return Ok(());
        }
        let ops = f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>(); let a = self.xg17_expected(&f.source, &f.ability_key, &ops)?;
        self.validate_xg17_declaration(&Declaration { actor: f.actor, source: f.source.clone(), ability: a.clone() })?;
        if f.steps.iter().any(|s| s.context != f.actor) || !same(&ops, &a.ops) || !f.already_paid.is_empty() || f.chosen_region.is_some() || f.targets.len() != a.targets.len() || f.targets.iter().zip(&a.targets).any(|(t, s)| !same(&t.spec, s)) { return Err("研讨会触发的完整操作、目标或支付被改写".into()); }
        if f.ability_key == KEY {
            let t = &f.targets[0];
            if t.id.is_empty() || t.region_instance.is_some() || t.public.instance_id != t.id || t.public.kind != "attachment" || t.public.controller.as_deref() != Some(player_id(f.actor).as_str()) || t.public.owner.is_none() || !t.public.valid || t.public.status != "valid" || t.public.invalid_reason.is_some() || t.public.region.is_some() { return Err("研讨会必须保留原本方秘社计划目标".into()); }
        } Ok(())
    }
    pub(crate) fn xg17_advance_target(&mut self, f: &ResolutionFrame) {
        if let Some(t) = f.targets.first().filter(|t| self.valid_bound_target(f.actor, &f.source, t)) { let id = t.id.clone(); self.advance_plan_instance(f.actor, &id); }
    }
    pub(crate) fn validate_xg17_state(&self) -> RuleResult<()> {
        let mut frames = BTreeSet::new(); let mut entries = BTreeSet::new();
        let mut remember = |s: &SourceSnapshot, ops: &[Op]| -> RuleResult<()> { if let [Op::XG17AdvancePlan { observation: Some(o) }] = ops { if !entries.insert((s.card.id.clone(), o.event_id.clone())) { return Err("同次声望角色进场不能重复发动同一研讨会".into()); } } Ok(()) };
        for item in &self.stack { if let Some(f) = &item.frame { if f.source.card.definition == "XG17" && !self.validated_cross_card_frame(f)? {
            self.validate_xg17_frame(f)?; remember(&f.source, &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>())?; let play = matches!(f.ability_key.as_str(), "deploy" | "reveal");
            if !frames.insert(f.frame_id.clone()) || item.id != f.frame_id || item.controller != f.actor || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked) || item.card.is_some() != play || item.reveal != (f.ability_key == "reveal") || item.deploy_region != (if play { f.source.region } else { None }) || item.target != f.targets.first().map(|t| t.id.clone())
                || item.card.as_ref().is_some_and(|c| c.definition != "XG17" || c.face_down || c.owner != f.source.card.owner || c.controller != f.actor) { return Err("研讨会堆叠响应外壳或来源实例被改写".into()); }
        } } }
        for e in &self.effects { match e {
            Effect::Declare { declaration: d } if d.source.card.definition == "XG17" && !self.validated_cross_card_declaration(d)? => { self.validate_xg17_declaration(d)?; remember(&d.source, &d.ability.ops)?; },
            Effect::Frame { frame: f } if f.source.card.definition == "XG17" && !self.validated_cross_card_frame(f)? => return Err("研讨会不能绕过响应堆叠保存直接帧".into()),
            _ => {},
        } }
        if let Some(p) = &self.pending { match &p.resolution {
            ChoiceResolution::Declare { declaration: d, stage } if d.source.card.definition == "XG17" && !self.validated_cross_card_declaration(d)? => {
                self.validate_xg17_declaration(d)?; remember(&d.source, &d.ability.ops)?;
                let (options, expected_stage) = if let Some(slot) = d.ability.targets.first() { (self.binding_options(d.actor, &d.source, slot), DeclareChoice::Target) } else { (vec![ChoiceOption { id: "accept".into(), label: "发动触发能力".into(), card: None }], DeclareChoice::Accept) };
                let expected = Choice { id: p.choice.id.clone(), kind: "trigger".into(), title: format!("{}：{}", catalog::card("XG17").name, d.ability.label), description: "选择符合数量限制的选项，然后确认。".into(), player_id: player_id(d.actor), options, preview_cards: vec![], min: Some(0), max: Some(1), amount: None, allow_decline: Some(true) };
                if !same(stage, &expected_stage) || p.seat != d.actor || p.choice.id.is_empty() || expected.options.is_empty() || !same(&p.choice, &expected) { return Err("研讨会待选只能选择原本方计划或放弃".into()); }
            },
            ChoiceResolution::Frame { frame: f, .. } if f.source.card.definition == "XG17" && !self.validated_cross_card_frame(f)? => return Err("研讨会推进为原子操作，不允许帧内续体".into()),
            _ => {},
        } } Ok(())
    }
}
