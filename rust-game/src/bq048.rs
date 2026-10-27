//! Closed BQ048 slice: printed core invocation and region damage.
//! Ready/marker interaction with an off-board core remains unresolved.
//! This is a paid command spell producer, never a free core player action.
use crate::{engine::RuleResult,model::*,rules::*};
pub(crate) const ID:&str="BQ048";
pub(crate) const KEY:&str="bq048-command";
fn same<T:serde::Serialize>(a:&T,b:&T)->bool{serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap()}
fn damage_target()->TargetSlotSpec {let mut t=target(Zone::Board,EntityKind::Character,Relation::Any,Range::Anywhere);t.predicate=Some(TargetPredicate::BQ048CoreOrTentacleRegion);t}
pub(crate) fn definition()->Definition {
 let mut a=ability(KEY,"位格压迫：选择模式",Timing::Fast,vec![],vec![],vec![],None);
 a.play_only=true;a.modes=vec![Mode{key:"core".into(),label:"发动一次核心能力".into(),targets:vec![],ops:vec![Op::BQ048InvokeCore]},Mode{key:"damage".into(),label:"对本方核心或触手角色所在地的目标角色造成1点伤害".into(),targets:vec![damage_target()],ops:vec![Op::DamageTarget{slot:0,amount:1}]}];with_abilities(vec![a])
}
fn selected(index:usize)->AbilitySpec{let mut a=definition().abilities.remove(0);a.ops=a.modes[index].ops.clone();a.targets=a.modes[index].targets.clone();a.modes.clear();a}
pub(crate) fn related(a:&AbilitySpec)->bool{a.key.starts_with("bq048-")||a.targets.iter().chain(a.modes.iter().flat_map(|m|&m.targets)).any(|t|t.predicate==Some(TargetPredicate::BQ048CoreOrTentacleRegion))||a.ops.iter().any(|o|matches!(o,Op::BQ048InvokeCore))||a.modes.iter().flat_map(|m|&m.ops).any(|o|matches!(o,Op::BQ048InvokeCore))}
pub(crate) fn validate_ability(id:&str,a:&AbilitySpec)->RuleResult<()>{if id!=ID||(!same(a,&definition().abilities[0])&&!same(a,&selected(0))&&!same(a,&selected(1))){return Err("BQ048须保留原快速命令已实现的模式程序".into());}Ok(())}
pub(crate) fn validate_definition(id:&str,d:&Definition)->RuleResult<()>{if (id==ID||d.abilities.iter().any(related))&&(id!=ID||!same(d,&definition())){return Err("BQ048命令模式不得改写或移植".into());}Ok(())}
pub(crate) fn validate_action(id:&str,a:&Action)->RuleResult<()>{if id!=ID{return Ok(());}
 if a.kind=="asset"&&*a==(Action{card_id:a.card_id.clone(),..Action::new("asset")}){return Ok(());}
 let mode=a.option.as_deref().ok_or("BQ048须明确选择模式")?;
 if !matches!(mode,"core"|"damage")||*a!=(Action{card_id:a.card_id.clone(),ability_id:a.ability_id.clone(),option:a.option.clone(),target_id:if mode=="damage"{a.target_id.clone()}else{None},..Action::new("play")})||a.ability_id.as_deref().is_some_and(|k|k!=KEY)||mode=="damage"&&a.target_id.is_none(){return Err("BQ048模式或目标载荷无效；ready尚未接入".into());}Ok(())
}
fn frame_related(f:&ResolutionFrame)->bool{f.source.card.definition==ID||f.targets.iter().any(|t|t.spec.predicate==Some(TargetPredicate::BQ048CoreOrTentacleRegion))||f.ability_key.starts_with("bq048-")||f.steps.iter().any(|s|matches!(s.op,Op::BQ048InvokeCore))}
impl Game {
 pub(crate) fn bq048_region_target(&self,actor:usize,source:&SourceSnapshot,target_id:&str)->bool {
  if source.card.definition!=ID{return false;}
  let Some((Some(region),_))=self.in_play(target_id) else{return false;};
  self.region_characters(region).any(|c| !c.face_down&&c.controller==actor&&(self.current_subtypes(c).iter().any(|s|s=="触手")||self.core_zones.iter().any(|z|z.owner==actor&&z.card.is_none()&&z.definition==c.definition&&c.owner==actor)))
 }
 pub(crate) fn validate_bq048_frame(&self,f:&ResolutionFrame)->RuleResult<()> {
  if !frame_related(f){return Ok(());}let c=&f.source.card;
  let damage=matches!(f.steps.as_slice(),[Step{op:Op::DamageTarget{slot:0,amount:1},..}]);
  let expected=if damage{Op::DamageTarget{slot:0,amount:1}}else{Op::BQ048InvokeCore};
  let targets_valid=if damage{matches!(f.targets.as_slice(),[t]if same(&t.spec,&damage_target())&&self.mystic_instance_valid(&t.id)&&t.public.instance_id==t.id&&t.region_instance.is_none()&&t.public.valid&&t.public.status=="valid"&&t.public.invalid_reason.is_none())}else{f.targets.is_empty()};
  if c.definition!=ID||c.face_down||c.owner>=self.players.len()||c.controller!=f.actor||f.actor>=self.players.len()||!self.mystic_instance_valid(&c.id)||!self.mystic_instance_valid(&f.frame_id)
   ||f.source.region.is_some()||f.source.source_region_instance.is_some()||f.source.play_source.is_some()||f.source.attachment_host_instance.is_some()||f.source.death_echo.is_some()||f.source.observed_death.is_some()||f.source.observed_exposure.is_some()||f.source.observed_mobility_grant.is_some()
   ||f.ability_key!=KEY||!targets_valid||f.chosen_region.is_some()||f.cursor>1||!same(&f.steps,&vec![Step{context:f.actor,op:expected}])
   ||!matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)]if f.money_units(0)<=1&&ids.iter().all(|id|self.mystic_instance_valid(id)&&*id!=c.id)&&ids.iter().collect::<std::collections::BTreeSet<_>>().len()==ids.len())
   ||f.cursor==0&&!matches!(f.guard,GuardState::Unchecked)||f.cursor==1&&!matches!(f.guard,GuardState::Accepted){return Err("BQ048命令须保留已付原帧和单次核心程序".into());}
  self.mobility_immutable_identity(c)
 }
 pub(crate) fn validate_bq048_play(&self,item:&StackItem)->RuleResult<()> {
  if !item.card.as_ref().is_some_and(|c|c.definition==ID)&&!item.frame.as_ref().is_some_and(frame_related){return Ok(());}
  let f=item.frame.as_ref().ok_or("BQ048真实事务缺帧")?;self.validate_bq048_frame(f)?;
  if f.source.card.definition!=ID||f.cursor!=0||item.id!=f.frame_id||item.controller!=f.actor||item.reveal||item.target.as_deref()!=f.targets.first().map(|t|t.id.as_str())||item.deploy_region.is_some()||item.card.as_ref().is_none_or(|c|!self.pack_a_transaction_matches(f,c)){return Err("BQ048实际事务与来源帧不符".into());}Ok(())
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
