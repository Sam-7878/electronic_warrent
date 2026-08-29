use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

/// 컨트랙트 관리자 주소
pub const ADMIN: Item<Addr> = Item::new("admin");

/// 사법 기관(법원/검찰) 공개키 목록
pub const JUDICIAL_PUBLIC_KEYS: Map<&str, bool> = Map::new("judicial_keys");

/// 사법 기관 공개키 수 카운터
pub const JUDICIAL_KEY_COUNT: Item<u32> = Item::new("judicial_key_count");

/// ZKP 검증 키 식별자 (Groth16/Plonk VK)
pub const ZKP_VERIFICATION_KEY_ID: Item<String> = Item::new("zkp_vk_id");

/// 검증 완료 이력: nullifier_merkle_root -> (is_approved, block_height)
pub const VERIFICATION_HISTORY: Map<&str, (bool, u64)> = Map::new("verify_history");

// --- 통계 카운터 ---

/// 총 검증 수행 횟수
pub const TOTAL_VERIFICATIONS: Item<u64> = Item::new("total_verifications");

/// 총 승인 횟수
pub const TOTAL_APPROVED: Item<u64> = Item::new("total_approved");

/// 총 기각 횟수
pub const TOTAL_REJECTED: Item<u64> = Item::new("total_rejected");
