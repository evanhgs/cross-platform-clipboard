mod support;

use cross_platform_clipboard::{application::ClipboardService, storage::SqliteRepository};
use support::TestDatabase;

/// Acceptation automatisable une copie reste disponible après redémarrage.
/// Le comportement UI/Wayland est validé manuellement, voir TESTING.md.
#[test]
fn user_can_find_a_copied_text_after_restarting_the_application() {
    let database = TestDatabase::new("acceptance");
    {
        let service = ClipboardService::new(SqliteRepository::open_at(database.path()).unwrap());
        service.record_text("adresse à conserver").unwrap();
    }

    let reopened_repository = SqliteRepository::open_at(database.path()).unwrap();
    let entries = reopened_repository.list_recent(10).unwrap();
    assert_eq!(entries[0].text.as_deref(), Some("adresse à conserver"));
}
