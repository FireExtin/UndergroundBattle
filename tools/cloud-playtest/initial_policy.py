"""Preserved first UI policy: legal but can spend the sole topdeck as an asset.

Keep this implementation as regression evidence. Do not constrain game legality
to compensate for a poor player's legal strategy. The normal runner uses a later
development policy; --policy initial deliberately reproduces this first policy.
"""


def choose_initial(run, seat, view):
    legal = view.get('legalActions', [])
    if not legal: return None
    own = next(p for p in view['players'] if p['id'] == view['you'])
    lookup = {c['instanceId']: c for c in view['hand'] + view['assets'] + view['graveyard'] + [c for r in view['regions'] for c in r['characters']]}
    assets = [a for a in legal if a['kind'] == 'asset']
    if assets:
        return min(assets, key=lambda a: run.card_value(lookup.get(a.get('cardId'), {})))
    deploy = [a for a in legal if a['kind'] == 'deploy']
    conceal = [a for a in legal if a['kind'] == 'conceal']
    if run.mode == 'duel':
        preferred = 0 if own['team'] == 0 or view['turn'] == 1 else 1
    else:
        preferred = 2 if view['turn'] <= 2 else (0 if seat % 2 == 0 else 4)

    def deploy_score(action):
        region = view['regions'][action['region']]
        enemy_count = sum(next(p['team'] for p in view['players'] if p['id'] == c['controller']) != own['team'] for c in region['characters'])
        focus = 12 if action['region'] == preferred else 0
        return run.card_value(lookup.get(action.get('cardId'), {})) + focus - enemy_count * .5 + region['influence'][own['team']] * .1

    if conceal and run.coverage['conceal'] < len(run.pages):
        return max(conceal, key=deploy_score)
    if deploy:
        return max(deploy, key=deploy_score)
    reveals = [a for a in legal if a['kind'] == 'reveal']
    if reveals:
        return max(reveals, key=lambda a: run.card_value(lookup.get(a.get('cardId'), {})))
    if conceal:
        return max(conceal, key=deploy_score)
    privileges = [a for a in legal if a['kind'] == 'privilege']
    if privileges: return privileges[0]
    plays = [a for a in legal if a['kind'] == 'play' and a.get('targetId') and lookup.get(a['targetId'], {}).get('controller') != view['you']]
    if plays and run.coverage['play'] < len(run.pages) * 2: return plays[0]
    return next((a for a in legal if a['kind'] == 'pass'), None)
