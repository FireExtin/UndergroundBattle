//! Closed original Burning Blood Path. No invented replacement/sacrifice for a second path.
use crate::{catalog,engine::RuleResult,model::*,rules::*};
pub(crate) const ID:&str="XQ13";
pub(crate) fn definition()->Definition{attachment_definition(target(Zone::Society,EntityKind::Any,Relation::ControlledByActor,Range::Anywhere))}
pub(crate) fn path(c:&Card)->bool{!c.face_down&&catalog::card(&c.definition).subtypes.iter().any(|s|s=="道途")}
pub(crate) fn validate_definition(id:&str,d:&Definition)->RuleResult<()>{if id==ID&&serde_json::to_value(d).unwrap()!=serde_json::to_value(definition()).unwrap(){return Err("焚血之道须保留原本方秘社结附定义".into());}Ok(())}
impl Game{
 pub(crate) fn xq13_path_entry(&self,actor:usize,c:&Card)->bool{if !path(c){return true;}let others=self.attachments.iter().filter(|a|a.card.id!=c.id&&a.card.controller==actor&&path(&a.card)&&self.attachment_host_valid(a)).collect::<Vec<_>>();others.is_empty()||!matches!(c.definition.as_str(),ID|crate::wm027::ID)&&!others.iter().any(|a|matches!(a.card.definition.as_str(),ID|crate::wm027::ID))}
 pub(crate) fn xq13_icons(&self,c:&Card,parts:(Icons,Icons,Icons))->(Icons,Icons,Icons){if c.face_down||self.in_play(&c.id).is_none_or(|(_,live)|live!=c)||catalog::card(&c.definition).kind!="character"||!self.current_subtypes(c).iter().any(|s|s=="吸血鬼")||!self.attachments.iter().any(|a|a.card.definition==ID&&!a.card.face_down&&a.card.controller==c.controller&&self.attachment_host_valid(a)){return parts;}let(mut p,mut t,mut o)=parts;p.combat=p.combat.saturating_add(p.investigation).saturating_add(p.influence).saturating_add(t.investigation).saturating_add(t.influence).saturating_add(o.investigation).saturating_add(o.influence);p.investigation=0;p.influence=0;t.investigation=0;t.influence=0;o.investigation=0;o.influence=0;(p,t,o)}
 pub(crate) fn validate_xq13_state(&self)->RuleResult<()>{for a in &self.attachments{if matches!(a.card.definition.as_str(),ID|crate::wm027::ID)&&self.attachment_host_valid(a)&&!self.xq13_path_entry(a.card.controller,&a.card){return Err("焚血之道不能与另一受控道途共存".into());}}Ok(())}
}
