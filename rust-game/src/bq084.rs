//! BQ084: optional anchor return during resolution, never an activation cost.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use std::collections::BTreeSet;
pub(crate) const ID:&str="BQ084";
pub(crate) const KEY:&str="bq084-anchor-grave-entry";
const CHOICE:&str="bq084-return-anchor";
const TITLE:&str="可以将本方秘社上的一张精神锚点洗回其拥有者牌库";
fn same<T:serde::Serialize>(a:&T,b:&T)->bool{serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap()}
fn owns(ops:&[Op])->bool{ops.iter().any(|op|match op{Op::BQ084AnchorEntry=>true,Op::ForEachLivingPlayer(v)|Op::ForEachLivingPlayerFromActor(v)=>owns(v),Op::IfTargetExhausted{exhausted,ready,..}=>owns(std::slice::from_ref(exhausted))||owns(std::slice::from_ref(ready)),_=>false})}
pub(crate) fn related(a:&AbilitySpec)->bool{a.key.starts_with("bq084-")||owns(&a.ops)||a.modes.iter().any(|m|owns(&m.ops))||a.targets.iter().any(|t|matches!(t.predicate,Some(TargetPredicate::BQ084OwnGraveCharacter)))}
fn frame_related(f:&ResolutionFrame)->bool{f.ability_key.starts_with("bq084-")||f.steps.iter().any(|s|owns(std::slice::from_ref(&s.op)))}
pub(crate) fn definition()->Definition{
 let mut t=target(Zone::Graveyard,EntityKind::Character,Relation::OwnedByActor,Range::Anywhere);t.predicate=Some(TargetPredicate::BQ084OwnGraveCharacter);
 let mut d=with_abilities(vec![ability(KEY,"进场：可以洗回一张精神锚点，将本人墓地目标角色放置本地区",Timing::Fast,vec![],vec![t],vec![Op::BQ084AnchorEntry],Some(Event::Enter))]);
 d.traits.spirit=true;d
}
pub(crate) fn validate_ability(id:&str,a:&AbilitySpec)->RuleResult<()>{if id!=ID||!same(a,&definition().abilities[0]){return Err("战栗凶灵只允许原可选锚点复归程序".into());}Ok(())}
pub(crate) fn validate_definition(id:&str,d:&Definition)->RuleResult<()>{if (id==ID||d.abilities.iter().any(related))&&(id!=ID||!same(d,&definition())){return Err("战栗凶灵完整能力和灵体不能增删或移植".into());}Ok(())}
pub(crate) fn validate_action(id:&str,a:&Action)->RuleResult<()>{
 if id!=ID{return Ok(());}let expected=match a.kind.as_str(){"deploy"|"conceal"=>Action{card_id:a.card_id.clone(),region:a.region,..Action::new(&a.kind)},"reveal"=>Action{card_id:a.card_id.clone(),..Action::new("reveal")},_=>return Err("战栗凶灵只有原角色派遣和现身操作".into())};
 if *a!=expected{return Err("战栗凶灵角色操作不能夹带锚点费用或额外载荷".into());}Ok(())
}
pub(crate) fn validate_choice(p:&Pending,a:&Action)->RuleResult<()>{
 let ours=match &p.resolution{ChoiceResolution::Declare{declaration,..}=>related(&declaration.ability),ChoiceResolution::Frame{frame,choice}=>frame_related(frame)||matches!(choice,FrameChoice::BQ084Anchor),_=>false};
 if ours&&*a!=(Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")}){return Err("战栗凶灵选择不能夹带费用、数量或地区".into());}Ok(())
}
impl Game{
 pub(crate) fn bq084_grave_target(&self,actor:usize,source:&SourceSnapshot,c:&Card)->bool{
  source.card.definition==ID&&self.bq095_reference_cards(&source.card).iter().any(|v|v==c)&&c.owner==self.bq095_reference_seat(&source.card)&&!c.face_down&&catalog::card(&c.definition).kind=="character"
 }
 fn bq084_source(&self,actor:usize,s:&SourceSnapshot)->RuleResult<()>{
  let c=&s.card;
  if actor>=self.players.len()||c.owner>=self.players.len()||c.definition!=ID||c.controller!=actor||c.face_down||!self.mystic_instance_valid(&c.id)
    ||s.region.is_none_or(|r|r>=self.regions.len())||s.source_region_instance.as_deref().is_none_or(|id|!self.mystic_instance_valid(id)||id==c.id)
    ||s.play_source.is_some()||s.death_echo.is_some()||s.observed_death.is_some()||s.observed_exposure.is_some()||s.observed_mobility_grant.is_some()||s.attachment_host_instance.is_some(){return Err("战栗凶灵原进场来源、执行者或地区无效".into());}
  self.mobility_immutable_identity(c)
 }
 pub(crate) fn validate_bq084_declaration(&self,d:&Declaration)->RuleResult<()>{
  if !related(&d.ability){return Ok(());}self.bq084_source(d.actor,&d.source)?;validate_ability(&d.source.card.definition,&d.ability)
 }
 pub(crate) fn validate_bq084_frame(&self,f:&ResolutionFrame)->RuleResult<()>{
  if !frame_related(f){return Ok(());}self.bq084_source(f.actor,&f.source)?;
  let a=definition().abilities.remove(0);
  if f.ability_key!=KEY||!self.mystic_instance_valid(&f.frame_id)||f.frame_id==f.source.card.id||!f.already_paid.is_empty()||!f.resource_payments.is_empty()||f.chosen_region.is_some()
    ||f.cursor>1||f.cursor==0&&!matches!(f.guard,GuardState::Unchecked)||f.cursor==1&&!matches!(f.guard,GuardState::Accepted)
    ||!same(&f.steps,&vec![Step{context:f.actor,op:Op::BQ084AnchorEntry}])||f.targets.len()!=1{return Err("战栗凶灵原程序、效果阶段或无费用触发被改写".into());}
  let t=&f.targets[0];
  if !same(&t.spec,&a.targets[0])||!self.mystic_instance_valid(&t.id)||t.public.instance_id!=t.id||t.region_instance.is_some()||t.public.kind!="character"
    ||t.public.owner!=Some(player_id(self.bq095_frozen_reference_seat(&f.source.card,&f.frame_id)))||t.public.controller!=Some(player_id(self.bq095_frozen_reference_seat(&f.source.card,&f.frame_id)))||t.public.region.is_some()||!t.public.valid||t.public.status!="valid"||t.public.invalid_reason.is_some(){return Err("战栗凶灵必须保留原本人墓地角色绑定".into());}Ok(())
 }
 fn bq084_options(&self,actor:usize)->Vec<ChoiceOption>{
  self.bq033_anchor_options(actor).iter().filter_map(|id|self.sealed_cards.iter().find(|s|s.card.id==*id)).map(|s|self.option(&s.card,actor,None,None)).collect()
 }
 pub(crate) fn bq084_start(&mut self,f:ResolutionFrame)->RuleResult<()>{
  self.validate_bq084_frame(&f)?;let options=self.bq084_options(f.actor);
  if options.is_empty(){return self.resolve_frame(f);}
  self.choice(f.actor,CHOICE,TITLE.into(),options,0,1,None,ChoiceResolution::Frame{frame:Box::new(f),choice:FrameChoice::BQ084Anchor});Ok(())
 }
 pub(crate) fn bq084_finish(&mut self,f:ResolutionFrame,seat:usize,a:&Action,selected:&[String])->RuleResult<()>{
  self.validate_bq084_frame(&f)?;
  if seat!=f.actor||f.cursor!=1||!matches!(f.guard,GuardState::Accepted)||selected.len()>1||*a!=(Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")}){return Err("战栗凶灵只能放弃或选择一张真实锚点".into());}
  if let Some(id)=selected.first(){
   // Reuse the real unseal/reset/owner-deck/shuffle primitive. Its receipt is
   // deliberately NOT appended to already_paid: this is an optional effect.
   self.bq033_pay_anchor(f.actor,id)?;
   let r=f.source.region.unwrap();
   if self.region_live(r)&&Some(&self.regions[r].card.id)==f.source.source_region_instance.as_ref()
    &&self.bq095_reference_cards(&f.source.card).iter().any(|c|c.id==f.targets[0].id){self.own_graveyard_entry(&f.source.card,f.actor,r,&f.targets[0].id)?;}
  }
  self.resolve_frame(f)
 }
 pub(crate) fn validate_bq084_state(&self)->RuleResult<()>{
  let mut ids=BTreeSet::new();
  for item in &self.stack{if let Some(f)=&item.frame{if frame_related(f){self.validate_bq084_frame(f)?;
   if !ids.insert(&f.frame_id)||item.id!=f.frame_id||item.controller!=f.actor||item.card.is_some()||item.deploy_region.is_some()||item.reveal||item.target.as_deref()!=Some(f.targets[0].id.as_str())||f.cursor!=0{return Err("战栗凶灵响应外壳或实例被改写".into());}
  }}}
  for e in &self.effects{match e{Effect::Declare{declaration}=>self.validate_bq084_declaration(declaration)?,Effect::Frame{frame} if frame_related(frame)=>return Err("战栗凶灵不能脱离原锚点待选重复续接".into()),Effect::DeathCaller{caller}=>self.validate_bq084_frame(&caller.frame)?,_=>()}}
  if let Some(p)=&self.pending{match &p.resolution{
   ChoiceResolution::Declare{declaration:d,stage} if related(&d.ability)=>{
    self.validate_bq084_declaration(d)?;
    let expected=Choice{id:p.choice.id.clone(),kind:"trigger".into(),title:format!("{}：{}",crate::batch3_trap::source_label(&d.source),d.ability.label),description:"选择符合数量限制的选项，然后确认。".into(),player_id:player_id(d.actor),options:self.binding_options(d.actor,&d.source,&d.ability.targets[0]),preview_cards:vec![],min:Some(0),max:Some(1),amount:None,allow_decline:Some(true)};
    if !matches!(stage,DeclareChoice::Target)||!self.mystic_instance_valid(&p.choice.id)||p.seat!=d.actor||expected.options.is_empty()||!same(&p.choice,&expected){return Err("战栗凶灵原墓地目标待选被改写".into());}
   },
   ChoiceResolution::Frame{frame:f,choice} if frame_related(f)||matches!(choice,FrameChoice::BQ084Anchor)=>{
    self.validate_bq084_frame(f)?;let expected=Choice{id:p.choice.id.clone(),kind:CHOICE.into(),title:TITLE.into(),description:"选择符合数量限制的选项，然后确认。".into(),player_id:player_id(f.actor),options:self.bq084_options(f.actor),preview_cards:vec![],min:Some(0),max:Some(1),amount:None,allow_decline:Some(true)};
    if !matches!(choice,FrameChoice::BQ084Anchor)||!self.mystic_instance_valid(&p.choice.id)||p.seat!=f.actor||f.cursor!=1||!matches!(f.guard,GuardState::Accepted)||!ids.insert(&f.frame_id)||expected.options.is_empty()||!same(&p.choice,&expected){return Err("战栗凶灵锚点待选、载体或续体被改写".into());}
   },_=>()
  }}Ok(())
 }
}
