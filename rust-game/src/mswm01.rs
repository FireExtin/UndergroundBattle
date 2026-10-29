//! Closed Manniu family: owner-bound whole-game type grant, live reveal bonus,
//! and a paid Specter action whose exposure condition is checked on resolution.
use crate::{catalog,engine::RuleResult,model::*,rules::*,society::{SocietyDefinition,SocietyDeckConstraint},mystic_contest::MysticShown};
use serde::{Serialize,Deserialize};
use std::collections::BTreeSet;
pub(crate) const ID:&str="MSWM01";
pub(crate) const GRANT:&str="mswm01-specter-grant";
pub(crate) const DRAW:&str="mswm01-specter-draw";
fn same<T:Serialize>(a:&T,b:&T)->bool{serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap()}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShownWitness { pub society:Card, pub grant:Option<Box<ResolutionFrame>> }
pub(crate) fn definition()->Definition {
 let mut grant=ability(GRANT,"本局拥有的吸血鬼和法师角色额外具有阴魂",Timing::Standard,vec![Cost::Assets(4),Cost::ExhaustSource],vec![],vec![Op::MSWM01{grant:true}],None);
 let mut draw=ability(DRAW,"横置本方阴魂：若秘社没有暴露，抓一张牌",Timing::Standard,vec![Cost::ExhaustSource,Cost::MSWM01ExhaustSpecter],vec![],vec![Op::MSWM01{grant:false}],None);
 grant.activation_only=true;draw.activation_only=true;with_abilities(vec![grant,draw])
}
pub(crate) fn society()->SocietyDefinition {
 let mut card:catalog::CardDefinition=serde_json::from_value(serde_json::json!({"id":ID,"name":"曼纽家族","kind":"society","type":"秘社/家族/吸血鬼","subtypes":["家族","吸血鬼"],"color":"蓝","society":"曼纽家族","unique":true,"supported":false,"text":"构筑仅蓝、黄、中立及其他派系吸血鬼或阴魂角色。隐秘对抗中你展示的阴魂牌获得+1。行动4、横置：本局所有你拥有的吸血鬼和法师角色额外具有阴魂类别。行动横置、横置一个本方阴魂角色：若本方秘社没有暴露，抓一张牌。"})).unwrap();
 card.abilities=definition().abilities.iter().map(|a|catalog::AbilitySummary{key:a.key.clone(),label:a.label.clone(),timing:"standard".into(),costs:a.costs.clone(),triggered:false}).collect();
 SocietyDefinition{card,subtitle:"不死阴魂".into(),printed_cost:None,starting_hand:6,deck_constraints:vec![SocietyDeckConstraint::BlueYellowNeutralOrPrintedVampireSpecter],unresolved_abilities:Default::default()}
}
pub(crate) fn related(a:&AbilitySpec)->bool{a.key.starts_with("mswm01-")||a.costs.iter().any(|c|matches!(c,Cost::MSWM01ExhaustSpecter))||a.ops.iter().any(|o|matches!(o,Op::MSWM01{..}))}
pub(crate) fn validate_ability(id:&str,a:&AbilitySpec)->RuleResult<()>{if id!=ID||!definition().abilities.iter().any(|b|same(a,b)){return Err("曼纽须保留原费用、时点和程序".into());}Ok(())}
pub(crate) fn validate_definition(id:&str,d:&Definition)->RuleResult<()>{if (id==ID||d.abilities.iter().any(related))&&(id!=ID||!same(d,&definition())){return Err("曼纽完整定义不可改写或移植".into());}Ok(())}
impl Game {
 pub(crate) fn mswm01_action(&self,actor:usize,a:&Action)->RuleResult<()> {
  if a.kind!="activate"||!self.players[actor].society_zone.card.as_ref().is_some_and(|c|c.definition==ID&&a.card_id.as_deref()==Some(&c.id)){return Ok(());}
  let key=a.ability_id.as_deref().ok_or("须指定曼纽行动")?;
  if !matches!(key,GRANT|DRAW)||*a!=(Action{card_id:a.card_id.clone(),ability_id:a.ability_id.clone(),cost_selected:if key==DRAW{a.cost_selected.clone()}else{None},..Action::new("activate")}){return Err("曼纽只声明原能力及原阴魂费用".into());}Ok(())
 }
 pub(crate) fn mswm01_candidates(&self,actor:usize)->Vec<&Card>{self.regions.iter().filter(|r|!r.vacant).flat_map(|r|&r.cards).filter(|c|!c.face_down&&!c.exhausted&&c.controller==actor&&catalog::card(&c.definition).kind=="character"&&self.current_subtypes(c).iter().any(|s|s=="阴魂")).collect()}
 pub(crate) fn mswm01_pay(&mut self,actor:usize,s:&SourceSnapshot,a:&AbilitySpec,input:&Action)->RuleResult<PaidCost>{
  validate_ability(&s.card.definition,a)?;if a.key!=DRAW{return Err("阴魂费用仅供原抓牌能力".into());}
  let ids=input.cost_selected.as_ref().filter(|v|v.len()==1).ok_or("须横置一个本方阴魂角色")?;
  let c=(*self.mswm01_candidates(actor).iter().find(|c|c.id==ids[0]).ok_or("费用须为本方未横置明置阴魂角色")?).clone();
  let receipt=crate::msjc12::ExhaustedCharacter{subtypes:self.current_subtypes(&c),card:c};self.board_mut(&ids[0]).ok_or("阴魂已离场")?.exhausted=true;Ok(PaidCost::MSWM01Specter(receipt))
 }
 pub(crate) fn mswm01_frame(&self,f:&ResolutionFrame)->RuleResult<()> {
  if f.source.card.definition!=ID&&!f.ability_key.starts_with("mswm01-")&&!f.steps.iter().any(|s|matches!(s.op,Op::MSWM01{..}))&&!f.already_paid.iter().any(|p|matches!(p,PaidCost::MSWM01Specter(_))){return Ok(());}
  let a=definition().abilities.into_iter().find(|a|a.key==f.ability_key).ok_or("未知曼纽能力")?;let s=&f.source;let c=&s.card;
  if f.actor>=self.players.len()||c.definition!=ID||c.owner!=f.actor||c.controller!=f.actor||c.face_down||c.exhausted||!self.mystic_instance_valid(&c.id)||!self.mystic_instance_valid(&f.frame_id)||s.region.is_some()||s.source_region_instance.is_some()||s.play_source.is_some()||s.attachment_host_instance.is_some()||s.death_echo.is_some()||s.observed_death.is_some()||s.observed_exposure.is_some()||s.observed_mobility_grant.is_some()||f.chosen_region.is_some()||!f.targets.is_empty()||f.cursor>1||f.cursor==0&&!matches!(f.guard,GuardState::Unchecked)||f.cursor==1&&!matches!(f.guard,GuardState::Accepted)||f.steps.len()!=1||f.steps[0].context!=f.actor||!same(&f.steps[0].op,&a.ops[0]){return Err("曼纽来源、目标或程序损坏".into());}
  self.mobility_immutable_identity(c)?;self.validate_resource_frame(f)?;
  match f.ability_key.as_str(){
   GRANT=>{let [PaidCost::Assets(ids),PaidCost::Exhausted(id)]=f.already_paid.as_slice()else{return Err("曼纽授予缺原四资产与横置费用".into());};if f.money_units(0)!=4||id!=&c.id||ids.iter().any(|id|!self.mystic_instance_valid(id))||ids.iter().collect::<BTreeSet<_>>().len()!=ids.len(){return Err("曼纽授予支付损坏".into());}},
   DRAW=>{let [PaidCost::Exhausted(id),PaidCost::MSWM01Specter(p)]=f.already_paid.as_slice()else{return Err("曼纽抓牌缺秘社及阴魂横置费用".into());};let v=&p.card;if id!=&c.id||!f.resource_payments.is_empty()||v.id==c.id||v.controller!=f.actor||v.owner>=self.players.len()||v.face_down||v.exhausted||!self.mystic_instance_valid(&v.id)||catalog::card(&v.definition).kind!="character"||!p.subtypes.iter().any(|s|s=="阴魂")||p.subtypes.iter().collect::<BTreeSet<_>>().len()!=p.subtypes.len(){return Err("曼纽阴魂费用快照损坏".into());}self.mobility_immutable_identity(v)?;},_=>unreachable!()
  }Ok(())
 }
 fn mswm01_grant_receipt(&self,f:&ResolutionFrame)->RuleResult<()> {self.mswm01_frame(f)?;if f.source.card.definition!=ID||f.ability_key!=GRANT||f.cursor!=1||!matches!(f.guard,GuardState::Accepted){return Err("整局授予须保留已结算的原四费行动".into());}Ok(())}
 pub(crate) fn mswm01_resolve(&mut self,f:&ResolutionFrame,grant:bool)->RuleResult<()> {
  self.mswm01_frame(f)?;if f.source.card.definition!=ID||f.cursor!=1||!matches!(f.steps[0].op,Op::MSWM01{grant:g}if g==grant){return Err("曼纽只消费原第一步".into());}
  if grant {if !self.mswm01_grants.iter().any(|v|v.actor==f.actor){self.mswm01_grants.push(f.clone());}}
  else if self.players[f.actor].society_zone.card.is_some()&&self.players[f.actor].society_zone.exposure==0 {self.draw(f.actor,1)?;}
  Ok(())
 }
 pub(crate) fn mswm01_shown(&self,seat:usize,c:&Card)->Option<ShownWitness>{
  let society=self.players[seat].society_zone.card.as_ref().filter(|s|s.definition==ID&&!s.face_down)?;
  if !self.current_subtypes(c).iter().any(|s|s=="阴魂"){return None;}
  let grant=if catalog::card(&c.definition).subtypes.iter().any(|s|s=="阴魂"){None}else{Some(Box::new(self.mswm01_grants.iter().find(|f|f.actor==c.owner)?.clone()))};
  Some(ShownWitness{society:society.clone(),grant})
 }
 pub(crate) fn mswm01_shown_bonus(&self,s:&MysticShown)->RuleResult<u32>{
  let Some(w)=&s.mswm01 else{return Ok(0);};let c=&w.society;
  if c.definition!=ID||c.owner!=s.seat||c.controller!=s.seat||c.face_down||!self.mystic_instance_valid(&c.id){return Err("阴魂加值须保留展示时本方明置曼纽秘社".into());}self.mobility_immutable_identity(c)?;
  let d=catalog::card(&s.card.definition);let printed=d.subtypes.iter().any(|s|s=="阴魂");
  if let Some(f)=&w.grant {self.mswm01_grant_receipt(f)?;if printed||f.actor!=s.card.owner||d.kind!="character"||!(d.subtypes.iter().any(|s|matches!(s.as_str(),"吸血鬼"|"法师"))||s.manor.is_some()&&d.subtypes.iter().any(|s|s=="人类")){return Err("展示阴魂授予须符合原拥有者及吸血鬼或法师类别".into());}}
  else if !printed{return Err("非印刷阴魂展示缺整局授予凭据".into());}Ok(1)
 }
 pub(crate) fn validate_mswm01_state(&self)->RuleResult<()> {
  let mut owners=BTreeSet::new();for f in &self.mswm01_grants{self.mswm01_grant_receipt(f)?;if !owners.insert(f.actor){return Err("曼纽整局授予不能重复堆叠".into());}}
  for e in &self.effects{match e{Effect::Frame{frame}=>self.mswm01_frame(frame)?,Effect::Declare{declaration:d}if d.source.card.definition==ID||related(&d.ability)=>return Err("曼纽不能伪造触发声明".into()),_=>{}}}
  for item in &self.stack{if let Some(f)=&item.frame{self.mswm01_frame(f)?;if f.source.card.definition==ID&&(item.card.is_some()||item.id!=f.frame_id||item.controller!=f.actor||item.reveal||item.deploy_region.is_some()||item.target.is_some()||f.cursor!=0){return Err("曼纽堆叠外壳损坏".into());}}}
  if let Some(p)=&self.pending{match &p.resolution{ChoiceResolution::Frame{frame,..}=>{self.mswm01_frame(frame)?;if frame.source.card.definition==ID{return Err("曼纽无待选续体".into());}},ChoiceResolution::Declare{declaration:d,..}if d.source.card.definition==ID||related(&d.ability)=>return Err("曼纽无触发待选".into()),_=>{}}}Ok(())
 }
}
