//! Spendable monetary credits, never cards/assets or loyalty. One credit is
//! produced by the exact paid BQ092 frame and expires at its printed step end.
use crate::{engine::RuleResult, model::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceStep {
    Prepare,
    Draw,
    Action(usize),
    Mobility,
    Confrontation(usize, usize),
    End,
}
impl ResourceStep {
    pub(crate) fn window(w: &Window) -> Option<Self> {
        Some(match w {
            Window::Prepare => Self::Prepare,
            Window::Draw => Self::Draw,
            Window::Action(t) => Self::Action(*t),
            Window::Mobility => Self::Mobility,
            Window::Before(r, k) | Window::After(r, k) => Self::Confrontation(*r, *k),
            Window::End => Self::End,
            Window::Win(..) => return None,
        })
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepCredit {
    pub id: String,
    pub actor: usize,
    pub turn: u32,
    pub step: ResourceStep,
    pub source: Box<ResolutionFrame>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourcePayment {
    pub paid_index: usize,
    pub credits: Vec<StepCredit>,
}

impl ResolutionFrame {
    /// Physical asset receipts keep their actual IDs/count. Credits are
    /// separately typed witnesses; old monetary guards compare the total.
    pub(crate) fn money_units(&self, index: usize) -> usize {
        let assets = match self.already_paid.get(index) {
            Some(
                PaidCost::Assets(ids)
                | PaidCost::BQ029ExtraAssets(ids)
                | PaidCost::CQ24ExtraAssets(ids),
            ) => ids.len(),
            _ => return usize::MAX,
        };
        assets.saturating_add(
            self.resource_payments
                .iter()
                .filter(|p| p.paid_index == index)
                .map(|p| p.credits.len())
                .sum::<usize>(),
        )
    }
}
impl PaidCost {
    pub(crate) fn extra_money(self, carmen: bool) -> Self {
        match self {
            Self::Assets(ids) => {
                if carmen {
                    Self::CQ24ExtraAssets(ids)
                } else {
                    Self::BQ029ExtraAssets(ids)
                }
            }
            Self::CapturedResources { receipt, credits } => Self::CapturedResources {
                receipt: Box::new((*receipt).extra_money(carmen)),
                credits,
            },
            _ => unreachable!("only actual captured monetary payments become extra costs"),
        }
    }
}
impl Game {
    pub(crate) fn resource_step(&self) -> Option<ResourceStep> {
        self.window
            .as_ref()
            .and_then(ResourceStep::window)
            .or_else(|| {
                self.win_contexts
                    .first()
                    .and_then(|c| ResourceStep::window(&c.resume_window))
            })
    }
    pub(crate) fn step_resource_count(&self, actor: usize) -> u32 {
        let step = self.resource_step();
        self.step_resources
            .iter()
            .filter(|c| c.actor == actor && c.turn == self.turn && step.as_ref() == Some(&c.step))
            .count() as u32
    }
    pub(crate) fn capture_money(&mut self, actor: usize, amount: u32) -> RuleResult<PaidCost> {
        if self.resources(actor) < amount {
            return Err("可用费用不足".into());
        }
        let step = self.resource_step();
        let mut remaining = amount;
        let mut credits = vec![];
        self.step_resources.retain(|c| {
            if remaining > 0
                && c.actor == actor
                && c.turn == self.turn
                && step.as_ref() == Some(&c.step)
            {
                credits.push(c.clone());
                remaining -= 1;
                false
            } else {
                true
            }
        });
        let ids = self.players[actor]
            .assets
            .iter_mut()
            .filter(|c| !c.exhausted)
            .take(remaining as usize)
            .map(|c| {
                c.exhausted = true;
                c.id.clone()
            })
            .collect();
        let paid = PaidCost::Assets(ids);
        Ok(if credits.is_empty() {
            paid
        } else {
            PaidCost::CapturedResources {
                receipt: Box::new(paid),
                credits,
            }
        })
    }
    pub(crate) fn prune_step_resources(&mut self) {
        let step = self.resource_step();
        let turn = self.turn;
        self.step_resources
            .retain(|c| c.turn == turn && step.as_ref() == Some(&c.step));
    }
    pub(crate) fn validate_step_credit(&self, c: &StepCredit) -> RuleResult<()> {
        if c.actor >= self.players.len()
            || c.turn != self.turn
            || !self.mystic_instance_valid(&c.id)
            || c.id == c.source.frame_id
            || self.resource_step().as_ref() != Some(&c.step)
            || c.source.actor != c.actor
            || c.source.cursor != 1
            || !matches!(c.source.guard, GuardState::Accepted)
            || c.source.ability_key != crate::import20_industry::FUND
            || !c.source.resource_payments.is_empty()
        {
            return Err("步骤费用须保留本步骤实际产业支付来源".into());
        }
        self.validate_industry_frame(&c.source)
    }
    pub(crate) fn validate_resource_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        let mut slots = BTreeSet::new();
        let mut ids = BTreeSet::new();
        for p in &f.resource_payments {
            if p.credits.is_empty()
                || !slots.insert(p.paid_index)
                || f.money_units(p.paid_index) == usize::MAX
            {
                return Err("步骤费用支付槽无效".into());
            }
            for c in &p.credits {
                self.validate_step_credit(c)?;
                if c.actor != f.actor
                    || !ids.insert(&c.id)
                    || self.step_resources.iter().any(|active| active.id == c.id)
                {
                    return Err("步骤费用只能由原玩家消费一次".into());
                }
            }
        }
        Ok(())
    }
    pub(crate) fn validate_step_resource_state(&self) -> RuleResult<()> {
        let mut ids = BTreeSet::new();
        for c in &self.step_resources {
            self.validate_step_credit(c)?;
            if !ids.insert(&c.id) {
                return Err("步骤费用重复".into());
            }
        }
        // Standard paid frames and their private continuations authenticate
        // their own resource witnesses in the shared frame dispatch/validators.
        for s in &self.stack {
            if let Some(f) = &s.frame {
                self.validate_resource_frame(f)?;
            }
        }
        for e in &self.effects {
            if let Effect::Frame { frame } = e {
                self.validate_resource_frame(frame)?;
            }
        }
        if let Some(Pending {
            resolution: ChoiceResolution::Frame { frame, .. },
            ..
        }) = &self.pending
        {
            self.validate_resource_frame(frame)?;
        }
        Ok(())
    }
}
