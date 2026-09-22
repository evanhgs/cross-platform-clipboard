mod support;

use cross_platform_clipboard::{application::ClipboardService, storage::SqliteRepository};
use support::TestDatabase;

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

#[test]
fn user_can_find_a_copied_image_after_restarting_the_application() {
    let database = TestDatabase::new("image-acceptance");
    {
        let service = ClipboardService::new(SqliteRepository::open_at(database.path()).unwrap());
        service
            .record_rgba_image(vec![10, 20, 30, 255], 1, 1)
            .unwrap();
    }

    let reopened_repository = SqliteRepository::open_at(database.path()).unwrap();
    let image = reopened_repository.list_recent(10).unwrap()[0]
        .image
        .clone()
        .unwrap();
    assert_eq!(image.rgba, vec![10, 20, 30, 255]);
}
