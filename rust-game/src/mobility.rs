//! One canonical mobility program, with a narrow BQ035 event-time grant receipt.
//! Historical witnesses establish bounded consistency, not cryptographic proof
//! against arbitrary coherent rewrites of an entire game history.
use crate::bq035::same;
use crate::{
    catalog,
    engine::RuleResult,
    model::*,
    rules::{self, *},
};
use std::collections::BTreeSet;

pub(crate) fn printed(id: &str) -> bool {
    matches!(id, "BQ026" | "JC014" | "JZ45" | "JC061" | "BQ035" | "WM039")
}
pub(crate) fn ability() -> AbilitySpec {
    let mut a = rules::ability(
        "mobility",
        "机动",
        Timing::Fast,
        vec![],
        vec![rules::target(
            Zone::Region,
            EntityKind::Any,
            Relation::Any,
            Range::Mobility,
        )],
        vec![Op::MoveOnBoard {
            entity: EntityRef::Source,
            region: RegionRef::Target(0),
        }],
        Some(Event::ConfrontationStart),
    );
    a.requires_ready_source = true;
    a
}
fn uses(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::MoveOnBoard { .. } | Op::XG12GrantedMobility { .. } => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => uses(body),
        Op::IfTargetExhausted {
            exhausted, ready, ..
        } => uses(std::slice::from_ref(exhausted)) || uses(std::slice::from_ref(ready)),
        _ => false,
    })
}
pub(crate) fn related_ability(a: &AbilitySpec) -> bool {
    a.key == "mobility"
        || a.event == Some(Event::ConfrontationStart)
        || uses(&a.ops)
        || a.modes.iter().any(|m| uses(&m.ops))
}
pub(crate) fn canonical(a: &AbilitySpec) -> bool {
    same(a, &ability()) || crate::batch4_shape::canonical_mobility(a)
}
pub(crate) fn canonical_target(slot: &TargetSlotSpec) -> bool {
    slot.range == Range::Mobility && same(slot, &ability().targets[0])
}
pub(crate) fn related_frame(f: &ResolutionFrame) -> bool {
    f.ability_key == "mobility"
        || f.source.observed_mobility_grant.is_some()
        || f.source.observed_exposure.is_some()
        || f.steps.iter().any(|s| uses(std::slice::from_ref(&s.op)))
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    for a in d.abilities.iter().filter(|a| related_ability(a)) {
        if !(printed(id) || id == "WM040") || !same(a, &ability()) {
            return Err("机动只能静态印在完整准入的原牌上，授予不能移植定义".into());
        }
    }
    Ok(())
}
impl Game {
    pub(crate) fn has_mobility(&self, c: &Card) -> bool {
        self.board(&c.id).is_some_and(|(_, live)| {
            !live.face_down && !self.attached_character(&live.id)
                && catalog::card(&live.definition).kind == "character"
                && (printed(&live.definition) || (live.definition == "WM040" && self.society_exposure(live.controller) > 0) || self.bq035_mobility_witness(&live.id).is_some() || self.xg12_mobility_grant(live).is_some())
        })
    }
    pub(crate) fn mobility_source(&self, c: &Card, region: usize) -> SourceSnapshot {
        SourceSnapshot {
            observed_exposure: (c.definition == "WM040" && self.society_exposure(c.controller) > 0).then(|| self.society_exposure(c.controller)),
            observed_death: None,
            observed_mobility_grant: (!(printed(&c.definition) || (c.definition == "WM040" && self.society_exposure(c.controller) > 0)))
                .then(|| self.bq035_mobility_witness(&c.id))
                .flatten(),
            card: c.clone(),
            region: Some(region),
            source_region_instance: None,
            attachment_host_instance: None,
            play_source: None,
        }
    }
    pub(crate) fn mobility_immutable_identity(&self, old: &Card) -> RuleResult<()> {
        let private = self.players.iter().flat_map(|p| {
            p.hand
                .iter()
                .chain(&p.deck)
                .chain(&p.assets)
                .chain(&p.graveyard)
                .chain(&p.score_cards)
                .chain(p.society_zone.card.iter())
        });
        let current = self
            .regions
            .iter()
            .flat_map(|r| std::iter::once(&r.card).chain(&r.cards))
            .chain(private)
            .chain(&self.world)
            .chain(self.attachments.iter().map(|a| &a.card))
            .chain(self.sealed_cards.iter().map(|a| &a.card))
            .chain(self.stack.iter().filter_map(|s| s.card.as_ref()))
            .chain(self.effects.iter().filter_map(|e| {
                if let Effect::Bury { card } = e {
                    Some(card)
                } else {
                    None
                }
            }))
            .filter(|live| live.id == old.id)
            .collect::<Vec<_>>();
        if current.len() > 1
            || current
                .iter()
                .any(|live| live.definition != old.definition || live.owner != old.owner)
        {
            return Err("机动历史UID与现存实体的印刷定义或拥有者矛盾".into());
        }
        Ok(())
    }
    pub(crate) fn validate_mobility_recipient(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        let c = &s.card;
        let admitted = |c: &Card, kind: &str| {
            catalog::catalog()
                .cards
                .iter()
                .any(|d| d.id == c.definition && d.kind == kind)
        };
        let in_mobility = self.window == Some(Window::Mobility)
            || self
                .win_contexts
                .iter()
                .any(|w| w.resume_window == Window::Mobility);
        if !in_mobility
            || self.status != "playing"
            || actor >= self.players.len()
            || c.controller != actor
            || c.owner >= self.players.len()
            || c.id.is_empty()
            || c.face_down
            || c.exhausted || self.attached_character(&c.id)
            || !admitted(c, "character")
            || s.region.is_none_or(|r| r >= self.regions.len())
            || s.source_region_instance.is_some()
            || s.attachment_host_instance.is_some()
            || s.play_source.is_some()
            || s.observed_death.is_some()
        {
            return Err("机动事件的来源、阶段、行动者或无关观察无效".into());
        }
        self.mobility_immutable_identity(c)?;
        Ok(())
    }
    pub(crate) fn validate_mobility_source(
        &self,
        actor: usize,
        s: &SourceSnapshot,
    ) -> RuleResult<()> {
        self.validate_mobility_recipient(actor,s)?;
        let c = &s.card;
        let admitted = |c: &Card, kind: &str| catalog::catalog().cards.iter().any(|d|d.id==c.definition&&d.kind==kind);
        if printed(&c.definition) || s.observed_exposure.is_some() {
            if s.observed_exposure.is_some_and(|n| c.definition != "WM040" || n == 0) || (printed(&c.definition) && s.observed_exposure.is_some()) { return Err("条件机动仅准入心理审察委员的真实暴露观察".into()); }
            if s.observed_mobility_grant.is_some() {
                return Err("印刷机动不能夹带授予观察".into());
            }
        } else {
            // The closed pool can produce vampires from printed vampires,
            // printed humans (JC036), and the conditional JC030 bat only.
            // Historical subtype changes may end after the event, but an
            // impossible printed recipient cannot be justified by a true bit.
            let print = catalog::card(&c.definition);
            if c.definition != "JC030"
                && !print.subtypes.iter().any(|s| s == "人类" || s == "吸血鬼")
            {
                return Err("当前准入卡池无法令该来源成为吸血鬼，不能伪造机动观察".into());
            }
            let w = s
                .observed_mobility_grant
                .as_ref()
                .ok_or("非印刷机动缺少真实BQ035事件见证")?;
            if w.turn != self.turn
                || !w.vampire
                || w.knight.definition != "BQ035"
                || w.knight.id.is_empty()
                || w.knight.face_down
                || w.knight.controller != actor
                || w.knight.owner >= self.players.len()
                || w.knight_region >= self.regions.len()
                || w.blood.card.id.is_empty()
                || w.blood.card.face_down
                || !admitted(&w.blood.card, "attachment")
                || catalog::card(&w.blood.card.definition).name != "鲜血"
                || w.blood.host_id != c.id
                || w.blood.card.owner >= self.players.len()
                || w.blood.card.controller >= self.players.len()
                || w.knight.id == c.id
                || w.blood.card.id == c.id
                || w.blood.card.id == w.knight.id
            {
                return Err("BQ035授予机动的回合、吸血鬼、骑士或鲜血宿主观察不一致".into());
            }
            self.mobility_immutable_identity(&w.knight)?;
            self.mobility_immutable_identity(&w.blood.card)?;
        }
        Ok(())
    }
    pub(crate) fn validate_mobility_event_source(&self, d: &Declaration) -> RuleResult<()> {
        if !canonical(&d.ability) { return Err("机动事件必须保留完整规范程序".into()); }
        if let Some(p) = crate::batch4_shape::mobility_proof(&d.ability) {
            self.validate_xg12_mobility_source(d.actor,&d.source,p)
        } else { self.validate_mobility_source(d.actor,&d.source) }
    }
    pub(crate) fn validated_mobility_declaration(&self, d: &Declaration) -> RuleResult<bool> {
        if !related_ability(&d.ability) && d.source.observed_mobility_grant.is_none() && d.source.observed_exposure.is_none() {
            return Ok(false);
        }
        if !canonical(&d.ability) {
            return Err("机动只准入完整原始事件、费用、目标和自身移动程序".into());
        }
        self.validate_mobility_event_source(d)?;
        self.validate_mobility_capture(&d.source.card.id)?;
        Ok(true)
    }
    pub(crate) fn validated_mobility_frame(&self, f: &ResolutionFrame) -> RuleResult<bool> {
        if !related_frame(f) {
            return Ok(false);
        }
        let a = if let [Step { op: Op::XG12GrantedMobility { proof }, .. }] = f.steps.as_slice() {
            crate::batch4_shape::mobility_ability((**proof).clone())
        } else { ability() };
        self.validate_mobility_event_source(&Declaration { actor:f.actor,source:f.source.clone(),ability:a.clone() })?;
        self.validate_mobility_capture(&f.source.card.id)?;
        if f.ability_key != a.key
            || f.frame_id.is_empty()
            || f.cursor != 0
            || !matches!(f.guard, GuardState::Unchecked)
            || !f.already_paid.is_empty()
            || f.chosen_region.is_some()
            || f.targets.len() != 1
            || !same(
                &f.steps,
                &vec![Step {
                    context: f.actor,
                    op: a.ops[0].clone(),
                }],
            )
        {
            return Err("机动帧游标、守卫、程序、费用或目标数量无效".into());
        }
        let t = &f.targets[0];
        let r =
            t.id.strip_prefix("region:")
                .and_then(|s| s.parse::<usize>().ok())
                .filter(|r| t.id == format!("region:{r}"))
                .ok_or("机动地区ID必须为规范地区索引")?;
        if r >= self.regions.len()
            || !self.regions_adjacent(r, f.source.region.unwrap())
            || !same(&t.spec, &a.targets[0])
            || t.region_instance.is_some()
            || t.public.instance_id != t.id
            || t.public.kind != "region"
            || t.public.region != Some(r)
            || t.public.owner.is_some()
            || t.public.controller.is_some()
            || !t.public.valid
            || t.public.status != "valid"
            || t.public.invalid_reason.is_some()
            || !catalog::catalog()
                .cards
                .iter()
                .any(|d| d.kind == "region" && d.name == t.public.label)
        {
            return Err("机动目标范围或冻结公开摘要不符".into());
        }
        Ok(true)
    }
    pub(crate) fn validate_mobility_state(&self) -> RuleResult<()> {
        let mut seen = BTreeSet::new();
        for u in self
            .turn_ability_usage
            .iter()
            .filter(|u| u.ability_key == "mobility")
        {
            if u.turn != self.turn
                || u.uses != 1
                || u.source_instance.is_empty()
                || !seen.insert(&u.source_instance)
            {
                return Err("机动每回合事件捕获记录无效或重复".into());
            }
        }
        let mut recipients = BTreeSet::new();
        for item in &self.stack {
            if let Some(f) = &item.frame {
                if self.validated_mobility_frame(f)? {
                    if !recipients.insert(f.source.card.id.clone())
                        || item.id != f.frame_id
                        || item.controller != f.actor
                        || item.card.is_some()
                        || item.deploy_region.is_some()
                        || item.reveal
                        || item.target.as_ref() != Some(&f.targets[0].id)
                    {
                        return Err("机动响应堆栈外壳或重复实例无效".into());
                    }
                }
            } else if item.label.ends_with("：机动") {
                return Err("机动响应壳缺少冻结帧".into());
            }
        }
        for e in &self.effects {
            match e {
                Effect::Declare { declaration: d } if self.validated_mobility_declaration(d)? => {
                    if !recipients.insert(d.source.card.id.clone()) {
                        return Err("同一实例重复排队机动".into());
                    }
                }
                Effect::Frame { frame } if related_frame(frame) => {
                    return Err("机动不得通过直接续体绕过响应".into())
                }
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Frame { frame, .. } if related_frame(frame) => {
                    return Err("机动没有帧内待选续体".into())
                }
                ChoiceResolution::Declare {
                    declaration: d,
                    stage,
                } if self.validated_mobility_declaration(d)? => {
                    if !recipients.insert(d.source.card.id.clone())
                        || !matches!(stage, DeclareChoice::Target)
                        || p.seat != d.actor
                        || p.choice.player_id != player_id(d.actor)
                        || p.choice.id.is_empty()
                        || p.choice.kind != "trigger"
                        || p.choice.min != Some(0)
                        || p.choice.max != Some(1)
                        || p.choice.allow_decline != Some(true)
                        || p.choice.amount.is_some()
                        || !p.choice.preview_cards.is_empty()
                        || p.choice.title
                            != format!("{}：机动", catalog::card(&d.source.card.definition).name)
                        || p.choice.description != "选择符合数量限制的选项，然后确认。"
                        || !same(
                            &p.choice.options,
                            &self.binding_options(d.actor, &d.source, &ability().targets[0]),
                        )
                    {
                        return Err("机动声明待选、行动者或真实地区选项不符".into());
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
    fn validate_mobility_capture(&self, instance: &str) -> RuleResult<()> {
        if self
            .turn_ability_usage
            .iter()
            .filter(|u| {
                u.ability_key == "mobility"
                    && u.source_instance == instance
                    && u.turn == self.turn
                    && u.uses == 1
            })
            .count()
            != 1
        {
            return Err("机动缺少本回合唯一事件捕获记录".into());
        }
        Ok(())
    }
}
