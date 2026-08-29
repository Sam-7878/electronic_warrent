use std::{collections::BTreeMap, env, fs, io::Write, path::PathBuf};

use domain_electronic_warrant::FormalTraceState;

fn state(name: &str, action: &str) -> FormalTraceState {
    let credentials_verified = !matches!(name, "Draft" | "Submitted" | "Rejected");
    let authorized = matches!(
        name,
        "Authorized"
            | "Active"
            | "PartiallyExecuted"
            | "FullyExecuted"
            | "Suspended"
            | "Revoked"
            | "Expired"
            | "Released"
    );
    FormalTraceState {
        warrant_state: name.into(),
        credentials_verified,
        authorized,
        domain_valid: true,
        nonce_used: authorized,
        activation_count: 0,
        reserved: 0,
        executed: 0,
        released: 0,
        now: 0,
        adapter_committed: false,
        audit_written: matches!(
            name,
            "Active"
                | "PartiallyExecuted"
                | "FullyExecuted"
                | "Suspended"
                | "Revoked"
                | "Expired"
                | "Released"
                | "Rejected"
                | "Failed"
        ),
        last_action: action.into(),
    }
}

fn prelude() -> Vec<FormalTraceState> {
    vec![
        state("Draft", "Init"),
        state("Submitted", "Submit"),
        state("CredentialVerified", "VerifyCredentials"),
        state("Authorized", "Authorize"),
    ]
}

fn activated() -> Vec<FormalTraceState> {
    let mut trace = prelude();
    let mut active = state("Active", "Activate");
    active.activation_count = 1;
    active.reserved = 3;
    active.adapter_committed = true;
    trace.push(active);
    trace
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("formal/traces/rust"));
    fs::create_dir_all(&output)?;

    let mut traces: BTreeMap<&str, Vec<FormalTraceState>> = BTreeMap::new();
    traces.insert("01_authorized_activation", activated());

    let mut partial = activated();
    let mut p = partial.last().cloned().unwrap();
    p.warrant_state = "PartiallyExecuted".into();
    p.executed = 1;
    p.last_action = "Execute".into();
    partial.push(p);
    traces.insert("02_partial_execution", partial);

    let mut full = activated();
    let mut p = full.last().cloned().unwrap();
    p.warrant_state = "FullyExecuted".into();
    p.executed = 3;
    p.last_action = "Execute".into();
    full.push(p);
    traces.insert("03_full_execution", full);

    let mut release = activated();
    let mut p = release.last().cloned().unwrap();
    p.warrant_state = "Released".into();
    p.released = 3;
    p.last_action = "Release".into();
    release.push(p);
    traces.insert("04_release", release);

    let mut expiry = activated();
    let mut tick = expiry.last().cloned().unwrap();
    for now in 1..=3 {
        tick.now = now;
        tick.last_action = "Tick".into();
        expiry.push(tick.clone());
    }
    tick.warrant_state = "Expired".into();
    tick.released = 3;
    tick.last_action = "Expire".into();
    expiry.push(tick);
    traces.insert("05_expiry", expiry);

    let mut revoke = activated();
    let mut p = revoke.last().cloned().unwrap();
    p.warrant_state = "Revoked".into();
    p.released = 3;
    p.last_action = "Revoke".into();
    revoke.push(p);
    traces.insert("06_revocation", revoke);

    let mut reject = vec![state("Draft", "Init"), state("Submitted", "Submit")];
    reject.push(state("Rejected", "RejectUnauthorized"));
    traces.insert("07_unauthorized_rejection", reject);

    let mut quarantine = prelude();
    quarantine.push(state("Suspended", "Quarantine"));
    let mut reviewed = state("Rejected", "ReviewQuarantine");
    reviewed.credentials_verified = true;
    reviewed.authorized = true;
    reviewed.nonce_used = true;
    quarantine.push(reviewed);
    traces.insert("08_quarantine_review", quarantine);

    let mut abort = prelude();
    abort.push(state("Failed", "Abort"));
    traces.insert("09_adapter_abort", abort);

    let mut suspended = activated();
    let mut p = suspended.last().cloned().unwrap();
    p.warrant_state = "Suspended".into();
    p.last_action = "Suspend".into();
    suspended.push(p.clone());
    p.warrant_state = "Revoked".into();
    p.released = 3;
    p.last_action = "Revoke".into();
    suspended.push(p);
    traces.insert("10_suspend_then_revoke", suspended);

    let mut replay = activated();
    replay.push(replay.last().cloned().unwrap());
    traces.insert("11_terminal_or_duplicate_stutter", replay);

    for (name, trace) in traces {
        let mut file = fs::File::create(output.join(format!("{name}.jsonl")))?;
        for snapshot in trace {
            serde_json::to_writer(&mut file, &snapshot)?;
            writeln!(file)?;
        }
    }
    Ok(())
}
