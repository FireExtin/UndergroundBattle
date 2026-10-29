//! XG13: original-entry identity, a conditional time marker, and the existing
//! one-use red/blood play discount. No new payment or entry interpreter.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub(crate) const ID: &str = "XG13";
pub(crate) const ENTRY: &str = "import20-xg13-cultist-entry";
pub(crate) const FAST: &str = "import20-xg13-sacrifice-discount";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CultistEntry {
    pub event_id: String,
    pub turn: u32,
    pub entered: SourceSnapshot,
    pub secrecy_at_entry: u32,
    pub cultist_tattoos: Vec<Attachment>,
}
fn same<T: Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
pub(crate) fn definition() -> Definition {
    let mut fast = ability(FAST, "快速行动：牺牲另一个邪教徒，下一张鸣钟教派或鲜血牌减2", Timing::Fast,
        vec![Cost::SacrificeControlledSubtype(vec!["邪教徒".into()])], vec![],
        vec![Op::CostReduction { filter: CardFilter::SocietyOrMagic {
            society: "鸣钟教派".into(), magic: MagicIcon::Blood }, amount: 2 }], None);
    fast.activation_only = true;
    with_abilities(vec![ability(ENTRY, "另一个本方邪教徒进场：若本方隐秘，在原角色放时间1", Timing::Fast,
        vec![], vec![], vec![Op::XG13EntryTime { observation: None }], Some(Event::XG13OtherCultistEntered)), fast])
}
fn owns(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::XG13EntryTime { .. } => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => owns(body),
        Op::IfTargetExhausted { exhausted, ready, .. } =>
            owns(std::slice::from_ref(exhausted)) || owns(std::slice::from_ref(ready)),
        _ => false,
    })
}
pub(crate) fn related(id: &str, a: &AbilitySpec) -> bool {
    id == ID || a.key.starts_with("import20-xg13-") || a.event == Some(Event::XG13OtherCultistEntered)
        || owns(&a.ops) || a.modes.iter().any(|m| owns(&m.ops))
}
pub(crate) fn frame_related(f: &ResolutionFrame) -> bool {
    f.source.card.definition == ID || f.ability_key.starts_with("import20-xg13-")
        || f.steps.iter().any(|s| owns(std::slice::from_ref(&s.op)))
}
fn canonical(a: &AbilitySpec) -> Option<AbilitySpec> {
    let mut expected = definition().abilities.into_iter().find(|b| b.key == a.key)?;
    if let [Op::XG13EntryTime { observation }] = a.ops.as_slice() {
        expected.ops = vec![Op::XG13EntryTime { observation: observation.clone() }];
    }
    Some(expected)
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if id != ID || canonical(a).is_none_or(|b| !same(a, &b)) {
        return Err("玛丽只能保留完整进场时间或牺牲减费程序".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if id == ID && !same(d, &definition()) || id != ID && d.abilities.iter().any(|a| related(id, a)) {
        return Err("玛丽完整定义不能删改或移植".into());
    }
    Ok(())
}
pub(crate) fn validate_action(a: &Action) -> RuleResult<()> {
    let expected = match a.kind.as_str() {
        "deploy" | "conceal" => Action { card_id: a.card_id.clone(), region: a.region, ..Action::new(&a.kind) },
        "reveal" => Action { card_id: a.card_id.clone(), ..Action::new("reveal") },
        "activate" if a.ability_id.as_deref() == Some(FAST) => Action {
            card_id: a.card_id.clone(), ability_id: a.ability_id.clone(), cost_selected: a.cost_selected.clone(), ..Action::new("activate") },
        _ => return Err("玛丽没有该行动或主动触发程序".into()),
    };
    if *a != expected { return Err("玛丽行动不可追加目标、模式或费用".into()); }
    Ok(())
}
impl Game {
    pub(crate) fn xg13_check_cost(&self, s: &SourceSnapshot, a: &Action) -> RuleResult<()> {
        if s.card.definition == ID && a.ability_id.as_deref() == Some(FAST)
            && a.cost_selected.as_ref().is_none_or(|ids| ids.len() != 1 || ids[0] == s.card.id) {
            return Err("必须牺牲另一个本方邪教徒，不能支付玛丽自身".into());
        }
        Ok(())
    }
    pub(crate) fn xg13_observe_entry(&mut self, id: &str) {
        let Some((region, c)) = self.board(id).filter(|(_, c)| !c.face_down
            && catalog::has_kind(catalog::card(&c.definition), "character")
            && self.current_subtypes(c).iter().any(|s| s == "邪教徒")
            && self.players[c.controller].society_zone.secrecy > 0) else { return; };
        let sources = self.in_play_cards().into_iter().filter(|(_, source)| source.definition == ID
            && source.id != c.id && !source.face_down && source.controller == c.controller
            && !self.players[source.controller].eliminated)
            .map(|(r, source)| self.source_snapshot(source, r)).collect::<Vec<_>>();
        if sources.is_empty() { return; }
        let mut entered = self.source_snapshot(c, Some(region));
        entered.source_region_instance = Some(self.regions[region].card.id.clone());
        let cultist_tattoos = self.attachments.iter().filter(|a| a.card.definition == "WM038"
            && !a.card.face_down && a.host_id == c.id && self.attachment_host_valid(a)).cloned().collect();
        let secrecy_at_entry = self.players[entered.card.controller].society_zone.secrecy;
        let observation = CultistEntry { event_id: self.id(), turn: self.turn, entered, secrecy_at_entry, cultist_tattoos };
        for source in sources {
            let mut a = definition().abilities.remove(0);
            a.ops = vec![Op::XG13EntryTime { observation: Some(Box::new(observation.clone())) }];
            self.effects.push_back(Effect::Declare { declaration: Declaration { actor: source.card.controller, source, ability: a } });
        }
    }
    fn xg13_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        let c = &s.card;
        if actor >= self.players.len() || c.definition != ID || c.controller != actor || c.owner >= self.players.len()
            || !self.mystic_instance_valid(&c.id) || s.region.is_none_or(|r| r >= self.regions.len())
            || s.source_region_instance.as_ref().is_none_or(|id| !self.mystic_instance_valid(id) || *id == c.id)
            || s.attachment_host_instance.is_some() || s.death_echo.is_some() || s.observed_death.is_some()
            || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some() {
            return Err("玛丽须保留原角色、操控者与地区身份".into());
        }
        self.mobility_immutable_identity(c)
    }
    fn xg13_entry<'a>(&self, actor: usize, source_id: &str, ops: &'a [Op]) -> RuleResult<&'a CultistEntry> {
        let [Op::XG13EntryTime { observation: Some(o) }] = ops else { return Err("玛丽缺少原邪教徒进场收据".into()); };
        let s = &o.entered; let c = &s.card;
        let Some(print) = catalog::catalog().cards.iter().find(|d| d.id == c.definition && catalog::runtime_implemented(d)) else { return Err("进场角色定义未知".into()); };
        if o.turn != self.turn || o.secrecy_at_entry == 0 || !self.mystic_instance_valid(&o.event_id) || o.event_id == source_id
            || c.id == source_id || c.id == o.event_id || !self.mystic_instance_valid(&c.id) || c.face_down
            || c.controller != actor || c.owner >= self.players.len() || !catalog::has_kind(print, "character")
            || s.region.is_none_or(|r| r >= self.regions.len())
            || s.source_region_instance.as_ref().is_none_or(|id| !self.mystic_instance_valid(id) || *id == c.id)
            || s.play_source.is_some() || s.death_echo.is_some() || s.observed_death.is_some()
            || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some() {
            return Err("邪教徒进场收据的原实体、事件或个人控制者无效".into());
        }
        self.mobility_immutable_identity(c)?;
        let mut ids = BTreeSet::new();
        for a in &o.cultist_tattoos {
            if a.card.definition != "WM038" || a.card.face_down || a.host_id != c.id || a.card.id == c.id
                || !self.mystic_instance_valid(&a.card.id) || !ids.insert(&a.card.id)
                || a.card.controller >= self.players.len() || a.card.owner >= self.players.len()
                || !print.subtypes.iter().any(|s| s == "人类") {
                return Err("获得邪教徒类别须保留原人类宿主与信徒烙印".into());
            }
            self.mobility_immutable_identity(&a.card)?;
        }
        if !print.subtypes.iter().any(|s| s == "邪教徒") && o.cultist_tattoos.is_empty() {
            return Err("进场者没有实际邪教徒类别".into());
        }
        Ok(o)
    }
    pub(crate) fn validate_xg13_declaration(&self, d: &Declaration) -> RuleResult<()> {
        self.xg13_source(d.actor, &d.source)?;
        validate_ability(ID, &d.ability)?;
        if d.source.card.face_down || d.source.play_source.is_some() || d.ability.key != ENTRY {
            return Err("玛丽只有实际另一邪教徒进场可声明触发".into());
        }
        self.xg13_entry(d.actor, &d.source.card.id, &d.ability.ops)?;
        Ok(())
    }
    pub(crate) fn validate_xg13_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if self.validated_cross_card_frame(f)? { return Ok(()); }
        self.xg13_source(f.actor, &f.source)?;
        let c = &f.source.card;
        if !self.mystic_instance_valid(&f.frame_id) || f.frame_id == c.id || f.chosen_region.is_some() && f.chosen_region != f.source.region
            || f.cursor > f.steps.len() || f.steps.iter().any(|s| s.context != f.actor)
            || !f.targets.is_empty() || f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked)
            || f.cursor > 0 && !matches!(f.guard, GuardState::Accepted) {
            return Err("玛丽帧必须保留原目标、程序、守卫与阶段".into());
        }
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            if !f.steps.is_empty() || f.chosen_region != f.source.region
                || f.ability_key == "deploy" && (c.face_down || !f.source.valid_character_play())
                || f.ability_key == "reveal" && (!c.face_down || f.source.play_source.is_some())
                || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if f.money_units(0) <= 2
                    && ids.iter().all(|id| f.source.valid_printed_payment_id(id)) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len()) {
                return Err("玛丽派遣/现身须保留实际出牌与支付".into());
            }
            return Ok(());
        }
        let mut a = definition().abilities.into_iter().find(|a| a.key == f.ability_key).ok_or("未知玛丽能力")?;
        a.ops = f.steps.iter().map(|s| s.op.clone()).collect();
        validate_ability(ID, &a)?;
        if c.face_down || f.source.play_source.is_some() || f.chosen_region.is_some() { return Err("玛丽场内能力来源无效".into()); }
        if f.ability_key == ENTRY {
            if !f.already_paid.is_empty() { return Err("进场时间不能追加费用".into()); }
            self.xg13_entry(f.actor, &c.id, &a.ops)?;
        } else {
            self.validate_batch20_payment(f, &a.costs)?;
            let [PaidCost::SacrificedSubtype { card: paid, .. }] = f.already_paid.as_slice() else { return Err("须保留原邪教徒牺牲费用".into()); };
            if paid.id == c.id || self.batch3_physical_cards().any(|now| now.id == paid.id) {
                return Err("须支付另一个邪教徒且原费用实体已离场".into());
            }
            self.mobility_immutable_identity(paid)?;
        }
        Ok(())
    }
    pub(crate) fn xg13_add_time(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        let ops = f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>();
        let id = self.xg13_entry(f.actor, &f.source.card.id, &ops)?.entered.card.id.clone();
        // Like the existing qualified-entry programs (BQ029/XQ37), freeze
        // the entry condition at that event. Later moon changes do not create
        // a missed entry or undo a declared trigger; only the original UID acts.
        if let Some(c) = self.board_mut(&id).filter(|c| !c.face_down && catalog::has_kind(catalog::card(&c.definition), "character")) {
            let placement_before = c.time_markers; let placement_id = c.id.clone(); c.time_markers = c.time_markers.saturating_add(1); self.record_board_marker_write(&placement_id, crate::batch4_markers::MarkerKind::Time, placement_before);
        }
        Ok(())
    }
    pub(crate) fn validate_xg13_state(&self) -> RuleResult<()> {
        let mut seen = BTreeSet::new();
        let mut remember = |source: &SourceSnapshot, ops: &[Op]| -> RuleResult<()> {
            if let [Op::XG13EntryTime { observation: Some(o) }] = ops {
                if !seen.insert((source.card.id.clone(), o.event_id.clone())) { return Err("同次邪教徒进场不能重复消费玛丽触发".into()); }
            }
            Ok(())
        };
        for s in &self.stack {
            if let Some(f) = &s.frame {
                if frame_related(f) && !self.validated_cross_card_frame(f)? {
                    self.validate_xg13_frame(f)?;
                    remember(&f.source, &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>())?;
                    let play = matches!(f.ability_key.as_str(), "deploy" | "reveal");
                    if s.id != f.frame_id || s.controller != f.actor || s.card.is_some() != play || s.reveal != (f.ability_key == "reveal")
                        || s.deploy_region != (if play { f.source.region } else { None }) || s.target.is_some()
                        || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked) {
                        return Err("玛丽响应堆叠必须保留原外壳".into());
                    }
                }
            }
        }
        for e in &self.effects { match e {
            Effect::Declare { declaration: d } if related(&d.source.card.definition, &d.ability) && !self.validated_cross_card_declaration(d)? => {
                self.validate_xg13_declaration(d)?; remember(&d.source, &d.ability.ops)?;
            },
            Effect::Frame { frame: f } if frame_related(f) && !self.validated_cross_card_frame(f)? => {
                self.validate_xg13_frame(f)?;
            },
            _ => {},
        } }
        if let Some(p) = &self.pending { match &p.resolution {
            ChoiceResolution::Declare { declaration: d, stage } if related(&d.source.card.definition, &d.ability) && !self.validated_cross_card_declaration(d)? => {
                self.validate_xg13_declaration(d)?; remember(&d.source, &d.ability.ops)?;
                let expected = Choice { id: p.choice.id.clone(), kind: "trigger".into(),
                    title: format!("{}：{}", catalog::card(ID).name, d.ability.label),
                    description: "选择符合数量限制的选项，然后确认。".into(), player_id: player_id(d.actor),
                    options: vec![ChoiceOption { id: "accept".into(), label: "发动触发能力".into(), card: None }],
                    preview_cards: vec![], min: Some(0), max: Some(1), amount: None, allow_decline: Some(true) };
                if p.seat != d.actor || p.choice.id.is_empty() || !matches!(stage, DeclareChoice::Accept) || !same(&p.choice, &expected) {
                    return Err("玛丽进场待选只接受原本人触发或放弃".into());
                }
            },
            ChoiceResolution::Frame { frame: f, .. } if frame_related(f) && !self.validated_cross_card_frame(f)? =>
                return Err("玛丽时间与减费是原子结算，没有私有帧内选择".into()),
            _ => {},
        } }
        Ok(())
    }
}
