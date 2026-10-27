//! BQ056 own-core entry observation, lock and enemy-chosen repression.
use crate::{engine::RuleResult,model::*,rules::*};
use serde::{Serialize,Deserialize};
pub(crate) const ID:&str="BQ056";
pub(crate) const KEY:&str="bq056-core-entry";
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {pub core:SourceSnapshot,pub event_id:String,pub turn:u32}
fn same<T:Serialize>(a:&T,b:&T)->bool{serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap()}
pub(crate) fn definition()->Definition {let mut d=with_abilities(vec![ability(KEY,"本方核心进场：放置锁定，遏制1",Timing::Fast,vec![],vec![target(Zone::Board,EntityKind::CharacterOrHidden,Relation::Any,Range::Anywhere)],vec![Op::BQ056Lock{entry:None},Op::BQ056Repress],Some(Event::OwnCoreEnter))]);d.modifiers.push(StaticModifier::BQ056LockedEnemyInvestigation);d}
pub(crate) fn related(a:&AbilitySpec)->bool{a.key.starts_with("bq056-")||a.ops.iter().any(|o|matches!(o,Op::BQ056Lock{..}|Op::BQ056Repress))}
pub(crate) fn validate_ability(id:&str,a:&AbilitySpec)->RuleResult<()>{let mut expected=definition().abilities.remove(0);if let Some(Op::BQ056Lock{entry})=a.ops.first(){expected.ops[0]=Op::BQ056Lock{entry:entry.clone()};}if id!=ID||!same(a,&expected){return Err("BQ056须保留原目标、核心进场观察、锁定及遏制程序".into());}Ok(())}
pub(crate) fn validate_definition(id:&str,d:&Definition)->RuleResult<()>{if (id==ID||d.abilities.iter().any(related)||d.modifiers.iter().any(|m|matches!(m,StaticModifier::BQ056LockedEnemyInvestigation)))&&(id!=ID||!same(d,&definition())){return Err("BQ056完整定义不能增删或移植".into());}Ok(())}
impl Game {
 pub(crate) fn bq056_investigation(&self,c:&Card,region:usize)->u32 {
  u32::from(c.definition==ID&&!c.face_down&&self.board(&c.id).is_some_and(|(r,live)|r==region&&live==c)&&self.region_live(region)&&self.region_characters(region).any(|other|self.is_enemy(c.controller,other)&&other.lock_markers>0))
 }
 pub(crate) fn observe_bq056_core_entry(&mut self,core:&SourceSnapshot){
  if core.card.face_down||!self.selected_core_card(&core.card)||core.card.controller!=core.card.owner||core.region.is_none(){return;}
  let observers=self.regions.iter().enumerate().filter(|(_,r)|!r.vacant).flat_map(|(r,v)|v.cards.iter().map(move|c|(r,c))).filter(|(_,c)|c.definition==ID&&!c.face_down&&c.controller==core.card.controller).map(|(r,c)|self.source_snapshot(c,Some(r))).collect::<Vec<_>>();
  if observers.is_empty(){return;}let entry=Entry{core:core.clone(),event_id:self.id(),turn:self.turn};
  for source in observers {let mut ability=definition().abilities.remove(0);ability.ops[0]=Op::BQ056Lock{entry:Some(Box::new(entry.clone()))};self.effects.push_back(Effect::Declare{declaration:Declaration{actor:source.card.controller,source,ability}});}
 }
 fn bq056_source(&self,actor:usize,s:&SourceSnapshot,ops:&[Op])->RuleResult<()> {
  let [Op::BQ056Lock{entry:Some(e)},Op::BQ056Repress]=ops else{return Err("BQ056缺完整原核心进场观察".into());};
  if actor>=self.players.len()||s.card.definition!=ID||e.turn!=self.turn||!self.mystic_instance_valid(&e.event_id)||!crate::core_zone::is_core(&e.core.card.definition)||e.core.card.owner!=actor{return Err("BQ056核心身份、拥有者或事件无效".into());}
  for source in [s,&e.core] {let c=&source.card;if c.controller!=actor||c.owner>=self.players.len()||c.face_down||!self.mystic_instance_valid(&c.id)||source.region.is_none_or(|r|r>=self.regions.len())||source.source_region_instance.as_deref().is_none_or(|id|!self.mystic_instance_valid(id))||source.play_source.is_some()||source.attachment_host_instance.is_some()||source.death_echo.is_some()||source.observed_death.is_some()||source.observed_exposure.is_some()||source.observed_mobility_grant.is_some(){return Err("BQ056原观察者或实际核心进场实体无效".into());}self.mobility_immutable_identity(c)?;}
  if s.card.id==e.core.card.id||s.card.id==e.event_id||e.core.card.id==e.event_id{return Err("BQ056观察事件和两实体必须不同".into());}Ok(())
 }
 pub(crate) fn validate_bq056_declaration(&self,d:&Declaration)->RuleResult<()> {if !related(&d.ability){return Ok(());}validate_ability(&d.source.card.definition,&d.ability)?;self.bq056_source(d.actor,&d.source,&d.ability.ops)}
 pub(crate) fn validate_bq056_frame(&self,f:&ResolutionFrame)->RuleResult<()> {
  if f.ability_key!=KEY&&!f.steps.iter().any(|s|matches!(s.op,Op::BQ056Lock{..}|Op::BQ056Repress)){return Ok(());}
  self.bq056_source(f.actor,&f.source,&f.steps.iter().map(|s|s.op.clone()).collect::<Vec<_>>())?;
  let [t]=f.targets.as_slice()else{return Err("BQ056须保留唯一原角色或暗藏者目标".into());};
  if f.ability_key!=KEY||!self.mystic_instance_valid(&f.frame_id)||f.cursor>2||!f.already_paid.is_empty()||!f.resource_payments.is_empty()||f.chosen_region.is_some()||f.steps.iter().any(|s|s.context!=f.actor)||!same(&t.spec,&definition().abilities[0].targets[0])||!self.mystic_instance_valid(&t.id)||t.public.instance_id!=t.id||t.region_instance.is_some()||!t.public.valid||t.public.status!="valid"||t.public.invalid_reason.is_some()||f.cursor==0&&!matches!(f.guard,GuardState::Unchecked)||f.cursor>0&&!matches!(f.guard,GuardState::Accepted){return Err("BQ056帧、费用或目标合同被改写".into());}Ok(())
 }
 fn bq056_opponents(&self,actor:usize)->Vec<ChoiceOption>{self.players.iter().filter(|p|!p.eliminated&&self.team(p.seat)!=self.team(actor)).map(|p|ChoiceOption{id:player_id(p.seat),label:p.name.clone(),card:None}).collect()}
 pub(crate) fn bq056_lock(&mut self,f:&ResolutionFrame)->RuleResult<()> {self.validate_bq056_frame(f)?;if f.ability_key!=KEY||f.cursor!=1{return Err("BQ056放锁须来自原程序第一步".into());}if let Some(c)=self.board_mut(&f.targets[0].id){c.lock_markers=c.lock_markers.checked_add(1).ok_or("锁定标志溢出")?;}Ok(())}
 pub(crate) fn bq056_repress(&mut self,f:ResolutionFrame)->RuleResult<()> {self.validate_bq056_frame(&f)?;if f.ability_key!=KEY||f.cursor!=2{return Err("BQ056遏制须来自原程序第二步".into());}let options=self.bq056_opponents(f.actor);if !options.is_empty(){self.choice(f.actor,"bq056-opponent","便衣警探：选择遏制的敌方玩家".into(),options,1,1,None,ChoiceResolution::Frame{frame:Box::new(f),choice:FrameChoice::BQ056Opponent});}Ok(())}
 pub(crate) fn bq056_choose_opponent(&mut self,f:&ResolutionFrame,chooser:usize,a:&Action,selected:&[String])->RuleResult<()> {
  self.validate_bq056_frame(f)?;if f.ability_key!=KEY||f.cursor!=2||chooser!=f.actor||*a!=(Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")})||selected.len()!=1||!self.bq056_opponents(f.actor).iter().any(|o|o.id==selected[0]){return Err("BQ056遏制对象须为实际敌方玩家".into());}
  let seat=selected[0].strip_prefix('p').and_then(|x|x.parse::<usize>().ok()).ok_or("敌方玩家无效")?;let regions=self.batch20_repress_regions(seat);
  if !regions.is_empty(){self.choice(seat,"bq056-repress","便衣警探：移除1个本方势力".into(),self.batch20_repress_options(&regions,1),1,1,Some(1),ChoiceResolution::Frame{frame:Box::new(f.clone()),choice:FrameChoice::BQ056Repress{seat,regions}});}Ok(())
 }
 pub(crate) fn bq056_finish(&mut self,f:&ResolutionFrame,chooser:usize,a:&Action,selected:&[String],seat:usize,regions:&[(usize,String,u32)])->RuleResult<()> {
  self.validate_bq056_frame(f)?;if f.ability_key!=KEY||f.cursor!=2||seat>=self.players.len()||chooser!=seat||self.team(seat)==self.team(f.actor)||selected.len()!=1||*a!=(Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")})||!same(&regions.to_vec(),&self.batch20_repress_regions(seat))||!self.batch20_repress_options(regions,1).iter().any(|o|o.id==selected[0]){return Err("BQ056遏制确认与原敌方势力不符".into());}
  self.wm040_repress_complete(seat,regions,selected)
 }
 pub(crate) fn validate_bq056_state(&self)->RuleResult<()> {
  for e in &self.effects {match e{Effect::Declare{declaration}=>self.validate_bq056_declaration(declaration)?,Effect::Frame{frame}=>self.validate_bq056_frame(frame)?,_=>{}}}
  for item in &self.stack {if let Some(f)=&item.frame{self.validate_bq056_frame(f)?;if f.ability_key==KEY&&(item.card.is_some()||item.id!=f.frame_id||item.controller!=f.actor||item.reveal||item.target.as_deref()!=f.targets.first().map(|t|t.id.as_str())||item.deploy_region.is_some()||f.cursor!=0){return Err("BQ056触发堆叠外壳不符".into());}}}
  if let Some(p)=&self.pending {match &p.resolution {
   ChoiceResolution::Declare{declaration:d,stage}if related(&d.ability)=>{self.validate_bq056_declaration(d)?;let expected=self.binding_options(d.actor,&d.source,&d.ability.targets[0]);if !matches!(stage,DeclareChoice::Target)||p.seat!=d.actor||p.choice.player_id!=player_id(d.actor)||p.choice.kind!="trigger"||p.choice.title!=format!("{}：{}",crate::batch3_trap::source_label(&d.source),d.ability.label)||!self.mystic_instance_valid(&p.choice.id)||p.choice.min!=Some(0)||p.choice.max!=Some(1)||p.choice.allow_decline!=Some(true)||p.choice.amount.is_some()||!p.choice.preview_cards.is_empty()||!same(&p.choice.options,&expected){return Err("BQ056原触发目标待选不符".into());}},
   ChoiceResolution::Frame{frame:f,choice}if f.ability_key==KEY||matches!(choice,FrameChoice::BQ056Opponent|FrameChoice::BQ056Repress{..})=>{if f.ability_key!=KEY{return Err("BQ056续体不能移植其他帧".into());}self.validate_bq056_frame(f)?;let(seat,kind,title,amount,options)=match choice{FrameChoice::BQ056Opponent=>(f.actor,"bq056-opponent","便衣警探：选择遏制的敌方玩家",None,self.bq056_opponents(f.actor)),FrameChoice::BQ056Repress{seat,regions}=>{if *seat>=self.players.len()||self.team(*seat)==self.team(f.actor)||regions.is_empty()||!same(regions,&self.batch20_repress_regions(*seat)){return Err("BQ056原遏制地区不符".into());}(*seat,"bq056-repress","便衣警探：移除1个本方势力",Some(1),self.batch20_repress_options(regions,1))},_=>return Err("BQ056续体种类无效".into())};if f.ability_key!=KEY||f.cursor!=2||p.seat!=seat||p.choice.player_id!=player_id(seat)||!self.mystic_instance_valid(&p.choice.id)||p.choice.kind!=kind||p.choice.title!=title||p.choice.amount!=amount||p.choice.min!=Some(1)||p.choice.max!=Some(1)||p.choice.allow_decline!=Some(false)||!p.choice.preview_cards.is_empty()||!same(&p.choice.options,&options){return Err("BQ056遏制待选被改写".into());}},_=>{}}
  }
  let mut events=std::collections::BTreeSet::new();
  let mut check=|source:&SourceSnapshot,ops:Vec<Op>|->RuleResult<()>{if let Some(Op::BQ056Lock{entry:Some(e)})=ops.first(){if !events.insert((e.event_id.clone(),source.card.id.clone())){return Err("BQ056同一观察者不能重复持有同一核心进场事务".into());}}Ok(())};
  for e in &self.effects{match e{Effect::Declare{declaration:d}=>check(&d.source,d.ability.ops.clone())?,Effect::Frame{frame:f}=>check(&f.source,f.steps.iter().map(|s|s.op.clone()).collect())?,_=>{}}}
  for item in &self.stack{if let Some(f)=&item.frame{check(&f.source,f.steps.iter().map(|s|s.op.clone()).collect())?;}}
  if let Some(p)=&self.pending{match &p.resolution{ChoiceResolution::Declare{declaration:d,..}=>check(&d.source,d.ability.ops.clone())?,ChoiceResolution::Frame{frame:f,..}=>check(&f.source,f.steps.iter().map(|s|s.op.clone()).collect())?,_=>{}}}
  Ok(())
 }
}
