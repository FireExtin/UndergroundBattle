//! JC060 original-card rules and disclosed prepared Native layouts. After each
//! layout, exported traces contain continuous real Room commands and views.
use crate::{
    catalog,
    jc029_tests::{board, envelope, fund, game},
    model::*,
    room::*,
    rules::*,
};
use serde_json::{json, Value};
use std::path::Path;

fn held(g: &mut Game, definition: &str, actor: usize) -> String {
    let card = g.make_card(definition, actor);
    let id = card.id.clone();
    g.players[actor].hand.push(card);
    id
}
fn combat(g: &Game, id: &str) -> u32 {
    let (region, card) = g.board(id).unwrap();
    g.current_icons(card, region).combat
}
fn domains(g: &Game, id: &str) -> u32 {
    g.current_magic_count(g.board(id).unwrap().1)
}
fn restore(g: &Game) {
    let raw = serde_json::to_string(g).unwrap();
    let restored = Game::from_persisted(&raw).unwrap();
    assert_eq!(serde_json::to_string(&restored).unwrap(), raw);
    let room = envelope(g);
    let state = serde_json::to_string(&room).unwrap();
    let restored_room = RoomEnvelope::from_persisted(&state).unwrap();
    assert_eq!(serde_json::to_string(&restored_room).unwrap(), state);
    for seat in 0..g.players.len() {
        assert_eq!(
            serde_json::to_value(room.view(seat, 0)).unwrap(),
            serde_json::to_value(restored_room.view(seat, 0)).unwrap()
        );
    }
}

#[test]
fn jc060_complete_original_fields_and_closed_definition() {
    let d = catalog::card("JC060");
    assert_eq!(d.name, "神秘学顾问");
    assert_eq!(d.kind, "character");
    assert_eq!(d.color, "灰");
    assert_eq!(d.cost, 3);
    assert_eq!(d.loyalty, ["灰色"]);
    assert_eq!(d.subtypes, ["人类", "学者"]);
    assert_eq!(d.magic_icon, MagicIcon::Mind);
    assert_eq!(d.magic, "心灵");
    assert_eq!(d.defense, Some(1));
    assert!(!d.unique);
    assert_eq!(
        d.permanent_icons,
        Icons {
            investigation: 1,
            combat: 0,
            influence: 1
        }
    );
    assert_eq!(d.temporary_icons, Icons::default());
    assert!(definition("JC060").abilities.is_empty());
    for variant in 0..5 {
        let mut registry = definitions().clone();
        let d = registry.get_mut("JC060").unwrap();
        match variant {
            0 => d.modifiers.clear(),
            1 => d
                .modifiers
                .push(StaticModifier::JC060FriendlyUnprintedMindAndCombat),
            2 => d.traits.public = true,
            3 => d.graveyard_face_up = true,
            _ => d.abilities.push(definition("JC006").abilities[0].clone()),
        }
        assert!(
            validate_definitions(&registry).is_err(),
            "definition variant {variant}"
        );
    }
    let mut registry = definitions().clone();
    registry
        .get_mut("JC125")
        .unwrap()
        .modifiers
        .push(StaticModifier::JC060FriendlyUnprintedMindAndCombat);
    assert!(validate_definitions(&registry).is_err());
}

#[test]
fn jc060_all_four_seats_all_regions_team_scope_and_multiple_sources() {
    for actor in 0..4 {
        for count in 0..=3 {
            let mut g = game(actor);
            for i in 0..count {
                let source = board(&mut g, "JC060", actor, i);
                g.board_mut(&source).unwrap().exhausted = true;
            }
            let hidden_source = board(&mut g, "JC060", actor ^ 1, 4);
            g.board_mut(&hidden_source).unwrap().face_down = true;
            let enemy = (actor + 2) % 4;
            board(&mut g, "JC060", enemy, 4);
            for seat in 0..4 {
                for region in 0..5 {
                    let target = board(&mut g, "JC125", seat, region);
                    let expected = if g.team(seat) == g.team(actor) {
                        count as u32
                    } else {
                        1
                    };
                    assert_eq!(combat(&g, &target), expected);
                    assert_eq!(domains(&g, &target), expected);
                    // Repeated queries cannot make a grant invalidate itself.
                    assert_eq!(domains(&g, &target), expected);
                }
            }
            restore(&g);
        }
    }
}

#[test]
fn jc060_printed_domains_exclude_sources_and_other_magic_characters() {
    let mut g = game(0);
    let no_magic = board(&mut g, "JC125", 1, 4);
    let hidden = board(&mut g, "JC125", 1, 3);
    g.board_mut(&hidden).unwrap().face_down = true;
    let printed = ["JC002", "JC104", "JC018", "JC060"]
        .iter()
        .map(|id| board(&mut g, id, 0, 2))
        .collect::<Vec<_>>();
    let before = printed.iter().map(|id| combat(&g, id)).collect::<Vec<_>>();
    board(&mut g, "JC060", 1, 4);
    assert_eq!((combat(&g, &no_magic), domains(&g, &no_magic)), (2, 2));
    assert_eq!((combat(&g, &hidden), domains(&g, &hidden)), (0, 0));
    for (id, before) in printed.iter().zip(before) {
        assert_eq!(combat(&g, id), before);
        assert_eq!(domains(&g, id), 1);
        assert_eq!(g.jc060_friendly_sources(g.board(id).unwrap().1), 0);
    }
    assert_eq!(catalog::card("JC125").magic_icon, MagicIcon::None);
    assert_eq!(catalog::card("JC125").permanent_icons.combat, 0);
    restore(&g);
}

#[test]
fn jc060_assets_hand_graveyard_sealed_and_attachments_never_receive_character_grant() {
    let mut g = game(0);
    board(&mut g, "JC060", 0, 0);
    let target = board(&mut g, "JC125", 0, 2);
    fund(&mut g, 0, "JC125", 1);
    fund(&mut g, 0, "JC060", 1);
    let hand = held(&mut g, "JC125", 0);
    let dead = g.make_card("JC125", 0);
    g.players[0].graveyard.push(dead);
    let sealed = g.make_card("JC125", 0);
    g.sealed_cards.push(SealedCard {
        card: sealed,
        host_id: target.clone(),
    });
    let attachment = g.make_card("JC073", 0);
    let attachment_id = attachment.id.clone();
    g.attachments.push(Attachment {
        card: attachment,
        host_id: target.clone(),
    });
    for card in g.players[0]
        .assets
        .iter()
        .chain(&g.players[0].hand)
        .chain(&g.players[0].graveyard)
        .chain(g.sealed_cards.iter().map(|s| &s.card))
    {
        assert_eq!(g.jc060_friendly_sources(card), 0);
        assert_eq!(g.jc060_current_domains(card), None);
    }
    assert_eq!(domains(&g, &attachment_id), 1);
    assert_eq!(
        g.jc060_friendly_sources(g.board(&attachment_id).unwrap().1),
        0
    );
    for viewer in 0..4 {
        let view = g.view(viewer);
        assert!(view.assets.iter().all(|a| a.current_domains.is_none()));
        if viewer == 0 {
            assert!(view
                .hand
                .iter()
                .find(|c| c.instance_id == hand)
                .unwrap()
                .current_domains
                .is_none());
        }
        assert!(view
            .sealed_cards
            .iter()
            .all(|s| s.card.current_domains.is_none()));
    }
    restore(&g);
}

#[test]
fn jc060_grant_is_permanent_while_exhausted_and_survives_initiative_change() {
    let mut g = game(0);
    let source = board(&mut g, "JC060", 1, 4);
    let target = board(&mut g, "JC069", 0, 0);
    assert_eq!(combat(&g, &target), 3);
    assert_eq!(
        g.current_permanent_combat(g.board(&target).unwrap().1, 0),
        1
    );
    g.first_team = 1;
    assert_eq!(combat(&g, &target), 1);
    g.board_mut(&source).unwrap().exhausted = true;
    g.board_mut(&target).unwrap().exhausted = true;
    assert_eq!(combat(&g, &target), 1);
    assert_eq!(domains(&g, &target), 1);
    assert_eq!(g.icons(g.board(&target).unwrap().1, 0), Icons::default());
    for viewer in 0..4 {
        let view = g.view(viewer);
        let c = view.regions[0]
            .characters
            .iter()
            .find(|c| c.instance_id == target)
            .unwrap();
        assert_eq!(c.magic.as_deref(), Some(""));
        assert_eq!(
            c.current_domains,
            Some(vec![MagicDomainCount {
                magic: "心灵".into(),
                count: 1
            }])
        );
        assert_eq!(c.icons, Some(Icons::default()));
    }
    restore(&g);
}

#[test]
fn jc060_source_and_recipient_current_control_and_expiry_recompute_both_grants() {
    let mut g = game(0);
    let source = board(&mut g, "JC060", 2, 4);
    let friendly = board(&mut g, "JC125", 1, 0);
    let enemy = board(&mut g, "JC125", 3, 0);
    assert_eq!((domains(&g, &friendly), domains(&g, &enemy)), (0, 1));
    g.add_control(
        &source,
        0,
        ControlLifetime::TurnEnd { turn: g.turn },
        SubtypeChange::None,
    );
    assert_eq!((domains(&g, &friendly), domains(&g, &enemy)), (1, 0));
    g.add_control(
        &enemy,
        1,
        ControlLifetime::TurnEnd { turn: g.turn },
        SubtypeChange::None,
    );
    assert_eq!((domains(&g, &enemy), combat(&g, &enemy)), (1, 1));
    assert_eq!(g.board(&source).unwrap().1.owner, 2);
    assert_eq!(g.board(&enemy).unwrap().1.owner, 3);
    restore(&g);
    g.turn += 1;
    g.settle_controls();
    assert_eq!((domains(&g, &friendly), domains(&g, &enemy)), (0, 1));
    assert_eq!(g.board(&source).unwrap().1.controller, 2);
    assert_eq!(g.board(&enemy).unwrap().1.controller, 3);
    restore(&g);
}

#[test]
fn jc060_stacking_decreases_on_hide_and_leave_without_stale_snapshot_grants() {
    let mut g = game(0);
    let source_a = board(&mut g, "JC060", 0, 0);
    let source_b = board(&mut g, "JC060", 1, 4);
    let target = board(&mut g, "JC125", 0, 2);
    assert_eq!((combat(&g, &target), domains(&g, &target)), (2, 2));
    assert!(g.hide_in_place(&source_a));
    assert_eq!((combat(&g, &target), domains(&g, &target)), (1, 1));
    g.return_hand(&source_b);
    assert_eq!((combat(&g, &target), domains(&g, &target)), (0, 0));
    board(&mut g, "JC060", 1, 3);
    let snapshot = g.board(&target).unwrap().1.clone();
    g.return_hand(&target);
    assert_eq!(g.jc060_friendly_sources(&snapshot), 0);
    assert_eq!(g.current_magic_count(&snapshot), 0);
    restore(&g);
}

#[test]
fn jc060_spirit_counts_two_granted_icons_and_only_the_contested_region() {
    let mut g = game(0);
    let spirit = board(&mut g, "JZ58", 0, 2);
    let recipient = board(&mut g, "JC125", 1, 2);
    board(&mut g, "LC12", 2, 2);
    board(&mut g, "LC12", 3, 2);
    let first = board(&mut g, "JC060", 0, 0);
    assert!(!g.spirit_protected(g.board(&spirit).unwrap().1)); // Two to two.
    let second = board(&mut g, "JC060", 1, 4);
    g.board_mut(&recipient).unwrap().exhausted = true;
    assert!(g.spirit_protected(g.board(&spirit).unwrap().1)); // Three to two.
    let (_, moved) = g.remove_board(&recipient).unwrap();
    g.regions[3].cards.push(moved);
    assert!(!g.spirit_protected(g.board(&spirit).unwrap().1));
    assert_eq!(domains(&g, &recipient), 2); // Aura range is global; spirit range is local.
    g.return_hand(&second);
    g.return_hand(&first);
    fund(&mut g, 0, "JC060", 3);
    assert!(!g.spirit_protected(g.board(&spirit).unwrap().1));
    restore(&g);
}

#[test]
fn jc060_cost_loyalty_and_printed_asset_domains_keep_existing_boundaries() {
    for poor_loyalty in [false, true] {
        let mut g = game(0);
        fund(
            &mut g,
            0,
            if poor_loyalty { "JC125" } else { "JC061" },
            if poor_loyalty { 3 } else { 2 },
        );
        let source = held(&mut g, "JC060", 0);
        let action = Action {
            card_id: Some(source),
            region: Some(2),
            ..Action::new("deploy")
        };
        assert!(!g.legal_actions(0).iter().any(|a| a.action == action));
        let before = serde_json::to_string(&g).unwrap();
        assert!(g.apply(0, action).is_err());
        assert_eq!(serde_json::to_string(&g).unwrap(), before);
        restore(&g);
    }
    let mut g = game(0);
    board(&mut g, "JC060", 0, 0);
    board(&mut g, "JC125", 0, 0);
    fund(&mut g, 0, "JC125", 2);
    assert!(!g.actor_has_asset_domain(0, &MagicIcon::Mind, 1));
    fund(&mut g, 0, "JC060", 1);
    assert!(g.actor_has_asset_domain(0, &MagicIcon::Mind, 1));
    assert!(!g.actor_has_asset_domain(1, &MagicIcon::Mind, 1));
    assert!(!CardFilter::HasPrintedMagic.matches(catalog::card("JC125")));
}

#[test]
fn jc060_aura_cannot_make_a_printed_nonmagic_hand_card_consume_black_cat_discount() {
    let mut g = game(0);
    board(&mut g, "JC060", 0, 0);
    let cat = board(&mut g, "JC112", 0, 2);
    fund(&mut g, 0, "JC061", 5);
    let ordinary = held(&mut g, "JC059", 0);
    let adviser = held(&mut g, "JC060", 0);
    g.apply(
        0,
        Action {
            card_id: Some(cat),
            ability_id: Some("reduce-next-magic".into()),
            ..Action::new("activate")
        },
    )
    .unwrap();
    assert!(g.stack.is_empty());
    let ordinary_card = g.players[0].hand.iter().find(|c| c.id == ordinary).unwrap();
    assert_eq!(g.effective_cost(0, ordinary_card), 3);
    assert_eq!(g.current_magic_count(ordinary_card), 0);
    g.apply(
        0,
        Action {
            card_id: Some(ordinary),
            region: Some(2),
            ..Action::new("deploy")
        },
    )
    .unwrap();
    crate::jc029_tests::pass_top(&mut g);
    assert_eq!(g.resources(0), 2);
    assert_eq!(g.modifiers.len(), 1);
    assert_eq!(g.modifiers[0].uses, 1);
    let adviser_card = g.players[0].hand.iter().find(|c| c.id == adviser).unwrap();
    assert_eq!(g.effective_cost(0, adviser_card), 2);
    g.apply(
        0,
        Action {
            card_id: Some(adviser),
            region: Some(2),
            ..Action::new("deploy")
        },
    )
    .unwrap();
    crate::jc029_tests::pass_top(&mut g);
    assert_eq!(g.resources(0), 0);
    assert_eq!(g.modifiers[0].uses, 0);
    restore(&g);
}

/// One explicit offline layout followed by continuous actual Native commands.
/// Every checkpoint reloads both strict loaders and replays the real journal.
struct Trace {
    name: &'static str,
    room: RoomEnvelope,
    initial: Value,
    steps: Vec<Value>,
}
impl Trace {
    fn new(name: &'static str, mut g: Game) -> Self {
        g.room_id = "600000000000000000000001".into();
        let room = envelope(&g);
        let initial = json!({"state":serde_json::to_string(&room).unwrap(),"views":(0..room.game.players.len()).map(|s|room.view(s,0)).collect::<Vec<_>>()});
        Self {
            name,
            room,
            initial,
            steps: vec![],
        }
    }
    fn command(&mut self, seat: usize, action: SessionAction, error: bool) {
        let state = serde_json::to_string(&self.room).unwrap();
        let command = RoomCommand {
            command_id: format!("jc060-{}-{}", self.name, self.steps.len()),
            expected_version: self.room.revision,
            action,
        };
        let expected = self
            .room
            .transition(seat, Some(command.clone()), 0)
            .unwrap();
        assert_eq!(
            expected.error_code.is_some(),
            error,
            "{:?}",
            expected.error_message
        );
        if !error {
            assert_eq!(
                serde_json::to_string(&self.room.replay_events(&expected.journal).unwrap())
                    .unwrap(),
                expected.state
            );
        }
        if error {
            assert_eq!(expected.state, state);
        }
        self.room = RoomEnvelope::from_persisted(&expected.state).unwrap();
        assert_eq!(serde_json::to_string(&self.room).unwrap(), expected.state);
        let game = serde_json::to_string(&self.room.game).unwrap();
        assert_eq!(
            serde_json::to_string(&Game::from_persisted(&game).unwrap()).unwrap(),
            game
        );
        self.steps.push(json!({"state":state,"seat":seat,"command":command,"serverNowMs":"0","expected":expected,"views":(0..self.room.game.players.len()).map(|s|self.room.view(s,0)).collect::<Vec<_>>()}));
    }
    fn action(&mut self, seat: usize, action: Action) {
        if let Some(w) = self.room.pacing.window.clone() {
            let intent_id = format!("jc060-intent-{}", self.steps.len());
            self.command(
                seat,
                SessionAction::BeginResponse {
                    window_id: w.id.clone(),
                    intent_id: intent_id.clone(),
                },
                false,
            );
            self.command(
                seat,
                SessionAction::SubmitResponse {
                    window_id: w.id,
                    intent_id,
                    action,
                },
                false,
            );
        } else {
            self.command(seat, SessionAction::Game { action }, false);
        }
    }
    fn finish(&mut self) {
        for _ in 0..64 {
            if self.room.game.pending.is_some() {
                return;
            }
            if self.room.game.stack.is_empty() {
                return;
            }
            let w = self
                .room
                .pacing
                .window
                .clone()
                .expect("responsive paid frame");
            let seat = *w
                .members
                .iter()
                .find(|(_, d)| matches!(d, Decision::Undecided { .. }))
                .unwrap()
                .0;
            self.command(seat, SessionAction::PassResponse { window_id: w.id }, false);
        }
        panic!("finite stack did not finish");
    }
    fn legal(
        &self,
        actor: usize,
        kind: &str,
        source: &str,
        target: Option<&str>,
        region: Option<usize>,
    ) -> Action {
        self.room
            .game
            .legal_actions(actor)
            .into_iter()
            .find(|a| {
                a.action.kind == kind
                    && a.action.card_id.as_deref() == Some(source)
                    && target.is_none_or(|t| a.action.target_id.as_deref() == Some(t))
                    && region.is_none_or(|r| a.action.region == Some(r))
            })
            .unwrap_or_else(|| panic!("{} missing {kind} for {source}", self.name))
            .action
    }
    fn export(&self) {
        if let Ok(dir) = std::env::var("JC060_NATIVE_DIR") {
            std::fs::create_dir_all(&dir).unwrap();
            let bytes=serde_json::to_vec(&json!({"name":self.name,"scope":"Explicit prepared Native layout; continuous real Room commands, strict restore and journal replay; not browser play or migration","catalog":catalog::catalog(),"initial":self.initial,"steps":self.steps})).unwrap();
            let path = Path::new(&dir).join(format!("{}.json", self.name));
            use std::io::Write;
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .unwrap()
                .write_all(&bytes)
                .unwrap();
        }
    }
}

impl Trace {
    fn response_priority(&mut self, actor: usize) {
        for _ in 0..8 {
            let window = self
                .room
                .pacing
                .window
                .clone()
                .expect("live response window");
            if window.holder_team == self.room.game.team(actor) {
                return;
            }
            let seat = *window
                .members
                .iter()
                .find(|(_, d)| matches!(d, Decision::Undecided { .. }))
                .unwrap()
                .0;
            self.command(
                seat,
                SessionAction::PassResponse {
                    window_id: window.id,
                },
                false,
            );
        }
        panic!("response priority did not reach actor");
    }
}

#[test]
fn jc060_real_paid_deployment_gives_remote_teammate_current_public_domains() {
    let mut g = game(0);
    fund(&mut g, 0, "JC061", 3);
    let own = board(&mut g, "JC125", 0, 0);
    let teammate = board(&mut g, "JC125", 1, 4);
    let enemy = board(&mut g, "JC125", 2, 2);
    let hidden = board(&mut g, "JC125", 1, 3);
    g.board_mut(&hidden).unwrap().face_down = true;
    let source = held(&mut g, "JC060", 0);
    let mut t = Trace::new("paid-deploy-team", g);
    assert_eq!(domains(&t.room.game, &teammate), 0);
    t.action(0, t.legal(0, "deploy", &source, None, Some(2)));
    assert_eq!(domains(&t.room.game, &teammate), 0); // Pending entry has no aura.
    t.finish();
    assert_eq!(t.room.game.resources(0), 0);
    for id in [&own, &teammate] {
        assert_eq!(
            (combat(&t.room.game, id), domains(&t.room.game, id)),
            (1, 1)
        );
    }
    assert_eq!(
        (
            domains(&t.room.game, &enemy),
            domains(&t.room.game, &hidden)
        ),
        (0, 0)
    );
    for viewer in 0..4 {
        let view = t.room.view(viewer, 0);
        let target = view.regions[4]
            .characters
            .iter()
            .find(|c| c.instance_id == teammate)
            .unwrap();
        assert_eq!(target.magic.as_deref(), Some(""));
        assert_eq!(
            target.current_domains,
            Some(vec![MagicDomainCount {
                magic: "心灵".into(),
                count: 1
            }])
        );
        assert_eq!(target.icons.unwrap().combat, 1);
        let concealed = view.regions[3]
            .characters
            .iter()
            .find(|c| c.instance_id == hidden)
            .unwrap();
        assert!(concealed.current_domains.is_none());
        if viewer != 1 {
            assert!(concealed.card_id.is_none() && concealed.magic.is_none());
        }
    }
    t.export();
}

#[test]
fn jc060_real_paid_reveal_activates_global_aura_even_while_source_exhausted() {
    let mut g = game(1);
    fund(&mut g, 1, "JC061", 3);
    let source = board(&mut g, "JC060", 1, 4);
    g.board_mut(&source).unwrap().face_down = true;
    g.board_mut(&source).unwrap().exhausted = true;
    let recipient = board(&mut g, "JC125", 0, 0);
    let mut t = Trace::new("paid-reveal-team", g);
    assert_eq!(domains(&t.room.game, &recipient), 0);
    t.action(1, t.legal(1, "reveal", &source, None, None));
    t.finish();
    assert_eq!(t.room.game.resources(1), 0);
    assert_eq!(
        (
            combat(&t.room.game, &recipient),
            domains(&t.room.game, &recipient)
        ),
        (1, 1)
    );
    let revealed = t.room.game.regions[4]
        .cards
        .iter()
        .find(|c| c.definition == "JC060")
        .unwrap();
    assert!(revealed.exhausted && !revealed.face_down);
    assert_ne!(revealed.id, source);
    t.export();
}

#[test]
fn jc060_real_return_removes_only_one_of_two_stacked_grants() {
    let mut g = game(0);
    let source = board(&mut g, "JC060", 0, 0);
    board(&mut g, "JC060", 1, 4);
    let recipient = board(&mut g, "JC125", 1, 4);
    let rescuer = board(&mut g, "JC075", 0, 0);
    fund(&mut g, 0, "JC125", 2);
    let mut t = Trace::new("stacking-return", g);
    assert_eq!(
        (
            combat(&t.room.game, &recipient),
            domains(&t.room.game, &recipient)
        ),
        (2, 2)
    );
    t.action(0, t.legal(0, "activate", &rescuer, Some(&source), None));
    t.finish();
    assert!(t.room.game.board(&source).is_none());
    assert_eq!(
        (
            combat(&t.room.game, &recipient),
            domains(&t.room.game, &recipient)
        ),
        (1, 1)
    );
    assert!(t.room.game.players[0]
        .hand
        .iter()
        .any(|c| c.definition == "JC060"));
    t.export();
}

#[test]
fn jc060_real_hide_removes_global_grant_and_never_exposes_hidden_current_domains() {
    let mut g = game(0);
    let source = board(&mut g, "JC060", 0, 0);
    let recipient = board(&mut g, "JC125", 1, 4);
    fund(&mut g, 0, "XQ16", 2);
    let lawyer = held(&mut g, "XQ16", 0);
    let mut t = Trace::new("source-hide", g);
    t.action(0, t.legal(0, "deploy", &lawyer, None, Some(0)));
    t.finish();
    for _ in 0..4 {
        let Some(p) = t.room.game.pending.clone() else {
            break;
        };
        let selected = if p.choice.options.iter().any(|o| o.id == "accept") {
            vec!["accept".into()]
        } else {
            assert!(p.choice.options.iter().any(|o| o.id == source));
            vec![source.clone()]
        };
        t.action(
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                selected: Some(selected),
                ..Action::new("choose")
            },
        );
        t.finish();
    }
    assert_eq!(
        (
            combat(&t.room.game, &recipient),
            domains(&t.room.game, &recipient)
        ),
        (0, 0)
    );
    assert!(t.room.game.board(&source).is_none());
    let hidden = t.room.game.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "JC060")
        .unwrap();
    assert!(hidden.face_down);
    let hidden_id = hidden.id.clone();
    for seat in 0..4 {
        let view = t.room.view(seat, 0);
        let hidden = view.regions[0]
            .characters
            .iter()
            .find(|c| c.instance_id == hidden_id)
            .unwrap();
        assert!(hidden.current_domains.is_none());
        if seat != 0 {
            assert!(hidden.card_id.is_none() && hidden.magic.is_none());
        }
        assert!(view.regions[4]
            .characters
            .iter()
            .find(|c| c.instance_id == recipient)
            .unwrap()
            .current_domains
            .is_none());
    }
    t.export();
}

#[test]
fn jc060_real_source_control_changes_both_teams_without_changing_ownership() {
    let mut g = game(0);
    let source = board(&mut g, "JC060", 2, 4);
    let ally = board(&mut g, "JC125", 1, 0);
    let enemy = board(&mut g, "JC125", 3, 4);
    fund(&mut g, 0, "JC104", 3);
    let spell = held(&mut g, "JC129", 0);
    let mut t = Trace::new("source-control", g);
    assert_eq!(
        (domains(&t.room.game, &ally), domains(&t.room.game, &enemy)),
        (0, 1)
    );
    t.action(0, t.legal(0, "play", &spell, Some(&source), None));
    t.finish();
    assert_eq!(t.room.game.board(&source).unwrap().1.owner, 2);
    assert_eq!(t.room.game.board(&source).unwrap().1.controller, 0);
    assert_eq!(
        (domains(&t.room.game, &ally), domains(&t.room.game, &enemy)),
        (1, 0)
    );
    assert_eq!(
        (combat(&t.room.game, &ally), combat(&t.room.game, &enemy)),
        (1, 0)
    );
    t.export();
}

#[test]
fn jc060_real_recipient_control_acquires_teammates_aura() {
    let mut g = game(0);
    board(&mut g, "JC060", 1, 4);
    let recipient = board(&mut g, "JC125", 3, 0);
    fund(&mut g, 0, "JC104", 3);
    let spell = held(&mut g, "JC129", 0);
    let mut t = Trace::new("recipient-control", g);
    assert_eq!(domains(&t.room.game, &recipient), 0);
    t.action(0, t.legal(0, "play", &spell, Some(&recipient), None));
    t.finish();
    assert_eq!(t.room.game.board(&recipient).unwrap().1.owner, 3);
    assert_eq!(t.room.game.board(&recipient).unwrap().1.controller, 0);
    assert_eq!(
        (
            combat(&t.room.game, &recipient),
            domains(&t.room.game, &recipient)
        ),
        (1, 1)
    );
    t.export();
}

#[test]
fn jc060_granted_magic_is_a_legal_real_return_spell_target() {
    let mut g = game(0);
    board(&mut g, "JC060", 0, 0);
    let recipient = board(&mut g, "JC125", 1, 4);
    fund(&mut g, 0, "JC002", 2);
    let gate = held(&mut g, "JC006", 0);
    let mut t = Trace::new("current-magic-return", g);
    assert_eq!(catalog::card("JC125").magic_icon, MagicIcon::None);
    t.action(0, t.legal(0, "play", &gate, Some(&recipient), None));
    t.finish();
    assert!(t.room.game.board(&recipient).is_none());
    assert!(t.room.game.players[1]
        .hand
        .iter()
        .any(|c| c.definition == "JC125"));
    assert_eq!(t.room.game.resources(0), 0);
    t.export();
}

#[test]
fn jc060_magic_target_is_rechecked_when_response_removes_last_source() {
    let mut g = game(2);
    let source = board(&mut g, "JC060", 0, 0);
    let recipient = board(&mut g, "JC125", 1, 4);
    fund(&mut g, 2, "JC002", 2);
    fund(&mut g, 0, "JC002", 2);
    let attack_gate = held(&mut g, "JC006", 2);
    let rescue_gate = held(&mut g, "JC006", 0);
    let mut t = Trace::new("magic-target-revalidation", g);
    t.action(2, t.legal(2, "play", &attack_gate, Some(&recipient), None));
    t.response_priority(0);
    t.action(0, t.legal(0, "play", &rescue_gate, Some(&source), None));
    t.finish();
    assert!(t.room.game.board(&source).is_none());
    assert!(t.room.game.board(&recipient).is_some());
    assert_eq!(
        (
            combat(&t.room.game, &recipient),
            domains(&t.room.game, &recipient)
        ),
        (0, 0)
    );
    assert_eq!((t.room.game.resources(0), t.room.game.resources(2)), (0, 0));
    assert!(t.room.game.players[2]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC006"));
    t.export();
}

#[test]
fn jc060_holy_water_uses_current_domains_for_every_local_character() {
    let mut g = game(0);
    let adviser = board(&mut g, "JC060", 0, 4);
    let rescuer = board(&mut g, "JC075", 0, 4);
    fund(&mut g, 0, "JC125", 2);
    let own = board(&mut g, "JC059", 0, 2);
    let ally = board(&mut g, "JC059", 1, 2);
    let enemy = board(&mut g, "JC059", 2, 2);
    let remote = board(&mut g, "JC059", 1, 3);
    let water = g.make_card("XQ07", 0);
    let source = water.id.clone();
    g.attachments.push(Attachment {
        card: water,
        host_id: own.clone(),
    });
    let mut t = Trace::new("holy-water-current-domain", g);
    t.action(0, t.legal(0, "activate", &source, None, None));
    t.finish();
    assert!(t.room.game.board(&source).is_none());
    for id in [&own, &ally] {
        assert_eq!(
            t.room
                .game
                .printed_defense_override(t.room.game.board(id).unwrap().1),
            Some(1)
        );
    }
    for id in [&enemy, &remote] {
        assert_eq!(
            t.room
                .game
                .printed_defense_override(t.room.game.board(id).unwrap().1),
            None
        );
    }
    t.action(0, t.legal(0, "activate", &rescuer, Some(&adviser), None));
    t.finish();
    assert_eq!(domains(&t.room.game, &own), 0);
    for id in [&own, &ally] {
        // The already resolved turn effect remains after the aura disappears.
        assert_eq!(
            t.room
                .game
                .printed_defense_override(t.room.game.board(id).unwrap().1),
            Some(1)
        );
    }
    t.export();
}

#[test]
fn jc060_real_spirit_damage_uses_stacked_granted_domain_count_not_card_count() {
    let mut g = game(0);
    let first = board(&mut g, "JC060", 0, 0);
    let recipient = board(&mut g, "JC125", 1, 2);
    let spirit = board(&mut g, "JZ58", 0, 2);
    board(&mut g, "LC12", 2, 2);
    board(&mut g, "LC12", 3, 2);
    let rescuer = board(&mut g, "JC075", 0, 0);
    fund(&mut g, 0, "JC061", 3);
    fund(&mut g, 0, "JC104", 6);
    let second = held(&mut g, "JC060", 0);
    let blast_a = held(&mut g, "JC102", 0);
    let blast_b = held(&mut g, "JC102", 0);
    let mut t = Trace::new("spirit-two-domains", g);
    assert!(!t
        .room
        .game
        .spirit_protected(t.room.game.board(&spirit).unwrap().1));
    t.action(0, t.legal(0, "deploy", &second, None, Some(0)));
    t.finish();
    assert_eq!(domains(&t.room.game, &recipient), 2);
    assert!(t
        .room
        .game
        .spirit_protected(t.room.game.board(&spirit).unwrap().1));
    t.action(0, t.legal(0, "play", &blast_a, Some(&spirit), None));
    t.finish();
    assert_eq!(t.room.game.board(&spirit).unwrap().1.damage, 0);
    t.action(0, t.legal(0, "activate", &rescuer, Some(&first), None));
    t.finish();
    assert_eq!(domains(&t.room.game, &recipient), 1);
    assert!(!t
        .room
        .game
        .spirit_protected(t.room.game.board(&spirit).unwrap().1));
    t.action(0, t.legal(0, "play", &blast_b, Some(&spirit), None));
    t.finish();
    assert!(t.room.game.board(&spirit).is_none());
    t.export();
}

#[test]
fn jc060_duel_paid_deployment_uses_two_player_friendship_and_views() {
    let mut g = Game::new(
        "jc060-duel".into(),
        "LOCAL".into(),
        "duel".into(),
        "P0".into(),
        "watchers".into(),
        60,
    )
    .unwrap();
    g.join("P1".into(), "watchers".into()).unwrap();
    for p in &mut g.players {
        p.ready = true;
    }
    g.apply(0, Action::new("start")).unwrap();
    while let Some(p) = g.pending.clone() {
        g.apply(
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                selected: Some(vec![]),
                ..Action::new("choose")
            },
        )
        .unwrap();
    }
    for p in &mut g.players {
        p.hand.clear();
        p.assets.clear();
        p.graveyard.clear();
    }
    for region in &mut g.regions {
        region.cards.clear();
        region.influence = [0, 0];
    }
    g.first_team = 0;
    g.begin_window(Window::Action(0));
    let own = board(&mut g, "JC125", 0, 0);
    let enemy = board(&mut g, "JC125", 1, 2);
    fund(&mut g, 0, "JC061", 3);
    let source = held(&mut g, "JC060", 0);
    let mut t = Trace::new("duel-paid-deploy", g);
    t.action(0, t.legal(0, "deploy", &source, None, Some(1)));
    t.finish();
    assert_eq!(
        (combat(&t.room.game, &own), domains(&t.room.game, &own)),
        (1, 1)
    );
    assert_eq!(
        (combat(&t.room.game, &enemy), domains(&t.room.game, &enemy)),
        (0, 0)
    );
    assert_eq!(t.room.game.resources(0), 0);
    for seat in 0..2 {
        let view = t.room.view(seat, 0);
        assert_eq!((view.players.len(), view.regions.len()), (2, 3));
        assert_eq!(
            view.regions[0]
                .characters
                .iter()
                .find(|c| c.instance_id == own)
                .unwrap()
                .current_domains
                .as_ref()
                .unwrap()[0]
                .count,
            1
        );
        assert!(view.regions[2]
            .characters
            .iter()
            .find(|c| c.instance_id == enemy)
            .unwrap()
            .current_domains
            .is_none());
    }
    t.export();
}
