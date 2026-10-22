use crate::{model::*,unit_support::*};

#[test]
fn jc060_attached_bat_real_paid_entry_keeps_friendly_granted_domain_and_combat_in_four_views(){
    let mut g=game();let host=field(&mut g,"JC029",0);let advisor=field(&mut g,"JC060",1);fund(&mut g,0,"JC029",1);let id=held(&mut g,"BQ026",0);
    act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});
    for _ in 0..64{if g.pending.is_some(){break;}if !g.effects.is_empty(){g.drive().unwrap();}else{pass(&mut g);}}
    assert!(g.pending.as_ref().unwrap().choice.options.iter().any(|o|o.id==host));choose(&mut g,vec![host]);top(&mut g);let bat=g.attachments.iter().find(|a|a.card.definition=="BQ026").unwrap().card.id.clone();
    let c=g.board(&bat).unwrap().1;assert_eq!(g.current_magic_count(c),1,"Attaching does not remove character type or friendly continuous grants");assert_eq!(g.current_icons(c,0).combat,2);
    for seat in 0..4{let a=g.view(seat).attachments.into_iter().find(|a|a.card.instance_id==bat).unwrap();assert_eq!(a.card.current_domains,Some(vec![MagicDomainCount{magic:"心灵".into(),count:1}]));assert_eq!(a.card.icons.unwrap().combat,2);}restore(&mut g);
    g.remove_dead(&advisor,RemovalCause::Destroy);assert_eq!(g.current_magic_count(g.board(&bat).unwrap().1),0);assert_eq!(g.current_icons(g.board(&bat).unwrap().1,0).combat,1);
    for seat in 0..4{assert_eq!(g.view(seat).attachments.iter().find(|a|a.card.instance_id==bat).unwrap().card.current_domains,None);}restore(&mut g);
}

#[test]
fn jc060_attached_current_room_entry_and_real_hide_response_export(){
    use crate::{catalog,room::*,batch3_abi94_tests::step};
    fn finish(room:&mut RoomEnvelope,host:&str,steps:&mut Vec<serde_json::Value>){
        for _ in 0..64{
            if let Some(p)=room.game.pending.clone(){
                assert!(p.choice.options.iter().any(|o|o.id==host),"Only the actual bat attachment target choice is expected");
                step(room,p.seat,Action{choice_id:Some(p.choice.id),selected:Some(vec![host.into()]),..Action::new("choose")}.into(),steps);
            }else if let Some(w)=room.pacing.window.clone(){
                let seat=*w.members.iter().find(|(_,d)|matches!(d,Decision::Undecided{..})).unwrap().0;
                step(room,seat,SessionAction::PassResponse{window_id:w.id},steps);
            }else{assert!(room.game.stack.is_empty()&&room.game.effects.is_empty());return;}
        }panic!("bounded actual Room command chain");
    }
    let mut g=game();g.seed=u64::MAX-1009;g.random=g.seed;
    let host=field(&mut g,"JC029",0);let advisor=field(&mut g,"JC060",1);
    fund(&mut g,0,"JC029",1);fund(&mut g,0,"JC063",2);let bat_hand=held(&mut g,"BQ026",0);let chase=held(&mut g,"JC063",0);
    let seed=g.seed.to_string();let mut room=crate::jc029_tests::envelope(&g);let initial=serde_json::json!({"state":serde_json::to_string(&room).unwrap(),"views":(0..4).map(|s|room.view(s,0)).collect::<Vec<_>>()});let mut steps=vec![];
    step(&mut room,0,Action{card_id:Some(bat_hand),region:Some(0),..Action::new("deploy")}.into(),&mut steps);finish(&mut room,&host,&mut steps);
    let bat=room.game.attachments.iter().find(|a|a.card.definition=="BQ026").unwrap().card.id.clone();
    assert_eq!(room.game.current_magic_count(room.game.board(&bat).unwrap().1),1);
    for seat in 0..4{let view=room.view(seat,0);let a=view.attachments.iter().find(|a|a.card.instance_id==bat).unwrap();assert_eq!(a.card.current_domains,Some(vec![MagicDomainCount{magic:"心灵".into(),count:1}]));assert_eq!(a.card.icons.unwrap().combat,2);}
    step(&mut room,0,Action{card_id:Some(chase),target_id:Some(advisor.clone()),option:Some("hide".into()),..Action::new("play")}.into(),&mut steps);finish(&mut room,&host,&mut steps);
    assert!(room.game.board(&advisor).is_none());assert_eq!(room.game.current_magic_count(room.game.board(&bat).unwrap().1),0);
    assert!(room.game.attachments.iter().any(|a|a.card.id==bat&&a.host_id==host));
    for seat in 0..4{let view=room.view(seat,0);let a=view.attachments.iter().find(|a|a.card.instance_id==bat).unwrap();assert_eq!(a.card.current_domains,None);assert_eq!(a.card.icons.unwrap().combat,1);}
    if let Ok(path)=std::env::var("JC060_ATTACHED_NATIVE_OUT"){
        std::fs::write(path,serde_json::to_vec(&serde_json::json!({"format":"batch3-native94-integration-smoke-v1","method":"Fresh explicit current layout; actual paid bat deployment/attachment and paid JC063 Hide, current Room passes/choices and four restored private views; no natural browser acceptance","catalog":catalog::catalog(),"initial":initial,"steps":steps,"seedDecimal":seed,"attachedBatInstance":bat})).unwrap()).unwrap();
    }
}
