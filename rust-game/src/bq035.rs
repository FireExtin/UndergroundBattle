//! The complete pinned BQ035 print. Continuous grants read live entities and
//! real attachments; their event observations are not permanent card modifiers.
use crate::{catalog, model::*, rules::{self, *}, engine::RuleResult};

pub(crate) fn definition() -> Definition {
    let mut d = rules::with_abilities(vec![crate::mobility::ability()]);
    d.modifiers = vec![StaticModifier::BQ035PrintedGlory,
        StaticModifier::BQ035OwnBloodVampireMobility,
        StaticModifier::BQ035OwnEquippedVampireGlory];
    d
}
pub(crate) fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    let own = d.modifiers.iter().any(|m| matches!(m, StaticModifier::BQ035PrintedGlory
        | StaticModifier::BQ035OwnBloodVampireMobility | StaticModifier::BQ035OwnEquippedVampireGlory));
    if (id == "BQ035" && !same(d, &definition())) || (id != "BQ035" && own) {
        return Err("BQ035完整机动、威名及两条本方吸血鬼持续能力不能改写或移植".into());
    }
    Ok(())
}
impl Game {
    fn bq035_beneficiary(&self, id: &str) -> Option<&Card> {
        let (_, c) = self.board(id)?;
        (!c.face_down && catalog::card(&c.definition).kind == "character"
            && self.current_subtypes(c).iter().any(|s| s == "吸血鬼")).then_some(c)
    }
    pub(crate) fn bq035_mobility_witness(&self, id: &str) -> Option<ObservedMobilityGrant> {
        let c = self.bq035_beneficiary(id)?;
        let (region, knight) = self.regions.iter().enumerate().filter(|(_, r)| !r.vacant)
            .flat_map(|(r, region)| region.cards.iter().map(move |c| (r, c)))
            .find(|(_, source)| !source.face_down && source.definition == "BQ035"
                && source.controller == c.controller)?;
        let blood = self.attachments.iter().find(|a| !a.card.face_down && a.host_id == c.id
            && catalog::card(&a.card.definition).name == "鲜血" && self.attachment_host_valid(a))?;
        Some(ObservedMobilityGrant { turn: self.turn, vampire: true,
            knight: knight.clone(), knight_region: region, blood: blood.clone() })
    }
    pub(crate) fn bq035_grants_glory(&self, id: &str) -> bool {
        let Some(c) = self.bq035_beneficiary(id) else { return false; };
        self.regions.iter().filter(|r| !r.vacant).flat_map(|r| &r.cards).any(|source|
            !source.face_down && source.definition == "BQ035" && source.controller == c.controller)
            && self.attachments.iter().any(|a| !a.card.face_down && a.host_id == c.id
                && self.attachment_host_valid(a)
                && catalog::card(&a.card.definition).subtypes.iter().any(|s| s == "装备" || s == "武器"))
    }
}
