//! Frozen combat participation and whole discard operations for JC034/JC040.
use crate::{batch20, catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CombatWinObservation {
    pub event_id: String,
    pub turn: u32,
    pub region: usize,
    pub region_instance: String,
    pub participant: Card,
    pub vampire: bool,
    pub combat: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscardObservation {
    pub event_id: String,
    pub turn: u32,
    pub holder: usize,
    pub cards: Vec<Card>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VampireRevealObservation {
    pub event_id: String,
    pub turn: u32,
    pub card: Card,
    pub vampire: bool,
}
fn same<T: Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
fn observation_id(ops: &[Op]) -> Option<&str> {
    match ops {
        [Op::BQ039BattleDiscard { observation: Some(o) }] => Some(&o.event_id),
        [Op::JC034CombatDiscard { observation: Some(o) }] => Some(&o.event_id),
        [Op::JC040RepressDiscard { observation: Some(o) }] => Some(&o.event_id),
        [Op::BQ080HideForAttachment { observation: Some(o) }] => Some(&o.event_id),
        [Op::JZ29ReturnOnVampireReveal { observation: Some(o) }] => Some(&o.event_id),
        _ => None,
    }
}
impl Game {
    pub(crate) fn observe_jz29_vampire_reveal(&mut self, revealed: &SourceSnapshot) {
        let c = &revealed.card;
        if c.face_down || catalog::card(&c.definition).kind != "character" || !self.current_subtypes(c).iter().any(|s| s == "吸血鬼") { return; }
        let sources = self.attachments.iter().filter(|a| a.card.definition == "JZ29" && !a.card.face_down
            && a.card.controller == c.controller && self.attachment_host_valid(a))
            .map(|a| self.source_snapshot(&a.card, None)).collect::<Vec<_>>();
        if sources.is_empty() { return; }
        let observation = VampireRevealObservation { event_id: self.id(), turn: self.turn, card: c.clone(), vampire: true };
        for source in sources {
            let mut ability = batch20::definition("JZ29").unwrap().abilities[2].clone();
            ability.ops = vec![Op::JZ29ReturnOnVampireReveal { observation: Some(observation.clone()) }];
            self.effects.push_back(Effect::Declare { declaration: Declaration { actor: c.controller, source, ability } });
        }
    }
    pub(crate) fn jc034_combat_observers(&mut self, team: usize, region: usize) -> Vec<Declaration> {
        let sources = self.attachments.iter().filter(|a| a.card.definition == "JC034" && !a.card.face_down
            && self.team(a.card.controller) == team && self.attachment_host_valid(a)).filter_map(|a| {
                let c = self.regions[region].cards.iter().find(|c| !c.face_down && c.controller == a.card.controller
                    && catalog::card(&c.definition).kind == "character" && self.icons(c, region).combat > 0
                    && self.current_subtypes(c).iter().any(|s| s == "吸血鬼"))?;
                Some((self.source_snapshot(&a.card, None), c.clone(), self.icons(c, region).combat))
            }).collect::<Vec<_>>();
        if sources.is_empty() { return vec![]; }
        let event_id = self.id();
        sources.into_iter().map(|(source, participant, combat)| {
            let mut ability = batch20::definition("JC034").unwrap().abilities[1].clone();
            ability.ops = vec![Op::JC034CombatDiscard { observation: Some(CombatWinObservation {
                event_id: event_id.clone(), turn: self.turn, region, region_instance: self.regions[region].card.id.clone(),
                participant, vampire: true, combat,
            }) }];
            Declaration { actor: source.card.controller, source, ability }
        }).collect()
    }
    pub(crate) fn capture_discard_observers(&self, holder: usize) -> Vec<SourceSnapshot> {
        self.regions.iter().enumerate().flat_map(|(r, reg)| reg.cards.iter().map(move |c| (r,c)))
            .filter(|(_,c)| matches!(c.definition.as_str(), "JC040" | "JZ28") && !c.face_down && self.team(c.controller) != self.team(holder))
            .map(|(r,c)| self.source_snapshot(c, Some(r))).collect()
    }
    pub(crate) fn emit_discard_observed(&mut self, holder: usize, cards: Vec<Card>, observers: Vec<SourceSnapshot>) {
        if cards.is_empty() || observers.is_empty() { return; }
        let observation = DiscardObservation { event_id: self.id(), turn: self.turn, holder, cards };
        for source in observers {
            let mut ability = if source.card.definition == "JZ28" {
                crate::next20_blue_gray::definition("JZ28").unwrap().abilities[1].clone()
            } else { batch20::definition("JC040").unwrap().abilities[0].clone() };
            ability.ops = if source.card.definition == "JZ28" { vec![Op::JZ28DiscardSearch { observation: Some(observation.clone()) }] }
                else { vec![Op::JC040RepressDiscard { observation: Some(observation.clone()) }] };
            self.effects.push_back(Effect::Declare { declaration: Declaration { actor: source.card.controller, source, ability } });
        }
    }
    pub(crate) fn validate_batch20_observation(&self, actor: usize, s: &SourceSnapshot, steps: &[Step]) -> RuleResult<()> {
        match steps {
            [Step { context, op: Op::JZ29ReturnOnVampireReveal { observation: Some(o) } }] => {
                let c = &o.card;
                if *context != actor || s.card.definition != "JZ29" || s.region.is_some()
                    || s.attachment_host_instance.as_deref() != Some(&format!("society:p{actor}"))
                    || o.event_id.is_empty() || o.turn != self.turn || !o.vampire || c.face_down
                    || c.id.is_empty() || c.controller != actor || c.owner >= self.players.len()
                    || catalog::card(&c.definition).kind != "character"
                    || !(catalog::card(&c.definition).subtypes.iter().any(|s| s == "吸血鬼" || s == "人类") || c.definition == "JC030") {
                    return Err("锐齿颈环缺少本方吸血鬼实际现身观察".into());
                }
                self.mobility_immutable_identity(c)?;
            }
            [Step { context, op: Op::JC034CombatDiscard { observation: Some(o) } }] => {
                let c = &o.participant;
                if *context != actor || s.card.definition != "JC034" || s.attachment_host_instance.as_deref() != Some(&format!("society:p{actor}"))
                    || o.event_id.is_empty() || o.turn != self.turn || o.region >= self.regions.len() || o.region_instance.is_empty()
                    || !o.vampire || o.combat == 0 || c.face_down || c.exhausted || c.controller != actor || c.owner >= self.players.len()
                    || c.id.is_empty() || catalog::card(&c.definition).kind != "character"
                    || !(catalog::card(&c.definition).subtypes.iter().any(|s| s == "吸血鬼" || s == "人类") || c.definition == "JC030") {
                    return Err("心兽之道缺少本方吸血鬼实际战斗贡献观察".into());
                }
                self.mobility_immutable_identity(c)?;
            }
            [Step { context, op: Op::JC040RepressDiscard { observation: Some(o) } }] => {
                let mut seen = BTreeSet::new();
                if *context != actor || s.card.definition != "JC040" || o.event_id.is_empty() || o.turn != self.turn
                    || o.holder >= self.players.len() || self.team(o.holder) == self.team(actor) || o.cards.is_empty()
                    || o.cards.iter().any(|c| c.id.is_empty() || !seen.insert(&c.id) || c.owner >= self.players.len()
                        || c.controller >= self.players.len() || c.face_down || !catalog::catalog().cards.iter().any(|d| d.id == c.definition)) {
                    return Err("安普莎缺少单次敌方真实弃牌观察".into());
                }
                for c in &o.cards { self.mobility_immutable_identity(c)?; }
            }
            _ => return Err("本批观察程序身份或事件收据无效".into()),
        }
        Ok(())
    }
    pub(crate) fn batch20_combat_discard_start(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        let seat = f.targets[0].id.strip_prefix('p').and_then(|s| s.parse::<usize>().ok()).filter(|s| *s < self.players.len()).ok_or("弃牌玩家无效")?;
        if self.players[seat].eliminated || self.players[seat].hand.is_empty() { return Ok(()); }
        let options = self.players[seat].hand.iter().map(|c| self.option(c, seat, None, None)).collect();
        self.choice(seat, "discard", "弃 1 张手牌".into(), options, 1, 1, None,
            ChoiceResolution::Frame { frame: Box::new(f), choice: FrameChoice::Discard { seat, redraw: false } });
        Ok(())
    }
    fn batch20_repress_amount(f: &ResolutionFrame) -> RuleResult<usize> {
        match f.steps.first().map(|s| &s.op) {
            Some(Op::JC040RepressDiscard { observation: Some(o) }) => Ok(o.cards.len()),
            Some(Op::JC065SacrificeHidden) if f.ability_key == "batch20-jc065-interrogate" => Ok(1),
            _ => Err("未知遏制数量来源".into()),
        }
    }
    pub(crate) fn batch20_repress_regions(&self, seat: usize) -> Vec<(usize, String, u32)> {
        self.regions.iter().enumerate().filter(|(_,r)| !r.vacant && r.influence[self.team(seat)] > 0)
            .map(|(r,reg)| (r, reg.card.id.clone(), reg.influence[self.team(seat)])).collect()
    }
    pub(crate) fn batch20_repress_options(&self, regions: &[(usize,String,u32)], amount: usize) -> Vec<ChoiceOption> {
        regions.iter().flat_map(|(r,_,count)| (0..(*count as usize).min(amount)).map(move |marker| ChoiceOption {
            id: format!("region:{r}:marker:{marker}"),
            label: format!("地区{}：{} · 本方势力标志{}（共{}）", r + 1, catalog::card(&self.regions[*r].card.definition).name, marker + 1, count), card: None,
        })).collect()
    }
    pub(crate) fn batch20_repress_start(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        let seat = f.targets[if f.ability_key == "batch20-jc065-interrogate" { 2 } else { 0 }].id.strip_prefix('p').and_then(|s| s.parse::<usize>().ok()).filter(|s| *s < self.players.len()).ok_or("遏制目标玩家无效")?;
        if self.players[seat].eliminated { return Ok(()); }
        let regions = self.batch20_repress_regions(seat);
        let n = Self::batch20_repress_amount(&f)?.min(regions.iter().map(|(_,_,n)| *n as usize).sum());
        if n == 0 { return Ok(()); }
        let options = self.batch20_repress_options(&regions, n);
        self.choice(seat, "influence", format!("遏制{n}：选择并移除本方势力标志"), options, n, n, Some(n as u32),
            ChoiceResolution::Frame { frame: Box::new(f), choice: FrameChoice::Batch20Repress { seat, regions } });
        Ok(())
    }
    pub(crate) fn validate_batch20_repress_pending(&self, p: &Pending, f: &ResolutionFrame, seat: usize, regions: &[(usize,String,u32)]) -> RuleResult<()> {
        self.validate_batch20_frame(f)?;
        if seat >= self.players.len() { return Err("遏制选择者无效".into()); }
        let n = Self::batch20_repress_amount(f)?.min(regions.iter().map(|(_,_,n)| *n as usize).sum());
        if !matches!(f.ability_key.as_str(), "batch20-jc040-repress" | "batch20-jc065-interrogate")
            || f.cursor != (if f.ability_key == "batch20-jc065-interrogate" { 2 } else { 1 }) || !matches!(f.guard, GuardState::Accepted)
            || f.targets[if f.ability_key == "batch20-jc065-interrogate" { 2 } else { 0 }].id != player_id(seat) || p.seat != seat || p.choice.player_id != player_id(seat)
            || p.choice.kind != "influence" || p.choice.title != format!("遏制{n}：选择并移除本方势力标志")
            || p.choice.min != Some(n) || p.choice.max != Some(n) || p.choice.amount != Some(n as u32) || p.choice.allow_decline != Some(false)
            || n == 0 || !same(&regions.to_vec(), &self.batch20_repress_regions(seat))
            || !same(&p.choice.options, &self.batch20_repress_options(regions, n)) {
            return Err("遏制数量、原地区、势力标志或待选载荷无效".into());
        }
        Ok(())
    }
    pub(crate) fn batch20_repress_complete(&mut self, f: &ResolutionFrame, chooser: usize, seat: usize, regions: &[(usize,String,u32)], selected: &[String]) -> RuleResult<()> {
        self.validate_batch20_frame(f)?;
        let n = Self::batch20_repress_amount(f)?.min(regions.iter().map(|(_,_,n)| *n as usize).sum());
        let options = self.batch20_repress_options(regions, n);
        if chooser != seat || selected.len() != n || selected.iter().collect::<BTreeSet<_>>().len() != n
            || selected.iter().any(|id| !options.iter().any(|o| &o.id == id)) { return Err("遏制须选择指定数量的不同标志".into()); }
        let team = self.team(seat);
        for (r, uid, _) in regions {
            let count = selected.iter().filter(|id| id.starts_with(&format!("region:{r}:marker:"))).count() as u32;
            let _reg = self.regions.get_mut(*r).filter(|r| !r.vacant && r.card.id == *uid && r.influence[team] >= count).ok_or("遏制原地区已失效")?;
            self.remove_influence_recorded(*r,team,count);
        }
        Ok(())
    }
    pub(crate) fn validate_batch20_observation_duplicates(&self) -> RuleResult<()> {
        let mut seen = BTreeSet::new();
        let mut check = |s: &SourceSnapshot, ops: &[Op]| -> RuleResult<()> {
            if let Some(id) = observation_id(ops) {
                if !seen.insert((s.card.id.clone(), id.to_owned())) { return Err("同一来源重复保存同一观察事件".into()); }
            }
            Ok(())
        };
        for s in &self.stack { if let Some(f) = &s.frame { check(&f.source, &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>())?; } }
        for e in &self.effects { match e {
            Effect::Declare { declaration:d } => check(&d.source, &d.ability.ops)?,
            Effect::Frame { frame:f } => check(&f.source, &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>())?,
            _ => {},
        } }
        if let Some(p) = &self.pending { match &p.resolution {
            ChoiceResolution::Declare { declaration:d, .. } => check(&d.source, &d.ability.ops)?,
            ChoiceResolution::Frame { frame:f, .. } => check(&f.source, &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>())?,
            _ => {},
        } }
        Ok(())
    }
}
