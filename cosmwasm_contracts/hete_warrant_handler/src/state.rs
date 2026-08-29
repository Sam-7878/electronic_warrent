use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use voting_common::types::{WarrantIssuedVP, WarrantRecord};

/// 컨트랙트 관리자
pub const ADMIN: Item<Addr> = Item::new("admin");

/// 사법 서명 검증기 (hete-judicial-verifier) 컨트랙트 주소
pub const JUDICIAL_VERIFIER_CONTRACT: Item<Addr> = Item::new("judicial_verifier");

/// 장부 A (voter-manager) 컨트랙트 주소
pub const VOTER_MANAGER_CONTRACT: Item<Addr> = Item::new("voter_manager");

/// 장부 B (tally-accumulator) 컨트랙트 주소
pub const TALLY_CONTRACT: Item<Addr> = Item::new("tally_contract");

/// 장부 C (reward-escrow) 컨트랙트 주소
pub const ESCROW_CONTRACT: Item<Addr> = Item::new("escrow_contract");

/// 국가 지정 국고 계좌
pub const NATIONAL_TREASURY_ADDRESS: Item<String> = Item::new("treasury_addr");

/// 사법 게이트웨이 권한 주소 목록
pub const GATEWAY_AUTHORITIES: Map<&str, bool> = Map::new("gateway_authorities");

/// 사법 게이트웨이 권한 주소 수 카운터
pub const GATEWAY_COUNT: Item<u32> = Item::new("gateway_count");

/// 영장 처리 기록: nullifier_merkle_root -> WarrantRecord
pub const WARRANT_RECORDS: Map<&str, WarrantRecord> = Map::new("warrant_records");

/// 영장 검증 대기 중인 VP 임시 보관: nullifier_merkle_root -> WarrantIssuedVP
pub const PENDING_WARRANT_VP: Map<&str, WarrantIssuedVP> = Map::new("pending_warrant_vp");

// --- 통계 카운터 ---

/// 총 영장 제출 수
pub const TOTAL_SUBMITTED: Item<u64> = Item::new("total_submitted");

/// 총 영장 승인 수
pub const TOTAL_APPROVED: Item<u64> = Item::new("total_approved");

/// 총 영장 기각 수
pub const TOTAL_REJECTED: Item<u64> = Item::new("total_rejected");

/// 총 롤백 수
pub const TOTAL_ROLLBACKS: Item<u64> = Item::new("total_rollbacks");
