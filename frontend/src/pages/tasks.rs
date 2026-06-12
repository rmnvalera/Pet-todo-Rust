use crate::api::tasks::{Task, fetch_tasks, get_token};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn TasksPage() -> impl IntoView {
    let navigate = use_navigate();

    if get_token().is_none() {
        navigate("/", Default::default());
    }

    let page = RwSignal::new(1i64);
    let tasks = RwSignal::new(Vec::<Task>::new());
    let total = RwSignal::new(0i64);
    let error = RwSignal::new(Option::<String>::None);

    let load_tasks = Action::new_local(move |&page_num: &i64| {
        let navigate = navigate.clone();
        async move {
            match fetch_tasks(page_num).await {
                Ok(data) => {
                    tasks.set(data.data);
                    total.set(data.total as i64);
                    error.set(None);
                }
                Err(e) => {
                    if e == "Session is dead" {
                        navigate("/", Default::default());
                    }
                    error.set(Some(e));
                }
            }
        }
    });

    load_tasks.dispatch(page.get_untracked());

    view! {
        <div>
            <h1>"Tasks"</h1>

            {move || error.get().map(|e| view! { <p style="color:red">{e}</p> })}

            {move || if load_tasks.pending().get() {
                view! { <p>"Loading..."</p> }.into_any()
            } else {
                view! {
                    <ul>
                        <For
                            each=move || tasks.get()
                            key=|t| t.id.clone()
                            children=|task| view! {
                                <li>
                                    <strong>{task.title.clone()}</strong>
                                    " — "
                                    {task.status.clone()}
                                    {task.description.map(|d| view! { <span>" · "{d}</span> })}
                                </li>
                            }
                        />
                    </ul>
                }.into_any()
            }}

            <div>
                <button
                    disabled=move || page.get() <= 1 || load_tasks.pending().get()
                    on:click=move |_| {
                        page.update(|p| *p -= 1);
                        load_tasks.dispatch(page.get_untracked());
                    }
                >
                    "← Back"
                </button>
                <span>" Page"{move || page.get()}" "</span>
                <button
                    disabled={move || page.get() * 10 >= total.get() || load_tasks.pending().get()}
                    on:click=move |_| {
                        page.update(|p| *p += 1);
                        load_tasks.dispatch(page.get_untracked());
                    }
                >
                    "Next →"
                </button>
            </div>
        </div>
    }
}
