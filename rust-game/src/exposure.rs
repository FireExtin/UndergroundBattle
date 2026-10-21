//! Verified WM039/WM040/WM042 and personal society sun/moon markers.
//! Marker cancellation is immediate; keywords always consult the controller.
use crate::{
    catalog,
    engine::RuleResult,
    model::*,
    rules::{self, *},
};
use serde::Serialize;
use std::collections::BTreeSet;

pub(crate) fn same<T: Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
pub(crate) fn definition(id: &str) -> Option<Definition> {
    match id {
        "WM039" => {
            let mut a = rules::ability(
                "wm039-analysis",
                "行动1：移除本方1暴露，自身获得调查1直到回合结束",
                Timing::Standard,
                vec![Cost::Assets(1)],
                vec![],
                vec![Op::WM039Analysis],
                None,
            );
            a.activation_only = true;
            a.per_turn_limit = Some(1);
            Some(rules::with_abilities(vec![a, crate::mobility::ability()]))
        }
        "WM040" => {
            let mut heal = rules::ability(
                "wm040-heal",
                "快速行动1、横置：放置本方1暴露，移除本地区全部本方角色的全部创伤",
                Timing::Fast,
                vec![Cost::Assets(1), Cost::ExhaustSource],
                vec![],
                vec![Op::WM040Heal],
                None,
            );
            heal.activation_only = true;
            Some(rules::with_abilities(vec![
                rules::ability(
                    "wm040-repress",
                    "进场触发：移除全部暴露，然后遏制实际移除数量",
                    Timing::Fast,
                    vec![],
                    vec![rules::target(
                        Zone::Player,
                        EntityKind::Any,
                        Relation::Any,
                        Range::Anywhere,
                    )],
                    vec![Op::WM040Repress],
                    Some(Event::Enter),
                ),
                heal,
                crate::mobility::ability(),
            ]))
        }
        "WM042" => {
            let mut a = rules::ability(
                "wm042-news",
                "快速行动：新闻管控",
                Timing::Fast,
                vec![],
                vec![],
                vec![],
                None,
            );
            a.play_only = true;
            a.modes = vec![
                Mode {
                    key: "expose".into(),
                    label: "所有秘社各放置1暴露".into(),
                    targets: vec![],
                    ops: vec![Op::WM042Exposure { add: true }],
                },
                Mode {
                    key: "cover".into(),
                    label: "所有秘社各移除1暴露".into(),
                    targets: vec![],
                    ops: vec![Op::WM042Exposure { add: false }],
                },
            ];
            Some(rules::with_abilities(vec![a]))
        }
        _ => None,
    }
}
fn uses(ops: &[Op]) -> bool {
    ops.iter().any(|o| match o {
        Op::WM039Analysis | Op::WM040Heal | Op::WM040Repress | Op::WM042Exposure { .. } => true,
        Op::ForEachLivingPlayer(b) | Op::ForEachLivingPlayerFromActor(b) => uses(b),
        Op::IfTargetExhausted {
            exhausted, ready, ..
        } => uses(std::slice::from_ref(exhausted)) || uses(std::slice::from_ref(ready)),
        _ => false,
    })
}
fn programs(id: &str) -> Vec<AbilitySpec> {
    let Some(d) = definition(id) else {
        return vec![];
    };
    let mut out = d.abilities.clone();
    for a in d.abilities {
        for m in &a.modes {
            let mut selected = a.clone();
            selected.label = format!("{}：{}", a.label, m.label);
            selected.targets = m.targets.clone();
            selected.ops = m.ops.clone();
            selected.modes.clear();
            out.push(selected);
        }
    }
    out
}
fn related(id: &str, key: &str, ops: &[Op]) -> bool {
    definition(id).is_some()
        || key.starts_with("wm039-")
        || key.starts_with("wm040-")
        || key.starts_with("wm042-")
        || uses(ops)
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if (related(id, &a.key, &a.ops) || a.modes.iter().any(|m| uses(&m.ops)))
        && !programs(id).iter().any(|p| same(a, p))
        && !(definition(id).is_some() && crate::jz22::runtime_ability_is_admitted(a))
    {
        return Err("暴露牌仅准入完整印刷程序与完整所选模式".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if definition(id).is_some_and(|p| !same(d, &p)) {
        return Err("暴露牌定义必须与完整原图规则一致".into());
    }
    Ok(())
}

impl Game {
    pub(crate) fn society_exposure(&self, seat: usize) -> u32 {
        self.players[seat].society_zone.exposure
    }
    pub(crate) fn hand_limit(&self, seat: usize) -> usize {
        let z = &self.players[seat].society_zone;
        7usize
            .saturating_add(z.secrecy as usize)
            .saturating_sub(z.exposure as usize)
    }
    pub(crate) fn add_society_markers(&mut self, seat: usize, exposure: u32, secrecy: u32) {
        if self.players[seat].eliminated {
            return;
        }
        let z = &mut self.players[seat].society_zone;
        let sun = u64::from(z.exposure) + u64::from(exposure);
        let moon = u64::from(z.secrecy) + u64::from(secrecy);
        z.exposure = sun.saturating_sub(moon).min(u64::from(u32::MAX)) as u32;
        z.secrecy = moon.saturating_sub(sun).min(u64::from(u32::MAX)) as u32;
    }
    pub(crate) fn current_public(&self, c: &Card, actor: usize) -> bool {
        rules::definition(&c.definition).traits.public
            || (c.definition == "WM040" && self.society_exposure(actor) > 0)
    }
    pub(crate) fn wm039_analysis(&mut self, f: &ResolutionFrame) {
        let z = &mut self.players[f.actor].society_zone;
        z.exposure = z.exposure.saturating_sub(1);
        // Removal is an effect, not a cost or an "if you do" condition.
        if self
            .board(&f.source.card.id)
            .is_some_and(|(_, c)| !c.face_down)
        {
            self.turn_attribute_modifiers.push(TurnAttributeModifier {
                target_instance: f.source.card.id.clone(),
                defense_bonus: 0,
                kill_bonus: 0,
                grants_retreat: false,
                printed_defense_override: None,
                ordinary_icons: Icons {
                    investigation: 1,
                    ..Default::default()
                },
                grants_renown: false,
                prevents_damage: false,
                expires_turn: self.turn,
            });
        }
    }
    pub(crate) fn wm040_heal(&mut self, f: &ResolutionFrame) {
        self.add_society_markers(f.actor, 1, 0);
        if let Some(r) = self.original_region(&f.source) {
            for c in &mut self.regions[r].cards {
                if !c.face_down
                    && c.controller == f.actor
                    && catalog::card(&c.definition).kind == "character"
                {
                    c.wounds = 0;
                }
            }
        }
    }
    pub(crate) fn wm042_exposure(&mut self, add: bool) {
        for seat in 0..self.players.len() {
            if self.players[seat].eliminated {
                continue;
            }
            if add {
                self.add_society_markers(seat, 1, 0);
            } else {
                let z = &mut self.players[seat].society_zone;
                z.exposure = z.exposure.saturating_sub(1);
            }
        }
    }
    pub(crate) fn wm040_repress_start(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        let removed: Vec<_> = self
            .players
            .iter()
            .filter(|p| !p.eliminated)
            .map(|p| (p.seat, p.society_zone.exposure))
            .collect();
        let count = removed.iter().fold(0u32, |n, (_, x)| n.saturating_add(*x));
        for (seat, _) in &removed {
            self.players[*seat].society_zone.exposure = 0;
        }
        let seat = f.targets[0]
            .id
            .strip_prefix('p')
            .and_then(|s| s.parse::<usize>().ok())
            .ok_or("遏制玩家无效")?;
        let regions = self.batch20_repress_regions(seat);
        let n = (count as usize).min(regions.iter().map(|(_, _, n)| *n as usize).sum());
        if n > 0 {
            self.choice(
                seat,
                "influence",
                format!("遏制{n}：选择并移除本方势力标志"),
                self.batch20_repress_options(&regions, n),
                n,
                n,
                Some(n as u32),
                ChoiceResolution::Frame {
                    frame: Box::new(f),
                    choice: FrameChoice::WM040Repress {
                        seat,
                        removed,
                        regions,
                    },
                },
            );
        }
        Ok(())
    }
    pub(crate) fn wm040_repress_complete(
        &mut self,
        seat: usize,
        regions: &[(usize, String, u32)],
        selected: &[String],
    ) -> RuleResult<()> {
        let team = self.team(seat);
        for (r, uid, _) in regions {
            let n = selected
                .iter()
                .filter(|id| id.starts_with(&format!("region:{r}:marker:")))
                .count() as u32;
            let reg = self
                .regions
                .get_mut(*r)
                .filter(|r| !r.vacant && r.card.id == *uid && r.influence[team] >= n)
                .ok_or("遏制原地区已失效")?;
            reg.influence[team] -= n;
        }
        Ok(())
    }
    pub(crate) fn validate_exposure_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if self.validated_cross_card_frame(f)? {
            return Ok(());
        }
        let ops: Vec<_> = f.steps.iter().map(|s| s.op.clone()).collect();
        if !related(&f.source.card.definition, &f.ability_key, &ops) {
            return Ok(());
        }
        let c = &f.source.card;
        if definition(&c.definition).is_none()
            || f.actor >= self.players.len()
            || c.controller != f.actor
            || c.owner >= self.players.len()
            || c.id.is_empty()
            || f.frame_id.is_empty()
            || f.source.observed_death.is_some()
            || f.source.observed_mobility_grant.is_some()
            || f.source.attachment_host_instance.is_some()
            || f.source.observed_exposure.is_some()
        {
            return Err("暴露牌来源与冻结行动者不符".into());
        }
        self.mobility_immutable_identity(c)?;
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            if c.definition == "WM042"
                || f.cursor != 0
                || !matches!(f.guard, GuardState::Unchecked)
                || !ops.is_empty()
                || !f.targets.is_empty()
                || f.chosen_region != f.source.region
                || f.source.region.is_none_or(|r| r >= self.regions.len())
                || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(_)])
                || (f.ability_key == "deploy" && (c.face_down || !f.source.valid_character_play()))
                || (f.ability_key == "reveal" && (!c.face_down || f.source.play_source.is_some()))
            {
                return Err("暴露角色派遣帧无效".into());
            }
            return Ok(());
        }
        let a = programs(&c.definition)
            .into_iter()
            .find(|a| a.key == f.ability_key && a.modes.is_empty() && same(&a.ops, &ops))
            .ok_or("暴露牌完整程序无效")?;
        if c.face_down
            || f.steps.iter().any(|s| s.context != f.actor)
            || f.chosen_region.is_some()
            || !same(
                &a.targets,
                &f.targets.iter().map(|t| t.spec.clone()).collect::<Vec<_>>(),
            )
            || f.targets
                .iter()
                .any(|t| t.public.instance_id != t.id || t.region_instance.is_some())
            || (f.cursor != 0
                && !(a.key == "wm040-repress"
                    && f.cursor == 1
                    && matches!(f.guard, GuardState::Accepted)))
            || (f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked))
        {
            return Err("暴露牌帧游标、目标或守卫无效".into());
        }
        if c.definition != "WM042"
            && (f.source.region.is_none_or(|r| r >= self.regions.len())
                || f.source
                    .source_region_instance
                    .as_ref()
                    .is_none_or(|s| s.is_empty())
                || f.source.play_source.is_some())
        {
            return Err("暴露角色必须冻结原地区与来源".into());
        }
        let assets = |ids: &[String], n: usize| {
            ids.len() == n
                && ids.iter().all(|s| !s.is_empty())
                && ids.iter().collect::<BTreeSet<_>>().len() == n
        };
        let paid = match a.key.as_str() {
            "wm039-analysis" => {
                matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if assets(ids, 1))
                    && self
                        .turn_ability_usage
                        .iter()
                        .filter(|u| {
                            u.source_instance == c.id
                                && u.ability_key == a.key
                                && u.turn == self.turn
                                && u.uses == 1
                        })
                        .count()
                        == 1
            }
            "wm040-heal" => {
                matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids), PaidCost::Exhausted(id)] if assets(ids, 1) && id == &c.id && !c.exhausted)
            }
            "wm040-repress" => f.already_paid.is_empty(),
            "wm042-news" => {
                matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if assets(ids, 1))
                    && f.source.region.is_none()
                    && f.source.play_source.is_none()
            }
            _ => false,
        };
        if !paid {
            return Err("暴露牌费用与每回合限次收据无效".into());
        }
        Ok(())
    }
    pub(crate) fn validate_exposure_state(&self) -> RuleResult<()> {
        for p in &self.players {
            if p.society_zone.exposure > 0 && p.society_zone.secrecy > 0 {
                return Err("暴露与隐秘必须立即等量抵消".into());
            }
        }
        let mut usage = BTreeSet::new();
        for u in self
            .turn_ability_usage
            .iter()
            .filter(|u| u.ability_key == "wm039-analysis")
        {
            if u.turn != self.turn
                || u.uses != 1
                || u.source_instance.is_empty()
                || !usage.insert(&u.source_instance)
            {
                return Err("精神分析师限次记录无效".into());
            }
        }
        let declaration = |d: &Declaration| -> RuleResult<()> {
            if self.validated_cross_card_declaration(d)? {
                return Ok(());
            }
            if related(&d.source.card.definition, &d.ability.key, &d.ability.ops) {
                validate_ability(&d.source.card.definition, &d.ability)?;
                if d.source.card.definition != "WM040"
                    || d.ability.key != "wm040-repress"
                    || d.actor >= self.players.len()
                    || d.source.card.controller != d.actor
                    || d.source.card.face_down
                    || d.source.card.id.is_empty()
                    || d.source.card.owner >= self.players.len()
                    || d.source.region.is_none_or(|r| r >= self.regions.len())
                    || d.source
                        .source_region_instance
                        .as_ref()
                        .is_none_or(|s| s.is_empty())
                    || d.source.play_source.is_some()
                    || d.source.observed_death.is_some()
                    || d.source.observed_mobility_grant.is_some()
                    || d.source.observed_exposure.is_some()
                {
                    return Err("心理审察委员进场声明无效".into());
                }
            }
            Ok(())
        };
        for item in &self.stack {
            if item
                .card
                .as_ref()
                .is_some_and(|c| definition(&c.definition).is_some())
                && item.frame.is_none()
            {
                return Err("暴露牌响应壳缺少帧".into());
            }
            if let Some(f) = &item.frame {
                self.validate_exposure_frame(f)?;
                if related(
                    &f.source.card.definition,
                    &f.ability_key,
                    &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>(),
                ) && !crate::mobility::related_frame(f)
                {
                    if item.id != f.frame_id
                        || item.controller != f.actor
                        || item.target.as_ref() != f.targets.first().map(|t| &t.id)
                        || f.cursor != 0
                    {
                        return Err("暴露牌响应壳与帧不符".into());
                    }
                    let play = matches!(f.ability_key.as_str(), "deploy" | "reveal" | "wm042-news");
                    if item.card.is_some() != play
                        || item.card.as_ref().is_some_and(|c| {
                            c.definition != f.source.card.definition
                                || c.owner != f.source.card.owner
                                || c.controller != f.actor
                                || c.face_down
                        })
                        || item.deploy_region
                            != (if f.ability_key == "wm042-news" {
                                None
                            } else if play {
                                f.source.region
                            } else {
                                None
                            })
                        || item.reveal != (f.ability_key == "reveal")
                    {
                        return Err("暴露牌响应实体无效".into());
                    }
                }
            }
        }
        for e in &self.effects {
            match e {
                Effect::Declare { declaration: d } => declaration(d)?,
                Effect::Frame { frame } => self.validate_exposure_frame(frame)?,
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Declare {
                    declaration: d,
                    stage,
                } => {
                    declaration(d)?;
                    if d.source.card.definition == "WM040" && d.ability.key == "wm040-repress" {
                        let expected =
                            self.binding_options(d.actor, &d.source, &d.ability.targets[0]);
                        if !matches!(stage, DeclareChoice::Target)
                            || p.seat != d.actor
                            || p.choice.player_id != player_id(d.actor)
                            || p.choice.kind != "trigger"
                            || p.choice.min != Some(0)
                            || p.choice.max != Some(1)
                            || p.choice.allow_decline != Some(true)
                            || p.choice.amount.is_some()
                            || !p.choice.preview_cards.is_empty()
                            || p.choice.title
                                != format!("{}：{}", catalog::card("WM040").name, d.ability.label)
                            || !same(&p.choice.options, &expected)
                        {
                            return Err("心理审察委员触发目标待选无效".into());
                        }
                    }
                }
                ChoiceResolution::Frame { frame: f, choice } => {
                    self.validate_exposure_frame(f)?;
                    if let FrameChoice::WM040Repress {
                        seat,
                        removed,
                        regions,
                    } = choice
                    {
                        let count = removed.iter().fold(0u32, |n, (_, x)| n.saturating_add(*x));
                        let n =
                            (count as usize).min(regions.iter().map(|(_, _, n)| *n as usize).sum());
                        if f.source.card.definition != "WM040"
                            || f.ability_key != "wm040-repress"
                            || f.cursor != 1
                            || !matches!(f.guard, GuardState::Accepted)
                            || *seat >= self.players.len()
                            || f.targets[0].id != player_id(*seat)
                            || p.seat != *seat
                            || p.choice.player_id != player_id(*seat)
                            || p.choice.kind != "influence"
                            || p.choice.min != Some(n)
                            || p.choice.max != Some(n)
                            || p.choice.amount != Some(n as u32)
                            || p.choice.allow_decline != Some(false)
                            || n == 0
                            || !p.choice.preview_cards.is_empty()
                            || p.choice.title != format!("遏制{n}：选择并移除本方势力标志")
                            || removed.iter().map(|(s, _)| *s).collect::<Vec<_>>()
                                != self
                                    .players
                                    .iter()
                                    .filter(|p| !p.eliminated)
                                    .map(|p| p.seat)
                                    .collect::<Vec<_>>()
                            || self
                                .players
                                .iter()
                                .any(|p| !p.eliminated && p.society_zone.exposure > 0)
                            || !same(regions, &self.batch20_repress_regions(*seat))
                            || !same(&p.choice.options, &self.batch20_repress_options(regions, n))
                        {
                            return Err("心理审察委员实际移除量或遏制待选无效".into());
                        }
                    } else if definition(&f.source.card.definition).is_some()
                        && !self.validated_cross_card_frame(f)?
                    {
                        return Err("暴露牌存在不支持的帧内选择".into());
                    }
                }
                _ => {}
            }
        }
        self.validate_rumor_state()
    }
    // Integration hook: the parallel black-card branch supplies the real observer.
    // Remove this stub when merging its implementation; never call both.
    pub(crate) fn observe_black_deck_to_graveyard(&mut self, _deck_seat: usize, _cards: &[Card]) {}
}
