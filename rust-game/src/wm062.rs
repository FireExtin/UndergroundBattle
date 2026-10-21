//! Complete WM062: a paid fast transaction, then a bounded successor receipt.
//! The optional move refers to the object actually produced by Hide, never a
//! name search or the retired character UID. Historical receipts are consistency
//! witnesses, not cryptographic proof against rewriting an entire game history.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use std::collections::BTreeSet;
const KEY: &str = "wm062-hide-then-adjacent-move";
const CHOICE: &str = "wm062-adjacent-move";
const TITLE: &str = "跑路：是否移动刚潜伏的暗藏者";
const DESCRIPTION: &str = "潜伏已经完成。可选择一个相邻地区移动，或不选择地区并确认，留在原地区。";
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
fn uses(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::WM062HideCriminalThenOptionalAdjacentMove => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => uses(body),
        Op::IfTargetExhausted {
            exhausted, ready, ..
        } => uses(std::slice::from_ref(exhausted)) || uses(std::slice::from_ref(ready)),
        _ => false,
    })
}
fn related(f: &ResolutionFrame) -> bool {
    f.source.card.definition == "WM062"
        || f.ability_key == KEY
        || f.steps.iter().any(|s| uses(std::slice::from_ref(&s.op)))
}
fn related_ability(a: &AbilitySpec) -> bool {
    a.key == KEY || uses(&a.ops) || a.modes.iter().any(|m| uses(&m.ops))
}
pub(crate) fn definition() -> Definition {
    let mut victim = target(
        Zone::Board,
        EntityKind::Character,
        Relation::ControlledByActor,
        Range::Anywhere,
    );
    victim.subtype = Some("罪犯".into());
    let mut play = ability(
        KEY,
        "快速行动：令本方罪犯潜伏，然后可移至相邻地区",
        Timing::Fast,
        vec![],
        vec![victim],
        vec![Op::WM062HideCriminalThenOptionalAdjacentMove],
        None,
    );
    play.play_only = true;
    with_abilities(vec![play])
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if (id == "WM062" || related_ability(a))
        && (id != "WM062" || !same(a, &definition().abilities[0]))
    {
        return Err("WM062只准入完整快速事务，不能拆分、改写或移植".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if (id == "WM062" && !same(d, &definition()))
        || (id != "WM062" && d.abilities.iter().any(related_ability))
    {
        return Err("WM062整卡定义不能拆分、改写或移植".into());
    }
    Ok(())
}
pub(crate) fn validate_action(id: &str, a: &Action) -> RuleResult<()> {
    if id == "WM062"
        && (a.kind != "play"
            || a.target_id.is_none()
            || a.region.is_some()
            || a.option.is_some()
            || a.choice_id.is_some()
            || a.selected.is_some()
            || a.top.is_some()
            || a.bottom.is_some()
            || a.allocations.is_some()
            || a.cost_selected.is_some()
            || a.deck_draft.is_some()
            || a.ability_id.as_deref().is_some_and(|key| key != KEY))
    {
        return Err("WM062须先指定一个本方罪犯，移动地区在潜伏后另选".into());
    }
    Ok(())
}
impl Game {
    pub(crate) fn validate_wm062_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if !related(f) {
            return Ok(());
        }
        let c = &f.source.card;
        let canonical = definition();
        let player = |id: &Option<String>| {
            id.as_deref()
                .and_then(|s| s.strip_prefix('p'))
                .and_then(|s| s.parse::<usize>().ok())
                .filter(|&seat| {
                    seat < self.players.len() && id.as_deref() == Some(player_id(seat).as_str())
                })
        };
        if c.definition != "WM062"
            || c.id.is_empty()
            || c.face_down
            || c.owner >= self.players.len()
            || c.controller != f.actor
            || f.actor >= self.players.len()
            || f.frame_id.is_empty()
            || f.source.region.is_some()
            || f.source.source_region_instance.is_some()
            || f.source.attachment_host_instance.is_some()
            || f.source.play_source.is_some()
            || f.source.observed_death.is_some()
            || f.source.observed_mobility_grant.is_some()
            || f.ability_key != KEY
            || f.chosen_region.is_some()
            || f.targets.len() != 1
            || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if ids.len() <= 2
                && ids.iter().all(|id| !id.is_empty()) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len())
            || !matches!(f.steps.as_slice(), [Step { context, op: Op::WM062HideCriminalThenOptionalAdjacentMove }] if *context == f.actor)
            || !((f.cursor == 0 && matches!(f.guard, GuardState::Unchecked))
                || (f.cursor == 1 && matches!(f.guard, GuardState::Accepted)))
        {
            return Err("WM062冻结来源、支付或完整续结算程序无效".into());
        }
        let t = &f.targets[0];
        if t.id.is_empty()
            || t.id == c.id
            || t.region_instance.is_some()
            || !same(&t.spec, &canonical.abilities[0].targets[0])
            || t.public.instance_id != t.id
            || t.public.kind != "character"
            || t.public.label.is_empty()
            || player(&t.public.owner).is_none()
            || player(&t.public.controller) != Some(f.actor)
            || t.public.region.is_none_or(|r| r >= self.regions.len())
            || !t.public.valid
            || t.public.status != "valid"
            || t.public.invalid_reason.is_some()
        {
            return Err("WM062唯一原角色目标或本方罪犯条件无效".into());
        }
        Ok(())
    }
    fn wm062_destinations(&self, region: usize) -> Vec<(usize, String)> {
        self.regions
            .iter()
            .enumerate()
            .filter(|(r, v)| !v.vacant && self.regions_adjacent(region, *r))
            .map(|(r, v)| (r, v.card.id.clone()))
            .collect()
    }
    fn wm062_options(&self, destinations: &[(usize, String)]) -> Vec<ChoiceOption> {
        destinations
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
    pub(crate) fn wm062_start(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        self.validate_wm062_frame(&f)?;
        let (region, before) = self
            .board(&f.targets[0].id)
            .map(|(r, c)| (r, c.clone()))
            .ok_or("WM062原目标已失效")?;
        let region_instance = self.regions[region].card.id.clone();
        let baseline = self
            .control_baselines
            .iter()
            .find(|b| b.target_instance == before.id)
            .cloned();
        let hidden = self
            .hide_in_place_successor(&before.id)
            .ok_or("WM062潜伏未完成")?;
        // Hide has completed before the optional second step is persisted.
        // Losing another criminal can make JZ48's existing damage lethal.
        // Use the shared cleanup; emitted triggers still wait for this effect.
        self.settle_deaths();
        self.note("跑路：目标已潜伏".into());
        let destinations = self.wm062_destinations(region);
        if destinations.is_empty() {
            return Ok(());
        }
        self.choice(
            f.actor,
            CHOICE,
            TITLE.into(),
            self.wm062_options(&destinations),
            0,
            1,
            None,
            ChoiceResolution::Frame {
                frame: Box::new(f),
                choice: FrameChoice::WM062AdjacentMove {
                    before,
                    hidden,
                    baseline,
                    region,
                    region_instance,
                    destinations,
                },
            },
        );
        self.pending.as_mut().unwrap().choice.description = DESCRIPTION.into();
        Ok(())
    }
    fn wm062_receipt(
        &self,
        f: &ResolutionFrame,
        before: &Card,
        hidden: &Card,
        baseline: Option<&ControlBaseline>,
        region: usize,
        region_instance: &str,
        destinations: &[(usize, String)],
    ) -> RuleResult<()> {
        self.validate_wm062_frame(f)?;
        let admitted = catalog::catalog().cards.iter().any(|d| {
            d.id == before.definition
                && catalog::runtime_implemented(d)
                && d.kind == "character"
                && d.subtypes.iter().any(|s| s == "罪犯")
        });
        if !related(f)
            || f.cursor != 1
            || !matches!(f.guard, GuardState::Accepted)
            || before.id != f.targets[0].id
            || before.face_down
            || !admitted
            || before.controller != f.actor
            || before.owner >= self.players.len()
            || f.targets[0].public.owner != Some(player_id(before.owner))
            || self.board(&before.id).is_some()
            || before.id == hidden.id
            || hidden.id.is_empty()
            || hidden.id == f.source.card.id
            || hidden.id == f.frame_id
            || !self.region_live(region)
            || self.regions[region].card.id != region_instance
            || self
                .board(&hidden.id)
                .is_none_or(|(r, c)| r != region || !same(c, hidden))
            || self
                .in_play_cards()
                .iter()
                .filter(|(_, c)| c.id == hidden.id)
                .count()
                != 1
            || baseline.is_some_and(|b| {
                b.target_instance != before.id || b.controller >= self.players.len()
            })
            || destinations.is_empty()
            || destinations != self.wm062_destinations(region).as_slice()
        {
            return Err("WM062潜伏前后实体、原地区或相邻地区见证无效".into());
        }
        let control = baseline.map_or(before.controller, |b| b.controller);
        let mut expected = before.clone();
        expected.id = hidden.id.clone();
        expected.controller = if self.players[control].eliminated {
            before.owner
        } else {
            control
        };
        expected.face_down = true;
        expected.time_markers = 0;
        expected.lock_markers = 0;
        expected.damage = 0;
        expected.wounds = 0;
        expected.shield = 0;
        if !same(&expected, hidden) {
            return Err("WM062所得暗藏者不等于原潜伏产生的新实体".into());
        }
        Ok(())
    }
    pub(crate) fn wm062_complete(
        &mut self,
        f: &ResolutionFrame,
        chooser: usize,
        a: &Action,
        selected: &[String],
        before: &Card,
        hidden: &Card,
        baseline: Option<&ControlBaseline>,
        region: usize,
        region_instance: &str,
        destinations: &[(usize, String)],
    ) -> RuleResult<()> {
        self.wm062_receipt(
            f,
            before,
            hidden,
            baseline,
            region,
            region_instance,
            destinations,
        )?;
        if chooser != f.actor
            || selected.len() > 1
            || a.card_id.is_some()
            || a.target_id.is_some()
            || a.ability_id.is_some()
            || a.region.is_some()
            || a.option.is_some()
            || a.top.is_some()
            || a.bottom.is_some()
            || a.allocations.is_some()
            || a.cost_selected.is_some()
            || a.deck_draft.is_some()
        {
            return Err("WM062移动选择者或续结算参数无效".into());
        }
        if let Some(id) = selected.first() {
            let dest = destinations
                .iter()
                .find(|(r, _)| id == &format!("region:{r}"))
                .ok_or("WM062只能选择一个有效相邻地区")?
                .0;
            self.move_on_board_instance(&hidden.id, dest);
            self.note(format!(
                "跑路：暗藏者从地区{}移至地区{}",
                region + 1,
                dest + 1
            ));
        } else {
            self.note("跑路：不移动，暗藏者留在原地区".into());
        }
        Ok(())
    }
    // A hand -> paid stack/Bury container keeps every Card field except UID
    // and markers reset by fresh(). Do not mutate state while checking it.
    fn wm062_container(&self, source: &Card, c: &Card) -> bool {
        let mut expected = source.clone();
        expected.id = c.id.clone();
        expected.time_markers = 0;
        expected.lock_markers = 0;
        !c.id.is_empty() && c.id != source.id && same(&expected, c)
    }
    pub(crate) fn validate_wm062_state(&self) -> RuleResult<()> {
        let mut frames = BTreeSet::new();
        for item in &self.stack {
            if let Some(f) = &item.frame {
                self.validate_wm062_frame(f)?;
                if related(f)
                    && (!frames.insert(&f.frame_id)
                        || item.id != f.frame_id
                        || item.controller != f.actor
                        || item.reveal
                        || item.deploy_region.is_some()
                        || item.target != f.targets.first().map(|t| t.id.clone())
                        || f.cursor != 0
                        || !matches!(f.guard, GuardState::Unchecked)
                        || item
                            .card
                            .as_ref()
                            .is_none_or(|c| !self.wm062_container(&f.source.card, c)))
                {
                    return Err("WM062已支付响应堆栈或事务容器无效".into());
                }
            } else if item.card.as_ref().is_some_and(|c| c.definition == "WM062") {
                return Err("WM062事务缺少已支付冻结帧".into());
            }
        }
        let declaration = |d: &Declaration| -> RuleResult<()> {
            if d.source.card.definition == "WM062" || related_ability(&d.ability) {
                return Err("WM062事务不能伪装成触发声明".into());
            }
            Ok(())
        };
        for effect in &self.effects {
            match effect {
                Effect::Frame { frame } if related(frame) => {
                    return Err("WM062不能绕过已支付响应堆栈或重放潜伏".into())
                }
                Effect::Declare { declaration: d } => declaration(d)?,
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Declare { declaration: d, .. } => declaration(d)?,
                ChoiceResolution::Frame { frame: f, choice }
                    if related(f) || matches!(choice, FrameChoice::WM062AdjacentMove { .. }) =>
                {
                    if !frames.insert(&f.frame_id) {
                        return Err("WM062不能重复冻结续结算".into());
                    }
                    let FrameChoice::WM062AdjacentMove {
                        before,
                        hidden,
                        baseline,
                        region,
                        region_instance,
                        destinations,
                    } = choice
                    else {
                        return Err("WM062不能使用其他待选程序".into());
                    };
                    self.wm062_receipt(
                        f,
                        before,
                        hidden,
                        baseline.as_ref(),
                        *region,
                        region_instance,
                        destinations,
                    )?;
                    let burial = self
                        .effects
                        .iter()
                        .filter_map(|e| match e {
                            Effect::Bury { card } if self.wm062_container(&f.source.card, card) => {
                                Some(card)
                            }
                            _ => None,
                        })
                        .count();
                    if burial != 1
                        || p.seat != f.actor
                        || p.choice.id.is_empty()
                        || p.choice.player_id != player_id(f.actor)
                        || p.choice.kind != CHOICE
                        || p.choice.title != TITLE
                        || p.choice.description != DESCRIPTION
                        || p.choice.min != Some(0)
                        || p.choice.max != Some(1)
                        || p.choice.amount.is_some()
                        || p.choice.allow_decline != Some(true)
                        || !p.choice.preview_cards.is_empty()
                        || !same(&p.choice.options, &self.wm062_options(destinations))
                    {
                        return Err("WM062可选移动的事务容器、选择者或选项无效".into());
                    }
                }
                _ => {}
            }
        }
        if self
            .in_play_cards()
            .iter()
            .any(|(_, c)| c.definition == "WM062")
        {
            return Err("WM062事务不能作为在场角色、暗藏者或附属".into());
        }
        Ok(())
    }
}
