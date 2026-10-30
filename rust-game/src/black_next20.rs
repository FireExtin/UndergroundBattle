//! Five image-verified black cards. Observations use the existing effect queue;
//! every recovery retains the event's actual instance and original region UID.
use crate::{
    catalog,
    engine::RuleResult,
    model::*,
    rules::{self, *},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub(crate) const CARDS: &[&str] = &["BQ077", "BQ085", "CQ26", "XG27", "XQ39"];
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeckGraveObservation {
    pub event_id: String,
    pub turn: u32,
    pub deck_seat: usize,
    pub cards: Vec<Card>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlackDeathObservation {
    pub event_id: String,
    pub turn: u32,
    pub dead: Card,
    pub subtypes: Vec<String>,
    pub region: usize,
    pub region_instance: String,
}
fn same<T: Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
fn owned(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::BlackDeckToGraveyard { .. }
        | Op::BlackDeathRecovery { .. }
        | Op::BQ085Prepare { .. }
        | Op::XQ39GraveCharacterToDeckTop => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => owned(body),
        Op::IfTargetExhausted {
            exhausted, ready, ..
        } => owned(std::slice::from_ref(exhausted)) || owned(std::slice::from_ref(ready)),
        _ => false,
    })
}
fn related(f: &ResolutionFrame) -> bool {
    CARDS.contains(&f.source.card.definition.as_str())
        || f.ability_key.starts_with("black20-")
        || f.steps.iter().any(|s| owned(std::slice::from_ref(&s.op)))
}
fn related_ability(a: &AbilitySpec) -> bool {
    a.key.starts_with("black20-")
        || owned(&a.ops)
        || a.modes.iter().any(|m| owned(&m.ops))
        || matches!(
            a.event,
            Some(Event::DeckToGraveyardObserved | Event::BlackCharacterDeathObserved)
        )
        || a.targets
            .iter()
            .chain(a.modes.iter().flat_map(|m| &m.targets))
            .any(|t| t.predicate == Some(TargetPredicate::BQ077WaterRegion))
}
pub(crate) fn definition(id: &str) -> Option<Definition> {
    let mut d = match id {
        "BQ077" => {
            let mut water = rules::target(
                Zone::Region,
                EntityKind::Any,
                Relation::Any,
                Range::Anywhere,
            );
            water.predicate = Some(TargetPredicate::BQ077WaterRegion);
            let mut d = rules::with_abilities(vec![rules::ability(
                "black20-bq077-return",
                "墓地触发：放置进场于目标水域地区",
                Timing::Fast,
                vec![],
                vec![water],
                vec![Op::BlackDeckToGraveyard { observation: None }],
                Some(Event::DeckToGraveyardObserved),
            )]);
            d.traits.cannot_be_equipped = true;
            d.modifiers.push(StaticModifier::BQ077WaterCombat);
            d
        }
        "BQ085" => {
            let mut ghoul = rules::target(
                Zone::Graveyard,
                EntityKind::Character,
                Relation::OwnedByActor,
                Range::Anywhere,
            );
            ghoul.subtype = Some("食尸鬼".into());
            rules::with_abilities(vec![
                rules::ability(
                    "black20-bq085-prepare",
                    "准备步骤强制触发：将墓地两张牌依次置于牌库底，否则牺牲",
                    Timing::Fast,
                    vec![],
                    vec![],
                    vec![Op::BQ085Prepare { event_id: None }],
                    Some(Event::Prepare),
                ),
                rules::ability(
                    "black20-bq085-return",
                    "触发：横置放置目标本方食尸鬼进场于本地区",
                    Timing::Fast,
                    vec![],
                    vec![ghoul],
                    vec![Op::BlackDeathRecovery { observation: None }],
                    Some(Event::BlackCharacterDeathObserved),
                ),
            ])
        }
        "CQ26" => {
            let mut d = rules::with_abilities(vec![rules::ability(
                "black20-cq26-loot",
                "触发：可以弃一张牌，若如此做，抓一张牌",
                Timing::Fast,
                vec![],
                vec![],
                vec![
                    Op::BlackDeckToGraveyard { observation: None },
                    Op::Discard {
                        player: PlayerRef::Actor,
                        count: Some(1),
                        redraw: true,
                        optional: true,
                    },
                ],
                Some(Event::DeckToGraveyardObserved),
            )]);
            d.modifiers.push(StaticModifier::CQ26GraveIcons);
            d
        }
        "XG27" => {
            let mut d = rules::with_abilities(vec![rules::ability(
                "black20-xg27-return",
                "墓地触发：支付1，以暗藏者放置进场于死者原地区",
                Timing::Fast,
                vec![Cost::Assets(1)],
                vec![],
                vec![Op::BlackDeathRecovery { observation: None }],
                Some(Event::BlackCharacterDeathObserved),
            )]);
            d.traits.spirit = true;
            d.modifiers.push(StaticModifier::XG27CurseReduction);
            d
        }
        "XQ39" => {
            let host = rules::target(
                Zone::Society,
                EntityKind::Any,
                Relation::ControlledByActor,
                Range::Anywhere,
            );
            let mut d = rules::attachment_definition(host);
            let mut discount = rules::ability(
                "black20-xq39-discount",
                "快速行动：下一张方碑牌正面打出或付费翻面减费1",
                Timing::Fast,
                vec![Cost::ExhaustSource],
                vec![],
                vec![Op::CostReductionOnFaceUpOrPaidReveal {
                    filter: CardFilter::PrintedColor("黑".into()),
                    amount: 1,
                }],
                None,
            );
            discount.activation_only = true;
            let mut recover = rules::ability(
                "black20-xq39-recover",
                "快速行动2，横置：目标本人墓地角色置于本人牌库顶",
                Timing::Fast,
                vec![Cost::Assets(2), Cost::ExhaustSource],
                vec![rules::target(
                    Zone::Graveyard,
                    EntityKind::Character,
                    Relation::OwnedByActor,
                    Range::Anywhere,
                )],
                vec![Op::XQ39GraveCharacterToDeckTop],
                None,
            );
            recover.activation_only = true;
            d.abilities.extend([discount, recover]);
            d
        }
        _ => return None,
    };
    // Keep all fields explicit through the same finite registry as older cards.
    d.graveyard_face_up = false;
    Some(d)
}
fn normalized(a: &AbilitySpec) -> AbilitySpec {
    let mut a = a.clone();
    for op in &mut a.ops {
        match op {
            Op::BlackDeckToGraveyard { observation } => *observation = None,
            Op::BlackDeathRecovery { observation } => *observation = None,
            Op::BQ085Prepare { event_id } => *event_id = None,
            _ => {}
        }
    }
    a
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if !CARDS.contains(&id) && !related_ability(a) {
        return Ok(());
    }
    if crate::jz22::runtime_ability_is_admitted(a)
        && matches!(id, "BQ077" | "BQ085" | "CQ26" | "XG27")
    {
        return Ok(());
    }
    if definition(id).is_none_or(|d| {
        !d.abilities
            .iter()
            .any(|expected| same(expected, &normalized(a)))
    }) {
        return Err("黑色五张只准入原图完整能力，观察与减费不得移植".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if definition(id).is_some_and(|expected| !same(d, &expected))
        || !CARDS.contains(&id)
            && (d.abilities.iter().any(related_ability)
                || d.modifiers.iter().any(|m| {
                    matches!(
                        m,
                        StaticModifier::BQ077WaterCombat
                            | StaticModifier::CQ26GraveIcons
                            | StaticModifier::XG27CurseReduction
                    )
                }))
    {
        return Err("黑色五张完整定义不可改写或移植".into());
    }
    Ok(())
}
impl Game {
    pub(crate) fn black_next20_icon_bonus(&self, c: &Card, region: usize) -> (Icons, Icons) {
        if c.face_down
            || !self
                .regions
                .get(region)
                .is_some_and(|r| !r.vacant && r.cards.iter().any(|live| live.id == c.id))
        {
            return (Icons::default(), Icons::default());
        }
        let water = c.definition == "BQ077"
            && catalog::card(&self.regions[region].card.definition)
                .subtypes
                .iter()
                .any(|s| s == "水域");
        let grave = c.definition == "CQ26" && self.bq095_reference_cards(c).len() >= 7;
        (
            Icons {
                combat: u32::from(water || grave),
                ..Default::default()
            },
            Icons {
                investigation: u32::from(grave),
                ..Default::default()
            },
        )
    }
    pub(crate) fn xg27_curse_reduction(&self, actor: usize, c: &Card) -> u32 {
        if !catalog::card(&c.definition)
            .subtypes
            .iter()
            .any(|s| s == "诅咒")
        {
            return 0;
        }
        self.regions
            .iter()
            .flat_map(|r| &r.cards)
            .filter(|c| c.controller == actor && !c.face_down && c.definition == "XG27")
            .count() as u32
    }
    /// One call per completed transfer batch, after replacements; cards are the
    /// fresh, actual graveyard instances, never the pre-replacement deck list.
    pub(crate) fn observe_black_deck_to_graveyard(&mut self, deck_seat: usize, cards: &[Card]) {
        if cards.is_empty() {
            return;
        }
        let observation = DeckGraveObservation {
            event_id: self.id(),
            turn: self.turn,
            deck_seat,
            cards: cards.to_vec(),
        };
        let mut sources = self
            .regions
            .iter()
            .enumerate()
            .flat_map(|(r, region)| {
                region
                    .cards
                    .iter()
                    .filter(move |c| {
                        c.definition == "CQ26" && !c.face_down && c.controller == deck_seat
                    })
                    .map(move |c| (r, c))
            })
            .filter(|(_,source)| {
                cards.iter().any(|g|catalog::card(&g.definition).kind=="character" && self.bq095_own_reference(source)==self.bq095_destination(g.owner))
            })
            .map(|(r, c)| self.source_snapshot(c, Some(r)))
            .collect::<Vec<_>>();
        for c in cards.iter().filter(|c| c.definition == "BQ077") {
            sources.push(self.source_snapshot(c, None));
        }
        sources.sort_by_key(|s| {
            (
                self.team(s.card.controller) != self.first_team,
                s.card.controller,
                s.card.id.clone(),
            )
        });
        for source in sources {
            let mut ability = definition(&source.card.definition).unwrap().abilities[0].clone();
            ability.ops[0] = Op::BlackDeckToGraveyard {
                observation: Some(observation.clone()),
            };
            self.effects.push_back(Effect::Declare {
                declaration: Declaration {
                    actor: source.card.controller,
                    source,
                    ability,
                },
            });
        }
    }
    pub(crate) fn capture_black_death_sources(&self) -> Vec<SourceSnapshot> {
        self.regions
            .iter()
            .enumerate()
            .flat_map(|(r, region)| {
                region
                    .cards
                    .iter()
                    .filter(|c| c.definition == "BQ085" && !c.face_down)
                    .map(move |c| (Some(r), c))
            })
            .chain(
                self.players.iter().enumerate().flat_map(|(seat,_)|self.bq095_self_grave_sources(seat))
                    .filter(|c| c.definition == "XG27")
                    .map(|c| (None, c)),
            )
            .filter(|(_, c)| !self.players[c.controller].eliminated)
            .map(|(r, c)| self.source_snapshot(c, r))
            .collect()
    }
    pub(crate) fn capture_black_death(&self, id: &str) -> Option<BlackDeathObservation> {
        let (region, dead) = self.board(id)?;
        if dead.face_down || catalog::card(&dead.definition).kind != "character" {
            return None;
        }
        Some(BlackDeathObservation {
            event_id: String::new(),
            turn: self.turn,
            dead: dead.clone(),
            subtypes: self.current_subtypes(dead),
            region,
            region_instance: self.regions[region].card.id.clone(),
        })
    }
    pub(crate) fn emit_black_death(
        &mut self,
        sources: &[SourceSnapshot],
        mut observation: BlackDeathObservation,
    ) -> BlackDeathObservation {
        observation.event_id = self.id();
        for source in sources {
            let actor = source.card.controller;
            let ghoul = source.card.definition == "BQ085";
            if (ghoul
                && (observation.dead.controller != actor
                    || observation.subtypes.iter().any(|s| s == "食尸鬼")))
                || (!ghoul && self.team(observation.dead.controller) == self.team(actor))
            {
                continue;
            }
            let mut ability = definition(&source.card.definition)
                .unwrap()
                .abilities
                .into_iter()
                .find(|a| a.event == Some(Event::BlackCharacterDeathObserved))
                .unwrap();
            ability.ops[0] = Op::BlackDeathRecovery {
                observation: Some(observation.clone()),
            };
            self.effects.push_back(Effect::Declare {
                declaration: Declaration {
                    actor,
                    source: source.clone(),
                    ability,
                },
            });
        }
        observation
    }
    pub(crate) fn black_next20_trigger_available(&self, d: &Declaration) -> bool {
        match (d.source.card.definition.as_str(), d.ability.event) {
            ("BQ077", Some(Event::DeckToGraveyardObserved)) => self.bq095_reference_cards(&d.source.card).iter()
                .any(|c| c.id == d.source.card.id),
            ("XG27", Some(Event::BlackCharacterDeathObserved)) => {
                self.resources(d.actor) >= 1
                    && self.bq095_reference_cards(&d.source.card).iter()
                        .any(|c| c.id == d.source.card.id)
            }
            _ => true,
        }
    }
    fn black_next20_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        let c = &s.card;
        if !CARDS.contains(&c.definition.as_str())
            || actor >= self.players.len()
            || c.controller != actor
            || c.owner >= self.players.len()
            || c.id.is_empty()
            || s.observed_death.is_some()
            || s.observed_mobility_grant.is_some() || s.observed_exposure.is_some()
            || s.region.is_some_and(|r| r >= self.regions.len())
            || (c.definition != "XQ39" && s.attachment_host_instance.is_some())
            || s.region.is_some() != s.source_region_instance.is_some()
            || s.source_region_instance
                .as_ref()
                .is_some_and(|id| id.is_empty() || id == &c.id)
            || s.source_region_instance.as_ref().is_some_and(|id| {
                self.regions
                    .iter()
                    .enumerate()
                    .any(|(r, region)| region.card.id == *id && Some(r) != s.region)
            })
        {
            return Err("黑色五张冻结来源身份、操控者或原地区无效".into());
        }
        self.mobility_immutable_identity(c)?;
        Ok(())
    }
    fn black_next20_observation(
        &self,
        actor: usize,
        s: &SourceSnapshot,
        ops: &[Op],
    ) -> RuleResult<()> {
        for op in ops {
            match op {
                Op::BlackDeckToGraveyard { observation } => {
                    let o = observation.as_ref().ok_or("牌库入墓观察缺失")?;
                    if o.event_id.is_empty()
                        || o.turn != self.turn
                        || o.deck_seat >= self.players.len()
                        || o.cards.is_empty()
                        || o.cards.iter().any(|c| {
                            c.id.is_empty()
                                || c.owner >= self.players.len()
                                || c.controller != c.owner
                                || c.face_down
                                || !catalog::catalog()
                                    .cards
                                    .iter()
                                    .any(|d| d.id == c.definition)
                        })
                        || o.cards.iter().map(|c| &c.id).collect::<BTreeSet<_>>().len()
                            != o.cards.len()
                        || s.card.face_down
                        || s.play_source.is_some()
                        || (s.card.definition == "BQ077"
                            && (s.region.is_some()
                                || !o.cards.iter().any(|c| same(c, &s.card))
                                || s.card.owner != actor))
                        || (s.card.definition == "CQ26"
                            && (o.deck_seat != actor
                                || s.region.is_none()
                                || !o
                                    .cards
                                    .iter()
                                    .any(|c| catalog::card(&c.definition).kind == "character" && self.bq095_reference_at(&s.card,&o.event_id)==self.bq095_destination_at(c.owner,&o.event_id))))
                    {
                        return Err("牌库入墓观察的批次、角色或实际来源不符".into());
                    }
                    for c in &o.cards {
                        self.mobility_immutable_identity(c)?;
                    }
                }
                Op::BlackDeathRecovery { observation } => {
                    let o = observation.as_ref().ok_or("死亡恢复观察缺失")?;
                    if o.event_id.is_empty()
                        || o.turn != self.turn
                        || o.dead.id.is_empty()
                        || o.dead.face_down
                        || o.dead.owner >= self.players.len()
                        || o.dead.controller >= self.players.len()
                        || !catalog::catalog()
                            .cards
                            .iter()
                            .any(|d| d.id == o.dead.definition && d.kind == "character")
                        || o.region >= self.regions.len()
                        || o.region_instance.is_empty()
                        || o.region_instance == o.dead.id
                        || o.subtypes.iter().any(|s| s.is_empty())
                        || s.card.face_down
                        || s.play_source.is_some()
                        || (s.card.definition == "BQ085"
                            && (s.region.is_none()
                                || o.dead.controller != actor
                                || o.subtypes.iter().any(|s| s == "食尸鬼")))
                        || (s.card.definition == "XG27"
                            && (s.region.is_some()
                                || s.card.owner != actor
                                || self.team(o.dead.controller) == self.team(actor)))
                    {
                        return Err("死亡恢复的真实类别、敌我或原地区快照不符".into());
                    }
                    self.mobility_immutable_identity(&o.dead)?;
                    if self
                        .regions
                        .iter()
                        .enumerate()
                        .any(|(r, region)| region.card.id == o.region_instance && r != o.region)
                    {
                        return Err("死亡原地区UID不属于记录的地区槽位".into());
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
    pub(crate) fn validate_black_next20_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if self.validated_cross_card_declaration(d)? {
            return Ok(());
        }
        if !CARDS.contains(&d.source.card.definition.as_str()) && !related_ability(&d.ability) {
            return Ok(());
        }
        if d.ability.event == Some(Event::Prepare) {
            return crate::jc035::validate_ability(&d.source.card.definition, &d.ability);
        }
        self.black_next20_source(d.actor, &d.source)?;
        validate_ability(&d.source.card.definition, &d.ability)?;
        if crate::jz22::runtime_ability_is_admitted(&d.ability) {
            let region = d
                .source
                .source_region_instance
                .as_deref()
                .ok_or("黑色角色奖励缺少原地区UID")?;
            if catalog::card(&d.source.card.definition).kind != "character"
                || d.source.card.face_down
                || d.source.play_source.is_some()
                || d.source.region.is_none()
                || !matches!(d.ability.ops.as_slice(), [Op::PlaceInfluence { region_instance, amount: 1 }] if region_instance == region)
            {
                return Err("黑色角色获授奖励必须使用原地区快照".into());
            }
            return Ok(());
        }
        if !matches!(
            d.ability.event,
            Some(Event::DeckToGraveyardObserved | Event::BlackCharacterDeathObserved)
        ) {
            return Err("黑色五张事件声明不是准入的触发".into());
        }
        self.black_next20_observation(d.actor, &d.source, &d.ability.ops)
    }
    pub(crate) fn validate_black_next20_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if self.validated_cross_card_frame(f)? || !related(f) {
            return Ok(());
        }
        if crate::jc035::preparation_frame(f) {
            return self.validate_jc035_frame(f);
        }
        self.black_next20_source(f.actor, &f.source)?;
        if f.frame_id.is_empty() || f.cursor > f.steps.len() {
            return Err("黑色五张帧身份或游标无效".into());
        }
        let printed_payment = matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)]
            if f.money_units(0) <= catalog::card(&f.source.card.definition).cost as usize && ids.iter().all(|id| !id.is_empty()) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len());
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            if catalog::card(&f.source.card.definition).kind != "character"
                || !f.steps.is_empty()
                || !f.targets.is_empty()
                || !printed_payment
                || f.cursor != 0
                || !matches!(f.guard, GuardState::Unchecked)
                || f.chosen_region != f.source.region
                || f.source.region.is_none()
                || (f.ability_key == "deploy"
                    && (f.source.card.face_down || !f.source.valid_character_play()))
                || (f.ability_key == "reveal"
                    && (!f.source.card.face_down || f.source.play_source.is_some()))
            {
                return Err("黑色五张派遣或现身事务不符".into());
            }
            return Ok(());
        }
        let mut expected = if let Some(a) = definition(&f.source.card.definition)
            .and_then(|d| d.abilities.into_iter().find(|a| a.key == f.ability_key))
        {
            a
        } else {
            if catalog::card(&f.source.card.definition).kind != "character" {
                return Err("非角色不能获得角色奖励".into());
            }
            let region = f
                .source
                .source_region_instance
                .as_deref()
                .ok_or("黑色角色奖励缺少原地区UID")?;
            match f.ability_key.as_str() {
                "renown" => crate::renown::renown_ability(region),
                "jc089-combat-glory" => rules::jc089_combat_glory_ability(region),
                _ => return Err("黑色五张帧使用未知程序".into()),
            }
        };
        self.black_next20_observation(
            f.actor,
            &f.source,
            &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>(),
        )?;
        for (expected, actual) in expected.ops.iter_mut().zip(&f.steps) {
            match (expected, &actual.op) {
                (
                    Op::BlackDeckToGraveyard {
                        observation: expected,
                    },
                    Op::BlackDeckToGraveyard { observation },
                ) => *expected = observation.clone(),
                (
                    Op::BlackDeathRecovery {
                        observation: expected,
                    },
                    Op::BlackDeathRecovery { observation },
                ) => *expected = observation.clone(),
                _ => {}
            }
        }
        let steps = expected
            .ops
            .iter()
            .map(|op| Step {
                context: f.actor,
                op: op.clone(),
            })
            .collect::<Vec<_>>();
        let payment = match f.ability_key.as_str() {
            "attach" => printed_payment,
            "black20-xq39-discount" => {
                matches!(f.already_paid.as_slice(), [PaidCost::Exhausted(id)] if *id == f.source.card.id)
            }
            "black20-xq39-recover" => {
                matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids), PaidCost::Exhausted(id)]
                if f.money_units(0) == 2 && ids[0] != ids[1] && ids.iter().all(|id| !id.is_empty()) && *id == f.source.card.id)
            }
            "black20-xg27-return" => {
                matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if f.money_units(0) == 1 && ids.iter().all(|id| !id.is_empty()))
            }
            _ => f.already_paid.is_empty(),
        };
        if !same(&steps, &f.steps)
            || f.source.card.face_down
            || f.source.play_source.is_some()
            || !payment
            || f.chosen_region.is_some()
            || f.targets.len() != expected.targets.len()
            || f.targets.iter().zip(&expected.targets).any(|(t, spec)| {
                !same(&t.spec, spec)
                    || t.id.is_empty()
                    || t.public.instance_id != t.id
                    || !t.public.valid
                    || t.public.status != "valid"
                    || t.public.invalid_reason.is_some()
                    || (spec.zone == Zone::Region
                        && t.region_instance.as_ref().is_none_or(|id| id.is_empty()))
                    || (spec.zone != Zone::Region && t.region_instance.is_some())
            })
        {
            return Err("黑色五张完整程序、目标绑定或费用收据不符".into());
        }
        for target in &f.targets {
            match target.spec.zone {
                Zone::Region => {
                    let r = target
                        .id
                        .strip_prefix("region:")
                        .and_then(|r| r.parse::<usize>().ok())
                        .filter(|r| *r < self.regions.len() && target.id == format!("region:{r}"))
                        .ok_or("食人鱼水域目标编号无效")?;
                    if target.public.kind != "region"
                        || target.public.region != Some(r)
                        || target.public.owner.is_some()
                        || target.public.controller.is_some()
                    {
                        return Err("食人鱼原地区公开摘要不符".into());
                    }
                }
                Zone::Society => {
                    if target.id != format!("society:p{}", f.actor)
                        || target.public.kind != "society"
                        || target.public.owner.as_deref() != Some(player_id(f.actor).as_str())
                        || target.public.controller.as_deref() != Some(player_id(f.actor).as_str())
                        || target.public.region.is_some()
                    {
                        return Err("方碑印记只能绑定本人秘社".into());
                    }
                }
                Zone::Graveyard => {
                    if target.public.kind != "character"
                        || target.public.region.is_some()
                        || target.public.owner.as_deref() != Some(player_id(self.bq095_frozen_reference_seat(&f.source.card,&f.frame_id)).as_str())
                    {
                        return Err("黑色墓地恢复须绑定本人角色".into());
                    }
                }
                _ => {}
            }
        }
        if f.source.card.definition == "XQ39"
            && f.ability_key != "attach"
            && (f.source.region.is_some()
                || f.source.attachment_host_instance.as_deref()
                    != Some(format!("society:p{}", f.actor).as_str()))
        {
            return Err("方碑印记行动必须冻结原本方秘社宿主".into());
        }
        Ok(())
    }
    pub(crate) fn black_deck_recovery(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        if f.source.card.definition != "BQ077" {
            return Ok(());
        }
        let target = f.targets.first().ok_or("食人鱼缺少水域目标")?;
        let Some(r) = target
            .id
            .strip_prefix("region:")
            .and_then(|r| r.parse::<usize>().ok())
            .filter(|r| {
                self.regions.get(*r).is_some_and(|region| {
                    !region.vacant && target.region_instance.as_ref() == Some(&region.card.id)
                })
            })
        else {
            return Ok(());
        };
        self.black_grave_entry(&f.source.card,f.actor, &f.source.card.id, r, false, false)
    }
    pub(crate) fn black_death_recovery(
        &mut self,
        f: &ResolutionFrame,
        o: &BlackDeathObservation,
    ) -> RuleResult<()> {
        let (region, region_id, target, hidden) = if f.source.card.definition == "BQ085" {
            (
                f.source.region,
                f.source.source_region_instance.as_ref(),
                f.targets.first().map(|t| t.id.as_str()),
                false,
            )
        } else {
            (
                Some(o.region),
                Some(&o.region_instance),
                Some(f.source.card.id.as_str()),
                true,
            )
        };
        let Some(region) = region.filter(|r| {
            self.regions
                .get(*r)
                .is_some_and(|r| !r.vacant && region_id == Some(&r.card.id))
        }) else {
            return Ok(());
        };
        if let Some(id) = target {
            self.black_grave_entry(&f.source.card,f.actor, id, region, hidden, !hidden)?;
        }
        Ok(())
    }
    fn black_grave_entry(
        &mut self,
        source:&Card,
        actor: usize,
        id: &str,
        region: usize,
        hidden: bool,
        exhausted: bool,
    ) -> RuleResult<()> {
        if !self.bq095_reference_cards(source).iter().any(|c|c.id==id && catalog::card(&c.definition).kind=="character") {return Ok(());}
        let c=self.bq095_take_reference(source,id)?;
        let mut c=self.reset_zone_card(c);c.controller=actor;
        c.face_down = hidden;
        c.exhausted = exhausted;
        self.apply_hidden_entry_slow(&mut c);
        let id = c.id.clone();
        let definition = c.definition.clone();
        self.regions[region].cards.push(c);
        if !hidden {
            self.enter_triggers(actor, &definition, &id, false);
        }
        Ok(())
    }
    pub(crate) fn xq39_grave_to_top(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        let id = &f.targets.first().ok_or("方碑印记缺少墓地角色")?.id;
        if self.bq095_reference_cards(&f.source.card).iter().any(|c|c.id==*id && catalog::card(&c.definition).kind=="character") {
            let c=self.bq095_take_reference(&f.source.card,id)?;
            let c = self.reset_zone_card(c);
            self.players[f.actor].deck.insert(0, c);
        }
        Ok(())
    }
    pub(crate) fn bq085_upkeep_start(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        if self.bq095_reference_cards(&f.source.card).len() < 2 {
            self.remove_dead(&f.source.card.id, RemovalCause::Sacrifice);
        } else {
            let options = self.bq095_reference_cards(&f.source.card).iter()
                .map(|c| self.option(c, f.actor, None, None))
                .collect();
            self.choice(
                f.actor,
                "bq085-bottom",
                "三号骸堆：选择两张墓地牌，第一张在新底牌段上方".into(),
                options,
                2,
                2,
                None,
                ChoiceResolution::Frame {
                    frame: Box::new(f),
                    choice: FrameChoice::BQ085Upkeep,
                },
            );
        }
        Ok(())
    }
    pub(crate) fn bq085_upkeep_complete(
        &mut self,
        f: &ResolutionFrame,
        selected: &[String],
    ) -> RuleResult<()> {
        if selected.len() != 2
            || selected[0] == selected[1]
            || selected
                .iter()
                .any(|id| !self.bq095_reference_cards(&f.source.card).iter().any(|c| c.id == *id))
        {
            return Err("骸堆准备必须选择两张不同的本人墓地牌".into());
        }
        for id in selected {
            let c=self.bq095_take_reference(&f.source.card,id)?;
            let c = self.reset_zone_card(c);
            self.players[f.actor].deck.push(c);
        }
        Ok(())
    }
    pub(crate) fn validate_bq085_upkeep(&self, p: &Pending, f: &ResolutionFrame) -> RuleResult<()> {
        self.players.get(f.actor).ok_or("骸堆席位无效")?;
        let options = self.bq095_reference_cards(&f.source.card).iter()
            .map(|c| self.option(c, f.actor, None, None))
            .collect::<Vec<_>>();
        if f.source.card.definition != "BQ085"
            || f.cursor != 1
            || !matches!(f.guard, GuardState::Accepted)
            || p.seat != f.actor
            || p.choice.player_id != player_id(f.actor)
            || p.choice.id.is_empty()
            || p.choice.kind != "bq085-bottom"
            || p.choice.min != Some(2)
            || p.choice.max != Some(2)
            || p.choice.allow_decline != Some(false)
            || p.choice.amount.is_some()
            || !p.choice.preview_cards.is_empty()
            || options.len() < 2
            || !same(&options, &p.choice.options)
        {
            return Err("骸堆准备待选与原强制帧或墓地选项不符".into());
        }
        Ok(())
    }
    pub(crate) fn validate_black_next20_state(&self) -> RuleResult<()> {
        for modifier in &self.modifiers {
            if matches!(&modifier.filter, CardFilter::PrintedColor(color) if color == "黑")
                && (modifier.actor >= self.players.len()
                    || modifier.amount != 1
                    || modifier.uses > 1
                    || !modifier.paid_reveal
                    || modifier.expires_turn == 0
                    || modifier.expires_turn > self.turn)
            {
                return Err("方碑印记减费只准入本回合一次正面打出或付费翻面减1".into());
            }
        }
        let mut seen = BTreeSet::new();
        let mut remember = |source: &SourceSnapshot, ops: &[Op]| -> RuleResult<()> {
            for op in ops {
                let event = match op {
                    Op::BlackDeckToGraveyard {
                        observation: Some(o),
                    } => Some(&o.event_id),
                    Op::BlackDeathRecovery {
                        observation: Some(o),
                    } => Some(&o.event_id),
                    _ => None,
                };
                if let Some(event) = event {
                    if !seen.insert((event.clone(), source.card.id.clone())) {
                        return Err("黑色观察同一来源/事件不能重复入队".into());
                    }
                }
            }
            Ok(())
        };
        for item in &self.stack {
            if let Some(f) = &item.frame {
                self.validate_black_next20_frame(f)?;
                if !related(f) || self.validated_cross_card_frame(f)? {
                    continue;
                }
                remember(
                    &f.source,
                    &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>(),
                )?;
                let transaction = matches!(f.ability_key.as_str(), "deploy" | "reveal" | "attach");
                if item.id != f.frame_id
                    || item.controller != f.actor
                    || item.target != f.targets.first().map(|t| t.id.clone())
                    || item.reveal != (f.ability_key == "reveal")
                    || transaction != item.card.is_some()
                    || item.deploy_region
                        != (if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
                            f.chosen_region
                        } else {
                            None
                        })
                    || f.cursor != 0
                    || !matches!(f.guard, GuardState::Unchecked)
                    || item.card.as_ref().is_some_and(|c| {
                        c.definition != f.source.card.definition
                            || c.owner != f.source.card.owner
                            || c.controller != f.actor
                            || c.face_down
                    })
                {
                    return Err("黑色五张堆栈事务外壳不符".into());
                }
            } else if item
                .card
                .as_ref()
                .is_some_and(|c| CARDS.contains(&c.definition.as_str()))
            {
                return Err("黑色五张实体缺少响应帧".into());
            }
        }
        for e in &self.effects {
            match e {
                Effect::Declare { declaration: d } => {
                    self.validate_black_next20_declaration(d)?;
                    if related_ability(&d.ability) {
                        remember(&d.source, &d.ability.ops)?;
                    }
                }
                Effect::Frame { frame }
                    if related(frame) && !self.validated_cross_card_frame(frame)? =>
                {
                    return Err("黑色五张不能跳过响应保存为执行帧".into())
                }
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Rumor { batch } => {
                    let f = batch.cause.as_ref().ok_or("传闻缺少黑色观察原帧")?;
                    self.validate_black_next20_frame(f)?;
                    remember(&f.source, &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>())?;
                }
                ChoiceResolution::Declare {
                    declaration: d,
                    stage,
                } if CARDS.contains(&d.source.card.definition.as_str())
                    || related_ability(&d.ability) =>
                {
                    if self.validated_cross_card_declaration(d)? {
                        return Ok(());
                    }
                    self.validate_black_next20_declaration(d)?;
                    remember(&d.source, &d.ability.ops)?;
                    let targeted = !d.ability.targets.is_empty();
                    let expected = if let Some(slot) = d.ability.targets.first() {
                        self.binding_options(d.actor, &d.source, slot)
                    } else {
                        vec![ChoiceOption {
                            id: "accept".into(),
                            label: "发动触发能力".into(),
                            card: None,
                        }]
                    };
                    if p.seat != d.actor
                        || p.choice.player_id != player_id(d.actor)
                        || p.choice.id.is_empty()
                        || p.choice.kind != "trigger"
                        || p.choice.min != Some(0)
                        || p.choice.max != Some(1)
                        || p.choice.allow_decline != Some(true)
                        || p.choice.amount.is_some()
                        || !p.choice.preview_cards.is_empty()
                        || (targeted && !matches!(stage, DeclareChoice::Target))
                        || (!targeted && !matches!(stage, DeclareChoice::Accept))
                        || expected.is_empty()
                        || !same(&expected, &p.choice.options)
                    {
                        return Err("黑色触发待选的原目标或阶段不符".into());
                    }
                }
                ChoiceResolution::Frame { frame: f, choice } if related(f) => {
                    if self.validated_cross_card_frame(f)? {
                        return Ok(());
                    }
                    self.validate_black_next20_frame(f)?;
                    remember(
                        &f.source,
                        &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>(),
                    )?;
                    match choice {
                        FrameChoice::BQ085Upkeep => self.validate_bq085_upkeep(p, f)?,
                        FrameChoice::Discard { seat, redraw } => {
                            let expected = self
                                .players
                                .get(*seat)
                                .ok_or("莉芮尔弃牌席位无效")?
                                .hand
                                .iter()
                                .map(|c| self.option(c, *seat, None, None))
                                .collect::<Vec<_>>();
                            if f.ability_key != "black20-cq26-loot"
                                || *seat != f.actor
                                || !redraw
                                || f.cursor != 2
                                || !matches!(f.guard, GuardState::Accepted)
                                || p.seat != *seat
                                || p.choice.player_id != player_id(*seat)
                                || p.choice.kind != "discard"
                                || p.choice.min != Some(0)
                                || p.choice.max != Some(1)
                                || p.choice.allow_decline != Some(true)
                                || p.choice.amount.is_some()
                                || !p.choice.preview_cards.is_empty()
                                || expected.is_empty()
                                || !same(&expected, &p.choice.options)
                            {
                                return Err("莉芮尔可选弃一抓一续体不符".into());
                            }
                        }
                        _ => return Err("黑色五张不支持其他帧内选择".into()),
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
