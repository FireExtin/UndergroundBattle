//! Two closed printed entry searches. JZ50 keeps its separate admitted program.
use crate::{
    catalog::{self, card, CardDefinition},
    engine::RuleResult,
    model::*,
    rules::Op,
};
use std::collections::BTreeSet;

pub(crate) fn bq104_search_match(d: &CardDefinition) -> bool {
    d.kind == "character" && d.cost <= 3 && d.subtypes.iter().any(|s| s == "雇员")
}
pub(crate) fn xq48_search_match(d: &CardDefinition) -> bool {
    d.id == "JC125" && d.kind == "character" && d.name == "无知路人"
}
impl Game {
    fn entry_search_actor(&self, frame: &ResolutionFrame, employee: bool) -> RuleResult<usize> {
        let (source, key) = if employee {
            ("BQ104", "bq104-entry-employee-search")
        } else {
            ("XQ48", "xq48-entry-passer-search")
        };
        if frame.source.card.definition != source
            || frame.ability_key != key
            || frame.source.card.controller != frame.actor
            || frame.source.card.face_down
            || frame.actor >= self.players.len()
            || frame.source.card.owner >= self.players.len()
            || frame.source.card.id.is_empty()
            || frame.source.region.is_none()
            || !frame
                .source
                .source_region_instance
                .as_ref()
                .is_some_and(|s| !s.is_empty())
            || !matches!(frame.guard, GuardState::Accepted)
            || frame.cursor != 1
            || frame.steps.len() != 1
            || frame.steps[0].context != frame.actor
            || !(if employee {
                matches!(
                    frame.steps[0].op,
                    Op::BQ104SearchEmployeeHiddenInSourceRegion
                )
            } else {
                matches!(frame.steps[0].op, Op::XQ48SearchPassersIntoSourceRegion)
            })
            || !frame.targets.is_empty()
            || !frame.already_paid.is_empty()
        {
            return Err("进场检索来源、程序或冻结行动者不符".into());
        }
        Ok(frame.actor)
    }
    fn entry_search_region(&self, frame: &ResolutionFrame) -> Option<usize> {
        let r = frame.source.region?;
        self.regions
            .get(r)
            .filter(|region| Some(&region.card.id) == frame.source.source_region_instance.as_ref())
            .map(|_| r)
    }
    fn entry_search_candidates(&self, actor: usize, employee: bool) -> RuleResult<Vec<&Card>> {
        let mut seen = BTreeSet::new();
        let mut candidates = Vec::new();
        for c in &self.players[actor].deck {
            let d = catalog::catalog()
                .cards
                .iter()
                .find(|d| d.id == c.definition)
                .ok_or("进场检索牌库含未准入定义")?;
            if c.id.is_empty() || c.owner >= self.players.len() || !seen.insert(&c.id) {
                return Err("进场检索牌库实例或拥有者无效".into());
            }
            if if employee {
                bq104_search_match(d)
            } else {
                xq48_search_match(d)
            } {
                candidates.push(c);
            }
        }
        Ok(candidates)
    }
    pub(crate) fn validate_entry_search_choice(&self) -> RuleResult<()> {
        let Some(p) = &self.pending else {
            return Ok(());
        };
        let framed = matches!(
            &p.resolution,
            ChoiceResolution::Frame {
                choice: FrameChoice::BQ104EmployeeSearch | FrameChoice::XQ48PasserSearch,
                ..
            }
        );
        if !framed
            && p.choice.kind != "bq104_employee_search"
            && p.choice.kind != "xq48_passer_search"
        {
            return Ok(());
        }
        let ChoiceResolution::Frame { frame, choice } = &p.resolution else {
            return Err("进场检索待选程序无效".into());
        };
        let employee = match choice {
            FrameChoice::BQ104EmployeeSearch => true,
            FrameChoice::XQ48PasserSearch => false,
            _ => return Err("进场检索待选程序无效".into()),
        };
        let actor = self.entry_search_actor(frame, employee)?;
        let candidates = self.entry_search_candidates(actor, employee)?;
        let expected = candidates
            .iter()
            .map(|c| self.option(c, actor, None, None))
            .collect::<Vec<_>>();
        let min = if employee && !candidates.is_empty() {
            1
        } else {
            0
        };
        let max = if employee {
            min
        } else {
            candidates.len().min(3)
        };
        if self.players[actor].eliminated
            || p.seat != actor
            || p.choice.player_id != format!("p{actor}")
            || p.choice.kind
                != if employee {
                    "bq104_employee_search"
                } else {
                    "xq48_passer_search"
                }
            || p.choice.min != Some(min)
            || p.choice.max != Some(max)
            || p.choice.allow_decline != Some(false)
            || serde_json::to_value(&p.choice.options).unwrap()
                != serde_json::to_value(expected).unwrap()
        {
            return Err("进场检索待选快照与行动者牌库不符".into());
        }
        Ok(())
    }
    pub(crate) fn entry_search_start(
        &mut self,
        frame: ResolutionFrame,
        employee: bool,
    ) -> RuleResult<()> {
        let actor = self.entry_search_actor(&frame, employee)?;
        if self.players[actor].eliminated {
            return Ok(());
        }
        // Owner-approved XQ48-only boundary: skip search, but still shuffle.
        if !employee && self.entry_search_region(&frame).is_none() {
            self.shuffle_player(actor);
            self.note(format!(
                "{} 的街头演说家原地区已离场，完成洗牌",
                self.players[actor].name
            ));
            return Ok(());
        }
        let candidates = self.entry_search_candidates(actor, employee)?;
        let min = if employee && !candidates.is_empty() {
            1
        } else {
            0
        };
        let max = if employee {
            min
        } else {
            candidates.len().min(3)
        };
        let options = candidates
            .iter()
            .map(|c| self.option(c, actor, None, None))
            .collect();
        // Empty searches retain an actor-private, explicitly completable choice.
        self.choice(
            actor,
            if employee {
                "bq104_employee_search"
            } else {
                "xq48_passer_search"
            },
            if employee {
                "检索一张费用不超过3的雇员，展示后暗藏进场并洗牌"
            } else {
                "检索零至三张无知路人置入原地区并洗牌"
            }
            .into(),
            options,
            min,
            max,
            None,
            ChoiceResolution::Frame {
                frame: Box::new(frame),
                choice: if employee {
                    FrameChoice::BQ104EmployeeSearch
                } else {
                    FrameChoice::XQ48PasserSearch
                },
            },
        );
        self.pending.as_mut().unwrap().choice.allow_decline = Some(false);
        Ok(())
    }
    pub(crate) fn entry_search_complete(
        &mut self,
        frame: &ResolutionFrame,
        selected: &[String],
        employee: bool,
    ) -> RuleResult<()> {
        let actor = self.entry_search_actor(frame, employee)?;
        if self.players[actor].eliminated {
            return Err("检索行动者已退出".into());
        }
        let candidates = self.entry_search_candidates(actor, employee)?;
        let eligible = candidates
            .iter()
            .map(|c| c.id.as_str())
            .collect::<BTreeSet<_>>();
        let min = if employee && !eligible.is_empty() {
            1
        } else {
            0
        };
        let max = if employee { min } else { eligible.len().min(3) };
        if selected.len() < min
            || selected.len() > max
            || selected.iter().collect::<BTreeSet<_>>().len() != selected.len()
            || selected.iter().any(|id| !eligible.contains(id.as_str()))
        {
            return Err("进场检索选择数量或牌库实例不符".into());
        }
        let region = self.entry_search_region(frame);
        if employee {
            if let Some(id) = selected.first() {
                let i = self.players[actor]
                    .deck
                    .iter()
                    .position(|c| &c.id == id)
                    .unwrap();
                self.note(format!(
                    "{} 展示检索的 {}",
                    self.players[actor].name,
                    card(&self.players[actor].deck[i].definition).name
                ));
                // Search/reveal precede placement. If placement is impossible,
                // retain the selected card in its deck and perform the shuffle.
                // The XQ48 skip-search ruling is not generalized to this card.
                if let Some(r) = region {
                    let c = self.players[actor].deck.remove(i);
                    let mut c = self.reset_zone_card(c);
                    c.controller = actor;
                    c.face_down = true;
                    self.regions[r].cards.push(c);
                    // Concealed entrants have no printed character abilities.
                }
            }
        } else if let Some(r) = region {
            let mut entered = Vec::new();
            for id in selected {
                let i = self.players[actor]
                    .deck
                    .iter()
                    .position(|c| &c.id == id)
                    .unwrap();
                let c = self.players[actor].deck.remove(i);
                let mut c = self.reset_zone_card(c);
                c.controller = actor;
                entered.push((c.definition.clone(), c.id.clone()));
                self.regions[r].cards.push(c);
            }
            // Insert the whole batch before queuing normal face-up Enter events.
            for (definition, id) in entered {
                self.enter_triggers(actor, &definition, &id, false);
            }
        }
        self.shuffle_player(actor);
        self.note(format!(
            "{} 完成{}检索并洗牌",
            self.players[actor].name,
            if employee {
                "猎头顾问"
            } else {
                "街头演说家"
            }
        ));
        Ok(())
    }
}
