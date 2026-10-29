//! Closed BQ045: paid tentacle exhaustion, and the separately invoked core program.
use crate::{catalog,engine::RuleResult,model::*,rules::*};
use serde::{Serialize,Deserialize};
use std::collections::BTreeSet;
pub(crate) const ID:&str="BQ045";
pub(crate) const HUNT:&str="bq045-locked-hunt";
pub(crate) const SHIELD:&str="bq045-entry-shield";
fn same<T:Serialize>(a:&T,b:&T)->bool{serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap()}
fn target_slot(locked:bool)->TargetSlotSpec{let mut t=target(Zone::Board,EntityKind::CharacterOrHidden,Relation::Any,Range::Anywhere);if locked{t.predicate=Some(TargetPredicate::BQ045Locked);}t}
pub(crate) fn definition()->Definition{
 let shield=ability(SHIELD,"护盾1：进场强制触发放置一个护盾",Timing::Fast,vec![],vec![],vec![Op::BQ045Shield],Some(Event::Enter));
 let mut hunt=ability(HUNT,"支付2并横置一个本方触手：消灭锁定角色，获得1名誉",Timing::Standard,vec![Cost::Assets(2),Cost::BQ045ExhaustTentacle],vec![target_slot(true)],vec![Op::Destroy(EntityRef::Target(0)),Op::BQ045Honor],None);hunt.activation_only=true;
 with_abilities(vec![shield,hunt])
}
pub(crate) fn related(a:&AbilitySpec)->bool{a.key.starts_with("bq045-")||a.costs.iter().any(|c|matches!(c,Cost::BQ045ExhaustTentacle))||a.targets.iter().any(|t|t.predicate==Some(TargetPredicate::BQ045Locked))||a.ops.iter().any(|o|matches!(o,Op::BQ045Shield|Op::BQ045Honor))}
pub(crate) fn validate_ability(id:&str,a:&AbilitySpec)->RuleResult<()>{if id!=ID||!definition().abilities.iter().any(|b|same(a,b)){return Err("赫卡忒须保留护盾1及原横置触手费用、锁定目标和名誉程序".into());}Ok(())}
pub(crate) fn validate_definition(id:&str,d:&Definition)->RuleResult<()>{if (id==ID||d.abilities.iter().any(related))&&(id!=ID||!same(d,&definition())){return Err("赫卡忒完整定义不可改写或移植".into());}Ok(())}
pub(crate) fn validate_action(id:&str,a:&Action)->RuleResult<()>{if id!=ID{return Ok(());}
 let expected=if matches!(a.kind.as_str(),"deploy"|"conceal"|"reveal"|"asset"){Action{card_id:a.card_id.clone(),region:if matches!(a.kind.as_str(),"deploy"|"conceal"){a.region}else{None},option:if a.kind=="deploy"&&a.option.as_deref()==Some("honor-rest"){a.option.clone()}else{None},..Action::new(&a.kind)}}else{Action{card_id:a.card_id.clone(),ability_id:Some(HUNT.into()),target_id:a.target_id.clone(),cost_selected:a.cost_selected.clone(),..Action::new("activate")}};
 if *a!=expected{return Err("赫卡忒只允许普通区域动作或原付费行动，核心不能作为免费行动".into());}Ok(())}
fn frame_related(f:&ResolutionFrame)->bool{f.ability_key.starts_with("bq045-")||f.already_paid.iter().any(|p|matches!(p,PaidCost::BQ045Exhausted(_)))||f.steps.iter().any(|s|matches!(s.op,Op::BQ045Shield|Op::BQ045Honor))}
impl Game{
 pub(crate) fn bq045_cost_candidates(&self,actor:usize)->Vec<String>{self.regions.iter().filter(|r|!r.vacant).flat_map(|r|&r.cards).filter(|c|c.controller==actor&&!c.exhausted&&crate::tentacle::character(c)).map(|c|c.id.clone()).collect()}
 pub(crate) fn bq045_pay(&mut self,actor:usize,s:&SourceSnapshot,spec:&AbilitySpec,a:&Action)->RuleResult<PaidCost>{
  validate_ability(&s.card.definition,spec)?;validate_action(&s.card.definition,a)?;
  if spec.key!=HUNT||s.card.controller!=actor||s.card.face_down||!self.board(&s.card.id).is_some_and(|(_,c)|c==&s.card){return Err("赫卡忒行动费用须来自实际本方明置来源".into());}
  let ids=a.cost_selected.as_ref().filter(|v|v.len()==1).ok_or("须横置恰好一个本方触手")?;
  if !self.bq045_cost_candidates(actor).contains(&ids[0]){return Err("触手费用须未横置、明置且由本人操控".into());}
  let(r,c)=self.board(&ids[0]).unwrap();let mut paid=self.source_snapshot(c,Some(r));paid.source_region_instance=Some(self.regions[r].card.id.clone());
  self.board_mut(&ids[0]).unwrap().exhausted=true;Ok(PaidCost::BQ045Exhausted(Box::new(paid)))
 }
 fn bq045_source(&self,actor:usize,s:&SourceSnapshot)->RuleResult<()>{let c=&s.card;
  if actor>=self.players.len()||c.definition!=ID||c.controller!=actor||c.owner>=self.players.len()||c.face_down||!self.mystic_instance_valid(&c.id)||s.region.is_none_or(|r|r>=self.regions.len())||s.source_region_instance.as_deref().is_none_or(|id|!self.mystic_instance_valid(id))||s.play_source.is_some()||s.attachment_host_instance.is_some()||s.death_echo.is_some()||s.observed_death.is_some()||s.observed_exposure.is_some()||s.observed_mobility_grant.is_some(){return Err("赫卡忒来源身份或原地区损坏".into());}self.mobility_immutable_identity(c)
 }
 pub(crate) fn validate_bq045_frame(&self,f:&ResolutionFrame)->RuleResult<()>{if !frame_related(f){return Ok(());}self.bq045_source(f.actor,&f.source)?;self.validate_resource_frame(f)?;
  let a=definition().abilities.into_iter().find(|a|a.key==f.ability_key).ok_or("赫卡忒未知帧")?;
  if !self.mystic_instance_number(&f.source.card.id).zip(self.mystic_instance_number(&f.frame_id)).is_some_and(|(a,b)|a<b)||f.cursor>a.ops.len()||f.chosen_region.is_some()||!same(&f.steps,&a.ops.iter().map(|o|Step{context:f.actor,op:o.clone()}).collect())||f.cursor==0&&!matches!(f.guard,GuardState::Unchecked)||f.cursor>0&&!matches!(f.guard,GuardState::Accepted){return Err("赫卡忒帧程序、游标或来源次序损坏".into());}
  if f.ability_key==SHIELD{if !f.targets.is_empty()||!f.already_paid.is_empty()||!f.resource_payments.is_empty(){return Err("护盾进场不是付费或指定目标行动".into());}}else{
   let [t]=f.targets.as_slice()else{return Err("赫卡忒须保留唯一原锁定目标".into());};
   if !same(&t.spec,&target_slot(true))||!self.mystic_instance_valid(&t.id)||t.public.instance_id!=t.id||t.region_instance.is_some()||!t.public.valid||t.public.status!="valid"||t.public.invalid_reason.is_some(){return Err("赫卡忒目标合同损坏".into());}
   let [PaidCost::Assets(ids),PaidCost::BQ045Exhausted(p)]=f.already_paid.as_slice()else{return Err("赫卡忒缺原两项支付回执".into());};let c=&p.card;
   if f.money_units(0)!=2||ids.iter().collect::<BTreeSet<_>>().len()!=ids.len()||ids.iter().any(|id|!self.mystic_instance_valid(id)||id==&c.id)||c.controller!=f.actor||c.owner>=self.players.len()||c.face_down||c.exhausted||!crate::tentacle::character(c)||p.region.is_none_or(|r|r>=self.regions.len())||p.source_region_instance.as_deref().is_none_or(|id|!self.mystic_instance_valid(id))||!self.mystic_instance_number(&c.id).zip(self.mystic_instance_number(&f.frame_id)).is_some_and(|(a,b)|a<b)||p.play_source.is_some()||p.attachment_host_instance.is_some()||p.death_echo.is_some()||p.observed_death.is_some()||p.observed_exposure.is_some()||p.observed_mobility_grant.is_some(){return Err("赫卡忒支付数额或原明置未横置触手损坏".into());}self.mobility_immutable_identity(c)?;
  }Ok(())
 }
 pub(crate) fn bq045_forced(&mut self,d:&Declaration)->RuleResult<bool>{if d.source.card.definition!=ID&&!related(&d.ability){return Ok(false);}validate_ability(&d.source.card.definition,&d.ability)?;self.bq045_source(d.actor,&d.source)?;if d.ability.key!=SHIELD{return Err("赫卡忒付费行动不能伪装进场触发".into());}
  if !self.players[d.actor].eliminated{let f=self.make_frame(d.actor,d.source.clone(),&d.ability,vec![],vec![],None);self.dispatch_frame(f,d.ability.label.clone(),None,ResponsePolicy::Respondable);}Ok(true)
 }
 pub(crate) fn bq045_step(&mut self,f:&ResolutionFrame)->RuleResult<()>{self.validate_bq045_frame(f)?;match(f.ability_key.as_str(),f.cursor){(SHIELD,1)=>{if let Some(c)=self.board_mut(&f.source.card.id).filter(|c|!c.face_down){let placement_before = c.shield; let placement_id = c.id.clone(); c.shield=c.shield.checked_add(1).ok_or("护盾溢出")?; self.record_board_marker_write(&placement_id, crate::batch4_markers::MarkerKind::Shield, placement_before);}},(HUNT,2)=>self.players[f.actor].honor=self.players[f.actor].honor.checked_add(1).ok_or("名誉溢出")?,_=>return Err("赫卡忒固定程序阶段不符".into())}Ok(())}
 pub(crate) fn validate_bq045_state(&self)->RuleResult<()>{
  for i in &self.stack{if let Some(f)=&i.frame{self.validate_bq045_frame(f)?;if frame_related(f)&&(i.card.is_some()||i.id!=f.frame_id||i.controller!=f.actor||i.reveal||i.deploy_region.is_some()||i.target.as_deref()!=f.targets.first().map(|t|t.id.as_str())||f.cursor!=0){return Err("赫卡忒能力响应外壳损坏".into());}}}
  for e in &self.effects{match e{Effect::Frame{frame}=>self.validate_bq045_frame(frame)?,Effect::DeathCaller{caller}=>self.validate_bq045_frame(&caller.frame)?,Effect::Declare{declaration:d}if d.source.card.definition==ID||related(&d.ability)=>{validate_ability(&d.source.card.definition,&d.ability)?;self.bq045_source(d.actor,&d.source)?;if d.ability.key!=SHIELD{return Err("赫卡忒付费行动不可变成触发".into());}},_=>{}}}
  if let Some(p)=&self.pending{match &p.resolution{ChoiceResolution::Frame{frame,choice:FrameChoice::BQ045Core(c)}=>self.validate_bq045_core_pending(p,frame,c)?,ChoiceResolution::Frame{frame,..}=>{self.validate_bq045_frame(frame)?;if frame_related(frame){return Err("赫卡忒普通行动没有额外选择".into());}},ChoiceResolution::Declare{declaration:d,..}if d.source.card.definition==ID||related(&d.ability)=>return Err("护盾进场强制触发不能改为可选".into()),_=>{}}}Ok(())
 }
}

#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreChoice{pub core:Box<SourceSnapshot>,pub phase:CorePhase}
#[derive(Clone,Debug,Serialize,Deserialize)]
pub enum CorePhase{Target,Opponent{locked:Box<Card>},Repress{locked:Box<Card>,seat:usize,regions:Vec<(usize,String,u32)>}}
impl Game{
 fn bq045_parent(&self,f:&ResolutionFrame)->RuleResult<()>{
  match f.source.card.definition.as_str(){"BQ048"=>{self.validate_bq048_frame(f)?;if f.cursor!=1||!matches!(f.steps[0].op,Op::BQ048InvokeCore){return Err("赫卡忒核心须由位格压迫原发动步骤调用".into());}},"BQ063"=>{self.validate_bq063_frame(f)?;if !matches!(f.cursor,1|2)||!matches!(f.steps[f.cursor-1].op,Op::BQ063InvokeCore){return Err("赫卡忒核心须保留神话威严发动序号".into());}},_=>return Err("未授权赫卡忒核心发动来源".into())}Ok(())
 }
 fn bq045_opponents(&self,actor:usize)->Vec<ChoiceOption>{self.players.iter().filter(|p|!p.eliminated&&self.team(p.seat)!=self.team(actor)).map(|p|ChoiceOption{id:player_id(p.seat),label:p.name.clone(),card:None}).collect()}
 fn bq045_core_options(&self,f:&ResolutionFrame,c:&CoreChoice)->RuleResult<(usize,String,String,Option<u32>,Vec<ChoiceOption>)>{
  self.bq045_parent(f)?;let actual=self.selected_core_source(f.actor)?;let s=&c.core;
  if s.card.definition!=ID||s.card.owner!=f.actor||s.card.controller!=f.actor||s.card.face_down||s.card.id!=actual.card.id||s.region!=actual.region||s.source_region_instance!=actual.source_region_instance||s.play_source.is_some()||s.attachment_host_instance.is_some()||s.death_echo.is_some()||s.observed_death.is_some()||s.observed_exposure.is_some()||s.observed_mobility_grant.is_some(){return Err("赫卡忒核心待选失去原核心身份或位置".into());}self.mobility_immutable_identity(&s.card)?;
  let check_locked=|card:&Card|->RuleResult<()>{if card.lock_markers==0||!self.board(&card.id).is_some_and(|(_,v)|same(v,card)){return Err("核心遏制须保留实际放锁后的原对象".into());}Ok(())};
  Ok(match &c.phase{CorePhase::Target=>(f.actor,"bq045-core-target".into(),"赫卡忒核心：选择角色或暗藏者放置锁定".into(),None,self.binding_options(f.actor,s,&target_slot(false))),CorePhase::Opponent{locked}=>{check_locked(locked)?;(f.actor,"bq045-core-opponent".into(),"赫卡忒核心：选择遏制的敌方玩家".into(),None,self.bq045_opponents(f.actor))},CorePhase::Repress{locked,seat,regions}=>{check_locked(locked)?;if *seat>=self.players.len()||self.team(*seat)==self.team(f.actor)||regions.is_empty()||!same(regions,&self.batch20_repress_regions(*seat)){return Err("核心遏制须保留敌方实际势力地区".into());}(*seat,"bq045-core-repress".into(),"赫卡忒核心：移除1个本方势力".into(),Some(1),self.batch20_repress_options(regions,1))}})
 }
 fn bq045_core_offer(&mut self,f:ResolutionFrame,c:CoreChoice)->RuleResult<()>{let(seat,kind,title,amount,options)=self.bq045_core_options(&f,&c)?;
  if options.is_empty(){self.effects.push_front(Effect::Frame{frame:Box::new(f)});}else{self.choice(seat,&kind,title,options,1,1,amount,ChoiceResolution::Frame{frame:Box::new(f),choice:FrameChoice::BQ045Core(c)});}Ok(())
 }
 pub(crate) fn bq045_core_start(&mut self,f:ResolutionFrame)->RuleResult<()>{self.bq045_parent(&f)?;let core=Box::new(self.selected_core_source(f.actor)?);self.bq045_core_offer(f,CoreChoice{core,phase:CorePhase::Target})}
 fn validate_bq045_core_pending(&self,p:&Pending,f:&ResolutionFrame,c:&CoreChoice)->RuleResult<()>{let(seat,kind,title,amount,options)=self.bq045_core_options(f,c)?;
  if options.is_empty()||p.seat!=seat||p.choice.player_id!=player_id(seat)||p.choice.kind!=kind||p.choice.title!=title||p.choice.min!=Some(1)||p.choice.max!=Some(1)||p.choice.allow_decline!=Some(false)||p.choice.amount!=amount||!p.choice.preview_cards.is_empty()||!self.mystic_instance_valid(&p.choice.id)||!same(&p.choice.options,&options)||self.stack.iter().any(|s|s.id==f.frame_id)||self.effects.iter().any(|e|matches!(e,Effect::Frame{frame} if frame.frame_id==f.frame_id)){return Err("赫卡忒核心待选合同或排他续体损坏".into());}Ok(())
 }
 pub(crate) fn bq045_core_choose(&mut self,f:ResolutionFrame,mut c:CoreChoice,chooser:usize,a:&Action,selected:&[String])->RuleResult<()>{let(seat,_,_,_,options)=self.bq045_core_options(&f,&c)?;
  if chooser!=seat||*a!=(Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")})||selected.len()!=1||!options.iter().any(|o|o.id==selected[0]){return Err("赫卡忒核心选择必须来自原待选".into());}
  match c.phase{CorePhase::Target=>{if self.shield_stops(f.actor,&selected[0]){self.effects.push_front(Effect::Frame{frame:Box::new(f)});return Ok(());}let card=self.board_mut(&selected[0]).ok_or("核心原目标已离场")?;let placement_before = card.lock_markers; let placement_id = card.id.clone(); card.lock_markers=card.lock_markers.checked_add(1).ok_or("锁定溢出")?; let locked=Box::new(card.clone());self.record_board_marker_write(&placement_id, crate::batch4_markers::MarkerKind::Lock, placement_before);c.phase=CorePhase::Opponent{locked};self.bq045_core_offer(f,c)},CorePhase::Opponent{locked}=>{let seat=selected[0].strip_prefix('p').and_then(|s|s.parse().ok()).ok_or("敌方席位无效")?;let regions=self.batch20_repress_regions(seat);if regions.is_empty(){self.effects.push_front(Effect::Frame{frame:Box::new(f)});Ok(())}else{c.phase=CorePhase::Repress{locked,seat,regions};self.bq045_core_offer(f,c)}},CorePhase::Repress{seat,regions,..}=>{self.wm040_repress_complete(seat,&regions,selected)?;self.effects.push_front(Effect::Frame{frame:Box::new(f)});Ok(())}}
 }
}
