mod support;

use cross_platform_clipboard::{application::ClipboardService, storage::SqliteRepository};
use support::TestDatabase;

/// Système local : le cas d'usage complet, sans UI et sans dépendance Wayland.
#[test]
fn copied_text_reaches_the_history_and_whitespace_is_ignored() {
    let database = TestDatabase::new("system");
    let service = ClipboardService::new(SqliteRepository::open_at(database.path()).unwrap());

    service.record_text("  texte utile  ").unwrap();
    service.record_text("   \n\t").unwrap();

    let entries = service.recent_entries(10).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].text.as_deref(), Some("texte utile"));
}
