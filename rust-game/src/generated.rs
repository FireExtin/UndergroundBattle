//! Project lifecycle ruling: provenance, not printed name, determines disappearance.
use crate::{engine::RuleResult,model::*};
use serde::{Deserialize,Serialize};
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize)]
pub enum GeneratedKind { Tentacle, Police, Cultist, Walker }
impl GeneratedKind {
    pub(crate) fn definition(self)->&'static str {match self{Self::Tentacle=>"TK010",Self::Police=>"TK003",Self::Cultist=>"TK008",Self::Walker=>"TK001"}}
}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneratedOrigin {pub kind:GeneratedKind,pub birth_instance:String,pub owner:usize}
pub(crate) fn is_generated(c:&Card)->bool{c.generated.is_some()}
impl Game {
    pub(crate) fn make_generated(&mut self,kind:GeneratedKind,owner:usize)->RuleResult<Card>{
        if owner>=self.players.len() || !crate::catalog::catalog().cards.iter().any(|d|d.id==kind.definition()) {return Err("生成角色须有实际登记定义及有效拥有者".into());}
        let mut c=self.make_card(kind.definition(),owner);
        c.generated=Some(GeneratedOrigin{kind,birth_instance:c.id.clone(),owner});Ok(c)
    }
    pub(crate) fn validate_generated_identity(&self,c:&Card)->RuleResult<()>{
        if let Some(origin)=&c.generated {
            if c.definition!=origin.kind.definition() || c.owner!=origin.owner || origin.owner>=self.players.len()
                || !self.mystic_instance_number(&origin.birth_instance).zip(self.mystic_instance_number(&c.id)).is_some_and(|(birth,now)|birth<=now) {
                return Err("生成物来源身份、出生实例或拥有者损坏".into());
            }
        }else if c.definition==crate::tentacle::ID {return Err("触手指示物缺少实际生成身份；不猜测迁移旧快照".into());}
        Ok(())
    }
    /// Called after ordinary departure cleanup, before any physical-zone arrival.
    pub(crate) fn generated_disappears(&mut self,c:&Card)->bool{
        if !is_generated(c){return false;}
        self.note("生成角色离场消失，不进入实体牌区".into());true
    }
    pub(crate) fn validate_generated_state(&self)->RuleResult<()>{
        let mut births=std::collections::BTreeSet::new();
        for c in self.batch3_physical_cards(){
            self.validate_generated_identity(c)?;
            if !is_generated(c){continue;}
            if !births.insert(&c.generated.as_ref().unwrap().birth_instance){return Err("同一出生身份不能存在两个生成角色实体".into());}
            let retained=self.regions.iter().any(|r|!r.vacant&&r.cards.iter().any(|v|std::ptr::eq(v,c)))
                ||self.sealed_cards.iter().any(|s|std::ptr::eq(&s.card,c))
                ||self.effects.iter().any(|e|matches!(e,Effect::Bury{card} if std::ptr::eq(card,c)));
            if !retained{return Err("生成角色不能存在于手牌、牌库、墓地或其他实体牌区".into());}
        }
        Ok(())
    }
}
