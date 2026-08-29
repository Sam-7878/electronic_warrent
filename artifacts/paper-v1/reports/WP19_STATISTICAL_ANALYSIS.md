# WP19 Statistical Analysis and Generated Material

The analysis pipeline generated:

- count, mean, median, standard deviation, P50/P95/P99, 95% t confidence
  intervals, minimum, maximum, and failure rate
- absolute/relative B0 overhead, Cohen's d, difference confidence intervals,
  and Mann–Whitney U results using independent-run means
- 14 figures in PNG and SVG
- 10 CSV tables

The 30 run means, rather than 210,000 within-run rows, are the sampling units for
the primary confidence intervals and tests. This avoids treating operations from
one run as independent experiments.

`SHA256SUMS.json` freezes every raw file before processing. The figure manifest
includes the hash of that raw inventory, and the verification script confirmed
no missing, added, or changed raw file.
