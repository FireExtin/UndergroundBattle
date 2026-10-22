//! Read-only probes of the exact A dependency. This is not a B runtime test:
//! no shared hook is replaced and no B program is registered. BeforeSelection
//! and After fixtures reuse receipts from a genuine paid WM022 contest, then
//! seed the documented public hook boundary to exercise ordinary Game commands.
use hegemony_server::{model::*, room::RoomEnvelope, rules};
use serde_json::{json, Value};

fn checkpoint(g: &Game) {
    let raw=serde_json::to_string(g).unwrap();
    assert_eq!(serde_json::to_string(&Game::from_persisted(&raw).unwrap()).unwrap(),raw);
    // A Game checkpoint needs an explicitly matched Room pacing fixture; the
    // public from_game constructor does not invent an active response window.
    let mut r=RoomEnvelope::from_game(g.clone());
    if let Some(top)=g.stack.last().filter(|_|g.status=="playing"&&g.pending.is_none()) {
        use hegemony_server::room::{PriorityWindow,Decision,INTENT_DURATION_MS};
        r.pacing.window_seq=1;
        r.pacing.window=Some(PriorityWindow{id:"response:1".into(),stack_top_id:top.id.clone(),holder_team:g.priority_team,
            members:g.players.iter().filter(|p|!p.eliminated&&p.seat/2==g.priority_team).map(|p|(p.seat,
                if g.passed.contains(&p.seat){Decision::Passed}else{Decision::Undecided{deadline_ms:INTENT_DURATION_MS}})).collect()});
    }
    let restored=RoomEnvelope::from_persisted(&serde_json::to_string(&r).unwrap()).unwrap();
    for seat in 0..4 {assert_eq!(serde_json::to_value(r.view(seat,0)).unwrap(),serde_json::to_value(restored.view(seat,0)).unwrap());}
}
fn apply(g: &mut Game, seat: usize, action: Action) {
    g.apply(seat,action).unwrap();checkpoint(g);
}
fn pass(g: &mut Game) -> Result<(),String> {
    let seat=(0..4).find(|s|g.legal_actions(*s).iter().any(|a|a.action.kind=="pass")).expect("pass candidate");
    g.apply(seat,Action::new("pass"))
}
fn choose(g: &mut Game, ids: Vec<String>) {
    let p=g.pending.as_ref().unwrap();let seat=p.seat;
    let action=Action{choice_id:Some(p.choice.id.clone()),selected:Some(ids),..Action::new("choose")};apply(g,seat,action);
}
fn held(g: &mut Game, definition: &str, seat: usize) -> String {
    let c=g.make_card(definition,seat);let id=c.id.clone();g.players[seat].hand.push(c);id
}
fn layout() -> Game {
    let mut g=Game::new("package-b-contract-probe".into(),"LOCAL".into(),"teams".into(),"P0".into(),"watchers".into(),9).unwrap();
    for seat in 1..4 {g.join(format!("P{seat}"),"watchers".into()).unwrap();}
    for p in &mut g.players {p.ready=true;}
    g.apply(0,Action::new("start")).unwrap();while g.pending.is_some(){choose(&mut g,vec![]);}
    for r in &mut g.regions {r.cards.clear();}
    for p in &mut g.players {p.hand.clear();p.deck.clear();p.assets.clear();p.graveyard.clear();}
    for seat in 0..4 {for _ in 0..8 {let c=g.make_card("JC125",seat);g.players[seat].deck.push(c);}}
    for _ in 0..10 {let c=g.make_card("WM022",0);g.players[0].assets.push(c);}
    g.effects.clear();g.stack.clear();g.pending=None;g.first_team=0;g.active_team=0;g.priority_team=0;
    g.passed.clear();g.team_passed=[false;2];g.window=Some(Window::Action(0));checkpoint(&g);g
}
fn start(g: &mut Game) {
    let mut c=g.make_card("WM022",0);c.face_down=true;let id=c.id.clone();g.regions[0].cards.push(c);
    apply(g,0,Action{card_id:Some(id),..Action::new("reveal")});
    for _ in 0..20 {if g.pending.is_some(){break;}pass(g).unwrap();checkpoint(g);}
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,"mystic-commit");
}
fn before_selection() -> Game {
    let mut g=layout();start(&mut g);
    // Undo only the top-to-hand move in the fixture; no submitted hand exists.
    for p in &mut g.players {let c=p.hand.pop().unwrap();p.deck.insert(0,c);}
    let mut raw=serde_json::to_value(&g).unwrap();let id=raw["mystic_contest"]["contest_id"].clone();
    raw["mystic_contest"]["phase"]=json!("BeforeSelection");raw["pending"]=Value::Null;
    raw["effects"]=json!([{"MysticContinue":{"contest_id":id}}]);
    let g=Game::from_persisted(&raw.to_string()).unwrap();checkpoint(&g);g
}

#[test]
fn package_b_probe_start_waits_for_an_ordinary_response_frame() {
    let mut g=before_selection();let card=held(&mut g,"WM061",0);
    let black=hegemony_server::catalog::catalog().cards.iter().find(|c|c.supported&&c.color=="黑"&&c.kind=="character"&&!c.keywords.iter().any(|k|k=="非资产")).unwrap().id.clone();
    let asset=g.make_card(&black,0);g.players[0].assets.push(asset);
    apply(&mut g,0,Action{card_id:Some(card),..Action::new("play")});
    let value=serde_json::to_value(&g).unwrap();assert_eq!(value["mystic_contest"]["phase"],"BeforeSelection");
    assert!(g.pending.is_none());assert_eq!(g.players[0].hand.len(),0);
    for _ in 0..20 {if g.pending.is_some(){break;}pass(&mut g).unwrap();checkpoint(&g);}
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,"mystic-commit");
    assert_eq!(g.players[0].deck.len(),2);assert_eq!(g.players[0].hand.len(),1);
}

#[test]
fn package_b_probe_response_generated_start_trigger_is_left_behind_selection() {
    let mut g=before_selection();let mut c=g.make_card("WM022",0);c.face_down=true;let id=c.id.clone();g.regions[0].cards.push(c);
    apply(&mut g,0,Action{card_id:Some(id),..Action::new("reveal")});
    for _ in 0..20 {if g.pending.is_some(){break;}pass(&mut g).unwrap();checkpoint(&g);}
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,"mystic-commit");
    assert!(g.effects.iter().any(|e|matches!(e,Effect::Declare{declaration} if declaration.ability.key=="pack-a-wm022-reveal")),
        "Probe records the current gap: the response-generated forced trigger still waits behind commitment");
}

#[test]
fn package_b_probe_nested_start_is_rejected_without_consuming_the_waiting_frame() {
    let mut g=before_selection();
    let source=g.mystic_contest.as_ref().unwrap().frame.source.clone();
    let ability=rules::definition("WM022").abilities[0].clone();
    g.effects.push_front(Effect::Declare{declaration:Declaration{actor:0,source,ability}});
    checkpoint(&g);pass(&mut g).unwrap();checkpoint(&g);
    assert!(g.stack.iter().any(|s|s.frame.as_ref().is_some_and(|f|f.ability_key=="pack-a-wm022-reveal")));
    for _ in 0..20 {
        let before=serde_json::to_string(&g).unwrap();
        if let Err(error)=pass(&mut g) {
            assert!(error.contains("隐秘对抗必须来自"),"{error}");assert_eq!(serde_json::to_string(&g).unwrap(),before);checkpoint(&g);return;
        }
    }
    panic!("Expected the explicitly unsupported nested contest boundary");
}

#[test]
fn package_b_probe_after_elimination_finishes_game_but_leaves_contest_validation_unsatisfied() {
    let mut g=layout();held(&mut g,"WM021",0);start(&mut g);
    for seat in 0..4 {let id=g.players[seat].hand.iter().find(|c|c.definition==if seat==0{"WM021"}else{"JC125"}).unwrap().id.clone();choose(&mut g,vec![id]);}
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,"mystic-reward");
    let mut raw=serde_json::to_value(&g).unwrap();let outcome=raw["mystic_history"][0].clone();let id=outcome["contest_id"].clone();
    raw["mystic_contest"]["phase"]=json!({"After":{"outcome":outcome,"next_shown":1,"post_frame":null}});raw["pending"]=Value::Null;
    // This ordinary Draw is a boundary probe for an observer caused by a B
    // post-effect. It does not purport to be B's missing blood operation.
    raw["effects"]=json!([{"Draw":{"seat":2,"count":1}},{"MysticContinue":{"contest_id":id}}]);
    let mut g=Game::from_persisted(&raw.to_string()).unwrap();g.players[2].deck.clear();g.players[3].eliminated=true;
    g.players[3].deck.clear();g.players[3].hand.clear();checkpoint(&g);
    let before=serde_json::to_string(&g).unwrap();let error=pass(&mut g).unwrap_err();
    assert!(error.contains("隐秘展示后置"),"{error}");assert_eq!(serde_json::to_string(&g).unwrap(),before);
    assert_eq!(g.status,"playing");assert!(!g.players[2].eliminated);checkpoint(&g);
}

#[test]
fn package_b_probe_empty_hand_and_deck_boundary_is_rejected_without_inventing_zero() {
    let mut g=layout();g.players[2].deck.clear();let mut c=g.make_card("WM022",0);c.face_down=true;let id=c.id.clone();g.regions[0].cards.push(c);
    apply(&mut g,0,Action{card_id:Some(id),..Action::new("reveal")});
    for _ in 0..20 {
        let before=serde_json::to_string(&g).unwrap();
        if let Err(error)=pass(&mut g) {assert!(error.contains("无手牌且牌库为空"),"{error}");assert_eq!(serde_json::to_string(&g).unwrap(),before);checkpoint(&g);return;}
    }
    panic!("Expected the unresolved empty-hand/empty-deck boundary");
}
