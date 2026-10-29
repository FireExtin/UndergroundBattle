//! BQ026 retains both printed types in either in-play container. Becoming
//! attached is not a second entry into play. FAQ physical page 5 supplies the
//! shared dual-type damage, movement and host-departure rules.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegionEntry { pub event_id: String, pub turn: u32 }

pub(crate) fn definition() -> Definition {
    let host = target(Zone::Board, EntityKind::Character, Relation::Any, Range::Anywhere);
    let mut d = attachment_definition(host);
    let mut t = target(Zone::Board, EntityKind::Character, Relation::ControlledByActor, Range::SourceRegion);
    t.subtype = Some("吸血鬼".into()); t.exclude_source = true;
    d.abilities = vec![ability("batch3-bq026-attach", "进入地区：可结附于本地区目标本方吸血鬼", Timing::Fast,
        vec![], vec![t], vec![Op::BQ026Attach { observation: None }], Some(Event::EnterRegion))];
    d.traits.cannot_be_equipped = true;
    d.traits.public = true;
    d
}
impl Game {
    pub(crate) fn region_characters(&self, r: usize) -> impl Iterator<Item = &Card> {
        self.regions[r].cards.iter().chain(self.region_attachment_cards(r))
            .filter(|c| catalog::card(&c.definition).kind == "character")
    }
    pub(crate) fn live_attachment_role(&self, c: &Card) -> bool {
        !c.face_down && catalog::catalog().cards.iter().find(|d|d.id==c.definition).is_some_and(|d|catalog::has_kind(d, "attachment"))
            && self.in_play(&c.id).is_some_and(|(_,live)|live==c)
            && self.attachments.iter().find(|a|a.card.id==c.id).is_none_or(|a|self.attachment_host_valid(a))
    }
    pub(crate) fn attached_character(&self, id: &str) -> bool {
        self.attachments.iter().any(|a| a.card.id == id && catalog::card(&a.card.definition).kind == "character")
    }
    pub(crate) fn bq026_spell_targetable(&self, actor: usize, source: &SourceSnapshot, target: &Card) -> bool {
        catalog::card(&source.card.definition).kind != "spell" || !self.is_enemy(actor, target) || !self.bq026_protected(target)
    }
    pub(crate) fn bq026_protected(&self, target: &Card) -> bool {
        !target.face_down && self.attachments.iter().any(|a| a.card.definition == "BQ026" && !a.card.face_down
                && a.host_id == target.id && self.attachment_host_valid(a))
    }
    pub(crate) fn bq026_observation(&self, s: &SourceSnapshot, ops: &[Op]) -> RuleResult<()> {
        let [Op::BQ026Attach { observation: Some(o) }] = ops else { return Err("蝙蝠群缺少真实进入地区观察".into()); };
        if s.card.definition != "BQ026" || o.turn != self.turn || o.event_id == s.card.id
            || o.event_id.strip_prefix('i').and_then(|s| s.parse::<u64>().ok()).is_none_or(|n| n == 0 || n > self.sequence)
            || s.attachment_host_instance.as_ref().is_some_and(|id| id.is_empty() || *id == s.card.id)
        { return Err("蝙蝠群进入地区的事件或宿主身份无效".into()); }
        Ok(())
    }
    pub(crate) fn bq026_attach(&mut self, f: &ResolutionFrame) {
        let target = &f.targets[0].id;
        let Some((r, c)) = self.board(&f.source.card.id) else { return; };
        // The bound target uses the original region and current target guards.
        // A source which has since moved cannot jump back to that old region.
        if c.face_down || c.definition != "BQ026" || Some(r) != f.source.region { return; }
        let changed = if let Some(a) = self.attachments.iter_mut().find(|a| a.card.id == f.source.card.id) {
            if a.host_id == *target { false } else { a.host_id = target.clone(); true }
        } else if let Some((_, c)) = self.remove_board(&f.source.card.id) {
            self.attachments.push(Attachment { card: c, host_id: target.clone() }); true
        } else { false };
        if changed { self.observe_attachment_committed(&f.source.card.id); }
        self.settle_deaths();
    }
    /// Expand a departure wave before freezing observers, retaining the supplied
    /// cause/snapshot when a child is already independently in the lethal set.
    pub(crate) fn dual_character_death_wave(&self, deaths: Vec<(String, RemovalCause, SourceSnapshot)>)
        -> Vec<(String, RemovalCause, SourceSnapshot)> {
        if !self.attachments.iter().any(|a| a.card.definition == "BQ026") { return deaths; }
        let mut entries = BTreeMap::new(); let mut roots = Vec::new();
        for (id, cause, source) in deaths { if !entries.contains_key(&id) { roots.push(id.clone()); entries.insert(id, (cause, source)); } }
        let mut todo = roots.clone();
        while let Some(host) = todo.pop() {
            for a in self.attachments.iter().filter(|a| a.host_id == host && catalog::card(&a.card.definition).kind == "character") {
                if !entries.contains_key(&a.card.id) {
                    entries.insert(a.card.id.clone(), (RemovalCause::Destroy, self.source_snapshot(&a.card, self.attachment_region(a))));
                    todo.push(a.card.id.clone());
                }
            }
        }
        // Bounded postorder also tolerates a forged cycle without recursion.
        let mut result = Vec::new(); let mut emitted = BTreeSet::new();
        let mut todo = roots.into_iter().rev().map(|id| (id, false)).collect::<Vec<_>>(); let mut expanded = BTreeSet::new();
        while let Some((id, ready)) = todo.pop() {
            if ready { if emitted.insert(id.clone()) { if let Some((cause, s)) = entries.remove(&id) { result.push((id, cause, s)); } } }
            else if expanded.insert(id.clone()) {
                todo.push((id.clone(), true));
                for a in self.attachments.iter().filter(|a| a.host_id == id && entries.contains_key(&a.card.id)) { todo.push((a.card.id.clone(), false)); }
            }
        }
        result
    }
    pub(crate) fn validate_bq026_state(&self) -> RuleResult<()> {
        for a in &self.attachments {
            let d=catalog::known_card(&a.card.definition).ok_or("附属不是登记卡牌")?;
            if d.kind == "character"
                && (a.card.definition != "BQ026" || a.card.face_down || self.attachment_region(a).is_none() || !self.attachment_host_valid(a))
            { return Err("双类型角色的结附状态或宿主图无效".into()); }
        }
        Ok(())
    }
    pub(crate) fn validate_bq026_pending(&self, p: &Pending, d: &Declaration, stage: &DeclareChoice) -> RuleResult<()> {
        self.validate_batch3_declaration(d)?;
        let options=self.binding_options(d.actor,&d.source,&d.ability.targets[0]);
        if !matches!(stage,DeclareChoice::Target) || options.is_empty() || p.seat!=d.actor
            || p.choice.player_id!=player_id(d.actor) || p.choice.id.is_empty()
            || p.choice.title!=format!("{}：{}",catalog::card("BQ026").name,d.ability.label)
            || p.choice.kind!="trigger" || p.choice.min!=Some(0) || p.choice.max!=Some(1)
            || p.choice.allow_decline!=Some(true) || p.choice.amount.is_some() || !p.choice.preview_cards.is_empty()
            || serde_json::to_value(options).unwrap()!=serde_json::to_value(&p.choice.options).unwrap()
        {return Err("蝙蝠群进入地区须保留原操控者的可选目标声明".into());} Ok(())
    }
}
