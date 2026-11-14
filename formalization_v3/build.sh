#!/bin/sh
set -eu
cd "$(dirname "$0")"
mkdir -p build
export TEXINPUTS="$PWD/build:${TEXINPUTS:-}"
for name in sync_hotstuff_part0_original sync_hotstuff_rr_part1_core sync_hotstuff_rr_part2_rotation_v4_6 sync_hotstuff_rr_part3_staking_v3 sync_hotstuff_rr_part4_force_v0_1
do
  latexmk -pdf -outdir=build -interaction=nonstopmode -halt-on-error "$name.tex"
  cp "build/$name.pdf" "$name.pdf"
done
