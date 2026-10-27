//! BQ046 printed ordinary action. Core invocation is a separate program;
//! it must not borrow this action's honor cost or become a free player action.
use crate::{engine::RuleResult, model::*, rules::*};
pub(crate) const ID:&str="BQ046";
pub(crate) const ACTION:&str="bq046-bottom-hand";
pub(crate) const CORE:&str="bq046-core-forecast";
fn same<T:serde::Serialize>(a:&T,b:&T)->bool { serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap() }
pub(crate) fn definition()->Definition {
    let mut a=ability(ACTION,"支付1名誉：牌库底一张置于手中（每回合一次）",Timing::Standard,
        vec![Cost::BQ046Honor],vec![],vec![Op::MoveBottomToHand{player:PlayerRef::Actor,count:1}],None);
    a.activation_only=true; a.per_turn_limit=Some(1);
    with_abilities(vec![a])
}
// Not inserted in ordinary abilities: only a verified external core invocation
// may dispatch this printed program once core-zone/provenance integration lands.
#[allow(dead_code)]
pub(crate) fn core_program()->Vec<Op> { vec![Op::Forecast{player:PlayerRef::Actor,count:3}] }
pub(crate) fn related(a:&AbilitySpec)->bool {a.key.starts_with("bq046-")}
pub(crate) fn validate_ability(id:&str,a:&AbilitySpec)->RuleResult<()> {
    if id!=ID || !same(a,&definition().abilities[0]) {return Err("BQ046普通行动须保留名誉1、标准时序、每回合一次及取底程序；核心不可冒充普通行动".into());} Ok(())
}
pub(crate) fn validate_definition(id:&str,d:&Definition)->RuleResult<()> {
    if (id==ID||d.abilities.iter().any(related))&&(id!=ID||!same(d,&definition())) {return Err("BQ046印刷普通行动不能增删或移植".into());} Ok(())
}
pub(crate) fn validate_action(id:&str,a:&Action)->RuleResult<()> {
    if id!=ID {return Ok(());}
    if a.kind=="deploy" {
        if *a!=(Action{card_id:a.card_id.clone(),region:a.region,..Action::new("deploy")}) {return Err("核心派遣不能夹带额外载荷".into());}
        return Ok(());
    }
    let expected=Action{card_id:a.card_id.clone(),ability_id:a.ability_id.clone(),..Action::new("activate")};
    if *a!=expected||a.ability_id.as_deref()!=Some(ACTION) {return Err("BQ046当前只开放内部布局中的普通行动；核心出入区与外部发动尚未接入".into());}
    Ok(())
}
fn related_frame(f:&ResolutionFrame)->bool {f.ability_key.starts_with("bq046-")||f.already_paid.iter().any(|p|matches!(p,PaidCost::BQ046Honor{..}))}
impl Game {
    pub(crate) fn bq046_pay_honor(&mut self,actor:usize,s:&SourceSnapshot,a:&AbilitySpec,n:u32)->RuleResult<PaidCost>{
        validate_ability(&s.card.definition,a)?;
        if n!=1||actor!=s.card.controller||actor>=self.players.len()||s.card.face_down
            ||!self.board(&s.card.id).is_some_and(|(_,c)|c==&s.card) {return Err("BQ046名誉费用须来自本方实际正面在场角色".into());}
        let before=self.players[actor].honor;
        self.players[actor].honor=before.checked_sub(1).ok_or("BQ046名誉不足；货币和分数不能代替")?;
        Ok(PaidCost::BQ046Honor{before})
    }
    pub(crate) fn validate_bq046_frame(&self,f:&ResolutionFrame)->RuleResult<()> {
        if !related_frame(f) {return Ok(());}
        let c=&f.source.card;
        if c.definition!=ID||c.face_down||c.controller!=f.actor||f.actor>=self.players.len()||c.owner>=self.players.len()
            ||!self.mystic_instance_valid(&c.id)||!self.mystic_instance_valid(&f.frame_id)||c.id==f.frame_id
            ||f.source.region.is_none_or(|r|r>=self.regions.len())
            ||f.source.source_region_instance.as_deref().is_none_or(|id|!self.mystic_instance_valid(id))
            ||f.source.play_source.is_some()||f.source.attachment_host_instance.is_some()||f.source.death_echo.is_some()
            ||f.source.observed_death.is_some()||f.source.observed_exposure.is_some()||f.source.observed_mobility_grant.is_some()
            ||f.ability_key!=ACTION||f.cursor>1||!f.targets.is_empty()||f.chosen_region.is_some()||!f.resource_payments.is_empty()
            ||!same(&f.steps,&vec![Step{context:f.actor,op:definition().abilities[0].ops[0].clone()}])
            ||!matches!(f.already_paid.as_slice(),[PaidCost::BQ046Honor{before}]if *before>=1)
            ||f.cursor==0&&!matches!(f.guard,GuardState::Unchecked)||f.cursor==1&&!matches!(f.guard,GuardState::Accepted)
            ||!self.turn_ability_usage.iter().any(|u|u.source_instance==c.id&&u.ability_key==ACTION&&u.turn==self.turn&&u.uses==1) {
            return Err("BQ046原帧、名誉收据、次数或取底程序被改写".into());
        }
        self.mobility_immutable_identity(c)
    }
    pub(crate) fn validate_bq046_state(&self)->RuleResult<()> {
        let mut ids=std::collections::BTreeSet::new();
        for item in &self.stack {if let Some(f)=&item.frame {if related_frame(f) {
            self.validate_bq046_frame(f)?;
            if !ids.insert(&f.frame_id)||item.id!=f.frame_id||item.controller!=f.actor||item.card.is_some()||item.reveal||item.deploy_region.is_some()||item.target.is_some()||f.cursor!=0 {return Err("BQ046响应外壳或唯一帧不符".into());}
        }}}
        for e in &self.effects {if let Effect::Frame{frame}=e {if related_frame(frame) {return Err("BQ046不应存在独立取底续体".into());}}}
        if self.pending.as_ref().is_some_and(|p|matches!(&p.resolution,ChoiceResolution::Frame{frame,..}if related_frame(frame))) {return Err("BQ046普通取底行动没有选择续体".into());}
        let mut uses=std::collections::BTreeSet::new();
        for u in &self.turn_ability_usage {if u.ability_key.starts_with("bq046-")&&(u.ability_key!=ACTION||u.uses!=1||u.turn!=self.turn||!self.mystic_instance_valid(&u.source_instance)||!uses.insert(&u.source_instance)) {return Err("BQ046每回合次数记录不符".into());}}
        Ok(())
    }
}
