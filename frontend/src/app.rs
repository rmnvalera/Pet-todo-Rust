use crate::pages::{login::LoginPage, tasks::TasksPage};
use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes fallback=|| "404">
                <Route path=path!("/") view=LoginPage/>
                <Route path=path!("/tasks") view=TasksPage/>
            </Routes>
        </Router>
    }
}

