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
}
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct CoreZoneView {
    pub player_id:String,
    pub definition:String,
    pub card:Option<CardView>,
}
pub(crate) fn is_core(id:&str)->bool {id==crate::bq046::ID}
impl Game {
    pub(crate) fn validate_core_zones(&self)->RuleResult<()> {
        let mut owners=std::collections::BTreeSet::new();
        for z in &self.core_zones {
            if z.owner>=self.players.len()||!is_core(&z.definition)||!owners.insert(z.owner) {return Err("核心区须有唯一拥有者及已实现核心身份".into());}
            if self.batch3_physical_cards().filter(|c|c.owner==z.owner&&c.definition==z.definition).count()>1 {return Err("所选核心不能同时存在多个物理副本".into());}
            if let Some(c)=&z.card {
                if c.definition!=z.definition||c.owner!=z.owner||c.controller!=z.owner||c.face_down||c.exhausted
                    ||c.damage!=0||c.wounds!=0||c.shield!=0||c.lock_markers!=0||c.time_markers!=0||c.charge_markers!=0
                    ||c.permanent_combat_markers!=0||c.permanent_defense_markers!=0
                    ||!self.mystic_instance_valid(&c.id)
                    ||self.batch3_physical_cards().filter(|live|live.id==c.id).count()!=1 {
                    return Err("核心区实体、拥有者、跨区状态或唯一性无效".into());
                }
                self.mobility_immutable_identity(c)?;
            }
        }
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
        let s=self.selected_core_source(seat)?;
        if s.card.definition!=crate::bq046::ID {return Err("仅实现莫佳娜核心预测3".into());}
        if self.pending.is_some() {return Err("不能覆盖尚未完成的选择".into());}
        self.effects.push_back(Effect::Forecast{seat,amount:3,draw:false});
        Ok(())
    }
    pub(crate) fn core_zone_views(&self,viewer:usize)->Vec<CoreZoneView> {
        self.core_zones.iter().map(|z|CoreZoneView{player_id:player_id(z.owner),definition:z.definition.clone(),
            card:z.card.as_ref().map(|c|self.card_view(c,viewer,None,Some("core")))}).collect()
    }
}
