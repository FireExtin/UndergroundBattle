//! Explicit offline layouts, real paid commands and optional declarations.
//! Primitive response changes below are named fixtures, not browser evidence.
use crate::jc029_tests::{apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject};
use crate::{catalog, deck, model::*, rules::*};

fn held(g: &mut Game, def: &str, seat: usize) -> String {
    let c = g.make_card(def, seat);
    let id = c.id.clone();
    g.players[seat].hand.push(c);
    id
}
fn move_fixture(g: &mut Game, id: &str, r: usize) {
    let (_, c) = g.remove_board(id).unwrap();
    g.regions[r].cards.push(c);
}
fn trigger(g: &mut Game, source: &str, target: Option<&str>, reveal: bool) {
    let (_, c) = g.board(source).unwrap();
    let (actor, def) = (c.controller, c.definition.clone());
    g.enter_triggers(actor, &def, source, reveal);
    g.drive().unwrap();
    checkpoint(g);
    choose(g, target.map(|id| vec![id.into()]).unwrap_or_default());
}
fn chase(source: &str, target: &str) -> Action {
    Action { card_id: Some(source.into()), target_id: Some(target.into()),
        ability_id: Some("chase-locked".into()), ..Action::new("activate") }
}
fn lock(g: &mut Game, id: &str, n: u32) {
    g.board_mut(id).unwrap().lock_markers = n;
}
fn revealed_dog(g: &mut Game, actor: usize) -> String {
    let old = board(g, "JC069", actor, 2);
    g.board_mut(&old).unwrap().face_down = true;
    fund(g, actor, "JC125", 2);
    apply(g, actor, Action {card_id: Some(old.clone()), ..Action::new("reveal")});
    pass_top(g);
    assert!(g.board(&old).is_none());
    g.regions[2].cards.iter().find(|c| c.definition == "JC069" && c.controller == actor).unwrap().id.clone()
}
fn fixture(name: &str, g: &Game) {
    checkpoint(g);
    if let Ok(dir) = std::env::var("GRAY_LOCK_FIXTURE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let room = envelope(g);
        std::fs::write(format!("{dir}/{name}.json"), serde_json::to_vec(&serde_json::json!({
            "scope":"explicit-offline-gray-lock-layout", "catalog":catalog::catalog(),
            "state":serde_json::to_string(&room).unwrap(),
            "views":(0..4).map(|s| room.view(s,0)).collect::<Vec<_>>()
        })).unwrap()).unwrap();
    }
}

#[test]
fn gray_lock_whole_original_fields_and_closed_definitions() {
    assert_eq!(catalog::catalog().cards.len(),116);
    for (id, name, subtype, permanent, temporary) in [
        ("JC069", "警犬", "狗", Icons::default(), Icons{combat:2,..Default::default()}),
        ("JZ43", "武装市民", "人类", Icons{influence:1,..Default::default()}, Icons{combat:1,..Default::default()}),
    ] {
        let c = catalog::card(id);
        assert_eq!((&*c.name, &*c.color, c.cost, c.defense), (name,"灰",2,Some(1)));
        assert_eq!(c.subtypes,[subtype]);
        assert!(c.loyalty.is_empty() && c.magic.is_empty() && !c.unique);
        assert_eq!(c.magic_icon,MagicIcon::None);
        assert_eq!((c.permanent_icons,c.temporary_icons),(permanent,temporary));
        assert_eq!(deck::copy_limit(c),Some(3));
    }
    let dog=definition("JC069");
    assert!(dog.traits.cannot_be_equipped && !dog.traits.public);
    assert_eq!(dog.abilities[0].event,Some(Event::Reveal));
    assert_eq!(dog.abilities[0].targets[0].range,Range::Anywhere);
    assert_eq!(dog.abilities[0].targets[0].relation,Relation::Any);
    let action=&dog.abilities[1];
    assert_eq!(action.timing,Timing::Fast);
    assert_eq!(action.per_turn_limit,Some(1));
    assert!(matches!(action.costs.as_slice(),[Cost::Assets(2)]));
    let citizen=definition("JZ43");
    assert!(citizen.traits.public && citizen.traits.city_play_only);
    assert_eq!(citizen.abilities[0].event,Some(Event::Enter));
    assert_eq!(citizen.abilities[0].targets[0].range,Range::SourceRegion);
    assert_eq!(citizen.abilities[0].targets[0].kind,EntityKind::Character);
    crate::rules::validate_definitions(definitions()).unwrap();
}

#[test]
fn gray_lock_reveal_optional_any_region_any_controller_hidden_or_faceup() {
    for actor in 0..4 { for controller in 0..4 { for hidden in [false,true] {
        let mut g=game(actor);
        let target=board(&mut g,"LC01",controller,4);
        g.board_mut(&target).unwrap().face_down=hidden;
        let dog=revealed_dog(&mut g,actor);
        assert!(g.pending.as_ref().unwrap().choice.options.iter().any(|o|o.id==target));
        choose(&mut g,vec![target.clone()]);
        pass_top(&mut g);
        assert_eq!(g.board(&target).unwrap().1.lock_markers,1);
        assert_eq!(g.board(&dog).unwrap().1.lock_markers,0);
        assert!(g.turn_ability_usage.is_empty());
        for viewer in 0..4 {
            let v=g.card_view(g.board(&target).unwrap().1,viewer,Some(4),None);
            assert_eq!(v.lock_markers,Some(1));
            if hidden && viewer!=controller {assert!(v.card_id.is_none() && v.text.is_none());}
        }
        if actor==0 && controller==1 && hidden { fixture("locked-enemy-hidden",&g); }
    }}}
    let mut g=game(0);
    let target=board(&mut g,"LC01",1,2);
    revealed_dog(&mut g,0);
    choose(&mut g,vec![]);
    assert!(g.stack.is_empty() && g.turn_ability_usage.is_empty());
    assert_eq!(g.board(&target).unwrap().1.lock_markers,0);
}

#[test]
fn gray_lock_dog_faceup_play_has_no_reveal_and_equipment_only_is_forbidden() {
    let mut g=game(0);
    let held=held(&mut g,"JC069",0);
    fund(&mut g,0,"JC125",2);
    apply(&mut g,0,Action{card_id:Some(held),region:Some(2),..Action::new("deploy")});
    pass_top(&mut g);
    assert!(g.pending.is_none());
    let dog=g.regions[2].cards.iter().find(|c|c.definition=="JC069").unwrap();
    let snapshot=g.source_snapshot(dog,Some(2));
    let equipment=&definition("BQ022").attachment.as_ref().unwrap().host;
    let ritual=&definition("JC089").attachment.as_ref().unwrap().host;
    assert!(!g.valid_binding(0,&snapshot,equipment,&dog.id));
    assert!(g.valid_binding(0,&snapshot,ritual,&dog.id));
}

#[test]
fn gray_lock_city_play_gate_is_atomic_and_effect_placement_is_distinct() {
    for actor in 0..4 {
        let mut g=game(actor);
        g.regions[2].card=g.make_card("DQJC107",0); // printed non-city
        let id=held(&mut g,"JZ43",actor);
        fund(&mut g,actor,"JC125",2);
        let deploy=Action{card_id:Some(id.clone()),region:Some(2),..Action::new("deploy")};
        reject(&mut g,actor,deploy.clone());
        reject(&mut g,actor,Action{card_id:Some(id.clone()),region:Some(2),..Action::new("conceal")});
        assert!(!g.legal_actions(actor).iter().any(|a|a.action.card_id.as_ref()==Some(&id) && a.action.kind=="deploy" && a.action.region==Some(2)));
        g.regions[2].card=g.make_card("DQJC112",0); // printed city
        apply(&mut g,actor,deploy);
        pass_top(&mut g);
        let citizen=g.regions[2].cards.iter().find(|c|c.definition=="JZ43").unwrap().id.clone();
        choose(&mut g,vec![citizen.clone()]);
        pass_top(&mut g);
        assert_eq!(g.board(&citizen).unwrap().1.lock_markers,1);
        // Explicit existing free-reveal placement, not a normal public-card conceal.
        g.regions[1].card=g.make_card("DQJC107",0);
        let placed=board(&mut g,"JZ43",actor,1);
        g.board_mut(&placed).unwrap().face_down=true;
        g.world_reveal(actor,&placed,false).unwrap();
        g.drive().unwrap();
        assert!(g.pending.is_some());
        choose(&mut g,vec![]);
    }
}

#[test]
fn gray_lock_citizen_resolution_time_branch_adds_one_or_deals_one_not_both() {
    for actor in 0..4 { for initial in [0,2] { for resolved in [0,3] {
        let mut g=game(actor);
        let source=board(&mut g,"JZ43",actor,2);
        let target=board(&mut g,"JC059",(actor+2)%4,2); // defense 2
        lock(&mut g,&target,initial);
        trigger(&mut g,&source,Some(&target),false);
        lock(&mut g,&target,resolved); // explicit response-time marker layout
        pass_top(&mut g);
        let c=g.board(&target).unwrap().1;
        assert_eq!((c.lock_markers,c.damage),if resolved==0 {(1,0)}else{(3,1)});
        if actor==0 && initial==0 && resolved==3 {fixture("citizen-one-damage",&g);}
    }}}
}

#[test]
fn gray_lock_citizen_local_target_rules_and_original_region_replacement() {
    for replacement_before_selection in [false,true] {
        let mut g=game(0);
        let source=board(&mut g,"JZ43",0,2);
        let target=board(&mut g,"LC01",1,2);
        let remote=board(&mut g,"LC01",1,3);
        let hidden=board(&mut g,"LC01",1,2);
        g.board_mut(&hidden).unwrap().face_down=true;
        let (_,c)=g.board(&source).unwrap();let snapshot=g.source_snapshot(c,Some(2));
        let slot=&definition("JZ43").abilities[0].targets[0];
        assert!(g.valid_binding(0,&snapshot,slot,&target));
        assert!(!g.valid_binding(0,&snapshot,slot,&remote));
        assert!(!g.valid_binding(0,&snapshot,slot,&hidden));
        g.enter_triggers(0,"JZ43",&source,false);g.drive().unwrap();
        if replacement_before_selection {
            g.regions[2].card=g.make_card("DQJC108",0);
            let choice_id=g.pending.as_ref().unwrap().choice.id.clone();
            reject(&mut g,0,Action{choice_id:Some(choice_id),selected:Some(vec![target.clone()]),..Action::new("choose")});
            choose(&mut g,vec![]);
        } else {
            choose(&mut g,vec![target.clone()]);
            g.regions[2].card=g.make_card("DQJC108",0);
            pass_top(&mut g);
        }
        assert_eq!(g.board(&target).unwrap().1.lock_markers,0);
    }
}

#[test]
fn gray_lock_citizen_source_leave_or_move_preserves_frozen_locality_and_controller() {
    for leave in [false,true] {
        let mut g=game(0);
        let source=board(&mut g,"JZ43",1,2);
        g.board_mut(&source).unwrap().controller=0;
        let target=board(&mut g,"LC01",2,2);
        trigger(&mut g,&source,Some(&target),false);
        assert_eq!(g.stack.last().unwrap().frame.as_ref().unwrap().actor,0);
        if leave {g.return_hand(&source);} else {move_fixture(&mut g,&source,4);}
        pass_top(&mut g);
        assert_eq!(g.board(&target).unwrap().1.lock_markers,1);
        if leave {assert!(g.players[1].hand.iter().any(|c|c.definition=="JZ43"));}
    }
}

#[test]
fn gray_lock_target_response_move_hide_or_new_instance_cancels_citizen_frame() {
    for change in 0..3 {
        let mut g=game(0);
        let source=board(&mut g,"JZ43",0,2);
        let target=board(&mut g,"LC01",1,2);
        trigger(&mut g,&source,Some(&target),false);
        match change {
            0=>move_fixture(&mut g,&target,3),
            1=>{let (r,c)=g.remove_board(&target).unwrap();let mut c=g.fresh(c);c.face_down=true;g.regions[r].cards.push(c);},
            _=>{g.return_hand(&target);board(&mut g,"LC01",1,2);},
        }
        pass_top(&mut g);
        assert!(g.regions.iter().flat_map(|r|&r.cards).all(|c|c.lock_markers==0 && c.damage==0));
    }
}

#[test]
fn gray_lock_shield_stops_all_effect_and_barrier_rejects_before_cost() {
    for def in ["JC069","JZ43"] { for actor in 0..4 {
        let mut g=game(actor);let target=board(&mut g,"JC059",(actor+2)%4,2);
        g.board_mut(&target).unwrap().shield=1;
        let source=board(&mut g,def,actor,2);
        trigger(&mut g,&source,Some(&target),def=="JC069");pass_top(&mut g);
        let c=g.board(&target).unwrap().1;assert_eq!((c.shield,c.lock_markers,c.damage),(0,0,0));
    }}
    let mut g=game(0);let dog=board(&mut g,"JC069",0,0);let target=board(&mut g,"JZ67",2,4);lock(&mut g,&target,1);fund(&mut g,0,"JC125",2);
    reject(&mut g,0,chase(&dog,&target));assert_eq!(g.resources(0),2);
}

#[test]
fn gray_lock_chase_is_fast_pays_two_no_exhaust_and_current_anchor_region() {
    for actor in 0..4 {for hidden in [false,true] {
        let mut g=game(actor);let dog=board(&mut g,"JC069",actor,0);
        g.board_mut(&dog).unwrap().exhausted=true;
        let target=board(&mut g,"LC01",(actor+1)%4,4);g.board_mut(&target).unwrap().face_down=hidden;lock(&mut g,&target,2);fund(&mut g,actor,"JC125",2);
        apply(&mut g,actor,chase(&dog,&target));
        assert_eq!(g.resources(actor),0);assert_eq!(g.board(&dog).unwrap().0,0);
        move_fixture(&mut g,&target,3); // response moves locked real target
        pass_top(&mut g);
        assert_eq!(g.board(&dog).unwrap().0,3);
        assert!(g.board(&dog).unwrap().1.exhausted);
        assert_eq!(g.current_icons(g.board(&dog).unwrap().1,3).investigation,1);
        assert_eq!(g.board(&target).unwrap().1.lock_markers,2);
        assert_eq!(g.turn_ability_usage.len(),1);
        if actor==0 && hidden {fixture("chase-current-hidden-anchor",&g);}
    }}
}

#[test]
fn gray_lock_chase_no_in_place_declaration_or_unlocked_target_and_failures_atomic() {
    let mut g=game(0);let dog=board(&mut g,"JC069",0,0);let target=board(&mut g,"LC01",1,0);lock(&mut g,&target,1);fund(&mut g,0,"JC125",2);
    reject(&mut g,0,chase(&dog,&target));
    move_fixture(&mut g,&target,4);lock(&mut g,&target,0);
    reject(&mut g,0,chase(&dog,&target));lock(&mut g,&target,1);
    g.players[0].assets.pop();reject(&mut g,0,chase(&dog,&target));
    assert!(g.turn_ability_usage.is_empty());
}

#[test]
fn gray_lock_chase_response_colocation_skips_move_but_independent_icon_still_applies() {
    let mut g=game(0);let dog=board(&mut g,"JC069",0,0);let target=board(&mut g,"LC01",1,4);lock(&mut g,&target,1);fund(&mut g,0,"JC125",2);
    apply(&mut g,0,chase(&dog,&target));move_fixture(&mut g,&target,0);pass_top(&mut g);
    assert_eq!(g.board(&dog).unwrap().0,0);
    assert_eq!(g.current_icons(g.board(&dog).unwrap().1,0).investigation,1);
}

#[test]
fn gray_lock_chase_guard_failure_still_spends_once_and_source_reentry_does_not_inherit() {
    for change in 0..4 {
        let mut g=game(0);let dog=board(&mut g,"JC069",0,0);let target=board(&mut g,"LC01",1,4);lock(&mut g,&target,1);fund(&mut g,0,"JC125",4);
        apply(&mut g,0,chase(&dog,&target));
        match change {
            0=>lock(&mut g,&target,0),
            1=>{g.return_hand(&target);},
            2=>{g.return_hand(&dog);board(&mut g,"JC069",0,0);},
            _=>{let (r,c)=g.remove_board(&dog).unwrap();let mut c=g.fresh(c);c.face_down=true;g.regions[r].cards.push(c);},
        }
        pass_top(&mut g);assert_eq!(g.resources(0),2);
        assert_eq!(g.turn_ability_usage[0].source_instance,dog);
        assert!(g.turn_attribute_modifiers.is_empty());
        if change<2 {let anchor=board(&mut g,"LC01",1,4);lock(&mut g,&anchor,1);reject(&mut g,0,chase(&dog,&anchor));}
    }
}

#[test]
fn gray_lock_per_turn_usage_is_original_instance_not_owner_or_controller() {
    let mut g=game(0);let a=board(&mut g,"JC069",1,0);g.board_mut(&a).unwrap().controller=0;
    let b=board(&mut g,"JC069",1,0);let target=board(&mut g,"LC01",1,4);lock(&mut g,&target,1);fund(&mut g,0,"JC125",4);fund(&mut g,1,"JC125",2);
    apply(&mut g,0,chase(&a,&target));pass_top(&mut g);
    apply(&mut g,1,chase(&b,&target));pass_top(&mut g);
    assert_eq!(g.turn_ability_usage.len(),2);
    move_fixture(&mut g,&target,0);
    g.board_mut(&a).unwrap().controller=1;
    fund(&mut g,1,"JC125",2);reject(&mut g,1,chase(&a,&target));
    assert_eq!(g.board(&a).unwrap().1.owner,1);
}

#[test]
fn gray_lock_marker_move_retains_while_leave_flip_and_death_clear() {
    for change in 0..5 {
        let mut g=game(0);let id=board(&mut g,"LC01",1,2);lock(&mut g,&id,3);
        move_fixture(&mut g,&id,4);assert_eq!(g.board(&id).unwrap().1.lock_markers,3);
        match change {
            0=>g.return_hand(&id),
            1=>g.to_bottom(&id),
            2=>g.remove_dead(&id,RemovalCause::Destroy),
            3=>{let (r,c)=g.remove_board(&id).unwrap();let mut c=g.fresh(c);c.face_down=true;g.regions[r].cards.push(c);},
            _=>{g.board_mut(&id).unwrap().face_down=true;g.world_reveal(1,&id,false).unwrap();},
        }
        assert!(g.board(&id).is_none());
        assert!(g.regions.iter().flat_map(|r|&r.cards).chain(g.players.iter().flat_map(|p|p.hand.iter().chain(&p.deck).chain(&p.graveyard))).all(|c|c.lock_markers==0));
        checkpoint(&g);
    }
}

#[test]
fn gray_lock_lethal_damage_uses_existing_controller_frozen_death_and_owner_graveyard() {
    let mut g=game(0);let source=board(&mut g,"JZ43",0,2);let target=board(&mut g,"JZ31",1,2);g.board_mut(&target).unwrap().controller=2;lock(&mut g,&target,1);
    trigger(&mut g,&source,Some(&target),false);pass_top(&mut g);
    assert!(g.board(&target).is_none());assert_eq!(g.players[1].graveyard[0].lock_markers,0);
    let pending=g.pending.as_ref().unwrap();assert_eq!(pending.seat,2);
    choose(&mut g,vec!["accept".into()]);pass_top(&mut g);
    assert_eq!(g.regions[2].influence[g.team(2)],1);
}

#[test]
fn gray_lock_zero_marker_fields_keep_old_card_and_view_serialization_shape() {
    let mut g=game(0);let id=board(&mut g,"LC01",0,2);let c=g.board(&id).unwrap().1;
    let v=serde_json::to_value(c).unwrap();assert!(v.get("lock_markers").is_none());
    let view=serde_json::to_value(g.card_view(c,0,Some(2),None)).unwrap();assert!(view.get("lockMarkers").is_none());
    let mut old=v;old.as_object_mut().unwrap().remove("lock_markers");let restored:Card=serde_json::from_value(old.clone()).unwrap();
    assert_eq!(restored.lock_markers,0);assert_eq!(serde_json::to_value(restored).unwrap(),old);
}

#[test]
fn gray_lock_restore_rejects_transplanted_program_and_malformed_frozen_frame() {
    let mut g=game(0);let dog=board(&mut g,"JC069",0,0);let target=board(&mut g,"LC01",1,4);lock(&mut g,&target,1);fund(&mut g,0,"JC125",2);
    apply(&mut g,0,chase(&dog,&target));
    for change in 0..10 {
        let mut broken=g.clone();let frame=broken.stack.last_mut().unwrap().frame.as_mut().unwrap();
        match change {
            0=>frame.source.card.definition="JC002".into(),
            1=>frame.actor=2,
            2=>frame.source.card.face_down=true,
            3=>frame.targets[0].spec.range=Range::SourceRegion,
            4=>frame.steps.push(frame.steps[0].clone()),
            5=>frame.steps[0].context=1,
            6=>frame.already_paid.clear(),
            7=>frame.steps[0].op=Op::ForEachLivingPlayer(vec![Op::JC069ChaseLockedTarget]),
            8=>frame.ability_key="reveal-lock".into(),
            _=>frame.cursor=2,
        }
        let serialized=serde_json::to_string(&broken).unwrap();
        assert!(Game::from_persisted(&serialized).is_err());
        let mut room=envelope(&g);room.game=broken;
        let state=serde_json::to_string(&room).unwrap();
        assert!(crate::room::RoomEnvelope::from_persisted(&state).is_err());
        if let Ok(dir)=std::env::var("GRAY_LOCK_INVALID_DIR") {
            std::fs::create_dir_all(&dir).unwrap();std::fs::write(format!("{dir}/invalid-{change:02}.json"),serde_json::to_vec(&serde_json::json!({"state":state})).unwrap()).unwrap();
        }
    }
}

#[test]
fn gray_lock_restore_rejects_impossible_accepted_cursor_and_frame_choice_containers() {
    let mut invalid_index=0;
    for kind in ["chase-locked", "reveal-lock", "citizen-lock"] {
        let mut g=game(0);
        let source=board(&mut g,if kind=="citizen-lock" {"JZ43"}else{"JC069"},0,0);
        let target=board(&mut g,"LC01",1,if kind=="citizen-lock" {0}else{4});
        if kind=="chase-locked" {
            lock(&mut g,&target,1);fund(&mut g,0,"JC125",2);
            apply(&mut g,0,chase(&source,&target));
        } else { trigger(&mut g,&source,Some(&target),kind=="reveal-lock"); }
        let original=g.stack.last().unwrap().frame.as_ref().unwrap().clone();
        checkpoint(&g);
        for container in 0..3 { for (guard,cursor) in [
            (GuardState::Unchecked,0),(GuardState::Accepted,0),(GuardState::Cancelled,0),
            (GuardState::Unchecked,1),(GuardState::Accepted,1),
        ] {
            let mut broken=g.clone();
            let mut frame=original.clone();frame.guard=guard;frame.cursor=cursor;
            match container {
                0=>broken.stack.last_mut().unwrap().frame=Some(frame),
                1=>{broken.stack.clear();broken.effects.push_front(Effect::Frame{frame:Box::new(frame)});},
                _=>{broken.stack.clear();broken.choice(0,"target","非法灰色帧暂停".into(),vec![],0,0,None,
                    ChoiceResolution::Frame{frame:Box::new(frame),choice:FrameChoice::Region});},
            }
            let legitimate=container<2 && cursor==0
                && matches!(if container==0 {&broken.stack.last().unwrap().frame.as_ref().unwrap().guard}
                    else {match broken.effects.front().unwrap(){Effect::Frame{frame}=>&frame.guard,_=>unreachable!()}},GuardState::Unchecked);
            let serialized=serde_json::to_string(&broken).unwrap();
            let mut room=envelope(&g);room.game=broken;
            if container>0 {room.pacing.window=None;}
            let state=serde_json::to_string(&room).unwrap();
            assert_eq!(Game::from_persisted(&serialized).is_ok(),legitimate,"{kind}/{container}/{cursor}");
            assert_eq!(crate::room::RoomEnvelope::from_persisted(&state).is_ok(),legitimate);
            if !legitimate {
                if let Ok(dir)=std::env::var("GRAY_LOCK_GUARD_INVALID_DIR") {
                    std::fs::create_dir_all(&dir).unwrap();
                    std::fs::write(format!("{dir}/invalid-guard-{invalid_index:02}.json"),serde_json::to_vec(&serde_json::json!({"state":state})).unwrap()).unwrap();
                }
                invalid_index+=1;
            }
        }}
        if kind=="chase-locked" {
            // Losing the lock during responses remains a valid persisted frame;
            // its genuine Unchecked target guard cancels the whole operation.
            lock(&mut g,&target,0);checkpoint(&g);pass_top(&mut g);
            assert_eq!(g.board(&source).unwrap().0,0);
            assert_eq!(g.current_icons(g.board(&source).unwrap().1,0).investigation,0);
        }
    }
    assert_eq!(invalid_index,39);
}

#[test]
fn gray_lock_canonical_gates_reject_transplants_nested_ops_modes_and_field_edits() {
    for id in ["JC069","JZ43"] {
        let canonical=definition(id);
        for i in 0..canonical.abilities.len() {
            for change in 0..10 {
                let mut a=canonical.abilities[i].clone();
                match change {
                    0=>a.timing=Timing::Standard,
                    1=>a.event=Some(Event::Death),
                    2=>a.costs.push(Cost::ExhaustSource),
                    3=>a.targets[0].range=Range::Mobility,
                    4=>a.targets[0].relation=Relation::ControlledByActor,
                    5=>a.per_turn_limit=Some(2),
                    6=>a.ops=vec![Op::ForEachLivingPlayer(a.ops.clone())],
                    7=>a.modes.push(Mode{key:"bad".into(),label:"bad".into(),targets:a.targets.clone(),ops:a.ops.clone()}),
                    8=>a.targets[0].kind=EntityKind::Any,
                    _=>a.response_policy=ResponsePolicy::Immediate,
                }
                assert!(crate::rules::validate_ability(id,&a).is_err(),"{id}/{i}/{change}");
            }
            assert!(crate::rules::validate_ability("JC002",&canonical.abilities[i]).is_err());
        }
        for change in 0..5 {
            let mut registry=definitions().clone();let d=registry.get_mut(id).unwrap();
            match change {
                0=>d.abilities.clear(),1=>d.abilities.push(d.abilities[0].clone()),
                2=>d.traits.public=!d.traits.public,3=>d.graveyard_face_up=true,
                _=>d.traits.cannot_be_equipped=!d.traits.cannot_be_equipped,
            }
            assert!(crate::rules::validate_definitions(&registry).is_err());
        }
    }
    let mut registry=definitions().clone();registry.get_mut("JC002").unwrap().traits.city_play_only=true;
    assert!(crate::rules::validate_definitions(&registry).is_err());
    let mut a=definition("JC002").abilities[0].clone();
    a.targets[0].predicate=Some(TargetPredicate::JC069LockedAnchor);
    assert!(crate::rules::validate_ability("JC002",&a).is_err());
}

#[test]
fn gray_lock_declaration_restore_wrong_actor_duplicate_old_choice_are_atomic() {
    let mut g=game(0);let source=board(&mut g,"JZ43",0,2);let target=board(&mut g,"LC01",1,2);
    g.enter_triggers(0,"JZ43",&source,false);g.drive().unwrap();checkpoint(&g);
    let command=Action{choice_id:Some(g.pending.as_ref().unwrap().choice.id.clone()),selected:Some(vec![target.clone()]),..Action::new("choose")};
    reject(&mut g,2,command.clone());let mut restored=Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
    apply(&mut g,0,command.clone());apply(&mut restored,0,command.clone());
    assert_eq!(serde_json::to_value(&g).unwrap(),serde_json::to_value(&restored).unwrap());
    reject(&mut g,0,command);pass_top(&mut g);pass_top(&mut restored);
    assert_eq!(serde_json::to_value(&g).unwrap(),serde_json::to_value(&restored).unwrap());
    assert_eq!(g.board(&target).unwrap().1.lock_markers,1);
}

#[test]
fn gray_lock_damage_prevention_and_self_target_use_existing_atomic_damage() {
    let mut g=game(0);let source=board(&mut g,"JZ43",0,2);let target=board(&mut g,"JC059",1,2);lock(&mut g,&target,2);
    g.turn_attribute_modifiers.push(TurnAttributeModifier{target_instance:target.clone(),defense_bonus:0,
        kill_bonus:0,grants_retreat:false,printed_defense_override:None,ordinary_icons:Icons::default(),
        grants_renown:false,prevents_damage:true,expires_turn:g.turn});
    trigger(&mut g,&source,Some(&target),false);pass_top(&mut g);
    assert_eq!((g.board(&target).unwrap().1.damage,g.board(&target).unwrap().1.lock_markers),(0,2));
    g.return_hand(&target);lock(&mut g,&source,1);trigger(&mut g,&source,Some(&source),false);pass_top(&mut g);
    assert!(g.board(&source).is_none());assert_eq!(g.players[0].graveyard[0].lock_markers,0);
}

#[test]
fn gray_lock_turn_end_expires_icon_and_reopens_original_dog_limit() {
    let mut g=game(0);let dog=board(&mut g,"JC069",0,0);let target=board(&mut g,"LC01",1,4);lock(&mut g,&target,1);fund(&mut g,0,"JC125",2);
    apply(&mut g,0,chase(&dog,&target));pass_top(&mut g);
    move_fixture(&mut g,&target,0);let previous=g.turn;
    // Advance the genuine four-seat windows and mandatory choices, no turn assignment.
    for _ in 0..600 {
        if g.turn>previous {break;}
        if let Some(p)=g.pending.as_ref() {
            let _ = p; crate::unit_support::resolve_choice(&mut g);
        } else {
            let s=(0..4).find(|s|g.legal_actions(*s).iter().any(|a|a.action.kind=="pass")).unwrap();
            apply(&mut g,s,Action::new("pass"));
        }
    }
    assert!(g.turn>previous);assert!(g.turn_ability_usage.is_empty());
    assert_eq!(g.current_icons(g.board(&dog).unwrap().1,4).investigation,0);
    while !g.legal_actions(0).iter().any(|a|a.action==chase(&dog,&target)) {
        if let Some(p)=g.pending.as_ref() {
            let _ = p; crate::unit_support::resolve_choice(&mut g);
        } else {
            let s=(0..4).find(|s|g.legal_actions(*s).iter().any(|a|a.action.kind=="pass")).unwrap();apply(&mut g,s,Action::new("pass"));
        }
        assert!(g.turn<previous+3,"chase did not reopen");
    }
    apply(&mut g,0,chase(&dog,&target));pass_top(&mut g);
    assert_eq!(g.board(&dog).unwrap().0,0);
    assert_eq!(g.current_icons(g.board(&dog).unwrap().1,0).investigation,1);
}

#[cfg(feature="native")]
#[tokio::test]
async fn gray_lock_sqlite_receipt_restore_and_duplicate_do_not_add_two_markers() {
    use crate::room::{RoomCommand,SessionAction};
    use crate::service::{CreateRoom,JoinRoom,Store};
    let dir=tempfile::tempdir().unwrap();let path=dir.path().join("gray-lock-receipts.sqlite");
    let store=Store::open(&path).unwrap();
    let host=store.create(CreateRoom{name:"P0".into(),mode:"teams".into(),deck_id:"watchers".into(),deck_draft:None}).await.unwrap();
    let mut sessions=vec![host];
    for seat in 1..4 {sessions.push(store.join(JoinRoom{invite_code:sessions[0].invite_code.clone(),name:format!("P{seat}"),deck_id:"watchers".into(),deck_draft:None}).await.unwrap());}
    drop(store);
    // Native SQLite test fixture only; production browser test will start naturally.
    let mut g=game(0);g.room_id=sessions[0].room_id.clone();g.invite_code=sessions[0].invite_code.clone();
    let source=board(&mut g,"JZ43",0,2);let target=board(&mut g,"LC01",1,2);
    g.enter_triggers(0,"JZ43",&source,false);g.drive().unwrap();
    let room=envelope(&g);let state=serde_json::to_string(&room).unwrap();
    let db=rusqlite::Connection::open(&path).unwrap();
    db.execute("UPDATE rooms SET state=?2,initial_state=?2,revision=?3 WHERE id=?1",rusqlite::params![g.room_id,state,room.revision]).unwrap();
    db.execute("DELETE FROM journal WHERE room_id=?1",[&g.room_id]).unwrap();drop(db);
    let command=RoomCommand{command_id:"gray-lock-one".into(),expected_version:room.revision,
        action:SessionAction::Game{action:Action{choice_id:Some(g.pending.as_ref().unwrap().choice.id.clone()),selected:Some(vec![target.clone()]),..Action::new("choose")}}};
    let expected=room.transition(0,Some(command.clone()),0).unwrap();assert!(expected.error_code.is_none());
    let store=Store::open(&path).unwrap();let accepted=store.command_at_now(&g.room_id,&sessions[0].token,command.clone(),0).await.unwrap();drop(store);
    let db=rusqlite::Connection::open(&path).unwrap();let before:String=db.query_row("SELECT state FROM rooms WHERE id=?1",[&g.room_id],|r|r.get(0)).unwrap();drop(db);
    let store=Store::open(&path).unwrap();let duplicate=store.command_at_now(&g.room_id,&sessions[0].token,command.clone(),90_000).await.unwrap();
    assert_eq!(serde_json::to_value(accepted).unwrap(),serde_json::to_value(duplicate).unwrap());
    let mut conflict=command;conflict.expected_version+=1;assert_eq!(store.command_at_now(&g.room_id,&sessions[0].token,conflict,90_000).await.unwrap_err().error,"command_id_conflict");
    let db=rusqlite::Connection::open(&path).unwrap();let after:String=db.query_row("SELECT state FROM rooms WHERE id=?1",[&g.room_id],|r|r.get(0)).unwrap();assert_eq!(after,before);
    let count:i64=db.query_row("SELECT count(*) FROM commands WHERE room_id=?1",[&g.room_id],|r|r.get(0)).unwrap();assert_eq!(count,1);
    let mut resumed=crate::room::RoomEnvelope::from_persisted(&after).unwrap().game;pass_top(&mut resumed);
    assert_eq!(resumed.board(&target).unwrap().1.lock_markers,1);
}

#[test]
fn gray_lock_continuous_room_protocol_pause_reopen_and_real_response_passes() {
    use crate::room::{Decision, RoomCommand, RoomEnvelope, SessionAction};
    for kind in ["citizen-lock", "citizen-damage", "dog-chase-hidden"] {
        let mut g=game(0);
        let target=board(&mut g,"JC059",3,2);
        let source=board(&mut g,if kind=="dog-chase-hidden" {"JC069"}else{"JZ43"},0,0);
        let first=if kind=="dog-chase-hidden" {
            g.board_mut(&target).unwrap().face_down=true;lock(&mut g,&target,1);fund(&mut g,0,"JC125",2);
            SessionAction::Game{action:chase(&source,&target)}
        } else {
            move_fixture(&mut g,&source,2);
            if kind=="citizen-damage" {lock(&mut g,&target,2);}
            g.enter_triggers(0,"JZ43",&source,false);g.drive().unwrap();
            SessionAction::Game{action:Action{choice_id:Some(g.pending.as_ref().unwrap().choice.id.clone()),selected:Some(vec![target.clone()]),..Action::new("choose")}}
        };
        let mut room=envelope(&g);let initial=serde_json::to_string(&room).unwrap();let mut steps=Vec::new();
        let mut record=|room:&mut RoomEnvelope, seat:usize, action:SessionAction, now:u64| {
            let command=RoomCommand{command_id:format!("gray-{kind}-{}",steps.len()),expected_version:room.revision,action};
            let transition=room.transition(seat,Some(command.clone()),now).unwrap();
            assert!(transition.error_code.is_none(),"{:?}",transition.error_message);
            *room=RoomEnvelope::from_persisted(&transition.state).unwrap();
            steps.push(serde_json::json!({"seat":seat,"command":command,"serverNowMs":now.to_string(),"transition":transition,"views":(0..4).map(|s|room.view(s,now)).collect::<Vec<_>>() }));
        };
        record(&mut room,0,first,10);record(&mut room,0,SessionAction::PauseRoom,11);
        assert!(room.pacing.pause.is_some());let saved=serde_json::to_string(&room).unwrap();
        room=RoomEnvelope::from_persisted(&saved).unwrap();assert_eq!(serde_json::to_string(&room).unwrap(),saved);
        record(&mut room,0,SessionAction::ResumeRoom,180_000);
        for n in 0..20 {
            if room.game.stack.is_empty(){break;}
            let window=room.pacing.window.clone().unwrap();
            let seat=*window.members.iter().find(|(_,d)|matches!(d,Decision::Undecided{..})).unwrap().0;
            record(&mut room,seat,SessionAction::PassResponse{window_id:window.id},180_001+n);
        }
        assert!(room.stack.is_empty());
        let c=room.game.board(&target).unwrap().1;
        assert_eq!((c.lock_markers,c.damage),match kind {"citizen-lock"=>(1,0),"citizen-damage"=>(2,1),_=>(1,0)});
        if kind=="dog-chase-hidden" {assert_eq!(room.game.board(&source).unwrap().0,2);assert_eq!(room.game.current_icons(room.game.board(&source).unwrap().1,2).investigation,1);}
        if let Ok(dir)=std::env::var("GRAY_LOCK_ROOM_TRACE_DIR") {
            std::fs::create_dir_all(&dir).unwrap();std::fs::write(format!("{dir}/{kind}.json"),serde_json::to_vec(&serde_json::json!({"scope":"explicit native layout then continuous real Room session protocol; pause/reopen and responses","initialState":initial,"steps":steps,"terminalState":serde_json::to_string(&room).unwrap()})).unwrap()).unwrap();
        }
    }
}
