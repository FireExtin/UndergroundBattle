//! JC066: seal one local character/hidden entity and copy sealed printed icons.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use std::collections::BTreeSet;

pub(crate) const KEY: &str = "jc066-entry-local-seal";
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
fn uses(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::JC066SealLocalTargetOnSource => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => uses(body),
        Op::IfTargetExhausted { exhausted, ready, .. } => uses(std::slice::from_ref(exhausted)) || uses(std::slice::from_ref(ready)),
        _ => false,
    })
}
fn own(a: &AbilitySpec) -> bool {
    a.key == KEY || uses(&a.ops) || a.modes.iter().any(|m| uses(&m.ops))
}
fn related(f: &ResolutionFrame) -> bool {
    f.source.card.definition == "JC066" || f.ability_key == KEY || f.steps.iter().any(|s| uses(std::slice::from_ref(&s.op)))
}
pub(crate) fn definition() -> Definition {
    Definition {
        abilities: vec![ability(KEY, "进场触发：封印本地区的目标角色或暗藏者", Timing::Fast,
            vec![], vec![target(Zone::Board, EntityKind::CharacterOrHidden, Relation::Any, Range::SourceRegion)],
            vec![Op::JC066SealLocalTargetOnSource], Some(Event::Enter))],
        modifiers: vec![StaticModifier::JC066SealedPrintedIcons],
        ..Default::default()
    }
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if own(a) && (id != "JC066" || !same(a, &definition().abilities[0])) {
        return Err("JC066只准入完整的可选本地区封印程序".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    let relevant = id == "JC066" || d.abilities.iter().any(own)
        || d.modifiers.iter().any(|m| matches!(m, StaticModifier::JC066SealedPrintedIcons));
    if relevant && (id != "JC066" || !same(d, &definition())) {
        return Err("JC066封印与印刷图标持续能力不能拆分、改写或移植".into());
    }
    Ok(())
}
pub(crate) fn validate_character_action(id: &str, a: &Action) -> RuleResult<()> {
    if id == "JC066" && (a.target_id.is_some() || a.ability_id.is_some() || a.option.is_some()
        || a.choice_id.is_some() || a.selected.is_some() || a.top.is_some() || a.bottom.is_some()
        || a.allocations.is_some() || a.cost_selected.is_some() || a.deck_draft.is_some()
        || a.kind == "reveal" && a.region.is_some()) {
        return Err("JC066派遣或付费现身不接受额外目标、模式或费用载荷".into());
    }
    Ok(())
}

impl Game {
    pub(crate) fn jc066_sealed_printed_icons(&self, c: &Card, region: usize) -> (Icons, Icons) {
        if c.definition != "JC066" || c.face_down
            || self.board(&c.id).is_none_or(|(r, live)| r != region || live.definition != "JC066" || live.face_down)
            || !crate::rules::definition("JC066").modifiers.iter().any(|m| matches!(m, StaticModifier::JC066SealedPrintedIcons)) {
            return (Icons::default(), Icons::default());
        }
        self.sealed_cards.iter().filter(|s| s.host_id == c.id)
            .filter_map(|s| catalog::catalog().cards.iter().find(|d| d.id == s.card.definition && d.kind == "character"))
            .fold((Icons::default(), Icons::default()), |(p,t), d| (p.add(d.permanent_icons), t.add(d.temporary_icons)))
    }
    fn jc066_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        if actor >= self.players.len() || s.card.controller != actor || s.card.owner >= self.players.len()
            || s.card.definition != "JC066" || s.card.id.is_empty()
            || s.region.is_none_or(|r| r >= self.regions.len())
            || s.source_region_instance.as_ref().is_none_or(|id| id.is_empty() || id == &s.card.id)
            || s.attachment_host_instance.is_some() || s.observed_death.is_some() || s.observed_mobility_grant.is_some() {
            return Err("JC066冻结角色来源、行动者或原地区无效".into());
        }
        let cards = self.players.iter().flat_map(|p| p.hand.iter().chain(&p.deck).chain(&p.assets)
            .chain(&p.graveyard).chain(&p.score_cards).chain(p.society_zone.card.iter()));
        if self.regions.iter().flat_map(|r| std::iter::once(&r.card).chain(&r.cards))
            .chain(cards).chain(&self.world).chain(self.attachments.iter().map(|a| &a.card))
            .chain(self.sealed_cards.iter().map(|s| &s.card))
            .any(|c| c.id == s.card.id && (c.definition != s.card.definition || c.owner != s.card.owner)) {
            return Err("JC066冻结来源与现有UID的印刷身份或拥有者矛盾".into());
        }
        Ok(())
    }
    fn jc066_trigger(&self, key: &str, s: &SourceSnapshot) -> RuleResult<AbilitySpec> {
        match key {
            KEY => Ok(definition().abilities[0].clone()),
            "renown" | "jc089-combat-glory" => {
                let id = s.source_region_instance.as_deref().ok_or("JC066奖励缺少原地区")?;
                Ok(if key == "renown" { crate::renown::renown_ability(id) } else { crate::rules::jc089_combat_glory_ability(id) })
            },
            _ => Err("JC066未知触发程序".into()),
        }
    }
    fn jc066_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if self.validated_cross_card_declaration(d)? { return Ok(()); }
        if d.source.card.definition != "JC066" && !own(&d.ability) { return Ok(()); }
        self.jc066_source(d.actor, &d.source)?;
        if d.source.card.face_down || d.source.play_source.is_some()
            || !same(&d.ability, &self.jc066_trigger(&d.ability.key, &d.source)?) {
            return Err("JC066触发声明与完整印刷程序不符".into());
        }
        Ok(())
    }
    fn jc066_target(&self, f: &ResolutionFrame) -> RuleResult<()> {
        let [t] = f.targets.as_slice() else { return Err("JC066须恰好冻结一个目标".into()); };
        let seat = |id: &Option<String>| id.as_ref().and_then(|id| id.strip_prefix('p')
            .and_then(|n| n.parse::<usize>().ok()).filter(|n| *n < self.players.len() && *id == player_id(*n)));
        let owner = seat(&t.public.owner).ok_or("JC066目标拥有者无效")?;
        let controller = seat(&t.public.controller).ok_or("JC066目标操控者无效")?;
        let label = match t.public.kind.as_str() {
            "hidden" => t.public.label == format!("暗藏者·{}", self.players[controller].name),
            "character" => catalog::catalog().cards.iter().any(|d| d.kind == "character" && d.name == t.public.label),
            _ => false,
        };
        if t.id.is_empty() || t.id == f.frame_id || t.region_instance.is_some()
            || !same(&t.spec, &definition().abilities[0].targets[0]) || t.public.instance_id != t.id
            || t.public.region != f.source.region || !label || !t.public.valid
            || t.public.status != "valid" || t.public.invalid_reason.is_some()
            || self.board(&t.id).is_some_and(|(_,c)| c.owner != owner
                || c.face_down != (t.public.kind == "hidden")
                || t.public.kind == "character" && catalog::card(&c.definition).name != t.public.label) {
            return Err("JC066冻结目标的种类、原地区或公开快照无效".into());
        }
        Ok(())
    }
    pub(crate) fn validate_jc066_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if self.validated_cross_card_frame(f)? { return Ok(()); }
        if !related(f) { return Ok(()); }
        self.jc066_source(f.actor, &f.source)?;
        if f.frame_id.is_empty() { return Err("JC066缺少帧身份".into()); }
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            if !f.steps.is_empty() || !f.targets.is_empty() || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked)
                || f.chosen_region != f.source.region
                || (if f.ability_key == "deploy" { f.source.card.face_down || !f.source.valid_character_play() }
                    else { !f.source.card.face_down || f.source.play_source.is_some() })
                || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if ids.iter().all(|id| !id.is_empty()) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len()) {
                return Err("JC066派遣或付费现身空程序无效".into());
            }
            return Ok(());
        }
        let a = self.jc066_trigger(&f.ability_key, &f.source)?;
        let steps: Vec<_> = a.ops.iter().map(|op| Step { context: f.actor, op: op.clone() }).collect();
        if f.source.card.face_down || f.source.play_source.is_some() || f.chosen_region.is_some()
            || !same(&f.steps, &steps) || f.cursor > steps.len() || !f.already_paid.is_empty() {
            return Err("JC066冻结程序、步骤、游标或费用被改写".into());
        }
        if f.ability_key == KEY { self.jc066_target(f)?; }
        else if !f.targets.is_empty() { return Err("JC066获授奖励不能夹带目标".into()); }
        Ok(())
    }
    pub(crate) fn jc066_seal_local_target(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        self.validate_jc066_frame(f)?;
        if f.ability_key != KEY || f.cursor != 1 || !matches!(f.guard, GuardState::Accepted) {
            return Err("JC066封印必须经过已接受的响应堆栈".into());
        }
        if !self.seal_host_valid(&f.source.card.id) { return Ok(()); }
        let t = &f.targets[0];
        if !self.valid_bound_target(f.actor, &f.source, t) { return Ok(()); }
        let Some((_, original)) = self.board(&t.id) else { return Ok(()); };
        let sealing_actor = original.controller;
        let Some((_, c)) = self.leave_board(&t.id) else { return Ok(()); };
        if !self.seal_host_valid(&f.source.card.id) {
            // P15: the sealed card is outside play, so its departing self-host
            // releases it immediately. No invalid self-host checkpoint is saved.
            let mut source = self.source_snapshot(&c, None); source.card.controller = sealing_actor;
            let card = self.reset_zone_card(c);
            self.sealed_cards.push(SealedCard { card, host_id: f.source.card.id.clone() });
            self.emit_event(sealing_actor, source, Event::Sealed);
            self.return_sealed_cards(&f.source.card.id);
        } else {
            self.seal_card_on_valid_host(c, sealing_actor, &f.source.card.id);
        }
        Ok(())
    }
    pub(crate) fn validate_jc066_state(&self) -> RuleResult<()> {
        let mut ids = BTreeSet::new();
        for item in &self.stack {
            if let Some(f) = &item.frame {
                self.validate_jc066_frame(f)?;
                if related(f) {
                    if !ids.insert(f.frame_id.clone()) || item.id != f.frame_id || item.controller != f.actor
                        || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked) {
                        return Err("JC066堆栈容器无效".into());
                    }
                    if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
                        if item.target.is_some() || item.deploy_region != f.source.region || item.reveal != (f.ability_key == "reveal")
                            || item.card.as_ref().is_none_or(|c| { let mut expected=f.source.card.clone(); expected.id=c.id.clone();
                                expected.face_down=false; expected.time_markers=0; expected.lock_markers=0;
                                c.id.is_empty() || c.id==f.source.card.id || !same(c,&expected) }) {
                            return Err("JC066派遣容器与冻结来源不符".into());
                        }
                    } else if item.card.is_some() || item.deploy_region.is_some() || item.reveal
                        || item.target != f.targets.first().map(|t| t.id.clone()) {
                        return Err("JC066能力容器不能夹带牌或改写目标".into());
                    }
                } else if item.card.as_ref().is_some_and(|c| c.definition == "JC066") {
                    return Err("JC066不能挂接其他牌的堆栈程序".into());
                }
            } else if item.card.as_ref().is_some_and(|c| c.definition == "JC066") {
                return Err("JC066堆栈缺少冻结帧".into());
            }
        }
        for effect in &self.effects {
            match effect {
                Effect::Declare { declaration } => self.jc066_declaration(declaration)?,
                Effect::Frame { frame } => {
                    self.validate_jc066_frame(frame)?;
                    if related(frame) && (!ids.insert(frame.frame_id.clone())
                        || matches!(frame.ability_key.as_str(), "deploy" | "reveal")
                        || frame.ability_key == KEY) {
                        return Err("JC066程序不能重复或绕过响应堆栈".into());
                    }
                },
                _ => {},
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Declare { declaration:d, stage } => {
                    self.jc066_declaration(d)?;
                    if d.source.card.definition == "JC066" && !self.validated_cross_card_declaration(d)? {
                        let options = match (d.ability.targets.as_slice(), stage) {
                            ([slot], DeclareChoice::Target) => self.binding_options(d.actor, &d.source, slot),
                            ([], DeclareChoice::Accept) => vec![ChoiceOption { id:"accept".into(),label:"发动触发能力".into(),card:None }],
                            _ => return Err("JC066触发待选阶段与冻结程序不符".into()),
                        };
                        let expected=Choice { id:p.choice.id.clone(),kind:"trigger".into(),
                            title:format!("{}：{}",catalog::card("JC066").name,d.ability.label),
                            description:"选择符合数量限制的选项，然后确认。".into(),player_id:player_id(d.actor),
                            options,preview_cards:vec![],min:Some(0),max:Some(1),amount:None,allow_decline:Some(true) };
                        if self.status != "playing" || p.seat != d.actor || self.players[d.actor].eliminated
                            || p.choice.id.is_empty() || expected.options.is_empty() || !same(&p.choice,&expected) {
                            return Err("JC066触发待选必须绑定原行动者与完整合法选项".into());
                        }
                    }
                },
                ChoiceResolution::Frame { frame, .. } => {
                    self.validate_jc066_frame(frame)?;
                    if related(frame) && !self.validated_cross_card_frame(frame)? {
                        return Err("JC066封印或奖励不能挂接额外帧待选".into());
                    }
                },
                _ => {},
            }
        }
        Ok(())
    }
}
