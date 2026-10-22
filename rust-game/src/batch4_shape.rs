//! XG12 keeps its attachment transfer separate from the shared, once-per-turn
//! mobility event. The latter freezes the recipient's personal secrecy grant.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub(crate) const KEY: &str = "batch4-xg12-transfer";
fn same<T: Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShapeMobility {
    pub turn: u32,
    pub secrecy: u32,
    pub vampire: bool,
    pub attachment: Attachment,
}
pub(crate) fn definition() -> Definition {
    let mut host = target(Zone::Board, EntityKind::Character, Relation::Any, Range::Anywhere);
    host.subtype = Some("吸血鬼".into());
    let mut d = crate::rules::attachment_definition(host.clone());
    let spec = d.attachment.as_mut().unwrap(); spec.host_icons = Icons { combat: 1, ..Icons::default() };
    spec.host_temporary_icons = Some(Icons { combat: 1, ..Icons::default() });
    d.modifiers.push(StaticModifier::XG12BatAndConditionalMobility);
    host.exclude_attachment_host = true;
    let mut a = ability(KEY, "行动2：转移至另一个吸血鬼角色", Timing::Standard, vec![Cost::Assets(2)],
        vec![host], vec![Op::XG12Reattach], None);
    a.activation_only = true; d.abilities.push(a); d
}
pub(crate) fn validate_action(a: &Action) -> RuleResult<()> {
    let expected = match a.kind.as_str() {
        "play" | "activate" => Action { card_id: a.card_id.clone(), ability_id: a.ability_id.clone(), target_id: a.target_id.clone(), ..Action::new(&a.kind) },
        _ => return Err("兽形变化只能结附或发动转移".into()),
    };
    let key = if a.kind == "play" { "attach" } else { KEY };
    if *a != expected || a.ability_id.as_deref().is_some_and(|k| k != key) { return Err("兽形变化不能夹带其他费用、目标或程序".into()); }
    Ok(())
}
pub(crate) fn mobility_ability(proof: ShapeMobility) -> AbilitySpec {
    let mut a = crate::mobility::ability(); a.ops = vec![Op::XG12GrantedMobility { proof: Box::new(proof) }]; a
}
pub(crate) fn mobility_proof(a: &AbilitySpec) -> Option<&ShapeMobility> {
    if let [Op::XG12GrantedMobility { proof }] = a.ops.as_slice() { Some(proof) } else { None }
}
pub(crate) fn canonical_mobility(a: &AbilitySpec) -> bool {
    mobility_proof(a).is_some_and(|p| same(a, &mobility_ability(p.clone())))
}
impl Game {
    pub(crate) fn xg12_mobility_grant(&self, c: &Card) -> Option<ShapeMobility> {
        if c.face_down || self.players[c.controller].society_zone.secrecy == 0
            || !self.board(&c.id).is_some_and(|(_,live)| live.controller == c.controller) || self.attached_character(&c.id) { return None; }
        let a = self.attachments.iter().filter(|a| a.card.definition == "XG12" && !a.card.face_down && a.host_id == c.id && self.attachment_host_valid(a))
            .min_by_key(|a| &a.card.id)?;
        Some(ShapeMobility { turn: self.turn, secrecy: self.players[c.controller].society_zone.secrecy,
            vampire: true, attachment: a.clone() })
    }
    pub(crate) fn xg12_event_ability(&self, actor: usize, source: &SourceSnapshot) -> Option<AbilitySpec> {
        if self.validate_mobility_source(actor, source).is_ok() { return None; }
        self.xg12_mobility_grant(&source.card).map(mobility_ability)
    }
    pub(crate) fn validate_xg12_mobility_source(&self, actor: usize, s: &SourceSnapshot, p: &ShapeMobility) -> RuleResult<()> {
        self.validate_mobility_recipient(actor,s)?;
        let c = &s.card; let a = &p.attachment;
        let d = catalog::card(&c.definition);
        if s.observed_exposure.is_some() || s.observed_mobility_grant.is_some() || p.turn != self.turn || p.secrecy == 0 || !p.vampire
            || a.card.definition != "XG12" || a.card.id.is_empty() || a.card.id == c.id || a.card.face_down
            || a.card.owner >= self.players.len() || a.card.controller >= self.players.len() || a.host_id != c.id
            || (c.definition != "JC030" && !d.subtypes.iter().any(|s| s == "人类" || s == "吸血鬼"))
            || self.batch3_physical_cards().any(|live| live.id == c.id) && self.board(&c.id).is_none()
        { return Err("兽形变化机动须保留个人隐秘、吸血鬼与原结附事件见证".into()); }
        self.mobility_immutable_identity(&a.card)
    }
    pub(crate) fn validate_xg12_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        let s = &f.source; let c = &s.card;
        if c.definition != "XG12" || c.id.is_empty() || c.face_down || c.owner >= self.players.len() || f.actor >= self.players.len() || c.controller != f.actor
            || s.play_source.is_some() || s.observed_death.is_some() || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some()
            || s.region.is_some_and(|r| r >= self.regions.len()) || f.frame_id.is_empty() || f.frame_id == c.id || f.chosen_region.is_some()
            || f.cursor > f.steps.len() || (f.cursor > 0 && !matches!(f.guard,GuardState::Accepted))
            || (f.cursor == 0 && !matches!(f.guard,GuardState::Unchecked) && !(f.ability_key == "attach" && f.steps.is_empty() && matches!(f.guard,GuardState::Accepted|GuardState::Cancelled)))
            || s.source_region_instance.as_ref().is_some_and(|id|id.is_empty()||*id==c.id)
            || s.region.zip(s.source_region_instance.as_ref()).is_some_and(|(r,id)|self.regions.iter().enumerate().any(|(other,live)|live.card.id==*id&&r!=other))
        { return Err("兽形变化来源、帧或冻结地区无效".into()); }
        self.mobility_immutable_identity(c)?;
        if self.batch3_physical_cards().any(|live|live.id==c.id) && (f.ability_key=="attach"||!self.attachments.iter().any(|a|a.card.id==c.id)) { return Err("兽形变化来源不能由私有区同UID冒充".into()); }
        let d=definition();let a=d.abilities.iter().find(|a|a.key==f.ability_key).ok_or("兽形变化未知程序")?;
        let steps=a.ops.iter().map(|op|Step{context:f.actor,op:op.clone()}).collect::<Vec<_>>();
        let paid=matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)]if !ids.iter().any(|id|id.is_empty()||*id==c.id)&&ids.iter().collect::<BTreeSet<_>>().len()==ids.len()&&(if f.ability_key=="attach"{ids.len()<=2}else{ids.len()==2}));
        if !same(&steps,&f.steps)||f.targets.len()!=1||!same(&a.targets[0],&f.targets[0].spec)||!paid { return Err("兽形变化程序、目标或支付不完整".into()); }
        if f.ability_key=="attach" {
            if s.region.is_some()||s.source_region_instance.is_some()||s.attachment_host_instance.is_some(){return Err("兽形变化打出来源不能预结附".into());}
        } else if s.region.is_none()||s.source_region_instance.is_none()||s.attachment_host_instance.as_ref().is_none_or(|id|id.is_empty()||*id==c.id)||s.attachment_host_instance.as_ref()==Some(&f.targets[0].id) {
            return Err("兽形变化转移必须保留原宿主并选择另一个吸血鬼".into());
        }
        let t=&f.targets[0];
        if t.id.is_empty()||t.region_instance.is_some()||t.public.instance_id!=t.id||t.public.kind!="character"||!t.public.valid||t.public.status!="valid"||t.public.invalid_reason.is_some()
            ||t.public.region.is_none_or(|r|r>=self.regions.len())||t.public.owner.is_none()||t.public.controller.is_none(){return Err("兽形变化吸血鬼目标摘要无效".into());}
        Ok(())
    }
    pub(crate) fn validate_xg12_state(&self) -> RuleResult<()> {
        let mut frames=BTreeSet::new();
        for item in &self.stack { if let Some(f)=&item.frame { if f.source.card.definition=="XG12" {
            self.validate_xg12_frame(f)?;
            if !frames.insert(&f.frame_id)||item.id!=f.frame_id||item.controller!=f.actor||f.cursor!=0||!matches!(f.guard,GuardState::Unchecked)
                ||item.card.is_some()!=(f.ability_key=="attach")||item.deploy_region.is_some()||item.reveal||item.target.as_ref()!=Some(&f.targets[0].id)
                ||item.card.as_ref().is_some_and(|c|c.definition!="XG12"||c.face_down||c.owner!=f.source.card.owner||c.controller!=f.actor){return Err("兽形变化响应堆叠外壳被改写".into());}
        } } }
        if self.effects.iter().any(|e|matches!(e,Effect::Frame{frame}if frame.source.card.definition=="XG12"))
            || self.pending.as_ref().is_some_and(|p|matches!(&p.resolution,ChoiceResolution::Frame{frame,..}if frame.source.card.definition=="XG12")){return Err("兽形变化转移没有直接续体或帧内选择".into());}
        Ok(())
    }
}
