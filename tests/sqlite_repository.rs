mod support;

use cross_platform_clipboard::storage::SqliteRepository;
use support::TestDatabase;

/// Intégration : SQLite réel, mais fichier temporaire et isolé.
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
