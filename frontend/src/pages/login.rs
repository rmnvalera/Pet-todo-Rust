use crate::api::auth::login;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn LoginPage() -> impl IntoView {
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error, set_error) = signal(Option::<String>::None);
    let (loading, set_loading) = signal(false);

    let navigate = use_navigate();

    let on_submit = move |_| {
        let email = email.get();
        let password = password.get();
        let navigate = navigate.clone();

        set_loading.set(true);
        set_error.set(None);

        leptos::task::spawn_local(async move {
            match login(email, password).await {
                Ok(resp) => {
                    let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
                    storage.set_item("token", &resp.token).unwrap();
                    navigate("/tasks", Default::default());
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    };

    view! {
        <div>
            <h1>"Sign in"</h1>
            {move || error.get().map(|e| view! { <p style="color:red">{e}</p> })}
            <input
                type="email"
                placeholder="Email"
                on:input=move |ev| set_email.set(event_target_value(&ev))
            />
            <input
                type="password"
                placeholder="password"
                on:input=move |ev| set_password.set(event_target_value(&ev))
            />
            <button
                on:click=on_submit
                disabled=move || loading.get()
            >
                {move || if loading.get() { "Loading..." } else { "Sign in" }}
            </button>
        </div>
    }
}
