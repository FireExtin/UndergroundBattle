//! Disclosed offline layouts followed by paid commands and exact saved views.
//! These tests are not a natural UI match or edits to a production database.
use crate::jc029_tests::{apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject};
use crate::{catalog, deck, model::*, room::*, rules::*};

fn held(g: &mut Game, def: &str, owner: usize) -> String {
    let c = g.make_card(def, owner); let id = c.id.clone();
    g.players[owner].hand.push(c); id
}
fn curse(g: &mut Game, actor: usize, host: &str) -> String {
    let source = held(g, "JC089", actor);
    let action = g.legal_actions(actor).into_iter().find(|a|
        a.action.kind == "play" && a.action.card_id.as_deref() == Some(&source)
            && a.action.target_id.as_deref() == Some(host)).unwrap().action;
    apply(g, actor, action);
    assert!(g.attachments.iter().all(|a| a.card.definition != "JC089") || g.stack.len() == 1);
    fixture("attachment-stack", g);
    pass_top(g);
    g.attachments.iter().rev().find(|a| a.card.definition == "JC089")
        .map(|a| a.card.id.clone()).unwrap_or_default()
}
fn stats(g: &Game, id: &str) -> (u32, u32) {
    let (r,c) = g.board(id).unwrap(); (g.defense(c,r),g.current_icons(c,r).combat)
}
fn fixture(kind: &str, g: &Game) {
    if let Ok(dir)=std::env::var("JC089_FRONTEND_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let r=envelope(g);
        std::fs::write(format!("{dir}/{kind}.json"),serde_json::to_vec(&serde_json::json!({
            "kind":kind,"state":serde_json::to_string(&r).unwrap(),
            "views":(0..4).map(|s|r.view(s,0)).collect::<Vec<_>>()
        })).unwrap()).unwrap();
    } checkpoint(g);
}
fn passes(g: &mut Game, count: usize) {
    for _ in 0..count {
        let s=(0..4).find(|s|g.legal_actions(*s).iter().any(|a|a.action.kind=="pass")).unwrap();
        apply(g,s,Action::new("pass"));
    }
}
fn win(g: &mut Game, region: usize) {
    g.begin_window(Window::Before(region,1)); passes(g,4);
    if g.pending.as_ref().is_some_and(|p|p.choice.kind=="recipient") {
        let id=g.pending.as_ref().unwrap().choice.options[0].id.clone();choose(g,vec![id]);
    }
    if g.pending.as_ref().is_some_and(|p|p.choice.kind=="damage") {
        let p=g.pending.clone().unwrap();let id=p.choice.options[0].id.clone();
        apply(g,p.seat,Action{choice_id:Some(p.choice.id),
            allocations:Some([(id,p.choice.amount.unwrap())].into_iter().collect()),..Action::new("choose")});
    }
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,"trigger");
    let ChoiceResolution::Declare{declaration,..}=&g.pending.as_ref().unwrap().resolution else {panic!()};
    assert_eq!(declaration.ability.event,Some(Event::CombatWon));
    assert_eq!(declaration.ability.key,"jc089-combat-glory");
}
fn field(actor:usize) -> (Game,String,String) {
    let mut g=game(actor); fund(&mut g,actor,"JC084",8);
    let host=board(&mut g,"LC01",actor,2);
    let a=curse(&mut g,actor,&host); (g,host,a)
}

#[test]
fn jc089_exact_printed_curse_and_complete_attached_host_definition() {
    let c=catalog::card("JC089");
    assert_eq!((&*c.name,&*c.kind,c.cost,&*c.color), ("毒血诅咒","attachment",2,"黑"));
    assert_eq!(c.loyalty,["黑色"]); assert_eq!(c.magic_icon,MagicIcon::Blood);
    assert_eq!(c.subtypes,["诅咒"]); assert!(!c.unique&&c.keywords.is_empty());
    assert_eq!(c.permanent_icons,Icons::default()); assert_eq!(c.temporary_icons,Icons::default());
    assert_eq!(deck::copy_limit(c),Some(3)); assert_eq!(catalog::catalog().cards.len(),116);
    assert!(catalog::catalog().cards.iter().any(|c|c.id=="JZ50"));
    let d=definition("JC089"); let s=d.attachment.as_ref().unwrap();
    assert_eq!(s.host.relation,Relation::Any); assert_eq!(s.host.kind,EntityKind::Character);
    assert!(!s.host.equipment_host&&!s.controls_host); assert_eq!(s.host_defense_bonus,0);
    assert_eq!(s.host_icons,Icons{combat:1,..Icons::default()});
    assert_eq!(s.host_temporary_icons,Some(s.host_icons));
    assert!(matches!(d.modifiers.as_slice(),[StaticModifier::JC089HostDefenseMinusOneAndGlory]));
    assert!(!d.traits.renown); assert!(definition("JZ49").traits.slow);
}

#[test]
fn jc089_paid_attachment_all_hosts_and_separate_owners_controller_matrix() {
    for actor in 0..4 { for owner in 0..4 {
        let mut g=game(actor); fund(&mut g,actor,"JC084",2);
        let host=board(&mut g,"LC01",owner,2);
        let a=curse(&mut g,actor,&host); assert!(!a.is_empty());
        assert_eq!(stats(&g,&host).0,3); assert_eq!(g.resources(actor),0);
        let c=g.board(&host).unwrap().1;
        assert_eq!(c.owner,owner); assert_eq!(c.controller,owner);
        let a=g.attachments.iter().find(|x|x.card.id==a).unwrap();
        assert_eq!((a.card.owner,a.card.controller),(actor,actor));
        assert!(g.has_jc089_glory(c)); assert!(!g.has_renown(c)); checkpoint(&g);
    }}
}

#[test]
fn jc089_illegal_loyalty_cost_hidden_asset_attachment_targets_are_atomic() {
    for variant in 0..5 {
        let mut g=game(0); let host=board(&mut g,"LC01",2,2);
        let source=held(&mut g,"JC089",0);
        fund(&mut g,0,if variant==0 {"JC003"} else {"JC084"},if variant==1 {1} else {2});
        let target=match variant {
            2=>{g.board_mut(&host).unwrap().face_down=true;host},
            3=>g.players[0].assets[0].id.clone(),
            4=>g.regions[2].card.id.clone(),
            _=>host,
        };
        reject(&mut g,0,Action{card_id:Some(source),target_id:Some(target),..Action::new("play")});
    }
}

#[test]
fn jc089_defense_one_dies_immediately_and_both_graveyards_keep_original_owners() {
    for actor in 0..4 { for owner in 0..4 {
        let mut g=game(actor); fund(&mut g,actor,"JC084",2);
        let host=board(&mut g,"JC125",owner,2);
        let a=curse(&mut g,actor,&host); assert!(a.is_empty());
        assert!(g.board(&host).is_none()&&g.attachments.is_empty());
        assert_eq!(g.players[owner].graveyard.iter().filter(|c|c.definition=="JC125").count(),1);
        assert_eq!(g.players[actor].graveyard.iter().filter(|c|c.definition=="JC089").count(),1);
        for c in &g.players[actor].graveyard { assert_eq!(c.controller,c.owner); }
        assert!(g.pending.is_none()); fixture("zero-defense-death",&g);
    }}
}

#[test]
fn jc089_one_permanent_one_temporary_follows_host_team_and_ignores_curse_team_exhaustion() {
    let (mut g,host,a)=field(0);
    for first in 0..2 {
        g.first_team=first; assert_eq!(stats(&g,&host),(3,if first==0 {2}else{1}));
        assert_eq!(g.current_permanent_combat(g.board(&host).unwrap().1,2),1);
        g.attachments.iter_mut().find(|x|x.card.id==a).unwrap().card.exhausted=true;
        assert_eq!(stats(&g,&host),(3,if first==0 {2}else{1}));
        if first==1 {fixture("without-initiative",&g);}
        g.board_mut(&host).unwrap().exhausted=true;
        assert_eq!(g.icons(g.board(&host).unwrap().1,2),Icons::default());
        assert!(g.has_jc089_glory(g.board(&host).unwrap().1));
        if first==0 {fixture("exhausted-host",&g);}
        g.board_mut(&host).unwrap().exhausted=false;
    }
    g.add_control(&host,2,ControlLifetime::TurnEnd { turn:g.turn },SubtypeChange::None);
    g.settle_controls(); g.first_team=1;
    assert_eq!(stats(&g,&host),(3,2)); assert_eq!(g.board(&host).unwrap().1.owner,0);
    assert_eq!(g.attachments[0].card.controller,0); fixture("stolen-host",&g);
}

#[test]
fn jc089_multiple_instances_stack_attributes_but_grant_only_one_glory() {
    let (mut g,host,a)=field(0); let b=curse(&mut g,0,&host);
    assert_ne!(a,b); assert_eq!(g.jc089_host_curses(g.board(&host).unwrap().1),2);
    assert_eq!(stats(&g,&host),(2,4)); fixture("double-curse",&g);
    g.remove_dead(&a,RemovalCause::Destroy); g.settle_deaths();
    assert_eq!(stats(&g,&host),(3,2)); assert!(g.has_jc089_glory(g.board(&host).unwrap().1));
    g.remove_dead(&b,RemovalCause::Destroy); g.settle_deaths();
    assert_eq!(stats(&g,&host),(4,0)); assert!(!g.has_jc089_glory(g.board(&host).unwrap().1));
    fixture("curse-removed",&g);
    let enemy=board(&mut g,"LC01",2,2);
    let own_curse=curse(&mut g,0,&host);
    fund(&mut g,2,"JC084",2);g.begin_window(Window::Action(1));
    let enemy_curse=curse(&mut g,2,&enemy);
    assert_ne!(own_curse,enemy_curse);assert_ne!(host,enemy);
    assert_eq!(stats(&g,&host),(3,2));assert_eq!(stats(&g,&enemy),(3,1));
    g.remove_dead(&own_curse,RemovalCause::Destroy);g.settle_deaths();
    assert_eq!(stats(&g,&host),(4,0));assert_eq!(stats(&g,&enemy),(3,1));
    fixture("separate-team-instances",&g);
}

#[test]
fn jc089_curse_is_not_equipment_and_can_attach_to_cannot_be_equipped_character() {
    let mut g=game(0); fund(&mut g,0,"JC084",2);
    let bat=board(&mut g,"JC030",0,2); // Explicit protection support prevents zero defense.
    let guard=board(&mut g,"JC059",0,2); assert_ne!(bat,guard);
    assert!(definition("JC030").traits.cannot_be_equipped);
    assert!(!curse(&mut g,0,&bat).is_empty()); assert!(g.board(&bat).is_some());
}

#[test]
fn jc089_lowered_defense_checks_damage_wounds_and_positive_bonuses_in_one_sum() {
    for (damage,wounds,alive) in [(2,0,true),(3,0,false),(0,2,true),(0,3,false)] {
        let mut g=game(0); fund(&mut g,0,"JC084",2);
        let host=board(&mut g,"LC01",0,2);
        g.board_mut(&host).unwrap().damage=damage; g.board_mut(&host).unwrap().wounds=wounds;
        curse(&mut g,0,&host); assert_eq!(g.board(&host).is_some(),alive); checkpoint(&g);
    }
    let mut g=game(0); fund(&mut g,0,"JC084",2);
    let host=board(&mut g,"JC125",0,2); board(&mut g,"JC059",0,2);
    curse(&mut g,0,&host); assert_eq!(stats(&g,&host).0,1);
}

#[test]
fn jc089_jz48_condition_loss_causes_second_lethal_batch_and_independent_owners() {
    let mut g=game(0); fund(&mut g,0,"JC084",2);
    let a=board(&mut g,"JZ48",0,2); let b=board(&mut g,"JZ48",0,2);
    for id in [&a,&b] {g.board_mut(id).unwrap().damage=1;}
    assert_eq!(stats(&g,&a).0,2);
    curse(&mut g,0,&a); assert!(g.board(&a).is_none()&&g.board(&b).is_none());
    assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition=="JZ48").count(),2);
    assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition=="JC089").count(),1);
    fixture("jz48-cascade",&g);
}

#[test]
fn jc089_target_leaves_or_hides_during_paid_response_no_new_instance_attachment() {
    for hide in [false,true] {
        let mut g=game(0); fund(&mut g,0,"JC084",2); fund(&mut g,2,"JC091",3);
        let host=board(&mut g,"LC01",2,2); let source=held(&mut g,"JC089",0);
        apply(&mut g,0,Action{card_id:Some(source),target_id:Some(host.clone()),..Action::new("play")});
        passes(&mut g,2); // Both winning-team seats yield the existing response window.
        if hide {
            let spell=held(&mut g,"JC063",2); fund(&mut g,2,"JC056",2);
            apply(&mut g,2,Action{card_id:Some(spell),option:Some("hide".into()),target_id:Some(host.clone()),..Action::new("play")});
        } else {
            let spell=held(&mut g,"JC091",2);
            apply(&mut g,2,Action{card_id:Some(spell),target_id:Some(host.clone()),..Action::new("play")});
        }
        pass_top(&mut g); pass_top(&mut g);
        assert!(g.attachments.is_empty()); assert!(g.board(&host).is_none());
        assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition=="JC089").count(),1);
        fixture("invalidated-target",&g);
    }
}

#[test]
fn jc089_host_move_preserves_curse_and_hide_cleans_attachment_without_death() {
    let (mut g,host,a)=field(0);
    let c=g.remove_board(&host).unwrap().1;g.regions[1].cards.push(c); g.settle_deaths();
    assert_eq!(g.board(&host).unwrap().0,1); assert_eq!(stats(&g,&host).0,3);
    assert_eq!(g.attachment_region(&g.attachments[0]),Some(1));
    g.board_mut(&host).unwrap().face_down=true; g.settle_deaths();
    assert!(g.attachments.is_empty()); assert!(g.players[0].graveyard.iter().any(|c|c.definition=="JC089"));
    assert!(!g.players[0].graveyard.iter().any(|c|c.definition=="LC01"));
    assert!(g.board(&a).is_none()); fixture("hidden-host",&g);
}

#[test]
fn jc089_won_combat_optional_reward_belongs_to_host_controller_even_without_kills() {
    for actor in 0..4 {for decline in [false,true] {
        let (mut g,host,_)=field(actor); let enemy=(actor+2)%4;
        g.add_control(&host,enemy,ControlLifetime::TurnEnd { turn:g.turn },SubtypeChange::None);g.settle_controls();
        g.first_team=g.team(enemy); win(&mut g,2);
        assert_eq!(g.pending.as_ref().unwrap().seat,enemy);
        fixture("glory-declaration",&g);
        choose(&mut g,if decline {vec![]} else {vec!["accept".into()]});
        if !decline { fixture("glory-stack",&g); pass_top(&mut g); }
        assert_eq!(g.regions[2].influence[g.team(enemy)],u32::from(!decline));
        assert_eq!(g.regions[2].influence[g.team(actor)],0);
        assert!(g.attachments.iter().all(|a|a.card.controller==actor));checkpoint(&g);
    }}
}

#[test]
fn jc089_many_curses_and_two_teammate_hosts_merge_one_glory_not_one_per_source() {
    let (mut g,host,_)=field(0);curse(&mut g,0,&host);
    let second=board(&mut g,"LC01",1,2);curse(&mut g,0,&second);
    win(&mut g,2);choose(&mut g,vec!["accept".into()]);
    assert_eq!(g.stack.len(),1);pass_top(&mut g);
    assert_eq!(g.regions[2].influence[0],1);assert!(g.pending.is_none());
    assert!(g.board(&second).is_some());fixture("one-merged-glory",&g);
}

#[test]
fn jc089_glory_does_not_trigger_on_other_contests_loss_tie_or_no_participation() {
    for kind in ["investigation","influence","loss","tie","exhausted","hidden"] {
        let (mut g,host,_)=field(0);
        let contest=match kind {"investigation"=>0,"influence"=>2,_=>1};
        match kind {
            "loss"=>{board(&mut g,"JC030",2,2);board(&mut g,"JC030",2,2);},
            "tie"=>{board(&mut g,"JC030",2,2);},
            "exhausted"=>g.board_mut(&host).unwrap().exhausted=true,
            "hidden"=>{g.board_mut(&host).unwrap().face_down=true;g.settle_deaths();},
            _=>{},
        }
        g.begin_window(Window::Before(2,contest));passes(&mut g,4);
        assert!(!matches!(&g.pending,Some(p) if matches!(&p.resolution,ChoiceResolution::Declare{declaration,..} if declaration.ability.event==Some(Event::CombatWon))));
        assert!(!g.effects.iter().any(|e|matches!(e,Effect::Declare{declaration} if declaration.ability.event==Some(Event::CombatWon))));checkpoint(&g);
    }
}

#[test]
fn jc089_glory_snapshot_survives_host_and_curse_departure_and_controller_change() {
    for printed in [false,true] {
        let (mut g,host,a)=if printed {let mut g=game(0);let host=board(&mut g,"JC018",0,2);(g,host,String::new())} else {field(0)};
        win(&mut g,2);choose(&mut g,vec!["accept".into()]);
        if !a.is_empty(){g.remove_dead(&a,RemovalCause::Destroy);}
        g.add_control(&host,2,ControlLifetime::TurnEnd { turn:g.turn },SubtypeChange::None);g.settle_controls();
        g.remove_dead(&host,RemovalCause::Destroy);checkpoint(&g);
        pass_top(&mut g);assert_eq!(g.regions[2].influence,[1,0]);
        assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition==if printed {"JC018"}else{"LC01"}).count(),1);
        fixture("glory-source-departed",&g);
    }
}

#[test]
fn jc089_glory_exact_region_instance_rejects_replacement_before_or_after_acceptance() {
    for printed in [false,true] {for before in [false,true] {
        let (mut g,host,_)=if printed {let mut g=game(0);let host=board(&mut g,"JC018",0,2);(g,host,String::new())} else {field(0)};
        win(&mut g,2);
        if !before {choose(&mut g,vec!["accept".into()]);}
        let old=g.regions[2].card.id.clone();g.regions[2].card=g.make_card("DQJC115",0);
        assert_ne!(old,g.regions[2].card.id);g.regions[2].influence=[0,0];
        if before {choose(&mut g,vec!["accept".into()]);}
        pass_top(&mut g);assert_eq!(g.regions[2].influence,[0,0]);
        assert!(g.board(&host).is_some());fixture("replacement-region",&g);
    }
    }
}
#[test]
fn jc018_printed_glory_never_becomes_renown_when_the_last_curse_leaves() {
    let mut g=game(0);let host=board(&mut g,"JC018",0,2);
    assert!(!g.has_renown(g.board(&host).unwrap().1));
    assert!(g.has_combat_glory(g.board(&host).unwrap().1));
    assert!(!g.has_jc089_glory(g.board(&host).unwrap().1));
    win(&mut g,2);choose(&mut g,vec!["accept".into()]);pass_top(&mut g);
    assert_eq!(g.regions[2].influence,[1,0]);fixture("uncursed-printed-glory",&g);
    g.begin_window(Window::Before(2,2));passes(&mut g,4);
    assert!(g.pending.is_none());assert_eq!(g.regions[2].influence,[2,0]); // One ordinary influence, no false renown.
    g.begin_window(Window::Action(0));fund(&mut g,0,"JC084",2);let curse_id=curse(&mut g,0,&host);
    assert!(g.has_combat_glory(g.board(&host).unwrap().1));
    assert!(!g.has_renown(g.board(&host).unwrap().1));
    g.remove_dead(&curse_id,RemovalCause::Destroy);g.settle_deaths();
    assert!(!g.has_renown(g.board(&host).unwrap().1));
    assert!(g.has_combat_glory(g.board(&host).unwrap().1));checkpoint(&g);
}

#[test]
fn jc018_printed_glory_decline_and_next_round_win_use_normal_window_progression() {
    let mut g=game(0);let host=board(&mut g,"JC018",0,2);
    g.regions[2].card=g.make_card("DQJC107",0); // Threshold four keeps the host in this region.
    fund(&mut g,0,"JC084",2);fund(&mut g,0,"LC01",2);
    let removal=held(&mut g,"JC005",0);
    let turn=g.turn;
    // Only the starting battle is an explicit layout boundary. Every subsequent
    // phase, cleanup, initiative change and battle uses actual legal passes.
    win(&mut g,2);choose(&mut g,vec![]);
    assert_eq!(g.regions[2].influence,[0,0]);
    for _ in 0..200 {
        if g.turn==turn+1 && g.window==Some(Window::Action(0)) {break;}
        assert!(g.pending.is_none(),"unexpected choice while advancing the round");
        passes(&mut g,1);
    }
    assert_eq!(g.turn,turn+1);assert_eq!(g.first_team,1);
    assert_eq!(g.window,Some(Window::Action(0)));
    assert_eq!(g.regions[2].influence,[1,0]); // Ordinary first-round influence only.
    assert!(g.has_combat_glory(g.board(&host).unwrap().1));
    assert!(!g.has_renown(g.board(&host).unwrap().1));
    let curse_id=curse(&mut g,0,&host);
    assert!(g.pending.is_none());assert_eq!(g.regions[2].influence,[1,0]);
    apply(&mut g,0,Action{card_id:Some(removal),target_id:Some(curse_id),..Action::new("play")});
    pass_top(&mut g);
    assert!(g.attachments.is_empty());assert!(g.pending.is_none());
    assert_eq!(g.regions[2].influence,[1,0]); // Adding/removing the curse cannot replay the declined reward.
    for _ in 0..200 {
        if g.pending.as_ref().is_some_and(|p|matches!(&p.resolution,
            ChoiceResolution::Declare{declaration,..} if declaration.ability.event==Some(Event::CombatWon))) {break;}
        if g.pending.as_ref().is_some_and(|p|p.choice.kind=="recipient") {
            let id=g.pending.as_ref().unwrap().choice.options[0].id.clone();choose(&mut g,vec![id]);
        } else {assert!(g.pending.is_none());passes(&mut g,1);}
    }
    assert_eq!(g.turn,turn+1);assert_eq!(g.window,Some(Window::After(2,1)));
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,"trigger");
    choose(&mut g,vec!["accept".into()]);pass_top(&mut g);
    assert_eq!(g.regions[2].influence,[2,0]); // Exactly one reward for this new winning combat.
    for _ in 0..200 {
        if g.turn==turn+2 {break;}
        assert!(g.pending.is_none());passes(&mut g,1);
    }
    assert_eq!(g.turn,turn+2);assert_eq!(g.regions[2].influence,[2,0]);
    assert!(g.board(&host).is_some());fixture("printed-glory-normal-next-round",&g);
}

#[test]
fn jc089_cursed_printed_glory_is_one_reward_while_separately_paid_true_renown_still_works() {
    for independent in [false,true] {
        let mut g=game(0);fund(&mut g,0,"JC056",3);fund(&mut g,0,"JC084",2);
        let guard=held(&mut g,"JC059",0);
        apply(&mut g,0,Action{card_id:Some(guard),region:Some(2),..Action::new("deploy")});pass_top(&mut g);
        let host=board(&mut g,"JC018",0,2);g.board_mut(&host).unwrap().damage=1;
        let curse_id=curse(&mut g,0,&host);assert_eq!(stats(&g,&host).0,2);
        let enemy=board(&mut g,"LC01",2,2); // Survives ordinary combat damage.
        let hidden=board(&mut g,"JC125",2,2);g.board_mut(&hidden).unwrap().face_down=true;
        // The hidden character ties the public guardian's influence, with no combat icons.
        if independent {
            fund(&mut g,0,"JC075",2);let spell=held(&mut g,"JC074",0);
            apply(&mut g,0,Action{card_id:Some(spell),target_id:Some(host.clone()),..Action::new("play")});pass_top(&mut g);
        }
        assert_eq!(g.has_renown(g.board(&host).unwrap().1),independent);
        fixture(if independent {"cursed-printed-glory-and-true-renown"} else {"cursed-printed-glory-only"},&g);
        g.first_team=1;win(&mut g,2);choose(&mut g,vec!["accept".into()]);pass_top(&mut g);
        assert!(g.board(&enemy).is_some());
        assert_eq!(g.regions[2].influence,[1,0]);
        g.begin_window(Window::Before(2,2));passes(&mut g,4);
        if independent {
            let ChoiceResolution::Declare{declaration,..}=&g.pending.as_ref().unwrap().resolution else {panic!()};
            assert_eq!(declaration.ability.event,Some(Event::RegionConfrontationsEnded));
            choose(&mut g,vec!["accept".into()]);pass_top(&mut g);
        } else {assert!(g.pending.is_none());}
        assert_eq!(g.regions[2].influence,[if independent {2}else{1},0]);
        g.remove_dead(&curse_id,RemovalCause::Destroy);g.settle_deaths();
        assert_eq!(g.has_renown(g.board(&host).unwrap().1),independent);
        assert!(g.has_combat_glory(g.board(&host).unwrap().1));
        fixture("printed-glory-curse-removed",&g);
    }
}

#[test]
fn jc089_paid_jc005_last_curse_removal_before_or_after_glory_cannot_restore_false_renown() {
    for response in [true,false] {
        let mut g=game(0);fund(&mut g,0,"JC056",3);fund(&mut g,0,"JC084",2);
        let guard=held(&mut g,"JC059",0);
        apply(&mut g,0,Action{card_id:Some(guard),region:Some(2),..Action::new("deploy")});pass_top(&mut g);
        let host=board(&mut g,"JC018",0,2);g.board_mut(&host).unwrap().damage=1;
        let curse_id=curse(&mut g,0,&host);fund(&mut g,0,"JC003",2);let spell=held(&mut g,"JC005",0);
        board(&mut g,"LC01",2,2);let hidden=board(&mut g,"JC125",2,2);g.board_mut(&hidden).unwrap().face_down=true;
        g.first_team=1;win(&mut g,2);choose(&mut g,vec!["accept".into()]);
        if !response {pass_top(&mut g);assert_eq!(g.regions[2].influence,[1,0]);}
        passes(&mut g,2);
        apply(&mut g,0,Action{card_id:Some(spell),target_id:Some(curse_id),..Action::new("play")});
        assert_eq!(g.stack.len(),if response {2}else{1});pass_top(&mut g);
        if response {assert_eq!(g.regions[2].influence,[0,0]);pass_top(&mut g);}
        assert!(g.attachments.is_empty());assert!(g.board(&host).is_some());
        assert!(g.has_combat_glory(g.board(&host).unwrap().1));assert!(!g.has_renown(g.board(&host).unwrap().1));
        assert_eq!(g.regions[2].influence,[1,0]);
        g.begin_window(Window::Before(2,2));passes(&mut g,4);
        assert!(g.pending.is_none());assert_eq!(g.regions[2].influence,[1,0]);
        fixture(if response {"last-curse-removed-during-glory"} else {"last-curse-removed-after-glory"},&g);
    }
}

#[test]
fn jc018_printed_glory_does_not_trigger_without_winning_combat_participation() {
    for kind in ["investigation","influence","loss","tie","exhausted","hidden","elsewhere"] {
        let mut g=game(0);let host=board(&mut g,"JC018",0,2);
        let contest=match kind {"investigation"=>0,"influence"=>2,_=>1};
        match kind {
            "loss"=>{board(&mut g,"JC030",2,2);board(&mut g,"JC030",2,2);},
            "tie"=>{board(&mut g,"JC030",2,2);},
            "exhausted"=>g.board_mut(&host).unwrap().exhausted=true,
            "hidden"=>g.board_mut(&host).unwrap().face_down=true,
            "elsewhere"=>{let c=g.remove_board(&host).unwrap().1;g.regions[1].cards.push(c);},_=>{},
        }
        g.begin_window(Window::Before(2,contest));passes(&mut g,4);
        assert!(!matches!(&g.pending,Some(p) if matches!(&p.resolution,ChoiceResolution::Declare{declaration,..} if declaration.ability.event==Some(Event::CombatWon))));
        assert!(!g.effects.iter().any(|e|matches!(e,Effect::Declare{declaration} if declaration.ability.event==Some(Event::CombatWon))));checkpoint(&g);
    }
}

#[test]
fn jc089_complete_definition_rejects_all_variants_and_combat_event_transplants() {
    for variant in 0..12 {
        let mut r=definitions().clone();let d=r.get_mut("JC089").unwrap();
        match variant {
            0=>d.modifiers.clear(),1=>d.modifiers.push(d.modifiers[0].clone()),
            2=>d.attachment.as_mut().unwrap().host.relation=Relation::ControlledByActor,
            3=>d.attachment.as_mut().unwrap().host.equipment_host=true,
            4=>d.attachment.as_mut().unwrap().host_icons.combat=2,
            5=>d.attachment.as_mut().unwrap().host_temporary_icons=None,
            6=>d.attachment.as_mut().unwrap().host_defense_bonus=1,
            7=>d.attachment.as_mut().unwrap().controls_host=true,
            8=>d.traits.renown=true,9=>d.abilities[0].event=Some(Event::CombatWon),
            10=>d.abilities[0].timing=Timing::Fast,_=>d.attachment=None,
        } assert!(validate_definitions(&r).is_err(),"variant {variant}");
    }
    for id in ["JC125","XQ47","JC018"] {
        let mut r=definitions().clone();r.insert(id.into(),definition("JC089").clone());assert!(validate_definitions(&r).is_err());
        let mut r=definitions().clone();r.get_mut(id).unwrap().modifiers.push(StaticModifier::JC089HostDefenseMinusOneAndGlory);assert!(validate_definitions(&r).is_err());
        let mut r=definitions().clone();let mut a=definition("JC003").abilities[0].clone();a.event=Some(Event::CombatWon);r.get_mut(id).unwrap().abilities.push(a);assert!(validate_definitions(&r).is_err());
    }
    let (mut g,host,_)=field(0);
    let runtime=g.jc089_combat_glory(0,2).unwrap().ability;
    g.remove_dead(&host,RemovalCause::Destroy); // Validation uses the frozen shape, not a surviving grant source.
    validate_ability("LC01",&runtime).unwrap();
    for id in ["JC125","JZ48","LC01"] {
        let mut r=definitions().clone();r.get_mut(id).unwrap().abilities.push(runtime.clone());
        assert!(validate_definitions(&r).is_err(),"exact runtime transplant {id}");
    }
    for variant in 0..4 {
        let mut a=runtime.clone();match variant {
            0=>a.ops=vec![Op::PlaceInfluence{region_instance:"fake".into(),amount:2}],
            1=>a.requires_ready_source=true,2=>a.event=None,_=>a.targets=definition("JC089").abilities[0].targets.clone(),
        }
        assert!(validate_ability("LC01",&a).is_err(),"runtime mutation {variant}");
    }
    validate_definitions(definitions()).unwrap();
}

#[test]
fn jc089_glory_paid_sacrifice_response_keeps_frozen_reward_after_host_graveyard() {
    let (mut g,host,_)=field(0);fund(&mut g,2,"JC084",2);
    let murder=held(&mut g,"JZ54",2);
    win(&mut g,2);choose(&mut g,vec!["accept".into()]);passes(&mut g,2);
    apply(&mut g,2,Action{card_id:Some(murder),target_id:Some("p0".into()),..Action::new("play")});
    assert_eq!(g.stack.len(),2);fixture("glory-sacrifice-response",&g);
    pass_top(&mut g);choose(&mut g,vec![host.clone()]);
    assert!(g.board(&host).is_none()&&g.attachments.is_empty());
    assert_eq!(g.regions[2].influence,[0,0]);pass_top(&mut g);
    assert_eq!(g.regions[2].influence,[1,0]);fixture("glory-response-final",&g);
}

#[test]
fn jc089_one_glory_wins_exact_region_threshold_and_team_score_ten() {
    let (mut g,_,_)=field(0);g.regions[2].card=g.make_card("DQJC109",0);
    g.regions[2].influence=[2,0];
    for id in ["DQJC107","DQJC109"] {let c=g.make_card(id,0);g.players[0].score_cards.push(c);}
    win(&mut g,2);choose(&mut g,vec!["accept".into()]);pass_top(&mut g);
    assert_eq!(g.regions[2].influence,[3,0]);assert!(matches!(g.window,Some(Window::Win(2,0))));
    g.close_window().unwrap();g.drive().unwrap();
    while let Some(p)=g.pending.clone() {let ids=p.choice.options.iter().map(|o|o.id.clone()).collect();choose(&mut g,ids);}
    assert_eq!(g.players[0].score_cards.iter().map(|c|catalog::card(&c.definition).points.unwrap_or(0)).sum::<u32>(),10);
    assert_eq!(g.status,"finished");assert_eq!(g.winner_team,Some(0));fixture("glory-score-ten",&g);
}

fn command(room:&mut RoomEnvelope,steps:&mut Vec<serde_json::Value>,seat:usize,action:SessionAction) {
    let state=serde_json::to_string(room).unwrap();
    let cmd=RoomCommand{command_id:format!("jc089-chain-{}",steps.len()),expected_version:room.revision,action};
    let expected=room.transition(seat,Some(cmd.clone()),0).unwrap();
    assert!(expected.error_code.is_none(),"{:?}",expected.error_message);
    assert_eq!(serde_json::to_string(&room.replay_events(&expected.journal).unwrap()).unwrap(),expected.state);
    *room=RoomEnvelope::from_persisted(&expected.state).unwrap();
    steps.push(serde_json::json!({"state":state,"seat":seat,"command":cmd,"expected":expected,
        "views":(0..4).map(|s|room.view(s,0)).collect::<Vec<_>>() }));
}

#[test]
fn jc089_continuous_paid_attachment_damage_response_chain_saves_replays_and_retries_once() {
    let mut g=game(0);fund(&mut g,0,"JC084",2);fund(&mut g,2,"JC102",2);
    let host=board(&mut g,"LC01",0,2);let source=held(&mut g,"JC089",0);let reply=held(&mut g,"JC102",2);
    let mut room=envelope(&g);let initial=serde_json::to_string(&room).unwrap();let mut steps=vec![];
    command(&mut room,&mut steps,0,SessionAction::Game{action:Action{
        card_id:Some(source),target_id:Some(host.clone()),..Action::new("play")}});
    while room.pacing.window.as_ref().unwrap().holder_team==0 {
        let w=room.pacing.window.clone().unwrap();let s=*w.members.iter().find(|(_,d)|matches!(d,Decision::Undecided{..})).unwrap().0;
        command(&mut room,&mut steps,s,SessionAction::PassResponse{window_id:w.id});
    }
    let w=room.pacing.window.clone().unwrap();
    command(&mut room,&mut steps,2,SessionAction::BeginResponse{window_id:w.id.clone(),intent_id:"jc089-damage-response".into()});
    command(&mut room,&mut steps,2,SessionAction::SubmitResponse{window_id:w.id,intent_id:"jc089-damage-response".into(),action:Action{
        card_id:Some(reply),target_id:Some(host.clone()),..Action::new("play")}});
    let accepted=steps.last().unwrap().clone();let saved=serde_json::to_string(&room).unwrap();
    let retry=room.transition(2,Some(serde_json::from_value(accepted["command"].clone()).unwrap()),0).unwrap();
    assert_eq!(retry.state,saved);assert!(retry.journal.is_empty());
    for _ in 0..30 {
        if room.stack.is_empty(){break;}let w=room.pacing.window.clone().unwrap();let s=*w.members.iter().find(|(_,d)|matches!(d,Decision::Undecided{..})).unwrap().0;
        command(&mut room,&mut steps,s,SessionAction::PassResponse{window_id:w.id});
    }
    assert!(room.pending.is_none()&&room.stack.is_empty());assert_eq!(stats(&room.game,&host),(3,2));
    assert_eq!(room.board(&host).unwrap().1.damage,1);assert_eq!(room.resources(0),0);assert_eq!(room.resources(2),0);
    fixture("paid-response-final",&room.game);
    if let Ok(dir)=std::env::var("JC089_CHAIN_DIR") {std::fs::create_dir_all(&dir).unwrap();std::fs::write(format!("{dir}/paid-response-chain.json"),serde_json::to_vec(&serde_json::json!({"initialState":initial,"steps":steps,"finalState":serde_json::to_string(&room).unwrap()})).unwrap()).unwrap();}
}

#[cfg(feature="native")]
#[tokio::test]
async fn jc089_sqlite_reopen_paid_zero_defense_duplicate_and_conflict_receipts() {
    use crate::service::{CreateRoom,JoinRoom,Store};
    let dir=tempfile::tempdir().unwrap();let path=dir.path().join("jc089-disclosed-test.sqlite");let store=Store::open(&path).unwrap();
    let host=store.create(CreateRoom{name:"P0".into(),mode:"teams".into(),deck_id:"watchers".into(),deck_draft:None}).await.unwrap();let mut sessions=vec![host];
    for seat in 1..4 {sessions.push(store.join(JoinRoom{invite_code:sessions[0].invite_code.clone(),name:format!("P{seat}"),deck_id:"watchers".into(),deck_draft:None}).await.unwrap());}drop(store);
    let mut g=game(0);fund(&mut g,0,"JC084",2);let target=board(&mut g,"JC125",2,2);let curse=held(&mut g,"JC089",0);
    g.room_id=sessions[0].room_id.clone();g.invite_code=sessions[0].invite_code.clone();let mut room=envelope(&g);
    // Only this temporary synthetic private Store fixture is initialized here.
    let db=rusqlite::Connection::open(&path).unwrap();db.execute("UPDATE rooms SET state=?1,revision=?2 WHERE id=?3",rusqlite::params![serde_json::to_string(&room).unwrap(),room.revision,g.room_id]).unwrap();drop(db);
    let mut cmds=vec![(0,RoomCommand{command_id:"jc089-store-paid".into(),expected_version:room.revision,action:SessionAction::Game{action:Action{card_id:Some(curse),target_id:Some(target.clone()),..Action::new("play")}}})];let mut accepted=vec![];
    for _ in 0..20 {
        let (seat,cmd)=cmds.last().unwrap().clone();let expected=room.transition(seat,Some(cmd.clone()),0).unwrap();assert!(expected.error_code.is_none());
        let store=Store::open(&path).unwrap();let v=store.command_at_now(&g.room_id,&sessions[seat].token,cmd.clone(),0).await.unwrap();drop(store);
        room=RoomEnvelope::from_persisted(&expected.state).unwrap();assert_eq!(serde_json::to_value(&v).unwrap(),serde_json::to_value(room.view(seat,0)).unwrap());accepted.push(serde_json::to_value(v).unwrap());
        let store=Store::open(&path).unwrap();let duplicate=store.command_at_now(&g.room_id,&sessions[seat].token,cmd.clone(),0).await.unwrap();assert_eq!(accepted.last().unwrap(),&serde_json::to_value(duplicate).unwrap());
        let mut conflict=cmd;conflict.expected_version+=1;assert_eq!(store.command_at_now(&g.room_id,&sessions[seat].token,conflict,0).await.unwrap_err().error,"command_id_conflict");drop(store);
        if room.stack.is_empty(){break;}let w=room.pacing.window.clone().unwrap();let s=*w.members.iter().find(|(_,d)|matches!(d,Decision::Undecided{..})).unwrap().0;
        cmds.push((s,RoomCommand{command_id:format!("jc089-store-pass-{}",cmds.len()),expected_version:room.revision,action:SessionAction::PassResponse{window_id:w.id}}));
    }
    assert!(room.stack.is_empty()&&room.board(&target).is_none());assert!(room.attachments.is_empty());
    assert_eq!(room.players[0].graveyard.iter().filter(|c|c.definition=="JC089").count(),1);assert_eq!(room.players[2].graveyard.iter().filter(|c|c.definition=="JC125").count(),1);
    let store=Store::open(&path).unwrap();assert_eq!(serde_json::to_value(store.command_at_now(&g.room_id,&sessions[0].token,cmds[0].1.clone(),90_000).await.unwrap()).unwrap(),accepted[0]);drop(store);
    let db=rusqlite::Connection::open(&path).unwrap();let saved:String=db.query_row("SELECT state FROM rooms WHERE id=?1",[&g.room_id],|r|r.get(0)).unwrap();assert_eq!(saved,serde_json::to_string(&room).unwrap());
    let n:u64=db.query_row("SELECT COUNT(*) FROM commands WHERE room_id=?1",[&g.room_id],|r|r.get(0)).unwrap();assert_eq!(n,cmds.len() as u64);
}
