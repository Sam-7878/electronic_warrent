# 전자영장 벤치마크 및 보안성 고도화 구현 계획

본 계획서는 SCI 저널 검증 수준의 보안성(온체인 프라이버시 보호, 프론트러닝 방어) 및 정량적 성능 평가 텔레메트리 구현을 목적으로 합니다.

## User Review Required

> [!IMPORTANT]
> - **프라이버시 보호를 위한 해시화**: 온체인 스토리지에 사용자 DID를 평문으로 저장하지 않고, `SHA-256(target_did || warrant_id || salt)` 형태의 해시값을 조회 키로 사용하여 GDPR 규제를 준수합니다.
> - **프론트러닝 방어 (Commitment)**: 경찰이 영장 집행 1~2블록 전에 `CommitWarrant`를 통해 `target_did` 해시와 `warrant_hash`를 먼저 제출하고, 해당 블록부터 영장 집행 완료 전까지 해당 지갑의 모든 거래를 잠정 거절합니다.
> - **가스/성능 텔레메트리**: 영장 검증 및 실행의 정량적 평가를 위해 `WarrantTelemetry` 이벤트를 발생시키고 가스 사용량 등의 지표를 추출합니다.

## Proposed Changes

### CosmWasm Contract Layer (burde/local_currency_dex/cosmwasm_adapter)

---

#### [MODIFY] [state.rs](file:///d:/_Work/goat_bank/burde/local_currency_dex/cosmwasm_adapter/src/state.rs)
- `WARRANT_REGISTRY` 맵의 키 타입을 `&str`에서 `&[u8]` (SHA-256 해시)로 변경합니다.
- 영장 커밋 정보를 저장할 `WARRANT_COMMITMENTS`와 `PENDING_COMMITMENTS` 맵을 신설합니다.
```rust
pub const WARRANT_REGISTRY: Map<&[u8], WarrantInfo> = Map::new("warrant_registry");
pub const WARRANT_COMMITMENTS: Map<&[u8], u64> = Map::new("warrant_commitments");
pub const PENDING_COMMITMENTS: Map<&[u8], Vec<Binary>> = Map::new("pending_commitments"); // key: target_did_hash
```

#### [MODIFY] [msg.rs](file:///d:/_Work/goat_bank/burde/local_currency_dex/cosmwasm_adapter/src/msg.rs)
- `ExecuteMsg`에 `CommitWarrant` 실행 메시지를 추가합니다.
```rust
    CommitWarrant {
        target_did_hash: Binary,
        warrant_hash: Binary,
    },
```
- `ExecuteWarrant` 실행 메시지에 `warrant_id: String`, `salt: String` 파라미터를 추가합니다.
- `PresentAndTransfer` 실행 메시지에 `warrant_id: Option<String>`, `salt: Option<String>` 파라미터를 추가합니다 (사용자가 자신의 영장 한도 검증을 위해 매핑 키를 복구할 수 있도록 지원).

#### [MODIFY] [contract.rs](file:///d:/_Work/goat_bank/burde/local_currency_dex/cosmwasm_adapter/src/contract.rs)
- `exec_commit_warrant` 함수를 구현하여 영장 커밋을 처리하고 블록 높이를 기록합니다.
- `exec_execute_warrant`:
  - `target_did`, `warrant_id`, `salt`를 결합하여 `warrant_hash` (SHA-256)를 생성합니다.
  - 해당 영장이 집행되면 `PENDING_COMMITMENTS` 및 `WARRANT_COMMITMENTS`에서 관련 항목을 삭제합니다.
  - `WarrantTelemetry` 이벤트를 발생시키고 서명 검증 가스 및 스토리지 작성 가스를 속성으로 담습니다.
- `exec_present_and_transfer`:
  - 유저의 `holder_did`에 대한 해시를 생성하여 `PENDING_COMMITMENTS`에 보류 중인 커밋이 있는지 확인하고, 있을 경우 `DomainError("Transaction locked due to pending regulatory commitment")`를 반환합니다.
  - 유효기간 내에 있고 `warrant_id`와 `salt`가 제공된 경우 해시 키를 생성해 `WARRANT_REGISTRY`에서 부분 동결 한도를 검사합니다.

### Script & Evaluation Layer (burde/local_currency_dex/testnet_docker)

---

#### [MODIFY] [crypto_helper.py](file:///d:/_Work/goat_bank/burde/local_currency_dex/testnet_docker/crypto_helper.py)
- `target_did`, `warrant_id`, `salt`를 사용하여 SHA-256 해시를 생성하고 쉘 변수로 출력하는 기능을 추가합니다.

#### [NEW] [run_scientific_evaluation.sh](file:///d:/_Work/goat_bank/hete/docs/work_reports/122-electronic_warrant_benchmark/run_scientific_evaluation.sh) (사용자 가이드라인 준수)
- 시나리오 1: 온체인 프라이버시 검증 (Plaintext DID 노출 여부 스토리지 덤프 감사)
- 시나리오 2: 프론트러닝 공격 차단 검증 (CommitWarrant 실행 후 거래 전송 시 잠금 거부 확인)
- 시나리오 3: 10회 이상 영장 집행을 반복하며 가스 사용량 등의 지표를 수집하여 `scientific_metrics.csv`를 자동 생성합니다.

## Verification Plan

### Automated Tests
- `run_scientific_evaluation.sh`를 실행하여 3가지 시나리오의 작동을 검증하고 `scientific_metrics.csv`가 정상적으로 기록되는지 확인합니다.
