//! Three closed producers reuse real sealing, graveyard entry, preparation,
//! source payment and the existing four-seat RNG/state stream.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use std::collections::BTreeSet;

fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
pub(crate) fn validate_action(id: &str, a: &Action) -> RuleResult<()> {
    let expected = match (id, a.kind.as_str()) {
        ("BQ089", "play") => Action { card_id: a.card_id.clone(), ability_id: a.ability_id.clone(),
            target_id: a.target_id.clone(), region: a.region, ..Action::new("play") },
        ("XQ26", "play") => Action { card_id: a.card_id.clone(), ability_id: a.ability_id.clone(), target_id: a.target_id.clone(), ..Action::new("play") },
        ("XG28" | "JC064", "play") => Action { card_id: a.card_id.clone(), ability_id: a.ability_id.clone(),
            region: a.region, ..Action::new("play") },
        ("XG28", "activate") => Action { card_id: a.card_id.clone(), ability_id: a.ability_id.clone(),
            target_id: a.target_id.clone(), ..Action::new("activate") },
        _ => return Err("第三批回收牌操作种类不符".into()),
    };
    if *a != expected { return Err("第三批回收声明含无关载荷".into()); }
    Ok(())
}
pub(crate) fn validate_choice_action(p: &Pending, a: &Action) -> RuleResult<()> {
    if matches!(&p.resolution, ChoiceResolution::Declare { declaration, .. } if matches!(declaration.source.card.definition.as_str(),"JZ25"|"BQ029"|"BQ026"|"JZ26"|"CQ07"|"XG14"))
        && *a != (Action { choice_id: a.choice_id.clone(), selected: a.selected.clone(), ..Action::new("choose") }) {
        return Err("第三批声明只接受原待选选择，不能附加其他载荷".into());
    }
    Ok(())
}

impl Game {
    pub(crate) fn validate_jz25_origin(&self, source: &SourceSnapshot) -> RuleResult<()> {
        let c = &source.card;
        if c.id.strip_prefix('i').and_then(|s| s.parse::<u64>().ok()).is_none_or(|n| n == 0 || n > self.sequence)
            || self.players.iter().any(|p| p.hand.iter().chain(&p.deck).chain(&p.graveyard).chain(&p.assets).chain(&p.score_cards).chain(p.society_zone.card.iter()).any(|other| other.id == c.id))
            || self.sealed_cards.iter().any(|s| s.card.id == c.id)
            || self.attachments.iter().any(|a| a.card.id == c.id)
            || self.world.iter().any(|other| other.id == c.id)
            || self.stack.iter().any(|s| s.card.as_ref().is_some_and(|other| other.id == c.id)) {
            return Err("猎杀者进场来源不能来自手牌、其他区域或未分配UID".into());
        }
        Ok(())
    }
    pub(crate) fn batch3_own_grave_character(&self, actor: usize, source: &SourceSnapshot, c: &Card) -> bool {
        matches!(source.card.definition.as_str(), "XG28" | "BQ089" | "BQ087")
            && self.players.get(actor).is_some_and(|p| p.graveyard.iter().any(|live| live == c))
            && c.owner == actor && !c.face_down
            && catalog::card(&c.definition).kind == "character"
            && (source.card.definition != "BQ089" || catalog::card(&c.definition).cost <=
                if self.players[actor].society_zone.secrecy > 0 { 5 } else { 3 })
    }
    pub(crate) fn bq089_action_candidates(&self, actor: usize, c: &Card) -> Vec<(Action, String)> {
        let d = crate::batch3::definition("BQ089").unwrap(); let a = &d.abilities[0];
        let source = self.source_snapshot(c, None);
        let mut out = vec![];
        for target in self.binding_options(actor, &source, &a.targets[0]) {
            for region in self.binding_options(actor, &source, &a.targets[1]) {
                out.push((Action { card_id: Some(c.id.clone()), ability_id: Some(a.key.clone()),
                    target_id: Some(target.id.clone()), region: region.id.strip_prefix("region:").and_then(|r| r.parse().ok()),
                    ..Action::new("play") }, format!("生化改造手术：{} → {}", target.label, region.label)));
            }
        }
        out
    }
    pub(crate) fn validate_batch3_grave_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        let c = &f.source.card;
        let d = crate::batch3::definition(&c.definition).ok_or("非准入回收来源")?;
        let mut a = d.abilities.iter().find(|a| a.key == f.ability_key).cloned().ok_or("回收程序不属于原来源")?;
        if a.key=="batch3-xq26-wanted" {
            let ops=f.steps.iter().map(|s|s.op.clone()).collect::<Vec<_>>();self.xq26_observation(f.actor,&f.source,&ops)?;a.ops=ops;
        }
        // Attachment commitment runs its target guard before this empty frame
        // reaches the interpreter. Stack containers still require Unchecked.
        let attachment_commit = a.play_only && catalog::card(&c.definition).kind == "attachment" && a.ops.is_empty();
        if f.actor >= self.players.len() || c.owner >= self.players.len() || c.controller != f.actor
            || c.id.is_empty() || c.face_down || f.source.play_source.is_some()
            || f.source.observed_death.is_some() || f.source.observed_mobility_grant.is_some() || f.source.observed_exposure.is_some()
            || f.frame_id.is_empty() || f.cursor > a.ops.len()
            || (f.cursor == 0 && !attachment_commit && !matches!(f.guard, GuardState::Unchecked))
            || (f.cursor > 0 && !matches!(f.guard, GuardState::Accepted))
            || !same(&f.steps, &a.ops.iter().map(|op| Step { context: f.actor, op: op.clone() }).collect::<Vec<_>>())
            || f.targets.len() != a.targets.len() {
            return Err("回收帧的原来源、程序、阶段或目标数不符".into());
        }
        self.mobility_immutable_identity(c)?;
        if c.definition == "JZ25" { self.validate_jz25_origin(&f.source)?; }
        if a.play_only {
            if f.source.region.is_some() || f.source.source_region_instance.is_some() || f.source.attachment_host_instance.is_some() {
                return Err("手牌打出不能伪装场内来源".into());
            }
        } else if c.definition=="XQ26" { self.xq26_validate_source(f.actor,&f.source)?;
        } else {
            let r = f.source.region.filter(|r| *r < self.regions.len()).ok_or("场内来源缺少地区")?;
            let uid = f.source.source_region_instance.as_deref().filter(|id| !id.is_empty() && *id != c.id).ok_or("场内来源缺少原地区UID")?;
            if self.regions.iter().enumerate().any(|(i, region)| region.card.id == uid && i != r)
                || f.source.attachment_host_instance.as_deref() != if c.definition == "XG28" { Some(uid) } else { None } {
                return Err("场内来源原地区或附属宿主不符".into());
            }
        }
        for (t, spec) in f.targets.iter().zip(&a.targets) {
            if !same(&t.spec, spec) || t.id.is_empty() || t.public.instance_id != t.id
                || !t.public.valid || t.public.status != "valid" || t.public.invalid_reason.is_some() {
                return Err("回收目标槽或公开绑定被改写".into());
            }
            match spec.zone {
                Zone::Region => {
                    let r = t.id.strip_prefix("region:").and_then(|s| s.parse::<usize>().ok()).filter(|r| *r < self.regions.len()).ok_or("目标地区无效")?;
                    if t.id != format!("region:{r}") || t.public.kind != "region" || t.public.region != Some(r)
                        || t.public.owner.is_some() || t.public.controller.is_some() || f.chosen_region != Some(r)
                        || t.region_instance.as_ref().is_none_or(|id| id.is_empty() || *id == c.id)
                        || self.regions.iter().enumerate().any(|(i, region)| Some(&region.card.id) == t.region_instance.as_ref() && i != r) {
                        return Err("目标地区必须保留原实体UID".into());
                    }
                }
                Zone::Player => {
                    let seat = t.id.strip_prefix('p').and_then(|s| s.parse::<usize>().ok()).filter(|s| *s < self.players.len()).ok_or("目标玩家无效")?;
                    if t.id != player_id(seat) || t.public.kind != "player" || t.public.controller != Some(t.id.clone())
                        || t.public.owner.is_some() || t.public.region.is_some() || t.region_instance.is_some() { return Err("玩家公开绑定不符".into()); }
                }
                Zone::Society => {
                    if t.id!=format!("society:p{}",f.actor) || t.public.kind!="society" || t.public.owner!=Some(player_id(f.actor)) || t.public.controller!=Some(player_id(f.actor)) || t.public.region.is_some() || t.region_instance.is_some() || f.chosen_region.is_some() { return Err("通缉令须绑定本方秘社区域".into()); }
                }
                Zone::Graveyard => {
                    if t.public.kind != "character" || t.public.owner != Some(player_id(f.actor)) || t.public.region.is_some()
                        || t.region_instance.is_some() || t.public.controller.as_deref().is_none_or(|id| !self.players.iter().any(|p| player_id(p.seat) == id)) {
                        return Err("墓地角色公开绑定不符".into());
                    }
                }
                _ => return Err("非准入回收目标区域".into()),
            }
        }
        if !a.targets.iter().any(|t| t.zone == Zone::Region) && f.chosen_region.is_some() { return Err("程序没有目标地区".into()); }
        let ids_valid = |ids: &[String]| ids.iter().all(|id| !id.is_empty() && *id != c.id && f.targets.iter().all(|t| t.id != *id))
            && ids.iter().collect::<BTreeSet<_>>().len() == ids.len();
        let paid = if a.play_only {
            matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if f.money_units(0) <= catalog::card(&c.definition).cost as usize && ids_valid(ids))
        } else if a.event.is_some() { f.already_paid.is_empty() }
        else { !c.exhausted && matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids), PaidCost::Exhausted(id)] if f.money_units(0) == 2 && ids_valid(ids) && id == &c.id) };
        if !paid { return Err("回收须保留真实原费用与横置收据".into()); }
        Ok(())
    }
    pub(crate) fn validate_jz25_pending(&self, p: &Pending, d: &Declaration, stage: &DeclareChoice) -> RuleResult<()> {
        let options = self.binding_options(d.actor, &d.source, &d.ability.targets[0]);
        if !matches!(stage, DeclareChoice::Target) || p.seat != d.actor || p.choice.player_id != player_id(d.actor)
            || p.choice.id.is_empty() || p.choice.title != format!("{}：{}", catalog::card("JZ25").name, d.ability.label)
            || p.choice.kind != "trigger" || p.choice.min != Some(0) || p.choice.max != Some(1)
            || p.choice.allow_decline != Some(true) || p.choice.amount.is_some() || !p.choice.preview_cards.is_empty()
            || !same(&options, &p.choice.options) { return Err("卡迪纳猎杀者须由原操控者选择目标玩家".into()); }
        Ok(())
    }
    pub(crate) fn jz25_seal_random_hand(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        let seat = f.targets[0].id.strip_prefix('p').and_then(|s| s.parse::<usize>().ok()).ok_or("封印目标玩家无效")?;
        if !self.seal_host_valid(&f.source.card.id) || self.players[seat].eliminated || self.players[seat].hand.is_empty() { return Ok(()); }
        let index = self.random_below(self.players[seat].hand.len());
        let card = self.players[seat].hand.remove(index);
        self.seal_card_on_valid_host(card, seat, &f.source.card.id);
        Ok(())
    }
    pub(crate) fn xg28_return(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        if let Some(index) = self.players[f.actor].graveyard.iter().position(|c| c.id == f.targets[0].id) {
            let c = self.take_grave_observed(f.actor,index);
            let c = self.reset_zone_card(c); self.players[f.actor].hand.push(c);
        }
        if self.players[f.actor].society_zone.secrecy > 0 {
            if let Some(a) = self.attachments.iter_mut().find(|a| a.card.id == f.source.card.id && !a.card.face_down) { a.card.exhausted = false; }
        }
        Ok(())
    }
    pub(crate) fn bq089_reanimate(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        self.world_graveyard_entry(f.actor, f.chosen_region.ok_or("改造手术缺少目标地区")?, &f.targets[0].id)
    }
}
