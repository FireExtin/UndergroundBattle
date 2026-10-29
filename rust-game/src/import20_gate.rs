//! CQ28: two selected items resolve in printed order (explicit user ruling).
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
    pub op_index: usize,
    pub turn: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateDraft {
    pub actor: usize,
    pub source: Card,
    pub modes: Vec<u8>,
    pub targets: Vec<String>,
}
fn same<T: Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
fn pair_keys() -> impl Iterator<Item = [u8; 2]> {
    (0..4).flat_map(|a| (a..4).map(move |b| [a, b]))
}
fn item_label(n: u8) -> &'static str {
    [
        "将顶六张置墓",
        "从墓地放置费用不大于2角色",
        "目标角色本回合防御-1",
        "抓一张",
    ][n as usize]
}
fn pair_key(pair: [u8; 2]) -> String {
    format!("pair-{}-{}", pair[0], pair[1])
}
fn mode(pair: [u8; 2]) -> Mode {
    let mut targets = vec![];
    let mut ops = vec![];
    for n in pair {
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
        key: pair_key(pair),
        label: format!(
            "{}；{}（按卡面顺序）",
            item_label(pair[0]),
            item_label(pair[1])
        ),
        targets,
        ops,
    }
}
pub(crate) fn definition() -> Definition {
    let mut a = ability(
        KEY,
        "选择两项，可重复；按卡面列项顺序结算",
        Timing::Standard,
        vec![],
        vec![],
        vec![],
        None,
    );
    a.play_only = true;
    a.modes = pair_keys().map(mode).collect();
    with_abilities(vec![a])
}
fn expanded(pair: [u8; 2]) -> AbilitySpec {
    let m = mode(pair);
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
        || (!same(a, &definition().abilities[0])
            && !pair_keys().any(|pair| same(a, &expanded(pair))))
    {
        return Err("咒欲之门须保留两个完整选项并按卡面顺序结算".into());
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
            || d.modes.len() > 2
            || d.modes.iter().any(|n| *n > 3)
            || d.modes.len() == 2 && d.modes[0] > d.modes[1]
            || d.modes.len() < 2 && !d.targets.is_empty()
        {
            return Err("咒欲之门须由原手牌持有者先声明模式目标".into());
        }
        Ok(self.source_snapshot(&d.source, None))
    }
    fn cq28_options(&self, d: &GateDraft) -> RuleResult<Vec<ChoiceOption>> {
        let s = self.cq28_source(d)?;
        if d.modes.len() == 2 {
            let a = expanded([d.modes[0], d.modes[1]]);
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
                expanded([*n, *n])
                    .targets
                    .iter()
                    .all(|slot| !self.cq28_target_ids(d.actor, &s, slot).is_empty())
            })
            .map(|n| ChoiceOption {
                id: format!("item-{n}"),
                label: item_label(n).into(),
                card: None,
            })
            .collect())
    }
    fn cq28_prompt(&mut self, d: GateDraft) -> RuleResult<()> {
        let options = self.cq28_options(&d)?;
        self.choice(
            d.actor,
            DRAFT,
            if d.modes.len() < 2 {
                format!(
                    "咒欲之门：选择第{}项，可重复；结算按卡面顺序",
                    d.modes.len() + 1
                )
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
            modes: vec![],
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
        if d.modes.len() < 2 {
            d.modes.push(
                selected[0]
                    .strip_prefix("item-")
                    .and_then(|n| n.parse().ok())
                    .ok_or("未知选项")?,
            );
            if d.modes.len() < 2 {
                return self.cq28_prompt(d);
            }
            d.modes.sort_unstable();
        } else {
            d.targets.push(selected[0].clone());
        }
        let pair = [d.modes[0], d.modes[1]];
        if d.targets.len() < expanded(pair).targets.len() {
            return self.cq28_prompt(d);
        }
        let play = Action {
            card_id: Some(d.source.id),
            ability_id: Some(KEY.into()),
            option: Some(pair_key(pair)),
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
                !pair_keys().any(|pair| key == pair_key(pair) && same(spec, &expanded(pair)))
            })
        {
            return Err("咒欲之门须在付款前声明两项及全部原目标，结算按卡面顺序".into());
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
        let modes = f
            .steps
            .iter()
            .map(|s| match s.op {
                Op::CQ28Mode { mode, .. } if mode < 4 => Some(mode),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()
            .ok_or("咒欲之门须保留完整双模式程序")?;
        if modes.len() != 2 || modes[0] > modes[1] {
            return Err("咒欲之门的两个选项必须按卡面顺序冻结".into());
        }
        let a = expanded([modes[0], modes[1]]);
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
                let c = self.take_grave_observed(f.actor,i);
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
                        op_index: f.cursor - 1,
                        turn: self.turn,
                    });
                }
            }
            3 => self.draw(f.actor, 1)?,
            _ => return Err("未知咒欲之门模式".into()),
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
                || m.op_index >= 2
                || m.slot >= m.frame.targets.len()
                || m.frame.cursor != m.op_index + 1
                || !matches!(m.frame.steps[m.op_index].op,Op::CQ28Mode{mode:2,slot} if slot==m.slot)
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
                if draft.modes.len() == 2 {
                    let a = expanded([draft.modes[0], draft.modes[1]]);
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
