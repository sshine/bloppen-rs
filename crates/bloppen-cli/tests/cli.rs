//! The CLI surface and the destructive-operation guard.
#![allow(clippy::expect_used)]

use std::process::Command;

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_bloppen-cli"))
}

#[test]
fn help_lists_the_product_and_shop_groups() {
    let output = cli().arg("--help").output().expect("the binary runs");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("product"), "{stdout}");
    assert!(stdout.contains("shop"), "{stdout}");
}

#[test]
fn product_help_lists_the_write_commands() {
    let output = cli()
        .args(["product", "--help"])
        .output()
        .expect("the binary runs");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for command in [
        "add", "active", "inactive", "delete", "move", "discount", "note", "print", "unprint",
        "upload", "list",
    ] {
        assert!(stdout.contains(command), "missing {command} in:\n{stdout}");
    }
}

#[test]
fn deleting_without_yes_is_refused_before_a_request() {
    let output = cli()
        .args([
            "product",
            "delete",
            "--booking-id",
            "1",
            "--product-id",
            "2",
        ])
        .output()
        .expect("the binary runs");
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--yes"), "{stderr}");
}

#[test]
fn product_id_operations_refuse_unknown_products_by_default() {
    let session = std::env::temp_dir().join(format!(
        "bloppen-cli-guard-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default()
    ));
    let _ = std::fs::remove_file(&session);
    let _ = std::fs::remove_file(session.with_file_name("created-products.json"));

    let output = cli()
        .arg("--session-file")
        .arg(&session)
        .args([
            "product",
            "active",
            "--booking-id",
            "1",
            "--product-id",
            "2",
        ])
        .output()
        .expect("the binary runs");
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("did not create") || stderr.contains("allow-existing-products"),
        "{stderr}"
    );
}
