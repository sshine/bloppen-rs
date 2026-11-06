//! The rules the tool surface has to keep.
#![allow(clippy::expect_used)]

mod common;

use bloppen_mcp::server::BloppenServer;

#[test]
fn every_tool_states_its_annotations() {
    for tool in BloppenServer::router().list_all() {
        let name = &tool.name;
        let annotations = tool
            .annotations
            .unwrap_or_else(|| panic!("{name} has no annotations"));
        assert!(annotations.title.is_some(), "{name} has no title");
        assert_eq!(
            annotations.read_only_hint,
            Some(true),
            "{name} is not read-only"
        );
        assert_eq!(
            annotations.destructive_hint,
            Some(false),
            "{name} claims to be destructive"
        );
        assert_eq!(
            annotations.open_world_hint,
            Some(true),
            "{name} does not say it reaches the site"
        );
    }
}

#[test]
fn names_are_prefixed_and_short() {
    for tool in BloppenServer::router().list_all() {
        let name = tool.name.as_ref();
        assert!(name.starts_with("bloppen_"), "{name} is not prefixed");
        assert!(name.len() <= 64, "{name} is too long");
        assert!(
            name.bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'),
            "{name} is not lowercase_with_underscores"
        );
    }
}

#[test]
fn every_tool_and_argument_is_described() {
    for tool in BloppenServer::router().list_all() {
        let name = &tool.name;
        assert!(
            tool.description
                .as_deref()
                .is_some_and(|description| !description.trim().is_empty()),
            "{name} has no description"
        );

        let Some(properties) = tool
            .input_schema
            .get("properties")
            .and_then(|p| p.as_object())
        else {
            continue;
        };
        for (argument, schema) in properties {
            assert!(
                schema
                    .get("description")
                    .and_then(|description| description.as_str())
                    .is_some_and(|description| !description.trim().is_empty()),
                "{name}.{argument} has no description"
            );
        }
    }
}

#[test]
fn the_read_only_surface_is_exactly_three_tools() {
    let mut names: Vec<String> = BloppenServer::router()
        .list_all()
        .into_iter()
        .map(|tool| tool.name.into())
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            "bloppen_fetch_page",
            "bloppen_search_products",
            "bloppen_session_status",
        ]
    );
}
