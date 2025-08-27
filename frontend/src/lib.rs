use leptos::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                <Route path="/" view=Home />
                <Route path="/login" view=Login />
            </Routes>
        </Router>
    }
}

#[component]
fn Home() -> impl IntoView {
    view! {
        <h1>"Welcome to the Coupon Site"</h1>
        <a href="/login">"Go to Login"</a>
    }
}

#[component]
fn Login() -> impl IntoView {
    let (email, set_email) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());

    view! {
        <div>
            <h2>"Login"</h2>
            <input
                type="email"
                placeholder="Email"
                on:input=move |ev| set_email.set(event_target_value(&ev))
            />
            <input
                type="password"
                placeholder="Password"
                on:input=move |ev| set_password.set(event_target_value(&ev))
            />
            <button on:click=move |_| {
                logging::log!("Email: {}, Password: {}", email.get(), password.get());
            }>"Login"</button>
        </div>
    }
}
