//! MSBQ02 dynamic Chaos: retain the paid stack item while selecting a late region.
//! Project timing choice: selection occurs during resolution, without a new response window.
use crate::{engine::RuleResult,model::*};
fn same<T:serde::Serialize>(a:&T,b:&T)->bool{serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap()}
impl Game {
 pub(crate) fn core_entry_needs_region(&self,item:&StackItem)->bool {
  !item.reveal&&item.deploy_region.is_none()&&item.card.as_ref().is_some_and(|c|crate::core_zone::is_core(&c.definition)&&self.msbq02_core(item.controller,c)&&!self.msbq02_chaos(c))
 }
 fn core_entry_regions(&self,item:&StackItem)->Vec<(usize,String)> {
  let Some(c)=&item.card else{return vec![]};(0..self.regions.len()).filter(|r|self.region_live(*r)&&!self.jc064_blocks_character(c,*r)).map(|r|(r,self.regions[r].card.id.clone())).collect()
 }
 fn core_entry_options(&self,regions:&[(usize,String)])->Vec<ChoiceOption>{regions.iter().map(|(r,_)|ChoiceOption{id:format!("region:{r}"),label:format!("进入地区{}",r+1),card:None}).collect()}
 pub(crate) fn begin_core_entry_region(&mut self,item:StackItem)->RuleResult<()> {
  self.validate_core_play(&item)?;
  if !self.core_entry_needs_region(&item)||self.pending.is_some(){return Err("核心补选不能替换原派遣或覆盖待选".into());}
  let regions=self.core_entry_regions(&item);
  if regions.is_empty(){self.effects.push_front(Effect::Bury{card:item.card.ok_or("核心补选缺实体")?});return Ok(());}
  let frame_id=item.id.clone();let seat=item.controller;self.stack.push(item);
  self.choice(seat,"core-entry-region","混乱已失效：选择核心进场地区".into(),self.core_entry_options(&regions),1,1,None,ChoiceResolution::CoreEntryRegion{frame_id,regions});Ok(())
 }
 pub(crate) fn validate_core_entry_region(&self)->RuleResult<()> {
  let Some(p)=&self.pending else{return Ok(())};let ChoiceResolution::CoreEntryRegion{frame_id,regions}=&p.resolution else{if p.choice.kind=="core-entry-region"{return Err("核心补选种类不能脱离原事务续体".into());}return Ok(())};
  let item=self.stack.last().filter(|i|i.id==*frame_id).ok_or("核心补选须绑定原栈顶事务")?;
  self.validate_core_play(item)?;
  if !self.core_entry_needs_region(item)||p.seat!=item.controller||p.choice.player_id!=player_id(item.controller)||regions.is_empty()||*regions!=self.core_entry_regions(item)
   ||p.choice.kind!="core-entry-region"||p.choice.title!="混乱已失效：选择核心进场地区"||p.choice.min!=Some(1)||p.choice.max!=Some(1)||p.choice.amount.is_some()||p.choice.allow_decline!=Some(false)||!p.choice.preview_cards.is_empty()||!self.mystic_instance_valid(&p.choice.id)||!same(&p.choice.options,&self.core_entry_options(regions))
   ||self.stack.iter().filter(|i|i.id==*frame_id).count()!=1||self.effects.iter().any(|e|matches!(e,Effect::Frame{frame}if frame.frame_id==*frame_id)) {return Err("核心补选的操控者、地区实例、数量或唯一续体损坏".into());}
  Ok(())
 }
 pub(crate) fn finish_core_entry_region(&mut self,frame_id:&str,regions:&[(usize,String)],a:&Action,selected:&[String])->RuleResult<()> {
  if *a!=(Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")})||selected.len()!=1{return Err("核心补选必须且只能选择一个地区".into());}
  let item=self.stack.last().filter(|i|i.id==frame_id).ok_or("核心补选原事务已取消")?;
  if !self.core_entry_needs_region(item)||regions!=self.core_entry_regions(item){return Err("核心补选地区资格已变化".into());}
  let r=regions.iter().find(|(r,_)|selected[0]==format!("region:{r}")).map(|(r,_)|*r).ok_or("核心补选地区无效")?;
  let item=self.stack.pop().unwrap();self.resolve_stack_with_core_region(item,Some(r))
 }
}
