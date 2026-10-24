//! Complete WM056: a paid, once-per-turn, untargeted adjacent influence move.
use crate::{
    catalog,
    engine::RuleResult,
    model::*,
    rules::{self, *},
};
use std::collections::BTreeSet;
const KEY: &str = "wm056-adjacent-influence";
const TITLE: &str = "选择赃物贩子移走势力标志的相邻地区";
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
pub(crate) fn definition() -> Definition {
    let mut a = rules::ability(
        KEY,
        "标准行动②：移动相邻地区一个本方势力至本地区（每回合一次）",
        Timing::Standard,
        vec![Cost::Assets(2)],
        vec![],
        vec![Op::WM056MoveOneAdjacentInfluence],
        None,
    );
    a.activation_only = true;
    a.per_turn_limit = Some(1);
    rules::with_abilities(vec![a])
}
fn uses(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::WM056MoveOneAdjacentInfluence => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => uses(body),
        Op::IfTargetExhausted {
            exhausted, ready, ..
        } => uses(std::slice::from_ref(exhausted)) || uses(std::slice::from_ref(ready)),
        _ => false,
    })
}
fn relevant(a: &AbilitySpec) -> bool {
    a.key == KEY || uses(&a.ops) || a.modes.iter().any(|m| uses(&m.ops))
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if !relevant(a) {
        return Ok(());
    }
    if id != "WM056" || !same(a, &definition().abilities[0]) {
        return Err("仅准入WM056完整相邻势力移动标准行动".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if (id == "WM056" && !same(d, &definition()))
        || (id != "WM056" && d.abilities.iter().any(relevant))
    {
        return Err("WM056完整定义或移动程序不能改写或移植".into());
    }
    Ok(())
}
fn related(f: &ResolutionFrame) -> bool {
    f.source.card.definition == "WM056"
        || f.ability_key == KEY
        || f.steps.iter().any(|s| uses(std::slice::from_ref(&s.op)))
}
impl Game {
    pub(crate) fn validate_wm056_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if self.validated_cross_card_frame(f)? { return Ok(()); }
        if !related(f) {
            return Ok(());
        }
        if f.source.card.definition == "WM056"
            && matches!(f.ability_key.as_str(), "renown" | "jc089-combat-glory")
            && !f.steps.iter().any(|s| uses(std::slice::from_ref(&s.op)))
        {
            return Ok(());
        }
        let c = &f.source.card;
        if c.definition != "WM056"
            || c.id.is_empty()
            || f.actor >= self.players.len()
            || c.owner >= self.players.len()
            || c.controller != f.actor
            || f.frame_id.is_empty()
            || f.source.region.is_none_or(|r| r >= self.regions.len())
            || f.source
                .source_region_instance
                .as_ref()
                .is_none_or(|id| id.is_empty())
            || f.source.observed_death.is_some()
            || f.source.attachment_host_instance.is_some()
        {
            return Err("WM056冻结来源、行动者或原地区实例无效".into());
        }
        let paid = |max: usize| {
            matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)]
            if f.money_units(0)<=max&&ids.iter().all(|id|!id.is_empty())&&ids.iter().collect::<BTreeSet<_>>().len()==ids.len())
        };
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            let origin = if f.ability_key == "deploy" {
                !c.face_down && f.source.valid_character_play()
            } else {
                c.face_down && f.source.play_source.is_none()
            };
            if !origin
                || !f.steps.is_empty()
                || !f.targets.is_empty()
                || f.cursor != 0
                || !matches!(f.guard, GuardState::Unchecked)
                || f.chosen_region != f.source.region
                || !paid(1)
            {
                return Err("WM056派遣或现身帧无效".into());
            }
            return Ok(());
        }
        if f.ability_key != KEY
            || c.face_down
            || f.source.play_source.is_some()
            || f.chosen_region.is_some()
            || self
                .turn_ability_usage
                .iter()
                .filter(|u| {
                    u.source_instance == c.id
                        && u.ability_key == KEY
                        && u.turn == self.turn
                        && u.uses == 1
                })
                .count()
                != 1
            || !matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)] if f.money_units(0)==2&&paid(2))
            || !f.targets.is_empty()
            || !matches!(f.steps.as_slice(),[Step{context,op:Op::WM056MoveOneAdjacentInfluence}] if *context==f.actor)
            || !((f.cursor == 0 && matches!(f.guard, GuardState::Unchecked))
                || (f.cursor == 1 && matches!(f.guard, GuardState::Accepted)))
        {
            return Err("WM056支付、次数程序、冻结目标或游标无效".into());
        }
        Ok(())
    }
    fn wm056_origins(&self, f: &ResolutionFrame) -> Vec<(usize, String)> {
        let Some(dest) = f.source.region.filter(|&r| {
            self.region_live(r)
                && f.source.source_region_instance.as_ref() == Some(&self.regions[r].card.id)
        }) else {
            return vec![];
        };
        let team = self.team(f.actor);
        self.regions
            .iter()
            .enumerate()
            .filter(|(r, v)| !v.vacant && self.regions_adjacent(*r, dest) && v.influence[team] > 0)
            .map(|(r, v)| (r, v.card.id.clone()))
            .collect()
    }
    fn wm056_options(&self, regions: &[(usize, String)]) -> Vec<ChoiceOption> {
        regions
            .iter()
            .map(|(r, _)| ChoiceOption {
                id: format!("region:{r}"),
                label: format!(
                    "地区{} {}",
                    r + 1,
                    catalog::card(&self.regions[*r].card.definition).name
                ),
                card: None,
            })
            .collect()
    }
    pub(crate) fn wm056_start(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        self.validate_wm056_frame(&f)?;
        let regions = self.wm056_origins(&f);
        if regions.is_empty() {
            return Ok(());
        }
        self.choice(
            f.actor,
            "wm056-influence-origin",
            TITLE.into(),
            self.wm056_options(&regions),
            1,
            1,
            Some(1),
            ChoiceResolution::Frame {
                frame: Box::new(f),
                choice: FrameChoice::WM056InfluenceOrigin { regions },
            },
        );
        Ok(())
    }
    pub(crate) fn wm056_move(
        &mut self,
        f: &ResolutionFrame,
        chooser: usize,
        regions: &[(usize, String)],
        selected: &[String],
    ) -> RuleResult<()> {
        self.validate_wm056_frame(f)?;
        let origin = selected
            .first()
            .and_then(|id| id.strip_prefix("region:"))
            .and_then(|n| n.parse::<usize>().ok());
        if chooser != f.actor
            || selected.len() != 1
            || regions != self.wm056_origins(f).as_slice()
            || origin.is_none_or(|r| !regions.iter().any(|(n, _)| *n == r))
        {
            return Err("WM056选择者、原地区或势力标志已失效".into());
        }
        let origin = origin.unwrap();
        let dest = f.source.region.unwrap();
        let team = self.team(f.actor);
        self.regions[origin].influence[team] -= 1;
        self.place_influence(f.actor, dest, 1);
        self.note(format!(
            "{} 将地区{}的一个本方势力标志移至地区{}",
            self.players[f.actor].name,
            origin + 1,
            dest + 1
        ));
        Ok(())
    }
    pub(crate) fn validate_wm056_state(&self) -> RuleResult<()> {
        let stored = |f: &ResolutionFrame| -> RuleResult<()> {
            self.validate_wm056_frame(f)?;
            if related(f)
                && f.ability_key == KEY
                && (f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked))
            {
                return Err("WM056堆栈不能保存已执行续体".into());
            }
            Ok(())
        };
        for s in &self.stack {
            if let Some(f) = &s.frame {
                if self.validated_cross_card_frame(f)? { continue; }
                stored(f)?;
                if related(f) || s.card.as_ref().is_some_and(|c| c.definition == "WM056") {
                    if f.source.card.definition != "WM056"
                        || s.id != f.frame_id
                        || s.controller != f.actor
                    {
                        return Err("WM056堆栈来源或冻结帧不一致".into());
                    }
                    if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
                        if s.card.as_ref().is_none_or(|c| {
                            c.definition != "WM056"
                                || c.owner != f.source.card.owner
                                || c.controller != f.actor
                                || c.face_down
                        }) || s.deploy_region != f.source.region
                            || s.reveal != (f.ability_key == "reveal")
                            || s.target.is_some()
                        {
                            return Err("WM056派遣堆栈外壳无效".into());
                        }
                    } else if s.card.is_some()
                        || s.target.is_some()
                        || s.deploy_region.is_some()
                        || s.reveal
                    {
                        return Err("WM056行动堆栈外壳无效".into());
                    }
                }
            } else if s.card.as_ref().is_some_and(|c| c.definition == "WM056") {
                return Err("WM056堆栈缺少冻结帧".into());
            }
        }
        let declaration = |d: &Declaration| -> RuleResult<()> {
            if self.validated_cross_card_declaration(d)? { return Ok(()); }
            if relevant(&d.ability)
                || (d.source.card.definition == "WM056"
                    && !crate::jz22::runtime_ability_is_admitted(&d.ability))
            {
                return Err("WM056标准行动不能伪装为触发声明".into());
            }
            Ok(())
        };
        for e in &self.effects {
            match e {
                Effect::Frame { frame } if crate::mobility::related_frame(frame) => return Err("机动不能绕过响应队列".into()),
                Effect::Frame { frame: f } => {
                    stored(f)?;
                    if related(f) && f.ability_key == KEY {
                        return Err("WM056行动不能绕过响应队列".into());
                    }
                }
                Effect::Declare { declaration: d } => declaration(d)?,
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Declare { declaration, .. } if self.validated_cross_card_declaration(declaration)? => {
                    self.validate_mobility_state()?;
                }
                ChoiceResolution::Frame { frame, .. } if crate::mobility::related_frame(frame) => return Err("机动没有帧内续体".into()),
                ChoiceResolution::Declare { declaration: d, .. } => declaration(d)?,
                ChoiceResolution::Frame { frame: f, choice }
                    if related(f) || matches!(choice, FrameChoice::WM056InfluenceOrigin { .. }) =>
                {
                    if f.source.card.definition != "WM056" || f.ability_key != KEY {
                        return Err("WM056待选不能移植到其他来源或程序".into());
                    }
                    self.validate_wm056_frame(f)?;
                    let FrameChoice::WM056InfluenceOrigin { regions } = choice else {
                        return Err("WM056不允许其他待选续体".into());
                    };
                    let expected = self.wm056_origins(f);
                    if f.ability_key != KEY
                        || f.cursor != 1
                        || !matches!(f.guard, GuardState::Accepted)
                        || p.seat != f.actor
                        || p.choice.kind != "wm056-influence-origin"
                        || p.choice.title != TITLE
                        || p.choice.min != Some(1)
                        || p.choice.max != Some(1)
                        || p.choice.id.is_empty()
                        || p.choice.player_id != format!("p{}", f.actor)
                        || p.choice.amount != Some(1)
                        || p.choice.allow_decline != Some(false)
                        || !p.choice.preview_cards.is_empty()
                        || p.choice.description != "选择符合数量限制的选项，然后确认。"
                        || expected.is_empty()
                        || regions != &expected
                        || !same(&p.choice.options, &self.wm056_options(regions))
                    {
                        return Err("WM056待选元数据、操控者或原相邻地区无效".into());
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
