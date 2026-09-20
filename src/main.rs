use cross_platform_clipboard::ui::App;
use tracing::Level;

fn main() {
    dioxus::logger::init(Level::INFO).expect("failed to init logger");
    // Étape suivante : appeler `SqliteRepository::open()` ici, puis injecter le
    // service dans l'application. Ne crée pas de `static Connection` : SQLite
    // doit rester possédée par un worker ou par le thread qui l'utilise.
    dioxus::launch(App);
}
