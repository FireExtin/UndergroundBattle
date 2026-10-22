//! Third twenty-card batch under development. Entries remain disabled for deck
//! construction until the complete batch is ready for its publication checks.
use crate::{
    catalog,
    engine::RuleResult,
    model::*,
    rules::{self, *},
};
use std::collections::BTreeSet;

pub(crate) const IMPLEMENTING: &[&str] = &["WM030", "BQ042", "WM033", "XG26", "WM035", "JZ25", "XG28", "BQ089", "JC062", "XQ22", "JC064", "BQ087", "BQ079", "WM041", "XQ26", "BQ037", "BQ029", "BQ026", "JZ26", "CQ07"];

fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}

pub(crate) fn definition(id: &str) -> Option<Definition> {
    match id {
        "CQ07" => Some(crate::batch3_phoebe::definition()),
        "JZ26" => Some(crate::batch3_trap::definition()),
        "BQ026" => Some(crate::batch3_bats::definition()),
        "BQ029" => Some(crate::batch3_blood::bq029_definition()),
        "BQ037" => Some(crate::batch3_blood::bq037_definition()),
        "XQ26" => Some(crate::batch3_warrant::definition()),
        "WM041" => Some(crate::batch3_alpha::definition()),
        "BQ079" => Some(crate::batch3_ghoul::definition()),
        "JC062" => Some(crate::batch3_inspect::definition()),
        "XQ22" => Some(crate::batch3_politician::definition()),
        "JC064" => Some(crate::batch3_law::definition()),
        "BQ087" => {
            let mut t=rules::target(Zone::Graveyard,EntityKind::Character,Relation::OwnedByActor,Range::Anywhere);
            t.predicate=Some(TargetPredicate::Batch3OwnGraveCharacter);
            Some(rules::with_abilities(vec![rules::ability("batch3-bq087-prepare", "准备强制触发：本方墓地目标角色进入随机地区", Timing::Fast,vec![],vec![t],vec![Op::Batch3Prepare{event_id:None}],Some(Event::Prepare))]))
        },
        "JZ25" => Some(rules::with_abilities(vec![rules::ability(
            "batch3-jz25-seal", "进场：目标玩家随机封印一张手牌在本牌上", Timing::Fast,
            vec![], vec![rules::target(Zone::Player, EntityKind::Any, Relation::Any, Range::Anywhere)],
            vec![Op::JZ25SealRandomHand], Some(Event::Enter),
        )])),
        "XG28" => {
            let mut d = rules::attachment_definition(rules::target(Zone::Region, EntityKind::Any, Relation::Any, Range::Anywhere));
            d.abilities[0].key = "batch3-xg28-attach".into();
            d.abilities.push(rules::ability("batch3-xg28-prepare", "准备步骤：牌库顶一张置于墓地", Timing::Fast,
                vec![], vec![], vec![Op::Batch3Prepare { event_id: None }], Some(Event::Prepare)));
            let mut t = rules::target(Zone::Graveyard, EntityKind::Character, Relation::OwnedByActor, Range::Anywhere);
            t.predicate = Some(TargetPredicate::Batch3OwnGraveCharacter);
            let mut a = rules::ability("batch3-xg28-return", "行动2、横置：墓地角色回手；本方隐秘则重置本牌", Timing::Standard,
                vec![Cost::Assets(2), Cost::ExhaustSource], vec![t], vec![Op::XG28GraveReturn], None);
            a.activation_only = true;
            d.abilities.push(a);
            Some(d)
        }
        "BQ089" => {
            let mut t = rules::target(Zone::Graveyard, EntityKind::Character, Relation::OwnedByActor, Range::Anywhere);
            t.predicate = Some(TargetPredicate::Batch3OwnGraveCharacter);
            let mut a = rules::ability("batch3-bq089-reanimate", "行动：墓地中印刷费用至多3的角色进入目标地区；隐秘时至多5", Timing::Standard,
                vec![], vec![t, rules::target(Zone::Region, EntityKind::Any, Relation::Any, Range::Anywhere)],
                vec![Op::BQ089GraveEntry], None);
            a.play_only = true;
            Some(rules::with_abilities(vec![a]))
        }
        "WM035" => {
            let mut d = rules::with_abilities(vec![rules::ability(
                "batch3-wm035-host-death",
                "本方宿主角色死亡：抓一张牌",
                Timing::Fast,
                vec![],
                vec![],
                vec![Op::WM035HostDeathDraw { observation: None }],
                Some(Event::WM035HostDeath),
            )]);
            d.modifiers.push(StaticModifier::WM035HumansGainHost);
            Some(d)
        }
        "XG26" => {
            let mut a = rules::ability(
                "batch3-xg26-barrier",
                "快速行动1、横置：目标本方附属获得屏障直到回合结束",
                Timing::Fast,
                vec![Cost::Assets(1), Cost::ExhaustSource],
                vec![rules::target(
                    Zone::Board,
                    EntityKind::Attachment,
                    Relation::ControlledByActor,
                    Range::Anywhere,
                )],
                vec![Op::XG26AttachmentBarrier],
                None,
            );
            a.activation_only = true;
            let mut d = rules::with_abilities(vec![a]);
            d.modifiers
                .push(StaticModifier::XG26InfluenceAndConditionalGuard);
            Some(d)
        }
        "WM033" => {
            let mut action = rules::ability(
                "batch3-wm033-return",
                "行动：移除自身1个时间标志，将本方墓地一张法术或灾难牌移回手中，每回合一次",
                Timing::Standard,
                vec![Cost::Batch3RemoveTime],
                vec![],
                vec![Op::WM033ReturnSpellOrDisaster],
                None,
            );
            action.activation_only = true;
            action.per_turn_limit = Some(1);
            Some(rules::with_abilities(vec![action]))
        }
        "BQ042" => {
            let mut target = rules::target(
                Zone::Board,
                EntityKind::Attachment,
                Relation::Any,
                Range::Anywhere,
            );
            target.predicate = Some(TargetPredicate::BQ042SocietyOrOriginalRegion);
            let mut action = rules::ability(
                "batch3-bq042-destroy",
                "行动：移除自身1个时间标志，消灭结附于本地区或秘社的目标附属，每回合一次",
                Timing::Standard,
                vec![Cost::Batch3RemoveTime],
                vec![target],
                vec![Op::BQ042DestroyAttachment],
                None,
            );
            action.activation_only = true;
            action.per_turn_limit = Some(1);
            Some(rules::with_abilities(vec![action]))
        }
        "WM030" => Some(Definition {
            modifiers: vec![StaticModifier::WM030CultistIcons],
            ..Definition::default()
        }),
        _ => None,
    }
}

pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if let Some(expected) = definition(id) {
        if !same(d, &expected) {
            return Err("第三批须保留完整固定定义".into());
        }
    } else if d.modifiers.iter().any(|m| {
        matches!(
            m,
            StaticModifier::WM041RenownControl | StaticModifier::JC064MartialLaw | StaticModifier::XQ22UnexhaustedSpellBan | StaticModifier::WM030CultistIcons
                | StaticModifier::XG26InfluenceAndConditionalGuard
                | StaticModifier::WM035HumansGainHost
        )
    }) || d.abilities.iter().any(related_ability)
    {
        return Err("第三批有限程序不能移植".into());
    }
    Ok(())
}

pub(crate) fn related_ability(a: &AbilitySpec) -> bool {
    matches!(a.event, Some(Event::CQ07SpellTargeted | Event::WM035HostDeath | Event::BQ079CorpseGrave | Event::WM041PublicCharacterPlayed | Event::XQ26NamedCharacterEntered | Event::BQ037OtherVampireEntered | Event::BQ029QualifiedEntry))
        || a.key.starts_with("batch3-")
        || uses(&a.ops)
        || a.modes.iter().any(|m| uses(&m.ops))
        || a.costs.iter().any(|c| matches!(c, Cost::Batch3RemoveTime))
        || a.targets
            .iter()
            .any(|t| matches!(t.predicate, Some(TargetPredicate::BQ042SocietyOrOriginalRegion | TargetPredicate::Batch3OwnGraveCharacter)))
}
fn uses(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::CQ07EnterMoon | Op::CQ07SpellTargeted { .. } | Op::JZ26Trap { .. } | Op::BQ026Attach { .. } | Op::JC062Inspect | Op::XQ22Orientation | Op::JZ25SealRandomHand | Op::XG28GraveReturn | Op::BQ089GraveEntry | Op::Batch3Prepare { .. }
        | Op::BQ029SelfFeed { .. }
        | Op::BQ037FeedPair { .. } | Op::BQ037RevealVampire
        | Op::XQ26Name | Op::XQ26Wanted { .. }
        | Op::WM041PublicPlay { .. }
        | Op::BQ079CorpseIcons { .. }
        | Op::WM035HostDeathDraw { .. }
        | Op::BQ042DestroyAttachment
        | Op::WM033ReturnSpellOrDisaster
        | Op::XG26AttachmentBarrier => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => uses(body),
        Op::IfTargetExhausted {
            exhausted, ready, ..
        } => uses(std::slice::from_ref(exhausted)) || uses(std::slice::from_ref(ready)),
        _ => false,
    })
}
fn related(f: &ResolutionFrame) -> bool {
    IMPLEMENTING.contains(&f.source.card.definition.as_str())
        || f.ability_key.starts_with("batch3-")
        || f.steps.iter().any(|s| uses(std::slice::from_ref(&s.op)))
        || f.already_paid
            .iter()
            .any(|p| matches!(p, PaidCost::Batch3RemovedTime(_)))
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if a.event == Some(Event::Prepare) && IMPLEMENTING.contains(&id) {
        return crate::jc035::validate_ability(id, a);
    }
    if (id=="BQ029" && a.key=="raid-1" || related_ability(a)) && canonical_ability(id, a).is_none_or(|expected| !same(&expected, a)) {
        return Err("第三批能力的来源、程序、费用或目标被修改".into());
    }
    Ok(())
}
fn canonical_ability(id: &str, a: &AbilitySpec) -> Option<AbilitySpec> {
    let mut expected = definition(id)?
        .abilities
        .into_iter()
        .find(|printed| printed.key == a.key)?;
    if a.key == crate::batch3_phoebe::TARGETED {
        if let [Op::CQ07SpellTargeted { observation }] = a.ops.as_slice() { expected.ops = vec![Op::CQ07SpellTargeted { observation: observation.clone() }]; }
    }
    if a.key == "batch3-jz26-trap" {
        if let [Op::JZ26Trap { observation }] = a.ops.as_slice() { expected.ops=vec![Op::JZ26Trap { observation:observation.clone() }]; }
    }
    if a.key == "batch3-bq026-attach" {
        if let [Op::BQ026Attach { observation }] = a.ops.as_slice() { expected.ops = vec![Op::BQ026Attach { observation: observation.clone() }]; }
    }
    if a.key == "batch3-bq029-feed" {
        if let [Op::BQ029SelfFeed { observation }] = a.ops.as_slice() { expected.ops = vec![Op::BQ029SelfFeed { observation: observation.clone() }]; }
    }
    if a.key == "batch3-bq037-feed" {
        if let [Op::BQ037FeedPair { observation }] = a.ops.as_slice() { expected.ops = vec![Op::BQ037FeedPair { observation: observation.clone() }]; }
    }
    if a.key == "batch3-wm041-public-play" {
        if let [Op::WM041PublicPlay { observation }] = a.ops.as_slice() { expected.ops = vec![Op::WM041PublicPlay { observation: observation.clone() }]; }
    }
    if a.key == "batch3-bq079-corpse" {
        if let [Op::BQ079CorpseIcons { observation }] = a.ops.as_slice() { expected.ops = vec![Op::BQ079CorpseIcons { observation: observation.clone() }]; }
    }
    if a.key == "batch3-wm035-host-death" {
        if let [Op::WM035HostDeathDraw { observation }] = a.ops.as_slice() {
            expected.ops = vec![Op::WM035HostDeathDraw {
                observation: observation.clone(),
            }];
        }
    }
    Some(expected)
}
pub(crate) fn validate_action(id: &str, a: &Action) -> RuleResult<()> {
    if !IMPLEMENTING.contains(&id) {
        return Ok(());
    }
    if matches!(id, "XQ26" | "XG28" | "BQ089" | "JC064") { return crate::batch3_grave::validate_action(id, a); }
    if id == "JC062" { return crate::batch3_inspect::validate_action(a); }
    if id == "XQ22" && a.kind == "activate" { return crate::batch3_politician::validate_action(a); }
    if id=="BQ029" && a.option.as_deref().is_some_and(|s|s!="pay-extra-1" || a.kind!="deploy") { return Err("赤鳞近卫只有正面打出可声明额外支付1".into()); }
    let expected = match a.kind.as_str() {
        "asset" if id=="BQ029" => Action{card_id:a.card_id.clone(),..Action::new("asset")},
        "deploy" | "conceal" if id=="BQ029" => Action{card_id:a.card_id.clone(),region:a.region,option:a.option.clone(),..Action::new(&a.kind)},
        "deploy" | "conceal" => Action {
            card_id: a.card_id.clone(),
            region: a.region,
            ..Action::new(&a.kind)
        },
        "reveal" => Action {
            card_id: a.card_id.clone(),
            ..Action::new("reveal")
        },
        "activate" if id=="BQ037" => Action {card_id:a.card_id.clone(),ability_id:a.ability_id.clone(),target_id:a.target_id.clone(),..Action::new("activate")},
        "activate" if matches!(id, "BQ042" | "WM033" | "XG26") => Action {
            card_id: a.card_id.clone(),
            ability_id: a.ability_id.clone(),
            target_id: if id != "WM033" {
                a.target_id.clone()
            } else {
                None
            },
            cost_selected: if id == "XG26" {
                None
            } else {
                a.cost_selected.clone()
            },
            ..Action::new("activate")
        },
        _ => return Err("第三批角色操作种类不符".into()),
    };
    if *a != expected {
        return Err("第三批声明包含非准入载荷".into());
    }
    Ok(())
}

impl Game {
    pub(crate) fn current_spirit(&self, c: &Card) -> bool {
        !c.face_down
            && (rules::definition(&c.definition).traits.spirit
                || (matches!(c.definition.as_str(), "BQ042" | "XG14" | "XG11")
                    && self
                        .players
                        .get(c.controller)
                        .is_some_and(|p| p.society_zone.secrecy > 0)))
    }
    pub(crate) fn batch3_entry_keywords(&mut self, definition: &str, id: &str) {
        if matches!(definition, "BQ042" | "WM033") {
            if let Some(c) = self.board_mut(id).filter(|c| !c.face_down) {
                c.time_markers = c.time_markers.saturating_add(1);
            }
        }
    }
    pub(crate) fn batch3_pay_time(
        &mut self,
        source: &SourceSnapshot,
        action: &Action,
    ) -> RuleResult<PaidCost> {
        match self.wm034_pay_time(source, action)? {
            PaidCost::SourceRemovedTime(payment) => Ok(PaidCost::Batch3RemovedTime(payment)),
            _ => Err("时间支付未生成真实载体收据".into()),
        }
    }
    fn batch3_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        let c = &s.card;
        if !IMPLEMENTING.contains(&c.definition.as_str())
            || c.id.is_empty()
            || actor >= self.players.len()
            || c.controller != actor
            || c.owner >= self.players.len()
            || s.region.is_none_or(|r| r >= self.regions.len())
            || s.source_region_instance
                .as_ref()
                .is_none_or(|id| id.is_empty() || id == &c.id)
            || (s.attachment_host_instance.is_some() && c.definition != "BQ026")
            || s.observed_death.is_some()
            || s.observed_mobility_grant.is_some()
            || s.observed_exposure.is_some()
            || self.batch3_physical_cards().any(|live|live.id==c.id && self.board(&c.id).is_none())
            || s.region
                .zip(s.source_region_instance.as_ref())
                .is_some_and(|(r, id)| {
                    self.regions
                        .iter()
                        .enumerate()
                        .any(|(other, region)| region.card.id == *id && r != other)
                })
        {
            return Err("第三批冻结来源、操控者或原地区身份不符".into());
        }
        self.mobility_immutable_identity(c)
    }
    pub(crate) fn validate_batch3_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if self.validated_cross_card_declaration(d)? {
            return Ok(());
        }
        if d.ability.event == Some(Event::Prepare) && IMPLEMENTING.contains(&d.source.card.definition.as_str()) {
            return crate::jc035::validate_ability(&d.source.card.definition, &d.ability);
        }
        if d.source.card.definition=="JZ26" {return self.validate_jz26_declaration(d);}
        if d.source.card.definition == "XQ26" { return self.validate_xq26_declaration(d); }
        if d.source.card.definition == "JZ25" {
            self.batch3_source(d.actor, &d.source)?;
            self.validate_jz25_origin(&d.source)?;
            let expected = definition("JZ25").unwrap().abilities.remove(0);
            if d.source.card.face_down || d.source.play_source.is_some() || !same(&expected, &d.ability) {
                return Err("卡迪纳猎杀者触发来源或程序不符".into());
            }
            return Ok(());
        }
        if IMPLEMENTING.contains(&d.source.card.definition.as_str()) || related_ability(&d.ability)
        {
            self.batch3_source(d.actor, &d.source)?;
            let expected = canonical_ability(&d.source.card.definition, &d.ability)
                .ok_or("第三批触发声明没有固定程序")?;
            if !matches!((d.source.card.definition.as_str(),d.ability.event),("CQ07",Some(Event::Enter | Event::CQ07SpellTargeted)) | ("BQ026",Some(Event::EnterRegion)) | ("WM035",Some(Event::WM035HostDeath)) | ("BQ079",Some(Event::BQ079CorpseGrave)) | ("WM041",Some(Event::WM041PublicCharacterPlayed)) | ("BQ037",Some(Event::BQ037OtherVampireEntered)) | ("BQ029",Some(Event::BQ029QualifiedEntry | Event::Reveal)))
                || d.source.card.face_down
                || d.source.play_source.is_some()
                || !same(&d.ability, &expected)
            {
                return Err("第三批触发声明的来源、事件或程序不符".into());
            }
            if d.source.card.definition=="CQ07" { self.cq07_observation(d.actor,&d.source,&d.ability.ops)?; } else if d.source.card.definition=="BQ026" { self.bq026_observation(&d.source,&d.ability.ops)?; } else if d.source.card.definition=="BQ029" { if d.ability.event!=Some(Event::Reveal) { self.bq029_observation(d.actor,&d.source,&d.ability.ops)?; } } else if d.source.card.definition == "BQ037" { self.bq037_observation(d.actor,&d.source,&d.ability.ops)?; } else if d.source.card.definition == "WM041" { self.wm041_observation(d.actor,&d.source,&d.ability.ops)?; } else if d.source.card.definition == "BQ079" { self.bq079_observation(d.actor,&d.source,&d.ability.ops)?; } else {
                self.wm035_observation_key(d.actor, &d.source, &d.ability.ops)?.ok_or("宿主触发缺少观察")?;
            }
        }
        Ok(())
    }
    pub(crate) fn validate_batch3_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if self.validated_cross_card_frame(f)? || !related(f) {
            return Ok(());
        }
        if f.source.card.definition=="JZ26" && !matches!(f.ability_key.as_str(),"deploy"|"reveal") {return self.validate_jz26_frame(f);}
        if crate::jc035::preparation_frame(f) { return self.validate_jc035_frame(f); }
        if f.source.card.definition == "JC062" { return self.validate_jc062_frame(f); }
        if f.source.card.definition == "XQ22" && !matches!(f.ability_key.as_str(), "deploy" | "reveal") { return self.validate_xq22_frame(f); }
        if matches!(f.source.card.definition.as_str(), "XQ26" | "XG28" | "BQ089" | "JC064")
            || (f.source.card.definition == "JZ25" && !matches!(f.ability_key.as_str(), "deploy" | "reveal")) {
            return self.validate_batch3_grave_frame(f);
        }
        self.batch3_source(f.actor, &f.source)?;
        if f.frame_id.is_empty()
            || f.cursor > f.steps.len()
            || (f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked))
            || (f.cursor > 0 && !matches!(f.guard, GuardState::Accepted))
        {
            return Err("第三批帧身份、守卫或游标不符".into());
        }
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            let paid = if f.source.card.definition=="BQ029" { self.bq029_printed_payment(f) } else { matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if ids.len() <= catalog::card(&f.source.card.definition).cost as usize
                && ids.iter().all(|id| !id.is_empty()) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len()) };
            if f.source.attachment_host_instance.is_some() || !f.steps.is_empty()
                || !f.targets.is_empty()
                || !paid
                || f.chosen_region != f.source.region
                || (f.ability_key == "deploy"
                    && (f.source.card.face_down || !f.source.valid_character_play()))
                || (f.ability_key == "reveal"
                    && (!f.source.card.face_down || f.source.play_source.is_some()))
            {
                return Err("第三批出场帧或支付不符".into());
            }
            return Ok(());
        }
        let mut a = definition(&f.source.card.definition)
            .and_then(|d| d.abilities.into_iter().find(|a| a.key == f.ability_key))
            .ok_or("非准入第三批执行帧")?;
        if f.source.card.definition == "CQ07" {
            if a.key == crate::batch3_phoebe::TARGETED {
                if let [Step { op: Op::CQ07SpellTargeted { observation }, .. }] = f.steps.as_slice() { a.ops = vec![Op::CQ07SpellTargeted { observation: observation.clone() }]; }
            }
            self.cq07_observation(f.actor,&f.source,&a.ops)?;
        }
        if a.key == "batch3-bq026-attach" {
            if let [Step { op: Op::BQ026Attach { observation }, .. }] = f.steps.as_slice() { a.ops = vec![Op::BQ026Attach { observation: observation.clone() }]; }
            self.bq026_observation(&f.source,&a.ops)?;
        }
        if a.key == "batch3-bq029-feed" {
            if let [Step { op: Op::BQ029SelfFeed { observation }, .. }] = f.steps.as_slice() { a.ops = vec![Op::BQ029SelfFeed { observation: observation.clone() }]; }
            self.bq029_observation(f.actor,&f.source,&a.ops)?;
        }
        if a.key == "batch3-bq037-feed" {
            if let [Step { op: Op::BQ037FeedPair { observation }, .. }] = f.steps.as_slice() { a.ops = vec![Op::BQ037FeedPair { observation: observation.clone() }]; }
            self.bq037_observation(f.actor,&f.source,&a.ops)?;
        }
        if a.key == "batch3-wm041-public-play" {
            if let [Step { op: Op::WM041PublicPlay { observation }, .. }] = f.steps.as_slice() { a.ops = vec![Op::WM041PublicPlay { observation: observation.clone() }]; }
            self.wm041_observation(f.actor,&f.source,&a.ops)?;
        }
        if a.key == "batch3-bq079-corpse" {
            if let [Step { op: Op::BQ079CorpseIcons { observation }, .. }] = f.steps.as_slice() { a.ops = vec![Op::BQ079CorpseIcons { observation: observation.clone() }]; }
            self.bq079_observation(f.actor,&f.source,&a.ops)?;
        }
        if a.key == "batch3-wm035-host-death" {
            if let [Step {
                op: Op::WM035HostDeathDraw { observation },
                ..
            }] = f.steps.as_slice()
            {
                a.ops = vec![Op::WM035HostDeathDraw {
                    observation: observation.clone(),
                }];
            }
            self.wm035_observation_key(f.actor, &f.source, &a.ops)?
                .ok_or("宿主帧缺少观察")?;
        }
        let steps = a
            .ops
            .iter()
            .map(|op| Step {
                context: f.actor,
                op: op.clone(),
            })
            .collect::<Vec<_>>();
        if !same(&steps, &f.steps)
            || f.source.card.face_down
            || f.source.play_source.is_some()
            || f.chosen_region.is_some()
            || f.targets.len() != a.targets.len()
            || f.targets.iter().zip(&a.targets).any(|(t, spec)| {
                !same(&t.spec, spec)
                    || t.id.is_empty()
                    || t.public.instance_id != t.id
                    || !t.public.valid
                    || t.public.status != "valid"
                    || t.public.invalid_reason.is_some()
                    || t.region_instance.is_some()
                    || t.public.kind != match spec.kind { EntityKind::Character=>"character",EntityKind::Hidden=>"hidden",_=>"attachment" }
                    || t.public
                        .owner
                        .as_deref()
                        .is_none_or(|id| !self.players.iter().any(|p| player_id(p.seat) == id))
                    || t.public
                        .controller
                        .as_deref()
                        .is_none_or(|id| !self.players.iter().any(|p| player_id(p.seat) == id))
            })
        {
            return Err("第三批程序、冻结目标或执行环境不符".into());
        }
        if a.event.is_some() {
            return if f.already_paid.is_empty() {
                Ok(())
            } else {
                Err("宿主死亡触发不应有费用收据".into())
            };
        }
        if f.ability_key == "batch3-bq037-reveal" {
            if !matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)] if ids.len()==1 && !ids[0].is_empty() && ids[0]!=f.source.card.id && f.targets.iter().all(|t|t.id!=ids[0]))
                || !self.turn_ability_usage.iter().any(|u|u.source_instance==f.source.card.id && u.ability_key==f.ability_key && u.turn==self.turn && u.uses==1) { return Err("穿刺公现身须保留支付1和本回合限次记录".into()); }return Ok(());
        }
        if f.ability_key == "batch3-xg26-barrier" {
            if f.source.card.exhausted
                || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids), PaidCost::Exhausted(id)]
                if ids.len() == 1 && !ids[0].is_empty() && ids[0] != f.source.card.id
                    && f.targets.iter().all(|target| target.id != ids[0]) && id == &f.source.card.id)
            {
                return Err("看场打手必须实际支付1并横置自身".into());
            }
            return Ok(());
        }
        let [PaidCost::Batch3RemovedTime(p)] = f.already_paid.as_slice() else {
            return Err("第三批缺少时间支付收据".into());
        };
        let carrier_definition = catalog::catalog()
            .cards
            .iter()
            .find(|d| d.id == p.carrier.definition)
            .ok_or("第三批时间费用载体不是已登记卡牌")?;
        if p.carrier.time_markers.checked_sub(1) != Some(p.remaining)
            || p.region != f.source.region
            || p.carrier.face_down
            || p.carrier.owner >= self.players.len()
            || p.carrier.controller >= self.players.len()
            || p.carrier.id.is_empty()
            || (p.carrier.id == f.source.card.id
                && (p.attachment_host.is_some() || !same(&p.carrier, &f.source.card)))
            || (p.carrier.id != f.source.card.id
                && (p.attachment_host.as_ref() != Some(&f.source.card.id)
                    || carrier_definition.kind != "attachment"))
        {
            return Err("第三批时间费用的数量或真实载体不符".into());
        }
        self.mobility_immutable_identity(&p.carrier)
    }
    pub(crate) fn validate_batch3_state(&self) -> RuleResult<()> {
        self.validate_entry_order_groups()?;
        self.validate_bq026_state()?;
        self.validate_inspection_receipts()?;
        self.validate_attachment_barrier_modifiers()?;
        self.validate_bq079_markers()?;
        self.validate_named_characters()?;
        let mut frames = BTreeSet::new();
        let mut observations = BTreeSet::new();
        let mut remember = |source: &SourceSnapshot, ops: &[Op]| -> RuleResult<()> {
            if matches!(ops, [Op::CQ07EnterMoon]) && !observations.insert((source.card.id.clone(), "CQ07-enter".into())) { return Err("菲碧同次进场不能重复补隐秘标志".into()); }
            if let [Op::CQ07SpellTargeted { observation: Some(o) }] = ops {
                if !observations.insert((source.card.id.clone(),format!("{}:{}",o.event_id,o.target.card.id))) { return Err("菲碧同次事务指定同一角色不能重复抓牌".into()); }
            }
            if matches!(ops, [Op::JZ25SealRandomHand])
                && !observations.insert((source.card.id.clone(), "JZ25-enter".into())) {
                return Err("同一猎杀者进场不能重复封印".into());
            }
            if let [Op::JZ26Trap { observation: Some(o) }] = ops {
                if !observations.insert((source.card.id.clone(),o.event_id.clone())) {return Err("同次检视或消灭不能重复毒物触发".into());}
            }
            if let [Op::BQ026Attach { observation: Some(o) }] = ops {
                if !observations.insert((source.card.id.clone(),o.event_id.clone())) { return Err("蝙蝠群同次进入地区不能重复触发".into()); }
            }
            if let [Op::BQ029SelfFeed { observation: Some(o) }] = ops {
                if !observations.insert((source.card.id.clone(),o.event_id.clone())) { return Err("赤鳞近卫同次进场不能重复饮血".into()); }
            }
            if let [Op::BQ037FeedPair { observation: Some(o) }] = ops {
                if !observations.insert((source.card.id.clone(),o.event_id.clone())) { return Err("同一吸血鬼进场不能重复饮血".into()); }
            }
            if matches!(ops,[Op::XQ26Name]) && !observations.insert((source.card.id.clone(),"XQ26-enter".into())) { return Err("通缉令进场命名不能重复".into()); }
            if let [Op::XQ26Wanted { observation: Some(o) }] = ops {
                if !observations.insert((source.card.id.clone(),o.event_id.clone())) { return Err("同一通缉入场不能由同一令重复执行".into()); }
            }
            if let [Op::WM041PublicPlay { observation: Some(o) }] = ops {
                if !observations.insert((source.card.id.clone(),o.event_id.clone())) { return Err("阿尔法先生同一公开打出不能重复奖励".into()); }
            }
            if let [Op::BQ079CorpseIcons { observation: Some(o) }] = ops {
                if !observations.insert((source.card.id.clone(), o.event_id.clone())) { return Err("食尸鬼同一入墓事件不能重复执行".into()); }
            }
            if let [Op::WM035HostDeathDraw {
                observation: Some(o),
            }] = ops
            {
                if !observations.insert((source.card.id.clone(), o.event_id.clone())) {
                    return Err("同一宿主死亡观察不能由同一来源重复执行".into());
                }
            }
            Ok(())
        };
        for s in &self.stack {
            if let Some(f) = &s.frame {
                self.validate_batch3_frame(f)?;
                if !related(f) || self.validated_cross_card_frame(f)? {
                    continue;
                }
                if let [Step { op, .. }] = f.steps.as_slice() {
                    remember(&f.source, std::slice::from_ref(op))?;
                }
                let deploy = matches!(f.ability_key.as_str(), "deploy" | "reveal");
                let played = matches!(f.ability_key.as_str(), "batch3-xq26-attach" | "batch3-xg28-attach" | "batch3-bq089-reanimate" | "batch3-jc062-investigate" | "batch3-jc064-attach");
                if !frames.insert(&f.frame_id)
                    || s.id != f.frame_id
                    || s.controller != f.actor
                    || s.target != f.targets.first().map(|t| t.id.clone())
                    || f.cursor != 0
                    || !matches!(f.guard, GuardState::Unchecked)
                    || (deploy || played) != s.card.is_some()
                    || s.card.as_ref().is_some_and(|c| {
                        c.definition != f.source.card.definition
                            || c.owner != f.source.card.owner
                            || c.controller != f.actor
                            || c.face_down
                    })
                    || s.reveal != (f.ability_key == "reveal")
                    || s.deploy_region != (if deploy { f.chosen_region } else { None })
                {
                    return Err("第三批响应堆叠外壳或实体不符".into());
                }
            } else if s
                .card
                .as_ref()
                .is_some_and(|c| IMPLEMENTING.contains(&c.definition.as_str()))
            {
                return Err("第三批堆叠实体缺少执行帧".into());
            }
        }
        for e in &self.effects {
            match e {
                Effect::Declare { declaration } => {
                    self.validate_batch3_declaration(declaration)?;
                    remember(&declaration.source, &declaration.ability.ops)?;
                }
                Effect::Frame { frame }
                    if related(frame) && !self.validated_cross_card_frame(frame)? =>
                {
                    return Err("第三批不能绕过响应堆叠".into())
                }
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Frame {
                    frame,
                    choice: FrameChoice::WM033GraveSpell,
                } => self.validate_wm033_pending(p, frame)?,
                ChoiceResolution::Frame { frame, choice: FrameChoice::JC062Inspection { inspected } } =>
                    self.validate_jc062_pending(p, frame, inspected)?,
                ChoiceResolution::Frame { frame, choice: FrameChoice::XQ22Orientation } => self.validate_xq22_pending(p, frame)?,
                ChoiceResolution::Frame { frame, choice: FrameChoice::BloodFeed { state } } => {
                    self.validate_blood_pending(p,frame,state)?;remember(&frame.source,&frame.steps.iter().map(|s|s.op.clone()).collect::<Vec<_>>())?;
                },
                ChoiceResolution::Frame { frame, choice: FrameChoice::BQ037Reveal } => self.validate_bq037_reveal_pending(p,frame)?,
                ChoiceResolution::Frame { frame, choice: FrameChoice::XQ26Name } => self.validate_xq26_pending(p, frame)?,
                ChoiceResolution::Frame { frame, choice: FrameChoice::JC064Upkeep } => self.validate_jc064_pending(p, frame)?,
                ChoiceResolution::Frame { frame, choice: FrameChoice::BQ079Upkeep } => self.validate_bq079_pending(p, frame)?,
                ChoiceResolution::Declare { declaration, stage } => {
                    self.validate_batch3_declaration(declaration)?;
                    self.batch3_reject_optional(declaration)?;
                    if declaration.source.card.definition == "CQ07" { self.validate_cq07_pending(p,declaration,stage)?; }
                    if declaration.source.card.definition == "JZ26" {self.validate_jz26_pending(p,declaration,stage)?;}
                    if declaration.source.card.definition == "BQ026" { self.validate_bq026_pending(p,declaration,stage)?; }
                    if declaration.source.card.definition == "WM035"
                        && declaration.ability.key == "batch3-wm035-host-death"
                    {
                        self.validate_wm035_pending(p, declaration, stage)?;
                    }
                    if declaration.source.card.definition == "BQ037" || declaration.ability.key=="batch3-bq029-feed" { self.validate_blood_trigger_pending(p,declaration,stage)?; }
                    if declaration.source.card.definition=="BQ029" && declaration.ability.key=="raid-1" { self.validate_bq029_raid_pending(p,declaration,stage)?; }
                    if declaration.source.card.definition == "XQ26" { self.validate_xq26_trigger_pending(p,declaration,stage)?; }
                    if declaration.source.card.definition == "BQ079" && declaration.ability.key == "batch3-bq079-corpse" { self.validate_bq079_trigger_pending(p,declaration,stage)?; }
                    if declaration.source.card.definition == "JZ25" {
                        self.validate_jz25_pending(p, declaration, stage)?;
                    }
                    remember(&declaration.source, &declaration.ability.ops)?;
                }
                ChoiceResolution::Frame { frame, .. }
                    if related(frame) && !self.validated_cross_card_frame(frame)? =>
                {
                    return Err("第三批当前程序没有结算中选择".into())
                }
                _ => {}
            }
        }
        Ok(())
    }
    /// Both printed grants are permanent-category icons. Exhausting a source
    /// does not blank its continuous text. 本方 is the current controller seat.
    pub(crate) fn wm030_cultist_icons(&self, c: &Card, region: usize) -> Icons {
        if c.face_down || catalog::card(&c.definition).kind != "character" {
            return Icons::default();
        }
        let sources = self
            .regions
            .iter()
            .filter(|r| !r.vacant)
            .flat_map(|r| &r.cards)
            .filter(|source| {
                source.definition == "WM030"
                    && !source.face_down
                    && source.controller == c.controller
                    && rules::definition(&source.definition)
                        .modifiers
                        .iter()
                        .any(|m| matches!(m, StaticModifier::WM030CultistIcons))
            })
            .count() as u32;
        if sources == 0
            || !self
                .regions
                .get(region)
                .is_some_and(|r| !r.vacant && r.cards.iter().any(|live| live == c))
            || !self.current_subtypes(c).iter().any(|s| s == "邪教徒")
        {
            return Icons::default();
        }
        Icons {
            investigation: if self
                .players
                .get(c.controller)
                .is_some_and(|p| p.society_zone.secrecy >= 3)
            {
                sources
            } else {
                0
            },
            combat: 0,
            influence: if self.regions[region].destroyed {
                sources
            } else {
                0
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unit_support::*;

    #[test]
    fn batch3_wm030_dynamic_personal_cultist_grants_are_permanent_icons() {
        let mut g = game();
        let source = field(&mut g, "WM030", 0);
        let own = field(&mut g, "JC042", 0);
        let ally = field(&mut g, "JC042", 1);
        let enemy = field(&mut g, "JC042", 2);
        g.first_team = 1; // Temporary icons are off for seat zero.
        let before = g.current_icons(g.board(&own).unwrap().1, 0);
        g.regions[0].destroyed = true;
        g.add_society_markers(0, 0, 2);
        assert_eq!(
            g.wm030_cultist_icons(g.board(&own).unwrap().1, 0),
            Icons {
                influence: 1,
                ..Icons::default()
            }
        );
        g.add_society_markers(0, 0, 1);
        let after = g.current_icons(g.board(&own).unwrap().1, 0);
        assert_eq!(after.investigation, before.investigation + 1);
        assert_eq!(after.influence, before.influence + 1);
        assert_eq!(
            g.wm030_cultist_icons(g.board(&ally).unwrap().1, 0),
            Icons::default()
        );
        assert_eq!(
            g.wm030_cultist_icons(g.board(&enemy).unwrap().1, 0),
            Icons::default()
        );
        g.board_mut(&source).unwrap().exhausted = true;
        assert_eq!(
            g.wm030_cultist_icons(g.board(&own).unwrap().1, 0)
                .investigation,
            1
        );
        g.add_society_markers(0, 1, 0); // Exposure cancels one secrecy immediately.
        assert_eq!(
            g.wm030_cultist_icons(g.board(&own).unwrap().1, 0)
                .investigation,
            0
        );
        restore(&mut g);
    }

    #[test]
    fn batch3_wm030_sources_stack_but_hidden_sources_and_targets_do_not_qualify() {
        let mut g = game();
        let a = field(&mut g, "WM030", 0);
        let b = field(&mut g, "WM030", 0);
        g.regions[0].destroyed = true;
        let c = g.board(&a).unwrap().1.clone();
        assert_eq!(g.wm030_cultist_icons(&c, 0).influence, 2);
        g.board_mut(&b).unwrap().face_down = true;
        assert_eq!(g.wm030_cultist_icons(&c, 0).influence, 1);
        let mut hidden = c.clone();
        hidden.face_down = true;
        assert_eq!(g.wm030_cultist_icons(&hidden, 0), Icons::default());
        g.board_mut(&a).unwrap().controller = 1;
        assert_eq!(g.wm030_cultist_icons(&c, 0), Icons::default()); // stale target snapshot
        let live = g.board(&a).unwrap().1.clone();
        assert_eq!(g.wm030_cultist_icons(&live, 0).influence, 1);
        let mut moved = live.clone();
        moved.id = "not-on-board".into();
        assert_eq!(g.wm030_cultist_icons(&moved, 0), Icons::default());
    }

    #[test]
    fn batch3_wm030_program_cannot_be_modified_or_transplanted() {
        let d = definition("WM030").unwrap();
        validate_definition("WM030", &d).unwrap();
        assert!(validate_definition("JC125", &d).is_err());
        assert!(validate_definition("WM030", &Definition::default()).is_err());
        let mut definitions = rules::definitions().clone();
        definitions.insert("WM030".into(), Definition::default());
        assert!(rules::validate_definitions(&definitions).is_err());
    }

    #[test]
    fn batch3_bq042_four_seat_view_tracks_conditional_spirit_without_revealing_hidden_print() {
        let mut g = game();
        let id = field(&mut g, "BQ042", 0);
        for moon in [false, true, false] {
            g.players[0].society_zone.secrecy = u32::from(moon);
            // An ally's moon cannot enable this personal keyword.
            g.players[1].society_zone.secrecy = 1;
            for seat in 0..4 {
                let view = g.view(seat);
                let c = view.regions[0].characters.iter().find(|c| c.instance_id == id).unwrap();
                assert_eq!(c.current_spirit_protection, moon.then_some(true));
            }
            restore(&mut g);
        }
        g.players[0].society_zone.secrecy = 1;
        g.board_mut(&id).unwrap().face_down = true;
        for seat in 0..4 {
            let view = g.view(seat);
            let c = view.regions[0].characters.iter().find(|c| c.instance_id == id).unwrap();
            assert_eq!(c.current_spirit_protection, None);
            if seat != 0 { assert_eq!(c.card_id, None); }
        }
        restore(&mut g);
    }

    #[test]
    fn batch3_bq042_entry_time_spirit_condition_and_exact_paid_action() {
        let mut g = game();
        fund(&mut g, 0, "JC047", 2);
        let source = held(&mut g, "BQ042", 0);
        act(
            &mut g,
            0,
            Action {
                card_id: Some(source.clone()),
                region: Some(0),
                ..Action::new("deploy")
            },
        );
        top(&mut g);
        let source = g.regions[0]
            .cards
            .iter()
            .find(|c| c.definition == "BQ042")
            .unwrap()
            .id
            .clone();
        assert_eq!(g.board(&source).unwrap().1.time_markers, 1);
        assert!(!g.current_spirit(g.board(&source).unwrap().1));
        g.add_society_markers(0, 0, 1);
        assert!(g.current_spirit(g.board(&source).unwrap().1));
        g.add_society_markers(0, 1, 0);
        assert!(!g.current_spirit(g.board(&source).unwrap().1));
        let attachment = g.make_card("JC048", 2);
        let target = attachment.id.clone();
        g.attachments.push(Attachment {
            card: attachment,
            host_id: g.regions[0].card.id.clone(),
        });
        let action = Action {
            card_id: Some(source.clone()),
            target_id: Some(target.clone()),
            cost_selected: Some(vec![source.clone()]),
            ..Action::new("activate")
        };
        act(&mut g, 0, action.clone());
        assert_eq!(g.board(&source).unwrap().1.time_markers, 0);
        let f = g.stack.last().unwrap().frame.as_ref().unwrap();
        assert!(
            matches!(f.already_paid.as_slice(), [PaidCost::Batch3RemovedTime(p)] if p.carrier.id == source && p.remaining == 0)
        );
        assert_eq!(f.targets[0].public.kind, "attachment");
        let mut forged = g.clone();
        if let PaidCost::Batch3RemovedTime(p) = &mut forged
            .stack
            .last_mut()
            .unwrap()
            .frame
            .as_mut()
            .unwrap()
            .already_paid[0]
        {
            p.remaining = 1;
        }
        assert!(Game::from_persisted(&serde_json::to_string(&forged).unwrap()).is_err());
        top(&mut g);
        assert!(!g.attachments.iter().any(|a| a.card.id == target));
        g.board_mut(&source).unwrap().time_markers = 1;
        reject(&mut g, 0, action); // Source instance has already used this turn.
    }

    #[test]
    fn batch3_bq042_rejects_character_hosts_and_other_regions_without_spending_time() {
        let mut g = game();
        let source = field(&mut g, "BQ042", 0);
        g.batch3_entry_keywords("BQ042", &source);
        let host = field(&mut g, "JC125", 2);
        let attachment = g.make_card("BQ022", 2);
        let wrong_host = attachment.id.clone();
        g.attachments.push(Attachment {
            card: attachment,
            host_id: host,
        });
        let attachment = g.make_card("JC048", 2);
        let wrong_region = attachment.id.clone();
        g.attachments.push(Attachment {
            card: attachment,
            host_id: g.regions[1].card.id.clone(),
        });
        for target in [wrong_host, wrong_region] {
            reject(
                &mut g,
                0,
                Action {
                    card_id: Some(source.clone()),
                    target_id: Some(target),
                    cost_selected: Some(vec![source.clone()]),
                    ..Action::new("activate")
                },
            );
        }
        assert_eq!(g.board(&source).unwrap().1.time_markers, 1);
        let mut defs = rules::definitions().clone();
        defs.get_mut("BQ042").unwrap().abilities[0].per_turn_limit = None;
        assert!(rules::validate_definitions(&defs).is_err());
        assert!(
            rules::validate_ability("JC125", &definition("BQ042").unwrap().abilities[0]).is_err()
        );
    }

    #[test]
    fn batch3_bq042_attached_time_can_pay_to_destroy_an_enemy_society_attachment() {
        let mut g = game();
        let source = field(&mut g, "BQ042", 0);
        let mut time = g.make_card("JZ41", 0);
        time.time_markers = 1;
        let carrier = time.id.clone();
        g.attachments.push(Attachment {
            card: time,
            host_id: source.clone(),
        });
        g.players[2].society_zone.card = Some(g.make_card("MSJC04", 2));
        let attachment = g.make_card("JC068", 2);
        let target = attachment.id.clone();
        g.attachments.push(Attachment {
            card: attachment,
            host_id: "society:p2".into(),
        });
        act(
            &mut g,
            0,
            Action {
                card_id: Some(source.clone()),
                target_id: Some(target.clone()),
                cost_selected: Some(vec![carrier.clone()]),
                ..Action::new("activate")
            },
        );
        assert_eq!(g.board_mut(&carrier).unwrap().time_markers, 0);
        let mut forged = g.clone();
        if let PaidCost::Batch3RemovedTime(p) = &mut forged
            .stack
            .last_mut()
            .unwrap()
            .frame
            .as_mut()
            .unwrap()
            .already_paid[0]
        {
            p.carrier.definition = "UNREGISTERED".into();
        }
        let invalid = serde_json::to_string(&forged).unwrap();
        assert!(std::panic::catch_unwind(|| Game::from_persisted(&invalid))
            .unwrap()
            .is_err());
        let mut transplanted = g.clone();
        transplanted
            .stack
            .last_mut()
            .unwrap()
            .frame
            .as_mut()
            .unwrap()
            .source
            .card
            .definition = "JC125".into();
        assert!(Game::from_persisted(&serde_json::to_string(&transplanted).unwrap()).is_err());
        top(&mut g);
        assert!(!g.attachments.iter().any(|a| a.card.id == target));
        assert!(g.attachments.iter().any(|a| a.card.id == carrier));
        restore(&mut g);
    }

    #[test]
    fn batch3_bq042_lost_target_cancels_without_refunding_time_or_limit() {
        let mut g = game();
        let source = field(&mut g, "BQ042", 0);
        g.batch3_entry_keywords("BQ042", &source);
        let attachment = g.make_card("JC048", 2);
        let target = attachment.id.clone();
        g.attachments.push(Attachment {
            card: attachment,
            host_id: g.regions[0].card.id.clone(),
        });
        act(
            &mut g,
            0,
            Action {
                card_id: Some(source.clone()),
                target_id: Some(target.clone()),
                cost_selected: Some(vec![source.clone()]),
                ..Action::new("activate")
            },
        );
        g.remove_dead(&target, RemovalCause::Sacrifice);
        g.drive().unwrap();
        top(&mut g);
        assert_eq!(g.board(&source).unwrap().1.time_markers, 0);
        assert!(g
            .turn_ability_usage
            .iter()
            .any(|u| u.source_instance == source && u.uses == 1));
        assert!(g
            .log
            .iter()
            .any(|entry| entry.text.contains("整个卡牌或能力效果取消")));
        restore(&mut g);
    }
}
