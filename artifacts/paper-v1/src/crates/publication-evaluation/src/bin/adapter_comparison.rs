use std::{
    env,
    fs::File,
    io::{BufWriter, Write},
    time::Instant,
};

use adapter_simulated_asset::SimulatedAssetAdapter;
use adapter_sqlite_asset::SqliteAssetAdapter;
use hete_adapter_api::{EnforcementAdapter, EnforcementCommand};
use hete_policy::{EnforcementAction, PseudonymousTargetRef, ResourceKind, ResourceRef};

fn resource(id: &str) -> ResourceRef {
    ResourceRef {
        resource_id: id.into(),
        kind: ResourceKind::Account,
        target: PseudonymousTargetRef::derive("adapter-benchmark", "subject", id, "w", b"salt"),
    }
}

fn command(id: &str, resource: ResourceRef) -> EnforcementCommand {
    EnforcementCommand {
        command_id: format!("c-{id}"),
        warrant_id: format!("w-{id}"),
        resource,
        action: EnforcementAction::Freeze,
        amount: 50,
        effective_at: 10,
        expires_at: 20,
        expected_version: 0,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 6 {
        return Err("usage: adapter_comparison RUNS OPS OUTPUT SOURCE_COMMIT HOST_ID".into());
    }
    let runs: usize = args[1].parse()?;
    let operations: usize = args[2].parse()?;
    let mut writer = BufWriter::new(File::create(&args[3])?);
    writeln!(
        writer,
        "run_id,operation_id,adapter_id,t_prepare_ns,t_commit_ns,t_total_ns,status,source_commit,host_id"
    )?;
    for run in 1..=runs {
        let mut simulated = SimulatedAssetAdapter::new(10);
        let mut sqlite = SqliteAssetAdapter::in_memory(10)?;
        for operation in 0..operations {
            for adapter_name in ["simulated", "sqlite"] {
                let id = format!("{run}-{operation}-{adapter_name}");
                let resource = resource(&id);
                let command = command(&id, resource.clone());
                let total = Instant::now();
                let (prepare_ns, commit_ns) = if adapter_name == "simulated" {
                    simulated.register_resource(resource, 100);
                    let started = Instant::now();
                    let prepared = simulated.prepare(&command)?;
                    let prepare = started.elapsed().as_nanos();
                    let started = Instant::now();
                    simulated.commit(prepared)?;
                    (prepare, started.elapsed().as_nanos())
                } else {
                    sqlite.register_resource(resource, 100)?;
                    let started = Instant::now();
                    let prepared = sqlite.prepare(&command)?;
                    let prepare = started.elapsed().as_nanos();
                    let started = Instant::now();
                    sqlite.commit(prepared)?;
                    (prepare, started.elapsed().as_nanos())
                };
                writeln!(
                    writer,
                    "adapter-{run:03},{id},{adapter_name},{prepare_ns},{commit_ns},{},success,{},{}",
                    total.elapsed().as_nanos(),
                    args[4],
                    args[5]
                )?;
            }
        }
    }
    writer.flush()?;
    Ok(())
}
