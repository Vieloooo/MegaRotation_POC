"""Small witnesses for report predicates, not a full protocol simulator.
Only checks arithmetic/equalities under each report's stated interpretation.
"""
from dataclasses import dataclass

# S1: future-cached publication passes listed time guards at an earlier event.
publication_key, deadline, current_key, expiry95 = 106, 111, 100, 105
assert publication_key < deadline and current_key < expiry95
assert publication_key >= expiry95  # at its actual event, admission is too late

# S2: stored and timeout-normalized cursors demand different prev values.
@dataclass(frozen=True)
class Cursor:
    height: int
    origin: int
    digest: int
stored = Cursor(100, 20, 123)
effective = Cursor(100, 20, 0)
assert stored != effective
assert not any(prev == stored and prev == effective for prev in [stored,effective])

# S3: universal conditions on an empty certificate set do not lower-bound x.
gpos, x, reset_position = [], 100, 200
assert all(p <= x for p in gpos)
assert x < reset_position
old_G, new_Gprime = [0,1], [0,1,2,3]
assert new_Gprime != old_G[:len(new_Gprime)]

# S4: accepted low target truncates G; a new expiry may predate pc.
cursor, G, D, target = 0, [0,1,2], [0,1,2], 1
assert target > cursor and len(D[:target+1]) < len(G)
pc, next_expiry, session_end = 102, 91+10, 101+10
assert next_expiry < pc < session_end

# S5: lineage membership does not imply inheritance of the conflict.
parent_chain_heights = [98,100]
child_anchor, conflict_height = 98,100
assert child_anchor in parent_chain_heights
assert child_anchor < conflict_height

# S6: inclusion uncertainty exceeds the interval tolerable by a fixed target.
a, b, prep, drift, epsilon = 12, 600, 1500, 1800, 120
assert b-a > drift-epsilon-prep
assert 1512 < b+prep

# S7: the next RESUME closes the session before its first slot.
for t in range(0,60,4):
    assert t+4+20 < t+25
print("7 predicate witnesses passed; no implementation or end-to-end proof claimed.")
