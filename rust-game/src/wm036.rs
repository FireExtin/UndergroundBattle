//! Closed Ashes to Ashes. Reuses the existing per-player sacrifice and death caller.
use crate::{catalog,engine::RuleResult,model::*,rules::*};
use std::collections::BTreeSet;
pub(crate) const ID:&str="WM036";
pub(crate) const PLAY:&str="wm036-each-sacrifice";
pub(crate) const SHOWN:&str="clock-wm036-shown";
pub(crate) const TIME_KIND:&str="wm036-shown-time-target";
pub(crate) const TIME_TITLE:&str="烬归烬：计时者可以在目标角色或附属放置时间1";
fn same<T:serde::Serialize>(a:&T,b:&T)->bool{serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap()}
pub(crate) fn definition()->Definition{
 let mut a=ability(PLAY,"行动：每位玩家牺牲一个角色",Timing::Standard,vec![],vec![],vec![Op::ForEachLivingPlayer(vec![Op::SacrificeChosen(PlayerRef::Context)])],None);
 a.play_only=true;with_abilities(vec![a])
}
pub(crate) fn validate_ability(a:&AbilitySpec)->RuleResult<()>{if !same(a,&definition().abilities[0]){return Err("烬归烬只准入完整每位玩家牺牲程序".into())}Ok(())}
pub(crate) fn validate_definition(d:&Definition)->RuleResult<()>{if !same(d,&definition()){return Err("烬归烬不能改写或移植其他能力".into())}Ok(())}
pub(crate) fn validate_action(a:&Action)->RuleResult<()>{if *a!=(Action{card_id:a.card_id.clone(),ability_id:a.ability_id.clone(),..Action::new("play")})||a.ability_id.as_deref().is_some_and(|k|k!=PLAY){return Err("烬归烬打出不预选目标或牺牲牌".into())}Ok(())}
pub(crate) fn related(f:&ResolutionFrame)->bool{f.source.card.definition==ID||matches!(f.ability_key.as_str(),PLAY|SHOWN)}
impl Game{
 pub(crate) fn validate_wm036_frame(&self,f:&ResolutionFrame)->RuleResult<()>{
  if f.ability_key==SHOWN{return self.validate_wm037_frame(f)}
  let c=&f.source.card;
  if c.definition!=ID||c.face_down||f.actor>=self.players.len()||c.controller!=f.actor||c.owner>=self.players.len()
   ||!self.mystic_instance_valid(&c.id)||!self.mystic_instance_valid(&f.frame_id)||f.frame_id==c.id
   ||f.source.region.is_some()||f.source.source_region_instance.is_some()||f.source.attachment_host_instance.is_some()
   ||f.source.play_source.is_some()||f.source.death_echo.is_some()||f.source.observed_death.is_some()
   ||f.source.observed_exposure.is_some()||f.source.observed_mobility_grant.is_some()||f.chosen_region.is_some()||!f.targets.is_empty(){return Err("烬归烬须绑定原事务、行动者及费用".into())}
  self.mobility_immutable_identity(c)?;
  let seats=f.steps.iter().map(|s|s.context).collect::<Vec<_>>();
  if f.ability_key!=PLAY||seats.is_empty()||!seats.contains(&f.actor)||seats.iter().any(|s|*s>=self.players.len())||seats.windows(2).any(|w|w[0]>=w[1])
   ||self.players.iter().any(|p|!p.eliminated&&!seats.contains(&p.seat))||f.steps.iter().any(|s|!matches!(s.op,Op::SacrificeChosen(PlayerRef::Context)))
   ||f.cursor>f.steps.len()||f.cursor==0&&!matches!(f.guard,GuardState::Unchecked)||f.cursor>0&&!matches!(f.guard,GuardState::Accepted)
   ||!matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)]if f.money_units(0)<=1&&ids.iter().all(|id|self.mystic_instance_valid(id)&&*id!=c.id)&&ids.iter().collect::<BTreeSet<_>>().len()==ids.len()){
   return Err("烬归烬须保留实际费用、全体玩家递增次序与牺牲游标".into())
  }Ok(())
 }
 fn wm036_options(&self,seat:usize)->Vec<ChoiceOption>{self.regions.iter().enumerate().filter(|(_,r)|!r.vacant).flat_map(|(r,v)|v.cards.iter().map(move|c|(r,c))).filter(|(_,c)|c.controller==seat&&!c.face_down&&catalog::card(&c.definition).kind=="character").map(|(r,c)|self.option(c,seat,Some(r),None)).collect()}
 pub(crate) fn validate_wm036_choice(&self,f:&ResolutionFrame,choice:&FrameChoice,seat:usize,a:&Action,selected:&[String])->RuleResult<()>{
  if !related(f)||f.ability_key==SHOWN{return Ok(())}self.validate_wm036_frame(f)?;
  if f.cursor==0||!matches!(choice,FrameChoice::Sacrifice{seat:s}if *s==seat&&*s==f.steps[f.cursor-1].context)||!matches!(f.guard,GuardState::Accepted)
   ||*a!=(Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")})||selected.len()!=1||!self.wm036_options(seat).iter().any(|o|o.id==selected[0]){return Err("烬归烬须由当前玩家选择其实际操控的一个明置角色".into())}Ok(())
 }
 pub(crate) fn validate_wm036_state(&self)->RuleResult<()>{
  let mut seen=BTreeSet::new();
  for item in &self.stack{
   if let Some(f)=&item.frame{if related(f)||item.card.as_ref().is_some_and(|c|c.definition==ID){
    self.validate_wm036_frame(f)?;
    if f.ability_key!=PLAY||f.cursor!=0||!matches!(f.guard,GuardState::Unchecked)||item.id!=f.frame_id||item.controller!=f.actor||item.target.is_some()||item.deploy_region.is_some()||item.reveal||item.card.as_ref().is_none_or(|c|c.definition!=ID||c.owner!=f.source.card.owner||c.controller!=f.actor||c.face_down){return Err("烬归烬真实事务堆栈与原帧不符".into())}
    if !seen.insert(&f.frame_id){return Err("烬归烬帧重复".into())}
   }}else if item.card.as_ref().is_some_and(|c|c.definition==ID){return Err("烬归烬事务缺少原帧".into())}
  }
  for e in &self.effects{match e{Effect::Frame{frame:f}if related(f)=>{self.validate_wm036_frame(f)?;if !seen.insert(&f.frame_id){return Err("烬归烬续体重复".into())}},Effect::Declare{declaration:d}if d.source.card.definition==ID=>return Err("烬归烬没有场内声明触发".into()),_=>()}}
  if let Some(p)=&self.pending{match &p.resolution{
   ChoiceResolution::Frame{frame:f,choice}if related(f)=>{self.validate_wm036_frame(f)?;if f.ability_key==SHOWN{return Ok(())}if !seen.insert(&f.frame_id){return Err("烬归烬待选帧重复".into())}
    if f.cursor==0||!matches!(choice,FrameChoice::Sacrifice{seat}if *seat==p.seat&&*seat==f.steps[f.cursor-1].context){return Err("烬归烬待选玩家或阶段不符".into())}
    let options=self.wm036_options(p.seat);if options.is_empty()||p.choice.player_id!=player_id(p.seat)||p.choice.kind!="target"||p.choice.title!="选择一个自己操控的角色牺牲"||p.choice.min!=Some(1)||p.choice.max!=Some(1)||p.choice.allow_decline!=Some(false)||p.choice.amount.is_some()||!p.choice.preview_cards.is_empty()||!same(&p.choice.options,&options){return Err("烬归烬待选须保留当前全部合法本方角色".into())}
   },ChoiceResolution::Declare{declaration:d,..}if d.source.card.definition==ID=>return Err("烬归烬不能伪造可选场内触发".into()),_=>()
  }}Ok(())
 }
}
