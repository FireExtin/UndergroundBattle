//! BQ001, original image c2c6de0b1d71e37a913c9b2e2c358a0e28702037d6983430bc7bc10a9a1b78a1.
//! The purple +X is the hidden-contest value symbol, not a regional influence icon.
use crate::{catalog, engine::RuleResult, model::*, mystic_contest::MysticShown, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub(crate) const ID: &str = "BQ001";
pub(crate) const KEY: &str = "bq001-reveal-star-seal-draw";
pub(crate) const CHOICE: &str = "bq001-star-hand";
const TITLE: &str = "选择本人手中一张星辰领域牌封印在本方秘社上";
fn same<T: Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnchorShownWitness {
    pub sources: Vec<SourceSnapshot>,
    pub anchors: Vec<SealedCard>,
}
pub(crate) fn definition() -> Definition {
    let mut d = with_abilities(vec![ability(
        KEY,
        "现身：封印一张星辰手牌，然后抓一张牌",
        Timing::Fast,
        vec![],
        vec![],
        vec![
            Op::BQ001SealStarHandOnSociety,
            Op::Draw {
                player: PlayerRef::Actor,
                count: 1,
                end: DeckEnd::Top,
            },
        ],
        Some(Event::Reveal),
    )]);
    d.modifiers.push(StaticModifier::BQ001MysticAnchorShown);
    d
}
fn owns(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::BQ001SealStarHandOnSociety => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => owns(body),
        Op::IfTargetExhausted {
            exhausted, ready, ..
        } => owns(std::slice::from_ref(exhausted)) || owns(std::slice::from_ref(ready)),
        _ => false,
    })
}
pub(crate) fn related(a: &AbilitySpec) -> bool {
    a.key.starts_with("bq001-") || owns(&a.ops) || a.modes.iter().any(|m| owns(&m.ops))
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if related(a) && (id != ID || !same(a, &definition().abilities[0])) {
        return Err("BQ001只允许原现身封印与随后抓牌程序".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    let marker = d
        .modifiers
        .iter()
        .any(|m| matches!(m, StaticModifier::BQ001MysticAnchorShown));
    if (id == ID || marker || d.abilities.iter().any(related))
        && (id != ID || !same(d, &definition()))
    {
        return Err("BQ001持续加值和现身完整定义不能增删或移植".into());
    }
    Ok(())
}
pub(crate) fn validate_action(id: &str, a: &Action) -> RuleResult<()> {
    if id != ID {
        return Ok(());
    }
    let expected = match a.kind.as_str() {
        "deploy" | "conceal" => Action {
            card_id: a.card_id.clone(),
            region: a.region,
            ..Action::new(&a.kind)
        },
        "reveal" => Action {
            card_id: a.card_id.clone(),
            ..Action::new("reveal")
        },
        _ => return Err("BQ001只允许普通派遣、暗藏和现身".into()),
    };
    if *a != expected {
        return Err("BQ001角色交易不能夹带额外目标、费用、数量或选择".into());
    }
    Ok(())
}
fn frame_related(f: &ResolutionFrame) -> bool {
    f.source.card.definition == ID
        || f.ability_key.starts_with("bq001-")
        || f.steps.iter().any(|s| owns(std::slice::from_ref(&s.op)))
}
pub(crate) fn validate_choice_action(p: &Pending, a: &Action) -> RuleResult<()> {
    let relevant = match &p.resolution {
        ChoiceResolution::Declare { declaration, .. } => related(&declaration.ability),
        ChoiceResolution::Frame { frame, choice } => {
            frame_related(frame) || matches!(choice, FrameChoice::BQ001StarHand)
        }
        _ => false,
    };
    if relevant
        && *a
            != (Action {
                choice_id: a.choice_id.clone(),
                selected: a.selected.clone(),
                ..Action::new("choose")
            })
    {
        return Err("BQ001待选只能提交原选择，不能夹带数量、排序、目标或费用".into());
    }
    Ok(())
}
impl Game {
    fn bq001_source(&self, seat: usize, s: &SourceSnapshot, play: bool) -> RuleResult<()> {
        let c = &s.card;
        if c.definition != ID
            || seat >= self.players.len()
            || c.owner >= self.players.len()
            || c.controller != seat
            || !self.mystic_instance_valid(&c.id)
            || s.region.is_none_or(|r| r >= self.regions.len())
            || s.source_region_instance
                .as_deref()
                .is_none_or(|id| !self.mystic_instance_valid(id) || id == c.id)
            || (!play && (c.face_down || s.play_source.is_some()))
            || s.death_echo.is_some()
            || s.observed_death.is_some()
            || s.observed_exposure.is_some()
            || s.observed_mobility_grant.is_some()
            || s.attachment_host_instance.is_some()
        {
            return Err("BQ001冻结来源、执行者或原地区无效".into());
        }
        self.mobility_immutable_identity(c)
    }
    pub(crate) fn validate_bq001_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if !related(&d.ability) {
            return Ok(());
        }
        self.bq001_source(d.actor, &d.source, false)?;
        if !same(&d.ability, &definition().abilities[0]) {
            return Err("BQ001只允许原可选现身触发".into());
        }
        Ok(())
    }
    pub(crate) fn validate_bq001_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if !frame_related(f) {
            return Ok(());
        }
        if self.validated_cross_card_frame(f)? {
            return Ok(());
        }
        let play = matches!(f.ability_key.as_str(), "deploy" | "reveal");
        self.bq001_source(f.actor, &f.source, play)?;
        if !self
            .mystic_instance_number(&f.source.card.id)
            .zip(self.mystic_instance_number(&f.frame_id))
            .is_some_and(|(a, b)| a < b)
            || !f.targets.is_empty()
        {
            return Err("BQ001帧实例顺序或目标被改写".into());
        }
        if play {
            if f.cursor != 0
                || !matches!(f.guard, GuardState::Unchecked)
                || !f.steps.is_empty()
                || f.chosen_region != f.source.region
                || f.ability_key == "deploy"
                    && (f.source.card.face_down || !f.source.valid_character_play())
                || f.ability_key == "reveal"
                    && (!f.source.card.face_down || f.source.play_source.is_some())
                || !matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)] if f.money_units(0)<=1&&ids.iter().all(|id|f.source.valid_printed_payment_id(id))&&ids.iter().collect::<BTreeSet<_>>().len()==ids.len())
            {
                return Err("BQ001派遣或现身须保留普通费用收据和原来源".into());
            }
            return self.validate_resource_frame(f);
        }
        let expected = definition().abilities.remove(0).ops;
        if f.ability_key != KEY
            || f.chosen_region.is_some()
            || !f.already_paid.is_empty()
            || !f.resource_payments.is_empty()
            || f.steps.iter().any(|s| s.context != f.actor)
            || !same(
                &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>(),
                &expected,
            )
            || f.cursor > 2
            || f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked)
            || f.cursor > 0 && !matches!(f.guard, GuardState::Accepted)
        {
            return Err("BQ001原封印/抓牌步骤、守卫或游标被改写".into());
        }
        Ok(())
    }
    fn bq001_options(&self, seat: usize) -> Vec<ChoiceOption> {
        self.players
            .get(seat)
            .into_iter()
            .filter(|p| !p.eliminated)
            .flat_map(|p| &p.hand)
            .filter(|c| catalog::card(&c.definition).magic == "星辰")
            .map(|c| self.option(c, seat, None, None))
            .collect()
    }
    pub(crate) fn bq001_start(&mut self, frame: ResolutionFrame) -> RuleResult<()> {
        self.validate_bq001_frame(&frame)?;
        let seat = frame.actor;
        let options = self.bq001_options(seat);
        // This is an effect choice, not an additional cost or a declared target.
        // An empty matching hand does not discard the following printed draw step.
        if options.is_empty() {
            return self.resolve_frame(frame);
        }
        self.choice(
            seat,
            CHOICE,
            TITLE.into(),
            options,
            1,
            1,
            None,
            ChoiceResolution::Frame {
                frame: Box::new(frame),
                choice: FrameChoice::BQ001StarHand,
            },
        );
        Ok(())
    }
    pub(crate) fn bq001_complete(
        &mut self,
        frame: ResolutionFrame,
        seat: usize,
        a: &Action,
        selected: &[String],
    ) -> RuleResult<()> {
        self.validate_bq001_frame(&frame)?;
        if seat != frame.actor
            || frame.cursor != 1
            || !matches!(frame.guard, GuardState::Accepted)
            || selected.len() != 1
            || *a
                != (Action {
                    choice_id: a.choice_id.clone(),
                    selected: a.selected.clone(),
                    ..Action::new("choose")
                })
            || !self.bq001_options(seat).iter().any(|o| o.id == selected[0])
        {
            return Err("BQ001只能封印原执行者一张真实星辰手牌".into());
        }
        self.seal_hand_card(seat, &selected[0], &format!("society:p{seat}"))?;
        self.resolve_frame(frame)
    }
    pub(crate) fn bq001_shown_witness(&self, seat: usize) -> Option<AnchorShownWitness> {
        let sources = self
            .regions
            .iter()
            .enumerate()
            .filter(|(_, r)| !r.vacant)
            .flat_map(|(r, region)| {
                region
                    .cards
                    .iter()
                    .filter(move |c| c.definition == ID && !c.face_down && c.controller == seat)
                    .map(move |c| self.source_snapshot(c, Some(r)))
            })
            .collect::<Vec<_>>();
        if sources.is_empty() {
            return None;
        }
        let host = format!("society:p{seat}");
        let anchors = self
            .sealed_cards
            .iter()
            .filter(|s| s.host_id == host && catalog::card(&s.card.definition).name == "精神锚点")
            .cloned()
            .collect();
        Some(AnchorShownWitness { sources, anchors })
    }
    pub(crate) fn bq001_shown_bonus(&self, shown: &MysticShown) -> RuleResult<u32> {
        let Some(w) = &shown.anchor_witness else {
            return Ok(0);
        };
        if w.sources.is_empty() {
            return Err("BQ001展示加值必须有原正面操控来源".into());
        }
        let mut ids = BTreeSet::new();
        for s in &w.sources {
            self.bq001_source(shown.seat, s, false)?;
            if !ids.insert(&s.card.id) {
                return Err("BQ001展示来源重复".into());
            }
        }
        for s in &w.anchors {
            let c = &s.card;
            if s.host_id != format!("society:p{}", shown.seat)
                || catalog::card(&c.definition).name != "精神锚点"
                || !self.mystic_instance_valid(&c.id)
                || !ids.insert(&c.id)
                || c.owner >= self.players.len()
                || c.controller != c.owner
                || c.face_down
                || c.exhausted
                || c.damage != 0
                || c.wounds != 0
                || c.shield != 0
                || c.time_markers != 0
                || c.charge_markers != 0
                || c.advancement_markers != 0
                || c.lock_markers != 0
                || (c.permanent_combat_markers != 0 || c.permanent_investigation_markers != 0)
                || c.permanent_defense_markers != 0
                || c.named_character.is_some()
                || c.chosen_number.is_some()
                || c.chosen_subtype.is_some()
            {
                return Err("BQ001计数须保留原本人秘社实际锚点".into());
            }
            self.mobility_immutable_identity(c)?;
        }
        u32::try_from(w.sources.len())
            .ok()
            .zip(u32::try_from(w.anchors.len()).ok())
            .and_then(|(a, b)| a.checked_mul(b))
            .ok_or_else(|| "BQ001隐秘持续加值溢出".into())
    }
    fn validate_bq001_pending(&self, p: &Pending) -> RuleResult<()> {
        match &p.resolution {
            ChoiceResolution::Declare {
                declaration: d,
                stage,
            } if related(&d.ability) => {
                self.validate_bq001_declaration(d)?;
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
                    options: vec![ChoiceOption {
                        id: "accept".into(),
                        label: "发动触发能力".into(),
                        card: None,
                    }],
                    preview_cards: vec![],
                    min: Some(0),
                    max: Some(1),
                    amount: None,
                    allow_decline: Some(true),
                };
                if !matches!(stage, DeclareChoice::Accept)
                    || p.seat != d.actor
                    || !self.mystic_instance_valid(&p.choice.id)
                    || !same(&p.choice, &expected)
                {
                    return Err("BQ001现身触发待选被改写".into());
                }
            }
            ChoiceResolution::Frame { frame: f, choice }
                if frame_related(f) && !self.validated_cross_card_frame(f)?
                    || matches!(choice, FrameChoice::BQ001StarHand) =>
            {
                self.validate_bq001_frame(f)?;
                let options = self.bq001_options(f.actor);
                let expected = Choice {
                    id: p.choice.id.clone(),
                    kind: CHOICE.into(),
                    title: TITLE.into(),
                    description: "选择符合数量限制的选项，然后确认。".into(),
                    player_id: player_id(f.actor),
                    options,
                    preview_cards: vec![],
                    min: Some(1),
                    max: Some(1),
                    amount: None,
                    allow_decline: Some(false),
                };
                if f.ability_key != KEY
                    || !matches!(choice, FrameChoice::BQ001StarHand)
                    || f.cursor != 1
                    || !matches!(f.guard, GuardState::Accepted)
                    || p.seat != f.actor
                    || expected.options.is_empty()
                    || !self.mystic_instance_valid(&p.choice.id)
                    || !same(&p.choice, &expected)
                {
                    return Err("BQ001原本人星辰手牌选择、游标或数量被改写".into());
                }
            }
            _ => (),
        }
        Ok(())
    }
    pub(crate) fn validate_bq001_state(&self) -> RuleResult<()> {
        let mut ids = BTreeSet::new();
        for item in &self.stack {
            if let Some(f) = &item.frame {
                self.validate_bq001_frame(f)?;
                if frame_related(f) && !self.validated_cross_card_frame(f)? {
                    let play = matches!(f.ability_key.as_str(), "deploy" | "reveal");
                    if !ids.insert(&f.frame_id)
                        || item.id != f.frame_id
                        || item.controller != f.actor
                        || item.card.is_some() != play
                        || f.cursor != 0
                        || !matches!(f.guard, GuardState::Unchecked)
                        || item.target.is_some()
                        || item.deploy_region != if play { f.source.region } else { None }
                        || item.reveal != (f.ability_key == "reveal")
                        || item
                            .card
                            .as_ref()
                            .is_some_and(|c| !self.pack_a_transaction_matches(f, c))
                    {
                        return Err("BQ001响应外壳、普通支付或实例对应被改写".into());
                    }
                }
            } else if item.card.as_ref().is_some_and(|c| c.definition == ID) {
                return Err("BQ001出牌缺少原支付帧".into());
            }
        }
        for e in &self.effects {
            match e {
                Effect::Declare { declaration: d } => self.validate_bq001_declaration(d)?,
                Effect::Frame { frame: f }
                    if frame_related(f) && !self.validated_cross_card_frame(f)? =>
                {
                    return Err("BQ001不能脱离原响应/手牌待选直接续接效果".into())
                }
                Effect::DeathCaller { caller } => self.validate_bq001_frame(&caller.frame)?,
                _ => (),
            }
        }
        if let Some(p) = &self.pending {
            self.validate_bq001_pending(p)?;
        }
        for state in self.mystic_states() {
            self.validate_bq001_frame(&state.frame)?;
        }
        Ok(())
    }
}
