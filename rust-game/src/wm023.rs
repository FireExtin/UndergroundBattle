//! Complete WM023: paid exhaustion, one frozen region, actor-private hand entry.
use crate::{
    catalog,
    engine::RuleResult,
    model::*,
    rules::{self, *},
};
use std::collections::BTreeSet;
const KEY: &str = "wm023-unique-hand-entry";
const TITLE: &str = "选择从手牌进场的独有法师或吸血鬼";
pub(crate) fn eligible(d: &catalog::CardDefinition) -> bool {
    d.kind == "character" && d.unique && d.subtypes.iter().any(|s| s == "法师" || s == "吸血鬼")
}
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
pub(crate) fn definition() -> Definition {
    let mut a = rules::ability(
        KEY,
        "标准行动①，横置：将手中独有法师或吸血鬼放置至目标地区",
        Timing::Standard,
        vec![Cost::Assets(1), Cost::ExhaustSource],
        vec![rules::target(
            Zone::Region,
            EntityKind::Any,
            Relation::Any,
            Range::Anywhere,
        )],
        vec![Op::WM023PutUniqueMageOrVampireFromHand],
        None,
    );
    a.activation_only = true;
    let mut d = rules::with_abilities(vec![a]);
    d.traits.public = true;
    d.traits.slow = true;
    d
}
fn uses(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::WM023PutUniqueMageOrVampireFromHand => true,
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
    if relevant(a) && (id != "WM023" || !same(a, &definition().abilities[0])) {
        return Err("只准入WM023完整手牌进场标准行动".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if (id == "WM023" && !same(d, &definition()))
        || (id != "WM023" && d.abilities.iter().any(relevant))
    {
        return Err("WM023完整卡面、迟缓或手牌程序不能改写或移植".into());
    }
    Ok(())
}
fn related(f: &ResolutionFrame) -> bool {
    f.source.card.definition == "WM023"
        || f.ability_key == KEY
        || f.steps.iter().any(|s| uses(std::slice::from_ref(&s.op)))
}
impl Game {
    pub(crate) fn validate_wm023_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if self.validated_cross_card_frame(f)? { return Ok(()); }
        if !related(f) {
            return Ok(());
        }
        if f.source.card.definition == "WM023"
            && matches!(f.ability_key.as_str(), "renown" | "jc089-combat-glory")
            && !f.steps.iter().any(|s| uses(std::slice::from_ref(&s.op)))
        {
            return Ok(());
        }
        let c = &f.source.card;
        if c.definition != "WM023"
            || c.id.is_empty()
            || f.actor >= self.players.len()
            || c.owner >= self.players.len()
            || c.controller != f.actor
            || f.frame_id.is_empty()
            || f.source.region.is_none_or(|r| r >= self.regions.len())
            || f.source
                .source_region_instance
                .as_ref()
                .is_none_or(|id| id.is_empty() || id == &c.id)
            || f.source.attachment_host_instance.is_some()
            || f.source.observed_death.is_some()
        {
            return Err("WM023冻结来源、操控者或原地区无效".into());
        }
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
                || !matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)]
                    if ids.len()<=4&&ids.iter().all(|id|!id.is_empty())&&ids.iter().collect::<BTreeSet<_>>().len()==ids.len())
            {
                return Err("WM023派遣或现身支付帧无效".into());
            }
            return Ok(());
        }
        if f.ability_key != KEY
            || c.face_down
            || c.exhausted
            || f.source.play_source.is_some()
            || !matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids),PaidCost::Exhausted(id)]
                if ids.len()==1&&!ids[0].is_empty()&&ids[0]!=c.id&&id==&c.id)
            || !matches!(f.steps.as_slice(),[Step{context,op:Op::WM023PutUniqueMageOrVampireFromHand}] if *context==f.actor)
            || !((f.cursor == 0 && matches!(f.guard, GuardState::Unchecked))
                || (f.cursor == 1 && matches!(f.guard, GuardState::Accepted)))
        {
            return Err("WM023资产/横置支付、来源或固定程序无效".into());
        }
        let t = f.targets.first().ok_or("WM023缺少目标地区")?;
        let r =
            t.id.strip_prefix("region:")
                .and_then(|n| n.parse::<usize>().ok())
                .filter(|r| *r < self.regions.len() && t.id == format!("region:{r}"));
        if f.targets.len() != 1
            || r.is_none()
            || f.chosen_region != r
            || !same(&t.spec, &definition().abilities[0].targets[0])
            || t.region_instance
                .as_ref()
                .is_none_or(|id| id.is_empty() || id == &c.id)
            || t.public.instance_id != t.id
            || t.public.kind != "region"
            || t.public.region != r
            || t.public.owner.is_some()
            || t.public.controller.is_some()
            || !t.public.valid
            || t.public.invalid_reason.is_some()
        {
            return Err("WM023目标地区规格或冻结实例无效".into());
        }
        Ok(())
    }
    fn wm023_options(&self, f: &ResolutionFrame) -> Vec<ChoiceOption> {
        self.players[f.actor]
            .hand
            .iter()
            .filter(|c| {
                let d = catalog::card(&c.definition);
                !c.face_down && eligible(d)
            })
            .map(|c| self.option(c, f.actor, None, None))
            .collect()
    }
    pub(crate) fn wm023_start(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        self.validate_wm023_frame(&f)?;
        if f.cursor != 1 || !matches!(f.guard, GuardState::Accepted) {
            return Err("WM023手牌选择续体尚未通过守卫".into());
        }
        if !self.valid_bound_target(f.actor, &f.source, &f.targets[0]) {
            return Ok(());
        }
        let options = self.wm023_options(&f);
        if options.is_empty() {
            return Ok(());
        }
        self.choice(
            f.actor,
            "wm023-hand-entry",
            TITLE.into(),
            options,
            1,
            1,
            Some(1),
            ChoiceResolution::Frame {
                frame: Box::new(f),
                choice: FrameChoice::WM023HandEntry,
            },
        );
        Ok(())
    }
    pub(crate) fn wm023_complete(
        &mut self,
        f: &ResolutionFrame,
        chooser: usize,
        selected: &[String],
    ) -> RuleResult<()> {
        self.validate_wm023_frame(f)?;
        if chooser != f.actor
            || selected.len() != 1
            || f.cursor != 1
            || !matches!(f.guard, GuardState::Accepted)
            || !self.valid_bound_target(f.actor, &f.source, &f.targets[0])
            || !self.wm023_options(f).iter().any(|o| o.id == selected[0])
        {
            return Err("WM023选择者、手牌或原目标地区无效".into());
        }
        let r = f.targets[0]
            .id
            .strip_prefix("region:")
            .unwrap()
            .parse::<usize>()
            .unwrap();
        let c = self.remove_hand(f.actor, &selected[0])?;
        let mut c = self.reset_zone_card(c);
        c.controller = f.actor;
        let id = c.id.clone();
        let name = catalog::card(&c.definition).name.clone();
        let def = c.definition.clone();
        self.regions[r].cards.push(c);
        self.enter_triggers(f.actor, &def, &id, false);
        self.note(format!(
            "{} 将手中的{}放置进场至地区{}",
            self.players[f.actor].name,
            name,
            r + 1
        ));
        Ok(())
    }
    pub(crate) fn validate_wm023_state(&self) -> RuleResult<()> {
        let stored = |f: &ResolutionFrame| -> RuleResult<()> {
            self.validate_wm023_frame(f)?;
            if related(f)
                && f.ability_key == KEY
                && (f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked))
            {
                return Err("WM023堆栈不能保存已执行续体".into());
            }
            Ok(())
        };
        for s in &self.stack {
            if let Some(f) = &s.frame {
                if self.validated_cross_card_frame(f)? { continue; }
                stored(f)?;
                if related(f) || s.card.as_ref().is_some_and(|c| c.definition == "WM023") {
                    if f.source.card.definition != "WM023"
                        || s.id != f.frame_id
                        || s.controller != f.actor
                    {
                        return Err("WM023堆栈来源和冻结帧不一致".into());
                    }
                    if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
                        if s.card.as_ref().is_none_or(|c| {
                            c.definition != "WM023"
                                || c.owner != f.source.card.owner
                                || c.controller != f.actor
                                || c.face_down
                        }) || s.deploy_region != f.source.region
                            || s.reveal != (f.ability_key == "reveal")
                            || s.target.is_some()
                        {
                            return Err("WM023派遣堆栈外壳无效".into());
                        }
                    } else if s.card.is_some()
                        || s.deploy_region.is_some()
                        || s.reveal
                        || s.target != f.targets.first().map(|t| t.id.clone())
                    {
                        return Err("WM023行动堆栈外壳无效".into());
                    }
                }
            } else if s.card.as_ref().is_some_and(|c| c.definition == "WM023") {
                return Err("WM023堆栈缺少冻结帧".into());
            }
        }
        let decl = |d: &Declaration| -> RuleResult<()> {
            if self.validated_cross_card_declaration(d)? { return Ok(()); }
            if relevant(&d.ability)
                || (d.source.card.definition == "WM023"
                    && !crate::jz22::runtime_ability_is_admitted(&d.ability))
            {
                return Err("WM023标准行动不能伪装为触发声明".into());
            }
            Ok(())
        };
        for e in &self.effects {
            match e {
                Effect::Frame { frame } if crate::mobility::related_frame(frame) => return Err("机动不能绕过响应队列".into()),
                Effect::Frame { frame: f } => {
                    stored(f)?;
                    if related(f) && f.ability_key == KEY {
                        return Err("WM023行动不能绕过响应队列".into());
                    }
                }
                Effect::Declare { declaration: d } => decl(d)?,
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Declare { declaration, .. } if self.validated_cross_card_declaration(declaration)? => {
                    self.validate_mobility_state()?;
                }
                ChoiceResolution::Frame { frame, .. } if crate::mobility::related_frame(frame) => return Err("机动没有帧内续体".into()),
                ChoiceResolution::Declare { declaration: d, .. } => decl(d)?,
                ChoiceResolution::Frame { frame: f, choice }
                    if related(f) || matches!(choice, FrameChoice::WM023HandEntry) =>
                {
                    if f.source.card.definition != "WM023" || f.ability_key != KEY {
                        return Err("WM023待选不能移植至其他来源或程序".into());
                    }
                    self.validate_wm023_frame(f)?;
                    if !matches!(choice, FrameChoice::WM023HandEntry)
                        || f.cursor != 1
                        || !matches!(f.guard, GuardState::Accepted)
                        || !self.valid_bound_target(f.actor, &f.source, &f.targets[0])
                    {
                        return Err("WM023原目标或私有手牌续体无效".into());
                    }
                    let expected = self.wm023_options(f);
                    if expected.is_empty()
                        || p.seat != f.actor
                        || p.choice.id.is_empty()
                        || p.choice.player_id != player_id(f.actor)
                        || p.choice.kind != "wm023-hand-entry"
                        || p.choice.title != TITLE
                        || p.choice.description != "选择符合数量限制的选项，然后确认。"
                        || p.choice.min != Some(1)
                        || p.choice.max != Some(1)
                        || p.choice.amount != Some(1)
                        || p.choice.allow_decline != Some(false)
                        || !p.choice.preview_cards.is_empty()
                        || !same(&p.choice.options, &expected)
                    {
                        return Err("WM023私有手牌选项或待选元数据无效".into());
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
