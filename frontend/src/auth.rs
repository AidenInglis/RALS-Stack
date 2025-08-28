use leptos::*;
use web_sys::window;

#[derive(Clone)]
pub struct AuthContext {
    pub token: RwSignal<Option<String>>,
}

impl AuthContext {
    pub fn new() -> Self {
        let storage = window().unwrap().local_storage().unwrap();
        let token = storage.and_then(|s| s.get_item("jwt").ok().flatten());

        AuthContext {
            token: create_rw_signal(token),
        }
    }

    pub fn set_token(&self, token: Option<String>) {
        if let Some(storage) = window().unwrap().local_storage().unwrap() {
            match &token {
                Some(t) => {
                    storage.set_item("jwt", t).unwrap();
                }
                None => {
                    storage.remove_item("jwt").unwrap();
                }
            }
        }
        self.token.set(token);
    }
}
