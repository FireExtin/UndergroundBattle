//! Deterministic, bounded rules interpreter. No clocks, I/O, or ambient randomness.
use crate::{
    catalog::{self, card},
    model::*,
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
        if mode != "duel" && mode != "teams" {
            return Err("模式必须为 duel 或 teams".into());
        }
        if catalog::deck(&deck_id).is_none() {
            return Err("未知牌组".into());
        }
        Ok(Self {
            room_id,
            invite_code,
            mode,
            version: 0,
            status: "lobby".into(),
            players: vec![Player::new(0, name, deck_id)],
            seed,
            random: seed.max(1),
            sequence: 0,
            first_team: 0,
            active_team: 0,
            priority_team: 0,
            turn: 0,
            winner_team: None,
            regions: vec![],
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
    fn id(&mut self) -> String {
        self.sequence += 1;
        format!("i{}", self.sequence)
    }
    fn random_below(&mut self, n: usize) -> usize {
        self.random ^= self.random << 13;
        self.random ^= self.random >> 7;
        self.random ^= self.random << 17;
        (self.random % n as u64) as usize
    }
    fn shuffle<T>(&mut self, cards: &mut [T]) {
        for i in (1..cards.len()).rev() {
            let j = self.random_below(i + 1);
            cards.swap(i, j)
        }
    }
    fn fresh(&mut self, mut c: Card) -> Card {
        c.id = self.id();
        c
    }
    pub fn make_card(&mut self, definition: &str, owner: usize) -> Card {
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
    fn note(&mut self, text: String) {
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
    fn can_standard(&self, seat: usize) -> bool {
        self.status == "playing"
            && self.pending.is_none()
            && self.stack.is_empty()
            && self.priority_team == self.team(seat)
            && self.window == Some(Window::Action(self.team(seat)))
            && !self.players[seat].eliminated
    }
    fn can_fast(&self, seat: usize) -> bool {
        self.status == "playing"
            && self.pending.is_none()
            && self.priority_team == self.team(seat)
            && !self.players[seat].eliminated
    }
    fn accessible(&self, seat: usize, region: usize) -> bool {
        region < self.regions.len()
            && (self.mode == "duel"
                || if seat % 2 == 0 {
                    region <= 2
                } else {
                    region >= 2
                })
    }
    fn resources(&self, seat: usize) -> u32 {
        self.players[seat]
            .assets
            .iter()
            .filter(|c| !c.exhausted)
            .count() as u32
    }
    fn loyalty(&self, seat: usize, definition: &str) -> bool {
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
    fn pay(&mut self, seat: usize, amount: u32) -> RuleResult<()> {
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
    fn board(&self, id: &str) -> Option<(usize, &Card)> {
        self.regions
            .iter()
            .enumerate()
            .find_map(|(i, r)| r.cards.iter().find(|c| c.id == id).map(|c| (i, c)))
    }
    fn board_mut(&mut self, id: &str) -> Option<&mut Card> {
        self.regions
            .iter_mut()
            .find_map(|r| r.cards.iter_mut().find(|c| c.id == id))
    }
    fn remove_board(&mut self, id: &str) -> Option<(usize, Card)> {
        for (i, r) in self.regions.iter_mut().enumerate() {
            if let Some(p) = r.cards.iter().position(|c| c.id == id) {
                return Some((i, r.cards.remove(p)));
            }
        }
        None
    }
    fn is_enemy(&self, seat: usize, c: &Card) -> bool {
        self.team(seat) != self.team(c.controller)
    }
    fn targetable(&self, seat: usize, c: &Card) -> bool {
        !self.is_enemy(seat, c)
            || c.face_down
            || !card(&c.definition).keywords.iter().any(|k| k == "屏障")
    }
    fn shield_stops(&mut self, actor: usize, target: &str) -> bool {
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
        if c.face_down {
            return Icons {
                influence: 1,
                ..Icons::default()
            };
        }
        let d = card(&c.definition);
        let mut result = d.permanent_icons;
        if self.team(c.controller) == self.first_team {
            result = result.add(d.temporary_icons);
        }
        if c.definition == "JC016"
            && !self.regions[region]
                .cards
                .iter()
                .any(|other| !other.face_down && self.is_enemy(c.controller, other))
        {
            result.influence += 1;
            if self.team(c.controller) == self.first_team {
                result.investigation += 1;
            }
        }
        result
    }
    pub fn defense(&self, c: &Card, region: usize) -> u32 {
        let bonus = self.regions[region]
            .cards
            .iter()
            .filter(|other| {
                !other.face_down
                    && other.definition == "JC059"
                    && other.controller == c.controller
                    && other.id != c.id
            })
            .count() as u32;
        (card(&c.definition).defense.unwrap_or(0) + bonus).saturating_sub(c.wounds)
    }
    fn reset_passes(&mut self) {
        self.passed.clear();
        self.team_passed = [false; 2];
    }
    fn priority_default(&self) -> usize {
        if let Some(Window::Action(team)) = self.window {
            team
        } else {
            self.first_team
        }
    }
    fn begin_window(&mut self, window: Window) {
        if let Window::Action(team) = window {
            self.active_team = team;
        }
        self.window = Some(window);
        self.priority_team = self.priority_default();
        self.reset_passes();
    }
    fn push_stack(
        &mut self,
        seat: usize,
        label: String,
        c: Option<Card>,
        region: Option<usize>,
        reveal: bool,
        effect: Option<Effect>,
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
            effect,
            target,
        });
        self.reset_passes();
    }
    pub fn apply(&mut self, seat: usize, action: Action) -> RuleResult<()> {
        if seat >= self.players.len() {
            return Err("无此座位".into());
        }
        // Work on a clone: every rejected command is a strict no-op, including PRNG/ID counters.
        let mut next = self.clone();
        next.version += 1;
        next.apply_inner(seat, action)?;
        next.drive()?;
        *self = next;
        Ok(())
    }
    fn apply_inner(&mut self, seat: usize, a: Action) -> RuleResult<()> {
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
                    let id = a.option.ok_or("请选择牌组")?;
                    if catalog::deck(&id).is_none() {
                        return Err("未知牌组".into());
                    }
                    self.players[seat].deck_id = id;
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
                let c = self.players[seat]
                    .hand
                    .iter()
                    .find(|c| c.id == id)
                    .ok_or("手牌引用已失效")?
                    .clone();
                let d = card(&c.definition);
                if d.kind != "character" {
                    return Err("只有角色可派遣".into());
                }
                if a.kind == "conceal" {
                    if self.players[seat].conceal_used || d.keywords.iter().any(|k| k == "公开") {
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
                    self.pay(seat, d.cost)?;
                    let c = self.remove_hand(seat, id)?;
                    let c = self.fresh(c);
                    self.note(format!("{} 打出 {}", self.players[seat].name, d.name));
                    self.push_stack(
                        seat,
                        format!("派遣 {}", d.name),
                        Some(c),
                        Some(r),
                        false,
                        None,
                        None,
                    );
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
                self.pay(seat, d.cost)?;
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
                    None,
                );
            }
            "play" => {
                let id = a.card_id.as_deref().ok_or("请选择事务")?;
                let c = self.players[seat]
                    .hand
                    .iter()
                    .find(|c| c.id == id)
                    .ok_or("手牌引用已失效")?
                    .clone();
                let d = card(&c.definition);
                if d.kind != "spell" {
                    return Err("该牌不是事务".into());
                }
                if c.definition == "JC092" && !standard {
                    return Err("死灵召唤为标准行动".into());
                }
                if c.definition == "JC118" && !matches!(self.window, Some(Window::Action(_))) {
                    return Err("停战协议只可在行动阶段".into());
                }
                if !self.loyalty(seat, &c.definition) {
                    return Err("忠诚不足".into());
                }
                let effect = self.event_effect(seat, &c.definition, &a)?;
                self.pay(seat, d.cost)?;
                let c = self.remove_hand(seat, id)?;
                let c = self.fresh(c);
                self.note(format!("{} 打出 {}", self.players[seat].name, d.name));
                self.push_stack(
                    seat,
                    d.name.clone(),
                    Some(c),
                    None,
                    false,
                    Some(effect),
                    a.target_id.clone(),
                );
            }
            "activate" => {
                let id = a.card_id.as_deref().ok_or("请选择能力来源")?;
                let (r, c) = self.board(id).ok_or("实体引用已失效")?;
                if c.controller != seat || c.face_down {
                    return Err("只能发动自己的正面角色能力".into());
                }
                let definition = c.definition.clone();
                let target = a.target_id.as_deref().ok_or("请选择目标")?;
                let effect = match definition.as_str() {
                    "LC19" => {
                        if !standard {
                            return Err("外科医生是标准行动".into());
                        }
                        self.validate_board_target(seat, target, false, Some(r))?;
                        Effect::Heal {
                            actor: seat,
                            target: target.into(),
                            region: Some(r),
                        }
                    }
                    "JC003" => {
                        if c.exhausted {
                            return Err("来源已横置".into());
                        }
                        let hidden = self.board(target).ok_or("目标实体已失效")?.1.face_down;
                        self.validate_board_target(seat, target, hidden, None)?;
                        Effect::Exhaust {
                            actor: seat,
                            target: target.into(),
                            hidden,
                            source: None,
                            region: None,
                        }
                    }
                    _ => return Err("该角色无可发动行动能力".into()),
                };
                self.pay(seat, 2)?;
                if definition == "JC003" {
                    self.board_mut(id).unwrap().exhausted = true;
                }
                self.note(format!(
                    "{} 发动 {}",
                    self.players[seat].name,
                    card(&definition).name
                ));
                self.push_stack(
                    seat,
                    format!("{} 的能力", card(&definition).name),
                    None,
                    None,
                    false,
                    Some(effect),
                    Some(target.into()),
                );
            }
            "privilege" => return self.privilege(seat),
            _ => return Err("当前操作不合法".into()),
        }
        Ok(())
    }
    fn remove_hand(&mut self, seat: usize, id: &str) -> RuleResult<Card> {
        let p = self.players[seat]
            .hand
            .iter()
            .position(|c| c.id == id)
            .ok_or("手牌实体已失效")?;
        Ok(self.players[seat].hand.remove(p))
    }
    fn validate_board_target(
        &self,
        actor: usize,
        id: &str,
        hidden: bool,
        region: Option<usize>,
    ) -> RuleResult<()> {
        let (r, c) = self.board(id).ok_or("目标实体已失效")?;
        if c.face_down != hidden
            || region.is_some_and(|wanted| r != wanted)
            || !self.targetable(actor, c)
        {
            return Err("目标类别、地区或屏障不合法".into());
        }
        Ok(())
    }
    fn event_effect(&self, seat: usize, definition: &str, a: &Action) -> RuleResult<Effect> {
        let target = a.target_id.as_deref().unwrap_or("");
        Ok(match definition {
            "XQ49" => {
                if !self
                    .players
                    .iter()
                    .any(|p| p.graveyard.iter().any(|c| c.id == target))
                {
                    return Err("葬礼需要墓地目标".into());
                }
                Effect::Funeral {
                    actor: seat,
                    target: target.into(),
                }
            }
            "JC118" => {
                let region = a.region.ok_or("请选择地区")?;
                if region >= self.regions.len() {
                    return Err("无此地区".into());
                }
                Effect::Ceasefire { region }
            }
            "XQ03" => {
                self.validate_board_target(seat, target, false, None)?;
                Effect::Exhaust {
                    actor: seat,
                    target: target.into(),
                    hidden: false,
                    source: None,
                    region: None,
                }
            }
            "JC063" => {
                let hide = match a.option.as_deref() {
                    Some("hide") => true,
                    Some("return") => false,
                    _ => return Err("请选择潜伏或回手".into()),
                };
                self.validate_board_target(seat, target, !hide, None)?;
                Effect::Chase {
                    actor: seat,
                    target: target.into(),
                    hide,
                }
            }
            "JC092" => {
                let region = a.region.ok_or("请选择地区")?;
                if region >= self.regions.len() {
                    return Err("无此地区".into());
                }
                if !self.players[seat]
                    .graveyard
                    .iter()
                    .any(|c| c.id == target && card(&c.definition).kind == "character")
                {
                    return Err("必须选择自己的墓地角色".into());
                }
                Effect::Summon {
                    seat,
                    target: target.into(),
                    region,
                }
            }
            _ => return Err("未实现事务，不开放".into()),
        })
    }
    fn start(&mut self) -> RuleResult<()> {
        self.status = "playing".into();
        self.regions.clear();
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
            self.players[seat].eliminated = false;
            let entries = catalog::deck(&self.players[seat].deck_id)
                .unwrap()
                .cards
                .clone();
            let mut pile = vec![];
            for entry in entries {
                for _ in 0..entry.count {
                    pile.push(self.make_card(&entry.card_id, seat));
                }
            }
            self.shuffle(&mut pile);
            self.players[seat].deck = pile;
        }
        let world_ids: Vec<_> = catalog::catalog()
            .cards
            .iter()
            .filter(|d| d.kind == "region")
            .map(|d| d.id.clone())
            .collect();
        let mut world = vec![];
        for definition in world_ids {
            for _ in 0..2 {
                world.push(self.make_card(&definition, 0));
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
            self.draw(seat, 6)?;
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
    fn draw(&mut self, seat: usize, count: usize) -> RuleResult<()> {
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
    fn eliminate(&mut self, seat: usize) {
        self.players[seat].eliminated = true;
        self.players[seat].hand.clear();
        self.players[seat].deck.clear();
        self.players[seat].assets.clear();
        self.players[seat].graveyard.clear();
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
            | Effect::Recover { seat: s, .. }
            | Effect::Summon { seat: s, .. }
            | Effect::Search { seat: s, .. }
            | Effect::Death { seat: s }
            | Effect::Trigger { seat: s, .. }
            | Effect::TargetTrigger { seat: s, .. }
            | Effect::Damage { seat: s, .. }
            | Effect::Unique { seat: s }
            | Effect::WorldMode { seat: s }
            | Effect::RegionTrigger { seat: s, .. } => *s != seat,
            Effect::Bury { card: c } => c.owner != seat,
            _ => true,
        });
        self.note(format!("{} 牌库耗尽，退出游戏", self.players[seat].name));
        let team = self.team(seat);
        if self.living(team).is_empty() {
            self.finish(1 - team);
        }
    }
    fn living(&self, team: usize) -> Vec<usize> {
        self.players
            .iter()
            .filter(|p| !p.eliminated && self.team(p.seat) == team)
            .map(|p| p.seat)
            .collect()
    }
    fn finish(&mut self, team: usize) {
        self.status = "finished".into();
        self.winner_team = Some(team);
        self.pending = None;
        self.effects.clear();
        self.stack.clear();
        self.note(format!("团队 {} 获胜", team + 1));
    }
    fn check_win(&mut self) {
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
    fn pass(&mut self, seat: usize) -> RuleResult<()> {
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
    fn resolve_stack(&mut self, mut item: StackItem) -> RuleResult<()> {
        self.note(format!("结算：{}", item.label));
        if let Some(c) = item.card.take() {
            if let Some(region) = item.deploy_region {
                let mut c = self.fresh(c);
                c.face_down = false;
                let id = c.id.clone();
                let definition = c.definition.clone();
                self.regions[region].cards.push(c);
                self.enter_triggers(item.controller, &definition, &id, item.reveal);
            } else {
                // Transactions enter the graveyard after executing their effect.
                self.effects.push_front(Effect::Bury { card: c });
            }
        }
        if let Some(effect) = item.effect {
            self.effects.push_front(effect);
        }
        Ok(())
    }
    fn enter_triggers(&mut self, seat: usize, definition: &str, id: &str, reveal: bool) {
        let label = format!("{}：进场触发", card(definition).name);
        match definition {
            "LC20" => self.effects.push_back(Effect::TargetTrigger {
                seat,
                label,
                source: id.into(),
                mode: "heal".into(),
            }),
            "LC23" if reveal => self.effects.push_back(Effect::TargetTrigger {
                seat,
                label: format!("{}：现身触发", card(definition).name),
                source: id.into(),
                mode: "angru".into(),
            }),
            "JC002" if reveal => self.effects.push_back(Effect::TargetTrigger {
                seat,
                label: format!("{}：现身触发", card(definition).name),
                source: id.into(),
                mode: "exhaust".into(),
            }),
            "LC24" => self.effects.push_back(Effect::Trigger {
                seat,
                label,
                effect: Box::new(Effect::Forecast {
                    seat,
                    amount: 2,
                    draw: true,
                }),
            }),
            "JC056" => {
                let region = self.board(id).unwrap().0;
                self.effects.push_back(Effect::Trigger {
                    seat,
                    label,
                    effect: Box::new(Effect::Police {
                        actor: seat,
                        region,
                    }),
                });
            }
            "JC086" => self.effects.push_back(Effect::TargetTrigger {
                seat,
                label,
                source: id.into(),
                mode: "recover".into(),
            }),
            _ => {}
        }
        if card(definition).unique {
            self.effects.push_front(Effect::Unique { seat });
        }
    }
    fn close_window(&mut self) -> RuleResult<()> {
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
                let mut sources: Vec<_> = self
                    .regions
                    .iter()
                    .flat_map(|r| r.cards.iter())
                    .filter(|c| !c.face_down && !c.exhausted && c.definition == "JC014")
                    .map(|c| (c.controller, c.id.clone()))
                    .collect();
                sources.sort_by_key(|(seat, _)| (self.team(*seat) != self.first_team, *seat));
                for (seat, source) in sources {
                    self.effects.push_back(Effect::TargetTrigger {
                        seat,
                        label: "公路骑士：机动".into(),
                        source,
                        mode: "mobility".into(),
                    });
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
                for owner in 0..self.players.len() {
                    self.effects.push_back(Effect::Bottom {
                        seat: owner,
                        region,
                    });
                }
                self.effects.push_back(Effect::Score { seat, region });
                self.begin_window(Window::After(region, 2));
            }
            Window::End => {
                self.effects.push_back(Effect::Cleanup);
                self.effects.push_back(Effect::NextTurn);
            }
        }
        Ok(())
    }
    fn contest_counts(&self, region: usize, contest: usize) -> [u32; 2] {
        let mut counts = [0; 2];
        for c in &self.regions[region].cards {
            counts[self.team(c.controller)] += self.icons(c, region).at(contest);
        }
        counts
    }
    fn contributors(&self, team: usize, region: usize, contest: usize) -> Vec<usize> {
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
    fn reward(
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
                        && c.definition == "JC016"
                })
                .count() as u32;
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
    fn privilege(&mut self, seat: usize) -> RuleResult<()> {
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
    fn choice(
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
    fn option(
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
    fn drive(&mut self) -> RuleResult<()> {
        let mut processed = 0;
        while self.pending.is_none() && self.status == "playing" {
            // Mobility has a specific first-team-before-rear-team order. Do not
            // declare the next move until the current responsive stack resolves.
            if !self.stack.is_empty()
                && matches!(self.effects.front(),Some(Effect::TargetTrigger{mode,..}) if mode=="mobility")
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
    fn effect(&mut self, e: Effect) -> RuleResult<()> {
        match e {
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
            Effect::Exhaust {
                actor,
                target,
                hidden,
                source,
                region,
            } => {
                if self
                    .validate_board_target(actor, &target, hidden, region)
                    .is_ok()
                    && !self.shield_stops(actor, &target)
                {
                    if let Some(source) = source {
                        if let Some(c) = self.board_mut(&source) {
                            c.exhausted = true;
                        }
                    }
                    if let Some(c) = self.board_mut(&target) {
                        c.exhausted = true;
                    }
                }
            }
            Effect::Heal {
                actor,
                target,
                region,
            } => {
                if self
                    .validate_board_target(actor, &target, false, region)
                    .is_ok()
                    && !self.shield_stops(actor, &target)
                {
                    self.board_mut(&target).unwrap().wounds = 0;
                }
            }
            Effect::Police { actor: _, region } => {
                for c in &mut self.regions[region].cards {
                    if c.face_down {
                        c.exhausted = true;
                    }
                }
            }
            Effect::Funeral { actor, target } => {
                for owner in 0..self.players.len() {
                    if let Some(i) = self.players[owner]
                        .graveyard
                        .iter()
                        .position(|c| c.id == target)
                    {
                        let c = self.players[owner].graveyard.remove(i);
                        let c = self.fresh(c);
                        self.players[owner].deck.push(c);
                        self.draw(actor, 1)?;
                        break;
                    }
                }
            }
            Effect::Ceasefire { region } => self.regions[region].skip = true,
            Effect::Chase {
                actor,
                target,
                hide,
            } => {
                if self
                    .validate_board_target(actor, &target, !hide, None)
                    .is_ok()
                    && !self.shield_stops(actor, &target)
                {
                    if let Some((region, mut c)) = self.remove_board(&target) {
                        c = self.fresh(c);
                        if hide {
                            c.face_down = true;
                            c.damage = 0;
                            c.wounds = 0;
                            c.shield = 0;
                            self.regions[region].cards.push(c);
                        } else {
                            c.face_down = false;
                            c.exhausted = false;
                            c.damage = 0;
                            c.wounds = 0;
                            c.shield = 0;
                            let owner = c.owner;
                            self.players[owner].hand.push(c);
                        }
                    }
                }
            }
            Effect::Recover { seat, target } => {
                if let Some(i) = self.players[seat]
                    .graveyard
                    .iter()
                    .position(|c| c.id == target && card(&c.definition).kind == "character")
                {
                    let c = self.players[seat].graveyard.remove(i);
                    let c = self.fresh(c);
                    self.players[seat].hand.push(c);
                }
            }
            Effect::Summon {
                seat,
                target,
                region,
            } => {
                if let Some(i) = self.players[seat]
                    .graveyard
                    .iter()
                    .position(|c| c.id == target && card(&c.definition).kind == "character")
                {
                    let mut c = self.players[seat].graveyard.remove(i);
                    c = self.fresh(c);
                    c.face_down = true;
                    c.exhausted = false;
                    c.damage = 0;
                    c.wounds = 0;
                    c.shield = 0;
                    self.regions[region].cards.push(c);
                }
            }
            Effect::Move { target, region } => {
                if let Some((_, c)) = self.remove_board(&target) {
                    let controller = c.controller;
                    let definition = c.definition.clone();
                    let id = c.id.clone();
                    self.regions[region].cards.push(c);
                    self.enter_triggers(controller, &definition, &id, false);
                }
            }
            Effect::ReturnHand { target } => {
                self.return_hand(&target);
            }
            Effect::Search { seat, kind } => {
                if self.players[seat].eliminated {
                    return Ok(());
                }
                let options: Vec<_> = self.players[seat]
                    .deck
                    .iter()
                    .filter(|c| kind == "any" || card(&c.definition).kind == kind)
                    .map(|c| self.option(c, seat, None, None))
                    .collect();
                if options.is_empty() {
                    self.shuffle_player(seat);
                    self.note(format!(
                        "{} 检索：没有符合条件的牌",
                        self.players[seat].name
                    ));
                } else {
                    self.choice(
                        seat,
                        "search",
                        if kind == "any" {
                            "从牌库选择一张牌，洗牌后置于牌库顶".into()
                        } else {
                            format!("检索 {}", kind)
                        },
                        options,
                        if kind == "any" { 1 } else { 0 },
                        1,
                        None,
                        ChoiceResolution::Search {
                            to_top: kind == "any",
                        },
                    );
                }
            }
            Effect::Death { seat } => {
                let options = self
                    .players
                    .iter()
                    .filter(|p| !p.eliminated)
                    .map(|p| ChoiceOption {
                        id: player_id(p.seat),
                        label: p.name.clone(),
                        card: None,
                    })
                    .collect();
                self.choice(
                    seat,
                    "target",
                    "暴躁血仆：死亡触发，目标玩家弃一张手牌".into(),
                    options,
                    0,
                    1,
                    None,
                    ChoiceResolution::Death,
                );
            }
            Effect::Trigger {
                seat,
                label,
                effect,
            } => {
                if !self.players[seat].eliminated {
                    self.choice(
                        seat,
                        "trigger",
                        label.clone(),
                        vec![ChoiceOption {
                            id: "accept".into(),
                            label: "发动触发能力".into(),
                            card: None,
                        }],
                        0,
                        1,
                        None,
                        ChoiceResolution::Trigger {
                            effect: *effect,
                            label,
                        },
                    );
                }
            }
            Effect::TargetTrigger {
                seat,
                label,
                source,
                mode,
            } => {
                if self.players[seat].eliminated {
                    return Ok(());
                }
                let options: Vec<_> = if mode == "recover" {
                    self.players[seat]
                        .graveyard
                        .iter()
                        .filter(|c| card(&c.definition).kind == "character")
                        .map(|c| self.option(c, seat, None, None))
                        .collect()
                } else if mode == "mobility" {
                    if let Some((region, character)) = self
                        .board(&source)
                        .filter(|(_, c)| !c.face_down && !c.exhausted)
                    {
                        let _ = character;
                        (0..self.regions.len())
                            .filter(|r| {
                                *r != region && (self.mode == "duel" || r.abs_diff(region) == 1)
                            })
                            .map(|r| ChoiceOption {
                                id: format!("region:{r}"),
                                label: format!(
                                    "移动至 {}",
                                    card(&self.regions[r].card.definition).name
                                ),
                                card: None,
                            })
                            .collect()
                    } else {
                        vec![]
                    }
                } else if let Some((region, _)) = self.board(&source) {
                    self.regions[region]
                        .cards
                        .iter()
                        .filter(|c| !c.face_down && self.targetable(seat, c))
                        .map(|c| self.option(c, seat, Some(region), None))
                        .collect()
                } else {
                    vec![]
                };
                if !options.is_empty() {
                    self.choice(
                        seat,
                        "target",
                        label.clone(),
                        options,
                        0,
                        1,
                        None,
                        if mode == "mobility" {
                            ChoiceResolution::Mobility { source }
                        } else {
                            ChoiceResolution::TargetTrigger {
                                source,
                                mode,
                                label,
                            }
                        },
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
            Effect::Bottom { seat, region } => {
                let retreat: Vec<_> = self.regions[region]
                    .cards
                    .iter()
                    .filter(|c| c.owner == seat && !c.face_down && c.definition == "JC016")
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
                    if definition == "DQJC114" {
                        self.effects.push_front(Effect::WorldMode { seat });
                    } else {
                        self.effects.push_front(Effect::Trigger {
                            seat,
                            label: format!("{}：赢取触发", card(&definition).name),
                            effect: Box::new(Effect::RegionTrigger { seat, definition }),
                        });
                    }
                }
            }
            Effect::RegionTrigger { seat, definition } => match definition.as_str() {
                "DQJC107" => {
                    for s in 0..self.players.len() {
                        self.effects.push_back(Effect::Search {
                            seat: s,
                            kind: "any".into(),
                        });
                    }
                }
                "DQJC112" => {
                    for s in 0..self.players.len() {
                        self.effects.push_back(Effect::Discard {
                            seat: s,
                            amount: self.players[s].hand.len(),
                            redraw: true,
                            optional: true,
                        });
                    }
                }
                "DQJC113" => self.effects.push_back(Effect::GlobalDamage),
                "DQJC114" => self.effects.push_back(Effect::WorldMode { seat }),
                "DQJC116" => {
                    for s in 0..self.players.len() {
                        self.effects.push_back(Effect::Search {
                            seat: s,
                            kind: "attachment".into(),
                        });
                    }
                }
                _ => return Err("地区效果未实现".into()),
            },
            Effect::GlobalDamage => {
                let targets = self
                    .regions
                    .iter()
                    .flat_map(|r| r.cards.iter())
                    .filter(|c| !c.face_down)
                    .map(|c| (c.id.clone(), 1))
                    .collect();
                self.damage(targets)?;
            }
            Effect::WorldMode { seat } => {
                self.choice(
                    seat,
                    "trigger",
                    "上海：选择每位玩家抓2张或弃2张".into(),
                    vec![
                        ChoiceOption {
                            id: "draw".into(),
                            label: "所有玩家抓两张牌".into(),
                            card: None,
                        },
                        ChoiceOption {
                            id: "discard".into(),
                            label: "所有玩家各自选择弃两张牌".into(),
                            card: None,
                        },
                    ],
                    0,
                    1,
                    None,
                    ChoiceResolution::WorldMode,
                );
            }
            Effect::Unique { seat } => {
                let options = self
                    .regions
                    .iter()
                    .enumerate()
                    .flat_map(|(r, region)| {
                        region
                            .cards
                            .iter()
                            .filter(move |c| {
                                c.controller == seat && !c.face_down && c.definition == "LC23"
                            })
                            .map(move |c| (r, c))
                    })
                    .map(|(r, c)| self.option(c, seat, Some(r), None))
                    .collect::<Vec<_>>();
                if options.len() > 1 {
                    self.choice(
                        seat,
                        "target",
                        "独有：选择一张安格鲁牺牲".into(),
                        options,
                        1,
                        1,
                        None,
                        ChoiceResolution::Unique,
                    );
                }
            }
            Effect::Cleanup => {
                for region in &mut self.regions {
                    region.skip = false;
                    for c in &mut region.cards {
                        c.damage = 0;
                    }
                }
                let mut discards = vec![];
                for seat in 0..self.players.len() {
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
            Effect::Bury { card: c } => {
                if !self.players[c.owner].eliminated {
                    let owner = c.owner;
                    let c = self.fresh(c);
                    self.players[owner].graveyard.push(c);
                }
            }
            Effect::NextTurn => {
                self.turn += 1;
                self.first_team = 1 - self.first_team;
                self.privilege_used = false;
                for p in &mut self.players {
                    p.asset_used = false;
                    p.conceal_used = false;
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
            Effect::WorldAll { draw } => {
                for seat in 0..self.players.len() {
                    self.effects.push_back(if draw {
                        Effect::Draw { seat, count: 2 }
                    } else {
                        Effect::Discard {
                            seat,
                            amount: 2,
                            redraw: false,
                            optional: false,
                        }
                    });
                }
            }
            Effect::Swap { .. } => return Err("不可达未开放效果".into()),
        }
        Ok(())
    }
    fn shuffle_player(&mut self, seat: usize) {
        let mut deck = std::mem::take(&mut self.players[seat].deck);
        self.shuffle(&mut deck);
        self.players[seat].deck = deck;
    }
    fn return_hand(&mut self, target: &str) {
        if let Some((_, mut c)) = self.remove_board(target) {
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
    fn to_bottom(&mut self, target: &str) {
        if let Some((_, mut c)) = self.remove_board(target) {
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
    fn kill(&mut self, target: &str) {
        if let Some((_, mut c)) = self.remove_board(target) {
            let controller = c.controller;
            let death = !c.face_down && c.definition == "XQ12";
            c = self.fresh(c);
            c.face_down = false;
            c.exhausted = false;
            c.damage = 0;
            c.wounds = 0;
            c.shield = 0;
            let owner = c.owner;
            self.players[owner].graveyard.push(c);
            if death {
                self.effects.push_back(Effect::Death { seat: controller });
            }
        }
    }
    fn damage(&mut self, allocations: BTreeMap<String, u32>) -> RuleResult<()> {
        let previous_effects = self.effects.len();
        for (target, amount) in allocations {
            if let Some(c) = self.board_mut(&target) {
                if !c.face_down {
                    c.damage += amount;
                }
            }
        }
        // Loss of a defense aura is checked again after the simultaneous lethal set.
        loop {
            let mut dead = vec![];
            for (r, region) in self.regions.iter().enumerate() {
                for c in &region.cards {
                    if !c.face_down && c.damage >= self.defense(c, r) {
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
            Effect::Death { seat } => (self.team(*seat) != self.first_team, *seat),
            _ => (true, usize::MAX),
        });
        self.effects.extend(simultaneous);
        Ok(())
    }
    fn choose(&mut self, seat: usize, a: Action) -> RuleResult<()> {
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
        ) && (selected.len() < p.choice.min.unwrap_or(0)
            || selected.len() > p.choice.max.unwrap_or(usize::MAX))
        {
            return Err("选择数量不符".into());
        }
        self.pending = None;
        match p.resolution {
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
            ChoiceResolution::Trigger { effect, label } => {
                if !selected.is_empty() {
                    self.push_stack(seat, label, None, None, false, Some(effect), None);
                }
            }
            ChoiceResolution::TargetTrigger {
                source,
                mode,
                label,
            } => {
                if let Some(target) = selected.first() {
                    let region = self.board(&source).map(|(r, _)| r);
                    let effect = match mode.as_str() {
                        "heal" => Effect::Heal {
                            actor: seat,
                            target: target.clone(),
                            region,
                        },
                        "recover" => Effect::Recover {
                            seat,
                            target: target.clone(),
                        },
                        "exhaust" | "angru" => Effect::Exhaust {
                            actor: seat,
                            target: target.clone(),
                            hidden: false,
                            source: if mode == "angru" { Some(source) } else { None },
                            region,
                        },
                        _ => return Err("未知目标触发".into()),
                    };
                    self.push_stack(
                        seat,
                        label,
                        None,
                        None,
                        false,
                        Some(effect),
                        Some(target.clone()),
                    );
                }
            }
            ChoiceResolution::Mobility { source } => {
                if let Some(target) = selected.first() {
                    let region = target
                        .strip_prefix("region:")
                        .ok_or("地区选项无效")?
                        .parse()
                        .map_err(|_| "地区无效")?;
                    self.push_stack(
                        seat,
                        "机动移动".into(),
                        None,
                        None,
                        false,
                        Some(Effect::Move {
                            target: source,
                            region,
                        }),
                        None,
                    );
                }
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
                            && (c.definition == "LC21" || c.definition == "LC22")
                    })
                    .collect::<Vec<_>>();
                let total = p.choice.amount.unwrap_or(0);
                let guard_assigned = guards
                    .iter()
                    .map(|c| allocations.get(&c.id).copied().unwrap_or(0).min(1))
                    .sum::<u32>();
                if guard_assigned < total.min(guards.len() as u32) {
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
                for id in ordered {
                    if self.board(&id).is_some_and(|(r, _)| r != region) {
                        return Err("地区引用失效".into());
                    }
                    self.to_bottom(&id);
                }
            }
            ChoiceResolution::Search { to_top } => {
                let chosen = selected
                    .first()
                    .and_then(|id| self.players[seat].deck.iter().position(|c| c.id == *id))
                    .map(|i| self.players[seat].deck.remove(i));
                self.shuffle_player(seat);
                if let Some(c) = chosen {
                    let c = self.fresh(c);
                    if to_top {
                        self.players[seat].deck.insert(0, c);
                        self.note(format!("{} 完成检索并置于牌库顶", self.players[seat].name));
                    } else {
                        let name = card(&c.definition).name.clone();
                        self.players[seat].hand.push(c);
                        self.note(format!("{} 展示检索的 {}", self.players[seat].name, name));
                    }
                }
            }
            ChoiceResolution::ReturnHand => {
                self.return_hand(&selected[0]);
            }
            ChoiceResolution::Death => {
                if let Some(target) = selected.first() {
                    let recipient = target
                        .strip_prefix('p')
                        .unwrap()
                        .parse::<usize>()
                        .map_err(|_| "玩家无效")?;
                    self.push_stack(
                        seat,
                        "暴躁血仆：死亡触发".into(),
                        None,
                        None,
                        false,
                        Some(Effect::Discard {
                            seat: recipient,
                            amount: 1,
                            redraw: false,
                            optional: false,
                        }),
                        Some(target.clone()),
                    );
                }
            }
            ChoiceResolution::Unique => {
                self.kill(&selected[0]);
            }
            ChoiceResolution::WorldMode => {
                if let Some(mode) = selected.first() {
                    self.push_stack(
                        seat,
                        format!(
                            "上海：{}",
                            if mode == "draw" {
                                "所有玩家抓两张牌"
                            } else {
                                "所有玩家各弃两张牌"
                            }
                        ),
                        None,
                        None,
                        false,
                        Some(Effect::WorldAll {
                            draw: mode == "draw",
                        }),
                        None,
                    );
                }
            }
            ChoiceResolution::Swap => return Err("未开放的选择".into()),
        }
        Ok(())
    }
    fn card_view(
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
            cost: if hidden || asset { None } else { Some(d.cost) },
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
                .map(|s| StackView {
                    id: s.id.clone(),
                    label: s.label.clone(),
                    controller: player_id(s.controller),
                    card_id: s.card.as_ref().map(|c| c.definition.clone()),
                    target_id: s.target.clone(),
                })
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
                } else if d.kind == "spell" {
                    match c.definition.as_str() {
                        "JC118" => {
                            for r in 0..self.regions.len() {
                                candidates.push((
                                    Action {
                                        card_id: Some(c.id.clone()),
                                        region: Some(r),
                                        ..Action::new("play")
                                    },
                                    format!(
                                        "停战协议 → {}",
                                        card(&self.regions[r].card.definition).name
                                    ),
                                ));
                            }
                        }
                        "XQ49" => {
                            for t in self.players.iter().flat_map(|p| p.graveyard.iter()) {
                                candidates.push((
                                    Action {
                                        card_id: Some(c.id.clone()),
                                        target_id: Some(t.id.clone()),
                                        ..Action::new("play")
                                    },
                                    format!("葬礼 → {}", card(&t.definition).name),
                                ));
                            }
                        }
                        "JC092" => {
                            for t in &self.players[seat].graveyard {
                                for r in 0..self.regions.len() {
                                    candidates.push((
                                        Action {
                                            card_id: Some(c.id.clone()),
                                            target_id: Some(t.id.clone()),
                                            region: Some(r),
                                            ..Action::new("play")
                                        },
                                        format!(
                                            "死灵召唤 {} → {}",
                                            card(&t.definition).name,
                                            card(&self.regions[r].card.definition).name
                                        ),
                                    ));
                                }
                            }
                        }
                        "XQ03" | "JC063" => {
                            for (r, t) in &board {
                                let name = self.card_view(t, seat, Some(*r), None).name;
                                let options = if c.definition == "JC063" {
                                    vec![Some("hide".to_string()), Some("return".to_string())]
                                } else {
                                    vec![None]
                                };
                                for option in options {
                                    candidates.push((
                                        Action {
                                            card_id: Some(c.id.clone()),
                                            target_id: Some(t.id.clone()),
                                            option: option.clone(),
                                            ..Action::new("play")
                                        },
                                        format!(
                                            "{} {} → {}",
                                            d.name,
                                            match option.as_deref() {
                                                Some("hide") => "潜伏",
                                                Some("return") => "回手",
                                                _ => "",
                                            },
                                            name
                                        ),
                                    ));
                                }
                            }
                        }
                        _ => {}
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
                } else if c.definition == "LC19" || c.definition == "JC003" {
                    for (tr, t) in &board {
                        let target = self.card_view(t, seat, Some(*tr), None);
                        candidates.push((
                            Action {
                                card_id: Some(c.id.clone()),
                                target_id: Some(t.id.clone()),
                                ..Action::new("activate")
                            },
                            format!(
                                "{} 能力 → {}（地区{}）",
                                card(&c.definition).name,
                                target.name,
                                r + 1
                            ),
                        ));
                    }
                }
            }
        }
        candidates
            .into_iter()
            .filter_map(|(action, label)| {
                let mut trial = self.clone();
                if trial.apply_inner(seat, action.clone()).is_ok() {
                    let id = format!(
                        "{}:{}:{}:{}:{}",
                        action.kind,
                        action.card_id.as_deref().unwrap_or(""),
                        action.target_id.as_deref().unwrap_or(""),
                        action.region.map(|r| r.to_string()).unwrap_or_default(),
                        action.option.as_deref().unwrap_or("")
                    );
                    Some(LegalAction {
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
        assert_eq!(c.cards.len(), 26);
        assert_eq!(
            c.cards
                .iter()
                .filter(|d| d.kind == "character" || d.kind == "spell")
                .count(),
            21
        );
        for deck in &c.decks {
            assert_eq!(deck.card_count, 50);
            assert_eq!(deck.cards.iter().map(|x| x.count).sum::<usize>(), 50);
            for e in &deck.cards {
                assert!(e.card_id == "JC125" || e.count <= 3);
                assert_ne!(card(&e.card_id).kind, "region");
            }
        }
        assert_eq!(card("LC23").unique, true);
        assert!(card("JC125").text.contains("数量没有限制"));
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
        g.board_mut(&id).unwrap().damage = 1;
        g.begin_window(Window::Mobility);
        g.effects.push_back(Effect::TargetTrigger {
            seat: 0,
            label: "机动".into(),
            source: id.clone(),
            mode: "mobility".into(),
        });
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
        g.effect(Effect::RegionTrigger {
            seat: 0,
            definition: "DQJC107".into(),
        })
        .unwrap();
        g.drive().unwrap();
        let selected = g.pending.as_ref().unwrap().choice.options[4].id.clone();
        let definition = g.players[0].deck[4].definition.clone();
        select(&mut g, vec![selected]);
        assert_eq!(g.players[0].deck[0].definition, definition);
        let other = g.pending.as_ref().unwrap().choice.options[0].id.clone();
        select(&mut g, vec![other]);
        g.effect(Effect::RegionTrigger {
            seat: 0,
            definition: "DQJC112".into(),
        })
        .unwrap();
        g.drive().unwrap();
        let h = g.players[0].hand.len();
        let pick = g.players[0].hand[0].id.clone();
        select(&mut g, vec![pick]);
        assert_eq!(g.players[0].hand.len(), h);
        select(&mut g, vec![]);
        g.effect(Effect::RegionTrigger {
            seat: 0,
            definition: "DQJC116".into(),
        })
        .unwrap();
        let rng = g.random;
        g.drive().unwrap();
        assert!(g.pending.is_none());
        assert_ne!(g.random, rng);
        g.effect(Effect::RegionTrigger {
            seat: 0,
            definition: "DQJC114".into(),
        })
        .unwrap();
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
        g.effect(Effect::RegionTrigger {
            seat: 0,
            definition: "DQJC113".into(),
        })
        .unwrap();
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
            ChoiceResolution::Forecast { .. } => {
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
        g.effect(Effect::Move {
            target: target.clone(),
            region: 1,
        })
        .unwrap();
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
