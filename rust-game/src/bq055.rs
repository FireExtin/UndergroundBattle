//! BQ055: own selected core discount and frozen reveal-time contest support.
use crate::{engine::RuleResult,model::*,rules::*,mystic_contest::MysticShown};
use serde::{Serialize,Deserialize};
pub(crate) const ID:&str="BQ055";
fn same<T:Serialize>(a:&T,b:&T)->bool{serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap()}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShownWitness { pub core:SourceSnapshot,pub observers:Vec<SourceSnapshot> }
pub(crate) fn definition()->Definition {let mut d=with_abilities(vec![]);d.traits.public=true;d.modifiers.push(StaticModifier::BQ055CoreSupport);d}
pub(crate) fn validate_definition(id:&str,d:&Definition)->RuleResult<()> {
 if (id==ID||d.modifiers.iter().any(|m|matches!(m,StaticModifier::BQ055CoreSupport)))&&(id!=ID||!same(d,&definition())) {return Err("BQ055公开、核心费用和隐秘加值定义不能改写或移植".into());}Ok(())
}
impl Game {
 fn bq055_observers(&self,seat:usize)->Vec<SourceSnapshot>{self.regions.iter().enumerate().filter(|(_,r)|!r.vacant).flat_map(|(r,v)|v.cards.iter().map(move|c|(r,c))).filter(|(_,c)|c.definition==ID&&c.controller==seat&&!c.face_down).map(|(r,c)|self.source_snapshot(c,Some(r))).collect()}
 pub(crate) fn bq055_reduction(&self,seat:usize,c:&Card)->u32 {
  if c.owner!=seat||!crate::core_zone::is_core(&c.definition)||!self.core_zones.iter().any(|z|z.owner==seat&&z.definition==c.definition){return 0;}
  self.bq055_observers(seat).len() as u32
 }
 pub(crate) fn bq055_shown_witness(&self,seat:usize)->Option<ShownWitness>{
  let core=self.selected_core_source(seat).ok()?;if core.region.is_none(){return None;}
  let observers=self.bq055_observers(seat);if observers.is_empty(){None}else{Some(ShownWitness{core,observers})}
 }
 pub(crate) fn bq055_shown_bonus(&self,shown:&MysticShown)->RuleResult<u32>{
  let Some(w)=&shown.core_support else{return Ok(0);};
  if w.observers.is_empty()||!crate::core_zone::is_core(&w.core.card.definition)||w.core.card.owner!=shown.seat{return Err("BQ055缺原在场本人核心及观察者".into());}
  let mut ids=std::collections::BTreeSet::new();
  for (n,s) in std::iter::once(&w.core).chain(&w.observers).enumerate(){let c=&s.card;
   if (n>0&&c.definition!=ID)||c.face_down||c.controller!=shown.seat||c.owner>=self.players.len()||!self.mystic_instance_valid(&c.id)||!ids.insert(&c.id)
    ||s.region.is_none_or(|r|r>=self.regions.len())||s.source_region_instance.as_deref().is_none_or(|id|!self.mystic_instance_valid(id))||s.play_source.is_some()||s.attachment_host_instance.is_some()||s.death_echo.is_some()||s.observed_death.is_some()||s.observed_exposure.is_some()||s.observed_mobility_grant.is_some(){return Err("BQ055原展示核心/观察者身份与地区不符".into());}
   self.mobility_immutable_identity(c)?;
  }
  u32::try_from(w.observers.len()).ok().and_then(|n|n.checked_mul(3)).ok_or("BQ055加值溢出".into())
 }
}
