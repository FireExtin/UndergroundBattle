//! Complete JC044: optional entry time marker and a paid standard repression.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use std::collections::BTreeSet;

pub(crate) const ENTRY: &str = "jc044-entry-one-time";
pub(crate) const ACTIVATE: &str = "jc044-repress-one";
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
fn uses(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::JC044AddOneTimeToOriginalSource => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => uses(body),
        Op::IfTargetExhausted { exhausted, ready, .. } => uses(std::slice::from_ref(exhausted)) || uses(std::slice::from_ref(ready)),
        _ => false,
    })
}
fn own(a: &AbilitySpec) -> bool {
    matches!(a.key.as_str(), ENTRY | ACTIVATE)
        || a.costs.iter().any(|c| matches!(c, Cost::RemoveOneTimeFromSelectedControlledCard))
        || uses(&a.ops) || a.modes.iter().any(|m| uses(&m.ops))
}
fn related(f: &ResolutionFrame) -> bool {
    f.source.card.definition == "JC044" || matches!(f.ability_key.as_str(), ENTRY | ACTIVATE)
        || f.steps.iter().any(|s| uses(std::slice::from_ref(&s.op)))
        || f.already_paid.iter().any(|p| matches!(p, PaidCost::RemovedTime(_)))
}
pub(crate) fn definition() -> Definition {
    with_abilities(vec![
        ability(ENTRY, "时间标志1：进场放置一个时间标志", Timing::Fast,
            vec![], vec![], vec![Op::JC044AddOneTimeToOriginalSource], Some(Event::Enter)),
        ability(ACTIVATE, "标准行动：移除一个本方时间标志，遏制1", Timing::Standard,
            vec![Cost::RemoveOneTimeFromSelectedControlledCard],
            vec![target(Zone::Player, EntityKind::Any, Relation::Any, Range::Anywhere)],
            vec![Op::RepressOpponentOne { slot: 0 }], None),
    ])
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    let repress = id == "JC044" && a.ops.iter().any(|op| matches!(op, Op::RepressOpponentOne { .. }));
    if (own(a) || repress) && (id != "JC044" || !definition().abilities.iter().any(|c| same(c, a))) {
        return Err("JC044只准入完整时间标志进场和标准遏制程序".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if id == "JC044" && !same(d, &definition()) || id != "JC044" && d.abilities.iter().any(own) {
        return Err("JC044的时间标志与移除时间费用不能拆分、改写或移植".into());
    }
    Ok(())
}
impl Game {
    /// Actual marked carriers, never an effective host-plus-attachments total.
    pub(crate) fn jc044_time_payments(&self, actor: usize) -> Vec<TimeMarkerPayment> {
        let mut payments = vec![];
        for (region, r) in self.regions.iter().enumerate().filter(|(_, r)| !r.vacant) {
            for c in &r.cards {
                if c.controller == actor && !c.face_down && c.time_markers > 0
                    && catalog::card(&c.definition).kind == "character" {
                    payments.push(TimeMarkerPayment { carrier: c.clone(), region: Some(region),
                        attachment_host: None, remaining: c.time_markers - 1 });
                }
            }
        }
        for a in &self.attachments {
            let c = &a.card;
            if c.controller == actor && !c.face_down && c.time_markers > 0 && self.attachment_host_valid(a) {
                payments.push(TimeMarkerPayment { carrier: c.clone(), region: self.attachment_region(a),
                    attachment_host: Some(a.host_id.clone()), remaining: c.time_markers - 1 });
            }
        }
        payments
    }
    pub(crate) fn jc044_pay_time(&mut self, actor: usize, a: &Action) -> RuleResult<PaidCost> {
        let ids = a.cost_selected.as_ref().filter(|ids| ids.len() == 1).ok_or("请选择一个本方时间标志载体作为费用")?;
        let payment = self.jc044_time_payments(actor).into_iter().find(|p| p.carrier.id == ids[0])
            .ok_or("时间费用必须来自自己操控的真实在场载体")?;
        self.board_mut(&payment.carrier.id).ok_or("时间费用载体已离场")?.time_markers = payment.remaining;
        self.note(format!("{} 从{}移除1个时间标志作为费用", self.players[actor].name, catalog::card(&payment.carrier.definition).name));
        Ok(PaidCost::RemovedTime(payment))
    }
    pub(crate) fn jc044_check_activation(&self, source: &SourceSnapshot, a: &Action) -> RuleResult<()> {
        if source.card.definition == "JC044" && (a.kind != "activate"
            || a.ability_id.as_deref().is_some_and(|key| key != ACTIVATE)
            || a.region.is_some() || a.option.is_some() || a.choice_id.is_some() || a.selected.is_some()
            || a.top.is_some() || a.bottom.is_some() || a.allocations.is_some() || a.deck_draft.is_some()) {
            return Err("JC044标准行动不接受额外模式、地区或待选载荷".into());
        }
        Ok(())
    }
    fn jc044_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        if actor >= self.players.len() || s.card.controller != actor || s.card.owner >= self.players.len()
            || s.card.definition != "JC044" || s.card.id.is_empty()
            || s.region.is_none_or(|r| r >= self.regions.len())
            || s.source_region_instance.as_ref().is_none_or(|id| id.is_empty() || id == &s.card.id)
            || s.attachment_host_instance.is_some() || s.observed_death.is_some() || s.observed_mobility_grant.is_some() {
            return Err("JC044冻结角色来源、行动者或原地区无效".into());
        }
        let cards = self.players.iter().flat_map(|p| p.hand.iter().chain(&p.deck).chain(&p.assets)
            .chain(&p.graveyard).chain(&p.score_cards).chain(p.society_zone.card.iter()));
        if self.regions.iter().flat_map(|r| std::iter::once(&r.card).chain(&r.cards))
            .chain(cards).chain(&self.world).chain(self.attachments.iter().map(|a| &a.card)).chain(self.sealed_cards.iter().map(|s| &s.card))
            .any(|c| c.id == s.card.id && (c.definition != s.card.definition || c.owner != s.card.owner)) {
            return Err("JC044冻结来源与现有UID的印刷身份或拥有者矛盾".into());
        }
        Ok(())
    }
    fn jc044_trigger(&self, key: &str, s: &SourceSnapshot) -> RuleResult<AbilitySpec> {
        if key == ENTRY { return Ok(definition().abilities[0].clone()); }
        let region = s.source_region_instance.as_deref().ok_or("JC044奖励缺少原地区")?;
        match key {
            "renown" => Ok(crate::renown::renown_ability(region)),
            "jc089-combat-glory" => Ok(crate::rules::jc089_combat_glory_ability(region)),
            _ => Err("JC044未知触发程序".into()),
        }
    }
    fn jc044_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if self.validated_cross_card_declaration(d)? { return Ok(()); }
        if d.source.card.definition != "JC044" && !own(&d.ability) { return Ok(()); }
        self.jc044_source(d.actor, &d.source)?;
        if d.source.card.face_down || d.source.play_source.is_some()
            || !same(&d.ability, &self.jc044_trigger(&d.ability.key, &d.source)?) {
            return Err("JC044触发声明与完整印刷程序不符".into());
        }
        Ok(())
    }
    fn jc044_player(&self, f: &ResolutionFrame) -> RuleResult<usize> {
        let t = f.targets.first().ok_or("JC044缺少目标玩家")?;
        let seat = t.id.strip_prefix('p').and_then(|s| s.parse::<usize>().ok())
            .filter(|s| *s < self.players.len() && t.id == player_id(*s)).ok_or("JC044玩家标识无效")?;
        if f.targets.len() != 1 || !same(&t.spec, &definition().abilities[1].targets[0])
            || t.region_instance.is_some() || t.public.instance_id != t.id || t.public.kind != "player"
            || t.public.owner.is_some() || t.public.controller.as_deref() != Some(t.id.as_str())
            || t.public.region.is_some() || t.public.label != self.players[seat].name
            || !t.public.valid || t.public.status != "valid" || t.public.invalid_reason.is_some() {
            return Err("JC044遏制的目标玩家绑定无效".into());
        }
        Ok(seat)
    }
    pub(crate) fn validate_jc044_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if self.validated_cross_card_frame(f)? { return Ok(()); }
        if !related(f) { return Ok(()); }
        self.jc044_source(f.actor, &f.source)?;
        if f.frame_id.is_empty() { return Err("JC044缺少帧身份".into()); }
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            if !f.steps.is_empty() || !f.targets.is_empty() || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked)
                || f.chosen_region != f.source.region || f.source.card.time_markers != 0
                || (if f.ability_key == "deploy" { f.source.card.face_down || !f.source.valid_character_play() }
                    else { !f.source.card.face_down || f.source.play_source.is_some() })
                || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if ids.iter().all(|id| !id.is_empty()) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len()) {
                return Err("JC044派遣或付费现身空程序无效".into());
            }
            return Ok(());
        }
        let ability = if f.ability_key == ACTIVATE { definition().abilities[1].clone() }
            else { self.jc044_trigger(&f.ability_key, &f.source)? };
        let steps: Vec<_> = ability.ops.iter().map(|op| Step { context: f.actor, op: op.clone() }).collect();
        if f.source.card.face_down || f.source.play_source.is_some() || f.chosen_region.is_some()
            || !same(&f.steps, &steps) || f.cursor > steps.len() {
            return Err("JC044冻结程序、步骤或游标被改写".into());
        }
        if f.ability_key == ACTIVATE {
            self.jc044_player(f)?;
            let [PaidCost::RemovedTime(p)] = f.already_paid.as_slice() else { return Err("JC044须恰好支付一个时间标志".into()); };
            let c = &p.carrier;
            let kind = catalog::catalog().cards.iter().find(|d| d.id == c.definition && catalog::runtime_implemented(d)).map(|d| d.kind.as_str());
            if c.id.is_empty() || c.owner >= self.players.len() || c.controller != f.actor || c.face_down
                || c.time_markers.checked_sub(1) != Some(p.remaining)
                || p.region.is_some_and(|r| r >= self.regions.len())
                || (match kind { Some("character") => p.region.is_none() || (c.definition != "BQ026" && p.attachment_host.is_some()) || p.attachment_host.as_ref().is_some_and(|id|id.is_empty() || id==&c.id),
                    Some("attachment") => p.attachment_host.as_ref().is_none_or(|id| id.is_empty() || id == &c.id), _ => true })
                || (c.id == f.source.card.id && (!same(c, &f.source.card) || p.region != f.source.region)) {
                return Err("JC044时间费用的真实载体、控制权或单枚扣除凭据无效".into());
            }
            if self.regions.iter().flat_map(|r| &r.cards).chain(self.attachments.iter().map(|a| &a.card))
                .any(|live| live.id == c.id && (live.definition != c.definition || live.owner != c.owner)) {
                return Err("JC044时间费用与现有UID印刷身份或拥有者矛盾".into());
            }
        } else if !f.already_paid.is_empty() || !f.targets.is_empty() {
            return Err("JC044进场/获授触发不能夹带支付或目标".into());
        }
        Ok(())
    }
    pub(crate) fn jc044_add_time(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        self.validate_jc044_frame(f)?;
        if f.ability_key != ENTRY || f.cursor != 1 || !matches!(f.guard, GuardState::Accepted) {
            return Err("JC044时间标志须来自已接受的进场触发".into());
        }
        if let Some(c) = self.board_mut(&f.source.card.id).filter(|c| c.definition == "JC044" && !c.face_down) {
            c.time_markers = c.time_markers.saturating_add(1);
        }
        Ok(())
    }
    pub(crate) fn validate_jc044_state(&self) -> RuleResult<()> {
        let mut frames = BTreeSet::new();
        for item in &self.stack {
            if let Some(f) = &item.frame {
                self.validate_jc044_frame(f)?;
                if related(f) {
                    if !frames.insert(f.frame_id.clone()) || item.id != f.frame_id || item.controller != f.actor
                        || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked) { return Err("JC044堆栈容器无效".into()); }
                    let entry = matches!(f.ability_key.as_str(), "deploy" | "reveal");
                    if entry {
                        if item.target.is_some() || item.deploy_region != f.source.region || item.reveal != (f.ability_key == "reveal")
                            || item.card.as_ref().is_none_or(|c| { let mut expected = f.source.card.clone(); expected.id = c.id.clone();
                                expected.face_down = false; expected.time_markers = 0; expected.charge_markers = 0; expected.lock_markers = 0;
                                c.id.is_empty() || c.id == f.source.card.id || !same(c, &expected) }) {
                            return Err("JC044派遣容器与冻结来源不符".into());
                        }
                    } else if item.card.is_some() || item.deploy_region.is_some() || item.reveal
                        || item.target != f.targets.first().map(|t| t.id.clone()) { return Err("JC044能力容器不能夹带牌或改写目标".into()); }
                } else if item.card.as_ref().is_some_and(|c| c.definition == "JC044") {
                    return Err("JC044不能挂接其他牌的堆栈程序".into());
                }
            } else if item.card.as_ref().is_some_and(|c| c.definition == "JC044") { return Err("JC044堆栈缺少冻结帧".into()); }
        }
        for e in &self.effects {
            match e {
                Effect::Declare { declaration } => self.jc044_declaration(declaration)?,
                Effect::Frame { frame } => { self.validate_jc044_frame(frame)?;
                    if related(frame) && (!frames.insert(frame.frame_id.clone())
                        || matches!(frame.ability_key.as_str(), ENTRY | ACTIVATE)
                            && (frame.cursor != 1 || !matches!(frame.guard, GuardState::Accepted))) {
                        return Err("JC044续体不能重复或绕过响应堆栈".into());
                    } },
                _ => {},
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Declare { declaration: d, stage } => {
                    self.jc044_declaration(d)?;
                    if d.source.card.definition == "JC044" && !self.validated_cross_card_declaration(d)? {
                        let expected = Choice { id: p.choice.id.clone(), kind: "trigger".into(),
                            title: format!("{}：{}", catalog::card("JC044").name, d.ability.label),
                            description: "选择符合数量限制的选项，然后确认。".into(), player_id: player_id(d.actor),
                            options: vec![ChoiceOption { id: "accept".into(), label: "发动触发能力".into(), card: None }],
                            preview_cards: vec![], min: Some(0), max: Some(1), amount: None, allow_decline: Some(true) };
                        if self.status != "playing" || p.seat != d.actor || p.choice.id.is_empty() || !matches!(stage, DeclareChoice::Accept)
                            || self.players[d.actor].eliminated || !same(&p.choice, &expected) { return Err("JC044触发待选必须绑定原行动者与完整Accept程序".into()); }
                    }
                },
                ChoiceResolution::Frame { frame, choice } => {
                    self.validate_jc044_frame(frame)?;
                    if related(frame) {
                        if !frames.insert(frame.frame_id.clone()) || frame.ability_key != ACTIVATE || frame.cursor != 1
                            || !matches!(frame.guard, GuardState::Accepted) { return Err("JC044遏制续体或响应守卫无效".into()); }
                        let FrameChoice::RepressOne { seat, regions } = choice else { return Err("JC044只能等待目标玩家移除势力".into()); };
                        let target = self.jc044_player(frame)?; let team = self.team(target);
                        let expected_regions: Vec<_> = self.regions.iter().enumerate().filter(|(_, r)| !r.vacant && r.influence[team] > 0)
                            .map(|(r, region)| (r, region.card.id.clone())).collect();
                        let options = expected_regions.iter().map(|(r, _)| ChoiceOption { id: format!("region:{r}"),
                            label: format!("地区{}：{}（本方势力{}）", r + 1, catalog::card(&self.regions[*r].card.definition).name, self.regions[*r].influence[team]), card: None }).collect();
                        let expected = Choice { id: p.choice.id.clone(), kind: "target".into(), title: "遏制1：选择移除本方势力的地区".into(),
                            description: "选择符合数量限制的选项，然后确认。".into(), player_id: player_id(target), options,
                            preview_cards: vec![], min: Some(1), max: Some(1), amount: None, allow_decline: Some(false) };
                        if self.status != "playing" || p.seat != target || *seat != target
                            || self.players[frame.actor].eliminated || self.players[target].eliminated
                            || p.choice.id.is_empty() || expected_regions.is_empty()
                            || !same(regions, &expected_regions) || !same(&p.choice, &expected) { return Err("JC044遏制待选的玩家、地区或完整元数据不符".into()); }
                    }
                },
                _ => {},
            }
        }
        Ok(())
    }
}
