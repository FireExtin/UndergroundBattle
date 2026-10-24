//! WM025/WM024 adapters. Payments, raid guards, death batches and contests all
//! remain in the existing interpreter. Public admission stays closed.
use crate::{catalog, engine::RuleResult, model::*, mystic_contest::ContestOutcome, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub(crate) const CARDS: &[&str] = &["WM025", "WM024"];
pub(crate) const WIN: &str = "import20-wm025-mystic-win";
pub(crate) const RAID: &str = "import20-wm024-reveal-raid";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssassinRaid {
    pub event_id: String,
    pub turn: u32,
    // Additive granted raids are observed at reveal. The printed X itself is
    // the enemy exposure count when this damage effect resolves.
    pub grants: Vec<Card>,
}
fn same<T: Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
pub(crate) fn definition(id: &str) -> Option<Definition> {
    Some(with_abilities(vec![match id {
        "WM025" => ability(WIN, "赢得隐秘对抗：各有本方吸血鬼的地区放置势力1", Timing::Fast,
            vec![], vec![], vec![Op::WM025MysticInfluence { contest_id: None }], Some(Event::MysticContestWon)),
        "WM024" => ability(RAID, "袭击X：按结算时敌方暴露；杀死声望角色则抓1", Timing::Fast,
            vec![], vec![target(Zone::Board, EntityKind::Character, Relation::Any, Range::SourceRegion)],
            vec![Op::WM024RevealRaid { observation: None }], Some(Event::Reveal)),
        _ => return None,
    }]))
}
pub(crate) fn owns(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::WM025MysticInfluence { .. } | Op::WM024RevealRaid { .. } => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => owns(body),
        Op::IfTargetExhausted { exhausted, ready, .. } =>
            owns(std::slice::from_ref(exhausted)) || owns(std::slice::from_ref(ready)),
        _ => false,
    })
}
pub(crate) fn related(a: &AbilitySpec) -> bool {
    matches!(a.key.as_str(), WIN | RAID) || owns(&a.ops) || a.modes.iter().any(|m| owns(&m.ops))
}
pub(crate) fn frame_related(f: &ResolutionFrame) -> bool {
    CARDS.contains(&f.source.card.definition.as_str()) || matches!(f.ability_key.as_str(), WIN | RAID)
        || f.steps.iter().any(|s| owns(std::slice::from_ref(&s.op)))
}
fn canonical(id: &str, a: &AbilitySpec) -> Option<AbilitySpec> {
    let mut expected = definition(id)?.abilities.remove(0);
    match (id, a.ops.as_slice()) {
        ("WM025", [Op::WM025MysticInfluence { contest_id }]) =>
            expected.ops = vec![Op::WM025MysticInfluence { contest_id: contest_id.clone() }],
        ("WM024", [Op::WM024RevealRaid { observation }]) =>
            expected.ops = vec![Op::WM024RevealRaid { observation: observation.clone() }],
        _ => return None,
    }
    Some(expected)
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if canonical(id, a).is_none_or(|b| !same(a, &b)) {
        return Err("吸血鬼新卡须保留完整原卡触发、目标与程序".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if CARDS.contains(&id) && !same(d, &definition(id).unwrap()) {
        return Err("吸血鬼新卡完整定义不可增删或变成无条件屏障".into());
    }
    Ok(())
}
pub(crate) fn validate_action(a: &Action) -> RuleResult<()> {
    let expected = match a.kind.as_str() {
        "deploy" | "conceal" => Action { card_id: a.card_id.clone(), region: a.region, ..Action::new(&a.kind) },
        "reveal" => Action { card_id: a.card_id.clone(), ..Action::new("reveal") },
        _ => return Err("本卡只有普通派遣、秘密派遣与现身操作".into()),
    };
    if *a != expected { return Err("本卡不可提前指定触发目标或夹带费用、选择载荷".into()); }
    Ok(())
}
pub(crate) fn validate_choice_action(p: &Pending, a: &Action) -> RuleResult<()> {
    if matches!(&p.resolution, ChoiceResolution::Declare { declaration: d, .. }
        if CARDS.contains(&d.source.card.definition.as_str()) || related(&d.ability))
        && *a != (Action { choice_id: a.choice_id.clone(), selected: a.selected.clone(), ..Action::new("choose") }) {
        return Err("吸血鬼触发待选只接受当前原选项或放弃".into());
    }
    Ok(())
}
impl Game {
    pub(crate) fn wm025_barrier(&self, c: &Card) -> bool {
        c.definition == "WM025" && self.players[c.controller].society_zone.secrecy > 0
    }
    pub(crate) fn wm025_reveal_reduction(&self, actor: usize, c: &Card) -> u32 {
        let mut revealed = c.clone(); revealed.face_down = false;
        if c.controller != actor || catalog::card(&c.definition).kind != "character"
            || !self.current_subtypes(&revealed).iter().any(|s| s == "吸血鬼") { return 0; }
        let sources = self.in_play_cards().into_iter().filter(|(r, source)| r.is_some()
            && source.definition == "WM025" && !source.face_down && source.controller == actor
            && !self.players[actor].eliminated).count() as u32;
        self.players[actor].society_zone.secrecy.saturating_mul(sources)
    }
    pub(crate) fn wm024_enemy_exposure(&self, actor: usize) -> u32 {
        self.players.iter().filter(|p| !p.eliminated && self.team(p.seat) != self.team(actor))
            .fold(0u32, |sum, p| sum.saturating_add(p.society_zone.exposure))
    }
    pub(crate) fn wm024_cost_reduction(&self, actor: usize, c: &Card) -> u32 {
        if c.definition == "WM024" { self.wm024_enemy_exposure(actor).saturating_mul(2) } else { 0 }
    }
    pub(crate) fn wm025_observe_win(&mut self, o: &ContestOutcome) {
        let Some(team) = o.winner_team else { return; };
        let sources = self.in_play_cards().into_iter().filter(|(r, c)| r.is_some() && c.definition == "WM025"
            && !c.face_down && !self.players[c.controller].eliminated
            && self.team(c.controller) == team && o.participants.contains(&c.controller))
            .map(|(r, c)| self.source_snapshot(c, r)).collect::<Vec<_>>();
        for source in sources {
            let mut a = definition("WM025").unwrap().abilities.remove(0);
            a.ops = vec![Op::WM025MysticInfluence { contest_id: Some(o.contest_id.clone()) }];
            self.effects.push_back(Effect::Declare { declaration: Declaration { actor: source.card.controller, source, ability: a } });
        }
    }
    pub(crate) fn wm025_influence(&mut self, actor: usize) {
        let regions = (0..self.regions.len()).filter(|r| self.region_live(*r)
            && self.region_characters(*r).any(|c| c.controller == actor && !c.face_down
                && self.current_subtypes(c).iter().any(|s| s == "吸血鬼"))).collect::<Vec<_>>();
        for region in regions { self.place_influence(actor, region, 1); }
    }
    pub(crate) fn wm024_reveal_ability(&mut self, source: &SourceSnapshot) -> AbilitySpec {
        let grants = self.regions.iter().flat_map(|r| &r.cards).filter(|c| c.definition == "JC038"
            && !c.face_down && c.id != source.card.id && self.team(c.controller) == self.team(source.card.controller))
            .cloned().collect();
        let observation = AssassinRaid { event_id: self.id(), turn: self.turn, grants };
        let mut a = definition("WM024").unwrap().abilities.remove(0);
        a.ops = vec![Op::WM024RevealRaid { observation: Some(observation) }]; a
    }
    pub(crate) fn wm024_raid_resolve(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        let [Step { op: Op::WM024RevealRaid { observation: Some(o) }, .. }] = f.steps.as_slice()
            else { return Err("杀手袭击缺少原现身观察".into()); };
        let amount = self.wm024_enemy_exposure(f.actor).saturating_add(o.grants.len() as u32);
        let deaths = self.damage_with_receipts([(f.targets[0].id.clone(), amount)].into())?;
        if deaths.iter().any(|d| d.was_renown) { self.draw(f.actor, 1); }
        Ok(())
    }
    fn import20_vampire_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        let c = &s.card;
        if !CARDS.contains(&c.definition.as_str()) || actor >= self.players.len() || c.owner >= self.players.len()
            || c.controller != actor || !self.mystic_instance_valid(&c.id)
            || s.region.is_none_or(|r| r >= self.regions.len())
            || s.source_region_instance.as_ref().is_none_or(|id| !self.mystic_instance_valid(id) || *id == c.id)
            || s.attachment_host_instance.is_some() || s.death_echo.is_some() || s.observed_death.is_some()
            || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some()
            || s.region.zip(s.source_region_instance.as_ref()).is_some_and(|(r, id)|
                self.regions.iter().enumerate().any(|(other, live)| live.card.id == *id && r != other)) {
            return Err("吸血鬼新卡须保留原来源、操控者与原地区身份".into());
        }
        self.mobility_immutable_identity(c)
    }
    fn import20_vampire_expected(&self, s: &SourceSnapshot, key: &str, ops: &[Op]) -> RuleResult<AbilitySpec> {
        if matches!(key, "renown" | "jc089-combat-glory") {
            let region = s.source_region_instance.as_deref().ok_or("奖励缺少原地区")?;
            return Ok(if key == "renown" { crate::renown::renown_ability(region) }
                else { crate::rules::jc089_combat_glory_ability(region) });
        }
        let mut a = definition(&s.card.definition).ok_or("非本切片来源")?.abilities.remove(0);
        if key != a.key { return Err("吸血鬼新卡未登记能力".into()); }
        a.ops = ops.to_vec();
        if canonical(&s.card.definition, &a).is_none_or(|expected| !same(&a, &expected)) {
            return Err("吸血鬼新卡程序不完整".into());
        }
        match a.ops.as_slice() {
            [Op::WM025MysticInfluence { contest_id: Some(id) }] => {
                let o = self.mystic_history.iter().find(|o| o.contest_id == *id).ok_or("阴影之君缺少真实获胜结果")?;
                if o.turn != self.turn || !o.participants.contains(&s.card.controller)
                    || o.winner_team != Some(self.team(s.card.controller)) { return Err("阴影之君须为本次实际获胜玩家".into()); }
                self.validate_mystic_outcome(o)?;
            }
            [Op::WM024RevealRaid { observation: Some(o) }] => {
                let mut seen = BTreeSet::new();
                if o.turn != self.turn || !self.mystic_instance_valid(&o.event_id) || o.event_id == s.card.id
                    || o.grants.iter().any(|g| g.definition != "JC038" || g.face_down || g.owner >= self.players.len()
                        || g.controller >= self.players.len() || !self.mystic_instance_valid(&g.id)
                        || g.id == s.card.id || !seen.insert(&g.id) || self.team(g.controller) != self.team(s.card.controller)) {
                    return Err("杀手袭击须保留实际现身及不同友方授予来源".into());
                }
                for g in &o.grants { self.mobility_immutable_identity(g)?; }
            }
            _ => return Err("吸血鬼触发缺少真实事件观察".into()),
        }
        Ok(a)
    }
    pub(crate) fn validate_import20_vampire_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if self.validated_cross_card_declaration(d)? { return Ok(()); }
        self.import20_vampire_source(d.actor, &d.source)?;
        let a = self.import20_vampire_expected(&d.source, &d.ability.key, &d.ability.ops)?;
        if d.source.card.face_down || d.source.play_source.is_some() || !same(&d.ability, &a) {
            return Err("吸血鬼触发须保留正面来源及完整原声明".into());
        }
        Ok(())
    }
    pub(crate) fn validate_import20_vampire_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if self.validated_cross_card_frame(f)? { return Ok(()); }
        self.import20_vampire_source(f.actor, &f.source)?;
        if !self.mystic_instance_number(&f.source.card.id).zip(self.mystic_instance_number(&f.frame_id))
            .is_some_and(|(source, frame)| source < frame)
            || f.cursor > f.steps.len() || f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked)
            || f.cursor > 0 && !matches!(f.guard, GuardState::Accepted) { return Err("吸血鬼帧身份或阶段无效".into()); }
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            if !f.steps.is_empty() || !f.targets.is_empty() || f.chosen_region != f.source.region
                || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if ids.len() <= catalog::card(&f.source.card.definition).cost as usize
                    && ids.iter().all(|id| self.mystic_instance_valid(id) && *id != f.source.card.id)
                    && ids.iter().collect::<BTreeSet<_>>().len() == ids.len())
                || f.ability_key == "deploy" && (f.source.card.face_down || !f.source.valid_character_play())
                || f.ability_key == "reveal" && (!f.source.card.face_down || f.source.play_source.is_some()) {
                return Err("吸血鬼派遣/现身须保留完整支付与原地区".into());
            }
            return Ok(());
        }
        let ops = f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>();
        let a = self.import20_vampire_expected(&f.source, &f.ability_key, &ops)?;
        self.validate_import20_vampire_declaration(&Declaration { actor: f.actor, source: f.source.clone(), ability: a.clone() })?;
        if f.steps.iter().any(|s| s.context != f.actor) || !same(&ops, &a.ops) || !f.already_paid.is_empty()
            || f.chosen_region.is_some() || f.targets.len() != a.targets.len()
            || f.targets.iter().zip(&a.targets).any(|(t, expected)| !same(&t.spec, expected)) {
            return Err("吸血鬼触发程序、目标或上下文不完整".into());
        }
        Ok(())
    }
    pub(crate) fn validate_import20_vampire_state(&self) -> RuleResult<()> {
        let mut events = BTreeSet::new();
        let mut remember = |s: &SourceSnapshot, ops: &[Op]| -> RuleResult<()> {
            let event = match ops {
                [Op::WM025MysticInfluence { contest_id: Some(id) }] => Some(id),
                [Op::WM024RevealRaid { observation: Some(o) }] => Some(&o.event_id),
                _ => None,
            };
            if event.is_some_and(|id| !events.insert((s.card.id.clone(), id.clone()))) {
                return Err("同一吸血鬼不能重复保存同一次触发".into());
            }
            Ok(())
        };
        for item in &self.stack { if let Some(f) = &item.frame { if frame_related(f) && !self.validated_cross_card_frame(f)? {
            self.validate_import20_vampire_frame(f)?;
            remember(&f.source, &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>())?;
            let play = matches!(f.ability_key.as_str(), "deploy" | "reveal");
            if item.id != f.frame_id || item.controller != f.actor || f.cursor != 0
                || item.card.is_some() != play || item.reveal != (f.ability_key == "reveal")
                || item.deploy_region != (if play { f.source.region } else { None })
                || item.target != f.targets.first().map(|t| t.id.clone()) {
                return Err("吸血鬼堆叠外壳或阶段不符".into());
            }
        } } }
        for effect in &self.effects { match effect {
            Effect::Declare { declaration: d } if CARDS.contains(&d.source.card.definition.as_str()) || related(&d.ability) => {
                self.validate_import20_vampire_declaration(d)?; remember(&d.source, &d.ability.ops)?;
            }
            Effect::Frame { frame: f } if frame_related(f) && !self.validated_cross_card_frame(f)? =>
                return Err("吸血鬼触发不可绕过响应堆叠".into()),
            _ => {},
        } }
        if let Some(p) = &self.pending { match &p.resolution {
            ChoiceResolution::Declare { declaration: d, stage } if CARDS.contains(&d.source.card.definition.as_str()) || related(&d.ability) => {
                if self.validated_cross_card_declaration(d)? { return Ok(()); }
                self.validate_import20_vampire_declaration(d)?; remember(&d.source, &d.ability.ops)?;
                let (options, expected_stage) = if let Some(t) = d.ability.targets.first() {
                    (self.binding_options(d.actor, &d.source, t), DeclareChoice::Target)
                } else { (vec![ChoiceOption { id: "accept".into(), label: "发动触发能力".into(), card: None }], DeclareChoice::Accept) };
                let expected = Choice { id: p.choice.id.clone(), kind: "trigger".into(), title: format!("{}：{}", catalog::card(&d.source.card.definition).name, d.ability.label),
                    description: "选择符合数量限制的选项，然后确认。".into(), player_id: player_id(d.actor), options, preview_cards: vec![],
                    min: Some(0), max: Some(1), amount: None, allow_decline: Some(true) };
                if !same(stage, &expected_stage) || p.seat != d.actor || p.choice.id.is_empty() || expected.options.is_empty() || !same(&p.choice, &expected) {
                    return Err("吸血鬼待选须保留原目标及可选次数".into());
                }
            }
            ChoiceResolution::Frame { frame: f, .. } if frame_related(f) && !self.validated_cross_card_frame(f)? =>
                return Err("本切片触发没有额外帧内选择".into()),
            _ => {},
        } }
        Ok(())
    }
}
