//! CQ10: one original player, sequential real draws/discards and a finite continuation.
use crate::{engine::RuleResult,model::*,rules::*};
use serde::{Serialize,Deserialize};
use std::collections::BTreeSet;
pub(crate) const ID:&str="CQ10";
pub(crate) const ENTRY:&str="cq10-entry-draw-discard";
pub(crate) const PREPARE:&str="cq10-prepare-most-hand";
const DISCARD:&str="cq10-discard";
const TITLE:&str="朱苏德拉：弃一张手牌";
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Continuation {pub turn:u32,pub frame:ResolutionFrame,pub waiting:bool}
fn same<T:Serialize>(a:&T,b:&T)->bool{serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap()}
fn owns(ops:&[Op])->bool{ops.iter().any(|o|match o{Op::CQ10Entry{..}|Op::CQ10Repeat|Op::CQ10Prepare{..}=>true,Op::ForEachLivingPlayer(v)|Op::ForEachLivingPlayerFromActor(v)=>owns(v),Op::IfTargetExhausted{exhausted,ready,..}=>owns(std::slice::from_ref(exhausted))||owns(std::slice::from_ref(ready)),_=>false})}
pub(crate) fn related(a:&AbilitySpec)->bool{a.key.starts_with("cq10-")||owns(&a.ops)||a.modes.iter().any(|m|owns(&m.ops))}
fn entry(f:&ResolutionFrame)->bool{f.ability_key==ENTRY||f.steps.iter().any(|s|matches!(s.op,Op::CQ10Entry{..}|Op::CQ10Repeat))}
pub(crate) fn definition()->Definition{with_abilities(vec![
 ability(ENTRY,"进场：抓一张，目标玩家弃一张，按隐秘重复",Timing::Fast,vec![],vec![target(Zone::Player,EntityKind::Any,Relation::Any,Range::Anywhere)],vec![Op::CQ10Entry{secrecy:None,completed:0},Op::CQ10Repeat],Some(Event::Enter)),
 ability(PREPARE,"准备：若手牌严格最多，在目标地区放置本方势力",Timing::Fast,vec![],vec![target(Zone::Region,EntityKind::Any,Relation::Any,Range::Anywhere)],vec![Op::CQ10Prepare{event_id:None}],Some(Event::Prepare))])}
pub(crate) fn validate_ability(id:&str,a:&AbilitySpec)->RuleResult<()>{
 if id!=ID{return Err("CQ10程序不能移植".into());}
 if a.key==PREPARE{return crate::jc035::validate_ability(id,a);}
 if !same(a,&definition().abilities[0]){return Err("CQ10进场只能保留完整原目标和程序".into());}Ok(())
}
pub(crate) fn validate_definition(id:&str,d:&Definition)->RuleResult<()>{if (id==ID||d.abilities.iter().any(related))&&(id!=ID||!same(d,&definition())){return Err("CQ10完整定义不能增删或移植".into());}Ok(())}
pub(crate) fn validate_action(id:&str,a:&Action)->RuleResult<()>{
 if id!=ID{return Ok(());}let expected=match a.kind.as_str(){"deploy"|"conceal"=>Action{card_id:a.card_id.clone(),region:a.region,..Action::new(&a.kind)},"reveal"=>Action{card_id:a.card_id.clone(),..Action::new("reveal")},_=>return Err("CQ10角色操作无此额外载荷".into())};
 if *a!=expected{return Err("CQ10角色操作不能夹带重复次数或目标".into());}Ok(())
}
pub(crate) fn validate_choice(p:&Pending,a:&Action)->RuleResult<()>{
 let ours=match &p.resolution{ChoiceResolution::Declare{declaration,..}=>related(&declaration.ability),ChoiceResolution::Frame{frame,choice}=>entry(frame)||matches!(choice,FrameChoice::CQ10Discard),_=>false};
 if ours&&*a!=(Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")}){return Err("CQ10选择不能夹带次数、费用或目标".into());}Ok(())
}
impl Game {
 pub(crate) fn validate_cq10_play(&self,item:&StackItem)->RuleResult<()>{
  if !item.card.as_ref().is_some_and(|c|c.definition==ID)&&!item.frame.as_ref().is_some_and(|f|f.source.card.definition==ID&&matches!(f.ability_key.as_str(),"deploy"|"reveal")){return Ok(());}
  let f=item.frame.as_ref().ok_or("CQ10角色事务缺少原帧")?;let c=&f.source.card;
  if c.definition!=ID||f.actor>=self.players.len()||c.owner>=self.players.len()||c.controller!=f.actor||!self.mystic_instance_valid(&c.id)||!self.mystic_instance_valid(&f.frame_id)
   ||f.source.region.is_none_or(|r|r>=self.regions.len())||f.source.source_region_instance.as_deref().is_none_or(|id|!self.mystic_instance_valid(id))
   ||f.source.attachment_host_instance.is_some()||f.source.death_echo.is_some()||f.source.observed_death.is_some()||f.source.observed_exposure.is_some()||f.source.observed_mobility_grant.is_some()
   ||f.cursor!=0||!matches!(f.guard,GuardState::Unchecked)||!f.steps.is_empty()||!f.targets.is_empty()||f.chosen_region!=f.source.region
   ||!matches!(f.ability_key.as_str(),"deploy"|"reveal")||f.ability_key=="deploy"&&(c.face_down||!f.source.valid_character_play())||f.ability_key=="reveal"&&(!c.face_down||f.source.play_source.is_some())
   ||!matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)] if f.money_units(0)<=8&&ids.iter().all(|id|self.mystic_instance_valid(id)&&*id!=c.id)&&ids.iter().collect::<BTreeSet<_>>().len()==ids.len())
   ||item.id!=f.frame_id||item.controller!=f.actor||item.target.is_some()||item.reveal!=(f.ability_key=="reveal")||item.deploy_region!=f.source.region
   ||item.card.as_ref().is_none_or(|c|!self.pack_a_transaction_matches(f,c)){return Err("CQ10角色事务、原帧或实际牌面不符".into());}self.mobility_immutable_identity(c)
 }
 fn cq10_source(&self,actor:usize,s:&SourceSnapshot)->RuleResult<()>{let c=&s.card;
  if actor>=self.players.len()||c.owner>=self.players.len()||c.controller!=actor||c.definition!=ID||c.face_down||!self.mystic_instance_valid(&c.id)
   ||s.region.is_none_or(|r|r>=self.regions.len())||s.source_region_instance.as_deref().is_none_or(|id|!self.mystic_instance_valid(id)||id==c.id)
   ||s.play_source.is_some()||s.attachment_host_instance.is_some()||s.death_echo.is_some()||s.observed_death.is_some()||s.observed_exposure.is_some()||s.observed_mobility_grant.is_some(){return Err("CQ10进场原来源无效".into());}self.mobility_immutable_identity(c)
 }
 pub(crate) fn validate_cq10_declaration(&self,d:&Declaration)->RuleResult<()>{if d.ability.key==ENTRY||d.ability.ops.iter().any(|o|matches!(o,Op::CQ10Entry{..})){self.cq10_source(d.actor,&d.source)?;validate_ability(&d.source.card.definition,&d.ability)?;}Ok(())}
 fn cq10_player(&self,f:&ResolutionFrame)->RuleResult<usize>{
  let [t]=f.targets.as_slice()else{return Err("CQ10须保留一个原玩家目标".into());};
  let p=t.id.strip_prefix('p').and_then(|s|s.parse::<usize>().ok()).filter(|p|*p<self.players.len()&&t.id==player_id(*p)).ok_or("CQ10玩家目标无效")?;
  if !same(&t.spec,&definition().abilities[0].targets[0])||t.public.instance_id!=t.id||t.public.kind!="player"||t.public.owner.is_some()||t.public.controller!=Some(player_id(p))||t.region_instance.is_some()||t.public.region.is_some()||!t.public.valid||t.public.status!="valid"||t.public.invalid_reason.is_some(){return Err("CQ10原玩家绑定被改写".into());}Ok(p)
 }
 pub(crate) fn validate_cq10_frame(&self,f:&ResolutionFrame)->RuleResult<()>{
  if !entry(f){return Ok(());}self.cq10_source(f.actor,&f.source)?;self.cq10_player(f)?;
  let [Step{context,op:Op::CQ10Entry{secrecy,completed}},Step{context:next_context,op:Op::CQ10Repeat}]=f.steps.as_slice()else{return Err("CQ10只能有一个有限进场程序".into());};
  if f.ability_key!=ENTRY||*context!=f.actor||*next_context!=f.actor||!self.mystic_instance_valid(&f.frame_id)||f.frame_id==f.source.card.id||!f.already_paid.is_empty()||!f.resource_payments.is_empty()||f.chosen_region.is_some(){return Err("CQ10来源、费用或程序无效".into());}
  match secrecy {
   None=>if *completed!=0||f.cursor!=0||!matches!(f.guard,GuardState::Unchecked)||self.cq10_continuations.iter().any(|r|r.frame.frame_id==f.frame_id){return Err("CQ10未结算进场不能已有进度".into());},
   Some(m)=>{
    if f.cursor!=1||!matches!(f.guard,GuardState::Accepted)||*completed>=u64::from(*m)+1{return Err("CQ10重复次数或游标无效".into());}
    let r=self.cq10_continuations.iter().find(|r|r.frame.frame_id==f.frame_id).ok_or("CQ10缺少原续体记录")?;
    if r.turn!=self.turn||!same(&r.frame,f){return Err("CQ10续体来源、原目标或重复进度被改写".into());}
   }
  }Ok(())
 }
 pub(crate) fn cq10_run(&mut self,mut f:ResolutionFrame)->RuleResult<()>{
  let initial=matches!(f.steps.as_slice(),[Step{op:Op::CQ10Entry{secrecy:None,completed:0},..},Step{op:Op::CQ10Repeat,..}]);
  if initial{
   if f.cursor!=1||!matches!(f.guard,GuardState::Accepted){return Err("CQ10进场必须经过原目标守卫".into());}
   let mut original=f.clone();original.cursor=0;original.guard=GuardState::Unchecked;self.validate_cq10_frame(&original)?;
   f.steps[0].op=Op::CQ10Entry{secrecy:Some(self.players[f.actor].society_zone.secrecy),completed:0};
   self.cq10_continuations.push(Continuation{turn:self.turn,frame:f.clone(),waiting:false});
  }
  self.validate_cq10_frame(&f)?;
  let seat=self.cq10_player(&f)?;
  loop{
   let Some(index)=self.cq10_continuations.iter().position(|r|r.frame.frame_id==f.frame_id)else{if self.status!="playing"{return Ok(());}return Err("CQ10续体丢失".into());};
   if self.cq10_continuations[index].waiting{return Err("CQ10待选尚未完成，不能重复抓牌".into());}
   let Op::CQ10Entry{secrecy:Some(m),completed}=f.steps[0].op else{return Err("CQ10缺少重复观察".into());};
   if completed>=u64::from(m)+1||self.status!="playing"||self.players[f.actor].eliminated&&(self.players[seat].eliminated||self.players[seat].hand.is_empty()){
    self.cq10_continuations.remove(index);return Ok(());
   }
   self.draw(f.actor,1)?;
   if self.status!="playing"{return Ok(());}
   if !self.players[seat].eliminated&&!self.players[seat].hand.is_empty(){
    self.cq10_continuations[index].waiting=true;
    let options=self.players[seat].hand.iter().map(|c|self.option(c,seat,None,None)).collect();
    self.choice(seat,DISCARD,TITLE.into(),options,1,1,None,ChoiceResolution::Frame{frame:Box::new(f),choice:FrameChoice::CQ10Discard});return Ok(());
   }
   f.steps[0].op=Op::CQ10Entry{secrecy:Some(m),completed:completed+1};self.cq10_continuations[index].frame=f.clone();
  }
 }
 pub(crate) fn cq10_discard(&mut self,f:ResolutionFrame,seat:usize,selected:&[String])->RuleResult<()>{
  self.validate_cq10_frame(&f)?;
  let i=self.cq10_continuations.iter().position(|r|r.frame.frame_id==f.frame_id).ok_or("CQ10缺少原弃牌续体")?;
  if !self.cq10_continuations[i].waiting||seat!=self.cq10_player(&f)?||selected.len()!=1||!self.players[seat].hand.iter().any(|c|c.id==selected[0]){return Err("CQ10只能由原玩家弃一张实际手牌".into());}
  // Keep the original frame unchanged until real discard/replacement observers
  // finish. CQ10Repeat advances the count only when that original frame resumes.
  self.cq10_continuations[i].waiting=false;
  self.effects.push_front(Effect::Frame{frame:Box::new(f.clone())});
  self.discard_effect_cards(seat,selected,&f,false)
 }
 pub(crate) fn cq10_repeat(&mut self,mut f:ResolutionFrame)->RuleResult<()>{
  if f.cursor!=2{return Err("CQ10重复必须经原弃牌后续步骤".into());}
  f.cursor=1;self.validate_cq10_frame(&f)?;
  let i=self.cq10_continuations.iter().position(|r|r.frame.frame_id==f.frame_id).ok_or("CQ10重复缺少原进度")?;
  if self.cq10_continuations[i].waiting{return Err("CQ10弃牌未完成不能推进次数".into());}
  let Op::CQ10Entry{secrecy:Some(m),completed}=f.steps[0].op else{return Err("CQ10重复观察无效".into());};
  if completed+1==u64::from(m)+1{self.cq10_continuations.remove(i);return Ok(());}
  f.steps[0].op=Op::CQ10Entry{secrecy:Some(m),completed:completed+1};self.cq10_continuations[i].frame=f.clone();self.cq10_run(f)
 }
 pub(crate) fn cq10_bound_region(&self,f:&ResolutionFrame)->bool{
  let [t]=f.targets.as_slice()else{return false;};let Some(r)=t.id.strip_prefix("region:").and_then(|s|s.parse::<usize>().ok()).filter(|r|*r<self.regions.len())else{return false;};
  same(&t.spec,&definition().abilities[1].targets[0])&&t.id==format!("region:{r}")&&f.chosen_region==Some(r)&&t.public.instance_id==t.id&&t.public.kind=="region"&&t.public.region==Some(r)&&t.public.owner.is_none()&&t.public.controller.is_none()&&t.public.valid&&t.public.status=="valid"&&t.public.invalid_reason.is_none()&&t.region_instance.as_deref().is_some_and(|id|self.mystic_instance_valid(id)&&id!=f.source.card.id&&!self.regions.iter().enumerate().any(|(i,v)|i!=r&&v.card.id==id))
 }
 pub(crate) fn cq10_prepare(&mut self,f:&ResolutionFrame)->RuleResult<()>{
  if !self.cq10_bound_region(f){return Err("CQ10准备缺少原地区绑定".into());}
  if !self.players[f.actor].eliminated&&self.players.iter().filter(|p|p.seat!=f.actor).all(|p|self.players[f.actor].hand.len()>p.hand.len()){
   let r=f.chosen_region.unwrap();if self.region_live(r)&&f.targets[0].region_instance.as_ref()==Some(&self.regions[r].card.id){self.place_influence(f.actor,r,1);}
  }Ok(())
 }
 pub(crate) fn validate_cq10_state(&self)->RuleResult<()>{
  let mut ids=BTreeSet::new();let mut carriers=BTreeSet::new();
  for r in &self.cq10_continuations{if !ids.insert(&r.frame.frame_id){return Err("CQ10重复续体记录".into());}self.validate_cq10_frame(&r.frame)?;}
  let mut stack_ids=BTreeSet::new();
  for item in &self.stack{self.validate_cq10_play(item)?;if let Some(f)=&item.frame{if entry(f){self.validate_cq10_frame(f)?;if !stack_ids.insert(&f.frame_id)||item.id!=f.frame_id||item.controller!=f.actor||item.card.is_some()||item.reveal||item.deploy_region.is_some()||item.target.as_deref()!=Some(f.targets[0].id.as_str())||f.cursor!=0{return Err("CQ10原响应外壳无效".into());}}}}
  for e in &self.effects{match e{
   Effect::Declare{declaration}=>self.validate_cq10_declaration(declaration)?,
   Effect::Frame{frame} if entry(frame)=>{self.validate_cq10_frame(frame)?;let r=self.cq10_continuations.iter().find(|r|r.frame.frame_id==frame.frame_id).ok_or("CQ10队列续体缺少记录")?;if r.waiting||!carriers.insert(&frame.frame_id){return Err("CQ10队列续体重复或仍在待选".into());}},
   _=>()
  }}
  if let Some(p)=&self.pending{match &p.resolution{
   ChoiceResolution::Frame{frame,choice} if entry(frame)||matches!(choice,FrameChoice::CQ10Discard)=>{
    self.validate_cq10_frame(frame)?;let seat=self.cq10_player(frame)?;
    let r=self.cq10_continuations.iter().find(|r|r.frame.frame_id==frame.frame_id).ok_or("CQ10待选缺少记录")?;
    let options=self.players[seat].hand.iter().map(|c|self.option(c,seat,None,None)).collect::<Vec<_>>();
    let expected=Choice{id:p.choice.id.clone(),kind:DISCARD.into(),title:TITLE.into(),description:"选择符合数量限制的选项，然后确认。".into(),player_id:player_id(seat),options,preview_cards:vec![],min:Some(1),max:Some(1),amount:None,allow_decline:Some(false)};
    if !r.waiting||!carriers.insert(&frame.frame_id)||!matches!(choice,FrameChoice::CQ10Discard)||p.seat!=seat||!self.mystic_instance_valid(&p.choice.id)||expected.options.is_empty()||!same(&p.choice,&expected){return Err("CQ10弃牌待选或唯一续体被改写".into());}
   },
   ChoiceResolution::Declare{declaration:d,stage} if d.ability.key==ENTRY=>{
    self.validate_cq10_declaration(d)?;
    let expected=Choice{id:p.choice.id.clone(),kind:"trigger".into(),title:format!("{}：{}",crate::batch3_trap::source_label(&d.source),d.ability.label),description:"选择符合数量限制的选项，然后确认。".into(),player_id:player_id(d.actor),options:self.binding_options(d.actor,&d.source,&d.ability.targets[0]),preview_cards:vec![],min:Some(0),max:Some(1),amount:None,allow_decline:Some(true)};
    if !matches!(stage,DeclareChoice::Target)||!self.mystic_instance_valid(&p.choice.id)||p.seat!=d.actor||expected.options.is_empty()||!same(&p.choice,&expected){return Err("CQ10原玩家目标待选被改写".into());}
   },_=>()
  }}
  if ids!=carriers{return Err("CQ10每条进度必须有且只有一个实际续体".into());}Ok(())
 }
}
