//! Package B's independent native slices. These five cards remain unsupported:
//! their contest clauses use the shared protocol through B finite adapters.
//! Blood uses the established grave/seal sources and normal attachment entry.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub(crate) const CARDS: &[&str] = &["BQ027", "BQ031", "BQ032", "BQ036", "BQ038"];
pub(crate) const FEED_TARGET: &str = "batch4-bq027-feed";
pub(crate) const ATTACH_DISCARD: &str = "batch4-bq031-attachment";
pub(crate) const REVEAL_FEED: &str = "batch4-bq032-feed";
pub(crate) const SEARCH: &str = "batch4-bq036-search";
pub(crate) const RAID_FEED: &str = "batch4-bq038-raid";
const BLOOD_CHOICE: &str = "batch4-vampire-blood";
const SEARCH_TITLE: &str = "菲尼克斯：查看顶六张，选择任意装备、武器或状态附属";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttachmentObservation {
    pub event_id: String,
    pub turn: u32,
    pub attachment: Attachment,
    pub host: SourceSnapshot,
    pub subtypes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RaidObservation {
    pub turn: u32,
    /// The accepted raid frame, frozen before responses, not an arbitrary effect.
    pub raid: Box<ResolutionFrame>,
}

/// One qualifying attachment is enough for a counted character. BQ038 freezes
/// these identities in its finite resolved start-bonus receipt.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bq038CharacterWitness {
    pub character: SourceSnapshot,
    pub attachment: Attachment,
}

fn same<T: Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
fn raid() -> AbilitySpec {
    ability("raid-1", "袭击1", Timing::Fast, vec![],
        vec![target(Zone::Board, EntityKind::Character, Relation::Any, Range::SourceRegion)],
        vec![Op::DamageTarget { slot: 0, amount: 1 }], Some(Event::Reveal))
}
pub(crate) fn definition(id: &str) -> Option<Definition> {
    let mut d = match id {
        "BQ027" => {
            let mut t = target(Zone::Board, EntityKind::Character, Relation::Any, Range::Anywhere);
            t.subtype = Some("吸血鬼".into());
            let mut a = ability(FEED_TARGET, "牺牲血畜：目标吸血鬼饮血1", Timing::Fast,
                vec![Cost::SacrificeSource], vec![t], vec![Op::BQ027FeedTarget], None);
            a.activation_only = true;
            with_abilities(vec![a])
        }
        "BQ031" => {
            let mut a = ability(ATTACH_DISCARD, "本方吸血鬼或忍者被结附：目标敌方弃1，每回合一次", Timing::Fast,
                vec![], vec![target(Zone::Player, EntityKind::Any, Relation::EnemyTeam, Range::Anywhere)],
                vec![Op::BQ031AttachmentDiscard { observation: None }], Some(Event::BQ031Attachment));
            a.per_turn_limit = Some(1);
            with_abilities(vec![raid(), a])
        }
        "BQ032" => with_abilities(vec![raid(), ability(REVEAL_FEED, "现身：饮血2（至多两张）", Timing::Fast,
            vec![], vec![], vec![Op::BQ032RevealFeed], Some(Event::Reveal))]),
        "BQ036" => with_abilities(vec![ability(SEARCH, "现身：顶六张寻找装备、武器或状态附属，展示入手，其余洗回", Timing::Fast,
            vec![], vec![], vec![Op::BQ036TopSix], Some(Event::Reveal))]),
        "BQ038" => {
            let mut d = crate::rules::attachment_definition(target(Zone::Society, EntityKind::Any, Relation::ControlledByActor, Range::Anywhere));
            d.abilities.push(ability(RAID_FEED, "本方角色袭击：可令其饮血1", Timing::Fast, vec![], vec![],
                vec![Op::BQ038RaidFeed { observation: None }], Some(Event::BQ038Raid)));
            d
        }
        _ => return None,
    };
    if let Some(a) = crate::batch4_vampire_contest::ability_for(id) { d.abilities.push(a); }
    Some(d)
}
pub(crate) fn canonical(id: &str, a: &AbilitySpec) -> Option<AbilitySpec> {
    if crate::batch4_vampire_contest::key(&a.key) { return crate::batch4_vampire_contest::canonical(id, a); }
    let mut expected = definition(id)?.abilities.into_iter().find(|x| x.key == a.key)?;
    match a.ops.as_slice() {
        [Op::BQ031AttachmentDiscard { observation }] if a.key == ATTACH_DISCARD =>
            expected.ops = vec![Op::BQ031AttachmentDiscard { observation: observation.clone() }],
        [Op::BQ038RaidFeed { observation }] if a.key == RAID_FEED =>
            expected.ops = vec![Op::BQ038RaidFeed { observation: observation.clone() }],
        _ => (),
    }
    Some(expected)
}
pub(crate) fn owns(ops: &[Op]) -> bool {
    if crate::batch4_vampire_contest::owns(ops) { return true; }
    ops.iter().any(|op| match op {
        Op::BQ027FeedTarget | Op::BQ031AttachmentDiscard { .. } | Op::BQ032RevealFeed | Op::BQ036TopSix | Op::BQ038RaidFeed { .. } => true,
        Op::ForEachLivingPlayer(ops) | Op::ForEachLivingPlayerFromActor(ops) => owns(ops),
        Op::IfTargetExhausted { exhausted, ready, .. } => owns(&[*exhausted.clone(), *ready.clone()]),
        _ => false,
    })
}
pub(crate) fn related_frame(f: &ResolutionFrame) -> bool {
    CARDS.contains(&f.source.card.definition.as_str()) || crate::batch4_vampire_contest::related(f) || f.steps.iter().any(|s| owns(std::slice::from_ref(&s.op)))
}
pub(crate) fn validate_action(id: &str, a: &Action) -> RuleResult<()> {
    let expected = match (id, a.kind.as_str()) {
        ("BQ032", "activate") if a.ability_id.as_deref() == Some(crate::batch4_vampire_contest::HAND) =>
            Action { card_id: a.card_id.clone(), ability_id: a.ability_id.clone(), ..Action::new("activate") },
        ("BQ036", "activate") if a.ability_id.as_deref() == Some(crate::batch4_vampire_contest::SACRIFICE) =>
            Action { card_id: a.card_id.clone(), ability_id: a.ability_id.clone(), cost_selected: a.cost_selected.clone(), ..Action::new("activate") },
        ("BQ038", "play") if a.ability_id.as_deref().is_none_or(|k| k == "attach") =>
            Action { card_id: a.card_id.clone(), ability_id: a.ability_id.clone(), target_id: a.target_id.clone(), ..Action::new("play") },
        ("BQ027", "activate") if a.ability_id.as_deref() == Some(FEED_TARGET) =>
            Action { card_id: a.card_id.clone(), ability_id: a.ability_id.clone(), target_id: a.target_id.clone(), ..Action::new("activate") },
        ("BQ027" | "BQ031" | "BQ032" | "BQ036", "deploy" | "conceal") =>
            Action { card_id: a.card_id.clone(), region: a.region, ..Action::new(&a.kind) },
        ("BQ027" | "BQ031" | "BQ032" | "BQ036", "reveal") =>
            Action { card_id: a.card_id.clone(), ..Action::new("reveal") },
        _ => return Err("本包尚未准入隐秘对抗行动，仅接受已实现的固定原生程序".into()),
    };
    if *a != expected { return Err("吸血鬼程序不能附带额外目标、费用或数量".into()); }
    Ok(())
}
pub(crate) fn validate_choice_action(p: &Pending, a: &Action) -> RuleResult<()> {
    let related = match &p.resolution {
        ChoiceResolution::Declare { declaration: d, .. } => CARDS.contains(&d.source.card.definition.as_str()),
        ChoiceResolution::Frame { frame: f, .. } => related_frame(f),
        _ => false,
    };
    if related && *a != (Action { choice_id: a.choice_id.clone(), selected: a.selected.clone(), ..Action::new("choose") }) {
        return Err("本包待选只接受当前选项集合".into());
    }
    Ok(())
}
fn searchable(c: &Card) -> bool {
    let d = catalog::card(&c.definition);
    catalog::has_kind(d, "attachment") && d.subtypes.iter().any(|s| matches!(s.as_str(), "装备" | "武器" | "状态"))
}
impl Game {
    /// Read-only cost eligibility for the BQ036 action adapter. The
    /// existing source-character sacrifice cost cannot express this predicate.
    pub(crate) fn bq036_contest_cost_candidates(&self, actor: usize) -> Vec<String> {
        if self.players.get(actor).is_none_or(|p| p.eliminated) { return vec![]; }
        self.attachments.iter().filter(|a| !a.card.face_down && a.card.controller == actor && self.attachment_host_valid(a))
            .filter(|a| self.board(&a.host_id).is_some_and(|(_, host)| !host.face_down && host.controller == actor
                && catalog::has_kind(catalog::card(&host.definition), "character")))
            .map(|a| a.card.id.clone()).collect()
    }

    /// BQ032 is a hand action with an explicit pay2/discard-self cost, not a
    /// printed-cost play. This helper only authenticates the pre-payment source.
    pub(crate) fn bq032_hand_contest_source(&self, actor: usize, id: &str) -> RuleResult<SourceSnapshot> {
        let player = self.players.get(actor).filter(|p| !p.eliminated).ok_or("宴会司仪手牌行动的玩家无效")?;
        let c = player.hand.iter().find(|c| c.id == id && c.definition == "BQ032" && !c.face_down
            && c.controller == actor && c.owner < self.players.len()).ok_or("神秘学博弈必须从本人原宴会司仪手牌弃置")?;
        if self.vampire_id(id).is_none() { return Err("宴会司仪手牌实例无效".into()); }
        self.mobility_immutable_identity(c)?;
        Ok(self.source_snapshot(c, None))
    }

    /// Evaluate at the BQ038 trigger's resolution, then retain these identities
    /// in its B-owned resolved bonus operation. A
    /// foreign-controlled attachment qualifies; only its character is personal.
    pub(crate) fn bq038_contest_count_witnesses(&self, actor: usize) -> Vec<Bq038CharacterWitness> {
        if self.players.get(actor).is_none_or(|p| p.eliminated) { return vec![]; }
        let mut out = vec![];
        for (r, region) in self.regions.iter().enumerate().filter(|(_, r)| !r.vacant) {
            for character in region.cards.iter().filter(|c| !c.face_down && c.controller == actor
                && catalog::has_kind(catalog::card(&c.definition), "character")) {
                if let Some(attachment) = self.attachments.iter().find(|a| a.host_id == character.id && !a.card.face_down
                    && self.attachment_host_valid(a) && catalog::has_kind(catalog::card(&a.card.definition), "attachment")
                    && self.current_subtypes(&a.card).iter().any(|s| matches!(s.as_str(), "装备" | "武器" | "状态"))) {
                    let mut source = self.source_snapshot(character, Some(r));
                    source.source_region_instance = Some(region.card.id.clone());
                    out.push(Bq038CharacterWitness { character: source, attachment: attachment.clone() });
                }
            }
        }
        out
    }

    fn vampire_id(&self, id: &str) -> Option<u64> {
        id.strip_prefix('i').and_then(|x| x.parse::<u64>().ok()).filter(|n| *n > 0 && *n <= self.sequence && id == format!("i{n}"))
    }
    pub(crate) fn vampire_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        let c = &s.card;
        if !CARDS.contains(&c.definition.as_str()) || actor >= self.players.len() || c.controller != actor || c.owner >= self.players.len()
            || self.vampire_id(&c.id).is_none() || c.face_down || s.play_source.is_some() || s.death_echo.is_some()
            || s.observed_death.is_some() || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some() {
            return Err("吸血鬼程序来源或原操控者无效".into());
        }
        self.mobility_immutable_identity(c)?;
        if c.definition == "BQ038" {
            if s.region.is_some() || s.source_region_instance.is_some() || s.attachment_host_instance.as_deref() != Some(&format!("society:p{actor}")) {
                return Err("掠食时刻观察须保留本人秘社宿主".into());
            }
        } else if s.region.is_none_or(|r| r >= self.regions.len()) || s.source_region_instance.as_ref().is_none_or(|id| self.vampire_id(id).is_none()) || s.attachment_host_instance.is_some() {
            return Err("吸血鬼来源须保留原地区实体".into());
        }
        if self.batch3_physical_cards().any(|now| now.id == c.id) && self.in_play(&c.id).is_none() {
            return Err("私有区不能冒充原在场来源".into());
        }
        Ok(())
    }
    pub(crate) fn validate_vampire_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if crate::batch4_vampire_contest::key(&d.ability.key) || crate::batch4_vampire_contest::owns(&d.ability.ops) { return self.validate_b_contest_declaration(d); }
        self.vampire_source(d.actor, &d.source)?;
        if d.ability.event.is_none() || canonical(&d.source.card.definition, &d.ability).is_none_or(|a| !same(&a, &d.ability)) {
            return Err("吸血鬼触发须保留固定原程序".into());
        }
        self.vampire_observation(d.actor, &d.source, &d.ability.ops)
    }
    fn vampire_observation(&self, actor: usize, source: &SourceSnapshot, ops: &[Op]) -> RuleResult<()> {
        match ops {
            [Op::BQ031AttachmentDiscard { observation: Some(o) }] => {
                let h = &o.host; let a = &o.attachment;
                let Some(event) = self.vampire_id(&o.event_id) else { return Err("雾羽千军结附事件身份无效".into()); };
                let printed = catalog::catalog().cards.iter().find(|d| d.id == h.card.definition && catalog::runtime_implemented(d)).ok_or("结附宿主不是准入牌面")?;
                let attached = catalog::catalog().cards.iter().find(|d| d.id == a.card.definition && catalog::runtime_implemented(d)).ok_or("结附观察不是准入附属")?;
                if source.card.definition != "BQ031" || o.turn != self.turn || h.card.controller != actor || h.card.face_down
                    || h.card.owner >= self.players.len() || a.card.owner >= self.players.len() || a.card.controller >= self.players.len()
                    || self.batch3_physical_cards().any(|c| c.id == o.event_id) || self.stack.iter().any(|s| s.id == o.event_id)
                    || h.region.is_none_or(|r| r >= self.regions.len()) || h.play_source.is_some() || h.death_echo.is_some()
                    || h.observed_death.is_some() || h.observed_exposure.is_some() || h.observed_mobility_grant.is_some()
                    || !catalog::has_kind(printed, "character")
                    || !catalog::has_kind(attached, "attachment") || a.card.face_down || a.host_id != h.card.id
                    || a.card.id == h.card.id || a.card.id == source.card.id
                    || !o.subtypes.iter().any(|s| matches!(s.as_str(), "吸血鬼" | "忍者"))
                    || o.subtypes.iter().collect::<BTreeSet<_>>().len() != o.subtypes.len()
                    || o.subtypes.iter().any(|s| !printed.subtypes.contains(s) && !matches!(s.as_str(), "吸血鬼" | "奴仆" | "邪教徒" | "宿主" | "不死生物" | "骷髅" | "恶魔" | "蝙蝠"))
                    || o.subtypes.iter().any(|s| s == "吸血鬼") && !(printed.subtypes.iter().any(|s| s == "吸血鬼" || s == "人类") || h.card.definition == "JC030")
                    || [&source.card.id, &h.card.id, &a.card.id].iter().any(|id| self.vampire_id(id).is_none_or(|n| n >= event)) {
                    return Err("雾羽千军须保留实际结附与个人吸血鬼/忍者观察".into());
                }
                for c in [&h.card, &a.card] {
                    self.mobility_immutable_identity(c)?;
                    if self.batch3_physical_cards().any(|now| now.id == c.id)
                        && (if c.id == a.card.id { !self.attachments.iter().any(|live| live.card.id == c.id) } else { self.in_play(&c.id).is_none() }) {
                        return Err("结附历史实体不能被私有区同UID替代".into());
                    }
                }
            }
            [Op::BQ038RaidFeed { observation: Some(o) }] => {
                if source.card.definition != "BQ038" || o.turn != self.turn || o.raid.actor != actor
                    || o.raid.source.card.face_down || self.vampire_id(&o.raid.frame_id).is_none()
                    || self.vampire_id(&source.card.id) >= self.vampire_id(&o.raid.frame_id)
                    || self.stack.iter().any(|item| item.id == o.raid.frame_id && item.frame.as_ref().is_none_or(|now| !same(now, &o.raid)))
                    || !self.vampire_is_raid(&o.raid)? {
                    return Err("掠食时刻须保留已发动的个人角色袭击".into());
                }
            }
            [Op::BQ031AttachmentDiscard { .. } | Op::BQ038RaidFeed { .. }] => return Err("观察触发缺少真实事件".into()),
            _ => (),
        }
        Ok(())
    }
    pub(crate) fn validate_vampire_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if crate::batch4_vampire_contest::related(f) { return self.validate_b_contest_frame(f); }
        let c = &f.source.card;
        if !CARDS.contains(&c.definition.as_str()) || f.actor >= self.players.len() || c.controller != f.actor || c.owner >= self.players.len()
            || self.vampire_id(&c.id).is_none() || self.vampire_id(&f.frame_id).is_none_or(|n| n <= self.vampire_id(&c.id).unwrap_or(u64::MAX)) {
            return Err("吸血鬼帧来源、操控者或编号无效".into());
        }
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            if c.definition == "BQ038" || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked) || !f.steps.is_empty() || !f.targets.is_empty()
                || f.source.region.is_none_or(|r| r >= self.regions.len()) || f.chosen_region != f.source.region
                || f.source.source_region_instance.as_ref().is_none_or(|id| self.vampire_id(id).is_none()) || f.source.attachment_host_instance.is_some()
                || f.source.death_echo.is_some() || f.source.observed_death.is_some() || f.source.observed_exposure.is_some() || f.source.observed_mobility_grant.is_some()
                || f.ability_key == "deploy" && (c.face_down || !f.source.valid_character_play())
                || f.ability_key == "reveal" && (!c.face_down || f.source.play_source.is_some())
                || !self.vampire_printed_payment(f) { return Err("吸血鬼派遣/现身须保留实际支付与原地区".into()); }
            return self.mobility_immutable_identity(c);
        }
        let mut a = definition(&c.definition).unwrap().abilities.into_iter().find(|a| a.key == f.ability_key).ok_or("本包尚未准入此程序")?;
        let ops = f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>();
        a.ops = ops.clone();
        if canonical(&c.definition, &a).is_none_or(|expected| !same(&a, &expected)) || f.cursor > f.steps.len() || f.steps.iter().any(|s| s.context != f.actor)
            || f.targets.len() != a.targets.len() || f.chosen_region.is_some()
            || f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked) && !(f.ability_key == "attach" && matches!(f.guard, GuardState::Accepted | GuardState::Cancelled))
            || f.cursor > 0 && !matches!(f.guard, GuardState::Accepted) { return Err("吸血鬼帧的程序、目标或游标被改写".into()); }
        if f.ability_key == "attach" {
            if c.definition != "BQ038" || c.face_down || f.source.region.is_some() || f.source.source_region_instance.is_some() || f.source.attachment_host_instance.is_some()
                || f.source.play_source.is_some() || f.source.death_echo.is_some() || f.source.observed_death.is_some() || f.source.observed_exposure.is_some() || f.source.observed_mobility_grant.is_some()
                || self.batch3_physical_cards().any(|x| x.id == c.id) || !self.vampire_printed_payment(f) { return Err("掠食时刻结附必须保留原手牌与支付".into()); }
            self.mobility_immutable_identity(c)?;
        } else {
            self.vampire_source(f.actor, &f.source)?;
            self.vampire_observation(f.actor, &f.source, &ops)?;
            if f.ability_key == FEED_TARGET {
                if !matches!(f.already_paid.as_slice(), [PaidCost::Sacrificed { old_instance, controller, owner }] if old_instance == &c.id && *controller == f.actor && *owner == c.owner)
                    || self.batch3_physical_cards().any(|now| now.id == c.id) { return Err("血畜须先支付原来源牺牲费用且不可重复".into()); }
            } else if !f.already_paid.is_empty() { return Err("本包触发不能追加支付".into()); }
            if f.ability_key == ATTACH_DISCARD && !self.turn_ability_usage.iter().any(|u| u.source_instance == c.id && u.ability_key == ATTACH_DISCARD && u.turn == self.turn && u.uses == 1) {
                return Err("雾羽千军触发缺少本回合已发动收据".into());
            }
        }
        for (t, expected) in f.targets.iter().zip(&a.targets) {
            if !same(&t.spec, expected) || t.id.is_empty() || t.public.instance_id != t.id || !t.public.valid || t.public.status != "valid" || t.public.invalid_reason.is_some() || t.region_instance.is_some() {
                return Err("吸血鬼目标原始绑定被改写".into());
            }
            match f.ability_key.as_str() {
                "attach" if t.id == format!("society:p{}", f.actor) && t.public.kind == "society" && t.public.region.is_none()
                    && t.public.owner.as_deref() == Some(&player_id(f.actor)) && t.public.controller.as_deref() == Some(&player_id(f.actor)) => (),
                ATTACH_DISCARD => {
                    let seat = t.id.strip_prefix('p').and_then(|x| x.parse::<usize>().ok()).filter(|s| *s < self.players.len()).ok_or("敌方玩家绑定无效")?;
                    if self.team(seat) == self.team(f.actor) || t.public.kind != "player" || t.public.region.is_some() || t.public.owner.is_some() || t.public.controller.as_deref() != Some(&t.id) { return Err("弃牌目标必须是原敌方玩家".into()); }
                }
                FEED_TARGET | "raid-1" if self.vampire_id(&t.id).is_some() && t.public.kind == "character" && t.public.region.is_some_and(|r| r < self.regions.len())
                    && (f.ability_key != "raid-1" || t.public.region == f.source.region) => (),
                _ => return Err("吸血鬼固定目标类型不符".into()),
            }
        }
        Ok(())
    }
    fn vampire_printed_payment(&self, f: &ResolutionFrame) -> bool {
        matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if f.money_units(0) <= catalog::card(&f.source.card.definition).cost as usize
            && ids.iter().all(|id| self.vampire_id(id).is_some() && *id != f.source.card.id)
            && ids.iter().collect::<BTreeSet<_>>().len() == ids.len()
            && !self.players.iter().enumerate().any(|(seat, p)| seat != f.actor && p.assets.iter().any(|c| ids.contains(&c.id))))
    }
    pub(crate) fn vampire_trigger_available(&self, d: &Declaration) -> bool {
        d.ability.key != ATTACH_DISCARD || !self.turn_ability_usage.iter().any(|u| u.source_instance == d.source.card.id && u.ability_key == ATTACH_DISCARD && u.turn == self.turn && u.uses >= 1)
    }
    pub(crate) fn bq031_observe_attachment(&mut self, id: &str) -> usize {
        let Some(attachment) = self.attachments.iter().find(|a| a.card.id == id && !a.card.face_down && self.attachment_host_valid(a)).cloned() else { return 0; };
        let Some((r, host)) = self.in_play(&attachment.host_id).filter(|(_, c)| !c.face_down && catalog::has_kind(catalog::card(&c.definition), "character")) else { return 0; };
        let actor = host.controller; let subtypes = self.current_subtypes(host);
        if self.players[actor].eliminated || !subtypes.iter().any(|s| matches!(s.as_str(), "吸血鬼" | "忍者")) { return 0; }
        let host = self.source_snapshot(host, r);
        let sources = self.in_play_cards().into_iter().filter(|(_, c)| c.definition == "BQ031" && !c.face_down && c.controller == actor)
            .filter(|(_, c)| !self.turn_ability_usage.iter().any(|u| u.source_instance == c.id && u.ability_key == ATTACH_DISCARD && u.turn == self.turn && u.uses >= 1))
            .map(|(r, c)| self.source_snapshot(c, r)).collect::<Vec<_>>();
        if sources.is_empty() { return 0; }
        let observation = AttachmentObservation { event_id: self.id(), turn: self.turn, attachment, host, subtypes };
        let count = sources.len();
        for source in sources {
            let mut a = definition("BQ031").unwrap().abilities.remove(1);
            a.ops = vec![Op::BQ031AttachmentDiscard { observation: Some(Box::new(observation.clone())) }];
            self.effects.push_back(Effect::Declare { declaration: Declaration { actor, source, ability: a } });
        }
        count
    }
    fn vampire_is_raid(&self, f: &ResolutionFrame) -> RuleResult<bool> {
        let printed = catalog::catalog().cards.iter().find(|d| d.id == f.source.card.definition && catalog::runtime_implemented(d));
        if f.actor >= self.players.len() || f.source.card.controller != f.actor || f.source.card.face_down || printed.is_none_or(|d| !catalog::has_kind(d, "character"))
            || f.source.card.owner >= self.players.len() || self.vampire_id(&f.source.card.id).is_none()
            || self.vampire_id(&f.frame_id).is_none_or(|n| n <= self.vampire_id(&f.source.card.id).unwrap_or(u64::MAX))
            || f.source.region.is_none_or(|r| r >= self.regions.len()) || f.source.play_source.is_some() || f.source.death_echo.is_some()
            || f.source.observed_death.is_some() || f.source.observed_exposure.is_some() || f.source.observed_mobility_grant.is_some()
            || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked) || f.chosen_region.is_some() || !f.already_paid.is_empty()
            || self.batch3_physical_cards().any(|c| c.id == f.source.card.id) && self.in_play(&f.source.card.id).is_none() { return Ok(false); }
        if f.ability_key == "jc038-granted-raid" { return self.validated_cross_card_frame(f); }
        if f.source.card.definition == "XG11" && f.ability_key == crate::batch4_spectral::KEY { self.validate_xg11_frame(f)?; return Ok(true); }
        let Some(a) = crate::rules::definition(&f.source.card.definition).abilities.iter().find(|a| a.key == f.ability_key && a.key.starts_with("raid-") && a.event == Some(Event::Reveal)) else { return Ok(false); };
        if !matches!(a.ops.as_slice(), [Op::DamageTarget { slot: 0, .. }]) || !same(&f.steps, &a.ops.iter().cloned().map(|op| Step { context: f.actor, op }).collect::<Vec<_>>())
            || f.targets.len() != 1 || !same(&f.targets[0].spec, &a.targets[0]) || f.targets[0].public.region != f.source.region || f.targets[0].public.kind != "character"
            || self.vampire_id(&f.targets[0].id).is_none() || f.targets[0].region_instance.is_some() || f.targets[0].public.instance_id != f.targets[0].id
            || !f.targets[0].public.valid || f.targets[0].public.status != "valid" || f.targets[0].public.invalid_reason.is_some() { return Ok(false); }
        self.mobility_immutable_identity(&f.source.card)?;
        Ok(true)
    }
    pub(crate) fn bq038_observe_raid(&mut self, f: &ResolutionFrame) {
        // Fast exit avoids a catalog/receipt scan for every ordinary frame.
        if !f.ability_key.starts_with("raid-") && f.ability_key != "jc038-granted-raid" && f.ability_key != crate::batch4_spectral::KEY { return; }
        if !self.vampire_is_raid(f).unwrap_or(false) { return; }
        let sources = self.attachments.iter().filter(|a| a.card.definition == "BQ038" && !a.card.face_down && a.card.controller == f.actor
            && a.host_id == format!("society:p{}", f.actor) && self.attachment_host_valid(a)).map(|a| self.source_snapshot(&a.card, None)).collect::<Vec<_>>();
        for source in sources {
            let mut a = definition("BQ038").unwrap().abilities.remove(1);
            a.ops = vec![Op::BQ038RaidFeed { observation: Some(Box::new(RaidObservation { turn: self.turn, raid: Box::new(f.clone()) })) }];
            self.effects.push_back(Effect::Declare { declaration: Declaration { actor: f.actor, source, ability: a } });
        }
    }
    fn vampire_feed_spec(&self, f: &ResolutionFrame) -> RuleResult<(String, usize, bool)> {
        match f.steps.as_slice() {
            [Step { op: Op::BQ027ShownFeed { .. }, .. }] => Ok((f.targets.first().ok_or("血畜展示后尚未绑定目标")?.id.clone(), 1, false)),
            [Step { op: Op::BQ031WinFeed { .. }, .. }] => Ok((f.source.card.id.clone(), 1, false)),
            [Step { op: Op::BQ027FeedTarget, .. }] => Ok((f.targets[0].id.clone(), 1, false)),
            [Step { op: Op::BQ032RevealFeed, .. }] => Ok((f.source.card.id.clone(), 2, true)),
            [Step { op: Op::BQ038RaidFeed { observation: Some(o) }, .. }] => Ok((o.raid.source.card.id.clone(), 1, false)),
            _ => Err("不是准入饮血程序".into()),
        }
    }
    pub(crate) fn vampire_feed_start(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        self.validate_vampire_frame(&f)?;
        let (recipient, amount, optional) = self.vampire_feed_spec(&f)?;
        // Project ruling 2026-10-22: BQ038 may observe any personal character's
        // raid, but an ineligible non-vampire moves no blood from either zone.
        // The original raid remains on the existing stack and continues.
        let Some(c) = self.blood_drinker(&recipient) else { return Ok(()); };
        let seat = c.controller; let title = format!("{}：饮血{amount}", catalog::card(&c.definition).name);
        let options = self.blood_options(seat); let max = amount.min(options.len());
        if max == 0 { return Ok(()); }
        self.choice(seat, BLOOD_CHOICE, title, options, if optional { 0 } else { max }, max, None,
            ChoiceResolution::Frame { frame: Box::new(f), choice: FrameChoice::VampireBlood });
        Ok(())
    }
    pub(crate) fn vampire_feed_finish(&mut self, f: &ResolutionFrame, chooser: usize, selected: &[String]) -> RuleResult<()> {
        self.validate_vampire_frame(f)?;
        let (host, amount, optional) = self.vampire_feed_spec(f)?;
        let c = self.blood_drinker(&host).ok_or("原饮血角色已失效")?;
        let options = self.blood_options(c.controller); let max = amount.min(options.len());
        if chooser != c.controller || f.cursor != 1 || !matches!(f.guard, GuardState::Accepted) || selected.len() > max || !optional && selected.len() != max
            || selected.iter().collect::<BTreeSet<_>>().len() != selected.len() || selected.iter().any(|id| !options.iter().any(|o| o.id == *id)) { return Err("饮血选择必须来自当前操控者墓地或本人秘社封印".into()); }
        // Insert the full selected batch before normal entry observations fire.
        let mut entered = vec![];
        for id in selected { entered.push(self.attach_selected_blood(chooser, &host, id)?); }
        for (definition, id) in &entered { self.enter_triggers(chooser, definition, id, false); }
        if f.ability_key == crate::batch4_vampire_contest::SHOWN {
            if let Some(crate::mystic_contest::MysticContest { phase: crate::mystic_contest::MysticPhase::After { post_frame, .. }, .. }) = &mut self.mystic_contest { *post_frame = None; }
        }
        Ok(())
    }
    pub(crate) fn bq031_discard(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        self.validate_vampire_frame(&f)?;
        let seat = f.targets[0].id.strip_prefix('p').unwrap().parse::<usize>().unwrap();
        if self.players[seat].eliminated || self.players[seat].hand.is_empty() { return Ok(()); }
        let options = self.players[seat].hand.iter().map(|c| self.option(c, seat, None, None)).collect();
        self.choice(seat, "discard", "弃 1 张手牌".into(), options, 1, 1, None,
            ChoiceResolution::Frame { frame: Box::new(f), choice: FrameChoice::Discard { seat, redraw: false } });
        Ok(())
    }
    pub(crate) fn bq036_search_start(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        self.validate_vampire_frame(&f)?;
        let inspected_ids = self.players[f.actor].deck.iter().take(6).map(|c| c.id.clone()).collect();
        let options = self.players[f.actor].deck.iter().take(6).filter(|c| searchable(c)).map(|c| self.option(c, f.actor, None, None)).collect::<Vec<_>>();
        let max = options.len();
        self.choice(f.actor, SEARCH, SEARCH_TITLE.into(), options, 0, max, None,
            ChoiceResolution::Frame { frame: Box::new(f), choice: FrameChoice::BQ036TopSix { inspected_ids } });
        Ok(())
    }
    pub(crate) fn bq036_search_finish(&mut self, f: &ResolutionFrame, inspected: &[String], selected: &[String]) -> RuleResult<()> {
        self.validate_vampire_frame(f)?;
        if f.ability_key != SEARCH || f.cursor != 1 || !matches!(f.guard, GuardState::Accepted)
            || self.players[f.actor].deck.iter().take(6).map(|c| c.id.clone()).collect::<Vec<_>>() != inspected
            || selected.iter().collect::<BTreeSet<_>>().len() != selected.len()
            || selected.iter().any(|id| !self.players[f.actor].deck.iter().take(6).any(|c| c.id == *id && searchable(c))) {
            return Err("菲尼克斯搜寻必须来自原顶六张的合法附属".into());
        }
        for id in selected {
            let i = self.players[f.actor].deck.iter().position(|c| c.id == *id).unwrap();
            let c = self.players[f.actor].deck.remove(i);
            self.note(format!("{} 展示菲尼克斯搜寻的 {}", self.players[f.actor].name, catalog::card(&c.definition).name));
            let c = self.reset_zone_card(c); self.players[f.actor].hand.push(c);
        }
        self.shuffle_player(f.actor);
        self.note(format!("{} 完成菲尼克斯顶六张搜寻并洗牌", self.players[f.actor].name));
        Ok(())
    }
    pub(crate) fn bq036_private_choice(&self, p: &Pending, viewer: usize) -> Choice {
        let mut out = p.choice.clone();
        if let ChoiceResolution::Frame { frame, choice: FrameChoice::BQ036TopSix { inspected_ids } } = &p.resolution {
            if viewer == frame.actor { out.preview_cards = inspected_ids.iter().filter_map(|id| self.players[viewer].deck.iter().find(|c| c.id == *id)).map(|c| self.card_view(c, viewer, None, None)).collect(); }
        }
        out
    }
    pub(crate) fn validate_vampire_state(&self) -> RuleResult<()> {
        self.validate_b_contest_post_state()?;
        let mut frames = BTreeSet::new(); let mut events = BTreeSet::new();
        let mut accepted_discards = BTreeSet::new();
        let mut remember = |source: &SourceSnapshot, ops: &[Op]| -> RuleResult<()> {
            let event = match ops { [Op::BQ031AttachmentDiscard { observation: Some(o) }] => Some(&o.event_id), [Op::BQ038RaidFeed { observation: Some(o) }] => Some(&o.raid.frame_id), [Op::BQ031WinFeed { contest_id: Some(id) }] => Some(id), [Op::BQ038ContestBonus { observation: Some(o) }] => Some(&o.contest_id), _ => None };
            if event.is_some_and(|e| !events.insert((source.card.id.clone(), e.clone()))) { return Err("同次结附或袭击不能重复触发同一观察者".into()); }
            Ok(())
        };
        for state in self.mystic_states() { for bonus in &state.bonuses {
            remember(&bonus.frame.source, &bonus.frame.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>())?;
        } }
        for item in &self.stack {
            let Some(f) = &item.frame else { if item.card.as_ref().is_some_and(|c| CARDS.contains(&c.definition.as_str())) { return Err("本包堆叠缺少完整帧".into()); } continue; };
            if !related_frame(f) || self.validated_cross_card_frame(f)? { continue; }
            self.validate_vampire_frame(f)?;
            if f.ability_key == ATTACH_DISCARD && !accepted_discards.insert(f.source.card.id.clone()) { return Err("雾羽千军每回合只能有一次已发动的触发".into()); }
            let play = matches!(f.ability_key.as_str(), "deploy" | "reveal" | "attach");
            if !frames.insert(f.frame_id.clone()) || item.id != f.frame_id || item.controller != f.actor || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked)
                || item.card.is_some() != play || item.target != f.targets.first().map(|t| t.id.clone()) || item.reveal != (f.ability_key == "reveal")
                || item.deploy_region != if matches!(f.ability_key.as_str(), "deploy" | "reveal") { f.source.region } else { None }
                || item.card.as_ref().is_some_and(|c| {
                    let mut expected = f.source.card.clone();
                    expected.id = c.id.clone(); expected.face_down = false;
                    expected.advancement_markers = 0; expected.named_character = None; expected.chosen_number = None; expected.chosen_subtype = None;
                    expected.time_markers = 0; expected.charge_markers = 0; expected.lock_markers = 0;
                    expected.permanent_combat_markers = 0; expected.permanent_defense_markers = 0;
                    self.vampire_id(&c.id).is_none_or(|n| n <= self.vampire_id(&f.source.card.id).unwrap() || n >= self.vampire_id(&f.frame_id).unwrap())
                        || !same(c, &expected)
                }) { return Err("本包堆叠外壳与原帧不符".into()); }
            remember(&f.source, &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>())?;
        }
        for e in &self.effects { match e {
            Effect::Declare { declaration: d } if CARDS.contains(&d.source.card.definition.as_str()) && !self.validated_cross_card_declaration(d)? => { self.validate_vampire_declaration(d)?; remember(&d.source, &d.ability.ops)?; }
            // All implemented programs either resolve atomically or pause in
            // their typed Pending. A queued frame is only transient within a
            // command, never a legitimate persisted response bypass.
            Effect::Frame { frame: f } if related_frame(f) && !self.validated_cross_card_frame(f)? => return Err("本包不能保存绕过响应堆叠的效果帧".into()),
            _ => (),
        } }
        let Some(p) = &self.pending else { return Ok(()); };
        let frame = match &p.resolution {
            ChoiceResolution::Frame { frame, .. } if related_frame(frame) && !self.validated_cross_card_frame(frame)? => Some(frame),
            _ => None,
        };
        let decl = match &p.resolution { ChoiceResolution::Declare { declaration, .. } if CARDS.contains(&declaration.source.card.definition.as_str()) && !self.validated_cross_card_declaration(declaration)? => Some(declaration), _ => None };
        if frame.is_some() || decl.is_some() {
            if p.choice.id.is_empty() || p.choice.player_id != player_id(p.seat) || p.choice.amount.is_some() || !p.choice.preview_cards.is_empty() || p.choice.description != "选择符合数量限制的选项，然后确认。" { return Err("本包待选须保留原玩家、编号与无额外载荷".into()); }
        }
        if let Some(d) = decl {
            self.validate_vampire_declaration(d)?; remember(&d.source, &d.ability.ops)?;
            let ChoiceResolution::Declare { stage, .. } = &p.resolution else { unreachable!() };
            let options = match (d.ability.targets.as_slice(), stage) {
                ([t], DeclareChoice::Target) => self.binding_options(d.actor, &d.source, t),
                ([], DeclareChoice::Accept) => vec![ChoiceOption { id: "accept".into(), label: "发动触发能力".into(), card: None }],
                _ => return Err("本包触发声明待选阶段无效".into()),
            };
            if p.seat != d.actor || p.choice.kind != "trigger" || p.choice.title != format!("{}：{}", catalog::card(&d.source.card.definition).name, d.ability.label)
                || p.choice.min != Some(0) || p.choice.max != Some(1) || p.choice.allow_decline != Some(true) || options.is_empty() || !same(&p.choice.options, &options) { return Err("本包触发待选不能更改原选项".into()); }
        }
        if let Some(f) = frame {
            self.validate_vampire_frame(f)?;
            if matches!(p.resolution, ChoiceResolution::Frame { choice: FrameChoice::MysticCommit { .. } | FrameChoice::MysticReward { .. }, .. }) {
                if !crate::batch4_vampire_contest::admitted_start(&f.source.card.definition, &f.ability_key, Self::mystic_op_bonus(f).unwrap_or(u32::MAX)) { return Err("B对抗待选不是原起始程序".into()); }
                return Ok(()); // Full shell/commit/reward checks belong to validate_mystic_state.
            }
            if !frames.insert(f.frame_id.clone()) || f.cursor != 1 || !matches!(f.guard, GuardState::Accepted) { return Err("本包结算中待选必须保留唯一原帧".into()); }
            if f.ability_key == ATTACH_DISCARD && !accepted_discards.insert(f.source.card.id.clone()) { return Err("雾羽千军每回合只能有一次已发动的触发".into()); }
            remember(&f.source, &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>())?;
            let ChoiceResolution::Frame { choice, .. } = &p.resolution else { unreachable!() };
            let (kind, title, seat, min, max, options) = match choice {
                FrameChoice::BQ027ShownTarget if f.ability_key == crate::batch4_vampire_contest::SHOWN && f.targets.is_empty() => {
                    let options = self.binding_options(f.actor, &f.source, &crate::batch4_vampire_contest::shown_target());
                    if options.is_empty() { return Err("无目标不能保留血畜后置待选".into()); }
                    (crate::batch4_vampire_contest::TARGET, crate::batch4_vampire_contest::TARGET_TITLE.into(), f.actor, 1, 1, options)
                }
                FrameChoice::VampireBlood => {
                    let (host, amount, optional) = self.vampire_feed_spec(f)?;
                    let c = self.blood_drinker(&host).ok_or("饮血待选原角色失效")?;
                    let options = self.blood_options(c.controller); let max = amount.min(options.len());
                    if max == 0 { return Err("没有鲜血不能留下饮血待选".into()); }
                    (BLOOD_CHOICE, format!("{}：饮血{amount}", catalog::card(&c.definition).name), c.controller, if optional { 0 } else { max }, max, options)
                }
                FrameChoice::BQ036TopSix { inspected_ids } if f.ability_key == SEARCH => {
                    if *inspected_ids != self.players[f.actor].deck.iter().take(6).map(|c| c.id.clone()).collect::<Vec<_>>() { return Err("菲尼克斯私有顶六张已被改写".into()); }
                    let options = self.players[f.actor].deck.iter().take(6).filter(|c| searchable(c)).map(|c| self.option(c, f.actor, None, None)).collect::<Vec<_>>();
                    (SEARCH, SEARCH_TITLE.into(), f.actor, 0, options.len(), options)
                }
                FrameChoice::Discard { seat, redraw: false } if f.ability_key == ATTACH_DISCARD && f.targets[0].id == player_id(*seat) => {
                    let options = self.players[*seat].hand.iter().map(|c| self.option(c, *seat, None, None)).collect::<Vec<_>>();
                    if options.is_empty() { return Err("空手牌不能留下弃牌待选".into()); }
                    ("discard", "弃 1 张手牌".into(), *seat, 1, 1, options)
                }
                _ => return Err("本包不能移植其他待选程序".into()),
            };
            if p.seat != seat || self.players[seat].eliminated || p.choice.kind != kind || p.choice.title != title || p.choice.min != Some(min) || p.choice.max != Some(max) || p.choice.allow_decline != Some(min == 0) || !same(&p.choice.options, &options) { return Err("本包结算待选与原选项或当前操控者不符".into()); }
        } else if matches!(p.choice.kind.as_str(), BLOOD_CHOICE | SEARCH) || matches!(p.resolution, ChoiceResolution::Frame { choice: FrameChoice::BQ027ShownTarget | FrameChoice::VampireBlood | FrameChoice::BQ036TopSix { .. }, .. }) { return Err("本包待选不能来自其他来源程序".into()); }
        Ok(())
    }
}
