//! Explicit primitive layouts plus real paid actions and strict Game/Room restore.
use crate::{catalog, jc054, model::*, room::RoomEnvelope as Room, rules::*, unit_support as u};
use serde_json::{json, Value};
use crate::room::{Decision, RoomCommand, SessionAction};

const SCENARIOS: &[&str] = &["paid-deploy", "natural-end", "mandatory-end", "mandatory-order", "controlled-conceal", "source-replaced", "paid-response-reveal", "pause-mandatory-order"];

struct Trace { name: &'static str, room: Room, initial: Value, steps: Vec<Value> }
impl Trace {
    fn new(name: &'static str, mut g: Game) -> Self {
        let index=SCENARIOS.iter().position(|n| *n==name).unwrap();
        g.room_id=format!("5400000000000000000000{index:02}");
        let room=crate::jc029_tests::envelope(&g);
        let initial=json!({"state":serde_json::to_string(&room).unwrap(),"views":(0..g.players.len()).map(|s|room.view(s,0)).collect::<Vec<_>>()});
        Self { name, room, initial, steps:vec![] }
    }
    fn command(&mut self, seat: usize, action: SessionAction, rejected: bool) {
        let state=serde_json::to_string(&self.room).unwrap();
        let command=RoomCommand { command_id:format!("jc054-{}-{}",self.name,self.steps.len()), expected_version:self.room.revision, action };
        let expected=self.room.transition(seat,Some(command.clone()),0).unwrap();
        assert_eq!(expected.error_code.is_some(),rejected,"{} {:?}: {:?}",self.name,command.action,expected.error_message);
        if rejected { assert_eq!(expected.state,state); }
        else { assert_eq!(serde_json::to_string(&self.room.replay_events(&expected.journal).unwrap()).unwrap(),expected.state); }
        self.room=Room::from_persisted(&expected.state).unwrap();
        restore_both(&self.room.game);
        self.steps.push(json!({"state":state,"seat":seat,"command":command,"serverNowMs":"0","expected":expected,
            "views":(0..self.room.game.players.len()).map(|s|self.room.view(s,0)).collect::<Vec<_>>() }));
    }
    fn action(&mut self, seat: usize, action: Action) {
        if let Some(w)=self.room.pacing.window.clone() {
            let intent_id=format!("jc054-intent-{}",self.steps.len());
            self.command(seat,SessionAction::BeginResponse { window_id:w.id.clone(),intent_id:intent_id.clone() },false);
            self.command(seat,SessionAction::SubmitResponse { window_id:w.id,intent_id,action },false);
        } else { self.command(seat,SessionAction::Game { action },false); }
    }
    fn pass(&mut self) {
        if let Some(w)=self.room.pacing.window.clone() {
            let seat=*w.members.iter().find(|(_,d)|matches!(d,Decision::Undecided { .. })).unwrap().0;
            self.command(seat,SessionAction::PassResponse { window_id:w.id },false);
        } else {
            let seat=(0..self.room.game.players.len()).find(|s|self.room.game.legal_actions(*s).iter().any(|a|a.action.kind=="pass")).unwrap();
            self.action(seat,Action::new("pass"));
        }
    }
    fn choose_first(&mut self) {
        let p=self.room.game.pending.clone().unwrap();
        self.action(p.seat,Action { choice_id:Some(p.choice.id),selected:Some(p.choice.options.iter().take(p.choice.min.unwrap_or(0)).map(|o|o.id.clone()).collect()),..Action::new("choose") });
    }
    fn finish(&mut self) {
        for _ in 0..128 {
            if self.room.game.pending.is_some() { self.choose_first(); }
            else if self.room.game.stack.is_empty() { return; }
            else { self.pass(); }
        }
        panic!("JC054 bounded response did not finish");
    }
    fn export(self) -> Value { json!({"name":self.name,"initial":self.initial,"steps":self.steps}) }
}

fn restore_both(g: &Game) {
    let state = serde_json::to_string(g).unwrap();
    let copy = Game::from_persisted(&state).unwrap();
    assert_eq!(serde_json::to_string(&copy).unwrap(), state);
    let room = crate::jc029_tests::envelope(g);
    let state = serde_json::to_string(&room).unwrap();
    assert_eq!(serde_json::to_string(&Room::from_persisted(&state).unwrap()).unwrap(), state);
}
fn top(g: &mut Game) {
    let id = g.stack.last().unwrap().id.clone();
    for _ in 0..64 {
        if !g.stack.iter().any(|s| s.id == id) { return; }
        assert!(g.pending.is_none()); u::pass(g);
    }
    panic!("one responsive frame did not resolve");
}
fn burst() -> (Game, String) {
    let mut g = u::game();
    let worm = u::field(&mut g, "JC054", 0);
    let hidden = g.hide_in_place_successor(&worm).unwrap();
    g.drive().unwrap();
    assert_eq!(g.stack[0].frame.as_ref().unwrap().ability_key, jc054::BURST);
    restore_both(&g);
    (g, hidden.id)
}

#[test]
fn jc054_complete_print_and_closed_definitions() {
    let c = catalog::card("JC054");
    assert_eq!((c.cost, c.defense, c.unique), (4, Some(1), false));
    assert_eq!(c.subtypes, ["蠕虫"]);
    assert!(c.loyalty.is_empty());
    assert_eq!((c.permanent_icons.combat, c.temporary_icons.combat), (1, 1));
    let d = jc054::definition();
    assert_eq!(d.abilities.len(), 2);
    assert_eq!((d.abilities[0].event, d.abilities[1].event), (Some(Event::Hide), Some(Event::EndStep)));
    for a in &d.abilities {
        jc054::validate_ability("JC054", a).unwrap();
        assert!(jc054::validate_ability("JC125", a).is_err());
        let mut bad = a.clone(); bad.response_policy = ResponsePolicy::Immediate;
        assert!(jc054::validate_ability("JC054", &bad).is_err());
    }
    let mut missing = d.clone(); missing.abilities.pop();
    assert!(jc054::validate_definition("JC054", &missing).is_err());
}

#[test]
fn jc054_real_paid_deploy_secret_deploy_and_reveal_do_not_emit_conceal() {
    let mut g = u::game();
    u::fund(&mut g, 0, "JC125", 4);
    let held = u::held(&mut g, "JC054", 0);
    u::act(&mut g, 0, Action { card_id: Some(held), region: Some(0), ..Action::new("deploy") });
    top(&mut g);
    assert!(g.effects.is_empty() && g.stack.is_empty());
    let mut h = u::game();
    u::fund(&mut h, 0, "JC125", 5);
    let held = u::held(&mut h, "JC054", 0);
    u::act(&mut h, 0, Action { card_id: Some(held), region: Some(0), ..Action::new("conceal") });
    let hidden = h.regions[0].cards[0].id.clone();
    assert!(h.effects.is_empty() && h.stack.is_empty());
    u::act(&mut h, 0, Action { card_id: Some(hidden), ..Action::new("reveal") });
    top(&mut h);
    assert!(h.effects.is_empty() && h.stack.is_empty());
    assert!(!h.regions[0].cards[0].face_down);
    restore_both(&h);
}

#[test]
fn jc054_end_and_burst_are_mandatory_separate_responsive_frames_with_simultaneous_damage() {
    let mut g = u::game();
    let worm = u::field(&mut g, "JC054", 0);
    g.board_mut(&worm).unwrap().exhausted = true;
    let ally = u::field(&mut g, "JC125", 1);
    let enemy = u::field(&mut g, "JC125", 2);
    let hidden = u::field(&mut g, "JC125", 3);
    g.board_mut(&hidden).unwrap().face_down = true;
    g.begin_window(Window::End); g.drive().unwrap();
    assert!(g.pending.is_none());
    assert_eq!(g.stack.len(), 1);
    assert_eq!(g.stack[0].frame.as_ref().unwrap().ability_key, jc054::END);
    restore_both(&g);
    top(&mut g);
    assert!(g.board(&worm).is_none());
    let successor = g.regions[0].cards.iter().find(|c| c.definition == "JC054").unwrap();
    assert!(successor.face_down && successor.exhausted);
    assert_eq!(g.stack.len(), 1);
    assert!(g.board(&ally).is_some() && g.board(&enemy).is_some());
    top(&mut g);
    assert!(g.board(&ally).is_none() && g.board(&enemy).is_none());
    assert!(g.board(&hidden).is_some());
    restore_both(&g);
}

#[test]
fn jc054_end_follows_original_uid_when_source_moves_and_its_burst_uses_the_new_region() {
    let mut g = u::game();
    let worm = u::field(&mut g, "JC054", 0);
    let original = u::field(&mut g, "JC125", 1);
    let destination = u::field(&mut g, "JC125", 3);
    g.move_on_board_instance(&destination, 1);
    g.begin_window(Window::End); g.drive().unwrap();
    g.move_on_board_instance(&worm, 1);
    top(&mut g);
    assert_eq!(g.stack[0].frame.as_ref().unwrap().source.region, Some(1));
    top(&mut g);
    assert!(g.board(&original).is_some() && g.board(&destination).is_none());
    restore_both(&g);
}

#[test]
fn jc054_original_region_replacement_does_not_retarget_burst() {
    let mut g = u::game();
    let victim = u::field(&mut g, "JC125", 1);
    let worm = u::field(&mut g, "JC054", 0);
    g.hide_in_place(&worm); g.drive().unwrap();
    g.regions[0].card = g.make_card("DQJC116", 0);
    top(&mut g);
    assert!(g.board(&victim).is_some());
    restore_both(&g);
}

#[test]
fn jc054_conceal_reveal_primitive_combination_hits_the_new_face_up_worm_not_a_legal_blink_action() {
    let mut g = u::game();
    let worm = u::field(&mut g, "JC054", 0);
    let hidden = g.hide_in_place_successor(&worm).unwrap();
    // No admitted card has Blink. Exercise shared identity transitions directly.
    let (_, hidden) = g.remove_board(&hidden.id).unwrap();
    let mut revealed = g.fresh(hidden); revealed.face_down = false;
    let fresh = revealed.id.clone();g.regions[0].cards.push(revealed);
    g.drive().unwrap();top(&mut g);
    assert!(g.board(&fresh).is_none());
    assert!(g.players[0].graveyard.iter().any(|c| c.definition == "JC054"));
    restore_both(&g);
}

#[test]
fn jc054_hidden_end_and_later_reveal_do_not_reobserve_this_end_step() {
    let mut g = u::game();u::fund(&mut g, 0, "JC125", 4);
    let worm = u::field(&mut g, "JC054", 0);g.board_mut(&worm).unwrap().face_down = true;
    g.begin_window(Window::End);g.drive().unwrap();assert!(g.stack.is_empty());
    u::act(&mut g, 0, Action { card_id: Some(worm), ..Action::new("reveal") });top(&mut g);
    g.begin_window(Window::End);g.drive().unwrap();assert!(g.stack.is_empty());
    restore_both(&g);
}

#[test]
fn jc054_multiple_mandatory_triggers_choose_order_and_first_team_enters_stack_first() {
    let mut g = u::game();g.first_team = 1;
    let a = u::field(&mut g, "JC054", 0);let b = u::field(&mut g, "JC054", 0);
    let c = u::field(&mut g, "JC054", 2);let d = u::field(&mut g, "JC054", 2);
    g.begin_window(Window::End);g.drive().unwrap();
    let p = g.pending.as_ref().unwrap().clone();assert_eq!(p.seat, 2);
    assert_eq!(p.choice.min, Some(1));assert_eq!(p.choice.allow_decline, Some(false));
    u::reject(&mut g, 2, Action { choice_id: Some(p.choice.id), selected: Some(vec![]), ..Action::new("choose") });
    u::choose(&mut g, vec![d]);assert_eq!(g.pending.as_ref().unwrap().seat, 0);
    assert_eq!(g.stack.iter().map(|s| s.controller).collect::<Vec<_>>(), [2, 2]);
    u::choose(&mut g, vec![b]);
    assert_eq!(g.stack.iter().map(|s| s.controller).collect::<Vec<_>>(), [2, 2, 0, 0]);
    assert_eq!(g.stack[1].frame.as_ref().unwrap().source.card.id, c);
    assert_eq!(g.stack[3].frame.as_ref().unwrap().source.card.id, a);
    restore_both(&g);
}

#[test]
fn jc054_world_conceal_batch_keeps_mandatory_order_and_hidden_identity_reset() {
    let mut g = u::game();
    u::field(&mut g, "JC054", 0);u::field(&mut g, "JC054", 0);
    g.world_hide(None, true);g.drive().unwrap();
    assert!(matches!(g.pending.as_ref().unwrap().resolution, ChoiceResolution::JC054TriggerOrder { .. }));
    assert!(g.regions[0].cards.iter().all(|c| c.face_down));
    restore_both(&g);
}

#[test]
fn jc054_conceal_removes_attachments_markers_and_control_but_keeps_orientation_and_trigger_actor() {
    let mut g = u::game();let worm = u::field(&mut g, "JC054", 2);
    g.add_control(&worm, 0, ControlLifetime::TurnEnd { turn: g.turn }, SubtypeChange::None);
    let attachment = g.make_card("JC073", 0);
    g.attachments.push(Attachment { card: attachment, host_id: worm.clone() });
    let source = g.board_mut(&worm).unwrap();source.exhausted = true;source.lock_markers = 3;source.time_markers = 2;source.shield = 2;
    g.begin_window(Window::End);g.drive().unwrap();top(&mut g);
    let successor = g.regions[0].cards.iter().find(|c| c.definition == "JC054").unwrap();
    assert_eq!((successor.owner, successor.controller), (2, 2));assert!(successor.face_down && successor.exhausted);
    assert_eq!((successor.lock_markers, successor.time_markers, successor.shield), (0, 0, 0));
    assert!(g.attachments.is_empty() && g.control_baselines.is_empty() && g.control_effects.is_empty());
    assert!(g.players[0].graveyard.iter().any(|c| c.definition == "JC073"));
    assert_eq!(g.stack.last().unwrap().controller, 0);
    restore_both(&g);top(&mut g);
}

#[test]
fn jc054_end_does_not_follow_a_replacement_with_the_same_printed_name() {
    let mut g=u::game();let old=u::field(&mut g,"JC054",0);
    g.begin_window(Window::End);g.drive().unwrap();
    let (_, c)=g.leave_board(&old).unwrap();let fresh=g.fresh(c);let id=fresh.id.clone();g.regions[0].cards.push(fresh);
    top(&mut g);assert!(!g.board(&id).unwrap().1.face_down);assert!(g.stack.is_empty());restore_both(&g);
}

#[test]
fn jc054_reconcealing_an_already_hidden_card_has_no_printed_trigger() {
    let mut g=u::game();let worm=u::field(&mut g,"JC054",0);
    g.board_mut(&worm).unwrap().face_down=true;g.hide_in_place(&worm);g.drive().unwrap();
    assert!(g.stack.is_empty() && g.effects.is_empty());restore_both(&g);
}

#[test]
fn jc054_burst_uses_shared_prevention_ignores_targeting_barriers_and_does_not_damage_other_regions() {
    let mut g=u::game();let worm=u::field(&mut g,"JC054",0);
    let protected=u::field(&mut g,"JC125",2);
    g.turn_attribute_modifiers.push(TurnAttributeModifier { target_instance:protected.clone(),defense_bonus:0,kill_bonus:0,
        grants_retreat:false,printed_defense_override:None,ordinary_icons:Icons::default(),grants_renown:false,prevents_damage:true,expires_turn:g.turn });
    let barrier=u::field(&mut g,"JC125",2);let net=g.make_card("JC073",2);
    g.attachments.push(Attachment { card:net,host_id:barrier.clone() });
    assert!(!g.targetable(0,g.board(&barrier).unwrap().1));
    let remote=u::field(&mut g,"JC125",3);g.move_on_board_instance(&remote,1);
    g.hide_in_place(&worm);g.drive().unwrap();top(&mut g);
    assert_eq!(g.board(&protected).unwrap().1.damage,0);assert!(g.board(&barrier).is_none());
    assert_eq!(g.board(&remote).unwrap().1.damage,0);restore_both(&g);
}

#[test]
fn jc054_end_is_independent_of_later_control_changes_but_conceal_observes_the_current_controller() {
    let mut g=u::game();let worm=u::field(&mut g,"JC054",0);
    g.begin_window(Window::End);g.drive().unwrap();assert_eq!(g.stack[0].controller,0);
    g.add_control(&worm,2,ControlLifetime::TurnEnd { turn:g.turn },SubtypeChange::None);
    top(&mut g);assert_eq!(g.stack.last().unwrap().controller,2);
    let hidden=g.regions[0].cards.iter().find(|c|c.definition=="JC054").unwrap();assert_eq!(hidden.controller,0);
    restore_both(&g);top(&mut g);
}

#[test]
fn jc054_granted_renown_keeps_its_frozen_public_source_after_the_grant_expires() {
    let mut g=u::game();let worm=u::field(&mut g,"JC054",0);
    g.turn_attribute_modifiers.push(TurnAttributeModifier { target_instance:worm,defense_bonus:0,kill_bonus:0,
        grants_retreat:false,printed_defense_override:None,ordinary_icons:Icons::default(),grants_renown:true,prevents_damage:false,expires_turn:g.turn });
    g.begin_window(Window::After(0,2));let region=g.regions[0].card.id.clone();g.declare_region_renown(0,&region);
    g.turn_attribute_modifiers.clear();g.drive().unwrap();
    let option=g.pending.as_ref().unwrap().choice.options[0].id.clone();u::choose(&mut g,vec![option]);
    top(&mut g);assert_eq!(g.regions[0].influence[0],1);restore_both(&g);
}

#[test]
fn jc054_observes_the_next_end_after_real_cleanup_and_turn_advance() {
    let (mut g,_)=burst();g.begin_window(Window::End);top(&mut g);
    let turn=g.turn;
    for _ in 0..16 {if g.turn>turn {break;}u::pass(&mut g);}
    assert_eq!(g.turn,turn+1);assert_eq!(g.jc054_end_step_turn,None);
    u::fund(&mut g,0,"JC125",4);
    for _ in 0..4 {if g.can_fast(0) {break;}u::pass(&mut g);}
    let hidden=g.regions[0].cards.iter().find(|c|c.definition=="JC054").unwrap().id.clone();
    u::act(&mut g,0,Action {card_id:Some(hidden),..Action::new("reveal")});top(&mut g);
    g.begin_window(Window::End);g.drive().unwrap();assert_eq!(g.stack.last().unwrap().frame.as_ref().unwrap().ability_key,jc054::END);
    restore_both(&g);
}

fn invalid_case(name: String, g: &Game, bad: &Value) -> Value {
    let game_state=bad.to_string();
    let game_error=Game::from_persisted(&game_state).err().expect("corrupt Game rejected");
    let mut room = serde_json::to_value(crate::jc029_tests::envelope(g)).unwrap();
    room["game"] = bad.clone();
    let state=room.to_string();let room_error=Room::from_persisted(&state).err().expect("corrupt Room rejected");
    json!({"name":name,"state":state,"gameState":game_state,"gameError":game_error,"roomError":room_error})
}
fn corrupt_frames(g: &Game) -> Vec<Value> {
    let base = serde_json::to_value(g).unwrap();let mut cases=vec![];
    let edits = [
        ("/stack/0/frame/actor", json!(1)),
        ("/stack/0/frame/ability_key", json!(jc054::END)),
        ("/stack/0/frame/source/card/face_down", json!(true)),
        ("/stack/0/frame/source/card/definition", json!("JC125")),
        ("/stack/0/frame/source/source_region_instance", json!("")),
        ("/stack/0/frame/steps/0/context", json!(1)),
        ("/stack/0/frame/cursor", json!(1)),
        ("/stack/0/frame/guard", json!("Accepted")),
        ("/stack/0/frame/steps/0/op/JC054ConcealBurst/observation/turn", json!(99)),
        ("/stack/0/frame/steps/0/op/JC054ConcealBurst/observation/successor/face_down", json!(false)),
        ("/stack/0/frame/steps/0/op/JC054ConcealBurst/observation/successor/exhausted", json!(true)),
        ("/stack/0/frame/steps/0/op/JC054ConcealBurst/observation/successor/damage", json!(1)),
    ];
    for (index,(path,value)) in edits.into_iter().enumerate() {let mut bad=base.clone();*bad.pointer_mut(path).unwrap()=value;cases.push(invalid_case(format!("frame-{index:02}"),g,&bad));}
    let mut duplicate = base.clone();let item = duplicate["stack"][0].clone();duplicate["stack"].as_array_mut().unwrap().push(item);cases.push(invalid_case("frame-duplicate".into(),g,&duplicate));
    let mut optional = base.clone();optional["pending"] = json!({"seat":0,"choice":{"id":"forged","kind":"ability","title":"可选","description":"","playerId":"p0","options":[],"min":0,"max":1},"resolution":{"Declare":{"declaration":{"actor":0,"source":base["stack"][0]["frame"]["source"],"ability":jc054::definition().abilities[0]},"stage":"Accept"}}});cases.push(invalid_case("optional-declare".into(),g,&optional));
    cases
}

fn corrupt_order() -> (Game, Vec<Value>) {
    let mut g = u::game();u::field(&mut g, "JC054", 0);u::field(&mut g, "JC054", 0);
    g.begin_window(Window::End);g.drive().unwrap();let base = serde_json::to_value(&g).unwrap();let mut cases=vec![];
    for (index,(path,value)) in [("/jc054_end_step_turn", json!(null)),("/pending/choice/min",json!(0)),("/pending/choice/allowDecline",json!(true)),("/pending/seat",json!(1))].into_iter().enumerate() {
        let mut bad=base.clone();*bad.pointer_mut(path).unwrap()=value;cases.push(invalid_case(format!("order-{index:02}"),&g,&bad));
    }
    restore_both(&g);(g,cases)
}

#[test]
fn jc054_corrupt_historical_frames_fail_both_independent_restore_paths() {
    let (g,_)=burst();assert_eq!(corrupt_frames(&g).len(),14);
}
#[test]
fn jc054_end_marker_and_mandatory_order_cannot_be_removed_or_made_optional() {
    assert_eq!(corrupt_order().1.len(),4);
}

fn corrupt_grants() -> Vec<Value> {
    let mut g=u::game();let worm=u::field(&mut g,"JC054",0);
    let d=Declaration {actor:0,source:g.source_snapshot(g.board(&worm).unwrap().1,Some(0)),ability:crate::renown::renown_ability(&g.regions[0].card.id)};
    let mut base=serde_json::to_value(&g).unwrap();base["effects"]=json!([{"Declare":{"declaration":d}}]);
    [("grant-hidden","/effects/0/Declare/declaration/source/card/face_down",json!(true)),
        ("grant-play-source","/effects/0/Declare/declaration/source/play_source",json!("Hand"))]
        .into_iter().map(|(name,path,value)| {let mut bad=base.clone();
            if let Some(slot)=bad.pointer_mut(path) {*slot=value;} else {bad["effects"][0]["Declare"]["declaration"]["source"]["play_source"]=value;}
            invalid_case(name.into(),&g,&bad)}).collect()
}
#[test]
fn jc054_granted_reward_rejects_hidden_or_play_transaction_snapshots() {
    assert_eq!(corrupt_grants().len(),2);
}

fn corrupt_instances() -> Vec<Value> {
    let mut g=u::game();u::field(&mut g,"JC054",0);let base=serde_json::to_value(&g).unwrap();
    let mut cases=vec![];
    for (name,field,value) in [("instance-owner","owner",json!(99)),("instance-controller","controller",json!(99)),("instance-empty","id",json!(""))] {
        let mut bad=base.clone();bad["regions"][0]["cards"][0][field]=value;cases.push(invalid_case(name.into(),&g,&bad));
    }
    for (name,definition) in [("instance-duplicate","JC054"),("instance-collision","JC125")] {
        let mut bad=base.clone();let mut copy=bad["regions"][0]["cards"][0].clone();copy["definition"]=json!(definition);
        bad["regions"][0]["cards"].as_array_mut().unwrap().push(copy);cases.push(invalid_case(name.into(),&g,&bad));
    }
    cases
}
#[test]
fn jc054_untriggered_entity_seats_and_ids_are_checked_before_end_step_observation() {
    assert_eq!(corrupt_instances().len(),5);
}

#[test]
fn jc054_native81_room_oracles() {
    let mut scenarios=vec![];
    let mut g=u::game();u::fund(&mut g,0,"JC125",4);let held=u::held(&mut g,"JC054",0);
    let mut t=Trace::new("paid-deploy",g);t.action(0,Action { card_id:Some(held),region:Some(0),..Action::new("deploy") });t.finish();scenarios.push(t.export());

    let mut g=u::game();u::field(&mut g,"JC054",0);let mut t=Trace::new("natural-end",g);
    for _ in 0..256 {
        if t.room.game.jc054_end_step_turn==Some(t.room.game.turn) { break; }
        if t.room.game.pending.is_some() { t.choose_first(); } else { t.pass(); }
    }
    assert_eq!(t.room.game.window,Some(Window::End));
    assert_eq!(t.room.game.stack.last().unwrap().frame.as_ref().unwrap().ability_key,jc054::END);
    t.finish();assert!(t.room.game.regions.iter().flat_map(|r|&r.cards).any(|c|c.definition=="JC054"&&c.face_down));scenarios.push(t.export());

    let mut g=u::game();u::field(&mut g,"JC054",0);u::field(&mut g,"JC125",1);u::field(&mut g,"JC125",2);
    g.begin_window(Window::End);g.drive().unwrap();let mut t=Trace::new("mandatory-end",g);t.finish();scenarios.push(t.export());

    let mut g=u::game();g.first_team=1;
    for seat in [0,0,2,2] {u::field(&mut g,"JC054",seat);}
    g.begin_window(Window::End);g.drive().unwrap();let mut t=Trace::new("mandatory-order",g);
    let p=t.room.game.pending.as_ref().unwrap().clone();
    t.command(p.seat,SessionAction::Game { action:Action { choice_id:Some(p.choice.id),selected:Some(vec![]),..Action::new("choose") } },true);
    t.finish();scenarios.push(t.export());

    let mut g=u::game();let worm=u::field(&mut g,"JC054",2);
    g.add_control(&worm,0,ControlLifetime::TurnEnd { turn:g.turn },SubtypeChange::None);
    g.begin_window(Window::End);g.drive().unwrap();let mut t=Trace::new("controlled-conceal",g);t.finish();
    let successor=t.room.game.regions[0].cards.iter().find(|c|c.definition=="JC054").unwrap();assert_eq!(successor.controller,2);scenarios.push(t.export());

    let mut g=u::game();let old=u::field(&mut g,"JC054",0);g.begin_window(Window::End);g.drive().unwrap();
    let (_,c)=g.leave_board(&old).unwrap();let fresh=g.fresh(c);g.regions[0].cards.push(fresh);
    let mut t=Trace::new("source-replaced",g);t.finish();assert!(t.room.game.regions[0].cards.iter().all(|c|!c.face_down));scenarios.push(t.export());

    let (mut g,hidden)=burst();u::fund(&mut g,0,"JC125",4);let mut t=Trace::new("paid-response-reveal",g);
    t.action(0,Action { card_id:Some(hidden),..Action::new("reveal") });t.finish();
    assert!(t.room.game.players[0].graveyard.iter().any(|c|c.definition=="JC054"));scenarios.push(t.export());

    let (g,mut invalid)=corrupt_order();let mut t=Trace::new("pause-mandatory-order",g);
    let choice=t.room.game.pending.as_ref().unwrap().choice.id.clone();
    t.command(0,SessionAction::PauseRoom,false);
    t.command(0,SessionAction::Game { action:Action { choice_id:Some(choice.clone()),selected:Some(vec![]),..Action::new("choose") } },true);
    t.command(0,SessionAction::ResumeRoom,false);assert_eq!(t.room.game.pending.as_ref().unwrap().choice.id,choice);
    t.finish();scenarios.push(t.export());
    let (g,_)=burst();invalid.extend(corrupt_frames(&g));invalid.extend(corrupt_grants());invalid.extend(corrupt_instances());
    assert_eq!(scenarios.len(),SCENARIOS.len());assert_eq!(invalid.len(),25);
    if let Ok(directory)=std::env::var("JC054_NATIVE_DIR") {
        std::fs::create_dir_all(&directory).unwrap();
        let output=json!({"format":"jc054-native81-v1","scope":"Explicit Native layouts followed by real Room commands, exact restore and journal replay; not online play or migration",
            "sourceBindings":source_bindings(),"catalog":catalog::catalog(),"scenarios":scenarios,"invalid":invalid});
        use std::io::Write;
        std::fs::OpenOptions::new().write(true).create_new(true).open(std::path::Path::new(&directory).join("native81.json")).unwrap()
            .write_all(&serde_json::to_vec(&output).unwrap()).unwrap();
    }
}

fn source_bindings() -> Value {
    use sha2::{Digest, Sha256};
    let files: &[(&str, &[u8])] = &[
        ("Cargo.lock", include_bytes!("../../Cargo.lock")),
        ("Cargo.toml", include_bytes!("../../Cargo.toml")),
        ("rust-game-wasm/Cargo.toml", include_bytes!("../../rust-game-wasm/Cargo.toml")),
        ("rust-game-wasm/build.sh", include_bytes!("../../rust-game-wasm/build.sh")),
        ("rust-game-wasm/src/lib.rs", include_bytes!("../../rust-game-wasm/src/lib.rs")),
        ("rust-game/Cargo.toml", include_bytes!("../Cargo.toml")),
        ("rust-game/data/cards.json", include_bytes!("../data/cards.json")),
        ("rust-game/src/attachment.rs", include_bytes!("attachment.rs")),
        ("rust-game/src/attributes.rs", include_bytes!("attributes.rs")),
        ("rust-game/src/bin/hegemony-audit.rs", include_bytes!("bin/hegemony-audit.rs")),
        ("rust-game/src/black_expansion.rs", include_bytes!("black_expansion.rs")),
        ("rust-game/src/black_expansion_tests.rs", include_bytes!("black_expansion_tests.rs")),
        ("rust-game/src/blue_expansion.rs", include_bytes!("blue_expansion.rs")),
        ("rust-game/src/blue_expansion_tests.rs", include_bytes!("blue_expansion_tests.rs")),
        ("rust-game/src/blue_minimum.rs", include_bytes!("blue_minimum.rs")),
        ("rust-game/src/blue_minimum_tests.rs", include_bytes!("blue_minimum_tests.rs")),
        ("rust-game/src/bounded_entry_fresh_tests.rs", include_bytes!("bounded_entry_fresh_tests.rs")),
        ("rust-game/src/bounded_search.rs", include_bytes!("bounded_search.rs")),
        ("rust-game/src/bounded_search_tests.rs", include_bytes!("bounded_search_tests.rs")),
        ("rust-game/src/bq030.rs", include_bytes!("bq030.rs")),
        ("rust-game/src/bq030_preparation.rs", include_bytes!("bq030_preparation.rs")),
        ("rust-game/src/bq035.rs", include_bytes!("bq035.rs")),
        ("rust-game/src/bq035_tests.rs", include_bytes!("bq035_tests.rs")),
        ("rust-game/src/catalog.rs", include_bytes!("catalog.rs")),
        ("rust-game/src/control.rs", include_bytes!("control.rs")),
        ("rust-game/src/control_tests.rs", include_bytes!("control_tests.rs")),
        ("rust-game/src/death_observer_tests.rs", include_bytes!("death_observer_tests.rs")),
        ("rust-game/src/death_observers.rs", include_bytes!("death_observers.rs")),
        ("rust-game/src/deck.rs", include_bytes!("deck.rs")),
        ("rust-game/src/deck_seal_search.rs", include_bytes!("deck_seal_search.rs")),
        ("rust-game/src/deck_seal_search_draft.rs", include_bytes!("deck_seal_search_draft.rs")),
        ("rust-game/src/deck_seal_search_tests.rs", include_bytes!("deck_seal_search_tests.rs")),
        ("rust-game/src/defence_equipment_tests.rs", include_bytes!("defence_equipment_tests.rs")),
        ("rust-game/src/dream_reveal_tests.rs", include_bytes!("dream_reveal_tests.rs")),
        ("rust-game/src/engine.rs", include_bytes!("engine.rs")),
        ("rust-game/src/equipment_abilities_tests.rs", include_bytes!("equipment_abilities_tests.rs")),
        ("rust-game/src/gray_expansion.rs", include_bytes!("gray_expansion.rs")),
        ("rust-game/src/gray_expansion_tests.rs", include_bytes!("gray_expansion_tests.rs")),
        ("rust-game/src/gray_lock.rs", include_bytes!("gray_lock.rs")),
        ("rust-game/src/gray_lock_tests.rs", include_bytes!("gray_lock_tests.rs")),
        ("rust-game/src/green_minimum_tests.rs", include_bytes!("green_minimum_tests.rs")),
        ("rust-game/src/hand_deck_tests.rs", include_bytes!("hand_deck_tests.rs")),
        ("rust-game/src/hand_interactions_tests.rs", include_bytes!("hand_interactions_tests.rs")),
        ("rust-game/src/jc004_tests.rs", include_bytes!("jc004_tests.rs")),
        ("rust-game/src/jc005_tests.rs", include_bytes!("jc005_tests.rs")),
        ("rust-game/src/jc008_tests.rs", include_bytes!("jc008_tests.rs")),
        ("rust-game/src/jc029_tests.rs", include_bytes!("jc029_tests.rs")),
        ("rust-game/src/jc030_tests.rs", include_bytes!("jc030_tests.rs")),
        ("rust-game/src/jc037.rs", include_bytes!("jc037.rs")),
        ("rust-game/src/jc037_tests.rs", include_bytes!("jc037_tests.rs")),
        ("rust-game/src/jc046.rs", include_bytes!("jc046.rs")),
        ("rust-game/src/jc046_tests.rs", include_bytes!("jc046_tests.rs")),
        ("rust-game/src/jc050.rs", include_bytes!("jc050.rs")),
        ("rust-game/src/jc050_tests.rs", include_bytes!("jc050_tests.rs")),
        ("rust-game/src/jc052.rs", include_bytes!("jc052.rs")),
        ("rust-game/src/jc052_tests.rs", include_bytes!("jc052_tests.rs")),
        ("rust-game/src/jc054.rs", include_bytes!("jc054.rs")),
        ("rust-game/src/jc054_tests.rs", include_bytes!("jc054_tests.rs")),
        ("rust-game/src/jc060.rs", include_bytes!("jc060.rs")),
        ("rust-game/src/jc060_tests.rs", include_bytes!("jc060_tests.rs")),
        ("rust-game/src/jc061_tests.rs", include_bytes!("jc061_tests.rs")),
        ("rust-game/src/jc087_tests.rs", include_bytes!("jc087_tests.rs")),
        ("rust-game/src/jc089_tests.rs", include_bytes!("jc089_tests.rs")),
        ("rust-game/src/jc094_tests.rs", include_bytes!("jc094_tests.rs")),
        ("rust-game/src/jc095.rs", include_bytes!("jc095.rs")),
        ("rust-game/src/jc095_tests.rs", include_bytes!("jc095_tests.rs")),
        ("rust-game/src/jz22.rs", include_bytes!("jz22.rs")),
        ("rust-game/src/jz22_tests.rs", include_bytes!("jz22_tests.rs")),
        ("rust-game/src/jz31_tests.rs", include_bytes!("jz31_tests.rs")),
        ("rust-game/src/jz35.rs", include_bytes!("jz35.rs")),
        ("rust-game/src/jz35_tests.rs", include_bytes!("jz35_tests.rs")),
        ("rust-game/src/jz46.rs", include_bytes!("jz46.rs")),
        ("rust-game/src/jz46_tests.rs", include_bytes!("jz46_tests.rs")),
        ("rust-game/src/jz48_tests.rs", include_bytes!("jz48_tests.rs")),
        ("rust-game/src/jz49_tests.rs", include_bytes!("jz49_tests.rs")),
        ("rust-game/src/jz50_search.rs", include_bytes!("jz50_search.rs")),
        ("rust-game/src/jz50_search_tests.rs", include_bytes!("jz50_search_tests.rs")),
        ("rust-game/src/jz55_tests.rs", include_bytes!("jz55_tests.rs")),
        ("rust-game/src/lib.rs", include_bytes!("lib.rs")),
        ("rust-game/src/main.rs", include_bytes!("main.rs")),
        ("rust-game/src/mill_public_tests.rs", include_bytes!("mill_public_tests.rs")),
        ("rust-game/src/mobility.rs", include_bytes!("mobility.rs")),
        ("rust-game/src/model.rs", include_bytes!("model.rs")),
        ("rust-game/src/msjc01_tests.rs", include_bytes!("msjc01_tests.rs")),
        ("rust-game/src/msjc06_tests.rs", include_bytes!("msjc06_tests.rs")),
        ("rust-game/src/msjc07_tests.rs", include_bytes!("msjc07_tests.rs")),
        ("rust-game/src/msjc08_tests.rs", include_bytes!("msjc08_tests.rs")),
        ("rust-game/src/msjc11_tests.rs", include_bytes!("msjc11_tests.rs")),
        ("rust-game/src/play_sources.rs", include_bytes!("play_sources.rs")),
        ("rust-game/src/protection_tests.rs", include_bytes!("protection_tests.rs")),
        ("rust-game/src/purple_tools_tests.rs", include_bytes!("purple_tools_tests.rs")),
        ("rust-game/src/red_time.rs", include_bytes!("red_time.rs")),
        ("rust-game/src/red_time_tests.rs", include_bytes!("red_time_tests.rs")),
        ("rust-game/src/region_aura_tests.rs", include_bytes!("region_aura_tests.rs")),
        ("rust-game/src/renown.rs", include_bytes!("renown.rs")),
        ("rust-game/src/renown_tests.rs", include_bytes!("renown_tests.rs")),
        ("rust-game/src/repress_assets_tests.rs", include_bytes!("repress_assets_tests.rs")),
        ("rust-game/src/resolution.rs", include_bytes!("resolution.rs")),
        ("rust-game/src/room.rs", include_bytes!("room.rs")),
        ("rust-game/src/rules.rs", include_bytes!("rules.rs")),
        ("rust-game/src/sealing.rs", include_bytes!("sealing.rs")),
        ("rust-game/src/sealing_tests.rs", include_bytes!("sealing_tests.rs")),
        ("rust-game/src/service.rs", include_bytes!("service.rs")),
        ("rust-game/src/society.rs", include_bytes!("society.rs")),
        ("rust-game/src/society_msjc09_tests.rs", include_bytes!("society_msjc09_tests.rs")),
        ("rust-game/src/unit_support.rs", include_bytes!("unit_support.rs")),
        ("rust-game/src/white_wound_tests.rs", include_bytes!("white_wound_tests.rs")),
        ("rust-game/src/win_flow_tests.rs", include_bytes!("win_flow_tests.rs")),
        ("rust-game/src/wm023.rs", include_bytes!("wm023.rs")),
        ("rust-game/src/wm023_tests.rs", include_bytes!("wm023_tests.rs")),
        ("rust-game/src/wm038.rs", include_bytes!("wm038.rs")),
        ("rust-game/src/wm038_tests.rs", include_bytes!("wm038_tests.rs")),
        ("rust-game/src/wm056.rs", include_bytes!("wm056.rs")),
        ("rust-game/src/wm056_tests.rs", include_bytes!("wm056_tests.rs")),
        ("rust-game/src/wm062.rs", include_bytes!("wm062.rs")),
        ("rust-game/src/wm062_tests.rs", include_bytes!("wm062_tests.rs")),
        ("rust-game/src/world.rs", include_bytes!("world.rs")),
        ("rust-game/src/wound_defence_tests.rs", include_bytes!("wound_defence_tests.rs")),
        ("rust-game/src/xq18_tests.rs", include_bytes!("xq18_tests.rs")),
        ("rust-game/src/xq24_tests.rs", include_bytes!("xq24_tests.rs")),
        ("rust-game/src/xq25.rs", include_bytes!("xq25.rs")),
        ("rust-game/src/xq25_tests.rs", include_bytes!("xq25_tests.rs")),
        ("rust-game/src/xq27.rs", include_bytes!("xq27.rs")),
        ("rust-game/src/xq27_tests.rs", include_bytes!("xq27_tests.rs")),
        ("rust-game/src/xq37.rs", include_bytes!("xq37.rs")),
        ("rust-game/src/xq37_tests.rs", include_bytes!("xq37_tests.rs")),
        ("rust-game/src/yellow_search_batch_tests.rs", include_bytes!("yellow_search_batch_tests.rs")),
    ];
    json!({"algorithm":"sha256","files":files.iter().map(|(path,bytes)| ((*path).to_owned(),format!("{:x}",Sha256::digest(bytes)))).collect::<std::collections::BTreeMap<_,_>>()})
}
