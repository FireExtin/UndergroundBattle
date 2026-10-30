//! First closed core identity/zone component, based on Hegemony printed p20.
//! Lobby selection, deployment/rest and command-card producers are separate
//! integration work. No public Action creates a core zone or invokes a core.
use crate::{engine::RuleResult,model::*};
use serde::{Serialize,Deserialize};
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreZone {
    pub owner:usize,
    pub definition:String,
    pub card:Option<Card>,
    #[serde(default)]
    pub rest:u32,
}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct CoreZoneView {
    pub player_id:String,
    pub definition:String,
    pub card:Option<CardView>,
    pub rest:u32,
}
pub(crate) fn is_core(id:&str)->bool {matches!(id,crate::bq046::ID|crate::bq045::ID|crate::bq044::ID)}
impl Game {
    pub(crate) fn validate_core_zones(&self)->RuleResult<()> {
        let mut owners=std::collections::BTreeSet::new();
        for z in &self.core_zones {
            if z.owner>=self.players.len()||!is_core(&z.definition)||!owners.insert(z.owner) {return Err("核心区须有唯一拥有者及已实现核心身份".into());}
            if self.batch3_physical_cards().filter(|c|c.owner==z.owner&&c.definition==z.definition).count()>1 {return Err("所选核心不能同时存在多个物理副本".into());}
            if let Some(c)=&z.card {
                if c.definition!=z.definition||c.owner!=z.owner||c.controller!=z.owner||c.face_down||c.exhausted
                    ||c.damage!=0||c.wounds!=0||c.shield!=0||c.lock_markers!=0||c.time_markers!=0||c.charge_markers!=0
                    ||c.permanent_investigation_markers != 0||c.permanent_defense_markers!=0
                    ||!self.mystic_instance_valid(&c.id)
                    ||self.batch3_physical_cards().filter(|live|live.id==c.id).count()!=1 {
                    return Err("核心区实体、拥有者、跨区状态或唯一性无效".into());
                }
                self.mobility_immutable_identity(c)?;
            }
        }
        for item in &self.stack {self.validate_core_play(item)?;}
        self.validate_core_departures()?;
        self.validate_core_entry_region()?;
        Ok(())
    }
    pub(crate) fn selected_core_source(&self,seat:usize)->RuleResult<SourceSnapshot> {
        self.validate_core_zones()?;
        if self.players.get(seat).is_none_or(|p|p.eliminated) {return Err("核心拥有者无效".into());}
        let z=self.core_zones.iter().find(|z|z.owner==seat).ok_or("未选择核心角色")?;
        if let Some(c)=&z.card {return Ok(self.source_snapshot(c,None));}
        let sources=self.regions.iter().enumerate().filter(|(_,r)|!r.vacant)
            .flat_map(|(r,v)|v.cards.iter().map(move|c|(r,c)))
            .filter(|(_,c)|c.definition==z.definition&&c.owner==seat&&c.controller==seat&&!c.face_down).collect::<Vec<_>>();
        if sources.len()!=1 {return Err("所选核心不在核心区或本人操控的正面场上".into());}
        Ok(self.source_snapshot(sources[0].1,Some(sources[0].0)))
    }
    /// Internal resolved-effect hook, not a declaration or player action.
    /// Future command producers must retain their own paid frame/provenance.
    #[allow(dead_code)]
    pub(crate) fn resolve_selected_bq046_core(&mut self,seat:usize)->RuleResult<()> {
        let (_,amount)=self.selected_bq046_core_program(seat)?;
        if self.pending.is_some() {return Err("不能覆盖尚未完成的选择".into());}
        self.effects.push_front(Effect::Forecast{seat,amount,draw:false});
        Ok(())
    }
    pub(crate) fn selected_bq046_core_program(&self,seat:usize)->RuleResult<(SourceSnapshot,usize)> {
        let s=self.selected_core_source(seat)?;
        if s.card.definition!=crate::bq046::ID {return Err("仅实现莫佳娜核心预测3".into());}
        Ok((s,3))
    }
    pub(crate) fn core_zone_views(&self,viewer:usize)->Vec<CoreZoneView> {
        self.core_zones.iter().map(|z|CoreZoneView{player_id:player_id(z.owner),definition:z.definition.clone(),rest:z.rest,
            card:z.card.as_ref().map(|c|self.card_view(c,viewer,None,Some("core")))}).collect()
    }
}

#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Departure {pub source:SourceSnapshot,pub card:Card,pub destination:Destination,pub turn:u32}
#[derive(Clone,Debug,Serialize,Deserialize)]
pub enum Destination {Hand,Bottom,Grave}
fn options()->Vec<ChoiceOption>{vec![ChoiceOption{id:"core".into(),label:"回核心区并增加1休整标志".into(),card:None},ChoiceOption{id:"destination".into(),label:"保留原离场去向".into(),card:None}]}
impl Game {
 pub(crate) fn selected_core_card(&self,c:&Card)->bool {is_core(&c.definition)&&self.core_zones.iter().any(|z|z.owner==c.owner&&z.definition==c.definition&&z.card.is_none())}
 pub(crate) fn defer_core_departure(&mut self,source:SourceSnapshot,card:Card,destination:Destination){
  self.effects.push_back(Effect::CoreReturn{departure:Box::new(Departure{source,card,destination,turn:self.turn})});
 }
 fn check_core_departure(&self,d:&Departure)->RuleResult<()> {
  let c=&d.card;let s=&d.source;
  if !self.selected_core_card(c)||d.turn!=self.turn||c.owner>=self.players.len()||s.card.owner!=c.owner||s.card.definition!=c.definition
   ||c.controller!=c.owner||c.face_down||c.exhausted||c.damage!=0||c.wounds!=0||c.shield!=0
   ||!self.mystic_instance_valid(&c.id)||!self.mystic_instance_valid(&s.card.id)||c.id==s.card.id
   ||s.region.is_none_or(|r|r>=self.regions.len())||s.source_region_instance.as_deref().is_none_or(|id|!self.mystic_instance_valid(id))
   ||s.play_source.is_some()||s.attachment_host_instance.is_some()||s.death_echo.is_some()||s.observed_death.is_some()||s.observed_exposure.is_some()||s.observed_mobility_grant.is_some(){return Err("核心离场选择须保留实际原角色和唯一暂存新实体".into());}
  self.mobility_immutable_identity(c)?;self.mobility_immutable_identity(&s.card)
 }
 pub(crate) fn core_departure_choice(&mut self,d:Departure)->RuleResult<()> {
  self.check_core_departure(&d)?;
  if self.players[d.card.owner].eliminated {return Ok(());}
  self.choice(d.card.owner,"core-return","核心离场：选择去向".into(),options(),1,1,None,ChoiceResolution::CoreReturn{departure:Box::new(d)});Ok(())
 }
 pub(crate) fn finish_core_departure(&mut self,d:Departure,a:&Action,selected:&[String])->RuleResult<()> {
  self.check_core_departure(&d)?;
  if *a!=(Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")})||selected.len()!=1{return Err("核心离场选择载荷无效".into());}
  let owner=d.card.owner;
  match selected[0].as_str(){
   "core"=>{let z=self.core_zones.iter_mut().find(|z|z.owner==owner).ok_or("核心区缺失")?;z.rest=z.rest.checked_add(1).ok_or("核心休整数量溢出")?;z.card=Some(d.card);},
   "destination"=>match d.destination{Destination::Hand=>self.players[owner].hand.push(d.card),Destination::Bottom=>self.players[owner].deck.push(d.card),Destination::Grave=>self.commit_owner_grave(d.card,None)},
   _=>return Err("核心离场选项无效".into())
  }Ok(())
 }
 pub(crate) fn validate_core_departures(&self)->RuleResult<()> {
  let pending=self.pending.as_ref().and_then(|p|if let ChoiceResolution::CoreReturn{departure}=&p.resolution{Some((p,departure.as_ref()))}else{None});
  let mut ids=std::collections::BTreeSet::new();
  for d in self.effects.iter().filter_map(|e|if let Effect::CoreReturn{departure}=e{Some(departure.as_ref())}else{None}).chain(pending.iter().map(|(_,d)|*d)){
   self.check_core_departure(d)?;
   if !ids.insert(&d.card.id)||self.batch3_physical_cards().filter(|c|c.id==d.card.id).count()!=1||self.batch3_physical_cards().any(|c|c.id==d.source.card.id){return Err("核心离场暂存实体重复或旧对象仍在场".into());}
  }
  if let Some((p,d))=pending {if p.seat!=d.card.owner||p.choice.player_id!=player_id(d.card.owner)||p.choice.kind!="core-return"||p.choice.title!="核心离场：选择去向"||p.choice.min!=Some(1)||p.choice.max!=Some(1)||p.choice.amount.is_some()||p.choice.allow_decline!=Some(false)||!p.choice.preview_cards.is_empty()||!self.mystic_instance_valid(&p.choice.id)||serde_json::to_value(&p.choice.options).unwrap()!=serde_json::to_value(options()).unwrap(){return Err("核心离场待选被改写".into());}}
  Ok(())
 }
}

impl Game {
 pub(crate) fn validate_core_play(&self,item:&StackItem)->RuleResult<()> {
  let relevant=item.card.as_ref().is_some_and(|c|is_core(&c.definition))||item.frame.as_ref().is_some_and(|f|matches!(f.source.play_source,Some(PlaySource::CoreZone{..}))||(is_core(&f.source.card.definition)&&matches!(f.ability_key.as_str(),"deploy"|"reveal")));
  if !relevant{return Ok(());}
  let f=item.frame.as_ref().ok_or("核心派遣事务缺原帧")?;let c=&f.source.card;
  let mut extra=if let Some(PlaySource::CoreZone{rest})=f.source.play_source {
    if c.owner!=f.actor||!self.core_zones.iter().any(|z|z.owner==f.actor&&z.definition==c.definition&&z.rest==rest&&z.card.is_none()){return Err("核心区派遣原拥有者或休整记录无效".into());}
    rest.checked_mul(2).ok_or("核心休整费用溢出")?
  }else{0};
  let ids=match f.already_paid.as_slice(){[PaidCost::Assets(ids)]=>ids,[PaidCost::Assets(ids),PaidCost::CoreRestHonor{before,rest}]if *rest>0&&matches!(f.source.play_source,Some(PlaySource::CoreZone{rest:r})if r==*rest)&&rest.checked_mul(2).is_some_and(|n|*before>=n)=>{extra=0;ids},_=>return Err("核心派遣费用回执无效".into())};
  if !is_core(&c.definition)||c.owner>=self.players.len()||c.controller!=f.actor||f.actor>=self.players.len()
   ||!matches!(f.ability_key.as_str(),"deploy"|"reveal")||f.ability_key=="deploy"&&(c.face_down||!f.source.valid_character_play())||f.ability_key=="reveal"&&(!c.face_down||f.source.play_source.is_some())||f.cursor!=0||!matches!(f.guard,GuardState::Unchecked)||!f.steps.is_empty()||!f.targets.is_empty()
   ||f.source.region.map_or(!(f.ability_key=="deploy"&&self.has_shoggoth(f.actor)&&self.msbq02_core(f.actor,c)),|r|r>=self.regions.len())||f.chosen_region!=f.source.region||f.source.source_region_instance.as_deref().map_or(f.source.region.is_some(),|id|f.source.region.is_none()||!self.mystic_instance_valid(id))
   ||f.source.attachment_host_instance.is_some()||f.source.death_echo.is_some()||f.source.observed_death.is_some()||f.source.observed_exposure.is_some()||f.source.observed_mobility_grant.is_some()
   ||!self.mystic_instance_valid(&f.frame_id)||!self.mystic_instance_valid(&c.id)||item.id!=f.frame_id||item.controller!=f.actor||item.reveal!=(f.ability_key=="reveal")||item.target.is_some()||item.deploy_region!=f.source.region
   ||!(f.money_units(0)>=extra as usize&&f.money_units(0)<=extra.saturating_add(crate::catalog::card(&c.definition).cost) as usize&&ids.iter().all(|id|self.mystic_instance_valid(id)&&*id!=c.id)&&ids.iter().collect::<std::collections::BTreeSet<_>>().len()==ids.len())
   ||item.card.as_ref().is_none_or(|actual|!self.pack_a_transaction_matches(f,actual)){return Err("核心派遣实体、原来源、休整费用或堆叠帧无效".into());}
  self.mobility_immutable_identity(c)
 }
}
