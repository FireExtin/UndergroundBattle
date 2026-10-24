//! Author native tests: real paid commands on explicit legal layouts. Normal
//! continuations roundtrip through Game; no browser or independent claims.
use crate::{catalog, import20_broker as b, model::*, rules::*, unit_support::*};
use crate::jc029_tests::pass_top;
fn corpse(g:&mut Game,id:&str,seat:usize)->String{let c=g.make_card(id,seat);let uid=c.id.clone();g.players[seat].graveyard.push(c);uid}
fn attachment(g:&mut Game)->String{
    fund(g,2,"JC029",3);let hand=held(g,"JC033",2);g.begin_window(Window::Action(1));
    act(g,2,Action{card_id:Some(hand),target_id:Some("society:p2".into()),..Action::new("play")});pass_top(g);
    g.begin_window(Window::Action(0));g.attachments.iter().find(|a|a.card.definition=="JC033").unwrap().card.id.clone()
}
fn to_declaration(g:&mut Game,key:&str){
    for _ in 0..100{
        if g.pending.as_ref().is_some_and(|p|matches!(&p.resolution,ChoiceResolution::Declare{declaration:d,..}if d.ability.key==key)){return;}
        if g.pending.is_some(){resolve_choice(g);}else{pass(g);}
    }panic!("missing {key}");
}
fn enter(g:&mut Game)->String{
    fund(g,0,"JC125",3);let hand=held(g,"BQ081",0);
    act(g,0,Action{card_id:Some(hand),region:Some(0),..Action::new("deploy")});to_declaration(g,b::ENTRY);
    g.regions[0].cards.iter().find(|c|c.definition=="BQ081").unwrap().id.clone()
}
fn bound_entry(g:&mut Game,target:&str){choose(g,vec![target.into()]);pass_top(g);}
fn seed_shown(g:&mut Game,bq_seat:Option<usize>)->Vec<String>{
    (0..4).map(|s|held(g,if bq_seat==Some(s){"BQ081"}else if s<2{"WM025"}else{"JC125"},s)).collect()
}
fn hand_contest(g:&mut Game){fund(g,0,"JC125",2);let source=held(g,"BQ090",0);
    act(g,0,Action{card_id:Some(source),ability_id:Some(crate::import20_contest::HAND.into()),..Action::new("activate")});pass_top(g);}
fn commits_to_rewards(g:&mut Game,shown:&[String]){
    for _ in 0..100{if let Some(p)=g.pending.clone(){
        if p.choice.kind==crate::mystic_contest::REWARD_KIND{return;}
        if p.choice.kind==crate::mystic_contest::COMMIT_KIND{choose(g,vec![shown[p.seat].clone()]);}else{resolve_choice(g);}
    }else if !g.stack.is_empty(){pass(g);}else{break;}}panic!("missing actual contest rewards");
}
fn finish(g:&mut Game){for _ in 0..150{if let Some(p)=g.pending.clone(){
    if p.choice.kind==crate::mystic_contest::REWARD_KIND{choose(g,vec![]);}else{resolve_choice(g);}
}else if !g.stack.is_empty(){pass(g);}else if !g.effects.is_empty(){g.drive().unwrap();restore(g);}else{return;}}panic!("bounded normal continuation");}
fn broker_win(g:&mut Game){let shown=seed_shown(g,None);hand_contest(g);commits_to_rewards(g,&shown);to_declaration(g,b::WIN);}
fn repress_to_peek(g:&mut Game){choose(g,vec!["p2".into()]);pass_top(g);let p=g.pending.as_ref().unwrap();
    assert_eq!(p.seat,2);assert!(matches!(p.resolution,ChoiceResolution::Frame{choice:FrameChoice::RepressOne{..},..}));
    choose(g,vec!["region:0".into()]);}

#[test]
fn import20_black_original_prints_single_latest_kernel_and_closed_public_gate(){
    assert_eq!(catalog::ENGINE_VERSION,"rust-v0.2.103-import20-broker");for id in b::CARDS{assert!(!catalog::card(id).supported);assert!(catalog::runtime_implemented(catalog::card(id)));}
    let c=catalog::card("BQ081");assert_eq!((c.cost,c.defense,c.color.as_str(),c.magic.as_str()),(3,Some(1),"黑","死亡"));
    assert_eq!(c.loyalty,["黑色"]);assert_eq!(c.subtypes,["不死生物","食尸鬼","植物"]);assert_eq!(c.permanent_icons,Icons{combat:1,influence:1,..Icons::default()});assert_eq!(c.temporary_icons.combat,1);
    assert!(c.text.contains("牌库顶的一张牌"));assert!(!c.text.contains("三张"));
    let c=catalog::card("WM057");assert_eq!((c.cost,c.defense,c.color.as_str()),(2,Some(1),"黑"));assert_eq!(c.loyalty,["黑色"]);assert_eq!(c.subtypes,["人类","罪犯","工程师"]);assert_eq!(c.temporary_icons.investigation,1);
    validate_definitions(definitions()).unwrap();
}

#[test]
fn import20_bq081_real_paid_entry_moves_own_character_to_bottom_then_destroys_bound_attachment(){
    let mut g=game();let target=attachment(&mut g);let character=corpse(&mut g,"JC125",0);let spell=corpse(&mut g,"BQ090",0);let enemy=corpse(&mut g,"JC070",2);
    enter(&mut g);assert_eq!(g.resources(0),0);bound_entry(&mut g,&target);
    let p=g.pending.clone().unwrap();assert!(matches!(p.resolution,ChoiceResolution::Frame{choice:FrameChoice::BQ081GraveCharacter,..}));assert_eq!(p.choice.options.len(),1);
    for wrong in [spell,enemy]{reject(&mut g,0,Action{choice_id:Some(p.choice.id.clone()),selected:Some(vec![wrong]),..Action::new("choose")});}
    choose(&mut g,vec![character.clone()]);assert!(!g.players[0].graveyard.iter().any(|c|c.id==character));assert_eq!(g.players[0].deck.last().unwrap().definition,"JC125");
    assert_ne!(g.players[0].deck.last().unwrap().id,character);assert!(!g.attachments.iter().any(|a|a.card.id==target));assert!(g.players[2].graveyard.iter().any(|c|c.definition=="JC033"));restore(&mut g);
}

#[test]
fn import20_bq081_empty_own_character_grave_does_not_destroy_or_take_enemy_cards(){
    let mut g=game();let target=attachment(&mut g);corpse(&mut g,"BQ090",0);corpse(&mut g,"JC070",2);enter(&mut g);bound_entry(&mut g,&target);
    assert!(g.pending.is_none());assert!(g.attachments.iter().any(|a|a.card.id==target));assert_eq!(g.players[2].graveyard.len(),1);restore(&mut g);
}

#[test]
fn import20_bq081_entry_can_be_declined_without_moving_a_grave_card(){
    let mut g=game();let target=attachment(&mut g);let c=corpse(&mut g,"JC125",0);enter(&mut g);choose(&mut g,vec![]);
    assert!(g.players[0].graveyard.iter().any(|x|x.id==c));assert!(g.attachments.iter().any(|a|a.card.id==target));assert!(g.stack.is_empty());restore(&mut g);
}

#[test]
fn import20_bq081_departed_target_cancels_before_grave_move_but_source_departure_does_not(){
    for source_leaves in [false,true]{let mut g=game();let target=attachment(&mut g);let c=corpse(&mut g,"JC125",0);let source=enter(&mut g);
        choose(&mut g,vec![target.clone()]);g.remove_dead(if source_leaves{&source}else{&target},RemovalCause::Destroy);restore(&mut g);pass_top(&mut g);
        if source_leaves{choose(&mut g,vec![c.clone()]);assert!(g.players[0].deck.last().is_some_and(|x|x.definition=="JC125"));assert!(!g.attachments.iter().any(|a|a.card.id==target));}
        else{assert!(g.pending.is_none());assert!(g.players[0].graveyard.iter().any(|x|x.id==c));}restore(&mut g);
    }
}

#[test]
fn import20_bq081_enemy_shield_stops_whole_effect_before_grave_character_moves(){
    let mut g=game();let target=attachment(&mut g);let c=corpse(&mut g,"JC125",0);enter(&mut g);choose(&mut g,vec![target.clone()]);
    g.attachments.iter_mut().find(|a|a.card.id==target).unwrap().card.shield=1;restore(&mut g);pass_top(&mut g);
    assert!(g.pending.is_none());assert!(g.players[0].graveyard.iter().any(|x|x.id==c));assert_eq!(g.attachments.iter().find(|a|a.card.id==target).unwrap().card.shield,0);restore(&mut g);
}

#[test]
fn import20_bq081_shown_bonus_one_and_exactly_one_personal_post_mill_even_when_losing(){
    for seat in [0,2]{let mut g=game();let shown=seed_shown(&mut g,Some(seat));hand_contest(&mut g);commits_to_rewards(&mut g,&shown);
        let o=g.mystic_history.last().unwrap();let s=o.shown.iter().find(|s|s.seat==seat).unwrap();assert_eq!(s.value,4);
        let before=g.players[seat].graveyard.len();let n=g.players[seat].deck.len();let original=g.players[seat].deck[0].clone();finish(&mut g);
        assert_eq!(g.players[seat].graveyard.len(),before+1);assert_eq!(g.players[seat].deck.len(),n-1);
        assert!(g.players[seat].graveyard.iter().any(|c|c.definition==original.definition&&c.id!=original.id));restore(&mut g);
        let stable=g.players[seat].graveyard.len();g.drive().unwrap();assert_eq!(g.players[seat].graveyard.len(),stable);
    }
}

#[test]
fn import20_bq081_mill_last_card_is_not_draw_and_does_not_eliminate(){
    let mut g=game();g.players[0].deck.clear();let shown=seed_shown(&mut g,Some(0));hand_contest(&mut g);commits_to_rewards(&mut g,&shown);assert_eq!(g.players[0].deck.len(),1);
    finish(&mut g);assert!(g.players[0].deck.is_empty());assert!(!g.players[0].eliminated);assert_eq!(g.status,"playing");restore(&mut g);
}

#[test]
fn import20_bq081_post_mill_rumor_sealing_pauses_and_keeps_the_same_contest_once(){
    let mut g=game();let host=field(&mut g,"JC056",0);g.players[0].deck.clear();for _ in 0..5{let c=g.make_card("WM042",0);g.players[0].deck.push(c);}
    let shown=seed_shown(&mut g,Some(0));hand_contest(&mut g);commits_to_rewards(&mut g,&shown);
    assert_eq!(g.players[0].deck[0].definition,"WM042","fixed initial seed places a real rumor atop the shuffled deck");
    for _ in 0..50{if g.pending.as_ref().is_some_and(|p|matches!(p.resolution,ChoiceResolution::Rumor{..})){break;}
        if g.pending.is_some(){resolve_choice(&mut g);}else{pass(&mut g);}}
    let p=g.pending.as_ref().unwrap();assert!(matches!(p.resolution,ChoiceResolution::Rumor{..}));let contest=g.mystic_contest.as_ref().unwrap().contest_id.clone();restore(&mut g);
    choose(&mut g,vec![host.clone()]);finish(&mut g);assert_eq!(g.sealed_cards.iter().filter(|s|s.host_id==host&&s.card.definition=="WM042").count(),1);
    assert_eq!(g.mystic_history.iter().filter(|o|o.contest_id==contest).count(),1);assert!(g.mystic_contest.is_none());restore(&mut g);
}

#[test]
fn import20_wm057_owner_activation_pays_two_taps_original_and_runs_shared_contest(){
    let mut g=game();let source=field(&mut g,"WM057",0);let shown=seed_shown(&mut g,None);fund(&mut g,0,"JC125",2);
    assert!(g.legal_actions(0).iter().any(|a|a.action.kind=="activate"&&a.action.card_id.as_deref()==Some(&source)));
    act(&mut g,0,Action{card_id:Some(source.clone()),ability_id:Some(b::ACT.into()),..Action::new("activate")});assert!(g.board(&source).unwrap().1.exhausted);assert_eq!(g.resources(0),0);pass_top(&mut g);
    commits_to_rewards(&mut g,&shown);assert_eq!(g.mystic_history.last().unwrap().ability_key,b::ACT);finish(&mut g);restore(&mut g);
}

#[test]
fn import20_wm057_owner_activation_rejects_bad_payload_payment_manual_win_and_exhaustion(){
    let mut g=game();let source=field(&mut g,"WM057",0);fund(&mut g,0,"JC125",1);let a=Action{card_id:Some(source.clone()),..Action::new("activate")};reject(&mut g,0,a.clone());fund(&mut g,0,"JC125",1);
    let mut bad=a.clone();bad.target_id=Some("p2".into());reject(&mut g,0,bad);
    let mut bad=a.clone();bad.ability_id=Some(b::WIN.into());reject(&mut g,0,bad);
    g.board_mut(&source).unwrap().exhausted=true;reject(&mut g,0,a);restore(&mut g);
}

#[test]
fn import20_wm057_source_departure_after_owner_payment_does_not_refund_or_cancel_contest(){
    let mut g=game();let source=field(&mut g,"WM057",0);let shown=seed_shown(&mut g,None);fund(&mut g,0,"JC125",2);
    act(&mut g,0,Action{card_id:Some(source.clone()),..Action::new("activate")});g.remove_dead(&source,RemovalCause::Destroy);restore(&mut g);pass_top(&mut g);
    commits_to_rewards(&mut g,&shown);finish(&mut g);assert_eq!(g.resources(0),0);assert_eq!(g.mystic_history.len(),1);assert!(g.board(&source).is_none());restore(&mut g);
}

#[test]
fn import20_wm057_actual_win_represses_then_optionally_inspects_without_leaking_four_views(){
    let mut g=game();field(&mut g,"WM057",0);let hidden=field(&mut g,"JC056",2);g.board_mut(&hidden).unwrap().face_down=true;g.regions[0].influence[1]=2;
    broker_win(&mut g);repress_to_peek(&mut g);assert_eq!(g.regions[0].influence[1],1);choose(&mut g,vec![hidden.clone()]);
    let p=g.pending.as_ref().unwrap();assert!(matches!(p.resolution,ChoiceResolution::Frame{choice:FrameChoice::WM057Inspection{..},..}));
    let own=serde_json::to_value(g.view(0)).unwrap();assert!(own.to_string().contains("JC056"));
    for seat in [1,3]{assert!(!serde_json::to_value(g.view(seat)).unwrap().to_string().contains("JC056"));}
    assert!(g.board(&hidden).unwrap().1.face_down);choose(&mut g,vec![]);assert_eq!(g.regions[0].influence[1],1);restore(&mut g);
}

#[test]
fn import20_wm057_declining_peek_keeps_repression_and_no_hidden_target_still_represses(){
    for has_hidden in [false,true]{let mut g=game();field(&mut g,"WM057",0);if has_hidden{let c=field(&mut g,"JC056",2);g.board_mut(&c).unwrap().face_down=true;}
        g.regions[0].influence[1]=1;broker_win(&mut g);choose(&mut g,vec!["p2".into()]);pass_top(&mut g);choose(&mut g,vec!["region:0".into()]);
        if has_hidden{choose(&mut g,vec![]);}assert_eq!(g.regions[0].influence[1],0);assert!(g.pending.is_none());restore(&mut g);
    }
}

#[test]
fn import20_wm057_zero_enemy_influence_does_not_block_optional_inspection(){
    let mut g=game();field(&mut g,"WM057",0);let hidden=field(&mut g,"JC056",2);g.board_mut(&hidden).unwrap().face_down=true;
    broker_win(&mut g);choose(&mut g,vec!["p2".into()]);pass_top(&mut g);
    assert!(g.pending.as_ref().is_some_and(|p|matches!(p.resolution,ChoiceResolution::Frame{choice:FrameChoice::WM057InspectTarget,..})));
    choose(&mut g,vec![hidden]);choose(&mut g,vec![]);restore(&mut g);
}

#[test]
fn import20_wm057_secret_plan_inspection_uses_real_hidden_society_attachment(){
    let mut g=game();field(&mut g,"WM057",0);fund(&mut g,2,"JC063",1);g.add_society_markers(2,0,1);let hand=held(&mut g,"WM046",2);g.begin_window(Window::Action(1));
    act(&mut g,2,Action{card_id:Some(hand),target_id:Some("society:p2".into()),option:Some("secret-plan".into()),..Action::new("play")});pass_top(&mut g);g.begin_window(Window::Action(0));
    let plan=g.attachments.iter().find(|a|a.card.definition=="WM046").unwrap().card.id.clone();broker_win(&mut g);choose(&mut g,vec!["p2".into()]);pass_top(&mut g);choose(&mut g,vec![plan.clone()]);
    assert!(serde_json::to_string(&g.view(0)).unwrap().contains("WM046"));assert!(!serde_json::to_string(&g.view(1)).unwrap().contains("WM046"));
    assert!(g.attachments.iter().find(|a|a.card.id==plan).unwrap().card.face_down);choose(&mut g,vec![]);restore(&mut g);
}

#[test]
fn import20_black_exact_registry_rejects_foreign_source_extra_steps_and_free_costs(){
    let a=definition("BQ081").abilities[0].clone();assert!(validate_ability("JC125",&a).is_err());
    let mut a=definition("WM057").abilities[0].clone();a.costs.clear();assert!(validate_ability("WM057",&a).is_err());
    let mut a=definition("WM057").abilities[1].clone();a.ops.push(Op::Draw{player:PlayerRef::Actor,count:1,end:DeckEnd::Top});assert!(validate_ability("WM057",&a).is_err());
}
