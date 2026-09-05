use dioxus::prelude::*;
static CSS: Asset = asset!("/assets/main.css");
use conn::AppDatabase;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    use_context_provider(|| {
        let conn = open
    })

    rsx! {
        
    }
}
