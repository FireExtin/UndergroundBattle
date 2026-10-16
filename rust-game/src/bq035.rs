//! The complete pinned BQ035 print. Continuous grants read live entities and
//! real attachments; their event observations are not permanent card modifiers.
use crate::{
    catalog,
    engine::RuleResult,
    model::*,
    rules::{self, *},
};

pub(crate) fn definition() -> Definition {
    let mut d = rules::with_abilities(vec![crate::mobility::ability()]);
    d.modifiers = vec![
        StaticModifier::BQ035PrintedGlory,
        StaticModifier::BQ035OwnBloodVampireMobility,
        StaticModifier::BQ035OwnEquippedVampireGlory,
    ];
    d
}
pub(crate) fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    let own = d.modifiers.iter().any(|m| {
        matches!(
            m,
            StaticModifier::BQ035PrintedGlory
                | StaticModifier::BQ035OwnBloodVampireMobility
                | StaticModifier::BQ035OwnEquippedVampireGlory
        )
    });
    if (id == "BQ035" && !same(d, &definition())) || (id != "BQ035" && own) {
        return Err("BQ035完整机动、威名及两条本方吸血鬼持续能力不能改写或移植".into());
    }
    Ok(())
}
pub(crate) fn validate_character_action(id: &str, a: &Action) -> RuleResult<()> {
    if id == "BQ035"
        && (a.target_id.is_some()
            || a.ability_id.is_some()
            || a.option.is_some()
            || a.choice_id.is_some()
            || a.selected.is_some()
            || a.top.is_some()
            || a.bottom.is_some()
            || a.allocations.is_some()
            || a.cost_selected.is_some()
            || a.deck_draft.is_some()
            || (a.kind == "reveal" && a.region.is_some()))
    {
        return Err("BQ035派遣、秘密派遣和现身不含额外目标或动作程序".into());
    }
    Ok(())
}
impl Game {
    fn bq035_snapshot(&self, actor: usize, source: &SourceSnapshot) -> RuleResult<()> {
        if source.card.definition != "BQ035"
            || source.card.id.is_empty()
            || actor >= self.players.len()
            || source.card.controller != actor
            || source.card.owner >= self.players.len()
            || source.region.is_none_or(|r| r >= self.regions.len())
            || source
                .source_region_instance
                .as_ref()
                .is_none_or(|id| id.is_empty())
            || source.observed_death.is_some()
            || source.observed_mobility_grant.is_some()
            || source.attachment_host_instance.is_some()
        {
            return Err("BQ035冻结来源、操控者或无关观察无效".into());
        }
        self.mobility_immutable_identity(&source.card)
    }
    fn bq035_reward(&self, key: &str, s: &SourceSnapshot) -> RuleResult<AbilitySpec> {
        let region = s
            .source_region_instance
            .as_deref()
            .ok_or("BQ035奖励缺少原地区身份")?;
        match key {
            "jc089-combat-glory" => Ok(rules::jc089_combat_glory_ability(region)),
            "renown" => Ok(crate::renown::renown_ability(region)),
            _ => Err("BQ035不准入其他运行时能力".into()),
        }
    }
    pub(crate) fn validate_bq035_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if self.validated_mobility_frame(f)? || f.source.card.definition != "BQ035" {
            return Ok(());
        }
        self.bq035_snapshot(f.actor, &f.source)?;
        if f.frame_id.is_empty()
            || f.cursor != 0
            || !matches!(f.guard, GuardState::Unchecked)
            || !f.targets.is_empty()
        {
            return Err("BQ035非机动程序不含目标、已执行游标或已通过守卫".into());
        }
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            let assets = matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)] if ids.iter().all(|id|!id.is_empty())
                && ids.iter().collect::<std::collections::BTreeSet<_>>().len()==ids.len());
            if !f.steps.is_empty()
                || !assets
                || f.chosen_region != f.source.region
                || (f.ability_key == "deploy"
                    && (f.source.card.face_down || f.source.play_source != Some(PlaySource::Hand)))
                || (f.ability_key == "reveal"
                    && (!f.source.card.face_down || f.source.play_source.is_some()))
            {
                return Err("BQ035付费进场帧或来源标记无效".into());
            }
        } else {
            let a = self.bq035_reward(&f.ability_key, &f.source)?;
            if f.source.card.face_down
                || f.source.play_source.is_some()
                || !f.already_paid.is_empty()
                || f.chosen_region.is_some()
                || !same(
                    &f.steps,
                    &a.ops
                        .iter()
                        .map(|op| Step {
                            context: f.actor,
                            op: op.clone(),
                        })
                        .collect::<Vec<_>>(),
                )
            {
                return Err("BQ035只准入精确既有威名或获授声望奖励".into());
            }
        }
        Ok(())
    }
    fn validate_bq035_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if self.validated_mobility_declaration(d)? || d.source.card.definition != "BQ035" {
            return Ok(());
        }
        self.bq035_snapshot(d.actor, &d.source)?;
        if d.source.card.face_down
            || d.source.play_source.is_some()
            || !same(&d.ability, &self.bq035_reward(&d.ability.key, &d.source)?)
        {
            return Err("BQ035触发声明不符".into());
        }
        Ok(())
    }
    pub(crate) fn validate_bq035_state(&self) -> RuleResult<()> {
        let mut ids = std::collections::BTreeSet::new();
        for c in self
            .regions
            .iter()
            .flat_map(|r| &r.cards)
            .filter(|c| c.definition == "BQ035")
        {
            if c.id.is_empty()
                || c.owner >= self.players.len()
                || c.controller >= self.players.len()
                || !ids.insert(&c.id)
            {
                return Err("BQ035在场身份或操控者无效".into());
            }
        }
        for item in &self.stack {
            if let Some(f) = &item.frame {
                self.validate_bq035_frame(f)?;
                if self.validated_mobility_frame(f)? {
                    continue;
                }
                if f.source.card.definition == "BQ035" {
                    if item.id != f.frame_id || item.controller != f.actor || item.target.is_some()
                    {
                        return Err("BQ035响应堆栈外壳无效".into());
                    }
                    if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
                        if item.card.as_ref().is_none_or(|c| {
                            c.definition != "BQ035"
                                || c.face_down
                                || c.owner != f.source.card.owner
                                || c.controller != f.actor
                        }) || item.deploy_region != f.source.region
                            || item.reveal != (f.ability_key == "reveal")
                        {
                            return Err("BQ035进场实体与付费帧不符".into());
                        }
                    } else if item.card.is_some() || item.deploy_region.is_some() || item.reveal {
                        return Err("BQ035奖励不能伪造进场卡".into());
                    }
                }
            } else if item.card.as_ref().is_some_and(|c| c.definition == "BQ035") {
                return Err("BQ035进场缺少冻结付费帧".into());
            }
        }
        for e in &self.effects {
            match e {
                Effect::Declare { declaration: d } => self.validate_bq035_declaration(d)?,
                Effect::Frame { frame: f } if f.source.card.definition == "BQ035" => {
                    return Err("BQ035不能绕过真实响应堆栈".into())
                }
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Declare { declaration: d, .. }
                    if self.validated_mobility_declaration(d)? =>
                {
                    self.validate_mobility_state()?
                }
                ChoiceResolution::Declare {
                    declaration: d,
                    stage,
                } if d.source.card.definition == "BQ035" => {
                    self.validate_bq035_declaration(d)?;
                    let options = vec![ChoiceOption {
                        id: "accept".into(),
                        label: "发动触发能力".into(),
                        card: None,
                    }];
                    if !matches!(stage, DeclareChoice::Accept)
                        || p.seat != d.actor
                        || p.choice.player_id != player_id(d.actor)
                        || p.choice.id.is_empty()
                        || p.choice.kind != "trigger"
                        || p.choice.min != Some(0)
                        || p.choice.max != Some(1)
                        || p.choice.allow_decline != Some(true)
                        || p.choice.amount.is_some()
                        || !p.choice.preview_cards.is_empty()
                        || p.choice.title != format!("幽冥骑士：{}", d.ability.label)
                        || p.choice.description != "选择符合数量限制的选项，然后确认。"
                        || !same(&p.choice.options, &options)
                    {
                        return Err("BQ035奖励待选形状无效".into());
                    }
                }
                ChoiceResolution::Frame { frame, .. }
                    if frame.source.card.definition == "BQ035" =>
                {
                    return Err("BQ035没有可保存的帧内待选".into())
                }
                _ => {}
            }
        }
        Ok(())
    }
    fn bq035_beneficiary(&self, id: &str) -> Option<&Card> {
        let (_, c) = self.board(id)?;
        (!c.face_down
            && catalog::card(&c.definition).kind == "character"
            && self.current_subtypes(c).iter().any(|s| s == "吸血鬼"))
        .then_some(c)
    }
    pub(crate) fn bq035_mobility_witness(&self, id: &str) -> Option<ObservedMobilityGrant> {
        let c = self.bq035_beneficiary(id)?;
        let (region, knight) = self
            .regions
            .iter()
            .enumerate()
            .filter(|(_, r)| !r.vacant)
            .flat_map(|(r, region)| region.cards.iter().map(move |c| (r, c)))
            .find(|(_, source)| {
                !source.face_down
                    && source.definition == "BQ035"
                    && source.controller == c.controller
            })?;
        let blood = self.attachments.iter().find(|a| {
            !a.card.face_down
                && a.host_id == c.id
                && catalog::card(&a.card.definition).name == "鲜血"
                && self.attachment_host_valid(a)
        })?;
        Some(ObservedMobilityGrant {
            turn: self.turn,
            vampire: true,
            knight: knight.clone(),
            knight_region: region,
            blood: blood.clone(),
        })
    }
    pub(crate) fn bq035_grants_glory(&self, id: &str) -> bool {
        let Some(c) = self.bq035_beneficiary(id) else {
            return false;
        };
        self.regions
            .iter()
            .filter(|r| !r.vacant)
            .flat_map(|r| &r.cards)
            .any(|source| {
                !source.face_down
                    && source.definition == "BQ035"
                    && source.controller == c.controller
            })
            && self.attachments.iter().any(|a| {
                !a.card.face_down
                    && a.host_id == c.id
                    && self.attachment_host_valid(a)
                    && catalog::card(&a.card.definition)
                        .subtypes
                        .iter()
                        .any(|s| s == "装备" || s == "武器")
            })
    }
}
