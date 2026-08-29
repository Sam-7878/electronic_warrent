# WP10 Publication Baseline

## 결과

- Branch: `paper/electronic-warrant-sci`
- Experiment source revision: `7d8ecad`
- Build profile: `release`
- Environment: Ubuntu 24.04 WSL2, Python 3.12.13, stable Rust, OpenJDK 21
- Final release tag and clean-tree manifest are created only after all evidence is committed.

`evaluation/generate_publication_manifest.py` refuses a dirty tracked tree and
records CPU, memory, storage, OS/kernel, tool versions, Cargo.lock, schema, and
fixture hashes. The generated manifest is intentionally ignored so that running
it after the release tag does not dirty the tagged source.

## 검증 명령

The required format, Clippy, workspace test, architecture, invariant, formal,
trace, and functional commands are included in CI. Full outcomes are summarized
in the final report and publication manifest.

## 해석 제한

The experiment rows reference the frozen implementation commit `7d8ecad`. The
final tag also contains reports and evidence, so its commit can be later than the
experiment source revision without changing the measured binary.
