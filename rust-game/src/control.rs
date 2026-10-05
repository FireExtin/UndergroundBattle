//! Finite resolved control/type effects. Vec order is activation order.
//! Last valid control wins; subtype grants compose independently in that order.
use crate::{catalog::card, model::*, rules::TargetSlotSpec};

impl Game {
    fn control_recipient(&self, effect: &ControlEffect) -> Option<usize> {
        let (_, target) = self.board(&effect.target_instance)?;
        if target.face_down || card(&target.definition).kind != "character" {
            return None;
        }
        let recipient = match &effect.lifetime {
            ControlLifetime::TurnEnd { turn } => {
                if *turn != self.turn {
                    return None;
                }
                effect.recipient
            }
            ControlLifetime::Attached { source_instance } => {
                let attachment = self
                    .attachments
                    .iter()
                    .find(|a| a.card.id == *source_instance && a.host_id == target.id)?;
                // Only this finite attachment family creates attached control.
                // Printed eligibility never depends on its own subtype overlay.
                let spec = crate::rules::definition(&attachment.card.definition)
                    .attachment
                    .as_ref()?;
                if !spec.controls_host
                    || !card(&target.definition)
                        .subtypes
                        .iter()
                        .any(|s| s == "人类")
                {
                    return None;
                }
                attachment.card.controller
            }
            ControlLifetime::SourceLeaves { source_instance } => {
                let (_, source) = self.board(source_instance)?;
                if source.face_down {
                    return None;
                }
                // Historical resolved trigger recipient is fixed, even if source changes controller.
                effect.recipient
            }
        };
        self.players
            .get(recipient)
            .filter(|p| !p.eliminated)
            .map(|_| recipient)
    }

    pub fn current_subtypes(&self, c: &Card) -> Vec<String> {
        if c.face_down {
            return vec![];
        }
        let mut result = card(&c.definition).subtypes.clone();
        if self.jc030_blood_assets_active(c) {
            result.push("吸血鬼".into());
        }
        for effect in self
            .control_effects
            .iter()
            .filter(|e| e.target_instance == c.id && self.control_recipient(e).is_some())
        {
            match effect.subtype_change {
                SubtypeChange::None => {}
                SubtypeChange::HumanToVampire => {
                    let human = result.iter().any(|s| s == "人类");
                    result.retain(|s| s != "人类");
                    if human && !result.iter().any(|s| s == "吸血鬼") {
                        result.push("吸血鬼".into());
                    }
                }
                SubtypeChange::AddSlave => {
                    if !result.iter().any(|s| s == "奴仆") {
                        result.push("奴仆".into());
                    }
                }
            }
        }
        result
    }
    pub(crate) fn target_subtypes(&self, c: &Card, slot: &TargetSlotSpec) -> Vec<String> {
        if slot.printed_subtype {
            card(&c.definition).subtypes.clone()
        } else {
            self.current_subtypes(c)
        }
    }
    pub(crate) fn add_control(
        &mut self,
        target: &str,
        recipient: usize,
        lifetime: ControlLifetime,
        subtype_change: SubtypeChange,
    ) {
        let Some((_, c)) = self.board(target) else {
            return;
        };
        if !self
            .control_baselines
            .iter()
            .any(|b| b.target_instance == target)
        {
            self.control_baselines.push(ControlBaseline {
                target_instance: target.into(),
                controller: c.controller,
            });
        }
        self.control_effects.push(ControlEffect {
            target_instance: target.into(),
            recipient,
            lifetime,
            subtype_change,
        });
        self.settle_controls();
    }
    /// Identity reset ends old targeted effects; synthetic baseline control remains unchanged.
    pub(crate) fn controller_after_reset(&self, target: &str) -> Option<usize> {
        let (_, c) = self.board(target)?;
        let baseline = self
            .control_baselines
            .iter()
            .find(|b| b.target_instance == target)
            .map_or(c.controller, |b| b.controller);
        Some(
            if self.players.get(baseline).is_some_and(|p| !p.eliminated) {
                baseline
            } else {
                c.owner
            },
        )
    }
    pub(crate) fn settle_controls(&mut self) {
        let effects = std::mem::take(&mut self.control_effects);
        self.control_effects = effects
            .into_iter()
            .filter(|e| self.control_recipient(e).is_some())
            .collect();
        let mut changed = vec![];
        for baseline in &self.control_baselines {
            let Some((_, c)) = self.board(&baseline.target_instance) else {
                continue;
            };
            let fallback = if self
                .players
                .get(baseline.controller)
                .is_some_and(|p| !p.eliminated)
            {
                baseline.controller
            } else {
                c.owner
            };
            let controller = self
                .control_effects
                .iter()
                .rev()
                .find(|e| e.target_instance == c.id)
                .and_then(|e| self.control_recipient(e))
                .unwrap_or(fallback);
            if c.controller != controller {
                changed.push((
                    c.id.clone(),
                    controller,
                    !c.face_down && card(&c.definition).unique,
                ));
            }
        }
        for (id, controller, unique) in changed {
            if let Some(c) = self.board_mut(&id) {
                c.controller = controller;
            }
            if unique
                && !self
                    .effects
                    .iter()
                    .any(|e| matches!(e, Effect::Unique { seat } if *seat == controller))
            {
                self.effects.push_back(Effect::Unique { seat: controller });
            }
        }
        let baselines = std::mem::take(&mut self.control_baselines);
        self.control_baselines = baselines
            .into_iter()
            .filter(|b| {
                self.board(&b.target_instance).is_some()
                    && self
                        .control_effects
                        .iter()
                        .any(|e| e.target_instance == b.target_instance)
            })
            .collect();
    }
}
