mod support;

use cross_platform_clipboard::storage::SqliteRepository;
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
