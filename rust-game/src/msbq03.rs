//! Closed printed Scarab society: corpse selection is an effect, never a cost.
use crate::{catalog,engine::RuleResult,model::*,rules::*,society::{SocietyDefinition,SocietyDeckConstraint}};
use std::collections::BTreeSet;
pub(crate) const ID:&str="MSBQ03";
pub(crate) const SEAL:&str="msbq03-seal";
pub(crate) const SEARCH:&str="msbq03-search";
const INDUSTRY:&str="圣甲虫产业";
fn same<T:serde::Serialize>(a:&T,b:&T)->bool{serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap()}
fn host()->TargetSlotSpec{let mut t=target(Zone::Board,EntityKind::Attachment,Relation::ControlledByActor,Range::Anywhere);t.predicate=Some(TargetPredicate::MSBQ03Industry);t}
pub(crate) fn definition()->Definition{
 let mut seal=ability(SEAL,"将墓地一张无名尸体封印在目标本方产业上",Timing::Standard,vec![Cost::Assets(1),Cost::ExhaustSource],vec![host()],vec![Op::MSBQ03{search:false}],None);seal.activation_only=true;
 let mut search=ability(SEARCH,"每局一次：检索产业，展示入手后洗牌",Timing::Standard,vec![Cost::Assets(3),Cost::ExhaustSource],vec![],vec![Op::MSBQ03{search:true}],None);search.activation_only=true;search.once_per_game=true;with_abilities(vec![seal,search])
}
pub(crate) fn society()->SocietyDefinition{
 let mut card:catalog::CardDefinition=serde_json::from_value(serde_json::json!({"id":ID,"name":"圣甲虫药业","kind":"society","type":"秘社/企业","subtypes":["企业"],"color":"黑","society":"圣甲虫药业","unique":true,"supported":false,"text":"构筑只允许黑、白、中立，无名尸体不限张。行动1、横置：将本人墓地一张无名尸体封印在目标本方圣甲虫产业附属上。本方产业每封印一张角色牌获得永久势力1。行动3、横置，每局一次：检索一张圣甲虫产业牌，展示入手后洗牌。"})).unwrap();
 card.abilities=definition().abilities.iter().map(|a|catalog::AbilitySummary{key:a.key.clone(),label:a.label.clone(),timing:"standard".into(),costs:a.costs.clone(),triggered:false}).collect();SocietyDefinition{card,subtitle:"斯卡拉跨国集团".into(),printed_cost:None,starting_hand:6,deck_constraints:vec![SocietyDeckConstraint::BlackWhiteNeutralCorpse],unresolved_abilities:Default::default()}
}
pub(crate) fn related(a:&AbilitySpec)->bool{a.key.starts_with("msbq03-")||a.ops.iter().any(|o|matches!(o,Op::MSBQ03{..}))||a.targets.iter().any(|t|t.predicate==Some(TargetPredicate::MSBQ03Industry))}
fn ours(f:&ResolutionFrame)->bool{f.source.card.definition==ID||f.ability_key.starts_with("msbq03-")||f.steps.iter().any(|s|matches!(s.op,Op::MSBQ03{..}))||f.targets.iter().any(|t|t.spec.predicate==Some(TargetPredicate::MSBQ03Industry))}
pub(crate) fn validate_ability(id:&str,a:&AbilitySpec)->RuleResult<()>{if id!=ID||!definition().abilities.iter().any(|b|same(a,b)){return Err("圣甲虫药业须保留原费用、目标、效果选择及限次".into());}Ok(())}
pub(crate) fn validate_definition(id:&str,d:&Definition)->RuleResult<()>{if (id==ID||d.abilities.iter().any(related))&&(id!=ID||!same(d,&definition())){return Err("圣甲虫药业完整定义不能删改或移植".into());}Ok(())}
impl Game{
 pub(crate) fn msbq03_industry(&self,actor:usize,s:&SourceSnapshot,c:&Card)->bool{s.card.definition==ID&&!c.face_down&&c.controller==actor&&catalog::card(&c.definition).subtitle.as_deref()==Some(INDUSTRY)&&self.attachments.iter().any(|a|a.card==*c&&self.attachment_host_valid(a))}
 pub(crate) fn msbq03_influence(&self,c:&Card)->u32{
  if c.face_down||!self.players.get(c.controller).is_some_and(|p|!p.eliminated&&p.society_zone.card.as_ref().is_some_and(|s|s.definition==ID&&!s.face_down&&s.controller==c.controller))||catalog::card(&c.definition).subtitle.as_deref()!=Some(INDUSTRY)||!self.attachments.iter().any(|a|a.card==*c&&self.attachment_host_valid(a)){return 0;}
  // Explicit printed category reference, not ordinary attributes of blank sealed cards.
  self.sealed_cards.iter().filter(|s|s.host_id==c.id&&catalog::has_kind(catalog::card(&s.card.definition),"character")).count() as u32
 }
 pub(crate) fn msbq03_action(&self,actor:usize,a:&Action)->RuleResult<()>{
  if a.kind!="activate"||!self.players[actor].society_zone.card.as_ref().is_some_and(|c|c.definition==ID&&a.card_id.as_deref()==Some(&c.id)){return Ok(());}
  let key=a.ability_id.as_deref().ok_or("须指定圣甲虫行动")?;
  if !matches!(key,SEAL|SEARCH)||*a!=(Action{card_id:a.card_id.clone(),ability_id:a.ability_id.clone(),target_id:if key==SEAL{a.target_id.clone()}else{None},..Action::new("activate")})||key==SEAL&&a.target_id.is_none(){return Err("圣甲虫尸体是结算选择，不能夹带费用选择或额外载荷".into());}Ok(())
 }
 pub(crate) fn msbq03_frame(&self,f:&ResolutionFrame)->RuleResult<()>{
  if !ours(f){return Ok(());}self.validate_resource_frame(f)?;let s=&f.source;let c=&s.card;let search=f.ability_key==SEARCH;
  let [PaidCost::Assets(ids),PaidCost::Exhausted(source)]=f.already_paid.as_slice()else{return Err("圣甲虫行动缺资产和横置支付".into());};
  if c.definition!=ID||f.actor>=self.players.len()||c.owner!=f.actor||c.controller!=f.actor||c.face_down||c.exhausted||!self.mystic_instance_number(&c.id).zip(self.mystic_instance_number(&f.frame_id)).is_some_and(|(a,b)|a<b)||s.region.is_some()||s.source_region_instance.is_some()||s.play_source.is_some()||s.attachment_host_instance.is_some()||s.death_echo.is_some()||s.observed_death.is_some()||s.observed_exposure.is_some()||s.observed_mobility_grant.is_some()||!matches!(f.ability_key.as_str(),SEAL|SEARCH)||f.chosen_region.is_some()||!same(&f.steps,&vec![Step{context:f.actor,op:Op::MSBQ03{search}}])||f.cursor>1||f.cursor==0&&!matches!(f.guard,GuardState::Unchecked)||f.cursor==1&&!matches!(f.guard,GuardState::Accepted)||source!=&c.id||f.money_units(0)!=if search{3}else{1}||ids.iter().collect::<BTreeSet<_>>().len()!=ids.len()||ids.iter().any(|id|id==&c.id||!self.mystic_instance_number(id).zip(self.mystic_instance_number(&f.frame_id)).is_some_and(|(a,b)|a<b)) {return Err("圣甲虫原来源、费用或程序损坏".into());}
  self.mobility_immutable_identity(c)?;
  if search{if !f.targets.is_empty()||!self.players[f.actor].society_zone.used_once_per_game.contains(SEARCH){return Err("产业检索须保留已消耗限次且无目标".into());}}
  else{let [t]=f.targets.as_slice()else{return Err("封印须有原产业附属目标".into());};if !same(&t.spec,&host())||!self.mystic_instance_valid(&t.id)||t.public.instance_id!=t.id||t.public.kind!="attachment"||t.public.controller!=Some(player_id(f.actor))||t.region_instance.is_some()||!t.public.valid||t.public.status!="valid"||t.public.invalid_reason.is_some(){return Err("圣甲虫产业原目标绑定损坏".into());}}
  Ok(())
 }
 fn msbq03_options(&self,f:&ResolutionFrame,search:bool)->Vec<ChoiceOption>{
  let p=&self.players[f.actor];if p.eliminated{return vec![];}
  if search{p.deck.iter().filter(|c|catalog::card(&c.definition).subtitle.as_deref()==Some(INDUSTRY)).map(|c|self.option(c,f.actor,None,None)).collect()}
  else{self.bq095_reference_cards(&f.source.card).iter().filter(|c|catalog::card(&c.definition).name=="无名尸体").map(|c|self.option(c,f.actor,None,None)).collect()}
 }
 pub(crate) fn msbq03_resolve(&mut self,f:ResolutionFrame,search:bool)->RuleResult<()>{
  self.msbq03_frame(&f)?;if f.cursor!=1||search!=(f.ability_key==SEARCH){return Err("圣甲虫选择须消费原步骤".into());}
  if !search&&(!self.valid_bound_target(f.actor,&f.source,&f.targets[0])||!self.seal_host_valid(&f.targets[0].id)){return Ok(());}
  let options=self.msbq03_options(&f,search);if options.is_empty(){if search&&!self.players[f.actor].eliminated{self.shuffle_player(f.actor);}return Ok(());}
  self.choice(f.actor,if search{"search"}else{"msbq03-corpse"},if search{"检索产业，展示入手后洗牌"}else{"选择墓地一张无名尸体封印"}.into(),options,1,1,None,ChoiceResolution::Frame{frame:Box::new(f),choice:FrameChoice::MSBQ03{search}});Ok(())
 }
 pub(crate) fn msbq03_finish(&mut self,f:&ResolutionFrame,search:bool,seat:usize,a:&Action,ids:&[String])->RuleResult<()>{
  self.msbq03_frame(f)?;if seat!=f.actor||f.cursor!=1||search!=(f.ability_key==SEARCH)||*a!=(Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")})||ids.len()!=1||!self.msbq03_options(f,search).iter().any(|o|o.id==ids[0]){return Err("圣甲虫须选择原区域的一张实际牌".into());}
  if search{let i=self.players[seat].deck.iter().position(|c|c.id==ids[0]).ok_or("检索牌不在牌库")?;let c=self.players[seat].deck.remove(i);let c=self.fresh(c);self.note(format!("{} 展示检索的 {}",self.players[seat].name,catalog::card(&c.definition).name));self.players[seat].hand.push(c);self.shuffle_player(seat);}
  else{if !self.valid_bound_target(seat,&f.source,&f.targets[0])||!self.seal_host_valid(&f.targets[0].id){return Err("原产业目标已失效".into());}let c=self.bq095_take_reference(&f.source.card,&ids[0])?;self.seal_card_on_valid_host(c,seat,&f.targets[0].id);}Ok(())
 }
 pub(crate) fn validate_msbq03_state(&self)->RuleResult<()>{
  let mut frames=BTreeSet::new();let mut check=|f:&ResolutionFrame|->RuleResult<()>{self.msbq03_frame(f)?;if ours(f)&&!frames.insert(f.frame_id.clone()){return Err("圣甲虫原帧不能重复存在".into());}Ok(())};
  for i in &self.stack{if let Some(f)=&i.frame{check(f)?;if ours(f)&&(i.card.is_some()||i.id!=f.frame_id||i.controller!=f.actor||i.reveal||i.deploy_region.is_some()||i.target.as_deref()!=f.targets.first().map(|t|t.id.as_str())||f.cursor!=0){return Err("圣甲虫实际堆叠外壳不符".into());}}}
  for e in &self.effects{match e{Effect::Frame{frame:f}=>{check(f)?;if ours(f){return Err("圣甲虫单步没有独立执行续体".into());}},Effect::Declare{declaration:d}if d.source.card.definition==ID||related(&d.ability)=>return Err("圣甲虫没有触发声明".into()),_=>{}}}
  if let Some(p)=&self.pending{match &p.resolution{ChoiceResolution::Frame{frame:f,choice}if ours(f)||matches!(choice,FrameChoice::MSBQ03{..})=>{
   if f.source.card.definition!=ID||f.actor>=self.players.len(){return Err("圣甲虫待选必须携带原秘社与有效行动者".into());}
   check(f)?;let FrameChoice::MSBQ03{search}=choice else{return Err("圣甲虫待选类型不能替换".into());};let options=self.msbq03_options(f,*search);
   let expected=Choice{id:p.choice.id.clone(),kind:if *search{"search"}else{"msbq03-corpse"}.into(),title:if *search{"检索产业，展示入手后洗牌"}else{"选择墓地一张无名尸体封印"}.into(),description:"选择符合数量限制的选项，然后确认。".into(),player_id:player_id(f.actor),options,min:Some(1),max:Some(1),amount:None,allow_decline:Some(false),preview_cards:vec![]};
   if !ours(f)||f.cursor!=1||*search!=(f.ability_key==SEARCH)||p.seat!=f.actor||!self.mystic_instance_valid(&p.choice.id)||expected.options.is_empty()||!same(&expected,&p.choice)||!*search&&(!self.valid_bound_target(f.actor,&f.source,&f.targets[0])||!self.seal_host_valid(&f.targets[0].id)){return Err("圣甲虫待选、原目标或实际可选牌损坏".into());}
  },ChoiceResolution::Declare{declaration:d,..}if d.source.card.definition==ID||related(&d.ability)=>return Err("圣甲虫没有触发待选".into()),_=>{}}}Ok(())
 }
}
