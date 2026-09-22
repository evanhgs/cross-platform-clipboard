mod support;

use cross_platform_clipboard::storage::SqliteRepository;
use rusqlite::Connection;
use support::TestDatabase;

#[test]
fn persists_and_loads_text_entries() {
    let database = TestDatabase::new("repository");
    let repository = SqliteRepository::open_at(database.path()).unwrap();

    repository.insert_text("première copie", "hash-1").unwrap();
    let entries = repository.list_recent(10).unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].text.as_deref(), Some("première copie"));
    assert_eq!(entries[0].content_hash, "hash-1");
}

#[test]
fn retains_only_the_200_newest_unpinned_entries() {
    let database = TestDatabase::new("retention");
    let repository = SqliteRepository::open_at(database.path()).unwrap();

    for number in 0..205 {
        repository
            .insert_text(&format!("copie-{number}"), &format!("hash-{number}"))
            .unwrap();
    }

    let entries = repository.list_recent(500).unwrap();
    assert_eq!(entries.len(), 200);
    assert_eq!(entries[0].text.as_deref(), Some("copie-204"));
}

#[test]
fn pinned_entries_survive_a_clear_and_can_be_unpinned() {
    let database = TestDatabase::new("pinned");
    let repository = SqliteRepository::open_at(database.path()).unwrap();
    repository.insert_text("à garder", "keep").unwrap();
    repository.insert_text("à effacer", "remove").unwrap();

    let pinned_id = repository.list_recent(10).unwrap()[1].id;
    repository.toggle_pinned(pinned_id).unwrap();
    repository.clear_unpinned().unwrap();

    let entries = repository.list_recent(10).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].text.as_deref(), Some("à garder"));
    assert!(entries[0].pinned);
}

#[test]
fn migrates_legacy_image_files_into_sqlite_blobs() {
    let database = TestDatabase::new("legacy-image");
    let image_path = database.path().parent().unwrap().join("legacy.png");
    image::RgbaImage::from_raw(1, 1, vec![40, 50, 60, 255])
        .unwrap()
        .save(&image_path)
        .unwrap();
    let connection = Connection::open(database.path()).unwrap();
    connection
        .execute_batch(
            "CREATE TABLE clipboard_entries(
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content_type TEXT NOT NULL,
                text_content TEXT,
                image_path TEXT,
                content_hash TEXT NOT NULL,
                pinned INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL DEFAULT (unixepoch())
            );",
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO clipboard_entries (content_type, image_path, content_hash) VALUES ('image', ?1, 'legacy')",
            [image_path.to_string_lossy().as_ref()],
        )
        .unwrap();
    drop(connection);

    let entries = SqliteRepository::open_at(database.path())
        .unwrap()
        .list_recent(10)
        .unwrap();
    assert_eq!(
        entries[0].image.as_ref().unwrap().rgba,
        vec![40, 50, 60, 255]
    );
}
