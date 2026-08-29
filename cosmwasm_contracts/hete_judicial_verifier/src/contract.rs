#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, JudicialKeyCountResponse, JudicialVerifierConfigResponse,
    QueryMsg, VerificationHistoryResponse,
};
use crate::state::{
    ADMIN, JUDICIAL_KEY_COUNT, JUDICIAL_PUBLIC_KEYS, TOTAL_APPROVED, TOTAL_REJECTED,
    TOTAL_VERIFICATIONS, VERIFICATION_HISTORY, ZKP_VERIFICATION_KEY_ID,
};
use voting_common::types::{JudicialVerificationResult, WarrantIssuedVP};

// ============================================================================
// Instantiate
// ============================================================================

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    ADMIN.save(deps.storage, &info.sender)?;

    // 사법 기관 공개키 등록
    let mut key_count = 0u32;
    for key in &msg.judicial_public_keys {
        JUDICIAL_PUBLIC_KEYS.save(deps.storage, key, &true)?;
        key_count += 1;
    }
    JUDICIAL_KEY_COUNT.save(deps.storage, &key_count)?;

    ZKP_VERIFICATION_KEY_ID.save(deps.storage, &msg.zkp_verification_key_id)?;

    // 통계 초기화
    TOTAL_VERIFICATIONS.save(deps.storage, &0u64)?;
    TOTAL_APPROVED.save(deps.storage, &0u64)?;
    TOTAL_REJECTED.save(deps.storage, &0u64)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", info.sender)
        .add_attribute("judicial_keys", key_count.to_string())
        .add_attribute("zkp_vk_id", msg.zkp_verification_key_id))
}

// ============================================================================
// Execute
// ============================================================================

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::VerifyJudicialWarrant { vp, expected_root } => {
            execute_verify_judicial_warrant(deps, env, vp, expected_root)
        }
        ExecuteMsg::AddJudicialPublicKey { key } => {
            execute_add_judicial_key(deps, info, key)
        }
        ExecuteMsg::RemoveJudicialPublicKey { key } => {
            execute_remove_judicial_key(deps, info, key)
        }
    }
}

/// 사법 기관 영장 서명 검증
/// warrant-handler가 SubMsg::reply_on_success로 호출
/// 검증 결과를 Response::set_data()로 반환하여 Reply에서 파싱
fn execute_verify_judicial_warrant(
    deps: DepsMut,
    env: Env,
    vp: WarrantIssuedVP,
    expected_root: String,
) -> Result<Response, ContractError> {
    // 1. VP 기본 무결성 검사
    if vp.nullifier_merkle_root.is_empty()
        || vp.judicial_signature.is_empty()
        || vp.survey_id.is_empty()
    {
        return Err(ContractError::MalformedWarrantVP {});
    }

    // 2. 머클루트 일치 여부 확인
    if vp.nullifier_merkle_root != expected_root {
        return Err(ContractError::MalformedWarrantVP {});
    }

    // 3. 사법 기관 공개키 서명 검증 (데모: 해시 기반 시뮬레이션)
    let key_count = JUDICIAL_KEY_COUNT.load(deps.storage)?;
    if key_count == 0 {
        return Err(ContractError::NoJudicialKeysRegistered {});
    }

    let is_signature_valid =
        verify_judicial_signature(deps.as_ref(), &vp.judicial_signature, &expected_root)?;

    if !is_signature_valid {
        return Err(ContractError::JudicialSignatureInvalid {
            merkle_root: expected_root,
        });
    }

    // 4. 검증 결과 기록
    VERIFICATION_HISTORY.save(
        deps.storage,
        &vp.nullifier_merkle_root,
        &(vp.is_approved, env.block.height),
    )?;

    // 5. 통계 업데이트
    let total = TOTAL_VERIFICATIONS.load(deps.storage).unwrap_or(0) + 1;
    TOTAL_VERIFICATIONS.save(deps.storage, &total)?;

    if vp.is_approved {
        let approved = TOTAL_APPROVED.load(deps.storage).unwrap_or(0) + 1;
        TOTAL_APPROVED.save(deps.storage, &approved)?;
    } else {
        let rejected = TOTAL_REJECTED.load(deps.storage).unwrap_or(0) + 1;
        TOTAL_REJECTED.save(deps.storage, &rejected)?;
    }

    // 6. 결과를 Response::data로 반환 (warrant-handler의 Reply에서 파싱)
    let result = JudicialVerificationResult {
        nullifier_merkle_root: vp.nullifier_merkle_root.clone(),
        is_signature_valid: true,
        is_approved: vp.is_approved,
    };

    Ok(Response::new()
        .set_data(to_json_binary(&result)?)
        .add_attribute("action", "verify_judicial_warrant")
        .add_attribute("nullifier_root", vp.nullifier_merkle_root)
        .add_attribute("is_approved", vp.is_approved.to_string())
        .add_attribute("survey_id", vp.survey_id))
}

/// 사법 기관 서명 검증 (데모용 간소화)
/// 실제 프로덕션에서는 ECDSA/EdDSA 서명 검증 또는 Groth16 페어링 연산 수행
fn verify_judicial_signature(
    deps: Deps,
    signature: &str,
    _expected_root: &str,
) -> Result<bool, ContractError> {
    // 데모: 서명이 비어있지 않고, 등록된 사법 공개키가 1개 이상이면 검증 통과
    // 실제 프로덕션: Fiat-Shamir Heuristic 대조 + 쌍선형 페어링 연산
    let keys: Vec<String> = JUDICIAL_PUBLIC_KEYS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .filter_map(|r| r.ok().map(|(k, _)| k))
        .collect();

    Ok(!signature.is_empty() && !keys.is_empty())
}

fn execute_add_judicial_key(
    deps: DepsMut,
    info: MessageInfo,
    key: String,
) -> Result<Response, ContractError> {
    let admin = ADMIN.load(deps.storage)?;
    if info.sender != admin {
        return Err(ContractError::Unauthorized {});
    }

    JUDICIAL_PUBLIC_KEYS.save(deps.storage, &key, &true)?;
    let count = JUDICIAL_KEY_COUNT.load(deps.storage)? + 1;
    JUDICIAL_KEY_COUNT.save(deps.storage, &count)?;

    Ok(Response::new()
        .add_attribute("action", "add_judicial_key")
        .add_attribute("count", count.to_string()))
}

fn execute_remove_judicial_key(
    deps: DepsMut,
    info: MessageInfo,
    key: String,
) -> Result<Response, ContractError> {
    let admin = ADMIN.load(deps.storage)?;
    if info.sender != admin {
        return Err(ContractError::Unauthorized {});
    }

    JUDICIAL_PUBLIC_KEYS.remove(deps.storage, &key);
    let count = JUDICIAL_KEY_COUNT.load(deps.storage)?;
    if count > 0 {
        JUDICIAL_KEY_COUNT.save(deps.storage, &(count - 1))?;
    }

    Ok(Response::new()
        .add_attribute("action", "remove_judicial_key")
        .add_attribute("remaining_count", count.saturating_sub(1).to_string()))
}

// ============================================================================
// Query
// ============================================================================

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_json_binary(&query_config(deps)?),
        QueryMsg::GetJudicialKeyCount {} => to_json_binary(&query_key_count(deps)?),
        QueryMsg::GetVerificationHistory {
            nullifier_merkle_root,
        } => to_json_binary(&query_verification_history(deps, nullifier_merkle_root)?),
    }
}

fn query_config(deps: Deps) -> StdResult<JudicialVerifierConfigResponse> {
    let admin = ADMIN.load(deps.storage)?;
    let key_count = JUDICIAL_KEY_COUNT.load(deps.storage)?;
    let zkp_vk_id = ZKP_VERIFICATION_KEY_ID.load(deps.storage)?;
    let total_verifications = TOTAL_VERIFICATIONS.load(deps.storage).unwrap_or(0);
    let total_approved = TOTAL_APPROVED.load(deps.storage).unwrap_or(0);
    let total_rejected = TOTAL_REJECTED.load(deps.storage).unwrap_or(0);

    Ok(JudicialVerifierConfigResponse {
        admin: admin.to_string(),
        judicial_key_count: key_count,
        zkp_verification_key_id: zkp_vk_id,
        total_verifications,
        total_approved,
        total_rejected,
    })
}

fn query_key_count(deps: Deps) -> StdResult<JudicialKeyCountResponse> {
    let count = JUDICIAL_KEY_COUNT.load(deps.storage)?;
    Ok(JudicialKeyCountResponse { count })
}

fn query_verification_history(
    deps: Deps,
    nullifier_merkle_root: String,
) -> StdResult<VerificationHistoryResponse> {
    let history = VERIFICATION_HISTORY.may_load(deps.storage, &nullifier_merkle_root)?;
    match history {
        Some((is_approved, height)) => Ok(VerificationHistoryResponse {
            nullifier_merkle_root,
            was_verified: true,
            is_approved,
            verified_at_height: height,
        }),
        None => Ok(VerificationHistoryResponse {
            nullifier_merkle_root,
            was_verified: false,
            is_approved: false,
            verified_at_height: 0,
        }),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use voting_common::types::WarrantIssuedVP;

    fn default_instantiate_msg() -> InstantiateMsg {
        InstantiateMsg {
            judicial_public_keys: vec![
                "judicial_pk_court_001".to_string(),
                "judicial_pk_prosecutor_001".to_string(),
            ],
            zkp_verification_key_id: "groth16_vk_v1".to_string(),
        }
    }

    fn default_warrant_vp() -> WarrantIssuedVP {
        WarrantIssuedVP {
            survey_id: "election_2026_001".to_string(),
            nullifier_merkle_root: "abcdef1234567890abcdef1234567890".to_string(),
            judicial_signature: "valid_judicial_sig_base64".to_string(),
            is_approved: true,
            reason: "조직적 봇넷 사기 확인".to_string(),
            tsa_timestamp_token: "tsa_token_001".to_string(),
            kics_receipt_number: 20260618001,
            tally_contract_addr: "tally_addr".to_string(),
            escrow_contract_addr: "escrow_addr".to_string(),
        }
    }

    #[test]
    fn test_instantiate_and_config() {
        let mut deps = mock_dependencies();
        let admin_addr = deps.api.addr_make("admin");
        let info = mock_info(admin_addr.as_str(), &[]);
        let msg = default_instantiate_msg();

        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.attributes.len(), 4);

        let config: JudicialVerifierConfigResponse =
            cosmwasm_std::from_json(
                query(deps.as_ref(), mock_env(), QueryMsg::GetConfig {}).unwrap(),
            )
            .unwrap();
        assert_eq!(config.admin, admin_addr.to_string());
        assert_eq!(config.judicial_key_count, 2);
        assert_eq!(config.zkp_verification_key_id, "groth16_vk_v1");
        assert_eq!(config.total_verifications, 0);
    }

    #[test]
    fn test_verify_judicial_warrant_approved() {
        let mut deps = mock_dependencies();
        let admin_addr = deps.api.addr_make("admin");
        let info = mock_info(admin_addr.as_str(), &[]);
        instantiate(deps.as_mut(), mock_env(), info, default_instantiate_msg()).unwrap();

        let vp = default_warrant_vp();
        let expected_root = vp.nullifier_merkle_root.clone();

        let handler_addr = deps.api.addr_make("warrant_handler");
        let handler_info = mock_info(handler_addr.as_str(), &[]);
        let res = execute(
            deps.as_mut(),
            mock_env(),
            handler_info,
            ExecuteMsg::VerifyJudicialWarrant {
                vp,
                expected_root: expected_root.clone(),
            },
        )
        .unwrap();

        // Response::data에 JudicialVerificationResult가 포함되어 있는지 확인
        assert!(res.data.is_some());
        let result: JudicialVerificationResult =
            cosmwasm_std::from_json(res.data.unwrap()).unwrap();
        assert!(result.is_signature_valid);
        assert!(result.is_approved);
        assert_eq!(result.nullifier_merkle_root, expected_root);

        // 통계 확인
        let config: JudicialVerifierConfigResponse =
            cosmwasm_std::from_json(
                query(deps.as_ref(), mock_env(), QueryMsg::GetConfig {}).unwrap(),
            )
            .unwrap();
        assert_eq!(config.total_verifications, 1);
        assert_eq!(config.total_approved, 1);
        assert_eq!(config.total_rejected, 0);
    }

    #[test]
    fn test_verify_judicial_warrant_rejected() {
        let mut deps = mock_dependencies();
        let admin_addr = deps.api.addr_make("admin");
        let info = mock_info(admin_addr.as_str(), &[]);
        instantiate(deps.as_mut(), mock_env(), info, default_instantiate_msg()).unwrap();

        let mut vp = default_warrant_vp();
        vp.is_approved = false;
        vp.reason = "증거 불충분으로 기각".to_string();
        let expected_root = vp.nullifier_merkle_root.clone();

        let handler_addr = deps.api.addr_make("warrant_handler");
        let handler_info = mock_info(handler_addr.as_str(), &[]);
        let res = execute(
            deps.as_mut(),
            mock_env(),
            handler_info,
            ExecuteMsg::VerifyJudicialWarrant {
                vp,
                expected_root,
            },
        )
        .unwrap();

        let result: JudicialVerificationResult =
            cosmwasm_std::from_json(res.data.unwrap()).unwrap();
        assert!(result.is_signature_valid);
        assert!(!result.is_approved);

        // 통계 확인
        let config: JudicialVerifierConfigResponse =
            cosmwasm_std::from_json(
                query(deps.as_ref(), mock_env(), QueryMsg::GetConfig {}).unwrap(),
            )
            .unwrap();
        assert_eq!(config.total_rejected, 1);
    }

    #[test]
    fn test_malformed_vp_rejected() {
        let mut deps = mock_dependencies();
        let admin_addr = deps.api.addr_make("admin");
        let info = mock_info(admin_addr.as_str(), &[]);
        instantiate(deps.as_mut(), mock_env(), info, default_instantiate_msg()).unwrap();

        // 빈 서명
        let mut vp = default_warrant_vp();
        vp.judicial_signature = "".to_string();
        let expected_root = vp.nullifier_merkle_root.clone();

        let handler_addr = deps.api.addr_make("warrant_handler");
        let handler_info = mock_info(handler_addr.as_str(), &[]);
        let err = execute(
            deps.as_mut(),
            mock_env(),
            handler_info,
            ExecuteMsg::VerifyJudicialWarrant {
                vp,
                expected_root,
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::MalformedWarrantVP {}));
    }

    #[test]
    fn test_mismatched_merkle_root_rejected() {
        let mut deps = mock_dependencies();
        let admin_addr = deps.api.addr_make("admin");
        let info = mock_info(admin_addr.as_str(), &[]);
        instantiate(deps.as_mut(), mock_env(), info, default_instantiate_msg()).unwrap();

        let vp = default_warrant_vp();

        let handler_addr = deps.api.addr_make("warrant_handler");
        let handler_info = mock_info(handler_addr.as_str(), &[]);
        let err = execute(
            deps.as_mut(),
            mock_env(),
            handler_info,
            ExecuteMsg::VerifyJudicialWarrant {
                vp,
                expected_root: "wrong_root_hash".to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::MalformedWarrantVP {}));
    }

    #[test]
    fn test_unauthorized_key_management() {
        let mut deps = mock_dependencies();
        let admin_addr = deps.api.addr_make("admin");
        let info = mock_info(admin_addr.as_str(), &[]);
        instantiate(deps.as_mut(), mock_env(), info, default_instantiate_msg()).unwrap();

        // 비관리자가 공개키 추가 시도
        let hacker_addr = deps.api.addr_make("hacker");
        let hacker_info = mock_info(hacker_addr.as_str(), &[]);
        let err = execute(
            deps.as_mut(),
            mock_env(),
            hacker_info,
            ExecuteMsg::AddJudicialPublicKey {
                key: "malicious_key".to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized {}));
    }

    #[test]
    fn test_add_and_remove_judicial_key() {
        let mut deps = mock_dependencies();
        let admin_addr = deps.api.addr_make("admin");
        let info = mock_info(admin_addr.as_str(), &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), default_instantiate_msg()).unwrap();

        // 키 추가
        execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            ExecuteMsg::AddJudicialPublicKey {
                key: "new_judicial_key".to_string(),
            },
        )
        .unwrap();

        let count: JudicialKeyCountResponse = cosmwasm_std::from_json(
            query(deps.as_ref(), mock_env(), QueryMsg::GetJudicialKeyCount {}).unwrap(),
        )
        .unwrap();
        assert_eq!(count.count, 3);

        // 키 제거
        execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::RemoveJudicialPublicKey {
                key: "new_judicial_key".to_string(),
            },
        )
        .unwrap();

        let count: JudicialKeyCountResponse = cosmwasm_std::from_json(
            query(deps.as_ref(), mock_env(), QueryMsg::GetJudicialKeyCount {}).unwrap(),
        )
        .unwrap();
        assert_eq!(count.count, 2);
    }

    #[test]
    fn test_verification_history() {
        let mut deps = mock_dependencies();
        let admin_addr = deps.api.addr_make("admin");
        let info = mock_info(admin_addr.as_str(), &[]);
        instantiate(deps.as_mut(), mock_env(), info, default_instantiate_msg()).unwrap();

        // 먼저 이력 없음 확인
        let history: VerificationHistoryResponse = cosmwasm_std::from_json(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::GetVerificationHistory {
                    nullifier_merkle_root: "nonexistent".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert!(!history.was_verified);

        // 영장 검증 수행
        let vp = default_warrant_vp();
        let expected_root = vp.nullifier_merkle_root.clone();
        let handler_addr = deps.api.addr_make("warrant_handler");
        let handler_info = mock_info(handler_addr.as_str(), &[]);
        execute(
            deps.as_mut(),
            mock_env(),
            handler_info,
            ExecuteMsg::VerifyJudicialWarrant {
                vp,
                expected_root: expected_root.clone(),
            },
        )
        .unwrap();

        // 이력 확인
        let history: VerificationHistoryResponse = cosmwasm_std::from_json(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::GetVerificationHistory {
                    nullifier_merkle_root: expected_root,
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert!(history.was_verified);
        assert!(history.is_approved);
    }
}
