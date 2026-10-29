//! WM031: actual first marker placement, forced respondable choice, then wounds.
use crate::{catalog,engine::RuleResult,model::*,rules::*,marker_placement::MarkerPlacement};
use serde::{Serialize,Deserialize};
use std::collections::{BTreeMap,BTreeSet};
pub(crate) const ID:&str="WM031";
pub(crate) const KEY:&str="wm031-first-marker";
const KIND:&str="wm031-character";
const TITLE:&str="选择一个本方角色承受凌虐恶魔的1点创伤";
fn same<T:Serialize>(a:&T,b:&T)->bool{serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap()}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation{pub placement:MarkerPlacement,pub source:SourceSnapshot,pub chain:Vec<Attachment>}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Victims{pub seat:usize,pub selected:Vec<(usize,String)>}
fn ability_for(event:Option<Box<Observation>>)->AbilitySpec{ability(KEY,"首次放置标志：每位玩家选择角色，各造成创伤1",Timing::Fast,vec![],vec![],vec![Op::WM031Wound{event}],Some(Event::MarkerPlaced))}
pub(crate) fn definition()->Definition{let mut d=with_abilities(vec![ability_for(None)]);d.traits.wound=1;d}
pub(crate) fn related(a:&AbilitySpec)->bool{a.key==KEY||a.event==Some(Event::MarkerPlaced)||a.ops.iter().any(|o|matches!(o,Op::WM031Wound{..}))}
pub(crate) fn validate_ability(id:&str,a:&AbilitySpec)->RuleResult<()>{let [Op::WM031Wound{event}]=a.ops.as_slice()else{return Err("凌虐恶魔固定程序缺失".into())};if id!=ID||!same(a,&ability_for(event.clone())){return Err("凌虐恶魔触发不能改写或移植".into());}Ok(())}
pub(crate) fn validate_definition(id:&str,d:&Definition)->RuleResult<()>{if (id==ID||d.abilities.iter().any(related))&&(id!=ID||!same(d,&definition())){return Err("凌虐恶魔须保留创伤及原首次触发".into());}Ok(())}
fn related_frame(f:&ResolutionFrame)->bool{f.ability_key==KEY||f.steps.iter().any(|s|matches!(s.op,Op::WM031Wound{..}))}
impl Game{
 pub(crate) fn wm031_observe(&mut self,event:&MarkerPlacement,_physical_first:bool){
  let mut id=event.carrier.id.clone();let mut chain=vec![];let mut visited=BTreeSet::new();
  while visited.insert(id.clone()) {
   let Some((region,c))=self.in_play(&id)else{break};
   if c.definition==ID&&!self.wm031_observations.iter().any(|o|o.source.card.id==id){
    let source=self.source_snapshot(c,region);let actor=c.controller;
    let o=Observation{placement:event.clone(),source:source.clone(),chain:chain.clone()};
    self.wm031_observations.push(o.clone());
    if !source.card.face_down&&!self.players[actor].eliminated{self.effects.push_back(Effect::Declare{declaration:Declaration{actor,source,ability:ability_for(Some(Box::new(o)))}});}
   }
   let Some(a)=self.attachments.iter().find(|a|a.card.id==id&&self.attachment_host_valid(a)).cloned()else{break};
   id=a.host_id.clone();chain.push(a);
  }
 }
 fn validate_wm031_observation(&self,o:&Observation)->RuleResult<()>{
  let e=&o.placement;let s=&o.source;let c=&s.card;
  if !self.first_marker_placements.iter().any(|p|p.carrier.id==e.carrier.id&&p.turn==e.turn)||e.turn!=self.turn||e.placed==0||e.before.checked_add(e.placed)!=Some(e.kind.count(&e.carrier))||c.definition!=ID||c.owner>=self.players.len()||c.controller>=self.players.len()||!self.mystic_instance_valid(&c.id)||!self.mystic_instance_valid(&e.carrier.id)||e.carrier.owner>=self.players.len()||e.carrier.controller>=self.players.len()||s.region.is_some_and(|r|r>=self.regions.len())||s.region.is_some()&&s.source_region_instance.as_deref().is_none_or(|id|!self.mystic_instance_valid(id))||s.region.is_none()&&s.attachment_host_instance.is_none()||s.play_source.is_some()||s.death_echo.is_some()||s.observed_death.is_some()||s.observed_exposure.is_some()||s.observed_mobility_grant.is_some(){return Err("凌虐恶魔宿主观察或实际放置身份损坏".into());}
  let mut id=e.carrier.id.clone();let mut seen=BTreeSet::new();
  for (i,a) in o.chain.iter().enumerate(){if a.card.id!=id||!seen.insert(id.clone())||!self.mystic_instance_valid(&a.host_id)||i==0&&a.card!=e.carrier{return Err("凌虐恶魔冻结附属链不连续".into());}self.mobility_immutable_identity(&a.card)?;id=a.host_id.clone();}
  if seen.contains(&id)||id!=c.id||o.chain.is_empty()&&e.carrier!=*c{return Err("凌虐恶魔实际承载者不能移植到其他宿主".into());}
  self.mobility_immutable_identity(c)?;self.mobility_immutable_identity(&e.carrier)
 }
 fn wm031_event(&self,actor:usize,s:&SourceSnapshot,a:&AbilitySpec)->RuleResult<()>{
  validate_ability(&s.card.definition,a)?;
  let [Op::WM031Wound{event:Some(e)}]=a.ops.as_slice()else{return Err("凌虐恶魔缺少实际首次放置".into())};
  if actor>=self.players.len()||s.card.controller!=actor||s.card.face_down||!same(s,&e.source)||!self.wm031_observations.iter().any(|o|same(o,e.as_ref()))||s.region.is_some_and(|r|r>=self.regions.len())||s.region.is_some()&&s.source_region_instance.as_deref().is_none_or(|id|!self.mystic_instance_valid(id))||s.region.is_none()&&s.attachment_host_instance.is_none()||s.play_source.is_some()||s.death_echo.is_some()||s.observed_death.is_some()||s.observed_exposure.is_some()||s.observed_mobility_grant.is_some(){return Err("凌虐恶魔首次放置与实际来源不符".into());}self.validate_wm031_observation(e)?;self.mobility_immutable_identity(&s.card)
 }
 pub(crate) fn wm031_forced(&mut self,d:&Declaration)->RuleResult<bool>{if !related(&d.ability){return Ok(false)}self.wm031_event(d.actor,&d.source,&d.ability)?;if !self.players[d.actor].eliminated{let f=self.make_frame(d.actor,d.source.clone(),&d.ability,vec![],vec![],None);self.dispatch_frame(f,d.ability.label.clone(),None,ResponsePolicy::Respondable);}Ok(true)}
 pub(crate) fn validate_wm031_frame(&self,f:&ResolutionFrame)->RuleResult<()>{if !related_frame(f){return Ok(())}let [Step{context,op:Op::WM031Wound{event}}]=f.steps.as_slice()else{return Err("凌虐恶魔须保留单一创伤程序".into())};self.wm031_event(f.actor,&f.source,&ability_for(event.clone()))?;if *context!=f.actor||f.ability_key!=KEY||f.cursor>1||f.cursor==0&&!matches!(f.guard,GuardState::Unchecked)||f.cursor==1&&!matches!(f.guard,GuardState::Accepted)||!f.targets.is_empty()||!f.already_paid.is_empty()||!f.resource_payments.is_empty()||f.chosen_region.is_some()||!self.mystic_instance_number(&f.source.card.id).zip(self.mystic_instance_number(&f.frame_id)).is_some_and(|(a,b)|a<b){return Err("凌虐恶魔帧、费用或游标损坏".into());}Ok(())}
 fn wm031_options(&self,seat:usize)->Vec<ChoiceOption>{self.in_play_cards().into_iter().filter(|(_,c)|!c.face_down&&c.controller==seat&&catalog::card(&c.definition).kind=="character").map(|(r,c)|ChoiceOption{id:c.id.clone(),label:catalog::card(&c.definition).name.clone(),card:Some(self.card_view(c,seat,r,None))}).collect()}
 fn wm031_victims(&self,f:&ResolutionFrame,v:&Victims)->RuleResult<()>{self.validate_wm031_frame(f)?;if f.cursor!=1||v.seat>self.players.len(){return Err("凌虐恶魔选择阶段错误".into());}let mut ids=BTreeSet::new();let mut seats=BTreeSet::new();for(s,id)in &v.selected{if *s>=v.seat||!ids.insert(id)||!seats.insert(s)||!self.wm031_options(*s).iter().any(|o|o.id==*id){return Err("凌虐恶魔原选择或当前操控权损坏".into());}}for seat in 0..v.seat{if !self.players[seat].eliminated&&!self.wm031_options(seat).is_empty()&&!seats.contains(&seat){return Err("凌虐恶魔不能跳过先前有合法角色的玩家".into());}}Ok(())}
 pub(crate) fn wm031_continue(&mut self,f:ResolutionFrame,mut v:Victims)->RuleResult<()>{self.wm031_victims(&f,&v)?;while v.seat<self.players.len(){let seat=v.seat;let options=self.wm031_options(seat);if !self.players[seat].eliminated&&!options.is_empty(){self.choice(seat,KIND,TITLE.into(),options,1,1,None,ChoiceResolution::Frame{frame:Box::new(f),choice:FrameChoice::WM031Victims(v)});self.pending.as_mut().unwrap().choice.allow_decline=Some(false);return Ok(())}v.seat+=1;}
  self.place_wounds(v.selected.into_iter().map(|(_,id)|(id,1)).collect::<BTreeMap<_,_>>())?;self.settle_deaths();Ok(())
 }
 pub(crate) fn wm031_choose(&mut self,f:ResolutionFrame,mut v:Victims,seat:usize,a:&Action,ids:&[String])->RuleResult<()>{self.wm031_victims(&f,&v)?;if seat!=v.seat||ids.len()!=1||*a!=(Action{choice_id:a.choice_id.clone(),selected:Some(ids.to_vec()),..Action::new("choose")})||!self.wm031_options(seat).iter().any(|o|o.id==ids[0]){return Err("凌虐恶魔须由本玩家选择一个实际操控角色".into());}v.selected.push((seat,ids[0].clone()));v.seat+=1;self.wm031_continue(f,v)}
 pub(crate) fn validate_wm031_state(&self)->RuleResult<()>{let mut observed=BTreeSet::new();for o in &self.wm031_observations{self.validate_wm031_observation(o)?;if !observed.insert(&o.source.card.id){return Err("凌虐恶魔同回合宿主首次观察重复".into());}}let mut declarations=BTreeSet::new();let mut sources=BTreeSet::new();let mut check=|f:&ResolutionFrame|->RuleResult<()>{self.validate_wm031_frame(f)?;if related_frame(f)&&!sources.insert(f.source.card.id.clone()){return Err("凌虐恶魔同次首次触发不能重复承载".into())}Ok(())};for i in &self.stack{if let Some(f)=&i.frame{check(f)?;if related_frame(f)&&(i.card.is_some()||i.id!=f.frame_id||i.controller!=f.actor||i.reveal||i.deploy_region.is_some()||i.target.is_some()||f.cursor!=0){return Err("凌虐恶魔响应外壳损坏".into());}}}for e in &self.effects{match e{Effect::Frame{frame}=>check(frame)?,Effect::DeathCaller{caller}=>check(&caller.frame)?,Effect::Declare{declaration:d}if related(&d.ability)=>{self.wm031_event(d.actor,&d.source,&d.ability)?;if !declarations.insert(d.source.card.id.clone()){return Err("凌虐恶魔首次声明重复".into());}},_=>()}}if let Some(p)=&self.pending{match &p.resolution{ChoiceResolution::Frame{frame,choice:FrameChoice::WM031Victims(v)}=>{check(frame)?;self.wm031_victims(frame,v)?;if v.seat>=self.players.len()||p.seat!=v.seat||p.choice.player_id!=player_id(v.seat)||p.choice.kind!=KIND||p.choice.title!=TITLE||p.choice.min!=Some(1)||p.choice.max!=Some(1)||p.choice.allow_decline!=Some(false)||p.choice.amount.is_some()||!p.choice.preview_cards.is_empty()||!same(&p.choice.options,&self.wm031_options(v.seat)){return Err("凌虐恶魔选择者或候选损坏".into());}},ChoiceResolution::Frame{frame,..}if related_frame(frame)=>return Err("凌虐恶魔不能使用外来选择".into()),ChoiceResolution::Declare{declaration:d,..}if related(&d.ability)=>return Err("凌虐恶魔强制触发不能改为可选".into()),_=>()}}drop(check);if declarations.iter().any(|id|sources.contains(id)){return Err("凌虐恶魔声明与活动帧不能重复".into());}Ok(())}
}
