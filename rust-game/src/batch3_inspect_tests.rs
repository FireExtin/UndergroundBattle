use crate::{model::*, rules, unit_support::*};
fn hidden(g:&mut Game,print:&str,seat:usize)->String { let id=field(g,print,seat);g.board_mut(&id).unwrap().face_down=true;id }
fn file(g:&mut Game)->String { fund(g,0,"JC057",1);held(g,"JC062",0) }
fn play(id:&str,targets:Vec<String>)->Action { Action{card_id:Some(id.into()),selected:Some(targets),..Action::new("play")} }
fn pending(g:&mut Game) { for _ in 0..32 { if g.pending.is_some(){restore(g);return;} pass(g); } panic!("inspection did not pause"); }

#[test]
fn batch3_jc062_faq_zero_target_draw_works_with_or_without_hidden_cards_and_at_fast_timing() {
    for have_hidden in [false,true] {
        let mut g=game();let id=file(&mut g);if have_hidden {hidden(&mut g,"JC029",2);}
        let hand=g.players[0].hand.len();let deck=g.players[0].deck.len();
        g.begin_window(Window::Before(0,1));
        act(&mut g,0,play(&id,vec![]));top(&mut g);
        assert_eq!(g.players[0].hand.len(),hand);assert_eq!(g.players[0].deck.len(),deck-1);
        assert!(g.players[0].assets[0].exhausted && g.pending.is_none());
        assert!(g.players[0].graveyard.iter().any(|c|c.definition=="JC062"));restore(&mut g);
    }
}

#[test]
fn batch3_jc062_one_and_two_targets_are_private_until_confirmation_then_draw_exactly_one() {
    for count in [1,2] {
        let mut g=game();let id=file(&mut g);let targets=vec![hidden(&mut g,"JC125",2),hidden(&mut g,"JC029",3)];
        let selected=targets[..count].to_vec();let hand=g.players[0].hand.len();let deck=g.players[0].deck.len();
        act(&mut g,0,play(&id,selected.clone()));pending(&mut g);
        assert_eq!(g.players[0].hand.len(),hand-1);assert_eq!(g.players[0].deck.len(),deck);
        let p=g.pending.clone().unwrap();let own=g.view(0).pending_choice.unwrap();assert_eq!(own.preview_cards.len(),count);
        for (i,preview) in own.preview_cards.iter().enumerate(){assert_eq!(preview.card_id.as_deref(),Some(if i==0{"JC125"}else{"JC029"}));}
        for seat in 1..4 {let view=g.view(seat);assert!(view.pending_choice.is_none());
            for target in &selected {let c=view.regions[0].characters.iter().find(|c|c.instance_id==*target).unwrap();
                if c.controller!=player_id(seat) {assert!(c.card_id.is_none());} assert!(c.face_down);}
        }
        let mut bad=g.clone();if let ChoiceResolution::Frame{choice:FrameChoice::JC062Inspection{inspected},..}=&mut bad.pending.as_mut().unwrap().resolution{inspected[0].definition="JC029".into();}
        assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
        reject(&mut g,1,Action{choice_id:Some(p.choice.id.clone()),selected:Some(vec![]),..Action::new("choose")});
        reject(&mut g,0,Action{choice_id:Some(p.choice.id.clone()),selected:Some(vec![]),target_id:Some(selected[0].clone()),..Action::new("choose")});
        choose(&mut g,vec![]);assert_eq!(g.players[0].hand.len(),hand);assert_eq!(g.players[0].deck.len(),deck-1);
        assert!(selected.iter().all(|id|g.board(id).unwrap().1.face_down));
        reject(&mut g,0,Action{choice_id:Some(p.choice.id),selected:Some(vec![]),..Action::new("choose")});restore(&mut g);
    }
}

#[test]
fn batch3_jc062_rejects_duplicate_excess_and_nonhidden_targets_before_payment() {
    let mut g=game();let id=file(&mut g);let hidden=hidden(&mut g,"JC125",2);let public=field(&mut g,"JC125",2);
    for targets in [vec![hidden.clone(),hidden.clone()],vec![hidden.clone(),hidden.clone(),public.clone()],vec![public],vec!["unknown".into()]] {
        reject(&mut g,0,play(&id,targets));assert!(!g.players[0].assets[0].exhausted);
    }
    let mut polluted=play(&id,vec![]);polluted.target_id=Some(hidden);reject(&mut g,0,polluted);
}

#[test]
fn batch3_jc062_real_response_reveal_invalidates_whole_two_target_effect_without_refund_or_draw() {
    let mut g=game();let id=file(&mut g);let a=hidden(&mut g,"JC125",2);let b=hidden(&mut g,"JC029",3);
    let deck=g.players[0].deck.len();act(&mut g,0,play(&id,vec![a.clone(),b.clone()]));
    for _ in 0..4 {if g.can_fast(2){break;} pass(&mut g);}
    act(&mut g,2,Action{card_id:Some(a.clone()),..Action::new("reveal")});top(&mut g);top(&mut g);
    assert_eq!(g.players[0].deck.len(),deck);assert!(g.players[0].hand.is_empty() && g.pending.is_none());
    assert!(g.players[0].assets[0].exhausted);assert!(g.board(&a).is_none());assert!(g.board(&b).unwrap().1.face_down);restore(&mut g);
}

#[test]
fn batch3_jc062_missing_target_is_not_replaced_by_same_print_and_actor_stays_frozen() {
    let mut g=game();let id=file(&mut g);let a=hidden(&mut g,"JC125",2);let deck=g.players[0].deck.len();
    act(&mut g,0,play(&id,vec![a.clone()]));g.remove_board(&a);let replacement=hidden(&mut g,"JC125",2);top(&mut g);
    assert!(g.pending.is_none());assert_eq!(g.players[0].deck.len(),deck);assert!(g.board(&replacement).unwrap().1.face_down);restore(&mut g);
}

#[test]
fn batch3_jc062_legal_candidate_sets_cover_zero_singles_and_pairs_without_permutations() {
    let mut g=game();let id=file(&mut g);for seat in 0..4{hidden(&mut g,"JC125",seat);}
    let c=g.players[0].hand.iter().find(|c|c.id==id).unwrap();let options=g.jc062_action_candidates(0,c);
    assert_eq!(options.len(),11);assert_eq!(options.iter().filter(|(a,_)|a.selected.as_ref().unwrap().len()==2).count(),6);
    let mut d=crate::batch3_inspect::definition();assert!(crate::batch3::validate_definition("JC125",&d).is_err());
    d.abilities[0].targets[0].max=3;assert!(rules::validate_ability("JC062",&d.abilities[0]).is_err());
    act(&mut g,0,play(&id,vec![]));let mut bad=g.clone();bad.stack.last_mut().unwrap().frame.as_mut().unwrap().already_paid.clear();
    assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
}
