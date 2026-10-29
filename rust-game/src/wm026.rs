//! Closed WM026 transformation, using the ordinary declaration/response frame.
use crate::{engine::RuleResult,model::*,rules::*};
use std::collections::BTreeSet;
pub(crate) const ID:&str="WM026";
pub(crate) const FORM:&str="ZHWM026";
pub(crate) const KEY:&str="wm026-transform";
fn same<T:serde::Serialize>(a:&T,b:&T)->bool{serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap()}
pub(crate) fn definition(mist:bool)->Definition{let mut d=with_abilities(vec![if mist{ability("wm026-prepare","准备步骤：古妮薇尔转回",Timing::Fast,vec![],vec![],vec![Op::WM026Prepare{event_id:None}],Some(Event::Prepare))}else{ability(KEY,"古妮薇尔死亡转变（保留伤害及标志）",Timing::Fast,vec![],vec![],vec![Op::WM026Transform],Some(Event::Death))}]);d.traits.spirit=mist;d}
pub(crate) fn related(a:&AbilitySpec)->bool{a.key==KEY||a.ops.iter().any(|o|matches!(o,Op::WM026Transform))}
pub(crate) fn validate_ability(id:&str,a:&AbilitySpec)->RuleResult<()>{if !matches!(id,ID|FORM)||!same(a,&definition(id==FORM).abilities[0]){return Err("古妮薇尔转变程序或形态不符".into());}Ok(())}
impl Game{
 pub(crate) fn wm026_waiting(&self,id:&str)->bool{self.wm026_pending.iter().any(|s|s.card.id==id&&s.card.definition==ID)}
 pub(crate) fn wm026_capture(&mut self,s:SourceSnapshot){if self.wm026_pending.iter().any(|p|p.card.id==s.card.id){return;}self.wm026_pending.push(s.clone());self.effects.push_back(Effect::Declare{declaration:Declaration{actor:s.card.controller,ability:definition(s.card.definition==FORM).abilities.remove(0),source:s}});}
 pub(crate) fn wm026_prepare_resolve(&mut self,f:&ResolutionFrame)->RuleResult<()>{if let Some(c)=self.board_mut(&f.source.card.id){if c.definition==FORM&&!c.face_down{c.definition=ID.into();}}self.settle_deaths();Ok(())}
 fn wm026_source(&self,actor:usize,s:&SourceSnapshot)->RuleResult<()>{if actor>=self.players.len()||s.card.controller!=actor||s.card.face_down||s.card.definition!=ID||s.region.is_none_or(|r|r>=self.regions.len())||s.source_region_instance.as_deref().is_none_or(|id|!self.mystic_instance_valid(id))||s.play_source.is_some()||s.attachment_host_instance.is_some()||s.death_echo.is_some()||s.observed_death.is_some()||s.observed_exposure.is_some()||s.observed_mobility_grant.is_some()||!self.wm026_pending.iter().any(|p|same(p,s)){return Err("古妮薇尔缺实际转变事件".into());}self.mobility_immutable_identity(&s.card)}
 pub(crate) fn wm026_forced(&mut self,d:&Declaration)->RuleResult<bool>{if !related(&d.ability){return Ok(false)}validate_ability(&d.source.card.definition,&d.ability)?;self.wm026_source(d.actor,&d.source)?;if self.players[d.actor].eliminated{self.wm026_pending.retain(|s|s.card.id!=d.source.card.id);return Ok(true)}let f=self.make_frame(d.actor,d.source.clone(),&d.ability,vec![],vec![],None);self.dispatch_frame(f,d.ability.label.clone(),None,ResponsePolicy::Respondable);Ok(true)}
 pub(crate) fn validate_wm026_frame(&self,f:&ResolutionFrame)->RuleResult<()>{if f.ability_key!=KEY&&!f.steps.iter().any(|s|matches!(s.op,Op::WM026Transform)){return Ok(())}self.wm026_source(f.actor,&f.source)?;if f.ability_key!=KEY||!same(&f.steps,&vec![Step{context:f.actor,op:Op::WM026Transform}])||f.cursor>1||f.cursor==0&&!matches!(f.guard,GuardState::Unchecked)||f.cursor==1&&!matches!(f.guard,GuardState::Accepted)||!f.targets.is_empty()||!f.already_paid.is_empty()||!f.resource_payments.is_empty()||f.chosen_region.is_some()||!self.mystic_instance_number(&f.source.card.id).zip(self.mystic_instance_number(&f.frame_id)).is_some_and(|(a,b)|a<b){return Err("古妮薇尔原转变帧损坏".into());}Ok(())}
 pub(crate) fn wm026_transform(&mut self,f:&ResolutionFrame)->RuleResult<()>{self.validate_wm026_frame(f)?;if f.cursor!=1{return Err("古妮薇尔转变阶段无效".into())}let s=&f.source;self.wm026_pending.retain(|p|p.card.id!=s.card.id);if self.players[f.actor].eliminated{return Ok(())}if let Some(c)=self.board_mut(&s.card.id){if !c.face_down&&c.definition==s.card.definition{c.definition=if c.definition==ID{FORM}else{ID}.into();}}self.settle_deaths();Ok(())}
 pub(crate) fn validate_wm026_state(&self)->RuleResult<()>{let mut pending=BTreeSet::new();for s in &self.wm026_pending{self.wm026_source(s.card.controller,s)?;if !pending.insert(s.card.id.clone()){return Err("古妮薇尔重复转变事件".into());}}
 let mut active=BTreeSet::new();for e in &self.effects{match e{Effect::Declare{declaration:d}if related(&d.ability)=>{validate_ability(&d.source.card.definition,&d.ability)?;self.wm026_source(d.actor,&d.source)?;if !active.insert(d.source.card.id.clone()){return Err("古妮薇尔重复声明".into())}},Effect::Frame{frame:f}if f.ability_key==KEY=>{self.validate_wm026_frame(f)?;if !active.insert(f.source.card.id.clone()){return Err("古妮薇尔重复续体".into())}},_=>()}}
 for i in &self.stack{if let Some(f)=&i.frame{self.validate_wm026_frame(f)?;if f.ability_key==KEY{if !active.insert(f.source.card.id.clone())||i.id!=f.frame_id||i.controller!=f.actor||i.card.is_some()||i.reveal||i.deploy_region.is_some()||i.target.is_some()||f.cursor!=0{return Err("古妮薇尔响应载体损坏".into());}}}}
 if active!=pending{return Err("古妮薇尔转变事件缺失或重复活动载体".into());}for c in self.batch3_physical_cards(){if c.definition==FORM&&(c.face_down||self.board(&c.id).is_none()){return Err("古妮薇尔雾形态不得进入实体牌区".into());}}Ok(())}
}

pub(crate) fn printed_id(id:&str)->&str{if id==FORM{ID}else{id}}

pub(crate) fn validate_definition(id:&str,d:&Definition)->RuleResult<()>{if matches!(id,ID|FORM)&&!same(d,&definition(id==FORM)){return Err("古妮薇尔形态完整规则损坏".into());}Ok(())}
