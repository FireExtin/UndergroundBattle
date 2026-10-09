//! XQ44/JZ02 only: private one-or-zero deck choice, direct sealing, one shuffle.
use crate::{
    catalog::{self, CardDefinition},
    engine::RuleResult,
    model::*,
    rules::Op,
};
use std::collections::BTreeSet;

pub(crate) fn dream_card(d: &CardDefinition) -> bool {
    d.subtypes.iter().any(|s| s == "梦境")
}
pub(crate) fn space_spell_card(d: &CardDefinition) -> bool {
    d.kind == "spell"
        && d.subtypes.iter().any(|s| s == "空间")
        && d.subtypes.iter().any(|s| s == "法术" || s == "事务-法术")
}
impl Game {
    fn deck_seal_actor(&self, frame: &ResolutionFrame, dream: bool) -> RuleResult<usize> {
        let source = if dream { "XQ44" } else { "JZ02" };
        let ability = &crate::rules::definition(source).abilities[0];
        if frame.source.card.definition != source
            || frame.ability_key != ability.key
            || frame.actor >= self.players.len()
            || frame.source.card.owner >= self.players.len()
            || frame.source.card.controller != frame.actor
            || frame.source.card.face_down
            || frame.source.card.id.is_empty()
            || !matches!(frame.guard, GuardState::Accepted)
            || frame.cursor != 1
            || frame.steps.len() != 1
            || frame.steps[0].context != frame.actor
            || !(if dream {
                matches!(frame.steps[0].op, Op::XQ44SearchDreamSealOnTarget)
            } else {
                matches!(frame.steps[0].op, Op::JZ02SearchSpaceSpellSealOnSource)
            })
            || frame.targets.len() != ability.targets.len()
            || (dream
                && (frame.targets[0].id.is_empty()
                    || frame.targets[0].public.instance_id != frame.targets[0].id
                    || serde_json::to_value(&frame.targets[0].spec).unwrap()
                        != serde_json::to_value(&ability.targets[0]).unwrap()))
            || (!dream && (!frame.already_paid.is_empty() || frame.source.region.is_none()))
        {
            return Err("牌库封印检索来源、完整程序或冻结行动者不符".into());
        }
        Ok(frame.actor)
    }
    fn deck_seal_host<'a>(&self, frame: &'a ResolutionFrame, dream: bool) -> &'a str {
        if dream {
            &frame.targets[0].id
        } else {
            &frame.source.card.id
        }
    }
    fn deck_seal_host_valid(&self, frame: &ResolutionFrame, dream: bool) -> bool {
        let host = self.deck_seal_host(frame, dream);
        self.seal_host_valid(host)
            && (dream
                || self
                    .board(host)
                    .is_some_and(|(_, c)| c.definition == "JZ02"))
    }
    fn deck_seal_candidates(&self, actor: usize, dream: bool) -> RuleResult<Vec<&Card>> {
        let mut seen = BTreeSet::new();
        let mut candidates = Vec::new();
        for c in &self.players[actor].deck {
            let d = catalog::catalog()
                .cards
                .iter()
                .find(|d| d.id == c.definition)
                .ok_or("牌库封印检索含未准入定义")?;
            if c.id.is_empty() || c.owner >= self.players.len() || !seen.insert(&c.id) {
                return Err("牌库封印检索实例或拥有者无效".into());
            }
            if if dream {
                dream_card(d)
            } else {
                space_spell_card(d)
            } {
                candidates.push(c);
            }
        }
        Ok(candidates)
    }
    pub(crate) fn validate_deck_seal_choice(&self) -> RuleResult<()> {
        let Some(p) = &self.pending else {
            return Ok(());
        };
        let framed = matches!(
            &p.resolution,
            ChoiceResolution::Frame {
                choice: FrameChoice::XQ44DreamSealSearch | FrameChoice::JZ02SpaceSealSearch,
                ..
            }
        );
        if !framed
            && p.choice.kind != "xq44_dream_seal_search"
            && p.choice.kind != "jz02_space_seal_search"
        {
            return Ok(());
        }
        let ChoiceResolution::Frame { frame, choice } = &p.resolution else {
            return Err("牌库封印待选程序无效".into());
        };
        let dream = match choice {
            FrameChoice::XQ44DreamSealSearch => true,
            FrameChoice::JZ02SpaceSealSearch => false,
            _ => return Err("牌库封印待选程序无效".into()),
        };
        let actor = self.deck_seal_actor(frame, dream)?;
        let candidates = self.deck_seal_candidates(actor, dream)?;
        let expected = candidates
            .iter()
            .map(|c| self.option(c, actor, None, None))
            .collect::<Vec<_>>();
        let count = usize::from(!candidates.is_empty());
        // A private choice is atomic; no ordinary command can invalidate its host.
        if self.players[actor].eliminated
            || !self.deck_seal_host_valid(frame, dream)
            || p.seat != actor
            || p.choice.player_id != format!("p{actor}")
            || p.choice.kind
                != if dream {
                    "xq44_dream_seal_search"
                } else {
                    "jz02_space_seal_search"
                }
            || p.choice.min != Some(count)
            || p.choice.max != Some(count)
            || p.choice.allow_decline != Some(false)
            || serde_json::to_value(&p.choice.options).unwrap()
                != serde_json::to_value(expected).unwrap()
        {
            return Err("牌库封印待选快照与行动者牌库或原载体不符".into());
        }
        Ok(())
    }
    pub(crate) fn deck_seal_search_start(
        &mut self,
        frame: ResolutionFrame,
        dream: bool,
    ) -> RuleResult<()> {
        let actor = self.deck_seal_actor(&frame, dream)?;
        if self.players[actor].eliminated {
            return Ok(());
        }
        if !self.deck_seal_host_valid(&frame, dream) {
            // JZ02 self-reference is not a declared target. Approved ruling:
            // skip searching when the original self is gone/hidden, then shuffle.
            // XQ44 is cancelled earlier by the ordinary whole-frame target guard.
            if !dream {
                self.shuffle_player(actor);
                self.note("西姆斯教授原实例已离场或翻暗，跳过检索并完成洗牌".into());
            }
            return Ok(());
        }
        let candidates = self.deck_seal_candidates(actor, dream)?;
        let count = usize::from(!candidates.is_empty());
        let options = candidates
            .iter()
            .map(|c| self.option(c, actor, None, None))
            .collect();
        self.choice(
            actor,
            if dream {
                "xq44_dream_seal_search"
            } else {
                "jz02_space_seal_search"
            },
            if dream {
                "检索一张梦境牌，封印在原目标角色上，然后洗牌"
            } else {
                "检索一张空间法术牌，封印在原西姆斯教授上，然后洗牌"
            }
            .into(),
            options,
            count,
            count,
            None,
            ChoiceResolution::Frame {
                frame: Box::new(frame),
                choice: if dream {
                    FrameChoice::XQ44DreamSealSearch
                } else {
                    FrameChoice::JZ02SpaceSealSearch
                },
            },
        );
        self.pending.as_mut().unwrap().choice.allow_decline = Some(false);
        Ok(())
    }
    pub(crate) fn deck_seal_search_complete(
        &mut self,
        frame: &ResolutionFrame,
        selected: &[String],
        dream: bool,
    ) -> RuleResult<()> {
        let actor = self.deck_seal_actor(frame, dream)?;
        if self.players[actor].eliminated {
            return Err("牌库封印检索行动者已退出".into());
        }
        let candidates = self.deck_seal_candidates(actor, dream)?;
        let count = usize::from(!candidates.is_empty());
        if selected.len() != count
            || selected
                .first()
                .is_some_and(|id| !candidates.iter().any(|c| c.id == *id))
        {
            return Err("有符合条件的牌必须选择一张；无牌才能选择零张".into());
        }
        let host = self.deck_seal_host(frame, dream).to_owned();
        if let Some(id) = selected.first() {
            let index = self.players[actor]
                .deck
                .iter()
                .position(|c| c.id == *id)
                .unwrap();
            let c = self.players[actor].deck.remove(index);
            if self.deck_seal_host_valid(frame, dream) {
                self.seal_card_on_valid_host(c, actor, &host);
            } else {
                // Defensive P4 boundary for an already determined A, not the
                // approved pre-search skip. No response window is added here.
                let owner = c.owner;
                let c = self.reset_zone_card(c);
                if !self.players[owner].eliminated {
                    self.players[owner].graveyard.push(c);
                }
                self.note("已选封印牌的原载体失效，置入拥有者墓地".into());
            }
        }
        self.shuffle_player(actor);
        self.note(format!(
            "{} 完成{}检索并洗牌",
            self.players[actor].name,
            if dream {
                "随风入梦"
            } else {
                "西姆斯教授"
            }
        ));
        Ok(())
    }
}
