//! Printed BQ040 reuse and the closed, actor-private BQ028 reveal program.
use crate::{
    catalog,
    engine::RuleResult,
    model::*,
    rules::{self, *},
};

fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}

pub(crate) fn definition(id: &str) -> Option<Definition> {
    match id {
        "BQ030" => Some(crate::bq030::definition()),
        "BQ040" => {
            // Construct the exact XQ14 definition without re-entering the rule
            // registry's OnceLock while that registry is being initialized.
            let mut host = rules::target(
                Zone::Board,
                EntityKind::Character,
                Relation::Any,
                Range::Anywhere,
            );
            host.subtype = Some("吸血鬼".into());
            let mut attach = rules::ability(
                "attach",
                "结附吸血鬼",
                Timing::Standard,
                vec![],
                vec![host.clone()],
                vec![],
                None,
            );
            attach.play_only = true;
            Some(Definition {
                abilities: vec![attach],
                attachment: Some(AttachmentSpec {
                    controls_host: false,
                    host_subtype_change: SubtypeChange::None,
                    host,
                    host_icons: Icons {
                        combat: 1,
                        ..Default::default()
                    },
                    host_temporary_icons: None,
                    host_barrier: false,
                    host_defense_bonus: 1,
                    host_leaves: HostLeaveDestination::OwnerGraveyard,
                }),
                ..Default::default()
            })
        }
        "BQ028" => {
            let mut d = rules::with_abilities(vec![rules::ability(
                "reveal-inspect-attachment",
                "现身触发：检视目标玩家手牌，可令其弃一张附属",
                Timing::Fast,
                vec![],
                vec![rules::target(
                    Zone::Player,
                    EntityKind::Any,
                    Relation::Any,
                    Range::Anywhere,
                )],
                vec![Op::BQ028InspectTargetHandAttachments],
                Some(Event::Reveal),
            )]);
            d.traits.cannot_be_equipped = true;
            Some(d)
        }
        _ => None,
    }
}

pub(crate) fn contains_bq028_op(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::BQ028InspectTargetHandAttachments => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => {
            contains_bq028_op(body)
        }
        Op::IfTargetExhausted {
            exhausted, ready, ..
        } => {
            contains_bq028_op(std::slice::from_ref(exhausted))
                || contains_bq028_op(std::slice::from_ref(ready))
        }
        _ => false,
    })
}

pub(crate) fn validate_ability(card_id: &str, a: &AbilitySpec) -> RuleResult<()> {
    crate::bq030::validate_ability(card_id, a)?;
    if card_id == "BQ028"
        || a.key == "reveal-inspect-attachment"
        || contains_bq028_op(&a.ops)
        || a.modes.iter().any(|m| contains_bq028_op(&m.ops))
    {
        if card_id != "BQ028"
            || (!same(a, &definition("BQ028").unwrap().abilities[0])
                && !crate::jz22::runtime_ability_is_admitted(a))
        {
            return Err("仅准入水沟鼠群密探的完整印刷现身检视能力".into());
        }
    }
    if card_id == "BQ040" && !same(a, &definition("BQ040").unwrap().abilities[0]) {
        return Err("仅准入鲜血BQ040的完整印刷结附能力".into());
    }
    Ok(())
}

pub(crate) fn validate_definition(card_id: &str, d: &Definition) -> RuleResult<()> {
    if let Some(expected) = definition(card_id) {
        if !same(d, &expected) {
            return Err(format!(
                "{card_id}仅准入完整印刷定义，不能修改特征、结附或程序"
            ));
        }
    }
    Ok(())
}

fn relevant_frame(f: &ResolutionFrame) -> bool {
    f.source.card.definition == "BQ028"
        || f.ability_key == "reveal-inspect-attachment"
        || f.steps
            .iter()
            .any(|s| contains_bq028_op(std::slice::from_ref(&s.op)))
}

impl Game {
    fn bq028_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        if actor >= self.players.len()
            || s.card.controller != actor
            || s.card.owner >= self.players.len()
            || s.card.definition != "BQ028"
            || s.card.id.is_empty()
            || s.card.time_markers != 0
            || s.card.face_down
            || s.region.is_none_or(|r| r >= self.regions.len())
            || s.source_region_instance
                .as_ref()
                .is_none_or(|id| id.is_empty() || id == &s.card.id)
            || s.attachment_host_instance.is_some()
            || s.observed_death.is_some()
            || s.play_source.is_some()
        {
            return Err("水沟鼠群密探现身来源、冻结行动者或地区无效".into());
        }
        Ok(())
    }

    fn bq028_frame_target(&self, f: &ResolutionFrame) -> RuleResult<usize> {
        let a = &definition("BQ028").unwrap().abilities[0];
        let t = f.targets.first().ok_or("密探缺少目标玩家")?;
        let seat =
            t.id.strip_prefix('p')
                .and_then(|s| s.parse::<usize>().ok())
                .filter(|s| *s < self.players.len() && t.id == format!("p{s}"))
                .ok_or("密探目标玩家标识无效")?;
        if f.targets.len() != 1
            || !same(&t.spec, &a.targets[0])
            || t.region_instance.is_some()
            || t.public.instance_id != t.id
            || t.public.kind != "player"
            || t.public.region.is_some()
            || t.public.owner.is_some()
            || t.public.controller.as_deref() != Some(t.id.as_str())
            || t.public.label != self.players[seat].name
            || !t.public.valid
            || t.public.status != "valid"
            || t.public.invalid_reason.is_some()
        {
            return Err("密探目标玩家绑定被修改".into());
        }
        Ok(seat)
    }

    pub(crate) fn validate_blue_expansion_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        self.validate_bq030_frame(f)?;
        if !relevant_frame(f) {
            return Ok(());
        }
        if f.source.card.definition != "BQ028" {
            return Err("密探闭合程序不能移植到其他来源".into());
        }
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            let s = &f.source;
            if f.frame_id.is_empty()
                || f.actor >= self.players.len()
                || s.card.controller != f.actor
                || s.card.owner >= self.players.len()
                || s.card.id.is_empty()
                || s.card.time_markers != 0
                || s.region.is_none_or(|r| r >= self.regions.len())
                || f.chosen_region != s.region
                || s.source_region_instance
                    .as_ref()
                    .is_none_or(|id| id.is_empty() || id == &s.card.id)
                || s.attachment_host_instance.is_some()
                || s.observed_death.is_some()
                || !f.steps.is_empty()
                || !f.targets.is_empty()
                || f.cursor != 0
                || !matches!(f.guard, GuardState::Unchecked)
                || (if f.ability_key == "deploy" {
                    s.card.face_down || s.play_source != Some(PlaySource::Hand)
                } else {
                    !s.card.face_down || s.play_source.is_some()
                })
            {
                return Err("密探派遣或现身空程序与来源不符".into());
            }
            return Ok(());
        }
        self.bq028_source(f.actor, &f.source)?;
        if matches!(f.ability_key.as_str(), "renown" | "jc089-combat-glory") {
            let [Step {
                context,
                op:
                    Op::PlaceInfluence {
                        region_instance,
                        amount: 1,
                    },
            }] = f.steps.as_slice()
            else {
                return Err("密探获授奖励必须保持完整原子程序".into());
            };
            let frozen = f.source.source_region_instance.as_deref().unwrap();
            let a = if f.ability_key == "renown" {
                crate::renown::renown_ability(frozen)
            } else {
                rules::jc089_combat_glory_ability(frozen)
            };
            if region_instance.is_empty()
                || region_instance != frozen
                || *context != f.actor
                || f.frame_id.is_empty()
                || !f.targets.is_empty()
                || !f.already_paid.is_empty()
                || f.chosen_region.is_some()
                || f.cursor != 0
                || !matches!(f.guard, GuardState::Unchecked)
                || !crate::jz22::runtime_ability_is_admitted(&a)
            {
                return Err("密探获授奖励绑定无效".into());
            }
            return Ok(());
        }
        let a = &definition("BQ028").unwrap().abilities[0];
        self.bq028_frame_target(f)?;
        if f.frame_id.is_empty()
            || f.ability_key != a.key
            || f.steps.len() != 1
            || f.steps[0].context != f.actor
            || !same(&f.steps[0].op, &a.ops[0])
            || !f.already_paid.is_empty()
            || f.chosen_region.is_some()
            || !((f.cursor == 0 && matches!(f.guard, GuardState::Unchecked))
                || (f.cursor == 1 && matches!(f.guard, GuardState::Accepted)))
        {
            return Err("密探完整程序、费用、守卫或游标被修改".into());
        }
        Ok(())
    }

    pub(crate) fn bq028_inspect_start(&mut self, frame: ResolutionFrame) -> RuleResult<()> {
        self.validate_blue_expansion_frame(&frame)?;
        if frame.cursor != 1 || !matches!(frame.guard, GuardState::Accepted) {
            return Err("密探检视必须从已通过守卫的完整程序开始".into());
        }
        let seat = self.bq028_frame_target(&frame)?;
        if self.players[seat].eliminated {
            return Ok(());
        }
        let inspected = self.players[seat].hand.clone();
        self.bq028_validate_inspected_hand(&inspected)?;
        let options: Vec<_> = inspected
            .iter()
            .filter(|c| catalog::card(&c.definition).kind == "attachment")
            .map(|c| self.option(c, frame.actor, None, None))
            .collect();
        let max = usize::from(!options.is_empty());
        self.choice(
            frame.actor,
            "bq028-hand-inspect",
            "检视手牌：可选一张附属令该玩家弃牌".into(),
            options,
            0,
            max,
            None,
            ChoiceResolution::Frame {
                frame: Box::new(frame),
                choice: FrameChoice::BQ028InspectAttachments { seat, inspected },
            },
        );
        // Full inspection is derived only in the actor's private Choice view.
        // Public logs and revealed_hands do not reveal the target's other cards.
        self.pending.as_mut().unwrap().choice.description =
            "检视全部手牌。可以选择一张附属，或不弃牌并确认。".into();
        Ok(())
    }

    pub(crate) fn bq028_inspect_complete(
        &mut self,
        frame: &ResolutionFrame,
        seat: usize,
        inspected: Vec<Card>,
        selected: &[String],
    ) -> RuleResult<()> {
        self.validate_blue_expansion_frame(frame)?;
        self.bq028_validate_inspected_hand(&inspected)?;
        if frame.cursor != 1
            || !matches!(frame.guard, GuardState::Accepted)
            || self.bq028_frame_target(frame)? != seat
            || self.players[seat].eliminated
            || !same(&inspected, &self.players[seat].hand)
            || selected.len() > 1
        {
            return Err("密探检视续体或原手牌实例已失效".into());
        }
        if let Some(id) = selected.first() {
            if !inspected
                .iter()
                .any(|c| c.id == *id && catalog::card(&c.definition).kind == "attachment")
            {
                return Err("密探只能选择所检视原手牌中的附属".into());
            }
            self.discard_hand_card(seat, id)?;
        }
        Ok(())
    }

    pub(crate) fn blue_expansion_private_choice(&self, p: &Pending, viewer: usize) -> Choice {
        let mut choice = p.choice.clone();
        if let ChoiceResolution::Frame {
            frame,
            choice: FrameChoice::BQ028InspectAttachments { inspected, .. },
        } = &p.resolution
        {
            if viewer == frame.actor {
                choice.preview_cards = inspected
                    .iter()
                    .map(|c| self.card_view(c, viewer, None, None))
                    .collect();
            }
        }
        choice
    }

    fn validate_bq028_declaration(&self, d: &Declaration) -> RuleResult<()> {
        let relevant = d.source.card.definition == "BQ028"
            || d.ability.key == "reveal-inspect-attachment"
            || contains_bq028_op(&d.ability.ops)
            || d.ability.modes.iter().any(|m| contains_bq028_op(&m.ops));
        if relevant {
            self.bq028_source(d.actor, &d.source)?;
            validate_ability(&d.source.card.definition, &d.ability)?;
            if let [Op::PlaceInfluence {
                region_instance, ..
            }] = d.ability.ops.as_slice()
            {
                if d.source.source_region_instance.as_ref() != Some(region_instance) {
                    return Err("密探获授奖励只能绑定冻结原地区".into());
                }
            }
        }
        Ok(())
    }

    fn bq028_validate_inspected_hand(&self, inspected: &[Card]) -> RuleResult<()> {
        let mut ids = std::collections::BTreeSet::new();
        for c in inspected {
            if c.id.is_empty()
                || !ids.insert(c.id.as_str())
                || c.owner >= self.players.len()
                || c.controller >= self.players.len()
                || c.face_down
                || !catalog::catalog().cards.iter().any(|d| {
                    d.id == c.definition
                        && d.supported
                        && matches!(d.kind.as_str(), "character" | "spell" | "attachment")
                })
            {
                return Err("密探检视手牌包含未知牌面或非法实例".into());
            }
        }
        Ok(())
    }

    pub(crate) fn validate_blue_expansion_state(&self) -> RuleResult<()> {
        self.validate_bq030_state()?;
        for item in &self.stack {
            if let Some(f) = &item.frame {
                self.validate_blue_expansion_frame(f)?;
                if relevant_frame(f) {
                    let play = matches!(f.ability_key.as_str(), "deploy" | "reveal");
                    if item.id != f.frame_id
                        || item.controller != f.actor
                        || f.cursor != 0
                        || !matches!(f.guard, GuardState::Unchecked)
                        || item.target != f.targets.first().map(|t| t.id.clone())
                        || (if play {
                            item.card.as_ref().is_none_or(|c| {
                                c.definition != "BQ028"
                                    || c.owner != f.source.card.owner
                                    || c.controller != f.actor
                                    || c.face_down
                                    || c.id.is_empty()
                                    || c.id == f.source.card.id
                            }) || item.deploy_region != f.source.region
                                || item.reveal != (f.ability_key == "reveal")
                        } else {
                            item.card.is_some() || item.deploy_region.is_some() || item.reveal
                        })
                    {
                        return Err("密探堆栈容器与冻结程序不符".into());
                    }
                }
            }
        }
        for e in &self.effects {
            match e {
                Effect::Frame { frame: f } => {
                    self.validate_blue_expansion_frame(f)?;
                    if relevant_frame(f)
                        && (f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked))
                    {
                        return Err("密探已执行续体只能保存在私密检视待选".into());
                    }
                }
                Effect::Declare { declaration: d } => self.validate_bq028_declaration(d)?,
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Declare {
                    declaration: d,
                    stage,
                } => {
                    self.validate_bq028_declaration(d)?;
                    if d.source.card.definition == "BQ028" {
                        let printed = d.ability.key == "reveal-inspect-attachment";
                        let expected = if printed {
                            self.binding_options(d.actor, &d.source, &d.ability.targets[0])
                        } else {
                            vec![ChoiceOption {
                                id: "accept".into(),
                                label: "发动触发能力".into(),
                                card: None,
                            }]
                        };
                        let expected_title =
                            format!("{}：{}", catalog::card("BQ028").name, d.ability.label);
                        if !(if printed {
                            matches!(stage, DeclareChoice::Target)
                        } else {
                            matches!(stage, DeclareChoice::Accept)
                        }) || p.seat != d.actor
                            || self.players[d.actor].eliminated
                            || p.choice.id.is_empty()
                            || p.choice.player_id != format!("p{}", d.actor)
                            || p.choice.kind != "trigger"
                            || p.choice.title != expected_title
                            || p.choice.description != "选择符合数量限制的选项，然后确认。"
                            || p.choice.min != Some(0)
                            || p.choice.max != Some(1)
                            || p.choice.allow_decline != Some(true)
                            || p.choice.amount.is_some()
                            || !p.choice.preview_cards.is_empty()
                            || !same(&p.choice.options, &expected)
                        {
                            return Err("密探现身声明选项或选择者无效".into());
                        }
                    }
                }
                ChoiceResolution::Frame { frame: f, choice } => {
                    if relevant_frame(f)
                        || matches!(choice, FrameChoice::BQ028InspectAttachments { .. })
                    {
                        self.validate_blue_expansion_frame(f)?;
                        let FrameChoice::BQ028InspectAttachments { seat, inspected } = choice
                        else {
                            return Err("密探不能使用其他暂停选择续体".into());
                        };
                        self.bq028_validate_inspected_hand(inspected)?;
                        if f.source.card.definition != "BQ028"
                            || self.bq028_frame_target(f)? != *seat
                            || self.players[*seat].eliminated
                            || !same(inspected, &self.players[*seat].hand)
                            || f.cursor != 1
                            || !matches!(f.guard, GuardState::Accepted)
                            || p.seat != f.actor
                            || self.players[f.actor].eliminated
                            || p.choice.id.is_empty()
                            || p.choice.player_id != format!("p{}", f.actor)
                            || p.choice.kind != "bq028-hand-inspect"
                            || p.choice.title != "检视手牌：可选一张附属令该玩家弃牌"
                            || p.choice.description
                                != "检视全部手牌。可以选择一张附属，或不弃牌并确认。"
                            || p.choice.min != Some(0)
                            || p.choice.allow_decline != Some(true)
                            || p.choice.amount.is_some()
                            || !p.choice.preview_cards.is_empty()
                        {
                            return Err("密探检视冻结手牌、程序或选择者无效".into());
                        }
                        let options: Vec<_> = inspected
                            .iter()
                            .filter(|c| catalog::card(&c.definition).kind == "attachment")
                            .map(|c| self.option(c, f.actor, None, None))
                            .collect();
                        if !same(&p.choice.options, &options)
                            || p.choice.max != Some(usize::from(!options.is_empty()))
                        {
                            return Err("密探附属选项与所检视手牌不符".into());
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
