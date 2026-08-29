use cosmwasm_schema::{cw_serde, QueryResponses};
use voting_common::types::{WarrantIssuedVP, WarrantRecord, WarrantStatus};

// ============================================================================
// InstantiateMsg - 전자영장 정산 오케스트레이터 초기화
// ============================================================================

#[cw_serde]
pub struct InstantiateMsg {
    /// 사법 서명 검증기 (hete-judicial-verifier) 컨트랙트 주소
    pub judicial_verifier_contract: String,
    /// 장부 A (hete-voter-manager) 컨트랙트 주소
    pub voter_manager_contract: String,
    /// 장부 B (hete-tally-accumulator) 컨트랙트 주소
    pub tally_contract: String,
    /// 장부 C (hete-reward-escrow) 컨트랙트 주소
    pub escrow_contract: String,
    /// 국가 지정 국고 계좌 (Treasury Address)
    pub national_treasury_address: String,
    /// 사법 게이트웨이 권한 주소 목록 (mTLS 릴레이어)
    pub authorized_gateway_addresses: Vec<String>,
}

// ============================================================================
// ExecuteMsg
// ============================================================================

#[cw_serde]
pub enum ExecuteMsg {
    /// 사법 기관 게이트웨이로부터 영장 발부/기각 결과 수신
    /// judicial-verifier에 서명 검증을 위임 (SubMsg)
    SubmitWarrantResult {
        warrant_proof: WarrantIssuedVP,
    },

    /// TTL 만료 시 관리자가 수동으로 Suspended 상태를 Active로 롤백
    RollbackSuspended {
        nullifier_merkle_root: String,
    },

    /// 사법 게이트웨이 권한 주소 추가 (관리자 전용)
    AddGatewayAuthority {
        address: String,
    },

    /// 사법 게이트웨이 권한 주소 제거 (관리자 전용)
    RemoveGatewayAuthority {
        address: String,
    },
}

// ============================================================================
// QueryMsg
// ============================================================================

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// 특정 영장 처리 상태 조회
    #[returns(WarrantStatusResponse)]
    GetWarrantStatus {
        nullifier_merkle_root: String,
    },

    /// 컨트랙트 설정 조회
    #[returns(WarrantHandlerConfigResponse)]
    GetConfig {},

    /// 전체 영장 처리 통계 조회
    #[returns(WarrantStatsResponse)]
    GetStats {},
}

// ============================================================================
// Query Responses
// ============================================================================

#[cw_serde]
pub struct WarrantStatusResponse {
    pub record: Option<WarrantRecord>,
}

#[cw_serde]
pub struct WarrantHandlerConfigResponse {
    pub admin: String,
    pub judicial_verifier_contract: String,
    pub voter_manager_contract: String,
    pub tally_contract: String,
    pub escrow_contract: String,
    pub national_treasury_address: String,
    pub gateway_count: u32,
}

#[cw_serde]
pub struct WarrantStatsResponse {
    pub total_submitted: u64,
    pub total_approved: u64,
    pub total_rejected: u64,
    pub total_rollbacks: u64,
}
