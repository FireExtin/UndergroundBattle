//! Finite typed rule declarations. Card identifiers occur only in this binding table.
use crate::model::Icons;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::OnceLock};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Timing {
    Standard,
    Fast,
    ActionFast,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponsePolicy {
    #[default]
    Respondable,
    Immediate,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    Enter,
    EnterRegion,
    Reveal,
    Death,
    ConfrontationStart,
    RegionWon,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Cost {
    Assets(u32),
    ExhaustSource,
    SacrificeSource,
    SacrificeSelectedControlledCharacter,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Zone {
    Board,
    Graveyard,
    Player,
    Region,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityKind {
    Any,
    Character,
    Attachment,
    Hidden,
    CharacterOrHidden,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Relation {
    Any,
    ControlledByActor,
    OwnedByActor,
    FriendlyTeam,
    EnemyTeam,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Range {
    Anywhere,
    SourceRegion,
    Mobility,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AttachmentHostCondition {
    CharacterOrActorAssetDomain { magic: MagicIcon },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TargetSlotSpec {
    pub zone: Zone,
    pub kind: EntityKind,
    pub relation: Relation,
    pub range: Range,
    pub subtype: Option<String>,
    #[serde(default)]
    pub subtypes_any: Vec<String>,
    #[serde(default)]
    pub printed_cost_max: Option<u32>,
    #[serde(default)]
    pub equipment_host: bool,
    #[serde(default)]
    pub requires_magic: bool,
    #[serde(default)]
    pub exclude_source: bool,
    #[serde(default)]
    pub attachment_host_condition: Option<AttachmentHostCondition>,
    pub min: usize,
    pub max: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityRef {
    Source,
    Target(usize),
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum RegionRef {
    SourceRegion,
    Target(usize),
    Chosen,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoardSelector {
    pub kind: EntityKind,
    pub relation: Relation,
    pub region: Option<RegionRef>,
    pub subtype: Option<String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerRef {
    Actor,
    Context,
    Target(usize),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Destination {
    OwnerHand,
    OwnerDeckBottom,
    ActorHand,
    HiddenInChosenRegion,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeckEnd {
    Top,
    Bottom,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MagicIcon {
    None,
    Blood,
    Mind,
    Death,
    Other(String),
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CardFilter {
    Any,
    Kind(String),
    SocietyOrMagic {
        society: String,
        magic: MagicIcon,
    },
    PrintedCostAndSubtypes {
        max_cost: u32,
        subtypes: Vec<String>,
    },
}
impl CardFilter {
    pub(crate) fn matches(&self, definition: &crate::catalog::CardDefinition) -> bool {
        match self {
            Self::Any => true,
            Self::Kind(kind) => definition.kind == *kind,
            Self::SocietyOrMagic { society, magic } => {
                definition.society == *society || definition.magic_icon == *magic
            }
            Self::PrintedCostAndSubtypes { max_cost, subtypes } => {
                definition.cost <= *max_cost
                    && subtypes.iter().any(|subtype| {
                        definition.subtypes.iter().any(|printed| {
                            // Early admitted spells retain the exact legacy "事务-法术" label.
                            printed == subtype
                                || printed.strip_prefix("事务-") == Some(subtype.as_str())
                        })
                    })
            }
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Op {
    Exhaust(EntityRef),
    // A finite single-board-target query, evaluated only after the frame guard.
    // Branches are restricted to existing atomic Exhaust / OwnerHand operations.
    IfTargetExhausted {
        slot: usize,
        exhausted: Box<Op>,
        ready: Box<Op>,
    },
    HealWounds(EntityRef),
    Move(EntityRef, Destination),
    Hide(EntityRef),
    MoveOnBoard {
        entity: EntityRef,
        region: RegionRef,
    },
    Destroy(EntityRef),
    SacrificeChosen(PlayerRef),
    SacrificeAndDraw(PlayerRef),
    FreeReveal {
        player: PlayerRef,
        require_loyalty: bool,
    },
    ChooseRegion,
    GraveyardEntry {
        player: PlayerRef,
        region: RegionRef,
    },
    HideMatching {
        except_subtype: Option<String>,
        mix_hidden: bool,
    },
    SimultaneousSearch {
        filter: CardFilter,
    },
    Draw {
        player: PlayerRef,
        count: usize,
        end: DeckEnd,
    },
    // MSJC09's post-colon condition is evaluated by the paid frame at resolution.
    // This is deliberately not a general conditional-activation permission.
    DrawIfActorHasInitiative {
        count: usize,
    },
    Forecast {
        player: PlayerRef,
        count: usize,
    },
    Discard {
        player: PlayerRef,
        count: Option<usize>,
        redraw: bool,
        optional: bool,
    },
    Search {
        player: PlayerRef,
        filter: CardFilter,
        to_top: bool,
        optional: bool,
    },
    ExhaustMatching(BoardSelector),
    DamageMatching {
        selector: BoardSelector,
        amount: u32,
    },
    SkipRegionThisRound(RegionRef),
    MoveBottomToHand {
        player: PlayerRef,
        count: usize,
    },
    CostReduction {
        filter: CardFilter,
        amount: u32,
    },
    ForEachLivingPlayer(Vec<Op>),
    ForEachLivingPlayerFromActor(Vec<Op>),
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mode {
    pub key: String,
    pub label: String,
    pub targets: Vec<TargetSlotSpec>,
    pub ops: Vec<Op>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AbilitySpec {
    pub key: String,
    pub label: String,
    pub timing: Timing,
    #[serde(default)]
    pub response_policy: ResponsePolicy,
    pub costs: Vec<Cost>,
    pub targets: Vec<TargetSlotSpec>,
    pub ops: Vec<Op>,
    pub event: Option<Event>,
    pub modes: Vec<Mode>,
    pub requires_ready_source: bool,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Traits {
    pub public: bool,
    pub barrier: bool,
    pub guard: u32,
    pub kill: u32,
    pub retreat: bool,
    pub unlimited_copies: bool,
    #[serde(default)]
    pub cannot_be_equipped: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StaticModifier {
    NoEnemyCharacters(Icons, Icons),
    OtherFriendlyCharactersDefense(u32),
    ConditionalIcons {
        condition: IconCondition,
        permanent: Icons,
        temporary: Icons,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum IconCondition {
    AssetDomain { magic: MagicIcon, minimum: usize },
    RegionInfluence { friendly: bool, minimum: u32 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostLeaveDestination {
    OwnerGraveyard,
    OwnerHand,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AttachmentSpec {
    pub host: TargetSlotSpec,
    pub host_icons: Icons,
    pub host_leaves: HostLeaveDestination,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Definition {
    pub traits: Traits,
    pub abilities: Vec<AbilitySpec>,
    pub modifiers: Vec<StaticModifier>,
    pub attachment: Option<AttachmentSpec>,
    pub graveyard_face_up: bool,
}

// These are interpreter limits, not rules for resolving partially invalid targets.
// Reject unsupported declarations before publishing actions or offering trigger choices.
pub(crate) fn validate_ability(card_id: &str, ability: &AbilitySpec) -> Result<(), String> {
    fn validate_program(
        location: &str,
        targets: &[TargetSlotSpec],
        ops: &[Op],
    ) -> Result<(), String> {
        if targets.len() > 1 {
            return Err(format!(
                "{location}: unsupported target slots {}; only zero or one slot is supported",
                targets.len()
            ));
        }
        for (index, slot) in targets.iter().enumerate() {
            if slot.min != 1 || slot.max != 1 {
                return Err(format!(
                    "{location}: target slot {index} requires min=max=1, got min={} max={}",
                    slot.min, slot.max
                ));
            }
            if let Some(AttachmentHostCondition::CharacterOrActorAssetDomain { magic }) =
                &slot.attachment_host_condition
            {
                if slot.zone != Zone::Board
                    || slot.kind != EntityKind::Attachment
                    || *magic == MagicIcon::None
                {
                    return Err(format!(
                        "{location}: attachment host condition requires a board attachment and a real asset domain"
                    ));
                }
            }
        }
        for op in ops {
            if let Op::IfTargetExhausted {
                slot,
                exhausted,
                ready,
            } = op
            {
                let valid_target = targets.get(*slot).is_some_and(|target| {
                    target.zone == Zone::Board && target.kind == EntityKind::Character
                });
                let valid_branch = |branch: &Op| match branch {
                    Op::Exhaust(EntityRef::Target(index))
                    | Op::Move(EntityRef::Target(index), Destination::OwnerHand) => index == slot,
                    _ => false,
                };
                if !valid_target || !valid_branch(exhausted) || !valid_branch(ready) {
                    return Err(format!(
                        "{location}: exhausted-state branch requires one bound board character and atomic operations on that same target"
                    ));
                }
            }
            if let Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) = op {
                if body.iter().any(|op| {
                    matches!(
                        op,
                        Op::ForEachLivingPlayer(_)
                            | Op::ForEachLivingPlayerFromActor(_)
                            | Op::IfTargetExhausted { .. }
                    )
                }) {
                    return Err(format!(
                        "{location}: nested ForEachLivingPlayer is unsupported"
                    ));
                }
            }
        }
        Ok(())
    }
    let location = format!("cardId={card_id} abilityKey={}", ability.key);
    validate_program(&location, &ability.targets, &ability.ops)?;
    for mode in &ability.modes {
        validate_program(
            &format!("{location} modeKey={}", mode.key),
            &mode.targets,
            &mode.ops,
        )?;
    }
    Ok(())
}

fn validate_definitions(definitions: &BTreeMap<String, Definition>) -> Result<(), String> {
    for (card_id, definition) in definitions {
        for ability in &definition.abilities {
            validate_ability(card_id, ability)?;
        }
        if let Some(attachment) = &definition.attachment {
            let host = &attachment.host;
            if host.zone != Zone::Board
                || host.kind != EntityKind::Character
                || host.min != 1
                || host.max != 1
                || host.range != Range::Anywhere
            {
                return Err(format!(
                    "cardId={card_id}: unsupported attachment host specification"
                ));
            }
            if !definition.abilities.iter().any(|a| {
                a.event.is_none()
                    && a.timing == Timing::Standard
                    && a.targets.len() == 1
                    && serde_json::to_value(&a.targets[0]).unwrap()
                        == serde_json::to_value(host).unwrap()
            }) {
                return Err(format!(
                    "cardId={card_id}: attachment requires matching standard play target"
                ));
            }
        }
    }
    Ok(())
}

fn target(zone: Zone, kind: EntityKind, relation: Relation, range: Range) -> TargetSlotSpec {
    TargetSlotSpec {
        zone,
        kind,
        relation,
        range,
        subtype: None,
        subtypes_any: vec![],
        printed_cost_max: None,
        equipment_host: false,
        requires_magic: false,
        exclude_source: false,
        attachment_host_condition: None,
        min: 1,
        max: 1,
    }
}
fn ability(
    key: &str,
    label: &str,
    timing: Timing,
    costs: Vec<Cost>,
    targets: Vec<TargetSlotSpec>,
    ops: Vec<Op>,
    event: Option<Event>,
) -> AbilitySpec {
    AbilitySpec {
        key: key.into(),
        label: label.into(),
        timing,
        response_policy: ResponsePolicy::Respondable,
        costs,
        targets,
        ops,
        event,
        modes: vec![],
        requires_ready_source: false,
    }
}
fn with_abilities(abilities: Vec<AbilitySpec>) -> Definition {
    Definition {
        abilities,
        ..Default::default()
    }
}

pub fn definitions() -> &'static BTreeMap<String, Definition> {
    static DEFINITIONS: OnceLock<BTreeMap<String, Definition>> = OnceLock::new();
    DEFINITIONS.get_or_init(|| {
        use EntityRef::{Source, Target};
        use PlayerRef::{Actor, Context};
        let local = target(
            Zone::Board,
            EntityKind::Character,
            Relation::Any,
            Range::SourceRegion,
        );
        let character = target(
            Zone::Board,
            EntityKind::Character,
            Relation::Any,
            Range::Anywhere,
        );
        let own_grave = target(
            Zone::Graveyard,
            EntityKind::Character,
            Relation::OwnedByActor,
            Range::Anywhere,
        );
        let player = target(
            Zone::Player,
            EntityKind::Any,
            Relation::Any,
            Range::Anywhere,
        );
        let region = target(
            Zone::Region,
            EntityKind::Any,
            Relation::Any,
            Range::Anywhere,
        );
        let mut m = BTreeMap::new();
        let mut another_own = target(
            Zone::Board,
            EntityKind::Character,
            Relation::ControlledByActor,
            Range::Anywhere,
        );
        another_own.exclude_source = true;
        m.insert(
            "JC075".into(),
            Definition {
                traits: Traits {
                    public: true,
                    ..Default::default()
                },
                abilities: vec![ability(
                    "rescue",
                    "快速行动",
                    Timing::Fast,
                    vec![Cost::Assets(2), Cost::ExhaustSource],
                    vec![another_own],
                    vec![Op::Move(Target(0), Destination::OwnerHand)],
                    None,
                )],
                ..Default::default()
            },
        );
        m.insert(
            "JC104".into(),
            with_abilities(vec![ability(
                "forecast-entry",
                "进场触发",
                Timing::Fast,
                vec![],
                vec![],
                vec![Op::Forecast {
                    player: Actor,
                    count: 3,
                }],
                Some(Event::Enter),
            )]),
        );
        m.insert(
            "JC047".into(),
            with_abilities(vec![ability(
                "damage-region",
                "标准行动",
                Timing::Standard,
                vec![],
                vec![region.clone()],
                vec![Op::DamageMatching {
                    selector: BoardSelector {
                        kind: EntityKind::Character,
                        relation: Relation::Any,
                        region: Some(RegionRef::Target(0)),
                        subtype: None,
                    },
                    amount: 1,
                }],
                None,
            )]),
        );
        m.insert(
            "JC007".into(),
            with_abilities(vec![ability(
                "search-cheap-spell-book-entry",
                "进场触发",
                Timing::Fast,
                vec![],
                vec![],
                vec![Op::Search {
                    player: Actor,
                    filter: CardFilter::PrintedCostAndSubtypes {
                        max_cost: 2,
                        subtypes: vec!["法术".into(), "书籍".into()],
                    },
                    to_top: false,
                    optional: false,
                }],
                Some(Event::Enter),
            )]),
        );
        let mut magic_character = character.clone();
        magic_character.requires_magic = true;
        let mut disintegrate_target = target(
            Zone::Board,
            EntityKind::Attachment,
            Relation::Any,
            Range::Anywhere,
        );
        disintegrate_target.attachment_host_condition =
            Some(AttachmentHostCondition::CharacterOrActorAssetDomain {
                magic: MagicIcon::Mind,
            });
        m.insert(
            "JC005".into(),
            with_abilities(vec![ability(
                "disintegrate",
                "消灭附属",
                Timing::Fast,
                vec![],
                vec![disintegrate_target],
                vec![Op::Destroy(Target(0))],
                None,
            )]),
        );
        m.insert(
            "JC006".into(),
            with_abilities(vec![ability(
                "return-magic",
                "快速行动",
                Timing::Fast,
                vec![],
                vec![magic_character],
                vec![Op::Move(Target(0), Destination::OwnerHand)],
                None,
            )]),
        );
        let mut another_friend = target(
            Zone::Board,
            EntityKind::Character,
            Relation::ControlledByActor,
            Range::Anywhere,
        );
        another_friend.exclude_source = true;
        m.insert(
            "XQ16".into(),
            Definition {
                traits: Traits {
                    public: true,
                    ..Default::default()
                },
                abilities: vec![ability(
                    "hide-friend-entry",
                    "进场触发",
                    Timing::Fast,
                    vec![],
                    vec![another_friend],
                    vec![Op::Hide(Target(0))],
                    Some(Event::Enter),
                )],
                ..Default::default()
            },
        );
        m.insert(
            "JC084".into(),
            Definition {
                modifiers: vec![
                    StaticModifier::ConditionalIcons {
                        condition: IconCondition::RegionInfluence {
                            friendly: true,
                            minimum: 1,
                        },
                        permanent: Icons::default(),
                        temporary: Icons {
                            influence: 1,
                            ..Icons::default()
                        },
                    },
                    StaticModifier::ConditionalIcons {
                        condition: IconCondition::RegionInfluence {
                            friendly: false,
                            minimum: 1,
                        },
                        permanent: Icons::default(),
                        temporary: Icons {
                            investigation: 1,
                            ..Icons::default()
                        },
                    },
                ],
                ..Definition::default()
            },
        );
        m.insert(
            "JC001".into(),
            Definition {
                traits: Traits {
                    cannot_be_equipped: true,
                    ..Default::default()
                },
                modifiers: vec![StaticModifier::ConditionalIcons {
                    condition: IconCondition::AssetDomain {
                        magic: MagicIcon::Mind,
                        minimum: 2,
                    },
                    permanent: Icons::default(),
                    temporary: Icons {
                        investigation: 1,
                        ..Icons::default()
                    },
                }],
                ..Definition::default()
            },
        );
        m.insert(
            "BQ083".into(),
            with_abilities(vec![ability(
                "destroy-local-entry",
                "进场触发",
                Timing::Fast,
                vec![],
                vec![target(
                    Zone::Board,
                    EntityKind::Character,
                    Relation::Any,
                    Range::SourceRegion,
                )],
                vec![Op::Destroy(Target(0))],
                Some(Event::Enter),
            )]),
        );
        m.insert(
            "JC085".into(),
            Definition {
                graveyard_face_up: true,
                modifiers: vec![StaticModifier::ConditionalIcons {
                    condition: IconCondition::AssetDomain {
                        magic: MagicIcon::Death,
                        minimum: 1,
                    },
                    permanent: Icons::default(),
                    temporary: Icons {
                        influence: 1,
                        ..Icons::default()
                    },
                }],
                ..Definition::default()
            },
        );
        m.insert(
            "JC125".into(),
            Definition {
                traits: Traits {
                    unlimited_copies: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        m.insert(
            "LC19".into(),
            Definition {
                traits: Traits {
                    public: true,
                    ..Default::default()
                },
                abilities: vec![ability(
                    "heal",
                    "移除创伤",
                    Timing::Standard,
                    vec![Cost::Assets(2), Cost::ExhaustSource],
                    vec![local.clone()],
                    vec![Op::HealWounds(Target(0))],
                    None,
                )],
                ..Default::default()
            },
        );
        m.insert(
            "LC20".into(),
            with_abilities(vec![ability(
                "heal-entry",
                "进场触发",
                Timing::Fast,
                vec![],
                vec![local.clone()],
                vec![Op::HealWounds(Target(0))],
                Some(Event::Enter),
            )]),
        );
        m.insert(
            "LC21".into(),
            Definition {
                traits: Traits {
                    public: true,
                    guard: 1,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        m.insert(
            "LC22".into(),
            Definition {
                traits: Traits {
                    public: true,
                    guard: 1,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        m.insert(
            "LC23".into(),
            with_abilities(vec![ability(
                "exhaust-reveal",
                "现身触发",
                Timing::Fast,
                vec![],
                vec![local.clone()],
                vec![Op::Exhaust(Source), Op::Exhaust(Target(0))],
                Some(Event::Reveal),
            )]),
        );
        m.insert(
            "LC24".into(),
            with_abilities(vec![ability(
                "forecast-entry",
                "进场触发",
                Timing::Fast,
                vec![],
                vec![],
                vec![
                    Op::Forecast {
                        player: Actor,
                        count: 2,
                    },
                    Op::Draw {
                        player: Actor,
                        count: 1,
                        end: DeckEnd::Top,
                    },
                ],
                Some(Event::Enter),
            )]),
        );
        m.insert(
            "XQ49".into(),
            with_abilities(vec![ability(
                "funeral",
                "快速行动",
                Timing::Fast,
                vec![],
                vec![target(
                    Zone::Graveyard,
                    EntityKind::Any,
                    Relation::Any,
                    Range::Anywhere,
                )],
                vec![
                    Op::Move(Target(0), Destination::OwnerDeckBottom),
                    Op::Draw {
                        player: Actor,
                        count: 1,
                        end: DeckEnd::Top,
                    },
                ],
                None,
            )]),
        );
        m.insert(
            "JC118".into(),
            with_abilities(vec![ability(
                "ceasefire",
                "快速行动",
                Timing::ActionFast,
                vec![],
                vec![region.clone()],
                vec![Op::SkipRegionThisRound(RegionRef::Target(0))],
                None,
            )]),
        );
        m.insert(
            "JC002".into(),
            with_abilities(vec![ability(
                "exhaust-reveal",
                "现身触发",
                Timing::Fast,
                vec![],
                vec![local.clone()],
                vec![Op::Exhaust(Target(0))],
                Some(Event::Reveal),
            )]),
        );
        m.insert(
            "JC004".into(),
            with_abilities(vec![ability(
                "exhaust-or-return-entry",
                "进场触发",
                Timing::Fast,
                vec![],
                vec![local],
                vec![Op::IfTargetExhausted {
                    slot: 0,
                    exhausted: Box::new(Op::Move(Target(0), Destination::OwnerHand)),
                    ready: Box::new(Op::Exhaust(Target(0))),
                }],
                Some(Event::Enter),
            )]),
        );
        m.insert(
            "JC003".into(),
            with_abilities(vec![ability(
                "exhaust",
                "快速行动",
                Timing::Fast,
                vec![Cost::Assets(2), Cost::ExhaustSource],
                vec![target(
                    Zone::Board,
                    EntityKind::CharacterOrHidden,
                    Relation::Any,
                    Range::Anywhere,
                )],
                vec![Op::Exhaust(Target(0))],
                None,
            )]),
        );
        m.insert(
            "XQ03".into(),
            with_abilities(vec![ability(
                "exhaust",
                "快速行动",
                Timing::Fast,
                vec![],
                vec![character.clone()],
                vec![Op::Exhaust(Target(0))],
                None,
            )]),
        );
        let mut mobility = ability(
            "mobility",
            "机动",
            Timing::Fast,
            vec![],
            vec![target(
                Zone::Region,
                EntityKind::Any,
                Relation::Any,
                Range::Mobility,
            )],
            vec![Op::MoveOnBoard {
                entity: Source,
                region: RegionRef::Target(0),
            }],
            Some(Event::ConfrontationStart),
        );
        mobility.requires_ready_source = true;
        m.insert(
            "JC014".into(),
            Definition {
                traits: Traits {
                    public: true,
                    ..Default::default()
                },
                abilities: vec![mobility],
                ..Default::default()
            },
        );
        m.insert(
            "JC016".into(),
            Definition {
                traits: Traits {
                    kill: 1,
                    retreat: true,
                    ..Default::default()
                },
                modifiers: vec![StaticModifier::NoEnemyCharacters(
                    Icons {
                        influence: 1,
                        ..Default::default()
                    },
                    Icons {
                        investigation: 1,
                        ..Default::default()
                    },
                )],
                ..Default::default()
            },
        );
        m.insert(
            "JZ08".into(),
            Definition {
                traits: Traits {
                    barrier: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        m.insert(
            "JC056".into(),
            Definition {
                traits: Traits {
                    public: true,
                    ..Default::default()
                },
                abilities: vec![ability(
                    "exhaust-hidden-entry",
                    "进场触发",
                    Timing::Fast,
                    vec![],
                    vec![],
                    vec![Op::ExhaustMatching(BoardSelector {
                        kind: EntityKind::Hidden,
                        relation: Relation::Any,
                        region: Some(RegionRef::SourceRegion),
                        subtype: None,
                    })],
                    Some(Event::Enter),
                )],
                ..Default::default()
            },
        );
        m.insert(
            "JC058".into(),
            with_abilities(vec![ability(
                "destroy-hidden-reveal",
                "现身触发",
                Timing::Fast,
                vec![],
                vec![target(
                    Zone::Board,
                    EntityKind::Hidden,
                    Relation::Any,
                    Range::SourceRegion,
                )],
                vec![Op::Destroy(Target(0))],
                Some(Event::Reveal),
            )]),
        );
        m.insert(
            "JC059".into(),
            Definition {
                traits: Traits {
                    public: true,
                    ..Default::default()
                },
                modifiers: vec![StaticModifier::OtherFriendlyCharactersDefense(1)],
                ..Default::default()
            },
        );
        let mut chase = ability(
            "chase",
            "快速行动",
            Timing::Fast,
            vec![],
            vec![],
            vec![],
            None,
        );
        chase.modes = vec![
            Mode {
                key: "hide".into(),
                label: "潜伏".into(),
                targets: vec![character.clone()],
                ops: vec![Op::Hide(Target(0))],
            },
            Mode {
                key: "return".into(),
                label: "回手".into(),
                targets: vec![target(
                    Zone::Board,
                    EntityKind::Hidden,
                    Relation::Any,
                    Range::Anywhere,
                )],
                ops: vec![Op::Move(Target(0), Destination::OwnerHand)],
            },
        ];
        m.insert("JC063".into(), with_abilities(vec![chase]));
        let mut cheap_character = target(
            Zone::Board,
            EntityKind::Character,
            Relation::Any,
            Range::SourceRegion,
        );
        cheap_character.printed_cost_max = Some(2);
        m.insert(
            "JC088".into(),
            with_abilities(vec![
                ability(
                    "destroy-cheap-reveal",
                    "现身触发",
                    Timing::Fast,
                    vec![],
                    vec![cheap_character],
                    vec![Op::Destroy(Target(0))],
                    Some(Event::Reveal),
                ),
                ability(
                    "hide-self",
                    "快速行动",
                    Timing::Fast,
                    vec![Cost::Assets(2)],
                    vec![],
                    vec![Op::Hide(Source)],
                    None,
                ),
            ]),
        );
        m.insert(
            "JC086".into(),
            with_abilities(vec![ability(
                "recover-entry",
                "进场触发",
                Timing::Fast,
                vec![],
                vec![own_grave.clone()],
                vec![Op::Move(Target(0), Destination::ActorHand)],
                Some(Event::Enter),
            )]),
        );
        m.insert(
            "XQ12".into(),
            with_abilities(vec![ability(
                "discard-death",
                "死亡触发",
                Timing::Fast,
                vec![],
                vec![player.clone()],
                vec![Op::Discard {
                    player: PlayerRef::Target(0),
                    count: Some(1),
                    redraw: false,
                    optional: false,
                }],
                Some(Event::Death),
            )]),
        );
        m.insert(
            "JC092".into(),
            with_abilities(vec![ability(
                "summon",
                "行动",
                Timing::Standard,
                vec![],
                vec![own_grave],
                vec![Op::Move(Target(0), Destination::HiddenInChosenRegion)],
                None,
            )]),
        );
        let mut reduce_next = ability(
            "reduce-next",
            "快速行动",
            Timing::Fast,
            vec![Cost::SacrificeSource],
            vec![],
            vec![Op::CostReduction {
                filter: CardFilter::SocietyOrMagic {
                    society: "鸣钟教派".into(),
                    magic: MagicIcon::Blood,
                },
                amount: 2,
            }],
            None,
        );
        reduce_next.response_policy = ResponsePolicy::Immediate;
        m.insert("JC042".into(), with_abilities(vec![reduce_next]));
        let mut human = character;
        human.subtype = Some("人类".into());
        m.insert(
            "JC091".into(),
            with_abilities(vec![ability(
                "destroy-human",
                "快速行动",
                Timing::ActionFast,
                vec![],
                vec![human],
                vec![Op::Destroy(Target(0))],
                None,
            )]),
        );
        m.insert(
            "JC049".into(),
            with_abilities(vec![ability(
                "take-bottom",
                "快速行动",
                Timing::Fast,
                vec![Cost::SacrificeSelectedControlledCharacter],
                vec![],
                vec![Op::MoveBottomToHand {
                    player: Actor,
                    count: 2,
                }],
                None,
            )]),
        );
        m.insert(
            "JZ54".into(),
            with_abilities(vec![ability(
                "sacrifice-player",
                "快速行动",
                Timing::Fast,
                vec![],
                vec![player],
                vec![Op::SacrificeChosen(PlayerRef::Target(0))],
                None,
            )]),
        );
        m.insert(
            "DQJC107".into(),
            with_abilities(vec![ability(
                "search-win",
                "赢取触发",
                Timing::Fast,
                vec![],
                vec![],
                vec![Op::ForEachLivingPlayer(vec![Op::Search {
                    player: Context,
                    filter: CardFilter::Any,
                    to_top: true,
                    optional: false,
                }])],
                Some(Event::RegionWon),
            )]),
        );
        m.insert(
            "DQJC112".into(),
            with_abilities(vec![ability(
                "swap-win",
                "赢取触发",
                Timing::Fast,
                vec![],
                vec![],
                vec![Op::ForEachLivingPlayer(vec![Op::Discard {
                    player: Context,
                    count: None,
                    redraw: true,
                    optional: true,
                }])],
                Some(Event::RegionWon),
            )]),
        );
        m.insert(
            "DQJC113".into(),
            with_abilities(vec![ability(
                "damage-win",
                "赢取触发",
                Timing::Fast,
                vec![],
                vec![],
                vec![Op::DamageMatching {
                    selector: BoardSelector {
                        kind: EntityKind::Character,
                        relation: Relation::Any,
                        region: None,
                        subtype: None,
                    },
                    amount: 1,
                }],
                Some(Event::RegionWon),
            )]),
        );
        let mut worldmode = ability(
            "mode-win",
            "赢取触发",
            Timing::Fast,
            vec![],
            vec![],
            vec![],
            Some(Event::RegionWon),
        );
        worldmode.modes = vec![
            Mode {
                key: "draw".into(),
                label: "所有玩家抓两张牌".into(),
                targets: vec![],
                ops: vec![Op::ForEachLivingPlayer(vec![Op::Draw {
                    player: Context,
                    count: 2,
                    end: DeckEnd::Top,
                }])],
            },
            Mode {
                key: "discard".into(),
                label: "所有玩家各弃两张牌".into(),
                targets: vec![],
                ops: vec![Op::ForEachLivingPlayer(vec![Op::Discard {
                    player: Context,
                    count: Some(2),
                    redraw: false,
                    optional: false,
                }])],
            },
        ];
        m.insert("DQJC114".into(), with_abilities(vec![worldmode]));
        for (id, key, ops) in [
            // Adopt the archived Florence FAQ for this release (user ruling).
            (
                "DQJC108",
                "free-reveal-win",
                vec![Op::ForEachLivingPlayerFromActor(vec![Op::FreeReveal {
                    player: Context,
                    require_loyalty: false,
                }])],
            ),
            (
                "DQJC109",
                "nonhuman-hide-win",
                vec![Op::HideMatching {
                    except_subtype: Some("人类".into()),
                    mix_hidden: false,
                }],
            ),
            (
                "DQJC110",
                "hide-and-mix-win",
                vec![Op::HideMatching {
                    except_subtype: None,
                    mix_hidden: true,
                }],
            ),
            (
                "DQJC111",
                "sacrifice-draw-win",
                vec![Op::ForEachLivingPlayerFromActor(vec![
                    Op::SacrificeAndDraw(Context),
                ])],
            ),
            (
                "DQJC115",
                "graveyard-entry-win",
                vec![
                    Op::ChooseRegion,
                    Op::ForEachLivingPlayerFromActor(vec![Op::GraveyardEntry {
                        player: Context,
                        region: RegionRef::Chosen,
                    }]),
                ],
            ),
            (
                "DQJC116",
                "simultaneous-search-win",
                vec![Op::SimultaneousSearch {
                    filter: CardFilter::Kind("attachment".into()),
                }],
            ),
        ] {
            m.insert(
                id.into(),
                with_abilities(vec![ability(
                    key,
                    "赢取触发",
                    Timing::Fast,
                    vec![],
                    vec![],
                    ops,
                    Some(Event::RegionWon),
                )]),
            );
        }
        let mut host = target(
            Zone::Board,
            EntityKind::Character,
            Relation::Any,
            Range::Anywhere,
        );
        host.subtypes_any = vec!["人类".into(), "吸血鬼".into()];
        host.equipment_host = true;
        m.insert(
            "BQ022".into(),
            Definition {
                abilities: vec![ability(
                    "attach",
                    "结附",
                    Timing::Standard,
                    vec![],
                    vec![host.clone()],
                    vec![],
                    None,
                )],
                attachment: Some(AttachmentSpec {
                    host,
                    host_icons: Icons {
                        combat: 1,
                        ..Default::default()
                    },
                    host_leaves: HostLeaveDestination::OwnerHand,
                }),
                ..Default::default()
            },
        );
        #[cfg(not(feature = "society-fixtures"))]
        m.insert(
            "MSJC09".into(),
            with_abilities(vec![ability(
                "drawWithInitiative",
                "先手抓牌",
                Timing::Standard,
                vec![Cost::Assets(3), Cost::ExhaustSource],
                vec![],
                vec![Op::DrawIfActorHasInitiative { count: 1 }],
                None,
            )]),
        );
        #[cfg(feature = "society-fixtures")]
        for id in [
            "FIXTURE_SOCIETY_SIX",
            "FIXTURE_SOCIETY_FOUR",
            "FIXTURE_SOCIETY_PENDING",
        ] {
            m.insert(
                id.into(),
                with_abilities(vec![ability(
                    "fixture-draw",
                    "内部基础验证：抓1",
                    Timing::Standard,
                    vec![Cost::Assets(1), Cost::ExhaustSource],
                    vec![],
                    vec![Op::Draw {
                        player: Actor,
                        count: 1,
                        end: DeckEnd::Top,
                    }],
                    None,
                )]),
            );
        }
        validate_definitions(&m)
            .unwrap_or_else(|error| panic!("Invalid released rule declaration: {error}"));
        m
    })
}
pub fn definition(id: &str) -> &'static Definition {
    definitions()
        .get(id)
        .expect("released card has complete typed rule declaration")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_multi_target_ability_is_rejected_before_registration() {
        let mut registry = definitions().clone();
        assert_eq!(
            registry.len(),
            crate::catalog::catalog().cards.len() + crate::catalog::catalog().societies.len()
        );
        assert!(registry.contains_key("JC001") && registry.contains_key("BQ083"));
        let ability = &mut registry.get_mut("LC20").unwrap().abilities[0];
        ability.targets.push(ability.targets[0].clone());
        let error = validate_definitions(&registry).unwrap_err();
        assert!(error.contains("cardId=LC20 abilityKey=heal-entry"));
        assert!(error.contains("unsupported target slots 2"));

        let ability = &mut registry.get_mut("LC20").unwrap().abilities[0];
        ability.targets.pop();
        ability.targets[0].min = 0;
        ability.targets[0].max = 2;
        let error = validate_definitions(&registry).unwrap_err();
        assert!(error.contains("cardId=LC20 abilityKey=heal-entry"));
        assert!(error.contains("requires min=max=1, got min=0 max=2"));
    }

    #[test]
    fn invalid_mode_targets_and_nested_iteration_are_rejected_before_registration() {
        let mut registry = definitions().clone();
        let mode = &mut registry.get_mut("JC063").unwrap().abilities[0].modes[0];
        mode.targets.push(mode.targets[0].clone());
        let error = validate_definitions(&registry).unwrap_err();
        assert!(error.contains("cardId=JC063 abilityKey=chase modeKey=hide"));
        assert!(error.contains("unsupported target slots 2"));

        let mode = &mut registry.get_mut("JC063").unwrap().abilities[0].modes[0];
        mode.targets.pop();
        mode.targets[0].max = 2;
        let error = validate_definitions(&registry).unwrap_err();
        assert!(error.contains("modeKey=hide"));
        assert!(error.contains("requires min=max=1, got min=1 max=2"));

        let mode = &mut registry.get_mut("JC063").unwrap().abilities[0].modes[0];
        mode.targets[0].max = 1;
        mode.ops = vec![Op::ForEachLivingPlayer(vec![Op::ForEachLivingPlayer(
            vec![],
        )])];
        let error = validate_definitions(&registry).unwrap_err();
        assert!(error.contains("cardId=JC063 abilityKey=chase modeKey=hide"));
        assert!(error.contains("nested ForEachLivingPlayer is unsupported"));
    }
}
