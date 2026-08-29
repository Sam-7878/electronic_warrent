# WP11 TLC Bounded Model Check

## 실행 결과

| Mode | Generated | Distinct | Depth | Deadlock | Violations |
|---|---:|---:|---:|---|---|
| Safety | 490 | 204 | 11 | 0 | 0 |
| Liveness | 294 | 130 | 10 | 0 | 0 |

TLC 2.19 (TLA+ tools v1.7.4, pinned SHA-1) ran with two workers. Raw stdout,
stderr, command, Java/TLC versions, config/model hashes, exit code, and parsed
summary are preserved under `formal/results/tlc/`.

Safety covers SAFE-001--009 inside the model. SAFE-010 is the companion Cargo
dependency/static-source check. Liveness covers authorized progress, expiry
progress, and quarantine review under the weak fairness assumptions written in
`LiveSpec`.

## 반례 처리

Development exploration found and corrected two abstraction defects: revocation
before activation was incorrectly required to imply an Adapter commit, and a
post-activation suspended state was omitted from published states. A liveness
property also originally excluded legal release/full-execution/revocation
outcomes after the expiry boundary. The history is recorded in
`formal/traces/counterexamples/README.md`.

## Claim boundary

This is bounded model checking of the recorded finite configuration. It is not
an unbounded mathematical proof and does not prove arbitrary executable code.
