//! XG18 shares ordinary character payment/entry; its asset origin is a frozen
//! permission, not an alternative deploy interpreter or client-supplied flag.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub(crate) const KEY: &str = "batch4-xg18-renown-play";
fn same<T: Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PlayedRenown { Printed, Exposure { count: u32 } }
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OmegaAssetPlay {
    pub source: Card,
    pub source_region: usize,
    pub source_region_instance: String,
    pub original_asset: Card,
    pub qualification: PlayedRenown,
    pub turn: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenownPlay {
    pub event_id: String,
    pub turn: u32,
    pub played: SourceSnapshot,
    pub qualification: PlayedRenown,
}
fn qualification_valid(c: &Card, q: &PlayedRenown) -> bool {
    let Some(d) = catalog::catalog().cards.iter().find(|d| d.id == c.definition && d.kind == "character" && catalog::runtime_implemented(d)) else { return false; };
    match q {
        PlayedRenown::Printed => definition_of_renown(&d.id),
        PlayedRenown::Exposure { count } => *count > 0 && matches!(d.id.as_str(), "WM041" | "XG17"),
    }
}
fn definition_of_renown(id: &str) -> bool { crate::rules::definition(id).traits.renown }
fn remember_play(source: &SourceSnapshot, ops: &[Op], seen: &mut BTreeSet<(String, String)>) -> RuleResult<()> {
    if let [Op::XG18RenownPlay { observation: Some(o) }] = ops {
        if !seen.insert((source.card.id.clone(), o.event_id.clone())) {
            return Err("同一欧米伽不能为一次出牌重复保存奖励".into());
        }
    }
    Ok(())
}
pub(crate) fn valid_asset_play(s: &SourceSnapshot, g: &OmegaAssetPlay) -> bool {
    let c = &s.card; let source = &g.source;
    let mut hand_like = g.original_asset.clone(); hand_like.face_down = false; hand_like.exhausted = false;
    source.definition == "XG18" && !source.face_down && !source.id.is_empty() && source.id != c.id
        && source.controller == c.controller && source.owner < 4 && source.controller < 4
        && g.source_region < 5 && !g.source_region_instance.is_empty()
        && g.source_region_instance != source.id && g.source_region_instance != c.id && g.turn > 0
        && c.owner == c.controller && c.owner < 4 && !c.face_down && !c.exhausted && !c.id.is_empty()
        && c == &hand_like && c.damage == 0 && c.wounds == 0 && c.shield == 0 && c.lock_markers == 0
        && c.time_markers == 0 && c.advancement_markers == 0 && c.permanent_combat_markers == 0
        && c.permanent_defense_markers == 0 && c.named_character.is_none()
        && qualification_valid(c, &g.qualification)
}
pub(crate) fn definition() -> Definition {
    let mut d = with_abilities(vec![ability(KEY, "强制：打出声望角色，双方秘社各放置暴露1", Timing::Fast,
        vec![], vec![], vec![Op::XG18RenownPlay { observation: None }], Some(Event::XG18RenownPlayed)),
        crate::mobility::ability()]);
    d.traits.public = true; d.modifiers.push(StaticModifier::XG18RenownAssetPlay); d
}
pub(crate) fn canonical(a: &AbilitySpec) -> Option<AbilitySpec> {
    let mut expected = definition().abilities.into_iter().find(|b| b.key == a.key)?;
    if let [Op::XG18RenownPlay { observation }] = a.ops.as_slice() {
        expected.ops = vec![Op::XG18RenownPlay { observation: observation.clone() }];
    }
    Some(expected)
}
pub(crate) fn validate_action(a: &Action) -> RuleResult<()> {
    let expected = match a.kind.as_str() {
        "deploy" | "conceal" => Action { card_id: a.card_id.clone(), region: a.region, ..Action::new(&a.kind) },
        "reveal" => Action { card_id: a.card_id.clone(), ..Action::new("reveal") },
        _ => return Err("欧米伽小姐只有普通派遣与现身操作".into()),
    };
    if *a != expected { return Err("欧米伽小姐不能附带其他目标、费用或私有载荷".into()); } Ok(())
}
impl Game {
    fn omega_sources(&self) -> impl Iterator<Item = (usize, &Card)> {
        self.regions.iter().enumerate().filter(|(_, r)| !r.vacant).flat_map(|(r, region)| region.cards.iter().map(move |c| (r, c)))
            .filter(|(_, c)| c.definition == "XG18" && !c.face_down && !self.players[c.controller].eliminated)
    }
    fn omega_played_renown(&self, c: &Card) -> Option<PlayedRenown> {
        if !self.has_renown(c) { return None; }
        if definition_of_renown(&c.definition) { return Some(PlayedRenown::Printed); }
        let count = self.society_exposure(c.controller);
        (matches!(c.definition.as_str(), "WM041" | "XG17") && count > 0).then_some(PlayedRenown::Exposure { count })
    }
    pub(crate) fn omega_can_play_assets(&self, actor: usize) -> bool {
        actor < self.players.len() && self.omega_sources().any(|(_, s)| s.controller == actor)
    }
    pub(crate) fn omega_asset_play(&self, actor: usize, c: &Card) -> Option<(Card, PlaySource)> {
        if actor >= self.players.len() || c.owner != actor || c.controller != actor { return None; }
        let (region, source) = self.omega_sources().find(|(_, s)| s.controller == actor)?;
        let mut hand_like = c.clone(); hand_like.face_down = false; hand_like.exhausted = false;
        let qualification = self.omega_played_renown(&hand_like)?;
        let grant = OmegaAssetPlay { source: source.clone(), source_region: region,
            source_region_instance: self.regions[region].card.id.clone(), original_asset: c.clone(), qualification, turn: self.turn };
        Some((hand_like, PlaySource::AssetByXG18(Box::new(grant))))
    }
    pub(crate) fn validate_omega_play_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        let Some(PlaySource::AssetByXG18(g)) = &s.play_source else { return Ok(()); };
        if actor >= self.players.len() || s.card.controller != actor || !valid_asset_play(s, g)
            || g.source.owner >= self.players.len() || g.source_region >= self.regions.len() || g.turn != self.turn
            || self.regions.iter().enumerate().any(|(r, live)| live.card.id == g.source_region_instance && r != g.source_region)
            || self.batch3_physical_cards().any(|c| c.id == s.card.id) {
            return Err("欧米伽资产出牌须保留原资产、原授予者与声望许可".into());
        }
        self.mobility_immutable_identity(&g.source)?; self.mobility_immutable_identity(&s.card)
    }
    pub(crate) fn omega_observe_last_play(&mut self) {
        let Some(f) = self.stack.last().and_then(|item| item.frame.as_ref()) else { return; };
        if f.ability_key != "deploy" { return; }
        let Some(qualification) = self.omega_played_renown(&f.source.card) else { return; };
        let o = RenownPlay { event_id: f.frame_id.clone(), turn: self.turn, played: f.source.clone(), qualification };
        let sources = self.omega_sources().filter(|(_, c)| c.controller == f.actor)
            .map(|(r, c)| self.source_snapshot(c, Some(r))).collect::<Vec<_>>();
        for source in sources {
            let mut a = definition().abilities.remove(0); a.ops = vec![Op::XG18RenownPlay { observation: Some(o.clone()) }];
            self.effects.push_back(Effect::Declare { declaration: Declaration { actor: source.card.controller, source, ability: a } });
        }
    }
    fn omega_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        let c = &s.card;
        if c.definition != "XG18" || c.id.is_empty() || actor >= self.players.len() || c.owner >= self.players.len()
            || c.controller != actor || s.region.is_none_or(|r| r >= self.regions.len())
            || s.source_region_instance.as_ref().is_some_and(|id| id.is_empty() || *id == c.id)
            || s.attachment_host_instance.is_some() || s.death_echo.is_some() || s.observed_death.is_some()
            || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some()
            || s.play_source.is_some() && !s.valid_character_play() { return Err("欧米伽来源、控制者或原地区无效".into()); }
        self.mobility_immutable_identity(c)
    }
    fn omega_observation(&self, actor: usize, ops: &[Op]) -> RuleResult<()> {
        let [Op::XG18RenownPlay { observation: Some(o) }] = ops else { return Err("欧米伽缺少实际打出声望角色观察".into()); };
        let p = &o.played; let c = &p.card;
        if actor >= self.players.len() || o.turn != self.turn || c.controller != actor || c.owner >= self.players.len()
            || c.id.is_empty() || c.face_down || !p.valid_character_play() || !qualification_valid(c, &o.qualification)
            || p.region.is_none_or(|r| r >= self.regions.len()) || p.attachment_host_instance.is_some()
            || p.death_echo.is_some() || p.observed_death.is_some() || p.observed_exposure.is_some() || p.observed_mobility_grant.is_some()
            || o.event_id == c.id || o.event_id.strip_prefix('i').and_then(|v| v.parse::<u64>().ok()).is_none_or(|n| n == 0 || n > self.sequence)
            || self.stack.iter().any(|item| item.id == o.event_id && item.frame.as_ref().is_none_or(|f| f.ability_key != "deploy" || f.actor != actor || !same(&f.source, p))) {
            return Err("欧米伽观察须绑定原出牌玩家、声望条件和实际事务".into());
        }
        self.validate_omega_play_source(actor, p)?; self.mobility_immutable_identity(c)
    }
    pub(crate) fn validate_omega_declaration(&self, d: &Declaration) -> RuleResult<()> {
        self.omega_source(d.actor, &d.source)?;
        if d.source.card.face_down || d.source.play_source.is_some() || d.ability.key != KEY
            || canonical(&d.ability).is_none_or(|a| !same(&a, &d.ability)) { return Err("欧米伽声明必须是完整强制打出触发".into()); }
        if let [Op::XG18RenownPlay { observation: Some(o) }] = d.ability.ops.as_slice() {
            if o.event_id == d.source.card.id || o.played.card.id == d.source.card.id {
                return Err("欧米伽观察来源不能冒充被打出的牌或实际事务".into());
            }
        }
        self.omega_observation(d.actor, &d.ability.ops)
    }
    pub(crate) fn omega_forced_declare(&mut self, d: &Declaration) -> RuleResult<bool> {
        if d.ability.event != Some(Event::XG18RenownPlayed) { return Ok(false); }
        self.validate_omega_declaration(d)?;
        if !self.players[d.actor].eliminated {
            let f = self.make_frame(d.actor, d.source.clone(), &d.ability, vec![], vec![], None);
            self.dispatch_frame(f, d.ability.label.clone(), None, ResponsePolicy::Respondable);
        }
        Ok(true)
    }
    pub(crate) fn omega_reject_optional(&self, d: &Declaration) -> RuleResult<()> {
        if d.ability.event == Some(Event::XG18RenownPlayed) { return Err("欧米伽的打出声望奖励是强制触发".into()); } Ok(())
    }
    pub(crate) fn validate_omega_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        self.omega_source(f.actor, &f.source)?;
        if f.frame_id.is_empty() || f.frame_id == f.source.card.id || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked)
            || !f.targets.is_empty() { return Err("欧米伽帧身份、阶段或目标无效".into()); }
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            if !f.steps.is_empty() || f.chosen_region != f.source.region
                || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if ids.len() <= 6 && ids.iter().all(|id| !id.is_empty() && *id != f.source.card.id) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len())
                || f.ability_key == "deploy" && (f.source.card.face_down || !f.source.valid_character_play())
                || f.ability_key == "reveal" && (!f.source.card.face_down || f.source.play_source.is_some()) { return Err("欧米伽派遣/现身费用或原地区无效".into()); }
            return Ok(());
        }
        let [Step { context, op }] = f.steps.as_slice() else { return Err("欧米伽只执行一次固定奖励".into()); };
        let mut a = definition().abilities.remove(0); a.ops = vec![op.clone()];
        if f.ability_key != KEY || *context != f.actor || f.chosen_region.is_some() || !f.already_paid.is_empty() { return Err("欧米伽奖励不能更改程序或添加费用".into()); }
        self.validate_omega_declaration(&Declaration { actor: f.actor, source: f.source.clone(), ability: a })
    }
    pub(crate) fn omega_resolve(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        if f.ability_key != KEY || f.cursor != 1 || !matches!(f.guard, GuardState::Accepted)
            || !matches!(f.steps.as_slice(), [Step { context, op: Op::XG18RenownPlay { .. } }] if *context == f.actor)
            || !f.targets.is_empty() || !f.already_paid.is_empty() || f.chosen_region.is_some() {
            return Err("欧米伽奖励必须经过原响应帧且只能执行一次固定程序".into());
        }
        self.omega_observation(f.actor, &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>())?;
        for seat in 0..self.players.len() { if !self.players[seat].eliminated { self.add_society_markers(seat, 1, 0); } }
        Ok(())
    }
    fn omega_check_declaration_origin(&self, d: &Declaration) -> RuleResult<()> {
        self.validate_omega_play_source(d.actor, &d.source)?;
        match d.ability.ops.as_slice() {
            [Op::WM041PublicPlay { observation: Some(o) }] => self.validate_omega_play_source(d.actor, &o.played)?,
            [Op::XG14DisasterInfluence { observation: Some(o) }] => self.validate_omega_play_source(d.actor, &o.played)?,
            _ => {},
        }
        if d.source.card.definition == "XG18" && !self.validated_cross_card_declaration(d)? { self.validate_omega_declaration(d)?; }
        Ok(())
    }
    pub(crate) fn validate_omega_state(&self) -> RuleResult<()> {
        let mut frames = BTreeSet::new();
        let mut plays = BTreeSet::new();
        for item in &self.stack { if let Some(f) = &item.frame {
            remember_play(&f.source, &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>(), &mut plays)?;
            self.validate_omega_play_source(f.actor, &f.source)?;
            if let Some(PlaySource::AssetByXG18(g)) = &f.source.play_source {
                if g.original_asset.exhausted && f.already_paid.iter().any(|p| matches!(p, PaidCost::Assets(ids) if ids.contains(&g.original_asset.id))) {
                    return Err("原已横置资产不能又用于支付自身出牌费用".into());
                }
            }
            if f.source.card.definition == "XG18" && !self.validated_cross_card_frame(f)? {
                self.validate_omega_frame(f)?; let play = matches!(f.ability_key.as_str(), "deploy" | "reveal");
                if !frames.insert(f.frame_id.clone()) || item.id != f.frame_id || item.controller != f.actor
                    || item.card.is_some() != play || item.reveal != (f.ability_key == "reveal")
                    || item.target.is_some()
                    || item.deploy_region != if play { f.source.region } else { None }
                    || item.card.as_ref().is_some_and(|c| c.definition != "XG18" || c.face_down || c.owner != f.source.card.owner || c.controller != f.actor) {
                    return Err("欧米伽堆叠外壳与原帧不符".into());
                }
            }
        } }
        for e in &self.effects { match e {
            Effect::Declare { declaration: d } => { self.omega_check_declaration_origin(d)?; remember_play(&d.source, &d.ability.ops, &mut plays)?; },
            Effect::Frame { frame: f } => { self.validate_omega_play_source(f.actor, &f.source)?; if f.source.card.definition == "XG18" && !self.validated_cross_card_frame(f)? { return Err("欧米伽不能绕过响应保存直接帧".into()); } },
            _ => {},
        } }
        if let Some(p) = &self.pending { match &p.resolution {
            ChoiceResolution::Declare { declaration: d, .. } => { self.omega_check_declaration_origin(d)?; self.omega_reject_optional(d)?; },
            ChoiceResolution::Frame { frame: f, .. } => self.validate_omega_play_source(f.actor, &f.source)?,
            _ => {},
        } }
        Ok(())
    }
}
