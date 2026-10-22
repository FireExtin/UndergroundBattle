//! Disclosed native layouts; every command uses apply plus exact restore/views.
use crate::{batch4_tattoo as tattoo, catalog, model::*, rules::*, unit_support::*};

fn setup() -> (Game, String, String) {
    let mut g = game(); let host = field(&mut g, "JC125", 0);
    fund(&mut g, 0, "JZ39", 4); let id = held(&mut g, "JZ39", 0); play(&mut g, &id, 0, &host);
    let source = g.attachments.iter().find(|a| a.card.definition == "JZ39").unwrap().card.id.clone();
    (g, host, source)
}
fn activate(g: &mut Game, source: &str, demon: &str, seat: usize) {
    act(g, 0, Action { card_id: Some(source.into()), ability_id: Some(tattoo::KEY.into()),
        target_id: Some(player_id(seat)), cost_selected: Some(vec![demon.into()]), ..Action::new("activate") });
}
fn until_choice(g: &mut Game) {
    for _ in 0..64 {
        if g.pending.is_some() { restore(g); return; }
        if !g.effects.is_empty() { g.drive().unwrap(); } else { pass(g); }
    }
    panic!("bounded tattoo enemy sacrifice");
}

#[test]
fn batch4_tattoo_exact_print_whole_definition_and_nested_cost_program_cannot_be_transplanted() {
    let c = catalog::card("JZ39"); assert_eq!((c.kind.as_str(),c.cost,c.magic.as_str(),c.supported), ("attachment",2,"鲜血",false));
    assert_eq!(c.subtypes,["纹身"]); assert_eq!(c.loyalty,["红色"]);
    let d = tattoo::definition(); assert_eq!(d.abilities.len(),2); assert_eq!(d.attachment.as_ref().unwrap().host.subtype.as_deref(),Some("人类"));
    for n in 0..5 {
        let mut bad = d.clone(); match n {
            0 => bad.abilities[1].costs = vec![Cost::Assets(1)],
            1 => bad.abilities[1].timing = Timing::Fast,
            2 => bad.abilities[1].targets[0].relation = Relation::Any,
            3 => bad.attachment.as_mut().unwrap().host.subtype = None,
            _ => bad.modifiers.clear(),
        }
        assert!(crate::batch4::validate_definition("JZ39", &bad).is_err());
    }
    let mut a = d.abilities[1].clone(); a.key = "foreign".into(); a.ops = vec![Op::ForEachLivingPlayer(vec![Op::JZ39EnemySacrifice])];
    assert!(crate::rules::validate_ability("JC125",&a).is_err());
    let mut a = d.abilities[1].clone(); a.ops.clear(); a.key = "foreign".into(); assert!(crate::batch4::validate_ability("JC125",&a).is_err());
}

#[test]
fn batch4_tattoo_actual_attach_adds_permanent_combat_kill_and_demon_to_human_with_four_seat_projection() {
    let (mut g, host, source) = setup(); let c = g.board(&host).unwrap().1;
    assert_eq!(g.current_permanent_combat(c,0),catalog::card("JC125").permanent_icons.combat + 1);
    assert_eq!(g.effective_kill(c),1); assert!(g.current_subtypes(c).iter().any(|s| s=="恶魔"));
    assert!(!catalog::card("JC125").subtypes.iter().any(|s| s=="恶魔"));
    for seat in 0..4 {
        let v = g.view(seat); let c = v.regions[0].characters.iter().find(|c|c.instance_id==host).unwrap();
        assert_eq!(c.current_kill,Some(1));
        assert!(c.current_subtypes.as_ref().unwrap().iter().any(|s|s=="恶魔"));
    }
    g.attachments.iter_mut().find(|a|a.card.id==source).unwrap().card.controller=2;
    assert_eq!(g.effective_kill(g.board(&host).unwrap().1),1); restore(&mut g);
}

#[test]
fn batch4_tattoo_nonhuman_hidden_and_unknown_payloads_cannot_attach_or_spend() {
    let mut g = game(); let beast = field(&mut g,"XG15",0); let human = field(&mut g,"JC125",0);
    g.board_mut(&human).unwrap().face_down=true; let id = held(&mut g,"JZ39",0); fund(&mut g,0,"JZ39",4);
    for host in [beast,human] {
        reject(&mut g,0,Action { card_id:Some(id.clone()),target_id:Some(host),..Action::new("play") });
        assert_eq!(g.resources(0),4);
    }
    let human = field(&mut g,"JC125",0);
    reject(&mut g,0,Action {card_id:Some(id),target_id:Some(human),region:Some(0),..Action::new("play")});
}

#[test]
fn batch4_tattoo_paid_own_host_death_keeps_the_ability_and_enemy_chooses_once_after_restore() {
    let (mut g, host, source) = setup(); let victim = field(&mut g,"JC125",2); let other = field(&mut g,"JC125",3);
    activate(&mut g,&source,&host,2); assert!(g.board(&host).is_none()); assert!(g.attachments.iter().all(|a|a.card.id!=source));
    assert_eq!(g.resources(0),1); assert_eq!(g.stack.len(),1); until_choice(&mut g);
    let p = g.pending.clone().unwrap(); assert_eq!(p.seat,2); assert_eq!(p.choice.options.iter().map(|o|o.id.clone()).collect::<Vec<_>>(),[victim.clone()]);
    for seat in [0,1,3] { assert!(g.view(seat).pending_choice.as_ref().is_none_or(|p|p.options.is_empty())); }
    reject(&mut g,0,Action {choice_id:Some(p.choice.id.clone()),selected:Some(vec![victim.clone()]),..Action::new("choose")});
    reject(&mut g,2,Action {choice_id:Some(p.choice.id.clone()),selected:Some(vec![other]),..Action::new("choose")});
    reject(&mut g,2,Action {choice_id:Some(p.choice.id.clone()),selected:Some(vec![]),..Action::new("choose")});
    reject(&mut g,2,Action {choice_id:Some(p.choice.id),selected:Some(vec![victim.clone()]),target_id:Some("p3".into()),..Action::new("choose")});
    choose(&mut g,vec![victim.clone()]); assert!(g.board(&victim).is_none()); assert_eq!(g.resources(0),1); restore(&mut g);
}

#[test]
fn batch4_tattoo_cost_rejects_ally_non_demon_hidden_duplicates_and_forged_payload_atomically() {
    let (mut g, host, source) = setup(); let ordinary = field(&mut g,"JC125",0); let allied = field(&mut g,"JC125",1);
    let card = g.make_card("JZ39",1); g.attachments.push(Attachment {card,host_id:allied.clone()});
    let hidden = field(&mut g,"JC125",0); g.board_mut(&hidden).unwrap().face_down=true;
    for ids in [vec![],vec![ordinary],vec![allied],vec![hidden],vec![host.clone(),host.clone()]] {
        reject(&mut g,0,Action {card_id:Some(source.clone()),ability_id:Some(tattoo::KEY.into()),target_id:Some("p2".into()),cost_selected:Some(ids),..Action::new("activate")});
        assert_eq!(g.resources(0),2); assert!(!g.attachments.iter().find(|a|a.card.id==source).unwrap().card.exhausted);
    }
    reject(&mut g,0,Action {card_id:Some(source.clone()),ability_id:Some(tattoo::KEY.into()),target_id:Some("p1".into()),cost_selected:Some(vec![host.clone()]),..Action::new("activate")});
    reject(&mut g,0,Action {card_id:Some(source),ability_id:Some(tattoo::KEY.into()),target_id:Some("p2".into()),cost_selected:Some(vec![host]),option:Some("foreign".into()),..Action::new("activate")});
}

#[test]
fn batch4_tattoo_enemy_without_characters_still_resolves_paid_cost_without_a_choice() {
    let (mut g,host,source)=setup(); activate(&mut g,&source,&host,2); top(&mut g);
    assert!(g.pending.is_none()); assert_eq!(g.resources(0),1); assert!(g.board(&host).is_none()); restore(&mut g);
}

#[test]
fn batch4_tattoo_legal_commands_enumerate_personal_demon_cost_and_only_enemy_players() {
    let (mut g,host,source)=setup(); let actions = g.legal_actions(0).into_iter().filter(|a|a.action.kind=="activate"&&a.action.card_id.as_deref()==Some(&source)).map(|a|a.action).collect::<Vec<_>>();
    assert_eq!(actions.len(),2); assert!(actions.iter().all(|a|a.cost_selected==Some(vec![host.clone()])&&matches!(a.target_id.as_deref(),Some("p2"|"p3"))));
    restore(&mut g);
}

#[test]
fn batch4_tattoo_saved_payment_program_and_source_cannot_be_rewritten_or_reused() {
    let (mut g,host,source)=setup(); activate(&mut g,&source,&host,2);
    for mode in 0..7 {
        let mut bad=g.clone(); let f=bad.stack[0].frame.as_mut().unwrap(); match mode {
            0=>f.steps[0].context=2,
            1=>f.ability_key="foreign".into(),
            2=>if let PaidCost::JZ39Demon(p)=&mut f.already_paid[2] {p.tattoos.clear();},
            3=>f.already_paid[1]=PaidCost::Exhausted("foreign".into()),
            4=>if let PaidCost::JZ39Demon(p)=&mut f.already_paid[2] {p.card.controller=1;},
            5=>{let copy=bad.stack[0].clone();bad.stack.push(copy);},
            _=>{let copy=f.source.card.clone();bad.players[0].hand.push(copy);},
        }
        assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    }
}

#[test]
fn batch4_tattoo_pending_victim_choice_metadata_and_continuation_are_closed() {
    let (mut g,host,source)=setup(); field(&mut g,"JC125",2); activate(&mut g,&source,&host,2); until_choice(&mut g);
    for mode in 0..4 {
        let mut bad=g.clone(); let p=bad.pending.as_mut().unwrap(); match mode {
            0=>p.seat=0,
            1=>p.choice.allow_decline=Some(true),
            2=>p.choice.options[0].id="foreign".into(),
            _=>if let ChoiceResolution::Frame {frame,..}=&mut p.resolution {frame.cursor=0;frame.guard=GuardState::Unchecked;},
        }
        assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    }
}

#[test]
fn batch4_tattoo_printed_demon_payment_keeps_the_tattoo_but_its_exhaustion_blocks_reuse() {
    let (mut g,host,source)=setup(); let demon=field(&mut g,"JZ33",0);
    activate(&mut g,&source,&demon,2); assert!(g.board(&host).is_some());
    assert!(g.attachments.iter().find(|a|a.card.id==source).unwrap().card.exhausted);
    assert!(matches!(g.stack[0].frame.as_ref().unwrap().already_paid[2],PaidCost::JZ39Demon(ref p)if p.tattoos.is_empty()));
    top(&mut g); let remaining=g.resources(0);
    reject(&mut g,0,Action {card_id:Some(source),ability_id:Some(tattoo::KEY.into()),target_id:Some("p2".into()),cost_selected:Some(vec![host.clone()]),..Action::new("activate")});
    assert!(g.board(&host).is_some()); assert_eq!(g.resources(0),remaining); restore(&mut g);
}

#[test]
fn batch4_tattoo_enemy_may_sacrifice_an_attached_dual_kind_character_without_its_host() {
    let (mut g,host,source)=setup(); let target_host=field(&mut g,"JC125",2); let bat=g.make_card("BQ026",2); let victim=bat.id.clone();
    g.attachments.push(Attachment {card:bat,host_id:target_host.clone()});
    assert!(g.attachment_host_valid(g.attachments.last().unwrap()));
    activate(&mut g,&source,&host,2); until_choice(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.options.len(),2);
    assert!(g.pending.as_ref().unwrap().choice.options.iter().any(|o|o.id==victim));
    choose(&mut g,vec![victim.clone()]); assert!(g.attachments.iter().all(|a|a.card.id!=victim));
    assert!(g.board(&target_host).is_some()); restore(&mut g);
}

#[test]
fn batch4_tattoo_real_hide_ends_combat_kill_and_demon_and_does_not_leak_the_hidden_successor() {
    let (mut g,old,source)=setup(); g.world_hide(None,false); g.drive().unwrap();
    assert!(g.board(&old).is_none()); assert!(g.attachments.iter().all(|a|a.card.id!=source));
    let hidden=g.regions[0].cards.iter().find(|c|c.face_down&&c.controller==0).unwrap();
    assert_eq!(g.effective_kill(hidden),0); assert!(g.current_subtypes(hidden).is_empty());
    let id=hidden.id.clone();
    for seat in 1..4 {let v=g.view(seat);let c=v.regions[0].characters.iter().find(|c|c.instance_id==id).unwrap();assert!(c.current_subtypes.is_none());assert!(c.current_kill.is_none());}
    restore(&mut g);
}

#[test]
fn batch4_tattoo_actual_human_to_vampire_change_removes_ineligible_tattoo_and_all_its_grants() {
    let (mut g,host,source)=setup(); fund(&mut g,0,"JC036",6); let change=held(&mut g,"JC036",0);
    play(&mut g,&change,0,&host); let c=g.board(&host).unwrap().1;
    assert!(g.current_subtypes(c).iter().any(|s|s=="吸血鬼"));
    assert!(!g.current_subtypes(c).iter().any(|s|s=="人类"||s=="恶魔"));
    assert_eq!(g.effective_kill(c),0); assert_eq!(g.current_permanent_combat(c,0),catalog::card("JC125").permanent_icons.combat);
    assert!(g.attachments.iter().all(|a|a.card.id!=source)); restore(&mut g);
}
