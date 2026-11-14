"""Finite checks of proof premises, not a full PBFT implementation/model checker."""
from itertools import product
from fractions import Fraction
from hashlib import sha256
from pathlib import Path
import json

def subsets(n): return range(1 << n)
def weight(mask, w): return sum(x for j,x in enumerate(w) if mask >> j & 1)
profiles=[(1,)*4,(1,)*7,(1,1,1,4),(1,1,2,3),(1,2,3,4,5)]
profiles += list(product(range(1,4),repeat=4))
checks=0;profiles_checked=0
for w in dict.fromkeys(profiles):
 n=len(w); W=sum(w)
 qs=[x for x in subsets(n) if 3*weight(x,w)>2*W]
 bs=[x for x in subsets(n) if 3*weight(x,w)<W]
 for b in bs:
  for c in qs:
   hc=c & ~b
   assert 3*weight(hc,w)>W
   for vc in qs:
    # Every future VIEWCHANGE quorum reports from at least one honest committer.
    assert hc & vc
    assert (c & vc & ~b)
    checks+=1
 profiles_checked+=1
# Retry timing with entry offsets 0, 1/4, 1/2, 3/4, 1 in units Delta_R.
timing_checks=0
for offsets in product([Fraction(x,4) for x in range(5)],repeat=4):
 if min(offsets)!=0: continue
 for primary in range(4):
  report_arrival=max(offsets)+1
  primary_send=offsets[primary]+2
  assert report_arrival<=primary_send
  newview_done=primary_send+1
  prepare_done=newview_done+1
  commit_done=prepare_done+1
  relay_done=commit_done+1
  assert relay_done<=7
  assert relay_done<8
  timing_checks+=1
out={
 'source_sha256':sha256(Path('formalization/main_PBFT.tex').read_bytes()).hexdigest(),
 'weighted_profiles':profiles_checked,
 'quorum_carryover_cases':checks,
 'bounded_entry_skew_timing_cases':timing_checks,
 'failures':0,
 'scope':'Finite quorum, timing-envelope, checks only; not an implementation test or exhaustive asynchronous execution proof.'
}
Path('formalization/reviews/pbft-revision-2026-09-28/invariant-results.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps(out,indent=2))
