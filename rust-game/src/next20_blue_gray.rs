//! Seven complete printed programs.
//! Flow reference: RETORETO/UnderWorldPro @ 0d60c78b68697867697f20f0c171e41445735cc4,
//! Cards/Effects/PlayerCards/JC/JC039_wangzuohuihuiji.cs:23-76 (two activations);
//! Script/Manager/DeckSearchFilter.cs:51-59 (exact printed-cost predicate).
//! Adaptation uses our frozen frames; original card images override the external
//! JC039 missing paid-reveal channel and incomplete unique-card metadata.
//! The other six have templates only in that reference revision. XG010's missing
//! first-action cost 1, CQ008's wrong target-region destination, and BQ059's omitted
//! X <= 5 limit are not copied. The image hashes and printed fields are pinned in
//! data/cards.json via card-specifications.json. Search choices use the existing
//! frame, payment, attachment and preparation queues; no separate execution kernel.
use crate::{
    catalog,
    engine::RuleResult,
    model::*,
    rules::{self, *},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub(crate) const CARDS: &[&str] = &["JC039", "JZ23", "JZ28", "XG10", "CQ08", "BQ059", "CQ17"];
const SEARCH: &str = "next-blue-gray-search";
const DESCRIPTION: &str = "选择一张符合条件的牌；没有符合牌时确认空选，然后洗牌。";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalVampireReveal {
    pub event_id: String,
    pub turn: u32,
    pub revealed: SourceSnapshot,
    pub vampire: bool,
}
fn same<T: Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
fn owns(ops: &[Op]) -> bool {
    ops.iter().any(|o| match o {
        Op::NextBlueGraySearch
        | Op::JZ23RevealReward { .. }
        | Op::JZ28DiscardSearch { .. }
        | Op::CQ08Prepare { .. }
        | Op::BQ059EntryShield => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => owns(body),
        Op::IfTargetExhausted {
            exhausted, ready, ..
        } => owns(&[*exhausted.clone(), *ready.clone()]),
        _ => false,
    })
}
pub(crate) fn related_ability(id: &str, a: &AbilitySpec) -> bool {
    (CARDS.contains(&id) && !matches!(a.key.as_str(), "renown" | "jc089-combat-glory"))
        || a.key.starts_with("next-bg-")
        || owns(&a.ops)
        || a.modes.iter().any(|m| owns(&m.ops))
        || a.targets
            .iter()
            .any(|s| s.predicate == Some(TargetPredicate::JC039BlueCharacter))
}
fn related(f: &ResolutionFrame) -> bool {
    CARDS.contains(&f.source.card.definition.as_str())
        || f.ability_key.starts_with("next-bg-")
        || f.steps.iter().any(|s| owns(std::slice::from_ref(&s.op)))
}
fn activation(
    key: &str,
    label: &str,
    timing: Timing,
    cost: u32,
    targets: Vec<TargetSlotSpec>,
) -> AbilitySpec {
    let mut a = rules::ability(
        key,
        label,
        timing,
        vec![Cost::Assets(cost), Cost::ExhaustSource],
        targets,
        vec![Op::NextBlueGraySearch],
        None,
    );
    a.activation_only = true;
    a
}
pub(crate) fn definition(id: &str) -> Option<Definition> {
    let d = match id {
        "JC039" => {
            let mut d = rules::attachment_definition(rules::target(
                Zone::Society,
                EntityKind::Any,
                Relation::ControlledByActor,
                Range::Anywhere,
            ));
            let mut discount = rules::ability(
                "next-bg-jc039-discount",
                "快速行动、横置：本回合下一张蓝牌打出或付费翻面减1",
                Timing::Fast,
                vec![Cost::ExhaustSource],
                vec![],
                vec![Op::CostReductionOnFaceUpOrPaidReveal {
                    filter: CardFilter::PrintedColor("蓝".into()),
                    amount: 1,
                }],
                None,
            );
            discount.activation_only = true;
            let mut target = rules::target(
                Zone::Board,
                EntityKind::Character,
                Relation::ControlledByActor,
                Range::Anywhere,
            );
            target.predicate = Some(TargetPredicate::JC039BlueCharacter);
            let mut hide = rules::ability(
                "next-bg-jc039-hide",
                "快速行动3、横置：目标本方蓝色角色潜伏",
                Timing::Fast,
                vec![Cost::Assets(3), Cost::ExhaustSource],
                vec![target],
                vec![Op::Hide(EntityRef::Target(0))],
                None,
            );
            hide.activation_only = true;
            d.abilities.extend([discount, hide]);
            d
        }
        "JZ23" => rules::with_abilities(vec![rules::ability(
            "next-bg-jz23-reveal",
            "其他本方吸血鬼在原地区现身：支付1，永久战斗标志，然后目标玩家弃1",
            Timing::Fast,
            vec![Cost::Assets(1)],
            vec![rules::target(
                Zone::Player,
                EntityKind::Any,
                Relation::Any,
                Range::Anywhere,
            )],
            vec![
                Op::JZ23RevealReward { observation: None },
                Op::Discard {
                    player: PlayerRef::Target(0),
                    count: Some(1),
                    redraw: false,
                    optional: false,
                },
            ],
            Some(Event::VampireRevealed),
        )]),
        "JZ28" => rules::with_abilities(vec![
            rules::ability(
                "raid-2",
                "袭击2",
                Timing::Fast,
                vec![],
                vec![rules::target(
                    Zone::Board,
                    EntityKind::Character,
                    Relation::Any,
                    Range::SourceRegion,
                )],
                vec![Op::DamageTarget { slot: 0, amount: 2 }],
                Some(Event::Reveal),
            ),
            rules::ability(
                "next-bg-jz28-discard",
                "敌方弃牌：查看顶四张，展示吸血鬼并暗藏于原地区，然后洗牌",
                Timing::Fast,
                vec![],
                vec![],
                vec![Op::JZ28DiscardSearch { observation: None }],
                Some(Event::DiscardObserved),
            ),
        ]),
        "XG10" => {
            let mut vampire = rules::target(
                Zone::Board,
                EntityKind::Character,
                Relation::Any,
                Range::Anywhere,
            );
            vampire.subtype = Some("吸血鬼".into());
            let mut human = rules::target(
                Zone::Board,
                EntityKind::Character,
                Relation::Any,
                Range::Anywhere,
            );
            human.subtype = Some("人类".into());
            rules::with_abilities(vec![
                activation(
                    "next-bg-xg10-blood",
                    "行动1、横置：检索鲜血结附目标吸血鬼，然后洗牌",
                    Timing::Standard,
                    1,
                    vec![vampire],
                ),
                activation(
                    "next-bg-xg10-embrace",
                    "行动5、横置：检索初拥结附目标人类，然后洗牌",
                    Timing::Standard,
                    5,
                    vec![human],
                ),
            ])
        }
        "CQ08" => rules::with_abilities(vec![
            rules::ability(
                "next-bg-cq08-reveal",
                "现身触发：检索诡异三姐妹，展示后暗藏于原地区并洗牌",
                Timing::Fast,
                vec![],
                vec![],
                vec![Op::NextBlueGraySearch],
                Some(Event::Reveal),
            ),
            rules::ability(
                "next-bg-cq08-prepare",
                "准备触发：操控三个三姐妹时检索独有吸血鬼，置入原地区；同名共享每回合一次",
                Timing::Fast,
                vec![],
                vec![],
                vec![Op::CQ08Prepare { event_id: None }],
                Some(Event::Prepare),
            ),
        ]),
        "BQ059" => {
            let mut abilities = vec![rules::ability(
                "next-bg-bq059-shield",
                "护盾2：进场强制触发放置两个护盾标志",
                Timing::Fast,
                vec![],
                vec![],
                vec![Op::BQ059EntryShield],
                Some(Event::Enter),
            )];
            for x in 0..=5 {
                abilities.push(activation(&format!("next-bg-bq059-search-{x}"), &format!("快速行动X={x}、横置：检索印刷费用{x}的士兵、工程师或机械，置入原地区并洗牌"), Timing::Fast, x, vec![]));
            }
            let mut d = rules::with_abilities(abilities);
            d.traits.public = true;
            d.traits.kill = 4;
            d.traits.guard = 4;
            d
        }
        "CQ17" => {
            let mut d = rules::with_abilities(vec![activation(
                "next-bg-cq17-search",
                "快速行动2、横置：检索印刷费用2的士兵，置入原地区并洗牌",
                Timing::Fast,
                2,
                vec![],
            )]);
            d.traits.public = true;
            d.traits.guard = 1;
            d
        }
        _ => return None,
    };
    Some(d)
}
fn normalized(a: &AbilitySpec) -> AbilitySpec {
    let mut a = a.clone();
    for op in &mut a.ops {
        match op {
            Op::JZ23RevealReward { observation } => *observation = None,
            Op::JZ28DiscardSearch { observation } => *observation = None,
            Op::CQ08Prepare { event_id } => *event_id = None,
            _ => {}
        }
    }
    a
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if definition(id).is_none_or(|d| {
        !d.abilities
            .iter()
            .any(|expected| same(expected, &normalized(a)))
    }) {
        return Err("蓝灰新增卡只准入原图对应的完整固定程序".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if definition(id).is_some_and(|expected| !same(d, &expected))
        || !CARDS.contains(&id) && d.abilities.iter().any(|a| related_ability(id, a))
    {
        return Err("蓝灰新增卡完整定义不可修改或移植".into());
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
        "play" | "activate" => Action {
            card_id: a.card_id.clone(),
            ability_id: a.ability_id.clone(),
            target_id: if a.kind == "play"
                || id == "XG10"
                || a.ability_id.as_deref() == Some("next-bg-jc039-hide")
            {
                a.target_id.clone()
            } else {
                None
            },
            ..Action::new(&a.kind)
        },
        _ => return Ok(()),
    };
    if *a != expected {
        return Err("蓝灰行动只接受本程序所需的来源、能力和目标字段".into());
    }
    Ok(())
}
pub(crate) fn validate_choice_action(p: &Pending, a: &Action) -> RuleResult<()> {
    let own = match &p.resolution {
        ChoiceResolution::Frame { frame, .. } => related(frame),
        ChoiceResolution::Declare { declaration, .. } => {
            CARDS.contains(&declaration.source.card.definition.as_str())
                || related_ability(&declaration.source.card.definition, &declaration.ability)
        }
        _ => false,
    };
    if own
        && *a
            != (Action {
                choice_id: a.choice_id.clone(),
                selected: a.selected.clone(),
                ..Action::new("choose")
            })
    {
        return Err("蓝灰待选只接受原choiceId和selected列表".into());
    }
    Ok(())
}
impl Game {
    pub(crate) fn cq17_combat_bonus(&self, c: &Card, r: usize) -> u32 {
        if c.face_down
            || catalog::card(&c.definition).kind != "character"
            || catalog::card(&c.definition).color != "灰"
        {
            return 0;
        }
        self.regions
            .get(r)
            .into_iter()
            .flat_map(|r| &r.cards)
            .filter(|s| {
                s.definition == "CQ17"
                    && !s.face_down
                    && s.id != c.id
                    && s.controller == c.controller
            })
            .count() as u32
    }
    pub(crate) fn next_blue_gray_retreat(&self, c: &Card) -> bool {
        if c.face_down || catalog::card(&c.definition).kind != "character" {
            return false;
        }
        self.board(&c.id).is_some_and(|(r, _)| {
            self.cq17_combat_bonus(c, r) > 0
                || self.regions[r].cards.iter().any(|s| {
                    s.definition == "BQ059" && !s.face_down && s.controller == c.controller
                })
        })
    }
    pub(crate) fn cq08_count(&self, actor: usize) -> usize {
        self.regions
            .iter()
            .flat_map(|r| &r.cards)
            .filter(|c| {
                !c.face_down
                    && c.controller == actor
                    && catalog::card(&c.definition).kind == "character"
                    && catalog::card(&c.definition).name == "诡异三姐妹"
            })
            .count()
    }
    pub(crate) fn observe_jz23_vampire_reveal(&mut self, revealed: &SourceSnapshot) {
        let c = &revealed.card;
        if c.face_down
            || catalog::card(&c.definition).kind != "character"
            || !self.current_subtypes(c).iter().any(|s| s == "吸血鬼")
        {
            return;
        }
        let Some(r) = revealed.region else {
            return;
        };
        let sources = self.regions[r]
            .cards
            .iter()
            .filter(|s| {
                s.definition == "JZ23"
                    && !s.face_down
                    && s.id != c.id
                    && s.controller == c.controller
            })
            .map(|s| self.source_snapshot(s, Some(r)))
            .collect::<Vec<_>>();
        if sources.is_empty() {
            return;
        }
        let mut frozen = revealed.clone();
        frozen.source_region_instance = Some(self.regions[r].card.id.clone());
        let observation = LocalVampireReveal {
            event_id: self.id(),
            turn: self.turn,
            revealed: frozen,
            vampire: true,
        };
        for source in sources {
            let mut ability = definition("JZ23").unwrap().abilities[0].clone();
            ability.ops[0] = Op::JZ23RevealReward {
                observation: Some(observation.clone()),
            };
            self.effects.push_back(Effect::Declare {
                declaration: Declaration {
                    actor: c.controller,
                    source,
                    ability,
                },
            });
        }
    }
    pub(crate) fn jz23_reward(&mut self, o: &LocalVampireReveal) {
        if let Some(c) = self.board_mut(&o.revealed.card.id).filter(|c| !c.face_down) {
            c.permanent_combat_markers = c.permanent_combat_markers.saturating_add(1);
        }
    }
    pub(crate) fn next_blue_gray_declare_forced(&mut self, d: &Declaration) -> RuleResult<bool> {
        if d.ability.key != "next-bg-bq059-shield" {
            return Ok(false);
        }
        self.validate_next_blue_gray_declaration(d)?;
        if !self.players[d.actor].eliminated {
            let f = self.make_frame(d.actor, d.source.clone(), &d.ability, vec![], vec![], None);
            self.dispatch_frame(
                f,
                d.ability.label.clone(),
                None,
                ResponsePolicy::Respondable,
            );
        }
        Ok(true)
    }
    fn next_blue_gray_options(&self, f: &ResolutionFrame) -> Vec<ChoiceOption> {
        let limit = if f.source.card.definition == "JZ28" {
            4
        } else {
            usize::MAX
        };
        self.players[f.actor]
            .deck
            .iter()
            .take(limit)
            .filter(|c| {
                let d = catalog::card(&c.definition);
                match f.source.card.definition.as_str() {
                    "JZ28" => d.kind == "character" && d.subtypes.iter().any(|s| s == "吸血鬼"),
                    "CQ08" if f.ability_key == "next-bg-cq08-reveal" => d.name == "诡异三姐妹",
                    "CQ08" => {
                        d.kind == "character"
                            && d.unique
                            && d.subtypes.iter().any(|s| s == "吸血鬼")
                    }
                    "CQ17" => {
                        d.kind == "character"
                            && d.cost == 2
                            && d.subtypes.iter().any(|s| s == "士兵")
                    }
                    "BQ059" => f
                        .ability_key
                        .strip_prefix("next-bg-bq059-search-")
                        .and_then(|x| x.parse::<u32>().ok())
                        .is_some_and(|x| {
                            x <= 5
                                && d.kind == "character"
                                && d.cost == x
                                && d.subtypes
                                    .iter()
                                    .any(|s| ["士兵", "工程师", "机械"].contains(&s.as_str()))
                        }),
                    "XG10" => {
                        d.kind == "attachment"
                            && d.name
                                == if f.ability_key == "next-bg-xg10-blood" {
                                    "鲜血"
                                } else {
                                    "初拥"
                                }
                            && f.targets.first().is_some_and(|t| {
                                self.attachment_host_valid(&Attachment {
                                    card: (*c).clone(),
                                    host_id: t.id.clone(),
                                })
                            })
                    }
                    _ => false,
                }
            })
            .map(|c| self.option(c, f.actor, None, None))
            .collect()
    }
    fn next_blue_gray_inspected(&self, f: &ResolutionFrame) -> Vec<String> {
        if f.source.card.definition == "JZ28" {
            self.players[f.actor]
                .deck
                .iter()
                .take(4)
                .map(|c| c.id.clone())
                .collect()
        } else {
            vec![]
        }
    }
    fn next_blue_gray_title(f: &ResolutionFrame) -> String {
        format!(
            "{}：检索并置入原地区或结附原目标，然后洗牌",
            catalog::card(&f.source.card.definition).name
        )
    }
    pub(crate) fn next_blue_gray_search_start(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        self.validate_next_blue_gray_frame(&f)?;
        if self.players[f.actor].eliminated {
            return Ok(());
        }
        let inspected_ids = self.next_blue_gray_inspected(&f);
        let options = self.next_blue_gray_options(&f);
        let n = usize::from(!options.is_empty());
        self.choice(
            f.actor,
            SEARCH,
            Self::next_blue_gray_title(&f),
            options,
            n,
            n,
            None,
            ChoiceResolution::Frame {
                frame: Box::new(f),
                choice: FrameChoice::NextBlueGraySearch { inspected_ids },
            },
        );
        let p = self.pending.as_mut().unwrap();
        p.choice.description = DESCRIPTION.into();
        p.choice.allow_decline = Some(false);
        Ok(())
    }
    pub(crate) fn next_blue_gray_search_complete(
        &mut self,
        f: &ResolutionFrame,
        chooser: usize,
        inspected: &[String],
        selected: &[String],
    ) -> RuleResult<()> {
        self.validate_next_blue_gray_frame(f)?;
        let options = self.next_blue_gray_options(f);
        if chooser != f.actor
            || f.cursor != 1
            || !matches!(f.guard, GuardState::Accepted)
            || inspected != self.next_blue_gray_inspected(f)
            || selected.len() != usize::from(!options.is_empty())
            || selected
                .first()
                .is_some_and(|s| !options.iter().any(|o| &o.id == s))
        {
            return Err("检索续体的选择者、原牌库或选项不符".into());
        }
        if let Some(id) = selected.first() {
            if f.source.card.definition == "XG10" {
                let host = f.targets[0].id.clone();
                let index = self.players[f.actor]
                    .deck
                    .iter()
                    .position(|c| &c.id == id)
                    .ok_or("检索牌已离开牌库")?;
                let c = self.players[f.actor].deck.remove(index);
                let mut c = self.reset_zone_card(c);
                c.controller = f.actor;
                let cid = c.id.clone();
                let definition = c.definition.clone();
                self.note(format!(
                    "{} 展示并结附检索的 {}",
                    self.players[f.actor].name,
                    catalog::card(&definition).name
                ));
                self.attachments.push(Attachment {
                    card: c,
                    host_id: host.clone(),
                });
                if let Some(spec) = rules::definition(&definition)
                    .attachment
                    .as_ref()
                    .filter(|s| s.controls_host)
                {
                    self.add_control(
                        &host,
                        f.actor,
                        ControlLifetime::Attached {
                            source_instance: cid.clone(),
                        },
                        spec.host_subtype_change.clone(),
                    );
                }
                self.enter_triggers(f.actor, &definition, &cid, false);
            } else if let Some(r) = self.original_region(&f.source) {
                let index = self.players[f.actor]
                    .deck
                    .iter()
                    .position(|c| &c.id == id)
                    .ok_or("检索牌已离开牌库")?;
                let c = self.players[f.actor].deck.remove(index);
                let mut c = self.reset_zone_card(c);
                c.controller = f.actor;
                c.face_down =
                    f.source.card.definition == "JZ28" || f.ability_key == "next-bg-cq08-reveal";
                let hidden = c.face_down;
                let cid = c.id.clone();
                let definition = c.definition.clone();
                self.note(format!(
                    "{} 展示检索的 {}",
                    self.players[f.actor].name,
                    catalog::card(&definition).name
                ));
                self.regions[r].cards.push(c);
                if !hidden {
                    self.enter_triggers(f.actor, &definition, &cid, false);
                }
            }
        }
        self.shuffle_player(f.actor);
        Ok(())
    }
    pub(crate) fn next_blue_gray_private_choice(&self, p: &Pending, viewer: usize) -> Choice {
        let mut c = p.choice.clone();
        if let ChoiceResolution::Frame {
            frame,
            choice: FrameChoice::NextBlueGraySearch { inspected_ids },
        } = &p.resolution
        {
            if viewer == frame.actor {
                c.preview_cards = inspected_ids
                    .iter()
                    .filter_map(|id| self.players[viewer].deck.iter().find(|c| &c.id == id))
                    .map(|c| self.card_view(c, viewer, None, None))
                    .collect();
            }
        }
        c
    }
    fn next_blue_gray_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        let c = &s.card;
        if !CARDS.contains(&c.definition.as_str())
            || c.id.is_empty()
            || actor >= self.players.len()
            || c.owner >= self.players.len()
            || c.controller != actor
            || s.observed_death.is_some()
            || s.observed_mobility_grant.is_some()
            || s.observed_exposure.is_some()
            || self
                .in_play(&c.id)
                .is_some_and(|(_, live)| live.definition != c.definition || live.owner != c.owner)
        {
            return Err("蓝灰程序来源或席位不符".into());
        }
        if c.definition == "JC039" {
            if s.region.is_some()
                || s.source_region_instance.is_some()
                || s.attachment_host_instance
                    .as_ref()
                    .is_some_and(|id| id != &format!("society:p{actor}"))
            {
                return Err("徽记来源必须为本方秘社附属".into());
            }
        } else {
            let r = s
                .region
                .filter(|r| *r < self.regions.len())
                .ok_or("蓝灰角色缺少原地区")?;
            let id = s
                .source_region_instance
                .as_deref()
                .filter(|id| !id.is_empty() && *id != c.id)
                .ok_or("蓝灰角色缺少原地区UID")?;
            if s.attachment_host_instance.is_some()
                || self
                    .regions
                    .iter()
                    .enumerate()
                    .any(|(i, region)| region.card.id == id && i != r)
            {
                return Err("蓝灰角色原地区身份不符".into());
            }
        }
        Ok(())
    }
    fn next_blue_gray_observation(
        &self,
        actor: usize,
        s: &SourceSnapshot,
        ops: &[Op],
    ) -> RuleResult<()> {
        for op in ops {
            match op {
                Op::JZ23RevealReward { observation } => {
                    let o = observation.as_ref().ok_or("召集人缺少真实现身观察")?;
                    let v = &o.revealed;
                    let c = &v.card;
                    if s.card.definition != "JZ23"
                        || o.event_id.is_empty()
                        || o.turn != self.turn
                        || !o.vampire
                        || c.id.is_empty()
                        || c.face_down
                        || c.id == s.card.id
                        || c.controller != actor
                        || c.owner >= self.players.len()
                        || catalog::card(&c.definition).kind != "character"
                        || v.region != s.region
                        || v.source_region_instance != s.source_region_instance
                        || v.source_region_instance.is_none()
                        || v.play_source.is_some()
                        || v.observed_death.is_some()
                        || v.observed_mobility_grant.is_some()
                        || v.observed_exposure.is_some()
                    {
                        return Err("召集人必须观察另一个本方吸血鬼在同一原地区现身".into());
                    }
                }
                Op::JZ28DiscardSearch { observation } => {
                    let o = observation.as_ref().ok_or("霍瓦特缺少真实弃牌观察")?;
                    if s.card.definition != "JZ28"
                        || o.event_id.is_empty()
                        || o.turn != self.turn
                        || o.holder >= self.players.len()
                        || self.team(o.holder) == self.team(actor)
                        || o.cards.is_empty()
                        || o.cards.iter().any(|c| {
                            c.id.is_empty() || c.owner >= self.players.len() || c.face_down
                        })
                        || o.cards.iter().map(|c| &c.id).collect::<BTreeSet<_>>().len()
                            != o.cards.len()
                    {
                        return Err("霍瓦特必须观察敌方一次实际弃牌操作".into());
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
    fn next_blue_gray_expected(&self, f: &ResolutionFrame) -> RuleResult<AbilitySpec> {
        if let Some(mut a) = definition(&f.source.card.definition)
            .and_then(|d| d.abilities.into_iter().find(|a| a.key == f.ability_key))
        {
            let ops = f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>();
            self.next_blue_gray_observation(f.actor, &f.source, &ops)?;
            for (expected, actual) in a.ops.iter_mut().zip(&ops) {
                if matches!(
                    expected,
                    Op::JZ23RevealReward { .. } | Op::JZ28DiscardSearch { .. }
                ) {
                    *expected = actual.clone();
                }
            }
            validate_ability(&f.source.card.definition, &a)?;
            return Ok(a);
        }
        let region = f
            .source
            .source_region_instance
            .as_deref()
            .ok_or("授予奖励缺少原地区")?;
        match f.ability_key.as_str() {
            "renown" => Ok(crate::renown::renown_ability(region)),
            "jc089-combat-glory" => Ok(rules::jc089_combat_glory_ability(region)),
            _ => Err("蓝灰来源未知程序".into()),
        }
    }
    pub(crate) fn validate_next_blue_gray_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if !related(f) || self.validated_cross_card_frame(f)? {
            return Ok(());
        }
        if crate::jc035::preparation_frame(f) {
            return self.validate_jc035_frame(f);
        }
        self.next_blue_gray_source(f.actor, &f.source)?;
        if f.frame_id.is_empty() || f.cursor > f.steps.len() {
            return Err("蓝灰帧身份或游标不符".into());
        }
        let printed = matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if ids.len() <= catalog::card(&f.source.card.definition).cost as usize && ids.iter().all(|id| !id.is_empty()) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len());
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            if catalog::card(&f.source.card.definition).kind != "character"
                || !f.steps.is_empty()
                || !f.targets.is_empty()
                || f.chosen_region != f.source.region
                || !printed
                || (f.ability_key == "deploy"
                    && (f.source.card.face_down || !f.source.valid_character_play()))
                || (f.ability_key == "reveal"
                    && (!f.source.card.face_down || f.source.play_source.is_some()))
            {
                return Err("蓝灰角色入场帧不符".into());
            }
        } else {
            let a = self.next_blue_gray_expected(f)?;
            let steps = a
                .ops
                .iter()
                .cloned()
                .map(|op| Step {
                    context: f.actor,
                    op,
                })
                .collect::<Vec<_>>();
            if !same(&f.steps, &steps)
                || f.source.card.face_down
                || f.source.play_source.is_some()
                || f.chosen_region.is_some()
                || f.targets.len() != a.targets.len()
                || f.targets.iter().zip(&a.targets).any(|(t, s)| {
                    !same(&t.spec, s)
                        || t.id.is_empty()
                        || t.public.instance_id != t.id
                        || t.region_instance.is_some()
                        || !t.public.valid
                        || t.public.status != "valid"
                        || t.public.invalid_reason.is_some()
                })
            {
                return Err("蓝灰固定程序或目标绑定被改写".into());
            }
            if f.ability_key == "attach" {
                if !printed {
                    return Err("徽记结附费用不符".into());
                }
            } else {
                self.validate_batch20_payment(f, &a.costs)?;
            }
        }
        Ok(())
    }
    pub(crate) fn validate_next_blue_gray_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if (!CARDS.contains(&d.source.card.definition.as_str())
            && !related_ability(&d.source.card.definition, &d.ability))
            || self.validated_cross_card_declaration(d)?
        {
            return Ok(());
        }
        if d.ability.event == Some(Event::Prepare) {
            return crate::jc035::validate_ability(&d.source.card.definition, &d.ability);
        }
        self.next_blue_gray_source(d.actor, &d.source)?;
        if crate::jz22::runtime_ability_is_admitted(&d.ability) {
            let region = d
                .source
                .source_region_instance
                .as_deref()
                .ok_or("蓝灰奖励缺少原地区UID")?;
            if catalog::card(&d.source.card.definition).kind != "character"
                || d.source.card.face_down
                || d.source.play_source.is_some()
                || d.source.region.is_none()
                || !matches!(d.ability.ops.as_slice(), [Op::PlaceInfluence { region_instance, amount: 1 }] if region_instance == region)
            {
                return Err("蓝灰授予奖励必须绑定原公开角色与原地区".into());
            }
            return Ok(());
        }
        validate_ability(&d.source.card.definition, &d.ability)?;
        if d.ability.event.is_none() || d.source.card.face_down || d.source.play_source.is_some() {
            return Err("蓝灰声明不是公开来源的固定触发".into());
        }
        self.next_blue_gray_observation(d.actor, &d.source, &d.ability.ops)
    }
    pub(crate) fn validate_next_blue_gray_search(
        &self,
        p: &Pending,
        f: &ResolutionFrame,
        ids: &[String],
    ) -> RuleResult<()> {
        self.validate_next_blue_gray_frame(f)?;
        let search_op = matches!(
            f.steps.first().map(|s| &s.op),
            Some(Op::NextBlueGraySearch | Op::JZ28DiscardSearch { .. } | Op::CQ08Prepare { .. })
        );
        let options = self.next_blue_gray_options(f);
        let n = usize::from(!options.is_empty());
        let expected = Choice {
            id: p.choice.id.clone(),
            kind: SEARCH.into(),
            title: Self::next_blue_gray_title(f),
            description: DESCRIPTION.into(),
            player_id: player_id(f.actor),
            options,
            preview_cards: vec![],
            min: Some(n),
            max: Some(n),
            amount: None,
            allow_decline: Some(false),
        };
        if !search_op
            || f.steps.len() != 1
            || f.cursor != 1
            || !matches!(f.guard, GuardState::Accepted)
            || p.seat != f.actor
            || p.choice.id.is_empty()
            || ids != self.next_blue_gray_inspected(f)
            || !same(&p.choice, &expected)
        {
            return Err("蓝灰检索待选不是原帧的完整私密续体".into());
        }
        Ok(())
    }
    pub(crate) fn validate_next_blue_gray_state(&self) -> RuleResult<()> {
        // This is the sole admitted blue printed-colour one-shot discount;
        // leave other colours and vampire/subtype reductions to their owners.
        if self.modifiers.iter().any(|m| {
            matches!(&m.filter, CardFilter::PrintedColor(color) if color == "蓝")
                && (!m.paid_reveal
                    || m.amount != 1
                    || m.uses > 1
                    || m.actor >= self.players.len()
                    || m.expires_turn == 0
                    || m.expires_turn > self.turn)
        }) {
            return Err("徽记减费必须保留双通道、减1及一次使用收据".into());
        }
        // Markers are physical, never a turn modifier or an off-board attribute.
        // XQ20 also permits valid attachments, including secret plans, to carry
        // them. Fresh identity creation still resets all physical markers.
        for p in &self.players {
            if p.hand
                .iter()
                .chain(&p.deck)
                .chain(&p.graveyard)
                .chain(&p.assets)
                .chain(&p.score_cards)
                .any(|c| c.permanent_combat_markers != 0)
            {
                return Err("永久图标标志不能继承到新区域实体".into());
            }
        }
        if self
            .world
            .iter()
            .chain(self.attachments.iter().filter(|a| !self.marker_move_carrier(&a.card.id)).map(|a| &a.card))
            .chain(self.sealed_cards.iter().map(|s| &s.card))
            .chain(self.stack.iter().filter_map(|s| s.card.as_ref()))
            .chain(
                self.players
                    .iter()
                    .filter_map(|p| p.society_zone.card.as_ref()),
            )
            .any(|c| c.permanent_combat_markers != 0)
        {
            return Err("永久图标标志不能存在于无效承载区域".into());
        }
        if self.regions.iter().flat_map(|r| &r.cards).chain(self.attachments.iter().map(|a| &a.card)).any(|c| {
            c.permanent_combat_markers > 0
                && !self.marker_move_carrier(&c.id)
        }) {
            return Err("永久图标标志只能存在于有效角色或附属".into());
        }
        let mut seen = BTreeSet::new();
        let mut receipt = |source: &SourceSnapshot, ops: &[Op]| -> RuleResult<()> {
            for op in ops {
                let id = match op {
                    Op::JZ23RevealReward {
                        observation: Some(o),
                    } => Some(&o.event_id),
                    Op::JZ28DiscardSearch {
                        observation: Some(o),
                    } => Some(&o.event_id),
                    _ => None,
                };
                if id.is_some_and(|id| !seen.insert((source.card.id.clone(), id.clone()))) {
                    return Err("蓝灰同一来源不能重放同一观察".into());
                }
            }
            Ok(())
        };
        for item in &self.stack {
            if let Some(f) = &item.frame {
                self.validate_next_blue_gray_frame(f)?;
                if related(f) {
                    receipt(
                        &f.source,
                        &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>(),
                    )?;
                    let transaction =
                        matches!(f.ability_key.as_str(), "deploy" | "reveal" | "attach");
                    if item.id != f.frame_id
                        || item.controller != f.actor
                        || f.cursor != 0
                        || !matches!(f.guard, GuardState::Unchecked)
                        || item.target != f.targets.first().map(|t| t.id.clone())
                        || transaction != item.card.is_some()
                        || item.deploy_region
                            != if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
                                f.chosen_region
                            } else {
                                None
                            }
                        || item.reveal != (f.ability_key == "reveal")
                        || item.card.as_ref().is_some_and(|c| {
                            c.definition != f.source.card.definition
                                || c.owner != f.source.card.owner
                                || c.controller != f.actor
                                || c.face_down
                                || c.permanent_combat_markers != 0
                        })
                    {
                        return Err("蓝灰响应堆栈外壳不符".into());
                    }
                }
            } else if item
                .card
                .as_ref()
                .is_some_and(|c| CARDS.contains(&c.definition.as_str()))
            {
                return Err("蓝灰事务缺少帧".into());
            }
        }
        for e in &self.effects {
            match e {
                Effect::Declare { declaration: d } => {
                    self.validate_next_blue_gray_declaration(d)?;
                    receipt(&d.source, &d.ability.ops)?;
                }
                Effect::Frame { frame } if related(frame) => {
                    return Err("蓝灰程序不能绕过响应堆栈保存".into())
                }
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Rumor { batch } => {
                    let f = batch.cause.as_ref().ok_or("传闻缺少蓝色观察原帧")?;
                    self.validate_next_blue_gray_frame(f)?;
                    receipt(
                        &f.source,
                        &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>(),
                    )?;
                }
                ChoiceResolution::Frame {
                    frame: f,
                    choice: FrameChoice::NextBlueGraySearch { inspected_ids },
                } => {
                    self.validate_next_blue_gray_search(p, f, inspected_ids)?;
                    receipt(
                        &f.source,
                        &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>(),
                    )?;
                }
                ChoiceResolution::Frame {
                    frame: f,
                    choice: FrameChoice::Discard { seat, redraw },
                } if related(f) => {
                    self.validate_next_blue_gray_frame(f)?;
                    receipt(
                        &f.source,
                        &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>(),
                    )?;
                    let options = self
                        .players
                        .get(*seat)
                        .ok_or("弃牌玩家无效")?
                        .hand
                        .iter()
                        .map(|c| self.option(c, *seat, None, None))
                        .collect::<Vec<_>>();
                    if f.ability_key != "next-bg-jz23-reveal"
                        || f.cursor != 2
                        || !matches!(f.guard, GuardState::Accepted)
                        || f.targets.first().is_none_or(|t| t.id != player_id(*seat))
                        || *redraw
                        || p.seat != *seat
                        || p.choice.kind != "discard"
                        || p.choice.min != Some(1)
                        || p.choice.max != Some(1)
                        || p.choice.allow_decline != Some(false)
                        || options.is_empty()
                        || !same(&p.choice.options, &options)
                    {
                        return Err("召集人弃牌续体不符".into());
                    }
                }
                ChoiceResolution::Declare {
                    declaration: d,
                    stage,
                } if CARDS.contains(&d.source.card.definition.as_str())
                    || related_ability(&d.source.card.definition, &d.ability) =>
                {
                    self.validate_next_blue_gray_declaration(d)?;
                    receipt(&d.source, &d.ability.ops)?;
                    let targeted = !d.ability.targets.is_empty();
                    let options = if targeted {
                        self.binding_options(d.actor, &d.source, &d.ability.targets[0])
                    } else {
                        vec![ChoiceOption {
                            id: "accept".into(),
                            label: "发动触发能力".into(),
                            card: None,
                        }]
                    };
                    if d.ability.key == "next-bg-bq059-shield"
                        || p.seat != d.actor
                        || p.choice.player_id != player_id(d.actor)
                        || p.choice.id.is_empty()
                        || p.choice.kind != "trigger"
                        || p.choice.min != Some(0)
                        || p.choice.max != Some(1)
                        || p.choice.allow_decline != Some(true)
                        || p.choice.amount.is_some()
                        || !p.choice.preview_cards.is_empty()
                        || p.choice.title
                            != format!(
                                "{}：{}",
                                catalog::card(&d.source.card.definition).name,
                                d.ability.label
                            )
                        || targeted != matches!(stage, DeclareChoice::Target)
                        || !targeted && !matches!(stage, DeclareChoice::Accept)
                        || !same(&p.choice.options, &options)
                    {
                        return Err("蓝灰触发待选不符".into());
                    }
                }
                ChoiceResolution::Frame { frame, .. } if related(frame) => {
                    return Err("蓝灰程序不支持该待选阶段".into())
                }
                _ => {}
            }
        }
        Ok(())
    }
}
