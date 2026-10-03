"""Purpose: deterministic proposed-policy cost/latency model; never changes production code.

All timings are ideal fake-clock/zero-RTT values, not real browser or game QA.
Run from the repository root; writes a compact JSON evidence file under docs/.
"""
import json
from pathlib import Path

BASELINE = "0ec1d7643fff64b3f8c98cdac343b11fea49d10e"
OUT = Path(__file__).resolve().parents[2] / "docs/evidence/autopass-background-2026-10-03/timing-model.json"


def rate(foreground, hidden_on, hidden_off, delay_on):
    return foreground * 60 / 1.5 + hidden_on * 60 / delay_on + hidden_off * 60 / 12


def discovery_model(delay_ms, samples=120_000):
    # Midpoint samples uniformly distributed across a healthy polling interval.
    post_delays = sorted(delay_ms - (i + 0.5) * delay_ms / samples + 550 for i in range(samples))
    return {
        "pollDelayMs": delay_ms, "autoPassDelayMs": 550, "uniformArrivalSamples": samples,
        "eventToPostMeanMs": round(sum(post_delays) / samples, 3),
        "eventToPostP95Ms": round(post_delays[int(samples * .95) - 1], 3),
        "eventToPostSupremumMs": delay_ms + 550,
    }


def main():
    scenarios = []
    for name, foreground, hidden_on, hidden_off in [
        ("one visible, three hidden opt-in", 1, 3, 0),
        ("four hidden opt-in", 0, 4, 0),
        ("four visible", 4, 0, 0),
        ("one visible, one hidden opt-in, two hidden off", 1, 1, 2),
        ("four hidden off", 0, 0, 4),
    ]:
        old = rate(foreground, hidden_on, hidden_off, 12)
        new = rate(foreground, hidden_on, hidden_off, 3)
        scenarios.append({"scenario": name, "currentPeriodicStateGetsPerMinute": old,
                          "proposedPeriodicStateGetsPerMinute": new, "increasePerMinute": new - old,
                          "increasePercent": round((new / old - 1) * 100, 3)})
    output = {
        "source": BASELINE,
        "method": "deterministic isolated mathematical model; no live API, actual browser, or engine",
        "assumptions": ["healthy online state reads", "zero RTT", "no browser timer throttling", "no pending command reconciliation duration", "initial full GET and command POST excluded from steady periodic GET counts"],
        "proposedDelayPolicyMs": {"foreground": 1500, "hiddenPlayingAutoPassOn": 3000, "hiddenOtherwise": 12000, "publicLobbyNoActiveSeatPeriodicPolls": 0},
        "firstDiscoveryAtServerTransition1Ms": [
            {"policy": "released", "staleViewKnowsOurTurn": False, "firstPollMs": 12000, "autoPostMs": 12550, "eventToPostMs": 12549},
            {"policy": "fast only after stale view knows our turn", "staleViewKnowsOurTurn": False, "firstPollMs": 12000, "autoPostMs": 12550, "eventToPostMs": 12549},
            {"policy": "proposed playing + opt-in, independent of whose turn", "staleViewKnowsOurTurn": False, "firstPollMs": 3000, "autoPostMs": 3550, "eventToPostMs": 3549},
        ],
        "uniformArrivalLatencyModels": [discovery_model(12000), discovery_model(3000)],
        "fourActiveSeatLoadModels": scenarios,
        "nonzeroRttRateFormula": "For a seat: 60 / (chosen_delay_seconds + state_RTT_seconds + awaited_pending_reconciliation_seconds) periodic GETs/minute; sum across active seats.",
        "realLimitations": ["Browser background throttling/suspension can extend both polling and 550ms submission timers without bound.", "A stale lobby view keeps the slower cadence until it discovers playing; this proposal improves playing-to-turn discovery, not lobby-to-start discovery.", "Response intent windows and private/public choosers pause automatic commands; 3s polling does not automatically complete those decisions.", "204 saves response body bytes but the server still authenticates, reads room/D1 and may process existing deadline ticks.", "Faster pending recovery may issue the original command retry sooner; it must preserve the original id/version/action."],
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(output, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"evidence": str(OUT), "scenarios": scenarios, "firstDiscoveryAtServerTransition1Ms": output["firstDiscoveryAtServerTransition1Ms"]}, ensure_ascii=False))


if __name__ == "__main__":
    main()
