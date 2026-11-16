# Slice A bound arithmetic correction

Independent literal construction after the initial contract review detected a four-byte undercount in the annex and in that review's arithmetic summary. The initial source expectations remain unchanged. No Builder candidate was inspected and no candidate test was run.

GFR FinalProof is `7 + frame(descriptor) + raw32(target) + frame(proof)`, so its maximum is `7+4+16384+32+4+32768 = 49,199`, **not49,195**. With256-byte source_id and ten197-byte Body messages, the full06 encoded attempt is `47+256+10*(8+197)+(8+49199) = 51,560`, **not51,556**. Both remain below the frozen public limits. Physical186, Body197, and private116,935<131,072 were correct.

This supersedes the two incorrect numbers in `contract-review-v1.md`. The lead was notified immediately before any candidate inspection. The construction assertion stopped before writing wire artifacts; the corrected `wire-vectors-v1.json` uses the explicit formulas and values above. The private bounds and helper/Publisher semantic review remain valid. Exact annex hashes after the lead's correction/freeze are recorded separately.
