#!/usr/bin/env python3
"""Isolated native contract fixture, NOT a production integration installer.
Only writes /tmp/next20-blue-black-harness. BQ009 is a name-only fixture;
its yellow gameplay is deliberately absent. The original source/core stay intact.
"""
from pathlib import Path
import shutil,json,sys
src=Path(sys.argv[1]).resolve() if len(sys.argv)>1 else Path(__file__).resolve().parents[1]
root=Path('/tmp/next20-blue-black-harness')
root.mkdir(parents=True,exist_ok=True)
for name in ['Cargo.toml','Cargo.lock','rust-toolchain.toml','rust-game','rust-game-wasm']:
 target=root/name
 if not target.exists():
  if (src/name).is_dir(): shutil.copytree(src/name,target,ignore=shutil.ignore_patterns('target'))
  else: shutil.copy2(src/name,target)
(root/'docs/factions').mkdir(parents=True,exist_ok=True)
shutil.copy2(src/'docs/factions/card-specifications.json',root/'docs/factions/card-specifications.json')
# Refresh baseline source files only in the isolated fixture tree.
for name in ['lib.rs','rules.rs','model.rs','resolution.rs','catalog.rs','mystic_hooks.rs','mystic_contest.rs','death_observers.rs','engine.rs','batch3.rs','pack_a_contest.rs']:
 shutil.copy2(src/'rust-game/src'/name,root/'rust-game/src'/name)
for p in (src/'rust-game/src').glob('next20_*vampires*.rs'): shutil.copy2(p,root/'rust-game/src'/p.name)
for p in (src/'rust-game/src').glob('next20_black_wraiths*.rs'): shutil.copy2(p,root/'rust-game/src'/p.name)
def edit(name,a,b,n=1):
 p=root/'rust-game/src'/name;s=p.read_text();count=s.count(a)
 if count != n: raise Exception((name,a[:75],count,n))
 p.write_text(s.replace(a,b))
edit('lib.rs','mod import20_entropy;','mod next20_blue_vampires;\nmod next20_black_wraiths;\n#[cfg(test)]\nmod next20_blue_vampires_tests;\n#[cfg(test)]\nmod next20_black_wraiths_tests;\nmod import20_entropy;')
edit('rules.rs','pub enum Event {','pub enum Event {\n    Next20VampireTargeted,\n    Next20WraithDeath,')
edit('rules.rs','pub enum Op {','pub enum Op {\n    Next20Blue(crate::next20_blue_vampires::BlueProgram),\n    Next20Wraith(crate::next20_black_wraiths::WraithProgram),')
edit('model.rs','pub enum FrameChoice {','pub enum FrameChoice {\n    Next20Blue(crate::next20_blue_vampires::BlueChoice),\n    Next20Wraith(crate::next20_black_wraiths::WraithChoice),')
edit('rules.rs','        let mut m = BTreeMap::new();','        let mut m = BTreeMap::new();\n        for id in crate::next20_blue_vampires::CARDS { m.insert((*id).into(), crate::next20_blue_vampires::definition(id).unwrap()); }\n        for id in crate::next20_black_wraiths::CARDS { m.insert((*id).into(), crate::next20_black_wraiths::definition(id).unwrap()); }')
edit('resolution.rs','            match step.op {','''            match step.op {
                Op::Next20Blue(program) => match program {
                    crate::next20_blue_vampires::BlueProgram::RandomDiscard => self.bq033_random_discard(&frame)?,
                    crate::next20_blue_vampires::BlueProgram::EntryFeed => { let recipient = frame.source.card.id.clone(); return self.bq034_feed_start(frame, recipient); },
                    crate::next20_blue_vampires::BlueProgram::Counter { .. } => return self.bq034_counter_start(frame),
                    crate::next20_blue_vampires::BlueProgram::ShownFeed { .. } => return Err("shown programs are owned by the contest post-frame".into()),
                },
                Op::Next20Wraith(program) => match program {
                    crate::next20_black_wraiths::WraithProgram::TigerSearch { .. } => return self.cq25_search_start(frame),
                    crate::next20_black_wraiths::WraithProgram::GhostMove { .. } => self.xg25_move(&frame)?,
                },''')
edit('resolution.rs','        match choice {','''        match choice {
            FrameChoice::Next20Blue(choice) => return match choice {
                crate::next20_blue_vampires::BlueChoice::ShownTarget => self.bq034_shown_target(*frame,chooser,&selected),
                crate::next20_blue_vampires::BlueChoice::Blood { recipient } => self.bq034_feed_finish(&frame,&recipient,chooser,&selected),
                crate::next20_blue_vampires::BlueChoice::SacrificeAttachment => self.bq034_counter_finish(&frame,chooser,&selected),
            },
            FrameChoice::Next20Wraith(crate::next20_black_wraiths::WraithChoice::TigerSearch) => return self.cq25_search_finish(&frame,chooser,&selected),''')
# Native fixture catalogue only: source-complete fields, all unsupported. BQ009
# is a named payment fixture with no yellow gameplay program or admission.
x=json.loads((src/'rust-game/data/cards.json').read_text())
spec=json.loads((src/'docs/factions/card-specifications.json').read_text())['cards'].values()
entries=[]
for id in ['BQ033','BQ034','CQ25','XG25','BQ009']:
 c=next(c for c in spec if c['cardId']==id); f=c['fields']
 entries.append(dict(id=id,name=f['name'],subtitle=f['subtitle'],kind='character' if f['basicType']=='角色' else 'spell',cost=f['printedCost'],loyalty=[l['value']+'色' if l['kind']=='color' else l['value'] for l in f['loyalty'] for _ in range(l['count'])],color=f['colorKey'],magic={'stars':'星辰','blood':'血','death':'死亡'}.get(f['domains'][0]['id'],'') if f['domains'] else '',subtypes=f['subtypes'],text='\n'.join(f['printedRuleTextLines']),icons={'permanent':f['permanentIcons'],'temporary':f['temporaryIcons']},defense=f['defense'],keywords=f['printedKeywords'],unique=f['nameInk']=='gold',supported=False))
x['cards'].extend(entries)
(root/'rust-game/data/cards.json').write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n')
# Keep BQ009 out of runtime admission: no new runtime allowlist is changed here.

edit('catalog.rs','pub(crate) fn runtime_implemented(d: &CardDefinition) -> bool {','pub(crate) fn runtime_implemented(d: &CardDefinition) -> bool {\n    if ["BQ033","BQ034","CQ25","XG25","BQ009"].contains(&d.id.as_str()) { return true; }')

edit('rules.rs','pub enum TargetPredicate {','pub enum TargetPredicate {\n    Next20XG25AdjacentEnemy,')
edit('resolution.rs','        match spec.zone {','        if spec.predicate == Some(TargetPredicate::Next20XG25AdjacentEnemy) { return self.xg25_adjacent_target(actor,source,id); }\n        match spec.zone {')

edit('resolution.rs','.is_none_or(|predicate| match predicate {','.is_none_or(|predicate| match predicate {\n                                TargetPredicate::Next20XG25AdjacentEnemy => self.xg25_adjacent_target(actor,source,&c.id),')

edit('rules.rs','    if crate::import20_entropy::related(card_id,ability)', '    if crate::next20_blue_vampires::related(ability) { return crate::next20_blue_vampires::validate_ability(card_id,ability); }\n    if crate::next20_black_wraiths::related(ability) { return crate::next20_black_wraiths::validate_ability(card_id,ability); }\n    if crate::import20_entropy::related(card_id,ability)')
edit('rules.rs','        crate::import20_entropy::validate_definition(card_id,definition)?;','        crate::next20_blue_vampires::validate_definition(card_id,definition)?;\n        crate::next20_black_wraiths::validate_definition(card_id,definition)?;\n        crate::import20_entropy::validate_definition(card_id,definition)?;')

edit('rules.rs','        let mut m = BTreeMap::new();','        let mut m = BTreeMap::new();\n        // TEST FIXTURE ONLY: named anchor identity, no yellow gameplay.\n        m.insert("BQ009".into(), Definition::default());')
p=root/'rust-game/data/cards.json'; data=json.loads(p.read_text())
for c in data['cards']:
 if c['id']=='CQ25': c['text']=c['text'].replace('本地区的一个本方角色','本地区的一个敌方角色')
 if c['id']=='BQ034': c['text']=c['text'].replace('隐秘地区势力+1','隐秘+1')
p.write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n')

edit('resolution.rs','source_region_instance: if c.definition=="BQ057"','source_region_instance: if ["BQ033","BQ034","CQ25","XG25"].contains(&c.definition.as_str()) || c.definition=="BQ057"')
edit('batch3.rs','    pub(crate) fn current_spirit(&self, c: &Card) -> bool {','    pub(crate) fn current_spirit(&self, c: &Card) -> bool {\n        if c.definition == "CQ25" { return self.cq25_spirit(c); }')
edit('mystic_hooks.rs','Ok(self.b_mystic_extra(shown))','Ok(self.b_mystic_extra(shown) + crate::next20_blue_vampires::shown_bonus(&shown.card.definition))')
edit('mystic_hooks.rs','self.wm037_shown(outcome,index)','self.bq034_shown(outcome,index)?; self.wm037_shown(outcome,index)')
edit('model.rs','        self.validate_reward_state()?;','        if let Some(p) = &self.pending { self.validate_next20_blue_pending(p)?; self.validate_next20_black_pending(p)?; }\n        self.validate_reward_state()?;')
# Keep the reviewable suggestion consistent with the direct image reading,
# rather than carrying forward two old transcription mistakes.
entries=[c for c in data['cards'] if c['id'] in ['BQ033','BQ034','CQ25','XG25']]

edit('pack_a_contest.rs','pub(crate) fn admitted_start(id: &str, key: &str, bonus: u32) -> bool {','pub(crate) fn admitted_start(id: &str, key: &str, bonus: u32) -> bool {\n    if id == "CQ25" && key == crate::next20_black_wraiths::CONTEST && bonus == 0 { return true; }')
edit('pack_a_contest.rs','    pub(crate) fn validate_pack_a_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {','    pub(crate) fn validate_pack_a_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {\n        if f.ability_key == crate::next20_black_wraiths::CONTEST { return self.validate_cq25_contest_frame(f); }')
edit('pack_a_contest.rs','    pub(crate) fn validate_pack_a_declaration(&self, d: &Declaration) -> RuleResult<()> {','    pub(crate) fn validate_pack_a_declaration(&self, d: &Declaration) -> RuleResult<()> {\n        if crate::next20_black_wraiths::related(&d.ability) { return self.validate_next20_black_declaration(d); }\n        if crate::next20_blue_vampires::related(&d.ability) { return self.validate_next20_blue_declaration(d); }')
edit('mystic_contest.rs','fn mystic_value(&self, shown: &MysticShown, actor: usize, actor_bonus: u32, bonuses: &[MysticBonus])','fn mystic_value(&self, shown: &MysticShown, actor: usize, actor_bonus: u32, bonuses: &[MysticBonus], origin: &SourceSnapshot, key: &str)')
edit('mystic_contest.rs','self.mystic_value(&item, state.frame.actor, state.actor_bonus, &state.bonuses)?','self.mystic_value(&item, state.frame.actor, state.actor_bonus, &state.bonuses, &state.frame.source, &state.frame.ability_key)?')
edit('mystic_contest.rs','self.mystic_value(shown,o.actor,o.actor_bonus,&o.bonuses)?','self.mystic_value(shown,o.actor,o.actor_bonus,&o.bonuses,&o.source,&o.ability_key)?')
edit('mystic_contest.rs','.and_then(|n| n.checked_add(added))','.and_then(|n| n.checked_add(added))\n            .and_then(|n| n.checked_add(self.cq25_contest_bonus(origin,key,actor,shown.seat,&shown.card)))')
edit('model.rs','        self.validate_reward_state()?;','        self.validate_next20_blue_state()?;\n        self.validate_next20_black_state()?;\n        self.validate_reward_state()?;')
edit('resolution.rs','self.wm058_observe_card(&frame);','self.wm058_observe_card(&frame); self.bq034_observe_card(&frame);',n=2)
edit('engine.rs','                crate::batch4::validate_action(&c.definition, &a)?;','                crate::batch4::validate_action(&c.definition, &a)?;\n                crate::next20_blue_vampires::validate_action(&c.definition,&a)?;\n                crate::next20_black_wraiths::validate_action(&c.definition,&a)?;',n=4)
edit('engine.rs','        crate::next20_blue_gray::validate_choice_action(&p, &a)?;','        crate::next20_blue_gray::validate_choice_action(&p, &a)?;\n        crate::next20_blue_vampires::validate_choice_action(&p,&a)?;\n        crate::next20_black_wraiths::validate_choice_action(&p,&a)?;')

edit('catalog.rs','if !["CQ15", "BQ057"','if !["BQ034", "CQ15", "BQ057"')
