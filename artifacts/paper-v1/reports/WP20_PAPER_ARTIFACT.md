# WP20 Paper Artifact

`evaluation/build_paper_artifact.py` creates `artifacts/paper-v1/` from the
tagged source. It includes source, formal model/results/traces, raw and processed
data, figures, tables, schemas, fixtures, reports, reproduction instructions,
an artifact manifest, and SHA-256 checksums.

Text copies replace the local absolute workspace path with `<WORKSPACE>` while
the original raw files remain immutable in the repository. The artifact secret
scanner rejects private-key headers, AWS-style access keys, Windows/WSL local
workspace paths, and the OpenBSD connection-file name.

The artifact directory is ignored by Git so it can be built after the tagged
tree is clean. Its manifest records the exact source commit and tag.
