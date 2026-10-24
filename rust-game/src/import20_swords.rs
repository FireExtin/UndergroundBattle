//! CQ15: an exact attachment grants a paid action to its actual host.
//! The returned sword receipt survives the response window; no control is lent.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub(crate) const PREFIX: &str = "cq15-host:";
pub(crate) const SEARCH: &str = "cq15-three-zone-search";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SwordPayment {
    pub grant: Attachment,
    pub returned: Card,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SwordSearch {
    pub controller: usize,
    pub name: String,
    pub offered: Vec<String>,
}
fn same<T: Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
pub(crate) fn definition() -> Definition {
    let mut host = target(
        Zone::Board,
        EntityKind::Character,
        Relation::Any,
        Range::Anywhere,
    );
    host.equipment_host = true;
    let mut d = attachment_definition(host);
    d.attachment.as_mut().unwrap().host_icons.combat = 2;
    d
}
pub(crate) fn granted(g: &Attachment) -> AbilitySpec {
    let mut a = ability(
        &format!("{PREFIX}{}", g.card.id),
        "横置宿主，退回双剑：消灭角色，检索并封印至多三张同名角色",
        Timing::Standard,
        vec![
            Cost::ExhaustSource,
            Cost::CQ15ReturnSword(Box::new(g.clone())),
        ],
        vec![target(
            Zone::Board,
            EntityKind::Character,
            Relation::Any,
            Range::Anywhere,
        )],
        vec![Op::CQ15DestroySearch {
            grant: Box::new(g.clone()),
        }],
        None,
    );
    a.activation_only = true;
    a
}
fn uses(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::CQ15DestroySearch { .. } => true,
        Op::ForEachLivingPlayer(b) | Op::ForEachLivingPlayerFromActor(b) => uses(b),
        Op::IfTargetExhausted {
            exhausted, ready, ..
        } => uses(std::slice::from_ref(exhausted)) || uses(std::slice::from_ref(ready)),
        _ => false,
    })
}
pub(crate) fn related(a: &AbilitySpec) -> bool {
    a.key.starts_with(PREFIX)
        || uses(&a.ops)
        || a.modes.iter().any(|m| uses(&m.ops))
        || a.costs
            .iter()
            .any(|c| matches!(c, Cost::CQ15ReturnSword(_)))
}
fn frame_related(f: &ResolutionFrame) -> bool {
    f.ability_key.starts_with(PREFIX)
        || f.steps.iter().any(|s| uses(std::slice::from_ref(&s.op)))
        || f.already_paid
            .iter()
            .any(|p| matches!(p, PaidCost::CQ15Returned(_)))
}
pub(crate) fn validate_ability(a: &AbilitySpec) -> RuleResult<()> {
    let [Op::CQ15DestroySearch { grant }] = a.ops.as_slice() else {
        return Err("双剑授予能力须保留完整来源与消灭检索程序".into());
    };
    if grant.card.definition != "CQ15"
        || grant.card.face_down
        || grant.card.id.is_empty()
        || grant.host_id.is_empty()
        || grant.host_id == grant.card.id
        || !same(a, &granted(grant))
    {
        return Err("双剑授予程序、费用或原宿主无效".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if d.abilities.iter().any(related) {
        return Err("双剑宿主授予不能移植为静态能力".into());
    }
    if id == "CQ15" && !same(d, &definition()) {
        return Err("CQ15须保留印刷结附与永久战斗2".into());
    }
    Ok(())
}
impl Game {
    pub(crate) fn cq15_wound(&self, c: &Card) -> u32 {
        if !c.face_down
            && self.attachments.iter().any(|a| {
                a.card.definition == "CQ15"
                    && !a.card.face_down
                    && a.host_id == c.id
                    && self.attachment_host_valid(a)
            })
        {
            2
        } else {
            0
        }
    }
    pub(crate) fn cq15_actions(&self, actor: usize) -> Vec<(Action, String)> {
        let mut out = vec![];
        for g in self.attachments.iter().filter(|a| {
            a.card.definition == "CQ15" && !a.card.face_down && self.attachment_host_valid(a)
        }) {
            if self
                .in_play(&g.host_id)
                .is_none_or(|(_, c)| c.controller != actor || c.face_down || c.exhausted)
            {
                continue;
            }
            for (_, t) in self
                .in_play_cards()
                .into_iter()
                .filter(|(_, c)| !c.face_down && catalog::card(&c.definition).kind == "character")
            {
                out.push((
                    Action {
                        card_id: Some(g.host_id.clone()),
                        ability_id: Some(format!("{PREFIX}{}", g.card.id)),
                        target_id: Some(t.id.clone()),
                        ..Action::new("activate")
                    },
                    format!("双剑宿主能力 → {}", catalog::card(&t.definition).name),
                ));
            }
        }
        out
    }
    pub(crate) fn cq15_begin(&mut self, actor: usize, a: &Action) -> RuleResult<()> {
        if *a
            != (Action {
                card_id: a.card_id.clone(),
                ability_id: a.ability_id.clone(),
                target_id: a.target_id.clone(),
                ..Action::new("activate")
            })
        {
            return Err("双剑只声明宿主、实际双剑和角色目标".into());
        }
        let sword = a
            .ability_id
            .as_deref()
            .and_then(|k| k.strip_prefix(PREFIX))
            .ok_or("双剑来源无效")?;
        let g = self
            .attachments
            .iter()
            .find(|g| {
                g.card.id == sword
                    && g.card.definition == "CQ15"
                    && !g.card.face_down
                    && self.attachment_host_valid(g)
            })
            .ok_or("原双剑已不再有效结附")?
            .clone();
        if a.card_id.as_deref() != Some(g.host_id.as_str()) {
            return Err("只能由这把双剑的实际宿主发动".into());
        }
        let (r, c) = self
            .in_play(&g.host_id)
            .filter(|(_, c)| {
                c.controller == actor
                    && !c.face_down
                    && !c.exhausted
                    && catalog::card(&c.definition).kind == "character"
            })
            .ok_or("双剑须由自己未横置的正面角色宿主发动")?;
        let mut source = self.source_snapshot(c, r);
        source.source_region_instance = r.map(|r| self.regions[r].card.id.clone());
        let spec = granted(&g);
        self.check_timing(actor, &spec)?;
        let targets = self.bind_action(actor, &source, &spec, a)?;
        let paid = self.pay_ability_costs(actor, &source, &spec, a)?;
        let f = self.make_frame(actor, source, &spec, targets, paid, None);
        self.dispatch_frame(
            f,
            "元屠、阿鼻双剑：宿主授予能力".into(),
            None,
            spec.response_policy,
        );
        Ok(())
    }
    pub(crate) fn cq15_return_cost(
        &mut self,
        actor: usize,
        s: &SourceSnapshot,
        g: &Attachment,
    ) -> RuleResult<PaidCost> {
        if g.host_id != s.card.id
            || s.card.controller != actor
            || g.card.definition != "CQ15"
            || !self
                .attachments
                .iter()
                .any(|a| same(a, g) && self.attachment_host_valid(a))
            || self.players[g.card.owner].eliminated
        {
            return Err("退回费用必须是授予原宿主能力的同一把双剑".into());
        }
        let (_, c) = self.leave_in_play(&g.card.id).ok_or("原双剑无法退回")?;
        let returned = self.reset_zone_card(c);
        self.players[returned.owner].hand.push(returned.clone());
        Ok(PaidCost::CQ15Returned(Box::new(SwordPayment {
            grant: g.clone(),
            returned,
        })))
    }
    pub(crate) fn cq15_validated_frame(&self, f: &ResolutionFrame) -> RuleResult<bool> {
        if !frame_related(f) {
            return Ok(false);
        }
        let [Step {
            context,
            op: Op::CQ15DestroySearch { grant },
        }] = f.steps.as_slice()
        else {
            return Err("双剑帧须保留完整单步消灭检索程序".into());
        };
        let a = granted(grant);
        validate_ability(&a)?;
        let c = &f.source.card;
        if f.actor >= self.players.len()
            || c.controller != f.actor
            || c.owner >= self.players.len()
            || c.face_down
            || c.exhausted
            || c.id != grant.host_id
            || *context != f.actor
            || f.ability_key != a.key
            || f.frame_id.is_empty()
            || f.cursor > 1
            || f.chosen_region.is_some()
            || f.source.play_source.is_some()
            || !catalog::catalog()
                .cards
                .iter()
                .any(|d| d.id == c.definition && d.kind == "character")
            || f.targets.len() != 1
            || !same(&f.targets[0].spec, &a.targets[0])
            || f.targets[0].id.is_empty()
            || f.targets[0].public.instance_id != f.targets[0].id
            || f.targets[0].public.kind != "character"
            || f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked)
            || f.cursor == 1 && !matches!(f.guard, GuardState::Accepted)
        {
            return Err("双剑帧宿主、目标或结算阶段无效".into());
        }
        let [PaidCost::Exhausted(id), PaidCost::CQ15Returned(p)] = f.already_paid.as_slice() else {
            return Err("双剑帧须保留宿主横置和双剑退回费用".into());
        };
        let mut expected = grant.card.clone();
        expected.id = p.returned.id.clone();
        expected.controller = expected.owner;
        expected.face_down = false;
        expected.exhausted = false;
        expected.damage = 0;
        expected.wounds = 0;
        expected.shield = 0;
        expected.advancement_markers = 0;
        expected.named_character = None;
        expected.chosen_number = None;
        expected.chosen_subtype = None;
        expected.time_markers = 0;
        expected.charge_markers = 0;
        expected.permanent_combat_markers = 0;
        expected.permanent_defense_markers = 0;
        expected.lock_markers = 0;
        if !same(&p.returned, &expected)
            || id != &c.id
            || !same(&p.grant, grant)
            || p.returned.id.is_empty()
            || p.returned.id == grant.card.id
            || p.returned.definition != "CQ15"
            || p.returned.owner != grant.card.owner
            || p.returned.controller != p.returned.owner
            || p.returned.face_down
            || p.returned.exhausted
            || p.returned.damage != 0
            || p.returned.wounds != 0
            || p.returned.shield != 0
            || !f.resource_payments.is_empty()
        {
            return Err("双剑退回凭据与原附属不符".into());
        }
        Ok(true)
    }
    fn cq15_search_ids(&self, s: &SwordSearch) -> Vec<String> {
        let p = &self.players[s.controller];
        p.deck
            .iter()
            .chain(&p.hand)
            .chain(&p.graveyard)
            .filter(|c| {
                catalog::card(&c.definition).kind == "character"
                    && catalog::card(&c.definition).name == s.name
            })
            .map(|c| c.id.clone())
            .collect()
    }
    fn cq15_search_options(&self, actor: usize, s: &SwordSearch) -> Vec<ChoiceOption> {
        let p = &self.players[s.controller];
        s.offered
            .iter()
            .filter_map(|id| {
                p.deck
                    .iter()
                    .chain(&p.hand)
                    .chain(&p.graveyard)
                    .find(|c| &c.id == id)
            })
            .map(|c| self.option(c, actor, None, None))
            .collect()
    }
    pub(crate) fn cq15_destroy_search(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        self.cq15_validated_frame(&f)?;
        if f.cursor != 1 || !matches!(f.guard, GuardState::Accepted) {
            return Err("双剑须通过正常响应与目标守卫".into());
        }
        let Some((_, c)) = self.in_play(&f.targets[0].id) else {
            return Ok(());
        };
        let mut search = SwordSearch {
            controller: c.controller,
            name: catalog::card(&c.definition).name.clone(),
            offered: vec![],
        };
        self.remove_dead(&f.targets[0].id, RemovalCause::Destroy);
        if self.players[f.actor].eliminated || self.players[search.controller].eliminated {
            return Ok(());
        }
        search.offered = self.cq15_search_ids(&search);
        let opts = self.cq15_search_options(f.actor, &search);
        let max = 3.min(opts.len());
        self.choice(
            f.actor,
            SEARCH,
            "双剑：从原操控者牌库、手牌和墓地封印至多三张同名角色".into(),
            opts,
            0,
            max,
            None,
            ChoiceResolution::Frame {
                frame: Box::new(f),
                choice: FrameChoice::CQ15Search { search },
            },
        );
        Ok(())
    }
    pub(crate) fn cq15_search_finish(
        &mut self,
        f: &ResolutionFrame,
        s: &SwordSearch,
        chooser: usize,
        a: &Action,
        selected: &[String],
    ) -> RuleResult<()> {
        if !self.cq15_validated_frame(f)? {
            return Err("双剑检索不能引用其他程序".into());
        }
        if chooser != f.actor
            || f.cursor != 1
            || s.controller >= self.players.len()
            || selected.len() > 3
            || s.offered != self.cq15_search_ids(s)
            || selected.iter().any(|id| !s.offered.contains(id))
            || selected.iter().collect::<BTreeSet<_>>().len() != selected.len()
            || *a
                != (Action {
                    choice_id: a.choice_id.clone(),
                    selected: a.selected.clone(),
                    ..Action::new("choose")
                })
        {
            return Err("双剑检索须由原发动者在冻结三区中选择至多三张".into());
        }
        let host = format!("society:p{}", f.actor);
        for id in selected {
            let p = &mut self.players[s.controller];
            let c = if let Some(i) = p.deck.iter().position(|c| &c.id == id) {
                p.deck.remove(i)
            } else if let Some(i) = p.hand.iter().position(|c| &c.id == id) {
                p.hand.remove(i)
            } else {
                let i = p
                    .graveyard
                    .iter()
                    .position(|c| &c.id == id)
                    .ok_or("双剑检索实体已离开原区")?;
                p.graveyard.remove(i)
            };
            self.seal_card_on_valid_host(c, f.actor, &host);
        }
        self.shuffle_player(s.controller);
        Ok(())
    }
    pub(crate) fn validate_cq15_state(&self) -> RuleResult<()> {
        for item in &self.stack {
            if let Some(f) = &item.frame {
                if self.cq15_validated_frame(f)?
                    && (f.cursor != 0
                        || item.id != f.frame_id
                        || item.controller != f.actor
                        || item.card.is_some()
                        || item.reveal
                        || item.deploy_region.is_some()
                        || item.target != f.targets.first().map(|t| t.id.clone()))
                {
                    return Err("双剑响应容器无效".into());
                }
            }
        }
        for e in &self.effects {
            if let Effect::Frame { frame } = e {
                self.cq15_validated_frame(frame)?;
            }
        }
        if let Some(p) = &self.pending {
            if let ChoiceResolution::Frame {
                frame,
                choice: FrameChoice::CQ15Search { search },
            } = &p.resolution
            {
                if !self.cq15_validated_frame(frame)? {
                    return Err("双剑检索必须引用原授予帧".into());
                }
                if p.seat != frame.actor
                    || frame.cursor != 1
                    || search.controller >= self.players.len()
                    || p.choice.kind != SEARCH
                    || p.choice.min != Some(0)
                    || p.choice.max != Some(3.min(search.offered.len()))
                    || search.offered != self.cq15_search_ids(search)
                    || !same(
                        &p.choice.options,
                        &self.cq15_search_options(frame.actor, search),
                    )
                {
                    return Err("双剑三区检索续体与选项不符".into());
                }
            }
        }
        Ok(())
    }
}
