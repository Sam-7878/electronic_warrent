#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Env, Event, MessageInfo, Reply, Response,
    StdResult, SubMsg, WasmMsg,
};

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, QueryMsg, WarrantHandlerConfigResponse, WarrantStatsResponse,
    WarrantStatusResponse,
};
use crate::state::{
    ADMIN, ESCROW_CONTRACT, GATEWAY_AUTHORITIES, GATEWAY_COUNT, JUDICIAL_VERIFIER_CONTRACT,
    NATIONAL_TREASURY_ADDRESS, PENDING_WARRANT_VP, TALLY_CONTRACT, TOTAL_APPROVED,
    TOTAL_REJECTED, TOTAL_ROLLBACKS, TOTAL_SUBMITTED, VOTER_MANAGER_CONTRACT, WARRANT_RECORDS,
};
use voting_common::types::{
    JudicialVerificationResult, WarrantIssuedVP, WarrantRecord, WarrantStatus,
    REPLY_JUDICIAL_VERIFY_ID,
};
use voting_common::{TransactionProtocol, StateProof};

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

    let jv = deps.api.addr_validate(&msg.judicial_verifier_contract)?;
    JUDICIAL_VERIFIER_CONTRACT.save(deps.storage, &jv)?;

    let vm = deps.api.addr_validate(&msg.voter_manager_contract)?;
    VOTER_MANAGER_CONTRACT.save(deps.storage, &vm)?;

    let tc = deps.api.addr_validate(&msg.tally_contract)?;
    TALLY_CONTRACT.save(deps.storage, &tc)?;

    let ec = deps.api.addr_validate(&msg.escrow_contract)?;
    ESCROW_CONTRACT.save(deps.storage, &ec)?;

    NATIONAL_TREASURY_ADDRESS.save(deps.storage, &msg.national_treasury_address)?;

    let mut gw_count = 0u32;
    for addr in &msg.authorized_gateway_addresses {
        GATEWAY_AUTHORITIES.save(deps.storage, addr, &true)?;
        gw_count += 1;
    }
    GATEWAY_COUNT.save(deps.storage, &gw_count)?;

    // 통계 초기화
    TOTAL_SUBMITTED.save(deps.storage, &0u64)?;
    TOTAL_APPROVED.save(deps.storage, &0u64)?;
    TOTAL_REJECTED.save(deps.storage, &0u64)?;
    TOTAL_ROLLBACKS.save(deps.storage, &0u64)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", info.sender)
        .add_attribute("judicial_verifier", msg.judicial_verifier_contract)
        .add_attribute("gateway_count", gw_count.to_string()))
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
        ExecuteMsg::SubmitWarrantResult { warrant_proof } => {
            execute_submit_warrant(deps, env, info, warrant_proof)
        }
        ExecuteMsg::RollbackSuspended {
            nullifier_merkle_root,
        } => execute_rollback_suspended(deps, env, info, nullifier_merkle_root),
        ExecuteMsg::AddGatewayAuthority { address } => {
            execute_add_gateway(deps, info, address)
        }
        ExecuteMsg::RemoveGatewayAuthority { address } => {
            execute_remove_gateway(deps, info, address)
        }
    }
}

// ── TransactionProtocol Types ──

pub struct SubmitWarrantPayload {
    pub warrant_proof: WarrantIssuedVP,
    pub sender: cosmwasm_std::Addr,
}

pub struct SubmitWarrantReceipt {
    pub sub_msg: SubMsg,
    pub event: Event,
}

/// WarrantTransactionEngine: TransactionProtocol 구현체
pub struct WarrantTransactionEngine<'a> {
    pub deps: DepsMut<'a>,
    pub env: Env,
}

impl<'a> TransactionProtocol for WarrantTransactionEngine<'a> {
    type Payload = SubmitWarrantPayload;
    type Receipt = SubmitWarrantReceipt;
    type Error = ContractError;

    fn pre_validate(&self, payload: &Self::Payload) -> Result<(), ContractError> {
        // 1. 게이트웨이 권한 검증 (ZTA Actor 인가)
        check_gateway_authority(self.deps.as_ref(), &payload.sender)?;

        let merkle_root = &payload.warrant_proof.nullifier_merkle_root;

        // 2. 중복 제출 방지 (Context 제약 조건 검증)
        if WARRANT_RECORDS.has(self.deps.storage, merkle_root) {
            let existing = WARRANT_RECORDS.load(self.deps.storage, merkle_root)?;
            if existing.status != WarrantStatus::Pending {
                return Err(ContractError::WarrantAlreadyFinalized {
                    merkle_root: merkle_root.clone(),
                });
            }
            return Err(ContractError::DuplicateWarrant {
                merkle_root: merkle_root.clone(),
            });
        }

        Ok(())
    }

    fn execute(&mut self, payload: Self::Payload) -> Result<Self::Receipt, ContractError> {
        let merkle_root = payload.warrant_proof.nullifier_merkle_root.clone();

        // 3. 영장 기록 생성 (Pending 상태)
        let record = WarrantRecord {
            nullifier_merkle_root: merkle_root.clone(),
            survey_id: payload.warrant_proof.survey_id.clone(),
            status: WarrantStatus::Pending,
            submitted_at_height: self.env.block.height,
            finalized_at_height: None,
            kics_receipt_number: payload.warrant_proof.kics_receipt_number,
            reason: payload.warrant_proof.reason.clone(),
        };
        WARRANT_RECORDS.save(self.deps.storage, &merkle_root, &record)?;

        // 4. VP 임시 보관 (Reply 핸들러에서 참조)
        PENDING_WARRANT_VP.save(self.deps.storage, &merkle_root, &payload.warrant_proof)?;

        // 5. 통계 업데이트
        let total = TOTAL_SUBMITTED.load(self.deps.storage).unwrap_or(0) + 1;
        TOTAL_SUBMITTED.save(self.deps.storage, &total)?;

        // 6. judicial-verifier에 서명 검증 위임 (SubMsg)
        let judicial_verifier = JUDICIAL_VERIFIER_CONTRACT.load(self.deps.storage)?;

        #[derive(serde::Serialize)]
        struct VerifyJudicialWarrantHelper {
            verify_judicial_warrant: VerifyJudicialWarrantFields,
        }
        #[derive(serde::Serialize)]
        struct VerifyJudicialWarrantFields {
            vp: WarrantIssuedVP,
            expected_root: String,
        }

        let verify_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: judicial_verifier.to_string(),
            msg: to_json_binary(&VerifyJudicialWarrantHelper {
                verify_judicial_warrant: VerifyJudicialWarrantFields {
                    vp: payload.warrant_proof.clone(),
                    expected_root: merkle_root.clone(),
                },
            })?,
            funds: vec![],
        });

        let payload_bin = cosmwasm_std::Binary::from(merkle_root.as_bytes());
        let sub_msg =
            SubMsg::reply_always(verify_msg, REPLY_JUDICIAL_VERIFY_ID).with_payload(payload_bin);

        // 7. 온체인 이벤트 방출
        let warrant_event = Event::new("hete_warrant_submitted")
            .add_attribute("survey_id", payload.warrant_proof.survey_id)
            .add_attribute("nullifier_root", merkle_root)
            .add_attribute("kics_receipt", payload.warrant_proof.kics_receipt_number.to_string())
            .add_attribute("status", "Pending");

        Ok(SubmitWarrantReceipt { sub_msg, event: warrant_event })
    }

    fn post_reconcile(&self) -> Result<(), ContractError> {
        // 사후 검증: 영장 데이터 적재 상태 및 통계 정합성 확인
        let total_submitted = TOTAL_SUBMITTED.load(self.deps.storage).unwrap_or(0);
        if total_submitted == 0 {
            return Err(ContractError::Std(cosmwasm_std::StdError::generic_err(
                "Post-reconciliation check failed: total_submitted cannot be zero after execution",
            )));
        }
        Ok(())
    }
}

/// 사법 기관 게이트웨이로부터 영장 결과 수신 및 검증 위임
fn execute_submit_warrant(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    warrant_proof: WarrantIssuedVP,
) -> Result<Response, ContractError> {
    let mut engine = WarrantTransactionEngine { deps, env };
    let payload = SubmitWarrantPayload {
        warrant_proof,
        sender: info.sender,
    };
    let is_approved_claim = payload.warrant_proof.is_approved.to_string();

    // 1. 사전 검증
    engine.pre_validate(&payload)?;

    // 2. 실행
    let receipt = engine.execute(payload)?;

    // 3. 사후 정산 검증
    engine.post_reconcile()?;

    Ok(Response::new()
        .add_submessage(receipt.sub_msg)
        .add_event(receipt.event)
        .add_attribute("action", "submit_warrant_result")
        .add_attribute("is_approved_claim", is_approved_claim))
}

/// TTL 만료 시 관리자가 수동 롤백
fn execute_rollback_suspended(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    nullifier_merkle_root: String,
) -> Result<Response, ContractError> {
    let admin = ADMIN.load(deps.storage)?;
    if info.sender != admin {
        return Err(ContractError::Unauthorized {});
    }

    // voter-manager에 롤백 메시지 발송
    let voter_manager = VOTER_MANAGER_CONTRACT.load(deps.storage)?;

    #[derive(serde::Serialize)]
    struct RollbackToActiveHelper {
        rollback_to_active: RollbackToActiveFields,
    }
    #[derive(serde::Serialize)]
    struct RollbackToActiveFields {
        nullifier_merkle_root: String,
    }

    let rollback_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: voter_manager.to_string(),
        msg: to_json_binary(&RollbackToActiveHelper {
            rollback_to_active: RollbackToActiveFields {
                nullifier_merkle_root: nullifier_merkle_root.clone(),
            },
        })?,
        funds: vec![],
    });

    // 영장 기록 업데이트
    if let Ok(mut record) = WARRANT_RECORDS.load(deps.storage, &nullifier_merkle_root) {
        record.status = WarrantStatus::Rejected;
        record.finalized_at_height = Some(env.block.height);
        record.reason = "관리자 수동 롤백 (TTL 만료)".to_string();
        WARRANT_RECORDS.save(deps.storage, &nullifier_merkle_root, &record)?;
    }

    let rollbacks = TOTAL_ROLLBACKS.load(deps.storage).unwrap_or(0) + 1;
    TOTAL_ROLLBACKS.save(deps.storage, &rollbacks)?;

    Ok(Response::new()
        .add_message(rollback_msg)
        .add_attribute("action", "rollback_suspended")
        .add_attribute("nullifier_root", nullifier_merkle_root))
}

/// 게이트웨이 권한 검증
fn check_gateway_authority(
    deps: Deps,
    sender: &cosmwasm_std::Addr,
) -> Result<(), ContractError> {
    let admin = ADMIN.load(deps.storage)?;
    if *sender == admin {
        return Ok(());
    }

    let is_gw = GATEWAY_AUTHORITIES
        .may_load(deps.storage, sender.as_str())?
        .unwrap_or(false);

    if !is_gw {
        return Err(ContractError::UnauthorizedGateway {});
    }
    Ok(())
}

fn execute_add_gateway(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    let admin = ADMIN.load(deps.storage)?;
    if info.sender != admin {
        return Err(ContractError::Unauthorized {});
    }
    GATEWAY_AUTHORITIES.save(deps.storage, &address, &true)?;
    let count = GATEWAY_COUNT.load(deps.storage)? + 1;
    GATEWAY_COUNT.save(deps.storage, &count)?;
    Ok(Response::new()
        .add_attribute("action", "add_gateway_authority")
        .add_attribute("address", address)
        .add_attribute("count", count.to_string()))
}

fn execute_remove_gateway(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    let admin = ADMIN.load(deps.storage)?;
    if info.sender != admin {
        return Err(ContractError::Unauthorized {});
    }
    GATEWAY_AUTHORITIES.remove(deps.storage, &address);
    let count = GATEWAY_COUNT.load(deps.storage)?;
    if count > 0 {
        GATEWAY_COUNT.save(deps.storage, &(count - 1))?;
    }
    Ok(Response::new()
        .add_attribute("action", "remove_gateway_authority")
        .add_attribute("address", address))
}

// ============================================================================
// Reply Handler - 사법 검증 결과 기반 복식 부기 정산 오케스트레이션
// ============================================================================

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        REPLY_JUDICIAL_VERIFY_ID => handle_judicial_verify_reply(deps, env, msg),
        _ => Err(ContractError::UnknownReplyId { id: msg.id }),
    }
}

/// 사법 검증기(judicial-verifier)의 검증 결과를 처리
/// 승인: tally 퍼지 + escrow 몰수 + voter-manager 블랙리스트 확정
/// 기각: voter-manager 롤백
fn handle_judicial_verify_reply(
    deps: DepsMut,
    env: Env,
    msg: Reply,
) -> Result<Response, ContractError> {
    // payload에서 nullifier_merkle_root 복원
    let merkle_root = String::from_utf8(msg.payload.to_vec()).map_err(|_| {
        ContractError::Std(cosmwasm_std::StdError::generic_err("Invalid payload UTF-8"))
    })?;

    match &msg.result {
        cosmwasm_std::SubMsgResult::Ok(sub_msg_response) => {
            // Reply 데이터에서 JudicialVerificationResult 파싱
            let data = sub_msg_response
                .data
                .as_ref()
                .ok_or(ContractError::ReplyDataParseError {})?;
            let verification_result: JudicialVerificationResult =
                cosmwasm_std::from_json(data)
                    .map_err(|_| ContractError::ReplyDataParseError {})?;

            // 보관된 VP 로드
            let warrant_vp = PENDING_WARRANT_VP.load(deps.storage, &merkle_root)?;

            if verification_result.is_approved {
                // ===== 시나리오 A: 영장 최종 승인 =====
                // 영구 몰수 정산 파이프라인 가동
                handle_warrant_approved(deps, env, &merkle_root, &warrant_vp)
            } else {
                // ===== 시나리오 B: 영장 기각 =====
                // 원상 복구 롤백
                handle_warrant_rejected(deps, env, &merkle_root)
            }
        }
        cosmwasm_std::SubMsgResult::Err(err) => {
            // 사법 검증 실패 → 안전하게 Pending 유지 (재시도 가능)
            Ok(Response::new()
                .add_attribute("reply", "judicial_verify_failed")
                .add_attribute("nullifier_root", merkle_root)
                .add_attribute("error", err))
        }
    }
}

/// 영장 승인 시 복식 부기 몰수 정산
fn handle_warrant_approved(
    deps: DepsMut,
    env: Env,
    merkle_root: &str,
    warrant_vp: &WarrantIssuedVP,
) -> Result<Response, ContractError> {
    // 1. 영장 기록 업데이트
    let mut record = WARRANT_RECORDS.load(deps.storage, merkle_root)?;
    record.status = WarrantStatus::Approved;
    record.finalized_at_height = Some(env.block.height);
    WARRANT_RECORDS.save(deps.storage, merkle_root, &record)?;

    // 2. 임시 VP 정리
    PENDING_WARRANT_VP.remove(deps.storage, merkle_root);

    // 3. 통계 업데이트
    let approved = TOTAL_APPROVED.load(deps.storage).unwrap_or(0) + 1;
    TOTAL_APPROVED.save(deps.storage, &approved)?;

    // 4. voter-manager에 PermanentBlacklist 확정 메시지
    let voter_manager = VOTER_MANAGER_CONTRACT.load(deps.storage)?;

    #[derive(serde::Serialize)]
    struct FinalizeLegalForfeitureHelper {
        finalize_legal_forfeiture: FinalizeLegalForfeitureFields,
    }
    #[derive(serde::Serialize)]
    struct FinalizeLegalForfeitureFields {
        nullifier_merkle_root: String,
    }

    let vm_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: voter_manager.to_string(),
        msg: to_json_binary(&FinalizeLegalForfeitureHelper {
            finalize_legal_forfeiture: FinalizeLegalForfeitureFields {
                nullifier_merkle_root: merkle_root.to_string(),
            },
        })?,
        funds: vec![],
    });

    // 5. tally에 PurgeSuspendedVotes 메시지
    let tally_contract = TALLY_CONTRACT.load(deps.storage)?;

    #[derive(serde::Serialize)]
    struct PurgeSuspendedVotesHelper {
        purge_suspended_votes: PurgeSuspendedVotesFields,
    }
    #[derive(serde::Serialize)]
    struct PurgeSuspendedVotesFields {
        nullifier_merkle_root: String,
    }

    let tally_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: tally_contract.to_string(),
        msg: to_json_binary(&PurgeSuspendedVotesHelper {
            purge_suspended_votes: PurgeSuspendedVotesFields {
                nullifier_merkle_root: merkle_root.to_string(),
            },
        })?,
        funds: vec![],
    });

    // 6. escrow에 ForfeitToTreasury 메시지
    let escrow_contract = ESCROW_CONTRACT.load(deps.storage)?;
    let treasury = NATIONAL_TREASURY_ADDRESS.load(deps.storage)?;

    #[derive(serde::Serialize)]
    struct ForfeitToTreasuryHelper {
        forfeit_to_treasury: ForfeitToTreasuryFields,
    }
    #[derive(serde::Serialize)]
    struct ForfeitToTreasuryFields {
        nullifier_merkle_root: String,
        treasury_address: String,
    }

    let escrow_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: escrow_contract.to_string(),
        msg: to_json_binary(&ForfeitToTreasuryHelper {
            forfeit_to_treasury: ForfeitToTreasuryFields {
                nullifier_merkle_root: merkle_root.to_string(),
                treasury_address: treasury,
            },
        })?,
        funds: vec![],
    });

    // 7. 영장 승인 완료 이벤트 방출
    let event = Event::new("hete_warrant_approved")
        .add_attribute("nullifier_root", merkle_root)
        .add_attribute("survey_id", &warrant_vp.survey_id)
        .add_attribute("kics_receipt", warrant_vp.kics_receipt_number.to_string())
        .add_attribute("settlement", "permanent_legal_forfeiture_committed");

    Ok(Response::new()
        .add_messages(vec![vm_msg, tally_msg, escrow_msg])
        .add_event(event)
        .add_attribute("reply", "warrant_approved")
        .add_attribute("nullifier_root", merkle_root))
}

/// 영장 기각 시 원상 복구
fn handle_warrant_rejected(
    deps: DepsMut,
    env: Env,
    merkle_root: &str,
) -> Result<Response, ContractError> {
    // 1. 영장 기록 업데이트
    let mut record = WARRANT_RECORDS.load(deps.storage, merkle_root)?;
    record.status = WarrantStatus::Rejected;
    record.finalized_at_height = Some(env.block.height);
    WARRANT_RECORDS.save(deps.storage, merkle_root, &record)?;

    // 2. 임시 VP 정리
    PENDING_WARRANT_VP.remove(deps.storage, merkle_root);

    // 3. 통계 업데이트
    let rejected = TOTAL_REJECTED.load(deps.storage).unwrap_or(0) + 1;
    TOTAL_REJECTED.save(deps.storage, &rejected)?;

    // 4. voter-manager에 롤백 메시지 발송
    let voter_manager = VOTER_MANAGER_CONTRACT.load(deps.storage)?;

    #[derive(serde::Serialize)]
    struct RollbackToActiveHelper {
        rollback_to_active: RollbackToActiveFields,
    }
    #[derive(serde::Serialize)]
    struct RollbackToActiveFields {
        nullifier_merkle_root: String,
    }

    let rollback_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: voter_manager.to_string(),
        msg: to_json_binary(&RollbackToActiveHelper {
            rollback_to_active: RollbackToActiveFields {
                nullifier_merkle_root: merkle_root.to_string(),
            },
        })?,
        funds: vec![],
    });

    // 5. 영장 기각 이벤트 방출
    let event = Event::new("hete_warrant_rejected")
        .add_attribute("nullifier_root", merkle_root)
        .add_attribute("settlement", "legal_warrant_rejected_and_rolled_back");

    Ok(Response::new()
        .add_message(rollback_msg)
        .add_event(event)
        .add_attribute("reply", "warrant_rejected")
        .add_attribute("nullifier_root", merkle_root))
}

// ============================================================================
// Query
// ============================================================================

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetWarrantStatus {
            nullifier_merkle_root,
        } => to_json_binary(&query_warrant_status(deps, nullifier_merkle_root)?),
        QueryMsg::GetConfig {} => to_json_binary(&query_config(deps)?),
        QueryMsg::GetStats {} => to_json_binary(&query_stats(deps)?),
    }
}

fn query_warrant_status(
    deps: Deps,
    nullifier_merkle_root: String,
) -> StdResult<WarrantStatusResponse> {
    let record = WARRANT_RECORDS.may_load(deps.storage, &nullifier_merkle_root)?;
    Ok(WarrantStatusResponse { record })
}

fn query_config(deps: Deps) -> StdResult<WarrantHandlerConfigResponse> {
    let admin = ADMIN.load(deps.storage)?;
    let jv = JUDICIAL_VERIFIER_CONTRACT.load(deps.storage)?;
    let vm = VOTER_MANAGER_CONTRACT.load(deps.storage)?;
    let tc = TALLY_CONTRACT.load(deps.storage)?;
    let ec = ESCROW_CONTRACT.load(deps.storage)?;
    let treasury = NATIONAL_TREASURY_ADDRESS.load(deps.storage)?;
    let gw_count = GATEWAY_COUNT.load(deps.storage)?;

    Ok(WarrantHandlerConfigResponse {
        admin: admin.to_string(),
        judicial_verifier_contract: jv.to_string(),
        voter_manager_contract: vm.to_string(),
        tally_contract: tc.to_string(),
        escrow_contract: ec.to_string(),
        national_treasury_address: treasury,
        gateway_count: gw_count,
    })
}

fn query_stats(deps: Deps) -> StdResult<WarrantStatsResponse> {
    Ok(WarrantStatsResponse {
        total_submitted: TOTAL_SUBMITTED.load(deps.storage).unwrap_or(0),
        total_approved: TOTAL_APPROVED.load(deps.storage).unwrap_or(0),
        total_rejected: TOTAL_REJECTED.load(deps.storage).unwrap_or(0),
        total_rollbacks: TOTAL_ROLLBACKS.load(deps.storage).unwrap_or(0),
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use voting_common::types::WarrantIssuedVP;

    fn setup_contract(
        deps: &mut cosmwasm_std::OwnedDeps<
            cosmwasm_std::MemoryStorage,
            cosmwasm_std::testing::MockApi,
            cosmwasm_std::testing::MockQuerier,
        >,
    ) -> String {
        let admin_addr = deps.api.addr_make("admin");
        let info = mock_info(admin_addr.as_str(), &[]);
        let msg = InstantiateMsg {
            judicial_verifier_contract: deps.api.addr_make("judicial_verifier").to_string(),
            voter_manager_contract: deps.api.addr_make("voter_manager").to_string(),
            tally_contract: deps.api.addr_make("tally_addr").to_string(),
            escrow_contract: deps.api.addr_make("escrow_addr").to_string(),
            national_treasury_address: "national_treasury_001".to_string(),
            authorized_gateway_addresses: vec![
                deps.api.addr_make("kics_gateway").to_string(),
            ],
        };
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        admin_addr.to_string()
    }

    fn default_warrant_vp() -> WarrantIssuedVP {
        WarrantIssuedVP {
            survey_id: "election_2026_001".to_string(),
            nullifier_merkle_root: "abcdef1234567890".to_string(),
            judicial_signature: "valid_judicial_sig".to_string(),
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
        let admin = setup_contract(&mut deps);

        let config: WarrantHandlerConfigResponse = cosmwasm_std::from_json(
            query(deps.as_ref(), mock_env(), QueryMsg::GetConfig {}).unwrap(),
        )
        .unwrap();
        assert_eq!(config.admin, admin);
        assert_eq!(config.gateway_count, 1);
        assert_eq!(config.national_treasury_address, "national_treasury_001");
    }

    #[test]
    fn test_submit_warrant_creates_submsg() {
        let mut deps = mock_dependencies();
        setup_contract(&mut deps);

        let vp = default_warrant_vp();
        let gw_addr = deps.api.addr_make("kics_gateway");
        let gw_info = mock_info(gw_addr.as_str(), &[]);

        let res = execute(
            deps.as_mut(),
            mock_env(),
            gw_info,
            ExecuteMsg::SubmitWarrantResult {
                warrant_proof: vp.clone(),
            },
        )
        .unwrap();

        // SubMsg가 1개 생성 (judicial-verifier 검증 위임)
        assert_eq!(res.messages.len(), 1);
        assert_eq!(res.messages[0].id, REPLY_JUDICIAL_VERIFY_ID);

        // 영장 기록 확인
        let status: WarrantStatusResponse = cosmwasm_std::from_json(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::GetWarrantStatus {
                    nullifier_merkle_root: vp.nullifier_merkle_root,
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert!(status.record.is_some());
        assert_eq!(status.record.unwrap().status, WarrantStatus::Pending);

        // 통계 확인
        let stats: WarrantStatsResponse = cosmwasm_std::from_json(
            query(deps.as_ref(), mock_env(), QueryMsg::GetStats {}).unwrap(),
        )
        .unwrap();
        assert_eq!(stats.total_submitted, 1);
    }

    #[test]
    fn test_unauthorized_gateway_rejected() {
        let mut deps = mock_dependencies();
        setup_contract(&mut deps);

        let hacker_addr = deps.api.addr_make("hacker");
        let hacker_info = mock_info(hacker_addr.as_str(), &[]);

        let err = execute(
            deps.as_mut(),
            mock_env(),
            hacker_info,
            ExecuteMsg::SubmitWarrantResult {
                warrant_proof: default_warrant_vp(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::UnauthorizedGateway {}));
    }

    #[test]
    fn test_duplicate_warrant_rejected() {
        let mut deps = mock_dependencies();
        setup_contract(&mut deps);

        let gw_addr = deps.api.addr_make("kics_gateway");
        let gw_info = mock_info(gw_addr.as_str(), &[]);

        // 첫 번째 제출 성공
        execute(
            deps.as_mut(),
            mock_env(),
            gw_info.clone(),
            ExecuteMsg::SubmitWarrantResult {
                warrant_proof: default_warrant_vp(),
            },
        )
        .unwrap();

        // 중복 제출 거부
        let err = execute(
            deps.as_mut(),
            mock_env(),
            gw_info,
            ExecuteMsg::SubmitWarrantResult {
                warrant_proof: default_warrant_vp(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::DuplicateWarrant { .. }));
    }

    #[test]
    fn test_rollback_suspended() {
        let mut deps = mock_dependencies();
        let admin = setup_contract(&mut deps);
        let admin_info = mock_info(&admin, &[]);

        // 관리자가 롤백 실행
        let res = execute(
            deps.as_mut(),
            mock_env(),
            admin_info,
            ExecuteMsg::RollbackSuspended {
                nullifier_merkle_root: "some_root".to_string(),
            },
        )
        .unwrap();

        // voter-manager에 롤백 메시지가 발송되었는지 확인
        assert_eq!(res.messages.len(), 1);

        // 통계 확인
        let stats: WarrantStatsResponse = cosmwasm_std::from_json(
            query(deps.as_ref(), mock_env(), QueryMsg::GetStats {}).unwrap(),
        )
        .unwrap();
        assert_eq!(stats.total_rollbacks, 1);
    }

    #[test]
    fn test_unauthorized_rollback_rejected() {
        let mut deps = mock_dependencies();
        setup_contract(&mut deps);

        let hacker_addr = deps.api.addr_make("hacker");
        let hacker_info = mock_info(hacker_addr.as_str(), &[]);

        let err = execute(
            deps.as_mut(),
            mock_env(),
            hacker_info,
            ExecuteMsg::RollbackSuspended {
                nullifier_merkle_root: "some_root".to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized {}));
    }

    #[test]
    fn test_gateway_management() {
        let mut deps = mock_dependencies();
        let admin = setup_contract(&mut deps);
        let admin_info = mock_info(&admin, &[]);

        // 게이트웨이 추가
        execute(
            deps.as_mut(),
            mock_env(),
            admin_info.clone(),
            ExecuteMsg::AddGatewayAuthority {
                address: "new_gateway".to_string(),
            },
        )
        .unwrap();

        let config: WarrantHandlerConfigResponse = cosmwasm_std::from_json(
            query(deps.as_ref(), mock_env(), QueryMsg::GetConfig {}).unwrap(),
        )
        .unwrap();
        assert_eq!(config.gateway_count, 2);

        // 게이트웨이 제거
        execute(
            deps.as_mut(),
            mock_env(),
            admin_info,
            ExecuteMsg::RemoveGatewayAuthority {
                address: "new_gateway".to_string(),
            },
        )
        .unwrap();

        let config: WarrantHandlerConfigResponse = cosmwasm_std::from_json(
            query(deps.as_ref(), mock_env(), QueryMsg::GetConfig {}).unwrap(),
        )
        .unwrap();
        assert_eq!(config.gateway_count, 1);
    }

    #[test]
    fn test_reply_judicial_verify_approved() {
        let mut deps = mock_dependencies();
        setup_contract(&mut deps);

        let vp = default_warrant_vp();
        let merkle_root = vp.nullifier_merkle_root.clone();

        // 1. Submit warrant first to store in pending
        let gw_addr = deps.api.addr_make("kics_gateway");
        let gw_info = mock_info(gw_addr.as_str(), &[]);
        execute(
            deps.as_mut(),
            mock_env(),
            gw_info,
            ExecuteMsg::SubmitWarrantResult {
                warrant_proof: vp.clone(),
            },
        )
        .unwrap();

        // 2. Call reply with success and is_approved: true
        let result = JudicialVerificationResult {
            nullifier_merkle_root: merkle_root.clone(),
            is_signature_valid: true,
            is_approved: true,
        };
        let reply_msg = Reply {
            id: REPLY_JUDICIAL_VERIFY_ID,
            result: cosmwasm_std::SubMsgResult::Ok(cosmwasm_std::SubMsgResponse {
                events: vec![],
                data: Some(to_json_binary(&result).unwrap()),
                msg_responses: vec![],
            }),
            payload: Binary::from(merkle_root.as_bytes()),
            gas_used: 0,
        };

        let res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap();

        // Should emit approved event and generate 3 submessages
        assert_eq!(res.messages.len(), 3);
        assert_eq!(res.events.len(), 1);
        assert_eq!(res.events[0].ty, "hete_warrant_approved");

        // Verify state is Approved
        let status: WarrantStatusResponse = cosmwasm_std::from_json(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::GetWarrantStatus {
                    nullifier_merkle_root: merkle_root,
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(status.record.unwrap().status, WarrantStatus::Approved);
    }

    #[test]
    fn test_reply_judicial_verify_rejected() {
        let mut deps = mock_dependencies();
        setup_contract(&mut deps);

        let vp = default_warrant_vp();
        let merkle_root = vp.nullifier_merkle_root.clone();

        // 1. Submit warrant first
        let gw_addr = deps.api.addr_make("kics_gateway");
        let gw_info = mock_info(gw_addr.as_str(), &[]);
        execute(
            deps.as_mut(),
            mock_env(),
            gw_info,
            ExecuteMsg::SubmitWarrantResult {
                warrant_proof: vp.clone(),
            },
        )
        .unwrap();

        // 2. Call reply with success and is_approved: false
        let result = JudicialVerificationResult {
            nullifier_merkle_root: merkle_root.clone(),
            is_signature_valid: true,
            is_approved: false,
        };
        let reply_msg = Reply {
            id: REPLY_JUDICIAL_VERIFY_ID,
            result: cosmwasm_std::SubMsgResult::Ok(cosmwasm_std::SubMsgResponse {
                events: vec![],
                data: Some(to_json_binary(&result).unwrap()),
                msg_responses: vec![],
            }),
            payload: Binary::from(merkle_root.as_bytes()),
            gas_used: 0,
        };

        let res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap();

        // Should emit rejected event and generate 1 submessage (rollback)
        assert_eq!(res.messages.len(), 1);
        assert_eq!(res.events.len(), 1);
        assert_eq!(res.events[0].ty, "hete_warrant_rejected");

        // Verify state is Rejected
        let status: WarrantStatusResponse = cosmwasm_std::from_json(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::GetWarrantStatus {
                    nullifier_merkle_root: merkle_root,
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(status.record.unwrap().status, WarrantStatus::Rejected);
    }
}
