//! Third twenty-card batch under development. Entries remain disabled for deck
//! construction until the complete batch is ready for its publication checks.
use crate::{
    catalog,
    engine::RuleResult,
    model::*,
    rules::{self, *},
};
use std::collections::BTreeSet;

pub(crate) const IMPLEMENTING: &[&str] = &["WM030", "BQ042"];

fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}

pub(crate) fn definition(id: &str) -> Option<Definition> {
    match id {
        "BQ042" => {
            let mut target = rules::target(
                Zone::Board,
                EntityKind::Attachment,
                Relation::Any,
                Range::Anywhere,
            );
            target.predicate = Some(TargetPredicate::BQ042SocietyOrOriginalRegion);
            let mut action = rules::ability(
                "batch3-bq042-destroy",
                "行动：移除自身1个时间标志，消灭结附于本地区或秘社的目标附属，每回合一次",
                Timing::Standard,
                vec![Cost::Batch3RemoveTime],
                vec![target],
                vec![Op::BQ042DestroyAttachment],
                None,
            );
            action.activation_only = true;
            action.per_turn_limit = Some(1);
            Some(rules::with_abilities(vec![action]))
        }
        "WM030" => Some(Definition {
            modifiers: vec![StaticModifier::WM030CultistIcons],
            ..Definition::default()
        }),
        _ => None,
    }
}

pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if let Some(expected) = definition(id) {
        if !same(d, &expected) {
            return Err("第三批须保留完整固定定义".into());
        }
    } else if d
        .modifiers
        .iter()
        .any(|m| matches!(m, StaticModifier::WM030CultistIcons))
        || d.abilities.iter().any(related_ability)
    {
        return Err("第三批有限程序不能移植".into());
    }
    Ok(())
}

pub(crate) fn related_ability(a: &AbilitySpec) -> bool {
    a.key.starts_with("batch3-")
        || uses(&a.ops)
        || a.modes.iter().any(|m| uses(&m.ops))
        || a.costs.iter().any(|c| matches!(c, Cost::Batch3RemoveTime))
        || a.targets
            .iter()
            .any(|t| t.predicate == Some(TargetPredicate::BQ042SocietyOrOriginalRegion))
}
fn uses(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::BQ042DestroyAttachment => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => uses(body),
        Op::IfTargetExhausted {
            exhausted, ready, ..
        } => uses(std::slice::from_ref(exhausted)) || uses(std::slice::from_ref(ready)),
        _ => false,
    })
}
fn related(f: &ResolutionFrame) -> bool {
    IMPLEMENTING.contains(&f.source.card.definition.as_str())
        || f.ability_key.starts_with("batch3-")
        || f.steps.iter().any(|s| uses(std::slice::from_ref(&s.op)))
        || f.already_paid
            .iter()
            .any(|p| matches!(p, PaidCost::Batch3RemovedTime(_)))
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if related_ability(a)
        && !definition(id).is_some_and(|d| d.abilities.iter().any(|expected| same(expected, a)))
    {
        return Err("第三批能力的来源、程序、费用或目标被修改".into());
    }
    Ok(())
}
pub(crate) fn validate_action(id: &str, a: &Action) -> RuleResult<()> {
    if !IMPLEMENTING.contains(&id) {
        return Ok(());
    }
    let expected = match a.kind.as_str() {
        "deploy" | "conceal" => Action {
            card_id: a.card_id.clone(),
            region: a.region,
            ..Action::new(&a.kind)
        },
        "reveal" => Action {
            card_id: a.card_id.clone(),
            ..Action::new("reveal")
        },
        "activate" if id == "BQ042" => Action {
            card_id: a.card_id.clone(),
            ability_id: a.ability_id.clone(),
            target_id: a.target_id.clone(),
            cost_selected: a.cost_selected.clone(),
            ..Action::new("activate")
        },
        _ => return Err("第三批角色操作种类不符".into()),
    };
    if *a != expected {
        return Err("第三批声明包含非准入载荷".into());
    }
    Ok(())
}

impl Game {
    pub(crate) fn current_spirit(&self, c: &Card) -> bool {
        !c.face_down
            && (rules::definition(&c.definition).traits.spirit
                || (c.definition == "BQ042"
                    && self
                        .players
                        .get(c.controller)
                        .is_some_and(|p| p.society_zone.secrecy > 0)))
    }
    pub(crate) fn batch3_entry_keywords(&mut self, definition: &str, id: &str) {
        if definition == "BQ042" {
            if let Some(c) = self.board_mut(id).filter(|c| !c.face_down) {
                c.time_markers = c.time_markers.saturating_add(1);
            }
        }
    }
    pub(crate) fn batch3_pay_time(
        &mut self,
        source: &SourceSnapshot,
        action: &Action,
    ) -> RuleResult<PaidCost> {
        match self.wm034_pay_time(source, action)? {
            PaidCost::SourceRemovedTime(payment) => Ok(PaidCost::Batch3RemovedTime(payment)),
            _ => Err("时间支付未生成真实载体收据".into()),
        }
    }
    fn batch3_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        let c = &s.card;
        if !IMPLEMENTING.contains(&c.definition.as_str())
            || c.id.is_empty()
            || actor >= self.players.len()
            || c.controller != actor
            || c.owner >= self.players.len()
            || s.region.is_none_or(|r| r >= self.regions.len())
            || s.source_region_instance
                .as_ref()
                .is_none_or(|id| id.is_empty() || id == &c.id)
            || s.attachment_host_instance.is_some()
            || s.observed_death.is_some()
            || s.observed_mobility_grant.is_some()
            || s.observed_exposure.is_some()
            || s.region
                .zip(s.source_region_instance.as_ref())
                .is_some_and(|(r, id)| {
                    self.regions
                        .iter()
                        .enumerate()
                        .any(|(other, region)| region.card.id == *id && r != other)
                })
        {
            return Err("第三批冻结来源、操控者或原地区身份不符".into());
        }
        self.mobility_immutable_identity(c)
    }
    pub(crate) fn validate_batch3_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if self.validated_cross_card_declaration(d)? {
            return Ok(());
        }
        if IMPLEMENTING.contains(&d.source.card.definition.as_str()) || related_ability(&d.ability)
        {
            return Err("第三批当前准入角色没有可选触发声明".into());
        }
        Ok(())
    }
    pub(crate) fn validate_batch3_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if self.validated_cross_card_frame(f)? || !related(f) {
            return Ok(());
        }
        self.batch3_source(f.actor, &f.source)?;
        if f.frame_id.is_empty()
            || f.cursor > f.steps.len()
            || (f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked))
            || (f.cursor > 0 && !matches!(f.guard, GuardState::Accepted))
        {
            return Err("第三批帧身份、守卫或游标不符".into());
        }
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            let paid = matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if ids.len() <= catalog::card(&f.source.card.definition).cost as usize
                && ids.iter().all(|id| !id.is_empty()) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len());
            if !f.steps.is_empty()
                || !f.targets.is_empty()
                || !paid
                || f.chosen_region != f.source.region
                || (f.ability_key == "deploy"
                    && (f.source.card.face_down || !f.source.valid_character_play()))
                || (f.ability_key == "reveal"
                    && (!f.source.card.face_down || f.source.play_source.is_some()))
            {
                return Err("第三批出场帧或支付不符".into());
            }
            return Ok(());
        }
        let a = definition(&f.source.card.definition)
            .and_then(|d| d.abilities.into_iter().find(|a| a.key == f.ability_key))
            .ok_or("非准入第三批执行帧")?;
        let steps = a
            .ops
            .iter()
            .map(|op| Step {
                context: f.actor,
                op: op.clone(),
            })
            .collect::<Vec<_>>();
        if !same(&steps, &f.steps)
            || f.source.card.face_down
            || f.source.play_source.is_some()
            || f.chosen_region.is_some()
            || f.targets.len() != a.targets.len()
            || f.targets.iter().zip(&a.targets).any(|(t, spec)| {
                !same(&t.spec, spec)
                    || t.id.is_empty()
                    || t.public.instance_id != t.id
                    || !t.public.valid
                    || t.public.status != "valid"
                    || t.public.invalid_reason.is_some()
                    || t.region_instance.is_some()
                    || t.public.kind != "attachment"
                    || t.public
                        .owner
                        .as_deref()
                        .is_none_or(|id| !self.players.iter().any(|p| player_id(p.seat) == id))
                    || t.public
                        .controller
                        .as_deref()
                        .is_none_or(|id| !self.players.iter().any(|p| player_id(p.seat) == id))
            })
        {
            return Err("第三批程序、冻结目标或执行环境不符".into());
        }
        let [PaidCost::Batch3RemovedTime(p)] = f.already_paid.as_slice() else {
            return Err("第三批缺少时间支付收据".into());
        };
        let carrier_definition = catalog::catalog()
            .cards
            .iter()
            .find(|d| d.id == p.carrier.definition)
            .ok_or("第三批时间费用载体不是已登记卡牌")?;
        if p.carrier.time_markers.checked_sub(1) != Some(p.remaining)
            || p.region != f.source.region
            || p.carrier.face_down
            || p.carrier.owner >= self.players.len()
            || p.carrier.controller >= self.players.len()
            || p.carrier.id.is_empty()
            || (p.carrier.id == f.source.card.id
                && (p.attachment_host.is_some() || !same(&p.carrier, &f.source.card)))
            || (p.carrier.id != f.source.card.id
                && (p.attachment_host.as_ref() != Some(&f.source.card.id)
                    || carrier_definition.kind != "attachment"))
        {
            return Err("第三批时间费用的数量或真实载体不符".into());
        }
        self.mobility_immutable_identity(&p.carrier)
    }
    pub(crate) fn validate_batch3_state(&self) -> RuleResult<()> {
        let mut frames = BTreeSet::new();
        for s in &self.stack {
            if let Some(f) = &s.frame {
                self.validate_batch3_frame(f)?;
                if !related(f) || self.validated_cross_card_frame(f)? {
                    continue;
                }
                let deploy = matches!(f.ability_key.as_str(), "deploy" | "reveal");
                if !frames.insert(&f.frame_id)
                    || s.id != f.frame_id
                    || s.controller != f.actor
                    || s.target != f.targets.first().map(|t| t.id.clone())
                    || f.cursor != 0
                    || !matches!(f.guard, GuardState::Unchecked)
                    || deploy != s.card.is_some()
                    || s.card.as_ref().is_some_and(|c| {
                        c.definition != f.source.card.definition
                            || c.owner != f.source.card.owner
                            || c.controller != f.actor
                            || c.face_down
                    })
                    || s.reveal != (f.ability_key == "reveal")
                    || s.deploy_region != (if deploy { f.chosen_region } else { None })
                {
                    return Err("第三批响应堆叠外壳或实体不符".into());
                }
            } else if s
                .card
                .as_ref()
                .is_some_and(|c| IMPLEMENTING.contains(&c.definition.as_str()))
            {
                return Err("第三批堆叠实体缺少执行帧".into());
            }
        }
        for e in &self.effects {
            match e {
                Effect::Declare { declaration } => self.validate_batch3_declaration(declaration)?,
                Effect::Frame { frame }
                    if related(frame) && !self.validated_cross_card_frame(frame)? =>
                {
                    return Err("第三批不能绕过响应堆叠".into())
                }
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Declare { declaration, .. } => {
                    self.validate_batch3_declaration(declaration)?
                }
                ChoiceResolution::Frame { frame, .. }
                    if related(frame) && !self.validated_cross_card_frame(frame)? =>
                {
                    return Err("第三批当前程序没有结算中选择".into())
                }
                _ => {}
            }
        }
        Ok(())
    }
    /// Both printed grants are permanent-category icons. Exhausting a source
    /// does not blank its continuous text. 本方 is the current controller seat.
    pub(crate) fn wm030_cultist_icons(&self, c: &Card, region: usize) -> Icons {
        if c.face_down || catalog::card(&c.definition).kind != "character" {
            return Icons::default();
        }
        let sources = self
            .regions
            .iter()
            .filter(|r| !r.vacant)
            .flat_map(|r| &r.cards)
            .filter(|source| {
                source.definition == "WM030"
                    && !source.face_down
                    && source.controller == c.controller
                    && rules::definition(&source.definition)
                        .modifiers
                        .iter()
                        .any(|m| matches!(m, StaticModifier::WM030CultistIcons))
            })
            .count() as u32;
        if sources == 0
            || !self
                .regions
                .get(region)
                .is_some_and(|r| !r.vacant && r.cards.iter().any(|live| live == c))
            || !self.current_subtypes(c).iter().any(|s| s == "邪教徒")
        {
            return Icons::default();
        }
        Icons {
            investigation: if self
                .players
                .get(c.controller)
                .is_some_and(|p| p.society_zone.secrecy >= 3)
            {
                sources
            } else {
                0
            },
            combat: 0,
            influence: if self.regions[region].destroyed {
                sources
            } else {
                0
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unit_support::*;

    #[test]
    fn batch3_wm030_dynamic_personal_cultist_grants_are_permanent_icons() {
        let mut g = game();
        let source = field(&mut g, "WM030", 0);
        let own = field(&mut g, "JC042", 0);
        let ally = field(&mut g, "JC042", 1);
        let enemy = field(&mut g, "JC042", 2);
        g.first_team = 1; // Temporary icons are off for seat zero.
        let before = g.current_icons(g.board(&own).unwrap().1, 0);
        g.regions[0].destroyed = true;
        g.add_society_markers(0, 0, 2);
        assert_eq!(
            g.wm030_cultist_icons(g.board(&own).unwrap().1, 0),
            Icons {
                influence: 1,
                ..Icons::default()
            }
        );
        g.add_society_markers(0, 0, 1);
        let after = g.current_icons(g.board(&own).unwrap().1, 0);
        assert_eq!(after.investigation, before.investigation + 1);
        assert_eq!(after.influence, before.influence + 1);
        assert_eq!(
            g.wm030_cultist_icons(g.board(&ally).unwrap().1, 0),
            Icons::default()
        );
        assert_eq!(
            g.wm030_cultist_icons(g.board(&enemy).unwrap().1, 0),
            Icons::default()
        );
        g.board_mut(&source).unwrap().exhausted = true;
        assert_eq!(
            g.wm030_cultist_icons(g.board(&own).unwrap().1, 0)
                .investigation,
            1
        );
        g.add_society_markers(0, 1, 0); // Exposure cancels one secrecy immediately.
        assert_eq!(
            g.wm030_cultist_icons(g.board(&own).unwrap().1, 0)
                .investigation,
            0
        );
        restore(&mut g);
    }

    #[test]
    fn batch3_wm030_sources_stack_but_hidden_sources_and_targets_do_not_qualify() {
        let mut g = game();
        let a = field(&mut g, "WM030", 0);
        let b = field(&mut g, "WM030", 0);
        g.regions[0].destroyed = true;
        let c = g.board(&a).unwrap().1.clone();
        assert_eq!(g.wm030_cultist_icons(&c, 0).influence, 2);
        g.board_mut(&b).unwrap().face_down = true;
        assert_eq!(g.wm030_cultist_icons(&c, 0).influence, 1);
        let mut hidden = c.clone();
        hidden.face_down = true;
        assert_eq!(g.wm030_cultist_icons(&hidden, 0), Icons::default());
        g.board_mut(&a).unwrap().controller = 1;
        assert_eq!(g.wm030_cultist_icons(&c, 0), Icons::default()); // stale target snapshot
        let live = g.board(&a).unwrap().1.clone();
        assert_eq!(g.wm030_cultist_icons(&live, 0).influence, 1);
        let mut moved = live.clone();
        moved.id = "not-on-board".into();
        assert_eq!(g.wm030_cultist_icons(&moved, 0), Icons::default());
    }

    #[test]
    fn batch3_wm030_program_cannot_be_modified_or_transplanted() {
        let d = definition("WM030").unwrap();
        validate_definition("WM030", &d).unwrap();
        assert!(validate_definition("JC125", &d).is_err());
        assert!(validate_definition("WM030", &Definition::default()).is_err());
        let mut definitions = rules::definitions().clone();
        definitions.insert("WM030".into(), Definition::default());
        assert!(rules::validate_definitions(&definitions).is_err());
    }

    #[test]
    fn batch3_bq042_entry_time_spirit_condition_and_exact_paid_action() {
        let mut g = game();
        fund(&mut g, 0, "JC047", 2);
        let source = held(&mut g, "BQ042", 0);
        act(
            &mut g,
            0,
            Action {
                card_id: Some(source.clone()),
                region: Some(0),
                ..Action::new("deploy")
            },
        );
        top(&mut g);
        let source = g.regions[0]
            .cards
            .iter()
            .find(|c| c.definition == "BQ042")
            .unwrap()
            .id
            .clone();
        assert_eq!(g.board(&source).unwrap().1.time_markers, 1);
        assert!(!g.current_spirit(g.board(&source).unwrap().1));
        g.add_society_markers(0, 0, 1);
        assert!(g.current_spirit(g.board(&source).unwrap().1));
        g.add_society_markers(0, 1, 0);
        assert!(!g.current_spirit(g.board(&source).unwrap().1));
        let attachment = g.make_card("JC048", 2);
        let target = attachment.id.clone();
        g.attachments.push(Attachment {
            card: attachment,
            host_id: g.regions[0].card.id.clone(),
        });
        let action = Action {
            card_id: Some(source.clone()),
            target_id: Some(target.clone()),
            cost_selected: Some(vec![source.clone()]),
            ..Action::new("activate")
        };
        act(&mut g, 0, action.clone());
        assert_eq!(g.board(&source).unwrap().1.time_markers, 0);
        let f = g.stack.last().unwrap().frame.as_ref().unwrap();
        assert!(
            matches!(f.already_paid.as_slice(), [PaidCost::Batch3RemovedTime(p)] if p.carrier.id == source && p.remaining == 0)
        );
        assert_eq!(f.targets[0].public.kind, "attachment");
        let mut forged = g.clone();
        if let PaidCost::Batch3RemovedTime(p) = &mut forged
            .stack
            .last_mut()
            .unwrap()
            .frame
            .as_mut()
            .unwrap()
            .already_paid[0]
        {
            p.remaining = 1;
        }
        assert!(Game::from_persisted(&serde_json::to_string(&forged).unwrap()).is_err());
        top(&mut g);
        assert!(!g.attachments.iter().any(|a| a.card.id == target));
        g.board_mut(&source).unwrap().time_markers = 1;
        reject(&mut g, 0, action); // Source instance has already used this turn.
    }

    #[test]
    fn batch3_bq042_rejects_character_hosts_and_other_regions_without_spending_time() {
        let mut g = game();
        let source = field(&mut g, "BQ042", 0);
        g.batch3_entry_keywords("BQ042", &source);
        let host = field(&mut g, "JC125", 2);
        let attachment = g.make_card("BQ022", 2);
        let wrong_host = attachment.id.clone();
        g.attachments.push(Attachment {
            card: attachment,
            host_id: host,
        });
        let attachment = g.make_card("JC048", 2);
        let wrong_region = attachment.id.clone();
        g.attachments.push(Attachment {
            card: attachment,
            host_id: g.regions[1].card.id.clone(),
        });
        for target in [wrong_host, wrong_region] {
            reject(
                &mut g,
                0,
                Action {
                    card_id: Some(source.clone()),
                    target_id: Some(target),
                    cost_selected: Some(vec![source.clone()]),
                    ..Action::new("activate")
                },
            );
        }
        assert_eq!(g.board(&source).unwrap().1.time_markers, 1);
        let mut defs = rules::definitions().clone();
        defs.get_mut("BQ042").unwrap().abilities[0].per_turn_limit = None;
        assert!(rules::validate_definitions(&defs).is_err());
        assert!(
            rules::validate_ability("JC125", &definition("BQ042").unwrap().abilities[0]).is_err()
        );
    }

    #[test]
    fn batch3_bq042_attached_time_can_pay_to_destroy_an_enemy_society_attachment() {
        let mut g = game();
        let source = field(&mut g, "BQ042", 0);
        let mut time = g.make_card("JZ41", 0);
        time.time_markers = 1;
        let carrier = time.id.clone();
        g.attachments.push(Attachment {
            card: time,
            host_id: source.clone(),
        });
        g.players[2].society_zone.card = Some(g.make_card("MSJC04", 2));
        let attachment = g.make_card("JC068", 2);
        let target = attachment.id.clone();
        g.attachments.push(Attachment {
            card: attachment,
            host_id: "society:p2".into(),
        });
        act(
            &mut g,
            0,
            Action {
                card_id: Some(source.clone()),
                target_id: Some(target.clone()),
                cost_selected: Some(vec![carrier.clone()]),
                ..Action::new("activate")
            },
        );
        assert_eq!(g.board_mut(&carrier).unwrap().time_markers, 0);
        let mut forged = g.clone();
        if let PaidCost::Batch3RemovedTime(p) = &mut forged
            .stack
            .last_mut()
            .unwrap()
            .frame
            .as_mut()
            .unwrap()
            .already_paid[0]
        {
            p.carrier.definition = "UNREGISTERED".into();
        }
        let invalid = serde_json::to_string(&forged).unwrap();
        assert!(std::panic::catch_unwind(|| Game::from_persisted(&invalid))
            .unwrap()
            .is_err());
        let mut transplanted = g.clone();
        transplanted
            .stack
            .last_mut()
            .unwrap()
            .frame
            .as_mut()
            .unwrap()
            .source
            .card
            .definition = "JC125".into();
        assert!(Game::from_persisted(&serde_json::to_string(&transplanted).unwrap()).is_err());
        top(&mut g);
        assert!(!g.attachments.iter().any(|a| a.card.id == target));
        assert!(g.attachments.iter().any(|a| a.card.id == carrier));
        restore(&mut g);
    }

    #[test]
    fn batch3_bq042_lost_target_cancels_without_refunding_time_or_limit() {
        let mut g = game();
        let source = field(&mut g, "BQ042", 0);
        g.batch3_entry_keywords("BQ042", &source);
        let attachment = g.make_card("JC048", 2);
        let target = attachment.id.clone();
        g.attachments.push(Attachment {
            card: attachment,
            host_id: g.regions[0].card.id.clone(),
        });
        act(
            &mut g,
            0,
            Action {
                card_id: Some(source.clone()),
                target_id: Some(target.clone()),
                cost_selected: Some(vec![source.clone()]),
                ..Action::new("activate")
            },
        );
        g.remove_dead(&target, RemovalCause::Sacrifice);
        g.drive().unwrap();
        top(&mut g);
        assert_eq!(g.board(&source).unwrap().1.time_markers, 0);
        assert!(g
            .turn_ability_usage
            .iter()
            .any(|u| u.source_instance == source && u.uses == 1));
        assert!(g
            .log
            .iter()
            .any(|entry| entry.text.contains("整个卡牌或能力效果取消")));
        restore(&mut g);
    }
}
