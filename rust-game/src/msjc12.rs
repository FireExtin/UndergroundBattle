//! Closed MSJC12: printed construction, live Human grant and paid three-character actions.
use crate::{catalog,engine::RuleResult,model::*,rules::*,society::{SocietyDefinition,SocietyDeckConstraint}};
use serde::{Serialize,Deserialize};
use std::collections::BTreeSet;
pub(crate) const ID:&str="MSJC12";
pub(crate) const DRAW:&str="msjc12-slave-draw";
pub(crate) const DISCARD:&str="msjc12-vampire-discard";
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExhaustedCharacter { pub card:Card, pub subtypes:Vec<String> }
fn same<T:Serialize>(a:&T,b:&T)->bool {serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap()}
fn required(key:&str)->Option<&'static str> {match key {DRAW=>Some("奴仆"),DISCARD=>Some("吸血鬼"),_=>None}}
pub(crate) fn definition()->Definition {
 let mut draw=ability(DRAW,"横置三个本方奴仆：抓一张牌",Timing::Standard,vec![Cost::Assets(1),Cost::ExhaustSource,Cost::MSJC12ExhaustTrio],vec![],vec![Op::Draw{player:PlayerRef::Actor,count:1,end:DeckEnd::Top}],None);
 let mut discard=ability(DISCARD,"横置三个本方吸血鬼：目标敌方玩家弃一张牌",Timing::Standard,vec![Cost::Assets(2),Cost::ExhaustSource,Cost::MSJC12ExhaustTrio],vec![target(Zone::Player,EntityKind::Any,Relation::EnemyTeam,Range::Anywhere)],vec![Op::Discard{player:PlayerRef::Target(0),count:Some(1),redraw:false,optional:false}],None);
 draw.activation_only=true;discard.activation_only=true;with_abilities(vec![draw,discard])
}
pub(crate) fn society()->SocietyDefinition {
 let mut card:catalog::CardDefinition=serde_json::from_value(serde_json::json!({"id":ID,"name":"爱弗罗德家族","kind":"society","type":"秘社/家族/吸血鬼","subtypes":["家族","吸血鬼"],"color":"蓝","society":"爱弗罗德家族","unique":true,"supported":false,"text":"构筑仅蓝、中立及其他派系吸血鬼或奴仆角色。所有印刷费用小于或等于2的本方人类角色额外具有奴仆类别。行动1，横置秘社及三个本方奴仆：抓一张牌。行动2，横置秘社及三个本方吸血鬼：目标敌方玩家弃一张牌。"})).unwrap();
 card.abilities=definition().abilities.iter().map(|a|catalog::AbilitySummary{key:a.key.clone(),label:a.label.clone(),timing:"standard".into(),costs:a.costs.clone(),triggered:false}).collect();
 SocietyDefinition{card,subtitle:"黑色家族".into(),printed_cost:None,starting_hand:6,deck_constraints:vec![SocietyDeckConstraint::BlueNeutralOrPrintedVampireSlave],unresolved_abilities:Default::default()}
}
pub(crate) fn related(a:&AbilitySpec)->bool {a.key.starts_with("msjc12-")||a.costs.iter().any(|c|matches!(c,Cost::MSJC12ExhaustTrio))}
pub(crate) fn validate_ability(id:&str,a:&AbilitySpec)->RuleResult<()> {if id!=ID||!definition().abilities.iter().any(|b|same(a,b)){return Err("爱弗罗德原费用、时点、目标和程序不可改写".into());}Ok(())}
pub(crate) fn validate_definition(id:&str,d:&Definition)->RuleResult<()> {if (id==ID||d.abilities.iter().any(related))&&(id!=ID||!same(d,&definition())) {return Err("爱弗罗德完整定义不可增删或移植".into());}Ok(())}
impl Game {
 pub(crate) fn msjc12_grants_slave(&self,c:&Card)->bool {
  !c.face_down&&catalog::card(&c.definition).kind=="character"&&catalog::card(&c.definition).cost<=2
   &&self.regions.iter().any(|r|!r.vacant&&r.cards.iter().any(|live|live.id==c.id))
   &&self.players.get(c.controller).is_some_and(|p|!p.eliminated&&p.society_zone.card.as_ref().is_some_and(|s|s.definition==ID&&!s.face_down))
 }
 fn msjc12_candidates(&self,actor:usize,key:&str)->Vec<&Card> {
  let Some(subtype)=required(key)else{return vec![];};
  self.regions.iter().filter(|r|!r.vacant).flat_map(|r|&r.cards).filter(|c|!c.face_down&&!c.exhausted&&c.controller==actor&&catalog::card(&c.definition).kind=="character"&&self.current_subtypes(c).iter().any(|s|s==subtype)).collect()
 }
 pub(crate) fn msjc12_trios(&self,actor:usize,key:&str)->Vec<Vec<String>> {
  let cards=self.msjc12_candidates(actor,key);let mut out=vec![];
  for i in 0..cards.len(){for j in i+1..cards.len(){for k in j+1..cards.len(){out.push(vec![cards[i].id.clone(),cards[j].id.clone(),cards[k].id.clone()]);}}}out
 }
 pub(crate) fn msjc12_pay_trio(&mut self,actor:usize,s:&SourceSnapshot,spec:&AbilitySpec,a:&Action)->RuleResult<PaidCost> {
  validate_ability(&s.card.definition,spec)?;
  if self.players.get(actor).and_then(|p|p.society_zone.card.as_ref()).is_none_or(|c|c.id!=s.card.id){return Err("爱弗罗德费用须来自本方真实秘社".into());}
  let ids=a.cost_selected.as_ref().filter(|ids|ids.len()==3&&ids.iter().collect::<BTreeSet<_>>().len()==3).ok_or("须横置三个不同的本方角色")?;
  let candidates=self.msjc12_candidates(actor,&spec.key);
  let mut receipt=vec![];for id in ids {let c=candidates.iter().find(|c|c.id==*id).ok_or("三角色费用须均未横置且符合本方类别")?;receipt.push(ExhaustedCharacter{card:(*c).clone(),subtypes:self.current_subtypes(c)});}
  for id in ids {self.board_mut(id).ok_or("费用角色已离场")?.exhausted=true;}
  Ok(PaidCost::MSJC12Trio(receipt))
 }
 fn msjc12_frame(&self,f:&ResolutionFrame)->RuleResult<()> {
  if f.source.card.definition!=ID&&!f.ability_key.starts_with("msjc12-")&&!f.already_paid.iter().any(|p|matches!(p,PaidCost::MSJC12Trio(_))){return Ok(());}
  let a=definition().abilities.into_iter().find(|a|a.key==f.ability_key).ok_or("爱弗罗德未知能力帧")?;
  let s=&f.source;let c=&s.card;
  if f.actor>=self.players.len()||c.definition!=ID||c.owner!=f.actor||c.controller!=f.actor||c.face_down||c.exhausted||!self.mystic_instance_valid(&c.id)||!self.mystic_instance_valid(&f.frame_id)||s.region.is_some()||s.source_region_instance.is_some()||s.play_source.is_some()||s.attachment_host_instance.is_some()||s.death_echo.is_some()||s.observed_death.is_some()||s.observed_exposure.is_some()||s.observed_mobility_grant.is_some()||f.chosen_region.is_some()||f.cursor>1||f.cursor==0&&!matches!(f.guard,GuardState::Unchecked)||f.cursor==1&&!matches!(f.guard,GuardState::Accepted)||f.steps.len()!=1||f.steps[0].context!=f.actor||!same(&f.steps[0].op,&a.ops[0]){return Err("爱弗罗德帧来源或执行程序损坏".into());}
  self.mobility_immutable_identity(c)?;
  if f.targets.len()!=a.targets.len(){return Err("爱弗罗德目标数量不符".into());}
  if let Some(t)=f.targets.first(){let seat=t.id.strip_prefix('p').and_then(|s|s.parse::<usize>().ok()).filter(|s|*s<self.players.len()).ok_or("弃牌目标须为玩家")?;
   if self.team(seat)==self.team(f.actor)||!same(&t.spec,&a.targets[0])||t.public.instance_id!=t.id||t.region_instance.is_some()||!t.public.valid||t.public.status!="valid"||t.public.invalid_reason.is_some(){return Err("弃牌原目标须为敌方玩家".into());}}
  let [PaidCost::Assets(ids),PaidCost::Exhausted(source),PaidCost::MSJC12Trio(cards)]=f.already_paid.as_slice()else{return Err("爱弗罗德缺完整三项费用".into());};
  let n=if f.ability_key==DRAW{1}else{2};
  if f.money_units(0)!=n||ids.iter().any(|id|!self.mystic_instance_valid(id))||ids.iter().collect::<BTreeSet<_>>().len()!=ids.len()||source!=&c.id||cards.len()!=3||cards.iter().map(|p|&p.card.id).collect::<BTreeSet<_>>().len()!=3 {return Err("爱弗罗德费用数目或实例重复".into());}
  for p in cards {let c=&p.card;if c.controller!=f.actor||c.owner>=self.players.len()||c.face_down||c.exhausted||c.id==s.card.id||!self.mystic_instance_valid(&c.id)||catalog::card(&c.definition).kind!="character"||!p.subtypes.iter().any(|s|Some(s.as_str())==required(&f.ability_key))||p.subtypes.iter().collect::<BTreeSet<_>>().len()!=p.subtypes.len(){return Err("三横置费用原角色或支付时类别损坏".into());}self.mobility_immutable_identity(c)?;}
  Ok(())
 }
 pub(crate) fn validate_msjc12_state(&self)->RuleResult<()> {
  for e in &self.effects {match e {Effect::Frame{frame}=>self.msjc12_frame(frame)?,Effect::Declare{declaration:d}if d.source.card.definition==ID||related(&d.ability)=>return Err("爱弗罗德行动不能改为触发声明".into()),_=>{}}}
  for s in &self.stack {if let Some(f)=&s.frame {self.msjc12_frame(f)?;if f.source.card.definition==ID&&(s.card.is_some()||s.id!=f.frame_id||s.controller!=f.actor||s.reveal||s.deploy_region.is_some()||s.target.as_deref()!=f.targets.first().map(|t|t.id.as_str())||f.cursor!=0){return Err("爱弗罗德堆叠外壳损坏".into());}}}
  if let Some(p)=&self.pending {match &p.resolution {
   ChoiceResolution::Declare{declaration:d,..}if d.source.card.definition==ID||related(&d.ability)=>return Err("爱弗罗德无触发声明待选".into()),
   ChoiceResolution::Frame{frame:f,choice}=>{self.msjc12_frame(f)?;if f.source.card.definition==ID {
    let FrameChoice::Discard{seat,redraw:false}=choice else{return Err("爱弗罗德只能产生原弃牌续体".into());};
    if *seat>=self.players.len()||f.ability_key!=DISCARD||f.cursor!=1||f.targets[0].id!=player_id(*seat)||p.seat!=*seat||p.choice.player_id!=player_id(*seat)||p.choice.kind!="discard"||p.choice.title!="弃 1 张手牌"||p.choice.min!=Some(1)||p.choice.max!=Some(1)||p.choice.allow_decline!=Some(false)||p.choice.amount.is_some()||!p.choice.preview_cards.is_empty()||!self.mystic_instance_valid(&p.choice.id)||self.players[*seat].hand.is_empty(){return Err("爱弗罗德弃牌选择边界损坏".into());}
    let expected=self.players[*seat].hand.iter().map(|c|self.option(c,*seat,None,None)).collect::<Vec<_>>();if !same(&p.choice.options,&expected){return Err("爱弗罗德弃牌必须读取敌方实际手牌".into());}
   }},_=>{}}}
  Ok(())
 }
}
