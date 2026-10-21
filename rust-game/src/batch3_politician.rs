//! XQ22's printed ability may be activated by every player. The payer stays
//! separate from the original controller and chooses orientation at resolution.
use crate::{catalog,engine::RuleResult,model::*,rules::*};
use std::collections::BTreeSet;
const KEY:&str="batch3-xq22-orient";
const TITLE:&str="州议员：选择横置或重置原角色";
fn same<T:serde::Serialize>(a:&T,b:&T)->bool{serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap()}
pub(crate) fn definition()->Definition{
    let mut a=ability(KEY,"快速行动2：横置或重置本牌，任何玩家可发动",Timing::Fast,vec![Cost::Assets(2)],vec![],vec![Op::XQ22Orientation],None);a.activation_only=true;
    let mut d=crate::rules::with_abilities(vec![a]);d.traits.public=true;d.modifiers.push(StaticModifier::XQ22UnexhaustedSpellBan);d
}
pub(crate) fn validate_action(a:&Action)->RuleResult<()>{
    if a.kind!="activate" || a.ability_id.as_deref().is_some_and(|id|id!=KEY)
        || *a!=(Action{card_id:a.card_id.clone(),ability_id:a.ability_id.clone(),..Action::new("activate")}){
        return Err("州议员能力只指定原来源，结算时选择横置或重置".into());
    }Ok(())
}
fn options()->Vec<ChoiceOption>{vec![ChoiceOption{id:"exhaust".into(),label:"横置".into(),card:None},ChoiceOption{id:"ready".into(),label:"重置".into(),card:None}]}
impl Game{
    pub(crate) fn xq22_spell_banned(&self)->bool{
        self.regions.iter().filter(|r|!r.vacant).flat_map(|r|&r.cards)
            .any(|c|c.definition=="XQ22" && !c.face_down && !c.exhausted)
    }
    pub(crate) fn validate_xq22_frame(&self,f:&ResolutionFrame)->RuleResult<()>{
        let c=&f.source.card;let d=definition();
        if c.definition!="XQ22" || c.id.is_empty() || c.face_down || c.owner>=self.players.len() || c.controller>=self.players.len() || f.actor>=self.players.len()
            || f.source.region.is_none_or(|r|r>=self.regions.len()) || f.source.source_region_instance.as_ref().is_none_or(|id|id.is_empty() || *id==c.id)
            || f.source.attachment_host_instance.is_some() || f.source.play_source.is_some() || f.source.observed_death.is_some() || f.source.observed_exposure.is_some() || f.source.observed_mobility_grant.is_some()
            || f.source.region.zip(f.source.source_region_instance.as_ref()).is_some_and(|(r,id)|self.regions.iter().enumerate().any(|(i,region)|region.card.id==*id && i!=r))
            || f.frame_id.is_empty() || f.ability_key!=KEY || f.chosen_region.is_some() || !f.targets.is_empty() || f.cursor>1
            || (f.cursor==0 && !matches!(f.guard,GuardState::Unchecked)) || (f.cursor==1 && !matches!(f.guard,GuardState::Accepted))
            || !same(&f.steps,&vec![Step{context:f.actor,op:d.abilities[0].ops[0].clone()}])
            || !matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)] if ids.len()==2 && ids.iter().all(|id|!id.is_empty() && *id!=c.id) && ids.iter().collect::<BTreeSet<_>>().len()==2 && !self.players.iter().enumerate().any(|(seat,p)|seat!=f.actor && p.assets.iter().any(|asset|ids.contains(&asset.id))))
            || self.players.iter().any(|p|p.hand.iter().chain(&p.deck).chain(&p.graveyard).chain(&p.assets).chain(&p.score_cards).any(|other|other.id==c.id)){
            return Err("州议员须保留任意玩家实际支付2、原公开来源和一次执行程序".into());
        }
        self.mobility_immutable_identity(c)
    }
    pub(crate) fn xq22_orientation_start(&mut self,f:ResolutionFrame)->RuleResult<()>{
        self.validate_xq22_frame(&f)?;
        if !self.board(&f.source.card.id).is_some_and(|(_,c)|c.definition=="XQ22" && !c.face_down){return Ok(());}
        self.choice(f.actor,"xq22-orientation",TITLE.into(),options(),1,1,None,ChoiceResolution::Frame{frame:Box::new(f),choice:FrameChoice::XQ22Orientation});Ok(())
    }
    pub(crate) fn validate_xq22_pending(&self,p:&Pending,f:&ResolutionFrame)->RuleResult<()>{
        self.validate_xq22_frame(f)?;
        if f.cursor!=1 || p.seat!=f.actor || p.choice.player_id!=player_id(f.actor) || p.choice.id.is_empty()
            || p.choice.kind!="xq22-orientation" || p.choice.title!=TITLE || p.choice.min!=Some(1) || p.choice.max!=Some(1) || p.choice.allow_decline!=Some(false)
            || p.choice.amount.is_some() || !p.choice.preview_cards.is_empty() || !same(&p.choice.options,&options())
            || !self.board(&f.source.card.id).is_some_and(|(_,c)|c.definition=="XQ22" && !c.face_down){return Err("州议员结算选择的付款者、来源或横置选项不符".into());}Ok(())
    }
    pub(crate) fn xq22_orientation_complete(&mut self,f:&ResolutionFrame,chooser:usize,a:&Action,selected:&[String])->RuleResult<()>{
        self.validate_xq22_frame(f)?;
        if f.cursor!=1 || chooser!=f.actor || selected.len()!=1 || !matches!(selected[0].as_str(),"exhaust"|"ready")
            || *a!=(Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")}){return Err("州议员只接受原付款者的横置或重置选择".into());}
        if let Some(c)=self.board_mut(&f.source.card.id).filter(|c|c.definition=="XQ22" && !c.face_down){c.exhausted=selected[0]=="exhaust";}
        Ok(())
    }
}

#[cfg(test)] mod tests{
    use super::*;use crate::unit_support::*;
    fn action(id:&str)->Action{Action{card_id:Some(id.into()),..Action::new("activate")}}
    fn pending(g:&mut Game){for _ in 0..32{if g.pending.is_some(){restore(g);return;}pass(g);}panic!("orientation choice missing");}
    #[test] fn batch3_xq22_all_four_payers_can_activate_enemy_or_own_source_and_choose_after_response(){
        for payer in 0..4{
            let mut g=game();let source=field(&mut g,"XQ22",2);fund(&mut g,payer,"JC125",2);g.begin_window(Window::Action(g.team(payer)));
            assert!(g.legal_actions(payer).iter().any(|a|a.action.kind=="activate" && a.action.card_id.as_deref()==Some(&source)));
            act(&mut g,payer,action(&source));let f=g.stack.last().unwrap().frame.as_ref().unwrap();assert_eq!(f.actor,payer);assert_eq!(f.source.card.controller,2);
            assert_eq!(g.players[payer].assets.iter().filter(|c|c.exhausted).count(),2);assert!(!g.board(&source).unwrap().1.exhausted);
            g.board_mut(&source).unwrap().controller=3;pending(&mut g);
            let p=g.pending.clone().unwrap();reject(&mut g,(payer+1)%4,Action{choice_id:Some(p.choice.id),selected:Some(vec!["exhaust".into()]),..Action::new("choose")});
            choose(&mut g,vec!["exhaust".into()]);assert!(g.board(&source).unwrap().1.exhausted);assert!(!g.xq22_spell_banned());restore(&mut g);
        }
    }
    #[test] fn batch3_xq22_spell_ban_is_global_live_and_does_not_block_attachments_or_activations(){
        let mut g=game();let source=field(&mut g,"XQ22",0);
        for seat in 0..4{
            fund(&mut g,seat,"JC057",3);let spell=held(&mut g,"JC062",seat);g.begin_window(Window::Action(g.team(seat)));
            reject(&mut g,seat,Action{card_id:Some(spell),selected:Some(vec![]),..Action::new("play")});
        }
        g.begin_window(Window::Action(0));fund(&mut g,0,"JC029",3);let attachment=held(&mut g,"JC035",0);
        assert!(g.legal_actions(0).iter().any(|a|a.action.card_id.as_deref()==Some(&attachment) && a.action.kind=="play"));
        act(&mut g,0,Action{card_id:Some(attachment),region:Some(0),..Action::new("play")});top(&mut g);
        assert!(g.xq22_spell_banned());g.board_mut(&source).unwrap().exhausted=true;assert!(!g.xq22_spell_banned());
        assert!(g.legal_actions(0).iter().any(|a| a.action.kind=="play" && g.players[0].hand.iter().any(|c|Some(&c.id)==a.action.card_id.as_ref() && c.definition=="JC062")));
        act(&mut g,0,action(&source));pending(&mut g);choose(&mut g,vec!["ready".into()]);assert!(g.xq22_spell_banned());
        g.board_mut(&source).unwrap().face_down=true;assert!(!g.xq22_spell_banned());g.board_mut(&source).unwrap().face_down=false;
        g.remove_dead(&source,RemovalCause::Destroy);assert!(!g.xq22_spell_banned());restore(&mut g);
    }
    #[test] fn batch3_xq22_missing_original_source_keeps_paid_cost_and_cannot_orient_same_name_replacement(){
        let mut g=game();let source=field(&mut g,"XQ22",2);fund(&mut g,0,"JC125",2);act(&mut g,0,action(&source));
        g.remove_dead(&source,RemovalCause::Destroy);let replacement=field(&mut g,"XQ22",2);top(&mut g);
        assert!(g.pending.is_none());assert!(!g.board(&replacement).unwrap().1.exhausted);assert!(g.players[0].assets.iter().all(|c|c.exhausted));restore(&mut g);
    }
    #[test] fn batch3_xq22_unpaid_forged_actor_and_polluted_choices_are_rejected(){
        let mut g=game();let source=field(&mut g,"XQ22",2);reject(&mut g,0,action(&source));fund(&mut g,0,"JC125",2);act(&mut g,0,action(&source));
        let mut bad=g.clone();bad.stack.last_mut().unwrap().frame.as_mut().unwrap().already_paid.clear();assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
        let mut bad=g.clone();let s=bad.stack.last_mut().unwrap();s.controller=3;let f=s.frame.as_mut().unwrap();f.actor=3;f.steps[0].context=3;assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
        pending(&mut g);let p=g.pending.clone().unwrap();reject(&mut g,0,Action{choice_id:Some(p.choice.id),selected:Some(vec!["exhaust".into()]),option:Some("ready".into()),..Action::new("choose")});
        let mut bad=g.clone();bad.pending.as_mut().unwrap().seat=2;assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
        assert!(crate::batch3::validate_definition("JC125",&definition()).is_err());
    }
}
