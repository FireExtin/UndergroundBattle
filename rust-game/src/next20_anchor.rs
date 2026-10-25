//! BQ009 printed dependency only; no new hand/deck-to-society producer.
//! Source: pinned original BQ009 image, SHA256 2f572697eeaec6b66322d5f5b4c15ddb97b89c778c4c8190ca1e2782e198f586.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use std::collections::BTreeSet;
pub(crate) const ID: &str = "BQ009";
pub(crate) const PLAY: &str = "next20-anchor-no-effect";
pub(crate) const SEALED: &str = "next20-anchor-sealed";
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
pub(crate) fn definition() -> Definition {
    let mut play = ability(
        PLAY,
        "结算此牌没有效果",
        Timing::Standard,
        vec![],
        vec![],
        vec![],
        None,
    );
    play.play_only = true;
    let mut sealed = ability(
        SEALED,
        "被封印在本方秘社：可遏制1或预测1",
        Timing::Fast,
        vec![],
        vec![],
        vec![],
        Some(Event::Sealed),
    );
    // Manual p3: repression chooses a target player, who removes their own
    // influence. It does not restrict the target to an opposing team.
    sealed.modes = vec![
        Mode {
            key: "repress".into(),
            label: "遏制1".into(),
            targets: vec![target(
                Zone::Player,
                EntityKind::Any,
                Relation::Any,
                Range::Anywhere,
            )],
            ops: vec![Op::RepressOpponentOne { slot: 0 }],
        },
        Mode {
            key: "forecast".into(),
            label: "预测1".into(),
            targets: vec![],
            ops: vec![Op::Forecast {
                player: PlayerRef::Actor,
                count: 1,
            }],
        },
    ];
    with_abilities(vec![play, sealed])
}
pub(crate) fn related(id: &str, a: &AbilitySpec) -> bool {
    id == ID || a.key.starts_with("next20-anchor-")
}
fn canonical(a: &AbilitySpec) -> Option<AbilitySpec> {
    let d = definition();
    if let Some(p) = d.abilities.iter().find(|p| same(*p, a)) {
        return Some(p.clone());
    }
    let sealed = &d.abilities[1];
    sealed.modes.iter().find_map(|m| {
        let mut p = sealed.clone();
        p.label = format!("{}：{}", p.label, m.label);
        p.targets = m.targets.clone();
        p.ops = m.ops.clone();
        p.modes.clear();
        same(&p, a).then_some(p)
    })
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if id != ID || canonical(a).is_none() {
        return Err("精神锚点只能保留原无效果事务及本人秘社封印模式".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if id == ID && !same(d, &definition())
        || id != ID
            && d.abilities
                .iter()
                .any(|a| a.key.starts_with("next20-anchor-"))
    {
        return Err("精神锚点完整定义不可删改或移植".into());
    }
    Ok(())
}
pub(crate) fn validate_action(id: &str, a: &Action) -> RuleResult<()> {
    if id != ID {
        return Ok(());
    }
    if a.kind != "play"
        || a.ability_id.as_deref().is_some_and(|k| k != PLAY)
        || *a
            != (Action {
                card_id: a.card_id.clone(),
                ability_id: a.ability_id.clone(),
                ..Action::new("play")
            })
    {
        return Err("精神锚点只接受原无效果事务打出".into());
    }
    Ok(())
}
pub(crate) fn validate_choice_action(p: &Pending, a: &Action) -> RuleResult<()> {
    let f = match &p.resolution {
        ChoiceResolution::Frame { frame, .. } if frame.source.card.definition == ID => Some(frame),
        _ => None,
    };
    let related = f.is_some()
        || matches!(&p.resolution,ChoiceResolution::Declare{declaration,..} if self::related(&declaration.source.card.definition,&declaration.ability));
    if !related {
        return Ok(());
    }
    let forecast = matches!(
        &p.resolution,
        ChoiceResolution::Frame {
            choice: FrameChoice::Forecast { .. },
            ..
        }
    );
    let expected = Action {
        choice_id: a.choice_id.clone(),
        selected: a.selected.clone(),
        top: forecast.then(|| a.top.clone()).flatten(),
        bottom: forecast.then(|| a.bottom.clone()).flatten(),
        ..Action::new("choose")
    };
    if *a != expected {
        return Err("精神锚点续体不能夹带其他行动或费用载荷".into());
    }
    Ok(())
}
fn frame_related(f: &ResolutionFrame) -> bool {
    f.source.card.definition == ID || f.ability_key.starts_with("next20-anchor-")
}
impl Game {
    fn anchor_source(&self, actor: usize, s: &SourceSnapshot, sealed: bool) -> RuleResult<()> {
        let c = &s.card;
        if c.definition != ID
            || actor >= self.players.len()
            || c.owner >= self.players.len()
            || c.controller != actor
            || c.face_down
            || !self.mystic_instance_valid(&c.id)
            || s.region.is_some()
            || s.source_region_instance.is_some()
            || s.play_source.is_some()
            || s.death_echo.is_some()
            || s.observed_death.is_some()
            || s.observed_exposure.is_some()
            || s.observed_mobility_grant.is_some()
            || s.attachment_host_instance != sealed.then(|| format!("society:p{actor}"))
        {
            return Err("精神锚点来源或原本人秘社封印见证无效".into());
        }
        self.mobility_immutable_identity(c)
    }
    pub(crate) fn validate_anchor_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if !related(&d.source.card.definition, &d.ability) {
            return Ok(());
        }
        self.anchor_source(d.actor, &d.source, true)?;
        validate_ability(&d.source.card.definition, &d.ability)?;
        if d.ability.key != SEALED {
            return Err("精神锚点只允许实际封印事件触发".into());
        }
        Ok(())
    }
    pub(crate) fn validate_anchor_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if !frame_related(f) {
            return Ok(());
        }
        let play = f.ability_key == PLAY;
        self.anchor_source(f.actor, &f.source, !play)?;
        if f.frame_id.is_empty()
            || !self
                .mystic_instance_number(&f.source.card.id)
                .zip(self.mystic_instance_number(&f.frame_id))
                .is_some_and(|(a, b)| a < b)
            || f.chosen_region.is_some()
            || f.cursor > f.steps.len()
            || f.steps.iter().any(|s| s.context != f.actor)
            || f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked)
            || f.cursor > 0 && !matches!(f.guard, GuardState::Accepted)
        {
            return Err("精神锚点帧来源、游标或守卫无效".into());
        }
        let mut expected = definition().abilities.remove(if play { 0 } else { 1 });
        if !play {
            let mode = expected
                .modes
                .iter()
                .find(|m| {
                    same(
                        &m.ops,
                        &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>(),
                    )
                })
                .cloned()
                .ok_or("精神锚点帧缺少已选原模式")?;
            expected.ops = mode.ops;
            expected.targets = mode.targets;
        }
        if f.ability_key != expected.key
            || !same(
                &expected.ops,
                &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>(),
            )
            || f.targets.len() != expected.targets.len()
        {
            return Err("精神锚点不得增加效果或替换模式".into());
        }
        for (t, slot) in f.targets.iter().zip(&expected.targets) {
            if !same(&t.spec, slot)
                || t.region_instance.is_some()
                || t.public.instance_id != t.id
                || !t
                    .id
                    .strip_prefix('p')
                    .and_then(|s| s.parse::<usize>().ok())
                    .is_some_and(|s| s < self.players.len() && t.id == player_id(s))
            {
                return Err("精神锚点遏制须绑定原目标玩家".into());
            }
        }
        if play {
            if !matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)] if ids.is_empty())
                || !f.resource_payments.is_empty()
            {
                return Err("精神锚点无效果事务仅有真实零费收据".into());
            }
        } else if !f.already_paid.is_empty() || !f.resource_payments.is_empty() {
            return Err("精神锚点封印触发不付额外费用".into());
        }
        Ok(())
    }
    fn validate_anchor_pending(&self, p: &Pending) -> RuleResult<()> {
        match &p.resolution {
            ChoiceResolution::Declare {
                declaration: d,
                stage,
            } if related(&d.source.card.definition, &d.ability) => {
                self.validate_anchor_declaration(d)?;
                let (options, valid) = if !d.ability.modes.is_empty() {
                    (
                        d.ability
                            .modes
                            .iter()
                            .map(|m| ChoiceOption {
                                id: m.key.clone(),
                                label: m.label.clone(),
                                card: None,
                            })
                            .collect(),
                        matches!(stage, DeclareChoice::Mode),
                    )
                } else if let Some(t) = d.ability.targets.first() {
                    (
                        self.binding_options(d.actor, &d.source, t),
                        matches!(stage, DeclareChoice::Target),
                    )
                } else {
                    return Err("精神锚点不能跳过模式选择".into());
                };
                let expected = Choice {
                    id: p.choice.id.clone(),
                    kind: "trigger".into(),
                    title: format!(
                        "{}：{}",
                        crate::batch3_trap::source_label(&d.source),
                        d.ability.label
                    ),
                    description: "选择符合数量限制的选项，然后确认。".into(),
                    player_id: player_id(d.actor),
                    options,
                    preview_cards: vec![],
                    min: Some(0),
                    max: Some(1),
                    amount: None,
                    allow_decline: Some(true),
                };
                if !valid
                    || p.seat != d.actor
                    || p.choice.id.is_empty()
                    || expected.options.is_empty()
                    || !same(&p.choice, &expected)
                {
                    return Err("精神锚点模式或目标待选被改写".into());
                }
            }
            ChoiceResolution::Frame { frame: f, choice } if frame_related(f) => {
                self.validate_anchor_frame(f)?;
                if f.ability_key != SEALED
                    || f.cursor != 1
                    || !matches!(f.guard, GuardState::Accepted)
                    || p.choice.id.is_empty()
                    || p.choice.player_id != player_id(p.seat)
                    || !p.choice.preview_cards.is_empty()
                {
                    return Err("精神锚点效果待选阶段无效".into());
                }
                let (options, seat, kind, title, amount) = match choice {
                    FrameChoice::Forecast { seat }
                        if *seat == f.actor
                            && matches!(
                                f.steps[0].op,
                                Op::Forecast {
                                    player: PlayerRef::Actor,
                                    count: 1
                                }
                            ) =>
                    {
                        let options = self.players[*seat]
                            .deck
                            .iter()
                            .take(1)
                            .map(|c| self.option(c, *seat, None, None))
                            .collect::<Vec<_>>();
                        let n = options.len();
                        (
                            options,
                            *seat,
                            "investigation",
                            format!("预测 {n}"),
                            Some(n as u32),
                        )
                    }
                    FrameChoice::RepressOne { seat, regions }
                        if f.targets.first().is_some_and(|t| t.id == player_id(*seat))
                            && matches!(f.steps[0].op, Op::RepressOpponentOne { slot: 0 }) =>
                    {
                        let current = self
                            .regions
                            .iter()
                            .enumerate()
                            .filter(|(_, r)| !r.vacant && r.influence[self.team(*seat)] > 0)
                            .map(|(i, r)| (i, r.card.id.clone()))
                            .collect::<Vec<_>>();
                        if *regions != current {
                            return Err("精神锚点遏制地区实例被改写".into());
                        }
                        let options = regions
                            .iter()
                            .map(|(r, _)| ChoiceOption {
                                id: format!("region:{r}"),
                                label: format!(
                                    "地区{}：{}（本方势力{}）",
                                    r + 1,
                                    catalog::card(&self.regions[*r].card.definition).name,
                                    self.regions[*r].influence[self.team(*seat)]
                                ),
                                card: None,
                            })
                            .collect();
                        (
                            options,
                            *seat,
                            "target",
                            "遏制1：选择移除本方势力的地区".into(),
                            None,
                        )
                    }
                    _ => return Err("精神锚点只能续接原预测或遏制模式".into()),
                };
                if p.seat != seat
                    || options.is_empty()
                    || p.choice.kind != kind
                    || p.choice.title != title
                    || p.choice.min != Some(1)
                    || p.choice.max != Some(1)
                    || p.choice.allow_decline != Some(false)
                    || p.choice.amount != amount
                    || !same(&p.choice.options, &options)
                {
                    return Err("精神锚点待选须保留原执行者、选项和数量".into());
                }
            }
            _ => (),
        }
        Ok(())
    }
    pub(crate) fn validate_anchor_state(&self) -> RuleResult<()> {
        let mut seen = BTreeSet::new();
        for item in &self.stack {
            if let Some(f) = &item.frame {
                self.validate_anchor_frame(f)?;
                if frame_related(f) {
                    let play = f.ability_key == PLAY;
                    if !seen.insert(f.frame_id.clone())
                        || item.id != f.frame_id
                        || item.controller != f.actor
                        || item.deploy_region.is_some()
                        || item.reveal
                        || item.target != f.targets.first().map(|t| t.id.clone())
                        || item.card.is_some() != play
                        || f.cursor != 0
                        || !matches!(f.guard, GuardState::Unchecked)
                        || item
                            .card
                            .as_ref()
                            .is_some_and(|c| !self.pack_a_transaction_matches(f, c))
                    {
                        return Err("精神锚点响应外壳或原事务实例被改写".into());
                    }
                }
            } else if item.card.as_ref().is_some_and(|c| c.definition == ID) {
                return Err("精神锚点事务缺少原帧".into());
            }
        }
        for e in &self.effects {
            match e {
                Effect::Declare { declaration: d } => self.validate_anchor_declaration(d)?,
                Effect::Frame { frame: f } if frame_related(f) => {
                    return Err("精神锚点不能绕过原响应或待选直接续帧".into())
                }
                _ => (),
            }
        }
        if let Some(p) = &self.pending {
            self.validate_anchor_pending(p)?;
        }
        Ok(())
    }
}
