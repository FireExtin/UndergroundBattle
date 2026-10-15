//! Complete printed JC060. Its continuous grant is derived from live friendly
//! advisers and immutable printed domains, never stored on a recipient.
use crate::{
    catalog::card,
    engine::RuleResult,
    model::{Card, Game, MagicDomainCount},
    rules::{Definition, MagicIcon, StaticModifier},
};

pub(crate) fn definition() -> Definition {
    Definition {
        modifiers: vec![StaticModifier::JC060FriendlyUnprintedMindAndCombat],
        ..Default::default()
    }
}

pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    let uses = d
        .modifiers
        .iter()
        .any(|m| matches!(m, StaticModifier::JC060FriendlyUnprintedMindAndCombat));
    if (id == "JC060" || uses)
        && (id != "JC060"
            || serde_json::to_value(d).unwrap() != serde_json::to_value(definition()).unwrap())
    {
        return Err("JC060只准入完整持续定义，不能改写或移植".into());
    }
    Ok(())
}

impl Game {
    /// Each live face-up adviser contributes one Mind and one permanent Combat
    /// to every currently friendly character with no *printed* domain. Hidden
    /// cards and assets are different card types, even when their print is a
    /// character. Exhaustion does not switch off a continuous ability.
    pub(crate) fn jc060_friendly_sources(&self, c: &Card) -> u32 {
        let printed = card(&c.definition);
        if c.face_down || printed.kind != "character" || printed.magic_icon != MagicIcon::None {
            return 0;
        }
        let Some(live) = self
            .regions
            .iter()
            .flat_map(|r| &r.cards)
            .find(|live| live.id == c.id)
        else {
            return 0;
        };
        if live.face_down || live.definition != c.definition || live.controller != c.controller {
            return 0;
        }
        u32::try_from(
            self.regions
                .iter()
                .flat_map(|r| &r.cards)
                .filter(|source| {
                    !source.face_down
                        && source.definition == "JC060"
                        && self.team(source.controller) == self.team(live.controller)
                        && crate::rules::definition(&source.definition)
                            .modifiers
                            .iter()
                            .any(|m| {
                                matches!(m, StaticModifier::JC060FriendlyUnprintedMindAndCombat)
                            })
                })
                .count(),
        )
        .unwrap_or(u32::MAX)
    }

    /// Current number of magic icons, shared by domain targets and spirit
    /// advantage. A printed domain contributes one; each adviser adds one.
    /// This is intentionally separate from immutable deck/asset filters.
    pub fn current_magic_count(&self, c: &Card) -> u32 {
        if c.face_down {
            return 0;
        }
        u32::from(card(&c.definition).magic_icon != MagicIcon::None)
            .saturating_add(self.jc060_friendly_sources(c))
    }

    pub(crate) fn jc060_current_domains(&self, c: &Card) -> Option<Vec<MagicDomainCount>> {
        let count = self.jc060_friendly_sources(c);
        (count > 0).then(|| {
            vec![MagicDomainCount {
                magic: "心灵".into(),
                count,
            }]
        })
    }
}
