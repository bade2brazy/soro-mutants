use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn unique_project() -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("soro-async-test-{}-{nonce}", std::process::id()));
    fs::create_dir_all(root.join("src")).unwrap();
    root
}

#[test]
fn discovers_mutants_in_async_functions_without_corrupting_syntax() {
    let root = unique_project();
    let src = root.join("src/lib.rs");
    fs::write(
        &src,
        r#"
pub struct AsyncHandler;

impl AsyncHandler {
    pub async fn handle_request(client: Address) {
        client.require_auth();
    }
}

pub async fn async_transfer(user: Address, recipient: Address) {
    user.require_auth();
}

pub async fn async_check(admin: Address) {
    admin.require_auth();
}
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_cargo-soro-mutants"))
        .args(["list", root.to_str().unwrap(), "--json"])
        .output()
        .expect("CLI should run");
    assert!(output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let mutants = report["mutants"].as_array().unwrap();
    assert_eq!(mutants.len(), 4);
    let observed: Vec<_> = mutants
        .iter()
        .map(|mutant| {
            (
                mutant["function"].as_str().unwrap(),
                mutant["operator"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        observed,
        vec![
            ("handle_request", "AUTH-001"),
            ("async_transfer", "AUTH-001"),
            ("async_transfer", "AUTH-002"),
            ("async_check", "AUTH-001"),
        ]
    );

    let _ = fs::remove_dir_all(&root);
}
