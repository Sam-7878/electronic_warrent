//! Offline electronic-warrant validation CLI.
//!
//! The command never resolves network identities or logs credential payloads.

use std::{env, fs};

use anyhow::{Context, Result, bail};
use domain_electronic_warrant::AssetFreezingWarrant;
use hete_policy::EnforcementAction;
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let arguments: Vec<_> = env::args().collect();
    if arguments.len() != 4 || arguments[1] != "validate" {
        bail!("usage: hete-warrant-verifier validate <warrant.json> <unix-time>");
    }
    let path = &arguments[2];
    let now = arguments[3]
        .parse::<u64>()
        .context("time must be an unsigned integer")?;
    let bytes = fs::read(path).context("unable to read warrant input")?;
    let warrant: AssetFreezingWarrant =
        serde_json::from_slice(&bytes).context("POLICY_SCHEMA_INVALID")?;
    warrant.validate(now).context("POLICY_DIGEST_MISMATCH")?;
    let message = warrant
        .signature_message(&EnforcementAction::Freeze)
        .context("POLICY_SCHEMA_INVALID")?;
    println!(
        "{}",
        serde_json::json!({
            "status": "valid",
            "policy_digest": warrant.common.policy_digest,
            "signature_message_digest": format!("sha256:{}", hex::encode(Sha256::digest(message))),
        })
    );
    Ok(())
}
