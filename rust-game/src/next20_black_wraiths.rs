//! CQ25/XG25 finite adapters. Deaths are observed before a simultaneous wave;
//! movement and sealing use the existing primitives. No second death engine.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub(crate) const CARDS: &[&str] = &["CQ25", "XG25"];
pub(crate) const CONTEST: &str = "next20-cq25-contest";
pub(crate) const SEARCH: &str = "next20-cq25-search";
pub(crate) const MOVE: &str = "next20-xg25-move";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WraithDeath {
    pub event_id: String,
    pub turn: u32,
    pub dead: SourceSnapshot,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum WraithProgram {
    TigerSearch {
        observation: Option<Box<WraithDeath>>,
    },
    GhostMove {
        sealed_host: Option<Box<SourceSnapshot>>,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum WraithChoice {
    TigerSearch,
}
fn same<T: Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
pub(crate) fn related(a: &AbilitySpec) -> bool {
    fn owns(ops: &[Op]) -> bool {
        ops.iter().any(|o| match o {
            Op::Next20Wraith(_) => true,
            Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => owns(body),
            Op::IfTargetExhausted {
                exhausted, ready, ..
            } => owns(&[*exhausted.clone(), *ready.clone()]),
            _ => false,
        })
    }
    a.key.starts_with("next20-cq25-")
        || a.key.starts_with("next20-xg25-")
        || a.event == Some(Event::Next20WraithDeath)
        || owns(&a.ops)
        || a.modes.iter().any(|m| owns(&m.ops))
}
pub(crate) fn frame_related(f: &ResolutionFrame) -> bool {
    related(&ability(
        &f.ability_key,
        "",
        Timing::Fast,
        vec![],
        vec![],
        f.steps.iter().map(|s| s.op.clone()).collect(),
        None,
    ))
}
pub(crate) fn definition(id: &str) -> Option<Definition> {
    Some(match id {
        "CQ25" => {
            let mut d = with_abilities(vec![
                ability(
                    CONTEST,
                    "进场：进行隐秘对抗；本人展示的诅咒牌+2",
                    Timing::Fast,
                    vec![],
                    vec![],
                    vec![Op::MysticContest { actor_bonus: 0 }],
                    Some(Event::Enter),
                ),
                ability(
                    SEARCH,
                    "本地区敌方角色死亡：从本人牌库或墓地寻找伥鬼封印在本来源上；若搜牌库则洗牌",
                    Timing::Fast,
                    vec![],
                    vec![],
                    vec![Op::Next20Wraith(WraithProgram::TigerSearch {
                        observation: None,
                    })],
                    Some(Event::Next20WraithDeath),
                ),
            ]);
            // The crescent before Spirit is a condition, not an unconditional
            // printed keyword. current_spirit must call cq25_spirit below.
            d.traits.spirit = false;
            d
        }
        "XG25" => {
            let mut t = target(
                Zone::Board,
                EntityKind::Character,
                Relation::EnemyTeam,
                Range::Anywhere,
            );
            t.predicate = Some(TargetPredicate::Next20XG25AdjacentEnemy);
            let mut a = ability(
                MOVE,
                "行动1：将相邻地区的目标敌方角色移动到本地区，每回合一次；封印在角色上也可发动",
                Timing::Standard,
                vec![Cost::Assets(1)],
                vec![t],
                vec![Op::Next20Wraith(WraithProgram::GhostMove {
                    sealed_host: None,
                })],
                None,
            );
            a.activation_only = true;
            a.per_turn_limit = Some(1);
            let mut d = with_abilities(vec![a]);
            d.traits.spirit = true;
            d
        }
        _ => return None,
    })
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if CARDS.contains(&id) && !same(d, &definition(id).unwrap()) {
        return Err("黑色新卡定义不得删改完整程序".into());
    }
    Ok(())
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    let mut normalized = a.clone();
    for o in &mut normalized.ops {
        match o {
            Op::Next20Wraith(WraithProgram::TigerSearch { observation }) => *observation = None,
            Op::Next20Wraith(WraithProgram::GhostMove { sealed_host }) => *sealed_host = None,
            _ => (),
        }
    }
    if definition(id).is_none_or(|d| !d.abilities.iter().any(|x| same(x, &normalized))) {
        return Err("黑色新卡不能移植来源或改写原卡程序".into());
    }
    Ok(())
}
pub(crate) fn validate_action(id: &str, a: &Action) -> RuleResult<()> {
    if !CARDS.contains(&id) {
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
        "activate" if id == "XG25" && a.ability_id.as_deref() == Some(MOVE) => Action {
            card_id: a.card_id.clone(),
            ability_id: a.ability_id.clone(),
            target_id: a.target_id.clone(),
            ..Action::new("activate")
        },
        _ => return Err("黑色新卡只接受原角色操作和伥鬼固定移动行动".into()),
    };
    if *a != expected {
        return Err("黑色新卡不可添加额外费用、地区或选择载荷".into());
    }
    Ok(())
}
pub(crate) fn validate_choice_action(p: &Pending, a: &Action) -> RuleResult<()> {
    let ours = match &p.resolution {
        ChoiceResolution::Declare { declaration, .. } => related(&declaration.ability),
        ChoiceResolution::Frame { frame, choice } => {
            matches!(choice, FrameChoice::Next20Wraith(_)) || frame_related(frame)
        }
        _ => false,
    };
    if ours
        && *a
            != (Action {
                choice_id: a.choice_id.clone(),
                selected: a.selected.clone(),
                ..Action::new("choose")
            })
    {
        return Err("黑色新卡待选只能提交原选项".into());
    }
    Ok(())
}
impl Game {
    pub(crate) fn cq25_spirit(&self, c: &Card) -> bool {
        c.definition == "CQ25"
            && !c.face_down
            && self
                .players
                .get(c.controller)
                .is_some_and(|p| !p.eliminated && p.society_zone.secrecy > 0)
    }
    /// Added only in CQ25's own entry contest, to the initiating player's shown
    /// Curse card. It is neither a global Curse bonus nor a team/flat +2 bonus.
    pub(crate) fn cq25_contest_bonus(
        &self,
        source: &SourceSnapshot,
        key: &str,
        actor: usize,
        seat: usize,
        shown: &Card,
    ) -> u32 {
        if source.card.definition == "CQ25"
            && key == CONTEST
            && seat == actor
            && source.card.controller == actor
            && catalog::card(&shown.definition)
                .subtypes
                .iter()
                .any(|s| s == "诅咒")
        {
            2
        } else {
            0
        }
    }
    pub(crate) fn validate_cq25_contest_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if f.source.card.definition != "CQ25"
            || f.ability_key != CONTEST
            || f.actor >= self.players.len()
            || f.source.card.controller != f.actor
            || f.source.card.owner >= self.players.len()
            || f.source.card.face_down
            || f.source.play_source.is_some()
            || f.source.attachment_host_instance.is_some()
            || f.source.death_echo.is_some()
            || f.source.observed_death.is_some()
            || f.source.observed_exposure.is_some()
            || f.source.observed_mobility_grant.is_some()
            || f.source.region.is_none_or(|r| r >= self.regions.len())
            || f.source
                .source_region_instance
                .as_deref()
                .is_none_or(|id| !self.mystic_instance_valid(id))
            || !self
                .mystic_instance_number(&f.source.card.id)
                .zip(self.mystic_instance_number(&f.frame_id))
                .is_some_and(|(s, f)| s < f)
            || !f.targets.is_empty()
            || !f.already_paid.is_empty()
            || !f.resource_payments.is_empty()
            || f.chosen_region.is_some()
            || !matches!(f.steps.as_slice(),[Step { context,op: Op::MysticContest { actor_bonus: 0 } }] if *context == f.actor)
            || f.cursor > 1
            || f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked)
            || f.cursor == 1 && !matches!(f.guard, GuardState::Accepted)
        {
            return Err("伥虎进场对抗须保留原地区、本人来源和零通用加值程序".into());
        }
        self.mobility_immutable_identity(&f.source.card)
    }
    /// Capture before *any* character in a simultaneous lethal wave leaves.
    pub(crate) fn cq25_capture_sources(&self) -> Vec<SourceSnapshot> {
        self.regions
            .iter()
            .enumerate()
            .filter(|(_, r)| !r.vacant)
            .flat_map(|(r, region)| region.cards.iter().map(move |c| (r, c)))
            .filter(|(_, c)| {
                c.definition == "CQ25" && !c.face_down && !self.players[c.controller].eliminated
            })
            .map(|(r, c)| self.next20_source(c, r))
            .collect()
    }
    pub(crate) fn cq25_observe_death(&mut self, sources: &[SourceSnapshot], dead: &SourceSnapshot) {
        if dead.card.face_down
            || catalog::card(&dead.card.definition).kind != "character"
            || dead.region.is_none()
        {
            return;
        }
        let eligible = sources
            .iter()
            .filter(|source| {
                source.region == dead.region
                    && source.source_region_instance == dead.source_region_instance
                    && self.is_enemy(source.card.controller, &dead.card)
            })
            .collect::<Vec<_>>();
        // Existing games without an eligible tiger retain their exact ID stream.
        if eligible.is_empty() {
            return;
        }
        let observation = WraithDeath {
            event_id: self.id(),
            turn: self.turn,
            dead: dead.clone(),
        };
        for source in eligible {
            let mut a = definition("CQ25").unwrap().abilities.remove(1);
            a.ops = vec![Op::Next20Wraith(WraithProgram::TigerSearch {
                observation: Some(Box::new(observation.clone())),
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
    fn cq25_validate_search(&self, f: &ResolutionFrame) -> RuleResult<()> {
        let [Step {
            op:
                Op::Next20Wraith(WraithProgram::TigerSearch {
                    observation: Some(o),
                }),
            context,
        }] = f.steps.as_slice()
        else {
            return Err("伥虎缺少实际死亡观察".into());
        };
        if f.source.card.definition != "CQ25"
            || f.source.card.face_down
            || f.source.card.controller != f.actor
            || f.actor >= self.players.len()
            || !self.mystic_instance_valid(&f.source.card.id)
            || f.source.card.owner >= self.players.len()
            || f.source.play_source.is_some()
            || f.source.attachment_host_instance.is_some()
            || f.source.death_echo.is_some()
            || f.source.observed_death.is_some()
            || f.source.observed_exposure.is_some()
            || f.source.observed_mobility_grant.is_some()
            || f.ability_key != SEARCH
            || *context != f.actor
            || o.turn != self.turn
            || !self
                .mystic_instance_number(&o.dead.card.id)
                .zip(self.mystic_instance_number(&o.event_id))
                .is_some_and(|(d, e)| d < e)
            || o.dead.card.owner >= self.players.len()
            || o.dead.card.controller >= self.players.len()
            || o.dead.card.face_down
            || !catalog::catalog()
                .cards
                .iter()
                .any(|d| d.id == o.dead.card.definition && d.kind == "character")
            || !self.is_enemy(f.actor, &o.dead.card)
            || f.source.region.is_none_or(|r| r >= self.regions.len())
            || f.source.region != o.dead.region
            || f.source
                .source_region_instance
                .as_deref()
                .is_none_or(|id| !self.mystic_instance_valid(id))
            || f.source.source_region_instance != o.dead.source_region_instance
            || !f.already_paid.is_empty()
            || !f.resource_payments.is_empty()
            || !f.targets.is_empty()
            || f.chosen_region.is_some()
            || f.cursor > 1
            || f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked)
            || f.cursor == 1 && !matches!(f.guard, GuardState::Accepted)
        {
            return Err("伥虎只观察原地区敌方角色的实际死亡".into());
        }
        self.mobility_immutable_identity(&f.source.card)?;
        self.mobility_immutable_identity(&o.dead.card)?;
        Ok(())
    }
    fn cq25_search_options(&self, actor: usize) -> Vec<ChoiceOption> {
        let matches = |c: &&Card| {
            catalog::card(&c.definition).name == "伥鬼"
                && catalog::card(&c.definition).kind == "character"
        };
        self.players[actor]
            .graveyard
            .iter()
            .filter(matches)
            .map(|c| self.option(c, actor, None, Some("伥鬼（墓地）")))
            .chain(
                self.players[actor]
                    .deck
                    .iter()
                    .filter(matches)
                    .map(|c| self.option(c, actor, None, Some("伥鬼（牌库）"))),
            )
            .collect()
    }
    fn cq25_search_choices(&self, actor: usize) -> Vec<ChoiceOption> {
        let mut options = self.cq25_search_options(actor);
        // Match the current bounded-search kernel: a matching card cannot be
        // silently ignored in a searched deck. Empty-deck search still shuffles.
        if !self.players[actor].deck.iter().any(|c| {
            catalog::card(&c.definition).name == "伥鬼"
                && catalog::card(&c.definition).kind == "character"
        }) {
            options.push(ChoiceOption {
                id: "search-deck-no-card".into(),
                label: "牌库中没有伥鬼，确认搜寻并洗牌".into(),
                card: None,
            });
        }
        options
    }
    pub(crate) fn cq25_search_start(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        self.cq25_validate_search(&f)?;
        if f.cursor != 1 || !matches!(f.guard, GuardState::Accepted) {
            return Err("伥虎搜索尚未通过原帧守卫".into());
        }
        // An empty confirmation remains necessary: searching the deck and
        // failing to find still shuffles. Choosing only the graveyard does not.
        let options = self.cq25_search_choices(f.actor);
        self.choice(
            f.actor,
            SEARCH,
            "伥虎：选择本人墓地或牌库中的伥鬼，或搜索牌库而未找到".into(),
            options,
            1,
            1,
            None,
            ChoiceResolution::Frame {
                frame: Box::new(f),
                choice: FrameChoice::Next20Wraith(WraithChoice::TigerSearch),
            },
        );
        Ok(())
    }
    pub(crate) fn cq25_search_finish(
        &mut self,
        f: &ResolutionFrame,
        chooser: usize,
        selected: &[String],
    ) -> RuleResult<()> {
        self.cq25_validate_search(f)?;
        if f.cursor != 1
            || !matches!(f.guard, GuardState::Accepted)
            || chooser != f.actor
            || selected.len() != 1
        {
            return Err("伥虎只能寻找一张本人伥鬼，或确认搜索牌库未找到".into());
        }
        let id = &selected[0];
        if !self
            .cq25_search_choices(chooser)
            .iter()
            .any(|o| &o.id == id)
        {
            return Err("不能略过实际可找到的牌库伥鬼或选择外来牌".into());
        }
        if id == "search-deck-no-card" {
            self.shuffle_player(chooser);
            return Ok(());
        }
        if !self
            .cq25_search_options(chooser)
            .iter()
            .any(|o| &o.id == id)
        {
            return Err("选择不是本人牌库或墓地里的原伥鬼".into());
        }
        let (c, deck) = if let Some(i) = self.players[chooser]
            .graveyard
            .iter()
            .position(|c| &c.id == id)
        {
            (self.players[chooser].graveyard.remove(i), false)
        } else {
            let i = self.players[chooser]
                .deck
                .iter()
                .position(|c| &c.id == id)
                .ok_or("原牌库伥鬼失效")?;
            (self.players[chooser].deck.remove(i), true)
        };
        if deck {
            self.shuffle_player(chooser);
        }
        if self
            .board(&f.source.card.id)
            .is_some_and(|(_, c)| c.definition == "CQ25" && !c.face_down)
        {
            self.seal_card_on_valid_host(c, chooser, &f.source.card.id);
        } else {
            let c = self.reset_zone_card(c);
            self.commit_owner_grave(c, None);
        }
        Ok(())
    }
    /// A sealed XG25 gets only its printed MOVE exception, never its Spirit or
    /// death text. The actor/ownership ambiguity for a foreign-controlled ghost
    /// is kept out of admission until ruled; ordinary owned ghosts are supported.
    pub(crate) fn xg25_sealed_source(
        &self,
        actor: usize,
        id: &str,
    ) -> RuleResult<(SourceSnapshot, SourceSnapshot)> {
        if actor >= self.players.len() {
            return Err("伥鬼封印行动玩家不存在".into());
        }
        let s = self
            .sealed_cards
            .iter()
            .find(|s| s.card.id == id && s.card.definition == "XG25" && s.card.owner == actor)
            .ok_or("不是本人封印的伥鬼")?;
        let (r, host) = self
            .board(&s.host_id)
            .filter(|(_, c)| !c.face_down && catalog::card(&c.definition).kind == "character")
            .ok_or("伥鬼必须封印在仍在场的角色上")?;
        if self.players[actor].eliminated {
            return Err("已淘汰玩家不可发动".into());
        }
        let mut source = self.next20_source(&s.card, r);
        source.card.controller = actor;
        source.attachment_host_instance = Some(s.host_id.clone());
        Ok((source, self.next20_source(host, r)))
    }
    pub(crate) fn xg25_move_eligible(&self, f: &ResolutionFrame) -> bool {
        let Some(t) = f.targets.first() else {
            return false;
        };
        self.xg25_adjacent_target(f.actor, &f.source, &t.id)
    }
    pub(crate) fn xg25_adjacent_target(
        &self,
        actor: usize,
        source: &SourceSnapshot,
        target: &str,
    ) -> bool {
        if actor >= self.players.len()
            || source.card.definition != "XG25"
            || source.card.controller != actor
        {
            return false;
        }
        let Some((target_region, target)) = self.board(target).filter(|(_, c)| {
            !c.face_down
                && catalog::card(&c.definition).kind == "character"
                && self.is_enemy(actor, c)
        }) else {
            return false;
        };
        let Some(r) = source.region.filter(|r| {
            self.region_live(*r)
                && source.source_region_instance.as_ref() == Some(&self.regions[*r].card.id)
        }) else {
            return false;
        };
        self.regions_adjacent(r, target_region) && self.targetable(actor, target)
    }
    pub(crate) fn xg25_move(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        self.validate_xg25_move_frame(f)?;
        if f.cursor != 1 || !matches!(f.guard, GuardState::Accepted) {
            return Err("伥鬼移动尚未通过原目标守卫".into());
        }
        if self.xg25_move_eligible(f) {
            self.move_on_board_instance(&f.targets[0].id, f.source.region.unwrap());
        }
        Ok(())
    }
    pub(crate) fn validate_xg25_move_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        let [Step {
            op: Op::Next20Wraith(WraithProgram::GhostMove { sealed_host }),
            context,
        }] = f.steps.as_slice()
        else {
            return Err("伥鬼缺少完整移动程序".into());
        };
        if f.source.card.definition != "XG25"
            || f.source.card.face_down
            || f.actor >= self.players.len()
            || f.source.card.controller != f.actor
            || !self
                .mystic_instance_number(&f.source.card.id)
                .zip(self.mystic_instance_number(&f.frame_id))
                .is_some_and(|(s, f)| s < f)
            || f.source.card.owner >= self.players.len()
            || f.source.play_source.is_some()
            || f.source.death_echo.is_some()
            || f.source.observed_death.is_some()
            || f.source.observed_exposure.is_some()
            || f.source.observed_mobility_grant.is_some()
            || f.ability_key != MOVE
            || *context != f.actor
            || f.targets.len() != 1
            || !same(
                &f.targets[0].spec,
                &definition("XG25").unwrap().abilities[0].targets[0],
            )
            || f.source.region.is_none_or(|r| r >= self.regions.len())
            || f.source.source_region_instance.is_none()
            || f.chosen_region.is_some()
            || f.cursor > 1
            || f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked)
            || f.cursor == 1 && !matches!(f.guard, GuardState::Accepted)
            || !matches!(f.already_paid.as_slice(),[PaidCost::Assets(_)] if f.money_units(0) == 1)
            || !self.turn_ability_usage.iter().any(|u| {
                u.source_instance == f.source.card.id
                    && u.ability_key == MOVE
                    && u.turn == self.turn
                    && u.uses == 1
            })
        {
            return Err("伥鬼须保留原地区、目标、支付1和每回合一次记录".into());
        }
        let t = &f.targets[0];
        let target_controller = t
            .public
            .controller
            .as_deref()
            .and_then(|s| s.strip_prefix('p'))
            .and_then(|s| s.parse::<usize>().ok());
        if t.id != t.public.instance_id
            || t.public.kind != "character"
            || !t.public.valid
            || t.public.status != "valid"
            || t.public.invalid_reason.is_some()
            || target_controller.is_none_or(|seat| {
                seat >= self.players.len() || self.team(seat) == self.team(f.actor)
            })
            || t.public
                .region
                .is_none_or(|r| !self.regions_adjacent(f.source.region.unwrap(), r))
            || t.region_instance.is_some()
        {
            return Err("伥鬼必须保留声明时的相邻敌方角色目标".into());
        }
        self.mobility_immutable_identity(&f.source.card)?;
        match sealed_host {
            Some(host)
                if host.region == f.source.region
                    && host.source_region_instance == f.source.source_region_instance
                    && f.source.attachment_host_instance.as_deref() == Some(&host.card.id)
                    && !host.card.face_down
                    && catalog::card(&host.card.definition).kind == "character"
                    && f.source.card.owner == f.actor =>
            {
                ()
            }
            None if f.source.attachment_host_instance.is_none() => (),
            _ => return Err("伥鬼封印发动须保留原角色载体和其地区".into()),
        }
        Ok(())
    }
    pub(crate) fn validate_next20_black_declaration(&self, d: &Declaration) -> RuleResult<()> {
        validate_ability(&d.source.card.definition, &d.ability)?;
        if d.ability.event.is_none()
            || d.actor >= self.players.len()
            || d.source.card.controller != d.actor
            || d.source.card.owner >= self.players.len()
            || d.source.card.face_down
            || d.source.play_source.is_some()
            || d.source.region.is_none_or(|r| r >= self.regions.len())
            || d.source.death_echo.is_some()
            || d.source.observed_death.is_some()
            || d.source.observed_exposure.is_some()
            || d.source.observed_mobility_grant.is_some()
            || d.source.attachment_host_instance.is_some()
            || !self.mystic_instance_valid(&d.source.card.id)
            || d.source
                .source_region_instance
                .as_deref()
                .is_none_or(|id| !self.mystic_instance_valid(id))
        {
            return Err("黑色新卡声明须保留原来源与地区实体".into());
        }
        self.mobility_immutable_identity(&d.source.card)?;
        if d.ability.key == SEARCH {
            let f = self.make_frame(d.actor, d.source.clone(), &d.ability, vec![], vec![], None);
            self.cq25_validate_search(&f)?;
        }
        Ok(())
    }
    pub(crate) fn validate_next20_black_program_frame(
        &self,
        f: &ResolutionFrame,
    ) -> RuleResult<()> {
        if !self
            .mystic_instance_number(&f.source.card.id)
            .zip(self.mystic_instance_number(&f.frame_id))
            .is_some_and(|(s, f)| s < f)
        {
            return Err("黑色新卡帧须保留先来源、后响应的真实身份".into());
        }
        match f.ability_key.as_str() {
            CONTEST => self.validate_cq25_contest_frame(f),
            SEARCH => self.cq25_validate_search(f),
            MOVE => self.validate_xg25_move_frame(f),
            _ => Err("黑色新卡操作不能借用其他程序来源".into()),
        }
    }
    pub(crate) fn validate_next20_black_pending(&self, p: &Pending) -> RuleResult<()> {
        self.validate_black_declaration_pending(p)?;
        // CQ25 commitments and rewards are owned by the shared contest
        // protocol; its pack-specific search owns the Wraith continuation.
        if let ChoiceResolution::Frame {
            frame,
            choice: FrameChoice::MysticCommit { .. } | FrameChoice::MysticReward { .. },
        } = &p.resolution
        {
            if frame.ability_key == CONTEST && frame.source.card.definition == "CQ25" {
                return self.validate_cq25_contest_frame(frame);
            }
        }
        if matches!(&p.resolution, ChoiceResolution::Frame { frame,choice } if frame_related(frame) && !matches!(choice,FrameChoice::Next20Wraith(_)))
        {
            return Err("黑色新卡不能借用其他卡的待选续体".into());
        }
        let ChoiceResolution::Frame {
            frame: f,
            choice: FrameChoice::Next20Wraith(WraithChoice::TigerSearch),
        } = &p.resolution
        else {
            return Ok(());
        };
        self.cq25_validate_search(f)?;
        let options = self.cq25_search_choices(f.actor);
        if f.cursor != 1
            || !matches!(f.guard, GuardState::Accepted)
            || p.seat != f.actor
            || p.choice.id.is_empty()
            || p.choice.player_id != player_id(f.actor)
            || p.choice.kind != SEARCH
            || p.choice.min != Some(1)
            || p.choice.max != Some(1)
            || p.choice.allow_decline != Some(false)
            || p.choice.amount.is_some()
            || !p.choice.preview_cards.is_empty()
            || !same(&p.choice.options, &options)
        {
            return Err("伥虎搜索须保留原玩家、原私有选项及单张限制".into());
        }
        Ok(())
    }
    pub(crate) fn validate_next20_black_state(&self) -> RuleResult<()> {
        let mut frames = BTreeSet::new();
        let mut events = BTreeSet::new();
        let remember = |source: &SourceSnapshot,
                        key: &str,
                        ops: &[Op],
                        events: &mut BTreeSet<(String, String, String)>|
         -> RuleResult<()> {
            let event = match ops {
                [Op::Next20Wraith(WraithProgram::TigerSearch {
                    observation: Some(o),
                })] => o.event_id.clone(),
                _ => key.into(),
            };
            if !events.insert((source.card.id.clone(), key.into(), event)) {
                return Err("伥虎不能重复保存同一次原死亡或进场观察".into());
            }
            Ok(())
        };
        for item in &self.stack {
            if let Some(f) = &item.frame {
                self.validate_next20_black_play_frame(f)?;
                if CARDS.contains(&f.source.card.definition.as_str())
                    && matches!(f.ability_key.as_str(), "deploy" | "reveal")
                    && (item.id != f.frame_id
                        || item.controller != f.actor
                        || item.card.is_none()
                        || item.deploy_region != f.source.region
                        || item.reveal != (f.ability_key == "reveal")
                        || item.target.is_some()
                        || item.card.as_ref().is_some_and(|c| {
                            c.definition != f.source.card.definition
                                || c.owner != f.source.card.owner
                                || c.controller != f.actor
                        }))
                {
                    return Err("黑卡派遣/现身的原响应对象被改写".into());
                }
                if frame_related(f) {
                    self.validate_next20_black_program_frame(f)?;
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
                        return Err("黑色新卡响应对象或程序阶段无效".into());
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
                self.validate_next20_black_play_frame(frame)?;
            }
            match e {
                Effect::Declare { declaration: d } if related(&d.ability) => {
                    self.validate_next20_black_declaration(d)?;
                    remember(&d.source, &d.ability.key, &d.ability.ops, &mut events)?;
                }
                Effect::Frame { frame: f } if frame_related(f) => {
                    return Err("黑色新卡不能绕过响应直接注入续帧".into())
                }
                _ => (),
            }
        }
        if let Some(p) = &self.pending {
            self.validate_next20_black_pending(p)?;
            if let ChoiceResolution::Frame { frame, .. } = &p.resolution {
                self.validate_next20_black_play_frame(frame)?;
            }
            match &p.resolution {
                ChoiceResolution::Declare { declaration: d, .. } if related(&d.ability) => {
                    self.validate_next20_black_declaration(d)?;
                    remember(&d.source, &d.ability.key, &d.ability.ops, &mut events)?;
                }
                ChoiceResolution::Frame { frame: f, .. } if frame_related(f) => {
                    self.validate_next20_black_pending(p)?;
                    if !frames.insert(f.frame_id.clone()) {
                        return Err("黑色待选与响应重复使用原帧".into());
                    }
                }
                _ => (),
            }
        }
        for state in self.mystic_states() {
            if state.frame.ability_key == CONTEST {
                self.validate_cq25_contest_frame(&state.frame)?;
            }
        }
        Ok(())
    }
    /// Resolution-time condition query, not a death-time declaration gate.
    /// The production pre-grave death/response continuation remains NOT WIRED.
    /// A future caller must hold the departed card outside physical zones until
    /// this finishes; committing to grave first would emit incorrect effects.
    pub(crate) fn xg25_replaces_death(&self, source: &SourceSnapshot) -> bool {
        source.card.definition == "XG25"
            && !source.card.face_down
            && source.card.controller < self.players.len()
            && self.players[source.card.controller].society_zone.secrecy > 0
    }
    pub(crate) fn xg25_death_target(&self) -> TargetSlotSpec {
        target(
            Zone::Board,
            EntityKind::Character,
            Relation::Any,
            Range::Anywhere,
        )
    }
    pub(crate) fn xg25_seal_instead_of_grave(
        &mut self,
        departed: Card,
        source: &SourceSnapshot,
        target: &str,
    ) -> RuleResult<()> {
        if source.card.definition != "XG25"
            || source.card.face_down
            || source.card.controller >= self.players.len()
            || departed.definition != "XG25"
            || departed.id != source.card.id
            || departed.owner >= self.players.len()
            || departed.owner != source.card.owner
            || !self.mystic_instance_valid(&departed.id)
            || source.region.is_none_or(|r| r >= self.regions.len())
            || source
                .source_region_instance
                .as_deref()
                .is_none_or(|id| !self.mystic_instance_valid(id))
            || self.batch3_physical_cards().any(|c| c.id == departed.id)
        {
            return Err("伥鬼替代必须持有尚未入墓的原死亡实例".into());
        }
        let actor = source.card.controller;
        if self.xg25_replaces_death(source)
            && self.valid_binding(actor, source, &self.xg25_death_target(), target)
            && self.seal_host_valid(target)
        {
            self.seal_card_on_valid_host(departed, actor, target);
        } else {
            let c = self.reset_zone_card(departed);
            self.commit_owner_grave(c, None);
        }
        Ok(())
    }
}

impl Game {
    /// User ruling 2026-10-25: only the sealed ghost's owner activates/pays/uses
    /// its once-per-turn slot. This exception never changes board control.
    pub(crate) fn xg25_begin_sealed_action(&mut self, actor: usize, a: &Action) -> RuleResult<()> {
        validate_action("XG25", a)?;
        if a.ability_id.as_deref() != Some(MOVE) {
            return Err("封印伥鬼仅能发动原移动行动".into());
        }
        let (source, host) =
            self.xg25_sealed_source(actor, a.card_id.as_deref().ok_or("缺少封印伥鬼")?)?;
        let mut spec = definition("XG25").unwrap().abilities.remove(0);
        spec.ops = vec![Op::Next20Wraith(WraithProgram::GhostMove {
            sealed_host: Some(Box::new(host)),
        })];
        self.check_timing(actor, &spec)?;
        let targets = self.bind_action(actor, &source, &spec, a)?;
        let paid = self.pay_ability_costs(actor, &source, &spec, a)?;
        let frame = self.make_frame(actor, source, &spec, targets, paid, None);
        self.dispatch_frame(
            frame,
            "封印伥鬼：拥有者支付1移动相邻敌方角色".into(),
            None,
            spec.response_policy,
        );
        Ok(())
    }
    pub(crate) fn xg25_sealed_actions(&self, actor: usize) -> Vec<(Action, String)> {
        let spec = &definition("XG25").unwrap().abilities[0];
        let mut actions = vec![];
        for sealed in &self.sealed_cards {
            if sealed.card.definition != "XG25" || sealed.card.owner != actor {
                continue;
            }
            let Ok((source, _)) = self.xg25_sealed_source(actor, &sealed.card.id) else {
                continue;
            };
            for target in self.binding_options(actor, &source, &spec.targets[0]) {
                actions.push((
                    Action {
                        card_id: Some(sealed.card.id.clone()),
                        ability_id: Some(MOVE.into()),
                        target_id: Some(target.id),
                        ..Action::new("activate")
                    },
                    "封印伥鬼：拥有者支付1，移动相邻敌方角色至载体原地区（每回合一次）".into(),
                ));
            }
        }
        actions
    }
    pub(crate) fn validate_next20_black_play_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if !CARDS.contains(&f.source.card.definition.as_str())
            || !matches!(f.ability_key.as_str(), "deploy" | "reveal")
        {
            return Ok(());
        }
        let c = &f.source.card;
        if f.actor >= self.players.len()
            || c.controller != f.actor
            || c.owner >= self.players.len()
            || !self
                .mystic_instance_number(&c.id)
                .zip(self.mystic_instance_number(&f.frame_id))
                .is_some_and(|(s, f)| s < f)
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
            || !matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)] if ids.iter().all(|id|f.source.valid_printed_payment_id(id)) && ids.iter().collect::<BTreeSet<_>>().len()==ids.len())
            || f.money_units(0) > catalog::card(&c.definition).cost as usize
        {
            return Err("黑卡派遣/现身须保留原地区、来源及真实费用".into());
        }
        self.mobility_immutable_identity(c)?;
        self.validate_resource_frame(f)
    }
    fn validate_black_declaration_pending(&self, p: &Pending) -> RuleResult<()> {
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
        self.validate_next20_black_declaration(d)?;
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
        if p.seat != d.actor
            || p.choice.id.is_empty()
            || !matches!(stage, DeclareChoice::Accept)
            || !same(&p.choice, &expected)
        {
            return Err("黑卡触发须由原玩家选择发动或放弃".into());
        }
        Ok(())
    }
}
