//! Two source-bound finite programs. The region/cohort policies are approved
//! project rules (2026-10-05), not claims of card-specific official FAQs.
use crate::{catalog::card, engine::RuleResult, model::*};

impl Game {
    fn blue_original_region(&self, frame: &ResolutionFrame) -> Option<usize> {
        let r = frame.source.region?;
        let instance = frame.source.source_region_instance.as_ref()?;
        self.regions
            .get(r)
            .filter(|r| &r.card.id == instance)
            .map(|_| r)
    }
    fn jc032_match(c: &Card) -> bool {
        let d = card(&c.definition);
        d.kind == "character" && d.subtypes.iter().any(|s| s == "吸血鬼")
    }
    pub(crate) fn jc032_start(&mut self, frame: ResolutionFrame) -> RuleResult<()> {
        if self.blue_original_region(&frame).is_none() {
            self.note("血族长老：原地区已替换，局部效果不执行；费用与次数保留".into());
            return Ok(());
        }
        let actor = frame.actor;
        let inspected_ids: Vec<_> = self.players[actor]
            .deck
            .iter()
            .take(6)
            .map(|c| c.id.clone())
            .collect();
        if inspected_ids.is_empty() {
            return Ok(());
        }
        let options: Vec<_> = self.players[actor]
            .deck
            .iter()
            .take(6)
            .filter(|c| Self::jc032_match(c))
            .map(|c| self.option(c, actor, None, None))
            .collect();
        let n = usize::from(!options.is_empty());
        self.choice(
            actor,
            "jc032_top_six",
            "查看顶六张并选择吸血鬼暗藏".into(),
            options,
            n,
            n,
            None,
            ChoiceResolution::Frame {
                frame: Box::new(frame),
                choice: FrameChoice::JC032TopSix { inspected_ids },
            },
        );
        let p = self.pending.as_mut().unwrap();
        p.choice.allow_decline = Some(false);
        p.choice.description = if n == 0 {
            "没有符合的吸血鬼角色。查看后确认，将剩余牌库洗牌。"
        } else {
            "必须选择一张符合的吸血鬼角色，展示后暗藏于原地区，再将剩余牌库洗牌。"
        }
        .into();
        Ok(())
    }
    pub(crate) fn jc032_complete(
        &mut self,
        frame: &ResolutionFrame,
        inspected_ids: Vec<String>,
        selected: &[String],
    ) -> RuleResult<()> {
        let r = self.blue_original_region(frame).ok_or("原地区实例已失效")?;
        let actor = frame.actor;
        let current: Vec<_> = self.players[actor]
            .deck
            .iter()
            .take(6)
            .map(|c| c.id.clone())
            .collect();
        if current != inspected_ids || current.is_empty() {
            return Err("所看顶六张已失效".into());
        }
        let has_match = self.players[actor]
            .deck
            .iter()
            .take(6)
            .any(Self::jc032_match);
        if selected.len() != usize::from(has_match) {
            return Err("有符合牌时必须选择一张，无符合牌时选择零张".into());
        }
        if let Some(id) = selected.first() {
            let i = self.players[actor]
                .deck
                .iter()
                .take(6)
                .position(|c| &c.id == id && Self::jc032_match(c))
                .ok_or("必须选择原顶六张内的吸血鬼角色")?;
            let c = self.players[actor].deck.remove(i);
            self.note(format!(
                "{} 展示检索的 {}",
                self.players[actor].name,
                card(&c.definition).name
            ));
            let mut c = self.reset_zone_card(c);
            c.controller = actor;
            c.face_down = true;
            self.regions[r].cards.push(c);
        }
        self.shuffle_player(actor);
        Ok(())
    }
    pub(crate) fn blue_private_choice(&self, p: &Pending, viewer: usize) -> Choice {
        let mut choice = p.choice.clone();
        if let ChoiceResolution::Frame {
            frame,
            choice: FrameChoice::JC032TopSix { inspected_ids },
        } = &p.resolution
        {
            if frame.actor == viewer {
                choice.preview_cards = inspected_ids
                    .iter()
                    .filter_map(|id| self.players[viewer].deck.iter().find(|c| &c.id == id))
                    .map(|c| self.card_view(c, viewer, None, None))
                    .collect();
            }
        }
        choice
    }
    pub(crate) fn jz24_start(&mut self, frame: ResolutionFrame) -> RuleResult<()> {
        if self.blue_original_region(&frame).is_none() {
            self.note("卡迪纳追迹人：原地区已替换，局部效果不执行".into());
            return Ok(());
        }
        // Freeze once at first execution, after responses. Never recheck hand
        // size while BQ022 recycling and successive sacrifices change hands.
        let remaining_players = (0..self.players.len())
            .map(|i| (frame.actor + i) % self.players.len())
            .filter(|s| {
                !self.players[*s].eliminated
                    && self.team(*s) != self.team(frame.actor)
                    && self.players[*s].hand.len() <= 3
            })
            .collect();
        self.jz24_next(frame, remaining_players)
    }
    fn jz24_next(
        &mut self,
        frame: ResolutionFrame,
        mut remaining_players: Vec<usize>,
    ) -> RuleResult<()> {
        let r = self
            .blue_original_region(&frame)
            .ok_or("原地区实例已失效")?;
        while !remaining_players.is_empty() {
            let seat = remaining_players.remove(0);
            let options: Vec<_> = self.regions[r]
                .cards
                .iter()
                .filter(|c| {
                    c.controller == seat && !c.face_down && card(&c.definition).kind == "character"
                })
                .map(|c| self.option(c, seat, Some(r), None))
                .collect();
            if options.is_empty() {
                continue;
            }
            self.choice(
                seat,
                "jz24_sacrifice",
                "牺牲一个原地区中自己操控的角色".into(),
                options,
                1,
                1,
                None,
                ChoiceResolution::Frame {
                    frame: Box::new(frame),
                    choice: FrameChoice::JZ24Sacrifice { remaining_players },
                },
            );
            return Ok(());
        }
        // Continue this frame before Death declarations already queued by
        // remove_dead, matching existing frame-choice continuation ordering.
        self.effects.push_front(Effect::Frame {
            frame: Box::new(frame),
        });
        Ok(())
    }
    pub(crate) fn jz24_complete(
        &mut self,
        frame: ResolutionFrame,
        seat: usize,
        remaining_players: Vec<usize>,
        selected: &[String],
    ) -> RuleResult<()> {
        let r = self
            .blue_original_region(&frame)
            .ok_or("原地区实例已失效")?;
        // Reuse the Pending seat passed by the normal choice handler; never
        // infer sacrifice authority from owner or a changed selected card.
        let id = selected
            .first()
            .filter(|_| selected.len() == 1)
            .ok_or("必须选择一个角色牺牲")?;
        if self.team(seat) == self.team(frame.actor)
            || !self.regions[r].cards.iter().any(|c| {
                &c.id == id
                    && c.controller == seat
                    && !c.face_down
                    && card(&c.definition).kind == "character"
            })
        {
            return Err("须选择原地区中自己操控的正面角色".into());
        }
        self.remove_dead(id, RemovalCause::Sacrifice);
        self.jz24_next(frame, remaining_players)
    }
}
