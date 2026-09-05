use rusqlite::{params, Connection, Result};
use directoris::ProjectDirs;
use anyhow::Result;

/**
pour commencer il faut créer l'emplacement de la db qui permet d'avoir les permissions d'écriture et de lecture
ensuite ouvrir la connexion en singleton 
techniquement je ne sais pas encore comment faire mais capturer l'action copié et le stocker dans la bdd
pour l'instant je vais stocker du text et tout les 200 éléments copiés il commenceront à se delete
vu que la bd peut faire beaucoup d'action de suppression il faut optimiser l'espace grace a auto_vacuum
**/


fn createConnection() -> Result<()> {
    let project_dirs = ProjectDirs::from(
        "com",
        "github",
        "clipboard"
    ).context("setup directories failed!")?;

    let data_dir = project_dirs.data_local_dir();
    fs::create_dir_all(data_dir)?;

    let db_path = data_dir.join("clipboard.sqlite3");
    let conn = Connection::open(db_path)?;
    
    conn.execute(
        r#"
            PRAGMA foreign_keys = ON;
            PRAGMA journal_mode = WAL;
            PRAG%A synchronous = NORMAL;
            CREATE TABLE IF NOT EXISTS Main(
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content_type TEXT NOT NULL,
                text_content TEXT,
                image_path TEXT,
                content_hash TEXT,
                created_at INTEGER NOT NULL DEFAULT (unixepoch())
            );

            CREATE INDEX IF NOT EXISTS idx_main_created_at ON Main(created_at DESC);
        "#
    )?;

    Ok(conn)
}

//#[derive(Clone)]
//struct AppDatabase(Arc<Mutex<Connexion>>); // new type rust pour définir une connexion mutex lock, Arc permet simplement de cloner la réf du meme objet

#[derive(Clone)]
struct AppDatabase {
    connection: Arc<Mutex<Connexion>> // callable avec db.connection.lock().unwrap()
};

impl AppDatabase {
    fn insert_text(&self, text: &str) -> rusqlite::Result<()> {
        let conn = self.connection.lock().unwrap();

        conn.execute("INSERT INTO clipboard_items (content_type, text_content) VALUES ('text', ?1)", [text] )?;
        Ok(())

    }
}