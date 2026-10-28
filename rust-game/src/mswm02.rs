//! Closed PSC society. Costs, once-per-game use and hidden contest share existing machinery.
use crate::{catalog,engine::RuleResult,model::*,rules::*,society::{SocietyDefinition,SocietyDeckConstraint},mystic_contest::MysticBonus};
use serde::Serialize;
pub(crate) const ID:&str="MSWM02";
pub(crate) const CLEAR:&str="mswm02-clear";
pub(crate) const CONTEST:&str="mswm02-media-contest";
fn same<T:Serialize>(a:&T,b:&T)->bool{serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap()}
fn society_target()->TargetSlotSpec{target(Zone::Society,EntityKind::Any,Relation::Any,Range::Anywhere)}
pub(crate) fn definition()->Definition{
 let mut clear=ability(CLEAR,"每局一次：移除目标秘社的一类全部标志",Timing::Standard,vec![Cost::ExhaustSource],vec![],vec![],None);clear.activation_only=true;clear.once_per_game=true;
 clear.modes=[("exposure","移除全部暴露",true),("secrecy","移除全部隐秘",false)].into_iter().map(|(key,label,exposure)|Mode{key:key.into(),label:label.into(),targets:vec![society_target()],ops:vec![Op::MSWM02Clear{exposure}]}).collect();
 let mut contest=ability(CONTEST,"弃媒体或互联网：隐秘对抗，按印刷费用加值",Timing::Standard,vec![Cost::Assets(2),Cost::ExhaustSource,Cost::MSWM02MediaDiscard],vec![],vec![Op::MysticContest{actor_bonus:0}],None);contest.activation_only=true;with_abilities(vec![clear,contest])
}
pub(crate) fn society()->SocietyDefinition{
 let mut card:catalog::CardDefinition=serde_json::from_value(serde_json::json!({"id":ID,"name":"公众安全委员会","kind":"society","type":"秘社/机构","subtypes":["机构"],"color":"灰","society":"公众安全委员会","unique":true,"supported":false,"text":"构筑只允许灰、黄、中立及其他派系特工、媒体、互联网牌。行动横置：移除目标秘社全部暴露或隐秘，每局限一次。行动2、横置、弃一张媒体或互联网牌：隐秘对抗，本人加值等于弃牌印刷费用。"})).unwrap();
 card.abilities=definition().abilities.iter().map(|a|catalog::AbilitySummary{key:a.key.clone(),label:a.label.clone(),timing:"standard".into(),costs:a.costs.clone(),triggered:false}).collect();SocietyDefinition{card,subtitle:"P.S.C".into(),printed_cost:None,starting_hand:6,deck_constraints:vec![SocietyDeckConstraint::GrayYellowNeutralOrPrintedAgentMediaInternet],unresolved_abilities:Default::default()}
}
pub(crate) fn related(a:&AbilitySpec)->bool{a.key.starts_with("mswm02-")||a.costs.iter().any(|c|matches!(c,Cost::MSWM02MediaDiscard))||a.ops.iter().any(|o|matches!(o,Op::MSWM02Clear{..}))}
pub(crate) fn validate_ability(id:&str,a:&AbilitySpec)->RuleResult<()>{let d=definition();let mut programs=d.abilities.clone();for base in d.abilities{for mode in &base.modes{let mut selected=base.clone();selected.targets=mode.targets.clone();selected.ops=mode.ops.clone();selected.modes.clear();programs.push(selected);}}if id!=ID||!programs.iter().any(|b|same(a,b)){return Err("公众安全委员会须保留原费用、限次、模式与对抗程序".into());}Ok(())}
pub(crate) fn validate_definition(id:&str,d:&Definition)->RuleResult<()>{if (id==ID||d.abilities.iter().any(related))&&(id!=ID||!same(d,&definition())){return Err("公众安全委员会定义不能删改或移植".into());}Ok(())}
impl Game {
 pub(crate) fn mswm02_media(&self,c:&Card)->bool{!c.face_down&&self.current_subtypes(c).iter().any(|s|matches!(s.as_str(),"媒体"|"互联网"))}
 pub(crate) fn mswm02_pay(&mut self,actor:usize,s:&SourceSnapshot,a:&AbilitySpec,input:&Action)->RuleResult<PaidCost>{validate_ability(&s.card.definition,a)?;if a.key!=CONTEST{return Err("弃牌仅供原隐秘行动".into());}let ids=input.cost_selected.as_ref().filter(|v|v.len()==1).ok_or("请选择一张媒体或互联网手牌支付")?;let c=self.players[actor].hand.iter().find(|c|c.id==ids[0]&&self.mswm02_media(c)).ok_or("须弃本人媒体或互联网手牌")?.clone();let subtypes=self.current_subtypes(&c);self.discard_hand_card(actor,&c.id)?;Ok(PaidCost::MSWM02Discard{card:c,subtypes})}
 pub(crate) fn mswm02_frame(&self,f:&ResolutionFrame)->RuleResult<()>{
  if f.source.card.definition!=ID&&!f.ability_key.starts_with("mswm02-")&&!f.already_paid.iter().any(|p|matches!(p,PaidCost::MSWM02Discard{..}))&&!f.steps.iter().any(|s|matches!(s.op,Op::MSWM02Clear{..})){return Ok(());}
  self.validate_resource_frame(f)?;
  let s=&f.source;let c=&s.card;
  if c.definition!=ID||f.actor>=self.players.len()||c.owner!=f.actor||c.controller!=f.actor||c.face_down||c.exhausted||!self.mystic_instance_valid(&c.id)||!self.mystic_instance_valid(&f.frame_id)||s.region.is_some()||s.source_region_instance.is_some()||s.play_source.is_some()||s.attachment_host_instance.is_some()||s.death_echo.is_some()||s.observed_death.is_some()||s.observed_exposure.is_some()||s.observed_mobility_grant.is_some()||f.chosen_region.is_some()||f.steps.len()!=1||f.steps[0].context!=f.actor||f.cursor>1||f.cursor==0&&!matches!(f.guard,GuardState::Unchecked)||f.cursor==1&&!matches!(f.guard,GuardState::Accepted){return Err("公众安全委员会帧来源或执行边界损坏".into());}self.mobility_immutable_identity(c)?;
  match f.ability_key.as_str(){
   CLEAR=>{if !matches!(f.steps[0].op,Op::MSWM02Clear{..})||!matches!(f.already_paid.as_slice(),[PaidCost::Exhausted(id)]if id==&c.id)||!f.resource_payments.is_empty()||!self.players[f.actor].society_zone.used_once_per_game.contains(CLEAR){return Err("清标志须保留横置支付及已消耗的每局次数".into());}let [t]=f.targets.as_slice()else{return Err("须保留唯一秘社目标".into());};if t.id.strip_prefix("society:p").and_then(|s|s.parse::<usize>().ok()).is_none_or(|seat|seat>=self.players.len()||t.id!=format!("society:p{seat}"))||!same(&t.spec,&society_target())||t.public.instance_id!=t.id||!t.public.valid||t.public.status!="valid"||t.public.invalid_reason.is_some()||t.region_instance.is_some(){return Err("原秘社目标不符".into());}},
   CONTEST=>{if !matches!(f.steps[0].op,Op::MysticContest{actor_bonus:0})||!f.targets.is_empty(){return Err("原隐秘对抗程序不可改写".into());}let [PaidCost::Assets(ids),PaidCost::Exhausted(source),PaidCost::MSWM02Discard{card,subtypes}]=f.already_paid.as_slice()else{return Err("隐秘对抗缺完整三项支付".into());};if f.money_units(0)!=2||ids.iter().any(|id|!self.mystic_instance_valid(id))||ids.iter().collect::<std::collections::BTreeSet<_>>().len()!=ids.len()||source!=&c.id||card.face_down||card.controller!=f.actor||card.owner>=self.players.len()||!self.mystic_instance_valid(&card.id)||!catalog::card(&card.definition).subtypes.iter().any(|s|matches!(s.as_str(),"媒体"|"互联网"))||!subtypes.iter().any(|s|matches!(s.as_str(),"媒体"|"互联网")){return Err("原媒体弃牌或资产费用不符".into());}self.mobility_immutable_identity(card)?;},
   _=>return Err("未知公众安全委员会程序".into())
  }Ok(())
 }
 pub(crate) fn mswm02_clear(&mut self,f:&ResolutionFrame,exposure:bool)->RuleResult<()>{self.mswm02_frame(f)?;if f.ability_key!=CLEAR||f.cursor!=1{return Err("清标志须消费原第一步".into());}if let Some(seat)=self.society_host_seat(&f.targets[0].id){if exposure{self.players[seat].society_zone.exposure=0;}else{self.players[seat].society_zone.secrecy=0;}}Ok(())}
 pub(crate) fn mswm02_start(&mut self,f:ResolutionFrame)->RuleResult<()>{self.mswm02_frame(&f)?;if f.ability_key!=CONTEST||f.cursor!=1{return Err("隐秘行动须来自原第一步".into());}let PaidCost::MSWM02Discard{card,..}=&f.already_paid[2]else{unreachable!()};let amount=catalog::card(&card.definition).cost;self.mystic_contest_start(f.clone(),0)?;self.mystic_add_bonus(f,amount)}
 pub(crate) fn mswm02_bonus(&self,b:&MysticBonus)->RuleResult<()>{self.mswm02_frame(&b.frame)?;if b.frame.ability_key!=CONTEST||b.frame.cursor!=1||!self.mystic_instance_valid(&b.contest_id){return Err("媒体加值缺原已结算对抗行动".into());}let PaidCost::MSWM02Discard{card,..}=&b.frame.already_paid[2]else{return Err("缺支付卡".into());};if b.amount!=catalog::card(&card.definition).cost{return Err("媒体加值只能等于原弃牌印刷费".into());}Ok(())}
 fn mswm02_contest_receipt(&self,frame_id:&str,actor:usize,source:&SourceSnapshot,key:&str,bonuses:&[MysticBonus])->RuleResult<()>{let own=bonuses.iter().filter(|b|b.frame.source.card.definition==ID).collect::<Vec<_>>();if source.card.definition==ID{if key!=CONTEST||own.len()!=1||own[0].frame.frame_id!=frame_id||own[0].frame.actor!=actor||!same(&own[0].frame.source,source){return Err("公众安全对抗须唯一保留自身原弃牌加值".into());}}else if !own.is_empty(){return Err("公众安全加值不能移植其他对抗".into());}for b in own{self.mswm02_bonus(b)?;}Ok(())}
 pub(crate) fn validate_mswm02_state(&self)->RuleResult<()>{
  for e in &self.effects{match e{Effect::Frame{frame}=>self.mswm02_frame(frame)?,Effect::Declare{declaration:d}if d.source.card.definition==ID||related(&d.ability)=>return Err("公众安全只有主动行动".into()),_=>{}}}
  for item in &self.stack{if let Some(f)=&item.frame{self.mswm02_frame(f)?;if f.source.card.definition==ID&&(item.card.is_some()||item.id!=f.frame_id||item.controller!=f.actor||item.reveal||item.deploy_region.is_some()||item.target.as_deref()!=f.targets.first().map(|t|t.id.as_str())||f.cursor!=0){return Err("公众安全堆叠外壳损坏".into());}}}
  if let Some(p)=&self.pending{match &p.resolution{ChoiceResolution::Frame{frame:f,choice}=>{self.mswm02_frame(f)?;if f.source.card.definition==ID&&!matches!(choice,FrameChoice::MysticCommit{..}|FrameChoice::MysticReward{..}){return Err("公众安全只能使用原隐秘待选".into());}},ChoiceResolution::Declare{declaration:d,..}if d.source.card.definition==ID||related(&d.ability)=>return Err("公众安全不产生触发声明".into()),_=>{}}}
  for state in self.mystic_states(){self.mswm02_frame(&state.frame)?;self.mswm02_contest_receipt(&state.frame.frame_id,state.frame.actor,&state.frame.source,&state.frame.ability_key,&state.bonuses)?;}
  for o in &self.mystic_history{self.mswm02_contest_receipt(&o.frame_id,o.actor,&o.source,&o.ability_key,&o.bonuses)?;}
  Ok(())
 }
}
