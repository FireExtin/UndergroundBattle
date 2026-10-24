//! WM058 watches card transactions, never an activated/triggered ability.
//! Every distinct personal Criminal target is observed once at declaration.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub(crate) const TAX: &str = "batch4-wm058-target-tax";
pub(crate) const REVEAL: &str = "batch4-wm058-reveal";
const PAY_TITLE: &str = "钱斯：支付1，否则终止原卡牌";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetedCard { pub event_id: String, pub turn: u32, pub card: SourceSnapshot, pub ability_key: String, pub target: SourceSnapshot, pub subtypes: Vec<String> }
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum TaxOutcome { Declined, Paid { frame_id: String }, Countered { frame_id: String }, NoCard { frame_id: String } }
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaxReceipt { pub source: SourceSnapshot, pub observation: TargetedCard, pub outcome: TaxOutcome }
fn same<T: Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
fn receipt_key(s: &SourceSnapshot, o: &TargetedCard) -> (String, String, String) { (s.card.id.clone(), o.event_id.clone(), o.target.card.id.clone()) }
pub(crate) fn definition() -> Definition { with_abilities(vec![
    ability(TAX, "本方罪犯成为敌方卡牌目标：其操控者支付1或终止该卡牌", Timing::Fast, vec![], vec![], vec![Op::WM058Tax { observation: None }], Some(Event::WM058CardTargeted)),
    ability(REVEAL, "现身：移除原地区全部敌方势力", Timing::Fast, vec![], vec![], vec![Op::WM058RemoveEnemyInfluence], Some(Event::Reveal)),
]) }
pub(crate) fn canonical(a: &AbilitySpec) -> Option<AbilitySpec> {
    let mut expected = definition().abilities.into_iter().find(|b| b.key == a.key)?;
    if a.key == TAX { if let [Op::WM058Tax { observation }] = a.ops.as_slice() { expected.ops = vec![Op::WM058Tax { observation: observation.clone() }]; } } Some(expected)
}
pub(crate) fn validate_action(a: &Action) -> RuleResult<()> {
    let expected = match a.kind.as_str() { "deploy" | "conceal" => Action { card_id: a.card_id.clone(), region: a.region, ..Action::new(&a.kind) },
        "reveal" => Action { card_id: a.card_id.clone(), ..Action::new("reveal") }, _ => return Err("钱斯只有正常派遣与现身操作".into()) };
    if *a != expected { return Err("钱斯不能预选额外费用或添加载荷".into()); } Ok(())
}
pub(crate) fn validate_choice_action(p: &Pending, a: &Action) -> RuleResult<()> {
    if matches!(&p.resolution, ChoiceResolution::Declare { declaration: d, .. } if d.source.card.definition == "WM058")
        && *a != (Action { choice_id: a.choice_id.clone(), selected: a.selected.clone(), ..Action::new("choose") }) { return Err("钱斯触发待选只接受原选择".into()); } Ok(())
}
impl Game {
    fn wm058_instance(&self, id: &str) -> Option<u64> { id.strip_prefix('i').and_then(|s| s.parse::<u64>().ok()).filter(|n| *n > 0 && *n <= self.sequence && id == format!("i{n}")) }
    pub(crate) fn wm058_observe_card(&mut self, f: &ResolutionFrame) {
        if f.targets.is_empty() || !matches!(catalog::card(&f.source.card.definition).kind.as_str(), "spell" | "attachment") { return; }
        let mut sources = self.regions.iter().enumerate().filter(|(_, r)| !r.vacant).flat_map(|(r, region)| region.cards.iter().map(move |c| (r, c)))
            .filter(|(_, c)| c.definition == "WM058" && !c.face_down && !self.players[c.controller].eliminated).map(|(r, c)| self.source_snapshot(c, Some(r))).collect::<Vec<_>>();
        if sources.is_empty() { return; } sources.sort_by_key(|s| (self.team(s.card.controller) != self.first_team, s.card.controller));
        let mut seen = BTreeSet::new();
        let targets = f.targets.iter().filter(|t| t.spec.zone == Zone::Board && seen.insert(t.id.clone())).filter_map(|t| self.board(&t.id))
            .filter(|(_, c)| !c.face_down && catalog::card(&c.definition).kind == "character")
            .filter_map(|(r, c)| { let subtypes = self.current_subtypes(c); subtypes.iter().any(|t| t == "罪犯").then(|| {
                let mut source = self.source_snapshot(c, Some(r)); source.source_region_instance = Some(self.regions[r].card.id.clone()); (source, subtypes)
            }) }).collect::<Vec<_>>();
        for s in sources { for (target, subtypes) in &targets {
            if target.card.controller != s.card.controller || !self.is_enemy(f.actor, &target.card) { continue; }
            let o = TargetedCard { event_id: f.frame_id.clone(), turn: self.turn, card: f.source.clone(), ability_key: f.ability_key.clone(), target: target.clone(), subtypes: subtypes.clone() };
            let mut a = definition().abilities.remove(0); a.ops = vec![Op::WM058Tax { observation: Some(Box::new(o)) }];
            self.effects.push_back(Effect::Declare { declaration: Declaration { actor: s.card.controller, source: s.clone(), ability: a } });
        } }
    }
    fn wm058_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        let c = &s.card;
        if c.definition != "WM058" || self.wm058_instance(&c.id).is_none() || actor >= self.players.len() || c.owner >= self.players.len() || c.controller != actor
            || s.region.is_none_or(|r| r >= self.regions.len()) || s.source_region_instance.as_ref().is_none_or(|id| self.wm058_instance(id).is_none() || *id == c.id)
            || s.region.zip(s.source_region_instance.as_ref()).is_some_and(|(r, id)| self.regions.iter().enumerate().any(|(other, region)| region.card.id == *id && other != r))
            || s.observed_death.is_some() || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some() || s.attachment_host_instance.is_some() || s.death_echo.is_some()
            || s.play_source.is_none() && self.batch3_physical_cards().any(|live| live.id == c.id) && self.board(&c.id).is_none() { return Err("钱斯须保留原角色、控制者及地区实体".into()); }
        self.mobility_immutable_identity(c)
    }
    fn wm058_observation(&self, actor: usize, s: &SourceSnapshot, o: &TargetedCard) -> RuleResult<()> {
        let c = &o.card.card; let t = &o.target.card;
        let print = catalog::catalog().cards.iter().find(|d| d.id == c.definition && catalog::runtime_implemented(d) && matches!(d.kind.as_str(), "spell" | "attachment")).ok_or("钱斯只观察实际打出的敌方事务或附属卡牌")?;
        let target_print = catalog::catalog().cards.iter().find(|d| d.id == t.definition && d.kind == "character" && catalog::runtime_implemented(d)).ok_or("钱斯目标须为当前角色")?;
        let body = crate::rules::definition(&c.definition).abilities.iter().any(|a| a.key == o.ability_key && a.event.is_none() && !a.activation_only && (print.kind == "spell" || a.key == "attach"));
        if !body || actor >= self.players.len() || c.controller >= self.players.len() || c.owner >= self.players.len() || c.face_down || t.face_down || t.owner >= self.players.len() || t.controller != actor || !self.is_enemy(c.controller, t)
            || o.turn != self.turn || self.wm058_instance(&o.event_id).is_none_or(|event| [s.card.id.as_str(), c.id.as_str(), t.id.as_str()].iter().any(|id| self.wm058_instance(id).is_none_or(|n| n >= event)))
            || o.card.region.is_some() || o.card.source_region_instance.is_some() || o.card.play_source.is_some() || o.card.attachment_host_instance.is_some() || o.card.death_echo.is_some() || o.card.observed_death.is_some() || o.card.observed_exposure.is_some() || o.card.observed_mobility_grant.is_some()
            || o.target.region.is_none_or(|r| r >= self.regions.len()) || o.target.source_region_instance.as_ref().is_none_or(|id| self.wm058_instance(id).is_none() || *id == t.id) || o.target.play_source.is_some() || o.target.death_echo.is_some() || o.target.observed_death.is_some() || o.target.observed_exposure.is_some() || o.target.observed_mobility_grant.is_some()
            || !target_print.subtypes.iter().any(|t| t == "罪犯") || !o.subtypes.iter().any(|t| t == "罪犯") || o.subtypes.iter().collect::<BTreeSet<_>>().len() != o.subtypes.len()
            || o.subtypes.iter().any(|t| !target_print.subtypes.contains(t) && !matches!(t.as_str(), "吸血鬼" | "奴仆" | "邪教徒" | "宿主" | "不死生物" | "骷髅" | "蝙蝠"))
            || self.batch3_physical_cards().any(|live| live.id == c.id) || self.batch3_physical_cards().any(|live| live.id == t.id) && self.board(&t.id).is_none()
            || self.stack.iter().any(|item| item.id == o.event_id && (item.card.as_ref().is_none_or(|card| card.definition != print.id || card.owner != c.owner || card.controller != c.controller)
                || item.frame.as_ref().is_none_or(|frame| !same(&frame.source, &o.card) || frame.actor != c.controller || frame.ability_key != o.ability_key || !frame.targets.iter().any(|target| target.id == t.id && target.spec.zone == Zone::Board)))) {
            return Err("钱斯须保留原敌方卡牌、个人罪犯资格及实际目标事件".into());
        } self.mobility_immutable_identity(c)?; self.mobility_immutable_identity(t)
    }
    fn wm058_expected(&self, s: &SourceSnapshot, key: &str, ops: &[Op]) -> RuleResult<AbilitySpec> {
        if key == "renown" || key == "jc089-combat-glory" { let region = s.source_region_instance.as_deref().ok_or("钱斯奖励缺少原地区")?; return Ok(if key == "renown" { crate::renown::renown_ability(region) } else { crate::rules::jc089_combat_glory_ability(region) }); }
        let mut a = definition().abilities.into_iter().find(|a| a.key == key).ok_or("钱斯未知程序")?; if key == TAX { a.ops = ops.to_vec(); } Ok(a)
    }
    pub(crate) fn validate_wm058_declaration(&self, d: &Declaration) -> RuleResult<()> {
        self.wm058_source(d.actor, &d.source)?; let expected = self.wm058_expected(&d.source, &d.ability.key, &d.ability.ops)?;
        if d.source.card.face_down || d.source.play_source.is_some() || !same(&expected, &d.ability) { return Err("钱斯须保留完整触发或既有奖励".into()); }
        if d.ability.key == TAX { let [Op::WM058Tax { observation: Some(o) }] = d.ability.ops.as_slice() else { return Err("钱斯缺少真实卡牌指定目标观察".into()); };
            self.wm058_observation(d.actor, &d.source, o)?; if canonical(&d.ability).is_none_or(|a| !same(&a, &d.ability)) { return Err("钱斯固定触发不能改写".into()); }
            if self.wm058_receipts.iter().any(|r| receipt_key(&r.source, &r.observation) == receipt_key(&d.source, o)) { return Err("钱斯的原卡牌目标触发已经消费".into()); }
        } Ok(())
    }
    pub(crate) fn validate_wm058_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        self.wm058_source(f.actor, &f.source)?;
        if self.wm058_instance(&f.frame_id).is_none_or(|n| n <= self.wm058_instance(&f.source.card.id).unwrap()) || f.cursor > f.steps.len()
            || f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked) || f.cursor > 0 && !matches!(f.guard, GuardState::Accepted) { return Err("钱斯帧身份、游标或守卫无效".into()); }
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            if !f.steps.is_empty() || !f.targets.is_empty() || f.chosen_region != f.source.region
                || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if f.money_units(0) <= 3 && ids.iter().all(|id| !id.is_empty() && *id != f.source.card.id) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len())
                || f.ability_key == "deploy" && (f.source.card.face_down || !f.source.valid_character_play()) || f.ability_key == "reveal" && (!f.source.card.face_down || f.source.play_source.is_some()) { return Err("钱斯派遣/现身不能更改地区、支付或程序".into()); } return Ok(());
        }
        let ops = f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>(); let a = self.wm058_expected(&f.source, &f.ability_key, &ops)?;
        self.validate_wm058_declaration(&Declaration { actor: f.actor, source: f.source.clone(), ability: a.clone() })?;
        if !same(&ops, &a.ops) || f.steps.iter().any(|s| s.context != f.actor) || !f.already_paid.is_empty() || !f.targets.is_empty() || f.chosen_region.is_some() { return Err("钱斯完整触发不能添加目标、费用或额外操作".into()); } Ok(())
    }
    fn wm058_tax<'a>(&self, f: &'a ResolutionFrame) -> RuleResult<&'a TargetedCard> { match f.steps.as_slice() { [Step { op: Op::WM058Tax { observation: Some(o) }, .. }] if f.ability_key == TAX => Ok(o), _ => Err("钱斯支付缺少原卡牌目标观察".into()) } }
    pub(crate) fn wm058_decline(&mut self, d: &Declaration) -> RuleResult<()> {
        if d.source.card.definition == "WM058" && d.ability.key == TAX { self.validate_wm058_declaration(d)?; let [Op::WM058Tax { observation: Some(o) }] = d.ability.ops.as_slice() else { unreachable!() };
            self.wm058_receipts.push(TaxReceipt { source: d.source.clone(), observation: *o.clone(), outcome: TaxOutcome::Declined });
        } Ok(())
    }
    fn wm058_card_present(&self, o: &TargetedCard) -> bool { self.stack.iter().any(|item| item.id == o.event_id && item.card.is_some()) }
    fn wm058_pay_options(&self, seat: usize) -> Vec<ChoiceOption> { let mut options = vec![]; if self.resources(seat) >= 1 { options.push(ChoiceOption { id: "pay".into(), label: "支付1".into(), card: None }); } options.push(ChoiceOption { id: "decline".into(), label: "不支付，终止原卡牌".into(), card: None }); options }
    fn wm058_finish_tax(&mut self, f: &ResolutionFrame, pay: bool) -> RuleResult<()> {
        self.validate_wm058_frame(f)?; let o = self.wm058_tax(f)?.clone();
        if f.cursor != 1 || !matches!(f.guard, GuardState::Accepted) || self.wm058_receipts.iter().any(|r| receipt_key(&r.source, &r.observation) == receipt_key(&f.source, &o)) { return Err("同一钱斯不能重复消费同次目标事件".into()); }
        let outcome = if let Some(index) = self.stack.iter().position(|item| item.id == o.event_id && item.card.is_some()) {
            if pay { self.pay(o.card.card.controller, 1)?; TaxOutcome::Paid { frame_id: f.frame_id.clone() } }
            else { let item = self.stack.remove(index); let card = item.card.unwrap(); let name = catalog::card(&card.definition).name.clone();
                // A countered transaction follows the existing Bury path,
                // including its owner and recorded caster, rather than a
                // board-character death/controller reset.
                if !self.players[card.owner].eliminated { let card = self.fresh(card); self.commit_owner_grave(card, None); }
                self.note(format!("钱斯：不支付额外1，终止{name}")); TaxOutcome::Countered { frame_id: f.frame_id.clone() } }
        } else { TaxOutcome::NoCard { frame_id: f.frame_id.clone() } };
        self.wm058_receipts.push(TaxReceipt { source: f.source.clone(), observation: o, outcome }); Ok(())
    }
    pub(crate) fn wm058_demand(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        self.validate_wm058_frame(&f)?; let o = self.wm058_tax(&f)?; let seat = o.card.card.controller;
        if !self.wm058_card_present(o) || self.resources(seat) == 0 || self.players[seat].eliminated { return self.wm058_finish_tax(&f, false); }
        self.choice(seat, "wm058-payment", PAY_TITLE.into(), self.wm058_pay_options(seat), 1, 1, None, ChoiceResolution::Frame { frame: Box::new(f), choice: FrameChoice::WM058Payment }); Ok(())
    }
    pub(crate) fn wm058_payment(&mut self, f: ResolutionFrame, actor: usize, a: &Action, selected: &[String]) -> RuleResult<()> {
        self.validate_wm058_frame(&f)?; let o = self.wm058_tax(&f)?;
        if actor != o.card.card.controller || selected.len() != 1 || !self.wm058_pay_options(actor).iter().any(|v| v.id == selected[0])
            || *a != (Action { choice_id: a.choice_id.clone(), selected: a.selected.clone(), ..Action::new("choose") }) { return Err("钱斯额外费用只能由原卡牌操控者按原选项支付或拒绝".into()); }
        self.wm058_finish_tax(&f, selected[0] == "pay")
    }
    pub(crate) fn wm058_reveal(&mut self, f: &ResolutionFrame) {
        if let Some(r) = f.source.region.filter(|r| !self.regions[*r].vacant && f.source.source_region_instance.as_ref() == Some(&self.regions[*r].card.id)) { let enemy = 1 - self.team(f.actor); self.regions[r].influence[enemy] = 0; }
    }
    pub(crate) fn validate_wm058_state(&self) -> RuleResult<()> {
        let mut frames = BTreeSet::new(); let mut events = BTreeSet::new();
        for r in &self.wm058_receipts { self.wm058_source(r.source.card.controller, &r.source)?; self.wm058_observation(r.source.card.controller, &r.source, &r.observation)?;
            if r.source.card.face_down || r.source.play_source.is_some() || !events.insert(receipt_key(&r.source, &r.observation)) { return Err("钱斯目标消费收据重复或来源无效".into()); }
            if let TaxOutcome::Paid { frame_id } | TaxOutcome::Countered { frame_id } | TaxOutcome::NoCard { frame_id } = &r.outcome {
                if self.wm058_instance(frame_id).is_none_or(|n| n <= self.wm058_instance(&r.observation.event_id).unwrap()) || !frames.insert(frame_id.clone()) { return Err("钱斯消费须保留后于卡牌声明的独立帧".into()); }
            }
        }
        let mut remember = |s: &SourceSnapshot, ops: &[Op]| -> RuleResult<()> { if let [Op::WM058Tax { observation: Some(o) }] = ops { if !events.insert(receipt_key(s, o)) { return Err("钱斯同次卡牌目标事件已消费或重复排队".into()); } } Ok(()) };
        for item in &self.stack { if let Some(f) = &item.frame { if f.source.card.definition == "WM058" && !self.validated_cross_card_frame(f)? {
            self.validate_wm058_frame(f)?; remember(&f.source, &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>())?; let play = matches!(f.ability_key.as_str(), "deploy" | "reveal");
            if !frames.insert(f.frame_id.clone()) || item.id != f.frame_id || item.controller != f.actor || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked) || item.card.is_some() != play || item.reveal != (f.ability_key == "reveal") || item.deploy_region != if play { f.source.region } else { None } || item.target.is_some()
                || item.card.as_ref().is_some_and(|c| c.definition != "WM058" || c.face_down || c.owner != f.source.card.owner || c.controller != f.actor) { return Err("钱斯堆叠外壳或帧重复".into()); }
        } } }
        for e in &self.effects { match e { Effect::Declare { declaration: d } if d.source.card.definition == "WM058" && !self.validated_cross_card_declaration(d)? => { self.validate_wm058_declaration(d)?; remember(&d.source, &d.ability.ops)?; }, Effect::Frame { frame: f } if f.source.card.definition == "WM058" && !self.validated_cross_card_frame(f)? => return Err("钱斯程序不能绕过响应".into()), _ => {}, } }
        if let Some(p) = &self.pending { match &p.resolution {
            ChoiceResolution::Declare { declaration: d, stage } if d.source.card.definition == "WM058" && !self.validated_cross_card_declaration(d)? => {
                self.validate_wm058_declaration(d)?; remember(&d.source, &d.ability.ops)?;
                let expected = Choice { id: p.choice.id.clone(), kind: "trigger".into(), title: format!("{}：{}", catalog::card("WM058").name, d.ability.label), description: "选择符合数量限制的选项，然后确认。".into(), player_id: player_id(d.actor), options: vec![ChoiceOption { id: "accept".into(), label: "发动触发能力".into(), card: None }], preview_cards: vec![], min: Some(0), max: Some(1), amount: None, allow_decline: Some(true) };
                if !matches!(stage, DeclareChoice::Accept) || p.seat != d.actor || p.choice.id.is_empty() || !same(&p.choice, &expected) { return Err("钱斯触发须由原操控者接受或放弃".into()); }
            },
            ChoiceResolution::Frame { frame: f, choice } if f.source.card.definition == "WM058" => {
                self.validate_wm058_frame(f)?; let o = self.wm058_tax(f)?; let seat = o.card.card.controller; remember(&f.source, &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>())?;
                let expected = Choice { id: p.choice.id.clone(), kind: "wm058-payment".into(), title: PAY_TITLE.into(), description: "选择符合数量限制的选项，然后确认。".into(), player_id: player_id(seat), options: self.wm058_pay_options(seat), preview_cards: vec![], min: Some(1), max: Some(1), amount: None, allow_decline: Some(false) };
                if !matches!(choice, FrameChoice::WM058Payment) || f.cursor != 1 || !matches!(f.guard, GuardState::Accepted) || self.resources(seat) == 0 || !self.wm058_card_present(o) || !frames.insert(f.frame_id.clone()) || p.seat != seat || p.choice.id.is_empty() || !same(&p.choice, &expected) { return Err("钱斯支付只能保存原卡牌操控者及原费用选项".into()); }
            },
            ChoiceResolution::Frame { choice: FrameChoice::WM058Payment, .. } => return Err("钱斯支付不能移植其他来源".into()), _ => {},
        } } Ok(())
    }
}
