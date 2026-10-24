//! WM044 native candidate, including entry-lock through the full MSJC14 society.
use crate::{engine::RuleResult, model::*, rules::*, mystic_contest::ContestOutcome};
use serde::Serialize;
use std::collections::BTreeSet;

pub(crate) const INSPECT: &str = "wm044-win-inspect";
pub(crate) const LOCK: &str = "wm044-sentinel-entry-lock";
const KIND: &str = "wm044-private-inspection";
const TITLE: &str = "情报分析中心：检视所选暗藏者或秘密计划";
fn same<T: Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }

pub(crate) fn definition() -> Definition {
    let mut d = attachment_definition(target(Zone::Society, EntityKind::Any, Relation::ControlledByActor, Range::Anywhere));
    let mut inspection = target(Zone::Board, EntityKind::Any, Relation::Any, Range::Anywhere);
    inspection.predicate = Some(TargetPredicate::WM044HiddenOrSecretPlan);
    d.abilities.push(ability(INSPECT, "赢得隐秘对抗：检视目标暗藏者或秘密计划", Timing::Fast,
        vec![], vec![inspection], vec![Op::WM044Inspect { contest_id: None }], Some(Event::MysticContestWon)));
    d.abilities.push(ability(LOCK, "进场：若本方秘社为哨兵，在目标暗藏者上放置锁定1", Timing::Fast,
        vec![], vec![target(Zone::Board, EntityKind::Hidden, Relation::Any, Range::Anywhere)],
        vec![Op::WM044SentinelLock], Some(Event::Enter)));
    d
}
fn owns(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::WM044Inspect { .. } | Op::WM044SentinelLock => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => owns(body),
        Op::IfTargetExhausted { exhausted, ready, .. } => owns(std::slice::from_ref(exhausted)) || owns(std::slice::from_ref(ready)),
        _ => false,
    })
}
pub(crate) fn related(a: &AbilitySpec) -> bool {
    a.key.starts_with("wm044-") || owns(&a.ops) || a.modes.iter().any(|m| owns(&m.ops))
        || a.targets.iter().any(|t| t.predicate == Some(TargetPredicate::WM044HiddenOrSecretPlan))
}
pub(crate) fn frame_related(f: &ResolutionFrame) -> bool {
    f.ability_key.starts_with("wm044-") || f.steps.iter().any(|s| owns(std::slice::from_ref(&s.op)))
}
fn canonical(a: &AbilitySpec) -> Option<AbilitySpec> {
    let mut expected = definition().abilities.into_iter().find(|b| b.key == a.key)?;
    if let [Op::WM044Inspect { contest_id }] = a.ops.as_slice() {
        if a.key == INSPECT { expected.ops = vec![Op::WM044Inspect { contest_id: contest_id.clone() }]; }
    }
    Some(expected)
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if id != "WM044" || canonical(a).is_none_or(|expected| !same(a, &expected)) {
        return Err("情报分析中心须保留原卡完整触发程序".into());
    }
    Ok(())
}

impl Game {
    /// Call only when a concealed character actually enters the board. This
    /// requirement does not reveal its print, emit printed entry triggers, or
    /// repeatedly exhaust an existing hidden card on movement/ready/hide.
    pub(crate) fn apply_hidden_entry_slow(&self, entrant: &mut Card) {
        if entrant.face_down && self.attachments.iter().any(|a| {
            a.card.definition == "WM044"
                && !a.card.face_down
                && self.players.get(a.card.controller).is_some_and(|p| !p.eliminated)
                && self.attachment_host_valid(a)
        }) {
            entrant.exhausted = true;
        }
    }
}

impl Game {
    // Exact source identity, never a generic gray society or a subtype alias.
    // Do not insert an incomplete MSJC14 definition merely to make this true.
    pub(crate) fn wm044_has_sentinel(&self, actor: usize) -> bool {
        self.players.get(actor).is_some_and(|p| !p.eliminated && p.society_zone.card.as_ref()
            .is_some_and(|c| c.definition == "MSJC14" && !c.face_down && c.controller == actor))
    }
    pub(crate) fn wm044_trigger_available(&self, d: &Declaration) -> bool {
        d.source.card.definition != "WM044" || d.ability.key != LOCK || self.wm044_has_sentinel(d.actor)
    }
    pub(crate) fn wm044_observe_win(&mut self, o: &ContestOutcome) {
        let Some(team) = o.winner_team else { return; };
        let sources = self.attachments.iter().filter(|a| a.card.definition == "WM044" && !a.card.face_down
            && self.attachment_host_valid(a) && !self.players[a.card.controller].eliminated
            && self.team(a.card.controller) == team && o.participants.contains(&a.card.controller))
            .map(|a| self.source_snapshot(&a.card, None)).collect::<Vec<_>>();
        for source in sources {
            let mut a = definition().abilities.remove(1);
            a.ops = vec![Op::WM044Inspect { contest_id: Some(o.contest_id.clone()) }];
            self.effects.push_back(Effect::Declare { declaration: Declaration { actor: source.card.controller, source, ability: a } });
        }
    }
    fn wm044_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        if s.card.definition != "WM044" || s.card.face_down || actor >= self.players.len()
            || s.card.owner >= self.players.len() || s.card.controller != actor
            || s.region.is_some() || s.source_region_instance.is_some() || s.play_source.is_some()
            || s.attachment_host_instance.as_deref() != Some(format!("society:p{actor}").as_str()) {
            return Err("情报分析中心触发须绑定原操控者与原秘社宿主".into());
        }
        self.mobility_immutable_identity(&s.card)
    }
    fn wm044_win_record(&self, actor: usize, ops: &[Op]) -> RuleResult<()> {
        if let [Op::WM044Inspect { contest_id }] = ops {
            let o = contest_id.as_ref().and_then(|id| self.mystic_history.iter().find(|o| o.contest_id == *id))
                .ok_or("情报分析中心检视须来自真实隐秘获胜")?;
            if o.turn != self.turn || !o.participants.contains(&actor) || o.winner_team != Some(self.team(actor)) {
                return Err("情报分析中心检视属于实际参与并获胜的原玩家".into());
            }
            self.validate_mystic_outcome(o)?;
        }
        Ok(())
    }
    pub(crate) fn validate_wm044_declaration(&self, d: &Declaration) -> RuleResult<()> {
        self.wm044_source(d.actor, &d.source)?;
        validate_ability(&d.source.card.definition, &d.ability)?;
        if d.ability.event.is_none() { return Err("情报分析中心声明须为触发能力".into()); }
        self.wm044_win_record(d.actor, &d.ability.ops)
    }
    pub(crate) fn validate_wm044_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        self.wm044_source(f.actor, &f.source)?;
        let mut a = definition().abilities.into_iter().find(|a| a.key == f.ability_key)
            .ok_or("情报分析中心触发键不符")?;
        if f.ability_key == INSPECT {
            let [Step { op: Op::WM044Inspect { contest_id }, .. }] = f.steps.as_slice() else { return Err("检视缺少真实获胜续体".into()); };
            a.ops = vec![Op::WM044Inspect { contest_id: contest_id.clone() }];
        }
        if !same(&f.steps, &a.ops.iter().map(|op| Step { context: f.actor, op: op.clone() }).collect::<Vec<_>>())
            || f.cursor > 1 || f.chosen_region.is_some() || !f.already_paid.is_empty()
            || f.targets.len() != 1 || !same(&f.targets[0].spec, &a.targets[0])
            || (f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked))
            || (f.cursor == 1 && !matches!(f.guard, GuardState::Accepted)) {
            return Err("情报分析中心触发须保留唯一原目标及完整程序".into());
        }
        self.wm044_win_record(f.actor, &a.ops)
    }
    pub(crate) fn wm044_lock(&mut self, f: &ResolutionFrame) {
        if self.wm044_has_sentinel(f.actor) {
            if let Some(c) = f.targets.first().and_then(|t| self.board_mut(&t.id)).filter(|c| c.face_down) {
                c.lock_markers = c.lock_markers.saturating_add(1);
            }
        }
    }
    pub(crate) fn wm044_inspect_start(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        self.validate_wm044_frame(&f)?;
        let c = self.in_play(&f.targets[0].id).map(|(_, c)| c.clone()).ok_or("检视原目标已离场")?;
        if !crate::batch4_plans::is_plan(&c) { self.observe_hidden_inspection(f.actor, &c.id); }
        self.choice(f.actor, KIND, TITLE.into(), vec![], 0, 0, None,
            ChoiceResolution::Frame { frame: Box::new(f), choice: FrameChoice::WM044Inspection { inspected: c } });
        self.pending.as_mut().unwrap().choice.allow_decline = Some(false);
        Ok(())
    }
    pub(crate) fn wm044_inspect_complete(&self, f: &ResolutionFrame, actor: usize, a: &Action, selected: &[String], c: &Card) -> RuleResult<()> {
        self.validate_wm044_frame(f)?;
        if actor != f.actor || f.cursor != 1 || !selected.is_empty() || c.id != f.targets[0].id
            || *a != (Action { choice_id: a.choice_id.clone(), selected: a.selected.clone(), ..Action::new("choose") }) {
            return Err("情报分析中心只能由原玩家确认检视".into());
        }
        Ok(())
    }
    pub(crate) fn wm044_private_choice(&self, p: &Pending, viewer: usize) -> Choice {
        let mut choice = p.choice.clone();
        if let ChoiceResolution::Frame { frame, choice: FrameChoice::WM044Inspection { inspected } | FrameChoice::WM057Inspection { inspected } } = &p.resolution {
            if viewer == frame.actor {
                let mut front = inspected.clone(); front.face_down = false;
                choice.preview_cards = vec![self.card_view(&front, viewer, None, None)];
            }
        }
        choice
    }
    pub(crate) fn validate_wm044_state(&self) -> RuleResult<()> {
        let mut seen = BTreeSet::new();
        let mut remember = |s: &SourceSnapshot, ops: &[Op]| -> RuleResult<()> {
            if let [Op::WM044Inspect { contest_id: Some(id) }] = ops {
                if !seen.insert((s.card.id.clone(), id.clone())) { return Err("同一情报中心不能重复处理同次获胜".into()); }
            }
            Ok(())
        };
        for item in &self.stack { if let Some(f) = &item.frame {
            if frame_related(f) { self.validate_wm044_frame(f)?; remember(&f.source, &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>())?; }
        } }
        for e in &self.effects { match e {
            Effect::Declare { declaration: d } if related(&d.ability) => { self.validate_wm044_declaration(d)?; remember(&d.source, &d.ability.ops)?; },
            Effect::Frame { frame: f } if frame_related(f) => { self.validate_wm044_frame(f)?; remember(&f.source, &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>())?; },
            _ => (),
        } }
        if let Some(p) = &self.pending { match &p.resolution {
            ChoiceResolution::Declare { declaration: d, .. } if related(&d.ability) => { self.validate_wm044_declaration(d)?; remember(&d.source, &d.ability.ops)?; },
            ChoiceResolution::Frame { frame: f, choice: FrameChoice::WM044Inspection { inspected: c } } => {
                self.validate_wm044_frame(f)?;
                remember(&f.source, &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>())?;
                if f.ability_key != INSPECT || f.cursor != 1 || f.targets[0].id != c.id || !c.face_down
                    || self.in_play(&c.id).is_none_or(|(_, live)| live != c)
                    || p.seat != f.actor || p.choice.player_id != player_id(f.actor) || p.choice.kind != KIND || p.choice.title != TITLE
                    || p.choice.min != Some(0) || p.choice.max != Some(0) || p.choice.allow_decline != Some(false)
                    || !p.choice.options.is_empty() || !p.choice.preview_cards.is_empty() {
                    return Err("情报分析中心私密确认须保留原目标及原玩家".into());
                }
            },
            _ => (),
        } }
        Ok(())
    }
}
