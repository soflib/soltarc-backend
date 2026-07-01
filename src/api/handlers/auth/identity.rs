// Rutas:
//   POST /auth/register  → register
//   POST /auth/login     → login
//   POST /auth/logout    → logout
//   GET  /auth/me        → me

use axum::{extract::{Extension, State}, http::StatusCode, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::ToSchema;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::api::middleware::roles::AuthUser;

use crate::infrastructure::db::app_state::AppState;
use crate::generated::auth::{RegisterRequest, LoginRequest, LogoutRequest, GetAllUsersRequest, GetUserRequest, RequestPasswordResetRequest, ConfirmPasswordResetRequest};
use super::grpc_to_http;

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterInput {
    pub email:          String,
    pub username:       String,
    pub password:       String,
    pub full_name:      Option<String>,
    pub phone:          Option<String>,
    pub role:           Option<String>,
    #[serde(default)]
    #[schema(default = false)]
    pub privat_db:      bool,
    pub tenant_id:      Option<String>,
    pub payment_id:     Option<String>,
    pub payment_plan:   Option<String>,
    pub payment_method: Option<String>,
    pub billing_period: Option<String>,
    /// Idioma de los datos de ejemplo/plantilla a sembrar en el alta ('es'|'en').
    /// Lo elige el admin en el registro. Default 'es' si no viene.
    pub seed_lang:      Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginInput {
    pub email:       String,
    pub password:    String,
    pub device_hint: Option<String>,
    pub ip_address:  Option<String>,
    pub user_agent:  Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LogoutInput {
    pub refresh_jti:  String,
    pub access_token: Option<String>,
}

// ── Register ─────────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = RegisterInput,
    responses(
        (status = 200, description = "User registered",  body = Value),
        (status = 400, description = "Bad request",      body = Value),
        (status = 500, description = "Internal error",   body = Value),
    ),
    tag = "Auth"
)]
pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterInput>,
) -> (StatusCode, Json<Value>) {
    debug!(email = %body.email, "POST /auth/register");

    // Si body.tenant_id viene vacío → es alta de admin que auto-crea su tenant.
    // En ese caso disparamos el seed de proveedores. Si viene poblado, es un
    // sub-user uniéndose a un tenant existente — sus proveedores ya fueron sembrados.
    let is_admin_registration = body
        .tenant_id
        .as_deref()
        .map_or(true, str::is_empty);

    // El plan comprado define el tope de proyectos del tenant (se guarda tras crear el tenant).
    let payment_plan = body.payment_plan.clone().unwrap_or_default();

    // Idioma elegido por el admin para los datos de ejemplo/plantilla ('es'|'en').
    // Default 'es'. Se persiste en cpa_tenant_limites y se propaga a cada seed; el
    // trigger de partidas (fn_cargar_plantilla_obra) lo relee desde ahí.
    let seed_lang: &str = match body.seed_lang.as_deref() {
        Some(s) if s.eq_ignore_ascii_case("en") => "en",
        _ => "es",
    };

    // Datos para el correo de bienvenida (antes de mover `body` al request gRPC).
    let admin_email = body.email.clone();
    let admin_name  = body.full_name.clone().unwrap_or_default();

    let req = RegisterRequest {
        email:          body.email.clone(),
        username:       body.username,
        password:       body.password,
        full_name:      body.full_name.unwrap_or_default(),
        phone:          body.phone.unwrap_or_default(),
        role:           body.role.unwrap_or_default(),
        privat_db:      body.privat_db,
        tenant_id:      body.tenant_id.unwrap_or_default(),
        payment_id:     body.payment_id.unwrap_or_default(),
        payment_plan:   payment_plan.clone(),
        payment_method: body.payment_method.unwrap_or_default(),
        billing_period: body.billing_period.unwrap_or_default(),
    };

    let mut client = state.auth_grpc.clone();
    match client.register(req).await {
        Ok(r) => {
            info!(email = %body.email, "POST /auth/register ← 200");

            if is_admin_registration {
                if let Some(u) = r.user.as_ref() {
                    match Uuid::parse_str(&u.tenant_id) {
                        Ok(tid) => {
                            // UUID del admin recién creado: atribuye a él los egresos/
                            // ingresos de ejemplo (si no parsea, queda en nil → "Sistema").
                            let admin_uid = Uuid::parse_str(&u.user_id).unwrap_or(Uuid::nil());
                            // Topes según el plan comprado: proyectos (plan5→5, plan10→10,
                            // plan20→20, dedicated/sin plan → ilimitado) y almacenamiento
                            // (plan5→10GB, plan10→15GB, plan20→20GB, resto → default 25GB).
                            // Solo en el alta del admin (es quien compra el plan); los
                            // sub-users no los tocan.
                            let max_proy    = crate::dal::tenant_limite::max_for_plan(&payment_plan);
                            let max_storage = crate::dal::tenant_limite::storage_for_plan(&payment_plan);
                            match crate::dal::tenant_limite::set_limite(&state.postgres, tid, max_proy, max_storage, Some(seed_lang)).await {
                                Ok(_)  => info!(tenant_id = %tid, plan = %payment_plan, ?max_proy, ?max_storage, "límites de tenant fijados"),
                                Err(e) => warn!(tenant_id = %tid, error = %e, "set límites tenant falló (registro OK)"),
                            }
                            // Orden importa: catálogos primero (proveedor referencia tipo/giro
                            // por FK de cpa_catalogos), luego proveedores y centros de costo.
                            match crate::dal::catalog_g::seed_for_tenant(&state.postgres, tid, seed_lang).await {
                                Ok(n)  => info!(tenant_id = %tid, rows = n, "seed catalogos cargado"),
                                Err(e) => warn!(tenant_id = %tid, error = %e, "seed catalogos falló (registro OK)"),
                            }
                            // Grupos de negocio (5 default). Sin dependencias; va temprano.
                            match crate::dal::gn_grupos::seed_for_tenant(&state.postgres, tid, seed_lang).await {
                                Ok(n)  => info!(tenant_id = %tid, rows = n, "seed grupos_negocio cargado"),
                                Err(e) => warn!(tenant_id = %tid, error = %e, "seed grupos_negocio falló (registro OK)"),
                            }
                            match crate::dal::proveedores::seed_for_tenant(&state.postgres, tid, seed_lang).await {
                                Ok(n)  => info!(tenant_id = %tid, rows = n, "seed proveedores cargado"),
                                Err(e) => warn!(tenant_id = %tid, error = %e, "seed proveedores falló (registro OK)"),
                            }
                            match crate::dal::centros_costo::seed_for_tenant(&state.postgres, tid, seed_lang).await {
                                Ok(n)  => info!(tenant_id = %tid, rows = n, "seed centros_costo cargado"),
                                Err(e) => warn!(tenant_id = %tid, error = %e, "seed centros_costo falló (registro OK)"),
                            }
                            match crate::dal::clientes::seed_for_tenant(&state.postgres, tid, seed_lang).await {
                                Ok(n)  => info!(tenant_id = %tid, rows = n, "seed clientes cargado"),
                                Err(e) => warn!(tenant_id = %tid, error = %e, "seed clientes falló (registro OK)"),
                            }
                            // Presupuestos: catálogos (tipos de costo, unidades, costos
                            // estimados) + 1 presupuesto demo con partidas. Va después de
                            // clientes porque el presupuesto demo se cuelga de "Cliente 1".
                            match crate::dal::ppto_seed::seed_for_tenant(&state.postgres, tid, seed_lang).await {
                                Ok(n)  => info!(tenant_id = %tid, rows = n, "seed presupuestos cargado"),
                                Err(e) => warn!(tenant_id = %tid, error = %e, "seed presupuestos falló (registro OK)"),
                            }
                            // Finanzas: saldos de banco (por cada banco del catálogo),
                            // egresos e ingresos de ejemplo. Van al final porque se
                            // cuelgan de proyecto/proveedor/cliente/banco ya sembrados.
                            match crate::dal::saldos_bancos::seed_for_tenant(&state.postgres, tid).await {
                                Ok(n)  => info!(tenant_id = %tid, rows = n, "seed saldos_banco cargado"),
                                Err(e) => warn!(tenant_id = %tid, error = %e, "seed saldos_banco falló (registro OK)"),
                            }
                            match crate::dal::egresos::seed_for_tenant(&state.postgres, tid, admin_uid, seed_lang).await {
                                Ok(n)  => info!(tenant_id = %tid, rows = n, "seed egresos cargado"),
                                Err(e) => warn!(tenant_id = %tid, error = %e, "seed egresos falló (registro OK)"),
                            }
                            match crate::dal::ingresos::seed_for_tenant(&state.postgres, tid, admin_uid, seed_lang).await {
                                Ok(n)  => info!(tenant_id = %tid, rows = n, "seed ingresos cargado"),
                                Err(e) => warn!(tenant_id = %tid, error = %e, "seed ingresos falló (registro OK)"),
                            }
                            // Sincronizar perfiles de negocio (gn_usuarios) con los
                            // usuarios reales del tenant: liga usuario_uuid y fija nivel
                            // default por rol. Habilita el control de acceso por nivel.
                            let mut gusers = state.auth_grpc.clone();
                            match gusers.get_all_users(GetAllUsersRequest { limit: 1000, offset: 0, tenant_id: u.tenant_id.clone() }).await {
                                Ok(resp) => {
                                    let lista: Vec<(Uuid, String, String)> = resp.users.into_iter()
                                        .filter_map(|us| Uuid::parse_str(&us.user_id).ok().map(|id| (id, us.email, us.role)))
                                        .collect();
                                    let n = crate::dal::gn_usuarios::sync_for_tenant(&state.postgres, tid, &lista).await;
                                    info!(tenant_id = %tid, usuarios = n, "sync gn_usuarios (perfiles) ok");
                                }
                                Err(e) => warn!(tenant_id = %tid, error = %e, "sync gn_usuarios falló (registro OK)"),
                            }

                            // Las asignaciones demo (cada proyecto template → usuarios demo
                            // arquitecto/finanzas/reportes) se siembran en SQL: el trigger
                            // trg_gn_usuarios_asigna_demo de seed_proyecto_asignaciones.sql las
                            // crea al nacer el perfil gn_usuarios. No hay plumbing en Rust.

                            // Correo de bienvenida al admin (best-effort, fire-and-forget):
                            // si falta config de Outlook o falla el envío, el registro sigue OK.
                            let to     = admin_email.clone();
                            let nombre = if admin_name.is_empty() { u.username.clone() } else { admin_name.clone() };
                            let dash   = std::env::var("DASHBOARD_URL")
                                .ok().filter(|s| !s.trim().is_empty())
                                .unwrap_or_else(|| "https://dashboard.soflib.com".to_string());
                            tokio::spawn(async move {
                                let vars = [("nombre", nombre.as_str()), ("dashboard_url", dash.as_str())];
                                match crate::infrastructure::email::outlook::send_template(&to, "welcome", &vars).await {
                                    Ok(_)  => info!(%to, "correo de bienvenida enviado"),
                                    Err(e) => warn!(%to, error = %e, "correo de bienvenida falló (registro OK)"),
                                }
                            });
                        },
                        Err(_) => warn!(raw = %u.tenant_id, "tenant_id no parseable a UUID; seeds omitidos"),
                    }
                }
            }

            (StatusCode::OK, Json(json!({
                "access_token": r.access_token,
                "refresh_jti":  r.refresh_jti,
                "expires_in":   r.expires_in,
                "user": r.user.map(|u| json!({
                    "user_id":   u.user_id,
                    "email":     u.email,
                    "username":  u.username,
                    "role":      u.role,
                    "status":    u.status,
                    "tenant_id": u.tenant_id,
                })),
            })))
        }
        Err(status) => {
            let (code, body) = grpc_to_http(status);
            (code, Json(body))
        }
    }
}

// ── Upgrade de plan ──────────────────────────────────────────────────────────
// El admin del tenant ya pagó el upgrade en Stripe (vía payments_backend
// /create/upgrade/intent, que SÍ acepta emails existentes); aquí solo
// actualizamos el tope de proyectos en soltarc.cpa_tenant_limites.

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpgradePlanInput {
    pub payment_plan:   String,          // plan5 | plan10 | plan20 | dedicated
    pub payment_id:     Option<String>,  // PaymentIntent de Stripe (auditoría en logs)
    pub billing_period: Option<String>,
}

#[utoipa::path(
    post,
    path = "/auth/plan/upgrade",
    request_body = UpgradePlanInput,
    responses(
        (status = 200, description = "Plan actualizado; nuevo cupo",        body = Value),
        (status = 400, description = "Plan desconocido o no es un upgrade", body = Value),
        (status = 403, description = "Solo el admin puede cambiar el plan", body = Value),
    ),
    tag = "Auth"
)]
pub async fn upgrade_plan(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<UpgradePlanInput>,
) -> (StatusCode, Json<Value>) {
    if !auth_user.role.eq_ignore_ascii_case("admin") {
        return (StatusCode::FORBIDDEN, Json(json!({ "mensaje": "Solo el administrador puede cambiar el plan." })));
    }
    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    // Solo los planes conocidos; max_for_plan: plan5→5, plan10→10, plan20→20, dedicated→None.
    let new_max = match body.payment_plan.as_str() {
        "plan5" | "plan10" | "plan20" | "dedicated" =>
            crate::dal::tenant_limite::max_for_plan(&body.payment_plan),
        _ => return (StatusCode::BAD_REQUEST, Json(json!({ "mensaje": "Plan desconocido." }))),
    };
    // Cuota de almacenamiento del nuevo plan (plan5→10GB, plan10→15GB, plan20→20GB).
    let new_storage = crate::dal::tenant_limite::storage_for_plan(&body.payment_plan);

    // Solo UPGRADE: el nuevo tope debe ser mayor al actual (None = ilimitado, ya es el máximo).
    let (usados, current_max) = match crate::dal::proyectos::cupo(&state.postgres, tenant_id).await {
        Ok(c) => c,
        Err(ret) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje }))),
    };
    match (current_max, new_max) {
        (None, _) =>
            return (StatusCode::BAD_REQUEST, Json(json!({ "mensaje": "Tu plan ya es ilimitado." }))),
        (Some(cur), Some(nm)) if nm <= cur =>
            return (StatusCode::BAD_REQUEST, Json(json!({ "mensaje": "El nuevo plan debe ser superior al actual." }))),
        _ => {}
    }

    if let Err(e) = crate::dal::tenant_limite::set_limite(&state.postgres, tenant_id, new_max, new_storage, None).await {
        warn!(tenant_id = %tenant_id, error = %e, "upgrade de plan: set límite falló");
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "mensaje": "No se pudo actualizar el plan." })));
    }

    info!(
        tenant_id = %tenant_id, plan = %body.payment_plan, ?new_max, ?new_storage,
        payment_id = %body.payment_id.as_deref().unwrap_or("-"),
        billing = %body.billing_period.as_deref().unwrap_or("-"),
        "plan actualizado (upgrade)"
    );
    (StatusCode::OK, Json(json!({
        "plan":          body.payment_plan,
        "max_proyectos": new_max,
        "usados":        usados,
        "ilimitado":     new_max.is_none(),
    })))
}

// ── Login ─────────────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginInput,
    responses(
        (status = 200, description = "Login successful",     body = Value),
        (status = 401, description = "Invalid credentials",  body = Value),
        (status = 500, description = "Internal error",       body = Value),
    ),
    tag = "Auth"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginInput>,
) -> (StatusCode, Json<Value>) {
    debug!(email = %body.email, "POST /auth/login");

    let req = LoginRequest {
        email:       body.email.clone(),
        password:    body.password,
        device_hint: body.device_hint.unwrap_or_default(),
        ip_address:  body.ip_address.unwrap_or_default(),
        user_agent:  body.user_agent.unwrap_or_default(),
    };

    let mut client = state.auth_grpc;
    match client.login(req).await {
        Ok(r) => {
            info!(email = %body.email, "POST /auth/login ← 200");
            (StatusCode::OK, Json(json!({
                "access_token":      r.access_token,
                "refresh_jti":       r.refresh_jti,
                "expires_in":        r.expires_in,
                "db_connection_url": r.db_connection_url,
                "user": r.user.map(|u| json!({
                    "user_id":   u.user_id,
                    "email":     u.email,
                    "username":  u.username,
                    "role":      u.role,
                    "status":    u.status,
                    "tenant_id": u.tenant_id,
                })),
            })))
        }
        Err(status) => {
            let (code, body) = grpc_to_http(status);
            (code, Json(body))
        }
    }
}

// ── Logout ────────────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/auth/logout",
    request_body = LogoutInput,
    responses(
        (status = 200, description = "Logged out",       body = Value),
        (status = 400, description = "Bad request",      body = Value),
        (status = 500, description = "Internal error",   body = Value),
    ),
    tag = "Auth"
)]
pub async fn logout(
    State(state): State<AppState>,
    Json(body): Json<LogoutInput>,
) -> (StatusCode, Json<Value>) {
    debug!(refresh_jti = %body.refresh_jti, "POST /auth/logout");

    let req = LogoutRequest {
        refresh_jti:  body.refresh_jti,
        access_token: body.access_token.unwrap_or_default(),
    };

    let mut client = state.auth_grpc;
    match client.logout(req).await {
        Ok(r) => {
            info!("POST /auth/logout ← 200 success={}", r.success);
            (StatusCode::OK, Json(json!({ "success": r.success })))
        }
        Err(status) => {
            let (code, body) = grpc_to_http(status);
            (code, Json(body))
        }
    }
}

// ── Me ────────────────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/auth/me",
    responses(
        (status = 200, description = "Current user info", body = Value),
        (status = 401, description = "Not authenticated", body = Value),
    ),
    tag = "Auth"
)]
pub async fn me(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<Value>) {
    debug!(username = %auth_user.username, "GET /auth/me");

    // Datos frescos de la BD: email/username pueden haber cambiado después de
    // emitirse el token (p.ej. desde "Guardar perfil"). El token solo se usa de
    // respaldo si el gRPC falla, para no tumbar la sesión.
    let mut client = state.auth_grpc;
    match client.get_user(GetUserRequest { user_id: auth_user.user_id.clone() }).await {
        Ok(u) => (StatusCode::OK, Json(json!({
            "user_id":   u.user_id,
            "email":     u.email,
            "username":  u.username,
            "full_name": u.full_name,
            "role":      u.role.clone(),
            "roles":     [u.role],
            "tenant_id": u.tenant_id,
        }))),
        Err(_) => (StatusCode::OK, Json(json!({
            "user_id":   auth_user.user_id,
            "username":  auth_user.username,
            "role":      auth_user.role.clone(),
            "roles":     [auth_user.role],
            "tenant_id": auth_user.tenant_id,
        }))),
    }
}

// ── Forgot password ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct ForgotPasswordInput {
    pub email: String,
}

/// Mensaje de confirmación cuando el correo SÍ está registrado y se envió el enlace.
const FORGOT_SENT_MSG: &str =
    "Te enviamos un enlace a tu correo para restablecer tu contraseña.";

#[utoipa::path(
    post,
    path = "/auth/forgot-password",
    request_body = ForgotPasswordInput,
    responses(
        (status = 200, description = "Respuesta genérica (siempre 200)", body = Value),
    ),
    tag = "Auth"
)]
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(body): Json<ForgotPasswordInput>,
) -> (StatusCode, Json<Value>) {
    debug!(email = %body.email, "POST /auth/forgot-password");

    let mut client = state.auth_grpc.clone();
    match client.request_password_reset(RequestPasswordResetRequest { email: body.email.clone() }).await {
        // Cuenta registrada (y activa): se manda el correo y se confirma al usuario.
        Ok(r) if r.found && !r.token.is_empty() => {
            let dash = std::env::var("DASHBOARD_URL")
                .ok().filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "https://dashboard.soflib.com".to_string());
            let reset_link = format!("{}/reset-password?token={}", dash.trim_end_matches('/'), r.token);
            let to     = if r.email.trim().is_empty() { body.email.clone() } else { r.email.clone() };
            let nombre = if r.full_name.trim().is_empty() { to.clone() } else { r.full_name.clone() };
            tokio::spawn(async move {
                let vars = [("nombre", nombre.as_str()), ("reset_link", reset_link.as_str())];
                match crate::infrastructure::email::outlook::send_template(&to, "password_reset", &vars).await {
                    Ok(_)  => info!(%to, "correo de recuperación enviado"),
                    Err(e) => warn!(%to, error = %e, "correo de recuperación falló"),
                }
            });
            (StatusCode::OK, Json(json!({ "found": true, "message": FORGOT_SENT_MSG })))
        }
        // No hay cuenta registrada (o no está activa) con ese correo → 404 para que el
        // front muestre "correo no registrado".
        Ok(_) => (StatusCode::NOT_FOUND, Json(json!({
            "error": "No hay una cuenta registrada con ese correo."
        }))),
        // Error de transporte/servidor con medusa.
        Err(status) => {
            warn!(error = %status, "request_password_reset gRPC error");
            (StatusCode::BAD_GATEWAY, Json(json!({
                "error": "No se pudo procesar la solicitud. Inténtalo de nuevo."
            })))
        }
    }
}

// ── Reset password ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct ResetPasswordInput {
    pub token:        String,
    pub new_password: String,
}

#[utoipa::path(
    post,
    path = "/auth/reset-password",
    request_body = ResetPasswordInput,
    responses(
        (status = 200, description = "Contraseña actualizada", body = Value),
        (status = 400, description = "Token inválido o expirado", body = Value),
    ),
    tag = "Auth"
)]
pub async fn reset_password(
    State(state): State<AppState>,
    Json(body): Json<ResetPasswordInput>,
) -> (StatusCode, Json<Value>) {
    debug!("POST /auth/reset-password");

    let mut client = state.auth_grpc.clone();
    match client.confirm_password_reset(ConfirmPasswordResetRequest {
        token:        body.token,
        new_password: body.new_password,
    }).await {
        Ok(_) => (StatusCode::OK, Json(json!({
            "message": "Contraseña actualizada. Ya puedes iniciar sesión."
        }))),
        // Token inválido/expirado o contraseña débil → SIEMPRE 400 (no 401): un 401
        // haría que el interceptor del dashboard intente refrescar/expulsar en vez de
        // mostrar "enlace inválido". Conservamos el mensaje del gRPC.
        Err(status) => {
            let (_code, body) = grpc_to_http(status);
            (StatusCode::BAD_REQUEST, Json(body))
        }
    }
}
