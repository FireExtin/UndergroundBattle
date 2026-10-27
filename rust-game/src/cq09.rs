//! CQ09 uses the shared contest history and the existing once-per-turn end boundary.
use crate::{catalog,engine::RuleResult,model::*,rules::*};
use serde::{Serialize,Deserialize};
use std::collections::BTreeSet;
pub(crate) const ID:&str="CQ09";
pub(crate) const REVEAL:&str="cq09-reveal-contest";
pub(crate) const END:&str="cq09-end-grave-return";
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EndObservation{pub turn:u32,pub event_id:String,pub winning_contest:String}
fn same<T:Serialize>(a:&T,b:&T)->bool{serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap()}
fn owns(ops:&[Op])->bool{ops.iter().any(|o|match o{Op::CQ09EndReturn{..}=>true,Op::ForEachLivingPlayer(v)|Op::ForEachLivingPlayerFromActor(v)=>owns(v),Op::IfTargetExhausted{exhausted,ready,..}=>owns(std::slice::from_ref(exhausted))||owns(std::slice::from_ref(ready)),_=>false})}
pub(crate) fn related(a:&AbilitySpec)->bool{a.key.starts_with("cq09-")||a.event==Some(Event::CQ09EndStep)||owns(&a.ops)||a.modes.iter().any(|m|owns(&m.ops))||a.targets.iter().any(|t|t.predicate==Some(TargetPredicate::CQ09OwnGraveCharacter))}
pub(crate) fn frame_related(f:&ResolutionFrame)->bool{f.ability_key.starts_with("cq09-")||f.steps.iter().any(|s|owns(std::slice::from_ref(&s.op)))}
pub(crate) fn declaration_event(d:&Declaration)->Option<&str>{match d.ability.ops.as_slice(){[Op::CQ09EndReturn{observation:Some(o)}]=>Some(&o.event_id),_=>None}}
pub(crate) fn definition()->Definition{
 let mut t=target(Zone::Graveyard,EntityKind::Character,Relation::OwnedByActor,Range::Anywhere);t.predicate=Some(TargetPredicate::CQ09OwnGraveCharacter);
 with_abilities(vec![ability(REVEAL,"现身：隐秘对抗，本人展示吸血鬼、法师或魔宠+3",Timing::Fast,vec![],vec![],vec![Op::MysticContest{actor_bonus:0}],Some(Event::Reveal)),
  ability(END,"结束：本回合曾获胜，将本人墓地费用不超过3的吸血鬼、法师或魔宠置于本地区",Timing::Fast,vec![],vec![t],vec![Op::CQ09EndReturn{observation:None}],Some(Event::CQ09EndStep))])
}
pub(crate) fn validate_ability(id:&str,a:&AbilitySpec)->RuleResult<()>{
 let mut expected=definition().abilities.into_iter().find(|v|v.key==a.key).ok_or("CQ09没有该印刷能力")?;
 if let [Op::CQ09EndReturn{observation}]=a.ops.as_slice(){expected.ops=vec![Op::CQ09EndReturn{observation:observation.clone()}];}
 if id!=ID||!same(a,&expected){return Err("CQ09只能保留完整原规则和范围".into());}Ok(())
}
pub(crate) fn validate_definition(id:&str,d:&Definition)->RuleResult<()>{if (id==ID||d.abilities.iter().any(related))&&(id!=ID||!same(d,&definition())){return Err("CQ09完整两条规则不可增删或移植".into());}Ok(())}
pub(crate) fn validate_action(id:&str,a:&Action)->RuleResult<()>{if id!=ID{return Ok(());}let expected=match a.kind.as_str(){"deploy"|"conceal"=>Action{card_id:a.card_id.clone(),region:a.region,..Action::new(&a.kind)},"reveal"=>Action{card_id:a.card_id.clone(),..Action::new("reveal")},_=>return Err("CQ09不能主动发动印刷触发".into())};if *a!=expected{return Err("CQ09角色操作不能夹带目标或对抗加值".into());}Ok(())}
pub(crate) fn validate_choice(p:&Pending,a:&Action)->RuleResult<()>{let ours=match &p.resolution{ChoiceResolution::Declare{declaration,..}=>related(&declaration.ability),ChoiceResolution::Frame{frame,..}=>frame_related(frame),_=>false};if ours&&*a!=(Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")}){return Err("CQ09选择不能夹带额外规则载荷".into());}Ok(())}
impl Game{
 pub(crate) fn validate_cq09_play(&self,item:&StackItem)->RuleResult<()>{
  if !item.card.as_ref().is_some_and(|c|c.definition==ID)&&!item.frame.as_ref().is_some_and(|f|f.source.card.definition==ID&&matches!(f.ability_key.as_str(),"deploy"|"reveal")){return Ok(());}
  let f=item.frame.as_ref().ok_or("CQ09角色事务缺少原帧")?;let c=&f.source.card;
  if c.definition!=ID||f.actor>=self.players.len()||c.owner>=self.players.len()||c.controller!=f.actor||!self.mystic_instance_valid(&c.id)||!self.mystic_instance_valid(&f.frame_id)
   ||f.source.region.is_none_or(|r|r>=self.regions.len())||f.source.source_region_instance.as_deref().is_none_or(|id|!self.mystic_instance_valid(id))
   ||f.source.attachment_host_instance.is_some()||f.source.death_echo.is_some()||f.source.observed_death.is_some()||f.source.observed_exposure.is_some()||f.source.observed_mobility_grant.is_some()
   ||f.cursor!=0||!matches!(f.guard,GuardState::Unchecked)||!f.steps.is_empty()||!f.targets.is_empty()||f.chosen_region!=f.source.region
   ||!matches!(f.ability_key.as_str(),"deploy"|"reveal")||f.ability_key=="deploy"&&(c.face_down||!f.source.valid_character_play())||f.ability_key=="reveal"&&(!c.face_down||f.source.play_source.is_some())
   ||!matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)] if f.money_units(0)<=6&&ids.iter().all(|id|self.mystic_instance_valid(id)&&*id!=c.id)&&ids.iter().collect::<BTreeSet<_>>().len()==ids.len())
   ||item.id!=f.frame_id||item.controller!=f.actor||item.target.is_some()||item.reveal!=(f.ability_key=="reveal")||item.deploy_region!=f.source.region
   ||item.card.as_ref().is_none_or(|c|!self.pack_a_transaction_matches(f,c)){return Err("CQ09角色事务、原帧或实际牌面不符".into());}self.mobility_immutable_identity(c)
 }

 pub(crate) fn cq09_bonus(&self,origin:&SourceSnapshot,key:&str,actor:usize,seat:usize,shown:&Card,vampire:bool)->u32{
  let d=catalog::card(&shown.definition);
  u32::from(origin.card.definition==ID&&key==REVEAL&&actor==seat&&(vampire||d.subtypes.iter().any(|s|s=="法师"||s=="魔宠")))*3
 }
 pub(crate) fn cq09_grave_target(&self,actor:usize,source:&SourceSnapshot,c:&Card)->bool{
  let d=catalog::card(&c.definition);source.card.definition==ID&&c.owner==actor&&!c.face_down&&d.kind=="character"&&d.cost<=3&&d.subtypes.iter().any(|s|matches!(s.as_str(),"吸血鬼"|"法师"|"魔宠"))&&self.players.get(actor).is_some_and(|p|p.graveyard.iter().any(|v|v==c))
 }
 fn cq09_win(&self,actor:usize)->Option<String>{self.mystic_history.iter().find(|o|o.turn==self.turn&&o.winner_team==Some(self.team(actor))&&o.participants.contains(&actor)).map(|o|o.contest_id.clone())}
 pub(crate) fn cq09_end_available(&self)->bool{self.regions.iter().filter(|r|!r.vacant).flat_map(|r|&r.cards).any(|c|c.definition==ID&&!c.face_down&&!self.players[c.controller].eliminated&&self.cq09_win(c.controller).is_some())}
 pub(crate) fn cq09_begin_end(&mut self,event_id:&str){
  let sources=self.regions.iter().enumerate().filter(|(_,r)|!r.vacant).flat_map(|(r,v)|v.cards.iter().map(move|c|(r,c))).filter(|(_,c)|c.definition==ID&&!c.face_down&&!self.players[c.controller].eliminated).filter_map(|(r,c)|self.cq09_win(c.controller).map(|win|(self.source_snapshot(c,Some(r)),win))).collect::<Vec<_>>();
  for(source,win)in sources{let mut a=definition().abilities.remove(1);a.ops=vec![Op::CQ09EndReturn{observation:Some(EndObservation{turn:self.turn,event_id:event_id.into(),winning_contest:win})}];self.effects.push_back(Effect::Declare{declaration:Declaration{actor:source.card.controller,source,ability:a}});}
 }
 fn cq09_source(&self,actor:usize,s:&SourceSnapshot)->RuleResult<()>{let c=&s.card;
  if actor>=self.players.len()||c.owner>=self.players.len()||c.controller!=actor||c.definition!=ID||c.face_down||!self.mystic_instance_valid(&c.id)||s.region.is_none_or(|r|r>=self.regions.len())||s.source_region_instance.as_deref().is_none_or(|id|!self.mystic_instance_valid(id)||id==c.id)||s.play_source.is_some()||s.attachment_host_instance.is_some()||s.death_echo.is_some()||s.observed_death.is_some()||s.observed_exposure.is_some()||s.observed_mobility_grant.is_some(){return Err("CQ09触发来源和原地区无效".into());}self.mobility_immutable_identity(c)
 }
 fn cq09_observation(&self,actor:usize,s:&SourceSnapshot,ops:&[Op])->RuleResult<()>{
  let [Op::CQ09EndReturn{observation:Some(o)}]=ops else{return Err("CQ09结束触发缺少真实获胜观察".into());};
  if o.turn!=self.turn||self.jc054_end_step_turn!=Some(self.turn)||!self.mystic_instance_number(&o.event_id).zip(self.mystic_instance_number(&s.card.id)).is_some_and(|(event,source)|event>source)||!self.mystic_instance_number(&o.event_id).zip(self.mystic_instance_number(&o.winning_contest)).is_some_and(|(event,contest)|event>contest)||self.mystic_history.iter().find(|v|v.contest_id==o.winning_contest).is_none_or(|v|v.turn!=o.turn||v.winner_team!=Some(self.team(actor))||!v.participants.contains(&actor)) {return Err("CQ09结束观察必须属于当回合本人真实获胜".into());}Ok(())
 }
 pub(crate) fn validate_cq09_declaration(&self,d:&Declaration)->RuleResult<()>{if !related(&d.ability){return Ok(());}self.cq09_source(d.actor,&d.source)?;validate_ability(&d.source.card.definition,&d.ability)?;if d.ability.key==END{self.cq09_observation(d.actor,&d.source,&d.ability.ops)?;}Ok(())}
 pub(crate) fn validate_cq09_frame(&self,f:&ResolutionFrame)->RuleResult<()>{
  if !frame_related(f){return Ok(());}self.cq09_source(f.actor,&f.source)?;
  let mut a=definition().abilities.into_iter().find(|a|a.key==f.ability_key).ok_or("CQ09帧能力不存在")?;
  if f.ability_key==END{let ops=f.steps.iter().map(|s|s.op.clone()).collect::<Vec<_>>();self.cq09_observation(f.actor,&f.source,&ops)?;a.ops=ops;}
  if !self.mystic_instance_valid(&f.frame_id)||f.frame_id==f.source.card.id||f.cursor>1||f.cursor==0&&!matches!(f.guard,GuardState::Unchecked)||f.cursor==1&&!matches!(f.guard,GuardState::Accepted)||!f.already_paid.is_empty()||!f.resource_payments.is_empty()||f.chosen_region.is_some()||!same(&f.steps,&a.ops.iter().map(|op|Step{context:f.actor,op:op.clone()}).collect::<Vec<_>>())||f.targets.len()!=a.targets.len(){return Err("CQ09原程序、响应阶段或费用被改写".into());}
  if let Some(t)=f.targets.first(){if !same(&t.spec,&a.targets[0])||!self.mystic_instance_valid(&t.id)||t.public.instance_id!=t.id||t.public.kind!="character"||t.public.owner!=Some(player_id(f.actor))||t.public.controller!=Some(player_id(f.actor))||t.public.region.is_some()||t.region_instance.is_some()||!t.public.valid||t.public.status!="valid"||t.public.invalid_reason.is_some(){return Err("CQ09原个人墓地角色目标被改写".into());}}Ok(())
 }
 pub(crate) fn cq09_return(&mut self,f:&ResolutionFrame)->RuleResult<()>{self.validate_cq09_frame(f)?;if f.cursor!=1||f.ability_key!=END{return Err("CQ09复归只能沿原结束帧".into());}let r=f.source.region.unwrap();if self.region_live(r)&&f.source.source_region_instance.as_ref()==Some(&self.regions[r].card.id)&&self.players[f.actor].graveyard.iter().any(|c|c.id==f.targets[0].id){self.world_graveyard_entry(f.actor,r,&f.targets[0].id)?;}Ok(())}
 pub(crate) fn validate_cq09_state(&self)->RuleResult<()>{
  let mut frames=BTreeSet::new();let mut events=BTreeSet::new();
  let mut remember=|s:&SourceSnapshot,ops:&[Op]|->RuleResult<()>{if let [Op::CQ09EndReturn{observation:Some(o)}]=ops{if !events.insert(s.card.id.clone()){return Err("CQ09原结束观察重复消费".into());}}Ok(())};
  for item in &self.stack{self.validate_cq09_play(item)?;if let Some(f)=&item.frame{if frame_related(f){self.validate_cq09_frame(f)?;remember(&f.source,&f.steps.iter().map(|s|s.op.clone()).collect::<Vec<_>>())?;if !frames.insert(&f.frame_id)||item.id!=f.frame_id||item.controller!=f.actor||item.card.is_some()||item.reveal||item.deploy_region.is_some()||item.target!=f.targets.first().map(|t|t.id.clone())||f.cursor!=0{return Err("CQ09响应外壳不符或重复".into());}}}}
  for e in &self.effects{match e{Effect::Declare{declaration:d} if related(&d.ability)=>{self.validate_cq09_declaration(d)?;remember(&d.source,&d.ability.ops)?;},Effect::Frame{frame} if frame_related(frame)=>return Err("CQ09不能伪造独立执行续体".into()),_=>()}}
  if let Some(p)=&self.pending{match &p.resolution{
   ChoiceResolution::Declare{declaration:d,stage} if related(&d.ability)=>{
    self.validate_cq09_declaration(d)?;remember(&d.source,&d.ability.ops)?;
    let (options,valid_stage)=if d.ability.key==END{(self.binding_options(d.actor,&d.source,&d.ability.targets[0]),matches!(stage,DeclareChoice::Target))}else{(vec![ChoiceOption{id:"accept".into(),label:"发动触发能力".into(),card:None}],matches!(stage,DeclareChoice::Accept))};
    let expected=Choice{id:p.choice.id.clone(),kind:"trigger".into(),title:format!("{}：{}",crate::batch3_trap::source_label(&d.source),d.ability.label),description:"选择符合数量限制的选项，然后确认。".into(),player_id:player_id(d.actor),options,preview_cards:vec![],min:Some(0),max:Some(1),amount:None,allow_decline:Some(true)};
    if !valid_stage||!self.mystic_instance_valid(&p.choice.id)||p.seat!=d.actor||expected.options.is_empty()||!same(&p.choice,&expected){return Err("CQ09触发待选被改写".into());}
   },
   ChoiceResolution::Frame{frame,choice} if frame_related(frame)=>{self.validate_cq09_frame(frame)?;if frame.ability_key!=REVEAL||!matches!(choice,FrameChoice::MysticCommit{..}|FrameChoice::MysticReward{..}){return Err("CQ09仅允许真实隐秘对抗待选".into());}},_=>()
  }}Ok(())
 }
}
