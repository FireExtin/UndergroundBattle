//! Finite typed rule declarations. Card identifiers occur only in this binding table.
use crate::model::{Icons, SubtypeChange};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::OnceLock};

fn is_zero(value: &u32) -> bool {
    *value == 0
}

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
    HandDiscard,
    Enter,
    EnterRegion,
    Reveal,
    Death,
    ReceiveWound,
    ConfrontationStart,
    RegionWon,
    RegionConfrontationsEnded,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Cost {
    Assets(u32),
    ExhaustSource,
    SacrificeSource,
    SacrificeSelectedControlledCharacter,
    DiscardSelectedHandCard,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Zone {
    AttachmentOrAsset,
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
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetPredicate {
    JC015NonHumanPrintedCostAtLeastThree,
    JZ55UniqueCharacter,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TargetSlotSpec {
    pub zone: Zone,
    pub kind: EntityKind,
    pub relation: Relation,
    pub range: Range,
    pub subtype: Option<String>,
    #[serde(default)]
    pub printed_subtype: bool,
    #[serde(default)]
    pub subtypes_any: Vec<String>,
    #[serde(default)]
    pub printed_cost_max: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub predicate: Option<TargetPredicate>,
    #[serde(default)]
    pub equipment_host: bool,
    #[serde(default)]
    pub requires_magic: bool,
    #[serde(default)]
    pub exclude_source: bool,
    #[serde(default, skip_serializing_if = "crate::model::is_false")]
    pub exclude_attachment_host: bool,
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
    NamedCharacter(String),
    PrintedColor(String),
    Any,
    HasPrintedMagic,
    Kind(String),
    PrintedColorAndUnique {
        color: String,
    },
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
            Self::PrintedColor(color) => definition.color == *color,
            Self::Any => true,
            Self::HasPrintedMagic => definition.magic_icon != MagicIcon::None,
            Self::Kind(kind) => definition.kind == *kind,
            Self::PrintedColorAndUnique { color } => {
                definition.color == *color && definition.unique
            }
            Self::SocietyOrMagic { society, magic } => {
                definition.society == *society || definition.magic_icon == *magic
            }
            Self::NamedCharacter(name) => {
                definition.kind == "character" && definition.name == *name
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchVisibility {
    #[default]
    Reveal,
    Private,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Op {
    // JZ31 only: one influence in the frozen source region, without parameters.
    PlaceOneInfluenceInSourceRegion,
    // Two closed blue programs, with no configurable search/loop interpreter.
    JC032TopSixVampireHidden,
    JZ24LocalSacrificeSnapshot,
    // Finite MSJC11 programs; no generic keyword or quantity interpreter.
    GrantTargetKillUntilTurnEnd {
        slot: usize,
    },
    GrantRegionRetreatUntilTurnEnd {
        slot: usize,
    },
    RepressOpponentOne {
        slot: usize,
    },
    // JC126 only: resolve the actor's current top card into their asset area.
    MoveActorDeckTopToAsset,
    WoundTarget {
        slot: usize,
        amount: u32,
    },
    DestroyPublicAttachmentOrAsset {
        slot: usize,
    },
    CostReductionOnFaceUpOrPaidReveal {
        filter: CardFilter,
        amount: u32,
    },
    RevealHandAndOfferSourceSacrifice {
        player: PlayerRef,
    },
    RandomHandToOwnerDeckTop {
        player: PlayerRef,
    },
    MoveDeckTopToGraveyard {
        player: PlayerRef,
        count: usize,
    },
    OfferShuffleIfActorControlsDreamDemon {
        player: PlayerRef,
    },
    GainControl {
        slot: usize,
        until_source_leaves: bool,
        subtype_change: SubtypeChange,
    },
    Exhaust(EntityRef),
    DamageTarget {
        slot: usize,
        amount: u32,
    },
    PreventTargetDamageUntilTurnEnd {
        slot: usize,
    },
    ReattachSource {
        slot: usize,
    },
    ModifyTargetUntilTurnEnd {
        slot: usize,
        defense_bonus: u32,
        ordinary_icons: Icons,
        #[serde(default, skip_serializing_if = "crate::model::is_false")]
        grants_renown: bool,
    },
    // The admitted weapon grants exactly these two ordinary icons to its
    // snapshotted host, without targeting or consulting its later attachment.
    ModifyAttachmentHostUntilTurnEnd,
    // Non-targeted resolution-time selection in the snapshotted source region.
    SetLocalMagicPrintedDefenseToOneUntilTurnEnd,
    // Exact region identity prevents an already declared reward reaching a
    // replacement region. Only the finite merged renown trigger uses this op.
    PlaceInfluence {
        region_instance: String,
        amount: u32,
    },
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
        #[serde(default)]
        visibility: SearchVisibility,
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
    #[serde(default, skip_serializing_if = "crate::model::is_false")]
    pub play_only: bool,
    #[serde(default, skip_serializing_if = "crate::model::is_false")]
    pub activation_only: bool,
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
    #[serde(default)]
    pub once_per_game: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub per_turn_limit: Option<u32>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Traits {
    #[serde(default, skip_serializing_if = "crate::model::is_false")]
    pub slow: bool,
    #[serde(default, skip_serializing_if = "crate::model::is_false")]
    pub spirit: bool,
    #[serde(default, skip_serializing_if = "crate::model::is_false")]
    pub renown: bool,
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
    JZ48OtherControlledCriminalInfluenceAndDefense,
    JC030BloodAssetsVampireAndInvestigation,
    LC30WeaponsCombatAndDefense,
    JC018MindAssetsCombat,
    AttachedRegionSpiritTemporaryIconsPermanent,
    PeekOwnDeckTop,
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
    #[serde(default)]
    pub controls_host: bool,
    #[serde(default)]
    pub host_subtype_change: SubtypeChange,
    pub host: TargetSlotSpec,
    pub host_icons: Icons,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_temporary_icons: Option<Icons>,
    #[serde(default, skip_serializing_if = "crate::model::is_false")]
    pub host_barrier: bool,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub host_defense_bonus: u32,
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
    let two_card_mill = ability.ops.iter().chain(ability.modes.iter().flat_map(|m| &m.ops))
        .any(|op| matches!(op, Op::MoveDeckTopToGraveyard { count: 2, .. }));
    if (card_id == "JZ49" || ability.key == "mill-two-entry" || two_card_mill)
        && (card_id != "JZ49" || serde_json::to_value(ability).unwrap()
            != serde_json::to_value(&jz49_definition().abilities[0]).unwrap())
    {
        return Err(format!("{card_id}: only the complete JZ49 entry mill is admitted"));
    }
    let death_influence = ability.ops.iter().chain(ability.modes.iter().flat_map(|m| &m.ops))
        .any(|op| matches!(op, Op::PlaceOneInfluenceInSourceRegion));
    if (card_id == "JZ31" || death_influence)
        && (card_id != "JZ31" || serde_json::to_value(ability).unwrap()
            != serde_json::to_value(&jz31_definition().abilities[0]).unwrap())
    {
        return Err(format!("{card_id}: only the complete JZ31 death influence ability is admitted"));
    }
    // Only XQ36 may reuse a player-loop mill, with its complete fixed program.
    // Do not relax the existing top-level target-player requirement.
    fn has_loop_mill(ops: &[Op], in_loop: bool) -> bool {
        ops.iter().any(|op| match op {
            Op::MoveDeckTopToGraveyard { .. } => in_loop,
            Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => {
                has_loop_mill(body, true)
            }
            _ => false,
        })
    }
    if (card_id == "XQ36"
        || has_loop_mill(&ability.ops, false)
        || ability.modes.iter().any(|m| has_loop_mill(&m.ops, false)))
        && serde_json::to_value(ability).unwrap()
            != serde_json::to_value(&xq36_definition().abilities[0]).unwrap()
    {
        return Err(format!("{card_id}: only the complete XQ36 entry mill is admitted"));
    }
    if card_id != "XQ36"
        && (has_loop_mill(&ability.ops, false)
            || ability.modes.iter().any(|m| has_loop_mill(&m.ops, false)))
    {
        return Err(format!("{card_id}: XQ36 player-loop mill cannot be transplanted"));
    }
    if ability.modes.iter().any(|mode| mode.ops.iter().any(|op| {
        matches!(op, Op::JC032TopSixVampireHidden | Op::JZ24LocalSacrificeSnapshot)
    })) {
        return Err(format!("{card_id}: blue finite operations cannot be mode operations"));
    }
    let finite_predicate = ability
        .targets
        .iter()
        .chain(ability.modes.iter().flat_map(|m| &m.targets))
        .any(|t| t.predicate.is_some());
    if finite_predicate || matches!(card_id, "JC015" | "JZ55") {
        let admitted = match card_id {
            "JC015" => Some(jc015_definition()),
            "JZ55" => Some(jz55_definition()),
            _ => None,
        };
        if admitted.is_none_or(|d| serde_json::to_value(ability).unwrap()
            != serde_json::to_value(&d.abilities[0]).unwrap()) {
            return Err(format!("{card_id}: only the complete JC015 or JZ55 target predicate ability is supported"));
        }
    }
    let execution_op = |op: &Op| {
        matches!(
            op,
            Op::GrantTargetKillUntilTurnEnd { .. } | Op::GrantRegionRetreatUntilTurnEnd { .. }
        )
    };
    if ability.modes.iter().any(|m| m.ops.iter().any(execution_op)) {
        return Err(format!(
            "{card_id}: MSJC11 grants cannot be mode operations"
        ));
    }
    if ability.ops.iter().any(execution_op) {
        let kill = matches!(
            ability.ops.as_slice(),
            [Op::GrantTargetKillUntilTurnEnd { slot: 0 }]
        );
        let retreat = matches!(
            ability.ops.as_slice(),
            [Op::GrantRegionRetreatUntilTurnEnd { slot: 0 }]
        );
        if card_id != "MSJC11"
            || (!kill && !retreat)
            || ability.key
                != if kill {
                    "grant-kill"
                } else {
                    "grant-region-retreat"
                }
            || ability.timing != Timing::Standard
            || ability.event.is_some()
            || ability.play_only
            || ability.activation_only
            || ability.requires_ready_source
            || ability.once_per_game
            || ability.per_turn_limit.is_some()
            || ability.response_policy != ResponsePolicy::Respondable
            || !ability.modes.is_empty()
            || !matches!(
                ability.costs.as_slice(),
                [Cost::Assets(3), Cost::ExhaustSource]
            )
            || ability.targets.len() != 1
            || !ability.targets.first().is_some_and(|t| {
                t.zone == if kill { Zone::Board } else { Zone::Region }
                    && t.kind
                        == if kill {
                            EntityKind::Character
                        } else {
                            EntityKind::Any
                        }
                    && t.relation == Relation::Any
                    && t.range == Range::Anywhere
                    && t.min == 1
                    && t.max == 1
                    && t.subtype.is_none()
                    && t.subtypes_any.is_empty()
                    && !t.printed_subtype
                    && t.printed_cost_max.is_none()
                    && t.predicate.is_none()
                    && !t.equipment_host
                    && !t.requires_magic
                    && !t.exclude_source
                    && !t.exclude_attachment_host
                    && t.attachment_host_condition.is_none()
            })
        {
            return Err(format!(
                "{card_id}: only the two admitted MSJC11 paid standard grants are supported"
            ));
        }
    }
    if ability.event == Some(Event::ReceiveWound)
        && (card_id != "LC06"
            || ability.timing != Timing::Fast
            || ability.response_policy != ResponsePolicy::Respondable
            || ability.play_only
            || ability.activation_only
            || ability.requires_ready_source
            || ability.once_per_game
            || ability.per_turn_limit.is_some()
            || !ability.costs.is_empty()
            || !ability.targets.is_empty()
            || !ability.modes.is_empty()
            || !matches!(
                ability.ops.as_slice(),
                [Op::Draw {
                    player: PlayerRef::Actor,
                    count: 2,
                    end: DeckEnd::Top
                }]
            ))
    {
        return Err(format!(
            "{card_id}: only the admitted self-wound draw trigger is supported"
        ));
    }
    if ability.modes.iter().any(|m| {
        m.ops.iter().any(|op| {
            matches!(
                op,
                Op::RepressOpponentOne { .. } | Op::MoveActorDeckTopToAsset
            )
        })
    }) {
        return Err(format!(
            "{card_id}: admitted repression and expansion cannot be mode operations"
        ));
    }
    if ability
        .targets
        .iter()
        .any(|t| t.zone == Zone::AttachmentOrAsset)
        && (card_id != "JC107"
            || ability.targets.len() != 1
            || ability.targets[0].kind != EntityKind::Any
            || ability.targets[0].relation != Relation::Any
            || ability.targets[0].range != Range::Anywhere
            || !matches!(
                ability.ops.as_slice(),
                [Op::DestroyPublicAttachmentOrAsset { slot: 0 }]
            ))
    {
        return Err(format!(
            "{card_id}: only the admitted one attachment-or-asset destruction is supported"
        ));
    }
    for op in &ability.ops {
        if matches!(op, Op::JC032TopSixVampireHidden | Op::JZ24LocalSacrificeSnapshot) {
            let expected = match (card_id, op) {
                ("JC032", Op::JC032TopSixVampireHidden) => jc032_definition(),
                ("JZ24", Op::JZ24LocalSacrificeSnapshot) => jz24_definition(),
                _ => return Err(format!("{card_id}: blue finite operation is not admitted here")),
            };
            if serde_json::to_value(ability).unwrap() != serde_json::to_value(&expected.abilities[0]).unwrap() {
                return Err(format!("{card_id}: only the complete admitted finite ability is supported"));
            }
        }
        if let Op::Search {
            filter: CardFilter::NamedCharacter(name),
            player,
            to_top,
            visibility,
            ..
        } = op
        {
            if card_id != "JZ61"
                || name != "噩梦残像"
                || !matches!(player, PlayerRef::Actor)
                || *to_top
                || *visibility != SearchVisibility::Reveal
                || ability.event != Some(Event::Death)
                || !ability.targets.is_empty()
                || !ability.costs.is_empty()
                || ability.ops.len() != 1
            {
                return Err(format!(
                    "{card_id}: only the admitted named-character death search is supported"
                ));
            }
        }
        match op {
            Op::RepressOpponentOne { slot }
                if *slot != 0
                    || !(card_id == "JZ58"
                        && ability.ops.len() == 1
                        && ability.event == Some(Event::Reveal)
                        || card_id == "JZ74"
                            && ability.play_only
                            && ability.modes.is_empty()
                            && ability.timing == Timing::Fast
                            && ability.event.is_none()
                            && matches!(ability.ops.as_slice(), [Op::RepressOpponentOne { slot: 0 }, Op::Draw { player: PlayerRef::Actor, count: 1, end: DeckEnd::Top }]))
                    || !ability.costs.is_empty()
                    || ability.targets.len() != 1
                    || ability.targets[0].zone != Zone::Player
                    || ability.targets[0].relation != Relation::EnemyTeam
                    || ability.targets[0].range != Range::Anywhere =>
            {
                return Err(format!(
                    "{card_id}: only the admitted reveal repression or repression-then-draw is supported"
                ))
            }
            Op::MoveActorDeckTopToAsset
                if card_id != "JC126"
                    || !ability.play_only
                    || !ability.modes.is_empty()
                    || ability.timing != Timing::Standard
                    || ability.event.is_some()
                    || !ability.costs.is_empty()
                    || !ability.targets.is_empty()
                    || ability.ops.len() != 1 =>
            {
                return Err(format!(
                    "{card_id}: only the admitted actor deck-top asset expansion is supported"
                ))
            }

            Op::WoundTarget { slot, amount }
                if card_id != "JZ59"
                    || *slot != 0
                    || *amount != 1
                    || ability.event != Some(Event::Death)
                    || !ability.costs.is_empty()
                    || ability.targets.len() != 1
                    || ability.targets[0].zone != Zone::Board
                    || ability.targets[0].kind != EntityKind::Character
                    || ability.targets[0].relation != Relation::Any
                    || ability.targets[0].range != Range::SourceRegion =>
            {
                return Err(format!(
                    "{card_id}: only the admitted one local death wound is supported"
                ))
            }
            Op::DestroyPublicAttachmentOrAsset { slot }
                if card_id != "JC107"
                    || *slot != 0
                    || !ability
                        .targets
                        .first()
                        .is_some_and(|t| t.zone == Zone::AttachmentOrAsset) =>
            {
                return Err(format!(
                    "{card_id}: invalid admitted public destruction target"
                ))
            }
            Op::CostReductionOnFaceUpOrPaidReveal { filter, amount }
                if card_id != "JC103"
                    || *amount != 1
                    || !matches!(filter,CardFilter::PrintedColor(c)if c=="紫")
                    || !ability.targets.is_empty()
                    || !matches!(ability.costs.as_slice(), [Cost::ExhaustSource]) =>
            {
                return Err(format!(
                    "{card_id}: invalid admitted purple paid-reveal discount"
                ))
            }
            _ => {}
        }
    }
    if ability.event == Some(Event::HandDiscard)
        && (card_id != "XQ34"
            || !matches!(ability.costs.as_slice(), [Cost::Assets(1)])
            || !matches!(
                ability.ops.as_slice(),
                [Op::Draw {
                    player: PlayerRef::Actor,
                    count: 1,
                    end: DeckEnd::Top
                }]
            )
            || !ability.targets.is_empty())
    {
        return Err(format!(
            "{card_id}: only the admitted paid self-discard draw trigger is supported"
        ));
    }
    if ability
        .costs
        .iter()
        .any(|c| matches!(c, Cost::DiscardSelectedHandCard))
        && (card_id != "XQ38"
            || !ability.activation_only
            || ability.event.is_some()
            || !matches!(
                ability.costs.as_slice(),
                [
                    Cost::Assets(2),
                    Cost::ExhaustSource,
                    Cost::DiscardSelectedHandCard
                ]
            ))
    {
        return Err(format!(
            "{card_id}: only the admitted grave recovery hand-discard cost is supported"
        ));
    }
    if let Some(limit) = ability.per_turn_limit {
        let is_admitted_jc032_action = card_id == "JC032"
            && serde_json::to_value(ability).unwrap()
                == serde_json::to_value(&jc032_definition().abilities[0]).unwrap();
        let is_admitted_host_grant = limit == 2
            && ability.activation_only
            && ability.event.is_none()
            && matches!(
                ability.costs.as_slice(),
                [Cost::SacrificeSelectedControlledCharacter]
            )
            && matches!(
                ability.ops.as_slice(),
                [Op::ModifyAttachmentHostUntilTurnEnd]
            )
            && ability.modes.is_empty();
        if !is_admitted_jc032_action && !is_admitted_host_grant {
            return Err(format!(
                "{card_id}: only the admitted host grant or JC032 once-per-turn action is supported"
            ));
        }
    }
    if ability.play_only && ability.activation_only {
        return Err(format!("{card_id}: conflicting action roles"));
    }
    if ability.once_per_game && ability.event.is_some() {
        return Err(format!(
            "{card_id}: game-limited abilities must be declared actions"
        ));
    }
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
            if let Op::GrantTargetKillUntilTurnEnd { slot }
            | Op::GrantRegionRetreatUntilTurnEnd { slot } = op
            {
                if !targets.get(*slot).is_some_and(|t| {
                    t.zone
                        == if matches!(op, Op::GrantTargetKillUntilTurnEnd { .. }) {
                            Zone::Board
                        } else {
                            Zone::Region
                        }
                }) {
                    return Err(format!(
                        "{location}: MSJC11 grant requires its exact bound target"
                    ));
                }
            }
            if let Op::RevealHandAndOfferSourceSacrifice { player }
            | Op::RandomHandToOwnerDeckTop { player }
            | Op::MoveDeckTopToGraveyard { player, .. }
            | Op::OfferShuffleIfActorControlsDreamDemon { player } = op
            {
                if !matches!(player, PlayerRef::Target(0))
                    || !targets
                        .first()
                        .is_some_and(|slot| slot.zone == Zone::Player)
                {
                    return Err(format!(
                        "{location}: admitted hand/deck effect requires its bound player"
                    ));
                }
            }
            if matches!(
                op,
                Op::ModifyAttachmentHostUntilTurnEnd
                    | Op::SetLocalMagicPrintedDefenseToOneUntilTurnEnd
            ) && !targets.is_empty()
            {
                return Err(format!(
                    "{location}: host and local printed-defense effects are non-targeted"
                ));
            }
            if let Op::PreventTargetDamageUntilTurnEnd { slot } | Op::ReattachSource { slot } = op {
                if !targets.get(*slot).is_some_and(|target| {
                    target.zone == Zone::Board && target.kind == EntityKind::Character
                }) {
                    return Err(format!(
                        "{location}: protection or reattachment requires one bound board character"
                    ));
                }
                if matches!(op, Op::ReattachSource { .. }) && !targets[*slot].equipment_host {
                    return Err(format!(
                        "{location}: reattachment requires an equipment host guard"
                    ));
                }
            }
            if let Op::DamageTarget { slot, amount } = op {
                if *amount != 1
                    || !targets.get(*slot).is_some_and(|target| {
                        target.zone == Zone::Board && target.kind == EntityKind::Character
                    })
                {
                    return Err(format!(
                        "{location}: damage requires one bound board character and one damage"
                    ));
                }
            }
            if let Op::GainControl { slot, .. } = op {
                if !targets
                    .get(*slot)
                    .is_some_and(|t| t.zone == Zone::Board && t.kind == EntityKind::Character)
                {
                    return Err(format!(
                        "{location}: control requires a bound board character"
                    ));
                }
            }
            if let Op::ModifyTargetUntilTurnEnd {
                slot,
                defense_bonus,
                ordinary_icons,
                grants_renown,
            } = op
            {
                if !targets.get(*slot).is_some_and(|target| {
                    target.zone == Zone::Board && target.kind == EntityKind::Character
                }) || (*defense_bonus == 0
                    && *ordinary_icons == Icons::default()
                    && !grants_renown)
                {
                    return Err(format!(
                        "{location}: turn attribute bonus requires a bound board character and a nonempty bonus"
                    ));
                }
            }
            if let Op::PlaceInfluence {
                region_instance,
                amount,
            } = op
            {
                if region_instance.is_empty() || *amount != 1 || !targets.is_empty() {
                    return Err(format!(
                        "{location}: renown requires an exact region and one influence"
                    ));
                }
            }
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
                            | Op::JC032TopSixVampireHidden
                            | Op::JZ24LocalSacrificeSnapshot
                            | Op::GrantTargetKillUntilTurnEnd { .. }
                            | Op::GrantRegionRetreatUntilTurnEnd { .. }
                            | Op::ForEachLivingPlayerFromActor(_)
                            | Op::RepressOpponentOne { .. }
                            | Op::MoveActorDeckTopToAsset
                            | Op::IfTargetExhausted { .. }
                            | Op::ModifyTargetUntilTurnEnd { .. }
                            | Op::GainControl { .. }
                            | Op::PlaceInfluence { .. }
                            | Op::PlaceOneInfluenceInSourceRegion
                            | Op::DamageTarget { .. }
                            | Op::PreventTargetDamageUntilTurnEnd { .. }
                            | Op::ReattachSource { .. }
                            | Op::ModifyAttachmentHostUntilTurnEnd
                            | Op::SetLocalMagicPrintedDefenseToOneUntilTurnEnd
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

pub(crate) fn validate_definitions(definitions: &BTreeMap<String, Definition>) -> Result<(), String> {
    for (card_id, definition) in definitions {
        if (card_id == "JZ48" || definition.modifiers.iter().any(|m|
            matches!(m, StaticModifier::JZ48OtherControlledCriminalInfluenceAndDefense)))
            && (card_id != "JZ48" || serde_json::to_value(definition).unwrap()
                != serde_json::to_value(jz48_definition()).unwrap())
        {
            return Err(format!("cardId={card_id}: only the complete JZ48 criminal condition is admitted"));
        }
        if (card_id == "JZ49" || definition.traits.slow)
            && (card_id != "JZ49" || serde_json::to_value(definition).unwrap()
                != serde_json::to_value(jz49_definition()).unwrap())
        {
            return Err(format!("cardId={card_id}: only the complete JZ49 slow definition is admitted"));
        }
        if card_id == "JZ55" && serde_json::to_value(definition).unwrap()
            != serde_json::to_value(jz55_definition()).unwrap() {
            return Err("cardId=JZ55: only the complete admitted immediate unique destroy definition is supported".into());
        }
        if card_id == "JZ31" && serde_json::to_value(definition).unwrap()
            != serde_json::to_value(jz31_definition()).unwrap()
        {
            return Err("cardId=JZ31: only the complete admitted death influence definition is supported".into());
        }
        if card_id == "XQ36"
            && serde_json::to_value(definition).unwrap()
                != serde_json::to_value(xq36_definition()).unwrap()
        {
            return Err("cardId=XQ36: only the complete admitted entry mill is supported".into());
        }
        let expected = match card_id.as_str() {
            "JC032" => Some(jc032_definition()),
            "JZ24" => Some(jz24_definition()),
            "MSJC03" => Some(msjc03_definition()),
            _ => None,
        };
        if expected.is_some_and(|d| serde_json::to_value(definition).unwrap() != serde_json::to_value(d).unwrap()) {
            return Err(format!("{card_id}: only the complete admitted blue definition is supported"));
        }
        if definition.modifiers.iter().any(|m| matches!(m, StaticModifier::JC030BloodAssetsVampireAndInvestigation)) && card_id != "JC030" {
            return Err(format!("cardId={card_id}: JC030 modifier is not admitted here"));
        }
        if card_id == "JC030" && serde_json::to_value(definition).unwrap() != serde_json::to_value(jc030_definition()).unwrap() {
            return Err("cardId=JC030: only the complete admitted blue bat definition is supported".into());
        }
        for modifier in &definition.modifiers {
            if matches!(modifier, StaticModifier::LC30WeaponsCombatAndDefense) && card_id != "LC30"
                || matches!(modifier, StaticModifier::JC018MindAssetsCombat) && card_id != "JC018"
            {
                return Err(format!(
                    "cardId={card_id}: green source modifier is not admitted here"
                ));
            }
        }
        for (id, whole) in [
            ("LC30", lc30_definition()),
            ("JC018", jc018_definition()),
            ("JC015", jc015_definition()),
        ] {
            if card_id == id
                && serde_json::to_value(definition).unwrap() != serde_json::to_value(whole).unwrap()
            {
                return Err(format!(
                    "cardId={id}: only the complete admitted green definition is supported"
                ));
            }
        }
        let conversions = definition
            .modifiers
            .iter()
            .filter(|m| {
                matches!(
                    m,
                    StaticModifier::AttachedRegionSpiritTemporaryIconsPermanent
                )
            })
            .count();
        if conversions > 0 && (card_id != "XQ43" || conversions != 1) {
            return Err(format!(
                "cardId={card_id}: only the admitted XQ43 region conversion is supported"
            ));
        }
        if card_id == "XQ43"
            && serde_json::to_value(definition).unwrap()
                != serde_json::to_value(xq43_definition()).unwrap()
        {
            return Err(
                "cardId=XQ43: only the complete admitted region attachment is supported".into(),
            );
        }
        for ability in &definition.abilities {
            validate_ability(card_id, ability)?;
        }
        if let Some(attachment) = &definition.attachment {
            let host = &attachment.host;
            if host.predicate.is_some() {
                return Err(format!(
                    "cardId={card_id}: finite target predicates cannot be an attachment host guard"
                ));
            }
            let admitted_region = card_id == "XQ43";
            if !admitted_region
                && (host.zone != Zone::Board
                    || host.kind != EntityKind::Character
                    || host.min != 1
                    || host.max != 1
                    || host.range != Range::Anywhere)
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
        printed_subtype: false,
        subtypes_any: vec![],
        printed_cost_max: None,
        predicate: None,
        equipment_host: false,
        requires_magic: false,
        exclude_source: false,
        exclude_attachment_host: false,
        attachment_host_condition: None,
        min: 1,
        max: 1,
    }
}
fn xq43_definition() -> Definition {
    let host = target(
        Zone::Region,
        EntityKind::Any,
        Relation::Any,
        Range::Anywhere,
    );
    let mut attach = ability(
        "attach-region",
        "结附地区",
        Timing::Standard,
        vec![],
        vec![host.clone()],
        vec![],
        None,
    );
    attach.play_only = true;
    Definition {
        abilities: vec![attach],
        modifiers: vec![StaticModifier::AttachedRegionSpiritTemporaryIconsPermanent],
        attachment: Some(AttachmentSpec {
            controls_host: false,
            host_subtype_change: SubtypeChange::None,
            host,
            host_icons: Icons::default(),
            host_temporary_icons: None,
            host_barrier: false,
            host_defense_bonus: 0,
            host_leaves: HostLeaveDestination::OwnerGraveyard,
        }),
        ..Definition::default()
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
        play_only: false,
        activation_only: false,
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
        once_per_game: false,
        per_turn_limit: None,
    }
}
fn with_abilities(abilities: Vec<AbilitySpec>) -> Definition {
    Definition {
        abilities,
        ..Default::default()
    }
}

fn xq36_definition() -> Definition {
    with_abilities(vec![ability(
        "entry-mill-each-four",
        "进场触发",
        Timing::Fast,
        vec![],
        vec![],
        vec![Op::ForEachLivingPlayer(vec![Op::MoveDeckTopToGraveyard {
            player: PlayerRef::Context,
            count: 4,
        }])],
        Some(Event::Enter),
    )])
}

fn lc30_definition() -> Definition {
    Definition {
        traits: Traits {
            guard: 2,
            ..Default::default()
        },
        modifiers: vec![StaticModifier::LC30WeaponsCombatAndDefense],
        ..Default::default()
    }
}
fn jc018_definition() -> Definition {
    Definition {
        traits: Traits {
            renown: true,
            ..Default::default()
        },
        modifiers: vec![StaticModifier::JC018MindAssetsCombat],
        ..Default::default()
    }
}
fn jc015_definition() -> Definition {
    let mut victim = target(
        Zone::Board,
        EntityKind::Character,
        Relation::EnemyTeam,
        Range::Anywhere,
    );
    victim.predicate = Some(TargetPredicate::JC015NonHumanPrintedCostAtLeastThree);
    with_abilities(vec![ability(
        "exorcise",
        "牺牲驱魔人消灭非人类",
        Timing::Standard,
        vec![Cost::ExhaustSource, Cost::SacrificeSource],
        vec![victim],
        vec![Op::Destroy(EntityRef::Target(0))],
        None,
    )])
}
fn jz55_definition() -> Definition {
    let mut victim = target(Zone::Board, EntityKind::Character, Relation::Any, Range::Anywhere);
    victim.predicate = Some(TargetPredicate::JZ55UniqueCharacter);
    let mut destroy = ability("destroy-unique", "快速行动（不可响应）", Timing::Fast,
        vec![], vec![victim], vec![Op::Destroy(EntityRef::Target(0))], None);
    destroy.response_policy = ResponsePolicy::Immediate;
    with_abilities(vec![destroy])
}
fn jc030_definition() -> Definition {
    let mut definition = with_abilities(vec![ability(
        "raid-1",
        "袭击1",
        Timing::Fast,
        vec![],
        vec![target(Zone::Board, EntityKind::Character, Relation::Any, Range::SourceRegion)],
        vec![Op::DamageTarget { slot: 0, amount: 1 }],
        Some(Event::Reveal),
    )]);
    definition.traits.cannot_be_equipped = true;
    definition.modifiers = vec![StaticModifier::JC030BloodAssetsVampireAndInvestigation];
    definition
}
fn jc032_definition() -> Definition {
    let mut action = ability("top-six-vampire-hidden", "顶六张吸血鬼暗藏（每回合一次）", Timing::Standard,
        vec![Cost::Assets(2)], vec![], vec![Op::JC032TopSixVampireHidden], None);
    action.per_turn_limit = Some(1);
    with_abilities(vec![action])
}
fn jz24_definition() -> Definition {
    with_abilities(vec![ability("local-enemy-sacrifice", "敌方本地牺牲", Timing::Fast,
        vec![], vec![], vec![Op::JZ24LocalSacrificeSnapshot], Some(Event::Reveal))])
}
fn jz31_definition() -> Definition {
    with_abilities(vec![ability("death-source-influence", "死亡触发：本地区放置一个本方势力标志", Timing::Fast,
        vec![], vec![], vec![Op::PlaceOneInfluenceInSourceRegion], Some(Event::Death))])
}
fn jz49_definition() -> Definition {
    let mut definition = with_abilities(vec![ability(
        "mill-two-entry", "进场触发：目标玩家牌库顶两张置墓", Timing::Fast,
        vec![], vec![target(Zone::Player, EntityKind::Any, Relation::Any, Range::Anywhere)],
        vec![Op::MoveDeckTopToGraveyard { player: PlayerRef::Target(0), count: 2 }],
        Some(Event::Enter),
    )]);
    definition.traits.slow = true;
    definition
}
fn jz48_definition() -> Definition {
    Definition {
        modifiers: vec![StaticModifier::JZ48OtherControlledCriminalInfluenceAndDefense],
        ..Definition::default()
    }
}
fn msjc03_definition() -> Definition {
    let mut d = msjc02_definition();
    d.abilities[1].key = "search-blue-unique".into();
    d.abilities[1].label = "蓝色独有检索（每局一次）".into();
    if let Op::Search { filter, .. } = &mut d.abilities[1].ops[0] {
        *filter = CardFilter::PrintedColorAndUnique { color: "蓝".into() };
    }
    d
}
fn msjc02_definition() -> Definition {
    let mut search = ability(
        "search-green-unique",
        "绿色独有检索（每局一次）",
        Timing::Standard,
        vec![Cost::Assets(4), Cost::ExhaustSource],
        vec![],
        vec![Op::Search {
            player: PlayerRef::Actor,
            filter: CardFilter::PrintedColorAndUnique {
                color: "绿".into()
            },
            to_top: false,
            optional: false,
            visibility: SearchVisibility::Reveal,
        }],
        None,
    );
    search.once_per_game = true;
    with_abilities(vec![
        ability(
            "drawWithInitiative",
            "先手抓牌",
            Timing::Standard,
            vec![Cost::Assets(3), Cost::ExhaustSource],
            vec![],
            vec![Op::DrawIfActorHasInitiative { count: 1 }],
            None,
        ),
        search,
    ])
}

pub fn definitions() -> &'static BTreeMap<String, Definition> {
    static DEFINITIONS: OnceLock<BTreeMap<String, Definition>> = OnceLock::new();
    DEFINITIONS.get_or_init(|| {
        use EntityRef::{Source, Target};
        use PlayerRef::{Actor, Context};
        let mut m = BTreeMap::new();
        m.insert("JC032".into(), jc032_definition());
        m.insert("JZ24".into(), jz24_definition());
        m.insert("JZ31".into(), jz31_definition());
        m.insert("JZ49".into(), jz49_definition());
        m.insert("JZ48".into(), jz48_definition());
        m.insert("JZ55".into(), jz55_definition());
        m.insert("JC030".into(), jc030_definition());
        m.insert("LC30".into(), lc30_definition());
        m.insert("JC018".into(), jc018_definition());
        m.insert("JC015".into(), jc015_definition());
        m.insert("XQ43".into(), xq43_definition());
        // Whole printed JC029: optional reveal only, one damage, any face-up
        // character in the source region. Reuses existing trigger/target/damage.
        m.insert(
            "JC029".into(),
            with_abilities(vec![ability(
                "raid-1",
                "袭击1",
                Timing::Fast,
                vec![],
                vec![target(
                    Zone::Board,
                    EntityKind::Character,
                    Relation::Any,
                    Range::SourceRegion,
                )],
                vec![Op::DamageTarget { slot: 0, amount: 1 }],
                Some(Event::Reveal),
            )]),
        );
        let mut accident = ability(
            "repress-draw",
            "快速行动",
            Timing::Fast,
            vec![],
            vec![target(
                Zone::Player,
                EntityKind::Any,
                Relation::EnemyTeam,
                Range::Anywhere,
            )],
            vec![
                Op::RepressOpponentOne { slot: 0 },
                Op::Draw {
                    player: Actor,
                    count: 1,
                    end: DeckEnd::Top,
                },
            ],
            None,
        );
        accident.play_only = true;
        m.insert("JZ74".into(), with_abilities(vec![accident]));
        let mut expansion = ability(
            "expand-assets",
            "标准行动",
            Timing::Standard,
            vec![],
            vec![],
            vec![Op::MoveActorDeckTopToAsset],
            None,
        );
        expansion.play_only = true;
        m.insert("JC126".into(), with_abilities(vec![expansion]));
        m.insert(
            "JZ58".into(),
            Definition {
                traits: Traits {
                    spirit: true,
                    ..Default::default()
                },
                abilities: vec![ability(
                    "reveal-repress",
                    "现身触发",
                    Timing::Fast,
                    vec![],
                    vec![target(
                        Zone::Player,
                        EntityKind::Any,
                        Relation::EnemyTeam,
                        Range::Anywhere,
                    )],
                    vec![Op::RepressOpponentOne { slot: 0 }],
                    Some(Event::Reveal),
                )],
                ..Default::default()
            },
        );
        m.insert(
            "JZ61".into(),
            with_abilities(vec![ability(
                "death-find-nightmare",
                "死亡触发",
                Timing::Fast,
                vec![],
                vec![],
                vec![Op::Search {
                    player: PlayerRef::Actor,
                    filter: CardFilter::NamedCharacter("噩梦残像".into()),
                    to_top: false,
                    optional: true,
                    visibility: SearchVisibility::Reveal,
                }],
                Some(Event::Death),
            )]),
        );
        m.insert(
            "JZ59".into(),
            with_abilities(vec![ability(
                "death-local-wound",
                "死亡触发",
                Timing::Fast,
                vec![],
                vec![target(
                    Zone::Board,
                    EntityKind::Character,
                    Relation::Any,
                    Range::SourceRegion,
                )],
                vec![Op::WoundTarget { slot: 0, amount: 1 }],
                Some(Event::Death),
            )]),
        );
        m.insert(
            "LC06".into(),
            with_abilities(vec![ability(
                "wound-draw-two",
                "受到创伤触发",
                Timing::Fast,
                vec![],
                vec![],
                vec![Op::Draw {
                    player: Actor,
                    count: 2,
                    end: DeckEnd::Top,
                }],
                Some(Event::ReceiveWound),
            )]),
        );
        let mut hermit_target = target(
            Zone::Board,
            EntityKind::Character,
            Relation::ControlledByActor,
            Range::SourceRegion,
        );
        hermit_target.exclude_source = true;
        let mut hermit = ability(
            "protect-local-character",
            "快速行动",
            Timing::Fast,
            vec![Cost::ExhaustSource],
            vec![hermit_target],
            vec![Op::ModifyTargetUntilTurnEnd {
                slot: 0,
                defense_bonus: 1,
                ordinary_icons: Icons::default(),
                grants_renown: false,
            }],
            None,
        );
        hermit.activation_only = true;
        m.insert("LC12".into(), with_abilities(vec![hermit]));
        let mut painter = ability(
            "reduce-next-purple",
            "快速行动",
            Timing::Fast,
            vec![Cost::ExhaustSource],
            vec![],
            vec![Op::CostReductionOnFaceUpOrPaidReveal {
                filter: CardFilter::PrintedColor("紫".into()),
                amount: 1,
            }],
            None,
        );
        painter.activation_only = true;
        painter.response_policy = ResponsePolicy::Immediate;
        m.insert("JC103".into(), with_abilities(vec![painter]));
        let mut collapse = ability(
            "destroy-attachment-or-asset",
            "标准行动",
            Timing::Standard,
            vec![],
            vec![target(
                Zone::AttachmentOrAsset,
                EntityKind::Any,
                Relation::Any,
                Range::Anywhere,
            )],
            vec![Op::DestroyPublicAttachmentOrAsset { slot: 0 }],
            None,
        );
        collapse.play_only = true;
        m.insert("JC107".into(), with_abilities(vec![collapse]));
        let any_player = target(
            Zone::Player,
            EntityKind::Any,
            Relation::Any,
            Range::Anywhere,
        );
        m.insert(
            "JC114".into(),
            with_abilities(vec![ability(
                "reveal-hand-discard",
                "标准行动",
                Timing::Standard,
                vec![Cost::Assets(1), Cost::ExhaustSource],
                vec![any_player.clone()],
                vec![Op::RevealHandAndOfferSourceSacrifice {
                    player: PlayerRef::Target(0),
                }],
                None,
            )]),
        );
        let mut inspiration = ability(
            "inspiration-draw",
            "快速行动",
            Timing::Fast,
            vec![],
            vec![],
            vec![Op::Draw {
                player: Actor,
                count: 1,
                end: DeckEnd::Top,
            }],
            None,
        );
        inspiration.play_only = true;
        m.insert(
            "XQ34".into(),
            with_abilities(vec![
                inspiration,
                ability(
                    "discard-draw",
                    "手牌触发",
                    Timing::Fast,
                    vec![Cost::Assets(1)],
                    vec![],
                    vec![Op::Draw {
                        player: Actor,
                        count: 1,
                        end: DeckEnd::Top,
                    }],
                    Some(Event::HandDiscard),
                ),
            ]),
        );
        let mut recover = ability(
            "recover-grave-discard",
            "标准行动",
            Timing::Standard,
            vec![
                Cost::Assets(2),
                Cost::ExhaustSource,
                Cost::DiscardSelectedHandCard,
            ],
            vec![target(
                Zone::Graveyard,
                EntityKind::Character,
                Relation::OwnedByActor,
                Range::Anywhere,
            )],
            vec![Op::Move(Target(0), Destination::ActorHand)],
            None,
        );
        recover.activation_only = true;
        m.insert(
            "XQ38".into(),
            with_abilities(vec![
                ability(
                    "mill-entry",
                    "进场触发",
                    Timing::Fast,
                    vec![],
                    vec![any_player.clone()],
                    vec![Op::MoveDeckTopToGraveyard {
                        player: PlayerRef::Target(0),
                        count: 3,
                    }],
                    Some(Event::Enter),
                ),
                recover,
            ]),
        );
        let mut amnesia = ability(
            "amnesia",
            "标准行动",
            Timing::Standard,
            vec![],
            vec![any_player],
            vec![
                Op::RandomHandToOwnerDeckTop {
                    player: PlayerRef::Target(0),
                },
                Op::Forecast {
                    player: Actor,
                    count: 2,
                },
                Op::OfferShuffleIfActorControlsDreamDemon {
                    player: PlayerRef::Target(0),
                },
            ],
            None,
        );
        amnesia.play_only = true;
        m.insert("JZ67".into(), with_abilities(vec![amnesia]));
        // Original scans: finite draw/discard, private search and paid sacrifice.
        // Reuse the admitted continuation interpreter; no new shared mechanism.
        for (id, key, draw_count, event) in [
            ("JC130", "research", 2, None),
            ("XQ17", "offering-death", 1, Some(Event::Death)),
        ] {
            let mut spec = ability(
                key,
                if event.is_some() {
                    "死亡触发"
                } else {
                    "标准行动"
                },
                if event.is_some() {
                    Timing::Fast
                } else {
                    Timing::Standard
                },
                vec![],
                vec![],
                vec![
                    Op::Draw {
                        player: Actor,
                        count: draw_count,
                        end: DeckEnd::Top,
                    },
                    Op::Discard {
                        player: Actor,
                        count: Some(1),
                        redraw: false,
                        optional: false,
                    },
                ],
                event,
            );
            spec.play_only = event.is_none();
            m.insert(id.into(), with_abilities(vec![spec]));
        }
        let mut supplies = ability(
            "airdrop",
            "标准行动",
            Timing::Standard,
            vec![],
            vec![],
            vec![Op::Search {
                player: Actor,
                filter: CardFilter::Any,
                to_top: false,
                optional: false,
                visibility: SearchVisibility::Private,
            }],
            None,
        );
        supplies.play_only = true;
        m.insert("JC131".into(), with_abilities(vec![supplies]));
        let mut dog = ability(
            "sacrifice-draw",
            "快速行动",
            Timing::Fast,
            vec![
                Cost::Assets(1),
                Cost::ExhaustSource,
                Cost::SacrificeSelectedControlledCharacter,
            ],
            vec![],
            vec![Op::Draw {
                player: Actor,
                count: 1,
                end: DeckEnd::Top,
            }],
            None,
        );
        dog.activation_only = true;
        m.insert("JC096".into(), with_abilities(vec![dog]));
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
        let mut knife_exhaust = ability(
            "exhaust-local-hidden",
            "横置暗藏者",
            Timing::Fast,
            vec![Cost::ExhaustSource],
            vec![target(
                Zone::Board,
                EntityKind::Hidden,
                Relation::Any,
                Range::SourceRegion,
            )],
            vec![Op::Exhaust(Target(0))],
            None,
        );
        knife_exhaust.activation_only = true;
        let mut knife_damage = ability(
            "sacrifice-local-damage",
            "牺牲飞刀造成伤害",
            Timing::Fast,
            vec![Cost::SacrificeSource],
            vec![local.clone()],
            vec![Op::DamageTarget { slot: 0, amount: 1 }],
            None,
        );
        knife_damage.activation_only = true;
        let mut blade_grant = ability(
            "sacrifice-character-host-icons",
            "牺牲角色强化宿主",
            Timing::Fast,
            vec![Cost::SacrificeSelectedControlledCharacter],
            vec![],
            vec![Op::ModifyAttachmentHostUntilTurnEnd],
            None,
        );
        blade_grant.activation_only = true;
        blade_grant.per_turn_limit = Some(2);
        let mut holy_water = ability(
            "sacrifice-local-printed-defense",
            "牺牲圣水改变印刷防御",
            Timing::Fast,
            vec![Cost::SacrificeSource],
            vec![],
            vec![Op::SetLocalMagicPrintedDefenseToOneUntilTurnEnd],
            None,
        );
        holy_water.activation_only = true;
        for (id, relation, combat, retreat, activations) in [
            ("JC116", Relation::Any, 1, true, vec![]),
            (
                "JC020",
                Relation::ControlledByActor,
                0,
                false,
                vec![knife_exhaust, knife_damage],
            ),
            ("JC093", Relation::Any, 0, false, vec![blade_grant]),
            (
                "XQ07",
                Relation::ControlledByActor,
                0,
                false,
                vec![holy_water],
            ),
        ] {
            let mut host = character.clone();
            host.equipment_host = true;
            host.relation = relation;
            let mut attach = ability(
                "attach",
                "结附角色",
                Timing::Standard,
                vec![],
                vec![host.clone()],
                vec![],
                None,
            );
            attach.play_only = true;
            let mut abilities = vec![attach];
            abilities.extend(activations);
            m.insert(
                id.into(),
                Definition {
                    traits: Traits {
                        retreat,
                        ..Default::default()
                    },
                    abilities,
                    attachment: Some(AttachmentSpec {
                        controls_host: false,
                        host_subtype_change: SubtypeChange::None,
                        host,
                        host_icons: Icons {
                            combat,
                            ..Default::default()
                        },
                        host_temporary_icons: None,
                        host_barrier: false,
                        host_defense_bonus: 0,
                        host_leaves: HostLeaveDestination::OwnerGraveyard,
                    }),
                    ..Default::default()
                },
            );
        }
        m.insert(
            "JC078".into(),
            with_abilities(vec![ability(
                "prevent-turn-damage",
                "快速伤害防护",
                Timing::Fast,
                vec![],
                vec![character.clone()],
                vec![Op::PreventTargetDamageUntilTurnEnd { slot: 0 }],
                None,
            )]),
        );
        let mut blood_host = character.clone();
        blood_host.subtype = Some("吸血鬼".into());
        let mut blood_attach = ability(
            "attach",
            "结附吸血鬼",
            Timing::Standard,
            vec![],
            vec![blood_host.clone()],
            vec![],
            None,
        );
        blood_attach.play_only = true;
        m.insert(
            "XQ14".into(),
            Definition {
                abilities: vec![blood_attach],
                attachment: Some(AttachmentSpec {
                    controls_host: false,
                    host_subtype_change: SubtypeChange::None,
                    host: blood_host,
                    host_icons: Icons {
                        combat: 1,
                        ..Default::default()
                    },
                    host_temporary_icons: None,
                    host_barrier: false,
                    host_defense_bonus: 1,
                    host_leaves: HostLeaveDestination::OwnerGraveyard,
                }),
                ..Default::default()
            },
        );
        let mut vest_host = character.clone();
        vest_host.equipment_host = true;
        let mut another_host = vest_host.clone();
        another_host.exclude_attachment_host = true;
        let mut vest_attach = ability(
            "attach",
            "结附角色",
            Timing::Standard,
            vec![],
            vec![vest_host.clone()],
            vec![],
            None,
        );
        vest_attach.play_only = true;
        let mut vest_transfer = ability(
            "reattach",
            "转移结附",
            Timing::Standard,
            vec![Cost::Assets(2)],
            vec![another_host],
            vec![Op::ReattachSource { slot: 0 }],
            None,
        );
        vest_transfer.activation_only = true;
        m.insert(
            "XQ47".into(),
            Definition {
                abilities: vec![vest_attach, vest_transfer],
                attachment: Some(AttachmentSpec {
                    controls_host: false,
                    host_subtype_change: SubtypeChange::None,
                    host: vest_host,
                    host_icons: Icons::default(),
                    host_temporary_icons: None,
                    host_barrier: false,
                    host_defense_bonus: 1,
                    host_leaves: HostLeaveDestination::OwnerGraveyard,
                }),
                ..Default::default()
            },
        );
        let mut cat_reduce = ability(
            "reduce-next-magic",
            "牺牲减费",
            Timing::Fast,
            vec![Cost::SacrificeSource],
            vec![],
            vec![Op::CostReduction {
                filter: CardFilter::HasPrintedMagic,
                amount: 1,
            }],
            None,
        );
        cat_reduce.response_policy = ResponsePolicy::Immediate;
        m.insert(
            "JC112".into(),
            Definition {
                traits: Traits {
                    public: true,
                    cannot_be_equipped: true,
                    ..Default::default()
                },
                abilities: vec![cat_reduce],
                ..Default::default()
            },
        );
        let mut hermit_target = local.clone();
        hermit_target.relation = Relation::ControlledByActor;
        hermit_target.exclude_source = true;
        m.insert(
            "JC071".into(),
            with_abilities(vec![ability(
                "protect-local-character",
                "快速防护",
                Timing::Fast,
                vec![Cost::ExhaustSource],
                vec![hermit_target],
                vec![Op::ModifyTargetUntilTurnEnd {
                    slot: 0,
                    defense_bonus: 1,
                    ordinary_icons: Icons::default(),
                    grants_renown: false,
                }],
                None,
            )]),
        );
        let mut dreamcatcher_host = character.clone();
        dreamcatcher_host.equipment_host = true;
        m.insert(
            "JC073".into(),
            Definition {
                abilities: vec![ability(
                    "attach",
                    "结附角色",
                    Timing::Standard,
                    vec![],
                    vec![dreamcatcher_host.clone()],
                    vec![],
                    None,
                )],
                attachment: Some(AttachmentSpec {
                    controls_host: false,
                    host_subtype_change: SubtypeChange::None,
                    host: dreamcatcher_host,
                    host_icons: Icons::default(),
                    host_temporary_icons: Some(Icons {
                        investigation: 1,
                        ..Icons::default()
                    }),
                    host_barrier: true,
                    host_defense_bonus: 0,
                    host_leaves: HostLeaveDestination::OwnerGraveyard,
                }),
                ..Default::default()
            },
        );
        m.insert(
            "JC102".into(),
            with_abilities(vec![ability(
                "damage-character",
                "行动阶段快速伤害",
                Timing::ActionFast,
                vec![],
                vec![character.clone()],
                vec![Op::DamageTarget { slot: 0, amount: 1 }],
                None,
            )]),
        );
        m.insert(
            "JC132".into(),
            with_abilities(vec![ability(
                "exhaust-region-characters",
                "横置地区角色",
                Timing::Standard,
                vec![],
                vec![region.clone()],
                vec![Op::ExhaustMatching(BoardSelector {
                    kind: EntityKind::Character,
                    relation: Relation::Any,
                    region: Some(RegionRef::Target(0)),
                    subtype: None,
                })],
                None,
            )]),
        );
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
                    visibility: SearchVisibility::Reveal,
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
        m.insert(
            "JC008".into(),
            with_abilities(vec![ability(
                "empower-until-turn-end",
                "本回合属性增强",
                Timing::Fast,
                vec![],
                vec![character.clone()],
                vec![Op::ModifyTargetUntilTurnEnd {
                    slot: 0,
                    defense_bonus: 1,
                    grants_renown: false,
                    ordinary_icons: Icons {
                        combat: 1,
                        ..Icons::default()
                    },
                }],
                None,
            )]),
        );
        m.insert(
            "JC070".into(),
            Definition {
                traits: Traits {
                    public: true,
                    renown: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        m.insert(
            "JC076".into(),
            Definition {
                traits: Traits {
                    renown: true,
                    ..Default::default()
                },
                abilities: vec![ability(
                    "draw-on-reveal",
                    "现身抓一张牌",
                    Timing::Fast,
                    vec![],
                    vec![],
                    vec![Op::Draw {
                        player: PlayerRef::Actor,
                        count: 1,
                        end: DeckEnd::Top,
                    }],
                    Some(Event::Reveal),
                )],
                ..Default::default()
            },
        );
        m.insert(
            "JC074".into(),
            with_abilities(vec![ability(
                "investigation-renown-until-turn-end",
                "本回合调查与声望",
                Timing::Fast,
                vec![],
                vec![character.clone()],
                vec![Op::ModifyTargetUntilTurnEnd {
                    slot: 0,
                    defense_bonus: 0,
                    ordinary_icons: Icons {
                        investigation: 2,
                        ..Icons::default()
                    },
                    grants_renown: true,
                }],
                None,
            )]),
        );
        m.insert(
            "JC129".into(),
            with_abilities(vec![ability(
                "control-until-turn-end",
                "本回合取得控制",
                Timing::Standard,
                vec![],
                vec![target(
                    Zone::Board,
                    EntityKind::Character,
                    Relation::EnemyTeam,
                    Range::Anywhere,
                )],
                vec![Op::GainControl {
                    slot: 0,
                    until_source_leaves: false,
                    subtype_change: SubtypeChange::None,
                }],
                None,
            )]),
        );
        let mut embrace_host = character.clone();
        embrace_host.subtype = Some("人类".into());
        embrace_host.printed_subtype = true;
        m.insert(
            "JC036".into(),
            Definition {
                abilities: vec![ability(
                    "attach",
                    "结附印刷人类",
                    Timing::Standard,
                    vec![],
                    vec![embrace_host.clone()],
                    vec![],
                    None,
                )],
                attachment: Some(AttachmentSpec {
                    host: embrace_host,
                    controls_host: true,
                    host_subtype_change: SubtypeChange::HumanToVampire,
                    host_icons: Icons::default(),
                    host_temporary_icons: None,
                    host_barrier: false,
                    host_defense_bonus: 0,
                    host_leaves: HostLeaveDestination::OwnerGraveyard,
                }),
                ..Default::default()
            },
        );
        let mut local_human = target(
            Zone::Board,
            EntityKind::Character,
            Relation::Any,
            Range::SourceRegion,
        );
        local_human.subtype = Some("人类".into());
        m.insert(
            "JZ27".into(),
            with_abilities(vec![ability(
                "control-until-source-leaves",
                "现身取得控制与奴仆",
                Timing::Fast,
                vec![],
                vec![local_human],
                vec![Op::GainControl {
                    slot: 0,
                    until_source_leaves: true,
                    subtype_change: SubtypeChange::AddSlave,
                }],
                Some(Event::Reveal),
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
                    visibility: SearchVisibility::Reveal,
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
                    controls_host: false,
                    host_subtype_change: SubtypeChange::None,
                    host,
                    host_icons: Icons {
                        combat: 1,
                        ..Default::default()
                    },
                    host_temporary_icons: None,
                    host_barrier: false,
                    host_defense_bonus: 0,
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
        m.insert(
            "WM003".into(),
            Definition {
                traits: Traits {
                    public: true,
                    ..Traits::default()
                },
                abilities: vec![ability(
                    "search-any-private",
                    "私密检索",
                    Timing::Fast,
                    vec![Cost::Assets(5), Cost::ExhaustSource],
                    vec![],
                    vec![Op::Search {
                        player: Actor,
                        filter: CardFilter::Any,
                        to_top: false,
                        optional: false,
                        visibility: SearchVisibility::Private,
                    }],
                    None,
                )],
                ..Definition::default()
            },
        );
        m.insert(
            "LC01".into(),
            Definition {
                modifiers: vec![StaticModifier::PeekOwnDeckTop],
                ..Definition::default()
            },
        );
        #[cfg(not(feature = "society-fixtures"))]
        {
            m.insert("MSJC02".into(), msjc02_definition());
            m.insert("MSJC03".into(), msjc03_definition());
            let mut search = ability(
                "search-purple-unique",
                "紫色独有检索（每局一次）",
                Timing::Standard,
                vec![Cost::Assets(4), Cost::ExhaustSource],
                vec![],
                vec![Op::Search {
                    player: Actor,
                    filter: CardFilter::PrintedColorAndUnique {
                        color: "紫".into()
                    },
                    to_top: false,
                    optional: false,
                    visibility: SearchVisibility::Reveal,
                }],
                None,
            );
            search.once_per_game = true;
            m.insert(
                "MSJC08".into(),
                with_abilities(vec![
                    ability(
                        "drawWithInitiative",
                        "先手抓牌",
                        Timing::Standard,
                        vec![Cost::Assets(3), Cost::ExhaustSource],
                        vec![],
                        vec![Op::DrawIfActorHasInitiative { count: 1 }],
                        None,
                    ),
                    search,
                ]),
            );
            m.insert(
                "MSJC11".into(),
                with_abilities(vec![
                    ability(
                        "grant-kill",
                        "授予杀伤1",
                        Timing::Standard,
                        vec![Cost::Assets(3), Cost::ExhaustSource],
                        vec![target(
                            Zone::Board,
                            EntityKind::Character,
                            Relation::Any,
                            Range::Anywhere,
                        )],
                        vec![Op::GrantTargetKillUntilTurnEnd { slot: 0 }],
                        None,
                    ),
                    ability(
                        "grant-region-retreat",
                        "本地区本方永久战斗角色获得撤回",
                        Timing::Standard,
                        vec![Cost::Assets(3), Cost::ExhaustSource],
                        vec![target(
                            Zone::Region,
                            EntityKind::Any,
                            Relation::Any,
                            Range::Anywhere,
                        )],
                        vec![Op::GrantRegionRetreatUntilTurnEnd { slot: 0 }],
                        None,
                    ),
                ]),
            );
            let mut search = ability(
                "search-white-unique",
                "白色独有检索（每局一次）",
                Timing::Standard,
                vec![Cost::Assets(4), Cost::ExhaustSource],
                vec![],
                vec![Op::Search {
                    player: Actor,
                    filter: CardFilter::PrintedColorAndUnique {
                        color: "白".into()
                    },
                    to_top: false,
                    optional: false,
                    visibility: SearchVisibility::Reveal,
                }],
                None,
            );
            search.once_per_game = true;
            m.insert(
                "MSJC06".into(),
                with_abilities(vec![
                    ability(
                        "drawWithInitiative",
                        "先手抓牌",
                        Timing::Standard,
                        vec![Cost::Assets(3), Cost::ExhaustSource],
                        vec![],
                        vec![Op::DrawIfActorHasInitiative { count: 1 }],
                        None,
                    ),
                    search,
                ]),
            );
            let mut search = ability(
                "search-yellow-unique",
                "黄色独有检索（每局一次）",
                Timing::Standard,
                vec![Cost::Assets(4), Cost::ExhaustSource],
                vec![],
                vec![Op::Search {
                    player: Actor,
                    filter: CardFilter::PrintedColorAndUnique {
                        color: "黄".into()
                    },
                    to_top: false,
                    optional: false,
                    visibility: SearchVisibility::Reveal,
                }],
                None,
            );
            search.once_per_game = true;
            m.insert(
                "MSJC01".into(),
                with_abilities(vec![
                    ability(
                        "drawWithInitiative",
                        "先手抓牌",
                        Timing::Standard,
                        vec![Cost::Assets(3), Cost::ExhaustSource],
                        vec![],
                        vec![Op::DrawIfActorHasInitiative { count: 1 }],
                        None,
                    ),
                    search,
                ]),
            );
            let mut search = ability(
                "search-black-unique",
                "黑色独有检索（每局一次）",
                Timing::Standard,
                vec![Cost::Assets(4), Cost::ExhaustSource],
                vec![],
                vec![Op::Search {
                    player: Actor,
                    filter: CardFilter::PrintedColorAndUnique {
                        color: "黑".into()
                    },
                    to_top: false,
                    optional: false,
                    visibility: SearchVisibility::Reveal,
                }],
                None,
            );
            search.once_per_game = true;
            m.insert(
                "MSJC07".into(),
                with_abilities(vec![
                    ability(
                        "drawWithInitiative",
                        "先手抓牌",
                        Timing::Standard,
                        vec![Cost::Assets(3), Cost::ExhaustSource],
                        vec![],
                        vec![Op::DrawIfActorHasInitiative { count: 1 }],
                        None,
                    ),
                    search,
                ]),
            );
        }
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
        m.insert("XQ36".into(), xq36_definition());
        m.insert("XQ46".into(), Definition {
            traits: Traits { public: true, ..Default::default() },
            ..Default::default()
        });
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
    fn jz55_closed_definition_rejects_target_policy_shape_changes_and_transplants() {
        let whole=definition("JZ55").clone();
        for variant in 0..16 {
            let mut r=definitions().clone();let d=r.get_mut("JZ55").unwrap();let a=&mut d.abilities[0];
            match variant {
                0=>a.targets[0].predicate=None,
                1=>a.targets[0].kind=EntityKind::CharacterOrHidden,
                2=>a.targets[0].relation=Relation::EnemyTeam,
                3=>a.targets[0].range=Range::SourceRegion,
                4=>a.response_policy=ResponsePolicy::Respondable,
                5=>a.timing=Timing::ActionFast,
                6=>a.event=Some(Event::Death),
                7=>a.costs.push(Cost::Assets(1)),
                8=>a.targets[0].max=2,
                9=>a.ops.push(Op::Destroy(EntityRef::Target(0))),
                10=>a.modes.push(Mode{key:"extra".into(),label:"extra".into(),targets:whole.abilities[0].targets.clone(),ops:whole.abilities[0].ops.clone()}),
                11=>a.targets[0].requires_magic=true,
                12=>a.per_turn_limit=Some(1),
                13=>d.traits.public=true,
                14=>d.modifiers.push(StaticModifier::PeekOwnDeckTop),
                _=>d.abilities.clear(),
            }
            assert!(validate_definitions(&r).is_err(),"shape {variant}");
        }
        for id in ["JC091","JC015","JC125"] {
            let mut r=definitions().clone();r.get_mut(id).unwrap().abilities=whole.abilities.clone();
            assert!(validate_definitions(&r).is_err(),"whole transplant {id}");
        }
        let mut r=definitions().clone();r.get_mut("JC091").unwrap().abilities[0].modes.push(Mode{
            key:"unique-mode".into(),label:"invalid".into(),targets:whole.abilities[0].targets.clone(),ops:whole.abilities[0].ops.clone()});
        assert!(validate_definitions(&r).is_err());
        let mut r=definitions().clone();r.get_mut("JC036").unwrap().attachment.as_mut().unwrap().host.predicate=whole.abilities[0].targets[0].predicate.clone();
        assert!(validate_definitions(&r).is_err());
    }

    #[test]
    fn jz31_closed_definition_and_operation_reject_mutations_modes_loops_and_transplants() {
        for variant in 0..15 {
            let mut r = definitions().clone();
            let d = r.get_mut("JZ31").unwrap();
            let a = &mut d.abilities[0];
            match variant {
                0 => a.event = Some(Event::Enter),
                1 => a.costs.push(Cost::ExhaustSource),
                2 => a.targets.push(target(Zone::Region, EntityKind::Any, Relation::Any, Range::Anywhere)),
                3 => a.ops.push(Op::PlaceOneInfluenceInSourceRegion),
                4 => a.ops = vec![Op::PlaceInfluence { region_instance:"i1".into(), amount:1 }],
                5 => a.response_policy = ResponsePolicy::Immediate,
                6 => a.modes.push(Mode { key:"extra".into(), label:"extra".into(), targets:vec![], ops:vec![Op::PlaceOneInfluenceInSourceRegion] }),
                7 => a.requires_ready_source = true,
                8 => a.once_per_game = true,
                9 => a.per_turn_limit = Some(1),
                10 => a.timing = Timing::Standard,
                11 => a.key = "other".into(),
                12 => d.traits.renown = true,
                13 => d.modifiers.push(StaticModifier::PeekOwnDeckTop),
                _ => d.graveyard_face_up = true,
            }
            assert!(validate_definitions(&r).is_err(), "variant {variant}");
        }
        for other in ["JZ59", "JZ24", "JC125", "XQ36"] {
            let mut r = definitions().clone();
            r.get_mut(other).unwrap().abilities = jz31_definition().abilities;
            assert!(validate_definitions(&r).is_err());
        }
        for ops in [
            vec![Op::PlaceOneInfluenceInSourceRegion],
            vec![Op::ForEachLivingPlayer(vec![Op::PlaceOneInfluenceInSourceRegion])],
            vec![Op::ForEachLivingPlayerFromActor(vec![Op::PlaceOneInfluenceInSourceRegion])],
        ] {
            let mut a = definition("XQ17").abilities[0].clone(); a.ops = ops.clone();
            assert!(validate_ability("XQ17", &a).is_err());
            a.ops.clear(); a.modes.push(Mode { key:"invalid".into(), label:"invalid".into(), targets:vec![], ops });
            assert!(validate_ability("XQ17", &a).is_err());
        }
    }

    #[test]
    fn mill_public_closed_entry_rejects_variants_modes_and_nested_transplants() {
        for variant in 0..12 {
            let mut r = definitions().clone();
            let d = r.get_mut("XQ36").unwrap();
            let a = &mut d.abilities[0];
            match variant {
                0 => a.ops = vec![Op::ForEachLivingPlayer(vec![Op::MoveDeckTopToGraveyard { player:PlayerRef::Context,count:5 }])],
                1 => a.ops = vec![Op::ForEachLivingPlayer(vec![Op::MoveDeckTopToGraveyard { player:PlayerRef::Actor,count:4 }])],
                2 => a.ops = vec![Op::ForEachLivingPlayerFromActor(vec![Op::MoveDeckTopToGraveyard { player:PlayerRef::Context,count:4 }])],
                3 => a.event = Some(Event::Death),
                4 => a.costs.push(Cost::Assets(1)),
                5 => a.targets.push(target(Zone::Player,EntityKind::Any,Relation::Any,Range::Anywhere)),
                6 => a.modes.push(Mode {key:"variant".into(),label:"variant".into(),targets:vec![],ops:a.ops.clone()}),
                7 => a.ops = vec![Op::ForEachLivingPlayer(a.ops.clone())],
                8 => a.ops.push(Op::Draw{player:PlayerRef::Actor,count:1,end:DeckEnd::Top}),
                9 => a.response_policy = ResponsePolicy::Immediate,
                10 => d.traits.public = true,
                _ => a.play_only = true,
            }
            assert!(validate_definitions(&r).is_err(), "variant {variant}");
        }
        let original = definition("XQ36").abilities[0].clone();
        for id in ["XQ46","XQ38","JC029"] {
            assert!(validate_ability(id,&original).is_err(), "direct transplant {id}");
            for depth in 0..=2 {
                let mut ops=original.ops.clone();
                for _ in 0..depth { ops=vec![Op::ForEachLivingPlayer(ops)]; }
                let mut ability=definition("XQ38").abilities[0].clone();
                ability.modes=vec![Mode{key:"transplant".into(),label:"invalid".into(),targets:vec![],ops}];
                assert!(validate_ability(id,&ability).is_err(),"mode/depth {id}/{depth}");
            }
        }
        let mut top_level=original;
        top_level.ops=vec![Op::MoveDeckTopToGraveyard{player:PlayerRef::Context,count:4}];
        assert!(validate_ability("XQ46",&top_level).is_err());
        assert!(validate_ability("XQ38",&definition("XQ38").abilities[0]).is_ok());
    }

    #[test]
    fn blue_closed_definitions_reject_variants_and_finite_operation_transplants() {
        for id in ["JC032", "JZ24", "MSJC03"] {
            for variant in 0..10 {
                let mut r = definitions().clone();
                let d = r.get_mut(id).unwrap();
                let i = if id == "MSJC03" { 1 } else { 0 };
                match variant {
                    0 => d.abilities[i].costs.push(Cost::ExhaustSource),
                    1 => d.abilities[i].per_turn_limit = Some(2),
                    2 => d.abilities[i].event = Some(Event::Enter),
                    3 => d.abilities[i].targets.push(target(Zone::Board, EntityKind::Character, Relation::Any, Range::Anywhere)),
                    4 => d.abilities[i].ops.push(Op::Draw { player: PlayerRef::Actor, count: 1, end: DeckEnd::Top }),
                    5 => d.abilities[i].ops = vec![Op::SacrificeChosen(PlayerRef::Actor)],
                    6 => d.modifiers.push(StaticModifier::PeekOwnDeckTop),
                    7 => d.abilities[i].per_turn_limit = None,
                    8 => d.abilities[i].costs = vec![Cost::Assets(1)],
                    _ => d.abilities[i].requires_ready_source = true,
                }
                // Removing an already absent limit is unchanged for these two.
                if variant == 7 && id != "JC032" { d.abilities[i].once_per_game = !d.abilities[i].once_per_game; }
                assert!(validate_definitions(&r).is_err(), "{id} variant {variant}");
            }
        }
        for (original, other) in [("JC032", "JC029"), ("JC032", "JC030"), ("JZ24", "JC029"), ("JZ24", "JC032")] {
            let mut r = definitions().clone();
            let transplanted = r[original].abilities[0].clone();
            r.get_mut(other).unwrap().abilities = vec![transplanted];
            assert!(validate_definitions(&r).is_err());
        }
        for op in [Op::JC032TopSixVampireHidden, Op::JZ24LocalSacrificeSnapshot] {
            let mut r = definitions().clone();
            r.get_mut("JC029").unwrap().abilities[0].modes.push(Mode {
                key: "unadmitted-blue-mode".into(), label: "invalid variant".into(),
                targets: vec![], ops: vec![op],
            });
            assert!(validate_definitions(&r).is_err(), "finite operations cannot be transplanted into modes");
        }
    }

    #[test]
    fn jc030_closed_definition_rejects_shape_changes_and_modifier_transplants() {
        for variant in 0..6 {
            let mut registry = definitions().clone();
            let d = registry.get_mut("JC030").unwrap();
            match variant {
                0 => d.modifiers.clear(),
                1 => d.modifiers.push(StaticModifier::PeekOwnDeckTop),
                2 => d.traits.cannot_be_equipped = false,
                3 => d.abilities[0].event = Some(Event::Enter),
                4 => d.abilities[0].targets[0].relation = Relation::EnemyTeam,
                _ => d.abilities[0].ops = vec![Op::DamageTarget { slot: 0, amount: 2 }],
            }
            assert!(validate_definitions(&registry).is_err());
        }
        for other in ["JC029", "JC125"] {
            let mut registry = definitions().clone();
            registry.get_mut(other).unwrap().modifiers.push(StaticModifier::JC030BloodAssetsVampireAndInvestigation);
            assert!(validate_definitions(&registry).is_err());
        }
    }

    #[test]
    fn green_shapes_reject_extra_modifiers_and_predicate_transplants() {
        for variant in 0..4 {
            let mut r = definitions().clone();
            match variant {
                0 => r
                    .get_mut("LC30")
                    .unwrap()
                    .modifiers
                    .push(StaticModifier::PeekOwnDeckTop),
                1 => r
                    .get_mut("JC125")
                    .unwrap()
                    .modifiers
                    .push(StaticModifier::JC018MindAssetsCombat),
                2 => r.get_mut("JC018").unwrap().traits.guard = 2,
                _ => {
                    r.get_mut("JC036")
                        .unwrap()
                        .attachment
                        .as_mut()
                        .unwrap()
                        .host
                        .predicate = Some(TargetPredicate::JC015NonHumanPrintedCostAtLeastThree)
                }
            }
            assert!(validate_definitions(&r).is_err());
        }
        let whole = jc015_definition().abilities.remove(0);
        for variant in 0..6 {
            let mut a = whole.clone();
            match variant {
                0 => a.targets[0].predicate = None,
                1 => a.targets[0].relation = Relation::Any,
                2 => a.targets[0].range = Range::SourceRegion,
                3 => a.costs = vec![Cost::SacrificeSource],
                4 => a.ops.push(Op::Destroy(EntityRef::Target(0))),
                _ => a.targets[0].printed_subtype = true,
            }
            assert!(validate_ability("JC015", &a).is_err());
        }
        assert!(validate_ability("JC129", &whole).is_err());
    }

    #[test]
    fn region_attachment_and_conversion_admit_only_the_whole_xq43() {
        for variant in 0..6 {
            let mut registry = definitions().clone();
            let d = registry.get_mut("XQ43").unwrap();
            match variant {
                0 => d.attachment = None,
                1 => d.attachment.as_mut().unwrap().host.kind = EntityKind::Character,
                2 => d.attachment.as_mut().unwrap().host.relation = Relation::ControlledByActor,
                3 => d.abilities[0].activation_only = true,
                4 => d
                    .modifiers
                    .push(StaticModifier::AttachedRegionSpiritTemporaryIconsPermanent),
                _ => d.attachment.as_mut().unwrap().host_leaves = HostLeaveDestination::OwnerHand,
            }
            assert!(validate_definitions(&registry)
                .unwrap_err()
                .contains("XQ43"));
        }
        let mut registry = definitions().clone();
        registry.insert("JC073".into(), xq43_definition());
        assert!(validate_definitions(&registry)
            .unwrap_err()
            .contains("JC073"));
    }

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
