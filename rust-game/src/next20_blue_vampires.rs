//! Image-verified BQ033/BQ034 adapters. The existing interpreter owns payment,
//! targets, responses, contests and attachment entry. See the separate wiring
//! contract before registering these definitions; public admission stays closed.
use crate::{
    catalog,
    engine::RuleResult,
    model::*,
    mystic_contest::{ContestOutcome, MysticPhase},
    rules::*,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub(crate) const CARDS: &[&str] = &["BQ033", "BQ034"];
pub(crate) const DISCARD: &str = "next20-bq033-discard";
pub(crate) const ENTRY: &str = "next20-bq034-entry";
pub(crate) const SHOWN: &str = "next20-bq034-shown";
pub(crate) const COUNTER: &str = "next20-bq034-counter";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnchorPayment {
    pub turn: u32,
    pub payer: usize,
    pub anchor: Card,
    pub society_host: String,
    pub deck_instance: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VampireTargeted {
    pub turn: u32,
    pub transaction: Box<ResolutionFrame>,
    pub target: SourceSnapshot,
    pub subtypes: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum BlueProgram {
    RandomDiscard,
    EntryFeed,
    ShownFeed {
        contest_id: String,
        index: usize,
    },
    Counter {
        observation: Option<Box<VampireTargeted>>,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum BlueChoice {
    ShownTarget,
    Blood { recipient: String },
    SacrificeAttachment,
}
fn same<T: Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
fn op(p: BlueProgram) -> Op {
    Op::Next20Blue(p)
}
pub(crate) fn shown_bonus(id: &str) -> u32 {
    if id == "BQ034" {
        1
    } else {
        0
    }
}
pub(crate) fn related(a: &AbilitySpec) -> bool {
    fn owns(ops: &[Op]) -> bool {
        ops.iter().any(|o| match o {
            Op::Next20Blue(_) => true,
            Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => owns(body),
            Op::IfTargetExhausted {
                exhausted, ready, ..
            } => owns(&[*exhausted.clone(), *ready.clone()]),
            _ => false,
        })
    }
    a.key.starts_with("next20-bq03")
        || a.event == Some(Event::Next20VampireTargeted)
        || owns(&a.ops)
        || a.modes.iter().any(|m| owns(&m.ops))
}
pub(crate) fn frame_related(f: &ResolutionFrame) -> bool {
    let a = ability(
        &f.ability_key,
        "",
        Timing::Fast,
        vec![],
        vec![],
        f.steps.iter().map(|s| s.op.clone()).collect(),
        None,
    );
    related(&a)
}
fn vampire_target() -> TargetSlotSpec {
    let mut t = target(
        Zone::Board,
        EntityKind::Character,
        Relation::ControlledByActor,
        Range::Anywhere,
    );
    t.subtype = Some("吸血鬼".into());
    t
}
pub(crate) fn definition(id: &str) -> Option<Definition> {
    Some(match id {
        "BQ033" => {
            let mut d = with_abilities(vec![ability(
                DISCARD,
                "进场：目标玩家随机弃一张手牌",
                Timing::Fast,
                vec![],
                vec![target(
                    Zone::Player,
                    EntityKind::Any,
                    Relation::Any,
                    Range::Anywhere,
                )],
                vec![op(BlueProgram::RandomDiscard)],
                Some(Event::Enter),
            )]);
            d.traits.spirit = true;
            d
        }
        "BQ034" => with_abilities(vec![
            ability(
                ENTRY,
                "进场：饮血1",
                Timing::Fast,
                vec![],
                vec![],
                vec![op(BlueProgram::EntryFeed)],
                Some(Event::Enter),
            ),
            ability(
                COUNTER,
                "本方吸血鬼成为敌方卡牌目标：可牺牲结附其上的本方附属，然后终止该卡牌",
                Timing::Fast,
                vec![],
                vec![],
                vec![op(BlueProgram::Counter { observation: None })],
                Some(Event::Next20VampireTargeted),
            ),
        ]),
        _ => return None,
    })
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if CARDS.contains(&id) && !same(d, &definition(id).unwrap()) {
        return Err("蓝色四卡切片定义须保留完整原卡程序".into());
    }
    Ok(())
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    let mut normalized = a.clone();
    for o in &mut normalized.ops {
        if let Op::Next20Blue(BlueProgram::Counter { observation }) = o {
            *observation = None;
        }
    }
    if definition(id).is_none_or(|d| {
        !d.abilities
            .iter()
            .any(|expected| same(expected, &normalized))
    }) {
        return Err("蓝色新卡不能移植来源、改写触发或追加操作".into());
    }
    Ok(())
}
pub(crate) fn validate_action(id: &str, a: &Action) -> RuleResult<()> {
    if !CARDS.contains(&id) {
        return Ok(());
    }
    let expected = match a.kind.as_str() {
        "deploy" => Action {
            card_id: a.card_id.clone(),
            region: a.region,
            cost_selected: if id == "BQ033" {
                a.cost_selected.clone()
            } else {
                None
            },
            ..Action::new("deploy")
        },
        "conceal" => Action {
            card_id: a.card_id.clone(),
            region: a.region,
            ..Action::new("conceal")
        },
        "reveal" => Action {
            card_id: a.card_id.clone(),
            cost_selected: if id == "BQ033" {
                a.cost_selected.clone()
            } else {
                None
            },
            ..Action::new("reveal")
        },
        _ => return Err("蓝色新卡只允许正常派遣、暗藏和现身".into()),
    };
    if *a != expected
        || id == "BQ033"
            && matches!(a.kind.as_str(), "deploy" | "reveal")
            && a.cost_selected.as_ref().is_none_or(|s| s.len() != 1)
    {
        return Err("热夜额外费用或角色行动含无关载荷".into());
    }
    Ok(())
}
pub(crate) fn validate_choice_action(p: &Pending, a: &Action) -> RuleResult<()> {
    let related = match &p.resolution {
        ChoiceResolution::Declare { declaration, .. } => {
            CARDS.contains(&declaration.source.card.definition.as_str())
                || self::related(&declaration.ability)
        }
        ChoiceResolution::Frame { frame, choice } => {
            matches!(choice, FrameChoice::Next20Blue(_))
                || frame.ability_key.starts_with("next20-bq03")
        }
        _ => false,
    };
    if related
        && *a
            != (Action {
                choice_id: a.choice_id.clone(),
                selected: a.selected.clone(),
                ..Action::new("choose")
            })
    {
        return Err("蓝色新卡待选只接受原选项，不接受额外费用、数量或目标".into());
    }
    Ok(())
}
impl Game {
    pub(crate) fn next20_source(&self, c: &Card, region: usize) -> SourceSnapshot {
        let mut s = self.source_snapshot(c, Some(region));
        s.source_region_instance = Some(self.regions[region].card.id.clone());
        s
    }
    /// Mandatory extra payment for face-up playing AND paid reveal (manual
    /// "暗藏者翻面", step 3). Conceal and putting directly into play do not pay.
    pub(crate) fn bq033_anchor_options(&self, actor: usize) -> Vec<String> {
        if self.players.get(actor).is_none_or(|p| p.eliminated) {
            return vec![];
        }
        let host = format!("society:p{actor}");
        self.sealed_cards
            .iter()
            .filter(|s| {
                s.host_id == host
                    && catalog::card(&s.card.definition).name == "精神锚点"
                    && self
                        .players
                        .get(s.card.owner)
                        .is_some_and(|p| !p.eliminated)
            })
            .map(|s| s.card.id.clone())
            .collect()
    }
    pub(crate) fn bq033_pay_anchor(
        &mut self,
        actor: usize,
        selected: &str,
    ) -> RuleResult<AnchorPayment> {
        if !self
            .bq033_anchor_options(actor)
            .iter()
            .any(|id| id == selected)
        {
            return Err("必须支付本人秘社实际封印的一张精神锚点".into());
        }
        let i = self
            .sealed_cards
            .iter()
            .position(|s| s.card.id == selected)
            .unwrap();
        let sealed = self.sealed_cards.remove(i);
        let owner = sealed.card.owner;
        let successor = self.reset_zone_card(sealed.card.clone());
        let receipt = AnchorPayment {
            turn: self.turn,
            payer: actor,
            anchor: sealed.card,
            society_host: sealed.host_id,
            deck_instance: successor.id.clone(),
        };
        self.players[owner].deck.push(successor);
        self.shuffle_player(owner);
        Ok(receipt)
    }
    pub(crate) fn validate_bq033_payment(&self, p: &AnchorPayment) -> RuleResult<()> {
        if p.turn != self.turn
            || p.payer >= self.players.len()
            || p.anchor.owner >= self.players.len()
            || p.society_host != format!("society:p{}", p.payer)
            || !catalog::catalog()
                .cards
                .iter()
                .any(|d| d.id == p.anchor.definition && d.name == "精神锚点")
            || !self
                .mystic_instance_number(&p.anchor.id)
                .zip(self.mystic_instance_number(&p.deck_instance))
                .is_some_and(|(a, b)| a < b)
            || self.batch3_physical_cards().any(|c| c.id == p.anchor.id)
        {
            return Err("热夜之梦额外费用须保留原封印及洗入拥有者牌库的真实后继".into());
        }
        Ok(())
    }
    pub(crate) fn bq033_random_discard(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        self.validate_next20_blue_frame(f)?;
        if f.cursor != 1 || !matches!(f.guard, GuardState::Accepted) {
            return Err("随机弃牌尚未通过原帧守卫".into());
        }
        let id = f.targets.first().ok_or("热夜之梦缺少目标玩家")?.id.clone();
        let seat = id
            .strip_prefix('p')
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|s| *s < self.players.len())
            .ok_or("目标不是玩家")?;
        if self.players[seat].eliminated || self.players[seat].hand.is_empty() {
            return Ok(());
        }
        let i = self.random_below(self.players[seat].hand.len());
        let discarded = self.players[seat].hand[i].id.clone();
        self.discard_hand_card(seat, &discarded)?;
        Ok(())
    }
    pub(crate) fn bq034_observe_card(&mut self, f: &ResolutionFrame) {
        // Called only for a real transaction, alongside WM058's existing hook.
        if !(catalog::card(&f.source.card.definition).kind == "spell"
            || catalog::has_kind(catalog::card(&f.source.card.definition), "attachment"))
        {
            return;
        }
        let sources = self
            .regions
            .iter()
            .enumerate()
            .filter(|(_, r)| !r.vacant)
            .flat_map(|(r, region)| region.cards.iter().map(move |c| (r, c)))
            .filter(|(_, c)| {
                c.definition == "BQ034" && !c.face_down && !self.players[c.controller].eliminated
            })
            .map(|(r, c)| self.next20_source(c, r))
            .collect::<Vec<_>>();
        let mut seen = BTreeSet::new();
        let targets = f
            .targets
            .iter()
            .filter(|t| t.spec.zone == Zone::Board && seen.insert(t.id.clone()))
            .filter_map(|t| self.board(&t.id))
            .filter(|(_, c)| !c.face_down && catalog::card(&c.definition).kind == "character")
            .filter_map(|(r, c)| {
                let subtypes = self.current_subtypes(c);
                subtypes
                    .iter()
                    .any(|s| s == "吸血鬼")
                    .then(|| (self.next20_source(c, r), subtypes))
            })
            .collect::<Vec<_>>();
        for source in sources {
            for (target, subtypes) in &targets {
                if target.card.controller != source.card.controller
                    || !self.is_enemy(f.actor, &target.card)
                {
                    continue;
                }
                let o = VampireTargeted {
                    turn: self.turn,
                    transaction: Box::new(f.clone()),
                    target: target.clone(),
                    subtypes: subtypes.clone(),
                };
                let mut a = definition("BQ034").unwrap().abilities.remove(1);
                a.ops = vec![op(BlueProgram::Counter {
                    observation: Some(Box::new(o)),
                })];
                self.effects.push_back(Effect::Declare {
                    declaration: Declaration {
                        actor: source.card.controller,
                        source: source.clone(),
                        ability: a,
                    },
                });
            }
        }
    }
    fn bq034_counter_observation<'a>(
        &self,
        f: &'a ResolutionFrame,
    ) -> RuleResult<&'a VampireTargeted> {
        let [Step {
            op:
                Op::Next20Blue(BlueProgram::Counter {
                    observation: Some(o),
                }),
            ..
        }] = f.steps.as_slice()
        else {
            return Err("雾羽弥生缺少原卡牌指定目标观察".into());
        };
        let t = &o.target.card;
        let tx = &o.transaction;
        let printed = catalog::catalog()
            .cards
            .iter()
            .find(|d| d.id == tx.source.card.definition)
            .ok_or("雾羽目标事件含未登记卡牌")?;
        let target_print = catalog::catalog()
            .cards
            .iter()
            .find(|d| d.id == t.definition && d.kind == "character")
            .ok_or("雾羽目标事件含未登记角色")?;
        if !(target_print
            .subtypes
            .iter()
            .any(|s| matches!(s.as_str(), "吸血鬼" | "人类"))
            || t.definition == "JC030")
            || o.subtypes.iter().collect::<BTreeSet<_>>().len() != o.subtypes.len()
            || o.subtypes.iter().any(|s| {
                !target_print.subtypes.contains(s)
                    && !matches!(
                        s.as_str(),
                        "吸血鬼"
                            | "奴仆"
                            | "邪教徒"
                            | "宿主"
                            | "不死生物"
                            | "骷髅"
                            | "蝙蝠"
                            | "恶魔"
                            | "罪犯"
                    )
            })
        {
            return Err("雾羽观察含不可能的吸血鬼类别快照".into());
        }
        if f.source.card.definition != "BQ034"
            || f.ability_key != COUNTER
            || o.turn != self.turn
            || t.face_down
            || t.owner >= self.players.len()
            || t.controller != f.actor
            || tx.actor >= self.players.len()
            || tx.source.card.owner >= self.players.len()
            || tx.source.card.controller != tx.actor
            || tx.source.card.face_down
            || !self.is_enemy(tx.actor, t)
            || !o.subtypes.iter().any(|s| s == "吸血鬼")
            || !tx
                .targets
                .iter()
                .any(|bound| bound.id == t.id && bound.spec.zone == Zone::Board)
            || !(printed.kind == "spell" || catalog::has_kind(printed, "attachment"))
            || !crate::rules::definition(&tx.source.card.definition)
                .abilities
                .iter()
                .any(|a| {
                    a.key == tx.ability_key
                        && a.event.is_none()
                        && !a.activation_only
                        && (printed.kind == "spell" || a.key == "attach")
                })
            || self.mystic_instance_number(&tx.frame_id).is_none()
            || self.stack.iter().any(|item| {
                item.id == tx.frame_id
                    && (item.card.is_none()
                        || item
                            .frame
                            .as_ref()
                            .is_none_or(|live| !same(live, tx.as_ref())))
            })
        {
            return Err("雾羽弥生只能回应原敌方卡牌实际指定的本人吸血鬼".into());
        }
        self.mobility_immutable_identity(&tx.source.card)?;
        self.mobility_immutable_identity(t)?;
        Ok(o)
    }
    fn bq034_sacrifice_options(&self, f: &ResolutionFrame) -> RuleResult<Vec<ChoiceOption>> {
        let o = self.bq034_counter_observation(f)?;
        if !self
            .stack
            .iter()
            .any(|s| s.id == o.transaction.frame_id && s.card.is_some())
        {
            return Ok(vec![]);
        }
        Ok(self
            .attachments
            .iter()
            .filter(|a| {
                a.host_id == o.target.card.id
                    && a.card.controller == f.actor
                    && !a.card.face_down
                    && self.attachment_host_valid(a)
            })
            .map(|a| self.option(&a.card, f.actor, None, None))
            .collect())
    }
    pub(crate) fn bq034_counter_start(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        self.validate_next20_blue_frame(&f)?;
        if f.cursor != 1 || !matches!(f.guard, GuardState::Accepted) {
            return Err("终止程序尚未通过原帧守卫".into());
        }
        let options = self.bq034_sacrifice_options(&f)?;
        if !options.is_empty() {
            self.choice(
                f.actor,
                COUNTER,
                "雾羽弥生：可牺牲目标吸血鬼上的本方附属并终止原卡牌".into(),
                options,
                0,
                1,
                None,
                ChoiceResolution::Frame {
                    frame: Box::new(f),
                    choice: FrameChoice::Next20Blue(BlueChoice::SacrificeAttachment),
                },
            );
        }
        Ok(())
    }
    pub(crate) fn bq034_counter_finish(
        &mut self,
        f: &ResolutionFrame,
        chooser: usize,
        selected: &[String],
    ) -> RuleResult<()> {
        self.validate_next20_blue_frame(f)?;
        if f.cursor != 1 || !matches!(f.guard, GuardState::Accepted) {
            return Err("终止选择阶段无效".into());
        }
        if chooser != f.actor
            || selected.len() > 1
            || selected.iter().any(|id| {
                !self
                    .bq034_sacrifice_options(f)
                    .unwrap_or_default()
                    .iter()
                    .any(|o| &o.id == id)
            })
        {
            return Err("雾羽弥生只能牺牲原目标上的本人附属一次".into());
        }
        let Some(id) = selected.first() else {
            return Ok(());
        };
        let tx = self
            .bq034_counter_observation(f)?
            .transaction
            .frame_id
            .clone();
        self.remove_dead(id, RemovalCause::Sacrifice);
        if self.in_play(id).is_some() {
            return Ok(());
        }
        if let Some(i) = self
            .stack
            .iter()
            .position(|s| s.id == tx && s.card.is_some())
        {
            let c = self.stack.remove(i).card.unwrap();
            if !self.players[c.owner].eliminated {
                let c = self.fresh(c);
                self.commit_owner_grave(c, None);
            }
        }
        Ok(())
    }
    pub(crate) fn bq034_feed_start(
        &mut self,
        f: ResolutionFrame,
        recipient: String,
    ) -> RuleResult<()> {
        self.validate_next20_blue_frame(&f)?;
        if f.cursor != 1 || !matches!(f.guard, GuardState::Accepted) {
            return Err("饮血尚未通过原帧守卫".into());
        }
        let expected = if f.ability_key == ENTRY {
            &f.source.card.id
        } else if f.ability_key == SHOWN {
            &f.targets.first().ok_or("雾羽展示后尚未绑定吸血鬼")?.id
        } else {
            return Err("不是雾羽弥生饮血程序".into());
        };
        if &recipient != expected {
            return Err("雾羽弥生不能更换饮血收受者".into());
        }
        let Some((_, c)) = self.board(&recipient).filter(|(_, c)| {
            !c.face_down && self.current_subtypes(c).iter().any(|s| s == "吸血鬼")
        }) else {
            self.bq034_complete_shown(&f);
            return Ok(());
        };
        let seat = c.controller;
        let options = self.blood_options(seat);
        if options.is_empty() {
            self.bq034_complete_shown(&f);
        }
        if !options.is_empty() {
            self.choice(
                seat,
                ENTRY,
                "饮血1：选择本人墓地或本人秘社封印的鲜血".into(),
                options,
                1,
                1,
                None,
                ChoiceResolution::Frame {
                    frame: Box::new(f),
                    choice: FrameChoice::Next20Blue(BlueChoice::Blood { recipient }),
                },
            );
        }
        Ok(())
    }
    pub(crate) fn bq034_feed_finish(
        &mut self,
        f: &ResolutionFrame,
        recipient: &str,
        chooser: usize,
        selected: &[String],
    ) -> RuleResult<()> {
        self.validate_next20_blue_frame(f)?;
        if f.cursor != 1 || !matches!(f.guard, GuardState::Accepted) {
            return Err("饮血选择阶段无效".into());
        }
        let expected = if f.ability_key == ENTRY {
            &f.source.card.id
        } else {
            &f.targets.first().ok_or("展示后缺少绑定吸血鬼")?.id
        };
        if recipient != expected || selected.len() != 1 {
            return Err("饮血不能更换原收受者或数量".into());
        }
        let (definition, id) = self.attach_selected_blood(chooser, recipient, &selected[0])?;
        self.enter_triggers(chooser, &definition, &id, false);
        if f.ability_key == SHOWN {
            if let Some(crate::mystic_contest::MysticContest {
                phase: MysticPhase::After { post_frame, .. },
                ..
            }) = &mut self.mystic_contest
            {
                *post_frame = None;
            }
        }
        Ok(())
    }
    pub(crate) fn bq034_shown(&mut self, o: &ContestOutcome, index: usize) -> RuleResult<()> {
        let shown = o.shown.get(index).ok_or("雾羽展示索引无效")?;
        if shown.card.definition != "BQ034" {
            return Ok(());
        }
        let f = ResolutionFrame {
            resource_payments: vec![],
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
                op: op(BlueProgram::ShownFeed {
                    contest_id: o.contest_id.clone(),
                    index,
                }),
            }],
            chosen_region: None,
        };
        let Some(state) = &mut self.mystic_contest else {
            return Err("雾羽展示后缺少原隐秘对抗".into());
        };
        let MysticPhase::After { post_frame, .. } = &mut state.phase else {
            return Err("雾羽不是展示后阶段".into());
        };
        *post_frame = Some(Box::new(f.clone()));
        self.validate_next20_blue_frame(&f)?;
        let options = self.binding_options(f.actor, &f.source, &vampire_target());
        if options.is_empty() {
            self.bq034_complete_shown(&f);
        }
        if !options.is_empty() {
            self.choice(
                f.actor,
                SHOWN,
                "雾羽弥生：目标本方吸血鬼饮血1".into(),
                options,
                1,
                1,
                None,
                ChoiceResolution::Frame {
                    frame: Box::new(f),
                    choice: FrameChoice::Next20Blue(BlueChoice::ShownTarget),
                },
            );
        }
        Ok(())
    }
    pub(crate) fn bq034_shown_target(
        &mut self,
        mut f: ResolutionFrame,
        chooser: usize,
        selected: &[String],
    ) -> RuleResult<()> {
        self.validate_next20_blue_frame(&f)?;
        if chooser != f.actor
            || selected.len() != 1
            || !f.targets.is_empty()
            || !self.valid_binding(f.actor, &f.source, &vampire_target(), &selected[0])
        {
            return Err("雾羽展示后须选择一个本人吸血鬼".into());
        }
        let t = vampire_target();
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
        self.bq034_feed_start(f, selected[0].clone())
    }
    pub(crate) fn validate_next20_blue_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if !CARDS.contains(&f.source.card.definition.as_str())
            || f.actor >= self.players.len()
            || f.source.card.controller != f.actor
            || f.source.card.owner >= self.players.len()
            || f.source.card.face_down
            || f.steps.len() != 1
            || f.steps[0].context != f.actor
            || f.cursor > 1
            || f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked)
            || f.cursor == 1 && !matches!(f.guard, GuardState::Accepted)
            || !f.already_paid.is_empty()
            || !f.resource_payments.is_empty()
            || f.chosen_region.is_some()
            || !self
                .mystic_instance_number(&f.source.card.id)
                .zip(self.mystic_instance_number(&f.frame_id))
                .is_some_and(|(s, f)| s < f)
            || f.source.play_source.is_some()
            || f.source.death_echo.is_some()
            || f.source.observed_death.is_some()
            || f.source.observed_exposure.is_some()
            || f.source.observed_mobility_grant.is_some()
            || f.source.attachment_host_instance.is_some()
        {
            return Err("蓝色新卡帧来源、费用、程序或游标无效".into());
        }
        self.mobility_immutable_identity(&f.source.card)?;
        if f.ability_key != SHOWN
            && (f.source.region.is_none_or(|r| r >= self.regions.len())
                || f.source
                    .source_region_instance
                    .as_deref()
                    .is_none_or(|id| !self.mystic_instance_valid(id)))
        {
            return Err("蓝色角色触发缺少原地区实体".into());
        }
        match (
            &f.steps[0].op,
            f.ability_key.as_str(),
            f.source.card.definition.as_str(),
        ) {
            (Op::Next20Blue(BlueProgram::RandomDiscard), DISCARD, "BQ033")
                if f.targets.len() == 1 =>
            {
                if !same(
                    &f.targets[0].spec,
                    &definition("BQ033").unwrap().abilities[0].targets[0],
                ) {
                    return Err("热夜之梦必须绑定原玩家目标".into());
                }
            }
            (Op::Next20Blue(BlueProgram::EntryFeed), ENTRY, "BQ034") if f.targets.is_empty() => (),
            (Op::Next20Blue(BlueProgram::Counter { .. }), COUNTER, "BQ034")
                if f.targets.is_empty() =>
            {
                self.bq034_counter_observation(f)?;
            }
            (Op::Next20Blue(BlueProgram::ShownFeed { contest_id, index }), SHOWN, "BQ034")
                if f.targets.len() <= 1 =>
            {
                let Some(state) = &self.mystic_contest else {
                    return Err("雾羽后置缺少原对抗".into());
                };
                let MysticPhase::After {
                    outcome,
                    next_shown,
                    post_frame,
                } = &state.phase
                else {
                    return Err("雾羽后置阶段无效".into());
                };
                if outcome.contest_id != *contest_id
                    || *next_shown != index + 1
                    || outcome
                        .shown
                        .get(*index)
                        .is_none_or(|s| s.seat != f.actor || s.card != f.source.card)
                    || post_frame
                        .as_ref()
                        .is_none_or(|saved| !same(saved.as_ref(), f))
                    || f.targets.iter().any(|t| !same(&t.spec, &vampire_target()))
                {
                    return Err("雾羽后置须保留同次本人展示与唯一后置帧".into());
                }
                if f.targets.iter().any(|t| {
                    t.id != t.public.instance_id
                        || t.region_instance.is_some()
                        || t.public.kind != "character"
                        || t.public.controller.as_deref() != Some(player_id(f.actor).as_str())
                        || !t.public.valid
                        || t.public.status != "valid"
                        || t.public.invalid_reason.is_some()
                        || !self.valid_binding(f.actor, &f.source, &t.spec, &t.id)
                        || !same(
                            &t.public,
                            &self.public_target(f.actor, &f.source, &t.spec, &t.id),
                        )
                }) {
                    return Err("雾羽展示后饮血须保留原本人吸血鬼目标绑定".into());
                }
            }
            _ => return Err("蓝色新卡不能混用来源和操作".into()),
        }
        Ok(())
    }
    pub(crate) fn validate_next20_blue_declaration(&self, d: &Declaration) -> RuleResult<()> {
        validate_ability(&d.source.card.definition, &d.ability)?;
        if d.actor >= self.players.len()
            || d.source.card.controller != d.actor
            || d.source.card.face_down
            || d.source.card.owner >= self.players.len()
            || d.source.play_source.is_some()
            || d.source.death_echo.is_some()
            || d.source.observed_death.is_some()
            || d.source.observed_exposure.is_some()
            || d.source.observed_mobility_grant.is_some()
            || d.source.attachment_host_instance.is_some()
            || !self.mystic_instance_valid(&d.source.card.id)
            || d.source.region.is_none_or(|r| r >= self.regions.len())
            || d.source
                .source_region_instance
                .as_deref()
                .is_none_or(|id| !self.mystic_instance_valid(id))
        {
            return Err("蓝色新卡声明须保留原正面来源和地区".into());
        }
        self.mobility_immutable_identity(&d.source.card)?;
        if d.ability.key == COUNTER {
            // The observation is validated independently of having allocated a
            // response-frame UID. No caller may mint an event from a card name.
            let f = ResolutionFrame {
                resource_payments: vec![],
                frame_id: String::new(),
                ability_key: COUNTER.into(),
                actor: d.actor,
                source: d.source.clone(),
                targets: vec![],
                already_paid: vec![],
                guard: GuardState::Unchecked,
                cursor: 0,
                steps: d
                    .ability
                    .ops
                    .iter()
                    .map(|op| Step {
                        context: d.actor,
                        op: op.clone(),
                    })
                    .collect(),
                chosen_region: None,
            };
            self.bq034_counter_observation(&f)?;
        }
        Ok(())
    }
    pub(crate) fn validate_next20_blue_pending(&self, p: &Pending) -> RuleResult<()> {
        self.validate_blue_declaration_pending(p)?;
        if matches!(&p.resolution, ChoiceResolution::Frame { frame,choice } if frame_related(frame) && !matches!(choice,FrameChoice::Next20Blue(_)))
        {
            return Err("蓝色新卡不能借用其他卡的待选续体".into());
        }
        let ChoiceResolution::Frame {
            frame: f,
            choice: FrameChoice::Next20Blue(choice),
        } = &p.resolution
        else {
            return Ok(());
        };
        self.validate_next20_blue_frame(f)?;
        if f.cursor != 1
            || !matches!(f.guard, GuardState::Accepted)
            || p.choice.id.is_empty()
            || p.choice.player_id != player_id(p.seat)
            || p.choice.amount.is_some()
            || !p.choice.preview_cards.is_empty()
        {
            return Err("蓝色新卡待选身份或阶段无效".into());
        }
        let (seat, options, min, max, kind) = match choice {
            BlueChoice::ShownTarget if f.ability_key == SHOWN && f.targets.is_empty() => (
                f.actor,
                self.binding_options(f.actor, &f.source, &vampire_target()),
                1,
                1,
                SHOWN,
            ),
            BlueChoice::SacrificeAttachment if f.ability_key == COUNTER => {
                (f.actor, self.bq034_sacrifice_options(f)?, 0, 1, COUNTER)
            }
            BlueChoice::Blood { recipient } if f.ability_key == ENTRY || f.ability_key == SHOWN => {
                let expected = if f.ability_key == ENTRY {
                    &f.source.card.id
                } else {
                    &f.targets.first().ok_or("展示后缺少原饮血目标")?.id
                };
                let (_, c) = self
                    .board(recipient)
                    .filter(|(_, c)| {
                        !c.face_down && self.current_subtypes(c).iter().any(|s| s == "吸血鬼")
                    })
                    .ok_or("饮血收受者已失效")?;
                if recipient != expected {
                    return Err("饮血待选更换了原收受者".into());
                }
                (c.controller, self.blood_options(c.controller), 1, 1, ENTRY)
            }
            _ => return Err("蓝色待选种类与程序不符".into()),
        };
        if p.seat != seat
            || options.is_empty()
            || !same(&p.choice.options, &options)
            || p.choice.kind != kind
            || p.choice.min != Some(min)
            || p.choice.max != Some(max)
            || p.choice.allow_decline != Some(min == 0)
        {
            return Err("蓝色新卡须保留当前操控者、原选项和数量限制".into());
        }
        Ok(())
    }
    pub(crate) fn validate_next20_blue_state(&self) -> RuleResult<()> {
        let mut frames = BTreeSet::new();
        let mut events = BTreeSet::new();
        let remember = |source: &SourceSnapshot,
                        key: &str,
                        ops: &[Op],
                        events: &mut BTreeSet<(String, String, String)>|
         -> RuleResult<()> {
            let event = match ops {
                [Op::Next20Blue(BlueProgram::Counter {
                    observation: Some(o),
                })] => format!("{}:{}", o.transaction.frame_id, o.target.card.id),
                _ => key.to_owned(),
            };
            if !events.insert((source.card.id.clone(), key.into(), event)) {
                return Err("蓝色新卡不能重复保存同次触发".into());
            }
            Ok(())
        };
        for item in &self.stack {
            if let Some(f) = &item.frame {
                self.validate_next20_blue_play_frame(f)?;
                if CARDS.contains(&f.source.card.definition.as_str())
                    && matches!(f.ability_key.as_str(), "deploy" | "reveal")
                    && (item.id != f.frame_id
                        || item.controller != f.actor
                        || item.card.is_none()
                        || item.deploy_region != f.source.region
                        || item.reveal != (f.ability_key == "reveal")
                        || item.target.is_some()
                        || item.card.as_ref().is_some_and(|c| {
!self.pack_a_transaction_matches(f, c)
                        }))
                {
                    return Err("蓝卡派遣或现身的原响应对象被改写".into());
                }
                if frame_related(f) {
                    self.validate_next20_blue_frame(f)?;
                    if !frames.insert(f.frame_id.clone())
                        || item.id != f.frame_id
                        || item.controller != f.actor
                        || item.card.is_some()
                        || item.deploy_region.is_some()
                        || item.reveal
                        || item.target != f.targets.first().map(|t| t.id.clone())
                        || f.cursor != 0
                        || !matches!(f.guard, GuardState::Unchecked)
                    {
                        return Err("蓝色触发的原响应对象或阶段无效".into());
                    }
                    remember(
                        &f.source,
                        &f.ability_key,
                        &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>(),
                        &mut events,
                    )?;
                }
            }
        }
        for e in &self.effects {
            if let Effect::Frame { frame } = e {
                self.validate_next20_blue_play_frame(frame)?;
            }
            match e {
                Effect::Declare { declaration: d } if related(&d.ability) => {
                    self.validate_next20_blue_declaration(d)?;
                    remember(&d.source, &d.ability.key, &d.ability.ops, &mut events)?;
                }
                Effect::Frame { frame: f } if frame_related(f) => {
                    return Err("蓝色新卡不能绕过原响应或待选直接续帧".into())
                }
                _ => (),
            }
        }
        if let Some(p) = &self.pending {
            self.validate_next20_blue_pending(p)?;
            if let ChoiceResolution::Frame { frame, .. } = &p.resolution {
                self.validate_next20_blue_play_frame(frame)?;
            }
            match &p.resolution {
                ChoiceResolution::Declare { declaration: d, .. } if related(&d.ability) => {
                    self.validate_next20_blue_declaration(d)?;
                    remember(&d.source, &d.ability.key, &d.ability.ops, &mut events)?;
                }
                ChoiceResolution::Frame { frame: f, .. } if frame_related(f) => {
                    self.validate_next20_blue_pending(p)?;
                    if !frames.insert(f.frame_id.clone()) {
                        return Err("蓝色新卡待选与响应对象重复".into());
                    }
                }
                _ => (),
            }
        }
        for state in self.mystic_states() {
            self.validate_next20_blue_play_frame(&state.frame)?;
        }
        if let Some(state) = &self.mystic_contest {
            if let MysticPhase::After {
                post_frame: Some(f),
                ..
            } = &state.phase
            {
                if f.ability_key == SHOWN {
                    self.validate_next20_blue_frame(f)?;
                }
            }
        }
        Ok(())
    }
}

impl Game {
    fn validate_blue_declaration_pending(&self, p: &Pending) -> RuleResult<()> {
        let ChoiceResolution::Declare {
            declaration: d,
            stage,
        } = &p.resolution
        else {
            return Ok(());
        };
        if !related(&d.ability) {
            return Ok(());
        }
        self.validate_next20_blue_declaration(d)?;
        let (options, stage_valid) = if let Some(slot) = d.ability.targets.first() {
            (
                self.binding_options(d.actor, &d.source, slot),
                matches!(stage, DeclareChoice::Target),
            )
        } else {
            (
                vec![ChoiceOption {
                    id: "accept".into(),
                    label: "发动触发能力".into(),
                    card: None,
                }],
                matches!(stage, DeclareChoice::Accept),
            )
        };
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
            options,
            preview_cards: vec![],
            min: Some(0),
            max: Some(1),
            amount: None,
            allow_decline: Some(true),
        };
        if p.seat != d.actor
            || p.choice.id.is_empty()
            || !stage_valid
            || expected.options.is_empty()
            || !same(&p.choice, &expected)
        {
            return Err("蓝卡声明待选只能保留原发动者、原目标或明确放弃".into());
        }
        Ok(())
    }
    /// Expand the ordinary engine candidates; no independent play/fee path.
    pub(crate) fn bq033_payment_candidates(
        &self,
        actor: usize,
        candidates: Vec<(Action, String)>,
    ) -> Vec<(Action, String)> {
        let anchors = self.bq033_anchor_options(actor);
        candidates
            .into_iter()
            .flat_map(|(a, label)| {
                let heat = matches!(a.kind.as_str(), "deploy" | "reveal")
                    && a.card_id.as_ref().is_some_and(|id| {
                        self.batch3_physical_cards()
                            .any(|c| c.id == *id && c.definition == "BQ033")
                    });
                if !heat {
                    return vec![(a, label)];
                }
                anchors
                    .iter()
                    .map(|anchor| {
                        let mut a = a.clone();
                        a.cost_selected = Some(vec![anchor.clone()]);
                        (a, format!("{label}（洗回本人秘社封印的精神锚点）"))
                    })
                    .collect()
            })
            .collect()
    }

    pub(crate) fn pay_bq033_extra(&mut self, actor: usize, a: &Action) -> RuleResult<PaidCost> {
        let selected = a
            .cost_selected
            .as_ref()
            .filter(|s| s.len() == 1)
            .ok_or("热夜之梦必须选择一张真实封印的精神锚点")?;
        Ok(PaidCost::BQ033Anchor(Box::new(
            self.bq033_pay_anchor(actor, &selected[0])?,
        )))
    }

    /// Authenticate the existing engine's ordinary deploy/reveal receipt.
    pub(crate) fn validate_next20_blue_play_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        let play = CARDS.contains(&f.source.card.definition.as_str())
            && matches!(f.ability_key.as_str(), "deploy" | "reveal");
        if f.already_paid
            .iter()
            .any(|p| matches!(p, PaidCost::BQ033Anchor(_)))
            && !(play && f.source.card.definition == "BQ033")
        {
            return Err("精神锚点收据只能属于热夜之梦的真实派遣或付费现身".into());
        }
        if !play {
            return Ok(());
        }
        let c = &f.source.card;
        if f.actor >= self.players.len()
            || c.controller != f.actor
            || c.owner >= self.players.len()
            || !self
                .mystic_instance_number(&c.id)
                .zip(self.mystic_instance_number(&f.frame_id))
                .is_some_and(|(s, frame)| s < frame)
            || f.cursor != 0
            || !matches!(f.guard, GuardState::Unchecked)
            || !f.steps.is_empty()
            || !f.targets.is_empty()
            || f.chosen_region != f.source.region
            || f.source.region.is_none_or(|r| r >= self.regions.len())
            || f.source
                .source_region_instance
                .as_deref()
                .is_none_or(|id| !self.mystic_instance_valid(id))
            || f.ability_key == "deploy" && (c.face_down || !f.source.valid_character_play())
            || f.ability_key == "reveal" && (!c.face_down || f.source.play_source.is_some())
        {
            return Err("蓝卡派遣或现身须保留原来源、地区和完整支付帧".into());
        }
        self.mobility_immutable_identity(c)?;
        let money = match f.already_paid.first() {
            Some(PaidCost::Assets(ids)) => ids,
            _ => return Err("蓝卡缺少真实印刷费用收据".into()),
        };
        if f.money_units(0) > catalog::card(&c.definition).cost as usize
            || money
                .iter()
                .any(|id| !f.source.valid_printed_payment_id(id))
            || money.iter().collect::<BTreeSet<_>>().len() != money.len()
        {
            return Err("蓝卡印刷费用收据被改写".into());
        }
        self.validate_resource_frame(f)?;
        if c.definition == "BQ033" {
            let [PaidCost::Assets(_), PaidCost::BQ033Anchor(anchor)] = f.already_paid.as_slice()
            else {
                return Err("热夜之梦派遣和付费现身必须已经实际支付锚点".into());
            };
            self.validate_bq033_payment(anchor)?;
            if anchor.payer != f.actor {
                return Err("精神锚点由原出牌玩家支付".into());
            }
        } else if f.already_paid.len() != 1 {
            return Err("雾羽弥生不能夹带额外支付收据".into());
        }
        Ok(())
    }

    fn bq034_complete_shown(&mut self, f: &ResolutionFrame) {
        if f.ability_key == SHOWN {
            if let Some(crate::mystic_contest::MysticContest {
                phase: MysticPhase::After { post_frame, .. },
                ..
            }) = &mut self.mystic_contest
            {
                if post_frame
                    .as_ref()
                    .is_some_and(|p| p.frame_id == f.frame_id)
                {
                    *post_frame = None;
                }
            }
        }
    }
}
