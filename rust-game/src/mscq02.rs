//! Closed Norvi society: two original optional triggers and a continuous region check.
use crate::{catalog,engine::RuleResult,model::*,rules::*,society::{SocietyDefinition,SocietyDeckConstraint}};
use serde::{Deserialize,Serialize};
use std::collections::BTreeSet;
pub(crate) const ID:&str="MSCQ02";
pub(crate) const START:&str="mscq02-start";
pub(crate) const WIN:&str="mscq02-win";
#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct Observation { pub instance:String, pub opening:bool }
fn same<T:Serialize>(a:&T,b:&T)->bool{serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap()}
fn mark(o:&Observation)->String{format!("mscq02-{}:{}",if o.opening{"start"}else{"win"},o.instance)}
pub(crate) fn definition()->Definition{with_abilities(vec![ability(START,"游戏开始：毁灭原地区三",Timing::Fast,vec![],vec![],vec![Op::MSCQ02{observation:None}],Some(Event::MSCQ02Start)),ability(WIN,"赢得已毁灭地区：抓一张牌",Timing::Fast,vec![],vec![],vec![Op::MSCQ02{observation:None}],Some(Event::MSCQ02Won))])}
pub(crate) fn society()->SocietyDefinition{
 let mut card:catalog::CardDefinition=serde_json::from_value(serde_json::json!({"id":ID,"name":"诺尔维隐修会","kind":"society","type":"秘社/秘密教派","subtypes":["秘密教派"],"color":"红","society":"诺尔维隐修会","unique":true,"supported":false,"text":"构筑仅红、中立及灾难、时间、命运牌。游戏开始设置地区后，毁灭地区三。地区上时间标志至少四则毁灭该地区。赢得已毁灭地区时抓一张牌。"})).unwrap();
 card.abilities=definition().abilities.iter().map(|a|catalog::AbilitySummary{key:a.key.clone(),label:a.label.clone(),timing:"fast".into(),costs:a.costs.clone(),triggered:true}).collect();SocietyDefinition{card,subtitle:"时序拨动者".into(),printed_cost:None,starting_hand:5,deck_constraints:vec![SocietyDeckConstraint::RedNeutralOrPrintedDisasterTimeFate],unresolved_abilities:Default::default()}
}
pub(crate) fn related(a:&AbilitySpec)->bool{a.key.starts_with("mscq02-")||matches!(a.event,Some(Event::MSCQ02Start|Event::MSCQ02Won))||a.ops.iter().any(|o|matches!(o,Op::MSCQ02{..}))}
pub(crate) fn validate_ability(id:&str,a:&AbilitySpec)->RuleResult<()>{let mut expected=definition().abilities.into_iter().find(|b|b.key==a.key).ok_or("未知诺尔维触发")?;if let [Op::MSCQ02{observation}]=a.ops.as_slice(){expected.ops=vec![Op::MSCQ02{observation:observation.clone()}];}if id!=ID||!same(a,&expected){return Err("诺尔维须保留原触发时点与完整程序".into());}Ok(())}
pub(crate) fn validate_definition(id:&str,d:&Definition)->RuleResult<()>{if (id==ID||d.abilities.iter().any(related))&&(id!=ID||!same(d,&definition())){return Err("诺尔维原规则不可删改或移植".into());}Ok(())}
impl Game {
 fn mscq02_active(&self,seat:usize)->bool{self.players.get(seat).is_some_and(|p|!p.eliminated&&p.society_zone.card.as_ref().is_some_and(|c|c.definition==ID&&!c.face_down&&c.controller==seat))}
 pub(crate) fn mscq02_continuous(&mut self){if !(0..self.players.len()).any(|s|self.mscq02_active(s)){return;}let regions=(0..self.regions.len()).filter(|r|self.physical_region_time_markers(*r)>=4).collect::<Vec<_>>();for r in regions{self.regions[r].destroyed=true;}}
 fn mscq02_observe(&mut self,seat:usize,o:Observation){if !self.mscq02_active(seat){return;}let tag=mark(&o);if self.players[seat].society_zone.mscq02_observed.contains(&tag){return;}self.players[seat].society_zone.mscq02_observed.insert(tag);let source=self.source_snapshot(self.players[seat].society_zone.card.as_ref().unwrap(),None);let mut a=definition().abilities.into_iter().find(|a|a.key==if o.opening{START}else{WIN}).unwrap();a.ops=vec![Op::MSCQ02{observation:Some(o)}];self.effects.push_back(Effect::Declare{declaration:Declaration{actor:seat,source,ability:a}});}
 pub(crate) fn mscq02_observe_start(&mut self){if let Some(r)=self.regions.get(2).filter(|r|!r.vacant){let id=r.card.id.clone();let mut seats=(0..self.players.len()).collect::<Vec<_>>();seats.sort_by_key(|s|self.team(*s)!=self.first_team);for seat in seats{self.mscq02_observe(seat,Observation{instance:id.clone(),opening:true});}}}
 pub(crate) fn mscq02_observe_win(&mut self,seat:usize){if let Some(c)=self.players[seat].score_cards.last(){let id=c.id.clone();self.mscq02_observe(seat,Observation{instance:id,opening:false});}}
 fn mscq02_source(&self,actor:usize,s:&SourceSnapshot,key:&str,ops:&[Op])->RuleResult<Observation>{
  let c=&s.card;let [Op::MSCQ02{observation:Some(o)}]=ops else{return Err("诺尔维触发缺实际事件观察".into());};
  if c.definition!=ID||actor>=self.players.len()||c.owner!=actor||c.controller!=actor||c.face_down||!self.mystic_instance_valid(&c.id)||!self.mystic_instance_valid(&o.instance)||s.region.is_some()||s.source_region_instance.is_some()||s.play_source.is_some()||s.attachment_host_instance.is_some()||s.death_echo.is_some()||s.observed_death.is_some()||s.observed_exposure.is_some()||s.observed_mobility_grant.is_some()||key!=if o.opening{START}else{WIN}||!self.players[actor].society_zone.mscq02_observed.contains(&mark(o)){return Err("诺尔维原秘社、事件或观察收据不符".into());}
  if !o.opening&&(!self.players[actor].score_cards.iter().any(|c|c.id==o.instance)||self.players[actor].destruction_markers==0){return Err("诺尔维赢区缺原已计分地区及毁灭标志".into());}self.mobility_immutable_identity(c)?;Ok(o.clone())
 }
 pub(crate) fn mscq02_declaration(&self,d:&Declaration)->RuleResult<()>{if d.source.card.definition==ID||related(&d.ability){validate_ability(&d.source.card.definition,&d.ability)?;self.mscq02_source(d.actor,&d.source,&d.ability.key,&d.ability.ops)?;}Ok(())}
 pub(crate) fn mscq02_frame(&self,f:&ResolutionFrame)->RuleResult<()>{if f.source.card.definition!=ID&&!f.ability_key.starts_with("mscq02-")&&!f.steps.iter().any(|s|matches!(s.op,Op::MSCQ02{..})){return Ok(());}self.mscq02_source(f.actor,&f.source,&f.ability_key,&f.steps.iter().map(|s|s.op.clone()).collect::<Vec<_>>())?;if !self.mystic_instance_valid(&f.frame_id)||f.frame_id==f.source.card.id||f.steps.len()!=1||f.steps[0].context!=f.actor||f.cursor>1||f.cursor==0&&!matches!(f.guard,GuardState::Unchecked)||f.cursor==1&&!matches!(f.guard,GuardState::Accepted)||!f.already_paid.is_empty()||!f.resource_payments.is_empty()||!f.targets.is_empty()||f.chosen_region.is_some(){return Err("诺尔维须保留单步原触发且无费用或目标".into());}Ok(())}
 pub(crate) fn mscq02_resolve(&mut self,f:&ResolutionFrame)->RuleResult<()>{self.mscq02_frame(f)?;if f.cursor!=1{return Err("诺尔维须先消费原步骤".into());}let o=self.mscq02_source(f.actor,&f.source,&f.ability_key,&f.steps.iter().map(|s|s.op.clone()).collect::<Vec<_>>())?;if o.opening{if let Some(r)=self.regions.get_mut(2).filter(|r|!r.vacant&&r.card.id==o.instance){r.destroyed=true;}}else{self.draw(f.actor,1)?;}Ok(())}
 pub(crate) fn validate_mscq02_state(&self)->RuleResult<()>{
  if (0..self.players.len()).any(|s|self.mscq02_active(s))&&(0..self.regions.len()).any(|r|!self.regions[r].destroyed&&self.physical_region_time_markers(r)>=4){return Err("诺尔维持续毁灭不能被跳过".into());}
  let mut seen=BTreeSet::new();let mut remember=|actor:usize,ops:&[Op]|->RuleResult<()>{if let [Op::MSCQ02{observation:Some(o)}]=ops{if !seen.insert((actor,mark(o))){return Err("诺尔维同一事件不能重复排队".into());}}Ok(())};
  for i in &self.stack{if let Some(f)=&i.frame{self.mscq02_frame(f)?;if f.source.card.definition==ID{remember(f.actor,&f.steps.iter().map(|s|s.op.clone()).collect::<Vec<_>>())?;if i.card.is_some()||i.id!=f.frame_id||i.controller!=f.actor||i.target.is_some()||i.deploy_region.is_some()||i.reveal||f.cursor!=0{return Err("诺尔维触发堆叠外壳损坏".into());}}}}
  for e in &self.effects{match e{Effect::Declare{declaration:d}=>{self.mscq02_declaration(d)?;if d.source.card.definition==ID{remember(d.actor,&d.ability.ops)?;}},Effect::Frame{frame:f}=>{self.mscq02_frame(f)?;if f.source.card.definition==ID{return Err("诺尔维没有额外执行续体".into());}},_=>{}}}
  if let Some(p)=&self.pending{match &p.resolution{ChoiceResolution::Declare{declaration:d,stage}=>{self.mscq02_declaration(d)?;if d.source.card.definition==ID{remember(d.actor,&d.ability.ops)?;if !matches!(stage,DeclareChoice::Accept)||p.seat!=d.actor||p.choice.kind!="trigger"||p.choice.min!=Some(0)||p.choice.max!=Some(1)||p.choice.options.len()!=1||p.choice.options[0].id!="accept"{return Err("诺尔维只能确认原可选触发".into());}}},ChoiceResolution::Frame{frame:f,..}=>{self.mscq02_frame(f)?;if f.source.card.definition==ID{return Err("诺尔维结算不产生额外选择".into());}},_=>{}}}Ok(())
 }
}
