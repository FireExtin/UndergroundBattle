//! Closed BQ048 slice: only the printed core-invocation mode is implemented.
//! The ready/marker and core-or-tentacle-region damage modes remain unavailable.
//! This is a paid command spell producer, never a free core player action.
use crate::{engine::RuleResult,model::*,rules::*};
pub(crate) const ID:&str="BQ048";
pub(crate) const KEY:&str="bq048-command";
fn same<T:serde::Serialize>(a:&T,b:&T)->bool{serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap()}
pub(crate) fn definition()->Definition {
 let mut a=ability(KEY,"位格压迫：发动一次核心能力",Timing::Fast,vec![],vec![],vec![],None);
 a.play_only=true;a.modes=vec![Mode{key:"core".into(),label:"发动一次核心能力".into(),targets:vec![],ops:vec![Op::BQ048InvokeCore]}];with_abilities(vec![a])
}
fn selected()->AbilitySpec{let mut a=definition().abilities.remove(0);a.ops=a.modes[0].ops.clone();a.modes.clear();a}
pub(crate) fn related(a:&AbilitySpec)->bool{a.key.starts_with("bq048-")||a.ops.iter().any(|o|matches!(o,Op::BQ048InvokeCore))||a.modes.iter().flat_map(|m|&m.ops).any(|o|matches!(o,Op::BQ048InvokeCore))}
pub(crate) fn validate_ability(id:&str,a:&AbilitySpec)->RuleResult<()>{if id!=ID||(!same(a,&definition().abilities[0])&&!same(a,&selected())){return Err("BQ048已接入模式仅为原快速命令的核心发动程序".into());}Ok(())}
pub(crate) fn validate_definition(id:&str,d:&Definition)->RuleResult<()>{if (id==ID||d.abilities.iter().any(related))&&(id!=ID||!same(d,&definition())){return Err("BQ048命令模式不得改写或移植".into());}Ok(())}
pub(crate) fn validate_action(id:&str,a:&Action)->RuleResult<()>{if id!=ID{return Ok(());}
 if a.kind=="asset"&&*a==(Action{card_id:a.card_id.clone(),..Action::new("asset")}){return Ok(());}
 if *a!=(Action{card_id:a.card_id.clone(),ability_id:a.ability_id.clone(),option:Some("core".into()),..Action::new("play")})||a.ability_id.as_deref().is_some_and(|k|k!=KEY){return Err("BQ048本候选仅接入印刷core模式；其他模式尚未实现".into());}Ok(())
}
fn frame_related(f:&ResolutionFrame)->bool{f.source.card.definition==ID||f.ability_key.starts_with("bq048-")||f.steps.iter().any(|s|matches!(s.op,Op::BQ048InvokeCore))}
impl Game {
 pub(crate) fn validate_bq048_frame(&self,f:&ResolutionFrame)->RuleResult<()> {
  if !frame_related(f){return Ok(());}let c=&f.source.card;
  if c.definition!=ID||c.face_down||c.owner>=self.players.len()||c.controller!=f.actor||f.actor>=self.players.len()||!self.mystic_instance_valid(&c.id)||!self.mystic_instance_valid(&f.frame_id)
   ||f.source.region.is_some()||f.source.source_region_instance.is_some()||f.source.play_source.is_some()||f.source.attachment_host_instance.is_some()||f.source.death_echo.is_some()||f.source.observed_death.is_some()||f.source.observed_exposure.is_some()||f.source.observed_mobility_grant.is_some()
   ||f.ability_key!=KEY||!f.targets.is_empty()||f.chosen_region.is_some()||f.cursor>1||!same(&f.steps,&vec![Step{context:f.actor,op:Op::BQ048InvokeCore}])
   ||!matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)]if f.money_units(0)<=1&&ids.iter().all(|id|self.mystic_instance_valid(id)&&*id!=c.id)&&ids.iter().collect::<std::collections::BTreeSet<_>>().len()==ids.len())
   ||f.cursor==0&&!matches!(f.guard,GuardState::Unchecked)||f.cursor==1&&!matches!(f.guard,GuardState::Accepted){return Err("BQ048命令须保留已付原帧和单次核心程序".into());}
  self.mobility_immutable_identity(c)
 }
 pub(crate) fn validate_bq048_play(&self,item:&StackItem)->RuleResult<()> {
  if !item.card.as_ref().is_some_and(|c|c.definition==ID)&&!item.frame.as_ref().is_some_and(frame_related){return Ok(());}
  let f=item.frame.as_ref().ok_or("BQ048真实事务缺帧")?;self.validate_bq048_frame(f)?;
  if f.source.card.definition!=ID||f.cursor!=0||item.id!=f.frame_id||item.controller!=f.actor||item.reveal||item.target.is_some()||item.deploy_region.is_some()||item.card.as_ref().is_none_or(|c|!self.pack_a_transaction_matches(f,c)){return Err("BQ048实际事务与来源帧不符".into());}Ok(())
 }
 pub(crate) fn validate_bq048_state(&self)->RuleResult<()> {
  for item in &self.stack{self.validate_bq048_play(item)?;}
  for e in &self.effects{if let Effect::Frame{frame}=e{self.validate_bq048_frame(frame)?;}}
  if self.pending.as_ref().is_some_and(|p|matches!(&p.resolution,ChoiceResolution::Frame{frame,..}if frame_related(frame))){return Err("BQ048单次发动不能伪装其他帧待选".into());}Ok(())
 }
 pub(crate) fn bq048_invoke(&mut self,f:&ResolutionFrame)->RuleResult<()> {
  self.validate_bq048_frame(f)?;
  if self.selected_core_source(f.actor).is_ok(){self.resolve_selected_bq046_core(f.actor)?;}
  Ok(())
 }
}
