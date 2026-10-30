//! Shared unit-test helpers: one real four-seat game, explicit layouts, and
//! actions that round-trip through persistence before and after each step.
#![allow(dead_code)]
use crate::{catalog, model::*};

pub(crate) fn game() -> Game {
    let mut g = Game::new(
        "unit".into(),
        "LOCAL".into(),
        "teams".into(),
        "P0".into(),
        "watchers".into(),
        8,
    )
    .unwrap();
    for s in 1..4 {
        g.join(format!("P{s}"), "watchers".into()).unwrap();
    }
    for p in &mut g.players {
        p.ready = true;
    }
    g.apply(0, Action::new("start")).unwrap();
    while g.pending.is_some() {
        choose(&mut g, vec![]);
    }
    for p in &mut g.players {
        p.hand.clear();
        p.assets.clear();
        p.graveyard.clear();
    }
    for r in &mut g.regions {
        r.cards.clear();
        r.influence = [0; 2];
    }
    g.regions[0].card = g.make_card("DQJC115", 0);
    g.first_team = 0;
    g.begin_window(Window::Action(0));
    g
}

/// Current directory scope, separately from the preserved older mechanism
/// fixtures. The entire prior263 raw records retain their pinned digest.
pub(crate) fn assert_current_serial_catalog() {
    use sha2::{Digest,Sha256};
    use serde_json::Value;
    fn sorted(v:&Value)->Value { match v {
        Value::Array(a)=>Value::Array(a.iter().map(sorted).collect()),
        Value::Object(o)=>{let mut keys=o.keys().collect::<Vec<_>>();keys.sort();let mut map=serde_json::Map::new();for k in keys{map.insert(k.clone(),sorted(&o[k]));}Value::Object(map)},
        _=>v.clone(),
    }}
    let raw:Value=serde_json::from_str(include_str!("../data/cards.json")).unwrap();
    let mut current=raw["cards"].as_array().unwrap().clone();
    for c in &mut current {if crate::import20_entropy::BATCH20.contains(&c["id"].as_str().unwrap()){assert_eq!(c["supported"],true);c["supported"]=serde_json::Value::Bool(false);}}
    let cards=&current;
    let digest=format!("{:x}",Sha256::digest(serde_json::to_vec(&sorted(&Value::Array(cards[..263].to_vec()))).unwrap()));
    assert_eq!(format!("{:x}",Sha256::digest(serde_json::to_vec(&sorted(&Value::Array(cards[..267].to_vec()))).unwrap())),"af27bd4495e49cff23c10211bc05975d1a1a0eaaf2c2e5a38e133669b9d8c937");
    assert_eq!(digest,"7f9ec9a7c352d0ce5114a15eae9d0cff6056a0d74a00e1c404d48714944f25a6");
    assert_eq!(cards[252..].iter().map(|c|c["id"].as_str().unwrap()).collect::<Vec<_>>(),["BQ090","BQ064","BQ039","WM028","WM025","WM024","BQ081","WM057","XG13","WM064","CQ24","BQ092","BQ093","CQ18","CQ19","XG20","BQ057","CQ15","CQ28","XQ19","BQ033","BQ034","CQ25","XG25","BQ009","BQ001","XQ35","BQ084","CQ10","CQ09","BQ086","BQ046","BQ055","BQ048","BQ056","BQ050","BQ051","TEST034","BQ063","TK010","BQ052","BQ045","BQ061","BQ049","BQ065","BQ067","BQ060","BQ062","CQ11","TK008","XG19","TK003","BQ088","TK001","CQ27","JZ53","JC097","JC028","JC053","XQ13","WM027","WM036","WM031","BQ044","WM026","ZHWM026","BQ095","WM063","BQ047"]);
    assert_eq!(format!("{:x}",Sha256::digest(serde_json::to_vec(&sorted(&Value::Array(cards[..269].to_vec()))).unwrap())),"d865602732ed70be23cb7679f136d82902ba36d2070435683d23835b42240e6f");
    assert_eq!(cards.len(),321);assert_eq!(catalog::catalog().cards.len(),cards.len());
    assert!(catalog::catalog().cards[..213].iter().all(|c|c.supported));
    assert!(catalog::catalog().cards[213..].iter().all(|c|c.supported==(crate::import20_entropy::BATCH20.contains(&c.id.as_str())||["BQ034","CQ25","XG25"].contains(&c.id.as_str()))));
    assert_eq!(catalog::catalog().cards.iter().filter(|c|c.supported).count(),236);
    assert_eq!(format!("{:x}",Sha256::digest(serde_json::to_vec(&sorted(&Value::Array(cards[..271].to_vec()))).unwrap())),"ded1ecf727d37296800157f503621fe88968bc4d6ec083483d4cef9095d4bbb4");
}
pub(crate) fn field(g: &mut Game, id: &str, seat: usize) -> String {
    let c = if id==crate::tentacle::ID {g.make_generated(crate::generated::GeneratedKind::Tentacle,seat).unwrap()}else{g.make_card(id, seat)};
    let id = c.id.clone();
    g.regions[0].cards.push(c);
    id
}
pub(crate) fn held(g: &mut Game, id: &str, seat: usize) -> String {
    let c = g.make_card(id, seat);
    let id = c.id.clone();
    g.players[seat].hand.push(c);
    id
}
pub(crate) fn fund(g: &mut Game, seat: usize, id: &str, n: usize) {
    for _ in 0..n {
        let c = g.make_card(id, seat);
        g.players[seat].assets.push(c);
    }
}
pub(crate) fn restore(g: &mut Game) {
    let state = serde_json::to_string(g).unwrap();
    let views = (0..4)
        .map(|s| serde_json::to_value(g.view(s)).unwrap())
        .collect::<Vec<_>>();
    *g = Game::from_persisted(&state).unwrap();
    assert_eq!(serde_json::to_string(g).unwrap(), state);
    for s in 0..4 {
        assert_eq!(serde_json::to_value(g.view(s)).unwrap(), views[s]);
    }
}
pub(crate) fn act(g: &mut Game, s: usize, a: Action) {
    restore(g);
    g.apply(s, a).unwrap();
    restore(g);
}
pub(crate) fn reject(g: &mut Game, s: usize, a: Action) {
    let before = serde_json::to_string(g).unwrap();
    assert!(g.apply(s, a).is_err());
    assert_eq!(serde_json::to_string(g).unwrap(), before);
    restore(g);
}
pub(crate) fn choose(g: &mut Game, ids: Vec<String>) {
    let p = g.pending.clone().unwrap();
    act(
        g,
        p.seat,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(ids),
            ..Action::new("choose")
        },
    );
}
pub(crate) fn pass(g: &mut Game) {
    let s = (0..4)
        .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
        .unwrap();
    act(g, s, Action::new("pass"));
}
pub(crate) fn top(g: &mut Game) {
    let n = g.stack.len();
    assert!(n > 0);
    for _ in 0..64 {
        if g.pending.is_some() {
            resolve_choice(g);
        } else if g.stack.len() < n {
            return;
        } else {
            pass(g);
        }
    }
    panic!("bounded stack");
}
pub(crate) fn resolve_choice(g: &mut Game) {
    let p = g.pending.clone().unwrap();
    if matches!(
        p.resolution,
        ChoiceResolution::Forecast { .. }
            | ChoiceResolution::Frame {
                choice: FrameChoice::Forecast { .. },
                ..
            }
    ) {
        let ids = p.choice.options.iter().map(|o| o.id.clone()).collect();
        act(
            g,
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                top: Some(ids),
                bottom: Some(vec![]),
                ..Action::new("choose")
            },
        );
    } else if matches!(p.resolution, ChoiceResolution::Bottom { .. }) {
        let ids = p.choice.options.iter().map(|o| o.id.clone()).collect();
        act(
            g,
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                bottom: Some(ids),
                ..Action::new("choose")
            },
        );
    } else if matches!(p.resolution, ChoiceResolution::Damage { .. }) {
        let mut allocations = std::collections::BTreeMap::new();
        allocations.insert(p.choice.options[0].id.clone(), p.choice.amount.unwrap());
        act(
            g,
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                allocations: Some(allocations),
                ..Action::new("choose")
            },
        );
    } else {
        let ids = p
            .choice
            .options
            .iter()
            .take(p.choice.min.unwrap_or(0))
            .map(|o| o.id.clone())
            .collect();
        choose(g, ids);
    }
}
pub(crate) fn play(g: &mut Game, source: &str, s: usize, target: &str) {
    act(
        g,
        s,
        Action {
            card_id: Some(source.into()),
            target_id: Some(target.into()),
            ..Action::new("play")
        },
    );
    top(g);
}
pub(crate) fn cast(g: &mut Game, id: &str, s: usize, target: &str) {
    let source = held(g, id, s);
    play(g, &source, s, target);
}
pub(crate) fn attach(g: &mut Game, id: &str, target: &str) -> String {
    cast(g, id, 0, target);
    g.attachments.last().unwrap().card.id.clone()
}
pub(crate) fn play_card(g: &mut Game, seat: usize, id: &str) {
    let kind = if g.players[seat]
        .hand
        .iter()
        .find(|c| c.id == id)
        .is_some_and(|c| catalog::card(&c.definition).kind == "character")
    {
        "deploy"
    } else {
        "play"
    };
    act(
        g,
        seat,
        Action {
            card_id: Some(id.into()),
            region: Some(0),
            ..Action::new(kind)
        },
    );
}
pub(crate) fn until_choice(g: &mut Game, kind: &str) {
    for _ in 0..80 {
        if g.pending.as_ref().is_some_and(|p| p.choice.kind == kind) {
            return;
        }
        if g.pending.is_some() {
            resolve_choice(g);
        } else {
            pass(g);
        }
    }
    panic!("missing {kind}");
}
pub(crate) fn play_target(g: &mut Game, id: &str, target: &str) {
    act(
        g,
        0,
        Action {
            card_id: Some(id.into()),
            target_id: Some(target.into()),
            ..Action::new("play")
        },
    );
}
pub(crate) fn resources(g: &Game, s: usize) -> usize {
    g.players[s].assets.iter().filter(|c| !c.exhausted).count()
}
pub(crate) fn drain(g: &mut Game) {
    for _ in 0..100 {
        if g.stack.is_empty() && g.pending.is_none() {
            return;
        }
        if g.pending.is_some() {
            resolve_choice(g);
        } else {
            pass(g);
        }
    }
    panic!("bounded drain");
}
pub(crate) fn kill_source(g: &mut Game, source: &str) {
    fund(g, 0, "JC091", 3);
    let c = held(g, "JC091", 0);
    play_target(g, &c, source);
    until_choice(g, "trigger");
}
pub(crate) fn deck_cards(g: &mut Game, seat: usize, ids: &[&str]) -> Vec<String> {
    g.players[seat].deck.clear();
    let cards = ids
        .iter()
        .map(|id| g.make_card(id, seat))
        .collect::<Vec<_>>();
    let out = cards.iter().map(|c| c.id.clone()).collect();
    g.players[seat].deck = cards;
    out
}
