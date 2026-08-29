# 전자영장 청구 및 자동 이첩 프로토콜 구현 Task

## Phase 1: 공통 타입 확장
- [x] voting-common/types.rs - NullifierStatus::PermanentBlacklist, RewardStatus::Forfeited 추가
- [x] voting-common/types.rs - 전자영장 관련 타입 추가 (WarrantIssuedVP, JudicialConfig, WarrantStatus 등)
- [x] voting-common/types.rs - Reply ID 상수 추가

## Phase 2: 신규 컨트랙트 - hete-judicial-verifier
- [x] Cargo.toml 작성
- [x] src/lib.rs, src/error.rs, src/msg.rs, src/state.rs 작성
- [x] src/contract.rs 작성 (사법 서명 검증 로직)
- [x] Unit tests 작성

## Phase 3: 신규 컨트랙트 - hete-warrant-handler
- [x] Cargo.toml 작성
- [x] src/lib.rs, src/error.rs, src/msg.rs, src/state.rs 작성
- [x] src/contract.rs 작성 (영장 정산 오케스트레이터)
- [x] Unit tests 작성

## Phase 4: 기존 컨트랙트 수정
- [x] hete-voter-manager: msg.rs, state.rs, contract.rs 확장
- [x] hete-reward-escrow: msg.rs, contract.rs 확장
- [x] hete-tally-accumulator: msg.rs, contract.rs 확장

## Phase 5: Workspace 설정 & 빌드/테스트
- [x] Cargo.toml workspace members 추가
- [x] cargo build --workspace
- [x] cargo test --workspace
- [x] 문서 작성 (docs/work_reports/201-electronic_warrant/)
