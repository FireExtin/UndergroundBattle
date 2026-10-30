//! Closed Black Market. Host omission uses a card-local project ruling: own society.
use crate::{catalog,engine::RuleResult,model::*,rules::*};
use serde::Serialize;
use std::collections::BTreeSet;
pub(crate) const ID:&str="WM063";
pub(crate) const DRAW:&str="wm063-draw";
pub(crate) const REPRESS:&str="wm063-repress";
pub(crate) const MARK:&str="wm063-combat";
fn same<T:Serialize>(a:&T,b:&T)->bool{serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap()}
pub(crate) fn definition()->Definition{
 let host=target(Zone::Society,EntityKind::Any,Relation::ControlledByActor,Range::Anywhere);
 let mut attach=ability("attach","结附于本方秘社（项目裁定）",Timing::Standard,vec![],vec![host.clone()],vec![],None);attach.play_only=true;
 let mut abilities=vec![attach];
 for (key,label,money,targets) in [(DRAW,"弃事务：抓一张",2,vec![]),(REPRESS,"弃角色：遏制2",1,vec![]),(MARK,"弃附属：目标角色放置战斗标志",1,vec![target(Zone::Board,EntityKind::Character,Relation::Any,Range::Anywhere)])]{
  let mut a=ability(key,label,Timing::Standard,vec![Cost::Assets(money),Cost::WM063Discard],targets,vec![Op::WM063Resolve],None);a.activation_only=true;abilities.push(a);
 }
 let mut d=with_abilities(abilities);d.attachment=Some(AttachmentSpec{controls_host:false,host_subtype_change:SubtypeChange::None,host,host_icons:Icons::default(),host_temporary_icons:None,host_barrier:false,host_defense_bonus:0,host_leaves:HostLeaveDestination::OwnerGraveyard});d
}
pub(crate) fn related(a:&AbilitySpec)->bool{a.key.starts_with("wm063-")||a.costs.iter().any(|c|matches!(c,Cost::WM063Discard))||a.ops.iter().any(|o|matches!(o,Op::WM063Resolve))}
pub(crate) fn validate_ability(id:&str,a:&AbilitySpec)->RuleResult<()>{if id!=ID||!definition().abilities.iter().any(|v|same(v,a)){return Err("黑市必须保留原费用和三项行动".into());}Ok(())}
pub(crate) fn validate_definition(id:&str,d:&Definition)->RuleResult<()>{if (id==ID||d.abilities.iter().any(related))&&(id!=ID||!same(d,&definition())){return Err("黑市定义不能移植或改写".into());}Ok(())}
pub(crate) fn cost_card(key:&str,c:&Card)->bool{!c.face_down&&match key{DRAW=>catalog::card(&c.definition).kind=="spell",REPRESS=>catalog::has_kind(catalog::card(&c.definition),"character"),MARK=>catalog::has_kind(catalog::card(&c.definition),"attachment"),_=>false}}
impl Game{
 pub(crate) fn wm063_borrow(&self,actor:usize,c:&Card)->bool{
  c.definition==ID&&!c.face_down&&actor<self.players.len()&&c.controller<self.players.len()&&!self.players[actor].eliminated&&!self.players[c.controller].eliminated&&self.team(actor)!=self.team(c.controller)&&self.players[actor].society_zone.secrecy>0&&self.attachments.iter().any(|a|a.card==*c&&self.attachment_host_valid(a))
 }
 pub(crate) fn wm063_pay(&mut self,actor:usize,s:&SourceSnapshot,a:&AbilitySpec,input:&Action)->RuleResult<PaidCost>{
  validate_ability(&s.card.definition,a)?;
  if *input!=(Action{card_id:Some(s.card.id.clone()),ability_id:Some(a.key.clone()),cost_selected:input.cost_selected.clone(),target_id:input.target_id.clone(),..Action::new("activate")}){return Err("黑市行动不能夹带额外载荷".into());}
  if actor!=s.card.controller&&!self.wm063_borrow(actor,&s.card){return Err("借用黑市须本人秘社有隐秘且为敌方".into());}
  let ids=input.cost_selected.as_ref().filter(|v|v.len()==1).ok_or("黑市须选择一张对应类型手牌支付")?;
  let c=self.players[actor].hand.iter().find(|c|c.id==ids[0]&&cost_card(&a.key,c)).ok_or("黑市弃牌类型不符")?.clone();
  let secrecy=self.players[actor].society_zone.secrecy;self.discard_hand_card(actor,&c.id)?;Ok(PaidCost::WM063Discard{card:c,secrecy})
 }
 pub(crate) fn wm063_frame(&self,f:&ResolutionFrame)->RuleResult<()>{
  if !(f.source.card.definition==ID&&f.ability_key!="attach")&&!f.ability_key.starts_with("wm063-")&&!f.steps.iter().any(|s|matches!(s.op,Op::WM063Resolve))&&!f.already_paid.iter().any(|p|matches!(p,PaidCost::WM063Discard{..})){return Ok(());}
  let s=&f.source;let c=&s.card;let a=definition().abilities.into_iter().find(|a|a.key==f.ability_key&&a.activation_only).ok_or("黑市未知行动")?;
  self.validate_resource_frame(f)?;
  if c.definition!=ID||f.actor>=self.players.len()||c.controller>=self.players.len()||c.owner>=self.players.len()||c.face_down||!self.mystic_instance_valid(&c.id)||!self.mystic_instance_valid(&f.frame_id)||s.region.is_some()||s.source_region_instance.is_some()||s.play_source.is_some()||s.attachment_host_instance.as_deref()!=Some(format!("society:p{}",c.controller).as_str())||s.death_echo.is_some()||s.observed_death.is_some()||s.observed_exposure.is_some()||s.observed_mobility_grant.is_some()||f.chosen_region.is_some()||f.steps.len()!=1||!matches!(f.steps[0].op,Op::WM063Resolve)||f.steps[0].context!=f.actor||f.cursor>1||f.cursor==0&&!matches!(f.guard,GuardState::Unchecked)||f.cursor==1&&!matches!(f.guard,GuardState::Accepted){return Err("黑市原来源、宿主或续体损坏".into());}
  self.mobility_immutable_identity(c)?;
  let [PaidCost::Assets(ids),PaidCost::WM063Discard{card,secrecy}]=f.already_paid.as_slice()else{return Err("黑市缺原两项支付".into());};
  let money=if f.ability_key==DRAW{2}else{1};
  if f.money_units(0)!=money||ids.iter().any(|id|!self.mystic_instance_valid(id))||ids.iter().collect::<BTreeSet<_>>().len()!=ids.len()||!cost_card(&f.ability_key,card)||card.controller!=f.actor||card.owner>=self.players.len()||!self.mystic_instance_valid(&card.id)||card.id==c.id||ids.contains(&card.id)||(f.actor!=c.controller&&(self.team(f.actor)==self.team(c.controller)||*secrecy==0)){return Err("黑市原弃牌、资产或借用资格不符".into());}self.mobility_immutable_identity(card)?;
  if f.targets.len()!=a.targets.len(){return Err("黑市原目标数量损坏".into());}
  for (t,spec) in f.targets.iter().zip(&a.targets){if !same(&t.spec,spec)||!self.mystic_instance_valid(&t.id)||t.public.instance_id!=t.id||!t.public.valid||t.public.status!="valid"||t.public.invalid_reason.is_some()||t.region_instance.is_some(){return Err("黑市原角色绑定损坏".into());}}
  Ok(())
 }
 pub(crate) fn wm063_play(&self,item:&StackItem)->RuleResult<()>{
  if !item.card.as_ref().is_some_and(|c|c.definition==ID)&&!item.frame.as_ref().is_some_and(|f|f.source.card.definition==ID&&f.ability_key=="attach"){return Ok(());}
  let f=item.frame.as_ref().ok_or("黑市结附缺原事务帧")?;let s=&f.source;let c=&s.card;self.validate_resource_frame(f)?;
  let [t]=f.targets.as_slice()else{return Err("黑市须唯一原本方秘社宿主")?;};
  if c.definition!=ID||f.actor>=self.players.len()||c.controller!=f.actor||c.owner>=self.players.len()||c.face_down||s.region.is_some()||s.source_region_instance.is_some()||s.attachment_host_instance.is_some()||s.play_source.is_some()||s.death_echo.is_some()||s.observed_death.is_some()||s.observed_exposure.is_some()||s.observed_mobility_grant.is_some()||f.ability_key!="attach"||f.cursor!=0||!matches!(f.guard,GuardState::Unchecked)||!f.steps.is_empty()||f.chosen_region.is_some()||!self.mystic_instance_valid(&c.id)||!self.mystic_instance_valid(&f.frame_id)||t.id!=format!("society:p{}",f.actor)||!same(&t.spec,&definition().abilities[0].targets[0])||t.public.instance_id!=t.id||!t.public.valid||t.public.status!="valid"||t.public.invalid_reason.is_some()||t.region_instance.is_some()||!matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)]if f.money_units(0)<=2&&ids.iter().all(|id|self.mystic_instance_valid(id))&&ids.iter().collect::<BTreeSet<_>>().len()==ids.len())||item.id!=f.frame_id||item.controller!=f.actor||item.target.as_deref()!=Some(t.id.as_str())||item.reveal||item.deploy_region.is_some()||item.card.as_ref().is_none_or(|c|!self.pack_a_transaction_matches(f,c)){return Err("黑市结附事务身份损坏".into());}self.mobility_immutable_identity(c)
 }
 fn wm063_players(&self)->Vec<ChoiceOption>{self.players.iter().filter(|p|!p.eliminated).map(|p|ChoiceOption{id:player_id(p.seat),label:p.name.clone(),card:None}).collect()}
 pub(crate) fn wm063_resolve(&mut self,f:ResolutionFrame)->RuleResult<()>{
  self.wm063_frame(&f)?;if f.source.card.definition!=ID||f.cursor!=1{return Err("黑市须消费原第一步")?;}
  match f.ability_key.as_str(){DRAW=>{self.draw(f.actor,1)?;},MARK=>{if self.valid_bound_target(f.actor,&f.source,&f.targets[0]){self.place_board_markers(&f.targets[0].id,crate::batch4_markers::MarkerKind::PermanentCombat,1);}},REPRESS=>{let options=self.wm063_players();if !options.is_empty(){self.choice(f.actor,"wm063-player","黑市：选择遏制玩家".into(),options,1,1,None,ChoiceResolution::Frame{frame:Box::new(f),choice:FrameChoice::WM063Player});}},_=>unreachable!()}Ok(())
 }
 fn wm063_choice(&self,f:&ResolutionFrame,chooser:usize,a:&Action,seat:usize)->RuleResult<()>{self.wm063_frame(f)?;if f.ability_key!=REPRESS||f.cursor!=1||chooser!=seat||*a!=(Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")}){return Err("黑市待选只能由原指定玩家提交选择".into());}Ok(())}
 pub(crate) fn wm063_player(&mut self,f:&ResolutionFrame,chooser:usize,a:&Action,selected:&[String])->RuleResult<()>{
  self.wm063_choice(f,chooser,a,f.actor)?;if selected.len()!=1||!self.wm063_players().iter().any(|o|o.id==selected[0]){return Err("请选择一名实际玩家".into());}let seat=selected[0][1..].parse::<usize>().map_err(|_|"玩家无效")?;let regions=self.batch20_repress_regions(seat);let n=2usize.min(regions.iter().map(|r|r.2 as usize).sum());
  if n>0{self.choice(seat,"wm063-repress",format!("黑市：移除{n}个本方势力"),self.batch20_repress_options(&regions,n),n,n,Some(n as u32),ChoiceResolution::Frame{frame:Box::new(f.clone()),choice:FrameChoice::WM063Repress{seat,regions}});}Ok(())
 }
 pub(crate) fn wm063_finish(&mut self,f:&ResolutionFrame,chooser:usize,a:&Action,selected:&[String],seat:usize,regions:&[(usize,String,u32)])->RuleResult<()>{
  self.wm063_choice(f,chooser,a,seat)?;if seat>=self.players.len()||!same(&regions.to_vec(),&self.batch20_repress_regions(seat)){return Err("黑市原遏制地区变化".into());}let n=2usize.min(regions.iter().map(|r|r.2 as usize).sum());let options=self.batch20_repress_options(regions,n);if n==0||selected.len()!=n||selected.iter().collect::<BTreeSet<_>>().len()!=n||selected.iter().any(|id|!options.iter().any(|o|o.id==*id)){return Err("黑市须移除指定数量不同势力".into());}self.wm040_repress_complete(seat,regions,selected)
 }
 pub(crate) fn validate_wm063_state(&self)->RuleResult<()>{
  let mut frames=BTreeSet::new();let mut remember=|f:&ResolutionFrame|->RuleResult<()>{self.wm063_frame(f)?;if f.ability_key.starts_with("wm063-")&&!frames.insert(f.frame_id.clone()){return Err("同一黑市行动不能重复续体".into());}Ok(())};
  for item in &self.stack{self.wm063_play(item)?;if let Some(f)=&item.frame{remember(f)?;if f.ability_key.starts_with("wm063-")&&(item.card.is_some()||item.id!=f.frame_id||item.controller!=f.actor||item.reveal||item.deploy_region.is_some()||item.target.as_deref()!=f.targets.first().map(|t|t.id.as_str())||f.cursor!=0){return Err("黑市行动堆叠外壳损坏".into());}}}
  for e in &self.effects{match e{Effect::Frame{frame:f}=>remember(f)?,Effect::Declare{declaration:d}if d.source.card.definition==ID||related(&d.ability)=>return Err("黑市没有触发声明".into()),_=>{}}}
  if let Some(p)=&self.pending{match &p.resolution{ChoiceResolution::Frame{frame:f,choice}=>{remember(f)?;if f.ability_key.starts_with("wm063-")||matches!(choice,FrameChoice::WM063Player|FrameChoice::WM063Repress{..}){
   if f.ability_key!=REPRESS||f.cursor!=1{return Err("黑市只有原遏制待选".into());}
   let (seat,kind,title,n,amount,options)=match choice{FrameChoice::WM063Player=>(f.actor,"wm063-player","黑市：选择遏制玩家".into(),1,None,self.wm063_players()),FrameChoice::WM063Repress{seat,regions}=>{if *seat>=self.players.len()||self.players[*seat].eliminated||regions.is_empty()||!same(regions,&self.batch20_repress_regions(*seat)){return Err("黑市遏制原区域损坏".into());}let n=2usize.min(regions.iter().map(|r|r.2 as usize).sum());(*seat,"wm063-repress",format!("黑市：移除{n}个本方势力"),n,Some(n as u32),self.batch20_repress_options(regions,n))},_=>return Err("黑市待选程序损坏".into())};
   let expected=Choice{id:p.choice.id.clone(),kind:kind.into(),title,description:"选择符合数量限制的选项，然后确认。".into(),player_id:player_id(seat),options,preview_cards:vec![],min:Some(n),max:Some(n),amount,allow_decline:Some(false)};
   if p.seat!=seat||!self.mystic_instance_valid(&p.choice.id)||!same(&p.choice,&expected){return Err("黑市公开待选数量或选项损坏".into());}
  }},ChoiceResolution::Declare{declaration:d,..}if d.source.card.definition==ID||related(&d.ability)=>return Err("黑市不能恢复触发声明".into()),_=>{}}}
  Ok(())
 }
}
