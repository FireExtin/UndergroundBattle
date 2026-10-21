//! Complete JC057: a private Reveal inspection and a live, non-targeted read permission.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use std::collections::BTreeSet;

pub(crate) const KEY: &str = "jc057-reveal-inspect-hand";
const CHOICE: &str = "jc057-hand-inspect";
const TITLE: &str = "监控员：检视目标玩家手牌";
const DESCRIPTION: &str = "仅你可见。确认后结束检视，不弃牌、不公开展示。";
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
fn uses(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::JC057InspectTargetHand => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => uses(body),
        Op::IfTargetExhausted { exhausted, ready, .. } =>
            uses(std::slice::from_ref(exhausted)) || uses(std::slice::from_ref(ready)),
        _ => false,
    })
}
fn own(a: &AbilitySpec) -> bool {
    a.key == KEY || uses(&a.ops) || a.modes.iter().any(|m| uses(&m.ops))
}
fn related(f: &ResolutionFrame) -> bool {
    f.source.card.definition == "JC057" || f.ability_key == KEY
        || f.steps.iter().any(|s| uses(std::slice::from_ref(&s.op)))
}
pub(crate) fn definition() -> Definition {
    Definition {
        abilities: vec![ability(KEY, "现身触发：检视目标玩家的手牌", Timing::Fast,
            vec![], vec![target(Zone::Player, EntityKind::Any, Relation::Any, Range::Anywhere)],
            vec![Op::JC057InspectTargetHand], Some(Event::Reveal))],
        modifiers: vec![StaticModifier::JC057PrivateLocalHiddenInspection],
        ..Default::default()
    }
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if own(a) && (id != "JC057" || !same(a, &definition().abilities[0])) {
        return Err("JC057只准入完整的可选现身手牌检视程序".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    let related = id == "JC057" || d.abilities.iter().any(own)
        || d.modifiers.iter().any(|m| matches!(m, StaticModifier::JC057PrivateLocalHiddenInspection));
    if related && (id != "JC057" || !same(d, &definition())) {
        return Err("JC057现身检视与持续权限不能拆分、改写或移植".into());
    }
    Ok(())
}

impl Game {
    /// This projection grants information, never a target, action, new identity or stored permission.
    pub(crate) fn jc057_private_inspections(&self, viewer: usize) -> Vec<PrivateInspectionView> {
        if self.status != "playing" || self.players.get(viewer).is_none_or(|p| p.eliminated) {
            return vec![];
        }
        self.regions.iter().enumerate().filter(|(_, r)| !r.vacant).flat_map(|(region, r)| {
            r.cards.iter().filter(move |source| source.definition == "JC057"
                && source.controller == viewer && !source.face_down && !source.exhausted)
                .filter_map(move |source| {
                    let cards: Vec<_> = r.cards.iter().filter(|c| c.face_down && self.jc057_inspection_known(viewer,&source.id,&c.id,region)).map(|c| {
                        let mut front = c.clone(); front.face_down = false;
                        // Inspect printed information; a hidden entity does not gain face-up stats.
                        self.card_view(&front, viewer, None, None)
                    }).collect();
                    (!cards.is_empty()).then(|| PrivateInspectionView {
                        source_instance_id: source.id.clone(), region, cards,
                    })
                })
        }).collect()
    }
    fn jc057_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        if actor >= self.players.len() || s.card.controller != actor || s.card.owner >= self.players.len()
            || s.card.definition != "JC057" || s.card.id.is_empty()
            || s.region.is_none_or(|r| r >= self.regions.len())
            || s.source_region_instance.as_ref().is_none_or(|id| id.is_empty() || id == &s.card.id)
            || s.attachment_host_instance.is_some() || s.observed_death.is_some()
            || s.observed_mobility_grant.is_some() {
            return Err("JC057冻结来源、行动者或地区无效".into());
        }
        let owner_cards = self.players.iter().flat_map(|p| p.hand.iter().chain(&p.deck)
            .chain(&p.assets).chain(&p.graveyard).chain(&p.score_cards).chain(p.society_zone.card.iter()));
        if self.regions.iter().flat_map(|r| std::iter::once(&r.card).chain(&r.cards))
            .chain(owner_cards).chain(&self.world).chain(self.attachments.iter().map(|a| &a.card))
            .chain(self.sealed_cards.iter().map(|s| &s.card))
            .any(|c| c.id == s.card.id && (c.definition != s.card.definition || c.owner != s.card.owner)) {
            return Err("JC057冻结来源与现有UID的印刷身份或拥有者矛盾".into());
        }
        Ok(())
    }
    fn jc057_trigger(&self, key: &str, s: &SourceSnapshot) -> RuleResult<AbilitySpec> {
        match key {
            KEY => Ok(definition().abilities[0].clone()),
            "renown" | "jc089-combat-glory" => {
                let region = s.source_region_instance.as_deref().filter(|id| !id.is_empty()).ok_or("JC057奖励缺少冻结地区")?;
                Ok(if key == "renown" { crate::renown::renown_ability(region) }
                    else { crate::rules::jc089_combat_glory_ability(region) })
            },
            _ => Err("JC057未知触发程序".into()),
        }
    }
    fn jc057_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if self.validated_cross_card_declaration(d)? { return Ok(()); }
        if d.source.card.definition != "JC057" && !own(&d.ability) { return Ok(()); }
        self.jc057_source(d.actor, &d.source)?;
        if d.source.card.face_down || d.source.play_source.is_some()
            || !same(&d.ability, &self.jc057_trigger(&d.ability.key, &d.source)?) {
            return Err("JC057现身或获授触发声明无效".into());
        }
        Ok(())
    }
    fn jc057_pending_declaration(&self, p: &Pending, d: &Declaration, stage: &DeclareChoice) -> RuleResult<()> {
        self.jc057_declaration(d)?;
        if d.source.card.definition != "JC057" && !own(&d.ability) { return Ok(()); }
        if p.seat != d.actor || p.choice.player_id != player_id(d.actor) {
            return Err("JC057触发待选必须属于冻结行动者".into());
        }
        // Mobility has its own canonical pending-container validation. Its
        // source snapshot intentionally differs from a Reveal/reward snapshot.
        if self.validated_cross_card_declaration(d)? { return Ok(()); }
        let options = match (d.ability.targets.as_slice(), stage) {
            ([slot], DeclareChoice::Target) => self.binding_options(d.actor, &d.source, slot),
            ([], DeclareChoice::Accept) => vec![ChoiceOption {
                id: "accept".into(), label: "发动触发能力".into(), card: None,
            }],
            _ => return Err("JC057触发待选阶段与冻结程序不符".into()),
        };
        let expected = Choice {
            id: p.choice.id.clone(), kind: "trigger".into(),
            title: format!("{}：{}", catalog::card(&d.source.card.definition).name, d.ability.label),
            description: "选择符合数量限制的选项，然后确认。".into(),
            player_id: player_id(d.actor), options, preview_cards: vec![],
            min: Some(0), max: Some(1), amount: None, allow_decline: Some(true),
        };
        if self.status != "playing" || self.players[d.actor].eliminated || p.choice.id.is_empty()
            || expected.options.is_empty() || !same(&p.choice, &expected) {
            return Err("JC057触发待选元数据或真实候选不符".into());
        }
        Ok(())
    }
    fn jc057_player(&self, f: &ResolutionFrame) -> RuleResult<usize> {
        let t = f.targets.first().ok_or("JC057缺少目标玩家")?;
        let seat = t.id.strip_prefix('p').and_then(|s| s.parse::<usize>().ok())
            .filter(|s| *s < self.players.len() && t.id == player_id(*s)).ok_or("JC057玩家标识无效")?;
        if f.targets.len() != 1 || !same(&t.spec, &definition().abilities[0].targets[0])
            || t.region_instance.is_some() || t.public.instance_id != t.id || t.public.kind != "player"
            || t.public.owner.is_some() || t.public.controller.as_deref() != Some(t.id.as_str())
            || t.public.region.is_some() || t.public.label != self.players[seat].name
            || !t.public.valid || t.public.status != "valid" || t.public.invalid_reason.is_some() {
            return Err("JC057目标玩家绑定无效".into());
        }
        Ok(seat)
    }
    pub(crate) fn validate_jc057_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if self.validated_cross_card_frame(f)? { return Ok(()); }
        if !related(f) { return Ok(()); }
        self.jc057_source(f.actor, &f.source)?;
        if f.frame_id.is_empty() { return Err("JC057缺少帧身份".into()); }
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            let s = &f.source;
            if !f.steps.is_empty() || !f.targets.is_empty() || f.cursor != 0
                || !matches!(f.guard, GuardState::Unchecked) || f.chosen_region != s.region
                || (if f.ability_key == "deploy" { s.card.face_down || !s.valid_character_play() }
                    else { !s.card.face_down || s.play_source.is_some() })
                || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if ids.iter().all(|id| !id.is_empty()) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len()) {
                return Err("JC057派遣或现身空程序无效".into());
            }
            return Ok(());
        }
        let a = self.jc057_trigger(&f.ability_key, &f.source)?;
        let steps: Vec<_> = a.ops.iter().map(|op| Step { context: f.actor, op: op.clone() }).collect();
        if f.source.card.face_down || f.source.play_source.is_some() || f.chosen_region.is_some()
            || !f.already_paid.is_empty() || !same(&f.steps, &steps) || f.cursor > steps.len() {
            return Err("JC057冻结触发程序或支付被改写".into());
        }
        if f.ability_key == KEY { self.jc057_player(f)?; }
        else if !f.targets.is_empty() { return Err("JC057获授奖励不指定目标".into()); }
        Ok(())
    }
    pub(crate) fn jc057_inspect_start(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        self.validate_jc057_frame(&f)?;
        if f.ability_key != KEY || f.cursor != 1 || !matches!(f.guard, GuardState::Accepted) {
            return Err("JC057检视必须来自已通过守卫的现身帧".into());
        }
        let seat = self.jc057_player(&f)?;
        if self.players[seat].eliminated { return Ok(()); }
        let inspected = self.players[seat].hand.clone();
        self.jc057_inspected_hand(&inspected)?;
        self.choice(f.actor, CHOICE, TITLE.into(), vec![], 0, 0, None,
            ChoiceResolution::Frame { frame: Box::new(f), choice: FrameChoice::JC057InspectHand { seat, inspected } });
        let p = self.pending.as_mut().unwrap(); p.choice.description = DESCRIPTION.into(); p.choice.allow_decline = Some(false);
        Ok(())
    }
    pub(crate) fn jc057_inspect_complete(&self, f: &ResolutionFrame, seat: usize, inspected: &[Card], chooser: usize, a: &Action, selected: &[String]) -> RuleResult<()> {
        self.validate_jc057_frame(f)?;
        self.jc057_inspected_hand(inspected)?;
        if f.ability_key != KEY || f.cursor != 1 || !matches!(f.guard, GuardState::Accepted)
            || chooser != f.actor || self.jc057_player(f)? != seat
            || !same(&self.players[seat].hand, &inspected.to_vec()) || !selected.is_empty()
            || a.kind != "choose" || a.card_id.is_some() || a.region.is_some() || a.target_id.is_some()
            || a.ability_id.is_some() || a.option.is_some() || a.top.is_some() || a.bottom.is_some()
            || a.allocations.is_some() || a.cost_selected.is_some() || a.deck_draft.is_some() {
            return Err("JC057私密手牌确认或冻结手牌无效".into());
        }
        Ok(())
    }
    fn jc057_inspected_hand(&self, inspected: &[Card]) -> RuleResult<()> {
        let mut ids = BTreeSet::new();
        for c in inspected {
            if c.id.is_empty() || !ids.insert(&c.id) || c.face_down
                || c.owner >= self.players.len() || c.controller >= self.players.len()
                || !catalog::catalog().cards.iter().any(|d| d.id == c.definition && catalog::runtime_implemented(d) && d.kind != "region") {
                return Err("JC057检视手牌的实体或印刷定义无效".into());
            }
        }
        Ok(())
    }
    pub(crate) fn jc057_private_choice(&self, p: &Pending, viewer: usize) -> Choice {
        let mut choice = p.choice.clone();
        if let ChoiceResolution::Frame { frame, choice: FrameChoice::JC057InspectHand { inspected, .. } } = &p.resolution {
            if viewer == frame.actor { choice.preview_cards = inspected.iter().map(|c| self.card_view(c, viewer, None, None)).collect(); }
        }
        choice
    }
    pub(crate) fn validate_jc057_state(&self) -> RuleResult<()> {
        let mut frames = BTreeSet::new();
        for item in &self.stack {
            if let Some(f) = &item.frame {
                self.validate_jc057_frame(f)?;
                if related(f) && (!frames.insert(f.frame_id.clone()) || item.id != f.frame_id || item.controller != f.actor
                    || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked)) {
                    return Err("JC057堆栈容器或响应检查点无效".into());
                }
                if related(f) {
                    let entry = matches!(f.ability_key.as_str(), "deploy" | "reveal");
                    if entry {
                        if item.target.is_some() || item.deploy_region != f.source.region
                            || item.reveal != (f.ability_key == "reveal")
                            || item.card.as_ref().is_none_or(|c| {
                                let mut expected = f.source.card.clone();
                                expected.id = c.id.clone(); expected.face_down = false;
                                expected.time_markers = 0; expected.lock_markers = 0;
                                c.id.is_empty() || c.id == f.source.card.id || !same(c, &expected)
                            }) {
                            return Err("JC057派遣/现身容器与冻结来源不一致".into());
                        }
                    } else if item.card.is_some() || item.deploy_region.is_some() || item.reveal
                        || item.target != f.targets.first().map(|t| t.id.clone()) {
                        return Err("JC057触发容器不能夹带牌或改写目标".into());
                    }
                } else if item.card.as_ref().is_some_and(|c| c.definition == "JC057") {
                    return Err("JC057不能挂接其他牌的堆栈程序".into());
                }
            } else if item.card.as_ref().is_some_and(|c| c.definition == "JC057") {
                return Err("JC057派遣/现身缺少冻结帧".into());
            }
        }
        for e in &self.effects {
            match e {
                Effect::Frame { frame } => {
                    self.validate_jc057_frame(frame)?;
                    if related(frame) && (!frames.insert(frame.frame_id.clone())
                        || (frame.ability_key == KEY && (frame.cursor != 1 || !matches!(frame.guard, GuardState::Accepted)))) {
                        return Err("JC057检视不能复制或绕过响应堆栈".into());
                    }
                },
                Effect::Declare { declaration } => self.jc057_declaration(declaration)?,
                _ => {},
            }
        }
        if let Some(p) = &self.pending {
            if p.choice.kind == CHOICE && !matches!(&p.resolution, ChoiceResolution::Frame { choice: FrameChoice::JC057InspectHand { .. }, .. }) {
                return Err("JC057待选类型与续体不一致".into());
            }
            match &p.resolution {
                ChoiceResolution::Declare { declaration, stage } => self.jc057_pending_declaration(p, declaration, stage)?,
                ChoiceResolution::Frame { frame, choice } => {
                    self.validate_jc057_frame(frame)?;
                    if related(frame) || matches!(choice, FrameChoice::JC057InspectHand { .. }) {
                        if !frames.insert(frame.frame_id.clone()) { return Err("JC057私密检视不能重复同一帧".into()); }
                        let FrameChoice::JC057InspectHand { seat, inspected } = choice else { return Err("JC057不能挂接其他待选程序".into()); };
                        let a = Action { choice_id: Some(p.choice.id.clone()), selected: Some(vec![]), ..Action::new("choose") };
                        self.jc057_inspect_complete(frame, *seat, inspected, p.seat, &a, &[])?;
                        if p.choice.id.is_empty() || p.choice.kind != CHOICE || p.choice.title != TITLE || p.choice.description != DESCRIPTION
                            || p.choice.player_id != player_id(p.seat) || !p.choice.options.is_empty() || !p.choice.preview_cards.is_empty()
                            || p.choice.min != Some(0) || p.choice.max != Some(0) || p.choice.amount.is_some() || p.choice.allow_decline != Some(false) {
                            return Err("JC057私密检视待选元数据无效".into());
                        }
                    }
                },
                _ => {},
            }
        }
        Ok(())
    }
}
