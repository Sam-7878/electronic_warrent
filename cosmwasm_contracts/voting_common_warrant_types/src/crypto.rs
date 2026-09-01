use crate::types::NullifierState;
use hete_core::ValidationError;
use sha2::{Digest, Sha256};

/// Nullifier 유도 함수
/// Nullifier = Hash(Key2_Public, VoteSessionID)
/// 중복 투표를 방지하기 위한 일회성 무효화 식별자 생성
pub fn derive_nullifier(key2_public: &str, vote_session_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key2_public.as_bytes());
    hasher.update(b"|");
    hasher.update(vote_session_id.as_bytes());
    hex::encode(hasher.finalize())
}

/// 서비스 키(Key 3) 유도 함수 (데모용 간소화)
/// 실제 프로덕션에서는 ZKP(영지식 증명) 메커니즘 적용
///
/// NOTE: Voting 도메인은 "SERVICE_KEY_V1|" 접두사를 사용하여
/// Banking 도메인의 hete_core::derive_service_key와는 다른 키 공간을 유지합니다.
pub fn derive_service_key(key1_credential: &str, key2_public: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"SERVICE_KEY_V1|");
    hasher.update(key1_credential.as_bytes());
    hasher.update(b"|");
    hasher.update(key2_public.as_bytes());
    hex::encode(hasher.finalize())
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct VoterVerificationRequestData {
    pub tee_attestation: String,
    pub trusted_roots: Vec<String>,
    pub malware_scan_leaf_hash: String,
    pub anoncreds_proof: String,
    pub trusted_issuers: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct VerificationRequest {
    pub credential: Option<hete_core::IdentityCredential>,
    pub context: Option<hete_core::QueryContext>,
    pub current_time: Option<u64>,
    pub required_scope: Option<String>,
    pub voter_data: Option<VoterVerificationRequestData>,
}

#[derive(Serialize, Deserialize)]
struct VerificationResponse {
    pub valid: bool,
    pub message: String,
}

fn query_openbsd_voter_verifier(voter_data: VoterVerificationRequestData) -> Result<bool, String> {
    use std::io::{Read, Write};
    use std::net::TcpStream;

    // Explicit injection has precedence so tests and deployments do not depend
    // on a developer-machine path. Configuration files are only fallbacks.
    let verifier_url = if let Ok(url) = std::env::var("HETE_VERIFIER_URL") {
        url
    } else if let Ok(content) =
        std::fs::read_to_string("/mnt/d/_Work/goat_bank/hete/local_local_security/open_bsd_connection.json")
    {
        if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
            let ip = config["verifier_ip"].as_str().unwrap_or("192.168.1.103");
            let port = config["verifier_port"].as_u64().unwrap_or(50051);
            format!("{}:{}", ip, port)
        } else {
            std::env::var("HETE_VERIFIER_URL").unwrap_or_else(|_| "mock".to_string())
        }
    } else if let Ok(content) =
        std::fs::read_to_string("d:/_Work/goat_bank/hete/local_local_security/open_bsd_connection.json")
    {
        if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
            let ip = config["verifier_ip"].as_str().unwrap_or("192.168.1.103");
            let port = config["verifier_port"].as_u64().unwrap_or(50051);
            format!("{}:{}", ip, port)
        } else {
            std::env::var("HETE_VERIFIER_URL").unwrap_or_else(|_| "mock".to_string())
        }
    } else {
        std::env::var("HETE_VERIFIER_URL").unwrap_or_else(|_| "mock".to_string())
    };

    if verifier_url == "mock" {
        return Ok(!voter_data.tee_attestation.is_empty()
            && !voter_data.trusted_roots.is_empty()
            && !voter_data.anoncreds_proof.is_empty()
            && !voter_data.trusted_issuers.is_empty());
    }

    let mut stream = TcpStream::connect(&verifier_url)
        .map_err(|e| format!("Failed to connect to OpenBSD verifier: {}", e))?;

    let req = VerificationRequest {
        credential: None,
        context: None,
        current_time: None,
        required_scope: None,
        voter_data: Some(voter_data),
    };

    let req_bytes = serde_json::to_vec(&req).map_err(|e| e.to_string())?;
    stream.write_all(&req_bytes).map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())?;

    let mut response_bytes = Vec::new();
    let mut buffer = [0; 4096];
    match stream.read(&mut buffer) {
        Ok(size) if size > 0 => {
            response_bytes.extend_from_slice(&buffer[..size]);
            let resp = serde_json::from_slice::<VerificationResponse>(&response_bytes)
                .map_err(|e| format!("Failed to parse verifier response: {}", e))?;
            Ok(resp.valid)
        }
        _ => Err("Received empty response from verifier".to_string()),
    }
}

/// TEE Attestation 데이터 검증 (데모용 간소화)
/// 실제 프로덕션에서는 X.509 인증서 체인 검증 + 하드웨어 서명 무결성 검사
pub fn verify_tee_attestation(
    attestation_data: &str,
    trusted_roots: &[String],
    _malware_scan_hash: &str,
) -> bool {
    let req = VoterVerificationRequestData {
        tee_attestation: attestation_data.to_string(),
        trusted_roots: trusted_roots.to_vec(),
        malware_scan_leaf_hash: _malware_scan_hash.to_string(),
        anoncreds_proof: "dummy_non_empty".to_string(),
        trusted_issuers: vec!["dummy_non_empty".to_string()],
    };
    query_openbsd_voter_verifier(req).unwrap_or(false)
}

/// AnonCreds 영지식 증명 검증 (데모용 간소화)
/// 실제 프로덕션에서는 Hyperledger AnonCreds 라이브러리 사용
pub fn verify_anoncreds_proof(proof: &str, trusted_issuers: &[String]) -> bool {
    let req = VoterVerificationRequestData {
        tee_attestation: "dummy_non_empty".to_string(),
        trusted_roots: vec!["dummy_non_empty".to_string()],
        malware_scan_leaf_hash: "".to_string(),
        anoncreds_proof: proof.to_string(),
        trusted_issuers: trusted_issuers.to_vec(),
    };
    query_openbsd_voter_verifier(req).unwrap_or(false)
}

/// 동형암호 투표값 암호화 (데모용 간소화)
/// 실제 프로덕션에서는 Paillier 또는 BFV 동형암호 사용
pub fn encrypt_vote(candidate_index: u32, session_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"HE_VOTE|");
    hasher.update(candidate_index.to_le_bytes());
    hasher.update(b"|");
    hasher.update(session_id.as_bytes());
    hex::encode(hasher.finalize())
}

// ============================================================================
// POA: VotingNullifierManager — NullifierLifecycle 구현체
// ============================================================================

/// Voting 도메인 전용 NullifierLifecycle 구현체.
///
/// hete-core의 `NullifierLifecycle` 트레이트를 만족하며,
/// Nullifier 유도는 기존 `derive_nullifier()` 함수에 위임하고,
/// 상태 전이 규칙을 강제합니다.
///
/// 허용되는 전이:
/// - Active → Suspended  (GNN 사기 탐지)
/// - Suspended → Voided  (복식 부기 정산 완료)
/// - Suspended → Active  (전자영장 기각 시 복원)
/// - Suspended → PermanentBlacklist  (전자영장 발부 확정)
pub struct VotingNullifierManager;

/// Voting-specific lifecycle. The generic trust-state lifecycle is not reused
/// because ballot settlement has terminal states (`Voided` and
/// `PermanentBlacklist`) that are not actor trust states.
pub trait VotingNullifierLifecycle {
    fn derive_nullifier(&self, key_material: &str, session_id: &str) -> String;
    fn transition_state(
        &mut self,
        current: NullifierState,
        target: NullifierState,
    ) -> Result<(), ValidationError>;
}

impl VotingNullifierLifecycle for VotingNullifierManager {
    fn derive_nullifier(&self, key_material: &str, session_id: &str) -> String {
        derive_nullifier(key_material, session_id)
    }

    fn transition_state(
        &mut self,
        current: NullifierState,
        target: NullifierState,
    ) -> Result<(), ValidationError> {
        match (&current, &target) {
            // GNN 사기 탐지: Active → Suspended
            (NullifierState::Active, NullifierState::Suspended) => Ok(()),
            // 복식 부기 정산 완료: Suspended → Voided
            (NullifierState::Suspended, NullifierState::Voided) => Ok(()),
            // 전자영장 기각 시 복원: Suspended → Active
            (NullifierState::Suspended, NullifierState::Active) => Ok(()),
            // 전자영장 발부 확정: Suspended → PermanentBlacklist
            (NullifierState::Suspended, NullifierState::PermanentBlacklist) => Ok(()),
            // 그 외 모든 전이는 불허
            _ => Err(ValidationError::ConstraintViolated(format!(
                "Invalid Nullifier state transition: {:?} → {:?}",
                current, target
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_nullifier_deterministic() {
        let n1 = derive_nullifier("pubkey_abc", "session_001");
        let n2 = derive_nullifier("pubkey_abc", "session_001");
        assert_eq!(n1, n2);
    }

    #[test]
    fn test_derive_nullifier_unique_per_session() {
        let n1 = derive_nullifier("pubkey_abc", "session_001");
        let n2 = derive_nullifier("pubkey_abc", "session_002");
        assert_ne!(n1, n2);
    }

    #[test]
    fn test_derive_nullifier_unique_per_voter() {
        let n1 = derive_nullifier("pubkey_abc", "session_001");
        let n2 = derive_nullifier("pubkey_def", "session_001");
        assert_ne!(n1, n2);
    }

    #[test]
    fn test_derive_service_key() {
        let key = derive_service_key("cred_001", "pubkey_abc");
        assert!(!key.is_empty());
        assert_eq!(key.len(), 64); // SHA256 hex output
    }

    #[test]
    fn test_encrypt_vote() {
        let ct = encrypt_vote(0, "session_001");
        assert!(!ct.is_empty());
    }

    // ── POA 트레이트 검증 테스트 ──

    #[test]
    fn test_voting_nullifier_manager_derive() {
        let mut mgr = VotingNullifierManager;
        let n = mgr.derive_nullifier("pubkey_abc", "session_001");
        assert_eq!(n, derive_nullifier("pubkey_abc", "session_001"));
    }

    #[test]
    fn test_voting_nullifier_manager_valid_transitions() {
        let mut mgr = VotingNullifierManager;
        // Active → Suspended (GNN 사기 탐지)
        assert!(mgr
            .transition_state(NullifierState::Active, NullifierState::Suspended)
            .is_ok());
        // Suspended → Voided (정산 완료)
        assert!(mgr
            .transition_state(NullifierState::Suspended, NullifierState::Voided)
            .is_ok());
        // Suspended → Active (영장 기각)
        assert!(mgr
            .transition_state(NullifierState::Suspended, NullifierState::Active)
            .is_ok());
        // Suspended → PermanentBlacklist (영장 발부)
        assert!(mgr
            .transition_state(
                NullifierState::Suspended,
                NullifierState::PermanentBlacklist
            )
            .is_ok());
    }

    #[test]
    fn test_voting_nullifier_manager_invalid_transitions() {
        let mut mgr = VotingNullifierManager;
        // Active → Voided (직접 전이 불가)
        assert!(mgr
            .transition_state(NullifierState::Active, NullifierState::Voided)
            .is_err());
        // Voided → Active (복원 불가)
        assert!(mgr
            .transition_state(NullifierState::Voided, NullifierState::Active)
            .is_err());
        // PermanentBlacklist → Active (해제 불가)
        assert!(mgr
            .transition_state(NullifierState::PermanentBlacklist, NullifierState::Active)
            .is_err());
    }
}
