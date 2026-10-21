//! Martial Law distinguishes playing/moving a character from concealment,
//! paid reveal and direct placement. Official FAQ physical page 4 confirms it.
use crate::{catalog,engine::RuleResult,model::*,rules::*};
const TITLE:&str="戒严：支付1，否则牺牲本牌";
fn same<T:serde::Serialize>(a:&T,b:&T)->bool{serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap()}
pub(crate) fn definition()->Definition{
    let mut d=crate::rules::attachment_definition(target(Zone::Region,EntityKind::Any,Relation::Any,Range::Anywhere));
    d.abilities[0].key="batch3-jc064-attach".into();d.modifiers.push(StaticModifier::JC064MartialLaw);
    d.abilities.push(ability("batch3-jc064-prepare",TITLE,Timing::Fast,vec![],vec![],vec![Op::Batch3Prepare{event_id:None}],Some(Event::Prepare)));d
}
fn options()->Vec<ChoiceOption>{vec![ChoiceOption{id:"pay".into(),label:"支付1".into(),card:None},ChoiceOption{id:"sacrifice".into(),label:"牺牲戒严".into(),card:None}]}
impl Game{
    pub(crate) fn jc064_blocks_character(&self,c:&Card,region:usize)->bool{
        !c.face_down && catalog::card(&c.definition).kind=="character" && catalog::card(&c.definition).color!="灰"
            && self.regions.get(region).is_some_and(|r|!r.vacant && self.attachments.iter().any(|a|a.card.definition=="JC064" && !a.card.face_down && a.host_id==r.card.id && self.attachment_host_valid(a)))
    }
    pub(crate) fn jc064_upkeep_start(&mut self,f:&ResolutionFrame)->RuleResult<()>{
        if self.resources(f.actor)==0{self.remove_dead(&f.source.card.id,RemovalCause::Sacrifice);}
        else{self.choice(f.actor,"jc064-upkeep",TITLE.into(),options(),1,1,None,ChoiceResolution::Frame{frame:Box::new(f.clone()),choice:FrameChoice::JC064Upkeep});}Ok(())
    }
    pub(crate) fn validate_jc064_pending(&self,p:&Pending,f:&ResolutionFrame)->RuleResult<()>{
        self.validate_jc035_frame(f)?;
        if f.source.card.definition!="JC064" || f.cursor!=1 || !matches!(f.guard,GuardState::Accepted) || self.resources(f.actor)==0
            || p.seat!=f.actor || p.choice.player_id!=player_id(f.actor) || p.choice.id.is_empty() || p.choice.kind!="jc064-upkeep" || p.choice.title!=TITLE
            || p.choice.min!=Some(1) || p.choice.max!=Some(1) || p.choice.allow_decline!=Some(false) || p.choice.amount.is_some() || !p.choice.preview_cards.is_empty() || !same(&p.choice.options,&options()){
            return Err("戒严维护须保留强制准备观察、原付款者和支付或牺牲选项".into());}Ok(())
    }
    pub(crate) fn jc064_upkeep_complete(&mut self,f:&ResolutionFrame,chooser:usize,a:&Action,selected:&[String])->RuleResult<()>{
        self.validate_jc035_frame(f)?;
        if chooser!=f.actor || f.source.card.definition!="JC064" || f.cursor!=1 || selected.len()!=1
            || *a!=(Action{choice_id:a.choice_id.clone(),selected:a.selected.clone(),..Action::new("choose")}){return Err("戒严维护确认来源或付款者不符".into());}
        match selected[0].as_str(){"pay"=>self.pay(f.actor,1)?,"sacrifice"=>self.remove_dead(&f.source.card.id,RemovalCause::Sacrifice),_=>return Err("须支付1或牺牲戒严".into())}Ok(())
    }
}

#[cfg(test)]mod tests{
    use super::*;use crate::unit_support::*;
    fn law(g:&mut Game,region:usize)->String{
        fund(g,0,"JC057",2);let id=held(g,"JC064",0);act(g,0,Action{card_id:Some(id),region:Some(region),..Action::new("play")});top(g);
        g.attachments.iter().find(|a|a.card.definition=="JC064").unwrap().card.id.clone()
    }
    fn pending(g:&mut Game){for _ in 0..32{if g.pending.is_some(){restore(g);return;}pass(g);}panic!("upkeep choice missing");}
    #[test]fn batch3_jc064_paid_attachment_blocks_every_non_gray_deployment_but_allows_conceal_reveal_and_direct_entry(){
        let mut g=game();let id=law(&mut g,0);fund(&mut g,0,"JC029",5);let blue=held(&mut g,"JC029",0);let gray=held(&mut g,"JC057",0);
        reject(&mut g,0,Action{card_id:Some(blue.clone()),region:Some(0),..Action::new("deploy")});
        assert!(g.legal_actions(0).iter().any(|a|a.action.card_id.as_deref()==Some(&gray) && a.action.kind=="deploy" && a.action.region==Some(0)));
        act(&mut g,0,Action{card_id:Some(blue),region:Some(0),..Action::new("conceal")});
        let hidden=g.regions[0].cards[0].id.clone();act(&mut g,0,Action{card_id:Some(hidden),..Action::new("reveal")});top(&mut g);
        assert!(g.regions[0].cards.iter().any(|c|c.definition=="JC029" && !c.face_down));
        let grave=g.make_card("JC125",0);let grave_id=grave.id.clone();g.players[0].graveyard.push(grave);g.world_graveyard_entry(0,0,&grave_id).unwrap();
        assert!(g.regions[0].cards.iter().any(|c|c.definition=="JC125"));
        g.attachments.iter_mut().find(|a|a.card.id==id).unwrap().card.exhausted=true;
        let blue_print=g.make_card("JC029",0);assert!(g.jc064_blocks_character(&blue_print,0));restore(&mut g);
    }
    #[test]fn batch3_jc064_response_attachment_cancels_pending_paid_deploy_without_character_death(){
        let mut g=game();fund(&mut g,0,"JC084",1);let black=held(&mut g,"JC084",0);
        act(&mut g,0,Action{card_id:Some(black),region:Some(0),..Action::new("deploy")});
        let law=g.make_card("JC064",2);g.attachments.push(Attachment{card:law,host_id:g.regions[0].card.id.clone()});top(&mut g);
        assert!(g.regions[0].cards.is_empty());assert!(g.players[0].assets[0].exhausted);assert!(g.players[0].graveyard.iter().any(|c|c.definition=="JC084"));restore(&mut g);
    }
    #[test]fn batch3_jc064_movement_denial_uses_current_face_and_source_departure_restores_destination(){
        let mut g=game();let law=law(&mut g,1);let blue=field(&mut g,"JC029",0);let gray=field(&mut g,"JC057",0);let hidden=field(&mut g,"JC125",2);g.board_mut(&hidden).unwrap().face_down=true;
        g.move_on_board_instance(&blue,1);assert_eq!(g.board(&blue).unwrap().0,0);
        g.move_on_board_instance(&gray,1);assert_eq!(g.board(&gray).unwrap().0,1);
        g.move_on_board_instance(&hidden,1);assert_eq!(g.board(&hidden).unwrap().0,1);
        g.remove_dead(&law,RemovalCause::Destroy);g.move_on_board_instance(&blue,1);assert_eq!(g.board(&blue).unwrap().0,1);restore(&mut g);
    }
    #[test]fn batch3_jc064_forced_upkeep_cannot_be_declined_and_uses_original_controller_after_response(){
        for pay in [false,true]{
            let mut g=game();let source=law(&mut g,0);fund(&mut g,0,"JC125",1);g.turn+=1;g.begin_window(Window::Prepare);g.drive().unwrap();
            assert_eq!(g.pending.as_ref().unwrap().choice.min,Some(1));let p=g.pending.clone().unwrap();reject(&mut g,0,Action{choice_id:Some(p.choice.id),selected:Some(vec![]),..Action::new("choose")});
            choose(&mut g,vec![source.clone()]);g.attachments.iter_mut().find(|a|a.card.id==source).unwrap().card.controller=2;pending(&mut g);
            let p=g.pending.clone().unwrap();reject(&mut g,2,Action{choice_id:Some(p.choice.id),selected:Some(vec!["pay".into()]),..Action::new("choose")});
            choose(&mut g,vec![if pay{"pay".into()}else{"sacrifice".into()}]);
            assert_eq!(g.attachments.iter().any(|a|a.card.id==source),pay);assert_eq!(g.players[0].assets.iter().filter(|c|!c.exhausted).count(),usize::from(!pay));restore(&mut g);
        }
    }
    #[test]fn batch3_jc064_unfunded_forced_upkeep_sacrifices_exact_source_without_a_bogus_choice(){
        let mut g=game();let source=law(&mut g,0);g.turn+=1;g.begin_window(Window::Prepare);g.drive().unwrap();choose(&mut g,vec![source.clone()]);top(&mut g);
        assert!(g.pending.is_none());assert!(!g.attachments.iter().any(|a|a.card.id==source));assert!(g.players[0].graveyard.iter().any(|c|c.definition=="JC064"));restore(&mut g);
    }
}
