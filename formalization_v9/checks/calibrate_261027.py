"""Recompute illustrative arithmetic only; not a consensus/derivation simulator.
Run: python3 formalization_v9/checks/calibrate_261027.py
"""
import json


def clock(seconds):
    day, seconds = divmod(seconds, 86400)
    h, seconds = divmod(seconds, 3600)
    m, sec = divmod(seconds, 60)
    return (f'D{day+1} ' if day else '') + f'{h:02d}:{m:02d}:{sec:02d}'


def scheduled(h):
    return (h - 1200) // 12


def origin(anchor_h, anchor_origin, h):
    return min(anchor_origin + h - anchor_h, max(anchor_origin, scheduled(h)))


def window(k):
    start = 7211 + 3600 * k
    return [start, start + 3599]


assert 1200 >= 1152 + 2 * 15 + 12
assert 1152 <= 1200 and 1200 + 12 <= 1800
assert 120 > 30 + 30 + 15
assert 36000 + 1200 + 3600 + 1800 == 42600 < 43200
assert 1200 + 300 <= 1500 <= 1800 - 120
assert scheduled(36000) == 2900 and scheduled(35999) == 2899
assert window(7) == [32411, 36010]
force_hmin = max(36011, (3050 * 12 + 1200 + 120) + 1)  # slot(act + T_init) + 1, act = ts_L1(3050) + M_a
assert force_hmin == 37921
assert origin(36010, 2900, force_hmin) == 3060
assert force_hmin - 36010 - 1 == 1910
for L, target, previous in [(7226, 88212, 43512), (6901, 84312, 39612)]:
    assert target == 12 * L + 1500
    assert target - previous == 44700  # previous is an input checkpoint, not derived here
    assert origin(target, L, target + 1) == L + 1
assert next(k for k in range(1, 100) if origin(88212, 7226, 88212 + k) == scheduled(88212+k)) == 27
assert 32490 - 32411 + 1 == 80
assert 36305 - 36000 == 305
assert 43333 - 36000 == 7333 and 39610 - 36000 + 1 == 3611
assert 78564 - 35401 + 1 == 43164
out = {
    'scope': 'Illustrative arithmetic; finalized latency and checkpoints are assumed inputs',
    'budget_slack_seconds': 43200 - 42600,
    'session_7': {'window': window(7), 'stop': clock(36011), 'end_key': clock(36131)},
    'height_36000': {
        'candidate_origin': 2900,
        'candidate_last_window_L1': 6500,
        'parent_origin': 2899,
        'fallback_parent_window_end_L1': 6499,
        'fallback_parent_window_end_time': clock(6499 * 12),
        'conservative_observation_time': clock(6501 * 12 + 1152),
    },
    'force': {'window': [37921,41520], 'first_slot': clock(37921), 'padding_count':1910},
    'resume_A': {'L1':7226, 'event_key':clock(7226*12+1200), 'target':88212,
                 'first_slot':clock(88213), 'catch_schedule_after_heights':27},
    'fill_if_assumed_checkpoint': {'count':44700,'rate_for_48_seconds':44700/48,
                                  'rate_for_300_seconds':44700/300},
    'B_replaced_suffix_if_tip_78564':43164,
}
print(json.dumps(out, ensure_ascii=False, indent=2))
