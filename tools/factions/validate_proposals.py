"""Validate disabled source-grounded design candidates, never game acceptance."""
from collections import Counter


def validate_proposals(proposals, rulings, specifications, coverage):
    errors = []
    def require(ok, message):
        if not ok:
            errors.append(message)
    specs = specifications['cards']
    mechanisms = {m['id'] for m in coverage['mechanisms']}
    require(proposals['schemaVersion'] == rulings['schemaVersion'] == 1, 'Unknown candidate schema')
    require(proposals['auditedCommit'] == rulings['auditedCommit'] == coverage['auditedCommit'], 'Candidate baseline differs')
    require(not proposals['officialPreconstructedDecks'], 'Candidates cannot claim official deck lists')
    def disabled(value):
        return not value['selectionEnabled'] and value['gameAcceptance'] == 'notAccepted' and value['strategyUiPlaytest'] == 'notRun'
    require(disabled(proposals), 'Source candidates cannot attest to game/UI acceptance')
    all_colors = set()
    for deck in proposals['decks']:
        require(disabled(deck), 'Deck source completion cannot unlock: ' + deck['id'])
        require(len(set(deck['colors'])) == 2 and '中立' not in deck['colors'], 'Candidate must keep two official colors without neutral')
        all_colors.update(deck['colors'])
        names = Counter()
        total = 0
        for entry in deck['entries']:
            spec = specs.get(entry['cardId'])
            require(spec is not None, 'Candidate has no full source: ' + entry['cardId'])
            if spec is None:
                continue
            fields = spec['fields']
            require(spec['sourceVerification']['status'] == 'completeGameplayFieldsForPinnedImage'
                    and fields['basicType'] in {'角色', '事务', '附属'}, 'Candidate contains incomplete or nonplayer source')
            require(entry['name'] == fields['name'] and entry['colorKey'] == fields['colorKey']
                    and entry['colorKey'] in deck['colors'] and entry['colorKey'] != '中立', 'Candidate field or neutral-color violation')
            require(entry['sourceSpecRef'] == 'docs/factions/card-specifications.json#cards/' + entry['cardId'], 'Candidate reference differs')
            require(type(entry['count']) is int and 1 <= entry['count'] <= 3, 'Candidate ordinary copy limit violated')
            if type(entry['count']) is int:
                total += entry['count']
                names[fields['name']] += entry['count']
        require(total == deck['totalCards'] == 50 and deck['neutralCardCount'] == 0, 'Candidate must contain 50 verified nonneutral cards')
        require(all(n <= 3 for n in names.values()), 'Same-name variants must share the copy limit')
        require(set(deck['sharedMechanismPriorities']) <= mechanisms, 'Candidate mechanism not audited')
    require(all_colors == {'黄', '绿', '蓝', '红', '灰', '白', '黑', '紫'}, 'Candidate set must cover eight official colors')
    world = rulings['officialBaseWorld']
    entries = world['entries']
    require(len(entries) == world['physicalCardCount'] == world['distinctNames'] == 10
            and len({e['cardId'] for e in entries}) == len({e['name'] for e in entries}) == 10
            and all(e['count'] == 1 for e in entries), 'Official base product requires ten distinct single physical cards')
    require(not world['selectionEnabled'] and world['runtimeAcceptance'] == 'notAccepted', 'World source inventory cannot attest to implementation')
    for entry in entries:
        spec = specs.get(entry['cardId'], {})
        fields = spec.get('fields', {})
        require(spec.get('sourceVerification', {}).get('status') == 'completeGameplayFieldsForPinnedImage'
                and fields.get('basicType') == '地区' and fields.get('name') == entry['name']
                and fields.get('printedCollectorCode') == entry['printedCollectorCode'], 'World inventory source differs')
    require(set(world['evidenceIds']) <= set(coverage['evidence']), 'Unknown world inventory evidence')
    require(rulings['JC125']['printedCost'] == specs['JC125']['fields']['printedCost'] == 0
            and not rulings['JC125']['loyalty'] and not rulings['JC125']['domains'], 'Zero-cost source cannot change its loyalty/domains')
    return errors
