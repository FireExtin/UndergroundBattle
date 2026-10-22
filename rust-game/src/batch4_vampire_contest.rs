//! B's finite adapters around the shared contest protocol. No selection,
//! response, reward, nesting, or terminal-state machine lives in this module.
use crate::{
    batch4_vampires::{self as v, Bq038CharacterWitness},
    catalog,
    engine::RuleResult,
    model::*,
    mystic_contest::{ContestOutcome, MysticBonus, MysticPhase, MysticShown},
    rules::*,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub(crate) const SHOWN: &str = "batch4-bq027-shown-feed";
pub(crate) const WIN: &str = "batch4-bq031-win-feed";
pub(crate) const HAND: &str = "batch4-bq032-hand-contest";
pub(crate) const SACRIFICE: &str = "batch4-bq036-contest";
pub(crate) const START: &str = "batch4-bq038-contest-bonus";
pub(crate) const TARGET: &str = "batch4-bq027-shown-target";
pub(crate) const TARGET_TITLE: &str = "血畜：目标本方吸血鬼饮血1";
fn same<T: Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttachmentPayment {
    pub turn: u32,
    pub attachment: Attachment,
    pub host: SourceSnapshot,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StartObservation {
    pub contest_id: String,
    pub turn: u32,
    /// None before resolution; Some(empty) is a genuine resolved +0.
    pub witnesses: Option<Vec<Bq038CharacterWitness>>,
    pub resolved_id: Option<String>,
}
pub(crate) fn key(key: &str) -> bool {
    matches!(key, SHOWN | WIN | HAND | SACRIFICE | START)
}
pub(crate) fn admitted_start(id: &str, key: &str, bonus: u32) -> bool {
    matches!(
        (id, key, bonus),
        ("BQ032", HAND, 0) | ("BQ036", SACRIFICE, 2)
    )
}
pub(crate) fn owns(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::BQ027ShownFeed { .. } | Op::BQ031WinFeed { .. } | Op::BQ038ContestBonus { .. } => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => owns(body),
        Op::IfTargetExhausted {
            exhausted, ready, ..
        } => owns(std::slice::from_ref(exhausted)) || owns(std::slice::from_ref(ready)),
        _ => false,
    })
}
pub(crate) fn related(f: &ResolutionFrame) -> bool {
    key(&f.ability_key)
        || f.steps.iter().any(|s| owns(std::slice::from_ref(&s.op)))
        || f.already_paid
            .iter()
            .any(|p| matches!(p, PaidCost::BQ036Attachment(_)))
}
pub(crate) fn ability_for(id: &str) -> Option<AbilitySpec> {
    let mut a = match id {
        "BQ031" => ability(
            WIN,
            "赢得隐秘对抗：饮血1",
            Timing::Fast,
            vec![],
            vec![],
            vec![Op::BQ031WinFeed { contest_id: None }],
            Some(Event::MysticContestWon),
        ),
        "BQ032" => ability(
            HAND,
            "手牌行动：支付2并弃自身，进行隐秘对抗",
            Timing::Standard,
            vec![Cost::Assets(2), Cost::DiscardSelectedHandCard],
            vec![],
            vec![Op::MysticContest { actor_bonus: 0 }],
            None,
        ),
        "BQ036" => ability(
            SACRIFICE,
            "行动：支付1并牺牲本方角色上的本方附属，隐秘对抗本人+2",
            Timing::Standard,
            vec![Cost::Assets(1), Cost::BQ036SacrificeAttachment],
            vec![],
            vec![Op::MysticContest { actor_bonus: 2 }],
            None,
        ),
        "BQ038" => ability(
            START,
            "进行隐秘对抗：按本方被装备、武器或状态结附的角色数加值",
            Timing::Fast,
            vec![],
            vec![],
            vec![Op::BQ038ContestBonus { observation: None }],
            Some(Event::BQ038MysticStart),
        ),
        _ => return None,
    };
    if matches!(id, "BQ032" | "BQ036") {
        a.activation_only = true;
    }
    Some(a)
}
pub(crate) fn canonical(id: &str, a: &AbilitySpec) -> Option<AbilitySpec> {
    let mut expected = ability_for(id).filter(|e| e.key == a.key)?;
    match a.ops.as_slice() {
        [Op::BQ031WinFeed { contest_id }] if a.key == WIN => {
            expected.ops = vec![Op::BQ031WinFeed {
                contest_id: contest_id.clone(),
            }]
        }
        [Op::BQ038ContestBonus { observation }] if a.key == START => {
            expected.ops = vec![Op::BQ038ContestBonus {
                observation: observation.clone(),
            }]
        }
        _ => (),
    }
    Some(expected)
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if canonical(id, a).is_none_or(|e| !same(&e, a)) {
        return Err("B隐秘条款只能使用完整固定程序".into());
    }
    Ok(())
}
pub(crate) fn shown_target() -> TargetSlotSpec {
    let mut t = target(
        Zone::Board,
        EntityKind::Character,
        Relation::ControlledByActor,
        Range::Anywhere,
    );
    t.subtype = Some("吸血鬼".into());
    t
}

impl Game {
    pub(crate) fn bq032_begin_hand(&mut self, actor: usize, a: &Action) -> RuleResult<()> {
        v::validate_action("BQ032", a)?;
        let source = self
            .bq032_hand_contest_source(actor, a.card_id.as_deref().ok_or("请选择宴会司仪手牌")?)?;
        let spec = ability_for("BQ032").unwrap();
        self.check_timing(actor, &spec)?;
        // The caller cannot substitute another discard or charge printed cost3.
        let payment = Action {
            cost_selected: Some(vec![source.card.id.clone()]),
            ..a.clone()
        };
        let paid = self.pay_ability_costs(actor, &source, &spec, &payment)?;
        let frame = self.make_frame(actor, source, &spec, vec![], paid, None);
        self.dispatch_frame(frame, spec.label, None, spec.response_policy);
        Ok(())
    }
    pub(crate) fn bq036_pay_attachment(
        &mut self,
        actor: usize,
        a: &Action,
    ) -> RuleResult<PaidCost> {
        let ids = a
            .cost_selected
            .as_ref()
            .filter(|ids| ids.len() == 1)
            .ok_or("菲尼克斯须牺牲一个合格附属")?;
        if !self.bq036_contest_cost_candidates(actor).contains(&ids[0]) {
            return Err("费用须是本方角色上的本方附属".into());
        }
        let attachment = self
            .attachments
            .iter()
            .find(|a| a.card.id == ids[0])
            .unwrap()
            .clone();
        let (r, host) = self.board(&attachment.host_id).unwrap();
        let mut host = self.source_snapshot(host, Some(r));
        host.source_region_instance = Some(self.regions[r].card.id.clone());
        let payment = AttachmentPayment {
            turn: self.turn,
            attachment: attachment.clone(),
            host,
        };
        self.remove_dead(&attachment.card.id, RemovalCause::Sacrifice);
        Ok(PaidCost::BQ036Attachment(Box::new(payment)))
    }
    fn b_contest_source_clean(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        if actor >= self.players.len()
            || s.card.controller != actor
            || s.card.owner >= self.players.len()
            || s.card.face_down
            || !self.mystic_instance_valid(&s.card.id)
            || s.play_source.is_some()
            || s.death_echo.is_some()
            || s.observed_death.is_some()
            || s.observed_exposure.is_some()
            || s.observed_mobility_grant.is_some()
        {
            return Err("B隐秘条款来源身份无效".into());
        }
        self.mobility_immutable_identity(&s.card)
    }
    fn b_contest_host_witness(
        &self,
        actor: usize,
        s: &SourceSnapshot,
        attachment: &Attachment,
        before: &str,
    ) -> RuleResult<()> {
        self.b_contest_source_clean(actor, s)?;
        let Some(d) = catalog::catalog()
            .cards
            .iter()
            .find(|d| d.id == s.card.definition && catalog::runtime_implemented(d))
        else {
            return Err("附属宿主未准入".into());
        };
        let Some(a) = catalog::catalog()
            .cards
            .iter()
            .find(|d| d.id == attachment.card.definition && catalog::runtime_implemented(d))
        else {
            return Err("见证附属未准入".into());
        };
        if !catalog::has_kind(d, "character")
            || !catalog::has_kind(a, "attachment")
            || attachment.card.face_down
            || attachment.card.owner >= self.players.len()
            || attachment.card.controller >= self.players.len()
            || attachment.host_id != s.card.id
            || s.region.is_none_or(|r| r >= self.regions.len())
            || s.attachment_host_instance.is_some()
            || s.source_region_instance
                .as_ref()
                .is_none_or(|id| !self.mystic_instance_valid(id))
            || [&s.card.id, &attachment.card.id].iter().any(|id| {
                !self
                    .mystic_instance_number(id)
                    .zip(self.mystic_instance_number(before))
                    .is_some_and(|(a, b)| a < b)
            })
        {
            return Err("附属见证须绑定原个人角色、原地区及原附属".into());
        }
        self.mobility_immutable_identity(&attachment.card)
    }
    fn b_contest_assets(&self, f: &ResolutionFrame, ids: &[String], count: usize) -> bool {
        ids.len() == count
            && ids.iter().collect::<BTreeSet<_>>().len() == count
            && ids.iter().all(|id| {
                *id != f.source.card.id
                    && self
                        .mystic_instance_number(id)
                        .zip(self.mystic_instance_number(&f.frame_id))
                        .is_some_and(|(a, b)| a < b)
            })
    }
    fn b_contest_win(&self, actor: usize, id: &str) -> RuleResult<()> {
        let outcome = self
            .mystic_history
            .iter()
            .find(|o| o.contest_id == id)
            .ok_or("雾羽千军获胜观察缺少实际结果")?;
        if outcome.turn != self.turn
            || !outcome.participants.contains(&actor)
            || outcome.winner_team != Some(self.team(actor))
        {
            return Err("雾羽千军须观察原个人玩家的获胜".into());
        }
        self.validate_mystic_outcome(outcome)
    }
    pub(crate) fn validate_b_contest_declaration(&self, d: &Declaration) -> RuleResult<()> {
        self.vampire_source(d.actor, &d.source)?;
        validate_ability(&d.source.card.definition, &d.ability)?;
        match d.ability.ops.as_slice() {
            [Op::BQ031WinFeed {
                contest_id: Some(id),
            }] => self.b_contest_win(d.actor, id),
            [Op::BQ038ContestBonus {
                observation: Some(o),
            }] if o.witnesses.is_none() && o.resolved_id.is_none() => {
                self.b_contest_start_receipt(d.actor, &d.source, o)
            }
            _ => Err("B隐秘触发缺少原事件或已被改为行动".into()),
        }
    }
    fn b_contest_start_receipt(
        &self,
        actor: usize,
        source: &SourceSnapshot,
        o: &StartObservation,
    ) -> RuleResult<()> {
        // Suspended outer receipts remain valid while an inner contest runs.
        // This is read-only authentication, not permission to award that layer.
        let state = self
            .mystic_states()
            .find(|s| s.contest_id == o.contest_id)
            .ok_or("掠食时刻缺少原对抗层")?;
        if state.turn != o.turn
            || o.turn != self.turn
            || !matches!(state.phase, MysticPhase::BeforeSelection)
            || !state.participants.contains(&actor)
            || !self
                .mystic_instance_number(&source.card.id)
                .zip(self.mystic_instance_number(&o.contest_id))
                .is_some_and(|(a, b)| a < b)
        {
            return Err("掠食时刻只能观察原对抗开始阶段".into());
        }
        Ok(())
    }
    pub(crate) fn validate_b_contest_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        let c = &f.source.card;
        self.b_contest_source_clean(f.actor, &f.source)?;
        if !v::CARDS.contains(&c.definition.as_str())
            || !self
                .mystic_instance_number(&c.id)
                .zip(self.mystic_instance_number(&f.frame_id))
                .is_some_and(|(a, b)| a < b)
            || f.chosen_region.is_some()
            || f.steps.len() != 1
            || f.steps[0].context != f.actor
            || f.cursor > 1
            || f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked)
            || f.cursor == 1 && !matches!(f.guard, GuardState::Accepted)
        {
            return Err("B隐秘帧的来源、程序或阶段无效".into());
        }
        if f.ability_key == SHOWN {
            return self.bq027_validate_post(f);
        }
        let mut a = ability_for(&c.definition)
            .filter(|a| a.key == f.ability_key)
            .ok_or("B隐秘帧不是原卡完整程序")?;
        a.ops = f.steps.iter().map(|s| s.op.clone()).collect();
        validate_ability(&c.definition, &a)?;
        if !f.targets.is_empty() {
            return Err("B对抗条款不能夹带目标".into());
        }
        if f.ability_key == HAND {
            if f.source.region.is_some()
                || f.source.source_region_instance.is_some()
                || f.source.attachment_host_instance.is_some()
                || self.batch3_physical_cards().any(|now| now.id == c.id)
                || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids), PaidCost::Discarded { old_instance, holder, owner }]
                    if self.b_contest_assets(f, ids, 2) && old_instance == &c.id && *holder == f.actor && *owner == c.owner)
            {
                return Err("宴会司仪须保留已付2及弃原手牌自身的收据".into());
            }
        } else {
            self.vampire_source(f.actor, &f.source)?;
            if f.ability_key == SACRIFICE {
                let [PaidCost::Assets(ids), PaidCost::BQ036Attachment(p)] =
                    f.already_paid.as_slice()
                else {
                    return Err("菲尼克斯缺少付1及附属牺牲收据".into());
                };
                self.b_contest_host_witness(f.actor, &p.host, &p.attachment, &f.frame_id)?;
                if !self.b_contest_assets(f, ids, 1)
                    || p.turn != self.turn
                    || p.attachment.card.controller != f.actor
                    || ids.contains(&p.attachment.card.id)
                    || self
                        .batch3_physical_cards()
                        .any(|c| c.id == p.attachment.card.id)
                {
                    return Err("菲尼克斯不能重复支付或改换牺牲附属".into());
                }
            } else {
                if !f.already_paid.is_empty() {
                    return Err("B隐秘触发没有额外费用".into());
                }
                match &f.steps[0].op {
                    Op::BQ031WinFeed {
                        contest_id: Some(id),
                    } => self.b_contest_win(f.actor, id)?,
                    Op::BQ038ContestBonus {
                        observation: Some(o),
                    } => {
                        if o.turn != self.turn
                            || !self
                                .mystic_instance_number(&o.contest_id)
                                .zip(self.mystic_instance_number(&f.frame_id))
                                .is_some_and(|(a, b)| a < b)
                            || !self
                                .mystic_instance_number(&c.id)
                                .zip(self.mystic_instance_number(&o.contest_id))
                                .is_some_and(|(a, b)| a < b)
                        {
                            return Err("掠食时刻事件或帧身份无效".into());
                        }
                        if let Some(ws) = &o.witnesses {
                            let capture = o
                                .resolved_id
                                .as_ref()
                                .filter(|id| {
                                    self.mystic_instance_number(&f.frame_id)
                                        .zip(self.mystic_instance_number(id))
                                        .is_some_and(|(a, b)| a < b)
                                })
                                .ok_or("掠食时刻缺少实际结算身份")?;
                            if f.cursor != 1 {
                                return Err("未结算掠食时刻不能预填加值".into());
                            }
                            let mut hosts = BTreeSet::new();
                            let mut attached = BTreeSet::new();
                            for w in ws {
                                self.b_contest_host_witness(
                                    f.actor,
                                    &w.character,
                                    &w.attachment,
                                    capture,
                                )?;
                                if !hosts.insert(&w.character.card.id)
                                    || !attached.insert(&w.attachment.card.id)
                                    || !catalog::card(&w.attachment.card.definition)
                                        .subtypes
                                        .iter()
                                        .any(|s| matches!(s.as_str(), "装备" | "武器" | "状态"))
                                {
                                    return Err("掠食时刻只按合格原宿主去重计数".into());
                                }
                            }
                        } else {
                            if o.resolved_id.is_some() {
                                return Err("未结算加值不能预填结算身份".into());
                            }
                            self.b_contest_start_receipt(f.actor, &f.source, o)?;
                        }
                    }
                    _ => return Err("B隐秘触发没有有限事件收据".into()),
                }
            }
        }
        Ok(())
    }
    pub(crate) fn b_mystic_start(&mut self, contest_id: &str) -> RuleResult<()> {
        let seats = self
            .mystic_contest
            .as_ref()
            .filter(|s| s.contest_id == contest_id)
            .ok_or("开始观察缺少原对抗")?
            .participants
            .clone();
        let sources = self
            .attachments
            .iter()
            .filter(|a| {
                a.card.definition == "BQ038"
                    && !a.card.face_down
                    && seats.contains(&a.card.controller)
                    && a.host_id == format!("society:p{}", a.card.controller)
                    && self.attachment_host_valid(a)
            })
            .map(|a| self.source_snapshot(&a.card, None))
            .collect::<Vec<_>>();
        let previous = self.effects.len();
        for source in sources {
            let mut ability = ability_for("BQ038").unwrap();
            ability.ops = vec![Op::BQ038ContestBonus {
                observation: Some(Box::new(StartObservation {
                    contest_id: contest_id.into(),
                    turn: self.turn,
                    witnesses: None,
                    resolved_id: None,
                })),
            }];
            self.effects.push_back(Effect::Declare {
                declaration: Declaration {
                    actor: source.card.controller,
                    source,
                    ability,
                },
            });
        }
        self.batch3_register_entry_order(contest_id, previous);
        Ok(())
    }
    pub(crate) fn bq038_resolve_bonus(&mut self, mut f: ResolutionFrame) -> RuleResult<()> {
        self.validate_b_contest_frame(&f)?;
        let Op::BQ038ContestBonus {
            observation: Some(o),
        } = &mut f.steps[0].op
        else {
            return Err("加值缺少开始收据".into());
        };
        if o.witnesses.is_some() {
            return Err("加值收据不可再次结算".into());
        }
        // Finding a suspended layer for validation must never permit paying
        // its bonus into the child or mutating its frozen receipt out of order.
        if self.mystic_contest.as_ref().is_none_or(|s| {
            s.contest_id != o.contest_id || !matches!(s.phase, MysticPhase::BeforeSelection)
        }) {
            return Err("掠食时刻只能结算当前活动的原对抗层".into());
        }
        let witnesses = self.bq038_contest_count_witnesses(f.actor);
        let amount = u32::try_from(witnesses.len()).map_err(|_| "掠食时刻计数溢出")?;
        o.witnesses = Some(witnesses);
        o.resolved_id = Some(self.id());
        self.mystic_add_bonus(f, amount)
    }
    pub(crate) fn b_mystic_validate_bonus(&self, b: &MysticBonus) -> RuleResult<()> {
        self.validate_b_contest_frame(&b.frame)?;
        let [Step {
            op: Op::BQ038ContestBonus {
                observation: Some(o),
            },
            ..
        }] = b.frame.steps.as_slice()
        else {
            return Err("未知B加值程序".into());
        };
        if b.frame.ability_key != START
            || b.frame.cursor != 1
            || o.contest_id != b.contest_id
            || o.witnesses
                .as_ref()
                .is_none_or(|ws| ws.len() != b.amount as usize)
        {
            return Err("掠食时刻加值不等于原结算见证数".into());
        }
        Ok(())
    }
    pub(crate) fn b_mystic_win(&mut self, o: &ContestOutcome) -> RuleResult<()> {
        let Some(team) = o.winner_team else {
            return Ok(());
        };
        let sources = self
            .in_play_cards()
            .into_iter()
            .filter(|(_, c)| {
                c.definition == "BQ031"
                    && !c.face_down
                    && o.participants.contains(&c.controller)
                    && self.team(c.controller) == team
            })
            .map(|(r, c)| self.source_snapshot(c, r))
            .collect::<Vec<_>>();
        for source in sources {
            let mut ability = ability_for("BQ031").unwrap();
            ability.ops = vec![Op::BQ031WinFeed {
                contest_id: Some(o.contest_id.clone()),
            }];
            self.effects.push_back(Effect::Declare {
                declaration: Declaration {
                    actor: source.card.controller,
                    source,
                    ability,
                },
            });
        }
        // The shared protocol registers A/B victory observers together once.
        Ok(())
    }
    pub(crate) fn b_mystic_extra(&self, shown: &MysticShown) -> u32 {
        if shown.card.definition == "BQ027" {
            2
        } else {
            0
        }
    }
    pub(crate) fn b_mystic_shown(&mut self, o: &ContestOutcome, index: usize) -> RuleResult<()> {
        let shown = o.shown.get(index).ok_or("展示后置游标无效")?;
        if shown.card.definition != "BQ027" {
            return Ok(());
        }
        let f = ResolutionFrame {
            frame_id: self.id(),
            ability_key: SHOWN.into(),
            actor: shown.seat,
            source: self.source_snapshot(&shown.card, None),
            targets: vec![],
            already_paid: vec![],
            guard: GuardState::Accepted,
            cursor: 1,
            steps: vec![Step {
                context: shown.seat,
                op: Op::BQ027ShownFeed {
                    contest_id: o.contest_id.clone(),
                    index,
                },
            }],
            chosen_region: None,
        };
        let Some(state) = &mut self.mystic_contest else {
            return Err("血畜后置缺少原对抗".into());
        };
        let MysticPhase::After { post_frame, .. } = &mut state.phase else {
            return Err("血畜不是展示后置阶段".into());
        };
        *post_frame = Some(Box::new(f.clone()));
        self.bq027_validate_post(&f)?;
        let options = self.binding_options(f.actor, &f.source, &shown_target());
        if options.is_empty() || self.players[f.actor].eliminated {
            return Ok(());
        }
        self.choice(
            f.actor,
            TARGET,
            TARGET_TITLE.into(),
            options,
            1,
            1,
            None,
            ChoiceResolution::Frame {
                frame: Box::new(f),
                choice: FrameChoice::BQ027ShownTarget,
            },
        );
        Ok(())
    }
    fn bq027_validate_post(&self, f: &ResolutionFrame) -> RuleResult<()> {
        let Some(state) = &self.mystic_contest else {
            return Err("血畜后置缺少原对抗".into());
        };
        let MysticPhase::After {
            outcome,
            next_shown,
            post_frame,
        } = &state.phase
        else {
            return Err("血畜后置不是原展示阶段".into());
        };
        let [Step {
            op: Op::BQ027ShownFeed { contest_id, index },
            ..
        }] = f.steps.as_slice()
        else {
            return Err("血畜后置缺少原展示收据".into());
        };
        let shown = outcome.shown.get(*index).ok_or("血畜展示索引无效")?;
        if shown.card.definition != "BQ027"
            || f.ability_key != SHOWN
            || *contest_id != outcome.contest_id
            || *next_shown != index + 1
            || f.actor != shown.seat
            || !same(&f.source, &self.source_snapshot(&shown.card, None))
            || !f.already_paid.is_empty()
            || f.cursor != 1
            || !matches!(f.guard, GuardState::Accepted)
            || f.targets.len() > 1
            || post_frame.as_ref().is_none_or(|p| !same(p.as_ref(), f))
        {
            return Err("血畜后置必须绑定同次本人展示与唯一后置帧".into());
        }
        if let Some(t) = f.targets.first() {
            if !same(&t.spec, &shown_target())
                || !self.mystic_instance_valid(&t.id)
                || t.id != t.public.instance_id
                || t.region_instance.is_some()
                || t.public.kind != "character"
                || t.public.controller.as_deref() != Some(&player_id(f.actor))
                || t.public.region.is_none_or(|r| r >= self.regions.len())
                || !t.public.valid
                || t.public.status != "valid"
                || t.public.invalid_reason.is_some()
                || !self.valid_binding(f.actor, &f.source, &t.spec, &t.id)
                || !same(
                    &t.public,
                    &self.public_target(f.actor, &f.source, &t.spec, &t.id),
                )
            {
                return Err("血畜展示后目标必须保留原个人吸血鬼绑定".into());
            }
        }
        Ok(())
    }
    pub(crate) fn bq027_choose_shown_target(
        &mut self,
        mut f: ResolutionFrame,
        chooser: usize,
        selected: &[String],
    ) -> RuleResult<()> {
        self.validate_b_contest_frame(&f)?;
        if chooser != f.actor || !f.targets.is_empty() || selected.len() != 1 {
            return Err("血畜展示后须选原个人目标一次".into());
        }
        let t = shown_target();
        if !self.valid_binding(f.actor, &f.source, &t, &selected[0]) {
            return Err("血畜目标不是本方吸血鬼".into());
        }
        f.targets.push(BoundTarget {
            id: selected[0].clone(),
            public: self.public_target(f.actor, &f.source, &t, &selected[0]),
            spec: t,
            region_instance: None,
        });
        let MysticPhase::After { post_frame, .. } =
            &mut self.mystic_contest.as_mut().unwrap().phase
        else {
            unreachable!()
        };
        *post_frame = Some(Box::new(f.clone()));
        self.vampire_feed_start(f)
    }
    pub(crate) fn validate_b_contest_post_state(&self) -> RuleResult<()> {
        for outcome in &self.mystic_history {
            let mut sources = BTreeSet::new();
            for bonus in &outcome.bonuses {
                if !sources.insert(&bonus.frame.source.card.id) {
                    return Err("同一掠食时刻不能重复为同次对抗加值".into());
                }
            }
        }
        let Some(state) = &self.mystic_contest else {
            return Ok(());
        };
        let MysticPhase::After {
            post_frame: Some(f),
            ..
        } = &state.phase
        else {
            return Ok(());
        };
        if f.ability_key != SHOWN {
            return Ok(());
        }
        self.validate_b_contest_frame(f)?;
        if self.pending.as_ref().is_none_or(|p| !matches!(&p.resolution, ChoiceResolution::Frame { frame, choice: FrameChoice::BQ027ShownTarget | FrameChoice::VampireBlood } if same(frame.as_ref(), f.as_ref()))) {
            return Err("血畜后置帧须由唯一原目标或饮血待选持有".into());
        }
        Ok(())
    }
}
