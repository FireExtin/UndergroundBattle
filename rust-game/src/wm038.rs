//! Complete fixed WM038: human attachment, cultist grant, original-host time and blood-play discount.
use crate::{
    catalog,
    engine::RuleResult,
    model::*,
    rules::{self, *},
};
use std::collections::BTreeSet;
const PLAY: &str = "wm038-attach-human";
const ENTRY: &str = "wm038-entry-host-time";
const FAST: &str = "wm038-sacrifice-cultist-blood-discount";
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
fn has_op(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::WM038AddOneTimeToOriginalHost
        | Op::CostReduction {
            filter: CardFilter::WM038BloodCard,
            ..
        } => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => has_op(body),
        Op::IfTargetExhausted {
            exhausted, ready, ..
        } => has_op(std::slice::from_ref(exhausted)) || has_op(std::slice::from_ref(ready)),
        _ => false,
    })
}
fn key(k: &str) -> bool {
    matches!(k, PLAY | ENTRY | FAST)
}
fn related(f: &ResolutionFrame) -> bool {
    f.source.card.definition == "WM038"
        || key(&f.ability_key)
        || f.steps.iter().any(|s| has_op(std::slice::from_ref(&s.op)))
}
pub(crate) fn definition() -> Definition {
    let mut host = rules::target(
        Zone::Board,
        EntityKind::Character,
        Relation::Any,
        Range::Anywhere,
    );
    host.subtype = Some("人类".into());
    let mut play = rules::ability(
        PLAY,
        "标准行动：结附于目标人类角色",
        Timing::Standard,
        vec![],
        vec![host.clone()],
        vec![],
        None,
    );
    play.play_only = true;
    let entry = rules::ability(
        ENTRY,
        "进场触发：在原宿主上放置一个时间标志",
        Timing::Fast,
        vec![],
        vec![],
        vec![Op::WM038AddOneTimeToOriginalHost],
        Some(Event::Enter),
    );
    let mut fast = rules::ability(
        FAST,
        "快速行动：牺牲邪教徒，下一张血脉领域牌减费2",
        Timing::Fast,
        vec![
            Cost::ExhaustSource,
            Cost::SacrificeSelectedControlledCharacter,
        ],
        vec![],
        vec![Op::CostReduction {
            filter: CardFilter::WM038BloodCard,
            amount: 2,
        }],
        None,
    );
    fast.activation_only = true;
    Definition {
        abilities: vec![play, entry, fast],
        modifiers: vec![StaticModifier::WM038HostCultistSubtype],
        attachment: Some(AttachmentSpec {
            controls_host: false,
            host_subtype_change: SubtypeChange::None,
            host,
            host_icons: Icons::default(),
            host_temporary_icons: None,
            host_barrier: false,
            host_defense_bonus: 0,
            host_leaves: HostLeaveDestination::OwnerGraveyard,
        }),
        ..Definition::default()
    }
}
pub(crate) fn validate_play_action(id: &str, a: &Action) -> RuleResult<()> {
    if id == "WM038"
        && (a.region.is_some()
            || a.target_id.is_none()
            || a.ability_id.as_deref().is_some_and(|s| s != PLAY)
            || a.option.is_some()
            || a.choice_id.is_some()
            || a.selected.is_some()
            || a.top.is_some()
            || a.bottom.is_some()
            || a.allocations.is_some()
            || a.cost_selected.is_some()
            || a.deck_draft.is_some())
    {
        return Err("WM038只接受人类角色结附目标".into());
    }
    Ok(())
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if id == "WM038" || key(&a.key) || has_op(&a.ops) || a.modes.iter().any(|m| has_op(&m.ops)) {
        if id != "WM038" || !definition().abilities.iter().any(|x| same(a, x)) {
            return Err("只准入完整WM038结附、原宿主时间及血脉减费程序".into());
        }
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if (id == "WM038" && !same(d, &definition()))
        || (id != "WM038"
            && (d
                .modifiers
                .iter()
                .any(|m| matches!(m, StaticModifier::WM038HostCultistSubtype))
                || d.abilities.iter().any(|a| {
                    key(&a.key) || has_op(&a.ops) || a.modes.iter().any(|m| has_op(&m.ops))
                })))
    {
        return Err("WM038完整定义不能改写或移植".into());
    }
    Ok(())
}
impl Game {
    pub(crate) fn wm038_grants_cultist(&self, c: &Card) -> bool {
        // Do not call attachment_host_valid here: that queries current_subtypes itself.
        !c.face_down
            && catalog::card(&c.definition).kind == "character"
            && self.regions.iter().any(|r| {
                !r.vacant
                    && r.cards
                        .iter()
                        .any(|live| live.id == c.id && !live.face_down)
            })
            && self
                .attachments
                .iter()
                .any(|a| a.card.definition == "WM038" && !a.card.face_down && a.host_id == c.id)
    }
    pub(crate) fn validate_wm038_activation_cost(
        &self,
        actor: usize,
        source: &SourceSnapshot,
        spec: &AbilitySpec,
        a: &Action,
    ) -> RuleResult<()> {
        if source.card.definition != "WM038" || spec.key != FAST {
            return Ok(());
        }
        validate_ability("WM038", spec)?;
        if a.region.is_some()
            || a.target_id.is_some()
            || a.option.is_some()
            || a.choice_id.is_some()
            || a.selected.is_some()
            || a.top.is_some()
            || a.bottom.is_some()
            || a.allocations.is_some()
            || a.deck_draft.is_some()
        {
            return Err("WM038减费行动不能添加目标或额外程序".into());
        }
        let ids = a
            .cost_selected
            .as_ref()
            .filter(|x| x.len() == 1)
            .ok_or("须选择一个自己操控的明置邪教徒支付牺牲费用")?;
        let (_, c) = self
            .board(&ids[0])
            .filter(|(_, c)| {
                c.controller == actor
                    && !c.face_down
                    && catalog::card(&c.definition).kind == "character"
                    && self.current_subtypes(c).iter().any(|s| s == "邪教徒")
            })
            .ok_or("牺牲费用必须是当前自己操控的邪教徒角色")?;
        if c.id == source.card.id {
            return Err("WM038附属不能作为邪教徒角色费用".into());
        }
        Ok(())
    }
    pub(crate) fn wm038_add_time(&mut self, f: &ResolutionFrame) {
        if let Some(id) = &f.source.attachment_host_instance {
            if let Some(c) = self
                .board_mut(id)
                .filter(|c| !c.face_down && catalog::card(&c.definition).kind == "character")
            {
                c.time_markers = c.time_markers.saturating_add(1);
            }
        }
    }
    fn validate_wm038_source(
        &self,
        s: &SourceSnapshot,
        actor: usize,
        in_play: bool,
    ) -> RuleResult<()> {
        let c = &s.card;
        if c.definition != "WM038"
            || c.id.is_empty()
            || actor >= self.players.len()
            || c.owner >= self.players.len()
            || c.controller != actor
            || c.face_down
            || s.play_source.is_some()
            || s.observed_death.is_some()
            || s.source_region_instance.is_some()
            || if in_play {
                s.region.is_none_or(|r| r >= self.regions.len())
                    || s.attachment_host_instance
                        .as_ref()
                        .is_none_or(|id| id.is_empty() || id == &c.id)
            } else {
                s.region.is_some() || s.attachment_host_instance.is_some()
            }
        {
            return Err("WM038来源或原宿主快照无效".into());
        }
        Ok(())
    }
    pub(crate) fn validate_wm038_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if !related(f) {
            return Ok(());
        }
        self.validate_wm038_source(&f.source, f.actor, f.ability_key != PLAY)?;
        if f.frame_id.is_empty() || f.cursor != 0 || f.chosen_region.is_some() {
            return Err("WM038帧身份、游标或额外地区无效".into());
        }
        let d = definition();
        match f.ability_key.as_str() {
            PLAY => {
                let t = f.targets.first().ok_or("WM038缺少人类宿主目标")?;
                if !f.steps.is_empty()
                    || f.targets.len() != 1
                    || !same(&t.spec, &d.abilities[0].targets[0])
                    || t.id.is_empty()
                    || t.region_instance.is_some()
                    || t.public.instance_id != t.id
                    || t.public.kind != "character"
                    || !t.public.valid
                    || t.public.invalid_reason.is_some()
                    || !matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)] if ids.len()<=1&&ids.iter().all(|id|!id.is_empty())&&ids.iter().collect::<BTreeSet<_>>().len()==ids.len())
                {
                    return Err("WM038结附目标或印刷支付无效".into());
                }
            }
            ENTRY => {
                if !f.targets.is_empty()
                    || !f.already_paid.is_empty()
                    || f.steps.len() != 1
                    || !matches!(f.steps[0].op, Op::WM038AddOneTimeToOriginalHost)
                    || f.steps[0].context != f.actor
                {
                    return Err("WM038进场只能给原宿主放置一个时间标志".into());
                }
            }
            FAST => {
                if !f.targets.is_empty()
                    || !same(
                        &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>(),
                        &d.abilities[2].ops,
                    )
                    || f.steps.iter().any(|s| s.context != f.actor)
                    || f.source.card.exhausted
                    || !matches!(f.already_paid.as_slice(),[PaidCost::Exhausted(id),PaidCost::Sacrificed{old_instance,controller,owner}]
                    if id==&f.source.card.id&&!old_instance.is_empty()&&old_instance!=id&&*controller==f.actor&&*owner<self.players.len())
                {
                    return Err("WM038减费程序或横置、牺牲凭据无效".into());
                }
            }
            _ => return Err("WM038未知运行程序".into()),
        }
        Ok(())
    }
    fn validate_wm038_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if d.source.card.definition == "WM038" || key(&d.ability.key) || has_op(&d.ability.ops) {
            self.validate_wm038_source(&d.source, d.actor, true)?;
            if !same(&d.ability, &definition().abilities[1]) {
                return Err("WM038只有完整原宿主时间进场触发声明".into());
            }
        }
        Ok(())
    }
    pub(crate) fn validate_wm038_state(&self) -> RuleResult<()> {
        let mut ids = BTreeSet::new();
        for a in self
            .attachments
            .iter()
            .filter(|a| a.card.definition == "WM038")
        {
            if a.card.id.is_empty()
                || !ids.insert(&a.card.id)
                || a.card.owner >= self.players.len()
                || a.card.controller >= self.players.len()
                || a.card.face_down
                || !self.attachment_host_valid(a)
            {
                return Err("WM038在场实例或人类宿主无效".into());
            }
        }
        for m in &self.modifiers {
            if matches!(m.filter, CardFilter::WM038BloodCard)
                && (m.actor >= self.players.len()
                    || m.amount != 2
                    || m.uses > 1
                    || m.paid_reveal
                    || m.expires_turn > self.turn)
            {
                return Err("WM038血脉打出减费状态无效".into());
            }
        }
        for s in &self.stack {
            if let Some(f) = &s.frame {
                self.validate_wm038_frame(f)?;
                if related(f) || s.card.as_ref().is_some_and(|c| c.definition == "WM038") {
                    if !related(f)
                        || !matches!(f.guard, GuardState::Unchecked)
                        || s.id != f.frame_id
                        || s.controller != f.actor
                        || s.deploy_region.is_some()
                        || s.reveal
                        || s.target != f.targets.first().map(|t| t.id.clone())
                        || if f.ability_key == PLAY {
                            s.card.as_ref().is_none_or(|c| {
                                c.definition != "WM038"
                                    || c.face_down
                                    || c.owner != f.source.card.owner
                                    || c.controller != f.actor
                            })
                        } else {
                            s.card.is_some()
                        }
                    {
                        return Err("WM038响应堆栈外壳无效".into());
                    }
                }
            } else if s.card.as_ref().is_some_and(|c| c.definition == "WM038") {
                return Err("WM038结附缺少冻结帧".into());
            }
        }
        for e in &self.effects {
            match e {
                Effect::Frame { frame: f } => {
                    self.validate_wm038_frame(f)?;
                    if related(f) {
                        return Err("WM038不能绕过响应堆栈".into());
                    }
                }
                Effect::Declare { declaration: d } => self.validate_wm038_declaration(d)?,
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Declare {
                    declaration: d,
                    stage,
                } => {
                    self.validate_wm038_declaration(d)?;
                    if d.source.card.definition == "WM038"
                        && (!matches!(stage, DeclareChoice::Accept) || p.seat != d.actor)
                    {
                        return Err("WM038进场接受待选座位或阶段无效".into());
                    }
                }
                ChoiceResolution::Frame { frame: f, .. } if related(f) => {
                    return Err("WM038没有结算续体待选".into())
                }
                _ => {}
            }
        }
        Ok(())
    }
}
