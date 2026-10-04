//! Deterministic, bounded rules interpreter. No clocks, I/O, or ambient randomness.
use crate::{
    catalog::{self, card},
    model::*,
    rules::{self, Event, StaticModifier},
};
use std::collections::{BTreeMap, BTreeSet};

pub type RuleResult<T> = Result<T, String>;

impl Game {
    pub fn new(
        room_id: String,
        invite_code: String,
        mode: String,
        name: String,
        deck_id: String,
        seed: u64,
    ) -> RuleResult<Self> {
        if catalog::deck(&deck_id).is_none() {
            return Err("未知牌组".into());
        }
        Self::new_lobby(
            room_id,
            invite_code,
            mode,
            Player::new(0, name, deck_id),
            seed,
        )
    }
    pub fn new_with_deck(
        room_id: String,
        invite_code: String,
        mode: String,
        name: String,
        draft: crate::deck::DeckDraft,
        seed: u64,
    ) -> RuleResult<Self> {
        let snapshot = crate::deck::validate(draft)?;
        let mut player = Player::new(0, name, "custom".into());
        player.deck_snapshot = Some(snapshot);
        Self::new_lobby(room_id, invite_code, mode, player, seed)
    }
    fn new_lobby(
        room_id: String,
        invite_code: String,
        mode: String,
        player: Player,
        seed: u64,
    ) -> RuleResult<Self> {
        if mode != "duel" && mode != "teams" {
            return Err("模式必须为 duel 或 teams".into());
        }
        Ok(Self {
            state_schema: 2,
            modifiers: vec![],
            turn_attribute_modifiers: vec![],
            control_effects: vec![],
            control_baselines: vec![],
            room_id,
            invite_code,
            mode,
            version: 0,
            status: "lobby".into(),
            players: vec![player],
            seed,
            random: seed.max(1),
            sequence: 0,
            first_team: 0,
            active_team: 0,
            priority_team: 0,
            turn: 0,
            winner_team: None,
            regions: vec![],
            attachments: vec![],
            region_return: None,
            world: vec![],
            stack: vec![],
            pending: None,
            effects: Default::default(),
            window: None,
            passed: Default::default(),
            team_passed: [false; 2],
            privilege_used: false,
            log: vec![],
            versions: Versions {
                rules: catalog::RULES_VERSION.into(),
                card_pool: catalog::POOL_VERSION.into(),
                engine: catalog::ENGINE_VERSION.into(),
            },
        })
    }
    pub fn team(&self, seat: usize) -> usize {
        if self.mode == "duel" {
            seat
        } else {
            seat / 2
        }
    }
    pub fn capacity(&self) -> usize {
        if self.mode == "duel" {
            2
        } else {
            4
        }
    }
    pub fn win_score(&self) -> u32 {
        if self.mode == "duel" {
            8
        } else {
            10
        }
    }
    pub(crate) fn id(&mut self) -> String {
        self.sequence += 1;
        format!("i{}", self.sequence)
    }
    pub(crate) fn random_below(&mut self, n: usize) -> usize {
        self.random ^= self.random << 13;
        self.random ^= self.random >> 7;
        self.random ^= self.random << 17;
        (self.random % n as u64) as usize
    }
    pub(crate) fn shuffle<T>(&mut self, cards: &mut [T]) {
        for i in (1..cards.len()).rev() {
            let j = self.random_below(i + 1);
            cards.swap(i, j)
        }
    }
    pub(crate) fn fresh(&mut self, mut c: Card) -> Card {
        c.id = self.id();
        c
    }
    pub fn make_card(&mut self, definition: &str, owner: usize) -> Card {
        // Only active definitions may be instantiated, before allocating an identity.
        card(definition);
        Card {
            id: self.id(),
            definition: definition.into(),
            owner,
            controller: owner,
            exhausted: false,
            face_down: false,
            damage: 0,
            wounds: 0,
            shield: 0,
        }
    }
    pub(crate) fn note(&mut self, text: String) {
        self.log.push(LogEntry {
            version: self.version,
            text,
        });
        if self.log.len() > 150 {
            self.log.remove(0);
        }
    }
    pub fn join(&mut self, name: String, deck_id: String) -> RuleResult<usize> {
        if self.status != "lobby" || self.players.len() >= self.capacity() {
            return Err("房间已开始或座位已满".into());
        }
        if catalog::deck(&deck_id).is_none() {
            return Err("未知牌组".into());
        }
        let seat = self.players.len();
        self.players.push(Player::new(seat, name.clone(), deck_id));
        self.version += 1;
        self.note(format!("{name} 加入座位 {}", seat + 1));
        Ok(seat)
    }
    pub fn join_with_deck(
        &mut self,
        name: String,
        draft: crate::deck::DeckDraft,
    ) -> RuleResult<usize> {
        if self.status != "lobby" || self.players.len() >= self.capacity() {
            return Err("房间已开始或座位已满".into());
        }
        let snapshot = crate::deck::validate(draft)?;
        let seat = self.players.len();
        let mut player = Player::new(seat, name.clone(), "custom".into());
        player.deck_snapshot = Some(snapshot);
        self.players.push(player);
        self.version += 1;
        self.note(format!("{name} 加入座位 {}", seat + 1));
        Ok(seat)
    }
    pub(crate) fn can_standard(&self, seat: usize) -> bool {
        self.status == "playing"
            && self.pending.is_none()
            && self.stack.is_empty()
            && self.priority_team == self.team(seat)
            && self.window == Some(Window::Action(self.team(seat)))
            && !self.players[seat].eliminated
    }
    pub(crate) fn can_fast(&self, seat: usize) -> bool {
        self.status == "playing"
            && self.pending.is_none()
            && self.priority_team == self.team(seat)
            && !self.players[seat].eliminated
    }
    pub(crate) fn accessible(&self, seat: usize, region: usize) -> bool {
        region < self.regions.len()
            && (self.mode == "duel"
                || if seat % 2 == 0 {
                    region <= 2
                } else {
                    region >= 2
                })
    }
    pub(crate) fn resources(&self, seat: usize) -> u32 {
        self.players[seat]
            .assets
            .iter()
            .filter(|c| !c.exhausted)
            .count() as u32
    }
    pub(crate) fn loyalty(&self, seat: usize, definition: &str) -> bool {
        let mut available: BTreeMap<String, usize> = BTreeMap::new();
        for c in &self.players[seat].assets {
            let d = card(&c.definition);
            *available.entry(format!("{}色", d.color)).or_default() += 1;
            if !d.magic.is_empty() {
                *available.entry(d.magic.clone()).or_default() += 1;
            }
        }
        let mut needed: BTreeMap<String, usize> = BTreeMap::new();
        for icon in &card(definition).loyalty {
            *needed.entry(icon.clone()).or_default() += 1;
        }
        needed
            .iter()
            .all(|(k, v)| available.get(k).copied().unwrap_or(0) >= *v)
    }
    pub(crate) fn pay(&mut self, seat: usize, amount: u32) -> RuleResult<()> {
        if self.resources(seat) < amount {
            return Err("可用资产不足".into());
        }
        for c in self.players[seat]
            .assets
            .iter_mut()
            .filter(|c| !c.exhausted)
            .take(amount as usize)
        {
            c.exhausted = true;
        }
        Ok(())
    }
    pub(crate) fn board(&self, id: &str) -> Option<(usize, &Card)> {
        self.regions
            .iter()
            .enumerate()
            .find_map(|(i, r)| r.cards.iter().find(|c| c.id == id).map(|c| (i, c)))
            .or_else(|| {
                self.attachments
                    .iter()
                    .find(|a| a.card.id == id)
                    .and_then(|a| {
                        self.regions
                            .iter()
                            .position(|r| r.cards.iter().any(|c| c.id == a.host_id))
                            .map(|r| (r, &a.card))
                    })
            })
    }
    pub(crate) fn board_mut(&mut self, id: &str) -> Option<&mut Card> {
        self.regions
            .iter_mut()
            .find_map(|r| r.cards.iter_mut().find(|c| c.id == id))
            .or_else(|| {
                self.attachments
                    .iter_mut()
                    .find(|a| a.card.id == id)
                    .map(|a| &mut a.card)
            })
    }
    pub(crate) fn remove_board(&mut self, id: &str) -> Option<(usize, Card)> {
        for (i, r) in self.regions.iter_mut().enumerate() {
            if let Some(p) = r.cards.iter().position(|c| c.id == id) {
                return Some((i, r.cards.remove(p)));
            }
        }
        if let Some(index) = self.attachments.iter().position(|a| a.card.id == id) {
            let region = self.board(&self.attachments[index].host_id)?.0;
            return Some((region, self.attachments.remove(index).card));
        }
        None
    }
    pub(crate) fn is_enemy(&self, seat: usize, c: &Card) -> bool {
        self.team(seat) != self.team(c.controller)
    }
    pub(crate) fn targetable(&self, seat: usize, c: &Card) -> bool {
        !self.is_enemy(seat, c) || c.face_down || !rules::definition(&c.definition).traits.barrier
    }
    pub(crate) fn shield_stops(&mut self, actor: usize, target: &str) -> bool {
        if let Some((_, c)) = self.board(target) {
            if self.is_enemy(actor, c) && c.shield > 0 {
                self.board_mut(target).unwrap().shield -= 1;
                self.note("护盾强制移除，终止该来源效果".into());
                return true;
            }
        }
        false
    }
    pub fn icons(&self, c: &Card, region: usize) -> Icons {
        if c.exhausted {
            return Icons::default();
        }
        self.current_icons(c, region)
    }
    /// Current attributes before exhaustion suppresses confrontation participation.
    /// Catalog fields remain the immutable printed values.
    pub fn current_icons(&self, c: &Card, region: usize) -> Icons {
        if c.face_down {
            return Icons {
                influence: 1,
                ..Icons::default()
            };
        }
        let d = card(&c.definition);
        if d.kind != "character" {
            return Icons::default();
        }
        let mut result = d.permanent_icons.add(self.turn_attribute_bonus(c).1);
        if self.team(c.controller) == self.first_team {
            result = result.add(d.temporary_icons);
        }
        for modifier in &rules::definition(&c.definition).modifiers {
            if let StaticModifier::NoEnemyCharacters(permanent, temporary) = modifier {
                if !self.regions[region].cards.iter().any(|other| {
                    !other.face_down
                        && card(&other.definition).kind == "character"
                        && self.is_enemy(c.controller, other)
                }) {
                    result = result.add(*permanent);
                    if self.team(c.controller) == self.first_team {
                        result = result.add(*temporary);
                    }
                }
            }
            if let StaticModifier::ConditionalIcons {
                condition,
                permanent,
                temporary,
            } = modifier
            {
                if self.icon_condition(c, region, condition) {
                    result = result.add(*permanent);
                    if self.team(c.controller) == self.first_team {
                        result = result.add(*temporary);
                    }
                }
            }
        }
        for attachment in self
            .attachments
            .iter()
            .filter(|a| a.host_id == c.id && self.attachment_host_valid(a))
        {
            if let Some(spec) = &rules::definition(&attachment.card.definition).attachment {
                result = result.add(spec.host_icons);
            }
        }
        result
    }
    pub fn defense(&self, c: &Card, region: usize) -> u32 {
        let bonus = self.regions[region]
            .cards
            .iter()
            .filter(|other| {
                !c.face_down
                    && card(&c.definition).kind == "character"
                    && !other.face_down
                    && self.team(other.controller) == self.team(c.controller)
                    && other.id != c.id
            })
            .flat_map(|other| rules::definition(&other.definition).modifiers.iter())
            .map(|m| match m {
                StaticModifier::OtherFriendlyCharactersDefense(n) => *n,
                _ => 0,
            })
            .sum::<u32>();
        (card(&c.definition).defense.unwrap_or(0) + bonus + self.turn_attribute_bonus(c).0)
            .saturating_sub(c.wounds)
    }
    pub(crate) fn reset_passes(&mut self) {
        self.passed.clear();
        self.team_passed = [false; 2];
    }
    pub(crate) fn priority_default(&self) -> usize {
        if let Some(Window::Action(team)) = self.window {
            team
        } else {
            self.first_team
        }
    }
    pub(crate) fn begin_window(&mut self, window: Window) {
        if let Window::Action(team) = window {
            self.active_team = team;
        }
        self.window = Some(window);
        self.priority_team = self.priority_default();
        self.reset_passes();
    }
    pub(crate) fn push_stack(
        &mut self,
        seat: usize,
        label: String,
        c: Option<Card>,
        region: Option<usize>,
        reveal: bool,
        target: Option<String>,
    ) {
        let id = self.id();
        self.stack.push(StackItem {
            id,
            label,
            controller: seat,
            card: c,
            deploy_region: region,
            reveal,
            target,
            frame: None,
        });
        self.reset_passes();
    }
    pub fn apply(&mut self, seat: usize, action: Action) -> RuleResult<()> {
        self.apply_at_revision(seat, action, self.version + 1)
    }
    pub(crate) fn apply_at_revision(
        &mut self,
        seat: usize,
        action: Action,
        revision: u64,
    ) -> RuleResult<()> {
        if seat >= self.players.len() {
            return Err("无此座位".into());
        }
        // Work on a clone: every rejected command is a strict no-op, including PRNG/ID counters.
        let mut next = self.clone();
        next.version = revision;
        next.apply_inner(seat, action)?;
        next.settle_deaths();
        next.drive()?;
        *self = next;
        Ok(())
    }
    pub(crate) fn apply_inner(&mut self, seat: usize, a: Action) -> RuleResult<()> {
        if self.pending.is_some() {
            if a.kind != "choose" {
                return Err("等待玩家完成选择".into());
            }
            return self.choose(seat, a);
        }
        if self.status == "lobby" || (self.status == "finished" && a.kind == "restart") {
            match a.kind.as_str() {
                "ready" if self.status == "lobby" => {
                    self.players[seat].ready = !self.players[seat].ready;
                    return Ok(());
                }
                "deck" if self.status == "lobby" => {
                    if let Some(draft) = a.deck_draft {
                        let snapshot = crate::deck::validate(draft)?;
                        self.players[seat].deck_id = "custom".into();
                        self.players[seat].deck_snapshot = Some(snapshot);
                    } else {
                        let id = a.option.ok_or("请选择牌组")?;
                        crate::deck::preset(&id)?;
                        self.players[seat].deck_id = id;
                        self.players[seat].deck_snapshot = None;
                    }
                    self.players[seat].ready = false;
                    return Ok(());
                }
                "start" | "restart" if seat == 0 => {
                    if self.players.len() != self.capacity()
                        || self.players.iter().any(|p| !p.ready)
                    {
                        return Err("需要全部座位到齐并准备".into());
                    }
                    return self.start();
                }
                _ => return Err("该大厅操作不可用".into()),
            }
        }
        if !self.can_fast(seat) {
            return Err("当前没有行动权".into());
        }
        if a.kind == "pass" {
            return self.pass(seat);
        }
        let standard = self.can_standard(seat);
        match a.kind.as_str() {
            "asset" if standard => {
                if self.players[seat].asset_used {
                    return Err("本回合已建立资产".into());
                }
                let mut c = self.remove_hand(seat, a.card_id.as_deref().ok_or("请选择手牌")?)?;
                c = self.fresh(c);
                self.players[seat].assets.push(c);
                self.players[seat].asset_used = true;
                self.note(format!("{} 建立资产", self.players[seat].name));
                self.reset_passes();
            }
            "deploy" | "conceal" => {
                if !standard {
                    return Err("只可在己方行动步骤派遣".into());
                }
                let id = a.card_id.as_deref().ok_or("请选择角色")?;
                let r = a.region.ok_or("请选择地区")?;
                if !self.accessible(seat, r) {
                    return Err("超出正常派遣距离".into());
                }
                let (c, play_source) = self.character_play_source(seat, id, a.kind == "conceal")?;
                let d = card(&c.definition);
                if d.kind != "character" {
                    return Err("只有角色可派遣".into());
                }
                if a.kind == "conceal" {
                    if self.players[seat].conceal_used
                        || rules::definition(&c.definition).traits.public
                    {
                        return Err("该角色不能秘密派遣或本回合已用".into());
                    }
                    self.pay(seat, 1)?;
                    let mut c = self.remove_hand(seat, id)?;
                    c = self.fresh(c);
                    c.face_down = true;
                    self.regions[r].cards.push(c);
                    self.players[seat].conceal_used = true;
                    self.note(format!(
                        "{} 秘密派遣至地区 {}",
                        self.players[seat].name,
                        r + 1
                    ));
                    self.reset_passes();
                } else {
                    if !self.loyalty(seat, &c.definition) {
                        return Err("忠诚不足".into());
                    }
                    let paid = self.pay_printed(seat, &c, true)?;
                    let mut snapshot = self.source_snapshot(&c, Some(r));
                    snapshot.play_source = Some(play_source);
                    let c = self.take_character_play_source(seat, id, play_source)?;
                    let c = self.fresh(c);
                    self.note(format!("{} 打出 {}", self.players[seat].name, d.name));
                    self.push_stack(
                        seat,
                        format!("派遣 {}", d.name),
                        Some(c),
                        Some(r),
                        false,
                        None,
                    );
                    self.stack.last_mut().unwrap().frame = Some(ResolutionFrame {
                        frame_id: self.stack.last().unwrap().id.clone(),
                        ability_key: "deploy".into(),
                        actor: seat,
                        source: snapshot,
                        targets: vec![],
                        already_paid: paid,
                        guard: GuardState::Unchecked,
                        cursor: 0,
                        steps: vec![],
                        chosen_region: Some(r),
                    });
                }
            }
            "reveal" => {
                let id = a.card_id.as_deref().ok_or("请选择暗藏者")?;
                let (r, c) = self.board(id).ok_or("实体引用已失效")?;
                if c.controller != seat || !c.face_down {
                    return Err("只能现身自己的暗藏者".into());
                }
                let d = card(&c.definition);
                if !self.loyalty(seat, &c.definition) {
                    return Err("忠诚不足".into());
                }
                let snapshot = self.source_snapshot(c, Some(r));
                let printed = c.clone();
                let paid = self.pay_printed(seat, &printed, false)?;
                let (_, mut c) = self.remove_board(id).unwrap();
                c = self.fresh(c);
                c.face_down = false;
                self.note(format!("{} 宣告现身 {}", self.players[seat].name, d.name));
                self.push_stack(
                    seat,
                    format!("现身 {}", d.name),
                    Some(c),
                    Some(r),
                    true,
                    None,
                );
                self.stack.last_mut().unwrap().frame = Some(ResolutionFrame {
                    frame_id: self.stack.last().unwrap().id.clone(),
                    ability_key: "reveal".into(),
                    actor: seat,
                    source: snapshot,
                    targets: vec![],
                    already_paid: paid,
                    guard: GuardState::Unchecked,
                    cursor: 0,
                    steps: vec![],
                    chosen_region: Some(r),
                });
            }
            "play" => {
                let id = a.card_id.as_deref().ok_or("请选择事务或附属")?;
                let c = self.players[seat]
                    .hand
                    .iter()
                    .find(|c| c.id == id)
                    .ok_or("手牌引用已失效")?
                    .clone();
                let d = card(&c.definition);
                if d.kind != "spell" && d.kind != "attachment" {
                    return Err("该牌不是事务或附属".into());
                }
                let spec = self.ability_for_action(&c.definition, &a)?;
                self.check_timing(seat, &spec)?;
                if !self.loyalty(seat, &c.definition) {
                    return Err("忠诚不足".into());
                }
                let source = self.source_snapshot(&c, None);
                let targets = self.bind_action(seat, &source, &spec, &a)?;
                if spec.ops.iter().any(|op| {
                    matches!(
                        op,
                        rules::Op::Move(_, rules::Destination::HiddenInChosenRegion)
                    )
                }) && a.region.is_none_or(|r| r >= self.regions.len())
                {
                    return Err("请选择合法地区".into());
                }
                let mut paid = self.pay_printed(seat, &c, true)?;
                paid.extend(self.pay_ability_costs(seat, &source, &spec, &a)?);
                let c = self.remove_hand(seat, id)?;
                let c = self.fresh(c);
                let frame = self.make_frame(seat, source, &spec, targets, paid, a.region);
                self.note(format!("{} 打出 {}", self.players[seat].name, d.name));
                self.dispatch_frame(frame, d.name.clone(), Some(c), spec.response_policy);
            }
            "activate" => {
                let id = a.card_id.as_deref().ok_or("请选择能力来源")?;
                let (region, c) = self.ability_source(id).ok_or("实体引用已失效")?;
                if c.controller != seat || c.face_down {
                    return Err("只能发动自己操控的正面角色能力".into());
                }
                let source = self.source_snapshot(c, region);
                let spec = self.ability_for_action(&c.definition, &a)?;
                self.check_timing(seat, &spec)?;
                let targets = self.bind_action(seat, &source, &spec, &a)?;
                let paid = self.pay_ability_costs(seat, &source, &spec, &a)?;
                let label = format!("{}：{}", card(&source.card.definition).name, spec.label);
                let frame = self.make_frame(seat, source, &spec, targets, paid, a.region);
                self.note(format!("{} 发动 {}", self.players[seat].name, label));
                self.dispatch_frame(frame, label, None, spec.response_policy);
            }
            "privilege" => return self.privilege(seat),
            _ => return Err("当前操作不合法".into()),
        }
        Ok(())
    }
    pub(crate) fn remove_hand(&mut self, seat: usize, id: &str) -> RuleResult<Card> {
        let p = self.players[seat]
            .hand
            .iter()
            .position(|c| c.id == id)
            .ok_or("手牌实体已失效")?;
        Ok(self.players[seat].hand.remove(p))
    }
    pub(crate) fn start(&mut self) -> RuleResult<()> {
        self.status = "playing".into();
        self.modifiers.clear();
        self.turn_attribute_modifiers.clear();
        self.regions.clear();
        self.attachments.clear();
        self.region_return = None;
        self.world.clear();
        self.stack.clear();
        self.pending = None;
        self.effects.clear();
        self.winner_team = None;
        self.turn = 0;
        for seat in 0..self.players.len() {
            self.players[seat].hand.clear();
            self.players[seat].deck.clear();
            self.players[seat].assets.clear();
            self.players[seat].graveyard.clear();
            self.players[seat].score_cards.clear();
            self.players[seat].society_zone = Default::default();
            self.players[seat].eliminated = false;
            let snapshot = match self.players[seat].deck_snapshot.clone() {
                Some(snapshot) => crate::deck::validate(snapshot)?,
                None => crate::deck::preset(&self.players[seat].deck_id)?,
            };
            let entries = snapshot.cards.clone();
            if let Some(id) = &snapshot.society_id {
                let mut entity = self.make_card(id, seat);
                entity.face_down = true;
                self.players[seat].society_zone.card = Some(entity);
            }
            self.players[seat].deck_snapshot = Some(snapshot);
            let mut pile = vec![];
            for entry in entries {
                for _ in 0..entry.count {
                    pile.push(self.make_card(&entry.card_id, seat));
                }
            }
            self.shuffle(&mut pile);
            self.players[seat].deck = pile;
        }
        // Start is one atomic command: no seat can observe a partially revealed set.
        for player in &mut self.players {
            if let Some(entity) = &mut player.society_zone.card {
                entity.face_down = false;
            }
        }
        let mut world = vec![];
        for entry in &catalog::catalog().world {
            for _ in 0..entry.count {
                world.push(self.make_card(&entry.card_id, 0));
            }
        }
        self.shuffle(&mut world);
        let count = if self.mode == "duel" { 3 } else { 5 };
        for _ in 0..count {
            self.regions.push(Region {
                card: world.remove(0),
                influence: [0; 2],
                cards: vec![],
                skip: false,
            })
        }
        self.world = world;
        self.first_team = self.random_below(2);
        self.turn = 1;
        self.privilege_used = false;
        for seat in 0..self.players.len() {
            self.players[seat].asset_used = false;
            self.players[seat].conceal_used = false;
            let starting_hand = self.players[seat]
                .deck_snapshot
                .as_ref()
                .and_then(|d| d.society_id.as_deref())
                .map(crate::society::definition)
                .transpose()?
                .map_or(6, |s| s.starting_hand);
            self.draw(seat, starting_hand)?;
            self.effects.push_back(Effect::Mulligan { seat });
        }
        self.begin_window(Window::Prepare);
        self.note(format!(
            "开始 {}；团队 {} 持先手",
            if self.mode == "duel" {
                "两人对决"
            } else {
                "四人协作"
            },
            self.first_team + 1
        ));
        Ok(())
    }
    pub(crate) fn draw(&mut self, seat: usize, count: usize) -> RuleResult<()> {
        if self.players[seat].eliminated {
            return Ok(());
        }
        for _ in 0..count {
            if self.players[seat].deck.is_empty() {
                self.eliminate(seat);
                break;
            }
            let c = self.players[seat].deck.remove(0);
            let c = self.fresh(c);
            self.players[seat].hand.push(c);
        }
        Ok(())
    }
    pub(crate) fn eliminate(&mut self, seat: usize) {
        self.players[seat].eliminated = true;
        self.players[seat].hand.clear();
        self.players[seat].deck.clear();
        self.players[seat].assets.clear();
        self.players[seat].graveyard.clear();
        let leaving = self
            .regions
            .iter()
            .flat_map(|r| r.cards.iter())
            .filter(|c| c.owner == seat)
            .map(|c| c.id.clone())
            .collect::<Vec<_>>();
        for id in leaving {
            self.host_leaves(&id);
        }
        self.attachments.retain(|a| a.card.owner != seat);
        for r in &mut self.regions {
            r.cards.retain(|c| c.owner != seat);
        }
        self.stack
            .retain(|s| s.controller != seat && s.card.as_ref().is_none_or(|c| c.owner != seat));
        self.effects.retain(|e| match e {
            Effect::Draw { seat: s, .. }
            | Effect::Mulligan { seat: s }
            | Effect::Discard { seat: s, .. }
            | Effect::Forecast { seat: s, .. }
            | Effect::Damage { seat: s, .. }
            | Effect::Unique { seat: s } => *s != seat,
            Effect::Declare { declaration } => declaration.actor != seat,
            Effect::Frame { frame } => frame.actor != seat,
            Effect::Bury { card } => card.owner != seat,
            _ => true,
        });
        self.note(format!("{} 牌库耗尽，退出游戏", self.players[seat].name));
        self.settle_controls();
        let team = self.team(seat);
        if self.living(team).is_empty() {
            self.finish(1 - team);
        }
    }
    pub(crate) fn living(&self, team: usize) -> Vec<usize> {
        self.players
            .iter()
            .filter(|p| !p.eliminated && self.team(p.seat) == team)
            .map(|p| p.seat)
            .collect()
    }
    pub(crate) fn finish(&mut self, team: usize) {
        self.status = "finished".into();
        self.winner_team = Some(team);
        self.pending = None;
        self.effects.clear();
        self.stack.clear();
        self.note(format!("团队 {} 获胜", team + 1));
    }
    pub(crate) fn check_win(&mut self) {
        let mut scores = [0; 2];
        for p in &self.players {
            scores[self.team(p.seat)] += p
                .score_cards
                .iter()
                .map(|c| card(&c.definition).points.unwrap_or(0))
                .sum::<u32>();
        }
        if scores[0] >= self.win_score() || scores[1] >= self.win_score() {
            if scores[0] > scores[1] {
                self.finish(0)
            } else if scores[1] > scores[0] {
                self.finish(1)
            }
        }
    }
    pub(crate) fn pass(&mut self, seat: usize) -> RuleResult<()> {
        if self.passed.contains(&seat) {
            return Err("本轮已让过，等待队友".into());
        }
        let team = self.team(seat);
        self.passed.insert(seat);
        if self.living(team).iter().all(|s| self.passed.contains(s)) {
            self.team_passed[team] = true;
            self.priority_team = 1 - team;
            if self.team_passed == [true; 2] {
                self.reset_passes();
                if let Some(item) = self.stack.pop() {
                    self.resolve_stack(item)?;
                    self.priority_team = self.priority_default();
                } else {
                    self.close_window()?;
                }
            }
        }
        Ok(())
    }
    pub(crate) fn resolve_stack(&mut self, mut item: StackItem) -> RuleResult<()> {
        self.note(format!("结算：{}", item.label));
        let mut frame = item.frame.take().ok_or("v2堆叠对象缺少已支付的通用frame")?;
        if let Some(c) = item.card.take() {
            if let Some(region) = item.deploy_region {
                let mut c = self.fresh(c);
                c.face_down = false;
                let id = c.id.clone();
                let definition = c.definition.clone();
                self.regions[region].cards.push(c);
                self.enter_triggers(item.controller, &definition, &id, item.reveal);
            } else if card(&c.definition).kind == "attachment" {
                if self.accept_frame_guard(&mut frame) {
                    let host_id = frame.targets.first().ok_or("附属缺少宿主目标")?.id.clone();
                    let mut c = self.fresh(c);
                    c.face_down = false;
                    c.controller = item.controller;
                    let id = c.id.clone();
                    let definition = c.definition.clone();
                    self.attachments.push(Attachment {
                        card: c,
                        host_id: host_id.clone(),
                    });
                    if let Some(spec) = rules::definition(&definition)
                        .attachment
                        .as_ref()
                        .filter(|s| s.controls_host)
                    {
                        self.add_control(
                            &host_id,
                            item.controller,
                            ControlLifetime::Attached {
                                source_instance: id.clone(),
                            },
                            spec.host_subtype_change.clone(),
                        );
                    }
                    self.enter_triggers(item.controller, &definition, &id, false);
                } else {
                    self.effects.push_front(Effect::Bury { card: c });
                }
            } else {
                // Transactions enter the graveyard after executing their effect.
                self.effects.push_front(Effect::Bury { card: c });
            }
        }
        self.effects.push_front(Effect::Frame {
            frame: Box::new(frame),
        });
        Ok(())
    }
    pub(crate) fn enter_triggers(&mut self, seat: usize, definition: &str, id: &str, reveal: bool) {
        if let Some((r, c)) = self.board(id) {
            let source = self.source_snapshot(c, Some(r));
            self.emit_event(seat, source.clone(), Event::Enter);
            self.emit_event(seat, source.clone(), Event::EnterRegion);
            if reveal {
                self.emit_event(seat, source, Event::Reveal);
            }
        }
        if card(definition).unique {
            self.effects.push_front(Effect::Unique { seat });
        }
    }
    pub(crate) fn close_window(&mut self) -> RuleResult<()> {
        match self.window.clone().ok_or("缺少步骤")? {
            Window::Prepare => {
                for seat in 0..self.players.len() {
                    self.effects.push_back(Effect::Draw { seat, count: 1 });
                }
                self.begin_window(Window::Draw);
            }
            Window::Draw => self.begin_window(Window::Action(self.first_team)),
            Window::Action(team) if team == self.first_team => {
                self.begin_window(Window::Action(1 - self.first_team))
            }
            Window::Action(_) => {
                self.begin_window(Window::Mobility);
                let mut sources = self
                    .regions
                    .iter()
                    .enumerate()
                    .flat_map(|(r, reg)| {
                        reg.cards
                            .iter()
                            .filter(|c| {
                                !c.face_down
                                    && !c.exhausted
                                    && rules::definition(&c.definition)
                                        .abilities
                                        .iter()
                                        .any(|a| a.event == Some(Event::ConfrontationStart))
                            })
                            .map(move |c| {
                                (
                                    c.controller,
                                    SourceSnapshot {
                                        card: c.clone(),
                                        region: Some(r),
                                        play_source: None,
                                    },
                                )
                            })
                    })
                    .collect::<Vec<_>>();
                sources.sort_by_key(|(seat, _)| (self.team(*seat) != self.first_team, *seat));
                for (seat, source) in sources {
                    self.emit_event(seat, source, Event::ConfrontationStart);
                }
            }
            Window::Mobility => self.begin_window(Window::Before(0, 0)),
            Window::Before(region, contest) => {
                self.begin_window(Window::After(region, contest));
                if !self.regions[region].skip {
                    let counts = self.contest_counts(region, contest);
                    if counts[0] != counts[1] {
                        let winner = if counts[0] > counts[1] { 0 } else { 1 };
                        self.reward(winner, region, contest, counts[0].abs_diff(counts[1]))?;
                    }
                }
            }
            Window::After(region, contest) => {
                if contest < 2 {
                    self.begin_window(Window::Before(region, contest + 1));
                } else if region + 1 < self.regions.len() {
                    self.begin_window(Window::Before(region + 1, 0));
                } else {
                    self.begin_window(Window::End);
                }
            }
            Window::Win(region, seat) => {
                self.effects
                    .push_back(Effect::PrepareRegionReturn { region });
                for owner in 0..self.players.len() {
                    self.effects.push_back(Effect::Bottom {
                        seat: owner,
                        region,
                    });
                }
                self.effects
                    .push_back(Effect::CommitRegionReturn { region });
                self.effects.push_back(Effect::Score { seat, region });
                self.begin_window(Window::After(region, 2));
            }
            Window::End => {
                self.effects.push_back(Effect::Cleanup);
            }
        }
        Ok(())
    }
    pub(crate) fn contest_counts(&self, region: usize, contest: usize) -> [u32; 2] {
        let mut counts = [0; 2];
        for c in &self.regions[region].cards {
            counts[self.team(c.controller)] += self.icons(c, region).at(contest);
        }
        counts
    }
    pub(crate) fn contributors(&self, team: usize, region: usize, contest: usize) -> Vec<usize> {
        self.living(team)
            .into_iter()
            .filter(|s| {
                self.regions[region]
                    .cards
                    .iter()
                    .any(|c| c.controller == *s && self.icons(c, region).at(contest) > 0)
            })
            .collect()
    }
    pub(crate) fn reward(
        &mut self,
        team: usize,
        region: usize,
        contest: usize,
        amount: u32,
    ) -> RuleResult<()> {
        self.note(format!(
            "地区 {} {}：团队 {} 差值 {}",
            region + 1,
            ["调查", "战斗", "势力"][contest],
            team + 1,
            amount
        ));
        let mut seats = self.contributors(team, region, contest);
        if contest == 2 && seats.is_empty() {
            seats = self.living(team);
        }
        if seats.is_empty() {
            return Ok(());
        }
        if contest == 1 {
            let kills = self.regions[region]
                .cards
                .iter()
                .filter(|c| {
                    self.team(c.controller) == team
                        && !c.face_down
                        && !c.exhausted
                        && rules::definition(&c.definition).traits.kill > 0
                })
                .map(|c| rules::definition(&c.definition).traits.kill)
                .sum::<u32>();
            let total = amount + kills;
            if seats.len() > 1 {
                self.effects.push_back(Effect::Recipient {
                    team,
                    region,
                    contest,
                    amount: total,
                    seats,
                });
            } else {
                self.effects.push_back(Effect::Damage {
                    seat: seats[0],
                    region,
                    amount: total,
                });
            }
        } else if contest == 0 {
            if seats.len() > 1 {
                self.effects.push_back(Effect::Recipient {
                    team,
                    region,
                    contest,
                    amount,
                    seats,
                });
            } else {
                self.effects.push_back(Effect::Forecast {
                    seat: seats[0],
                    amount: amount as usize,
                    draw: true,
                });
            }
        } else {
            let enemy = 1 - team;
            let removed = amount.min(self.regions[region].influence[enemy]);
            self.regions[region].influence[enemy] -= removed;
            self.regions[region].influence[team] += amount - removed;
            if self.regions[region].influence[team]
                >= card(&self.regions[region].card.definition)
                    .threshold
                    .unwrap_or(3)
            {
                self.effects.push_back(Effect::Award {
                    seat: seats[0],
                    region,
                });
            }
        }
        Ok(())
    }
    pub(crate) fn privilege(&mut self, seat: usize) -> RuleResult<()> {
        let Some(Window::Before(region, contest)) = self.window else {
            return Err("仅可在对抗前发动先手特权".into());
        };
        let counts = self.contest_counts(region, contest);
        if !self.stack.is_empty()
            || self.privilege_used
            || self.team(seat) != self.first_team
            || counts[0] != counts[1]
            || self.regions[region].skip
            || !self.regions[region]
                .cards
                .iter()
                .any(|c| c.controller == seat && self.icons(c, region).at(contest) > 0)
        {
            return Err("先手特权条件不足".into());
        }
        self.pay(seat, 1)?;
        self.privilege_used = true;
        self.begin_window(Window::After(region, contest));
        self.reward(self.first_team, region, contest, 1)?;
        Ok(())
    }
    pub(crate) fn choice(
        &mut self,
        seat: usize,
        kind: &str,
        title: String,
        options: Vec<ChoiceOption>,
        min: usize,
        max: usize,
        amount: Option<u32>,
        resolution: ChoiceResolution,
    ) {
        let id = self.id();
        self.pending = Some(Pending {
            seat,
            resolution,
            choice: Choice {
                id,
                kind: kind.into(),
                title,
                description: if kind == "damage" {
                    "分配全部伤害；护卫角色须优先承受其护卫数值。".into()
                } else if kind == "investigation" || kind == "order" {
                    "将所有选项按所选顺序分到牌库顶和底。".into()
                } else {
                    "选择符合数量限制的选项，然后确认。".into()
                },
                player_id: player_id(seat),
                options,
                min: Some(min),
                max: Some(max),
                amount,
                allow_decline: Some(min == 0 && kind != "damage"),
            },
        });
    }
    pub(crate) fn option(
        &self,
        c: &Card,
        viewer: usize,
        region: Option<usize>,
        kind: Option<&str>,
    ) -> ChoiceOption {
        let projected = self.card_view(c, viewer, region, kind);
        ChoiceOption {
            id: c.id.clone(),
            label: projected.name.clone(),
            card: Some(projected),
        }
    }
    pub(crate) fn drive(&mut self) -> RuleResult<()> {
        let mut processed = 0;
        while self.pending.is_none() && self.status == "playing" {
            // Mobility has a specific first-team-before-rear-team order. Do not
            // declare the next move until the current responsive stack resolves.
            if !self.stack.is_empty()
                && matches!(self.effects.front(),Some(Effect::Declare{declaration}) if declaration.ability.event==Some(Event::ConfrontationStart))
            {
                break;
            }
            let Some(effect) = self.effects.pop_front() else {
                break;
            };
            processed += 1;
            if processed > 1000 {
                return Err("效果步骤超过有界上限".into());
            }
            self.effect(effect)?;
        }
        Ok(())
    }
    pub(crate) fn effect(&mut self, e: Effect) -> RuleResult<()> {
        match e {
            Effect::Declare { declaration } => self.declare_trigger(declaration)?,
            Effect::Frame { frame } => self.resolve_frame(*frame)?,
            Effect::Draw { seat, count } => self.draw(seat, count)?,
            Effect::Mulligan { seat } => {
                let options = self.players[seat]
                    .hand
                    .iter()
                    .map(|c| self.option(c, seat, None, None))
                    .collect();
                let max = self.players[seat].hand.len();
                self.choice(
                    seat,
                    "mulligan",
                    "一次再调度：暂放任意手牌，抓等量，再洗回原牌".into(),
                    options,
                    0,
                    max,
                    None,
                    ChoiceResolution::Mulligan,
                );
            }
            Effect::Discard {
                seat,
                amount,
                redraw,
                optional,
            } => {
                if !self.players[seat].eliminated && !self.players[seat].hand.is_empty() {
                    let options = self.players[seat]
                        .hand
                        .iter()
                        .map(|c| self.option(c, seat, None, None))
                        .collect();
                    let count = amount.min(self.players[seat].hand.len());
                    self.choice(
                        seat,
                        "discard",
                        if redraw {
                            "弃任意手牌并抓等量".into()
                        } else {
                            format!("弃 {} 张手牌", count)
                        },
                        options,
                        if optional { 0 } else { count },
                        count,
                        None,
                        ChoiceResolution::Discard { redraw },
                    );
                }
            }
            Effect::Forecast { seat, amount, draw } => {
                if self.players[seat].eliminated {
                    return Ok(());
                }
                let options: Vec<_> = self.players[seat]
                    .deck
                    .iter()
                    .take(amount)
                    .map(|c| self.option(c, seat, None, None))
                    .collect();
                if options.is_empty() {
                    if draw {
                        self.draw(seat, 1)?
                    }
                } else {
                    let n = options.len();
                    self.choice(
                        seat,
                        "investigation",
                        format!("预测 {}{}", n, if draw { "，然后抓一张牌" } else { "" }),
                        options,
                        n,
                        n,
                        Some(n as u32),
                        ChoiceResolution::Forecast { draw },
                    );
                }
            }
            Effect::Damage {
                seat,
                region,
                amount,
            } => {
                let options: Vec<_> = self.regions[region]
                    .cards
                    .iter()
                    .filter(|c| !c.face_down && self.is_enemy(seat, c))
                    .map(|c| self.option(c, seat, Some(region), None))
                    .collect();
                if !options.is_empty() && amount > 0 {
                    let target_count = options.len();
                    self.choice(
                        seat,
                        "damage",
                        format!("地区 {}：分配 {} 点同时战斗伤害", region + 1, amount),
                        options,
                        0,
                        target_count,
                        Some(amount),
                        ChoiceResolution::Damage { region },
                    );
                }
            }
            Effect::Recipient {
                team: _,
                region,
                contest,
                amount,
                seats,
            } => {
                let chooser = seats[0];
                let options = seats
                    .into_iter()
                    .map(|s| ChoiceOption {
                        id: player_id(s),
                        label: self.players[s].name.clone(),
                        card: None,
                    })
                    .collect();
                self.choice(
                    chooser,
                    "recipient",
                    "选择本次实际参战的奖励执行者".into(),
                    options,
                    1,
                    1,
                    Some(amount),
                    ChoiceResolution::Recipient {
                        region,
                        contest,
                        amount,
                    },
                );
            }
            Effect::Award { seat, region } => {
                self.note(format!(
                    "团队 {} 达到地区阈值，赢区前快速行动窗口",
                    self.team(seat) + 1
                ));
                self.begin_window(Window::Win(region, seat));
            }
            Effect::PrepareRegionReturn { region } => self.prepare_region_return(region)?,
            Effect::CommitRegionReturn { region } => self.commit_region_return(region)?,
            Effect::Bottom { seat, region } => {
                if self
                    .region_return
                    .as_ref()
                    .is_some_and(|batch| batch.region == region)
                {
                    let ids = self.region_return.as_ref().unwrap().bottom[seat].clone();
                    let options = ids
                        .iter()
                        .map(|id| {
                            self.board(id)
                                .map(|(_, c)| self.option(c, seat, Some(region), None))
                                .ok_or("赢区回底对象已失效")
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    if options.len() <= 1 {
                        self.region_return.as_mut().unwrap().orders[seat] = Some(ids);
                    } else {
                        let n = options.len();
                        self.choice(
                            seat,
                            "region_return",
                            "排列自己的回底牌；所有人确认后一起移区".into(),
                            options,
                            n,
                            n,
                            None,
                            ChoiceResolution::Bottom { region },
                        );
                    }
                    return Ok(());
                }
                let retreat: Vec<_> = self.regions[region]
                    .cards
                    .iter()
                    .filter(|c| {
                        c.owner == seat
                            && !c.face_down
                            && rules::definition(&c.definition).traits.retreat
                    })
                    .map(|c| c.id.clone())
                    .collect();
                for id in retreat {
                    self.return_hand(&id);
                }
                let options: Vec<_> = self.regions[region]
                    .cards
                    .iter()
                    .filter(|c| c.owner == seat)
                    .map(|c| self.option(c, seat, Some(region), None))
                    .collect();
                if options.len() == 1 {
                    let id = options[0].id.clone();
                    self.to_bottom(&id);
                } else if !options.is_empty() {
                    let n = options.len();
                    self.choice(
                        seat,
                        "region_return",
                        "按自己的顺序将赢区中的牌放到牌库底".into(),
                        options,
                        n,
                        n,
                        None,
                        ChoiceResolution::Bottom { region },
                    );
                }
            }
            Effect::Score { seat, region } => {
                let definition = self.regions[region].card.definition.clone();
                let mut scored = self.regions[region].card.clone();
                scored.owner = seat;
                scored.controller = seat;
                scored = self.fresh(scored);
                self.players[seat].score_cards.push(scored);
                self.note(format!(
                    "{} 赢得 {}（{}分）",
                    self.players[seat].name,
                    card(&definition).name,
                    card(&definition).points.unwrap_or(0)
                ));
                if !self.world.is_empty() {
                    let c = self.world.remove(0);
                    let c = self.fresh(c);
                    self.regions[region] = Region {
                        card: c,
                        influence: [0; 2],
                        cards: vec![],
                        skip: false,
                    };
                }
                self.check_win();
                if self.status == "playing" {
                    let source = self.source_snapshot(
                        self.players[seat].score_cards.last().unwrap(),
                        Some(region),
                    );
                    self.emit_event(seat, source, Event::RegionWon);
                }
            }
            Effect::Unique { seat } => {
                let mut options = vec![];
                let mut group = None;
                for (r, region) in self.regions.iter().enumerate() {
                    for c in &region.cards {
                        if c.controller == seat && !c.face_down && card(&c.definition).unique {
                            let name = &card(&c.definition).name;
                            let count = self
                                .regions
                                .iter()
                                .flat_map(|r| r.cards.iter())
                                .filter(|other| {
                                    other.controller == seat
                                        && !other.face_down
                                        && card(&other.definition).name == *name
                                })
                                .count();
                            if count > 1 && group.as_ref().is_none_or(|n| n == name) {
                                group = Some(name.clone());
                                options.push(self.option(c, seat, Some(r), None));
                            }
                        }
                    }
                }
                if options.len() > 1 {
                    self.choice(
                        seat,
                        "target",
                        "独有：选择同名独有角色中的一张牺牲".into(),
                        options,
                        1,
                        1,
                        None,
                        ChoiceResolution::Unique,
                    );
                }
            }
            Effect::Cleanup => {
                // Printed rules 4.2.1/2: finish hand-limit choices first.
                // The queued atomic cleanup survives a paused choice/reload.
                self.effects.push_front(Effect::FinishCleanup);
                let mut discards = vec![];
                for seat in self
                    .living(self.first_team)
                    .into_iter()
                    .chain(self.living(1 - self.first_team))
                {
                    if self.players[seat].hand.len() > 7 {
                        discards.push(Effect::Discard {
                            seat,
                            amount: self.players[seat].hand.len() - 7,
                            redraw: false,
                            optional: false,
                        });
                    }
                }
                for e in discards.into_iter().rev() {
                    self.effects.push_front(e);
                }
            }
            Effect::FinishCleanup => {
                // Printed 4.2.3: damage removal and end-of-turn expiry are one
                // batch. Never settle lethal damage between these mutations.
                for region in &mut self.regions {
                    region.skip = false;
                    for c in &mut region.cards {
                        c.damage = 0;
                    }
                }
                self.turn_attribute_modifiers.clear();
                self.modifiers.clear();
                self.control_effects
                    .retain(|effect| !matches!(effect.lifetime, ControlLifetime::TurnEnd { .. }));
                let queued = self.effects.len();
                self.settle_deaths();
                if self.effects.len() > queued || !self.stack.is_empty() || self.pending.is_some() {
                    // 4.2.4: existing responsive death triggers finish under the
                    // current initiative, then another cleanup after all pass.
                    self.begin_window(Window::End);
                } else {
                    self.effects.push_back(Effect::NextTurn);
                }
            }
            Effect::Bury { card: c } => {
                if !self.players[c.owner].eliminated {
                    let owner = c.owner;
                    let c = self.fresh(c);
                    self.players[owner].graveyard.push(c);
                }
            }
            Effect::NextTurn => {
                self.turn += 1;
                self.modifiers.clear();
                self.first_team = 1 - self.first_team;
                self.privilege_used = false;
                for p in &mut self.players {
                    p.asset_used = false;
                    p.conceal_used = false;
                    if let Some(c) = &mut p.society_zone.card {
                        c.exhausted = false;
                    }
                    for c in &mut p.assets {
                        c.exhausted = false;
                    }
                }
                for r in &mut self.regions {
                    for c in &mut r.cards {
                        c.exhausted = false;
                    }
                }
                self.begin_window(Window::Prepare);
                self.note(format!(
                    "第 {} 回合，团队 {} 持先手",
                    self.turn,
                    self.first_team + 1
                ));
            }
        }
        Ok(())
    }
    pub(crate) fn shuffle_player(&mut self, seat: usize) {
        let mut deck = std::mem::take(&mut self.players[seat].deck);
        self.shuffle(&mut deck);
        self.players[seat].deck = deck;
    }
    pub(crate) fn return_hand(&mut self, target: &str) {
        if let Some((_, mut c)) = self.leave_board(target) {
            c = self.fresh(c);
            c.face_down = false;
            c.exhausted = false;
            c.damage = 0;
            c.wounds = 0;
            c.shield = 0;
            let owner = c.owner;
            self.players[owner].hand.push(c);
        }
    }
    pub(crate) fn to_bottom(&mut self, target: &str) {
        if let Some((_, mut c)) = self.leave_board(target) {
            c = self.fresh(c);
            c.face_down = false;
            c.exhausted = false;
            c.damage = 0;
            c.wounds = 0;
            c.shield = 0;
            let owner = c.owner;
            self.players[owner].deck.push(c);
        }
    }
    pub(crate) fn kill(&mut self, target: &str) {
        self.remove_dead(target, RemovalCause::Lethal);
    }
    pub(crate) fn damage(&mut self, allocations: BTreeMap<String, u32>) -> RuleResult<()> {
        for (target, amount) in allocations {
            if let Some(c) = self.board_mut(&target) {
                if !c.face_down {
                    c.damage += amount;
                }
            }
        }
        self.settle_deaths();
        Ok(())
    }
    pub(crate) fn settle_deaths(&mut self) {
        self.settle_controls();
        self.prune_turn_attribute_modifiers();
        self.settle_attachments();
        self.settle_controls();
        let previous_effects = self.effects.len();
        // Loss of a defense aura is checked again after the simultaneous lethal set.
        loop {
            let mut dead = vec![];
            for (r, region) in self.regions.iter().enumerate() {
                for c in &region.cards {
                    if !c.face_down
                        && card(&c.definition).kind == "character"
                        && c.damage >= self.defense(c, r)
                    {
                        dead.push(c.id.clone());
                    }
                }
            }
            if dead.is_empty() {
                break;
            }
            for id in dead {
                self.kill(&id);
            }
        }
        let mut simultaneous = self
            .effects
            .split_off(previous_effects)
            .into_iter()
            .collect::<Vec<_>>();
        simultaneous.sort_by_key(|e| match e {
            Effect::Declare { declaration } if declaration.ability.event == Some(Event::Death) => (
                self.team(declaration.actor) != self.first_team,
                declaration.actor,
            ),
            _ => (true, usize::MAX),
        });
        self.effects.extend(simultaneous);
        self.prune_turn_attribute_modifiers();
    }
    pub(crate) fn choose(&mut self, seat: usize, a: Action) -> RuleResult<()> {
        let p = self.pending.clone().ok_or("没有待选")?;
        if p.seat != seat || a.choice_id.as_deref() != Some(&p.choice.id) {
            return Err("选择者或选择ID不符".into());
        }
        let option_ids = p
            .choice
            .options
            .iter()
            .map(|o| o.id.clone())
            .collect::<BTreeSet<_>>();
        let selected = a.selected.clone().unwrap_or_default();
        let selected_set = selected.iter().cloned().collect::<BTreeSet<_>>();
        if selected_set.len() != selected.len() || !selected_set.is_subset(&option_ids) {
            return Err("选项重复或已失效".into());
        }
        if !matches!(
            p.resolution,
            ChoiceResolution::Forecast { .. }
                | ChoiceResolution::Bottom { .. }
                | ChoiceResolution::Damage { .. }
                | ChoiceResolution::Frame {
                    choice: FrameChoice::Forecast { .. },
                    ..
                }
        ) && (selected.len() < p.choice.min.unwrap_or(0)
            || selected.len() > p.choice.max.unwrap_or(usize::MAX))
        {
            return Err("选择数量不符".into());
        }
        self.pending = None;
        match p.resolution {
            ChoiceResolution::Declare { declaration, stage } => {
                self.choose_declaration(declaration, stage, selected)?
            }
            ChoiceResolution::Frame { frame, choice } => {
                self.choose_frame(frame, choice, a, selected, option_ids)?
            }
            ChoiceResolution::Mulligan => {
                let mut held = vec![];
                for id in selected {
                    held.push(self.remove_hand(seat, &id)?)
                }
                let n = held.len();
                self.draw(seat, n)?;
                for c in held {
                    let c = self.fresh(c);
                    self.players[seat].deck.push(c);
                }
                self.shuffle_player(seat);
                self.note(format!(
                    "{} 完成一次再调度（{}张）",
                    self.players[seat].name, n
                ));
            }
            ChoiceResolution::Discard { redraw } => {
                let count = selected.len();
                for id in selected {
                    let c = self.remove_hand(seat, &id)?;
                    let c = self.fresh(c);
                    self.players[seat].graveyard.push(c);
                }
                if redraw {
                    self.draw(seat, count)?;
                }
                self.note(format!("{} 弃掉 {} 张手牌", self.players[seat].name, count));
            }
            ChoiceResolution::Forecast { draw } => {
                let top = a.top.unwrap_or_default();
                let bottom = a.bottom.unwrap_or_default();
                let all = top.iter().chain(bottom.iter()).cloned().collect::<Vec<_>>();
                let set = all.iter().cloned().collect::<BTreeSet<_>>();
                if set != option_ids || set.len() != all.len() {
                    return Err("必须将所有预测牌完整分到顶或底".into());
                }
                let mut inspected = std::mem::take(&mut self.players[seat].deck);
                let remainder = inspected.split_off(all.len());
                let mut map = inspected
                    .into_iter()
                    .map(|c| (c.id.clone(), c))
                    .collect::<BTreeMap<_, _>>();
                let mut pile = vec![];
                for id in top {
                    pile.push(map.remove(&id).ok_or("预测牌引用失效")?)
                }
                pile.extend(remainder);
                for id in bottom {
                    pile.push(map.remove(&id).ok_or("预测牌引用失效")?)
                }
                self.players[seat].deck = pile;
                if draw {
                    self.draw(seat, 1)?;
                }
                self.note(format!("{} 完成预测排序", self.players[seat].name));
            }
            ChoiceResolution::Damage { region } => {
                let allocations = a.allocations.unwrap_or_default();
                if !allocations.keys().all(|id| option_ids.contains(id))
                    || allocations.values().map(|x| *x as u64).sum::<u64>()
                        != p.choice.amount.unwrap_or(0) as u64
                {
                    return Err("须向合法角色分配全部伤害".into());
                }
                let guards = self.regions[region]
                    .cards
                    .iter()
                    .filter(|c| {
                        !c.face_down
                            && self.is_enemy(seat, c)
                            && rules::definition(&c.definition).traits.guard > 0
                    })
                    .collect::<Vec<_>>();
                let total = p.choice.amount.unwrap_or(0);
                let guard_assigned = guards
                    .iter()
                    .map(|c| {
                        allocations
                            .get(&c.id)
                            .copied()
                            .unwrap_or(0)
                            .min(rules::definition(&c.definition).traits.guard)
                    })
                    .sum::<u32>();
                if guard_assigned
                    < total.min(
                        guards
                            .iter()
                            .map(|c| rules::definition(&c.definition).traits.guard)
                            .sum(),
                    )
                {
                    return Err("必须先向护卫分配护卫值".into());
                }
                self.damage(allocations)?;
            }
            ChoiceResolution::Recipient {
                region,
                contest,
                amount,
            } => {
                let recipient = selected[0]
                    .strip_prefix('p')
                    .ok_or("执行者无效")?
                    .parse::<usize>()
                    .map_err(|_| "执行者无效")?;
                if contest == 0 {
                    self.effects.push_front(Effect::Forecast {
                        seat: recipient,
                        amount: amount as usize,
                        draw: true,
                    })
                } else {
                    self.effects.push_front(Effect::Damage {
                        seat: recipient,
                        region,
                        amount,
                    })
                }
            }
            ChoiceResolution::Bottom { region } => {
                let ordered = a.bottom.or(a.selected).unwrap_or_default();
                let set = ordered.iter().cloned().collect::<BTreeSet<_>>();
                if set != option_ids || set.len() != ordered.len() {
                    return Err("请完整排列自己的所有地区牌".into());
                }
                if let Some(batch) = &mut self.region_return {
                    if batch.region != region
                        || batch.bottom[seat].iter().cloned().collect::<BTreeSet<_>>() != set
                    {
                        return Err("赢区回底批次已失效".into());
                    }
                    batch.orders[seat] = Some(ordered);
                    return Ok(());
                }
                for id in ordered {
                    if self.board(&id).is_some_and(|(r, _)| r != region) {
                        return Err("地区引用失效".into());
                    }
                    self.to_bottom(&id);
                }
            }
            ChoiceResolution::Unique => {
                self.remove_dead(&selected[0], RemovalCause::Sacrifice);
            }
        }
        Ok(())
    }
    pub(crate) fn card_view(
        &self,
        c: &Card,
        viewer: usize,
        region: Option<usize>,
        kind: Option<&str>,
    ) -> CardView {
        let d = card(&c.definition);
        let hidden = c.face_down && c.controller != viewer;
        let asset = kind == Some("asset");
        CardView {
            current_subtypes: if hidden || asset || c.face_down || region.is_none() {
                None
            } else {
                let current = self.current_subtypes(c);
                (current != d.subtypes).then_some(current)
            },
            instance_id: c.id.clone(),
            card_id: if hidden || asset {
                None
            } else {
                Some(d.id.clone())
            },
            name: if asset {
                "资产".into()
            } else if hidden {
                "暗藏者".into()
            } else {
                d.name.clone()
            },
            owner: player_id(c.owner),
            controller: player_id(c.controller),
            kind: kind
                .unwrap_or(if c.face_down { "hidden" } else { &d.kind })
                .into(),
            region,
            exhausted: c.exhausted,
            face_down: c.face_down,
            cost: if hidden || asset || d.kind == "society" {
                None
            } else {
                Some(d.cost)
            },
            effective_cost: if hidden
                || asset
                || !self.players[viewer].hand.iter().any(|own| own.id == c.id)
            {
                None
            } else {
                Some(self.effective_cost(c.controller, c))
            },
            text: if hidden || asset {
                None
            } else {
                Some(d.text.clone())
            },
            icons: if hidden || asset {
                None
            } else {
                Some(
                    region
                        .map(|r| self.icons(c, r))
                        .unwrap_or(d.permanent_icons),
                )
            },
            defense: if hidden || asset || d.kind != "character" {
                None
            } else {
                Some(
                    region
                        .map(|r| self.defense(c, r))
                        .unwrap_or(d.defense.unwrap_or(0)),
                )
            },
            damage: if hidden || asset {
                None
            } else {
                Some(c.damage)
            },
            wounds: if hidden || asset {
                None
            } else {
                Some(c.wounds)
            },
            shield: if hidden || asset {
                None
            } else {
                Some(c.shield)
            },
            color: if hidden { None } else { Some(d.color.clone()) },
            magic: if hidden { None } else { Some(d.magic.clone()) },
            used_once_per_game: if hidden || d.kind != "society" {
                None
            } else {
                self.society_usage(&c.id)
                    .filter(|used| !used.is_empty())
                    .map(|used| used.iter().cloned().collect())
            },
        }
    }
    pub fn view(&self, seat: usize) -> View {
        let (phase, step) = match &self.window {
            Some(Window::Prepare) => ("start", "prepare".into()),
            Some(Window::Draw) => ("start", "draw".into()),
            Some(Window::Action(team)) => ("action", format!("team:{team}")),
            Some(Window::Mobility) => ("confrontation", "mobility".into()),
            Some(Window::Before(r, c)) => (
                "confrontation",
                format!(
                    "region:{}:{}:before",
                    r,
                    ["investigation", "combat", "influence"][*c]
                ),
            ),
            Some(Window::After(r, c)) => (
                "confrontation",
                format!(
                    "region:{}:{}:after",
                    r,
                    ["investigation", "combat", "influence"][*c]
                ),
            ),
            Some(Window::Win(r, _)) => ("confrontation", format!("region:{r}:win")),
            Some(Window::End) => ("end", "end".into()),
            None => ("lobby", "lobby".into()),
        };
        let regions = self
            .regions
            .iter()
            .enumerate()
            .map(|(index, r)| {
                let d = card(&r.card.definition);
                RegionView {
                    id: r.card.id.clone(),
                    index,
                    card_id: d.id.clone(),
                    name: d.name.clone(),
                    threshold: d.threshold.unwrap_or(0),
                    points: d.points.unwrap_or(0),
                    influence: r.influence,
                    skip_confrontation: r.skip,
                    icons_by_team: {
                        let investigation = self.contest_counts(index, 0);
                        let combat = self.contest_counts(index, 1);
                        let influence = self.contest_counts(index, 2);
                        [0, 1].map(|team| Icons {
                            investigation: investigation[team],
                            combat: combat[team],
                            influence: influence[team],
                        })
                    },
                    characters: r
                        .cards
                        .iter()
                        .map(|c| self.card_view(c, seat, Some(index), None))
                        .collect(),
                }
            })
            .collect();
        View {
            room_id: self.room_id.clone(),
            invite_code: self.invite_code.clone(),
            version: self.version,
            mode: self.mode.clone(),
            status: self.status.clone(),
            you: player_id(seat),
            players: self
                .players
                .iter()
                .map(|p| PlayerView {
                    id: player_id(p.seat),
                    seat: p.seat,
                    name: p.name.clone(),
                    team: self.team(p.seat),
                    deck_id: p.deck_id.clone(),
                    deck_name: p
                        .deck_snapshot
                        .as_ref()
                        .map(|d| d.name.clone())
                        .or_else(|| catalog::deck(&p.deck_id).map(|d| d.name.clone()))
                        .unwrap_or_default(),
                    ready: p.ready,
                    eliminated: p.eliminated,
                    hand_count: p.hand.len(),
                    deck_count: p.deck.len(),
                    score: p
                        .score_cards
                        .iter()
                        .map(|c| card(&c.definition).points.unwrap_or(0))
                        .sum(),
                })
                .collect(),
            first_team: self.first_team,
            active_team: self.active_team,
            priority_team: self.priority_team,
            turn: self.turn,
            phase: phase.into(),
            step,
            win_score: self.win_score(),
            winner_team: self.winner_team,
            regions,
            society_zones: self.society_views(seat),
            attachments: self
                .attachments
                .iter()
                .filter_map(|a| {
                    self.board(&a.host_id).map(|(r, _)| AttachmentView {
                        card: self.card_view(&a.card, seat, Some(r), Some("attachment")),
                        host_id: a.host_id.clone(),
                    })
                })
                .collect(),
            private_deck_top: if self.status == "playing"
                && !self.players[seat].eliminated
                && self.regions.iter().flat_map(|r| &r.cards).any(|c| {
                    c.controller == seat
                        && !c.face_down
                        && rules::definition(&c.definition)
                            .modifiers
                            .iter()
                            .any(|m| matches!(m, rules::StaticModifier::PeekOwnDeckTop))
                }) {
                self.players[seat]
                    .deck
                    .first()
                    .map(|c| self.card_view(c, seat, None, None))
            } else {
                None
            },
            hand: self.players[seat]
                .hand
                .iter()
                .map(|c| self.card_view(c, seat, None, None))
                .collect(),
            assets: self
                .players
                .iter()
                .flat_map(|p| p.assets.iter())
                .map(|c| self.card_view(c, seat, None, Some("asset")))
                .collect(),
            graveyard: self
                .players
                .iter()
                .flat_map(|p| p.graveyard.iter())
                .map(|c| self.card_view(c, seat, None, None))
                .collect(),
            score_cards: self
                .players
                .iter()
                .flat_map(|p| p.score_cards.iter())
                .map(|c| self.card_view(c, seat, None, None))
                .collect(),
            stack: self
                .stack
                .iter()
                .map(|s| {
                    if let Some(frame) = &s.frame {
                        self.frame_view(frame, s.label.clone(), "awaitingResponses")
                    } else {
                        StackView {
                            id: s.id.clone(),
                            label: s.label.clone(),
                            controller: player_id(s.controller),
                            card_id: s.card.as_ref().map(|c| c.definition.clone()),
                            target_id: s.target.clone(),
                            ability_id: None,
                            targets: s.target.iter().cloned().collect(),
                            target_summaries: vec![],
                            resolution_state: "awaitingResponses".into(),
                        }
                    }
                })
                .chain(self.pending.iter().filter_map(|p| {
                    if let ChoiceResolution::Frame { frame, .. } = &p.resolution {
                        Some(self.frame_view(
                            frame,
                            format!("{}：结算中", card(&frame.source.card.definition).name),
                            "resolving",
                        ))
                    } else {
                        None
                    }
                }))
                .collect(),
            pending_choice: self
                .pending
                .as_ref()
                .filter(|p| p.seat == seat)
                .map(|p| p.choice.clone()),
            waiting_choice: self.pending.as_ref().map(|p| WaitingChoice {
                player_id: player_id(p.seat),
                kind: p.choice.kind.clone(),
                title: p.choice.title.clone(),
            }),
            legal_actions: self.legal_actions(seat),
            log: self.log.clone(),
            versions: self.versions.clone(),
            your_deck: self.players[seat]
                .deck_snapshot
                .clone()
                .or_else(|| crate::deck::preset(&self.players[seat].deck_id).ok()),
            world_deck_count: self.world.len(),
        }
    }
    pub fn legal_actions(&self, seat: usize) -> Vec<LegalAction> {
        if let Some(p) = &self.pending {
            return if p.seat == seat {
                vec![LegalAction {
                    action: Action {
                        kind: "choose".into(),
                        choice_id: Some(p.choice.id.clone()),
                        ..Action::default()
                    },
                    id: format!("choose:{}", p.choice.id),
                    label: "确认选择".into(),
                    description: None,
                    source_zone_id: None,
                }]
            } else {
                vec![]
            };
        }
        let mut candidates: Vec<(Action, String)> = vec![];
        if self.status == "lobby" {
            candidates.push((
                Action::new("ready"),
                if self.players[seat].ready {
                    "取消准备"
                } else {
                    "准备"
                }
                .into(),
            ));
            for deck in &catalog::catalog().decks {
                candidates.push((
                    Action {
                        option: Some(deck.id.clone()),
                        ..Action::new("deck")
                    },
                    format!("选择牌组：{}", deck.name),
                ));
            }
            if seat == 0 {
                candidates.push((Action::new("start"), "开始游戏".into()));
            }
        } else if self.status == "finished" {
            if seat == 0 {
                candidates.push((Action::new("restart"), "重新开局".into()));
            }
        } else if self.can_fast(seat) {
            if !self.passed.contains(&seat) {
                candidates.push((Action::new("pass"), "让过".into()));
            }
            candidates.push((Action::new("privilege"), "先手特权（支付1）".into()));
            let board = self
                .regions
                .iter()
                .enumerate()
                .flat_map(|(r, region)| region.cards.iter().map(move |c| (r, c)))
                .collect::<Vec<_>>();
            for c in &self.players[seat].hand {
                let d = card(&c.definition);
                if self.can_standard(seat) {
                    candidates.push((
                        Action {
                            card_id: Some(c.id.clone()),
                            ..Action::new("asset")
                        },
                        format!("{} 建立资产", d.name),
                    ));
                }
                if d.kind == "character" {
                    for r in 0..self.regions.len() {
                        for kind in ["deploy", "conceal"] {
                            candidates.push((
                                Action {
                                    card_id: Some(c.id.clone()),
                                    region: Some(r),
                                    ..Action::new(kind)
                                },
                                format!(
                                    "{} {} → {}",
                                    if kind == "deploy" {
                                        "派遣"
                                    } else {
                                        "秘密派遣"
                                    },
                                    d.name,
                                    card(&self.regions[r].card.definition).name
                                ),
                            ));
                        }
                    }
                } else if d.kind == "spell" || d.kind == "attachment" {
                    candidates.extend(self.rule_action_candidates(seat, c, None, "play"));
                }
            }
            if self.can_standard(seat) {
                for c in &self.players[seat].graveyard {
                    if card(&c.definition).kind != "character"
                        || !rules::definition(&c.definition).graveyard_face_up
                    {
                        continue;
                    }
                    for r in 0..self.regions.len() {
                        candidates.push((
                            Action {
                                card_id: Some(c.id.clone()),
                                region: Some(r),
                                ..Action::new("deploy")
                            },
                            format!(
                                "墓地正面打出 {} → {}",
                                card(&c.definition).name,
                                card(&self.regions[r].card.definition).name
                            ),
                        ));
                    }
                }
            }
            for (r, c) in &board {
                if c.controller != seat {
                    continue;
                }
                if c.face_down {
                    candidates.push((
                        Action {
                            card_id: Some(c.id.clone()),
                            ..Action::new("reveal")
                        },
                        format!("现身 {}", card(&c.definition).name),
                    ));
                } else {
                    candidates.extend(self.rule_action_candidates(seat, c, Some(*r), "activate"));
                }
            }
            if let Some(c) = &self.players[seat].society_zone.card {
                candidates.extend(self.rule_action_candidates(seat, c, None, "activate"));
            }
        }
        candidates
            .into_iter()
            .filter_map(|(action, label)| {
                let mut trial = self.clone();
                if trial.apply_inner(seat, action.clone()).is_ok() {
                    let id = serde_json::to_string(&action).expect("legal action JSON");
                    Some(LegalAction {
                        source_zone_id: self.society_source_zone(action.card_id.as_deref()),
                        action,
                        id,
                        label,
                        description: None,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn game(mode: &str) -> Game {
        let mut g = Game::new(
            "test".into(),
            "INVITE".into(),
            mode.into(),
            "A".into(),
            "watchers".into(),
            7323,
        )
        .unwrap();
        for seat in 1..g.capacity() {
            g.join(
                format!("P{seat}"),
                ["watchers", "hunters", "keepers", "reclaimers"][seat].into(),
            )
            .unwrap();
        }
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
        g.first_team = 0;
        g.begin_window(Window::Action(0));
        g
    }
    fn board(g: &mut Game, id: &str, seat: usize, region: usize) -> String {
        let c = g.make_card(id, seat);
        let instance = c.id.clone();
        g.regions[region].cards.push(c);
        instance
    }
    fn hand(g: &mut Game, id: &str, seat: usize) -> String {
        let c = g.make_card(id, seat);
        let instance = c.id.clone();
        g.players[seat].hand.push(c);
        instance
    }
    fn resource(g: &mut Game, seat: usize, color: &str, amount: usize) {
        for _ in 0..amount {
            let c = g.make_card(color, seat);
            g.players[seat].assets.push(c);
        }
    }
    fn region_effect_fixture(g: &mut Game, definition: &str) {
        let c = g.make_card(definition, 0);
        let source = g.source_snapshot(&c, None);
        let spec = rules::definition(definition).abilities[0].clone();
        if !spec.modes.is_empty() {
            g.declare_trigger(Declaration {
                actor: 0,
                source,
                ability: spec,
            })
            .unwrap();
        } else {
            let frame = g.make_frame(0, source, &spec, vec![], vec![], None);
            g.effects.push_back(Effect::Frame {
                frame: Box::new(frame),
            });
        }
    }
    fn move_fixture(g: &mut Game, target: &str, region: usize) {
        let (r, c) = g.board(target).unwrap();
        let source = g.source_snapshot(c, Some(r));
        let slot = rules::TargetSlotSpec {
            zone: rules::Zone::Region,
            kind: rules::EntityKind::Any,
            relation: rules::Relation::Any,
            range: rules::Range::Anywhere,
            subtype: None,
            printed_subtype: false,
            subtypes_any: vec![],
            printed_cost_max: None,
            equipment_host: false,
            requires_magic: false,
            exclude_source: false,
            attachment_host_condition: None,
            min: 1,
            max: 1,
        };
        let id = format!("region:{region}");
        let public = g.public_target(c.controller, &source, &slot, &id);
        let mut spec = rules::definition("JC014").abilities[0].clone();
        spec.requires_ready_source = false;
        let frame = g.make_frame(
            c.controller,
            source,
            &spec,
            vec![BoundTarget {
                id,
                spec: slot,
                public,
            }],
            vec![],
            None,
        );
        g.resolve_frame(frame).unwrap();
    }
    fn pass_stack(g: &mut Game) {
        let mut n = 0;
        while !g.stack.is_empty() && g.pending.is_none() {
            n += 1;
            assert!(n < 100);
            let seat = g
                .living(g.priority_team)
                .into_iter()
                .find(|s| !g.passed.contains(s))
                .unwrap();
            g.apply(seat, Action::new("pass")).unwrap();
        }
    }
    fn select(g: &mut Game, selected: Vec<String>) {
        let p = g.pending.clone().unwrap();
        g.apply(
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                selected: Some(selected),
                ..Action::new("choose")
            },
        )
        .unwrap();
    }
    fn accept(g: &mut Game) {
        select(g, vec!["accept".into()]);
        pass_stack(g);
    }
    #[test]
    fn catalog_is_restricted_real_complete_and_decks_are_legal() {
        let c = catalog::catalog();
        assert_eq!(c.cards.len(), 56);
        let active = c
            .cards
            .iter()
            .map(|d| d.id.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            active,
            rules::definitions()
                .keys()
                .filter(|id| c.cards.iter().any(|card| card.id == **id))
                .cloned()
                .collect::<BTreeSet<_>>()
        );
        assert!(active.contains("DQJC116"));
        assert_eq!(c.cards.iter().filter(|d| d.kind != "region").count(), 46);
        assert_eq!(c.decks.len(), 5);
        for deck in &c.decks {
            assert_eq!(deck.card_count, 50);
            assert_eq!(deck.cards.iter().map(|x| x.count).sum::<usize>(), 50);
            let detectives = deck
                .cards
                .iter()
                .filter(|e| e.card_id == "JC058")
                .map(|e| e.count)
                .sum::<usize>();
            assert_eq!(detectives, if deck.id == "keepers" { 3 } else { 0 });
            if deck.id == "keepers" {
                assert_eq!(
                    deck.cards
                        .iter()
                        .find(|e| e.card_id == "JC125")
                        .unwrap()
                        .count,
                    14
                );
            }
            for e in &deck.cards {
                assert!(e.card_id == "JC125" || e.count <= 3);
                assert_ne!(card(&e.card_id).kind, "region");
            }
        }
        assert_eq!(card("LC23").unique, true);
        assert!(card("JC125").text.contains("数量没有限制"));
        let expected = (107..=116)
            .map(|n| (format!("DQJC{n}"), 1))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            c.world
                .iter()
                .map(|e| (e.card_id.clone(), e.count))
                .collect::<BTreeMap<_, _>>(),
            expected
        );
        let mut g = game("duel");
        let mut actual = BTreeMap::new();
        for card in g.regions.iter().map(|r| &r.card).chain(g.world.iter()) {
            *actual.entry(card.definition.clone()).or_insert(0) += 1;
        }
        assert_eq!(actual, expected);
        let before = serde_json::to_string(&g).unwrap();
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            g.make_card("unreviewed-world-card", 0)
        }))
        .is_err());
        assert_eq!(serde_json::to_string(&g).unwrap(), before);
    }
    #[test]
    fn hidden_information_projection_and_reveal_identity() {
        let mut g = game("teams");
        resource(&mut g, 0, "JC125", 4);
        let id = board(&mut g, "LC23", 0, 1);
        g.board_mut(&id).unwrap().face_down = true;
        for viewer in [1, 2, 3] {
            let v = g.view(viewer);
            let c = &v.regions[1].characters[0];
            assert_eq!(c.name, "暗藏者");
            assert!(
                c.card_id.is_none()
                    && c.text.is_none()
                    && c.cost.is_none()
                    && c.icons.is_none()
                    && c.color.is_none()
                    && c.magic.is_none()
            );
            assert_eq!(v.hand.len(), 6);
            assert_eq!(v.hand[0].owner, player_id(viewer));
        }
        assert_eq!(g.icons(g.board(&id).unwrap().1, 1).influence, 1);
        g.board_mut(&id).unwrap().exhausted = true;
        g.apply(
            0,
            Action {
                card_id: Some(id.clone()),
                ..Action::new("reveal")
            },
        )
        .unwrap();
        assert!(g.board(&id).is_none());
        pass_stack(&mut g);
        let c = &g.regions[1].cards[0];
        assert_ne!(c.id, id);
        assert!(c.exhausted);
        assert!(!c.face_down);
    }
    #[test]
    fn next_batch_owl_uses_controller_mind_two_and_temporary_icons_only() {
        assert_eq!(card("JC001").cost, 1);
        assert_eq!(card("JC001").loyalty, vec!["黄色"]);
        assert_eq!(card("JC001").subtypes, vec!["鸟", "魔宠"]);
        let mut g = game("teams");
        let owl = board(&mut g, "JC001", 1, 2);
        g.board_mut(&owl).unwrap().controller = 0;
        resource(&mut g, 1, "JC003", 2); // Owner assets grant no controller condition.
        assert_eq!(g.icons(g.board(&owl).unwrap().1, 2).investigation, 0);
        resource(&mut g, 0, "JC003", 1);
        assert_eq!(g.icons(g.board(&owl).unwrap().1, 2).investigation, 0);
        resource(&mut g, 0, "JC003", 1);
        for a in &mut g.players[0].assets {
            a.exhausted = true;
        }
        assert_eq!(
            g.icons(g.board(&owl).unwrap().1, 2),
            Icons {
                investigation: 1,
                combat: 0,
                influence: 1
            }
        );
        g.first_team = 1;
        assert_eq!(g.icons(g.board(&owl).unwrap().1, 2), Icons::default());
        g.first_team = 0;
        g.board_mut(&owl).unwrap().face_down = true;
        for viewer in [1, 2, 3] {
            let v = g.view(viewer);
            let c = v.regions[2]
                .characters
                .iter()
                .find(|c| c.instance_id == owl)
                .unwrap();
            assert!(c.card_id.is_none() && c.icons.is_none() && c.name == "暗藏者");
        }
        assert_eq!(
            g.view(0).regions[2]
                .characters
                .iter()
                .find(|c| c.instance_id == owl)
                .unwrap()
                .card_id
                .as_deref(),
            Some("JC001")
        );
        let saved = serde_json::to_string(&g).unwrap();
        let mut g: Game = serde_json::from_str(&saved).unwrap();
        assert_eq!(
            g.icons(g.board(&owl).unwrap().1, 2),
            Icons {
                influence: 1,
                ..Icons::default()
            }
        );
        g.board_mut(&owl).unwrap().face_down = false;
        g.board_mut(&owl).unwrap().exhausted = true;
        assert_eq!(g.icons(g.board(&owl).unwrap().1, 2), Icons::default());
    }
    #[test]
    fn next_batch_owl_equipment_host_predicate_blocks_without_blocking_other_targets() {
        let mut g = game("duel");
        let owl = board(&mut g, "JC001", 0, 0);
        let equipment = g.make_card("BQ022", 0);
        let source = g.source_snapshot(&equipment, None);
        let mut host = rules::definition("BQ022")
            .attachment
            .as_ref()
            .unwrap()
            .host
            .clone();
        // Explicit primitive fixture: relax BQ022's Human/Vampire restriction solely
        // to isolate equipment immunity. The published BQ022 definition stays exact.
        host.subtypes_any.clear();
        assert!(!g.valid_binding(0, &source, &host, &owl));
        let spell = g.make_card("XQ03", 0);
        let snapshot = g.source_snapshot(&spell, None);
        let target = &rules::definition("XQ03").abilities[0].targets[0];
        assert!(g.valid_binding(0, &snapshot, target, &owl));
        let ordinary = board(&mut g, "JC125", 0, 0);
        assert!(g.valid_binding(0, &source, &host, &ordinary));
        let equipment_id = hand(&mut g, "BQ022", 0);
        resource(&mut g, 0, "JC125", 1);
        let actions = g.view(0).legal_actions;
        assert!(!actions.iter().any(|a| a.action.kind == "play"
            && a.action.card_id.as_deref() == Some(equipment_id.as_str())
            && a.action.target_id.as_deref() == Some(owl.as_str())));
        assert!(actions.iter().any(|a| a.action.kind == "play"
            && a.action.card_id.as_deref() == Some(equipment_id.as_str())
            && a.action.target_id.as_deref() == Some(ordinary.as_str())));
    }
    #[test]
    fn next_batch_skeleton_deploy_pays_four_and_enter_targets_all_sides_in_same_region() {
        assert_eq!(card("BQ083").cost, 4);
        assert_eq!(card("BQ083").loyalty, vec!["黑色", "黑色", "黑色"]);
        assert_eq!(card("BQ083").society, card("JC085").society);
        let mut g = game("teams");
        let victims = (0..4)
            .map(|seat| board(&mut g, "JC125", seat, 2))
            .collect::<Vec<_>>();
        let outside = board(&mut g, "JC125", 3, 3);
        let hidden = board(&mut g, "JC125", 1, 2);
        g.board_mut(&hidden).unwrap().face_down = true;
        let friendly_barrier = board(&mut g, "JZ08", 1, 2);
        let protected = board(&mut g, "JZ08", 2, 2);
        assert!(!g.is_enemy(0, g.board(&friendly_barrier).unwrap().1));
        assert!(g.is_enemy(0, g.board(&protected).unwrap().1));
        resource(&mut g, 0, "JC085", 3);
        resource(&mut g, 0, "JC125", 1);
        let source = hand(&mut g, "BQ083", 0);
        g.apply(
            0,
            Action {
                card_id: Some(source.clone()),
                region: Some(2),
                ..Action::new("deploy")
            },
        )
        .unwrap();
        assert_eq!(g.resources(0), 0);
        assert!(g.board(&source).is_none());
        pass_stack(&mut g);
        let new_source = g.regions[2]
            .cards
            .iter()
            .find(|c| c.definition == "BQ083")
            .unwrap()
            .id
            .clone();
        assert_ne!(new_source, source);
        let options = &g.pending.as_ref().unwrap().choice.options;
        for id in victims.iter().chain([&new_source, &friendly_barrier]) {
            assert!(options.iter().any(|o| o.id == *id));
        }
        for id in [&outside, &hidden, &protected] {
            assert!(!options.iter().any(|o| o.id == *id));
        }
        let saved = serde_json::to_string(&g).unwrap();
        let mut g: Game = serde_json::from_str(&saved).unwrap();
        select(&mut g, vec![victims[3].clone()]);
        assert!(g.board(&victims[3]).is_some()); // The trigger is respondable, never immediate.
        assert_eq!(
            g.stack.last().unwrap().frame.as_ref().unwrap().targets[0].id,
            victims[3]
        );
        pass_stack(&mut g);
        assert!(g.board(&victims[3]).is_none());
        assert_eq!(g.players[3].graveyard.last().unwrap().definition, "JC125");
        assert_eq!(g.resources(0), 0);
        assert!(g.board(&new_source).is_some());
    }
    #[test]
    fn next_batch_skeleton_cannot_deploy_without_loyalty_and_hidden_deploy_does_not_trigger() {
        let mut g = game("duel");
        resource(&mut g, 0, "JC125", 4);
        let source = hand(&mut g, "BQ083", 0);
        let victim = board(&mut g, "JC125", 1, 0);
        let before = serde_json::to_string(&g).unwrap();
        assert!(g
            .apply(
                0,
                Action {
                    card_id: Some(source.clone()),
                    region: Some(0),
                    ..Action::new("deploy")
                }
            )
            .unwrap_err()
            .contains("忠诚"));
        assert_eq!(serde_json::to_string(&g).unwrap(), before);
        g.apply(
            0,
            Action {
                card_id: Some(source),
                region: Some(0),
                ..Action::new("conceal")
            },
        )
        .unwrap();
        assert!(g.pending.is_none() && g.stack.is_empty() && g.board(&victim).is_some());
        assert_eq!(g.resources(0), 3);
        assert!(g.regions[0]
            .cards
            .iter()
            .any(|c| c.definition == "BQ083" && c.face_down));
    }
    #[test]
    fn next_batch_skeleton_trigger_can_decline_and_revalidates_after_target_or_source_hides() {
        for hide_source in [false, true] {
            let mut g = game("duel");
            let victim = board(&mut g, "JC125", 1, 0);
            resource(&mut g, 0, "JC085", 4);
            resource(&mut g, 1, "JC063", 4);
            let killer = hand(&mut g, "BQ083", 0);
            let chase = hand(&mut g, "JC063", 1);
            g.apply(
                0,
                Action {
                    card_id: Some(killer),
                    region: Some(0),
                    ..Action::new("deploy")
                },
            )
            .unwrap();
            pass_stack(&mut g);
            let source = g.regions[0]
                .cards
                .iter()
                .find(|c| c.definition == "BQ083")
                .unwrap()
                .id
                .clone();
            let mut declined = g.clone();
            select(&mut declined, vec![]);
            assert!(
                declined.pending.is_none()
                    && declined.stack.is_empty()
                    && declined.board(&victim).is_some()
            );
            select(&mut g, vec![victim.clone()]);
            g.apply(0, Action::new("pass")).unwrap();
            g.apply(
                1,
                Action {
                    card_id: Some(chase),
                    target_id: Some(if hide_source {
                        source.clone()
                    } else {
                        victim.clone()
                    }),
                    option: Some("hide".into()),
                    ..Action::new("play")
                },
            )
            .unwrap();
            let encoded = serde_json::to_string(&g).unwrap();
            let mut g: Game = serde_json::from_str(&encoded).unwrap();
            pass_stack(&mut g);
            assert_eq!(g.resources(0), 0);
            assert_eq!(
                g.players[1]
                    .graveyard
                    .iter()
                    .any(|c| c.definition == "JC125"),
                hide_source
            );
            if hide_source {
                assert!(g.board(&source).is_none());
                assert!(g.regions[0]
                    .cards
                    .iter()
                    .any(|c| c.definition == "BQ083" && c.face_down));
            } else {
                assert!(g.regions[0]
                    .cards
                    .iter()
                    .any(|c| c.definition == "JC125" && c.face_down));
            }
        }
    }

    #[test]
    fn semantic_scope_friend_enemy_and_barrier_use_controller_team_not_owner() {
        // Explicit control primitives; the admitted pool has no takeover card.
        let mut g = game("teams");
        let aura = board(&mut g, "JC059", 2, 0);
        g.board_mut(&aura).unwrap().controller = 0;
        let friend = board(&mut g, "JC125", 2, 0);
        g.board_mut(&friend).unwrap().controller = 1;
        let enemy = board(&mut g, "JC125", 0, 0);
        g.board_mut(&enemy).unwrap().controller = 2;
        assert_eq!(g.defense(g.board(&friend).unwrap().1, 0), 2);
        assert_eq!(g.defense(g.board(&enemy).unwrap().1, 0), 1);
        assert_eq!(g.defense(g.board(&aura).unwrap().1, 0), 2);
        let friendly_barrier = board(&mut g, "JZ08", 2, 1);
        g.board_mut(&friendly_barrier).unwrap().controller = 1;
        let enemy_barrier = board(&mut g, "JZ08", 0, 1);
        g.board_mut(&enemy_barrier).unwrap().controller = 2;
        assert!(g.targetable(0, g.board(&friendly_barrier).unwrap().1));
        assert!(!g.targetable(0, g.board(&enemy_barrier).unwrap().1));
        let elite = board(&mut g, "JC016", 2, 2);
        g.board_mut(&elite).unwrap().controller = 0;
        let teammate = board(&mut g, "JC125", 2, 2);
        g.board_mut(&teammate).unwrap().controller = 1;
        let hidden_enemy = board(&mut g, "JC125", 0, 2);
        g.board_mut(&hidden_enemy).unwrap().controller = 2;
        g.board_mut(&hidden_enemy).unwrap().face_down = true;
        assert_eq!(g.icons(g.board(&elite).unwrap().1, 2).influence, 1);
        g.board_mut(&hidden_enemy).unwrap().face_down = false;
        assert_eq!(g.icons(g.board(&elite).unwrap().1, 2).influence, 0);
    }

    #[test]
    fn semantic_scope_recycling_keyword_uses_equipment_owner_despite_another_controller() {
        // P17's explicit Recycling definition returns to the owner, unlike a generic "you".
        let mut g = game("teams");
        let host = board(&mut g, "LC22", 2, 4);
        g.board_mut(&host).unwrap().controller = 0;
        let mut equipment = g.make_card("BQ022", 3);
        equipment.controller = 1;
        let old = equipment.id.clone();
        g.attachments.push(Attachment {
            card: equipment,
            host_id: host.clone(),
        });
        let chase = hand(&mut g, "JC063", 0);
        resource(&mut g, 0, "JC063", 2);
        g.apply(
            0,
            Action {
                card_id: Some(chase),
                target_id: Some(host.clone()),
                option: Some("hide".into()),
                ..Action::new("play")
            },
        )
        .unwrap();
        let mut g: Game = serde_json::from_str(&serde_json::to_string(&g).unwrap()).unwrap();
        pass_stack(&mut g);
        assert!(g.attachments.is_empty());
        let returned = g.players[3]
            .hand
            .iter()
            .find(|c| c.definition == "BQ022")
            .unwrap();
        assert_ne!(returned.id, old);
        assert_eq!((returned.owner, returned.controller), (3, 3));
        for seat in 0..3 {
            assert!(!g
                .view(seat)
                .hand
                .iter()
                .any(|c| c.card_id.as_deref() == Some("BQ022")));
        }
        let hidden = g.regions[4]
            .cards
            .iter()
            .find(|c| c.definition == "LC22")
            .unwrap();
        assert!(hidden.face_down && hidden.id != host);
        assert_eq!((hidden.owner, hidden.controller), (2, 0));
        assert!(g.resources(0) == 0);
    }

    #[test]
    fn semantic_scope_your_grave_is_personal_for_recovery_and_summon() {
        for definition in ["JC086", "JC092"] {
            let mut g = game("teams");
            let mut ids = vec![];
            for seat in 0..3 {
                let c = g.make_card("LC20", seat);
                ids.push(c.id.clone());
                g.players[seat].graveyard.push(c);
            }
            let source = hand(&mut g, definition, 0);
            resource(&mut g, 0, "JC085", 2);
            resource(&mut g, 0, "JC014", 1);
            if definition == "JC086" {
                g.apply(
                    0,
                    Action {
                        card_id: Some(source),
                        region: Some(0),
                        ..Action::new("deploy")
                    },
                )
                .unwrap();
                pass_stack(&mut g);
                assert_eq!(
                    g.pending
                        .as_ref()
                        .unwrap()
                        .choice
                        .options
                        .iter()
                        .map(|o| o.id.clone())
                        .collect::<Vec<_>>(),
                    vec![ids[0].clone()]
                );
                for id in &ids[1..] {
                    let before = serde_json::to_string(&g).unwrap();
                    let choice = g.pending.as_ref().unwrap().choice.id.clone();
                    assert!(g
                        .apply(
                            0,
                            Action {
                                choice_id: Some(choice),
                                selected: Some(vec![id.clone()]),
                                ..Action::new("choose")
                            }
                        )
                        .is_err());
                    assert_eq!(serde_json::to_string(&g).unwrap(), before);
                }
                select(&mut g, vec![ids[0].clone()]);
                pass_stack(&mut g);
                let returned = g.players[0]
                    .hand
                    .iter()
                    .find(|c| c.definition == "LC20")
                    .unwrap();
                assert_ne!(returned.id, ids[0]);
                assert_eq!((returned.owner, returned.controller), (0, 0));
            } else {
                for id in &ids[1..] {
                    let before = serde_json::to_string(&g).unwrap();
                    assert!(g
                        .apply(
                            0,
                            Action {
                                card_id: Some(source.clone()),
                                region: Some(4),
                                target_id: Some(id.clone()),
                                ..Action::new("play")
                            }
                        )
                        .is_err());
                    assert_eq!(serde_json::to_string(&g).unwrap(), before);
                }
                g.apply(
                    0,
                    Action {
                        card_id: Some(source),
                        region: Some(4),
                        target_id: Some(ids[0].clone()),
                        ..Action::new("play")
                    },
                )
                .unwrap();
                pass_stack(&mut g);
                let entered = g.regions[4]
                    .cards
                    .iter()
                    .find(|c| c.definition == "LC20")
                    .unwrap();
                assert!(entered.face_down && entered.id != ids[0]);
                assert_eq!((entered.owner, entered.controller), (0, 0));
            }
            for seat in 1..3 {
                assert_eq!(g.players[seat].graveyard[0].id, ids[seat]);
            }
        }
    }

    #[test]
    fn rescue_forecast_rescue_printed_fields_public_deploy_and_one_asset_cost() {
        let d = card("JC075");
        assert_eq!((d.cost, d.defense), (1, Some(1)));
        assert_eq!(d.loyalty, vec!["白色"]);
        assert_eq!(d.subtypes, vec!["人类", "医疗员"]);
        assert_eq!(d.magic_icon, rules::MagicIcon::None);
        assert_eq!(d.permanent_icons.influence, 1);
        assert_eq!(d.temporary_icons, Icons::default());
        assert!(d.rule_traits.public);
        let mut g = game("teams");
        let source = hand(&mut g, "JC075", 0);
        resource(&mut g, 0, "JC075", 2);
        let before = serde_json::to_string(&g).unwrap();
        assert!(g
            .apply(
                0,
                Action {
                    card_id: Some(source.clone()),
                    region: Some(0),
                    ..Action::new("conceal")
                }
            )
            .is_err());
        assert_eq!(serde_json::to_string(&g).unwrap(), before);
        let mut no_loyalty = g.clone();
        no_loyalty.players[0].assets.clear();
        resource(&mut no_loyalty, 0, "JC125", 2);
        let before = serde_json::to_string(&no_loyalty).unwrap();
        assert!(no_loyalty
            .apply(
                0,
                Action {
                    card_id: Some(source.clone()),
                    region: Some(0),
                    ..Action::new("deploy")
                }
            )
            .is_err());
        assert_eq!(serde_json::to_string(&no_loyalty).unwrap(), before);
        g.apply(
            0,
            Action {
                card_id: Some(source.clone()),
                region: Some(0),
                ..Action::new("deploy")
            },
        )
        .unwrap();
        assert_eq!(g.resources(0), 1);
        pass_stack(&mut g);
        let deployed = g.regions[0]
            .cards
            .iter()
            .find(|c| c.definition == "JC075")
            .unwrap();
        assert_ne!(deployed.id, source);
        assert!(!deployed.face_down && !deployed.exhausted);
        assert!(g.pending.is_none());
    }

    #[test]
    fn rescue_forecast_rescue_pays_and_exhausts_returns_to_owner_and_recycles_equipment() {
        let mut g = game("teams");
        let source = board(&mut g, "JC075", 0, 0);
        // Explicit control/ownership primitive: no control-transfer card is admitted.
        let target = board(&mut g, "LC21", 2, 4);
        g.board_mut(&target).unwrap().controller = 0;
        let equipment = g.make_card("BQ022", 3);
        g.attachments.push(Attachment {
            card: equipment,
            host_id: target.clone(),
        });
        let teammate = board(&mut g, "JC001", 1, 4);
        let enemy = board(&mut g, "JC125", 0, 1);
        g.board_mut(&enemy).unwrap().controller = 2;
        let hidden = board(&mut g, "JC125", 0, 0);
        g.board_mut(&hidden).unwrap().face_down = true;
        resource(&mut g, 0, "JC125", 2);
        let action = |id: String| Action {
            card_id: Some(source.clone()),
            target_id: Some(id),
            ability_id: Some("rescue".into()),
            ..Action::new("activate")
        };
        for id in [source.clone(), teammate.clone(), enemy, hidden] {
            let before = serde_json::to_string(&g).unwrap();
            assert!(g.apply(0, action(id)).is_err());
            assert_eq!(serde_json::to_string(&g).unwrap(), before);
        }
        g.apply(0, action(target.clone())).unwrap();
        assert_eq!(g.resources(0), 0);
        assert!(g.board(&source).unwrap().1.exhausted);
        assert!(g.board(&target).is_some());
        let mut g: Game = serde_json::from_str(&serde_json::to_string(&g).unwrap()).unwrap();
        pass_stack(&mut g);
        assert!(g.board(&target).is_none());
        let returned = g.players[2]
            .hand
            .iter()
            .find(|c| c.definition == "LC21")
            .unwrap();
        assert_ne!(returned.id, target);
        assert_eq!((returned.owner, returned.controller), (2, 2));
        assert!(!returned.face_down && !returned.exhausted);
        assert!(g.attachments.is_empty());
        assert!(g.players[3].hand.iter().any(|c| c.definition == "BQ022"));
        assert!(g.board(&source).unwrap().1.exhausted);
        assert!(!g.board(&teammate).unwrap().1.face_down);
    }

    #[test]
    fn rescue_forecast_rescue_costs_are_atomic_and_response_hide_invalidates_the_old_target() {
        let mut g = game("teams");
        let source = board(&mut g, "JC075", 0, 0);
        let target = board(&mut g, "JC088", 0, 4);
        let chase = hand(&mut g, "JC063", 2);
        resource(&mut g, 0, "JC125", 2);
        resource(&mut g, 2, "JC063", 2);
        let action = Action {
            card_id: Some(source.clone()),
            target_id: Some(target.clone()),
            ability_id: Some("rescue".into()),
            ..Action::new("activate")
        };
        for assets in 0..2 {
            let mut failed = g.clone();
            failed.players[0].assets.truncate(assets);
            let before = serde_json::to_string(&failed).unwrap();
            assert!(failed.apply(0, action.clone()).is_err());
            assert_eq!(serde_json::to_string(&failed).unwrap(), before);
        }
        let mut exhausted = g.clone();
        exhausted.board_mut(&source).unwrap().exhausted = true;
        let before = serde_json::to_string(&exhausted).unwrap();
        assert!(exhausted.apply(0, action.clone()).is_err());
        assert_eq!(serde_json::to_string(&exhausted).unwrap(), before);
        g.apply(0, action).unwrap();
        while g.priority_team != 1 {
            let seat = g
                .living(g.priority_team)
                .into_iter()
                .find(|s| !g.passed.contains(s))
                .unwrap();
            g.apply(seat, Action::new("pass")).unwrap();
        }
        g.apply(
            2,
            Action {
                card_id: Some(chase),
                target_id: Some(target.clone()),
                option: Some("hide".into()),
                ..Action::new("play")
            },
        )
        .unwrap();
        while g.stack.len() > 1 {
            let seat = g
                .living(g.priority_team)
                .into_iter()
                .find(|s| !g.passed.contains(s))
                .unwrap();
            g.apply(seat, Action::new("pass")).unwrap();
        }
        let fresh = g.regions[4]
            .cards
            .iter()
            .find(|c| c.definition == "JC088")
            .unwrap()
            .id
            .clone();
        assert_ne!(fresh, target);
        let mut g: Game = serde_json::from_str(&serde_json::to_string(&g).unwrap()).unwrap();
        pass_stack(&mut g);
        let stayed = g.board(&fresh).unwrap().1;
        assert!(stayed.face_down);
        assert_eq!(g.resources(0), 0);
        assert!(g.board(&source).unwrap().1.exhausted);
    }

    #[test]
    fn rescue_forecast_forecast_three_is_private_orders_both_ends_and_does_not_draw() {
        let d = card("JC104");
        assert_eq!((d.cost, d.defense), (2, Some(1)));
        assert_eq!(d.loyalty, vec!["紫色"]);
        assert_eq!(d.subtypes, vec!["人类", "法师"]);
        assert_eq!(d.magic_icon, rules::MagicIcon::Other("星辰".into()));
        assert_eq!(d.permanent_icons, Icons::default());
        assert_eq!(d.temporary_icons.investigation, 1);
        assert!(!d.rule_traits.public);
        let mut g = game("teams");
        let source = hand(&mut g, "JC104", 0);
        g.players[0].deck.clear();
        for definition in ["JC006", "XQ03", "JC125", "JZ08", "LC21"] {
            let c = g.make_card(definition, 0);
            g.players[0].deck.push(c);
        }
        let original = g.players[0]
            .deck
            .iter()
            .map(|c| c.id.clone())
            .collect::<Vec<_>>();
        let hand_count = g.players[0].hand.len() - 1;
        resource(&mut g, 0, "JC104", 2);
        g.apply(
            0,
            Action {
                card_id: Some(source),
                region: Some(0),
                ..Action::new("deploy")
            },
        )
        .unwrap();
        pass_stack(&mut g);
        assert_eq!(g.pending.as_ref().unwrap().choice.kind, "trigger");
        accept(&mut g);
        let pending = g.pending.clone().unwrap();
        assert_eq!(pending.choice.kind, "investigation");
        assert_eq!(
            pending
                .choice
                .options
                .iter()
                .map(|o| o.id.clone())
                .collect::<Vec<_>>(),
            original[..3]
        );
        for viewer in 1..4 {
            let v = g.view(viewer);
            assert!(v.pending_choice.is_none());
            for id in &original {
                assert!(!serde_json::to_string(&v).unwrap().contains(id));
            }
        }
        for (top, bottom) in [
            (vec![original[0].clone()], vec![]),
            (
                vec![original[0].clone(), original[0].clone()],
                original[1..3].to_vec(),
            ),
            (original[..3].to_vec(), vec!["foreign".into()]),
        ] {
            let before = serde_json::to_string(&g).unwrap();
            assert!(g
                .apply(
                    0,
                    Action {
                        choice_id: Some(pending.choice.id.clone()),
                        top: Some(top),
                        bottom: Some(bottom),
                        ..Action::new("choose")
                    }
                )
                .is_err());
            assert_eq!(serde_json::to_string(&g).unwrap(), before);
        }
        let mut g: Game = serde_json::from_str(&serde_json::to_string(&g).unwrap()).unwrap();
        assert_eq!(g.pending.as_ref().unwrap().choice.options.len(), 3);
        g.apply(
            0,
            Action {
                choice_id: Some(pending.choice.id),
                top: Some(vec![original[2].clone(), original[0].clone()]),
                bottom: Some(vec![original[1].clone()]),
                ..Action::new("choose")
            },
        )
        .unwrap();
        assert_eq!(
            g.players[0]
                .deck
                .iter()
                .map(|c| c.id.clone())
                .collect::<Vec<_>>(),
            [
                original[2].clone(),
                original[0].clone(),
                original[3].clone(),
                original[4].clone(),
                original[1].clone()
            ]
        );
        assert_eq!(g.players[0].hand.len(), hand_count);
        assert_eq!(g.resources(0), 0);
        assert!(g.pending.is_none());
    }

    #[test]
    fn rescue_forecast_forecast_can_decline_and_handles_short_or_empty_decks_without_draw() {
        for length in 0..=3 {
            let mut g = game("duel");
            let source = hand(&mut g, "JC104", 0);
            g.players[0].deck.truncate(length);
            let original = g.players[0]
                .deck
                .iter()
                .map(|c| c.id.clone())
                .collect::<Vec<_>>();
            let hand_count = g.players[0].hand.len() - 1;
            resource(&mut g, 0, "JC104", 2);
            g.apply(
                0,
                Action {
                    card_id: Some(source),
                    region: Some(0),
                    ..Action::new("deploy")
                },
            )
            .unwrap();
            pass_stack(&mut g);
            let mut declined = g.clone();
            select(&mut declined, vec![]);
            assert!(declined.pending.is_none());
            assert_eq!(
                declined.players[0]
                    .deck
                    .iter()
                    .map(|c| c.id.clone())
                    .collect::<Vec<_>>(),
                original
            );
            accept(&mut g);
            if length > 0 {
                let pending = g.pending.clone().unwrap();
                assert_eq!(pending.choice.options.len(), length);
                g.apply(
                    0,
                    Action {
                        choice_id: Some(pending.choice.id),
                        top: Some(vec![]),
                        bottom: Some(original.iter().rev().cloned().collect()),
                        ..Action::new("choose")
                    },
                )
                .unwrap();
                assert_eq!(
                    g.players[0]
                        .deck
                        .iter()
                        .map(|c| c.id.clone())
                        .collect::<Vec<_>>(),
                    original.iter().rev().cloned().collect::<Vec<_>>()
                );
            }
            assert!(g.pending.is_none());
            assert_eq!(g.players[0].hand.len(), hand_count);
            assert_eq!(g.status, "playing");
            assert!(!g.players[0].eliminated);
        }
    }

    #[test]
    fn rescue_forecast_forecast_waits_for_normal_reveal_after_secret_deployment() {
        let mut g = game("duel");
        let source = hand(&mut g, "JC104", 0);
        resource(&mut g, 0, "JC104", 3);
        g.apply(
            0,
            Action {
                card_id: Some(source),
                region: Some(0),
                ..Action::new("conceal")
            },
        )
        .unwrap();
        pass_stack(&mut g);
        assert!(g.pending.is_none());
        assert_eq!(g.resources(0), 2);
        let hidden = g.regions[0]
            .cards
            .iter()
            .find(|c| c.definition == "JC104")
            .unwrap()
            .id
            .clone();
        g.apply(
            0,
            Action {
                card_id: Some(hidden),
                ..Action::new("reveal")
            },
        )
        .unwrap();
        assert_eq!(g.resources(0), 0);
        pass_stack(&mut g);
        accept(&mut g);
        let pending = g.pending.clone().unwrap();
        assert_eq!(pending.choice.options.len(), 3);
        let top = pending
            .choice
            .options
            .iter()
            .map(|o| o.id.clone())
            .collect();
        g.apply(
            0,
            Action {
                choice_id: Some(pending.choice.id),
                top: Some(top),
                bottom: Some(vec![]),
                ..Action::new("choose")
            },
        )
        .unwrap();
        assert!(g.pending.is_none() && g.stack.is_empty());
    }

    #[test]
    fn rescue_forecast_two_cards_are_legal_custom_deck_cards_and_shuffle_into_normal_start() {
        let mut draft = crate::deck::preset("watchers").unwrap();
        draft.id = "rescue-forecast-legal".into();
        draft.name = "白紫有限混搭".into();
        draft
            .cards
            .iter_mut()
            .find(|e| e.card_id == "JC125")
            .unwrap()
            .count -= 6;
        for id in ["JC075", "JC104"] {
            draft.cards.push(catalog::DeckEntry {
                card_id: id.into(),
                count: 3,
            });
        }
        assert!(crate::deck::validate(draft.clone()).is_ok());
        for id in ["JC075", "JC104"] {
            assert_eq!(card(id).deck_copy_limit, Some(3));
            let mut invalid = draft.clone();
            invalid
                .cards
                .iter_mut()
                .find(|e| e.card_id == id)
                .unwrap()
                .count = 4;
            invalid
                .cards
                .iter_mut()
                .find(|e| e.card_id == "JC125")
                .unwrap()
                .count -= 1;
            assert!(crate::deck::validate(invalid).unwrap_err().contains("同名"));
        }
        let mut g = Game::new_with_deck(
            "local".into(),
            "LOCAL".into(),
            "teams".into(),
            "P0".into(),
            draft.clone(),
            5531,
        )
        .unwrap();
        for seat in 1..4 {
            g.join_with_deck(format!("P{seat}"), draft.clone()).unwrap();
        }
        for seat in 0..4 {
            g.apply(seat, Action::new("ready")).unwrap();
        }
        g.apply(0, Action::new("start")).unwrap();
        for p in &g.players {
            assert_eq!(p.hand.len() + p.deck.len(), 50);
            for id in ["JC075", "JC104"] {
                assert_eq!(
                    p.hand
                        .iter()
                        .chain(&p.deck)
                        .filter(|c| c.definition == id)
                        .count(),
                    3
                );
            }
        }
    }

    #[test]
    fn fire_scholar_fire_has_exact_printed_fields_and_standard_region_payment() {
        assert_eq!(card("JC047").cost, 3);
        assert_eq!(card("JC047").loyalty, vec!["红色"]);
        assert_eq!(card("JC047").subtypes, vec!["灾难"]);
        let mut g = game("teams");
        let source = hand(&mut g, "JC047", 0);
        resource(&mut g, 0, "JC042", 3);
        let legal = g
            .view(0)
            .legal_actions
            .into_iter()
            .filter(|a| a.action.kind == "play" && a.action.card_id.as_deref() == Some(&source))
            .collect::<Vec<_>>();
        assert_eq!(
            legal
                .iter()
                .filter_map(|a| a.action.region)
                .collect::<BTreeSet<_>>(),
            (0..5).collect()
        );
        let play = Action {
            card_id: Some(source.clone()),
            region: Some(2),
            ..Action::new("play")
        };
        let mut rejected = g.clone();
        rejected.players[0].assets[0].exhausted = true;
        let before = serde_json::to_string(&rejected).unwrap();
        assert!(rejected.apply(0, play.clone()).is_err());
        assert_eq!(serde_json::to_string(&rejected).unwrap(), before);
        let mut wrong_loyalty = g.clone();
        wrong_loyalty.players[0].assets.clear();
        resource(&mut wrong_loyalty, 0, "JC125", 3);
        let before = serde_json::to_string(&wrong_loyalty).unwrap();
        assert!(wrong_loyalty.apply(0, play.clone()).is_err());
        assert_eq!(serde_json::to_string(&wrong_loyalty).unwrap(), before);
        g.apply(0, play).unwrap();
        assert_eq!(g.resources(0), 0);
        assert_eq!(g.stack.len(), 1);
        let mut responding = g.clone();
        let second = hand(&mut responding, "JC047", 2);
        resource(&mut responding, 2, "JC042", 3);
        let before = serde_json::to_string(&responding).unwrap();
        assert!(responding
            .apply(
                2,
                Action {
                    card_id: Some(second),
                    region: Some(2),
                    ..Action::new("play")
                }
            )
            .is_err());
        assert_eq!(serde_json::to_string(&responding).unwrap(), before);
    }
    #[test]
    fn fire_scholar_fire_damages_all_visible_seats_and_barriers_only_in_its_region() {
        let mut g = game("teams");
        let source = hand(&mut g, "JC047", 0);
        resource(&mut g, 0, "JC042", 3);
        let doomed = (0..4)
            .map(|seat| board(&mut g, if seat == 2 { "JZ08" } else { "JC125" }, seat, 2))
            .collect::<Vec<_>>();
        g.board_mut(&doomed[0]).unwrap().controller = 1;
        let survivor = board(&mut g, "JC059", 3, 2);
        for id in &doomed {
            let (region, card) = g.board(id).unwrap();
            let prior_damage = g.defense(card, region) - 1;
            g.board_mut(id).unwrap().damage = prior_damage;
        }
        let hidden = board(&mut g, "JC125", 2, 2);
        g.board_mut(&hidden).unwrap().face_down = true;
        let outside = board(&mut g, "JC125", 2, 3);
        let equip = g.make_card("BQ022", 1);
        let equip_id = equip.id.clone();
        g.attachments.push(Attachment {
            card: equip,
            host_id: doomed[0].clone(),
        });
        let survivor_equipment = g.make_card("BQ022", 3);
        g.attachments.push(Attachment {
            card: survivor_equipment,
            host_id: survivor.clone(),
        });
        g.apply(
            0,
            Action {
                card_id: Some(source),
                region: Some(2),
                ..Action::new("play")
            },
        )
        .unwrap();
        let encoded = serde_json::to_string(&g).unwrap();
        let mut g: Game = serde_json::from_str(&encoded).unwrap();
        pass_stack(&mut g);
        for (seat, id) in doomed.iter().enumerate() {
            assert!(g.board(id).is_none());
            assert!(g.players[seat]
                .graveyard
                .iter()
                .any(|c| c.definition == if seat == 2 { "JZ08" } else { "JC125" }));
        }
        assert_eq!(g.board(&survivor).unwrap().1.damage, 1);
        assert_eq!(g.board(&hidden).unwrap().1.damage, 0);
        assert!(
            g.board(&outside).is_some(),
            "Damage must stay in the selected region"
        );
        assert_eq!(g.board(&outside).unwrap().1.damage, 0);
        assert_eq!(g.attachments.len(), 1);
        assert_eq!(g.attachments[0].host_id, survivor);
        let returned = g.players[1]
            .hand
            .iter()
            .find(|c| c.definition == "BQ022")
            .unwrap();
        assert_ne!(returned.id, equip_id);
        assert!(g.players[0]
            .graveyard
            .iter()
            .any(|c| c.definition == "JC047"));
    }
    #[test]
    fn fire_scholar_fire_collects_current_characters_after_a_response_hide() {
        let mut g = game("teams");
        let source = hand(&mut g, "JC047", 0);
        resource(&mut g, 0, "JC042", 3);
        let victim = board(&mut g, "JC125", 2, 2);
        let neighbor = board(&mut g, "JC125", 3, 2);
        let chase = hand(&mut g, "JC063", 2);
        resource(&mut g, 2, "JC056", 2);
        g.apply(
            0,
            Action {
                card_id: Some(source),
                region: Some(2),
                ..Action::new("play")
            },
        )
        .unwrap();
        g.apply(0, Action::new("pass")).unwrap();
        g.apply(1, Action::new("pass")).unwrap();
        g.apply(
            2,
            Action {
                card_id: Some(chase),
                target_id: Some(victim.clone()),
                option: Some("hide".into()),
                ..Action::new("play")
            },
        )
        .unwrap();
        pass_stack(&mut g);
        assert!(g.board(&victim).is_none());
        assert!(g.board(&neighbor).is_none());
        let hidden = g.regions[2].cards.iter().find(|c| c.owner == 2).unwrap();
        assert!(hidden.face_down && hidden.id != victim);
        assert_eq!(hidden.damage, 0);
        assert_eq!(g.resources(0), 0);
    }
    #[test]
    fn fire_scholar_scholar_filters_printed_spell_cost_then_reveals_and_shuffles() {
        assert_eq!(card("JC007").cost, 4);
        assert_eq!(card("JC007").loyalty, vec!["黄色", "黄色"]);
        assert_eq!(card("JC007").subtypes, vec!["人类", "学者"]);
        assert_eq!(card("JC007").defense, Some(1));
        assert_eq!(card("JC007").permanent_icons.influence, 1);
        assert_eq!(card("JC007").temporary_icons.investigation, 1);
        assert_eq!(card("JC007").temporary_icons.influence, 1);
        let mut g = game("teams");
        let source = hand(&mut g, "JC007", 0);
        resource(&mut g, 0, "JC003", 4);
        g.players[0].deck = [
            "JC006", "XQ03", "JC092", "JC049", "JC047", "JC063", "JC125", "JC003",
        ]
        .into_iter()
        .map(|id| g.make_card(id, 0))
        .collect();
        let expected = g.players[0].deck[..4]
            .iter()
            .map(|c| c.id.clone())
            .collect::<BTreeSet<_>>();
        let picked = g.players[0].deck[1].id.clone();
        g.apply(
            0,
            Action {
                card_id: Some(source),
                region: Some(0),
                ..Action::new("deploy")
            },
        )
        .unwrap();
        pass_stack(&mut g);
        accept(&mut g);
        let pending = g.pending.clone().unwrap();
        assert_eq!(pending.choice.kind, "search");
        assert_eq!(
            pending
                .choice
                .options
                .iter()
                .map(|o| o.id.clone())
                .collect::<BTreeSet<_>>(),
            expected
        );
        for seat in 1..4 {
            let v = g.view(seat);
            assert!(v.pending_choice.is_none());
            let text = serde_json::to_string(&v).unwrap();
            for id in &expected {
                assert!(!text.contains(id));
            }
        }
        let invalid = g.players[0].deck[5].id.clone();
        let before = serde_json::to_string(&g).unwrap();
        assert!(g
            .apply(
                0,
                Action {
                    choice_id: Some(pending.choice.id),
                    selected: Some(vec![invalid]),
                    ..Action::new("choose")
                }
            )
            .is_err());
        assert_eq!(serde_json::to_string(&g).unwrap(), before);
        let random = g.random;
        let encoded = serde_json::to_string(&g).unwrap();
        let mut g: Game = serde_json::from_str(&encoded).unwrap();
        select(&mut g, vec![picked.clone()]);
        let found = g.players[0]
            .hand
            .iter()
            .find(|c| c.definition == "XQ03")
            .unwrap();
        assert_ne!(found.id, picked);
        assert!(!g.players[0].deck.iter().any(|c| c.id == picked));
        assert_ne!(g.random, random);
        for seat in 0..4 {
            assert!(g
                .view(seat)
                .log
                .iter()
                .any(|line| line.text.contains("展示检索的 力场束缚")));
        }
        assert_eq!(g.resources(0), 0);
    }
    #[test]
    fn fire_scholar_scholar_can_decline_entry_and_empty_search_still_shuffles() {
        for decline in [true, false] {
            let mut g = game("teams");
            let source = hand(&mut g, "JC007", 0);
            resource(&mut g, 0, "JC003", 4);
            g.players[0].deck = ["JC125", "JC003", "JC125"]
                .into_iter()
                .map(|id| g.make_card(id, 0))
                .collect();
            g.apply(
                0,
                Action {
                    card_id: Some(source),
                    region: Some(0),
                    ..Action::new("deploy")
                },
            )
            .unwrap();
            pass_stack(&mut g);
            let random = g.random;
            let deck = g.players[0].deck.clone();
            if decline {
                select(&mut g, vec![]);
                assert_eq!(g.random, random);
                assert_eq!(
                    serde_json::to_string(&g.players[0].deck).unwrap(),
                    serde_json::to_string(&deck).unwrap()
                );
            } else {
                accept(&mut g);
                assert_ne!(g.random, random);
                assert_eq!(g.players[0].deck.len(), 3);
            }
            assert!(g.pending.is_none() && g.stack.is_empty());
            assert_eq!(g.resources(0), 0);
        }
    }
    #[test]
    fn fire_scholar_scholar_secret_deployment_waits_for_normal_reveal_entry() {
        let mut g = game("teams");
        let source = hand(&mut g, "JC007", 0);
        resource(&mut g, 0, "JC003", 6);
        g.players[0].deck = ["JC006", "JC125"]
            .into_iter()
            .map(|id| g.make_card(id, 0))
            .collect();
        g.apply(
            0,
            Action {
                card_id: Some(source),
                region: Some(0),
                ..Action::new("conceal")
            },
        )
        .unwrap();
        pass_stack(&mut g);
        assert!(g.pending.is_none());
        assert_eq!(g.resources(0), 5);
        let hidden = g.regions[0]
            .cards
            .iter()
            .find(|c| c.definition == "JC007")
            .unwrap()
            .id
            .clone();
        g.apply(
            0,
            Action {
                card_id: Some(hidden),
                ..Action::new("reveal")
            },
        )
        .unwrap();
        pass_stack(&mut g);
        accept(&mut g);
        assert_eq!(g.pending.as_ref().unwrap().choice.kind, "search");
        assert_eq!(g.resources(0), 1);
    }

    #[test]
    fn fire_scholar_scholar_filter_accepts_books_and_legacy_spells_but_uses_printed_cost() {
        // Primitive fixture only: a book is not being added to the playable catalog.
        let rules::Op::Search { filter, .. } = &rules::definition("JC007").abilities[0].ops[0]
        else {
            panic!("Expected actual scholar search program")
        };
        assert!(filter.matches(card("XQ03")));
        assert!(filter.matches(card("JC092")));
        let mut book = card("JC006").clone();
        book.subtypes = vec!["书籍".into()];
        assert!(filter.matches(&book));
        book.cost = 3;
        assert!(!filter.matches(&book));
        let mut expensive_spell = card("JC006").clone();
        expensive_spell.cost = 3;
        assert!(!filter.matches(&expensive_spell));
        assert!(!filter.matches(card("JC063")));
        assert!(!filter.matches(card("JC125")));
        let decoded: rules::CardFilter =
            serde_json::from_str(&serde_json::to_string(filter).unwrap()).unwrap();
        assert!(decoded.matches(card("XQ03")));
        assert!(!decoded.matches(&expensive_spell));
        let old: rules::CardFilter = serde_json::from_str(r#"{"Kind":"attachment"}"#).unwrap();
        assert!(old.matches(card("BQ022")));
    }

    #[test]
    fn control_pair_gate_requires_a_visible_character_with_any_magic_domain() {
        assert_eq!(card("JC006").cost, 2);
        assert_eq!(card("JC006").loyalty, vec!["黄色"]);
        assert_eq!(card("JC006").subtypes, vec!["法术", "异界"]);
        assert_eq!(card("JC006").magic_icon, rules::MagicIcon::Mind);
        let mut g = game("teams");
        let source = hand(&mut g, "JC006", 0);
        resource(&mut g, 0, "JC003", 2);
        let accepted = [
            board(&mut g, "JC001", 0, 0),
            board(&mut g, "JC002", 1, 1),
            board(&mut g, "JC085", 2, 2),
            board(&mut g, "BQ083", 3, 4),
        ];
        g.board_mut(&accepted[0]).unwrap().exhausted = true;
        let plain = board(&mut g, "JC125", 2, 3);
        let hidden = board(&mut g, "JC001", 1, 4);
        g.board_mut(&hidden).unwrap().face_down = true;
        let equipment = g.make_card("BQ022", 2);
        let equipment_id = equipment.id.clone();
        g.attachments.push(Attachment {
            card: equipment,
            host_id: accepted[1].clone(),
        });
        let targets = g
            .view(0)
            .legal_actions
            .iter()
            .filter(|a| {
                a.action.kind == "play" && a.action.card_id.as_deref() == Some(source.as_str())
            })
            .filter_map(|a| a.action.target_id.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(targets, accepted.iter().cloned().collect());
        for target in [plain, hidden, equipment_id] {
            let before = serde_json::to_string(&g).unwrap();
            assert!(g
                .apply(
                    0,
                    Action {
                        card_id: Some(source.clone()),
                        target_id: Some(target),
                        ..Action::new("play")
                    }
                )
                .is_err());
            assert_eq!(serde_json::to_string(&g).unwrap(), before);
        }
    }
    #[test]
    fn control_pair_gate_pays_two_returns_to_owner_and_recycles_normal_attachment() {
        let mut g = game("teams");
        let source = hand(&mut g, "JC006", 0);
        let host = board(&mut g, "JC002", 2, 4);
        g.board_mut(&host).unwrap().controller = 0;
        let equipment = g.make_card("BQ022", 3);
        let equipment_id = equipment.id.clone();
        g.attachments.push(Attachment {
            card: equipment,
            host_id: host.clone(),
        });
        assert!(g.attachment_host_valid(&g.attachments[0]));
        resource(&mut g, 0, "JC085", 2);
        let before = serde_json::to_string(&g).unwrap();
        assert!(g
            .apply(
                0,
                Action {
                    card_id: Some(source.clone()),
                    target_id: Some(host.clone()),
                    ..Action::new("play")
                }
            )
            .unwrap_err()
            .contains("忠诚"));
        assert_eq!(serde_json::to_string(&g).unwrap(), before);
        resource(&mut g, 0, "JC003", 2);
        g.apply(
            0,
            Action {
                card_id: Some(source),
                target_id: Some(host.clone()),
                ..Action::new("play")
            },
        )
        .unwrap();
        assert_eq!(g.resources(0), 2);
        assert!(g.board(&host).is_some());
        let mut g: Game = serde_json::from_str(&serde_json::to_string(&g).unwrap()).unwrap();
        pass_stack(&mut g);
        assert!(g.board(&host).is_none() && g.attachments.is_empty());
        let returned = g.players[2].hand.last().unwrap();
        assert_eq!(
            (
                returned.definition.as_str(),
                returned.owner,
                returned.controller
            ),
            ("JC002", 2, 2)
        );
        assert_ne!(returned.id, host);
        let recycled = g.players[3].hand.last().unwrap();
        assert_eq!(
            (
                recycled.definition.as_str(),
                recycled.owner,
                recycled.controller
            ),
            ("BQ022", 3, 3)
        );
        assert_ne!(recycled.id, equipment_id);
        assert_eq!(g.players[0].graveyard.last().unwrap().definition, "JC006");
        assert_eq!(g.resources(0), 2);
    }
    #[test]
    fn control_pair_gate_revalidates_after_response_hide_without_refunding_payment() {
        let mut g = game("duel");
        let target = board(&mut g, "JC001", 1, 0);
        let gate = hand(&mut g, "JC006", 0);
        let chase = hand(&mut g, "JC063", 1);
        resource(&mut g, 0, "JC003", 2);
        resource(&mut g, 1, "JC063", 2);
        g.apply(
            0,
            Action {
                card_id: Some(gate),
                target_id: Some(target.clone()),
                ..Action::new("play")
            },
        )
        .unwrap();
        g.apply(0, Action::new("pass")).unwrap();
        g.apply(
            1,
            Action {
                card_id: Some(chase),
                target_id: Some(target.clone()),
                option: Some("hide".into()),
                ..Action::new("play")
            },
        )
        .unwrap();
        let mut g: Game = serde_json::from_str(&serde_json::to_string(&g).unwrap()).unwrap();
        pass_stack(&mut g);
        assert!(g.board(&target).is_none());
        let hidden = g.regions[0]
            .cards
            .iter()
            .find(|c| c.definition == "JC001")
            .unwrap();
        assert!(hidden.face_down && hidden.id != target);
        assert!(!g.players[1].hand.iter().any(|c| c.definition == "JC001"));
        assert_eq!(g.resources(0), 0);
        assert_eq!(g.players[0].graveyard.last().unwrap().definition, "JC006");
    }
    #[test]
    fn control_pair_lawyer_is_public_and_hides_only_another_actor_controlled_character() {
        assert_eq!(card("XQ16").cost, 2);
        assert_eq!(card("XQ16").loyalty, vec!["蓝色"]);
        assert_eq!(card("XQ16").subtypes, vec!["人类", "律师"]);
        assert_eq!(card("XQ16").magic_icon, rules::MagicIcon::None);
        assert_eq!(card("XQ16").permanent_icons.influence, 1);
        let mut g = game("teams");
        let source = hand(&mut g, "XQ16", 0);
        let own = board(&mut g, "JC088", 0, 2);
        let ally = board(&mut g, "JC001", 2, 4);
        g.board_mut(&ally).unwrap().controller = 0;
        let enemy = board(&mut g, "JC085", 0, 1);
        g.board_mut(&enemy).unwrap().controller = 2;
        let public_ally = board(&mut g, "LC21", 1, 3);
        g.board_mut(&public_ally).unwrap().controller = 0;
        let teammate = board(&mut g, "JC085", 1, 4);
        let hidden = board(&mut g, "JC125", 1, 0);
        g.board_mut(&hidden).unwrap().face_down = true;
        resource(&mut g, 0, "XQ16", 2);
        let before = serde_json::to_string(&g).unwrap();
        assert!(g
            .apply(
                0,
                Action {
                    card_id: Some(source.clone()),
                    region: Some(0),
                    ..Action::new("conceal")
                }
            )
            .is_err());
        assert_eq!(serde_json::to_string(&g).unwrap(), before);
        g.apply(
            0,
            Action {
                card_id: Some(source.clone()),
                region: Some(0),
                ..Action::new("deploy")
            },
        )
        .unwrap();
        assert_eq!(g.resources(0), 0);
        pass_stack(&mut g);
        let new_source = g.regions[0]
            .cards
            .iter()
            .find(|c| c.definition == "XQ16")
            .unwrap()
            .id
            .clone();
        assert_ne!(source, new_source);
        let pending = g.pending.clone().unwrap();
        let targets = pending
            .choice
            .options
            .iter()
            .map(|o| o.id.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            targets,
            [own, ally.clone(), public_ally.clone()]
                .into_iter()
                .collect()
        );
        for target in [new_source.clone(), enemy, hidden, teammate.clone()] {
            let before = serde_json::to_string(&g).unwrap();
            assert!(g
                .apply(
                    0,
                    Action {
                        choice_id: Some(pending.choice.id.clone()),
                        selected: Some(vec![target]),
                        ..Action::new("choose")
                    }
                )
                .is_err());
            assert_eq!(serde_json::to_string(&g).unwrap(), before);
        }
        // Public forbids secret deployment; a legal effect may still hide it.
        let mut public_branch = g.clone();
        select(&mut public_branch, vec![public_ally.clone()]);
        pass_stack(&mut public_branch);
        assert!(public_branch.board(&public_ally).is_none());
        assert!(public_branch.regions[3]
            .cards
            .iter()
            .any(|c| c.definition == "LC21" && c.face_down));
        select(&mut g, vec![ally.clone()]);
        assert!(g.board(&ally).is_some());
        let mut g: Game = serde_json::from_str(&serde_json::to_string(&g).unwrap()).unwrap();
        pass_stack(&mut g);
        assert!(g.board(&ally).is_none());
        let fresh = g.regions[4]
            .cards
            .iter()
            .find(|c| c.definition == "JC001")
            .unwrap();
        assert!(fresh.face_down && fresh.id != ally);
        assert_eq!((fresh.owner, fresh.controller), (2, 0));
        for viewer in 0..4 {
            let view = g.view(viewer);
            let c = view.regions[4]
                .characters
                .iter()
                .find(|c| c.instance_id == fresh.id)
                .unwrap();
            assert_eq!(
                c.card_id.as_deref(),
                if viewer == 0 { Some("JC001") } else { None }
            );
        }
        assert!(g.board(&new_source).is_some());
        assert!(!g.board(&teammate).unwrap().1.face_down);
    }
    #[test]
    fn control_pair_lawyer_can_decline_and_without_another_friend_has_no_trigger_choice() {
        for with_friend in [false, true] {
            let mut g = game("duel");
            let enemy = board(&mut g, "JC001", 1, 0);
            let friend = with_friend.then(|| board(&mut g, "JC125", 0, 1));
            let source = hand(&mut g, "XQ16", 0);
            resource(&mut g, 0, "XQ16", 2);
            g.apply(
                0,
                Action {
                    card_id: Some(source),
                    region: Some(0),
                    ..Action::new("deploy")
                },
            )
            .unwrap();
            pass_stack(&mut g);
            if with_friend {
                select(&mut g, vec![]);
            }
            assert!(g.pending.is_none() && g.stack.is_empty());
            assert!(g.board(&enemy).is_some());
            if let Some(friend) = friend {
                assert!(!g.board(&friend).unwrap().1.face_down);
            }
            assert_eq!(g.resources(0), 0);
        }
    }
    #[test]
    fn control_pair_lawyer_revalidates_after_enemy_response_without_rehiding_new_instance() {
        let mut g = game("teams");
        let target = board(&mut g, "JC001", 1, 4);
        g.board_mut(&target).unwrap().controller = 0;
        let lawyer = hand(&mut g, "XQ16", 0);
        let chase = hand(&mut g, "JC063", 2);
        resource(&mut g, 0, "XQ16", 2);
        resource(&mut g, 2, "JC063", 2);
        g.apply(
            0,
            Action {
                card_id: Some(lawyer),
                region: Some(0),
                ..Action::new("deploy")
            },
        )
        .unwrap();
        pass_stack(&mut g);
        select(&mut g, vec![target.clone()]);
        for seat in [0, 1] {
            g.apply(seat, Action::new("pass")).unwrap();
        }
        g.apply(
            2,
            Action {
                card_id: Some(chase),
                target_id: Some(target.clone()),
                option: Some("hide".into()),
                ..Action::new("play")
            },
        )
        .unwrap();
        // Resolve only the response, then retain the old target guard in a saved frame.
        while g.stack.len() > 1 {
            let seat = g
                .living(g.priority_team)
                .into_iter()
                .find(|s| !g.passed.contains(s))
                .unwrap();
            g.apply(seat, Action::new("pass")).unwrap();
        }
        let fresh = g.regions[4]
            .cards
            .iter()
            .find(|c| c.definition == "JC001")
            .unwrap()
            .id
            .clone();
        assert_ne!(fresh, target);
        let mut g: Game = serde_json::from_str(&serde_json::to_string(&g).unwrap()).unwrap();
        pass_stack(&mut g);
        assert_eq!(g.regions[4].cards[0].id, fresh);
        assert!(g.regions[4].cards[0].face_down);
        assert_eq!(g.resources(0), 0);
    }

    #[test]
    fn control_pair_lawyer_rejects_target_control_transferred_to_teammate_during_response() {
        // Explicit primitive arrangement: the current pool has no control-transfer card.
        let mut g = game("teams");
        let target = board(&mut g, "JC001", 0, 4);
        let lawyer = hand(&mut g, "XQ16", 0);
        resource(&mut g, 0, "XQ16", 2);
        g.apply(
            0,
            Action {
                card_id: Some(lawyer),
                region: Some(0),
                ..Action::new("deploy")
            },
        )
        .unwrap();
        pass_stack(&mut g);
        select(&mut g, vec![target.clone()]);
        g.board_mut(&target).unwrap().controller = 1;
        let mut g: Game = serde_json::from_str(&serde_json::to_string(&g).unwrap()).unwrap();
        pass_stack(&mut g);
        let stayed = g.board(&target).unwrap().1;
        assert!(!stayed.face_down);
        assert_eq!((stayed.owner, stayed.controller), (0, 1));
        assert_eq!(g.resources(0), 0);
    }

    #[test]
    fn assassin_reveal_uses_printed_cost_guard_for_all_four_owners_in_its_region() {
        let mut g = game("teams");
        let source = board(&mut g, "JC088", 0, 2);
        g.board_mut(&source).unwrap().face_down = true;
        let accepted = [
            board(&mut g, "JC125", 0, 2),
            board(&mut g, "JC084", 1, 2),
            board(&mut g, "JC085", 2, 2),
            board(&mut g, "JC084", 3, 2),
        ];
        let expensive = board(&mut g, "JC086", 2, 2);
        // A payment modifier cannot turn an expensive print into a legal target.
        g.modifiers.push(CostModifier {
            actor: 2,
            filter: rules::CardFilter::Any,
            amount: 20,
            expires_turn: g.turn,
            uses: 1,
        });
        assert_eq!(g.effective_cost(2, g.board(&expensive).unwrap().1), 0);
        let hidden = board(&mut g, "JC084", 2, 2);
        g.board_mut(&hidden).unwrap().face_down = true;
        let outside = board(&mut g, "JC084", 3, 3);
        resource(&mut g, 0, "JC088", 3);
        g.apply(
            0,
            Action {
                card_id: Some(source.clone()),
                ..Action::new("reveal")
            },
        )
        .unwrap();
        pass_stack(&mut g);
        let pending = g.pending.as_ref().unwrap();
        assert_eq!(pending.choice.kind, "trigger");
        assert_eq!(
            pending
                .choice
                .options
                .iter()
                .map(|o| o.id.clone())
                .collect::<BTreeSet<_>>(),
            accepted.iter().cloned().collect()
        );
        assert_eq!(card("JC125").cost, 0);
        assert_eq!(card("JC085").cost, 2);
        assert!(card("JC086").cost > 2);
        select(&mut g, vec![accepted[3].clone()]);
        assert_eq!(
            g.stack.last().unwrap().frame.as_ref().unwrap().targets[0]
                .spec
                .printed_cost_max,
            Some(2)
        );
        let json = serde_json::to_string(&g).unwrap();
        let mut g: Game = serde_json::from_str(&json).unwrap();
        pass_stack(&mut g);
        assert!(g.board(&accepted[3]).is_none());
        assert_eq!(g.players[3].graveyard.last().unwrap().definition, "JC084");
        for id in accepted
            .iter()
            .take(3)
            .chain([&expensive, &hidden, &outside].into_iter())
        {
            assert!(g.board(id).is_some());
        }

        let mut ordinary = game("duel");
        let victim = board(&mut ordinary, "JC085", 1, 0);
        let source = hand(&mut ordinary, "JC088", 0);
        resource(&mut ordinary, 0, "JC088", 3);
        ordinary
            .apply(
                0,
                Action {
                    card_id: Some(source),
                    region: Some(0),
                    ..Action::new("deploy")
                },
            )
            .unwrap();
        pass_stack(&mut ordinary);
        assert!(ordinary.pending.is_none() && ordinary.board(&victim).is_some());
    }

    #[test]
    fn assassin_paid_hide_preserves_exhaustion_and_returns_attachment_to_its_owner() {
        let mut g = game("duel");
        let source = board(&mut g, "JC088", 0, 0);
        g.board_mut(&source).unwrap().exhausted = true;
        resource(&mut g, 0, "JC088", 1);
        let rejected = serde_json::to_string(&g).unwrap();
        let action = Action {
            card_id: Some(source.clone()),
            ability_id: Some("hide-self".into()),
            ..Action::new("activate")
        };
        assert!(g.apply(0, action.clone()).is_err());
        assert_eq!(serde_json::to_string(&g).unwrap(), rejected);
        resource(&mut g, 0, "JC088", 1);
        let equipment = g.make_card("BQ022", 1);
        g.attachments.push(Attachment {
            card: equipment,
            host_id: source.clone(),
        });
        g.apply(0, action).unwrap();
        assert_eq!(g.resources(0), 0);
        assert!(g.board(&source).is_some());
        pass_stack(&mut g);
        assert!(g.board(&source).is_none());
        let hidden = g.regions[0]
            .cards
            .iter()
            .find(|c| c.definition == "JC088")
            .unwrap();
        assert!(hidden.face_down && hidden.exhausted);
        assert_eq!(hidden.controller, 0);
        assert!(g.attachments.is_empty());
        assert_eq!(g.players[1].hand.last().unwrap().definition, "BQ022");
        assert!(!g
            .legal_actions(0)
            .iter()
            .any(|a| a.action.kind == "activate" && a.action.card_id.as_ref() == Some(&hidden.id)));
    }

    #[test]
    fn assassin_reveal_target_that_hides_in_response_is_not_destroyed_or_repaid() {
        let mut g = game("duel");
        let source = board(&mut g, "JC088", 0, 0);
        g.board_mut(&source).unwrap().face_down = true;
        let target = board(&mut g, "JC085", 1, 0);
        resource(&mut g, 0, "JC088", 3);
        resource(&mut g, 1, "JC063", 4);
        let hide = hand(&mut g, "JC063", 1);
        g.apply(
            0,
            Action {
                card_id: Some(source),
                ..Action::new("reveal")
            },
        )
        .unwrap();
        pass_stack(&mut g);
        select(&mut g, vec![target.clone()]);
        g.apply(0, Action::new("pass")).unwrap();
        g.apply(
            1,
            Action {
                card_id: Some(hide),
                target_id: Some(target.clone()),
                option: Some("hide".into()),
                ..Action::new("play")
            },
        )
        .unwrap();
        pass_stack(&mut g);
        assert!(g.board(&target).is_none());
        let saved = g.regions[0]
            .cards
            .iter()
            .find(|c| c.definition == "JC085")
            .unwrap();
        assert_ne!(saved.id, target);
        assert!(saved.face_down);
        assert_eq!(g.resources(0), 0);
        assert!(!g.players[1]
            .graveyard
            .iter()
            .any(|c| c.definition == "JC085"));
    }

    #[test]
    fn assassin_reveal_can_decline_or_have_no_target_and_keeps_exhaustion() {
        for exhausted in [false, true] {
            for eligible in [false, true] {
                let mut g = game("duel");
                let old_source = board(&mut g, "JC088", 0, 0);
                let source = g.board_mut(&old_source).unwrap();
                source.face_down = true;
                source.exhausted = exhausted;
                let victim = board(&mut g, if eligible { "JC085" } else { "JC086" }, 1, 0);
                resource(&mut g, 0, "JC088", 3);
                g.apply(
                    0,
                    Action {
                        card_id: Some(old_source.clone()),
                        ..Action::new("reveal")
                    },
                )
                .unwrap();
                assert_eq!(g.resources(0), 0); // Paid before the response window.
                assert!(g.board(&old_source).is_none());
                pass_stack(&mut g);
                if eligible {
                    assert_eq!(g.pending.as_ref().unwrap().choice.min, Some(0));
                    select(&mut g, vec![]);
                }
                assert!(g.pending.is_none() && g.stack.is_empty());
                assert!(g.board(&victim).is_some());
                let face_up = g.regions[0]
                    .cards
                    .iter()
                    .find(|c| c.definition == "JC088")
                    .unwrap();
                assert_ne!(face_up.id, old_source);
                assert!(!face_up.face_down);
                assert_eq!(face_up.exhausted, exhausted);
                assert_eq!(g.resources(0), 0);
            }
        }
    }

    #[test]
    fn assassin_self_hide_responds_to_its_trigger_without_cancelling_the_paid_frame() {
        let mut g = game("teams");
        let old_source = board(&mut g, "JC088", 0, 0);
        g.board_mut(&old_source).unwrap().face_down = true;
        let victim = board(&mut g, "JC085", 3, 0);
        resource(&mut g, 0, "JC088", 5);
        g.apply(
            0,
            Action {
                card_id: Some(old_source),
                ..Action::new("reveal")
            },
        )
        .unwrap();
        assert_eq!(g.resources(0), 2);
        pass_stack(&mut g);
        select(&mut g, vec![victim.clone()]);
        let face_up = g.regions[0]
            .cards
            .iter()
            .find(|c| c.definition == "JC088")
            .unwrap()
            .id
            .clone();
        g.apply(
            0,
            Action {
                card_id: Some(face_up.clone()),
                ability_id: Some("hide-self".into()),
                ..Action::new("activate")
            },
        )
        .unwrap();
        assert_eq!(g.resources(0), 0);
        assert!(g.board(&face_up).is_some()); // Effect waits on the stack; payment does not.
        assert_eq!(g.stack.len(), 2);
        assert!(
            matches!(&g.stack.last().unwrap().frame.as_ref().unwrap().already_paid[..], [PaidCost::Assets(ids)] if ids.len() == 2)
        );
        let mut g: Game = serde_json::from_str(&serde_json::to_string(&g).unwrap()).unwrap();
        pass_stack(&mut g);
        assert!(g.board(&face_up).is_none() && g.board(&victim).is_none());
        assert_eq!(g.players[3].graveyard.last().unwrap().definition, "JC085");
        let hidden = g.regions[0]
            .cards
            .iter()
            .find(|c| c.definition == "JC088")
            .unwrap();
        assert!(hidden.face_down);
        assert_ne!(hidden.id, face_up);
        assert_eq!(g.resources(0), 0);
        for seat in 0..4 {
            let view = serde_json::to_value(g.view(seat)).unwrap();
            let projected = view["regions"][0]["characters"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["instanceId"] == hidden.id)
                .unwrap();
            if seat == 0 {
                assert_eq!(projected["cardId"], "JC088");
            } else {
                assert_eq!(projected["name"], "暗藏者");
                for secret in [
                    "cardId", "text", "cost", "icons", "defense", "color", "magic",
                ] {
                    assert!(projected.get(secret).is_none());
                }
            }
        }
    }

    #[test]
    fn assassin_self_hide_uses_controller_payment_and_privacy_after_control_transfer() {
        let mut g = game("teams");
        // Synthetic control transfer: the admitted pool has no control-stealing card.
        let source = board(&mut g, "JC088", 1, 0);
        g.board_mut(&source).unwrap().controller = 0;
        resource(&mut g, 0, "JC088", 2);
        resource(&mut g, 1, "JC088", 2);
        g.apply(
            0,
            Action {
                card_id: Some(source.clone()),
                ability_id: Some("hide-self".into()),
                ..Action::new("activate")
            },
        )
        .unwrap();
        assert_eq!((g.resources(0), g.resources(1)), (0, 2));
        pass_stack(&mut g);
        let hidden = &g.regions[0].cards[0];
        assert_eq!((hidden.owner, hidden.controller), (1, 0));
        assert!(hidden.face_down);
        assert_ne!(hidden.id, source);
        assert_eq!(
            g.view(0).regions[0].characters[0].card_id.as_deref(),
            Some("JC088")
        );
        for seat in 1..4 {
            assert!(g.view(seat).regions[0].characters[0].card_id.is_none());
        }
    }

    #[test]
    fn detective_only_reveal_triggers_once_can_decline_and_handles_exhausted_or_no_target() {
        let mut deployed = game("duel");
        let hidden = board(&mut deployed, "LC21", 1, 0);
        deployed.board_mut(&hidden).unwrap().face_down = true;
        let detective = hand(&mut deployed, "JC058", 0);
        resource(&mut deployed, 0, "JC058", 3);
        deployed
            .apply(
                0,
                Action {
                    card_id: Some(detective),
                    region: Some(0),
                    ..Action::new("deploy")
                },
            )
            .unwrap();
        pass_stack(&mut deployed);
        assert!(deployed.pending.is_none()); // Ordinary face-up deployment is not Reveal.
        assert!(deployed.board(&hidden).is_some());

        for (has_target, exhausted) in [(true, false), (true, true), (false, false), (false, true)]
        {
            let mut g = game("duel");
            // Explicit initial layout; every declaration and choice below is an Action.
            let detective = board(&mut g, "JC058", 0, 0);
            let c = g.board_mut(&detective).unwrap();
            c.face_down = true;
            c.exhausted = exhausted;
            let target = has_target.then(|| {
                let id = board(&mut g, "LC21", 1, 0);
                g.board_mut(&id).unwrap().face_down = true;
                id
            });
            resource(&mut g, 0, "JC058", 3);
            g.apply(
                0,
                Action {
                    card_id: Some(detective.clone()),
                    ..Action::new("reveal")
                },
            )
            .unwrap();
            assert!(g.pending.is_none()); // The Reveal card must first resolve.
            pass_stack(&mut g);
            let revealed = g.regions[0]
                .cards
                .iter()
                .find(|c| c.definition == "JC058")
                .unwrap();
            assert_ne!(revealed.id, detective);
            assert!(!revealed.face_down);
            assert_eq!(revealed.exhausted, exhausted);
            assert_eq!(g.resources(0), 0);
            if let Some(target) = target {
                let pending = g.pending.as_ref().unwrap();
                assert_eq!(pending.choice.kind, "trigger");
                assert_eq!(pending.choice.options.len(), 1);
                assert_eq!(pending.choice.options[0].id, target);
                assert_eq!(pending.choice.allow_decline, Some(true));
                select(&mut g, vec![]);
                assert!(g.board(&target).is_some());
            } else {
                assert!(g.pending.is_none()); // No legal hidden target means no declaration.
            }
            assert!(g.stack.is_empty() && g.effects.is_empty() && g.pending.is_none());
        }
    }
    #[test]
    fn detective_targets_all_sides_same_region_without_private_leaks_or_hidden_death() {
        let mut g = game("teams");
        let detective = board(&mut g, "JC058", 0, 2);
        g.board_mut(&detective).unwrap().face_down = true;
        let own = board(&mut g, "JC125", 0, 2);
        let teammate = board(&mut g, "LC21", 1, 2);
        let enemy = board(&mut g, "XQ12", 2, 2);
        let distant = board(&mut g, "LC20", 3, 1);
        let already_revealing = board(&mut g, "JC125", 3, 2);
        for id in [&own, &teammate, &enemy, &distant, &already_revealing] {
            g.board_mut(id).unwrap().face_down = true;
        }
        let face_up = board(&mut g, "LC23", 2, 2);
        resource(&mut g, 0, "JC058", 3);
        resource(&mut g, 3, "JC125", 1);
        for seat in [0, 1] {
            g.apply(seat, Action::new("pass")).unwrap();
        }
        g.apply(
            3,
            Action {
                card_id: Some(already_revealing.clone()),
                ..Action::new("reveal")
            },
        )
        .unwrap();
        let revealing_stack_card = g.stack[0].card.as_ref().unwrap().id.clone();
        for seat in [2, 3] {
            g.apply(seat, Action::new("pass")).unwrap();
        }
        g.apply(
            0,
            Action {
                card_id: Some(detective),
                ..Action::new("reveal")
            },
        )
        .unwrap();
        for seat in [0, 1, 2, 3] {
            g.apply(seat, Action::new("pass")).unwrap();
        }
        assert_eq!(g.stack.len(), 1); // The earlier Reveal is still a stack card, not a hidden entity.
        let choice = g.pending.as_ref().unwrap().choice.clone();
        assert_eq!(
            choice
                .options
                .iter()
                .map(|o| o.id.clone())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([own.clone(), teammate.clone(), enemy.clone()])
        );
        for excluded in [
            &distant,
            &face_up,
            &already_revealing,
            &revealing_stack_card,
        ] {
            assert!(choice.options.iter().all(|o| o.id != *excluded));
        }
        for id in [&teammate, &enemy] {
            let c = choice
                .options
                .iter()
                .find(|o| o.id == *id)
                .unwrap()
                .card
                .as_ref()
                .unwrap();
            assert_eq!(c.name, "暗藏者");
            assert!(c.card_id.is_none() && c.text.is_none() && c.cost.is_none());
        }
        assert_eq!(
            choice
                .options
                .iter()
                .find(|o| o.id == own)
                .unwrap()
                .card
                .as_ref()
                .unwrap()
                .card_id
                .as_deref(),
            Some("JC125")
        );
        for viewer in [1, 2, 3] {
            let view = g.view(viewer);
            assert!(view.pending_choice.is_none());
            assert_eq!(view.waiting_choice.unwrap().player_id, "p0");
        }
        let before = serde_json::to_string(&g).unwrap();
        assert!(g
            .apply(
                0,
                Action {
                    choice_id: Some(choice.id.clone()),
                    selected: Some(vec![face_up]),
                    ..Action::new("choose")
                }
            )
            .is_err());
        assert_eq!(serde_json::to_string(&g).unwrap(), before);

        for (target, owner) in [(own, 0), (teammate, 1), (enemy, 2)] {
            let mut branch = Game::from_persisted(&before).unwrap();
            select(&mut branch, vec![target.clone()]);
            for viewer in 0..4 {
                let view = branch.view(viewer);
                let summary = &view.stack.last().unwrap().target_summaries[0];
                assert!(summary.label.starts_with("暗藏者·"));
                assert_eq!(summary.kind, "hidden");
                assert!(summary.valid);
            }
            for seat in [0, 1, 2, 3] {
                branch.apply(seat, Action::new("pass")).unwrap();
            }
            assert_eq!(branch.stack.len(), 1); // Full-team passing resolves only the top ability.
            assert!(branch.pending.is_none()); // XQ12 is hidden, so its printed Death does not trigger.
            assert!(branch.board(&target).is_none());
            assert_eq!(branch.players[owner].graveyard.len(), 1);
            assert_ne!(branch.players[owner].graveyard[0].id, target);
            assert!(!branch.players[owner].graveyard[0].face_down);
            pass_stack(&mut branch);
            assert!(branch.pending.is_none());
        }
    }
    #[test]
    fn detective_target_reveal_cancels_without_refund_but_source_death_keeps_effect() {
        for kill_source in [false, true] {
            let mut g = game("duel");
            let detective = board(&mut g, "JC058", 0, 0);
            g.board_mut(&detective).unwrap().face_down = true;
            let target = board(&mut g, if kill_source { "XQ12" } else { "LC21" }, 1, 0);
            g.board_mut(&target).unwrap().face_down = true;
            resource(&mut g, 0, "JC058", 3);
            let murder = if kill_source {
                resource(&mut g, 1, "JC091", 3);
                Some(hand(&mut g, "JC091", 1))
            } else {
                resource(&mut g, 1, "JC125", card("LC21").cost as usize);
                None
            };
            g.apply(
                0,
                Action {
                    card_id: Some(detective),
                    ..Action::new("reveal")
                },
            )
            .unwrap();
            pass_stack(&mut g);
            let source = g.regions[0]
                .cards
                .iter()
                .find(|c| c.definition == "JC058")
                .unwrap()
                .id
                .clone();
            select(&mut g, vec![target.clone()]);
            assert_eq!(g.stack.len(), 1);
            g.apply(0, Action::new("pass")).unwrap();
            let response = if let Some(murder) = murder {
                Action {
                    card_id: Some(murder),
                    target_id: Some(source.clone()),
                    ..Action::new("play")
                }
            } else {
                Action {
                    card_id: Some(target.clone()),
                    ..Action::new("reveal")
                }
            };
            g.apply(1, response).unwrap();
            if !kill_source {
                let summary = &g.view(0).stack[0].target_summaries[0];
                assert_eq!(summary.status, "missing"); // Declaration already removed the old hidden entity.
                assert!(!summary.valid);
            }
            g.apply(1, Action::new("pass")).unwrap();
            g.apply(0, Action::new("pass")).unwrap();
            assert_eq!(g.stack.len(), 1);
            assert_eq!(g.resources(0), 0);
            assert_eq!(g.resources(1), 0);
            if kill_source {
                assert!(g.board(&source).is_none());
                assert!(g.view(1).stack[0].target_summaries[0].valid);
            }
            let persisted = serde_json::to_string(&g).unwrap();
            let mut resumed = Game::from_persisted(&persisted).unwrap();
            assert_eq!(serde_json::to_string(&resumed).unwrap(), persisted);
            pass_stack(&mut resumed);
            assert!(resumed.stack.is_empty() && resumed.pending.is_none());
            if kill_source {
                assert!(resumed.board(&target).is_none());
                assert_eq!(resumed.players[0].graveyard[0].definition, "JC058");
                assert_eq!(resumed.players[1].graveyard.len(), 2); // Murder then the destroyed hidden XQ12.
            } else {
                assert!(resumed.players[1].graveyard.is_empty());
                let new_target = resumed.regions[0]
                    .cards
                    .iter()
                    .find(|c| c.definition == "LC21")
                    .unwrap();
                assert_ne!(new_target.id, target);
                assert!(!new_target.face_down);
                assert!(resumed
                    .log
                    .iter()
                    .any(|e| e.text.contains("坚毅的刑警：原目标") && e.text.contains("费用不退")));
            }
        }
    }
    #[test]
    fn teams_require_all_passes_and_own_action_resets_pass_records() {
        let mut g = game("teams");
        g.apply(0, Action::new("pass")).unwrap();
        assert_eq!(g.priority_team, 0);
        assert!(g.passed.contains(&0));
        let id = g.players[1].hand[0].id.clone();
        g.apply(
            1,
            Action {
                card_id: Some(id),
                ..Action::new("asset")
            },
        )
        .unwrap();
        assert!(g.passed.is_empty());
        g.apply(0, Action::new("pass")).unwrap();
        g.apply(1, Action::new("pass")).unwrap();
        assert_eq!(g.priority_team, 1);
        g.apply(2, Action::new("pass")).unwrap();
        assert_eq!(g.window, Some(Window::Action(0)));
        g.apply(3, Action::new("pass")).unwrap();
        assert_eq!(g.window, Some(Window::Action(1)));
    }
    #[test]
    fn distance_public_restriction_and_rejected_command_are_noop() {
        let mut g = game("teams");
        resource(&mut g, 0, "JC014", 5);
        let public = hand(&mut g, "JC014", 0);
        let before = serde_json::to_string(&g).unwrap();
        assert!(g
            .apply(
                0,
                Action {
                    card_id: Some(public.clone()),
                    region: Some(4),
                    ..Action::new("deploy")
                }
            )
            .is_err());
        assert_eq!(serde_json::to_string(&g).unwrap(), before);
        assert!(g
            .apply(
                0,
                Action {
                    card_id: Some(public),
                    region: Some(0),
                    ..Action::new("conceal")
                }
            )
            .is_err());
        assert_eq!(serde_json::to_string(&g).unwrap(), before);
    }
    #[test]
    fn icon_difference_investigation_sorts_then_draws_one() {
        let mut g = game("duel");
        board(&mut g, "LC24", 0, 0);
        board(&mut g, "LC24", 0, 0);
        let hand_before = g.players[0].hand.len();
        let deck_before = g.players[0].deck.len();
        g.begin_window(Window::Before(0, 0));
        g.close_window().unwrap();
        g.drive().unwrap();
        let p = g.pending.clone().unwrap();
        assert_eq!(p.choice.kind, "investigation");
        assert_eq!(p.choice.amount, Some(4));
        let ids = p
            .choice
            .options
            .iter()
            .map(|c| c.id.clone())
            .collect::<Vec<_>>();
        let expected = card(&g.players[0].deck[3].definition).id.clone();
        g.apply(
            0,
            Action {
                choice_id: Some(p.choice.id),
                top: Some(vec![ids[3].clone()]),
                bottom: Some(ids[..3].to_vec()),
                ..Action::new("choose")
            },
        )
        .unwrap();
        assert_eq!(g.players[0].hand.len(), hand_before + 1);
        assert_eq!(g.players[0].deck.len(), deck_before - 1);
        assert_eq!(g.players[0].hand.last().unwrap().definition, expected);
    }
    #[test]
    fn combat_difference_kill_guard_simultaneous_damage_and_barrier() {
        let mut g = game("duel");
        board(&mut g, "JC016", 0, 0);
        let guard = board(&mut g, "LC21", 1, 0);
        let target = board(&mut g, "LC24", 1, 0);
        let barrier = board(&mut g, "JZ08", 1, 0);
        g.board_mut(&guard).unwrap().exhausted = true;
        g.begin_window(Window::Before(0, 1));
        g.close_window().unwrap();
        g.drive().unwrap();
        let p = g.pending.clone().unwrap();
        assert_eq!(p.choice.amount, Some(2));
        let before = serde_json::to_string(&g).unwrap();
        assert!(g
            .apply(
                0,
                Action {
                    choice_id: Some(p.choice.id.clone()),
                    allocations: Some(BTreeMap::from([(target.clone(), 2)])),
                    ..Action::new("choose")
                }
            )
            .is_err());
        assert_eq!(serde_json::to_string(&g).unwrap(), before);
        g.apply(
            0,
            Action {
                choice_id: Some(p.choice.id),
                allocations: Some(BTreeMap::from([(guard.clone(), 1), (barrier.clone(), 1)])),
                ..Action::new("choose")
            },
        )
        .unwrap();
        assert!(g.board(&guard).is_none() && g.board(&barrier).is_none());
        assert!(g.board(&target).is_some());
    }
    #[test]
    fn influence_difference_cancels_enemy_and_privilege_requires_contribution() {
        let mut g = game("duel");
        board(&mut g, "LC23", 0, 0);
        board(&mut g, "LC22", 0, 0);
        g.regions[0].influence = [0, 1];
        g.begin_window(Window::Before(0, 2));
        g.close_window().unwrap();
        g.drive().unwrap();
        assert_eq!(g.regions[0].influence, [1, 0]);
        g.begin_window(Window::Before(1, 0));
        resource(&mut g, 0, "JC125", 2);
        assert!(g.apply(0, Action::new("privilege")).is_err());
        board(&mut g, "LC20", 0, 1);
        board(&mut g, "JC003", 1, 1);
        g.apply(0, Action::new("privilege")).unwrap();
        assert!(g.privilege_used);
        assert_eq!(g.pending.as_ref().unwrap().choice.amount, Some(1));
    }
    #[test]
    fn board_move_primitive_preserves_entity_without_enter_or_unique_events() {
        // Explicit primitive fixture: LC20 has no card ability that moves itself.
        let mut g = game("duel");
        board(&mut g, "JC059", 0, 0);
        board(&mut g, "JC059", 0, 1);
        let counselor = board(&mut g, "LC20", 0, 0);
        let c = g.board_mut(&counselor).unwrap();
        c.exhausted = true;
        c.damage = 1;
        let original = serde_json::to_string(g.board(&counselor).unwrap().1).unwrap();

        move_fixture(&mut g, &counselor, 1);
        let (region, c) = g.board(&counselor).unwrap();
        assert_eq!(region, 1);
        assert_eq!(serde_json::to_string(c).unwrap(), original);
        assert!(
            g.effects.is_empty(),
            "moving must not declare LC20's Enter ability"
        );

        let before_same_region = serde_json::to_string(&g).unwrap();
        move_fixture(&mut g, &counselor, 1);
        assert_eq!(serde_json::to_string(&g).unwrap(), before_same_region);

        let unique = board(&mut g, "LC23", 0, 0);
        move_fixture(&mut g, &unique, 1);
        assert!(g.effects.is_empty(), "moving must not re-run Unique checks");

        g.enter_triggers(0, "LC20", &counselor, false);
        assert_eq!(g.effects.len(), 1);
        assert!(matches!(
            g.effects.front(),
            Some(Effect::Declare { declaration })
                if declaration.ability.event == Some(Event::Enter)
        ));
        g.enter_triggers(0, "LC23", &unique, true);
        assert!(g
            .effects
            .iter()
            .any(|effect| matches!(effect, Effect::Unique { .. })));
        assert!(g.effects.iter().any(|effect| matches!(
            effect,
            Effect::Declare { declaration }
                if declaration.ability.event == Some(Event::Reveal)
        )));
    }

    #[test]
    fn surgeon_is_public_and_heal_pays_two_assets_and_exhausts_source_atomically() {
        let mut g = game("duel");
        let doctor = hand(&mut g, "LC19", 0);
        let target = board(&mut g, "JC059", 0, 0);
        g.board_mut(&target).unwrap().wounds = 1;
        resource(&mut g, 0, "JC125", 6);

        let before_conceal = serde_json::to_string(&g).unwrap();
        assert!(g
            .apply(
                0,
                Action {
                    card_id: Some(doctor.clone()),
                    region: Some(0),
                    ..Action::new("conceal")
                }
            )
            .is_err());
        assert_eq!(serde_json::to_string(&g).unwrap(), before_conceal);
        g.apply(
            0,
            Action {
                card_id: Some(doctor),
                region: Some(0),
                ..Action::new("deploy")
            },
        )
        .unwrap();
        pass_stack(&mut g);
        let doctor = g.regions[0]
            .cards
            .iter()
            .find(|c| c.definition == "LC19")
            .unwrap()
            .id
            .clone();
        let heal = Action {
            card_id: Some(doctor.clone()),
            target_id: Some(target.clone()),
            ability_id: Some("heal".into()),
            ..Action::new("activate")
        };
        let before_payment = g.resources(0);
        g.apply(0, heal.clone()).unwrap();
        assert_eq!(g.resources(0), before_payment - 2);
        assert!(g.board(&doctor).unwrap().1.exhausted);
        assert_eq!(g.board(&target).unwrap().1.wounds, 1);
        pass_stack(&mut g);
        assert_eq!(g.board(&target).unwrap().1.wounds, 0);

        let before_repeat = serde_json::to_string(&g).unwrap();
        let error = g.apply(0, heal).unwrap_err();
        assert!(error.contains("横置"));
        assert_eq!(serde_json::to_string(&g).unwrap(), before_repeat);
    }

    #[test]
    fn heal_entry_angru_and_witch_card_abilities() {
        let mut g = game("duel");
        let doctor = board(&mut g, "LC19", 0, 0);
        let target = board(&mut g, "JC059", 0, 0);
        g.board_mut(&target).unwrap().wounds = 1;
        resource(&mut g, 0, "JC125", 10);
        g.apply(
            0,
            Action {
                card_id: Some(doctor),
                target_id: Some(target.clone()),
                ..Action::new("activate")
            },
        )
        .unwrap();
        pass_stack(&mut g);
        assert_eq!(g.board(&target).unwrap().1.wounds, 0);
        g.board_mut(&target).unwrap().wounds = 1;
        let counselor = board(&mut g, "LC20", 0, 0);
        g.enter_triggers(0, "LC20", &counselor, false);
        g.drive().unwrap();
        select(&mut g, vec![target.clone()]);
        pass_stack(&mut g);
        assert_eq!(g.board(&target).unwrap().1.wounds, 0);
        let ang = board(&mut g, "LC23", 0, 0);
        g.enter_triggers(0, "LC23", &ang, true);
        g.drive().unwrap();
        select(&mut g, vec![target.clone()]);
        pass_stack(&mut g);
        assert!(g.board(&target).unwrap().1.exhausted && g.board(&ang).unwrap().1.exhausted);
        let witch = board(&mut g, "LC24", 0, 0);
        g.enter_triggers(0, "LC24", &witch, false);
        g.drive().unwrap();
        accept(&mut g);
        assert_eq!(g.pending.as_ref().unwrap().choice.amount, Some(2));
    }
    #[test]
    fn watchers_target_category_locks_and_shield_terminates() {
        let mut g = game("duel");
        let source = board(&mut g, "JC003", 0, 0);
        let target = board(&mut g, "JC125", 1, 1);
        resource(&mut g, 0, "JC125", 5);
        g.apply(
            0,
            Action {
                card_id: Some(source.clone()),
                target_id: Some(target.clone()),
                ..Action::new("activate")
            },
        )
        .unwrap();
        assert!(g.board(&source).unwrap().1.exhausted);
        g.board_mut(&target).unwrap().shield = 1;
        pass_stack(&mut g);
        assert!(!g.board(&target).unwrap().1.exhausted);
        assert_eq!(g.board(&target).unwrap().1.shield, 0);
        let keeper = board(&mut g, "JC002", 0, 1);
        g.enter_triggers(0, "JC002", &keeper, true);
        g.drive().unwrap();
        select(&mut g, vec![target.clone()]);
        let (_, c) = g.remove_board(&target).unwrap();
        let mut c = g.fresh(c);
        c.face_down = true;
        g.regions[1].cards.push(c);
        pass_stack(&mut g);
        assert!(!g.regions[1].cards.last().unwrap().exhausted);
    }
    #[test]
    fn mobility_adjacent_only_and_preserves_entity() {
        let mut g = game("teams");
        let id = board(&mut g, "JC014", 0, 2);
        board(&mut g, "JC059", 0, 2);
        board(&mut g, "JC059", 0, 3);
        g.board_mut(&id).unwrap().damage = 1;
        g.begin_window(Window::Mobility);
        let (r, c) = g.board(&id).unwrap();
        let source = g.source_snapshot(c, Some(r));
        g.emit_event(0, source, Event::ConfrontationStart);
        g.drive().unwrap();
        let p = g.pending.as_ref().unwrap();
        assert_eq!(
            p.choice
                .options
                .iter()
                .map(|x| x.id.as_str())
                .collect::<Vec<_>>(),
            vec!["region:1", "region:3"]
        );
        select(&mut g, vec!["region:3".into()]);
        pass_stack(&mut g);
        assert_eq!(g.board(&id).unwrap().0, 3);
        assert_eq!(g.board(&id).unwrap().1.damage, 1);
    }
    #[test]
    fn police_security_and_chase_apply_printed_rules() {
        let mut g = game("duel");
        let hidden = board(&mut g, "LC23", 1, 0);
        g.board_mut(&hidden).unwrap().face_down = true;
        let police = board(&mut g, "JC056", 0, 0);
        g.enter_triggers(0, "JC056", &police, false);
        g.drive().unwrap();
        accept(&mut g);
        assert!(g.board(&hidden).unwrap().1.exhausted);
        let security = board(&mut g, "JC059", 0, 0);
        assert_eq!(g.defense(g.board(&police).unwrap().1, 0), 2);
        assert_eq!(g.defense(g.board(&security).unwrap().1, 0), 2);
        let barrier = board(&mut g, "JZ08", 1, 0);
        let spell = hand(&mut g, "JC063", 0);
        resource(&mut g, 0, "JC056", 5);
        assert!(g
            .apply(
                0,
                Action {
                    card_id: Some(spell.clone()),
                    target_id: Some(barrier),
                    option: Some("hide".into()),
                    ..Action::new("play")
                }
            )
            .is_err());
        g.apply(
            0,
            Action {
                card_id: Some(spell),
                target_id: Some(police.clone()),
                option: Some("hide".into()),
                ..Action::new("play")
            },
        )
        .unwrap();
        pass_stack(&mut g);
        assert!(g.board(&police).is_none());
        assert!(g.regions[0]
            .cards
            .iter()
            .any(|c| c.definition == "JC056" && c.face_down));
    }

    #[test]
    fn all_transaction_effects_and_recursion_death_choice() {
        let mut g = game("duel");
        let c = g.make_card("LC21", 1);
        let grave = c.id.clone();
        g.players[1].graveyard.push(c);
        let funeral = hand(&mut g, "XQ49", 0);
        resource(&mut g, 0, "JC125", 5);
        let n = g.players[0].hand.len();
        g.apply(
            0,
            Action {
                card_id: Some(funeral),
                target_id: Some(grave.clone()),
                ..Action::new("play")
            },
        )
        .unwrap();
        pass_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), n);
        assert!(g.players[1].graveyard.iter().all(|c| c.id != grave));
        assert_eq!(g.players[1].deck.last().unwrap().definition, "LC21");
        let target = board(&mut g, "LC20", 1, 0);
        resource(&mut g, 0, "JC002", 3);
        let spell = hand(&mut g, "XQ03", 0);
        g.apply(
            0,
            Action {
                card_id: Some(spell),
                target_id: Some(target.clone()),
                ..Action::new("play")
            },
        )
        .unwrap();
        pass_stack(&mut g);
        assert!(g.board(&target).unwrap().1.exhausted);
        let spell = hand(&mut g, "JC118", 0);
        g.apply(
            0,
            Action {
                card_id: Some(spell),
                region: Some(0),
                ..Action::new("play")
            },
        )
        .unwrap();
        pass_stack(&mut g);
        assert!(g.regions[0].skip);
        let c = g.make_card("LC22", 0);
        let grave = c.id.clone();
        g.players[0].graveyard.push(c);
        resource(&mut g, 0, "JC086", 4);
        let necro = hand(&mut g, "JC092", 0);
        g.apply(
            0,
            Action {
                card_id: Some(necro),
                target_id: Some(grave),
                region: Some(2),
                ..Action::new("play")
            },
        )
        .unwrap();
        pass_stack(&mut g);
        assert!(g.regions[2]
            .cards
            .iter()
            .any(|c| c.definition == "LC22" && c.face_down));
        let corpse = g.make_card("LC20", 0);
        let corpse_id = corpse.id.clone();
        g.players[0].graveyard.push(corpse);
        let thief = board(&mut g, "JC086", 0, 2);
        g.enter_triggers(0, "JC086", &thief, false);
        g.drive().unwrap();
        select(&mut g, vec![corpse_id]);
        pass_stack(&mut g);
        assert_eq!(g.players[0].hand.last().unwrap().definition, "LC20");
        let slave = board(&mut g, "XQ12", 0, 2);
        g.damage(BTreeMap::from([(slave, 1)])).unwrap();
        g.drive().unwrap();
        select(&mut g, vec!["p1".into()]);
        pass_stack(&mut g);
        assert_eq!(g.pending.as_ref().unwrap().seat, 1);
        assert_eq!(g.pending.as_ref().unwrap().choice.kind, "discard");
    }

    #[test]
    fn normal_hide_resolution_projects_concealed_print_to_controller_instead_of_owner() {
        let mut g = game("duel");
        // Synthetic control transfer: no admitted card can naturally perform it.
        // The hide transition and subsequent projection use normal engine paths.
        let target = board(&mut g, "LC22", 1, 0);
        g.board_mut(&target).unwrap().controller = 0;
        let spell = hand(&mut g, "JC063", 0);
        resource(&mut g, 0, "JC056", 5);
        g.apply(
            0,
            Action {
                card_id: Some(spell),
                target_id: Some(target),
                option: Some("hide".into()),
                ..Action::new("play")
            },
        )
        .unwrap();
        pass_stack(&mut g);
        let controlled = g.regions[0]
            .cards
            .iter()
            .find(|c| c.definition == "LC22")
            .unwrap();
        assert_eq!((controlled.owner, controlled.controller), (1, 0));
        assert!(controlled.face_down);
        let mine = g.view(0);
        let former_owner = g.view(1);
        assert_eq!(
            mine.regions[0].characters[0].card_id.as_deref(),
            Some("LC22")
        );
        assert!(former_owner.regions[0].characters[0].card_id.is_none());
        assert!(former_owner.regions[0].characters[0].text.is_none());
        assert_eq!(former_owner.regions[0].characters[0].name, "暗藏者");
    }
    #[test]
    fn won_region_returns_cards_ordered_and_retreat_hand_no_death() {
        let mut g = game("duel");
        let a = board(&mut g, "LC21", 0, 0);
        let b = board(&mut g, "LC20", 0, 0);
        let retreat = board(&mut g, "JC016", 0, 0);
        let old_deck = g.players[0].deck.len();
        g.effect(Effect::Bottom { seat: 0, region: 0 }).unwrap();
        assert!(g.board(&retreat).is_none());
        assert_eq!(g.players[0].hand.last().unwrap().definition, "JC016");
        let p = g.pending.clone().unwrap();
        g.apply(
            0,
            Action {
                choice_id: Some(p.choice.id),
                bottom: Some(vec![b.clone(), a.clone()]),
                ..Action::new("choose")
            },
        )
        .unwrap();
        assert_eq!(g.players[0].deck.len(), old_deck + 2);
        assert_eq!(g.players[0].deck[old_deck].definition, "LC20");
        assert_eq!(g.players[0].deck[old_deck + 1].definition, "LC21");
        assert_ne!(g.players[0].deck[old_deck].id, b);
        assert!(g.players[0].graveyard.is_empty());
    }
    #[test]
    fn unique_mandatory_sacrifice_and_world_effects() {
        let mut g = game("duel");
        let first = board(&mut g, "LC23", 0, 0);
        let second = board(&mut g, "LC23", 0, 1);
        g.enter_triggers(0, "LC23", &second, false);
        g.drive().unwrap();
        assert_eq!(g.pending.as_ref().unwrap().choice.min, Some(1));
        select(&mut g, vec![first]);
        assert_eq!(g.players[0].graveyard.last().unwrap().definition, "LC23");
        region_effect_fixture(&mut g, "DQJC107");
        g.drive().unwrap();
        let selected = g.pending.as_ref().unwrap().choice.options[4].id.clone();
        let definition = g.players[0].deck[4].definition.clone();
        select(&mut g, vec![selected]);
        assert_eq!(g.players[0].deck[0].definition, definition);
        let other = g.pending.as_ref().unwrap().choice.options[0].id.clone();
        select(&mut g, vec![other]);
        region_effect_fixture(&mut g, "DQJC112");
        g.drive().unwrap();
        let h = g.players[0].hand.len();
        let pick = g.players[0].hand[0].id.clone();
        select(&mut g, vec![pick]);
        assert_eq!(g.players[0].hand.len(), h);
        select(&mut g, vec![]);
        region_effect_fixture(&mut g, "DQJC114");
        g.drive().unwrap();
        select(&mut g, vec!["discard".into()]);
        pass_stack(&mut g);
        assert_eq!(g.pending.as_ref().unwrap().choice.min, Some(2));
        let ids = g.pending.as_ref().unwrap().choice.options[..2]
            .iter()
            .map(|x| x.id.clone())
            .collect();
        select(&mut g, ids);
        let ids = g.pending.as_ref().unwrap().choice.options[..2]
            .iter()
            .map(|x| x.id.clone())
            .collect();
        select(&mut g, ids);
        let hidden = board(&mut g, "JC125", 1, 1);
        g.board_mut(&hidden).unwrap().face_down = true;
        let chara = board(&mut g, "JC125", 1, 1);
        region_effect_fixture(&mut g, "DQJC113");
        g.drive().unwrap();
        assert!(g.board(&chara).is_none());
        assert!(g.board(&hidden).is_some());
    }
    fn bot_choice(g: &Game) -> (usize, Action) {
        let p = g.pending.as_ref().unwrap();
        let mut a = Action {
            choice_id: Some(p.choice.id.clone()),
            ..Action::new("choose")
        };
        match p.resolution {
            ChoiceResolution::Forecast { .. }
            | ChoiceResolution::Frame {
                choice: FrameChoice::Forecast { .. },
                ..
            } => {
                a.top = Some(p.choice.options.iter().map(|o| o.id.clone()).collect());
                a.bottom = Some(vec![]);
            }
            ChoiceResolution::Bottom { .. } => {
                a.bottom = Some(p.choice.options.iter().map(|o| o.id.clone()).collect())
            }
            ChoiceResolution::Damage { region } => {
                let amount = p.choice.amount.unwrap();
                let guards = p
                    .choice
                    .options
                    .iter()
                    .filter(|o| {
                        g.board(&o.id)
                            .is_some_and(|(_, c)| c.definition == "LC21" || c.definition == "LC22")
                    })
                    .collect::<Vec<_>>();
                let mut allocations = BTreeMap::new();
                let mut remaining = amount;
                for guard in guards {
                    if remaining > 0 {
                        allocations.insert(guard.id.clone(), 1);
                        remaining -= 1;
                    }
                }
                if remaining > 0 {
                    let id = p.choice.options[0].id.clone();
                    *allocations.entry(id).or_default() += remaining;
                }
                a.allocations = Some(allocations);
                let _ = region;
            }
            _ => {
                a.selected = Some(
                    p.choice
                        .options
                        .iter()
                        .take(p.choice.min.unwrap_or(0))
                        .map(|o| o.id.clone())
                        .collect(),
                )
            }
        };
        (p.seat, a)
    }
    #[test]
    fn mobility_first_team_resolves_before_rear_team_and_responses_resolve_lifo() {
        let mut g = game("teams");
        let first = board(&mut g, "JC014", 0, 1);
        let rear = board(&mut g, "JC014", 2, 2);
        let exhausted = board(&mut g, "JC014", 1, 3);
        g.board_mut(&exhausted).unwrap().exhausted = true;
        g.begin_window(Window::Action(1));
        g.close_window().unwrap();
        g.drive().unwrap();
        assert_eq!(g.pending.as_ref().unwrap().seat, 0);
        select(&mut g, vec!["region:0".into()]);
        assert!(g.pending.is_none());
        assert_eq!(g.stack.len(), 1);
        assert_eq!(g.board(&rear).unwrap().0, 2);
        // Put an actual response above the already-declared move.
        resource(&mut g, 0, "JC002", 3);
        let spell = hand(&mut g, "XQ03", 0);
        g.apply(
            0,
            Action {
                card_id: Some(spell),
                target_id: Some(first.clone()),
                ..Action::new("play")
            },
        )
        .unwrap();
        for seat in [0, 1, 2, 3] {
            g.apply(seat, Action::new("pass")).unwrap();
        }
        assert_eq!(g.stack.len(), 1);
        assert!(g.board(&first).unwrap().1.exhausted);
        assert_eq!(g.board(&first).unwrap().0, 1);
        assert!(g.pending.is_none());
        pass_stack(&mut g);
        assert_eq!(g.board(&first).unwrap().0, 0);
        assert!(g.board(&first).unwrap().1.exhausted);
        assert_eq!(g.pending.as_ref().unwrap().seat, 2);
        select(&mut g, vec!["region:3".into()]);
        pass_stack(&mut g);
        assert_eq!(g.board(&rear).unwrap().0, 3);
        assert!(g.pending.is_none());
    }
    #[test]
    fn local_targets_revalidate_and_events_enter_graveyard_after_effect() {
        let mut g = game("duel");
        let doctor = board(&mut g, "LC19", 0, 0);
        let target = board(&mut g, "JC059", 0, 0);
        g.board_mut(&target).unwrap().wounds = 1;
        resource(&mut g, 0, "JC125", 5);
        g.apply(
            0,
            Action {
                card_id: Some(doctor),
                target_id: Some(target.clone()),
                ..Action::new("activate")
            },
        )
        .unwrap();
        move_fixture(&mut g, &target, 1);
        pass_stack(&mut g);
        assert_eq!(g.board(&target).unwrap().1.wounds, 1);
        let spell = hand(&mut g, "JC118", 0);
        g.apply(
            0,
            Action {
                card_id: Some(spell),
                region: Some(1),
                ..Action::new("play")
            },
        )
        .unwrap();
        let item = g.stack.pop().unwrap();
        g.resolve_stack(item).unwrap();
        assert!(!g.regions[1].skip);
        assert!(g.players[0]
            .graveyard
            .iter()
            .all(|c| c.definition != "JC118"));
        g.drive().unwrap();
        assert!(g.regions[1].skip);
        assert_eq!(g.players[0].graveyard.last().unwrap().definition, "JC118");
    }
    #[test]
    fn funeral_target_recovered_in_response_cancels_draw_without_refunding_cost() {
        let mut g = game("duel");
        // Initial layout fixture: a graveyard character and an already concealed thief.
        // All changes after this setup use the same Actions as the live service.
        let corpse = g.make_card("LC20", 1);
        let old_target = corpse.id.clone();
        g.players[1].graveyard.push(corpse);
        let thief = board(&mut g, "JC086", 1, 0);
        g.board_mut(&thief).unwrap().face_down = true;
        resource(&mut g, 0, "JC125", 2);
        resource(&mut g, 1, "JC086", 3);
        let funeral = hand(&mut g, "XQ49", 0);
        let actor_hand_before = g.players[0].hand.len();
        let target_deck_before = g.players[1].deck.len();
        let target_hand_before = g.players[1].hand.len();

        g.apply(
            0,
            Action {
                card_id: Some(funeral),
                target_id: Some(old_target.clone()),
                ..Action::new("play")
            },
        )
        .unwrap();
        assert_eq!(g.resources(0), 1);
        g.apply(0, Action::new("pass")).unwrap();
        g.apply(
            1,
            Action {
                card_id: Some(thief),
                ..Action::new("reveal")
            },
        )
        .unwrap();
        pass_stack(&mut g);
        assert_eq!(g.pending.as_ref().unwrap().seat, 1);
        assert_eq!(g.stack.len(), 1); // Funeral is still waiting under the entry trigger.
        select(&mut g, vec![old_target.clone()]);
        pass_stack(&mut g);

        assert!(g.stack.is_empty() && g.pending.is_none());
        assert!(g.players[1].graveyard.iter().all(|c| c.id != old_target));
        assert_eq!(g.players[1].hand.len(), target_hand_before + 1);
        let recovered = g.players[1].hand.last().unwrap();
        assert_eq!(recovered.definition, "LC20");
        assert_ne!(recovered.id, old_target);
        assert_eq!(g.players[1].deck.len(), target_deck_before); // No Funeral move to deck.
        assert_eq!(g.players[0].hand.len(), actor_hand_before - 1); // No attached draw.
        assert_eq!(g.resources(0), 1); // The accepted cost remains paid.
        assert_eq!(g.players[0].graveyard.last().unwrap().definition, "XQ49");
    }
    #[test]
    fn response_hide_and_reveal_reenters_with_new_identity_old_spell_cannot_hit() {
        let mut g = game("duel");
        // Initial layout fixture; the response, flip, departure and reentry use Actions.
        let old_target = board(&mut g, "LC20", 1, 0);
        resource(&mut g, 0, "JC002", 2);
        resource(&mut g, 1, "JC063", 4);
        let binding = hand(&mut g, "XQ03", 0);
        let chase = hand(&mut g, "JC063", 1);
        g.apply(
            0,
            Action {
                card_id: Some(binding),
                target_id: Some(old_target.clone()),
                ..Action::new("play")
            },
        )
        .unwrap();
        g.apply(0, Action::new("pass")).unwrap();
        g.apply(
            1,
            Action {
                card_id: Some(chase),
                target_id: Some(old_target.clone()),
                option: Some("hide".into()),
                ..Action::new("play")
            },
        )
        .unwrap();
        g.apply(1, Action::new("pass")).unwrap();
        g.apply(0, Action::new("pass")).unwrap(); // Only Chase resolves.
        assert_eq!(g.stack.len(), 1);
        assert!(g.board(&old_target).is_none());
        let hidden = g.regions[0].cards[0].id.clone();
        assert_ne!(hidden, old_target);
        assert!(g.board(&hidden).unwrap().1.face_down);

        g.apply(0, Action::new("pass")).unwrap();
        g.apply(
            1,
            Action {
                card_id: Some(hidden.clone()),
                ..Action::new("reveal")
            },
        )
        .unwrap();
        assert!(g.board(&hidden).is_none()); // The character has left the board for the stack.
        assert!(g.regions[0].cards.is_empty());
        assert_eq!(g.stack.len(), 2);
        g.apply(1, Action::new("pass")).unwrap();
        g.apply(0, Action::new("pass")).unwrap(); // Only Reveal resolves and declares its trigger.
        assert_eq!(g.stack.len(), 1);
        assert_eq!(g.pending.as_ref().unwrap().seat, 1);
        select(&mut g, vec![]); // Decline the optional LC20 entry trigger.
        let reentered = g.regions[0].cards[0].id.clone();
        assert_ne!(reentered, hidden);
        assert_ne!(reentered, old_target);
        assert!(!g.board(&reentered).unwrap().1.face_down);
        pass_stack(&mut g);

        assert_eq!(g.regions[0].cards.len(), 1);
        assert_eq!(g.regions[0].cards[0].id, reentered);
        assert!(!g.regions[0].cards[0].exhausted); // Old XQ03 does not target the new entity.
        assert_eq!(g.resources(0), 0);
        assert_eq!(g.resources(1), 0);
        assert_eq!(g.players[0].graveyard.last().unwrap().definition, "XQ03");
        assert_eq!(g.players[1].graveyard.last().unwrap().definition, "JC063");
    }
    #[test]
    fn murder_response_sacrifices_disciple_as_cost_and_independent_reduction_resolves() {
        let mut g = game("duel");
        // Initial layout fixture; every declaration, payment, response and resolution is an Action.
        let disciple = board(&mut g, "JC042", 0, 0);
        resource(&mut g, 0, "JC042", 2);
        resource(&mut g, 1, "JC091", 3);
        let murder = hand(&mut g, "JC091", 1);
        let future = hand(&mut g, "JC049", 0);
        g.apply(0, Action::new("pass")).unwrap();
        g.apply(
            1,
            Action {
                card_id: Some(murder),
                target_id: Some(disciple.clone()),
                ..Action::new("play")
            },
        )
        .unwrap();
        g.apply(1, Action::new("pass")).unwrap();
        g.apply(
            0,
            Action {
                card_id: Some(disciple.clone()),
                ..Action::new("activate")
            },
        )
        .unwrap();
        assert!(g.board(&disciple).is_none());
        assert_eq!(g.players[0].graveyard.len(), 1); // Sacrifice is a paid cost.
                                                     // Cost reduction resolves immediately; it never offers its own response window.
        assert_eq!(g.modifiers.len(), 1);
        assert_eq!(g.modifiers[0].actor, 0);
        assert_eq!(g.stack.len(), 1); // Only the original Murder remains on the stack.
        assert_eq!(g.stack[0].card.as_ref().unwrap().definition, "JC091");
        assert!(g.pending.is_none());
        let view = g.view(1);
        assert_eq!(view.stack[0].target_summaries[0].status, "missing");
        assert!(!view.stack[0].target_summaries[0].valid);
        let restored = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_string(&restored).unwrap(),
            serde_json::to_string(&g).unwrap()
        );
        g.apply(0, Action::new("pass")).unwrap();
        g.apply(1, Action::new("pass")).unwrap();
        assert!(g.stack.is_empty()); // One full pass round resolves the original Murder.
        assert_eq!(g.modifiers.len(), 1);
        assert_eq!(g.modifiers[0].actor, 0);
        assert_eq!(g.resources(1), 0); // Failed Murder does not refund its three assets.
        assert_eq!(g.players[1].graveyard.last().unwrap().definition, "JC091");
        assert_eq!(g.players[0].graveyard.len(), 1); // Original target was not destroyed again.
        let discounted = g
            .view(0)
            .hand
            .into_iter()
            .find(|c| c.instance_id == future)
            .unwrap();
        assert_eq!(discounted.effective_cost, Some(0));
        assert!(g
            .log
            .iter()
            .any(|entry| entry.text.contains("整个卡牌或能力效果取消")));
    }
    #[test]
    fn disciple_reduction_on_empty_stack_is_immediate_and_restores_without_repayment() {
        let mut g = game("duel");
        let disciple = board(&mut g, "JC042", 0, 0);
        let action = Action {
            card_id: Some(disciple.clone()),
            ..Action::new("activate")
        };
        assert!(g.stack.is_empty());
        let priority = g.priority_team;
        let window = g.window.clone();
        g.apply(0, action.clone()).unwrap();
        assert_eq!(g.modifiers.len(), 1);
        assert_eq!(g.modifiers[0].actor, 0);
        assert!(g.stack.is_empty());
        assert!(g.effects.is_empty());
        assert!(g.pending.is_none());
        assert_eq!(g.priority_team, priority);
        assert_eq!(g.window, window);
        assert!(g.passed.is_empty());
        assert!(g.board(&disciple).is_none());
        assert_eq!(g.players[0].graveyard.len(), 1);

        let serialized = serde_json::to_string(&g).unwrap();
        let mut restored = Game::from_persisted(&serialized).unwrap();
        assert!(restored.apply(0, action).is_err());
        assert_eq!(serde_json::to_string(&restored).unwrap(), serialized);
        // Same schema and pool do not make a previous engine state compatible.
        let mut previous: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        previous["versions"]["engine"] = "rust-v0.2.1".into();
        let error = Game::from_persisted(&serde_json::to_string(&previous).unwrap()).unwrap_err();
        assert!(error.contains(catalog::ENGINE_VERSION));
    }
    #[test]
    fn immediate_policy_keeps_cost_death_trigger_responsive_and_choice_resumable() {
        let mut g = game("duel");
        let target = board(&mut g, "LC21", 1, 0);
        let fee = board(&mut g, "XQ12", 1, 0);
        g.board_mut(&fee).unwrap().controller = 0;
        resource(&mut g, 0, "JC002", 2);
        let spell = hand(&mut g, "XQ03", 0);
        g.apply(
            0,
            Action {
                card_id: Some(spell),
                target_id: Some(target.clone()),
                ..Action::new("play")
            },
        )
        .unwrap();
        let lower_stack_id = g.stack[0].id.clone();
        // Explicit primitive contract fixture: XQ12 has its real Death trigger,
        // but this synthetic paid program is not an additional released card ability.
        // JC042 itself has no Death trigger in the supported pool.
        let (region, c) = g.board(&fee).unwrap();
        let source = g.source_snapshot(c, Some(region));
        let spec = rules::definition("JC042").abilities[0].clone();
        let paid = g
            .pay_ability_costs(0, &source, &spec, &Action::new("activate"))
            .unwrap();
        assert!(matches!(
            &paid[..],
            [PaidCost::Sacrificed {
                controller: 0,
                owner: 1,
                ..
            }]
        ));
        let frame = g.make_frame(0, source, &spec, vec![], paid, None);
        g.dispatch_frame(
            frame,
            "不可响应的原语测试".into(),
            None,
            spec.response_policy,
        );
        g.settle_deaths();
        g.drive().unwrap();
        assert_eq!(g.modifiers.len(), 1);
        assert_eq!(g.stack.len(), 1);
        assert_eq!(g.stack[0].id, lower_stack_id);
        assert!(g.board(&fee).is_none());
        assert_eq!(g.players[1].graveyard.len(), 1);
        let pending = g.pending.as_ref().unwrap();
        assert_eq!(pending.seat, 0); // Death belongs to the last controller, not the owner.
        let ChoiceResolution::Declare { declaration, .. } = &pending.resolution else {
            panic!("fee death must retain its independent declaration");
        };
        assert_eq!(declaration.ability.event, Some(Event::Death));
        assert_eq!(
            declaration.ability.response_policy,
            rules::ResponsePolicy::Respondable
        );
        let serialized = serde_json::to_string(&g).unwrap();
        let mut restored = Game::from_persisted(&serialized).unwrap();
        assert_eq!(serde_json::to_string(&restored).unwrap(), serialized);
        select(&mut restored, vec!["p1".into()]);
        assert_eq!(restored.stack.len(), 2); // Death gets its own normal response window.
        assert_eq!(restored.stack[0].id, lower_stack_id);
        assert_eq!(restored.players[1].graveyard.len(), 1);
        restored.apply(0, Action::new("pass")).unwrap();
        restored.apply(1, Action::new("pass")).unwrap();
        assert_eq!(restored.stack.len(), 1);
        assert_eq!(restored.stack[0].id, lower_stack_id);
        let pending = restored.pending.as_ref().unwrap();
        assert_eq!(pending.seat, 1);
        let ChoiceResolution::Frame { frame, .. } = &pending.resolution else {
            panic!("death discard must retain its resumable frame");
        };
        assert!(matches!(frame.guard, GuardState::Accepted));
        assert_eq!(frame.cursor, 1);
        let discarded = pending.choice.options[0].id.clone();
        let serialized = serde_json::to_string(&restored).unwrap();
        let mut resumed = Game::from_persisted(&serialized).unwrap();
        assert_eq!(serde_json::to_string(&resumed).unwrap(), serialized);
        select(&mut resumed, vec![discarded]);
        assert!(resumed.pending.is_none());
        assert_eq!(resumed.modifiers.len(), 1); // No repeated immediate effect or sacrifice.
        assert_eq!(resumed.players[1].graveyard.len(), 2);
        assert_eq!(resumed.stack.len(), 1);
        pass_stack(&mut resumed);
        assert!(resumed.board(&target).unwrap().1.exhausted);
        assert_eq!(resumed.players[1].graveyard.len(), 2);
    }
    #[test]
    fn bottom_to_hand_insufficient_cards_does_not_draw_or_eliminate_and_pays_controlled_cost() {
        for remaining in 0..=1 {
            let mut g = game("duel");
            // Initial fixture includes a character owned by the opponent but controlled by the caster.
            let sacrifice = board(&mut g, "LC21", 1, 0);
            g.board_mut(&sacrifice).unwrap().controller = 0;
            resource(&mut g, 0, "JC042", 2);
            let spell = hand(&mut g, "JC049", 0);
            g.players[0].deck.truncate(remaining);
            let hand_before = g.players[0].hand.len();
            let old_bottom = g.players[0].deck.last().map(|c| c.id.clone());
            let legal = g
                .legal_actions(0)
                .into_iter()
                .find(|a| {
                    a.action.card_id.as_deref() == Some(&spell)
                        && a.action.cost_selected.as_ref() == Some(&vec![sacrifice.clone()])
                })
                .unwrap();
            assert!(legal.label.contains("称职的保镖") && legal.label.contains(&sacrifice));
            g.apply(0, legal.action).unwrap();
            assert!(g.board(&sacrifice).is_none());
            assert_eq!(g.resources(0), 0);
            pass_stack(&mut g);
            assert!(!g.players[0].eliminated);
            assert_eq!(g.status, "playing");
            assert_eq!(g.players[0].hand.len(), hand_before - 1 + remaining);
            assert!(g.players[0].deck.is_empty());
            if let Some(old) = old_bottom {
                assert_ne!(g.players[0].hand.last().unwrap().id, old);
            }
            assert_eq!(g.players[1].graveyard.last().unwrap().definition, "LC21");
            assert_eq!(g.players[0].graveyard.last().unwrap().definition, "JC049");
        }
    }
    #[test]
    fn invalid_additional_sacrifice_rolls_back_assets_modifier_and_every_state_field() {
        let mut g = game("duel");
        let disciple = board(&mut g, "JC042", 0, 0);
        let enemy = board(&mut g, "LC21", 1, 0);
        resource(&mut g, 0, "JC042", 2);
        let spell = hand(&mut g, "JC049", 0);
        g.apply(
            0,
            Action {
                card_id: Some(disciple),
                ..Action::new("activate")
            },
        )
        .unwrap();
        pass_stack(&mut g);
        let before = serde_json::to_string(&g).unwrap();
        assert!(g
            .apply(
                0,
                Action {
                    card_id: Some(spell),
                    cost_selected: Some(vec![enemy]),
                    ..Action::new("play")
                }
            )
            .is_err());
        assert_eq!(serde_json::to_string(&g).unwrap(), before);
        assert_eq!(g.modifiers[0].uses, 1);
        assert_eq!(g.resources(0), 2);
    }
    #[test]
    fn reduction_only_face_up_play_matches_society_or_blood_loyalty_and_expires() {
        let mut g = game("duel");
        let disciple = board(&mut g, "JC042", 0, 0);
        let hidden = board(&mut g, "LC23", 0, 1);
        g.board_mut(&hidden).unwrap().face_down = true;
        let target = board(&mut g, "LC21", 1, 0);
        resource(&mut g, 0, "JC042", 4);
        resource(&mut g, 0, "JC002", 2);
        resource(&mut g, 0, "JC091", 2);
        let nonmatching = hand(&mut g, "XQ03", 0);
        let blood = hand(&mut g, "JZ54", 0);
        g.apply(
            0,
            Action {
                card_id: Some(disciple),
                ..Action::new("activate")
            },
        )
        .unwrap();
        pass_stack(&mut g);
        g.apply(
            0,
            Action {
                card_id: Some(nonmatching),
                target_id: Some(target),
                ..Action::new("play")
            },
        )
        .unwrap();
        pass_stack(&mut g);
        assert_eq!(g.modifiers[0].uses, 1);
        let resources = g.resources(0);
        g.apply(
            0,
            Action {
                card_id: Some(hidden),
                ..Action::new("reveal")
            },
        )
        .unwrap();
        assert_eq!(g.resources(0), resources - 3);
        pass_stack(&mut g);
        select(&mut g, vec![]);
        assert_eq!(g.modifiers[0].uses, 1);
        let resources = g.resources(0);
        g.apply(
            0,
            Action {
                card_id: Some(blood),
                target_id: Some("p0".into()),
                ..Action::new("play")
            },
        )
        .unwrap();
        assert_eq!(g.resources(0), resources);
        assert_eq!(g.modifiers[0].uses, 0);
        pass_stack(&mut g);
        let id = g.pending.as_ref().unwrap().choice.options[0].id.clone();
        select(&mut g, vec![id]);
        // A separate no-red-assets fixture verifies reduction never supplies loyalty.
        let mut no_loyalty = game("duel");
        let source = board(&mut no_loyalty, "JC042", 0, 0);
        let spell = hand(&mut no_loyalty, "JC049", 0);
        let fee = board(&mut no_loyalty, "LC21", 0, 0);
        no_loyalty
            .apply(
                0,
                Action {
                    card_id: Some(source),
                    ..Action::new("activate")
                },
            )
            .unwrap();
        pass_stack(&mut no_loyalty);
        let before = serde_json::to_string(&no_loyalty).unwrap();
        assert!(no_loyalty
            .apply(
                0,
                Action {
                    card_id: Some(spell),
                    cost_selected: Some(vec![fee]),
                    ..Action::new("play")
                }
            )
            .is_err());
        assert_eq!(serde_json::to_string(&no_loyalty).unwrap(), before);
        let turn = no_loyalty.turn;
        while no_loyalty.turn == turn && no_loyalty.status == "playing" {
            if no_loyalty.pending.is_some() {
                let (seat, a) = bot_choice(&no_loyalty);
                no_loyalty.apply(seat, a).unwrap();
            } else {
                let seat = no_loyalty
                    .living(no_loyalty.priority_team)
                    .into_iter()
                    .find(|s| !no_loyalty.passed.contains(s))
                    .unwrap();
                no_loyalty.apply(seat, Action::new("pass")).unwrap();
            }
        }
        assert!(no_loyalty.modifiers.is_empty());
    }
    #[test]
    fn effect_sacrifice_private_choice_restores_accepted_guard_and_current_control() {
        let mut g = game("duel");
        let sacrifice = board(&mut g, "LC20", 0, 0);
        g.board_mut(&sacrifice).unwrap().controller = 1;
        resource(&mut g, 0, "JC091", 2);
        let spell = hand(&mut g, "JZ54", 0);
        g.apply(
            0,
            Action {
                card_id: Some(spell),
                target_id: Some("p1".into()),
                ..Action::new("play")
            },
        )
        .unwrap();
        pass_stack(&mut g);
        let pending = g.pending.clone().unwrap();
        assert_eq!(pending.seat, 1);
        assert_eq!(pending.choice.min, Some(1));
        let ChoiceResolution::Frame { frame, .. } = &pending.resolution else {
            panic!("generic frame continuation required")
        };
        assert!(matches!(frame.guard, GuardState::Accepted));
        assert_eq!(frame.cursor, 1);
        assert_eq!(frame.already_paid.len(), 1);
        assert!(g.view(0).pending_choice.is_none());
        assert_eq!(
            g.view(0).stack.last().unwrap().target_summaries[0].status,
            "guardAccepted"
        );
        let mut restored = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
        let a = Action {
            choice_id: Some(pending.choice.id),
            selected: Some(vec![sacrifice.clone()]),
            ..Action::new("choose")
        };
        g.apply(1, a.clone()).unwrap();
        restored.apply(1, a).unwrap();
        assert_eq!(
            serde_json::to_string(&g).unwrap(),
            serde_json::to_string(&restored).unwrap()
        );
        assert!(g.board(&sacrifice).is_none());
        assert_eq!(g.resources(0), 0);
        assert_eq!(g.players[0].graveyard.len(), 2);
    }
    #[test]
    fn source_leaves_after_angru_trigger_target_exhaust_still_resolves() {
        let mut g = game("duel");
        let source = board(&mut g, "LC23", 0, 0);
        g.board_mut(&source).unwrap().face_down = true;
        let target = board(&mut g, "LC21", 1, 0);
        resource(&mut g, 0, "JC125", 3);
        resource(&mut g, 1, "JC091", 3);
        let murder = hand(&mut g, "JC091", 1);
        g.apply(
            0,
            Action {
                card_id: Some(source),
                ..Action::new("reveal")
            },
        )
        .unwrap();
        pass_stack(&mut g);
        let revealed = g.regions[0]
            .cards
            .iter()
            .find(|c| c.controller == 0)
            .unwrap()
            .id
            .clone();
        select(&mut g, vec![target.clone()]);
        g.apply(0, Action::new("pass")).unwrap();
        g.apply(
            1,
            Action {
                card_id: Some(murder),
                target_id: Some(revealed.clone()),
                ..Action::new("play")
            },
        )
        .unwrap();
        g.apply(1, Action::new("pass")).unwrap();
        g.apply(0, Action::new("pass")).unwrap();
        assert!(g.board(&revealed).is_none());
        assert!(!g.board(&target).unwrap().1.exhausted);
        pass_stack(&mut g);
        assert!(g.board(&target).unwrap().1.exhausted);
    }
    #[test]
    fn teams_both_teammates_can_respond_and_reduction_belongs_to_fixed_actor() {
        let mut g = game("teams");
        let first = board(&mut g, "JC042", 0, 0);
        let teammate = board(&mut g, "JC042", 1, 2);
        resource(&mut g, 2, "JC091", 3);
        let murder = hand(&mut g, "JC091", 2);
        for seat in [0, 1] {
            g.apply(seat, Action::new("pass")).unwrap();
        }
        g.apply(
            2,
            Action {
                card_id: Some(murder),
                target_id: Some(first.clone()),
                ..Action::new("play")
            },
        )
        .unwrap();
        for seat in [2, 3] {
            g.apply(seat, Action::new("pass")).unwrap();
        }
        g.apply(
            0,
            Action {
                card_id: Some(first),
                ..Action::new("activate")
            },
        )
        .unwrap();
        g.apply(0, Action::new("pass")).unwrap();
        assert!(g.passed.contains(&0));
        g.apply(
            1,
            Action {
                card_id: Some(teammate),
                ..Action::new("activate")
            },
        )
        .unwrap();
        assert!(g.passed.is_empty());
        assert_eq!(g.priority_team, 0);
        assert_eq!(g.stack.len(), 1); // Immediate reductions add no responsive objects.
        assert_eq!(g.modifiers.len(), 2);
        for seat in [0, 1, 2, 3] {
            g.apply(seat, Action::new("pass")).unwrap();
        }
        assert!(g.stack.is_empty());
        assert_eq!(
            g.modifiers.iter().map(|m| m.actor).collect::<Vec<_>>(),
            vec![0, 1]
        );
        assert_eq!(g.players[0].graveyard.len(), 1);
        assert_eq!(g.players[1].graveyard.len(), 1);
        assert_eq!(g.resources(2), 0);
    }
    #[test]
    fn sacrifice_defense_aura_immediately_kills_zero_defense_character_at_stable_point() {
        let mut g = game("duel");
        let aura = board(&mut g, "JC059", 1, 0);
        let patient = board(&mut g, "LC20", 1, 0);
        g.board_mut(&patient).unwrap().wounds = 1;
        assert_eq!(g.defense(g.board(&patient).unwrap().1, 0), 1);
        resource(&mut g, 0, "JC091", 2);
        let ritual = hand(&mut g, "JZ54", 0);
        g.apply(
            0,
            Action {
                card_id: Some(ritual),
                target_id: Some("p1".into()),
                ..Action::new("play")
            },
        )
        .unwrap();
        pass_stack(&mut g);
        select(&mut g, vec![aura]);
        assert!(g.board(&patient).is_none());
        assert_eq!(g.players[1].graveyard.len(), 2);
        assert_eq!(g.players[0].graveyard.last().unwrap().definition, "JZ54");
    }
    #[test]
    fn private_forecast_and_damage_choices_roundtrip_without_leaking() {
        let mut g = game("teams");
        g.effects.push_back(Effect::Forecast {
            seat: 1,
            amount: 3,
            draw: true,
        });
        g.drive().unwrap();
        let private = g.view(1).pending_choice.unwrap();
        for viewer in [0, 2, 3] {
            let v = g.view(viewer);
            assert!(v.pending_choice.is_none());
            assert_eq!(v.waiting_choice.unwrap().player_id, "p1");
            let public = serde_json::to_string(&g.view(viewer)).unwrap();
            for option in &private.options {
                assert!(!public.contains(&option.id));
            }
        }
        let mut restored: Game = serde_json::from_str(&serde_json::to_string(&g).unwrap()).unwrap();
        let (seat, action) = bot_choice(&g);
        g.apply(seat, action.clone()).unwrap();
        restored.apply(seat, action).unwrap();
        assert_eq!(
            serde_json::to_string(&g).unwrap(),
            serde_json::to_string(&restored).unwrap()
        );
        board(&mut g, "JC059", 2, 0);
        g.effects.push_back(Effect::Damage {
            seat: 0,
            region: 0,
            amount: 2,
        });
        g.drive().unwrap();
        let mut restored: Game = serde_json::from_str(&serde_json::to_string(&g).unwrap()).unwrap();
        let (seat, action) = bot_choice(&g);
        g.apply(seat, action.clone()).unwrap();
        restored.apply(seat, action).unwrap();
        assert_eq!(
            serde_json::to_string(&g).unwrap(),
            serde_json::to_string(&restored).unwrap()
        );
    }
    #[test]
    fn simultaneous_death_triggers_are_declared_first_team_then_rear_team() {
        let mut g = game("teams");
        g.first_team = 1;
        let rear = board(&mut g, "XQ12", 0, 0);
        let first = board(&mut g, "XQ12", 2, 4);
        g.damage(BTreeMap::from([(rear, 1), (first, 1)])).unwrap();
        g.drive().unwrap();
        assert_eq!(g.pending.as_ref().unwrap().seat, 2);
        select(&mut g, vec!["p0".into()]);
        assert_eq!(g.pending.as_ref().unwrap().seat, 0);
        select(&mut g, vec!["p2".into()]);
        assert_eq!(
            g.stack.iter().map(|s| s.controller).collect::<Vec<_>>(),
            vec![2, 0]
        );
    }
    #[test]
    fn each_start_phase_draws_exactly_one_per_living_player() {
        let mut g = game("teams");
        let counts = g
            .players
            .iter()
            .map(|p| (p.hand.len(), p.deck.len()))
            .collect::<Vec<_>>();
        g.begin_window(Window::Prepare);
        g.close_window().unwrap();
        g.drive().unwrap();
        assert_eq!(g.window, Some(Window::Draw));
        for (p, (hand, deck)) in g.players.iter().zip(counts) {
            assert_eq!(p.hand.len(), hand + 1);
            assert_eq!(p.deck.len(), deck - 1);
        }
        g.effects.push_back(Effect::Damage {
            seat: 0,
            region: 0,
            amount: 1,
        });
        board(&mut g, "LC20", 2, 0);
        g.drive().unwrap();
        assert_eq!(
            g.pending.as_ref().unwrap().choice.allow_decline,
            Some(false)
        );
    }
    #[test]
    fn team_elimination_removes_owned_zones_and_stack_but_preserves_score() {
        let mut g = game("teams");
        let score = g.make_card("DQJC107", 0);
        g.players[0].score_cards.push(score);
        board(&mut g, "LC23", 0, 0);
        resource(&mut g, 0, "JC125", 4);
        let card = hand(&mut g, "LC20", 0);
        g.apply(
            0,
            Action {
                card_id: Some(card),
                region: Some(0),
                ..Action::new("deploy")
            },
        )
        .unwrap();
        g.players[0].deck.clear();
        g.draw(0, 1).unwrap();
        assert!(g.players[0].eliminated);
        assert_eq!(g.status, "playing");
        assert_eq!(g.view(1).players[0].score, 4);
        assert!(g.stack.iter().all(|s| s.controller != 0));
        assert!(g
            .regions
            .iter()
            .all(|r| r.cards.iter().all(|c| c.owner != 0)));
        assert!(g.players[0].hand.is_empty() && g.players[0].assets.is_empty());
        g.apply(1, Action::new("pass")).unwrap();
        assert_eq!(g.priority_team, 1);
    }
    #[test]
    fn deterministic_complete_duel_and_team_games_and_restart() {
        for mode in ["duel", "teams"] {
            let mut g = game(mode);
            let initial = g.clone();
            let mut replay = initial.clone();
            let mut transcript = vec![];
            for move_no in 0..20000 {
                if g.status == "finished" {
                    break;
                }
                let (seat, action) = if g.pending.is_some() {
                    bot_choice(&g)
                } else {
                    let team = g.priority_team;
                    let seat = g
                        .living(team)
                        .into_iter()
                        .find(|s| !g.passed.contains(s))
                        .unwrap();
                    let actions = g.legal_actions(seat);
                    let action = if team == 0 {
                        actions
                            .iter()
                            .find(|a| {
                                a.action.kind == "asset"
                                    && a.action.card_id.as_ref().is_some_and(|id| {
                                        g.players[seat].hand.iter().any(|c| {
                                            c.id == *id && card(&c.definition).kind == "spell"
                                        })
                                    })
                            })
                            .or_else(|| actions.iter().find(|a| a.action.kind == "asset"))
                            .or_else(|| actions.iter().find(|a| a.action.kind == "deploy"))
                            .or_else(|| actions.iter().find(|a| a.action.kind == "conceal"))
                            .or_else(|| actions.iter().find(|a| a.action.kind == "pass"))
                    } else {
                        actions.iter().find(|a| a.action.kind == "pass")
                    }
                    .unwrap()
                    .action
                    .clone();
                    (seat, action)
                };
                g.apply(seat, action.clone())
                    .unwrap_or_else(|e| panic!("move {move_no} {mode} {action:?}: {e}"));
                transcript.push((seat, action));
            }
            assert_eq!(g.status, "finished", "mode {mode}, turn {}", g.turn);
            assert!(g.winner_team.is_some());
            for (seat, action) in transcript {
                replay.apply(seat, action).unwrap();
            }
            assert_eq!(
                serde_json::to_string(&g).unwrap(),
                serde_json::to_string(&replay).unwrap()
            );
            let old_random = g.random;
            g.apply(0, Action::new("restart")).unwrap();
            assert_ne!(g.random, old_random);
            assert_ne!(
                g.players[0]
                    .hand
                    .iter()
                    .map(|c| &c.definition)
                    .collect::<Vec<_>>(),
                initial.players[0]
                    .hand
                    .iter()
                    .map(|c| &c.definition)
                    .collect::<Vec<_>>()
            );
            replay.apply(0, Action::new("restart")).unwrap();
            assert_eq!(
                serde_json::to_string(&g).unwrap(),
                serde_json::to_string(&replay).unwrap()
            );
            assert_eq!(g.status, "playing");
            assert_eq!(g.turn, 1);
            assert_eq!(g.pending.as_ref().unwrap().choice.kind, "mulligan");
            assert!(g.players.iter().all(|p| p.score_cards.is_empty()));
        }
    }
}
