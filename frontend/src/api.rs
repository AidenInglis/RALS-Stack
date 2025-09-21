use serde::{Deserialize, Serialize};
use reqwest::Client;
use web_sys::console; // logging

const GRAPHQL_ENDPOINT: &str = "http://localhost:3000/graphql";

#[derive(Serialize)]
struct GraphQLRequestBody {
    query: String,
    variables: serde_json::Value,
}

pub async fn login(email: String, password: String) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();

    let query = r#"
        mutation Login($email: String!, $password: String!) {
            login(input: { email: $email, password: $password })
        }
    "#;


    let body = GraphQLRequestBody {
        query: query.to_string(),
        variables: serde_json::json!({ "email": email, "password": password }),
    };

    let res = client.post(GRAPHQL_ENDPOINT)
        .json(&body)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    // 👇 Log raw response in browser devtools console
    console::log_1(&format!("GraphQL response: {:?}", res).into());

    if let Some(token) = res["data"]["login"].as_str() {
        Ok(token.to_string())
    } else {
        Err(format!("Login failed: {:?}", res).into())
    }
}
