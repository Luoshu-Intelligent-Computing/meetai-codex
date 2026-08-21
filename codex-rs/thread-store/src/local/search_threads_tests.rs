use codex_protocol::ThreadId;
use codex_protocol::protocol::SessionSource;
use codex_rollout::RolloutConfig;
use codex_rollout::ThreadItem;
use pretty_assertions::assert_eq;
use tempfile::TempDir;
use uuid::Uuid;

use super::ThreadSearchItem;
use super::cursor_from_thread_search_item;
use crate::SearchThreadsParams;
use crate::SortDirection;
use crate::ThreadSortKey;
use crate::local::LocalThreadStore;
use crate::local::test_support::test_config;
use crate::local::test_support::write_session_file;

#[test]
fn recency_cursor_includes_thread_id_tie_breaker() {
    let thread_id = ThreadId::from_string("00000000-0000-0000-0000-000000000123")
        .expect("thread ID should parse");
    let item = ThreadSearchItem {
        item: ThreadItem {
            thread_id: Some(thread_id),
            recency_at: Some("2026-01-27T12:34:56Z".to_string()),
            ..Default::default()
        },
        snippet: String::new(),
    };

    let cursor = cursor_from_thread_search_item(&item, ThreadSortKey::RecencyAt)
        .expect("cursor should build");

    assert_eq!(
        serde_json::to_string(&cursor).expect("cursor should serialize"),
        format!("\"2026-01-27T12:34:56Z|{thread_id}\"")
    );
}

#[tokio::test]
async fn content_search_finds_rollouts_not_yet_indexed_in_state_db() {
    let home = TempDir::new().expect("create Codex home");
    let config = test_config(home.path());
    let rollout_config = RolloutConfig {
        codex_home: config.codex_home.clone(),
        sqlite: config.sqlite.clone(),
        cwd: home.path().to_path_buf(),
        model_provider_id: config.default_model_provider_id.clone(),
        generate_memories: false,
    };
    let state_db = codex_rollout::state_db::try_init(&rollout_config)
        .await
        .expect("initialize empty state DB");
    write_session_file(home.path(), "2025-01-03T10-00-00", Uuid::new_v4())
        .expect("write rollout after state DB initialization");
    let store = LocalThreadStore::new(config, Some(state_db));

    let page = super::search_threads(
        &store,
        SearchThreadsParams {
            page_size: 20,
            cursor: None,
            sort_key: ThreadSortKey::RecencyAt,
            sort_direction: SortDirection::Desc,
            allowed_sources: vec![SessionSource::Cli],
            archived: false,
            search_term: "Hello from user".to_string(),
        },
    )
    .await
    .expect("search rollout content");

    assert_eq!(page.items.len(), 1);
    assert!(page.items[0].snippet.contains("Hello from user"));
}
