//! Printed hegemony p17: wound replaces allocated combat damage with equal
//! wound markers; kill alone adds damage. The allocating player must have
//! participated with a wound character. X is not an extra amount or multiplier.
use crate::{engine::RuleResult, model::*, rules::Event};
use std::collections::BTreeMap;

impl Game {
    /// The existing reward policy: actual contributors in seat order. Influence
    /// alone falls back to the living team when no character contributed.
    pub(crate) fn reward_recipient_seats(&self, team: usize, region: usize, contest: usize) -> RuleResult<Vec<usize>> {
        if team > 1 || contest > 2 || !self.region_live(region) { return Err("奖励地区或对抗类别无效".into()); }
        let mut seats = self.contributors(team, region, contest);
        if contest == 2 && seats.is_empty() { seats = self.living(team); }
        Ok(seats)
    }
    /// Do not trust persisted options or min/max when consuming a recipient.
    /// Run before taking pending, so invalid input leaves even direct choose unchanged.
    pub(crate) fn selected_reward_recipient(&self, chooser: usize, region: usize, contest: usize, selected: &[String]) -> RuleResult<usize> {
        let [id] = selected else { return Err("必须恰好选择一位实际参战的奖励领取者".into()); };
        if chooser >= self.players.len() { return Err("奖励选择者无效".into()); }
        let recipient = id.strip_prefix('p').and_then(|s| s.parse::<usize>().ok()).ok_or("奖励领取者无效")?;
        let seats = self.reward_recipient_seats(self.team(chooser), region, contest)?;
        if seats.first() != Some(&chooser) || !seats.contains(&recipient) || *id != player_id(recipient) {
            return Err("奖励只可交给本次实际参战的获胜方玩家".into());
        }
        Ok(recipient)
    }
    pub(crate) fn combat_recipient_seats(&self, team: usize, region: usize, amount: u32, sources: &[String]) -> RuleResult<Vec<usize>> {
        self.validate_combat_wound_reward(region, team, amount, sources, None)?;
        let seats = self.reward_recipient_seats(team, region, 1)?;
        let counts = self.contest_counts(region, 1);
        if self.window != Some(Window::After(region, 1)) || counts[team] <= counts[1 - team] || seats.len() < 2 {
            return Err("战斗奖励领取续体须保留获胜方的实际参战玩家".into());
        }
        Ok(seats)
    }
    /// Frozen instance IDs travel through recipient selection and allocation.
    /// Neither pause allows game actions, so restores can verify the exact live
    /// participants without rebinding a replacement of the same printed card.
    pub(crate) fn combat_wound_sources(&self, region: usize, team: usize, seat: Option<usize>) -> Vec<String> {
        if !self.region_live(region) { return vec![]; }
        self.region_characters(region).filter(|c| c.controller < self.players.len()
            && self.team(c.controller) == team && seat.is_none_or(|s| c.controller == s)
            && !c.face_down && !c.exhausted && self.icons(c, region).combat > 0
            && self.effective_wound(c) > 0).map(|c| c.id.clone()).collect()
    }
    fn validate_combat_wound_reward(&self, region: usize, team: usize, amount: u32, sources: &[String], seat: Option<usize>) -> RuleResult<()> {
        if sources != self.combat_wound_sources(region, team, seat) {
            return Err("战斗创伤须保留实际参战玩家与原角色实例".into());
        }
        if !sources.is_empty() {
            if self.window != Some(Window::After(region, 1)) { return Err("创伤替代只能用于已获胜的战斗奖励".into()); }
            let counts = self.contest_counts(region, 1);
            let kill = self.region_characters(region).filter(|c| self.team(c.controller) == team
                && !c.face_down && !c.exhausted && self.icons(c, region).combat > 0)
                .map(|c| self.effective_kill(c)).sum::<u32>();
            if team > 1 || counts[team] <= counts[1 - team] || amount != counts[team] - counts[1 - team] + kill {
                return Err("创伤不额外增加或改写战斗伤害总数".into());
            }
        }
        Ok(())
    }
    pub(crate) fn validate_combat_wound_state(&self) -> RuleResult<()> {
        for e in &self.effects { match e {
            Effect::Damage { seat, region, amount, wound_sources } => {
                if *seat >= self.players.len() { return Err("战斗创伤玩家无效".into()); }
                self.validate_combat_wound_reward(*region, self.team(*seat), *amount, wound_sources, Some(*seat))?;
            }
            Effect::Recipient { team, region, contest, amount, wound_sources, seats } => {
                // A retired region cancels its queued reward at execution.
                if *contest == 1 && self.region_live(*region) {
                    if *seats != self.combat_recipient_seats(*team, *region, *amount, wound_sources)? {
                        return Err("战斗奖励队列不能改写实际领取者名单".into());
                    }
                }
                else if *contest == 1 && !wound_sources.is_empty() { return Err("失效地区不能保留创伤奖励".into()); }
                else if !wound_sources.is_empty() { return Err("非战斗奖励不能携带创伤".into()); }
            }
            _ => (),
        } }
        if let Some(p) = &self.pending { match &p.resolution {
            ChoiceResolution::Damage { region, wound_sources } => {
                if p.seat >= self.players.len() { return Err("战斗创伤玩家无效".into()); }
                self.validate_combat_wound_reward(*region, self.team(p.seat), p.choice.amount.unwrap_or(0), wound_sources, Some(p.seat))?;
                if !wound_sources.is_empty() {
                    let options: Vec<_> = self.region_characters(*region).filter(|c| !c.face_down && self.is_enemy(p.seat, c))
                        .map(|c| self.option(c, p.seat, Some(*region), None)).collect();
                    if serde_json::to_value(&options).unwrap() != serde_json::to_value(&p.choice.options).unwrap()
                        || p.choice.player_id != player_id(p.seat) || p.choice.kind != "damage"
                        || p.choice.min != Some(0) || p.choice.max != Some(options.len()) {
                        return Err("战斗创伤只能分配给原地区的敌方角色".into());
                    }
                }
            }
            ChoiceResolution::Recipient { region, contest, amount, wound_sources } => {
                if p.seat >= self.players.len() { return Err("战斗创伤玩家无效".into()); }
                if *contest == 1 {
                    let seats = self.combat_recipient_seats(self.team(p.seat), *region, *amount, wound_sources)?;
                    let expected = Choice { id: p.choice.id.clone(), kind: "recipient".into(),
                        title: "选择本次实际参战的奖励执行者".into(), description: "选择符合数量限制的选项，然后确认。".into(),
                        player_id: player_id(seats[0]), options: seats.iter().map(|s| ChoiceOption {
                            id: player_id(*s), label: self.players[*s].name.clone(), card: None,
                        }).collect(), preview_cards: vec![], min: Some(1), max: Some(1), amount: Some(*amount), allow_decline: Some(false) };
                    if p.seat != seats[0] || p.choice.id.is_empty()
                        || serde_json::to_value(&p.choice).unwrap() != serde_json::to_value(&expected).unwrap() {
                        return Err("战斗奖励必须由原选择者恰好选择一名实际参战玩家".into());
                    }
                }
                else if !wound_sources.is_empty() { return Err("非战斗奖励不能携带创伤".into()); }
            }
            _ => (),
        } }
        Ok(())
    }
    /// Apply every marker before checking lethal defense, preserving real
    /// ReceiveWound events (including lethal recipients) and avoiding Damage
    /// prevention/received-damage triggers for replacement markers.
    pub(crate) fn place_wounds(&mut self, allocations: BTreeMap<String, u32>) -> RuleResult<()> {
        let mut recipients = vec![];
        for (id, amount) in allocations.into_iter().filter(|(_, n)| *n > 0) {
            if let Some((_, c)) = self.in_play(&id) {
                c.wounds.checked_add(amount).ok_or("创伤标志超出数值范围")?;
                self.place_board_markers(&id, crate::batch4_markers::MarkerKind::Wound, amount);
                recipients.push(id);
            }
        }
        for id in recipients {
            if let Some((region, c)) = self.board(&id) {
                self.emit_event(c.controller, self.source_snapshot(c, Some(region)), Event::ReceiveWound);
            }
        }
        Ok(())
    }
}
