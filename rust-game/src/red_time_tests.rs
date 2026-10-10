//! Explicit native layouts with real payment, declaration, response and Room
//! commands. These tests are developer regressions, not independent UI playtests.
use crate::jc029_tests::{apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject};
use crate::{catalog, model::*, room::*, rules::*};
use std::sync::atomic::{AtomicUsize, Ordering};

static EVIDENCE: AtomicUsize = AtomicUsize::new(0);

fn initial(actor: usize) -> Game {
    let mut g = game(actor);
    for r in &mut g.regions { r.influence = [0; 2]; }
    g
}
fn held(g: &mut Game, def: &str, seat: usize) -> String {
    let c = g.make_card(def, seat); let id = c.id.clone(); g.players[seat].hand.push(c); id
}
fn attached(g: &mut Game, def: &str, seat: usize, host: &str) -> String {
    let c = g.make_card(def, seat); let id = c.id.clone();
    g.attachments.push(Attachment { card: c, host_id: host.into() }); id
}
fn fixture(label: &str, g: &Game) {
    checkpoint(g);
    if let Ok(dir) = std::env::var("RED_TIME_EVIDENCE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let r = envelope(g); let n = EVIDENCE.fetch_add(1, Ordering::Relaxed);
        std::fs::write(format!("{dir}/{n:04}-{label}.json"), serde_json::to_vec(&serde_json::json!({
            "state": serde_json::to_string(&r).unwrap(),
            "views": (0..4).map(|s| r.view(s, 0)).collect::<Vec<_>>()
        })).unwrap()).unwrap();
    }
}
fn invalid(g: &Game, v: serde_json::Value) {
    assert!(Game::from_persisted(&serde_json::to_string(&v).unwrap()).is_err());
    let mut r = serde_json::to_value(envelope(g)).unwrap(); r["game"] = v;
    let state = serde_json::to_string(&r).unwrap();
    assert!(RoomEnvelope::from_persisted(&state).is_err());
    if let Ok(dir) = std::env::var("RED_TIME_INVALID_DIR") {
        std::fs::create_dir_all(&dir).unwrap(); let n = EVIDENCE.fetch_add(1, Ordering::Relaxed);
        std::fs::write(format!("{dir}/invalid-{n:04}.json"), serde_json::to_vec(&serde_json::json!({"state": state})).unwrap()).unwrap();
    }
}
fn deploy_jz30(g: &mut Game, actor: usize, accept: bool) -> String {
    fund(g, actor, "XQ18", 1); let h = held(g, "JZ30", actor);
    assert_eq!(g.players[actor].hand.iter().find(|c| c.id == h).unwrap().time_markers, 0);
    apply(g, actor, Action { card_id: Some(h), region: Some(2), ..Action::new("deploy") });
    pass_top(g);
    let id = g.regions[2].cards.iter().find(|c| c.definition == "JZ30" && c.controller == actor).unwrap().id.clone();
    assert_eq!(g.board(&id).unwrap().1.time_markers, 0);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, "trigger");
    fixture("jz30-entry-choice", g);
    choose(g, if accept { vec!["accept".into()] } else { vec![] });
    if accept { assert_eq!(g.board(&id).unwrap().1.time_markers, 0); pass_top(g); }
    assert_eq!(g.board(&id).unwrap().1.time_markers, u32::from(accept));
    id
}
fn reveal_xq18(g: &mut Game, actor: usize) -> String {
    // Prepared hidden card layout; actual paid conceal has its own full case.
    let old = board(g, "XQ18", actor, 2); g.board_mut(&old).unwrap().face_down = true;
    fund(g, actor, "XQ18", 2);
    apply(g, actor, Action { card_id: Some(old.clone()), ..Action::new("reveal") });
    assert!(g.pending.is_none()); pass_top(g);
    let ChoiceResolution::Declare { declaration: d, .. } = &g.pending.as_ref().unwrap().resolution else { panic!("XQ18 declaration expected") };
    let id = d.source.card.id.clone();
    assert_ne!(old, id); fixture("xq18-mode-choice", g); id
}
fn target_time(g: &mut Game, actor: usize, target: &str, add: bool) {
    reveal_xq18(g, actor);
    choose(g, vec![if add { "add-time".into() } else { "remove-time".into() }]);
    fixture("xq18-target-choice", g); choose(g, vec![target.into()]);
    fixture("xq18-target-stack", g); pass_top(g);
}
fn cast(g: &mut Game, def: &str, actor: usize, target: &str) {
    give_priority(g, actor);
    let h = held(g, def, actor);
    let a = g.legal_actions(actor).into_iter().find(|a| a.action.kind == "play"
        && a.action.card_id.as_deref() == Some(&h) && a.action.target_id.as_deref() == Some(target)).unwrap().action;
    apply(g, actor, a); pass_top(g);
}
fn give_priority(g: &mut Game, actor: usize) {
    // Trigger declaration preserves the current team priority. Opponents must
    // receive it through real passes before BeginResponse/SubmitResponse.
    for _ in 0..4 {
        if g.priority_team == g.team(actor) { return; }
        assert!(!g.stack.is_empty() && g.pending.is_none());
        let seat = (0..4).find(|&s| g.legal_actions(s).iter().any(|a| a.action.kind == "pass")).unwrap();
        apply(g, seat, Action::new("pass"));
    }
    panic!("response team did not receive priority");
}
fn forecast(g: &mut Game, top: Vec<String>, bottom: Vec<String>) {
    let p = g.pending.clone().unwrap();
    apply(g, p.seat, Action { choice_id: Some(p.choice.id), top: Some(top), bottom: Some(bottom), ..Action::new("choose") });
}

#[test]
fn red_time_original_fields_and_whole_printed_programs() {
    let x = catalog::card("XQ18");
    assert_eq!((&*x.name, x.cost, x.defense, &*x.magic), ("计时人", 2, Some(1), "死亡"));
    assert_eq!(x.loyalty, ["红色", "红色"]); assert_eq!(x.subtypes, ["人类", "法师", "邪教徒"]);
    assert_eq!(x.permanent_icons, Icons::default());
    assert_eq!(x.temporary_icons, Icons { investigation: 1, ..Icons::default() });
    assert!(x.keywords.is_empty() && !x.unique);
    let j = catalog::card("JZ30");
    assert_eq!((&*j.name, j.cost, j.defense, &*j.magic), ("末世论者", 1, Some(1), ""));
    assert_eq!(j.loyalty, ["红色"]); assert_eq!(j.subtypes, ["人类", "流浪者"]);
    assert_eq!(j.permanent_icons, Icons { influence: 1, ..Icons::default() });
    assert_eq!(j.temporary_icons, Icons::default()); assert!(!j.unique);
    assert_eq!(j.keywords, ["公开", "时间标志1"]);
    assert!(definition("JZ30").traits.public);
    assert_eq!(definition("XQ18").abilities[0].event, Some(Event::Reveal));
    assert_eq!(definition("JZ30").abilities.iter().map(|a| a.event).collect::<Vec<_>>(),
        [Some(Event::Enter), Some(Event::Death)]);
    for id in ["XQ18", "JZ30"] {
        let d = definition(id); crate::red_time::validate_definition(id, &d).unwrap();
        for a in &d.abilities { validate_ability(id, a).unwrap(); assert_eq!(a.response_policy, ResponsePolicy::Respondable);
            assert!(a.costs.is_empty() && !a.requires_ready_source && a.per_turn_limit.is_none()); }
    }
}

#[test]
fn red_time_jz30_real_public_payment_and_optional_respondable_entry() {
    for actor in 0..4 { for accept in [false, true] {
        let mut g = initial(actor); let h = held(&mut g, "JZ30", actor);
        fund(&mut g, actor, "JC125", 1);
        reject(&mut g, actor, Action { card_id: Some(h.clone()), region: Some(2), ..Action::new("deploy") });
        reject(&mut g, actor, Action { card_id: Some(h), region: Some(2), ..Action::new("conceal") });
        g.players[actor].hand.clear(); g.players[actor].assets.clear();
        let id = deploy_jz30(&mut g, actor, accept);
        assert_eq!(g.players[actor].assets.iter().filter(|c| c.exhausted).count(), 1);
        assert_eq!(g.effective_time_markers(g.board(&id).unwrap().1), u32::from(accept));
        fixture("jz30-entry-resolved", &g);
    }}
}

#[test]
fn red_time_xq18_real_deploy_is_silent_conceal_and_paid_reveal_only_trigger_once() {
    let mut g = initial(0); fund(&mut g, 0, "XQ18", 2); let h = held(&mut g, "XQ18", 0);
    apply(&mut g, 0, Action { card_id: Some(h), region: Some(2), ..Action::new("deploy") });
    pass_top(&mut g); assert!(g.pending.is_none() && g.stack.is_empty());
    let visible = g.regions[2].cards[0].id.clone();
    assert_eq!(g.current_icons(g.board(&visible).unwrap().1, 2).investigation, 1);
    g.first_team = 1; assert_eq!(g.current_icons(g.board(&visible).unwrap().1, 2).investigation, 0);
    g.first_team = 0;
    for c in &mut g.players[0].assets { c.exhausted = false; }
    let h = held(&mut g, "XQ18", 0);
    apply(&mut g, 0, Action { card_id: Some(h), region: Some(1), ..Action::new("conceal") });
    assert!(g.pending.is_none()); let hidden = g.regions[1].cards[0].id.clone();
    for c in &mut g.players[0].assets { c.exhausted = false; }
    apply(&mut g, 0, Action { card_id: Some(hidden.clone()), ..Action::new("reveal") });
    assert_eq!(g.players[0].assets.iter().filter(|c| c.exhausted).count(), 2);
    pass_top(&mut g); assert!(g.board(&hidden).is_none());
    assert_eq!(g.pending.as_ref().unwrap().choice.options.iter().map(|o| o.id.as_str()).collect::<Vec<_>>(), ["add-time", "remove-time"]);
    choose(&mut g, vec![]); assert!(g.pending.is_none());
    for c in &mut g.players[0].assets { c.exhausted = false; }
    let face_up = g.regions[1].cards[0].id.clone();
    reject(&mut g, 0, Action { card_id: Some(face_up), ..Action::new("reveal") });
}

#[test]
fn red_time_xq18_cost_and_double_red_loyalty_fail_atomically() {
    for asset_defs in [vec!["JC125", "JC125"], vec!["XQ18", "JC125"], vec!["XQ18"]] {
        let mut g = initial(0); for d in asset_defs { fund(&mut g, 0, d, 1); }
        let h = held(&mut g, "XQ18", 0);
        reject(&mut g, 0, Action { card_id: Some(h), region: Some(2), ..Action::new("deploy") });
        let hidden = board(&mut g, "XQ18", 0, 1); g.board_mut(&hidden).unwrap().face_down = true;
        reject(&mut g, 0, Action { card_id: Some(hidden), ..Action::new("reveal") });
    }
}

#[test]
fn red_time_xq18_both_modes_all_relations_regions_characters_and_region_attachments() {
    for actor in 0..4 { for owner in 0..4 { for add in [true, false] {
        let mut g = initial(actor); let target = board(&mut g, "JC125", owner, 0);
        g.board_mut(&target).unwrap().time_markers = u32::from(!add);
        target_time(&mut g, actor, &target, add);
        assert_eq!(g.board(&target).unwrap().1.time_markers, u32::from(add));
        assert_eq!(g.current_icons(g.board(&target).unwrap().1, 0), Icons::default());
        fixture("xq18-character-marker", &g);
    }}}
    for add in [true, false] {
        let mut g = initial(0); let host = g.regions[0].card.id.clone();
        let target = attached(&mut g, "JC090", 2, &host);
        g.board_mut(&target).unwrap().time_markers = u32::from(!add);
        target_time(&mut g, 0, &target, add);
        assert_eq!(g.board(&target).unwrap().1.time_markers, u32::from(add));
        fixture("xq18-attachment-marker", &g);
    }
}

#[test]
fn red_time_xq18_no_time_removal_can_resolve_and_target_choice_can_decline() {
    let mut g = initial(0); let target = board(&mut g, "JC125", 2, 0);
    target_time(&mut g, 0, &target, false); assert_eq!(g.board(&target).unwrap().1.time_markers, 0);
    reveal_xq18(&mut g, 0); choose(&mut g, vec!["add-time".into()]); choose(&mut g, vec![]);
    assert!(g.pending.is_none() && g.stack.is_empty()); assert_eq!(g.board(&target).unwrap().1.time_markers, 0);
}

#[test]
fn red_time_xq18_target_restrictions_hidden_assets_hand_grave_region_and_barrier() {
    let mut g = initial(0); let hidden = board(&mut g, "JC125", 1, 0); g.board_mut(&hidden).unwrap().face_down = true;
    let barrier = board(&mut g, "JZ08", 2, 0);
    let h = held(&mut g, "JC125", 2); fund(&mut g, 2, "JC125", 1); let asset = g.players[2].assets[0].id.clone();
    let c = g.make_card("JC125", 2); let grave = c.id.clone(); g.players[2].graveyard.push(c);
    let visible = board(&mut g, "JC125", 3, 0);
    reveal_xq18(&mut g, 0); choose(&mut g, vec!["add-time".into()]);
    let p = g.pending.clone().unwrap(); let ids = p.choice.options.iter().map(|o| o.id.clone()).collect::<Vec<_>>();
    assert!(ids.contains(&visible)); assert!(!ids.contains(&hidden) && !ids.contains(&barrier));
    for id in [hidden, barrier, h, asset, grave, "region:0".into(), "p0".into(), "unknown".into()] {
        reject(&mut g, 0, Action { choice_id: Some(p.choice.id.clone()), selected: Some(vec![id]), ..Action::new("choose") });
    }
    choose(&mut g, vec![visible]); pass_top(&mut g);
}

#[test]
fn red_time_xq18_real_rescue_response_invalidates_original_target_without_repayment() {
    let mut g = initial(0); let target = board(&mut g, "JC125", 2, 0);
    let rescue = board(&mut g, "JC075", 2, 1); fund(&mut g, 2, "JC075", 2);
    reveal_xq18(&mut g, 0); choose(&mut g, vec!["add-time".into()]); choose(&mut g, vec![target.clone()]);
    let paid = g.players[0].assets.iter().filter(|c| c.exhausted).count();
    give_priority(&mut g, 2);
    apply(&mut g, 2, Action { card_id: Some(rescue), target_id: Some(target.clone()), ..Action::new("activate") });
    pass_top(&mut g); assert!(g.board(&target).is_none());
    let fresh = board(&mut g, "JC125", 2, 0); pass_top(&mut g);
    assert_eq!(g.board(&fresh).unwrap().1.time_markers, 0);
    assert_eq!(g.players[0].assets.iter().filter(|c| c.exhausted).count(), paid);
    assert!(g.log.iter().any(|l| l.text.contains("效果取消"))); fixture("xq18-invalidated-target", &g);
}

#[test]
fn red_time_xq18_source_departure_retains_accepted_target_effect() {
    let mut g = initial(0); let target = board(&mut g, "JC125", 2, 0);
    let source = reveal_xq18(&mut g, 0); choose(&mut g, vec!["add-time".into()]); choose(&mut g, vec![target.clone()]);
    fund(&mut g, 2, "JC084", 3); cast(&mut g, "JC091", 2, &source);
    assert!(g.board(&source).is_none()); pass_top(&mut g);
    assert_eq!(g.board(&target).unwrap().1.time_markers, 1); fixture("xq18-source-departed", &g);
}

#[test]
fn red_time_jz30_entry_response_death_freezes_zero_and_cannot_mark_replacement() {
    let mut g = initial(0); fund(&mut g, 0, "XQ18", 1); let h = held(&mut g, "JZ30", 0);
    apply(&mut g, 0, Action { card_id: Some(h), region: Some(2), ..Action::new("deploy") }); pass_top(&mut g);
    let old = g.regions[2].cards[0].id.clone(); choose(&mut g, vec!["accept".into()]);
    fund(&mut g, 2, "JC084", 3); cast(&mut g, "JC091", 2, &old);
    let ChoiceResolution::Declare { declaration: d, .. } = &g.pending.as_ref().unwrap().resolution else { panic!() };
    assert_eq!(d.source.card.time_markers, 0); choose(&mut g, vec!["accept".into()]); pass_top(&mut g);
    assert!(g.pending.is_none()); let fresh = board(&mut g, "JZ30", 0, 2); pass_top(&mut g);
    assert_eq!(g.board(&fresh).unwrap().1.time_markers, 0);
    assert!(g.players[0].graveyard.iter().all(|c| c.time_markers == 0)); fixture("jz30-entry-source-departed", &g);
}

#[test]
fn red_time_jz30_real_lethal_death_freezes_time_private_forecast_and_restore_no_draw() {
    for actor in 0..4 {
        let mut g = initial(actor); let source = deploy_jz30(&mut g, actor, true);
        g.board_mut(&source).unwrap().time_markers = 3; // Explicit prepared accumulated-mark layout.
        let owner = (actor + 1) % 4; g.board_mut(&source).unwrap().owner = owner;
        fund(&mut g, actor, "JC104", 2); cast(&mut g, "JC102", actor, &source);
        assert!(g.board(&source).is_none()); assert_eq!(g.players[owner].graveyard[0].time_markers, 0);
        let ChoiceResolution::Declare { declaration: d, .. } = &g.pending.as_ref().unwrap().resolution else { panic!() };
        assert_eq!(d.actor, actor); assert_eq!(d.source.card.time_markers, 3); fixture("jz30-death-declaration", &g);
        choose(&mut g, vec!["accept".into()]); fixture("jz30-death-stack", &g);
        let fresh = board(&mut g, "JZ30", owner, 0); g.board_mut(&fresh).unwrap().time_markers = 7;
        pass_top(&mut g); let p = g.pending.clone().unwrap(); assert_eq!(p.choice.amount, Some(3));
        assert_eq!(p.seat, actor); assert_eq!(p.choice.options.len(), 3);
        for s in 0..4 { assert_eq!(g.view(s).pending_choice.is_some(), s == actor); }
        fixture("jz30-private-forecast", &g);
        let before_hand = g.players[actor].hand.len(); let before_deck = g.players[actor].deck.len();
        let ids = p.choice.options.iter().map(|o| o.id.clone()).collect::<Vec<_>>();
        let room = envelope(&g); g = RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).unwrap().game;
        forecast(&mut g, vec![ids[2].clone()], vec![ids[0].clone(), ids[1].clone()]);
        assert_eq!(g.players[actor].deck[0].id, ids[2]);
        assert_eq!(g.players[actor].deck[before_deck-2].id, ids[0]); assert_eq!(g.players[actor].deck[before_deck-1].id, ids[1]);
        assert_eq!(g.players[actor].deck.len(), before_deck); assert_eq!(g.players[actor].hand.len(), before_hand);
        assert!(g.pending.is_none() && g.stack.is_empty()); assert_eq!(g.board(&fresh).unwrap().1.time_markers, 7);
        fixture("jz30-forecast-resumed", &g);
    }
}

#[test]
fn red_time_jz30_zero_short_empty_deck_and_declined_death_are_finite() {
    for (marks, len, accept) in [(0, 3, true), (5, 2, true), (2, 0, true), (2, 3, false)] {
        let mut g = initial(0); let source = board(&mut g, "JZ30", 0, 2); g.board_mut(&source).unwrap().time_markers = marks;
        g.players[0].deck.truncate(len); let random = g.random; let hand = g.players[0].hand.len();
        g.remove_dead(&source, RemovalCause::Sacrifice); g.drive().unwrap(); checkpoint(&g);
        choose(&mut g, if accept { vec!["accept".into()] } else { vec![] });
        if accept { pass_top(&mut g); }
        if marks > 0 && len > 0 && accept {
            let p = g.pending.clone().unwrap(); assert_eq!(p.choice.options.len(), len);
            let ids = p.choice.options.into_iter().map(|o| o.id).collect::<Vec<_>>(); forecast(&mut g, vec![], ids);
        }
        assert!(g.pending.is_none() && g.stack.is_empty()); assert!(!g.players[0].eliminated);
        assert_eq!(g.players[0].deck.len(), len); assert_eq!(g.players[0].hand.len(), hand); assert_eq!(g.random, random);
        checkpoint(&g);
    }
}

#[test]
fn red_time_attachment_inheritance_transfer_and_death_snapshot_are_exact() {
    let mut g = initial(0); let priest = board(&mut g, "JC045", 0, 1); let j = deploy_jz30(&mut g, 0, true);
    let vest = attached(&mut g, "XQ47", 0, &priest);
    target_time(&mut g, 0, &vest, true);
    assert_eq!(g.board(&priest).unwrap().1.time_markers, 0);
    assert_eq!(g.effective_time_markers(g.board(&priest).unwrap().1), 1);
    assert_eq!(g.current_icons(g.board(&priest).unwrap().1, 1).influence, 1);
    fund(&mut g, 0, "XQ47", 2);
    apply(&mut g, 0, Action { card_id: Some(vest.clone()), target_id: Some(j.clone()), ability_id: Some("reattach".into()), ..Action::new("activate") });
    pass_top(&mut g); assert_eq!(g.board(&vest).unwrap().1.time_markers, 1);
    assert_eq!(g.effective_time_markers(g.board(&priest).unwrap().1), 0);
    assert_eq!(g.current_icons(g.board(&priest).unwrap().1, 1).influence, 0);
    assert_eq!(g.board(&j).unwrap().1.time_markers, 1); assert_eq!(g.effective_time_markers(g.board(&j).unwrap().1), 2);
    // Remove the unrelated observer without a death, then destroy through a paid spell.
    g.return_hand(&priest); fund(&mut g, 0, "JC084", 3); cast(&mut g, "JC091", 0, &j);
    assert!(g.board(&vest).is_none()); assert!(g.players[0].graveyard.iter().all(|c| c.time_markers == 0));
    let ChoiceResolution::Declare { declaration: d, .. } = &g.pending.as_ref().unwrap().resolution else { panic!() };
    assert_eq!(d.source.card.time_markers, 2); choose(&mut g, vec!["accept".into()]); pass_top(&mut g);
    let ids = g.pending.as_ref().unwrap().choice.options.iter().map(|o| o.id.clone()).collect::<Vec<_>>(); assert_eq!(ids.len(), 2);
    fixture("jz30-attachment-inherited-forecast", &g); forecast(&mut g, ids, vec![]);
}

#[test]
fn red_time_all_marked_characters_and_attachments_reset_on_leave_and_hide() {
    for mode in 0..3 {
        let mut g = initial(0); let c = board(&mut g, "JC125", 0, 0); let a = attached(&mut g, "XQ47", 0, &c);
        g.board_mut(&c).unwrap().time_markers = 2; g.board_mut(&a).unwrap().time_markers = 3;
        fixture("ordinary-character-time", &g);
        match mode { 0 => g.return_hand(&c), 1 => g.to_bottom(&c), _ => {
            fund(&mut g, 0, "JC056", 2); let h = held(&mut g, "JC063", 0);
            let action = g.legal_actions(0).into_iter().find(|x| x.action.kind == "play"
                && x.action.card_id.as_deref() == Some(&h) && x.action.target_id.as_deref() == Some(&c)
                && x.action.option.as_deref() == Some("hide")).unwrap().action;
            apply(&mut g, 0, action); pass_top(&mut g);
        }}
        assert!(g.players.iter().flat_map(|p| p.hand.iter().chain(&p.deck).chain(&p.graveyard)).all(|c| c.time_markers == 0));
        assert!(!g.attachments.iter().any(|x| x.card.id == a)); checkpoint(&g);
    }
}

#[test]
fn red_time_persisted_program_alias_mode_target_actor_and_cursor_tampering_rejected() {
    let mut g = initial(0); let c = board(&mut g, "JC125", 2, 0); reveal_xq18(&mut g, 0);
    let base = serde_json::to_value(&g).unwrap();
    for (key, value) in [("actor", serde_json::json!(2)), ("ability", serde_json::json!({}))] {
        let mut v = base.clone(); v["pending"]["resolution"]["Declare"]["declaration"][key] = value; invalid(&g, v);
    }
    choose(&mut g, vec!["add-time".into()]);
    let mut v = serde_json::to_value(&g).unwrap();
    v["pending"]["resolution"]["Declare"]["declaration"]["ability"]["targets"][0]["predicate"] = serde_json::Value::Null;
    invalid(&g, v); choose(&mut g, vec![c]);
    let base = serde_json::to_value(&g).unwrap();
    let i = g.stack.len()-1;
    for (key, value) in [("ability_key", serde_json::json!("forged")), ("actor", serde_json::json!(2)),
        ("cursor", serde_json::json!(1)), ("guard", serde_json::json!("Accepted"))] {
        let mut v = base.clone(); v["stack"][i]["frame"][key] = value; invalid(&g, v);
    }
    let mut v = base.clone(); v["stack"][i]["frame"]["steps"][0]["op"] = serde_json::json!("JZ30ForecastFrozenTime"); invalid(&g, v);
    let mut v = base; v["stack"][i]["frame"]["targets"][0]["spec"]["relation"] = serde_json::json!("FriendlyTeam"); invalid(&g, v);
}

#[test]
fn red_time_forecast_chooser_count_top_cards_guard_and_resume_tampering_rejected() {
    let mut g = initial(0); let c = board(&mut g, "JZ30", 0, 2); g.board_mut(&c).unwrap().time_markers = 3;
    g.remove_dead(&c, RemovalCause::Destroy); g.drive().unwrap(); choose(&mut g, vec!["accept".into()]); pass_top(&mut g);
    fixture("jz30-forecast-tamper-base", &g); let base = serde_json::to_value(&g).unwrap();
    for key in ["cursor", "actor"] {
        let mut v = base.clone(); v["pending"]["resolution"]["Frame"]["frame"][key] = serde_json::json!(0); if key == "actor" { v["pending"]["resolution"]["Frame"]["frame"][key] = serde_json::json!(2); }
        invalid(&g, v);
    }
    let mut v = base.clone(); v["pending"]["resolution"]["Frame"]["frame"]["guard"] = serde_json::json!("Unchecked"); invalid(&g, v);
    let mut v = base.clone(); v["pending"]["resolution"]["Frame"]["choice"]["Forecast"]["seat"] = serde_json::json!(2); invalid(&g, v);
    let mut v = base.clone(); v["pending"]["choice"]["amount"] = serde_json::json!(2); invalid(&g, v);
    let mut v = base.clone(); v["pending"]["choice"]["options"][0]["id"] = serde_json::json!("forged"); invalid(&g, v);
    let mut v = base; v["pending"]["resolution"]["Frame"]["frame"]["source"]["card"]["time_markers"] = serde_json::json!(2); invalid(&g, v);
    let p = g.pending.clone().unwrap(); let ids = p.choice.options.iter().map(|o| o.id.clone()).collect::<Vec<_>>();
    reject(&mut g, 2, Action { choice_id: Some(p.choice.id.clone()), top: Some(ids.clone()), bottom: Some(vec![]), ..Action::new("choose") });
    reject(&mut g, 0, Action { choice_id: Some(p.choice.id), top: Some(vec![ids[0].clone(), ids[0].clone()]), bottom: Some(vec![ids[1].clone()]), ..Action::new("choose") });
    forecast(&mut g, ids, vec![]);
}

#[test]
fn red_time_persisted_markers_accept_current_in_play_and_reject_non_play_and_hidden() {
    let mut g = initial(0); let c = board(&mut g, "JC125", 0, 0); g.board_mut(&c).unwrap().time_markers = 4;
    let host = g.regions[1].card.id.clone(); let a = attached(&mut g, "JC090", 1, &host); g.board_mut(&a).unwrap().time_markers = 2;
    fixture("time-marker-valid-board-and-attachment", &g);
    let h = held(&mut g, "JZ30", 0); let base = serde_json::to_value(&g).unwrap();
    let mut v = base.clone(); let i = g.players[0].hand.iter().position(|c| c.id == h).unwrap(); v["players"][0]["hand"][i]["time_markers"] = serde_json::json!(1); invalid(&g, v);
    let mut v = base.clone(); v["regions"][0]["cards"][0]["face_down"] = serde_json::json!(true); invalid(&g, v);
    let mut v = base.clone(); v["world"][0]["time_markers"] = serde_json::json!(1); invalid(&g, v);
    let mut v = base.clone(); v["regions"][0]["card"]["time_markers"] = serde_json::json!(1); invalid(&g, v);
    let mut v = base; v["attachments"][0]["hostId"] = serde_json::json!("missing"); invalid(&g, v);
}

fn room_action(r: &mut RoomEnvelope, seat: usize, action: Action) {
    let id = EVIDENCE.fetch_add(1, Ordering::Relaxed);
    let session = if action.kind == "pass" && r.pacing.window.is_some() {
        SessionAction::PassResponse { window_id: r.pacing.window.as_ref().unwrap().id.clone() }
    } else if let Some(window) = r.pacing.window.clone() {
        let intent_id = format!("red-intent-{id}");
        let begin = RoomCommand { command_id: format!("red-begin-{id}"), expected_version: r.revision,
            action: SessionAction::BeginResponse { window_id: window.id.clone(), intent_id: intent_id.clone() } };
        let out = r.transition(seat, Some(begin), 0).unwrap(); assert!(out.error_code.is_none(), "{:?}", out.error_message);
        *r = RoomEnvelope::from_persisted(&out.state).unwrap();
        SessionAction::SubmitResponse { window_id: window.id, intent_id, action }
    } else { SessionAction::Game { action } };
    let cmd = RoomCommand { command_id: format!("red-command-{id}"), expected_version: r.revision, action: session };
    let before = serde_json::to_string(r).unwrap(); let out = r.transition(seat, Some(cmd.clone()), 0).unwrap();
    assert!(out.error_code.is_none(), "{:?}", out.error_message);
    *r = RoomEnvelope::from_persisted(&out.state).unwrap();
    if let Ok(dir) = std::env::var("RED_TIME_EVIDENCE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(format!("{dir}/room-step-{id:04}.json"), serde_json::to_vec(&serde_json::json!({
            "state": before, "seat": seat, "command": cmd, "expected": out,
            "views": (0..4).map(|s| r.view(s,0)).collect::<Vec<_>>()
        })).unwrap()).unwrap();
    }
}
fn room_pass_top(r: &mut RoomEnvelope) {
    let n = r.game.stack.len(); assert!(n > 0);
    for _ in 0..32 { if r.game.stack.len() < n || r.game.pending.is_some() { return; }
        let seat = (0..4).find(|&s| r.game.legal_actions(s).iter().any(|a| a.action.kind == "pass")).unwrap();
        room_action(r, seat, Action::new("pass"));
    } panic!("room top did not resolve");
}
#[test]
fn red_time_real_room_paid_jz30_entry_destroy_and_private_forecast_restore() {
    let mut g = initial(0); fund(&mut g, 0, "XQ18", 1); fund(&mut g, 0, "JC084", 3);
    let h = held(&mut g, "JZ30", 0); let spell = held(&mut g, "JC091", 0); let mut r = envelope(&g);
    room_action(&mut r, 0, Action { card_id: Some(h), region: Some(2), ..Action::new("deploy") }); room_pass_top(&mut r);
    let p = r.game.pending.clone().unwrap(); room_action(&mut r, 0, Action { choice_id: Some(p.choice.id), selected: Some(vec!["accept".into()]), ..Action::new("choose") });
    room_pass_top(&mut r); let source = r.game.regions[2].cards[0].id.clone(); assert_eq!(r.game.board(&source).unwrap().1.time_markers, 1);
    let a = r.game.legal_actions(0).into_iter().find(|a| a.action.kind == "play" && a.action.card_id.as_deref() == Some(&spell)
        && a.action.target_id.as_deref() == Some(&source)).unwrap().action;
    room_action(&mut r, 0, a); room_pass_top(&mut r);
    let p = r.game.pending.clone().unwrap(); room_action(&mut r, 0, Action { choice_id: Some(p.choice.id), selected: Some(vec!["accept".into()]), ..Action::new("choose") });
    room_pass_top(&mut r); let p = r.game.pending.clone().unwrap(); assert_eq!(p.choice.amount, Some(1));
    r = RoomEnvelope::from_persisted(&serde_json::to_string(&r).unwrap()).unwrap();
    for seat in 1..4 { assert!(r.view(seat,0).game.pending_choice.is_none()); }
    room_action(&mut r, 0, Action { choice_id: Some(p.choice.id), top: Some(vec![]), bottom: Some(vec![p.choice.options[0].id.clone()]), ..Action::new("choose") });
    assert!(r.game.pending.is_none() && r.game.stack.is_empty());
    assert_eq!(r.game.players[0].deck.last().unwrap().id, p.choice.options[0].id);
    fixture("room-jz30-finished", &r.game);
}

#[test]
fn red_time_real_granted_renown_glory_freeze_region_and_reject_other_region_in_game_and_room() {
    for def in ["XQ18", "JZ30"] { for glory in [false, true] {
        let mut g = initial(0); let source = board(&mut g, def, 0, 2);
        if glory {
            // Vest preserves the defense-one character under JC089's -1.
            attached(&mut g, "XQ47", 0, &source); fund(&mut g, 0, "JC084", 2);
            cast(&mut g, "JC089", 0, &source); assert!(g.has_combat_glory(g.board(&source).unwrap().1));
        } else {
            fund(&mut g, 0, "JC075", 2); cast(&mut g, "JC074", 0, &source);
            assert!(g.has_renown(g.board(&source).unwrap().1));
        }
        g.begin_window(Window::Before(2, if glory { 1 } else { 2 }));
        for _ in 0..4 {
            let seat = (0..4).find(|&s| g.legal_actions(s).iter().any(|a| a.action.kind == "pass")).unwrap();
            apply(&mut g, seat, Action::new("pass"));
        }
        let p = g.pending.clone().expect("real confrontation granted trigger");
        let ChoiceResolution::Declare { declaration: d, .. } = &p.resolution else { panic!() };
        assert_eq!(d.source.card.id, source);
        assert_eq!(d.ability.key, if glory { "jc089-combat-glory" } else { "renown" });
        assert_eq!(d.source.source_region_instance.as_ref(), Some(&g.regions[2].card.id));
        let wrong = g.regions[0].card.id.clone(); let mut v = serde_json::to_value(&g).unwrap();
        v["pending"]["resolution"]["Declare"]["declaration"]["ability"]["ops"][0]["PlaceInfluence"]["region_instance"] = serde_json::json!(wrong);
        invalid(&g, v); fixture("red-granted-reward-choice", &g);
        let mut r = envelope(&g); room_action(&mut r, 0, Action { choice_id: Some(p.choice.id), selected: Some(vec!["accept".into()]), ..Action::new("choose") });
        let mut v = serde_json::to_value(&r.game).unwrap(); let i = r.game.stack.len()-1;
        v["stack"][i]["frame"]["steps"][0]["op"]["PlaceInfluence"]["region_instance"] = serde_json::json!(r.game.regions[0].card.id);
        invalid(&r.game, v); fixture("red-granted-reward-stack", &r.game);
        let before = r.game.regions[2].influence[0]; room_pass_top(&mut r);
        assert_eq!(r.game.regions[2].influence[0], before+1); assert_eq!(r.game.regions[0].influence, [0,0]);
        fixture("red-granted-reward-finished", &r.game);
    }}
}

#[test]
fn red_time_unknown_definitions_return_errors_without_panicking_before_forecast_projection() {
    let mut g = initial(0); let c = board(&mut g, "JZ30", 0, 2); g.board_mut(&c).unwrap().time_markers = 3;
    g.remove_dead(&c, RemovalCause::Destroy); g.drive().unwrap(); choose(&mut g, vec!["accept".into()]); pass_top(&mut g);
    let base = serde_json::to_value(&g).unwrap();
    let catch_invalid = |g: &Game, v: serde_json::Value| {
        let state = serde_json::to_string(&v).unwrap();
        let game_result = std::panic::catch_unwind(|| Game::from_persisted(&state));
        assert!(game_result.is_ok(), "Game load must return Err, never panic on unknown definitions");
        assert!(game_result.unwrap().is_err());
        let mut r = serde_json::to_value(envelope(&g)).unwrap(); r["game"] = v.clone();
        let state = serde_json::to_string(&r).unwrap();
        let room_result = std::panic::catch_unwind(|| RoomEnvelope::from_persisted(&state));
        assert!(room_result.is_ok(), "Room load must return Err, never panic on unknown definitions");
        assert!(room_result.unwrap().is_err());
        invalid(&g, v);
    };
    let mut v = base.clone(); v["players"][0]["deck"][0]["definition"] = serde_json::json!("unknown");
    v["players"][0]["deck"][0]["time_markers"] = serde_json::json!(0); catch_invalid(&g, v.clone());
    // Bypass loading only to exercise the operation's own defensive gate. No
    // operation is allowed to publish choices from these unknown printed cards.
    let mut malformed: Game = serde_json::from_value(v).unwrap();
    let ChoiceResolution::Frame { frame, .. } = malformed.pending.clone().unwrap().resolution else { panic!() };
    let before = serde_json::to_string(&malformed).unwrap();
    let start = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| malformed.jz30_forecast_frozen_time(&frame)));
    assert!(start.is_ok() && start.unwrap().is_err());
    assert_eq!(serde_json::to_string(&malformed).unwrap(), before);
    let p = malformed.pending.clone().unwrap(); let ids = p.choice.options.iter().map(|o| o.id.clone()).collect::<Vec<_>>();
    let choose_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| malformed.apply(0,
        Action { choice_id: Some(p.choice.id), top: Some(ids), bottom: Some(vec![]), ..Action::new("choose") })));
    assert!(choose_result.is_ok() && choose_result.unwrap().is_err());
    assert_eq!(serde_json::to_string(&malformed).unwrap(), before);

    let mut v = base.clone(); let mut card = serde_json::to_value(g.make_card("JC125", 0)).unwrap();
    card["definition"] = serde_json::json!("unknown"); card["time_markers"] = serde_json::json!(1);
    v["regions"][0]["cards"].as_array_mut().unwrap().push(card); catch_invalid(&g, v);
    let mut v = base.clone(); let mut card = serde_json::to_value(g.make_card("XQ47", 0)).unwrap();
    card["definition"] = serde_json::json!("unknown"); card["time_markers"] = serde_json::json!(1);
    v["attachments"].as_array_mut().unwrap().push(serde_json::json!({"card":card,"hostId":g.regions[0].card.id})); catch_invalid(&g, v);
    let mut v = base; let mut host = serde_json::to_value(g.make_card("JC125", 0)).unwrap();
    let host_id = host["id"].clone(); host["definition"] = serde_json::json!("unknown");
    v["regions"][0]["cards"].as_array_mut().unwrap().push(host);
    let mut card = serde_json::to_value(g.make_card("XQ47", 0)).unwrap(); card["time_markers"] = serde_json::json!(1);
    v["attachments"].as_array_mut().unwrap().push(serde_json::json!({"card":card,"hostId":host_id})); catch_invalid(&g, v);
}
