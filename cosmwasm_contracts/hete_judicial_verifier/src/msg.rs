use cosmwasm_schema::{cw_serde, QueryResponses};
use voting_common::types::WarrantIssuedVP;

// ============================================================================
// InstantiateMsg - 사법 기관 서명 검증기 초기화
// ============================================================================

#[cw_serde]
pub struct InstantiateMsg {
    /// 사법 기관(법원/검찰) 공개키 목록 (Base64 인코딩)
    pub judicial_public_keys: Vec<String>,
    /// ZKP 검증 파라미터 (Groth16/Plonk 검증 키 식별자)
    pub zkp_verification_key_id: String,
}

// ============================================================================
// ExecuteMsg
// ============================================================================

#[cw_serde]
pub enum ExecuteMsg {
    /// 사법 기관 영장 서명 검증 (warrant-handler에서 SubMsg로 호출)
    /// 검증 결과를 Response::data에 JudicialVerificationResult로 반환
    VerifyJudicialWarrant {
        vp: WarrantIssuedVP,
        expected_root: String,
    },

    /// 사법 기관 공개키 추가 (관리자 전용)
    AddJudicialPublicKey {
        key: String,
    },

    /// 사법 기관 공개키 제거 (관리자 전용)
    RemoveJudicialPublicKey {
        key: String,
    },
}

// ============================================================================
// QueryMsg
// ============================================================================

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// 컨트랙트 설정 조회
    #[returns(JudicialVerifierConfigResponse)]
    GetConfig {},

    /// 등록된 사법 기관 공개키 수 조회
    #[returns(JudicialKeyCountResponse)]
    GetJudicialKeyCount {},

    /// 검증 이력 조회
    #[returns(VerificationHistoryResponse)]
    GetVerificationHistory {
        nullifier_merkle_root: String,
    },
}

// ============================================================================
// Query Responses
// ============================================================================

#[cw_serde]
pub struct JudicialVerifierConfigResponse {
    pub admin: String,
    pub judicial_key_count: u32,
    pub zkp_verification_key_id: String,
    pub total_verifications: u64,
    pub total_approved: u64,
    pub total_rejected: u64,
}

#[cw_serde]
pub struct JudicialKeyCountResponse {
    pub count: u32,
}

#[cw_serde]
pub struct VerificationHistoryResponse {
    pub nullifier_merkle_root: String,
    pub was_verified: bool,
    pub is_approved: bool,
    pub verified_at_height: u64,
}
