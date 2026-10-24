//! BQ039/WM028 use event-time witnesses and existing declaration/prepare queues.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub(crate) const CARDS: &[&str] = &["BQ039", "WM028"];
pub(crate) const EYE_WIN: &str = "import20-bq039-battle-win";
pub(crate) const HALL_PREP: &str = "import20-wm028-prepare";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EyeBattleObservation {
    pub event_id: String,
    pub turn: u32,
    pub region: usize,
    pub region_instance: String,
    pub host: Card,
    pub combat: u32,
    pub winner_team: usize,
}
fn same<T: Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
pub(crate) fn definition(id: &str) -> Option<Definition> {
    let host = match id {
        "BQ039" => target(Zone::Board, EntityKind::Character, Relation::Any, Range::Anywhere),
        "WM028" => target(Zone::Society, EntityKind::Any, Relation::ControlledByActor, Range::Anywhere),
        _ => return None,
    };
    let mut attach = ability("attach", "结附目标", Timing::Standard, vec![], vec![host.clone()], vec![], None);
    attach.play_only = true;
    let mut abilities = vec![attach];
    if id == "BQ039" {
        abilities.push(ability(EYE_WIN, "赢得宿主参与的战斗：目标敌方玩家弃1", Timing::Fast,
            vec![], vec![target(Zone::Player, EntityKind::Any, Relation::EnemyTeam, Range::Anywhere)],
            vec![Op::BQ039BattleDiscard { observation: None }], Some(Event::BQ039HostBattleWon)));
        abilities.push(crate::import20_contest::hand_ability(id).unwrap());
    } else {
        abilities.push(ability(HALL_PREP, "准备强制触发：持有先手标志则隐秘对抗", Timing::Fast,
            vec![], vec![], vec![Op::WM028Prepare { event_id: None }], Some(Event::Prepare)));
    }
    Some(Definition { abilities, attachment: Some(AttachmentSpec {
        controls_host: false, host_subtype_change: SubtypeChange::None, host,
        host_icons: Icons::default(), host_temporary_icons: None, host_barrier: false,
        host_defense_bonus: 0, host_leaves: HostLeaveDestination::OwnerGraveyard,
    }), ..Definition::default() })
}
pub(crate) fn owns(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::BQ039BattleDiscard { .. } | Op::WM028Prepare { .. } => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => owns(body),
        Op::IfTargetExhausted { exhausted, ready, .. } =>
            owns(std::slice::from_ref(exhausted)) || owns(std::slice::from_ref(ready)),
        _ => false,
    })
}
pub(crate) fn related(id: &str, a: &AbilitySpec) -> bool {
    CARDS.contains(&id) || matches!(a.key.as_str(), EYE_WIN | HALL_PREP | crate::import20_contest::EYE_HAND)
        || a.event == Some(Event::BQ039HostBattleWon) || owns(&a.ops) || a.modes.iter().any(|m| owns(&m.ops))
}
pub(crate) fn frame_related(f: &ResolutionFrame) -> bool {
    CARDS.contains(&f.source.card.definition.as_str())
        || matches!(f.ability_key.as_str(), EYE_WIN | HALL_PREP | crate::import20_contest::EYE_HAND)
        || f.steps.iter().any(|s| owns(std::slice::from_ref(&s.op)))
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if id == "WM028" && a.event == Some(Event::Prepare) { return crate::jc035::validate_ability(id, a); }
    let mut expected = definition(id).and_then(|d| d.abilities.into_iter().find(|b| b.key == a.key))
        .ok_or("新隐秘附属能力来源或程序无效")?;
    if a.key == EYE_WIN {
        if let [Op::BQ039BattleDiscard { observation }] = a.ops.as_slice() {
            expected.ops = vec![Op::BQ039BattleDiscard { observation: observation.clone() }];
        }
    }
    if same(a, &expected) { Ok(()) } else { Err("新隐秘附属只能保留完整印刷程序".into()) }
}
pub(crate) fn validate_action(id: &str, a: &Action) -> RuleResult<()> {
    if !CARDS.contains(&id) { return Ok(()); }
    let expected = if a.kind == "play" {
        Action { card_id: a.card_id.clone(), ability_id: a.ability_id.clone(), target_id: a.target_id.clone(), ..Action::new("play") }
    } else if id == "BQ039" && a.kind == "activate" && a.ability_id.as_deref() == Some(crate::import20_contest::EYE_HAND) {
        Action { card_id: a.card_id.clone(), ability_id: a.ability_id.clone(), ..Action::new("activate") }
    } else { return Err("附属触发不能改为主动行动".into()); };
    if *a == expected { Ok(()) } else { Err("新隐秘附属行动不可夹带额外目标或费用".into()) }
}
pub(crate) fn admitted_start(id: &str, key: &str, bonus: u32) -> bool {
    id == "WM028" && key == HALL_PREP && bonus == 0
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if CARDS.contains(&id) && definition(id).is_none_or(|expected| !same(d, &expected)) {
        Err("新隐秘附属须保留完整宿主、费用和触发定义".into())
    } else { Ok(()) }
}

impl Game {
    pub(crate) fn import20_eye_battle_observers(&mut self, team: usize, region: usize) -> Vec<Declaration> {
        let sources = self.attachments.iter().filter(|a| a.card.definition == "BQ039" && !a.card.face_down
            && self.team(a.card.controller) == team && self.attachment_host_valid(a))
            .filter_map(|a| {
                // Exactly the physical set counted by this combat, including
                // either side's host. Inherited icons are not another participant.
                let host = self.regions[region].cards.iter().chain(self.region_attachment_cards(region))
                    .find(|c| c.id == a.host_id && !c.face_down && !c.exhausted
                        && catalog::has_kind(catalog::card(&c.definition), "character")
                        && self.icons(c, region).combat > 0)?;
                Some((self.source_snapshot(&a.card, Some(region)), host.clone(), self.icons(host, region).combat))
            }).collect::<Vec<_>>();
        if sources.is_empty() { return vec![]; }
        let event_id = self.id();
        sources.into_iter().map(|(source, host, combat)| {
            let mut ability = definition("BQ039").unwrap().abilities.remove(1);
            ability.ops = vec![Op::BQ039BattleDiscard { observation: Some(Box::new(EyeBattleObservation {
                event_id: event_id.clone(), turn: self.turn, region,
                region_instance: self.regions[region].card.id.clone(), host, combat, winner_team: team,
            })) }];
            Declaration { actor: source.card.controller, source, ability }
        }).collect()
    }
    fn import20_eye_witness(&self, actor: usize, source: &SourceSnapshot, o: &EyeBattleObservation) -> RuleResult<()> {
        if actor >= self.players.len() || source.card.definition != "BQ039" || source.card.controller != actor
            || source.card.owner >= self.players.len() || source.card.face_down || source.play_source.is_some()
            || source.observed_death.is_some() || source.observed_exposure.is_some()
            || source.observed_mobility_grant.is_some() || source.death_echo.is_some()
            || source.region != Some(o.region) || source.source_region_instance.as_ref() != Some(&o.region_instance)
            || source.attachment_host_instance.as_ref() != Some(&o.host.id)
            || o.turn != self.turn || o.region >= self.regions.len() || o.winner_team != self.team(actor)
            || o.host.face_down || o.host.exhausted || o.host.controller >= self.players.len()
            || o.host.owner >= self.players.len() || !catalog::catalog().cards.iter()
                .any(|d| d.id == o.host.definition && catalog::has_kind(d, "character")) || o.combat == 0
            || [&source.card.id, &o.host.id, &o.region_instance].iter().any(|id|
                !self.mystic_instance_number(id).zip(self.mystic_instance_number(&o.event_id)).is_some_and(|(source,event)| source < event)) {
            return Err("恫吓之瞳缺少原宿主实际参与且附属方获胜的战斗观察".into());
        }
        self.mobility_immutable_identity(&source.card)?;
        self.mobility_immutable_identity(&o.host)
    }
    pub(crate) fn validate_import20_observer_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if d.source.card.definition == "WM028" {
            if d.ability.key != HALL_PREP || d.ability.event != Some(Event::Prepare) {
                return Err("阴影厅堂声明只能来自原准备触发".into());
            }
            return self.jc035_declaration(d);
        }
        validate_ability(&d.source.card.definition, &d.ability)?;
        let [Op::BQ039BattleDiscard { observation: Some(o) }] = d.ability.ops.as_slice()
            else { return Err("恫吓之瞳不能伪造主动获胜触发".into()); };
        self.import20_eye_witness(d.actor, &d.source, o)
    }
    pub(crate) fn validate_import20_observer_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if f.ability_key == crate::import20_contest::EYE_HAND { return self.validate_import20_frame(f); }
        if f.ability_key == HALL_PREP { return self.validate_jc035_frame(f); }
        let id = f.source.card.definition.as_str();
        if f.ability_key == "attach" {
            let expected = definition(id).ok_or("隐秘附属来源不符")?.abilities.remove(0);
            let s = &f.source;
            if f.actor >= self.players.len() || s.card.controller != f.actor || s.card.face_down
                || s.region.is_some() || s.source_region_instance.is_some() || s.attachment_host_instance.is_some()
                || s.play_source.is_some() || s.death_echo.is_some() || s.observed_death.is_some()
                || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some()
                || f.chosen_region.is_some() || !f.steps.is_empty() || f.cursor != 0
                || !matches!(f.guard, GuardState::Unchecked | GuardState::Accepted)
                || f.targets.len() != 1 || !same(&f.targets[0].spec, &expected.targets[0])
                || !matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)] if ids.len() <= catalog::card(id).cost as usize
                    && ids.iter().collect::<BTreeSet<_>>().len()==ids.len()
                    && ids.iter().all(|id| *id!=s.card.id && self.mystic_instance_number(id)
                        .zip(self.mystic_instance_number(&f.frame_id)).is_some_and(|(a,f)|a<f))) {
                return Err("隐秘附属出牌须保留原目标、来源和实际费用".into());
            }
            return self.mobility_immutable_identity(&s.card);
        }
        let [Step { context, op: Op::BQ039BattleDiscard { observation: Some(o) } }] = f.steps.as_slice()
            else { return Err("新隐秘附属只准入完整准备或宿主战斗程序".into()); };
        self.import20_eye_witness(f.actor, &f.source, o)?;
        let expected = definition("BQ039").unwrap().abilities.remove(1);
        if f.ability_key != EYE_WIN || *context != f.actor || f.cursor > 1
            || f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked)
            || f.cursor == 1 && !matches!(f.guard, GuardState::Accepted)
            || f.chosen_region.is_some() || !f.already_paid.is_empty() || f.targets.len() != 1
            || !same(&f.targets[0].spec, &expected.targets[0])
            || !self.mystic_instance_number(&o.event_id).zip(self.mystic_instance_number(&f.frame_id)).is_some_and(|(event,frame)|event<frame) {
            return Err("恫吓之瞳只消费原战斗事件与一个敌方玩家目标".into());
        }
        Ok(())
    }
}
