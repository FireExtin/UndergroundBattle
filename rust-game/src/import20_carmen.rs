//! CQ24's attachment and optional new host are both bound before responses.
//! Permanent attachment control composes with the existing control order.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub(crate) const ID: &str = "CQ24";
pub(crate) const ENTRY: &str = "import20-cq24-take-attachment";
const HOST: &str = "cq24-new-attachment-host";
const TITLE: &str = "卡门：可选择新的结附目标；放弃则保留原宿主";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExtraPayment { pub play_instance: String, pub play_frame: String, pub assets: Vec<String>, #[serde(default,skip_serializing_if="Vec::is_empty")] pub credits: Vec<crate::step_resources::StepCredit> }
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CarmenEntry { pub event_id: String, pub turn: u32, pub secrecy: u32, pub enemies: Vec<(usize, u32)>, pub extra: Option<ExtraPayment> }
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CarmenDraft { pub declaration: Declaration, pub target: BoundTarget, pub attachment: CarmenAttachment }
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CarmenAttachment { pub card: Card, pub host_id: Option<String> }
fn same<T: Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
pub(crate) fn definition() -> Definition {
    let mut t = target(Zone::Board, EntityKind::Attachment, Relation::EnemyTeam, Range::Anywhere); t.printed_cost_max = Some(3);
    with_abilities(vec![ability(ENTRY, "进场：已额外支付3或本人隐秘高于敌方时，获得费用不大于3的原敌方附属", Timing::Fast,
        vec![], vec![t], vec![Op::CQ24TakeAttachment { observation: None, attachment: None, reattach: false }], Some(Event::CQ24QualifiedEntry))])
}
pub(crate) fn secret_plan_cost_target(source: &SourceSnapshot, slot: &TargetSlotSpec, c: &Card) -> bool {
    // Behind the Veil's printed Plan rules explicitly keep a secret Plan an
    // Attachment with printed cost 1. This uses only that public facade, not
    // the hidden front, and belongs solely to CQ24's original target slot.
    source.card.definition == ID && crate::batch4_plans::secret(c)
        && same(slot, &definition().abilities[0].targets[0])
}
fn owns(ops: &[Op]) -> bool { ops.iter().any(|op| match op {
    Op::CQ24TakeAttachment { .. } => true,
    Op::ForEachLivingPlayer(b) | Op::ForEachLivingPlayerFromActor(b) => owns(b),
    Op::IfTargetExhausted { exhausted, ready, .. } => owns(std::slice::from_ref(exhausted)) || owns(std::slice::from_ref(ready)), _ => false,
}) }
pub(crate) fn related(id: &str, a: &AbilitySpec) -> bool { id == ID || a.key == ENTRY || owns(&a.ops) || a.modes.iter().any(|m| owns(&m.ops)) || a.event == Some(Event::CQ24QualifiedEntry)
    || a.targets.iter().any(|t| matches!(t.attachment_host_condition, Some(AttachmentHostCondition::CQ24Host { .. }))) }
pub(crate) fn frame_related(f: &ResolutionFrame) -> bool { f.source.card.definition == ID || f.ability_key == ENTRY || f.steps.iter().any(|s| owns(std::slice::from_ref(&s.op)))
    || f.targets.iter().any(|t| matches!(t.spec.attachment_host_condition, Some(AttachmentHostCondition::CQ24Host { .. }))) }
fn host_slot(source: SourceSnapshot) -> Option<TargetSlotSpec> {
    let original = crate::rules::definitions().get(&source.card.definition)?.attachment.as_ref()?.host.clone();
    let mut slot = target(original.zone, original.kind, Relation::Any, Range::Anywhere);
    slot.attachment_host_condition = Some(AttachmentHostCondition::CQ24Host { attachment: Box::new(source), host: Box::new(original) });
    Some(slot)
}
fn canonical(a: &AbilitySpec) -> Option<AbilitySpec> {
    let [Op::CQ24TakeAttachment { observation, attachment, reattach }] = a.ops.as_slice() else { return None; };
    let mut result = definition().abilities.remove(0);
    result.ops = a.ops.clone();
    if *reattach {
        let attachment = attachment.as_ref()?;
        let mut source = SourceSnapshot { card: attachment.card.clone(), region: None, source_region_instance: None, attachment_host_instance: None,
            play_source: None, death_echo: None, observed_death: None, observed_exposure: None, observed_mobility_grant: None };
        let Some(AttachmentHostCondition::CQ24Host { attachment: frozen, .. }) = a.targets.get(1)?.attachment_host_condition.as_ref() else { return None; };
        // The projection is for target binding, not a physical early transfer.
        source.card.controller = frozen.card.controller;
        source.region = frozen.region; source.source_region_instance = frozen.source_region_instance.clone();
        source.attachment_host_instance = frozen.attachment_host_instance.clone();
        if !same(&source, frozen) { return None; }
        result.targets.push(host_slot(source)?);
    }
    if attachment.is_none() && *reattach || observation.is_none() && attachment.is_some() { return None; }
    Some(result)
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if id != ID || canonical(a).is_none_or(|b| !same(a, &b)) { return Err("卡门只能保留完整原费用上限、进场收据及结附目标程序".into()); } Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if id == ID && !same(d, &definition()) || id != ID && d.abilities.iter().any(|a| related(id, a)) { return Err("卡门完整定义不能改写或移植".into()); } Ok(())
}
pub(crate) fn validate_action(a: &Action) -> RuleResult<()> {
    let expected = match a.kind.as_str() {
        "deploy" if a.option.as_deref().is_none_or(|s| s == "pay-extra-3") => Action { card_id: a.card_id.clone(), region: a.region, option: a.option.clone(), ..Action::new("deploy") },
        "conceal" => Action { card_id: a.card_id.clone(), region: a.region, ..Action::new("conceal") },
        "reveal" => Action { card_id: a.card_id.clone(), ..Action::new("reveal") },
        _ => return Err("卡门只有普通派遣、秘密派遣及现身；额外3仅在正面打出时支付".into()),
    };
    if *a != expected { return Err("卡门出牌不可夹带触发目标或额外程序".into()); } Ok(())
}
impl Game {
    fn cq24_current_attachment(&self, id: &str) -> Option<CarmenAttachment> {
        let (_, c) = self.in_play(id).filter(|(_,c)| self.live_attachment_role(c) || crate::batch4_plans::secret(c) && self.attachments.iter().any(|a| a.card.id == c.id && self.attachment_host_valid(a)))?;
        Some(CarmenAttachment { card: c.clone(), host_id: self.attachments.iter().find(|a| a.card.id == id).map(|a| a.host_id.clone()) })
    }
    pub(crate) fn cq24_retreat(&self, c: &Card) -> bool { c.definition == ID && !c.face_down && self.players[c.controller].society_zone.secrecy > 0 }
    pub(crate) fn cq24_pay_extra(&mut self, actor: usize) -> RuleResult<PaidCost> {
        Ok(self.capture_money(actor,3)?.extra_money(true))
    }
    pub(crate) fn cq24_observe_entry(&mut self, id: &str, extra: Option<ExtraPayment>) {
        let Some((r, c)) = self.board(id).filter(|(_, c)| c.definition == ID && !c.face_down) else { return; };
        let actor = c.controller; let secrecy = self.players[actor].society_zone.secrecy;
        let enemies = self.players.iter().filter(|p| !p.eliminated && self.team(p.seat) != self.team(actor)).map(|p| (p.seat, p.society_zone.secrecy)).collect::<Vec<_>>();
        // Hegemony printed21 defines enemy players as the whole opposing team:
        // personal level must be higher than each, not a sum or a friendly level.
        if extra.is_none() && (enemies.is_empty() || !enemies.iter().all(|(_, n)| secrecy > *n)) { return; }
        let source = self.source_snapshot(c, Some(r));
        let observation = CarmenEntry { event_id: self.id(), turn: self.turn, secrecy, enemies, extra };
        let mut ability = definition().abilities.remove(0);
        ability.ops = vec![Op::CQ24TakeAttachment { observation: Some(Box::new(observation)), attachment: None, reattach: false }];
        self.effects.push_back(Effect::Declare { declaration: Declaration { actor, source, ability } });
    }
    fn cq24_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        let c = &s.card;
        if actor >= self.players.len() || c.definition != ID || c.controller != actor || c.owner >= self.players.len()
            || !self.mystic_instance_valid(&c.id) || s.region.is_none_or(|r| r >= self.regions.len())
            || s.source_region_instance.as_ref().is_none_or(|id| !self.mystic_instance_valid(id) || *id == c.id)
            || s.attachment_host_instance.is_some() || s.death_echo.is_some() || s.observed_death.is_some() || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some() {
            return Err("卡门须保留原进场角色、操控者与地区".into());
        } self.mobility_immutable_identity(c)
    }
    fn cq24_observation<'a>(&self, actor: usize, ops: &'a [Op]) -> RuleResult<&'a CarmenEntry> {
        let [Op::CQ24TakeAttachment { observation: Some(o), .. }] = ops else { return Err("卡门缺少实际合格进场收据".into()); };
        let mut seats = BTreeSet::new();
        if o.turn != self.turn || !self.mystic_instance_valid(&o.event_id) || o.enemies.is_empty()
            || o.enemies.iter().any(|(s, _)| *s >= self.players.len() || self.team(*s) == self.team(actor) || !seats.insert(*s))
            || self.players.iter().any(|p| !p.eliminated && self.team(p.seat) != self.team(actor) && !seats.contains(&p.seat)) {
            return Err("卡门的原回合、敌方席位或事件无效".into());
        }
        if let Some(p) = &o.extra {
            if !self.mystic_instance_valid(&p.play_instance) || !self.mystic_instance_valid(&p.play_frame) || p.play_instance == p.play_frame
                || p.assets.len() + p.credits.len() != 3 || p.assets.iter().any(|id| !self.mystic_instance_valid(id) || *id == p.play_instance)
                || p.assets.iter().collect::<BTreeSet<_>>().len() != 3 { return Err("卡门额外3须为实际不同资产与原出牌".into()); }
            for c in &p.credits { self.validate_step_credit(c)?; if c.actor != actor { return Err("卡门额外费用不能借用其他玩家步骤费用".into()); } }
        } else if !o.enemies.iter().all(|(_, n)| o.secrecy > *n) { return Err("未额外支付的卡门须在进场时高于每个敌方玩家的隐秘等级".into()); }
        Ok(o)
    }
    pub(crate) fn validate_cq24_declaration(&self, d: &Declaration) -> RuleResult<()> {
        self.cq24_source(d.actor, &d.source)?; validate_ability(ID, &d.ability)?;
        if d.source.card.face_down || d.source.play_source.is_some()
            || !matches!(d.ability.ops.as_slice(), [Op::CQ24TakeAttachment { attachment: None, reattach: false, .. }]) {
            return Err("卡门声明不能提前夹带重结附或场外来源".into());
        } self.cq24_observation(d.actor, &d.ability.ops)?; Ok(())
    }
    fn cq24_host_source(&self, actor: usize, a: &CarmenAttachment) -> SourceSnapshot {
        let mut s = self.source_snapshot(&a.card, self.in_play(&a.card.id).and_then(|(r,_)| r)); s.card.controller = actor; s
    }
    pub(crate) fn cq24_host_binding(&self, actor: usize, source: &SourceSnapshot, slot: &TargetSlotSpec, id: &str) -> bool {
        let Some(AttachmentHostCondition::CQ24Host { attachment, host }) = &slot.attachment_host_condition else { return false; };
        source.card.definition == ID && source.card.controller == actor && attachment.card.controller == actor && id != attachment.card.id
            && catalog::catalog().cards.iter().any(|d| d.id == attachment.card.definition && catalog::runtime_implemented(d)
                && catalog::has_kind(d, "attachment") && d.cost <= 3)
            && host_slot((**attachment).clone()).is_some_and(|expected| same(slot, &expected))
            && self.valid_binding(actor, attachment, host, id)
    }
    pub(crate) fn cq24_begin_host(&mut self, d: Declaration, selected: Vec<String>) -> RuleResult<()> {
        self.validate_cq24_declaration(&d)?;
        if selected.is_empty() { return Ok(()); }
        if selected.len() != 1 { return Err("卡门只选择一个原敌方附属".into()); }
        let target = self.bind_action(d.actor, &d.source, &d.ability, &Action { target_id: Some(selected[0].clone()), ..Action::new("play") })?.remove(0);
        let attachment = self.cq24_current_attachment(&target.id).ok_or("卡门原附属已失效")?;
        let draft = CarmenDraft { declaration: d, target, attachment };
        let slot = host_slot(self.cq24_host_source(draft.declaration.actor, &draft.attachment)).ok_or("卡门原目标没有完整结附规格")?;
        let options = self.binding_options(draft.declaration.actor, &draft.declaration.source, &slot);
        if options.is_empty() { return self.cq24_finish_declaration(draft, None); }
        self.choice(draft.declaration.actor, HOST, TITLE.into(), options, 0, 1, None,
            ChoiceResolution::CQ24Reattach { draft: Box::new(draft) }); Ok(())
    }
    fn cq24_finish_declaration(&mut self, draft: CarmenDraft, new_id: Option<&str>) -> RuleResult<()> {
        self.validate_cq24_draft(&draft)?;
        let mut d = draft.declaration; let original_id = draft.target.id;
        if new_id.is_some() {
            let slot = host_slot(self.cq24_host_source(d.actor, &draft.attachment)).ok_or("重结附缺少原规格")?;
            d.ability.targets.push(slot);
        }
        let [Op::CQ24TakeAttachment { attachment, reattach, .. }] = d.ability.ops.as_mut_slice() else { unreachable!() };
        *attachment = Some(Box::new(draft.attachment)); *reattach = new_id.is_some();
        validate_ability(ID, &d.ability)?;
        let action = if let Some(id) = new_id { Action { selected: Some(vec![original_id,id.into()]), ..Action::new("play") } }
            else { Action { target_id: Some(original_id), ..Action::new("play") } };
        let targets = self.bind_action(d.actor, &d.source, &d.ability, &action)?;
        let frame = self.make_frame(d.actor, d.source, &d.ability, targets, vec![], None);
        self.dispatch_frame(frame, d.ability.label, None, ResponsePolicy::Respondable); Ok(())
    }
    pub(crate) fn cq24_bind(&self, actor: usize, source: &SourceSnapshot, spec: &AbilitySpec, a: &Action) -> RuleResult<Vec<BoundTarget>> {
        validate_ability(ID,spec)?;
        let ids = a.selected.as_ref().filter(|ids| ids.len()==2).ok_or("卡门重结附须预绑定原附属和一个新宿主")?;
        let [Op::CQ24TakeAttachment { attachment:Some(att),reattach:true,.. }] = spec.ops.as_slice() else { return Err("卡门重结附缺少完整原附属程序".into()); };
        if source.card.definition!=ID || actor!=source.card.controller || ids[0]!=att.card.id || ids[0]==ids[1] || spec.targets.len()!=2 {
            return Err("卡门两个目标的来源、次序或实例无效".into());
        }
        spec.targets.iter().zip(ids).map(|(slot,id)| {
            if !self.valid_binding(actor,source,slot,id) { return Err("卡门原附属或新宿主不满足原目标条件".into()); }
            let region_instance = if slot.zone==Zone::Region {
                let r=id.strip_prefix("region:").and_then(|n|n.parse::<usize>().ok()).filter(|r|*r<self.regions.len()).ok_or("卡门地区宿主无效")?;
                Some(self.regions[r].card.id.clone())
            } else {None};
            Ok(BoundTarget { region_instance,id:id.clone(),spec:slot.clone(),public:self.public_target(actor,source,slot,id) })
        }).collect()
    }
    fn validate_cq24_draft(&self, d: &CarmenDraft) -> RuleResult<()> {
        self.validate_cq24_declaration(&d.declaration)?;
        if !same(&d.target.spec, &definition().abilities[0].targets[0]) || d.target.id != d.attachment.card.id
            || !self.valid_bound_target(d.declaration.actor, &d.declaration.source, &d.target)
            || self.cq24_current_attachment(&d.target.id).is_none_or(|a| !same(&a, &d.attachment)) { return Err("卡门重结附草稿必须保留原敌方附属和原绑定".into()); }
        Ok(())
    }
    pub(crate) fn cq24_choose_host(&mut self, draft: CarmenDraft, actor: usize, a: &Action, selected: &[String]) -> RuleResult<()> {
        if actor != draft.declaration.actor || selected.len() > 1 || *a != (Action { choice_id: a.choice_id.clone(), selected: a.selected.clone(), ..Action::new("choose") }) {
            return Err("卡门重结附只接受原玩家的一个新宿主或保留原宿主".into());
        } self.cq24_finish_declaration(draft, selected.first().map(String::as_str))
    }
    pub(crate) fn cq24_take_attachment(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        let ops = f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>();
        let o = self.cq24_observation(f.actor, &ops)?;
        let lifetime = ControlLifetime::CQ24Attachment { source_instance: f.source.card.id.clone(), event_id: o.event_id.clone(), turn: o.turn };
        let id = f.targets[0].id.clone();
        if self.cq24_current_attachment(&id).is_none() { return Ok(()); }
        self.add_control(&id, f.actor, lifetime, SubtypeChange::None);
        if let Some(host) = f.targets.get(1) {
            let host = host.region_instance.as_deref().unwrap_or(&host.id).to_string();
            self.reattach_source(&id, &host);
        } else { self.settle_deaths(); }
        Ok(())
    }
    pub(crate) fn validate_cq24_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if self.validated_cross_card_frame(f)? { return Ok(()); }
        self.cq24_source(f.actor, &f.source)?; let c = &f.source.card;
        if !self.mystic_instance_valid(&f.frame_id) || f.frame_id == c.id || f.cursor > f.steps.len()
            || f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked) || f.cursor > 0 && !matches!(f.guard, GuardState::Accepted)
            || f.steps.iter().any(|s| s.context != f.actor) { return Err("卡门帧身份、阶段或守卫无效".into()); }
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            let assets = match f.already_paid.as_slice() {
                [PaidCost::Assets(ids)] => ids.clone(),
                [PaidCost::Assets(ids), PaidCost::CQ24ExtraAssets(extra)] if f.ability_key == "deploy" && f.money_units(1) == 3 => {
                    let mut all = ids.clone(); all.extend(extra.clone()); all
                }, _ => return Err("卡门派遣/现身须保留实际印刷及额外3支付".into()),
            };
            let printed = match &f.already_paid[0] { PaidCost::Assets(ids) => ids, _ => unreachable!() };
            if f.money_units(0) > 1 || assets.iter().any(|id| !f.source.valid_printed_payment_id(id)) || assets.iter().collect::<BTreeSet<_>>().len() != assets.len()
                || !f.steps.is_empty() || !f.targets.is_empty() || f.chosen_region != f.source.region
                || f.ability_key == "deploy" && (c.face_down || !f.source.valid_character_play())
                || f.ability_key == "reveal" && (!c.face_down || f.source.play_source.is_some()) { return Err("卡门出牌须保留原来源、地区与真实支付".into()); }
            return Ok(());
        }
        let mut a = definition().abilities.remove(0); a.ops = f.steps.iter().map(|s| s.op.clone()).collect();
        a.targets = f.targets.iter().map(|t| t.spec.clone()).collect(); validate_ability(ID, &a)?;
        let o = self.cq24_observation(f.actor, &a.ops)?;
        let [Op::CQ24TakeAttachment { attachment: Some(att), reattach, .. }] = a.ops.as_slice() else { return Err("卡门响应帧缺少原附属快照".into()); };
        let Some(print) = catalog::catalog().cards.iter().find(|d| d.id == att.card.definition && catalog::runtime_implemented(d)) else { return Err("卡门附属定义未知".into()); };
        if f.ability_key != ENTRY || c.face_down || f.source.play_source.is_some() || f.chosen_region.is_some() || !f.already_paid.is_empty()
            || (att.card.face_down && !crate::batch4_plans::secret(&att.card)) || att.card.controller >= self.players.len() || self.team(att.card.controller) == self.team(f.actor)
            || att.card.owner >= self.players.len() || !catalog::has_kind(print, "attachment") || print.cost > 3
            || att.card.id == c.id || !self.mystic_instance_valid(&att.card.id)
            || att.host_id.as_ref().is_some_and(|id| id.is_empty() || *id == att.card.id)
            || att.host_id.is_none() && print.kind != "character"
            || f.targets.first().is_none_or(|t| t.id != att.card.id) || f.targets.len() != if *reattach { 2 } else { 1 }
            || o.event_id == c.id || o.event_id == att.card.id { return Err("卡门只能获得原费用不大于3的敌方附属并保留所有预绑定目标".into()); }
        self.mobility_immutable_identity(&att.card)?;
        if *reattach {
            let Some(AttachmentHostCondition::CQ24Host { attachment: projected, .. }) = &f.targets[1].spec.attachment_host_condition else { return Err("重结附缺少原附属目标规格".into()); };
            if projected.card.controller != f.actor || f.targets[1].id == att.card.id { return Err("卡门新宿主绑定的操控者或实体无效".into()); }
        } Ok(())
    }
    pub(crate) fn validate_cq24_state(&self) -> RuleResult<()> {
        for e in &self.control_effects { if let ControlLifetime::CQ24Attachment { source_instance, event_id, turn } = &e.lifetime {
            if e.recipient >= self.players.len() || *turn == 0 || *turn > self.turn || !self.mystic_instance_valid(source_instance)
                || !self.mystic_instance_valid(event_id) || source_instance == event_id || source_instance == &e.target_instance
                || !matches!(e.subtype_change, SubtypeChange::None) || !self.cq24_current_attachment(&e.target_instance).is_some_and(|a| (!a.card.face_down || crate::batch4_plans::secret(&a.card))
                    && catalog::has_kind(catalog::card(&a.card.definition), "attachment") && catalog::card(&a.card.definition).cost <= 3) {
                return Err("卡门的永久附属控制记录无效".into());
            }
        } }
        for item in &self.stack { if let Some(f) = &item.frame { if frame_related(f) && !self.validated_cross_card_frame(f)? {
            self.validate_cq24_frame(f)?; let play = matches!(f.ability_key.as_str(), "deploy" | "reveal");
            if item.id != f.frame_id || item.controller != f.actor || item.card.is_some() != play || item.reveal != (f.ability_key == "reveal")
                || item.deploy_region != (if play { f.source.region } else { None }) || item.target != f.targets.first().map(|t| t.id.clone())
                || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked) { return Err("卡门响应帧外壳无效".into()); }
        } } }
        for e in &self.effects { match e {
            Effect::Declare { declaration: d } if related(&d.source.card.definition, &d.ability) => self.validate_cq24_declaration(d)?,
            Effect::Frame { frame: f } if frame_related(f) => self.validate_cq24_frame(f)?, _ => {},
        } }
        if let Some(p) = &self.pending { match &p.resolution {
            ChoiceResolution::CQ24Reattach { draft } => {
                self.validate_cq24_draft(draft)?;
                let d = &draft.declaration; let slot = host_slot(self.cq24_host_source(d.actor, &draft.attachment)).ok_or("卡门重结附规格缺失")?;
                let expected = Choice { id: p.choice.id.clone(), kind: HOST.into(), title: TITLE.into(), description: "选择符合数量限制的选项，然后确认。".into(),
                    player_id: player_id(d.actor), options: self.binding_options(d.actor, &d.source, &slot), preview_cards: vec![], min: Some(0), max: Some(1), amount: None, allow_decline: Some(true) };
                if p.seat != d.actor || p.choice.id.is_empty() || expected.options.is_empty() || !same(&p.choice, &expected) { return Err("卡门只能在原响应前选择一个合法新宿主或保留原宿主".into()); }
            },
            ChoiceResolution::Declare { declaration: d, stage } if related(&d.source.card.definition, &d.ability) => {
                self.validate_cq24_declaration(d)?;
                let expected = Choice { id: p.choice.id.clone(), kind: "trigger".into(), title: format!("{}：{}", catalog::card(ID).name, d.ability.label),
                    description: "选择符合数量限制的选项，然后确认。".into(), player_id: player_id(d.actor), options: self.binding_options(d.actor, &d.source, &d.ability.targets[0]),
                    preview_cards: vec![], min: Some(0), max: Some(1), amount: None, allow_decline: Some(true) };
                if p.seat != d.actor || p.choice.id.is_empty() || !matches!(stage, DeclareChoice::Target) || expected.options.is_empty() || !same(&p.choice, &expected) { return Err("卡门进场只选择原敌方附属或放弃".into()); }
            },
            ChoiceResolution::Frame { frame: f, .. } if frame_related(f) && !self.validated_cross_card_frame(f)? => return Err("卡门重结附须在响应前完成绑定，不能在结算中临时改选".into()), _ => {},
        } } Ok(())
    }
}
