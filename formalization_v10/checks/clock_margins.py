"""Recompute the clock-drift margins of 10 §2.3; illustrative arithmetic only.
Run: python3 formalization_v10/checks/clock_margins.py
Times in milliseconds so that delta_G = 1 ms stays exact.
"""
S = 1000
MIN = 60 * S
H = 60 * MIN

d_G = 1                  # delta_G
D_G = 2 * d_G            # Delta_G
d_L1 = 4 * S             # delta_L1
T_L1 = 1152 * S          # 19.2 min
b_L1 = 12 * S
b_L2 = 1 * S
D_ts = 15 * S
T_pub = T_E = 30 * S
T_grace = 2 * MIN
T_dep = 20 * MIN
T_drift = 30 * MIN
M_a = 20 * MIN
W_exp, W_seq, T_batch = 10 * H, 12 * H, 1 * H
T_prep, T_ready, eps = 25 * MIN, 5 * MIN, 2 * MIN
T_init = 2 * MIN
handoff = 500            # 09:00:10.5 -> 09:00:11

rows = [
    ("M_a >= T_L1+b_L1+D_ts+d_L1", M_a - (T_L1 + b_L1 + D_ts + d_L1), 17 * S),
    ("T_L1+d_L1+d_G <= T_dep", T_dep - (T_L1 + d_L1 + d_G), 44 * S - 1),
    ("T_dep+b_L1 <= T_drift", T_drift - (T_dep + b_L1), 9 * MIN + 48 * S),
    ("T_grace > T_pub+T_E+D_ts+d_G", T_grace - (T_pub + T_E + D_ts + d_G), 45 * S - 1),
    ("W_exp+M_a+T_batch+T_drift+d_L1 <= W_seq",
     W_seq - (W_exp + M_a + T_batch + T_drift + d_L1), 9 * MIN + 56 * S),
    ("T_prep >= M_a", T_prep - M_a, 5 * MIN),
    ("M_a+T_ready+d_G <= T_prep+b_L2", T_prep + b_L2 - (M_a + T_ready + d_G), 999),
    ("T_prep <= T_drift-eps", T_drift - eps - T_prep, 3 * MIN),
    ("D_G < handoff margin", handoff - D_G, 498),
]
for name, margin, expected in rows:
    assert margin == expected, (name, margin, expected)
    assert margin >= 0
    print(f"{name:42s} margin {margin / S:10.3f} s")

# G2 = no: watermark wait D_ts+T_E added to the T_prep lower bound
assert T_prep + b_L2 - (M_a + T_ready + D_ts + T_E + d_G) < 0
assert 26 * MIN + b_L2 - (M_a + T_ready + D_ts + T_E + d_G) >= 0

# M_a = 15 min (261027): wait after key x
assert (-15 * MIN + b_L1 + d_L1 + T_L1) == 4 * MIN + 28 * S

# 2.6 example: T_L1 = 25 min
M_a_25 = 25 * MIN + b_L1 + D_ts + d_L1
assert M_a_25 == 25 * MIN + 31 * S
assert M_a_25 + T_ready + d_G + eps > T_drift + b_L2

# Force: worst real preparation after act(U)
assert T_init - D_ts - T_E - d_G == 75 * S - 1

# RESUME: L finalized -> activation, general execution
assert M_a - T_L1 - d_L1 == 44 * S
print("all clock-drift margins check")
