//! Original identity only: closed clock-group WIP, not a completed society.
//! No initial construction and no executable actions until the group closes.
use crate::{catalog,society::{SocietyDefinition,SocietyDeckConstraint}};
pub(crate) const ID:&str="MSJZ03";
pub(crate) fn society()->SocietyDefinition{
 let mut card:catalog::CardDefinition=serde_json::from_value(serde_json::json!({"id":ID,"name":"末日钟","kind":"society","subtypes":["圣物","永恒意志"],"color":"红","society":"末日钟","unique":true,"supported":false,"text":"不能作为初始秘社牌。持续：不能打出或付费翻面非灾难类别的牌。持续：不能获得分数；若计分区有三个或更多毁灭标志，赢得游戏。行动横置：目标角色、附属或地区放置时间标志，然后该地区时间至少四则毁灭。行动横置、弃一张牌：目标地区生成钟摆祭司角色指示物。"})).unwrap();
 card.abilities=definition().abilities.iter().map(|a|catalog::AbilitySummary{key:a.key.clone(),label:a.label.clone(),timing:"standard".into(),costs:a.costs.clone(),triggered:false}).collect();
 SocietyDefinition{card,subtitle:"熵之子".into(),printed_cost:None,starting_hand:0,deck_constraints:vec![SocietyDeckConstraint::NotStartingSociety],unresolved_abilities:[("clock-group-wip".into(),"工程未完成：转换入口、持续规则及两项行动；不是待用户裁定".into())].into_iter().collect()}
}

use crate::{engine::RuleResult,model::*};
use serde::{Serialize,Deserialize};
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZeroScore {pub seat:usize,pub card_id:String,pub destroyed:bool}
impl Game{
 pub(crate) fn msjz03_active(&self,seat:usize)->bool{self.players.get(seat).is_some_and(|p|!p.eliminated&&p.society_zone.card.as_ref().is_some_and(|c|c.definition==ID&&!c.face_down&&c.controller==seat&&c.owner==seat))}
 pub(crate) fn msjz03_allow_play(&self,seat:usize,c:&Card)->RuleResult<()>{if self.msjz03_active(seat)&&!catalog::card(&c.definition).subtypes.iter().any(|s|s=="灾难"){return Err("末日钟只能打出或付费现身灾难类别牌".into());}Ok(())}
 pub(crate) fn msjz03_score(&mut self,seat:usize,card_id:String,destroyed:bool){if self.msjz03_active(seat){self.msjz03_zero_scores.push(ZeroScore{seat,card_id,destroyed});}}
 pub(crate) fn validate_msjz03_state(&self)->RuleResult<()>{self.msjz03_action_state()?;let mut ids=std::collections::BTreeSet::new();let mut destroyed=vec![0u32;self.players.len()];for z in &self.msjz03_zero_scores{if z.seat>=self.players.len()||!self.mystic_instance_valid(&z.card_id)||!ids.insert(&z.card_id)||!self.players[z.seat].score_cards.iter().any(|c|c.id==z.card_id&&c.owner==z.seat&&c.controller==z.seat){return Err("末日钟零分地区收据身份损坏".into());}if z.destroyed{destroyed[z.seat]+=1;}}if self.players.iter().any(|p|destroyed[p.seat]>p.destruction_markers){return Err("末日钟零分毁灭收据超过实际标志".into());}Ok(())}
}

use crate::rules::*;
pub(crate) const ADD:&str="msjz03-time";
pub(crate) const GENERATE:&str="msjz03-priest";
fn same<T:Serialize>(a:&T,b:&T)->bool{serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap()}
fn carrier()->TargetSlotSpec{let mut t=target(Zone::Board,EntityKind::Any,Relation::Any,Range::Anywhere);t.predicate=Some(TargetPredicate::MSJZ03Carrier);t}
fn region()->TargetSlotSpec{target(Zone::Region,EntityKind::Any,Relation::Any,Range::Anywhere)}
pub(crate) fn definition()->Definition{
 let mut add=ability(ADD,"横置：目标时间1，然后其所在地区时间至少4则毁灭",Timing::Standard,vec![Cost::ExhaustSource],vec![],vec![],None);add.activation_only=true;
 add.modes=vec![Mode{key:"carrier".into(),label:"角色或附属".into(),targets:vec![carrier()],ops:vec![Op::MSJZ03Time]},Mode{key:"region".into(),label:"地区".into(),targets:vec![region()],ops:vec![Op::MSJZ03Time]}];
 let mut make=ability(GENERATE,"横置并弃一张牌：目标地区生成钟摆祭司",Timing::Standard,vec![Cost::ExhaustSource,Cost::DiscardSelectedHandCard],vec![region()],vec![Op::MSJZ03Generate],None);make.activation_only=true;
 with_abilities(vec![add,make])
}
fn owns(op:&Op)->bool{match op{Op::MSJZ03Time|Op::MSJZ03Generate=>true,Op::ForEachLivingPlayer(v)|Op::ForEachLivingPlayerFromActor(v)=>v.iter().any(owns),Op::IfTargetExhausted{exhausted,ready,..}=>owns(exhausted)||owns(ready),_=>false}}
pub(crate) fn related(a:&AbilitySpec)->bool{a.key.starts_with("msjz03-")||a.ops.iter().chain(a.modes.iter().flat_map(|m|&m.ops)).any(owns)||a.targets.iter().chain(a.modes.iter().flat_map(|m|&m.targets)).any(|t|t.predicate==Some(TargetPredicate::MSJZ03Carrier))}
fn variants()->Vec<AbilitySpec>{let d=definition();let mut all=d.abilities.clone();for a in d.abilities{for m in &a.modes{let mut f=a.clone();f.targets=m.targets.clone();f.ops=m.ops.clone();f.modes.clear();all.push(f);}}all}
pub(crate) fn validate_ability(id:&str,a:&AbilitySpec)->RuleResult<()>{if id!=ID||!variants().iter().any(|v|same(v,a)){return Err("末日钟须保留两项原行动、费用与模式".into())}Ok(())}
pub(crate) fn validate_definition(id:&str,d:&Definition)->RuleResult<()>{if(id==ID||d.abilities.iter().any(related))&&(id!=ID||!same(d,&definition())){return Err("末日钟定义不能改写或移植".into())}Ok(())}
pub(crate) fn action(a:&Action)->RuleResult<()>{let mut e=Action{card_id:a.card_id.clone(),ability_id:a.ability_id.clone(),..Action::new("activate")};match a.ability_id.as_deref(){Some(ADD)=>{e.option=a.option.clone();match a.option.as_deref(){Some("carrier")=>e.target_id=a.target_id.clone(),Some("region")=>e.region=a.region,_=>return Err("末日钟时间须选择目标模式".into())}},Some(GENERATE)=>{e.region=a.region;e.cost_selected=a.cost_selected.clone();},_=>return Err("末日钟仅有两项原行动".into())}if *a!=e{return Err("末日钟行动含多余目标或费用载荷".into())}Ok(())}
impl Game{
 pub(crate) fn msjz03_carrier(&self,c:&Card)->bool{(!c.face_down&&catalog::card(&c.definition).kind=="character")||self.attachments.iter().any(|a|a.card.id==c.id&&self.attachment_host_valid(a)&&(!c.face_down||crate::batch4_plans::secret(c))&&catalog::has_kind(&catalog::card(&c.definition),"attachment"))}
 pub(crate) fn msjz03_frame(&self,f:&ResolutionFrame)->RuleResult<()>{
  let rel=f.ability_key.starts_with("msjz03-")||f.steps.iter().any(|s|owns(&s.op));if !rel {if f.source.card.definition==ID&&!self.validated_cross_card_frame(f)?{return Err("末日钟不能借用外来源帧".into())}return Ok(())}
  let a=variants().into_iter().find(|a|a.key==f.ability_key&&a.modes.is_empty()&&same(&a.targets,&f.targets.iter().map(|t|t.spec.clone()).collect::<Vec<_>>())).ok_or("末日钟原程序或目标模式损坏")?;
  let c=&f.source.card;
  let costs=match(f.ability_key.as_str(),f.already_paid.as_slice()){(ADD,[PaidCost::Exhausted(id)])=>*id==c.id,(GENERATE,[PaidCost::Exhausted(id),PaidCost::Discarded{old_instance,holder,owner}])=>*id==c.id&&*holder==f.actor&&*owner<self.players.len()&&self.mystic_instance_valid(old_instance)&&*old_instance!=c.id,_=>false};
  if c.definition!=ID||c.face_down||c.owner!=f.actor||c.controller!=f.actor||f.actor>=self.players.len()||!self.mystic_instance_valid(&c.id)||!self.mystic_instance_valid(&f.frame_id)||f.frame_id==c.id||f.source.region.is_some()||f.source.source_region_instance.is_some()||f.source.attachment_host_instance.is_some()||f.source.play_source.is_some()||f.source.death_echo.is_some()||f.source.observed_death.is_some()||f.source.observed_exposure.is_some()||f.source.observed_mobility_grant.is_some()||!costs||!f.resource_payments.is_empty()||f.cursor>1||f.cursor==0&&!matches!(f.guard,GuardState::Unchecked)||f.cursor==1&&!matches!(f.guard,GuardState::Accepted)||!same(&f.steps,&a.ops.iter().map(|o|Step{context:f.actor,op:o.clone()}).collect::<Vec<_>>()){return Err("末日钟来源、费用或续体阶段损坏".into())}
  let t=&f.targets[0];if !t.public.valid||t.public.status!="valid"||t.public.invalid_reason.is_some()||t.public.instance_id!=t.id{return Err("末日钟原目标摘要损坏".into())}
  if t.spec.zone==Zone::Region{let r=t.id.strip_prefix("region:").and_then(|s|s.parse::<usize>().ok()).filter(|r|*r<self.regions.len()).ok_or("末日钟目标地区无效")?;if t.id!=format!("region:{r}")||f.chosen_region!=Some(r)||t.public.region!=Some(r)||t.public.kind!="region"||t.public.owner.is_some()||t.public.controller.is_some()||t.region_instance.as_deref().is_none_or(|s|!self.mystic_instance_valid(s)){return Err("末日钟须绑定原地区实体".into())}}
  else if f.chosen_region.is_some()||t.region_instance.is_some()||!self.mystic_instance_valid(&t.id)||!matches!(t.public.kind.as_str(),"character"|"attachment")||t.public.region.is_some_and(|r|r>=self.regions.len()){return Err("末日钟角色附属绑定损坏".into())}
  self.mobility_immutable_identity(c)
 }
 pub(crate) fn msjz03_step(&mut self,f:&ResolutionFrame)->RuleResult<()>{self.msjz03_frame(f)?;if f.cursor!=1{return Err("末日钟必须消费原步骤".into())}let t=&f.targets[0];if !self.valid_bound_target(f.actor,&f.source,t){return Ok(())}
  if f.ability_key==GENERATE{let r=f.chosen_region.ok_or("生成缺少目标地区")?;let c=self.make_generated(crate::generated::GeneratedKind::PendulumPriest,f.actor)?;let id=c.id.clone();self.regions[r].cards.push(c);self.enter_triggers(f.actor,"TK009",&id,false);}
  else{let r=if t.spec.zone==Zone::Region{let r=f.chosen_region.ok_or("时间缺少目标地区")?;self.regions[r].card.time_markers=self.regions[r].card.time_markers.saturating_add(1);Some(r)}else{let r=self.in_play(&t.id).and_then(|(r,_)|r);self.place_board_markers(&t.id,crate::batch4_markers::MarkerKind::Time,1);r};if let Some(r)=r.filter(|r|self.region_live(*r)){if self.physical_region_time_markers(r)>=4{self.regions[r].destroyed=true;}}}Ok(())
 }
 fn msjz03_action_state(&self)->RuleResult<()>{let mut seen=std::collections::BTreeSet::new();for i in &self.stack{self.msjz03_token_play(i)?;if let Some(f)=&i.frame{self.msjz03_frame(f)?;if f.ability_key.starts_with("msjz03-")&&(i.id!=f.frame_id||i.controller!=f.actor||i.card.is_some()||i.reveal||i.deploy_region.is_some()||i.target.as_deref()!=f.targets.first().map(|t|t.id.as_str())||f.cursor!=0||!seen.insert(&f.frame_id)){return Err("末日钟堆叠外壳或重复帧损坏".into())}}}
  for e in &self.effects{if let Effect::Frame{frame:f}=e{self.msjz03_frame(f)?;if f.ability_key.starts_with("msjz03-"){return Err("末日钟单步不应保留游离续体".into())}}}if let Some(Pending{resolution:ChoiceResolution::Frame{frame:f,..},..})=&self.pending{self.msjz03_frame(f)?;if f.ability_key.starts_with("msjz03-"){return Err("末日钟没有帧内待选".into())}}Ok(())
 }
 pub(crate) fn msjz03_token_play(&self,i:&StackItem)->RuleResult<()>{if !i.card.as_ref().is_some_and(|c|c.definition=="TK009")&&!i.frame.as_ref().is_some_and(|f|f.source.card.definition=="TK009"&&matches!(f.ability_key.as_str(),"deploy"|"reveal")){return Ok(())}let f=i.frame.as_ref().ok_or("钟摆祭司现身缺帧")?;let c=&f.source.card;
  if c.definition!="TK009"||!c.face_down||f.actor>=self.players.len()||c.controller!=f.actor||!self.mystic_instance_valid(&f.frame_id)||f.frame_id==c.id||f.ability_key!="reveal"||f.source.region.is_none_or(|r|r>=self.regions.len())||f.source.source_region_instance.as_deref().is_none_or(|s|!self.mystic_instance_valid(s))||f.source.play_source.is_some()||f.source.death_echo.is_some()||f.source.observed_death.is_some()||f.source.observed_exposure.is_some()||f.source.observed_mobility_grant.is_some()||f.source.attachment_host_instance.is_some()||f.cursor!=0||!matches!(f.guard,GuardState::Unchecked)||!f.steps.is_empty()||!f.targets.is_empty()||f.chosen_region!=f.source.region||!matches!(f.already_paid.as_slice(),[PaidCost::Assets(v)]if v.is_empty())||!f.resource_payments.is_empty()||i.id!=f.frame_id||i.controller!=f.actor||!i.reveal||i.deploy_region!=f.source.region||i.target.is_some()||i.card.as_ref().is_none_or(|c|!self.pack_a_transaction_matches(f,c)){return Err("钟摆祭司须保留真实零费现身事务".into())}self.validate_generated_identity(c)?;self.mobility_immutable_identity(c)
 }
}
