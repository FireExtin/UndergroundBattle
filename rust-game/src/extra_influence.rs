//! A region/turn counter, never attached to Natris' changing card instance.
use crate::{engine::RuleResult, model::*};
use serde::{Serialize, Deserialize};
use std::collections::BTreeSet;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InfluenceProgress {
    pub turn: u32,
    pub region: usize,
    pub region_instance: String,
    pub completed: u32,
    pub ended: bool,
}
impl Game {
    fn allowed_influence_contests(&self, region: usize) -> u32 {
        1 + self.regions[region].cards.iter().filter(|c| c.definition == "JC067" && !c.face_down).count() as u32
    }
    fn end_influence_contests(&mut self, index: usize) {
        let p = &mut self.influence_contests[index];
        if p.ended { return; }
        p.ended = true;
        self.effects.push_back(Effect::RegionConfrontationsEnded { region: p.region, region_instance: p.region_instance.clone() });
    }
    pub(crate) fn complete_influence_contest(&mut self, region: usize) {
        let uid = self.regions[region].card.id.clone();
        let index = self.influence_contests.iter().position(|p| p.turn == self.turn && p.region_instance == uid)
            .unwrap_or_else(|| {
                self.influence_contests.push(InfluenceProgress { turn: self.turn, region, region_instance: uid, completed: 0, ended: false });
                self.influence_contests.len() - 1
            });
        let p = &mut self.influence_contests[index];
        p.completed += 1;
        if p.completed >= self.allowed_influence_contests(region) { self.end_influence_contests(index); }
    }
    pub(crate) fn continue_influence_contests(&mut self, region: usize) -> bool {
        if !self.region_live(region) || self.regions[region].skip { return false; }
        let uid = &self.regions[region].card.id;
        // A replacement has no progress record and gets no fresh contests.
        let Some(index) = self.influence_contests.iter().position(|p| p.turn == self.turn && &p.region_instance == uid) else { return false; };
        if self.influence_contests[index].ended { return false; }
        if self.influence_contests[index].completed < self.allowed_influence_contests(region) {
            self.begin_window(Window::Before(region, 2));
        } else {
            // A granting mayor left during the after-window. Finish the region
            // now; do not advance while its final responsive reward is queued.
            self.end_influence_contests(index);
            self.begin_window(Window::After(region, 2));
        }
        true
    }
    pub(crate) fn validate_influence_contests(&self) -> RuleResult<()> {
        let mut seen = BTreeSet::new();
        for p in &self.influence_contests {
            if p.turn != self.turn || p.region >= self.regions.len() || p.region_instance.is_empty()
                || p.completed == 0 || p.completed > 1 + self.players.len() as u32 || !seen.insert(&p.region_instance) {
                return Err("势力对抗进度的回合、地区或计数无效".into());
            }
            if self.regions[p.region].card.id != p.region_instance && !p.ended {
                return Err("替换地区不得保留未结束的对抗进度".into());
            }
        }
        Ok(())
    }
}
