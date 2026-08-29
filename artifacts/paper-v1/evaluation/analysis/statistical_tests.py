#!/usr/bin/env python3
"""Baseline effect sizes and non-parametric tests over independent run means."""

from __future__ import annotations

import csv
import math
import statistics
from collections import defaultdict
from pathlib import Path

from scipy import stats


def main() -> int:
    grouped: dict[tuple[str, str], list[float]] = defaultdict(list)
    for path in sorted(Path("evaluation/results/raw/full_benchmark").glob("run_*.csv")):
        with path.open(newline="", encoding="utf-8") as handle:
            for row in csv.DictReader(handle): grouped[(row["run_id"], row["baseline_id"])].append(float(row["t_total_ns"]))
    samples: dict[str, list[float]] = defaultdict(list)
    for (_, baseline), values in grouped.items(): samples[baseline].append(statistics.fmean(values))
    control = samples["B0"]
    rows = []
    for baseline in sorted(samples):
        if baseline == "B0": continue
        treatment = samples[baseline]
        difference = statistics.fmean(treatment) - statistics.fmean(control)
        pooled = math.sqrt(((len(control)-1)*statistics.variance(control)+(len(treatment)-1)*statistics.variance(treatment))/(len(control)+len(treatment)-2))
        effect = difference / pooled if pooled else 0.0
        standard_error = math.sqrt(statistics.variance(control)/len(control)+statistics.variance(treatment)/len(treatment))
        margin = stats.t.ppf(.975, len(control)+len(treatment)-2) * standard_error
        test = stats.mannwhitneyu(control, treatment, alternative="two-sided")
        rows.append({"control": "B0", "treatment": baseline, "absolute_overhead_ns": difference,
                     "relative_overhead": statistics.fmean(treatment)/statistics.fmean(control)-1,
                     "cohen_d": effect, "difference_ci95_low": difference-margin,
                     "difference_ci95_high": difference+margin, "mann_whitney_u": test.statistic,
                     "p_value": test.pvalue, "sample_unit": "independent_run_mean", "n_control": len(control), "n_treatment": len(treatment)})
    output = Path("evaluation/results/processed/baseline_comparisons.csv")
    with output.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0])); writer.writeheader(); writer.writerows(rows)
    return 0


if __name__ == "__main__": raise SystemExit(main())
