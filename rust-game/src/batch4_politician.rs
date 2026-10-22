//! WM029: marker/seal conditions remain live; its resolved death effect is a
//! turn-wide conditional rule, not a snapshot of the characters then present.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub(crate) const KEY: &str = "batch4-wm029-death-aura";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PoliticianAura { pub source: SourceSnapshot, pub actor: usize, pub frame_id: String, pub expires_turn: u32 }
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkerRenown { pub attachments: Vec<Attachment>, pub aura: Option<Box<PoliticianAura>> }
fn same<T: Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
pub(crate) fn marker_count(c: &Card) -> u32 {
    // Damage is a turn-local amount, not a marker. Wounds and shields are
    // explicitly markers in the Hegemony glossary (printed pages17/18).
    [c.advancement_markers, c.permanent_combat_markers, c.permanent_defense_markers,
        c.time_markers, c.wounds, c.shield, c.lock_markers].into_iter().fold(0, u32::saturating_add)
}
pub(crate) fn definition() -> Definition {
    let mut d = with_abilities(vec![ability(KEY, "本回合：带标志角色获得声望，带封印角色获得屏障", Timing::Fast,
        vec![], vec![], vec![Op::WM029TurnAura], Some(Event::Death))]);
    d.traits.public = true; d
}
pub(crate) fn canonical(a: &AbilitySpec) -> Option<AbilitySpec> { definition().abilities.into_iter().find(|s| s.key == a.key) }
pub(crate) fn validate_action(a: &Action) -> RuleResult<()> {
    let expected = match a.kind.as_str() {
        "deploy" | "conceal" => Action { card_id: a.card_id.clone(), region: a.region, ..Action::new(&a.kind) },
        "reveal" => Action { card_id: a.card_id.clone(), ..Action::new("reveal") },
        _ => return Err("被胁迫的政要只有正常派遣及现身操作".into()),
    };
    if *a != expected { return Err("政要不能预选死亡效果或附加额外载荷".into()); } Ok(())
}
pub(crate) fn validate_choice_action(p: &Pending, a: &Action) -> RuleResult<()> {
    if matches!(&p.resolution, ChoiceResolution::Declare { declaration: d, .. } if d.source.card.definition == "WM029")
        && *a != (Action { choice_id: a.choice_id.clone(), selected: a.selected.clone(), ..Action::new("choose") }) {
        return Err("政要触发待选只接受原选择".into());
    } Ok(())
}
impl Game {
    fn wm029_marker_attachments(&self, c: &Card) -> Vec<Attachment> {
        // A dual-type attached character may itself carry an attachment. A
        // marker on that attachment is also on its host, then on that host's
        // host. Retain the complete path as the entry-time witness.
        let mut result = BTreeMap::new();
        for leaf in self.attachments.iter().filter(|a| marker_count(&a.card) > 0) {
            let mut path = vec![]; let mut visited = BTreeSet::new(); let mut current = leaf;
            loop {
                if current.card.face_down || !visited.insert(&current.card.id) || !self.attachment_host_valid(current) { break; }
                path.push(current.clone());
                if current.host_id == c.id { for a in path { result.insert(a.card.id.clone(), a); } break; }
                let Some(parent) = self.attachments.iter().find(|a| a.card.id == current.host_id) else { break; };
                current = parent;
            }
        } result.into_values().collect()
    }
    pub(crate) fn wm029_has_markers(&self, c: &Card) -> bool {
        marker_count(c) > 0 || !self.wm029_marker_attachments(c).is_empty()
    }
    fn wm029_eligible(&self, c: &Card) -> bool {
        !c.face_down && catalog::card(&c.definition).kind == "character"
            && (c.definition == "WM029" || self.politician_auras.iter().any(|a| a.expires_turn == self.turn))
    }
    pub(crate) fn wm029_renown(&self, c: &Card) -> bool { self.wm029_eligible(c) && self.wm029_has_markers(c) }
    pub(crate) fn wm029_barrier(&self, c: &Card) -> bool {
        self.wm029_eligible(c) && self.sealed_cards.iter().any(|s| s.host_id == c.id)
    }
    pub(crate) fn wm029_renown_basis(&self, c: &Card) -> Option<MarkerRenown> {
        if !self.wm029_renown(c) { return None; }
        Some(MarkerRenown { attachments: self.wm029_marker_attachments(c), aura: if c.definition == "WM029" { None }
            else { self.politician_auras.iter().find(|a| a.expires_turn == self.turn).cloned().map(Box::new) } })
    }
    fn wm029_instance(&self, id: &str) -> Option<u64> {
        id.strip_prefix('i').and_then(|s| s.parse::<u64>().ok()).filter(|n| *n > 0 && *n <= self.sequence && id == format!("i{n}"))
    }
    fn wm029_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        let c = &s.card;
        if c.definition != "WM029" || self.wm029_instance(&c.id).is_none() || actor >= self.players.len() || c.owner >= self.players.len() || c.controller != actor
            || s.region.is_none_or(|r| r >= self.regions.len()) || s.source_region_instance.as_ref().is_none_or(|id| self.wm029_instance(id).is_none() || *id == c.id)
            || s.region.zip(s.source_region_instance.as_ref()).is_some_and(|(r, id)| self.regions.iter().enumerate().any(|(other, region)| region.card.id == *id && other != r))
            || s.observed_death.is_some() || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some() || s.attachment_host_instance.is_some()
            || s.play_source.is_none() && self.batch3_physical_cards().any(|live| live.id == c.id) && self.board(&c.id).is_none() {
            return Err("政要须保留原角色、控制者及地区实体".into());
        } self.mobility_immutable_identity(c)
    }
    fn wm029_expected(&self, s: &SourceSnapshot, key: &str) -> RuleResult<AbilitySpec> {
        if key == KEY { return Ok(definition().abilities.remove(0)); }
        let region = s.source_region_instance.as_deref().ok_or("政要奖励缺少原地区")?;
        match key { "renown" => Ok(crate::renown::renown_ability(region)), "jc089-combat-glory" => Ok(crate::rules::jc089_combat_glory_ability(region)), _ => Err("政要未知程序".into()) }
    }
    pub(crate) fn validate_wm029_declaration(&self, d: &Declaration) -> RuleResult<()> {
        self.wm029_source(d.actor, &d.source)?;
        if d.source.death_echo.is_some() { return self.validate_statue_echo_declaration(d); }
        let a = self.wm029_expected(&d.source, &d.ability.key)?;
        if d.source.card.face_down || d.source.play_source.is_some() || !same(&a, &d.ability)
            || a.key == KEY && self.batch3_physical_cards().any(|c| c.id == d.source.card.id) {
            return Err("政要须保留真实死亡后的完整触发或既有奖励".into());
        } Ok(())
    }
    pub(crate) fn validate_wm029_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        self.wm029_source(f.actor, &f.source)?;
        if f.source.death_echo.is_some() { return self.validate_statue_echo_frame(f); }
        if self.wm029_instance(&f.frame_id).is_none_or(|n| n <= self.wm029_instance(&f.source.card.id).unwrap())
            || f.cursor > f.steps.len() || f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked) || f.cursor > 0 && !matches!(f.guard, GuardState::Accepted) {
            return Err("政要帧身份、游标或守卫被改写".into());
        }
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            if !f.steps.is_empty() || !f.targets.is_empty() || f.chosen_region != f.source.region || f.source.death_echo.is_some()
                || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if ids.len() <= 1 && ids.iter().all(|id| !id.is_empty() && *id != f.source.card.id) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len())
                || f.ability_key == "deploy" && (f.source.card.face_down || !f.source.valid_character_play())
                || f.ability_key == "reveal" && (!f.source.card.face_down || f.source.play_source.is_some()) {
                return Err("政要派遣/现身须保留原地区、实际支付及空程序".into());
            } return Ok(());
        }
        let a = self.wm029_expected(&f.source, &f.ability_key)?;
        self.validate_wm029_declaration(&Declaration { actor: f.actor, source: f.source.clone(), ability: a.clone() })?;
        let expected = a.ops.iter().map(|op| Step { context: f.actor, op: op.clone() }).collect::<Vec<_>>();
        if !same(&f.steps, &expected) || !f.already_paid.is_empty() || f.chosen_region.is_some() || !f.targets.is_empty() { return Err("政要死亡持续效果不能改写程序或添加目标、费用".into()); }
        Ok(())
    }
    fn wm029_aura_valid(&self, a: &PoliticianAura) -> RuleResult<()> {
        let f = ResolutionFrame { frame_id: a.frame_id.clone(), actor: a.actor, source: a.source.clone(), ability_key: KEY.into(),
            targets: vec![], already_paid: vec![], chosen_region: None, steps: vec![Step { context: a.actor, op: Op::WM029TurnAura }], cursor: 1, guard: GuardState::Accepted };
        if a.expires_turn != self.turn { return Err("政要全场持续效果只能保留当前回合".into()); } self.validate_wm029_frame(&f)
    }
    pub(crate) fn validate_marker_renown(&self, c: &Card, witness: &MarkerRenown) -> RuleResult<()> {
        let mut ids = BTreeSet::new();
        if marker_count(c) == 0 && !witness.attachments.iter().any(|a| marker_count(&a.card) > 0) { return Err("声望进场缺少实际指示物标志".into()); }
        for a in &witness.attachments {
            let ac = &a.card;
            if ac.face_down || self.wm029_instance(&ac.id).is_none() || ac.id == c.id
                || ac.owner >= self.players.len() || ac.controller >= self.players.len() || !ids.insert(&ac.id)
                || self.batch3_physical_cards().any(|live| live.id == ac.id) && !self.attachments.iter().any(|live| live.card.id == ac.id)
                || !catalog::catalog().cards.iter().any(|d| d.id == ac.definition && catalog::runtime_implemented(d) && (d.kind == "attachment" || d.id == "BQ026")) {
                return Err("附属指示物声望须保留原宿主及实际标志来源".into());
            } self.mobility_immutable_identity(ac)?;
        }
        for a in &witness.attachments {
            let mut current = a; let mut visited = BTreeSet::new();
            let spec = crate::rules::definition(&a.card.definition).attachment.as_ref().ok_or("标志宿主链缺少实际附属定义")?;
            let host = if a.host_id == c.id { c } else { &witness.attachments.iter().find(|other| other.card.id == a.host_id).ok_or("标志宿主链缺少中间角色")?.card };
            let relation = match spec.host.relation { Relation::Any => true, Relation::ControlledByActor => host.controller == a.card.controller,
                Relation::OwnedByActor => host.owner == a.card.controller, Relation::FriendlyTeam => self.team(host.controller) == self.team(a.card.controller), Relation::EnemyTeam => self.team(host.controller) != self.team(a.card.controller) };
            if spec.host.zone != Zone::Board || spec.host.kind != EntityKind::Character || host.face_down || catalog::card(&host.definition).kind != "character" || !relation {
                return Err("指示物只能沿原角色附属宿主关系继承".into());
            }
            while current.host_id != c.id {
                if !visited.insert(&current.card.id) { return Err("附属标志继承不能循环".into()); }
                current = witness.attachments.iter().find(|other| other.card.id == current.host_id).ok_or("附属标志声望缺少通向原角色的完整宿主链")?;
            }
        }
        match &witness.aura {
            None if c.definition == "WM029" => Ok(()),
            Some(a) => self.wm029_aura_valid(a),
            _ => Err("其他带标志角色缺少已结算政要死亡效果".into()),
        }
    }
    pub(crate) fn wm029_resolve(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        self.validate_wm029_frame(f)?;
        if f.cursor != 1 || !matches!(f.guard, GuardState::Accepted) || self.politician_auras.iter().any(|a| a.frame_id == f.frame_id) { return Err("政要死亡效果只能消费一次原完整帧".into()); }
        self.politician_auras.push(PoliticianAura { source: f.source.clone(), actor: f.actor, frame_id: f.frame_id.clone(), expires_turn: self.turn }); Ok(())
    }
    pub(crate) fn validate_wm029_state(&self) -> RuleResult<()> {
        let mut frames = BTreeSet::new(); let mut deaths = BTreeSet::new();
        let key = |s: &SourceSnapshot| (s.card.id.clone(), s.death_echo.as_ref().map(|p| (p.ritual.card.id.clone(), p.event_id.clone())));
        for a in &self.politician_auras { self.wm029_aura_valid(a)?;
            if !frames.insert(a.frame_id.clone()) || !deaths.insert(key(&a.source)) { return Err("政要已结算死亡效果重复".into()); }
        }
        let mut remember = |s: &SourceSnapshot, ability: &str| -> RuleResult<()> { if ability == KEY && !deaths.insert(key(s)) { return Err("同一政要死亡触发不能重复消费".into()); } Ok(()) };
        for item in &self.stack { if let Some(f) = &item.frame { if f.source.card.definition == "WM029" && !self.validated_cross_card_frame(f)? {
            self.validate_wm029_frame(f)?; remember(&f.source, &f.ability_key)?; let play = matches!(f.ability_key.as_str(), "deploy" | "reveal");
            if !frames.insert(f.frame_id.clone()) || item.id != f.frame_id || item.controller != f.actor || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked)
                || item.card.is_some() != play || item.reveal != (f.ability_key == "reveal") || item.deploy_region != if play { f.source.region } else { None } || item.target.is_some()
                || item.card.as_ref().is_some_and(|c| c.definition != "WM029" || c.face_down || c.owner != f.source.card.owner || c.controller != f.actor) { return Err("政要堆叠外壳或已消费死亡重复".into()); }
        } } }
        for e in &self.effects { match e {
            Effect::Declare { declaration: d } if d.source.card.definition == "WM029" && !self.validated_cross_card_declaration(d)? => { self.validate_wm029_declaration(d)?; remember(&d.source, &d.ability.key)?; },
            Effect::Frame { frame: f } if f.source.card.definition == "WM029" && !self.validated_cross_card_frame(f)? => return Err("政要死亡效果不能绕过响应".into()), _ => {},
        } }
        if let Some(p) = &self.pending { match &p.resolution {
            ChoiceResolution::Declare { declaration: d, stage } if d.source.card.definition == "WM029" && !self.validated_cross_card_declaration(d)? => {
                self.validate_wm029_declaration(d)?; remember(&d.source, &d.ability.key)?;
                let expected = Choice { id: p.choice.id.clone(), kind: "trigger".into(), title: format!("{}：{}", catalog::card("WM029").name, d.ability.label),
                    description: "选择符合数量限制的选项，然后确认。".into(), player_id: player_id(d.actor), options: vec![ChoiceOption { id: "accept".into(), label: "发动触发能力".into(), card: None }], preview_cards: vec![], min: Some(0), max: Some(1), amount: None, allow_decline: Some(true) };
                if !matches!(stage, DeclareChoice::Accept) || p.seat != d.actor || p.choice.id.is_empty() || !same(&p.choice, &expected) { return Err("政要死亡只能由原操控者接受或放弃".into()); }
            },
            ChoiceResolution::Frame { frame: f, .. } if f.source.card.definition == "WM029" && !self.validated_cross_card_frame(f)? => return Err("政要死亡效果没有额外结算续体".into()), _ => {},
        } } Ok(())
    }
}
