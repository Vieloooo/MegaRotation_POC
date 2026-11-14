#!/bin/bash
# Rebuild all ten vector figures and their Markdown previews.
# Dependencies: XeLaTeX (fontspec, xeCJK, TikZ, standalone), Poppler.
set -euo pipefail

figure_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_dir="$(cd -- "$figure_dir/../../.." && pwd)"
pdf_dir="$repo_dir/output/pdf"
scratch_parent="$repo_dir/tmp/pdfs"
mkdir -p "$pdf_dir" "$scratch_parent"
figure_scratch="$(mktemp -d "$scratch_parent/261027-figures.XXXXXX")"
trap 'rm -rf -- "$figure_scratch"' EXIT

cd "$figure_dir"
if ! xelatex -interaction=nonstopmode -halt-on-error \
  -output-directory="$figure_scratch" \
  -jobname=261027_liveness_recovery_figures main.tex \
  > "$figure_scratch/build.txt" 2>&1; then
  cat "$figure_scratch/build.txt"
  exit 1
fi
if rg -n 'Missing character:|Overfull|^!' "$figure_scratch/261027_liveness_recovery_figures.log"; then
  exit 1
fi

figure_ids=(1 2 2a 2b 2c 3 3b 4 4b 5)
figure_names=(architecture four_chains publisher_failure batch_failure recovery_alignment voting block_lifecycle resume gfr_ordering recovery_examples)
cp "$figure_scratch/261027_liveness_recovery_figures.pdf" "$pdf_dir/261027_liveness_recovery_figures.pdf"
pdfseparate "$pdf_dir/261027_liveness_recovery_figures.pdf" "$figure_scratch/page-%d.pdf"
for figure_offset in "${!figure_names[@]}"; do
  figure_page=$((figure_offset+1))
  figure_id="${figure_ids[$figure_offset]}"
  figure_name="${figure_names[$figure_offset]}"
  cp "$figure_scratch/page-$figure_page.pdf" "$pdf_dir/261027_fig${figure_id}_${figure_name}.pdf"
  pdftoppm -singlefile -png -scale-to 2200 \
    "$figure_scratch/page-$figure_page.pdf" "$figure_dir/figure$figure_id"
done
printf 'Built ten vector figures, a combined PDF, and ten Markdown previews.\n'
