use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::error;
use utoipa::ToSchema;

use crate::infrastructure::db::app_state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct ChatInput {
    pub endpoint:       String,
    pub email:          String,
    pub message:        String,
    pub business_model: String,
    pub session_id:     Option<String>,
    pub language:       Option<String>,
}

#[derive(Debug, Serialize)]
struct AgentRequest {
    endpoint:       String,
    email:          String,
    message:        String,
    business_model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id:     Option<String>,
    language:       String,
}

/// Chat con el agente de IA (proxy → agents-core)
#[utoipa::path(
    post,
    path = "/ai/chat",
    request_body = ChatInput,
    responses(
        (status = 200, description = "Respuesta del asistente", body = Value),
        (status = 500, description = "Error interno"),
    ),
    tag = "AI"
)]
pub async fn chat(
    State(_state): State<AppState>,
    Json(input): Json<ChatInput>,
) -> Json<Value> {
    let url = match std::env::var("AGENTS_CORE_URL") {
        Ok(u) => u,
        Err(_) => {
            return Json(json!({ "error": "AGENTS_CORE_URL not configured" }));
        }
    };

    let body = AgentRequest {
        endpoint:       input.endpoint,
        email:          input.email,
        message:        input.message,
        business_model: input.business_model,
        session_id:     input.session_id,
        language:       input.language.unwrap_or_else(|| "es".to_string()),
    };

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await;

    match resp {
        Ok(r) => match r.json::<Value>().await {
            Ok(v) => Json(v),
            Err(e) => {
                error!("Failed to parse agents-core response: {}", e);
                Json(json!({ "error": "Error al procesar respuesta del agente" }))
            }
        },
        Err(e) => {
            let mut chain = format!("{}", e);
            let mut source = std::error::Error::source(&e);
            while let Some(s) = source {
                chain.push_str(" | caused by: ");
                chain.push_str(&format!("{}", s));
                source = s.source();
            }
            error!(
                "agents-core call failed url={} is_connect={} is_timeout={} is_request={} chain=\"{}\"",
                url,
                e.is_connect(),
                e.is_timeout(),
                e.is_request(),
                chain
            );
            Json(json!({ "error": "Error al conectar con el agente IA" }))
        }
    }
}
