# 현행 전자영장청구 시스템 아키텍처 분석 및 독립 리비전(`hete_sandbox`) 설계 보고서

**문서 번호:** HETE-ARCH-04  
**작성일:** 2026-07-22  
**작성 목적:** 기존 전자영장청구 시스템의 소스 구조, 계약 간 의존성, 데이터 흐름을 분석하고, SCI 저널용 논문 업그레이드를 위해 기존 Voting System과의 종속성을 해제(Decoupling)하여 `hete_sandbox` 독립 프로토콜로 리비전(Revision)하기 위한 아키텍처 명세서 작성  

---

## 1. 개요 (Overview)

### 1.1 배경 및 목적
현행 **HETE(Heterogeneous Entity Trust Engine)** 프로젝트의 전자영장청구 시스템은 사기/부정 탐지 시스템(VDS) 및 사법 기관(KICS)과 연동하여 온체인 자산 및 유권자 권한을 영장 결과에 따라 강제 집행(몰수/동결/원상복구)하는 규제 기술(RegTech) 온체인 백엔드로 구현되었습니다.

그러나 현재 전자영장 관련 모듈(`hete-warrant-handler`, `hete-judicial-verifier`)은 **투표 시스템(`voting_contracts`) 및 널리파이어(Nullifier) 소거 파이프라인에 강하게 결합(Tightly Coupled)**되어 있습니다.

SCI 저널 수준의 학술적/기술적 우수성을 확보하고, 금융/DeFi/CBDC/ZTA 등 범용 블록체인 규제 영장 체계로 논문을 업그레이드하기 위해서는 현행 아키텍처의 소스 구조와 종속성을 명확히 정의하고, 신규 리포지토리인 **`hete_sandbox`**에서 투표 시스템에 의존하지 않는 **독자적(Independent & Standalone) 전자영장 프로토콜 엔진**으로 분리·재설계해야 합니다.

### 1.2 주요 분석 내용
1. **현행 소스 코드 및 파일 구조 분석** (`hete/voting_contracts`, `burde/local_currency_dex/cosmwasm_adapter`)
2. **전자영장 처리 파이프라인 및 데이터 흐름 분석** (Multi-Agency 서명 검증, 이중 장부 정산, GDPR 프라이버시 해시, 프론트러닝 방어)
3. **Voting System 종속성(Coupling) 상세 분석** (Nullifier, Survey ID, Encrypted Vote Purging, Reward Escrow 종속 구조)
4. **`hete_sandbox` 독자적 전자영장 프로토콜 전환(Decoupling) 전략 명세**

---

## 2. 현행 소스 코드 및 모듈 인벤토리 (Source Code Inventory)

기존 전자영장 관련 모듈은 `hete` 모놀리식 리포지토리의 투표 계약 모듈 및 `burde`의 지역화폐 DEX 어댑터에 분산되어 존재합니다.

```
[goat_bank Workspace]
├── hete/
│   ├── docs/
│   │   ├── architecture/
│   │   │   ├── 01_risk_evidence_analysis.md
│   │   │   ├── 02_hete_source_analysis.md
│   │   │   ├── 03_hete_system_architecture.md
│   │   │   └── 04_electronic_warrant_architecture.md  <-- [ 본 문서 ]
│   │   └── work_reports/
│   │       ├── 110-electronic_warrant_did_integration/
│   │       ├── 121-electronic_warrant_integration_update/
│   │       ├── 122-electronic_warrant_benchmark/
│   │       └── 201-electronic_warrant/
│   ├── voting_contracts/
│   │   ├── contracts/
│   │   │   ├── hete-warrant-handler/          <-- [ 영장 정산 오케스트레이터 ]
│   │   │   │   ├── src/ (contract.rs, msg.rs, state.rs, error.rs)
│   │   │   │   └── Cargo.toml
│   │   │   ├── hete-judicial-verifier/        <-- [ 사법 기관 서명/ZKP 검증기 ]
│   │   │   │   ├── src/ (contract.rs, msg.rs, state.rs, error.rs)
│   │   │   │   └── Cargo.toml
│   │   │   ├── hete-voter-manager/            <-- [ 장부 A: 유권자/널리파이어 상태 관리 ]
│   │   │   ├── hete-tally-accumulator/        <-- [ 장부 B: 동형암호 투표 집계/퍼지 ]
│   │   │   └── hete-reward-escrow/            <-- [ 장부 C: 참여 보상 에스크로/몰수 ]
│   │   └── packages/
│   │       └── voting-common/                 <-- [ 공통 타입 팩 (WarrantIssuedVP 등) ]
│   │           └── src/ (types.rs, lib.rs)
│   └── tests/
│       ├── simulation_agents.py
│       └── simulation_did_agents.py
│
└── burde/
    ├── local_currency_dex/
    │   ├── cosmwasm_adapter/                  <-- [ DEX 영장집행/동결 어댑터 ]
    │   │   └── src/ (contract.rs, msg.rs, state.rs)
    │   └── testnet_docker/
    │       ├── crypto_helper.py               <-- [ ED25519 다중 서명 & SHA-256 해시 헬퍼 ]
    │       ├── run_warrant_simulation.sh      <-- [ 다중 서명/동결/만료 시뮬레이션 ]
    │       └── run_scientific_evaluation.sh  <-- [ SCI 저널 벤치마크 평가 스크립트 ]
```

### 2.1 주요 온체인 스마트 컨트랙트 명세
| 컨트랙트명 | 위치 | 역할 및 주요 엔트리포인트 | 종속성 수준 |
| :--- | :--- | :--- | :--- |
| **`hete-warrant-handler`** | `hete/voting_contracts/contracts/` | 사법 게이트웨이로부터 `SubmitWarrantResult` 수신, `judicial-verifier` 서명 검증 위임, 검증 결과(Reply)에 따라 장부 A/B/C에 아토믹 정산 SubMessage 전달 | **High** (`voting-common`, `voter-manager`, `tally`, `escrow`) |
| **`hete-judicial-verifier`** | `hete/voting_contracts/contracts/` | 법원/검찰/경찰 사법 기관 공개키 PKI 등록, ZKP 검증 스키마 확인, `WarrantIssuedVP` ED25519 디지털 서명 무결성 판정 | **Medium** (`WarrantIssuedVP` 내 `survey_id`, `nullifier_root`) |
| **`hete-voter-manager`** | `hete/voting_contracts/contracts/` | 장부 A: VDS 감지 사기 널리파이어 `Suspended` 전환 및 영장 승인 시 `FinalizeLegalForfeiture` (`PermanentBlacklist`) 확정 | **Critical** (Voting Domain Core) |
| **`hete-tally-accumulator`**| `hete/voting_contracts/contracts/` | 장부 B: 영장 승인 시 `PurgeSuspendedVotes` 실행 (격리된 동형암호 표값 원장 파기) | **Critical** (Voting Domain Core) |
| **`hete-reward-escrow`** | `hete/voting_contracts/contracts/` | 장부 C: 영장 승인 시 `ForfeitToTreasury` 실행 (에스크로 환수 상태 보상금을 국고 계좌로 영구 귀속) | **Critical** (Voting Domain Core) |
| **`cosmwasm_adapter`** | `burde/local_currency_dex/` | 계좌/지갑 단위 영장 집행 (`ExecuteWarrant`), 프론트러닝 방어 (`CommitWarrant`), DID 해시 조회 (`ACTIVE_WARRANTS_BY_DID`) | **Low** (DEX/Wallet Domain) |

---

## 3. 현행 전자영장 시스템 아키텍처 및 데이터 흐름 (Current Architecture)

### 3.1 전체 시스템 토폴로지 (System Topology)

```mermaid
graph TB
    subgraph "Off-chain Regulatory Gateway"
        VDS["Fraud Detection System (VDS)"] -->|WriteOff Fraud Event| RELAYER["Off-chain Event Listener"]
        RELAYER -->|Wrap Evidence| KICS["사법 기관 (Court / Prosecution KICS)"]
        KICS -->|Issue Warrant VP<br/>(Court VC + Police VP + Prosecutor VP)| GW["mTLS Gateway Authority"]
    end

    subgraph "On-chain Settlement Core (CosmWasm)"
        GW -->|SubmitWarrantResult| WH["hete-warrant-handler<br/>(Warrant Orchestrator)"]
        WH -->|VerifyJudicialWarrant<br/>SubMsg| JV["hete-judicial-verifier<br/>(Judicial PKI & Signature Verifier)"]
        JV -->|Reply Data<br/>(is_approved, is_valid)| WH

        WH -->|Scenario A: Approved<br/>SubMsg 1: FinalizeLegalForfeiture| VM["hete-voter-manager<br/>(Ledger A: Blacklist)"]
        WH -->|Scenario A: Approved<br/>SubMsg 2: PurgeSuspendedVotes| TA["hete-tally-accumulator<br/>(Ledger B: Void Votes)"]
        WH -->|Scenario A: Approved<br/>SubMsg 3: ForfeitToTreasury| RE["hete-reward-escrow<br/>(Ledger C: Treasury)"]

        WH -->|Scenario B: Rejected<br/>SubMsg: RollbackToActive| VM
    end

    subgraph "Local Currency DEX Extension (burde)"
        WH -.->|Cross-chain Signal Event| DEX["cosmwasm_adapter<br/>(Account Freeze & Seizure)"]
    end
```

### 3.2 핵심 기능 및 제어 파이프라인

#### (1) 다중 기관 사법 승인 체계 (Multi-Agency Approval)
* 영장 집행의 오남용을 방지하기 위해 **법원(Court)**의 영장 발부 VC, **경찰(Police)**의 집행 요청 VP, **검찰(Prosecutor)**의 집행 승인 VP 서명을 모두 검증하는 3중 다중 서명(Multi-Agency Signature) 구조를 가집니다.
* `judicial-verifier`는 등록된 사법 기관 공개키(`judicial_public_keys`)를 대조하여 서명의 수학적 무결성을 검증합니다.

#### (2) 복식 부기 원장 3단계 아토믹 정산 (Triple-Ledger Atomic Settlement)
* **Preemptive Quarantine (사전 격리):** 의심 널리파이어 발생 시 장부 A(`voter-manager`)는 `Suspended`, 장부 B(`tally`)는 표값 임시 소거, 장부 C(`escrow`)는 보상금 임시 환수를 수행합니다.
* **Legal Finalization (사법 마감):** 
  * `warrant-handler`가 사법 검증 승인을 수신하면 단일 트랜잭션 수신 내에서 SubMessage 3개를 동시 발송(`vm_msg`, `tally_msg`, `escrow_msg`)하여 복식 부기 정산 오차 0.00%의 아토믹(Atomic) 마감을 보장합니다.

#### (3) 온체인 프라이버시 및 프론트러닝 방어 (GDPR & Anti-Front-Running)
* **GDPR compliance 해시 매핑:** `burde/cosmwasm_adapter`는 target_did 텍스트 대신 `SHA-256(target_did || warrant_id || salt)` 해시값을 스토리지 키로 활용하여 블록체인 상에 혐의자 식별 정보가 노출되지 않도록 은닉합니다.
* **Pre-Commitment Lock:** 영장 집행 직전 `CommitWarrant` 단계를 통해 `target_did_hash`와 `warrant_hash`의 블록 높이를 온체인에 우선 커밋하여, 혐의자가 영장 집행 사실을 눈치채고 자산을 빼돌리는 프론트러닝(Front-Running) 송금을 차단합니다.

---

## 4. 투표 시스템 종속성 분석 (Voting System Coupling Analysis)

현행 전자영장 시스템 모듈은 **투표 시스템 도메인 논리 및 데이터 구조에 완전히 엮여 있으며**, 독립적인 보안 규제 모듈로 작동하기에 다음과 같은 구조적 한계가 존재합니다.

### 4.1 데이터 구조 및 인터페이스의 도메인 종속성
1. **`WarrantIssuedVP` 구조체 종속성 (`voting-common/src/types.rs`)**:
   ```rust
   pub struct WarrantIssuedVP {
       pub survey_id: String,                  // [종속] 투표/여론조사 세션 ID
       pub nullifier_merkle_root: String,       // [종속] 투표 무효화 키 머클루트
       pub judicial_signature: String,
       pub is_approved: bool,
       pub reason: String,
       pub tsa_timestamp_token: String,
       pub kics_receipt_number: u64,
       pub tally_contract_addr: String,        // [종속] 투표 집계 컨트랙트 주소
       pub escrow_contract_addr: String,       // [종속] 투표 보상 에스크로 주소
   }
   ```
   * 영장 증명서 객체 내부에 `survey_id`, `nullifier_merkle_root`, `tally_contract_addr` 등 투표 관련 필드가 하드코딩되어 있어 일반 계좌 동결이나 DeFi/자산 몰수 프로토콜로 재사용이 불가능합니다.

2. **상태 머신 및 인터페이스 메시지 종속성**:
   * `FinalizeLegalForfeitureMsg` (유권자 블랙리스트)
   * `PurgeSuspendedVotesMsg` (격리 투표 파기)
   * `ForfeitToTreasuryMsg` (투표 참가 리워드 국고 귀속)
   * 전자영장의 실행 대상이 "계좌 잔액/토큰 자산/스마트 컨트랙트 권한"이 아닌 "유권자 널리파이어 및 투표 표값"에 국한되어 있습니다.

### 4.2 아키텍처 및 정산 파이프라인의 강결합 문제
* `hete-warrant-handler` 내부의 `handle_warrant_approved` 로직이 `VOTER_MANAGER_CONTRACT`, `TALLY_CONTRACT`, `ESCROW_CONTRACT`의 3개 특정 투표 컨트랙트 주소를 인스턴스화 시점에 필수적으로 요구합니다.
* 이에 따라 투표 시스템이 구축되지 않은 환경에서는 전자영장 핸들러 단독으로 배치(Deploy)되거나 테스트될 수 없습니다.

### 4.3 SCI 저널 논문 관점에서의 한계점
* SCI급 논문(e.g., IEEE TIFS, IEEE TDSC)으로 논문을 구성하기 위해서는 **"Generalized Blockchain Electronic Warrant & Regulatory Enforcement Protocol"**이라는 범용 프레임워크를 제시해야 합니다.
* 현행 코드는 "특정 널리파이어 투표 소거용 부속 모듈"로 인식될 위험이 크며, 범용 블록체인 규제 기술(RegTech) 연구로서의 가치를 독자적으로 인정받기 어렵습니다.

---

## 5. `hete_sandbox` 독립 리비전 설계 전략 (Decoupling Architecture)

새로 구성할 **`hete_sandbox`** 리포지토리에서는 기존 투표 시스템과의 결합을 완벽히 제거하고, **독립적인 범용 블록체인 전자영장 실행 엔진(Standalone RegTech Core)**으로 재설계합니다.

```
[기존: Coupled Architecture]
Warrant Handler ──(하드코딩 SubMsg)──> Voter Manager / Tally Accumulator / Reward Escrow (Voting Only)

[신규: Decoupled Architecture in hete_sandbox]
Warrant Engine (Standalone Core)
       ├── Judicial Verifier (Multi-Agency PKI)
       ├── State Machine (Pending -> PreCommit -> Executed -> Forfeited/Released)
       └── Asset Adapter Interface (Generic Plugin Interface)
               ├── Bank Wallet Adapter (Freeze / Seizure / Unfreeze)
               ├── Token/Vault Adapter (Balance Lock / Treasury Forfeiture)
               └── Zero-Trust Identity Adapter (DID Quarantining)
```

### 5.1 핵심 독립화 방향 (Key Decoupling Features)

1. **대상 식별자 추상화 (Target Identifier Abstraction)**:
   * `nullifier_merkle_root` 및 `survey_id` 대신 범용 대상 식별자 `target_resource_hash` (`SHA-256(Target_DID || Resource_ID || Salt)`) 도입.
   * 유권자가 아닌 계좌(Account), 지갑(Wallet), 스마트 컨트랙트 금고(Vault), DID 자격증명 등에 공통 적용 가능하도록 설계.

2. **범용 영장 상태 머신 (Universal Warrant Lifecycle State Machine)**:
   ```
   [Requested] ──> [Pre-Commitment Lock] ──> [Judicial Verifying] ──> [Executed (Approved)] ──> [Asset Forfeited]
                                                                  └──> [Executed (Rejected)] ──> [Asset Released]
   ```

3. **플러그인형 자산 어댑터 인터페이스 (`AssetAdapter` Trait)**:
   * `warrant-handler`가 특정 투표 컨트랙트를 직접 호출하는 대신, 규격화된 범용 훅(Plugin Hook) 인터페이스를 통해 임의의 자산/토큰 컨트랙트로 이벤트를 라우팅합니다.
   * `freeze_asset(target, amount, ttl)`
   * `forfeit_asset(target, treasury_addr)`
   * `release_asset(target)`

4. **SCI 저널용 평가 텔레메트리 모듈 내장**:
   * 프라이버시 보호 검증 (GDPR Plaintext Audit)
   * 프론트러닝 차단 성공률 (Pre-Commitment Locking Rate)
   * 다중 서명 검증 및 자산 몰수 라텐시/가스비 소모량 (Gas & Latency Benchmarks)

---

## 6. 결론 및 향후 추진 일정 (Conclusion & Roadmap)

본 문서(`04_electronic_warrant_architecture.md`)를 통해 현행 전자영장청구 시스템의 소스 구조, 동작 아키텍처, 그리고 기존 Voting System에 대한 구조적 종속성을 정밀하게 분석하였습니다.

### 향후 추진 단계:
1. **`hete_sandbox` 독립 리포지토리 모듈 구조 설계**:
   * `warrant-core` (독립 영장 핸들러 및 상태 머신)
   * `judicial-verifier` (독립 사법 기관 PKI 및 ZKP 검증기)
   * `asset-adapter-interface` (범용 규제 집행 훅 인터페이스)
2. **소스 코드 분리 및 리팩토링 구현**:
   * `voting_contracts`에서 투표 관련 로직을 제거하고 범용 데이터 구조로 재설계.
3. **SCI 저널 벤치마크 및 논문 텍스트 동기화**:
   * `run_scientific_evaluation.sh` 벤치마크 결과를 바탕으로 논문 아키텍처 다이어그램 및 실험 데이터 갱신.

---
**문서 저장 위치:** `hete/docs/architecture/04_electronic_warrant_architecture.md`
