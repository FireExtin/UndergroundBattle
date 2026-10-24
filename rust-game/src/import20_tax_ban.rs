//! Printed-cost ban and Pete's actual played-attachment tax. The shared
//! paid frame, response stack, Pending choice and entry path remain authoritative.
use crate::{catalog, engine::RuleResult, model::*, mystic_contest::MysticShown, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub(crate) const CARDS: &[&str] = &["XG20", "BQ057"];
pub(crate) const NUMBER: &str = "import20-xg20-number";
pub(crate) const TAX: &str = "import20-bq057-tax";
pub(crate) const ASSET: &str = "import20-bq057-asset-entry";
const PAY_KIND: &str = "pete-extra-payment";
const DRAW_KIND: &str = "pete-optional-draw";
const NUMBER_TITLE: &str = "禁令：选择一个非负整数作为印刷费用";
const PAY_TITLE: &str = "皮特：额外支付2，否则其操控者可以抓1";
const DRAW_TITLE: &str = "皮特：对方未支付额外2，你可以抓1";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttachmentPlayed {
    pub turn: u32,
    pub play: Box<ResolutionFrame>,
}
fn same<T: Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
pub(crate) fn definition(id: &str) -> Option<Definition> {
    if id == "XG20" {
        let mut d = attachment_definition(target(
            Zone::Society,
            EntityKind::Any,
            Relation::ControlledByActor,
            Range::Anywhere,
        ));
        d.abilities.push(ability(
            NUMBER,
            "进场：选择禁用的印刷费用",
            Timing::Fast,
            vec![],
            vec![],
            vec![Op::XG20ChooseNumber],
            Some(Event::Enter),
        ));
        return Some(d);
    }
    if id != "BQ057" {
        return None;
    }
    let mut asset = ability(
        ASSET,
        "资产区行动，支付名誉3：放置进目标地区",
        Timing::Standard,
        vec![Cost::Honor(3)],
        vec![target(
            Zone::Region,
            EntityKind::Any,
            Relation::Any,
            Range::Anywhere,
        )],
        vec![Op::PeteFromAsset],
        None,
    );
    asset.activation_only = true;
    let mut d = with_abilities(vec![
        ability(
            TAX,
            "敌方打出附属：额外支付2，否则你可以抓1",
            Timing::Fast,
            vec![],
            vec![],
            vec![Op::PeteTax { observation: None }],
            Some(Event::PeteAttachmentPlayed),
        ),
        asset,
    ]);
    d.traits.public = true;
    d.traits.retreat = true;
    Some(d)
}
fn owns(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::XG20ChooseNumber | Op::PeteTax { .. } | Op::PeteFromAsset => true,
        Op::ForEachLivingPlayer(b) | Op::ForEachLivingPlayerFromActor(b) => owns(b),
        Op::IfTargetExhausted {
            exhausted, ready, ..
        } => owns(std::slice::from_ref(exhausted)) || owns(std::slice::from_ref(ready)),
        _ => false,
    })
}
pub(crate) fn related(id: &str, a: &AbilitySpec) -> bool {
    CARDS.contains(&id)
        || [NUMBER, TAX, ASSET].contains(&a.key.as_str())
        || owns(&a.ops)
        || a.modes.iter().any(|m| owns(&m.ops))
        || a.costs.iter().any(|c| matches!(c, Cost::Honor(_)))
}
pub(crate) fn frame_related(f: &ResolutionFrame) -> bool {
    CARDS.contains(&f.source.card.definition.as_str())
        || [NUMBER, TAX, ASSET].contains(&f.ability_key.as_str())
        || f.steps.iter().any(|s| owns(std::slice::from_ref(&s.op)))
        || f.already_paid
            .iter()
            .any(|p| matches!(p, PaidCost::Honor { .. }))
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    let mut expected = definition(id)
        .and_then(|d| d.abilities.into_iter().find(|b| b.key == a.key))
        .ok_or("禁令/皮特程序只属于原卡")?;
    if let [Op::PeteTax { observation }] = a.ops.as_slice() {
        expected.ops = vec![Op::PeteTax {
            observation: observation.clone(),
        }];
    }
    if !same(&expected, a) {
        return Err("禁令/皮特不能改写原时序、费用、事件或完整程序".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if let Some(expected) = definition(id) {
        if !same(&expected, d) {
            return Err("禁令/皮特须保留完整印刷定义".into());
        }
    } else if d.abilities.iter().any(|a| related(id, a)) {
        return Err("禁令/皮特程序不能移植到其他印刷卡".into());
    }
    Ok(())
}
pub(crate) fn validate_action(id: &str, a: &Action) -> RuleResult<()> {
    if !CARDS.contains(&id) {
        return Ok(());
    }
    let expected = match (id, a.kind.as_str()) {
        ("XG20", "play") => Action {
            card_id: a.card_id.clone(),
            ability_id: a.ability_id.clone(),
            target_id: a.target_id.clone(),
            ..Action::new("play")
        },
        ("BQ057", "deploy" | "conceal") => Action {
            card_id: a.card_id.clone(),
            region: a.region,
            ..Action::new(&a.kind)
        },
        ("BQ057", "reveal") => Action {
            card_id: a.card_id.clone(),
            ..Action::new("reveal")
        },
        _ => return Err("禁令/皮特只能按原打出、派遣或现身程序行动".into()),
    };
    if *a != expected {
        return Err("禁令/皮特声明不能夹带无关载荷".into());
    }
    Ok(())
}
pub(crate) fn validate_choice_action(p: &Pending, a: &Action) -> RuleResult<()> {
    let numeric = matches!(
        p.resolution,
        ChoiceResolution::Frame {
            choice: FrameChoice::XG20Number,
            ..
        }
    );
    let related = match &p.resolution {
        ChoiceResolution::Declare { declaration: d, .. } => {
            related(&d.source.card.definition, &d.ability)
        }
        ChoiceResolution::Frame { frame: f, .. } => frame_related(f),
        _ => false,
    };
    if related
        && *a
            != (Action {
                choice_id: a.choice_id.clone(),
                selected: a.selected.clone(),
                amount: if numeric { a.amount } else { None },
                ..Action::new("choose")
            })
    {
        return Err("禁令/皮特待选只能提交原选择，不能夹带其他命令".into());
    }
    if numeric && (a.amount.is_none() || a.selected.as_ref().is_some_and(|s| !s.is_empty())) {
        return Err("禁令须由原待选提交一个非负整数".into());
    }
    Ok(())
}
impl Game {
    pub(crate) fn xg20_numeric_action(&self, a: &Action) -> bool {
        a.kind == "choose"
            && self.pending.as_ref().is_some_and(|p| {
                matches!(
                    p.resolution,
                    ChoiceResolution::Frame {
                        choice: FrameChoice::XG20Number,
                        ..
                    }
                )
            })
    }
    pub(crate) fn xg20_check_play(&self, actor: usize, c: &Card) -> RuleResult<()> {
        let cost = catalog::card(&c.definition).cost;
        if self.attachments.iter().any(|a| {
            a.card.definition == "XG20"
                && !a.card.face_down
                && !self.players[a.card.controller].eliminated
                && self.attachment_host_valid(a)
                && self.team(a.card.controller) != self.team(actor)
                && a.card.chosen_number == Some(cost)
        }) {
            return Err(format!(
                "禁令禁止敌方正面打出或付费现身印刷费用为{cost}的牌"
            ));
        }
        Ok(())
    }
    fn tax_ban_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        if !CARDS.contains(&s.card.definition.as_str())
            || actor >= self.players.len()
            || s.card.controller != actor
            || s.card.owner >= self.players.len()
            || !self.mystic_instance_valid(&s.card.id)
            || s.death_echo.is_some()
            || s.observed_death.is_some()
            || s.observed_exposure.is_some()
            || s.observed_mobility_grant.is_some()
        {
            return Err("禁令/皮特须保留原来源、操控者和实例".into());
        }
        self.mobility_immutable_identity(&s.card)
    }
    pub(crate) fn xg20_shown_witness(&self, s: &MysticShown) -> RuleResult<()> {
        if let Some(a) = &s.politician_asset {
            if s.card.definition != "XG20"
                || a.controller != s.seat
                || a.owner >= self.players.len()
                || a.id == s.card.id
                || !self.mystic_instance_valid(&a.id)
                || !catalog::catalog()
                    .cards
                    .iter()
                    .any(|d| d.id == a.definition && d.subtypes.iter().any(|s| s == "政治家"))
            {
                return Err("禁令隐秘加值须保留展示当时的本方政治家资产见证".into());
            }
            self.mobility_immutable_identity(a)?;
        }
        Ok(())
    }
    fn pete_observation<'a>(&self, f: &'a ResolutionFrame) -> RuleResult<&'a AttachmentPlayed> {
        match f.steps.as_slice() {
            [Step {
                op: Op::PeteTax {
                    observation: Some(o),
                },
                ..
            }] if f.ability_key == TAX => Ok(o),
            _ => Err("皮特须绑定原实际打出附属事件".into()),
        }
    }
    fn pete_witness(
        &self,
        actor: usize,
        s: &SourceSnapshot,
        o: &AttachmentPlayed,
    ) -> RuleResult<()> {
        self.tax_ban_source(actor, s)?;
        let p = &o.play;
        let c = &p.source.card;
        let Some(printed) = catalog::catalog()
            .cards
            .iter()
            .find(|d| d.id == c.definition)
        else {
            return Err("皮特观察未知印刷牌".into());
        };
        let deploy = p.ability_key == "deploy" && catalog::has_kind(printed, "character");
        let spec = crate::rules::definition(&c.definition)
            .abilities
            .iter()
            .find(|a| a.key == p.ability_key && a.play_only);
        if s.card.definition != "BQ057"
            || s.card.face_down
            || s.play_source.is_some()
            || s.region.is_none()
            || s.source_region_instance.is_none()
            || s.attachment_host_instance.is_some()
            || o.turn != self.turn
            || p.actor >= self.players.len()
            || c.controller != p.actor
            || self.team(p.actor) == self.team(actor)
            || !catalog::has_kind(printed, "attachment")
            || !self
                .mystic_instance_number(&c.id)
                .zip(self.mystic_instance_number(&p.frame_id))
                .is_some_and(|(a, b)| a < b)
            || !self
                .mystic_instance_number(&s.card.id)
                .zip(self.mystic_instance_number(&p.frame_id))
                .is_some_and(|(a, b)| a < b)
            || p.cursor != 0
            || !matches!(p.guard, GuardState::Unchecked)
            || c.face_down && !crate::batch4_plans::secret(c)
            || p.source.attachment_host_instance.is_some()
            || p.source.observed_death.is_some()
            || p.source.observed_exposure.is_some()
            || p.source.observed_mobility_grant.is_some()
            || !matches!(p.already_paid.first(),Some(PaidCost::Assets(ids))if p.money_units(0)<=printed.cost as usize&&ids.iter().collect::<BTreeSet<_>>().len()==ids.len()&&ids.iter().all(|id|p.source.valid_printed_payment_id(id)))
        {
            return Err("皮特税只观察原敌方的真实附属打出，不是进场、翻面或能力".into());
        }
        if deploy {
            if !p.source.valid_character_play()
                || p.source.region.is_none()
                || p.chosen_region != p.source.region
                || !p.targets.is_empty()
                || !p.steps.is_empty()
            {
                return Err("双类型角色打出须保留原派遣事实".into());
            }
        } else {
            let a = spec.ok_or("皮特观察不属于原附属打出程序")?;
            if p.source.region.is_some()
                || p.source.source_region_instance.is_some()
                || p.source.play_source.is_some()
                || p.targets.len() != a.targets.len()
                || p.targets
                    .iter()
                    .zip(&a.targets)
                    .any(|(t, a)| !same(&t.spec, a))
            {
                return Err("皮特观察须保留原附属打出宿主目标".into());
            }
        }
        self.mobility_immutable_identity(c)?;
        self.validate_resource_frame(p)?;
        self.validate_pack_a_frame(p)
    }
    pub(crate) fn validate_tax_ban_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if self.validated_cross_card_declaration(d)? {
            return Ok(());
        }
        self.tax_ban_source(d.actor, &d.source)?;
        validate_ability(&d.source.card.definition, &d.ability)?;
        if d.ability.event.is_none() || d.source.card.face_down || d.source.play_source.is_some() {
            return Err("禁令/皮特只能声明原触发".into());
        }
        if d.ability.key == NUMBER {
            if d.source.region.is_some()
                || d.source.attachment_host_instance.as_deref()
                    != Some(format!("society:p{}", d.actor).as_str())
            {
                return Err("禁令进场须保留原本人秘社".into());
            }
        } else if let [Op::PeteTax {
            observation: Some(o),
        }] = d.ability.ops.as_slice()
        {
            self.pete_witness(d.actor, &d.source, o)?;
        } else {
            return Err("皮特缺少实际出牌观察".into());
        }
        Ok(())
    }
    pub(crate) fn validate_tax_ban_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        self.validate_resource_frame(f)?;
        if self.validated_cross_card_frame(f)? {
            return Ok(());
        }
        self.tax_ban_source(f.actor, &f.source)?;
        if !self
            .mystic_instance_number(&f.source.card.id)
            .zip(self.mystic_instance_number(&f.frame_id))
            .is_some_and(|(a, b)| a < b)
            || f.cursor > f.steps.len()
            || f.cursor > 0 && !matches!(f.guard, GuardState::Accepted)
            || f.cursor == 0
                && !matches!(f.guard, GuardState::Unchecked)
                && !(f.steps.is_empty()
                    && matches!(f.guard, GuardState::Accepted | GuardState::Cancelled))
        {
            return Err("禁令/皮特原帧身份、守卫或阶段无效".into());
        }
        if f.source.card.definition == "BQ057"
            && matches!(f.ability_key.as_str(), "deploy" | "reveal")
        {
            if !f.steps.is_empty()
                || !f.targets.is_empty()
                || f.chosen_region != f.source.region
                || f.source.region.is_none()
                || f.ability_key == "deploy"
                    && (!f.source.valid_character_play() || f.source.card.face_down)
                || f.ability_key == "reveal" && !f.source.card.face_down
                || !matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)]if f.money_units(0)<=3&&ids.iter().all(|id|f.source.valid_printed_payment_id(id)))
            {
                return Err("皮特派遣/现身须保留原地区和实际费用".into());
            }
            return Ok(());
        }
        let mut a = definition(&f.source.card.definition)
            .and_then(|d| d.abilities.into_iter().find(|a| a.key == f.ability_key))
            .ok_or("未知禁令/皮特程序")?;
        if f.ability_key == "attach" {
            if !f.steps.is_empty()
                || f.source.region.is_some()
                || f.source.attachment_host_instance.is_some()
                || f.targets.len() != 1
                || !same(&f.targets[0].spec, &a.targets[0])
                || !matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)]if f.money_units(0)<=3&&ids.iter().all(|id|f.source.valid_printed_payment_id(id)))
            {
                return Err("禁令打出须保留原本人秘社及实际费用".into());
            }
            return Ok(());
        }
        a.ops = f.steps.iter().map(|s| s.op.clone()).collect();
        validate_ability(&f.source.card.definition, &a)?;
        if f.chosen_region.is_some()
            || f.steps.iter().any(|s| s.context != f.actor)
            || f.targets.len() != a.targets.len()
            || f.targets
                .iter()
                .zip(&a.targets)
                .any(|(t, a)| !same(&t.spec, a))
        {
            return Err("禁令/皮特不能更换执行环境或目标".into());
        }
        if a.event.is_some() {
            if !f.already_paid.is_empty() {
                return Err("禁令/皮特触发没有声明费用".into());
            }
            self.validate_tax_ban_declaration(&Declaration {
                actor: f.actor,
                source: f.source.clone(),
                ability: a,
            })?;
        } else if f.ability_key == ASSET {
            if f.source.region.is_some()
                || f.source.attachment_host_instance.is_some()
                || f.source.play_source.is_some()
                || f.source.card.face_down
                || !matches!(f.already_paid.as_slice(),[PaidCost::Honor{before,amount:3}]if *before>=3)
            {
                return Err("皮特资产行动只支付原有名誉3，不是地区分数或货币".into());
            }
        } else {
            return Err("禁令/皮特未准入该主动程序".into());
        }
        Ok(())
    }
    pub(crate) fn xg20_number_start(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        self.validate_tax_ban_frame(&f)?;
        if !self
            .in_play(&f.source.card.id)
            .is_some_and(|(_, c)| c.definition == "XG20" && !c.face_down)
        {
            return Ok(());
        }
        self.choice(
            f.actor,
            NUMBER,
            NUMBER_TITLE.into(),
            vec![],
            0,
            0,
            None,
            ChoiceResolution::Frame {
                frame: Box::new(f),
                choice: FrameChoice::XG20Number,
            },
        );
        Ok(())
    }
    pub(crate) fn xg20_number_finish(
        &mut self,
        f: &ResolutionFrame,
        actor: usize,
        a: &Action,
    ) -> RuleResult<()> {
        self.validate_tax_ban_frame(f)?;
        if actor != f.actor
            || a.amount.is_none()
            || f.cursor != 1
            || *a
                != (Action {
                    choice_id: a.choice_id.clone(),
                    selected: a.selected.clone(),
                    amount: a.amount,
                    ..Action::new("choose")
                })
            || a.selected.as_ref().is_some_and(|s| !s.is_empty())
        {
            return Err("禁令只接受原操控者一次选择非负整数".into());
        }
        if let Some(c) = self
            .board_mut(&f.source.card.id)
            .filter(|c| c.definition == "XG20" && !c.face_down)
        {
            c.chosen_number = a.amount;
        }
        self.note(format!("禁令选择印刷费用{}", a.amount.unwrap()));
        Ok(())
    }
    pub(crate) fn pete_observe_attachment(&mut self, play: &ResolutionFrame) {
        if !catalog::has_kind(catalog::card(&play.source.card.definition), "attachment")
            || !(play.ability_key == "deploy"
                || crate::rules::definition(&play.source.card.definition)
                    .abilities
                    .iter()
                    .any(|a| a.key == play.ability_key && a.play_only))
        {
            return;
        }
        let mut sources = self
            .in_play_cards()
            .into_iter()
            .filter(|(_, c)| {
                c.definition == "BQ057"
                    && !c.face_down
                    && !self.players[c.controller].eliminated
                    && self.team(c.controller) != self.team(play.actor)
            })
            .map(|(r, c)| self.source_snapshot(c, r))
            .collect::<Vec<_>>();
        sources.sort_by_key(|s| {
            (
                self.team(s.card.controller) != self.first_team,
                s.card.controller,
            )
        });
        for source in sources {
            let mut a = definition("BQ057").unwrap().abilities.remove(0);
            a.ops = vec![Op::PeteTax {
                observation: Some(Box::new(AttachmentPlayed {
                    turn: self.turn,
                    play: Box::new(play.clone()),
                })),
            }];
            self.effects.push_back(Effect::Declare {
                declaration: Declaration {
                    actor: source.card.controller,
                    source,
                    ability: a,
                },
            });
        }
    }
    fn pete_pay_options(&self, actor: usize) -> Vec<ChoiceOption> {
        let mut out = vec![ChoiceOption {
            id: "decline".into(),
            label: "不支付额外2".into(),
            card: None,
        }];
        if self.resources(actor) >= 2 {
            out.insert(
                0,
                ChoiceOption {
                    id: "pay".into(),
                    label: "支付额外2".into(),
                    card: None,
                },
            );
        }
        out
    }
    fn pete_draw_options() -> Vec<ChoiceOption> {
        vec![ChoiceOption {
            id: "draw".into(),
            label: "抓一张牌".into(),
            card: None,
        }]
    }
    fn pete_draw_start(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        if self.players[f.actor].eliminated {
            return Ok(());
        }
        self.choice(
            f.actor,
            DRAW_KIND,
            DRAW_TITLE.into(),
            Self::pete_draw_options(),
            0,
            1,
            None,
            ChoiceResolution::Frame {
                frame: Box::new(f),
                choice: FrameChoice::PeteDraw,
            },
        );
        Ok(())
    }
    pub(crate) fn pete_tax_start(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        self.validate_tax_ban_frame(&f)?;
        let actor = self.pete_observation(&f)?.play.actor;
        if self.players[actor].eliminated || self.resources(actor) < 2 {
            return self.pete_draw_start(f);
        }
        self.choice(
            actor,
            PAY_KIND,
            PAY_TITLE.into(),
            self.pete_pay_options(actor),
            1,
            1,
            None,
            ChoiceResolution::Frame {
                frame: Box::new(f),
                choice: FrameChoice::PetePay,
            },
        );
        Ok(())
    }
    pub(crate) fn pete_pay_finish(
        &mut self,
        f: ResolutionFrame,
        actor: usize,
        a: &Action,
        selected: &[String],
    ) -> RuleResult<()> {
        self.validate_tax_ban_frame(&f)?;
        if actor != self.pete_observation(&f)?.play.actor
            || selected.len() != 1
            || !self
                .pete_pay_options(actor)
                .iter()
                .any(|o| o.id == selected[0])
            || *a
                != (Action {
                    choice_id: a.choice_id.clone(),
                    selected: a.selected.clone(),
                    ..Action::new("choose")
                })
        {
            return Err("皮特额外费用只由原出牌者支付或拒绝".into());
        }
        if selected[0] == "pay" {
            self.pay(actor, 2)?;
            self.note("皮特：出牌者支付额外2".into());
            Ok(())
        } else {
            self.pete_draw_start(f)
        }
    }
    pub(crate) fn pete_draw_finish(
        &mut self,
        f: &ResolutionFrame,
        actor: usize,
        a: &Action,
        selected: &[String],
    ) -> RuleResult<()> {
        self.validate_tax_ban_frame(f)?;
        if actor != f.actor
            || selected.len() > 1
            || selected.first().is_some_and(|s| s != "draw")
            || *a
                != (Action {
                    choice_id: a.choice_id.clone(),
                    selected: a.selected.clone(),
                    ..Action::new("choose")
                })
        {
            return Err("皮特抓牌只能由原观察者选择一次或放弃".into());
        }
        if !selected.is_empty() && !self.players[actor].eliminated {
            self.draw(actor, 1)?;
        }
        Ok(())
    }
    pub(crate) fn pete_pay_honor(
        &mut self,
        actor: usize,
        s: &SourceSnapshot,
        a: &AbilitySpec,
        n: u32,
    ) -> RuleResult<PaidCost> {
        if s.card.definition != "BQ057"
            || a.key != ASSET
            || n != 3
            || actor != s.card.controller
            || !self.players[actor].assets.iter().any(|c| *c == s.card)
        {
            return Err("名誉3只用于皮特原资产区行动".into());
        }
        let before = self.players[actor].honor;
        self.players[actor].honor = before
            .checked_sub(n)
            .ok_or("名誉点不足3；地区分数和货币不能代替")?;
        Ok(PaidCost::Honor { before, amount: n })
    }
    pub(crate) fn pete_begin_asset(&mut self, actor: usize, a: &Action) -> RuleResult<()> {
        if *a
            != (Action {
                card_id: a.card_id.clone(),
                ability_id: Some(ASSET.into()),
                region: a.region,
                ..Action::new("activate")
            })
        {
            return Err("皮特资产行动只选择本方原资产及目标地区".into());
        }
        let c = self.players[actor]
            .assets
            .iter()
            .find(|c| {
                Some(c.id.as_str()) == a.card_id.as_deref()
                    && c.definition == "BQ057"
                    && c.controller == actor
            })
            .ok_or("皮特不在本方资产区")?
            .clone();
        let source = self.source_snapshot(&c, None);
        let spec = definition("BQ057").unwrap().abilities.remove(1);
        self.check_timing(actor, &spec)?;
        let targets = self.bind_action(actor, &source, &spec, a)?;
        let paid = self.pay_ability_costs(actor, &source, &spec, a)?;
        let frame = self.make_frame(actor, source, &spec, targets, paid, None);
        self.dispatch_frame(
            frame,
            "皮特：名誉3，从资产区放置进场".into(),
            None,
            spec.response_policy,
        );
        Ok(())
    }
    pub(crate) fn pete_place_from_asset(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        self.validate_tax_ban_frame(f)?;
        let r = f
            .targets
            .first()
            .and_then(|t| t.id.strip_prefix("region:"))
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|r| self.region_live(*r));
        if let (Some(r), Some(index)) = (
            r,
            self.players[f.actor].assets.iter().position(|c| {
                c.id == f.source.card.id && c.definition == "BQ057" && c.controller == f.actor
            }),
        ) {
            let c = self.players[f.actor].assets.remove(index);
            let mut c = self.reset_zone_card(c);
            c.controller = f.actor;
            let id = c.id.clone();
            self.regions[r].cards.push(c);
            self.enter_triggers(f.actor, "BQ057", &id, false);
        }
        Ok(())
    }
    pub(crate) fn validate_tax_ban_state(&self) -> RuleResult<()> {
        let mut events = BTreeSet::new();
        let mut ids = BTreeSet::new();
        let mut remember = |actor: usize, s: &SourceSnapshot, ops: &[Op]| -> RuleResult<()> {
            if let [Op::PeteTax {
                observation: Some(o),
            }] = ops
            {
                self.pete_witness(actor, s, o)?;
                if !events.insert((s.card.id.clone(), o.play.frame_id.clone())) {
                    return Err("同一皮特不能重复观察同次附属打出".into());
                }
            }
            Ok(())
        };
        for item in &self.stack {
            if let Some(f) = &item.frame {
                if !frame_related(f) || self.validated_cross_card_frame(f)? {
                    continue;
                }
                self.validate_tax_ban_frame(f)?;
                remember(
                    f.actor,
                    &f.source,
                    &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>(),
                )?;
                let played = matches!(f.ability_key.as_str(), "attach" | "deploy" | "reveal");
                if !ids.insert(&f.frame_id)
                    || item.id != f.frame_id
                    || item.controller != f.actor
                    || f.cursor != 0
                    || !matches!(f.guard, GuardState::Unchecked)
                    || item.card.is_some() != played
                    || item.reveal != (f.ability_key == "reveal")
                    || item.target != f.targets.first().map(|t| t.id.clone())
                    || item.deploy_region
                        != if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
                            f.chosen_region
                        } else {
                            None
                        }
                {
                    return Err("禁令/皮特响应外壳与原帧不符".into());
                }
            }
        }
        for e in &self.effects {
            match e {
                Effect::Declare { declaration: d }
                    if related(&d.source.card.definition, &d.ability)
                        && !self.validated_cross_card_declaration(d)? =>
                {
                    self.validate_tax_ban_declaration(d)?;
                    remember(d.actor, &d.source, &d.ability.ops)?;
                }
                Effect::Frame { frame: f }
                    if frame_related(f) && !self.validated_cross_card_frame(f)? =>
                {
                    self.validate_tax_ban_frame(f)?;
                }
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Declare {
                    declaration: d,
                    stage,
                } if related(&d.source.card.definition, &d.ability)
                    && !self.validated_cross_card_declaration(d)? =>
                {
                    self.validate_tax_ban_declaration(d)?;
                    remember(d.actor, &d.source, &d.ability.ops)?;
                    if !matches!(stage, DeclareChoice::Accept)
                        || p.seat != d.actor
                        || p.choice.options.len() != 1
                        || p.choice.options[0].id != "accept"
                        || p.choice.min != Some(0)
                        || p.choice.max != Some(1)
                    {
                        return Err("禁令/皮特触发只接受原玩家发动或放弃".into());
                    }
                }
                ChoiceResolution::Frame { frame: f, choice }
                    if frame_related(f) && !self.validated_cross_card_frame(f)? =>
                {
                    self.validate_tax_ban_frame(f)?;
                    remember(
                        f.actor,
                        &f.source,
                        &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>(),
                    )?;
                    let (seat, kind, title, options, min, max) = match choice {
                        FrameChoice::XG20Number if f.ability_key == NUMBER => {
                            (f.actor, NUMBER, NUMBER_TITLE, vec![], 0, 0)
                        }
                        FrameChoice::PetePay if f.ability_key == TAX => {
                            let seat = self.pete_observation(f)?.play.actor;
                            (seat, PAY_KIND, PAY_TITLE, self.pete_pay_options(seat), 1, 1)
                        }
                        FrameChoice::PeteDraw if f.ability_key == TAX => (
                            f.actor,
                            DRAW_KIND,
                            DRAW_TITLE,
                            Self::pete_draw_options(),
                            0,
                            1,
                        ),
                        _ => return Err("禁令/皮特只准入原有帧内选择".into()),
                    };
                    let expected = Choice {
                        id: p.choice.id.clone(),
                        kind: kind.into(),
                        title: title.into(),
                        description: "选择符合数量限制的选项，然后确认。".into(),
                        player_id: player_id(seat),
                        options,
                        preview_cards: vec![],
                        min: Some(min),
                        max: Some(max),
                        amount: None,
                        allow_decline: Some(min == 0),
                    };
                    if f.cursor != 1
                        || !matches!(f.guard, GuardState::Accepted)
                        || p.seat != seat
                        || p.choice.id.is_empty()
                        || !same(&p.choice, &expected)
                    {
                        return Err("禁令/皮特待选须保留原玩家、完整选项及数量".into());
                    }
                }
                ChoiceResolution::Frame {
                    choice: FrameChoice::XG20Number | FrameChoice::PetePay | FrameChoice::PeteDraw,
                    ..
                } => return Err("禁令/皮特选择不能移植其他来源".into()),
                _ => {}
            }
        }
        Ok(())
    }
}
