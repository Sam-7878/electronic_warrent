# 전자영장청구 기능 고도화 및 확장 구현 계획

본 계획서는 기존 전자영장 기능에 안전장치 및 규제 기술(RegTech) 고도화 개념을 도입하기 위한 설계 및 구현 일정을 기술합니다.

## User Review Required

> [!IMPORTANT]
> - **가상 잔액 검증**: 스마트 컨트랙트 자체는 사용자 잔액을 보관하지 않으므로, 송금 검증 시 `TRANSFER_LOG`에 기록된 성공한 거래들의 합산 금액과 초기 가상 잔액(50,000)을 바탕으로 현재 가상 잔액을 계산하여 정밀 동결 한도를 검사합니다.
> - **검찰(Prosecutor) DID 설정**: 다중 기관 승인 검증을 위해 `did:yongin:prosecutor001`을 검찰의 고유 DID로 등록하고 사용합니다.

## Proposed Changes

### CosmWasm Contract Layer (burde/local_currency_dex/cosmwasm_adapter)

---

#### [MODIFY] [state.rs](file:///d:/_Work/goat_bank/burde/local_currency_dex/cosmwasm_adapter/src/state.rs)
- 기존 `FROZEN_WALLETS` 맵을 삭제하고, 신규 영장 정보 구조체인 `WarrantInfo`와 `WARRANT_REGISTRY` 맵을 정의합니다.
```rust
pub struct WarrantInfo {
    pub is_frozen: bool,
    pub frozen_amount: Uint128,
    pub expiration_time: u64,
}
```

#### [MODIFY] [msg.rs](file:///d:/_Work/goat_bank/burde/local_currency_dex/cosmwasm_adapter/src/msg.rs)
- `ExecuteMsg::ExecuteWarrant` 열거형 파라미터에 `frozen_amount: Uint128`, `expiration_time: u64`, `prosecutor_vp_signature: Binary`를 추가합니다.

#### [MODIFY] [contract.rs](file:///d:/_Work/goat_bank/burde/local_currency_dex/cosmwasm_adapter/src/contract.rs)
- `exec_register_agent`: 허용되는 역할(role) 목록에 `"prosecutor"`를 추가합니다.
- `exec_execute_warrant`:
  - 법원(Court)의 VC 검증을 수행합니다.
  - 경찰(Police)의 VP 서명과 함께 검찰(Prosecutor)의 VP 서명(`prosecutor_vp_signature`)을 검증합니다.
  - 검증 성공 시 `WARRANT_REGISTRY`에 DID별로 `WarrantInfo`를 저장합니다.
  - 영장 집행 직후 `wasm-warrant-interchain-signal` 이벤트를 발생(Emit)시킵니다.
- `exec_present_and_transfer`:
  - `WARRANT_REGISTRY`에서 유저 DID 조회 후, 만료 시점 초과 시 자동으로 송금을 승인합니다.
  - 유효기간 내에 있을 경우, `TRANSFER_LOG`에 기반한 가상 잔액에서 송금 요청액을 뺀 잔액이 `frozen_amount` 미만이 되는지 확인하여 초과 시 `DomainError("Transfer rejected: Frozen limit violated")`를 발생시킵니다.

### Script & Test Layer (burde/local_currency_dex/testnet_docker)

---

#### [MODIFY] [crypto_helper.py](file:///d:/_Work/goat_bank/burde/local_currency_dex/testnet_docker/crypto_helper.py)
- 검찰(Prosecutor)의 Ed25519 키쌍을 생성하는 로직을 추가합니다.
- 경찰과 검찰이 서명하는 VP 메시지 서명 데이터를 생성하여 쉘 변수(`PROSECUTOR_VP_SIGNATURE` 등)로 출력합니다.

#### [MODIFY] [run_warrant_simulation.sh](file:///d:/_Work/goat_bank/burde/local_currency_dex/testnet_docker/run_warrant_simulation.sh)
- 검찰 Agent 등록 단계를 추가합니다.
- 테스트 시나리오 1(정밀 부분 동결 및 한도 침범 거부), 테스트 시나리오 2(유효기간 만료 및 자동 해제), 테스트 시나리오 3(IBC 교차 사슬 전파 로그 검증)을 포함하도록 확장합니다.

## Verification Plan

### Automated Tests
- `run_warrant_simulation.sh` 스크립트를 실행하여 3가지 시나리오가 모두 정상 작동하는지 확인합니다.
- 실행 로그에서 `wasm-warrant-interchain-signal` 이벤트와 타겟 DID가 출력되었는지 확인합니다.
