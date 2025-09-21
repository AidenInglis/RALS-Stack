use leptos::*;
use leptos_router::*;
use web_sys::window;

mod api;
mod auth;
use auth::AuthContext;



#[component]
pub fn App() -> impl IntoView {
    let auth = AuthContext::new();
    provide_context(auth);

    view! {
        <Router>
            <Routes>
                <Route path="/" view=Home />
                <Route path="/login" view=Login />
                <Route path="/register" view=Register />
                <Route path="/secret" view=SecretPage />
            </Routes>
        </Router>
    }
}

#[component]
fn Home() -> impl IntoView {
    view! {
        <main style="display:flex;flex-direction:column;align-items:center;justify-content:center;
                     height:100vh;background:linear-gradient(135deg,#ece9e6,#ffffff);font-family:sans-serif;">
            <h1 style="font-size:3rem;color:#333;margin-bottom:1rem;">"Welcome to the Coupon Site"</h1>
            <p style="font-size:1.2rem;color:#555;margin-bottom:2rem;max-width:600px;text-align:center;">
                "Get the best deals and manage your coupons securely. Login to access your dashboard or register for a new account."
            </p>
            <div style="display:flex;gap:1rem;">
                <a href="/login"
                   style="padding:0.75rem 1.5rem;background:#4CAF50;color:white;
                          border-radius:8px;text-decoration:none;font-weight:bold;">
                    "Login"
                </a>
                <a href="/register"
                   style="padding:0.75rem 1.5rem;background:#2196F3;color:white;
                          border-radius:8px;text-decoration:none;font-weight:bold;">
                    "Register"
                </a>
            </div>
        </main>
    }
}

#[component]
fn Login() -> impl IntoView {
    let navigate = use_navigate();
    let auth = use_context::<auth::AuthContext>().expect("AuthContext missing");

    let (email, set_email) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());

    // clone handles for FnMut
    let auth_clone = auth.clone();
    let navigate_clone = navigate.clone();

    let on_submit = move |_| {
        let email = email.get();
        let password = password.get();
        let auth = auth_clone.clone();
        let navigate = navigate_clone.clone();

        leptos::spawn_local(async move {
            match api::login(email, password).await {
                Ok(token) => {
                    auth.set_token(Some(token.clone()));
                    navigate("/secret", Default::default());
                }
                Err(err) => logging::error!("❌ Login failed: {:?}", err),
            }
        });
    };

    view! {
        <div style="display:flex;flex-direction:column;align-items:center;
                    justify-content:center;height:100vh;background:#f4f4f4;">
            <div style="padding:2rem;background:white;border-radius:12px;
                        box-shadow:0 4px 8px rgba(0,0,0,0.1);width:300px;">
                <h2 style="margin-bottom:1rem;text-align:center;">"Login"</h2>
                <input type="email" placeholder="Email"
                       style="width:100%;padding:0.5rem;margin-bottom:1rem;border:1px solid #ccc;border-radius:6px;"
                       on:input=move |ev| set_email.set(event_target_value(&ev)) />
                <input type="password" placeholder="Password"
                       style="width:100%;padding:0.5rem;margin-bottom:1rem;border:1px solid #ccc;border-radius:6px;"
                       on:input=move |ev| set_password.set(event_target_value(&ev)) />
                <button on:click=on_submit
                        style="width:100%;padding:0.75rem;background:#4CAF50;color:white;
                               border:none;border-radius:6px;cursor:pointer;">
                    "Login"
                </button>
                <p style="margin-top:1rem;text-align:center;">
                    "Don't have an account? " <a href="/register" style="color:#2196F3;">"Register"</a>
                </p>
            </div>
        </div>
    }
}


#[component]
fn SecretPage() -> impl IntoView {
    let auth = use_context::<auth::AuthContext>().unwrap();
    let navigate = use_navigate();
    let (message, set_message) = create_signal("Checking authentication...".to_string());

    leptos::spawn_local(async move {
        if let Some(storage) = window().unwrap().local_storage().unwrap() {
            if let Ok(Some(token)) = storage.get_item("jwt") {
                // Call backend /secret route
                let client = reqwest::Client::new();
                let res = client
                    .get("http://localhost:3000/secret")
                    .header("Authorization", format!("Bearer {}", token))
                    .send()
                    .await;

                match res {
                    Ok(r) if r.status().is_success() => {
                        let text = r.text().await.unwrap_or_else(|_| "Welcome!".to_string());
                        set_message.set(format!("✅ {}", text));
                    }
                    _ => {
                        storage.remove_item("jwt").unwrap();
                        set_message.set("❌ Session expired. Redirecting...".to_string());
                        navigate("/login", Default::default());
                    }
                }
            } else {
                set_message.set("❌ Not logged in. Redirecting...".to_string());
                navigate("/login", Default::default());
            }
        }
    });

    // Animation keyframes (inline <style>)
    let style = r#"
        @keyframes fadeIn {
            from { opacity: 0; transform: translateY(20px); }
            to { opacity: 1; transform: translateY(0); }
        }
    "#;

    view! {
        <>
            <style>{style}</style>
            <div style="
                display:flex;flex-direction:column;align-items:center;
                justify-content:center;height:100vh;background:linear-gradient(135deg,#e0f7fa,#ffffff);
                font-family:sans-serif;animation:fadeIn 0.8s ease-out;">
                <div style="padding:2rem;background:white;border-radius:12px;
                            box-shadow:0 4px 10px rgba(0,0,0,0.15);max-width:500px;text-align:center;">
                    <h2 style="margin-bottom:1rem;color:#333;">"🔒 Secret Page"</h2>
                    <p style="font-size:1.1rem;color:#555;">{message}</p>
                </div>
            </div>
        </>
    }
}


#[component]
fn ProtectedRoute() -> impl IntoView {
    let auth = use_context::<AuthContext>().expect("AuthContext missing");
    let navigate = use_navigate();

    // reactive check
    create_effect(move |_| {
        if auth.token.get().is_none() {
            navigate("/login", Default::default());
        }
    });

    view! {
        <SecretPage />
    }
}

#[component]
fn Register() -> impl IntoView {
    let (email, set_email) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());

    let on_submit = move |_| {
        // TODO: hook into backend mutation (register user)
        logging::log!("📦 Register with email={} password={}", email.get(), password.get());

    };

    view! {
        <div style="display:flex;flex-direction:column;align-items:center;
                    justify-content:center;height:100vh;background:#f4f4f4;">
            <div style="padding:2rem;background:white;border-radius:12px;
                        box-shadow:0 4px 8px rgba(0,0,0,0.1);width:300px;">
                <h2 style="margin-bottom:1rem;text-align:center;">"Register"</h2>
                <input type="email" placeholder="Email"
                       style="width:100%;padding:0.5rem;margin-bottom:1rem;border:1px solid #ccc;border-radius:6px;"
                       on:input=move |ev| set_email.set(event_target_value(&ev)) />
                <input type="password" placeholder="Password"
                       style="width:100%;padding:0.5rem;margin-bottom:1rem;border:1px solid #ccc;border-radius:6px;"
                       on:input=move |ev| set_password.set(event_target_value(&ev)) />
                <button on:click=on_submit
                        style="width:100%;padding:0.75rem;background:#2196F3;color:white;
                               border:none;border-radius:6px;cursor:pointer;">
                    "Register"
                </button>
                <p style="margin-top:1rem;text-align:center;">
                    "Already have an account? " <a href="/login" style="color:#4CAF50;">"Login"</a>
                </p>
            </div>
        </div>
    }
}
