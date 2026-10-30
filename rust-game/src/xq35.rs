//! XQ35: the printed graveyard action, with an actual sealed payment and a
//! respondable frame. Public deck admission remains closed pending review.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub(crate) const ID: &str = "XQ35";
pub(crate) const KEY: &str = "xq35-grave-return";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Payment { pub turn: u32, pub host: String, pub card: Card }
fn same<T: Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap() }
pub(crate) fn definition() -> Definition {
    let mut a=ability(KEY,"墓地行动：支付1并封印另一张墓地角色，横置回到地区一",Timing::Standard,
        vec![Cost::Assets(1),Cost::XQ35SealCharacter],vec![],vec![Op::XQ35GraveReturn],None);
    a.activation_only=true; a.per_turn_limit=Some(1); with_abilities(vec![a])
}
fn owns(ops:&[Op])->bool { ops.iter().any(|o|match o {
 Op::XQ35GraveReturn=>true,
 Op::ForEachLivingPlayer(body)|Op::ForEachLivingPlayerFromActor(body)=>owns(body),
 Op::IfTargetExhausted{exhausted,ready,..}=>owns(std::slice::from_ref(exhausted))||owns(std::slice::from_ref(ready)),
 _=>false
}) }
pub(crate) fn related(a: &AbilitySpec)->bool {
    a.key.starts_with("xq35-") || a.costs.iter().any(|c|matches!(c,Cost::XQ35SealCharacter))
        || owns(&a.ops) || a.modes.iter().any(|m|owns(&m.ops))
}
pub(crate) fn validate_ability(id:&str,a:&AbilitySpec)->RuleResult<()> {
    if id!=ID || !same(a,&definition().abilities[0]) {return Err("白骨仆人只能使用完整原墓地行动".into());} Ok(())
}
fn related_frame(f:&ResolutionFrame)->bool {
    f.ability_key.starts_with("xq35-") || f.already_paid.iter().any(|p|matches!(p,PaidCost::XQ35Sealed(_)))
        || f.steps.iter().any(|s|owns(std::slice::from_ref(&s.op)))
}
impl Game {
    pub(crate) fn xq35_begin(&mut self,actor:usize,a:&Action)->RuleResult<()> {
        if *a!=(Action{card_id:a.card_id.clone(),ability_id:Some(KEY.into()),cost_selected:a.cost_selected.clone(),..Action::new("activate")}) {
            return Err("白骨仆人墓地行动只接受来源和一张封印费用".into());
        }
        let c=self.bq095_self_grave_sources(actor).into_iter().find(|c|Some(c.id.as_str())==a.card_id.as_deref() && c.definition==ID)
            .ok_or("白骨仆人必须在本人墓地")?;
        let source=self.source_snapshot(c,None); let spec=definition().abilities.remove(0);
        self.check_timing(actor,&spec)?;
        let paid=self.pay_ability_costs(actor,&source,&spec,a)?;
        let frame=self.make_frame(actor,source,&spec,vec![],paid,None);
        self.dispatch_frame(frame,spec.label,None,spec.response_policy); Ok(())
    }
    pub(crate) fn xq35_pay(&mut self,actor:usize,source:&SourceSnapshot,a:&Action)->RuleResult<PaidCost> {
        let selected=a.cost_selected.as_deref().ok_or("请选择另一张墓地角色作为费用")?;
        if source.card.definition!=ID || source.card.owner!=actor || source.card.controller!=actor || selected.len()!=1
            || selected[0]==source.card.id || !self.bq095_reference_cards(&source.card).iter().any(|c|same(c,&source.card)) {
            return Err("白骨仆人须封印本人墓地另一张真实角色".into());
        }
        let host=format!("society:p{actor}");
        if !self.seal_host_valid(&host) {return Err("本方秘社封印载体无效".into());}
        if !self.bq095_reference_cards(&source.card).iter().any(|c|c.id==selected[0] && c.owner==actor && catalog::card(&c.definition).kind=="character") {return Err("封印费用不是本人墓地角色".into());}
        let card=self.bq095_take_reference(&source.card,&selected[0])?;
        self.seal_card_on_valid_host(card.clone(),actor,&host);
        Ok(PaidCost::XQ35Sealed(Payment{turn:self.turn,host,card}))
    }
    pub(crate) fn xq35_actions(&self,actor:usize)->Vec<(Action,String)> {
        let mut out=vec![];
        for source in self.bq095_self_grave_sources(actor).into_iter().filter(|c|c.definition==ID) {
            for c in self.bq095_reference_cards(source).iter().filter(|c|c.id!=source.id && catalog::card(&c.definition).kind=="character") {
                out.push((Action{card_id:Some(source.id.clone()),ability_id:Some(KEY.into()),cost_selected:Some(vec![c.id.clone()]),..Action::new("activate")},
                    format!("白骨仆人：支付1，封印{}，横置回到地区一",catalog::card(&c.definition).name)));
            }
        } out
    }
    pub(crate) fn validate_xq35_frame(&self,f:&ResolutionFrame)->RuleResult<()> {
        if f.source.card.definition==ID && matches!(f.ability_key.as_str(),"deploy"|"reveal") {return Err("白骨仆人不能恢复正面打出或付费现身帧".into());}
        if !related_frame(f) {return Ok(());}
        let c=&f.source.card;
        let [PaidCost::Assets(assets),PaidCost::XQ35Sealed(p)]=f.already_paid.as_slice() else {return Err("白骨仆人缺少真实资产和封印费用收据".into());};
        if f.actor>=self.players.len() || c.definition!=ID || c.owner!=f.actor || c.controller!=f.actor || c.face_down
            || !self.mystic_instance_valid(&c.id) || !self.mystic_instance_valid(&f.frame_id) || c.id==f.frame_id
            || f.source.region.is_some() || f.source.source_region_instance.is_some() || f.source.play_source.is_some()
            || f.source.death_echo.is_some() || f.source.observed_death.is_some() || f.source.observed_exposure.is_some()
            || f.source.observed_mobility_grant.is_some() || f.source.attachment_host_instance.is_some()
            || f.ability_key!=KEY || !f.targets.is_empty() || f.chosen_region.is_some()
            || f.cursor>1 || (f.cursor==0 && !matches!(f.guard,GuardState::Unchecked))
            || (f.cursor==1 && !matches!(f.guard,GuardState::Accepted))
            || !same(&f.steps,&vec![Step{context:f.actor,op:Op::XQ35GraveReturn}])
            || f.money_units(0)!=1 || assets.iter().any(|id|!self.mystic_instance_valid(id) || *id==c.id)
            || p.turn!=self.turn || p.host!=format!("society:p{}",f.actor)
            || p.card.id==c.id || assets.contains(&p.card.id) || !self.mystic_instance_valid(&p.card.id)
            || p.card.owner!=f.actor || p.card.controller!=f.actor || p.card.face_down
            || catalog::card(&p.card.definition).kind!="character"
            || self.batch3_physical_cards().any(|now|now.id==p.card.id)
            || !self.turn_ability_usage.iter().any(|u|u.source_instance==c.id && u.ability_key==KEY && u.turn==self.turn && u.uses==1) {
            return Err("白骨仆人来源、程序、支付或限次记录被改写".into());
        }
        self.mobility_immutable_identity(c)?; self.mobility_immutable_identity(&p.card)
    }
    pub(crate) fn xq35_return(&mut self,f:&ResolutionFrame)->RuleResult<()> {
        self.validate_xq35_frame(f)?;
        // A departed source or unavailable first region does not refund costs.
        if !self.region_live(0) {return Ok(());}
        if !self.bq095_reference_cards(&f.source.card).iter().any(|c|c.id==f.source.card.id && same(c,&f.source.card)) {return Ok(());}
        let c=self.bq095_take_reference(&f.source.card,&f.source.card.id)?;
        let mut c=self.reset_zone_card(c); c.exhausted=true;
        let id=c.id.clone(); self.regions[0].cards.push(c);
        self.enter_triggers(f.actor,ID,&id,false); Ok(())
    }
    pub(crate) fn validate_xq35_state(&self)->RuleResult<()> {
        let mut ids=BTreeSet::new();
        for item in &self.stack {
            if item.card.as_ref().is_some_and(|c| c.definition == ID) {
                return Err("白骨仆人不能作为正面打出或付费现身事务恢复".into());
            }
            if let Some(f)=&item.frame { if related_frame(f) || f.source.card.definition==ID {
                self.validate_xq35_frame(f)?;
                if !ids.insert(&f.frame_id) || item.id!=f.frame_id || item.controller!=f.actor || item.card.is_some()
                    || item.reveal || item.deploy_region.is_some() || item.target.is_some() || f.cursor!=0 {
                    return Err("白骨仆人响应外壳或重复续体无效".into());
                }
            }}
        }
        for e in &self.effects {match e {
            Effect::Frame{frame} if related_frame(frame)=>{self.validate_xq35_frame(frame)?;if !ids.insert(&frame.frame_id){return Err("白骨仆人续体重复".into());}},
            Effect::DeathCaller{caller}=>self.validate_xq35_frame(&caller.frame)?,
            _=>()
        }}
        if let Some(p)=&self.pending {if let ChoiceResolution::Frame{frame,..}=&p.resolution {if related_frame(frame){return Err("白骨仆人结算不产生额外待选".into());}}}
        Ok(())
    }
}
