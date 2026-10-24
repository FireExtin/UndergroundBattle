//! CQ28 unambiguous repeated-same modes. Mixed-order modes await a ruling.
//! All modes/targets are declared before the existing printed-cost transaction.
use crate::{engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
pub(crate) const KEY: &str = "cq28-two-modes";
pub(crate) const DRAFT: &str = "cq28-before-payment";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateDefense {
    pub frame: Box<ResolutionFrame>,
    pub slot: usize,
    pub turn: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateDraft {
    pub actor: usize,
    pub source: Card,
    pub mode: Option<u8>,
    pub targets: Vec<String>,
}
fn same<T: Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
fn mode(n: u8) -> Mode {
    let labels = [
        "将顶六张置墓，两次",
        "从墓地放置费用不大于2角色，两次",
        "角色防御-1，两次",
        "抓一张，两次",
    ];
    let mut targets = vec![];
    let mut ops = vec![];
    for _ in 0..2 {
        let start = targets.len();
        match n {
            1 => {
                let mut grave = target(
                    Zone::Graveyard,
                    EntityKind::Character,
                    Relation::OwnedByActor,
                    Range::Anywhere,
                );
                grave.printed_cost_max = Some(2);
                targets.push(grave);
                targets.push(target(
                    Zone::Region,
                    EntityKind::Any,
                    Relation::Any,
                    Range::Anywhere,
                ));
            }
            2 => targets.push(target(
                Zone::Board,
                EntityKind::Character,
                Relation::Any,
                Range::Anywhere,
            )),
            _ => {}
        }
        ops.push(Op::CQ28Mode {
            mode: n,
            slot: start,
        });
    }
    Mode {
        key: format!("repeat-{n}"),
        label: labels[n as usize].into(),
        targets,
        ops,
    }
}
pub(crate) fn definition() -> Definition {
    let mut a = ability(
        KEY,
        "选择两项（当前只开放同项重复；混合顺序待裁定）",
        Timing::Standard,
        vec![],
        vec![],
        vec![],
        None,
    );
    a.play_only = true;
    a.modes = (0..4).map(mode).collect();
    with_abilities(vec![a])
}
fn expanded(n: u8) -> AbilitySpec {
    let m = mode(n);
    let mut a = definition().abilities.remove(0);
    a.targets = m.targets;
    a.ops = m.ops;
    a.modes.clear();
    a
}
fn uses(ops: &[Op]) -> bool {
    ops.iter().any(|o| match o {
        Op::CQ28Mode { .. } => true,
        Op::ForEachLivingPlayer(b) | Op::ForEachLivingPlayerFromActor(b) => uses(b),
        Op::IfTargetExhausted {
            exhausted, ready, ..
        } => uses(std::slice::from_ref(exhausted)) || uses(std::slice::from_ref(ready)),
        _ => false,
    })
}
pub(crate) fn related(id: &str, a: &AbilitySpec) -> bool {
    id == "CQ28" || a.key == KEY || uses(&a.ops) || a.modes.iter().any(|m| uses(&m.ops))
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if id != "CQ28"
        || (!same(a, &definition().abilities[0]) && !(0..4).any(|n| same(a, &expanded(n))))
    {
        return Err("咒欲之门只准入两次完整同项；混合模式顺序仍待明确".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if (id == "CQ28" || d.abilities.iter().any(|a| related(id, a)))
        && (id != "CQ28" || !same(d, &definition()))
    {
        return Err("咒欲之门不能拆分、改写或移植".into());
    }
    Ok(())
}
impl Game {
    fn cq28_target_ids(
        &self,
        actor: usize,
        source: &SourceSnapshot,
        slot: &TargetSlotSpec,
    ) -> Vec<String> {
        let ids = match slot.zone {
            Zone::Region => (0..self.regions.len())
                .map(|r| format!("region:{r}"))
                .collect(),
            Zone::Graveyard => self.players[actor]
                .graveyard
                .iter()
                .map(|c| c.id.clone())
                .collect(),
            _ => self
                .in_play_cards()
                .iter()
                .map(|(_, c)| c.id.clone())
                .collect::<Vec<_>>(),
        };
        ids.into_iter()
            .filter(|id| self.valid_binding(actor, source, slot, id))
            .collect()
    }
    fn cq28_source(&self, d: &GateDraft) -> RuleResult<SourceSnapshot> {
        if d.actor >= self.players.len()
            || d.source.definition != "CQ28"
            || d.source.owner != d.actor
            || d.source.controller != d.actor
            || d.source.face_down
            || !self.players[d.actor].hand.contains(&d.source)
            || d.mode.is_some_and(|n| n > 3)
        {
            return Err("咒欲之门须由原手牌持有者先声明模式目标".into());
        }
        Ok(self.source_snapshot(&d.source, None))
    }
    fn cq28_options(&self, d: &GateDraft) -> RuleResult<Vec<ChoiceOption>> {
        let s = self.cq28_source(d)?;
        if let Some(n) = d.mode {
            let a = expanded(n);
            if d.targets.len() >= a.targets.len() {
                return Err("咒欲之门目标声明已完成".into());
            }
            let slot = &a.targets[d.targets.len()];
            return Ok(self
                .cq28_target_ids(d.actor, &s, slot)
                .into_iter()
                .map(|id| ChoiceOption {
                    id: id.clone(),
                    label: self.public_target(d.actor, &s, slot, &id).label,
                    card: if slot.zone == Zone::Graveyard {
                        self.players[d.actor]
                            .graveyard
                            .iter()
                            .find(|c| c.id == id)
                            .map(|c| self.card_view(c, d.actor, None, None))
                    } else {
                        None
                    },
                })
                .collect());
        }
        Ok((0..4)
            .filter(|n| {
                expanded(*n)
                    .targets
                    .iter()
                    .all(|slot| !self.cq28_target_ids(d.actor, &s, slot).is_empty())
            })
            .map(|n| ChoiceOption {
                id: format!("repeat-{n}"),
                label: mode(n).label,
                card: None,
            })
            .collect())
    }
    fn cq28_prompt(&mut self, d: GateDraft) -> RuleResult<()> {
        let options = self.cq28_options(&d)?;
        self.choice(
            d.actor,
            DRAFT,
            if d.mode.is_none() {
                "咒欲之门：选择重复同项（混合模式暂未开放）".into()
            } else {
                format!("咒欲之门：支付前选择第{}个目标", d.targets.len() + 1)
            },
            options,
            1,
            1,
            None,
            ChoiceResolution::CQ28Draft { draft: Box::new(d) },
        );
        Ok(())
    }
    pub(crate) fn cq28_begin(&mut self, actor: usize, c: &Card, a: &Action) -> RuleResult<()> {
        if *a
            != (Action {
                card_id: Some(c.id.clone()),
                ..Action::new("play")
            })
        {
            return Err("咒欲之门先通过模式和目标声明选择".into());
        }
        self.check_timing(actor, &definition().abilities[0])?;
        if self.xq22_spell_banned() {
            return Err("州议员禁止打出事务牌".into());
        }
        self.xg20_check_play(actor, c)?;
        if !self.loyalty(actor, "CQ28") {
            return Err("咒欲之门需要四黑忠诚".into());
        }
        let mut trial = self.clone();
        trial.pay_printed(actor, c, true)?;
        self.cq28_prompt(GateDraft {
            actor,
            source: c.clone(),
            mode: None,
            targets: vec![],
        })
    }
    pub(crate) fn cq28_choose(
        &mut self,
        mut d: GateDraft,
        actor: usize,
        a: &Action,
        selected: &[String],
    ) -> RuleResult<()> {
        self.cq28_source(&d)?;
        if actor != d.actor
            || selected.len() != 1
            || *a
                != (Action {
                    choice_id: a.choice_id.clone(),
                    selected: a.selected.clone(),
                    ..Action::new("choose")
                })
            || !self.cq28_options(&d)?.iter().any(|o| o.id == selected[0])
        {
            return Err("咒欲之门仅接受当前单项模式或原合法目标".into());
        }
        if d.mode.is_none() {
            d.mode = Some(
                selected[0]
                    .strip_prefix("repeat-")
                    .and_then(|n| n.parse().ok())
                    .ok_or("未知重复模式")?,
            )
        } else {
            d.targets.push(selected[0].clone())
        }
        let n = d.mode.unwrap();
        if d.targets.len() < expanded(n).targets.len() {
            return self.cq28_prompt(d);
        }
        let play = Action {
            card_id: Some(d.source.id),
            ability_id: Some(KEY.into()),
            option: Some(format!("repeat-{n}")),
            selected: Some(d.targets),
            ..Action::new("play")
        };
        self.apply_inner(actor, play)
    }
    pub(crate) fn cq28_bind(
        &self,
        actor: usize,
        s: &SourceSnapshot,
        spec: &AbilitySpec,
        a: &Action,
    ) -> RuleResult<Vec<BoundTarget>> {
        validate_ability("CQ28", spec)?;
        if *a
            != (Action {
                card_id: a.card_id.clone(),
                ability_id: Some(KEY.into()),
                option: a.option.clone(),
                selected: a.selected.clone(),
                ..Action::new("play")
            })
            || a.selected
                .as_ref()
                .is_none_or(|v| v.len() != spec.targets.len())
            || a.option.as_deref().is_none_or(|key| {
                !(0..4).any(|n| key == format!("repeat-{n}") && same(spec, &expanded(n)))
            })
        {
            return Err("咒欲之门须在付款前声明两次同项及全部原目标".into());
        }
        spec.targets
            .iter()
            .zip(a.selected.as_ref().unwrap())
            .map(|(slot, id)| {
                if !self.cq28_target_ids(actor, s, slot).contains(id) {
                    return Err("咒欲之门原声明目标无效".into());
                }
                let region_instance = if slot.zone == Zone::Region {
                    let r = id
                        .strip_prefix("region:")
                        .and_then(|n| n.parse::<usize>().ok())
                        .ok_or("地区目标格式无效")?;
                    Some(self.regions[r].card.id.clone())
                } else {
                    None
                };
                Ok(BoundTarget {
                    id: id.clone(),
                    spec: slot.clone(),
                    public: self.public_target(actor, s, slot, id),
                    region_instance,
                })
            })
            .collect()
    }
    pub(crate) fn validate_cq28_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        let relevant = f.source.card.definition == "CQ28"
            || f.ability_key == KEY
            || f.steps.iter().any(|s| uses(std::slice::from_ref(&s.op)));
        if !relevant {
            return Ok(());
        }
        let n = match f.steps.first().map(|s| &s.op) {
            Some(Op::CQ28Mode { mode, .. }) if *mode < 4 => *mode,
            _ => return Err("咒欲之门须保留完整双模式程序".into()),
        };
        let a = expanded(n);
        if f.actor >= self.players.len()
            || f.source.card.definition != "CQ28"
            || f.source.card.owner != f.actor
            || f.source.card.controller != f.actor
            || f.source.card.face_down
            || f.source.region.is_some()
            || f.ability_key != KEY
            || f.frame_id.is_empty()
            || f.cursor > 2
            || f.steps.len() != 2
            || f.steps
                .iter()
                .zip(&a.ops)
                .any(|(s, o)| s.context != f.actor || !same(&s.op, o))
            || f.targets.len() != a.targets.len()
            || f.targets
                .iter()
                .zip(&a.targets)
                .any(|(t, s)| !same(&t.spec, s) || t.id.is_empty() || t.public.instance_id != t.id)
            || f.chosen_region.is_some()
            || !matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)]if f.money_units(0)<=4&&ids.iter().all(|id|f.source.valid_printed_payment_id(id)))
            || f.cursor > 0 && !matches!(f.guard, GuardState::Accepted)
        {
            return Err("咒欲之门须保留原出牌、双实例、原目标与实际费用".into());
        }
        self.validate_resource_frame(f)
    }
    pub(crate) fn cq28_resolve_mode(
        &mut self,
        f: &ResolutionFrame,
        n: u8,
        slot: usize,
    ) -> RuleResult<()> {
        self.validate_cq28_frame(f)?;
        match n {
            0 => {
                self.mill_cards_with_cause(f.actor, 6, Some(f))?;
            }
            1 => {
                let t = &f.targets[slot];
                let region = &f.targets[slot + 1];
                if !self.valid_bound_target(f.actor, &f.source, t)
                    || !self.valid_bound_target(f.actor, &f.source, region)
                {
                    return Ok(());
                }
                let r = region
                    .id
                    .strip_prefix("region:")
                    .and_then(|n| n.parse::<usize>().ok())
                    .ok_or("缺少原地区")?;
                let i = self.players[f.actor]
                    .graveyard
                    .iter()
                    .position(|c| c.id == t.id)
                    .ok_or("原角色已不在墓地")?;
                let c = self.players[f.actor].graveyard.remove(i);
                let mut c = self.reset_zone_card(c);
                c.controller = f.actor;
                let id = c.id.clone();
                let definition = c.definition.clone();
                self.regions[r].cards.push(c);
                self.enter_triggers(f.actor, &definition, &id, false);
            }
            2 => {
                let t = &f.targets[slot];
                if self.valid_bound_target(f.actor, &f.source, t) {
                    self.cq28_defense_reductions.push(GateDefense {
                        frame: Box::new(f.clone()),
                        slot,
                        turn: self.turn,
                    });
                }
            }
            3 => self.draw(f.actor, 1)?,
            _ => return Err("未开放混合或未知模式".into()),
        }
        Ok(())
    }
    pub(crate) fn cq28_defense_penalty(&self, c: &Card) -> u32 {
        if c.face_down {
            return 0;
        }
        self.cq28_defense_reductions
            .iter()
            .filter(|m| {
                m.turn == self.turn && m.frame.targets.get(m.slot).is_some_and(|t| t.id == c.id)
            })
            .count() as u32
    }
    pub(crate) fn validate_cq28_state(&self) -> RuleResult<()> {
        for m in &self.cq28_defense_reductions {
            self.validate_cq28_frame(&m.frame)?;
            if m.turn != self.turn
                || m.slot > 1
                || m.frame.cursor != m.slot + 1
                || !matches!(m.frame.steps[m.slot].op, Op::CQ28Mode { mode: 2, .. })
            {
                return Err("咒欲之门防御减值须来自原结算实例与本回合".into());
            }
        }
        for item in &self.stack {
            if let Some(f) = &item.frame {
                self.validate_cq28_frame(f)?;
            }
        }
        for e in &self.effects {
            if let Effect::Frame { frame } = e {
                self.validate_cq28_frame(frame)?;
            }
        }
        if let Some(p) = &self.pending {
            if let ChoiceResolution::CQ28Draft { draft } = &p.resolution {
                let s = self.cq28_source(draft)?;
                if p.seat != draft.actor
                    || p.choice.kind != DRAFT
                    || p.choice.min != Some(1)
                    || p.choice.max != Some(1)
                    || !same(&p.choice.options, &self.cq28_options(draft)?)
                {
                    return Err("咒欲之门声明续体无效".into());
                }
                if let Some(n) = draft.mode {
                    let a = expanded(n);
                    if draft
                        .targets
                        .iter()
                        .zip(&a.targets)
                        .any(|(id, t)| !self.valid_binding(draft.actor, &s, t, id))
                    {
                        return Err("咒欲之门已声明目标失效".into());
                    }
                }
            }
        }
        Ok(())
    }
}
