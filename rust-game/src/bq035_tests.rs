//! Explicit offline layouts, followed by real Game actions and exact restore.
use crate::{catalog, model::*, room::*, rules::*, unit_support as u};
use serde_json::{json, Value};
use std::path::Path;

fn field(g: &mut Game, def: &str, seat: usize, r: usize) -> String {
    let c = g.make_card(def, seat);
    let id = c.id.clone();
    g.regions[r].cards.push(c);
    id
}
fn attach(g: &mut Game, def: &str, seat: usize, host: &str) -> String {
    let card = g.make_card(def, seat);
    let id = card.id.clone();
    g.attachments.push(Attachment {
        card,
        host_id: host.into(),
    });
    let spec = definition(def).attachment.clone().unwrap();
    if spec.controls_host {
        g.add_control(
            host,
            seat,
            ControlLifetime::Attached {
                source_instance: id.clone(),
            },
            spec.host_subtype_change,
        );
    }
    assert!(
        g.attachment_host_valid(g.attachments.last().unwrap()),
        "invalid prepared {def}"
    );
    id
}
fn granted() -> (Game, String, String, String) {
    let mut g = u::game();
    let host = field(&mut g, "JC029", 0, 0);
    let knight = field(&mut g, "BQ035", 0, 4);
    let blood = attach(&mut g, "XQ14", 0, &host);
    (g, host, knight, blood)
}
fn start_mobility(g: &mut Game) {
    g.begin_window(Window::Action(1 - g.first_team));
    for _ in 0..8 {
        if g.window == Some(Window::Mobility) {
            return;
        }
        u::pass(g);
    }
    panic!("did not naturally enter mobility");
}
fn pending_declaration(g: &mut Game) -> &mut Declaration {
    match &mut g.pending.as_mut().unwrap().resolution {
        ChoiceResolution::Declare { declaration, .. } => declaration,
        _ => panic!("not a declaration"),
    }
}
fn invalid(g: &Game, name: &str) {
    assert!(
        Game::from_persisted(&serde_json::to_string(g).unwrap()).is_err(),
        "Game accepted {name}"
    );
    // A malformed Game must not pass through the helper that deliberately
    // validates positive fixtures before returning an envelope.
    let room = crate::room::RoomEnvelope::from_game(g.clone());
    let raw = serde_json::to_string(&room).unwrap();
    assert!(
        crate::room::RoomEnvelope::from_persisted(&raw).is_err(),
        "Room accepted {name}"
    );
    if let Ok(dir) = std::env::var("BQ035_INVALID_DIR") {
        let dir = std::path::Path::new(&dir);
        std::fs::create_dir_all(dir).unwrap();
        let path = dir.join(format!("{name}.json"));
        let bytes = serde_json::to_vec(&json!({"state":raw})).unwrap();
        if path.exists() {
            assert_eq!(std::fs::read(&path).unwrap(), bytes);
        } else {
            std::fs::write(path, bytes).unwrap();
        }
    }
}

#[test]
fn bq035_complete_fields_and_definition_close_every_printed_clause() {
    let c = catalog::card("BQ035");
    assert_eq!(
        (&*c.name, &*c.color, c.cost, c.defense),
        ("幽冥骑士", "蓝", 5, Some(3))
    );
    assert_eq!(c.loyalty, ["蓝色"]);
    assert_eq!(c.magic_icon, MagicIcon::Blood);
    assert_eq!(c.subtypes, ["吸血鬼", "猎手"]);
    assert_eq!(c.keywords, ["机动", "威名"]);
    assert_eq!(
        c.permanent_icons,
        Icons {
            investigation: 0,
            combat: 2,
            influence: 1
        }
    );
    assert_eq!(
        c.temporary_icons,
        Icons {
            investigation: 1,
            combat: 0,
            influence: 0
        }
    );
    assert!(!c.unique);
    assert!(!definition("BQ035").traits.renown);
    for variant in 0..9 {
        let mut d = definition("BQ035").clone();
        match variant {
            0 => {
                d.modifiers.pop();
            }
            1 => d.modifiers.clear(),
            2 => d.modifiers.push(d.modifiers[0].clone()),
            3 => d.abilities.clear(),
            4 => d.traits.renown = true,
            5 => d.abilities[0].response_policy = ResponsePolicy::Immediate,
            6 => d.abilities[0].requires_ready_source = false,
            7 => d.abilities[0].costs = vec![Cost::Assets(1)],
            _ => d.abilities[0].ops = vec![Op::Exhaust(EntityRef::Source)],
        }
        let mut registry = definitions().clone();
        registry.insert("BQ035".into(), d);
        assert!(
            validate_definitions(&registry).is_err(),
            "variant {variant}"
        );
    }
    for id in ["JC125", "JC029", "JC020", "JC006"] {
        let mut registry = definitions().clone();
        let mut d = definition(id).clone();
        d.abilities.push(crate::mobility::ability());
        registry.insert(id.into(), d);
        assert!(
            validate_definitions(&registry).is_err(),
            "static transplant {id}"
        );
    }
}

#[test]
fn bq035_aura_uses_current_same_controller_across_regions_not_team_or_owner() {
    let mut g = u::game();
    let knight = field(&mut g, "BQ035", 0, 4);
    for seat in 0..4 {
        let host = field(&mut g, "JC029", seat, seat);
        attach(&mut g, "XQ14", 3, &host);
        assert_eq!(g.has_mobility(g.board(&host).unwrap().1), seat == 0);
    }
    g.board_mut(&knight).unwrap().exhausted = true;
    assert!(g.has_mobility(&g.regions[0].cards[0]));
    g.board_mut(&knight).unwrap().controller = 2;
    assert!(!g.has_mobility(&g.regions[0].cards[0]));
    assert!(g.has_mobility(&g.regions[2].cards[0]));
    g.board_mut(&knight).unwrap().face_down = true;
    assert!(!g.has_mobility(&g.regions[2].cards[0]));
}

#[test]
fn bq035_real_named_blood_and_equipment_are_independent_boolean_grants() {
    for blood in ["XQ14", "BQ040"] {
        let mut g = u::game();
        field(&mut g, "BQ035", 0, 4);
        let host = field(&mut g, "JC029", 0, 0);
        attach(&mut g, blood, 3, &host);
        attach(&mut g, blood, 1, &host);
        assert!(g.has_mobility(g.board(&host).unwrap().1));
        assert!(!g.has_combat_glory(g.board(&host).unwrap().1));
        for equipment in ["JC020", "JC093", "BQ022"] {
            let id = attach(&mut g, equipment, 0, &host);
            assert!(g.has_combat_glory(g.board(&host).unwrap().1));
            g.attachments.retain(|a| a.card.id != id);
        }
        assert!(!g.has_combat_glory(g.board(&host).unwrap().1));
    }
}

#[test]
fn bq035_printed_knight_keeps_both_keywords_without_any_attachment_or_other_source() {
    let mut g = u::game();
    let knight = field(&mut g, "BQ035", 0, 0);
    let c = g.board(&knight).unwrap().1;
    assert!(g.has_mobility(c) && g.has_combat_glory(c));
    assert!(!g.has_renown(c));
    g.board_mut(&knight).unwrap().exhausted = true;
    assert!(g.has_mobility(g.board(&knight).unwrap().1));
    start_mobility(&mut g);
    assert!(g.pending.is_none());
}

#[test]
fn bq035_grants_recompute_on_loss_rehost_control_and_ignore_stale_card_copies() {
    let (mut g, host, knight, blood) = granted();
    let old = g.board(&host).unwrap().1.clone();
    let second = field(&mut g, "JC029", 0, 1);
    g.attachments
        .iter_mut()
        .find(|a| a.card.id == blood)
        .unwrap()
        .host_id = second.clone();
    assert!(!g.has_mobility(&old));
    assert!(g.has_mobility(g.board(&second).unwrap().1));
    g.board_mut(&second).unwrap().controller = 2;
    assert!(!g.has_mobility(g.board(&second).unwrap().1));
    g.board_mut(&knight).unwrap().controller = 2;
    assert!(g.has_mobility(g.board(&second).unwrap().1));
    g.return_hand(&knight);
    assert!(!g.has_mobility(g.board(&second).unwrap().1));
}

#[test]
fn bq035_current_subtype_and_bat_assets_qualify_without_illegal_equipment() {
    let mut g = u::game();
    field(&mut g, "BQ035", 0, 4);
    let human = field(&mut g, "JC125", 1, 0);
    attach(&mut g, "JC036", 0, &human);
    attach(&mut g, "XQ14", 2, &human);
    assert!(g.has_mobility(g.board(&human).unwrap().1));
    let bat = field(&mut g, "JC030", 0, 1);
    u::fund(&mut g, 0, "XQ14", 2);
    attach(&mut g, "XQ14", 0, &bat);
    assert!(g.has_mobility(g.board(&bat).unwrap().1));
    g.players[0].assets.pop();
    assert!(!g.has_mobility(g.board(&bat).unwrap().1));
}

#[test]
fn bq035_continuous_grant_ignores_barrier_and_does_not_spend_shields() {
    let (mut g, host, _, _) = granted();
    g.board_mut(&host).unwrap().shield = 2;
    attach(&mut g, "JC093", 2, &host);
    let c = g.board(&host).unwrap().1;
    assert!(g.has_mobility(c) && g.has_combat_glory(c));
    assert_eq!(c.shield, 2);
}

#[test]
fn bq035_current_projection_is_only_for_public_in_play_characters() {
    let (mut g, host, _, _) = granted();
    let c = g.board(&host).unwrap().1.clone();
    for viewer in 0..4 {
        assert_eq!(
            g.card_view(&c, viewer, Some(0), None).current_mobility,
            Some(true)
        );
        assert_eq!(g.card_view(&c, viewer, None, None).current_mobility, None);
        assert_eq!(
            g.card_view(&c, viewer, Some(0), Some("asset"))
                .current_mobility,
            None
        );
    }
    g.board_mut(&host).unwrap().face_down = true;
    let c = g.board(&host).unwrap().1;
    for viewer in 0..4 {
        assert_eq!(g.card_view(c, viewer, Some(0), None).current_mobility, None);
    }
}

#[test]
fn bq035_natural_granted_mobility_is_optional_adjacent_and_preserves_uid_and_blood() {
    let (mut g, host, _, blood) = granted();
    start_mobility(&mut g);
    u::restore(&mut g);
    assert_eq!(pending_declaration(&mut g).source.card.id, host);
    assert_eq!(
        g.pending
            .as_ref()
            .unwrap()
            .choice
            .options
            .iter()
            .map(|o| o.id.as_str())
            .collect::<Vec<_>>(),
        ["region:1"]
    );
    u::choose(&mut g, vec!["region:1".into()]);
    assert_eq!(g.stack.last().unwrap().target.as_deref(), Some("region:1"));
    u::top(&mut g);
    assert_eq!(g.board(&host).unwrap().0, 1);
    assert_eq!(
        g.attachments
            .iter()
            .find(|a| a.card.id == blood)
            .unwrap()
            .host_id,
        host
    );
}

#[test]
fn bq035_granted_mobility_decline_does_not_requeue_on_repeated_events_or_reattach() {
    let (mut g, host, _, _) = granted();
    start_mobility(&mut g);
    u::choose(&mut g, vec![]);
    assert!(g.pending.as_ref().is_some_and(|p|matches!(&p.resolution,ChoiceResolution::Declare{declaration,..} if declaration.source.card.definition=="BQ035")));
    assert!(!g
        .effects
        .iter()
        .any(|e| matches!(e,Effect::Declare{declaration}if declaration.source.card.id==host)));
    let source = g.mobility_source(g.board(&host).unwrap().1, 0);
    g.emit_event(0, source, Event::ConfrontationStart);
    assert!(!g
        .effects
        .iter()
        .any(|e| matches!(e,Effect::Declare{declaration}if declaration.source.card.id==host)));
    assert_eq!(
        g.turn_ability_usage
            .iter()
            .filter(|u| u.ability_key == "mobility" && u.source_instance == host)
            .count(),
        1
    );
}

#[test]
fn bq035_queued_grant_survives_real_source_departure_and_attachment_cleanup() {
    for remove in ["knight", "blood"] {
        let (mut g, host, knight, blood) = granted();
        start_mobility(&mut g);
        if remove == "knight" {
            g.return_hand(&knight);
        } else {
            g.remove_dead(&blood, RemovalCause::Destroy);
        }
        assert!(!g.has_mobility(g.board(&host).unwrap().1));
        u::restore(&mut g);
        u::choose(&mut g, vec!["region:1".into()]);
        u::top(&mut g);
        assert_eq!(g.board(&host).unwrap().0, 1);
    }
}

#[test]
fn bq035_accepted_grant_survives_witness_departure_and_exhaustion_but_not_receiver_departure() {
    for remove in ["knight", "blood", "receiver", "exhaust"] {
        let (mut g, host, knight, blood) = granted();
        start_mobility(&mut g);
        u::choose(&mut g, vec!["region:1".into()]);
        match remove {
            "knight" => g.return_hand(&knight),
            "blood" => g.remove_dead(&blood, RemovalCause::Destroy),
            "receiver" => g.return_hand(&host),
            _ => g.board_mut(&host).unwrap().exhausted = true,
        }
        u::restore(&mut g);
        u::top(&mut g);
        if remove == "receiver" {
            assert!(g.board(&host).is_none());
        } else {
            assert_eq!(g.board(&host).unwrap().0, 1);
        }
        if remove == "exhaust" {
            assert!(g.board(&host).unwrap().1.exhausted);
        }
    }
}

#[test]
fn bq035_every_admitted_human_or_vampire_uses_its_existing_validator_with_real_grant() {
    let ids = catalog::catalog()
        .cards
        .iter()
        .filter(|c| {
            c.kind == "character" && c.subtypes.iter().any(|s| s == "人类" || s == "吸血鬼")
        })
        .map(|c| c.id.clone())
        .collect::<Vec<_>>();
    assert!(ids.len() > 40);
    for id in ids {
        if crate::mobility::printed(&id) {
            continue;
        }
        let mut g = u::game();
        let host = field(&mut g, &id, 1, 0);
        field(&mut g, "BQ035", 0, 4);
        if catalog::card(&id).subtypes.iter().any(|s| s == "人类") {
            attach(&mut g, "JC036", 0, &host);
        } else {
            g.board_mut(&host).unwrap().controller = 0;
        }
        attach(&mut g, "XQ14", 0, &host);
        assert!(
            g.has_mobility(g.board(&host).unwrap().1),
            "current grant {id}"
        );
        start_mobility(&mut g);
        assert_eq!(
            pending_declaration(&mut g).source.card.id,
            host,
            "pending {id}"
        );
        u::restore(&mut g);
        u::choose(&mut g, vec!["region:1".into()]);
        u::restore(&mut g);
        assert_eq!(
            g.stack
                .last()
                .unwrap()
                .frame
                .as_ref()
                .unwrap()
                .source
                .card
                .definition,
            id
        );
    }
}

#[test]
fn bq035_pending_observation_and_program_tampering_reject_both_storage_entries() {
    let (mut g, _, _, _) = granted();
    start_mobility(&mut g);
    for variant in 0..23 {
        let mut bad = g.clone();
        let d = pending_declaration(&mut bad);
        match variant {
            0 => d.actor = 1,
            1 => d.source.card.owner = 3,
            2 => d.source.card.controller = 2,
            3 => d.source.card.id.clear(),
            4 => d.source.region = Some(99),
            5 => d.source.observed_mobility_grant = None,
            6 => d.source.observed_mobility_grant.as_mut().unwrap().vampire = false,
            7 => d.source.observed_mobility_grant.as_mut().unwrap().turn += 1,
            8 => {
                d.source
                    .observed_mobility_grant
                    .as_mut()
                    .unwrap()
                    .knight
                    .definition = "JC125".into()
            }
            9 => {
                d.source
                    .observed_mobility_grant
                    .as_mut()
                    .unwrap()
                    .knight
                    .controller = 1
            }
            10 => {
                d.source
                    .observed_mobility_grant
                    .as_mut()
                    .unwrap()
                    .knight
                    .owner = 2
            }
            11 => {
                d.source
                    .observed_mobility_grant
                    .as_mut()
                    .unwrap()
                    .blood
                    .card
                    .definition = "JC089".into()
            }
            12 => {
                d.source
                    .observed_mobility_grant
                    .as_mut()
                    .unwrap()
                    .blood
                    .host_id = "other".into()
            }
            13 => {
                d.source
                    .observed_mobility_grant
                    .as_mut()
                    .unwrap()
                    .blood
                    .card
                    .face_down = true
            }
            14 => d.ability.ops = vec![],
            15 => d.ability.targets[0].range = Range::Anywhere,
            16 => d.ability.response_policy = ResponsePolicy::Immediate,
            17 => d.ability.costs = vec![Cost::Assets(1)],
            18 => d.ability.requires_ready_source = false,
            19 => d.ability.event = Some(Event::Enter),
            20 => d.source.source_region_instance = Some("unrelated-region".into()),
            21 => d.source.play_source = Some(PlaySource::Hand),
            _ => d.source.card.exhausted = true,
        }
        invalid(&bad, &format!("pending-observation-{variant:02}"));
    }
    for variant in 0..7 {
        let mut bad = g.clone();
        let p = bad.pending.as_mut().unwrap();
        match variant {
            0 => p.seat = 1,
            1 => p.choice.max = Some(2),
            2 => p.choice.options.clear(),
            3 => p.choice.allow_decline = Some(false),
            4 => p.choice.player_id = "p3".into(),
            5 => p.choice.title = "forged".into(),
            _ => {
                if let ChoiceResolution::Declare { stage, .. } = &mut p.resolution {
                    *stage = DeclareChoice::Accept
                }
            }
        }
        invalid(&bad, &format!("pending-shape-{variant:02}"));
    }
}

#[test]
fn bq035_response_frame_tampering_and_duplicate_containers_reject() {
    let (mut g, _, _, _) = granted();
    start_mobility(&mut g);
    u::choose(&mut g, vec!["region:1".into()]);
    for variant in 0..15 {
        let mut bad = g.clone();
        let item = bad.stack.last_mut().unwrap();
        let f = item.frame.as_mut().unwrap();
        match variant {
            0 => f.cursor = 1,
            1 => f.guard = GuardState::Accepted,
            2 => f.steps.push(f.steps[0].clone()),
            3 => f.steps[0].context = 2,
            4 => f.already_paid.push(PaidCost::Assets(vec![])),
            5 => f.targets.push(f.targets[0].clone()),
            6 => f.targets[0].id = "region:01".into(),
            7 => f.targets[0].spec.range = Range::Anywhere,
            8 => f.chosen_region = Some(1),
            9 => f.source.observed_mobility_grant = None,
            10 => f.ability_key = "forged".into(),
            11 => item.controller = 3,
            12 => item.target = None,
            13 => item.frame = None,
            _ => item.card = Some(f.source.card.clone()),
        }
        invalid(&bad, &format!("response-frame-{variant:02}"));
    }
    let mut bad = g.clone();
    let f = bad.stack.last().unwrap().frame.as_ref().unwrap().clone();
    bad.effects.push_front(Effect::Frame { frame: Box::new(f) });
    invalid(&bad, "direct-frame-and-stack");
    let mut bad = g.clone();
    let f = bad.stack.last().unwrap().frame.as_ref().unwrap();
    let d = Declaration {
        actor: f.actor,
        source: f.source.clone(),
        ability: crate::mobility::ability(),
    };
    bad.effects.push_back(Effect::Declare { declaration: d });
    invalid(&bad, "duplicate-stack-and-declaration");
}

#[test]
fn bq035_paid_deploy_in_every_seat_checks_blue_loyalty_and_atomic_payment() {
    for actor in 0..4 {
        let mut g = u::game();
        g.begin_window(Window::Action(g.team(actor)));
        let held = u::held(&mut g, "BQ035", actor);
        u::fund(&mut g, actor, "JC125", 5);
        let a = Action {
            card_id: Some(held.clone()),
            region: Some(2),
            ..Action::new("deploy")
        };
        u::reject(&mut g, actor, a.clone());
        g.players[actor].assets.pop();
        u::fund(&mut g, actor, "JC029", 1);
        u::act(&mut g, actor, a);
        assert_eq!(
            g.players[actor]
                .assets
                .iter()
                .filter(|a| a.exhausted)
                .count(),
            5
        );
        u::top(&mut g);
        let c = g.regions[2]
            .cards
            .iter()
            .find(|c| c.definition == "BQ035")
            .unwrap();
        assert_eq!(c.controller, actor);
        assert!(g.has_mobility(c) && g.has_combat_glory(c));
        assert!(g.pending.is_none());
    }
}

#[test]
fn bq035_hidden_conceal_and_paid_reveal_have_no_fake_entry_trigger() {
    let mut g = u::game();
    let source = u::held(&mut g, "BQ035", 0);
    u::fund(&mut g, 0, "JC029", 8);
    u::act(
        &mut g,
        0,
        Action {
            card_id: Some(source),
            region: Some(0),
            ..Action::new("conceal")
        },
    );
    let hidden = g.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "BQ035")
        .unwrap()
        .id
        .clone();
    assert!(!g.has_mobility(g.board(&hidden).unwrap().1));
    assert!(!g.has_combat_glory(g.board(&hidden).unwrap().1));
    u::reject(
        &mut g,
        0,
        Action {
            card_id: Some(hidden.clone()),
            target_id: Some(hidden.clone()),
            ..Action::new("reveal")
        },
    );
    u::act(
        &mut g,
        0,
        Action {
            card_id: Some(hidden),
            ..Action::new("reveal")
        },
    );
    u::top(&mut g);
    assert!(g.pending.is_none());
    let c = g.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "BQ035")
        .unwrap();
    assert!(!c.face_down && g.has_mobility(c) && g.has_combat_glory(c));
}

#[test]
fn bq035_trigger_cannot_be_forged_as_an_ordinary_action() {
    let mut g = u::game();
    let knight = field(&mut g, "BQ035", 0, 0);
    u::reject(
        &mut g,
        0,
        Action {
            card_id: Some(knight),
            ability_id: Some("mobility".into()),
            target_id: Some("region:1".into()),
            ..Action::new("activate")
        },
    );
}

fn combat_choice(g: &mut Game, region: usize) {
    g.begin_window(Window::Before(region, 1));
    for _ in 0..4 {
        u::pass(g);
    }
    if g.pending
        .as_ref()
        .is_some_and(|p| p.choice.kind == "recipient")
    {
        let id = g.pending.as_ref().unwrap().choice.options[0].id.clone();
        u::choose(g, vec![id]);
    }
    assert!(
        matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Declare{declaration,..}
        if declaration.ability.key=="jc089-combat-glory")
    );
}

#[test]
fn bq035_printed_and_granted_glory_use_real_combat_without_enemy_or_kill() {
    for printed in [false, true] {
        for decline in [false, true] {
            let mut g = u::game();
            if printed {
                field(&mut g, "BQ035", 0, 0);
            } else {
                field(&mut g, "BQ035", 0, 4);
                let host = field(&mut g, "JC029", 0, 0);
                attach(&mut g, "BQ022", 2, &host);
            }
            combat_choice(&mut g, 0);
            u::restore(&mut g);
            u::choose(
                &mut g,
                if decline {
                    vec![]
                } else {
                    vec!["accept".into()]
                },
            );
            if !decline {
                u::top(&mut g);
            }
            assert_eq!(g.regions[0].influence, [u32::from(!decline), 0]);
        }
    }
}

#[test]
fn bq035_many_glory_sources_merge_and_frozen_reward_survives_all_departures() {
    let (mut g, host, knight, _) = granted();
    let equipment = attach(&mut g, "BQ022", 0, &host);
    attach(&mut g, "JC089", 3, &host);
    field(&mut g, "JC018", 1, 0);
    combat_choice(&mut g, 0);
    assert_eq!(g.pending.as_ref().unwrap().seat, 0);
    assert!(!g.effects.iter().any(
        |e| matches!(e,Effect::Declare{declaration}if declaration.ability.key=="jc089-combat-glory")
    ));
    u::choose(&mut g, vec!["accept".into()]);
    g.return_hand(&knight);
    g.remove_dead(&equipment, RemovalCause::Destroy);
    g.return_hand(&host);
    u::restore(&mut g);
    u::top(&mut g);
    assert_eq!(g.regions[0].influence, [1, 0]);
}

#[test]
fn bq035_glory_replacement_fizzles_but_mobility_keeps_its_existing_slot_contract() {
    let mut g = u::game();
    field(&mut g, "BQ035", 0, 0);
    combat_choice(&mut g, 0);
    u::choose(&mut g, vec!["accept".into()]);
    g.regions[0].card = g.make_card("DQJC115", 0);
    u::restore(&mut g);
    u::top(&mut g);
    assert_eq!(g.regions[0].influence, [0, 0]);
    let (mut g, host, _, _) = granted();
    start_mobility(&mut g);
    u::choose(&mut g, vec!["region:1".into()]);
    g.regions[1].card = g.make_card("DQJC114", 0);
    u::restore(&mut g);
    u::top(&mut g);
    assert_eq!(g.board(&host).unwrap().0, 1);
}

#[test]
fn bq035_capture_records_and_wrong_phase_cannot_be_forged_or_replayed() {
    let (mut g, _, _, _) = granted();
    start_mobility(&mut g);
    for variant in 0..5 {
        let mut bad = g.clone();
        match variant {
            0 => bad.turn_ability_usage.clear(),
            1 => bad.turn_ability_usage[0].uses = 2,
            2 => bad
                .turn_ability_usage
                .push(bad.turn_ability_usage[0].clone()),
            3 => bad.window = Some(Window::Action(0)),
            _ => bad.turn += 1,
        }
        invalid(&bad, &format!("capture-record-{variant:02}"));
    }
}

#[test]
fn bq035_impossible_recipient_and_duplicate_live_uid_reject_but_control_change_keeps_actor() {
    let (mut g, host, _, _) = granted();
    start_mobility(&mut g);
    let mut bad = g.clone();
    let d = pending_declaration(&mut bad);
    d.source.card.id = "invented-departed-demon".into();
    d.source.card.definition = "JC046".into();
    d.source
        .observed_mobility_grant
        .as_mut()
        .unwrap()
        .blood
        .host_id = d.source.card.id.clone();
    invalid(&bad, "impossible-demon-vampire-history");
    let mut bad = g.clone();
    let duplicate = bad.board(&host).unwrap().1.clone();
    bad.regions[1].cards.push(duplicate);
    invalid(&bad, "duplicate-live-recipient-uid");
    u::choose(&mut g, vec!["region:1".into()]);
    g.add_control(
        &host,
        2,
        ControlLifetime::TurnEnd { turn: g.turn },
        SubtypeChange::None,
    );
    u::restore(&mut g);
    u::top(&mut g);
    let (r, c) = g.board(&host).unwrap();
    assert_eq!((r, c.controller, c.owner), (1, 2, 0));
}

// Prepared initial layouts; every following transition is a real Room command,
// full journal replay, exact restore and every-seat private projection.
struct Trace {
    name: &'static str,
    room: RoomEnvelope,
    initial: Value,
    steps: Vec<Value>,
}
impl Trace {
    fn new(name: &'static str, mut g: Game) -> Self {
        g.room_id = "350000000000000000000001".into();
        let room = crate::jc029_tests::envelope(&g);
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
            command_id: format!("bq035-{}-{}", self.name, self.steps.len()),
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
            "{} action {:?}: {:?}",
            self.name,
            command.action,
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
            let intent_id = format!("bq035-intent-{}", self.steps.len());
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
        if let Ok(dir) = std::env::var("BQ035_NATIVE_DIR") {
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

#[test]
fn bq035_room_continuous_paid_grants_responses_duel_and_round_boundary() {
    // Real paid knight, blood and equipment, followed by natural stage progression.
    let mut g = u::game();
    let host = field(&mut g, "JC029", 0, 0);
    let knight = u::held(&mut g, "BQ035", 0);
    let blood = u::held(&mut g, "XQ14", 0);
    let weapon = u::held(&mut g, "BQ022", 0);
    u::fund(&mut g, 0, "JC029", 8);
    let mut t = Trace::new("paid-attachments-mobility-glory-next-turn", g);
    t.action(0, t.legal(0, "deploy", &knight, None, Some(2)));
    t.finish();
    t.action(0, t.legal(0, "play", &blood, Some(&host), None));
    t.finish();
    t.action(0, t.legal(0, "play", &weapon, Some(&host), None));
    t.finish();
    assert!(t
        .room
        .game
        .has_mobility(t.room.game.board(&host).unwrap().1));
    t.until(|g|matches!(&g.pending,Some(p)if matches!(&p.resolution,ChoiceResolution::Declare{declaration,..}if declaration.ability.key=="mobility"&&declaration.source.card.id==host)));
    t.command(0, SessionAction::PauseRoom, false);
    t.command(0, SessionAction::ResumeRoom, false);
    let p = t.room.game.pending.clone().unwrap();
    t.command(
        1,
        SessionAction::Game {
            action: Action {
                choice_id: Some(p.choice.id),
                selected: Some(vec!["region:1".into()]),
                ..Action::new("choose")
            },
        },
        true,
    );
    t.choose(vec!["region:1".into()]);
    t.finish();
    assert_eq!(t.room.game.board(&host).unwrap().0, 1);
    t.until(|g|matches!(&g.pending,Some(p)if matches!(&p.resolution,ChoiceResolution::Declare{declaration,..}if declaration.ability.key=="jc089-combat-glory"&&declaration.source.card.id==host)));
    t.choose(vec!["accept".into()]);
    t.finish();
    assert_eq!(t.room.game.regions[1].influence[0], 1);
    let turn = t.room.game.turn;
    t.until(|g| g.turn > turn);
    t.export();

    for queued in [false, true] {
        for blood_response in [false, true] {
            let mut g = u::game();
            let host = field(&mut g, "JC029", 0, if queued { 1 } else { 0 });
            let knight = field(&mut g, "BQ035", 0, 4);
            let blood = attach(&mut g, "XQ14", 0, &host);
            let first = queued.then(|| field(&mut g, "JC014", 0, 0));
            let response = if blood_response { "JC005" } else { "JC006" };
            let spell = u::held(&mut g, response, 2);
            u::fund(&mut g, 2, response, 2);
            g.begin_window(Window::Action(1));
            let name = match (queued, blood_response) {
                (false, false) => "accepted-knight-real-bounce",
                (false, true) => "accepted-blood-real-destroy",
                (true, false) => "queued-knight-real-bounce",
                (true, true) => "queued-blood-real-destroy",
            };
            let mut t = Trace::new(name, g);
            let initial = first.as_ref().unwrap_or(&host).clone();
            t.until(|g|matches!(&g.pending,Some(p)if matches!(&p.resolution,ChoiceResolution::Declare{declaration,..}if declaration.source.card.id==initial)));
            t.choose(vec!["region:1".into()]);
            t.response_priority(2);
            t.action(
                2,
                t.legal(
                    2,
                    "play",
                    &spell,
                    Some(if blood_response { &blood } else { &knight }),
                    None,
                ),
            );
            t.finish();
            if queued {
                assert_eq!(pending_declaration(&mut t.room.game).source.card.id, host);
                t.choose(vec!["region:0".into()]);
                t.finish();
            }
            assert_eq!(
                t.room.game.board(&host).unwrap().0,
                if queued { 0 } else { 1 }
            );
            assert!(!t
                .room
                .game
                .has_mobility(t.room.game.board(&host).unwrap().1));
            t.export();
        }
    }

    for printed in [false, true] {
        let mut g = u::game();
        let knight = field(&mut g, "BQ035", 0, if printed { 0 } else { 4 });
        if !printed {
            let host = field(&mut g, "JC029", 0, 0);
            attach(&mut g, "BQ022", 0, &host);
        }
        let spell = u::held(&mut g, "JC006", 2);
        u::fund(&mut g, 2, "JC006", 2);
        g.begin_window(Window::Before(0, 1));
        let mut t = Trace::new(
            if printed {
                "printed-glory-real-source-bounce"
            } else {
                "granted-glory-real-source-bounce"
            },
            g,
        );
        t.until(|g|matches!(&g.pending,Some(p)if matches!(&p.resolution,ChoiceResolution::Declare{declaration,..}if declaration.ability.key=="jc089-combat-glory")));
        t.choose(vec!["accept".into()]);
        t.response_priority(2);
        t.action(2, t.legal(2, "play", &spell, Some(&knight), None));
        t.finish();
        assert_eq!(t.room.game.regions[0].influence, [1, 0]);
        t.export();
    }

    let mut g = u::game();
    let host = field(&mut g, "JC095", 2, 0);
    attach(&mut g, "JC036", 0, &host);
    field(&mut g, "BQ035", 0, 4);
    attach(&mut g, "BQ040", 0, &host);
    g.regions[1].influence = [0, 2];
    g.begin_window(Window::Action(1));
    let mut t = Trace::new("converted-jc095-keeps-mobility-separate-from-entry", g);
    t.until(|g|matches!(&g.pending,Some(p)if matches!(&p.resolution,ChoiceResolution::Declare{declaration,..}if declaration.source.card.id==host)));
    t.choose(vec!["region:1".into()]);
    t.finish();
    assert_eq!(t.room.game.board(&host).unwrap().0, 1);
    assert_eq!(t.room.game.regions[1].influence, [0, 2]);
    t.export();

    let mut g = u::game();
    let pilot = field(&mut g, "JC061", 0, 0);
    let hidden = field(&mut g, "JC125", 2, 1);
    g.board_mut(&hidden).unwrap().face_down = true;
    u::fund(&mut g, 0, "JC060", 1);
    g.begin_window(Window::Action(1));
    let mut t = Trace::new("printed-jc061-mobility-then-paid-entry", g);
    t.until(|g| g.pending.is_some());
    t.choose(vec!["region:1".into()]);
    t.finish();
    assert!(
        matches!(&t.room.game.pending.as_ref().unwrap().resolution,ChoiceResolution::Declare{declaration,..}if declaration.ability.key=="enter-region-paid-hidden-destroy")
    );
    t.choose(vec![hidden.clone()]);
    t.finish();
    assert!(t.room.game.board(&hidden).is_none());
    assert_eq!(t.room.game.board(&pilot).unwrap().0, 1);
    t.export();

    let mut g = Game::new(
        "bq035-duel".into(),
        "LOCAL".into(),
        "duel".into(),
        "P0".into(),
        "watchers".into(),
        8,
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
    for r in &mut g.regions {
        r.cards.clear();
        r.influence = [0, 0];
    }
    for p in &mut g.players {
        p.hand.clear();
        p.assets.clear();
    }
    let host = field(&mut g, "JC029", 0, 0);
    field(&mut g, "BQ035", 0, 1);
    attach(&mut g, "XQ14", 0, &host);
    g.first_team = 0;
    g.begin_window(Window::Action(1));
    let mut t = Trace::new("duel-grant-can-use-nonadjacent-slot", g);
    t.until(|g| g.pending.is_some());
    assert_eq!(
        t.room
            .game
            .pending
            .as_ref()
            .unwrap()
            .choice
            .options
            .iter()
            .map(|o| o.id.as_str())
            .collect::<Vec<_>>(),
        ["region:1", "region:2"]
    );
    t.choose(vec!["region:2".into()]);
    t.finish();
    assert_eq!(t.room.game.board(&host).unwrap().0, 2);
    t.export();

    let mut g = u::game();
    let held = u::held(&mut g, "BQ035", 0);
    u::fund(&mut g, 0, "JC125", 5);
    let mut t = Trace::new("paid-loyalty-and-extra-payload-rejected", g);
    t.command(
        0,
        SessionAction::Game {
            action: Action {
                card_id: Some(held.clone()),
                region: Some(0),
                ..Action::new("deploy")
            },
        },
        true,
    );
    t.command(
        0,
        SessionAction::Game {
            action: Action {
                card_id: Some(held),
                region: Some(0),
                target_id: Some("fake".into()),
                ..Action::new("deploy")
            },
        },
        true,
    );
    t.export();
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
impl Trace {
    fn choose(&mut self, selected: Vec<String>) {
        let p = self.room.game.pending.clone().expect("pending choice");
        self.action(
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                selected: Some(selected),
                ..Action::new("choose")
            },
        );
    }
    fn advance(&mut self) {
        if let Some(p) = self.room.game.pending.clone() {
            let ids = p
                .choice
                .options
                .iter()
                .map(|o| o.id.clone())
                .collect::<Vec<_>>();
            let action = if matches!(
                &p.resolution,
                ChoiceResolution::Forecast { .. }
                    | ChoiceResolution::Frame {
                        choice: FrameChoice::Forecast { .. },
                        ..
                    }
            ) {
                Action {
                    choice_id: Some(p.choice.id),
                    top: Some(ids),
                    bottom: Some(vec![]),
                    ..Action::new("choose")
                }
            } else if matches!(&p.resolution, ChoiceResolution::Bottom { .. }) {
                Action {
                    choice_id: Some(p.choice.id),
                    bottom: Some(ids),
                    ..Action::new("choose")
                }
            } else if matches!(&p.resolution, ChoiceResolution::Damage { .. }) {
                Action {
                    choice_id: Some(p.choice.id),
                    allocations: Some(
                        [(ids[0].clone(), p.choice.amount.unwrap())]
                            .into_iter()
                            .collect(),
                    ),
                    ..Action::new("choose")
                }
            } else {
                Action {
                    choice_id: Some(p.choice.id),
                    selected: Some(ids.into_iter().take(p.choice.min.unwrap_or(0)).collect()),
                    ..Action::new("choose")
                }
            };
            self.action(p.seat, action);
        } else if let Some(window) = self.room.pacing.window.clone() {
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
        } else {
            let seat = (0..self.room.game.players.len())
                .find(|s| {
                    self.room
                        .game
                        .legal_actions(*s)
                        .iter()
                        .any(|a| a.action.kind == "pass")
                })
                .unwrap();
            self.action(seat, Action::new("pass"));
        }
    }
    fn until(&mut self, predicate: impl Fn(&Game) -> bool) {
        for _ in 0..192 {
            if predicate(&self.room.game) {
                return;
            }
            self.advance();
        }
        panic!("{} did not reach bounded state", self.name);
    }
}
