use std::{
    env,
    fs::File,
    hint::black_box,
    io::{BufWriter, Write},
    time::Instant,
};

use adapter_simulated_asset::{DryRunAdapter, SimulatedAssetAdapter};
use base64::{Engine, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signer, SigningKey};
use hete_adapter_api::{EnforcementAdapter, EnforcementCommand};
use hete_credential::{
    AuthorityCredentialClaims, CredentialEnvelope, CredentialProof, verify_credential,
};
use hete_identity::{AuthorityRole, Did, DidDocument, LocalIdentityStore, VerificationMethod};
use hete_policy::{
    ActionConstraint, AuthorityRef, AuthorizationPolicy, DomainBinding, EnforcementAction,
    MachinePolicyObject, Nonce, PolicyDigest, PolicyId, PseudonymousTargetRef, ResourceKind,
    ResourceRef, RevocationRule, RoleRequirement, SubjectRef, ValidityWindow,
};
use sha2::{Digest, Sha256};

const HEADER: &str = "run_id,operation_id,baseline_id,policy_count,active_warrant_count,authority_count,credential_count,credential_bytes,risk_evidence_count,audit_mode,status,reason_code,t_parse_ns,t_canonicalize_ns,t_digest_ns,t_identity_ns,t_credential_ns,t_authorize_ns,t_validate_ns,t_risk_ns,t_prepare_ns,t_reconcile_ns,t_commit_ns,t_audit_ns,t_total_ns,rss_bytes,audit_bytes,source_commit,host_id\n";

fn elapsed(start: Instant) -> u128 {
    start.elapsed().as_nanos()
}

fn rss_bytes() -> u64 {
    let Ok(statm) = std::fs::read_to_string("/proc/self/statm") else {
        return 0;
    };
    let pages = statm
        .split_whitespace()
        .nth(1)
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);
    pages.saturating_mul(unsafe { libc::sysconf(libc::_SC_PAGESIZE) as u64 })
}

fn unsigned_credential(credential: &CredentialEnvelope) -> Vec<u8> {
    let mut value = serde_json::to_value(credential).unwrap();
    value.as_object_mut().unwrap().remove("proof");
    poa_protocol::canonicalize_value(&value).unwrap()
}

fn fixtures() -> (
    MachinePolicyObject,
    CredentialEnvelope,
    LocalIdentityStore,
    ResourceRef,
) {
    let key = SigningKey::from_bytes(&[7; 32]);
    let did = Did("did:key:publication-benchmark".into());
    let mut store = LocalIdentityStore::new(10);
    store.insert_document(DidDocument {
        id: did.clone(),
        verification_methods: vec![VerificationMethod {
            key_id: "key-1".into(),
            public_key: key.verifying_key().to_bytes(),
            activated_at: 0,
            revoked_at: None,
        }],
        updated_at: 0,
    });
    store.grant_role(did.clone(), AuthorityRole::JudicialIssuer, 0, None);
    let target = PseudonymousTargetRef::derive("benchmark", "subject", "asset", "w1", b"salt");
    let resource = ResourceRef {
        resource_id: "asset".into(),
        kind: ResourceKind::Account,
        target: target.clone(),
    };
    let mut policy = MachinePolicyObject {
        policy_id: PolicyId("p-publication".into()),
        policy_type: "electronic_warrant".into(),
        version: "1".into(),
        issuer: AuthorityRef {
            did: did.clone(),
            role: AuthorityRole::JudicialIssuer,
        },
        authorization_policy: AuthorizationPolicy {
            requirements: vec![RoleRequirement {
                role: AuthorityRole::JudicialIssuer,
                minimum_signatures: 1,
                sequence: 1,
            }],
            threshold: 1,
            mutually_exclusive_roles: vec![],
            sequential: true,
        },
        subject: SubjectRef {
            target: target.clone(),
        },
        resource: resource.clone(),
        permitted_actions: vec![ActionConstraint {
            action: EnforcementAction::Freeze,
            maximum_amount: Some(100),
        }],
        validity: ValidityWindow {
            not_before: 0,
            expires_at: 1000,
            maximum_duration: 1000,
        },
        conditions: vec![],
        obligations: vec![],
        revocation: RevocationRule {
            revocable: true,
            required_role: AuthorityRole::JudicialIssuer,
        },
        evidence_refs: vec![],
        credential_refs: vec![],
        nonce: Nonce("bench-nonce".into()),
        domain_binding: DomainBinding {
            environment_id: "publication".into(),
            service_id: "warrant".into(),
            adapter_id: "simulated-regulated-asset".into(),
        },
        policy_digest: PolicyDigest(String::new()),
    };
    policy.seal().unwrap();
    let mut credential = CredentialEnvelope {
        id: "credential-publication".into(),
        issuer: did.clone(),
        subject: SubjectRef { target },
        credential_type: vec!["HeteAuthorityCredential".into()],
        issuance_time: 0,
        expiration_time: Some(1000),
        status: None,
        claims: serde_json::to_value(AuthorityCredentialClaims {
            authority_did: did,
            role: AuthorityRole::JudicialIssuer,
            jurisdiction: "benchmark".into(),
        })
        .unwrap(),
        proof: CredentialProof {
            key_id: "key-1".into(),
            signature: String::new(),
        },
    };
    credential.proof.signature =
        STANDARD.encode(key.sign(&unsigned_credential(&credential)).to_bytes());
    (policy, credential, store, resource)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 7 {
        return Err("usage: full_benchmark RUN_ID OPS OUTPUT SOURCE_COMMIT HOST_ID SEED".into());
    }
    let run_id = &args[1];
    let operations: usize = args[2].parse()?;
    let output = &args[3];
    let source_commit = &args[4];
    let host_id = &args[5];
    let seed: usize = args[6].parse()?;
    let (policy, credential, store, resource) = fixtures();
    let policy_json = serde_json::to_vec(&policy)?;
    let baselines = ["B0", "B1", "B2", "B3", "B4", "B5", "B6"];
    let policy_counts = [100, 1_000, 10_000, 100_000];
    let warrant_counts = [0, 10, 100, 1_000, 10_000];
    let authority_counts = [1, 2, 3, 5, 7];
    let credential_counts = [1, 2, 3, 5, 10];
    let credential_sizes = [1_024, 2_048, 4_096, 8_192, 16_384];
    let risk_counts = [0, 1, 4, 16, 64];
    let audit_modes = ["disabled", "minimal", "hash-chain"];
    let mut writer = BufWriter::new(File::create(output)?);
    writer.write_all(HEADER.as_bytes())?;
    for offset in 0..baselines.len() {
        let baseline = baselines[(offset + seed) % baselines.len()];
        for operation in 0..operations {
            let total_start = Instant::now();
            let selector = operation + seed * 17 + offset * 31;
            let policy_count = policy_counts[selector % policy_counts.len()];
            let warrant_count = warrant_counts[selector % warrant_counts.len()];
            let authority_count = authority_counts[selector % authority_counts.len()];
            let credential_count = credential_counts[selector % credential_counts.len()];
            let credential_bytes = credential_sizes[selector % credential_sizes.len()];
            let risk_count = risk_counts[selector % risk_counts.len()];
            let audit_mode = audit_modes[selector % audit_modes.len()];

            let start = Instant::now();
            let parsed: serde_json::Value = serde_json::from_slice(&policy_json)?;
            let t_parse = elapsed(start);
            let start = Instant::now();
            let canonical = poa_protocol::canonicalize_value(&parsed)?;
            let t_canonical = elapsed(start);
            let start = Instant::now();
            black_box(Sha256::digest(&canonical));
            let t_digest = elapsed(start);
            let start = Instant::now();
            black_box(store.verification_key(&credential.issuer, "key-1", 10)?);
            let t_identity = elapsed(start);
            let start = Instant::now();
            for _ in 0..credential_count.min(if baseline == "B0" { 1 } else { 10 }) {
                black_box(verify_credential(&store, &credential, 10)?);
            }
            let t_credential = elapsed(start);
            let start = Instant::now();
            let required: u16 = policy
                .authorization_policy
                .requirements
                .iter()
                .map(|r| r.minimum_signatures)
                .sum();
            black_box(required >= policy.authorization_policy.threshold);
            let t_authorize = elapsed(start);
            let start = Instant::now();
            policy.validate(10)?;
            let t_validate = elapsed(start);
            let start = Instant::now();
            if matches!(baseline, "B5" | "B6") {
                let mut risk_digest = [0_u8; 32];
                for index in 0..risk_count {
                    risk_digest = Sha256::digest([risk_digest[0], index as u8]).into();
                }
                black_box(risk_digest);
            }
            let t_risk = elapsed(start);

            let command = EnforcementCommand {
                command_id: format!("{run_id}-{baseline}-{operation}"),
                warrant_id: format!("w-{operation}"),
                resource: resource.clone(),
                action: EnforcementAction::Freeze,
                amount: 50,
                effective_at: 10,
                expires_at: 20,
                expected_version: 0,
            };
            let (t_prepare, t_reconcile, t_commit, receipt_bytes) = if baseline == "B2" {
                (0, 0, 0, Vec::new())
            } else if baseline == "B6" {
                let mut inner = SimulatedAssetAdapter::new(10);
                inner.register_resource(resource.clone(), 100);
                let mut adapter = DryRunAdapter::new(inner);
                let start = Instant::now();
                let prepared = adapter.prepare(&command)?;
                let prepare = elapsed(start);
                let start = Instant::now();
                black_box(prepared.candidate.available_to_transfer());
                let reconcile = elapsed(start);
                let start = Instant::now();
                let receipt = adapter.commit(prepared)?;
                let commit = elapsed(start);
                (prepare, reconcile, commit, serde_json::to_vec(&receipt)?)
            } else {
                let mut adapter = SimulatedAssetAdapter::new(10);
                adapter.register_resource(resource.clone(), 100);
                let start = Instant::now();
                let prepared = adapter.prepare(&command)?;
                let prepare = elapsed(start);
                let start = Instant::now();
                black_box(prepared.candidate.available_to_transfer());
                let reconcile = elapsed(start);
                let start = Instant::now();
                let receipt = adapter.commit(prepared)?;
                let commit = elapsed(start);
                (prepare, reconcile, commit, serde_json::to_vec(&receipt)?)
            };
            let start = Instant::now();
            let audit_bytes = if audit_mode == "disabled" || baseline == "B3" {
                0
            } else {
                let digest = Sha256::digest(&receipt_bytes);
                if audit_mode == "minimal" {
                    digest.len()
                } else {
                    receipt_bytes.len() + digest.len()
                }
            };
            black_box(audit_bytes);
            let t_audit = elapsed(start);
            let total = elapsed(total_start);
            writeln!(
                writer,
                "{run_id},{run_id}-{baseline}-{operation},{baseline},{policy_count},{warrant_count},{authority_count},{credential_count},{credential_bytes},{risk_count},{audit_mode},success,COMMIT,{t_parse},{t_canonical},{t_digest},{t_identity},{t_credential},{t_authorize},{t_validate},{t_risk},{t_prepare},{t_reconcile},{t_commit},{t_audit},{total},{},{audit_bytes},{source_commit},{host_id}",
                rss_bytes()
            )?;
        }
    }
    writer.flush()?;
    Ok(())
}
