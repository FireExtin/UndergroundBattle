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
