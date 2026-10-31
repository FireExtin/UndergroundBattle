//! Original BQ058 non-task slice. Entry task search is deliberately absent;
//! the card remains closed and must never be described as fully implemented.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub(crate) const ID: &str = "BQ058";
pub(crate) const KEY: &str = "bq058-core-evacuate";
const TITLE: &str = "控制小组：将本方核心移至相邻地区";
fn same<T: Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Grant {
    pub observer: SourceSnapshot,
    pub recipient: Card,
    pub turn: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Win {
    pub core: SourceSnapshot,
    pub region: usize,
    pub region_instance: String,
    pub event_id: String,
    pub turn: u32,
}
pub(crate) fn definition() -> Definition {
    let mut d = with_abilities(vec![ability(
        KEY,
        "赢得地区：移动本方核心",
        Timing::Fast,
        vec![],
        vec![],
        vec![Op::BQ058Evacuate { win: None }],
        Some(Event::BQ058RegionWon),
    )]);
    d.modifiers.push(StaticModifier::BQ058CoreSupport);
    d
}
pub(crate) fn related(a: &AbilitySpec) -> bool {
    a.key.starts_with("bq058-") || a.ops.iter().any(|o| matches!(o, Op::BQ058Evacuate { .. }))
}
fn frame_related(f: &ResolutionFrame) -> bool {
    f.ability_key.starts_with("bq058-")
        || f.steps
            .iter()
            .any(|s| matches!(s.op, Op::BQ058Evacuate { .. }))
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    let mut expected = definition().abilities.remove(0);
    if let [Op::BQ058Evacuate { win }] = a.ops.as_slice() {
        expected.ops = vec![Op::BQ058Evacuate { win: win.clone() }];
    }
    if id != ID || !same(a, &expected) {
        return Err("BQ058仅准入原赢区核心移动程序".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if (id == ID
        || d.abilities.iter().any(related)
        || d.modifiers
            .iter()
            .any(|m| matches!(m, StaticModifier::BQ058CoreSupport)))
        && (id != ID || !same(d, &definition()))
    {
        return Err("BQ058非任务定义不能增删或移植".into());
    }
    Ok(())
}
pub(crate) fn mobility_ability(p: Grant) -> AbilitySpec {
    let mut a = crate::mobility::ability();
    a.ops = vec![Op::BQ058Move { proof: Box::new(p) }];
    a
}
pub(crate) fn mobility_proof(a: &AbilitySpec) -> Option<&Grant> {
    if let [Op::BQ058Move { proof }] = a.ops.as_slice() {
        Some(proof)
    } else {
        None
    }
}
pub(crate) fn canonical_mobility(a: &AbilitySpec) -> bool {
    mobility_proof(a).is_some_and(|p| same(a, &mobility_ability(p.clone())))
}
impl Game {
    fn bq058_recipient(&self, c: &Card) -> bool {
        !c.face_down
            && c.owner == c.controller
            && self.selected_core_card(c)
            && self
                .board(&c.id)
                .is_some_and(|(r, live)| self.region_live(r) && live == c)
    }
    fn bq058_observers(&self, actor: usize) -> Vec<SourceSnapshot> {
        self.regions
            .iter()
            .enumerate()
            .filter(|(_, r)| !r.vacant)
            .flat_map(|(r, region)| region.cards.iter().map(move |c| (r, c)))
            .filter(|(_, c)| c.definition == ID && !c.face_down && c.controller == actor)
            .map(|(r, c)| self.source_snapshot(c, Some(r)))
            .collect()
    }
    pub(crate) fn bq058_bonus(&self, c: &Card, region: usize) -> u32 {
        if self.bq058_recipient(c) && self.board(&c.id).is_some_and(|(r, _)| r == region) {
            self.bq058_observers(c.controller).len() as u32
        } else {
            0
        }
    }
    pub(crate) fn bq058_mobility(&self, c: &Card) -> Option<Grant> {
        if !self.bq058_recipient(c) {
            return None;
        }
        self.bq058_observers(c.controller)
            .into_iter()
            .next()
            .map(|observer| Grant {
                observer,
                recipient: c.clone(),
                turn: self.turn,
            })
    }
    pub(crate) fn bq058_event_ability(
        &self,
        actor: usize,
        s: &SourceSnapshot,
    ) -> Option<AbilitySpec> {
        if self.validate_mobility_source(actor, s).is_ok()
            || self.bq049_event_ability(actor, s).is_some()
            || self.xg12_event_ability(actor, s).is_some()
        {
            return None;
        }
        self.bq058_mobility(&s.card).map(mobility_ability)
    }
    fn bq058_snapshot(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        if actor >= self.players.len()
            || s.card.controller != actor
            || s.card.owner >= self.players.len()
            || s.card.face_down
            || !self.mystic_instance_valid(&s.card.id)
            || s.region.is_none_or(|r| r >= self.regions.len())
            || s.source_region_instance
                .as_deref()
                .is_none_or(|id| !self.mystic_instance_valid(id))
            || s.play_source.is_some()
            || s.attachment_host_instance.is_some()
            || s.death_echo.is_some()
            || s.observed_death.is_some()
            || s.observed_exposure.is_some()
            || s.observed_mobility_grant.is_some()
        {
            return Err("BQ058原明置在场观察身份无效".into());
        }
        self.mobility_immutable_identity(&s.card)
    }
    pub(crate) fn validate_bq058_mobility_source(
        &self,
        actor: usize,
        s: &SourceSnapshot,
        p: &Grant,
    ) -> RuleResult<()> {
        self.validate_mobility_recipient(actor, s)?;
        self.bq058_snapshot(actor, &p.observer)?;
        if p.turn != self.turn
            || p.observer.card.definition != ID
            || p.recipient.owner != actor
            || p.recipient.controller != actor
            || p.recipient.face_down
            || !crate::core_zone::is_core(&p.recipient.definition)
            || !self
                .core_zones
                .iter()
                .any(|z| z.owner == actor && z.definition == p.recipient.definition)
            || s.card.id != p.recipient.id
            || s.card.owner != p.recipient.owner
            || s.card.definition != p.recipient.definition
            || s.card.generated != p.recipient.generated
            || s.observed_exposure.is_some()
            || s.observed_mobility_grant.is_some()
        {
            return Err("BQ058机动原核心接收者或回合无效".into());
        }
        self.mobility_immutable_identity(&p.recipient)
    }
    // Called at the existing Win-window close, before the return batch is frozen.
    pub(crate) fn observe_bq058_win(&mut self, seat: usize, region: usize) {
        let Some(core) = self.regions[region]
            .cards
            .iter()
            .find(|c| c.owner == seat && self.bq058_recipient(c))
            .cloned()
        else {
            return;
        };
        let observers = self.bq058_observers(seat);
        if observers.is_empty() {
            return;
        }
        let win = Win {
            core: self.source_snapshot(&core, Some(region)),
            region,
            region_instance: self.regions[region].card.id.clone(),
            event_id: self.id(),
            turn: self.turn,
        };
        for source in observers {
            let mut ability = definition().abilities.remove(0);
            ability.ops = vec![Op::BQ058Evacuate {
                win: Some(Box::new(win.clone())),
            }];
            self.effects.push_back(Effect::Declare {
                declaration: Declaration {
                    actor: seat,
                    source,
                    ability,
                },
            });
        }
    }
    fn bq058_win<'a>(
        &self,
        actor: usize,
        source: &SourceSnapshot,
        ops: &'a [Op],
    ) -> RuleResult<&'a Win> {
        let [Op::BQ058Evacuate { win: Some(w) }] = ops else {
            return Err("BQ058缺少原赢区观察".into());
        };
        self.bq058_snapshot(actor, source)?;
        self.bq058_snapshot(actor, &w.core)?;
        if source.card.definition != ID || w.turn != self.turn || !self.mystic_instance_valid(&w.event_id)
            || w.core.card.owner != actor || !crate::core_zone::is_core(&w.core.card.definition)
            || !self.core_zones.iter().any(|z| z.owner == actor && z.definition == w.core.card.definition)
            || w.core.region != Some(w.region) || w.core.source_region_instance.as_deref() != Some(&w.region_instance)
            || !self.region_live(w.region) || self.regions[w.region].card.id != w.region_instance
            || self.region_return.is_some() || !self.effects.iter().any(|e| matches!(e, Effect::PrepareRegionReturn { region } if *region == w.region))
            || !self.effects.iter().any(|e| matches!(e, Effect::Score { seat,region } if *seat == actor && *region == w.region))
            || [source.card.id.as_str(),w.core.card.id.as_str(),w.event_id.as_str()].into_iter().collect::<BTreeSet<_>>().len() != 3 { return Err("BQ058赢区观察不匹配原清场事务".into()); }
        Ok(w)
    }
    pub(crate) fn validate_bq058_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if related(&d.ability) {
            validate_ability(&d.source.card.definition, &d.ability)?;
            self.bq058_win(d.actor, &d.source, &d.ability.ops)?;
        }
        Ok(())
    }
    pub(crate) fn validate_bq058_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if !frame_related(f) {
            return Ok(());
        }
        self.bq058_win(
            f.actor,
            &f.source,
            &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>(),
        )?;
        if f.ability_key != KEY
            || !self.mystic_instance_valid(&f.frame_id)
            || f.cursor > 1
            || !f.targets.is_empty()
            || f.chosen_region.is_some()
            || !f.already_paid.is_empty()
            || !f.resource_payments.is_empty()
            || f.steps.iter().any(|s| s.context != f.actor)
            || f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked)
            || f.cursor == 1 && !matches!(f.guard, GuardState::Accepted)
        {
            return Err("BQ058赢区帧程序或执行阶段损坏".into());
        }
        Ok(())
    }
    fn bq058_destinations(&self, actor: usize, w: &Win) -> Vec<ChoiceOption> {
        let Some((r, c)) = self.board(&w.core.card.id) else {
            return vec![];
        };
        if r != w.region
            || c.owner != actor
            || !self.bq058_recipient(c)
            || self.players[actor].eliminated
        {
            return vec![];
        }
        (0..self.regions.len())
            .filter(|&to| {
                self.region_live(to)
                    && self.regions_adjacent(r, to)
                    && !self.jc064_blocks_character(c, to)
            })
            .map(|to| ChoiceOption {
                id: format!("region:{to}"),
                label: catalog::card(&self.regions[to].card.definition)
                    .name
                    .clone(),
                card: None,
            })
            .collect()
    }
    pub(crate) fn bq058_evacuate(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        self.validate_bq058_frame(&f)?;
        if f.cursor != 1 {
            return Err("BQ058移动阶段无效".into());
        }
        let ops = f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>();
        let w = self.bq058_win(f.actor, &f.source, &ops)?;
        let options = self.bq058_destinations(f.actor, w);
        if !options.is_empty() {
            self.choice(
                f.actor,
                "bq058-destination",
                TITLE.into(),
                options,
                0,
                1,
                None,
                ChoiceResolution::Frame {
                    frame: Box::new(f),
                    choice: FrameChoice::BQ058Destination,
                },
            );
        }
        Ok(())
    }
    pub(crate) fn bq058_finish(
        &mut self,
        f: &ResolutionFrame,
        seat: usize,
        a: &Action,
        ids: &[String],
    ) -> RuleResult<()> {
        self.validate_bq058_frame(f)?;
        let ops = f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>();
        let w = self.bq058_win(f.actor, &f.source, &ops)?;
        if f.cursor != 1
            || seat != f.actor
            || ids.len() > 1
            || *a
                != (Action {
                    choice_id: a.choice_id.clone(),
                    selected: Some(ids.to_vec()),
                    ..Action::new("choose")
                })
            || ids
                .first()
                .is_some_and(|id| !self.bq058_destinations(seat, w).iter().any(|o| &o.id == id))
        {
            return Err("BQ058只接受原核心的当前相邻地区或放弃".into());
        }
        if let Some(id) = ids.first() {
            let to = id
                .strip_prefix("region:")
                .and_then(|x| x.parse().ok())
                .ok_or("BQ058地区无效")?;
            let core = w.core.card.id.clone();
            self.move_on_board_instance(&core, to);
        }
        Ok(())
    }
    pub(crate) fn validate_bq058_state(&self) -> RuleResult<()> {
        let mut seen = BTreeSet::new();
        let mut check = |actor: usize, s: &SourceSnapshot, ops: Vec<Op>| -> RuleResult<()> {
            if let [Op::BQ058Evacuate { win: Some(w) }] = ops.as_slice() {
                self.bq058_win(actor, s, &ops)?;
                if !seen.insert((w.event_id.clone(), s.card.id.clone())) {
                    return Err("BQ058同一赢区观察重复续体".into());
                }
            }
            Ok(())
        };
        for e in &self.effects {
            match e {
                Effect::Declare { declaration: d } => {
                    self.validate_bq058_declaration(d)?;
                    check(d.actor, &d.source, d.ability.ops.clone())?;
                }
                Effect::Frame { frame: f } => {
                    self.validate_bq058_frame(f)?;
                    check(
                        f.actor,
                        &f.source,
                        f.steps.iter().map(|s| s.op.clone()).collect(),
                    )?;
                }
                _ => {}
            }
        }
        for item in &self.stack {
            if let Some(f) = &item.frame {
                self.validate_bq058_frame(f)?;
                check(
                    f.actor,
                    &f.source,
                    f.steps.iter().map(|s| s.op.clone()).collect(),
                )?;
                if frame_related(f)
                    && (f.cursor != 0
                        || item.id != f.frame_id
                        || item.controller != f.actor
                        || item.card.is_some()
                        || item.deploy_region.is_some()
                        || item.reveal
                        || item.target.is_some())
                {
                    return Err("BQ058赢区响应外壳损坏".into());
                }
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Declare {
                    declaration: d,
                    stage,
                } if related(&d.ability) => {
                    self.validate_bq058_declaration(d)?;
                    check(d.actor, &d.source, d.ability.ops.clone())?;
                    let expected = vec![ChoiceOption {
                        id: "accept".into(),
                        label: "发动触发能力".into(),
                        card: None,
                    }];
                    if !matches!(stage, DeclareChoice::Accept)
                        || p.seat != d.actor
                        || p.choice.kind != "trigger"
                        || p.choice.title
                            != format!(
                                "{}：{}",
                                crate::batch3_trap::source_label(&d.source),
                                d.ability.label
                            )
                        || !same(&p.choice.options, &expected)
                    {
                        return Err("BQ058赢区触发待选损坏".into());
                    }
                    self.bq058_choice_shape(p, d.actor)?;
                }
                ChoiceResolution::Frame { frame: f, choice }
                    if frame_related(f) || matches!(choice, FrameChoice::BQ058Destination) =>
                {
                    self.validate_bq058_frame(f)?;
                    check(
                        f.actor,
                        &f.source,
                        f.steps.iter().map(|s| s.op.clone()).collect(),
                    )?;
                    let ops = f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>();
                    let w = self.bq058_win(f.actor, &f.source, &ops)?;
                    let expected = self.bq058_destinations(f.actor, w);
                    if !frame_related(f)
                        || f.cursor != 1
                        || !matches!(choice, FrameChoice::BQ058Destination)
                        || p.seat != f.actor
                        || p.choice.kind != "bq058-destination"
                        || p.choice.title != TITLE
                        || expected.is_empty()
                        || !same(&p.choice.options, &expected)
                    {
                        return Err("BQ058相邻地区待选损坏".into());
                    }
                    self.bq058_choice_shape(p, f.actor)?;
                }
                _ => {}
            }
        }
        Ok(())
    }
    fn bq058_choice_shape(&self, p: &Pending, actor: usize) -> RuleResult<()> {
        if p.choice.player_id != player_id(actor)
            || !self.mystic_instance_valid(&p.choice.id)
            || p.choice.min != Some(0)
            || p.choice.max != Some(1)
            || p.choice.allow_decline != Some(true)
            || p.choice.amount.is_some()
            || !p.choice.preview_cards.is_empty()
        {
            return Err("BQ058可选移动数量或玩家损坏".into());
        }
        Ok(())
    }
}
