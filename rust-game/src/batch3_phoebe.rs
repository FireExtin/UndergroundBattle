//! CQ07 is the reviewed blue replacement for deferred CQ11. Target observations
//! freeze the actual paid enemy spell and each distinct face-up character.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub(crate) const ENTER: &str = "batch3-cq07-enter";
pub(crate) const TARGETED: &str = "batch3-cq07-targeted";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellTargetObservation {
    pub event_id: String,
    pub turn: u32,
    pub spell: SourceSnapshot,
    pub spell_ability: String,
    pub target: SourceSnapshot,
    pub subtypes: Vec<String>,
}

fn same<T: Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
pub(crate) fn definition() -> Definition {
    with_abilities(vec![
        ability(ENTER, "进场：本方秘社没有隐秘标志时放置隐秘1", Timing::Fast,
            vec![], vec![], vec![Op::CQ07EnterMoon], Some(Event::Enter)),
        ability(TARGETED, "本方吸血鬼或法师成为敌方事务目标：抓1", Timing::Fast,
            vec![], vec![], vec![Op::CQ07SpellTargeted { observation: None }], Some(Event::CQ07SpellTargeted)),
    ])
}

impl Game {
    pub(crate) fn cq07_observe_spell_targets(&mut self, frame: &ResolutionFrame) {
        if frame.targets.is_empty() { return; }
        let mut sources = self.regions.iter().enumerate().filter(|(_, r)| !r.vacant)
            .flat_map(|(r, region)| region.cards.iter().map(move |c| (r, c)))
            .filter(|(_, c)| c.definition == "CQ07" && !c.face_down && !self.players[c.controller].eliminated)
            .map(|(r, c)| self.source_snapshot(c, Some(r))).collect::<Vec<_>>();
        if sources.is_empty() { return; }
        sources.sort_by_key(|s| (self.team(s.card.controller) != self.first_team, s.card.controller));
        let mut seen = BTreeSet::new();
        let targets = frame.targets.iter().filter(|t| seen.insert(t.id.clone()))
            .filter_map(|t| self.board(&t.id))
            .filter(|(_, c)| !c.face_down && catalog::card(&c.definition).kind == "character")
            .filter_map(|(r, c)| {
                let types = self.current_subtypes(c);
                types.iter().any(|t| t == "吸血鬼" || t == "法师")
                    .then(|| (self.source_snapshot(c, Some(r)), types))
            }).collect::<Vec<_>>();
        for source in sources {
            for (target, subtypes) in &targets {
                if target.card.controller != source.card.controller || !self.is_enemy(frame.actor, &target.card) { continue; }
                let observation = SpellTargetObservation {
                    event_id: frame.frame_id.clone(), turn: self.turn, spell: frame.source.clone(),
                    spell_ability: frame.ability_key.clone(), target: target.clone(), subtypes: subtypes.clone(),
                };
                let mut a = definition().abilities.remove(1);
                a.ops = vec![Op::CQ07SpellTargeted { observation: Some(observation) }];
                self.effects.push_back(Effect::Declare { declaration: Declaration {
                    actor: source.card.controller, source: source.clone(), ability: a,
                }});
            }
        }
    }
    pub(crate) fn cq07_observation(&self, actor: usize, source: &SourceSnapshot, ops: &[Op]) -> RuleResult<()> {
        if matches!(ops, [Op::CQ07EnterMoon]) { return Ok(()); }
        let [Op::CQ07SpellTargeted { observation: Some(o) }] = ops else {
            return Err("菲碧触发缺少真实事务指定目标观察".into());
        };
        let number = |id: &str| id.strip_prefix('i').and_then(|v| v.parse::<u64>().ok()).filter(|n| *n > 0 && *n <= self.sequence);
        let spell = &o.spell.card;
        let target = &o.target.card;
        let printed = catalog::catalog().cards.iter().find(|d| d.id == target.definition && d.kind == "character")
            .ok_or("菲碧观察的目标不是登记角色")?;
        let spell_print = catalog::catalog().cards.iter().find(|d| d.id == spell.definition && d.kind == "spell")
            .ok_or("菲碧只能观察敌方事务牌")?;
        if source.card.definition != "CQ07" || source.card.controller != actor || source.card.face_down
            || actor >= self.players.len() || target.controller != actor || target.owner >= self.players.len()
            || target.face_down || spell.face_down || spell.owner >= self.players.len() || spell.controller >= self.players.len()
            || !self.is_enemy(spell.controller, target) || !catalog::runtime_implemented(spell_print)
            || !rules_for_spell(&spell.definition, &o.spell_ability)
            || o.spell.region.is_some() || o.spell.source_region_instance.is_some() || o.spell.attachment_host_instance.is_some()
            || o.spell.play_source.is_some() || o.spell.observed_death.is_some() || o.spell.observed_mobility_grant.is_some() || o.spell.observed_exposure.is_some()
            || o.target.region.is_none_or(|r| r >= self.regions.len()) || o.target.play_source.is_some()
            || o.target.observed_death.is_some() || o.target.observed_mobility_grant.is_some() || o.target.observed_exposure.is_some()
            || !o.subtypes.iter().any(|t| t == "吸血鬼" || t == "法师")
            || !printed.subtypes.iter().any(|t| matches!(t.as_str(), "吸血鬼" | "法师" | "人类")) && target.definition != "JC030"
            || o.subtypes.iter().collect::<BTreeSet<_>>().len() != o.subtypes.len()
            || o.subtypes.iter().any(|t| !printed.subtypes.contains(t) && !matches!(t.as_str(), "吸血鬼" | "奴仆" | "邪教徒" | "宿主" | "不死生物" | "骷髅"))
            || o.turn != self.turn || number(&o.event_id).is_none() || number(&target.id).is_none() || number(&spell.id).is_none()
            || number(&o.event_id) <= number(&target.id) || number(&o.event_id) <= number(&spell.id)
            || o.event_id == source.card.id
            || self.batch3_physical_cards().any(|c| c.id == spell.id)
            || self.batch3_physical_cards().any(|c| c.id == target.id && self.board(&target.id).is_none())
            || self.stack.iter().any(|item| item.id == o.event_id && item.frame.as_ref().is_none_or(|f|
                !same(&f.source, &o.spell) || f.actor != spell.controller || f.ability_key != o.spell_ability
                    || !f.targets.iter().any(|t| t.id == target.id && t.spec.zone == Zone::Board)))
        { return Err("菲碧须保留原敌方事务、目标资格、玩家及事件身份".into()); }
        self.mobility_immutable_identity(spell)?;
        self.mobility_immutable_identity(target)
    }
    pub(crate) fn validate_cq07_pending(&self, p: &Pending, d: &Declaration, stage: &DeclareChoice) -> RuleResult<()> {
        self.validate_batch3_declaration(d)?;
        let options = vec![ChoiceOption { id: "accept".into(), label: "发动触发能力".into(), card: None }];
        if !matches!(stage, DeclareChoice::Accept) || p.seat != d.actor || p.choice.player_id != player_id(d.actor)
            || p.choice.id.is_empty() || p.choice.title != format!("{}：{}", catalog::card("CQ07").name, d.ability.label)
            || p.choice.kind != "trigger" || p.choice.min != Some(0) || p.choice.max != Some(1)
            || p.choice.allow_decline != Some(true) || p.choice.amount.is_some() || !p.choice.preview_cards.is_empty()
            || !same(&p.choice.options, &options)
        { return Err("菲碧必须保留原玩家的可选触发选择".into()); }
        Ok(())
    }
}
fn rules_for_spell(id: &str, key: &str) -> bool {
    crate::rules::definition(id).abilities.iter().any(|a| a.key == key && a.event.is_none() && !a.activation_only)
}
