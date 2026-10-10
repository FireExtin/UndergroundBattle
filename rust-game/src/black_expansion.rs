//! Closed WM059 entry-region destruction and BQ078 named graveyard recovery.
use crate::{
    catalog,
    engine::RuleResult,
    model::*,
    rules::{self, *},
};
use std::collections::BTreeSet;

pub(crate) fn definition(id: &str) -> Option<Definition> {
    match id {
        "WM059" => {
            let mut victim = rules::target(
                Zone::Board,
                EntityKind::Character,
                Relation::EnemyTeam,
                Range::SourceRegion,
            );
            victim.printed_cost_max = Some(1);
            let mut d = rules::with_abilities(vec![rules::ability(
                "wm059-enter-region-destroy",
                "进入地区触发：消灭印刷费用不超过一的目标敌方角色",
                Timing::Fast,
                vec![],
                vec![victim],
                vec![Op::Destroy(EntityRef::Target(0))],
                Some(Event::EnterRegion),
            )]);
            d.traits.kill = 1;
            Some(d)
        }
        "BQ078" => {
            let mut d = rules::with_abilities(vec![
                rules::ability(
                    "bq078-entry-mill-four",
                    "进场触发：目标玩家牌库顶四张置墓",
                    Timing::Fast,
                    vec![],
                    vec![rules::target(
                        Zone::Player,
                        EntityKind::Any,
                        Relation::Any,
                        Range::Anywhere,
                    )],
                    vec![Op::MoveDeckTopToGraveyard {
                        player: PlayerRef::Target(0),
                        count: 4,
                    }],
                    Some(Event::Enter),
                ),
                rules::ability(
                    "bq078-death-corpse-return",
                    "死亡触发：将你墓地中的一张无名尸体置于你的手中",
                    Timing::Fast,
                    vec![],
                    vec![],
                    vec![Op::BQ078ReturnNamelessCorpse],
                    Some(Event::Death),
                ),
            ]);
            d.traits.public = true;
            Some(d)
        }
        _ => None,
    }
}
pub(crate) fn contains_black_expansion_op(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::BQ078ReturnNamelessCorpse => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => {
            contains_black_expansion_op(body)
        }
        Op::IfTargetExhausted {
            exhausted, ready, ..
        } => {
            contains_black_expansion_op(std::slice::from_ref(exhausted))
                || contains_black_expansion_op(std::slice::from_ref(ready))
        }
        _ => false,
    })
}
fn relevant_definition(id: &str) -> bool {
    matches!(id, "WM059" | "BQ078")
}
fn relevant_key(key: &str) -> bool {
    matches!(
        key,
        "wm059-enter-region-destroy" | "bq078-entry-mill-four" | "bq078-death-corpse-return"
    )
}
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
fn wm059_selector(targets: &[TargetSlotSpec]) -> bool {
    targets.iter().any(|t| {
        t.zone == Zone::Board
            && t.kind == EntityKind::Character
            && t.relation == Relation::EnemyTeam
            && t.range == Range::SourceRegion
            && t.printed_cost_max == Some(1)
    })
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if !relevant_definition(id)
        && !relevant_key(&a.key)
        && !contains_black_expansion_op(&a.ops)
        && !wm059_selector(&a.targets)
        && !a
            .modes
            .iter()
            .any(|m| contains_black_expansion_op(&m.ops) || wm059_selector(&m.targets))
    {
        return Ok(());
    }
    let admitted = definition(id).is_some_and(|d| {
        d.abilities.iter().any(|expected| same(a, expected))
            || crate::jz22::runtime_ability_is_admitted(a)
    });
    if !admitted {
        return Err(format!("{id}: 黑色扩展仅准入完整印刷能力或原获授奖励"));
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if let Some(expected) = definition(id) {
        if !same(d, &expected) {
            return Err(format!("{id}: 黑色扩展仅准入完整印刷定义"));
        }
    }
    for a in &d.abilities {
        validate_ability(id, a)?;
    }
    Ok(())
}
fn relevant_frame(f: &ResolutionFrame) -> bool {
    relevant_definition(&f.source.card.definition)
        || relevant_key(&f.ability_key)
        || f.steps
            .iter()
            .any(|s| contains_black_expansion_op(std::slice::from_ref(&s.op)))
}
impl Game {
    fn black_source(&self, actor: usize, source: &SourceSnapshot) -> RuleResult<()> {
        if !relevant_definition(&source.card.definition)
            || actor >= self.players.len()
            || source.card.owner >= self.players.len()
            || source.card.controller != actor
            || source.card.id.is_empty()
            || source.region.is_none_or(|r| r >= self.regions.len())
            || source
                .source_region_instance
                .as_ref()
                .is_none_or(|id| id.is_empty())
            || source.observed_death.is_some()
            || source.attachment_host_instance.is_some()
        {
            return Err("黑色扩展冻结来源、行动者或原地区实例不符".into());
        }
        Ok(())
    }
    fn black_frame_ability(&self, f: &ResolutionFrame) -> RuleResult<AbilitySpec> {
        let region = f
            .source
            .source_region_instance
            .as_deref()
            .ok_or("黑色扩展缺少原地区")?;
        if let Some(a) = definition(&f.source.card.definition)
            .and_then(|d| d.abilities.into_iter().find(|a| a.key == f.ability_key))
        {
            return Ok(a);
        }
        match f.ability_key.as_str() {
            "renown" => Ok(crate::renown::renown_ability(region)),
            "jc089-combat-glory" => Ok(rules::jc089_combat_glory_ability(region)),
            _ => Err("黑色扩展来源不允许其他程序或能力别名".into()),
        }
    }
    pub(crate) fn validate_black_expansion_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if !relevant_frame(f) {
            return Ok(());
        }
        self.black_source(f.actor, &f.source)?;
        if f.frame_id.is_empty() {
            return Err("黑色扩展帧实例为空".into());
        }
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            let origin_matches = if f.ability_key == "deploy" {
                !f.source.card.face_down && f.source.play_source == Some(PlaySource::Hand)
            } else {
                f.source.card.face_down && f.source.play_source.is_none()
            };
            if !origin_matches
                || !f.steps.is_empty()
                || !f.targets.is_empty()
                || f.cursor != 0
                || !matches!(f.guard, GuardState::Unchecked)
                || f.chosen_region != f.source.region
            {
                return Err("黑色扩展派遣/现身来源或空程序不符".into());
            }
            return Ok(());
        }
        if f.source.card.face_down
            || f.source.play_source.is_some()
            || f.chosen_region.is_some()
            || !f.already_paid.is_empty()
        {
            return Err("黑色扩展触发不能伪造暗藏、支付或派遣".into());
        }
        let a = self.black_frame_ability(f)?;
        if f.steps.len() != 1
            || f.steps[0].context != f.actor
            || !same(&f.steps[0].op, &a.ops[0])
            || f.targets.len() != a.targets.len()
            || f.targets.iter().zip(&a.targets).any(|(t, s)| {
                !same(&t.spec, s)
                    || t.id.is_empty()
                    || t.public.instance_id != t.id
                    || t.region_instance.is_some()
            })
        {
            return Err("黑色扩展原子程序或冻结目标规格不符".into());
        }
        let paused_return = f.ability_key == "bq078-death-corpse-return"
            && f.cursor == 1
            && matches!(f.guard, GuardState::Accepted);
        if !paused_return && (f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked)) {
            return Err("黑色扩展原子程序不能保存已执行游标或通过守卫".into());
        }
        Ok(())
    }
    fn black_declaration(&self, d: &Declaration) -> RuleResult<()> {
        validate_ability(&d.source.card.definition, &d.ability)?;
        self.black_source(d.actor, &d.source)?;
        if d.source.card.face_down || d.source.play_source.is_some() {
            return Err("黑色扩展声明只能来自公开的在场或死亡前快照".into());
        }
        if let [Op::PlaceInfluence {
            region_instance, ..
        }] = d.ability.ops.as_slice()
        {
            if d.source.source_region_instance.as_ref() != Some(region_instance) {
                return Err("黑色扩展获授奖励原地区不符".into());
            }
        }
        Ok(())
    }
    pub(crate) fn validate_black_expansion_state(&self) -> RuleResult<()> {
        let stored_frame = |f: &ResolutionFrame| -> RuleResult<()> {
            self.validate_black_expansion_frame(f)?;
            if relevant_frame(f) && (f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked)) {
                return Err("黑色扩展待执行帧不能跳过守卫或重放已执行步骤".into());
            }
            Ok(())
        };
        for s in &self.stack {
            if let Some(f) = &s.frame {
                stored_frame(f)?;
                if relevant_frame(f) {
                    if s.id != f.frame_id || s.controller != f.actor {
                        return Err("黑色扩展堆栈实体与冻结帧不符".into());
                    }
                    if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
                        if s.card.as_ref().is_none_or(|c| {
                            c.definition != f.source.card.definition
                                || c.owner != f.source.card.owner
                                || c.controller != f.actor
                                || c.face_down
                        }) || s.deploy_region != f.source.region
                            || s.reveal != (f.ability_key == "reveal")
                            || s.target.is_some()
                        {
                            return Err("黑色扩展派遣/现身堆栈卡或地区不符".into());
                        }
                    } else if s.card.is_some()
                        || s.deploy_region.is_some()
                        || s.reveal
                        || s.target != f.targets.first().map(|t| t.id.clone())
                    {
                        return Err("黑色扩展触发堆栈不能伪造派遣卡".into());
                    }
                }
            } else if s
                .card
                .as_ref()
                .is_some_and(|c| relevant_definition(&c.definition))
            {
                return Err("黑色扩展堆栈缺少冻结帧".into());
            }
        }
        for e in &self.effects {
            match e {
                Effect::Frame { frame } => stored_frame(frame)?,
                Effect::Declare { declaration: d }
                    if relevant_definition(&d.source.card.definition)
                        || relevant_key(&d.ability.key)
                        || contains_black_expansion_op(&d.ability.ops)
                        || d.ability
                            .modes
                            .iter()
                            .any(|m| contains_black_expansion_op(&m.ops)) =>
                {
                    self.black_declaration(d)?
                }
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Declare {
                    declaration: d,
                    stage,
                } if relevant_definition(&d.source.card.definition)
                    || relevant_key(&d.ability.key)
                    || contains_black_expansion_op(&d.ability.ops)
                    || d.ability
                        .modes
                        .iter()
                        .any(|m| contains_black_expansion_op(&m.ops)) =>
                {
                    self.black_declaration(d)?;
                    let targeted = !d.ability.targets.is_empty();
                    let correct_stage = if targeted {
                        matches!(stage, DeclareChoice::Target)
                    } else {
                        matches!(stage, DeclareChoice::Accept)
                    };
                    let options = if targeted {
                        self.binding_options(d.actor, &d.source, &d.ability.targets[0])
                    } else {
                        vec![ChoiceOption {
                            id: "accept".into(),
                            label: "发动触发能力".into(),
                            card: None,
                        }]
                    };
                    if !correct_stage
                        || p.seat != d.actor
                        || p.choice.player_id != player_id(d.actor)
                        || p.choice.id.is_empty()
                        || p.choice.kind != "trigger"
                        || p.choice.min != Some(0)
                        || p.choice.max != Some(1)
                        || p.choice.amount.is_some()
                        || p.choice.allow_decline != Some(true)
                        || !p.choice.preview_cards.is_empty()
                        || !same(&p.choice.options, &options)
                    {
                        return Err("黑色扩展触发选择席位、阶段或元数据不符".into());
                    }
                }
                ChoiceResolution::Frame { frame: f, choice }
                    if relevant_frame(f)
                        || matches!(choice, FrameChoice::BQ078CorpseReturn)
                        || p.choice.kind == "bq078_corpse_return" =>
                {
                    if !matches!(choice, FrameChoice::BQ078CorpseReturn) {
                        return Err("黑色扩展原子程序不能暂停为其他选择".into());
                    }
                    self.bq078_return_actor(f)?;
                    let options = self.bq078_corpse_options(f.actor)?;
                    if self.players[f.actor].eliminated
                        || options.is_empty()
                        || p.seat != f.actor
                        || p.choice.player_id != player_id(f.actor)
                        || p.choice.id.is_empty()
                        || p.choice.kind != "bq078_corpse_return"
                        || p.choice.min != Some(1)
                        || p.choice.max != Some(1)
                        || p.choice.amount.is_some()
                        || p.choice.allow_decline != Some(false)
                        || !p.choice.preview_cards.is_empty()
                        || !same(&p.choice.options, &options)
                    {
                        return Err("BQ078回收选择与冻结行动者当前墓地不符".into());
                    }
                }
                _ if p.choice.kind == "bq078_corpse_return" => {
                    return Err("BQ078回收缺少完整固定帧".into())
                }
                _ => {}
            }
        }
        Ok(())
    }
    fn bq078_return_actor(&self, f: &ResolutionFrame) -> RuleResult<usize> {
        self.validate_black_expansion_frame(f)?;
        if f.source.card.definition != "BQ078"
            || f.ability_key != "bq078-death-corpse-return"
            || f.cursor != 1
            || !matches!(f.guard, GuardState::Accepted)
        {
            return Err("BQ078回收来源或续体不符".into());
        }
        Ok(f.actor)
    }
    fn bq078_corpse_options(&self, actor: usize) -> RuleResult<Vec<ChoiceOption>> {
        let p = self.players.get(actor).ok_or("BQ078回收行动者无效")?;
        let mut options = vec![];
        let mut instances = BTreeSet::new();
        for c in &p.graveyard {
            // Validate every inspected graveyard definition before filtering or
            // projecting it. A corrupt non-candidate must also return an error,
            // never reach catalog::card's trusted-runtime panic path.
            let d = catalog::catalog()
                .cards
                .iter()
                .find(|d| d.id == c.definition)
                .ok_or("BQ078回收墓地含未准入定义")?;
            if c.owner >= self.players.len()
                || c.controller >= self.players.len()
                || c.id.is_empty()
                || !instances.insert(c.id.as_str())
            {
                return Err("BQ078回收墓地拥有者、操控者或牌实例无效".into());
            }
            if d.name == "无名尸体" {
                options.push(self.option(c, actor, None, None));
            }
        }
        Ok(options)
    }
    pub(crate) fn bq078_return_start(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        let actor = self.bq078_return_actor(&f)?;
        if self.players[actor].eliminated {
            return Ok(());
        }
        let options = self.bq078_corpse_options(actor)?;
        if options.is_empty() {
            return Ok(());
        }
        self.choice(
            actor,
            "bq078_corpse_return",
            "将你墓地中的一张无名尸体置于你的手中".into(),
            options,
            1,
            1,
            None,
            ChoiceResolution::Frame {
                frame: Box::new(f),
                choice: FrameChoice::BQ078CorpseReturn,
            },
        );
        Ok(())
    }
    pub(crate) fn bq078_return_complete(
        &mut self,
        f: &ResolutionFrame,
        chooser: usize,
        selected: &[String],
    ) -> RuleResult<()> {
        let actor = self.bq078_return_actor(f)?;
        if chooser != actor
            || self.players[actor].eliminated
            || selected.len() != 1
            || selected.iter().collect::<BTreeSet<_>>().len() != 1
        {
            return Err("BQ078回收必须由冻结行动者选择一张无名尸体".into());
        }
        let options = self.bq078_corpse_options(actor)?;
        if !options.iter().any(|o| o.id == selected[0]) {
            return Err("BQ078回收的本方墓地实例或牌名已失效".into());
        }
        let i = self.players[actor]
            .graveyard
            .iter()
            .position(|c| c.id == selected[0])
            .ok_or("BQ078回收的本方墓地实例或牌名已失效")?;
        let c = self.players[actor].graveyard.remove(i);
        let c = self.reset_zone_card(c);
        self.note(format!(
            "{} 将墓地中的无名尸体置于其手中",
            self.players[actor].name
        ));
        // “你” is the controller in the death snapshot. The destination is that
        // actor's hand even when the selected card has a different owner.
        self.players[actor].hand.push(c);
        Ok(())
    }
}
