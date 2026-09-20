mod support;

use cross_platform_clipboard::{application::ClipboardService, storage::SqliteRepository};
use support::TestDatabase;

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

#[test]
fn identical_text_is_saved_only_once() {
    let database = TestDatabase::new("deduplication");
    let service = ClipboardService::new(SqliteRepository::open_at(database.path()).unwrap());

    service.record_text("même texte").unwrap();
    service.record_text("même texte").unwrap();

    assert_eq!(service.recent_entries(10).unwrap().len(), 1);
}

#[test]
fn rgba_image_is_saved_as_a_local_png() {
    let database = TestDatabase::new("image");
    let service = ClipboardService::new(SqliteRepository::open_at(database.path()).unwrap());

    service
        .record_rgba_image(vec![255, 0, 0, 255, 0, 255, 0, 255], 2, 1)
        .unwrap();

    let entry = service.recent_entries(10).unwrap().pop().unwrap();
    let path = entry.image_path.unwrap();
    assert!(std::path::Path::new(&path).is_file());
}
