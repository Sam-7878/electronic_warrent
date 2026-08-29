use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

// ============================================================================
// Nullifier 상태 관리 - 복식 부기 정산의 핵심 상태 머신
// ============================================================================

/// Voting-domain lifecycle for a one-time ballot nullifier.
///
/// This is intentionally separate from `hete_core::NullifierState`, which is
/// a generic ZTA trust/quarantine lifecycle and has different semantics.
#[cw_serde]
pub enum NullifierState {
    Active,
    Suspended,
    Voided,
    PermanentBlacklist,
}

/// 장부 A에 기록되는 Nullifier 레코드
#[cw_serde]
pub struct NullifierRecord {
    /// 해시 기반 일회성 무효화 식별자
    pub nullifier: String,
    /// 현재 상태
    pub status: NullifierState,
    /// 연계된 투표 세션 ID
    pub vote_session_id: String,
    /// 생성 블록 높이
    pub created_at_height: u64,
    /// 상태 변경 블록 높이
    pub updated_at_height: u64,
}

// ============================================================================
// 투표 세션 관리
// ============================================================================

/// 투표 세션 메타데이터
#[cw_serde]
pub struct VoteSession {
    /// 고유 투표 세션 식별자
    pub session_id: String,
    /// 투표 세션 제목/설명
    pub title: String,
    /// 투표 시작 블록 높이
    pub start_height: u64,
    /// 투표 종료 블록 높이
    pub end_height: u64,
    /// 후보자 목록 (인덱스로 참조)
    pub candidates: Vec<String>,
    /// 세션 상태
    pub is_active: bool,
}

// ============================================================================
// 동형암호 시뮬레이션 타입 (데모용)
// ============================================================================

/// 동형암호화된 투표값 (데모에서는 u128로 시뮬레이션)
/// 실제 프로덕션에서는 Paillier/BFV 등의 실제 동형암호 라이브러리 사용
#[cw_serde]
pub struct EncryptedVote {
    /// 암호화된 투표 데이터 (데모: base64 인코딩된 바이트)
    pub ciphertext: String,
    /// 후보자 인덱스 (평문 - 데모 검증용, 프로덕션에서는 제거)
    pub candidate_index: u32,
}

/// 집계 바스켓: 동형 합산 중간 결과
#[cw_serde]
pub struct TallyBucket {
    /// 후보자 인덱스
    pub candidate_index: u32,
    /// 암호화된 누적 합 (데모: 단순 카운트)
    pub encrypted_count: u64,
    /// 정상 투표 수
    pub valid_vote_count: u64,
    /// 소거된 투표 수
    pub voided_vote_count: u64,
}

// ============================================================================
// 리워드 에스크로 타입
// ============================================================================

/// 리워드 기록
#[cw_serde]
pub struct RewardRecord {
    /// 연계된 Nullifier
    pub nullifier: String,
    /// 리워드 금액 (최소 단위)
    pub amount: u128,
    /// 리워드 상태
    pub status: RewardStatus,
}

/// 리워드 상태
#[cw_serde]
pub enum RewardStatus {
    /// 에스크로에 락업된 상태
    Locked,
    /// 유권자에게 지급 완료
    Released,
    /// 사기 판정으로 몰수/환수됨
    Clawedback,
    /// 전자영장 발부 확인 후 국고 귀속 완료
    Forfeited,
}

// ============================================================================
// TEE 하드웨어 인증 관련 타입 (슬라이드 58 기반)
// ============================================================================

/// TEE 하드웨어 무결성 정책
#[cw_serde]
pub struct HardwarePolicy {
    /// Attestation 챌린지 타임아웃 (초)
    pub allowed_attestation_challenge_timeout_secs: u64,
    /// 강제 보안 수준: "STRONG_BOX" 또는 "TEE"
    pub mandated_security_level: String,
    /// 허용된 지갑 앱 패키지 토큰 명부
    pub verified_package_names: Vec<String>,
}

/// 유권자 검증 요청 데이터
#[cw_serde]
pub struct VoterVerificationData {
    /// 임시 서비스 키 (Key 3) - 공개키 부분
    pub service_key_public: String,
    /// TEE 하드웨어 Attestation 데이터 (Base64)
    pub tee_attestation: String,
    /// 말웨어 탐지 로그의 리프 해시 (Leaf Hash)
    pub malware_scan_leaf_hash: String,
    /// AnonCreds 영지식 증명 (Base64)
    pub anoncreds_proof: String,
}

// ============================================================================
// Cross-Contract 메시지 타입
// ============================================================================

/// 장부 B (tally-accumulator)에 전달되는 표값 소거 메시지
#[cw_serde]
pub struct VoidEncryptedVoteMsg {
    pub nullifier: String,
}

/// 리워드 에스크로에 전달되는 보상 환수 메시지
#[cw_serde]
pub struct ClawbackRewardMsg {
    pub nullifier: String,
}

/// Cross-contract query: Nullifier 상태 조회
#[cw_serde]
pub struct QueryNullifierStatusMsg {
    pub nullifier: String,
}

/// Nullifier 상태 조회 응답
#[cw_serde]
pub struct NullifierStatusResponse {
    pub nullifier: String,
    pub status: NullifierState,
}

// ============================================================================
// 정산(Reconciliation) 통계
// ============================================================================

/// 복식 부기 정산 결과 통계
#[cw_serde]
pub struct ReconciliationStats {
    /// 장부 A 발급 토큰 총수
    pub ledger_a_total_issued: u64,
    /// 장부 B 접수 투표 총수
    pub ledger_b_total_received: u64,
    /// 소거(Write-off)된 Nullifier 수
    pub total_voided: u64,
    /// 환수된 리워드 수
    pub total_clawedback: u64,
    /// 정산 불일치 여부 (0.00% 목표)
    pub reconciliation_error_rate: String,
}

// ============================================================================
// SubMsg Reply ID 상수
// ============================================================================

/// 장부 B (tally-accumulator) 표값 소거 Reply ID
pub const REPLY_TALLY_ID: u64 = 1;
/// 리워드 에스크로 보상 환수 Reply ID
pub const REPLY_REWARD_ID: u64 = 2;
/// 사법 기관 서명 검증 Reply ID (warrant-handler용)
pub const REPLY_JUDICIAL_VERIFY_ID: u64 = 10;
/// 영장 승인 후 tally 퍼지 Reply ID
pub const REPLY_WARRANT_TALLY_PURGE_ID: u64 = 11;
/// 영장 승인 후 에스크로 몰수 Reply ID
pub const REPLY_WARRANT_ESCROW_FORFEIT_ID: u64 = 12;
/// 영장 기각 시 voter-manager 롤백 Reply ID
pub const REPLY_WARRANT_ROLLBACK_ID: u64 = 13;

// ============================================================================
// 전자영장 관련 타입 (Electronic Warrant Protocol)
// ============================================================================

/// 전자영장 처리 상태
#[cw_serde]
pub enum WarrantStatus {
    /// 영장 접수 및 사법 서명 검증 대기 중
    Pending,
    /// 영장 발부 확인 → 영구 몰수 처리 완료
    Approved,
    /// 영장 기각 → 원상 복구 처리 완료
    Rejected,
}

/// 사법 기관으로부터 수신하는 전자영장 발부 확인서 (Warrant Issued VP)
/// 사법 기관 게이트웨이가 mTLS를 통해 전달한 증명서
#[cw_serde]
pub struct WarrantIssuedVP {
    /// 문제가 된 여론조사/투표 ID
    pub survey_id: String,
    /// 사기 널리파이어 집합의 머클루트
    pub nullifier_merkle_root: String,
    /// 사법 기관 공개키로 서명된 디지털 서명 (Base64)
    pub judicial_signature: String,
    /// 영장 발부 여부 (true = 승인, false = 기각)
    pub is_approved: bool,
    /// 영장 발부/기각 사유 (사법 기관 코멘트)
    pub reason: String,
    /// 시점확인기관(TSA) 타임스탬프 토큰
    pub tsa_timestamp_token: String,
    /// KICS 사법 내부 접수 고유 연번
    pub kics_receipt_number: u64,
    /// 집계 컨트랙트 주소 (장부 B)
    pub tally_contract_addr: String,
    /// 에스크로 컨트랙트 주소 (장부 C)
    pub escrow_contract_addr: String,
}

/// 사법 기관 설정 (Judicial Configuration)
#[cw_serde]
pub struct JudicialConfig {
    /// 사법 기관(법원/검찰) 공개키 목록
    pub judicial_public_keys: Vec<String>,
    /// 국가 지정 국고 계좌 (Treasury Address)
    pub national_treasury_address: String,
    /// 사법 인증 기관(Crypto-Verifier) 컨트랙트 주소
    pub judicial_verifier_address: String,
}

/// 전자영장 처리 기록
#[cw_serde]
pub struct WarrantRecord {
    /// 사기 널리파이어 집합의 머클루트 (고유 식별자)
    pub nullifier_merkle_root: String,
    /// 문제가 된 투표 세션 ID
    pub survey_id: String,
    /// 현재 영장 처리 상태
    pub status: WarrantStatus,
    /// 영장 접수 블록 높이
    pub submitted_at_height: u64,
    /// 최종 처리 완료 블록 높이
    pub finalized_at_height: Option<u64>,
    /// KICS 접수 번호
    pub kics_receipt_number: u64,
    /// 영장 승인/기각 사유
    pub reason: String,
}

/// 사법 기관 서명 검증 결과 (judicial-verifier → warrant-handler Reply 데이터)
#[cw_serde]
pub struct JudicialVerificationResult {
    /// 검증 대상 널리파이어 머클루트
    pub nullifier_merkle_root: String,
    /// 서명 검증 통과 여부
    pub is_signature_valid: bool,
    /// 영장 발부 여부 (사법 기관의 최종 결정)
    pub is_approved: bool,
}

// ============================================================================
// 전자영장 Cross-Contract 메시지 타입
// ============================================================================

/// 장부 B에 전달되는 격리 투표 영구 파기 메시지
#[cw_serde]
pub struct PurgeSuspendedVotesMsg {
    pub nullifier_merkle_root: String,
}

/// 리워드 에스크로에 전달되는 국고 귀속 몰수 메시지
#[cw_serde]
pub struct ForfeitToTreasuryMsg {
    pub nullifier_merkle_root: String,
    pub treasury_address: String,
}

/// voter-manager에 전달되는 상태 복원 메시지 (영장 기각 시)
#[cw_serde]
pub struct RollbackToActiveMsg {
    pub nullifier_merkle_root: String,
}

/// voter-manager에 전달되는 영구 블랙리스트 확정 메시지
#[cw_serde]
pub struct FinalizeLegalForfeitureMsg {
    pub nullifier_merkle_root: String,
}
