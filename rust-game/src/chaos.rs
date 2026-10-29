//! Hegemony physical PDF19 / printed18: a face-up Chaos character enters a
//! random live region. Movement is not entry into play and does not reroll.
use crate::{model::*, rules};

impl Game {
    pub(crate) fn wm031_chaos(&self,c:&Card)->bool {c.definition==crate::wm031::ID && self.society_exposure(c.controller)>0}
    pub(crate) fn chaos_region(&mut self) -> Option<usize> {
        let regions = (0..self.regions.len()).filter(|r| self.region_live(*r)).collect::<Vec<_>>();
        if regions.is_empty() { return None; }
        Some(regions[self.random_below(regions.len())])
    }
    pub(crate) fn route_chaos_entry(&mut self, id: &str) {
        let Some((from, c)) = self.board(id) else { return; };
        if c.face_down || !(rules::definition(&c.definition).traits.chaos||self.wm031_chaos(c)) { return; }
        let Some(region) = self.chaos_region() else { return; };
        if from != region {
            if let Some((_, c)) = self.remove_board(id) { self.regions[region].cards.push(c); }
        }
        self.note(format!("混乱进场：随机地区{}", region + 1));
    }
}
