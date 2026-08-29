# HETE SCI 논문 제출용 추가 개발·검증 작업지시서

**문서 번호:** HETE-SCI-EW-DEV-002  
**대상 저장소:** `hete_sandbox`  
**대상 논문 제목:** *HETE: A Machine-Verifiable Policy Framework for DID-Based Multi-Authority Enforcement of Asset-Freezing Warrants*  
**HETE 정의:** **High-Assurance Enforcement and Trust Engine**  
**문서 목적:** 현재 구현된 HETE 전자영장 reference domain을 SCI 논문 제출 가능한 수준으로 끌어올리기 위해, 남아 있는 정형 검증·publication-grade 실험·외부 Adapter 검증·재현성 패키지·논문 근거자료를 완성하기 위한 후속 작업지시서  
**적용 원칙:** 기존 구조를 대폭 재설계하지 않고, 현재 구현의 증거 수준을 강화한다.

---

## 1. 현재 상태와 남은 결손

현재 `hete_sandbox`에는 다음이 구현되어 있다.

- 기존 `hete` 및 Voting Domain과 독립된 전자영장 reference domain
- `hete-policy`, `hete-identity`, `hete-credential`, `hete-adapter-api`
- `domain-electronic-warrant`
- `adapter-simulated-asset`
- DID/credential 기반 다기관 승인
- domain-separated signature와 nonce replay 방지
- amount-bounded reservation
- expiry, revocation, release
- RiskEvidence 기반 Quarantine
- `prepare` / `commit` / `rollback` Adapter 계약
- optimistic version conflict 검증
- audit hash chain과 pseudonymous target reference
- 31개 기능 fixture
- 83개 Rust test
- 10,000건 property-based amount-conservation test
- ARCH-001~015 architecture invariant
- SAFE-001~010 정형 속성 매핑
- B0~B6 baseline 평가 프레임워크
- smoke benchmark, attack simulation, privacy audit
- TLA+ 모델 및 설정 파일

현재 시스템은 구조와 구현 면에서는 논문 작성 가능한 수준에 도달했으나, 다음 사항은 SCI 논문 제출 근거로 아직 불충분하다.

1. TLC를 통한 실제 bounded model checking 미실행
2. full benchmark 미실행
3. 30 independent runs와 95% CI 미확보
4. attack simulation의 충분한 반복·통계 미확보
5. privacy/linkability 정량분석 부족
6. WSL2 smoke 결과만 존재
7. clean release artifact 및 paper tag 미확정
8. 외부 Adapter 또는 실제 policy-aware asset 검증 부재
9. AI Agent ready infrastructure의 두 번째 domain 재사용 증거 부족
10. 최종 Claim–Evidence Matrix가 smoke 결과 기준

본 작업은 위 결손을 해소하는 데 한정한다.

---

## 2. 최종 목표

본 후속 개발이 완료되면 다음 문장을 논문에서 근거를 갖고 사용할 수 있어야 한다.

> HETE provides a machine-verifiable, multi-authority policy-enforcement framework whose authorization, amount, expiry, revocation, replay-prevention, atomicity, auditability, and domain-separation properties were evaluated through executable tests, property-based testing, bounded model checking, baseline comparison, attack simulation, privacy-surface analysis, and an external policy-aware asset adapter.

단, 다음 주장은 계속 금지한다.

- universal account freezing
- consensus-level transaction ordering control
- complete front-running prevention
- complete anonymity
- GDPR compliance guarantee
- legal validity determination
- production readiness
- full W3C VC interoperability
- cross-chain asset recovery
- AI Agent autonomous legal authority

---

## 3. 작업 범위

### 포함 범위

- TLC 실제 실행 및 결과 증거화
- Rust property test 강화
- full-scale benchmark
- concurrency benchmark
- attack campaign
- privacy and linkability evaluation
- native Linux publication host 검증
- external policy-aware asset Adapter
- 선택적 second-domain reuse demonstration
- paper artifact freeze
- statistical processing
- figure/table generation
- Claim–Evidence Matrix 갱신
- 논문용 결과 요약 문서 작성

### 제외 범위

- 전체 법률서비스 제품화
- 외부 공공기관 시스템 연동
- 실제 법원·검찰·수사기관 credential 발급
- public blockchain universal freeze
- private relay 또는 encrypted mempool 구현
- threshold cryptography 신규 구현
- full JSON-LD 및 full W3C VC stack
- cross-chain seizure
- 실제 AI Agent autonomous execution
- 대규모 UI 개발

---

## 4. 개발 원칙

### 4.1 Evidence-first

모든 추가 기능은 논문 claim과 연결되어야 한다. 다음 산출물이 없는 기능은 구현 완료로 보지 않는다.

- test
- benchmark
- raw result
- manifest
- statistical summary
- claim mapping
- limitations update

### 4.2 Clean and reproducible publication host

최종 실험은 다음 조건에서 실행한다.

- native Ubuntu 24.04 LTS 또는 동급 native Linux
- WSL2 결과는 보조 결과로만 사용
- clean Git working tree
- fixed source commit
- `Cargo.lock` 고정
- release build
- CPU governor와 background load 기록
- CPU model, core count, RAM, storage, kernel version 기록
- experiment start/end timestamp 기록
- raw data directory 사전 정리
- 결과 파일 SHA-256 생성

### 4.3 No hidden failure

다음은 삭제하거나 평균에서 임의 제외하지 않는다.

- failed run
- timeout
- outlier
- adapter conflict
- audit failure
- model checker counterexample
- OS incident

제외가 필요한 경우 사전 정의된 기준을 사용하고, 원자료는 보존한다.

### 4.4 Claim restriction

실험에서 측정하지 않은 항목은 논문에서 주장하지 않는다.

- policy-processing time을 end-to-end enforcement latency라고 부르지 않는다.
- simulated sealed ingress를 confidential transaction system이라고 부르지 않는다.
- traceability check를 formal verification이라고 단정하지 않는다.
- privacy exposure count 0을 anonymity라고 부르지 않는다.

---

## 5. 작업 패키지 개요

| WP | 작업명 | 우선순위 |
|---|---|---:|
| WP10 | Publication Baseline Freeze | 필수 |
| WP11 | TLC Bounded Model Checking | 필수 |
| WP12 | Formal–Executable Conformance | 필수 |
| WP13 | Publication-Grade Full Benchmark | 필수 |
| WP14 | Concurrency and Failure Campaign | 필수 |
| WP15 | Security Attack Campaign | 필수 |
| WP16 | Privacy and Linkability Campaign | 필수 |
| WP17 | External Policy-Aware Asset Adapter | 강력 권장 |
| WP18 | Domain-Generalization Demonstration | 권장 |
| WP19 | Statistical Analysis and Figure Generation | 필수 |
| WP20 | Paper Artifact Freeze and Evidence Package | 필수 |
| WP21 | Final Claim Audit | 필수 |

---

# 6. WP10 — Publication Baseline Freeze

## 목적

최종 SCI 평가의 기준이 되는 source, build, environment, schema, fixture를 고정한다.

## 작업

1. 현재 repository의 모든 변경 사항 정리
2. 다음 command가 clean tree에서 통과하도록 조치

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
python evaluation/check_architecture.py
python evaluation/check_warrant_invariants.py
python evaluation/check_formal_model.py
python evaluation/experiments/functional_correctness.py
```

3. paper branch 또는 release branch 생성
4. release candidate tag 생성

```text
paper/electronic-warrant-sci
v0.2.0-paper-rc1
```

5. publication manifest 생성

```text
evaluation/results/manifests/publication_baseline.json
```

필수 필드:

```json
{
  "source_commit": "",
  "git_tag": "",
  "working_tree": "clean",
  "rustc": "",
  "cargo": "",
  "python": "",
  "os": "",
  "kernel": "",
  "cpu_model": "",
  "physical_cores": 0,
  "logical_cores": 0,
  "memory_gb": 0,
  "storage_type": "",
  "build_profile": "release",
  "cargo_lock_sha256": "",
  "schema_sha256": {},
  "fixture_sha256": {},
  "timezone": "",
  "experiment_operator": ""
}
```

## 완료 조건

- clean working tree
- 모든 test pass
- release build pass
- baseline manifest 생성
- source commit과 tag 고정
- fixture와 schema hash 고정

---

# 7. WP11 — TLC Bounded Model Checking

## 목적

현재 작성된 `ElectronicWarrant.tla`와 `ElectronicWarrant.cfg`를 실제 TLC로 실행하여 SAFE-001~010과 bounded liveness를 검증한다.

## 환경

- OpenJDK LTS
- TLC version 명시
- TLA+ tools jar 또는 공식 Toolbox CLI
- 실행 command script화

권장 경로:

```text
formal/scripts/run_tlc.sh
formal/results/tlc/
```

## 필수 실행

### A. Safety-only configuration

검증 대상:

- SAFE-001 UnauthorizedExecution
- SAFE-002 NoReplay
- SAFE-003 AmountBound
- SAFE-004 Conservation
- SAFE-005 NoPostExpiryExecution
- SAFE-006 RevocationSafety
- SAFE-007 DomainBinding
- SAFE-008 Atomicity
- SAFE-009 AuditCompleteness
- SAFE-010 DomainNeutralCore에 대응하는 정적·모델 속성

### B. Deadlock check

- deadlock 없음 또는 허용 terminal deadlock 명시
- terminal state를 deadlock으로 볼지 모델상 종료로 볼지 명확히 정의

### C. Bounded liveness

- LIVE-001 AuthorizedExecutionProgress
- LIVE-002 ExpirationProgress
- LIVE-003 QuarantineReview

공정성 가정:

```text
Weak fairness:
- scheduler tick
- adapter availability
- audit storage write
- review action
```

liveness가 state explosion을 일으키면 작은 domain으로 bounded 실행하고 제한을 명시한다.

## 결과 저장

```text
formal/results/tlc/<run_id>/
├── command.txt
├── java_version.txt
├── tlc_version.txt
├── model_sha256.txt
├── config_sha256.txt
├── stdout.log
├── stderr.log
├── summary.json
└── counterexample/
```

`summary.json` 필드:

```json
{
  "run_id": "",
  "property_set": [],
  "status": "passed|failed|inconclusive",
  "states_generated": 0,
  "distinct_states": 0,
  "state_depth": 0,
  "elapsed_seconds": 0,
  "workers": 0,
  "deadlock_found": false,
  "invariant_violations": [],
  "liveness_violations": [],
  "model_sha256": "",
  "config_sha256": ""
}
```

## 반례 처리

1. 반례 trace 보존
2. 모델 오류인지 구현 오류인지 분류
3. 관련 Rust test 추가
4. 코드 또는 모델 수정
5. 수정 전후 결과 비교 보고서 작성
6. 최종 Claim–Evidence Matrix에 반영

## 완료 조건

- SAFE 속성 TLC 실행 완료
- deadlock 결과 확보
- bounded liveness 결과 확보 또는 명확한 inconclusive 사유
- stdout 원문과 summary JSON 저장
- 논문에서 “bounded model checking”이라고 표현 가능한 증거 확보

---

# 8. WP12 — Formal–Executable Conformance

## 목적

TLA+ 모델과 Rust 구현의 상태 전이 및 속성 의미가 불일치하지 않도록 trace 수준에서 연결한다.

## 작업

1. Rust 상태를 정형 모델 상태로 변환하는 serializer 작성

```rust
pub struct FormalTraceState {
    pub warrant_state: String,
    pub nonce_used: bool,
    pub authorized: bool,
    pub reserved_amount: u128,
    pub executed_amount: u128,
    pub released_amount: u128,
    pub expired: bool,
    pub revoked: bool,
    pub adapter_committed: bool,
    pub audit_written: bool,
}
```

2. 주요 Rust test에서 transition trace JSONL 출력
3. TLA+ example trace와 Rust trace 비교
4. 상태명과 terminal outcome mapping 고정
5. reason code와 formal action mapping 문서화

권장 산출물:

```text
formal/properties/FORMAL_EXECUTABLE_MAPPING.md
formal/traces/rust/
formal/traces/tla/
evaluation/check_trace_conformance.py
```

## 최소 trace scenario

- 정상 활성화
- unauthorized reject
- replay reject
- expiry reject
- revocation 후 재활성화 차단
- partial execution
- full execution
- adapter commit failure
- risk quarantine
- duplicate execution
- concurrent stale snapshot

## 완료 조건

- trace mapping checker 통과
- 모델과 구현의 상태 이름·전이 의미 일치
- 불일치가 있으면 수정 기록

---

# 9. WP13 — Publication-Grade Full Benchmark

## 목적

B0~B6 baseline을 SCI 논문에 사용할 수 있는 반복·통계 수준으로 평가한다.

## Build 및 실행 조건

- `cargo build --release`
- debug build 결과 사용 금지
- warm-up 분리
- 주요 조건 30 independent runs
- 각 run 최소 1,000 measured operations
- seed 고정 및 기록
- run 순서 randomization
- CPU affinity 또는 scheduling condition 기록

## Baseline

| ID | 설명 |
|---|---|
| B0 | single administrator |
| B1 | N-of-M multisig |
| B2 | DID/credential validation only |
| B3 | HETE single authority |
| B4 | HETE multi-authority, RiskEvidence disabled |
| B5 | full HETE with RiskEvidence and audit |
| B6 | dry-run observer |

## 독립 변수

| 변수 | 값 |
|---|---|
| Policy 수 | 100, 1K, 10K, 100K |
| Active warrant 수 | 0, 10, 100, 1K, 10K |
| Authority 수 | 1, 2, 3, 5, 7 |
| Credential 수 | 1, 2, 3, 5, 10 |
| Credential 크기 | 1KB, 2KB, 4KB, 8KB, 16KB |
| RiskEvidence 수 | 0, 1, 4, 16, 64 |
| Audit mode | disabled, minimal, hash-chain |

## 측정 항목

```text
T_parse
T_canonicalize
T_policy_digest
T_identity_resolve
T_credential_verify
T_authorize
T_validate
T_risk
T_prepare
T_reconcile
T_commit
T_audit
T_total
```

추가:

- throughput
- CPU time
- peak RSS
- allocation count 가능 시
- audit bytes per action
- storage growth
- failure count
- quarantine count

## 결과 파일

```text
evaluation/results/raw/full_benchmark/
├── run_001.csv
├── ...
├── run_030.csv
└── benchmark_manifest.json
```

각 row 필수 필드:

```text
run_id
operation_id
baseline_id
policy_count
active_warrant_count
authority_count
credential_count
credential_bytes
risk_evidence_count
audit_mode
status
reason_code
t_parse_ns
t_canonicalize_ns
t_digest_ns
t_identity_ns
t_credential_ns
t_authorize_ns
t_validate_ns
t_risk_ns
t_prepare_ns
t_reconcile_ns
t_commit_ns
t_audit_ns
t_total_ns
rss_bytes
source_commit
host_id
```

## 완료 조건

- B0~B6 전체 실행
- 30 independent runs
- 조건별 1,000 operations 이상
- raw data와 manifest 보존
- 실패율 기록
- 처리 단계별 latency 확보

---

# 10. WP14 — Concurrency and Failure Campaign

## 목적

동시 요청과 Adapter 실패에서 amount·state·atomicity 불변식이 유지되는지 검증한다.

## 동시성 수준

```text
1
4
16
64
128
256
```

## 시나리오

1. 동일 대상에 복수 영장
2. 동일 영장 duplicate execute
3. transfer와 freeze 동시 실행
4. expiry와 execute boundary
5. revocation과 commit 경쟁
6. stale snapshot
7. audit write failure
8. prepare failure
9. commit failure
10. rollback failure simulation
11. high reservation contention
12. multiple asset contention

## Failure injection rate

```text
0%
0.1%
1%
5%
10%
```

## 측정

- successful commit rate
- stale conflict rate
- retry count
- invariant violation count
- partial publication count
- deadlock count
- timeout count
- throughput
- P50/P95/P99
- starvation indicator
- rollback success rate

## 필수 성공 기준

```text
partial state publication = 0
amount invariant violation = 0
terminal resurrection = 0
duplicate success = 0
unaudited terminal outcome = 0
```

---

# 11. WP15 — Security Attack Campaign

## 목적

위협 모델에 정의된 공격을 반복 실행하고 ASR/BRR을 정량화한다.

## 반복 기준

- 공격 조건별 최소 1,000 attempts
- 주요 공격 30 independent runs
- seed 기록
- attacker capability 고정
- expected non-guarantee와 defense failure 구분

## 공격군

### Credential 공격

- signature forgery
- missing required role
- wrong role
- duplicate role
- mutual-exclusion violation
- wrong approval order
- changed amount
- changed target
- changed expiry
- wrong key ID
- expired credential
- revoked key
- stale DID document

### Replay 공격

- same-domain replay
- cross-domain replay
- cross-adapter replay
- cross-resource replay
- nonce reuse
- policy digest reuse

### Policy 공격

- unknown critical field
- privilege expansion
- threshold downgrade
- duration expansion
- amount expansion
- action expansion
- schema downgrade
- digest mismatch

### State/Adapter 공격

- stale snapshot
- duplicate command
- commit race
- expiry race
- revocation race
- capability misdeclaration
- audit suppression
- partial commit attempt

### Public commitment 공격

- commit observation
- target hash enumeration
- high-priority ordering
- reveal delay
- commit censorship
- commit expiry
- metadata correlation

### AI Agent boundary 공격

- agent-generated over-amount policy
- unauthorized adapter invocation
- threshold bypass attempt
- quarantine release attempt
- expired delegation
- wrong policy type
- human-confirmation bypass

## 지표

```text
ASR = successful unauthorized or escape outcomes / total attempts
BRR = 1 - ASR
FRR = valid requests rejected / valid attempts
FAR = invalid requests accepted / invalid attempts
```

## 보고 규칙

- public commitment의 target secrecy 실패는 expected non-guarantee로 별도 표기
- consensus ordering 결과를 HETE core defense 실패로 왜곡하지 않음
- 단일 키 compromise는 authorization threshold 구성별로 분리
- 공격 성공 정의를 scenario별로 명시

## 완료 조건

- threat model의 모든 평가 가능 공격에 결과 존재
- ASR/BRR 표와 confidence interval 생성
- expected non-guarantee 별도 분류
- 공격 결과를 논문 limitation과 연결

---

# 12. WP16 — Privacy and Linkability Campaign

## 목적

현재의 privacy-oriented design이 직접 평문 노출을 줄이는 정도와 남아 있는 linkability를 정량화한다.

## 검사 표면

- input policy JSON
- credential envelope
- process log
- error output
- audit record
- adapter receipt
- database dump
- CSV
- JSONL
- telemetry
- temporary file
- network fixture
- external Adapter event
- crash report

## 실험

### A. Plaintext exposure audit

- 금지 subject DID
- 인명
- 사건번호
- raw credential
- salt
- case narrative

### B. Stable digest linkability

동일 subject에 대해:

- same salt
- rotated salt
- different resource
- different warrant
- different epoch
- low-entropy input
- high-entropy input

### C. Cross-run correlation

공격자가 다음 정보를 알고 있다고 가정한다.

- public asset ID
- warrant timestamp window
- candidate DID list
- target hash scheme 일부
- no salt
- leaked salt
- partial mapping

### D. Crypto-shredding simulation

- off-chain mapping 삭제 전 복구
- mapping 삭제 후 복구
- salt 삭제 후 복구
- backup 존재 시 복구

## 지표

- plaintext hit count
- artifact count scanned
- field count scanned
- unique pseudonymous ID count
- same-subject correlation rate
- cross-run linkability rate
- dictionary attack success rate
- salt rotation reduction ratio
- audit bytes per record
- retained sensitive field count

## 완료 조건

- plaintext exposure report
- linkability matrix
- re-identification assumptions
- crypto-shredding simulation 결과
- GDPR compliance 주장을 하지 않는 limitation 유지

---

# 13. WP17 — External Policy-Aware Asset Adapter

## 목적

in-memory simulated Adapter만 사용했다는 외적 타당성 한계를 보완한다.

## 권장 구현

다음 중 하나를 선택한다.

### Option A — CosmWasm policy-aware token Adapter

- 별도 crate 또는 별도 integration directory
- HETE core와 직접 결합 금지
- Adapter API bridge
- freeze reservation
- transfer check
- release
- expiry
- execution receipt
- event minimization

### Option B — Embedded transactional DB Adapter

- SQLite 또는 PostgreSQL
- real transaction boundary
- prepare/commit simulation
- optimistic version
- crash/failure recovery
- external process boundary

### Option C — Permissioned ledger Adapter

- 로컬 permissioned chain
- 최소 token/vault
- policy hook
- block inclusion과 finality 측정

가장 논문 친화적인 선택은 **Option A 또는 B**다.

## 필수 기능

- manifest
- authoritative balance
- prepare
- commit
- rollback
- amount-bounded reservation
- expiry
- revocation/release
- receipt
- failure injection
- adapter version

## 외부 Adapter 실험

- 정상 동결
- 부분 동결
- transfer rejection
- expiry release
- revocation
- duplicate command
- stale version
- commit failure
- restart recovery
- audit linkage

## 시간 분리

블록체인 Adapter인 경우:

```text
T_rpc_submit
T_mempool_accept
T_block_inclusion
T_execution
T_finality
```

DB Adapter인 경우:

```text
T_ipc
T_transaction_begin
T_prepare
T_commit
T_durable_flush
T_response
```

## 완료 조건

- simulated Adapter와 외부 Adapter 결과 비교
- core code 수정 최소화
- Adapter API 재사용 증명
- end-to-end overhead 확보
- external system atomicity 한계 문서화

---

# 14. WP18 — Domain-Generalization Demonstration

## 목적

HETE가 전자영장 전용 시스템이 아니라 AI Agent 기반 서비스의 공통 intra-structure로 재사용 가능하다는 증거를 제공한다.

## 권장 두 번째 domain

### Option A — High-Value Payment Approval

- requester
- risk reviewer
- approver
- amount limit
- expiry
- audit
- dry-run

### Option B — AI Agent Tool Delegation

- human delegator
- agent DID
- allowed tool
- amount/resource scope
- expiry
- human confirmation
- revocation

### Option C — Sensitive Data Access Release

- requester
- data owner
- compliance approver
- resource scope
- time window
- audit obligation

AI Agent 비전을 고려하면 **Option B**가 가장 적합하다.

## 구현 범위

완전한 서비스 구현은 필요 없다.

- 기존 `MachinePolicyObject` 재사용
- 기존 `hete-identity`, `hete-credential` 재사용
- 기존 Adapter API 재사용
- 신규 core 수정 금지
- domain-specific schema와 fixture만 추가
- 5~10개 기능 test
- one benchmark comparison

## 평가 질문

- core 수정 없이 새 domain을 추가할 수 있는가?
- policy object 필드는 재사용 가능한가?
- authority policy는 그대로 동작하는가?
- audit와 risk path를 재사용할 수 있는가?
- domain-specific code 비율은 어느 정도인가?

## 지표

- reused lines/modules
- modified core files
- new domain files
- implementation effort
- architecture invariant result
- policy-processing overhead

## 완료 조건

- core 수정 0 또는 최소
- ARCH invariant 유지
- 두 번째 domain fixture 통과
- 논문에서 “domain-neutral framework” 주장 보강

---

# 15. WP19 — 통계 분석 및 Figure/Table 생성

## 목적

원자료에서 논문 표와 그림을 자동 생성한다.

## 통계

모든 주요 결과에 다음을 계산한다.

- count
- mean
- median
- standard deviation
- P50
- P95
- P99
- 95% confidence interval
- minimum
- maximum
- failure rate

baseline 비교:

- absolute overhead
- relative overhead
- effect size
- confidence interval

필요 시:

- Mann–Whitney U 또는 적절한 비모수 검정
- Kruskal–Wallis
- bootstrap CI

검정 선택은 데이터 분포와 반복 구조를 보고 결정한다.

## Figure

1. Architecture overview
2. Trust boundary sequence
3. Warrant lifecycle
4. Baseline total latency
5. Stage-level latency decomposition
6. Throughput vs concurrency
7. Latency vs authority count
8. Latency vs credential size
9. Memory/storage growth
10. Attack ASR/BRR
11. Privacy/linkability matrix
12. Simulated vs external Adapter
13. Formal verification summary
14. Second-domain reuse comparison

## Table

1. Related work comparison
2. Threat assumptions and non-guarantees
3. Formal properties
4. Functional scenario result
5. Baseline statistical result
6. Attack result
7. Privacy result
8. Adapter capability comparison
9. Claim–Evidence Matrix
10. Limitations

## 자동 생성

```text
evaluation/analysis/
├── aggregate_results.py
├── statistical_tests.py
├── generate_figures.py
├── generate_tables.py
└── verify_raw_hashes.py
```

출력:

```text
evaluation/results/processed/
evaluation/results/figures/
evaluation/results/tables/
```

## 완료 조건

- 모든 figure/table 자동 생성
- raw data hash 검증
- 수작업 숫자 입력 없음
- 논문 결과와 processed file 일치

---

# 16. WP20 — Paper Artifact Freeze

## 목적

논문 제출 시 공개 가능한 재현성 패키지를 생성한다.

## 산출물 구조

```text
artifacts/paper-v1/
├── README.md
├── LICENSE
├── SOURCE_COMMIT.txt
├── ENVIRONMENT.md
├── REPRODUCE.md
├── Cargo.lock
├── requirements-lock.txt
├── manifests/
├── formal/
│   ├── model/
│   └── results/
├── raw-data/
├── processed-data/
├── figures/
├── tables/
├── scripts/
├── schemas/
├── fixtures/
└── checksums.sha256
```

## `REPRODUCE.md`

```bash
cargo build --release --workspace
cargo test --release --workspace --all-features
python evaluation/check_architecture.py
bash formal/scripts/run_tlc.sh
python evaluation/run_warrant_evaluation.py --full
python evaluation/experiments/concurrency_benchmark.py --runs 30
python evaluation/experiments/attack_simulation.py --runs 30 --attempts 1000
python evaluation/experiments/privacy_surface_audit.py --full
python evaluation/analysis/aggregate_results.py
python evaluation/analysis/generate_figures.py
python evaluation/analysis/generate_tables.py
```

## Artifact tag

```text
v0.2.0-paper-artifact
```

## 완료 조건

- clean tag
- checksums 생성
- full reproduction guide
- raw/processed/figure 일치
- secret, private key, local path 제거
- WSL path 의존 제거

---

# 17. WP21 — Final Claim Audit

## 목적

논문의 모든 주요 주장이 실제 근거와 일치하는지 최종 검토한다.

## Claim 상태

```text
SUPPORTED
PARTIALLY_SUPPORTED
NOT_SUPPORTED
FUTURE_WORK
NON_GUARANTEE
```

## 최종 Claim–Evidence Matrix

| Claim ID | Claim | Scope | Implementation | Test | Formal | Experiment | Result | Limitation | Status |
|---|---|---|---|---|---|---|---|---|---|

## 삭제 또는 완화 대상 표현

근거가 없으면 다음을 삭제한다.

- real-time
- complete prevention
- fully secure
- privacy-preserving
- GDPR compliant
- scalable
- production-ready
- universally applicable
- formally proven
- fault tolerant

대체 표현:

| 과도한 표현 | 권장 표현 |
|---|---|
| formally proven | checked by bounded model exploration under stated bounds |
| scalable | stable over the evaluated scale range |
| privacy-preserving | reduces direct plaintext disclosure |
| front-running resistant | prevents post-activation transfers for HETE-aware assets under the evaluated model |
| production-ready | feasible in the evaluated reference environment |
| universal | domain-neutral at the policy and adapter interface level |

## 완료 조건

- Abstract의 모든 정량 수치가 result file과 연결
- Conclusion이 실제 결과 범위를 넘지 않음
- Limitation과 non-guarantee 반영
- 결과 없는 claim 없음

---

# 18. 추가 Architecture Invariant

기존 ARCH-001~015에 다음을 추가한다.

```text
ARCH-016: publication experiment는 clean Git tree에서만 실행 가능하다.
ARCH-017: benchmark result에는 source commit과 host manifest가 반드시 포함된다.
ARCH-018: raw data 파일은 processing script에 의해 수정되지 않는다.
ARCH-019: external Adapter는 hete-adapter-api에만 의존한다.
ARCH-020: second domain 추가 시 poa-core 수정이 없어야 한다.
ARCH-021: debug build 결과를 publication table에 사용하지 않는다.
ARCH-022: TLC 결과 없이 formal proof claim을 생성하지 않는다.
ARCH-023: WSL2 결과는 native-host 결과와 명시적으로 구분한다.
ARCH-024: result figure는 raw data와 hash-linked되어야 한다.
ARCH-025: secret, private key, absolute local path가 artifact에 포함되지 않는다.
```

---

# 19. CI/CD 추가 요구사항

## Pull Request CI

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
python evaluation/check_architecture.py
python evaluation/check_warrant_invariants.py
python evaluation/check_formal_model.py
python evaluation/check_trace_conformance.py
python evaluation/experiments/functional_correctness.py --smoke
```

## Main/Release CI

- TLC safety run
- reduced liveness run
- release build
- benchmark smoke
- artifact secret scan
- checksum verification

## Manual Publication Job

- full benchmark
- 30-run concurrency
- 30-run attack campaign
- full privacy campaign
- external Adapter benchmark
- figure/table generation
- artifact package

---

# 20. 완료 기준

본 후속 작업은 다음 조건을 모두 만족할 때 완료된 것으로 본다.

## Formal

- TLC 실제 실행 완료
- SAFE-001~010 결과 확보
- bounded liveness 결과 확보 또는 명확한 제한
- counterexample 처리 기록
- formal–Rust trace mapping 완료

## Experiment

- B0~B6 full benchmark
- 30 independent runs
- 조건별 1,000 operations 이상
- concurrency campaign
- failure injection
- attack campaign
- privacy/linkability campaign
- native publication host 실행

## External validity

- 외부 Adapter 최소 하나
- simulated Adapter와 비교
- end-to-end component timing
- Adapter API 재사용 증명

## Generality

다음 중 하나:

- second domain demonstration 완료
- 또는 논문에서 AI Agent/general service 확장 주장을 future work로 제한

## Reproducibility

- clean tag
- paper artifact
- raw data
- manifests
- checksums
- scripts
- figure/table regeneration
- secret scan

## Paper evidence

- 최종 Claim–Evidence Matrix
- Results Summary
- Formal Verification Report
- Attack Report
- Privacy Report
- External Adapter Report
- Limitations 업데이트

---

# 21. 우선순위 및 실행 순서

## Phase 1 — 필수 검증 기반

1. WP10 Publication Baseline Freeze
2. WP11 TLC Bounded Model Checking
3. WP12 Formal–Executable Conformance

## Phase 2 — 핵심 정량 실험

4. WP13 Full Benchmark
5. WP14 Concurrency and Failure
6. WP15 Security Attack Campaign
7. WP16 Privacy and Linkability

## Phase 3 — 외적 타당성

8. WP17 External Adapter
9. WP18 Domain-Generalization Demonstration

## Phase 4 — 논문 패키지

10. WP19 Statistical Analysis
11. WP20 Paper Artifact Freeze
12. WP21 Final Claim Audit

---

# 22. 최종 개발 Agent 지시

개발 Agent는 다음 규칙을 따른다.

1. 현재 repository를 실제 검사한 후 경로와 API를 확인한다.
2. 기존 구조와 충돌하는 제안은 core 중립성을 유지하는 방향으로 조정한다.
3. 각 WP 시작 전에 acceptance test를 정의한다.
4. 각 WP를 독립적인 commit 또는 명확한 work report 단위로 분리한다.
5. smoke 결과를 full 결과로 표시하지 않는다.
6. TLC가 실행되지 않았으면 formal verification 완료라고 기록하지 않는다.
7. failed run과 counterexample을 삭제하지 않는다.
8. 성능이 예상보다 낮아도 결과를 보존한다.
9. external Adapter가 core와 직접 결합되지 않도록 한다.
10. second domain 구현을 위해 `poa-core`를 수정하지 않는다.
11. 모든 raw data에 run ID와 source commit을 포함한다.
12. 논문 claim보다 구현·증거 범위를 우선한다.
13. AI Agent는 draft, validation, monitoring, dry-run 범위에서만 취급한다.
14. 실제 집행은 human-authorized credential 없이는 불가능해야 한다.
15. 최종 보고서에는 미해결 위험을 반드시 포함한다.

---

# 23. 필수 산출물 목록

```text
docs/work_reports/105_electronic_warrant_addional_dev/
├── WP10_PUBLICATION_BASELINE.md
├── WP11_TLC_MODEL_CHECK.md
├── WP12_FORMAL_EXECUTABLE_CONFORMANCE.md
├── WP13_FULL_BENCHMARK.md
├── WP14_CONCURRENCY_FAILURE.md
├── WP15_ATTACK_CAMPAIGN.md
├── WP16_PRIVACY_LINKABILITY.md
├── WP17_EXTERNAL_ADAPTER.md
├── WP18_DOMAIN_GENERALIZATION.md
├── WP19_STATISTICAL_ANALYSIS.md
├── WP20_PAPER_ARTIFACT.md
└── WP21_FINAL_CLAIM_AUDIT.md

docs/scientific_evidence/
├── FORMAL_VERIFICATION_REPORT.md
├── FULL_BENCHMARK_REPORT.md
├── CONCURRENCY_REPORT.md
├── ATTACK_EVALUATION_REPORT.md
├── PRIVACY_EVALUATION_REPORT.md
├── EXTERNAL_ADAPTER_REPORT.md
├── DOMAIN_GENERALIZATION_REPORT.md
├── RESULTS_SUMMARY_FINAL.md
├── LIMITATIONS_FINAL.md
└── CLAIM_EVIDENCE_MATRIX_FINAL.md
```

---

# 24. 논문 작성 가능 시점

다음 시점부터 Results와 Abstract의 정량 수치를 확정할 수 있다.

- WP11 완료
- WP13 완료
- WP15 완료
- WP16 완료
- WP19 완료

다음 시점부터 논문 제출본을 고정할 수 있다.

- WP20 완료
- WP21 완료

Architecture, Design, Threat Model, Implementation, Formal Properties 초안은 현재 자료로 먼저 작성할 수 있으나, 최종 Abstract와 Conclusion은 모든 실험 완료 후 확정한다.

---

# 25. 최종 기대 결과

본 작업이 완료되면 HETE 전자영장 논문은 다음 네 측면에서 SCI 심사에 대응할 수 있다.

1. **Architecture**
   - domain-neutral HETE core
   - configurable multi-authority policy
   - replaceable enforcement adapter
   - independent electronic-warrant profile

2. **Technical correctness**
   - amount, expiry, revocation, replay, domain binding
   - atomic prepare/commit
   - explicit non-guarantees
   - HETE-aware asset scope

3. **Security and formal evidence**
   - threat model
   - property-based test
   - TLC bounded checking
   - attack campaign
   - privacy/linkability analysis

4. **Experimental rigor**
   - B0~B6 baselines
   - full repeated runs
   - confidence intervals
   - native host
   - external Adapter
   - reproducible paper artifact

최종적으로 HETE는 법률서비스 완제품이 아니라, 다음을 실증하는 SCI reference framework로 정의한다.

> A high-assurance, machine-verifiable policy infrastructure for human-authorized, multi-authority, bounded, revocable, and auditable enforcement across HETE-aware services.
