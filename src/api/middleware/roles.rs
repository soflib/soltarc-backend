use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Populated by `require_auth` and stored in request extensions.
/// All role middleware reads from here — no extra gRPC call needed.
#[derive(Clone, Debug)]
pub struct AuthUser {
    pub user_id:   String,
    pub username:  String,
    pub role:      String,
    pub tenant_id: String,
}

impl AuthUser {
    fn is_admin(&self) -> bool {
        self.role.eq_ignore_ascii_case("Admin")
    }

    /// Returns true if the user is Admin OR has the given role.
    fn has_role(&self, role: &str) -> bool {
        self.is_admin() || self.role.eq_ignore_ascii_case(role)
    }
}

fn forbidden() -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(json!({ "error": "insufficient permissions" })),
    )
        .into_response()
}

fn no_user() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "authentication required" })),
    )
        .into_response()
}

// ── Role guards ────────────────────────────────────────────────────────────────

/// Roles: Admin
/// Used by: user/tenant/password management, system security.
pub async fn require_admin(request: Request, next: Next) -> Response {
    match request.extensions().get::<AuthUser>() {
        Some(u) if u.is_admin() => next.run(request).await,
        Some(_) => forbidden(),
        None => no_user(),
    }
}

/// Roles: Admin | Finanzas
/// Used by: CatCenCosto, CatProvee, CatTpoCost, Egresos, Ingresos,
///          SaldosBanco, Finanzas operations, ReportesFinan, FlujoCaja.
pub async fn require_finanzas(request: Request, next: Next) -> Response {
    match request.extensions().get::<AuthUser>() {
        Some(u) if u.has_role("Finanzas") => next.run(request).await,
        Some(_) => forbidden(),
        None => no_user(),
    }
}

/// Roles: Admin | Arquitecto
/// Used by: CatGral, CatUnid, AccRapMgr, ConfigMGR, GposNeg, GNUsuarios,
///          Proyectos, DetalleProyecto, XRef, PlanDeObra, PlanSemanal,
///          PPTOaProy, PPTOPart, Presupuesto.
pub async fn require_arquitecto(request: Request, next: Next) -> Response {
    match request.extensions().get::<AuthUser>() {
        Some(u) if u.has_role("Arquitecto") => next.run(request).await,
        Some(_) => forbidden(),
        None => no_user(),
    }
}

/// Roles: Admin | Arquitecto | Finanzas
/// Used by: CatCtes (Clients catalog), CatCtosEstim (Estimated Costs).
pub async fn require_arquitecto_or_finanzas(request: Request, next: Next) -> Response {
    match request.extensions().get::<AuthUser>() {
        Some(u) if u.has_role("Arquitecto") || u.has_role("Finanzas") => {
            next.run(request).await
        }
        Some(_) => forbidden(),
        None => no_user(),
    }
}

/// Roles: Admin | Reportes | Finanzas
/// Used by: ReprtesPPTO, ReporteProy (project reports and budget reports).
/// Finanzas is included because estado_de_cuenta lives in this route group.
pub async fn require_reportes(request: Request, next: Next) -> Response {
    match request.extensions().get::<AuthUser>() {
        Some(u) if u.has_role("Reportes") || u.has_role("Finanzas") => {
            next.run(request).await
        }
        Some(_) => forbidden(),
        None => no_user(),
    }
}
