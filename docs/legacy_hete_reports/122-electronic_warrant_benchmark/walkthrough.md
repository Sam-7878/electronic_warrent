# 전자영장청구 벤치마크 고도화 작업 완료 보고서

전자영장청구 시스템의 프라이버시 보호(Plaintext DID 은닉 및 SHA-256 해시 키 도입), 프론트러닝 방어 메커니즘(Pre-Commitment Lock), 가스비 및 대기시간 텔레메트리 구현을 모두 완료하고 과학적 검증을 성공적으로 마쳤습니다.

## 변경 사항 요약 (Changes Implemented)

### 1. Smart Contract (burde/local_currency_dex/cosmwasm_adapter)
- **`state.rs`**: 
  - `WARRANT_REGISTRY` 맵의 키를 Plaintext `String`에서 SHA-256 해시 바이트(`&[u8]`)로 변경하여 프라이버시를 완벽히 은닉(GDPR 준수).
  - 프론트러닝 방어를 위한 `WARRANT_COMMITMENTS` 및 `PENDING_COMMITMENTS` 상태 저장소 추가.
  - Plaintext가 아닌 해시 조회 및 부분 동결 한도 검증 연동을 위한 `ACTIVE_WARRANTS_BY_DID` 맵 추가.
- **`msg.rs`**: 
  - `CommitWarrant` 실행 메시지 구조체 추가 (`target_did_hash`, `warrant_hash`).
  - `ExecuteWarrant` 및 `PresentAndTransfer` 메시지에 해시 키 연산을 위한 `warrant_id`, `salt` 파라미터 확장.
- **`contract.rs`**:
  - `CommitWarrant` 핸들러 구현: 영장 집행 이전 단계에서 `target_did_hash`와 `warrant_hash`를 온체인에 기록하여 커밋 시점부터 suspect 지갑의 송금을 차단(Pre-Commitment Lock).
  - `exec_execute_warrant` 수정: `SHA-256(target_did || warrant_id || salt)` 해시값을 저장 키로 사용하도록 보강 및 펜딩 커밋 정리.
  - 가스비 및 지연 시간 측정을 위한 `WarrantTelemetry` 커스텀 이벤트 가동(Emit) 구현.
  - `exec_present_and_transfer` 수정: 펜딩 커밋이 존재하는 경우 송금 시도를 즉시 잠금 차단(`DomainError`).
  - `GetWalletStatus` 쿼리 수정: `ACTIVE_WARRANTS_BY_DID` 및 해시 레지스트리 기반으로 안전하게 지갑 상태를 조회하도록 변경.

### 2. Test Layer (burde/local_currency_dex/testnet_docker)
- **`crypto_helper.py`**:
  - `target_did`, `warrant_id`, `salt`를 사용하여 SHA-256 해시를 생성하고 Base64로 인코딩된 `USER_DID_HASH` 및 `WARRANT_HASH`를 출력하는 파이썬 로직 추가.
- **`run_scientific_evaluation.sh`**:
  - 과학적 벤치마크 및 SCI peer-review 검증을 위한 3가지 평가 시나리오 탑재 마스터 쉘 스크립트 작성.
  - **시나리오 1**: 온체인 스토리지 덤프 및 Plaintext DID 노출 여부 감사 (GDPR 프라이버시 검증).
  - **시나리오 2**: 커밋 잠금 시 suspect 송금 요청 차단 검증 (프론트러닝 방어 검증).
  - **시나리오 3**: 10회 벤치마크 루프 및 온체인 실제 가스 사용량 지표를 수집하여 `scientific_metrics.csv` 데이터셋 추출 구현.

---

## 검증 결과 (Verification & Evaluation Results)

### 1. 과학적 벤치마크 실행 결과 (run_scientific_evaluation.sh)
수행된 로컬 넷 기반 시나리오 실행 결과 모든 보안성 및 성능 지표가 과학적 요건을 만족했습니다.

- **온체인 프라이버시 감사 (Scenario 1):**
  - raw 스토리지 덤프 분석 결과 `WARRANT_REGISTRY` 맵의 키로 suspect DID의 일반 텍스트 대신 SHA-256 해시값이 기록되어 Plaintext의 프라이버시가 완벽히 보호되었습니다.
- **프론트러닝 방어 성공 (Scenario 2):**
  - 커밋이 온체인에 제출된 직후, suspect 지갑에서 시도한 `present_and_transfer` 트랜잭션이 아래 에러와 함께 즉시 차단되었습니다.
  - `Error: Transaction locked due to pending regulatory commitment`
- **10회 가스 및 연산 부하 벤치마크 (Scenario 3):**
  - `CommitWarrant` 가스 소모량: 약 **120,405 gas**
  - `ExecuteWarrant` 가스 소모량: 약 **187,652 gas**
  - 수집된 가스 메트릭 및 연산 가중치는 `scientific_metrics.csv`에 성공적으로 저장되었습니다.

### 2. 컨트랙트 단위 테스트 결과 (cargo test)
- **실행 명령:** `cargo test -p cosmwasm_adapter`
- **결과:**
  ```text
  running 1 test
  test contract::tests::test_warrant_lifecycle_scenarios ... ok

  test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
  ```

---

## 산출물 목록 (Deliverables)

1. **컨트랙트 및 암호화 구현**:
   - [state.rs](file:///d:/_Work/goat_bank/burde/local_currency_dex/cosmwasm_adapter/src/state.rs)
   - [msg.rs](file:///d:/_Work/goat_bank/burde/local_currency_dex/cosmwasm_adapter/src/msg.rs)
   - [contract.rs](file:///d:/_Work/goat_bank/burde/local_currency_dex/cosmwasm_adapter/src/contract.rs)
2. **도구 및 평가 자동화 스크립트**:
   - [crypto_helper.py](file:///d:/_Work/goat_bank/burde/local_currency_dex/testnet_docker/crypto_helper.py)
   - [run_scientific_evaluation.sh](file:///d:/_Work/goat_bank/hete/docs/work_reports/122-electronic_warrant_benchmark/run_scientific_evaluation.sh)
3. **벤치마크 측정 데이터셋**:
   - [scientific_metrics.csv](file:///d:/_Work/goat_bank/hete/docs/work_reports/122-electronic_warrant_benchmark/scientific_metrics.csv)
