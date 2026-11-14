# Original-source preservation

All authoring in this task targets `main_PBFT.tex`, its new `*-pbft.tex` figures, and this review directory. The original `main_v1.tex` and its shared figures were never written by this task.

The initial source had SHA-256 `417d05474b267842e2c82ca76b027aa872ad5a247c1e07c8237faf7e9db7d943`. During work, the live original changed to SHA-256 `8b35398116f4b744556058129713a73f309fac65a22f6fb4cce01fbb4fcdf905` (mtime 2026-09-28 12:30:01). The change removes three lines describing honest verification, persistence, and retention. It was not made by this task and was not reverted.

The starting source was reconstructed into `source/main_v1.tex` from that single observed difference, and its SHA-256 was checked against the initial hash. The PBFT draft keeps verification, signing persistence, and retention as explicit protocol requirements necessary for its security arguments. Original figure hashes still match the starting baseline.

Final observation: the live original changed again during work and has SHA-256 `ed64009d7395b9a9ff3ef8db1ba10f2017fe56cc085b7c504c2be6e5093dd4f8`. Its observed difference from the starting snapshot is recorded in `source-observed.diff`. It was not overwritten or reverted.
