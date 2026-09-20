use cross_platform_clipboard::ui::App;
use std::time::Duration;
use tracing::Level;

fn main() {
    dioxus::logger::init(Level::INFO).expect("failed to init logger");
    cross_platform_clipboard::clipboard::start_text_watcher(Duration::from_millis(750));
    dioxus::launch(App);
}
