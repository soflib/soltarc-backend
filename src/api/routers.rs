use axum::{middleware, routing::{delete, get, post, put}, Router};
use crate::api::middleware::auth::require_auth;
use crate::api::middleware::roles::{
    require_admin,
    require_finanzas,
    require_arquitecto,
    require_reportes,
    require_arquitecto_or_finanzas,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use sqlx::PgPool;

use crate::infrastructure::db::app_state::AppState;
use crate::infrastructure::grpc::client::AuthGrpcClient;
use crate::api::handlers::health::health;
use crate::api::handlers::auth::{identity, tokens, sessions, users, roles, tenants, tenant_secrets};
use crate::api::handlers::catalog_g::general_cat;
use crate::api::handlers::catalog_g::clients;
use crate::api::handlers::catalog_g::mtto_center_costs;
use crate::api::handlers::catalog_g::providers;
use crate::api::handlers::catalog_g::quick_access;
use crate::api::handlers::clients as portal;
use crate::api::handlers::clients::project_images as han_proj_img;
use crate::api::handlers::operaciones::proyectos as han_proy;
use crate::api::handlers::operaciones::detalle_proyecto as han_det_proy;
use crate::api::handlers::operaciones::xref as han_xref;
use crate::api::handlers::ai::chat as han_ai_chat;
use crate::api::handlers::plan_obra::plan_obra as han_plan_obra;
use crate::api::handlers::plan_obra::plan_semanal as han_plan_sem;
use crate::api::handlers::finanzas::{
    egresos as han_egr,
    finanzas_reports as han_fin_rep,
    flujo_caja as han_flujo,
    ingresos as han_ing,
    saldos_bancos as han_sdo,
};

use crate::domain::models::catalog_g::CatalogGInput;
use crate::api::handlers::operaciones::proyectos::{ProyectosInput, GpoUsrInput};
use crate::api::handlers::operaciones::detalle_proyecto::{DetalleProyectosInput, ActualizaFechasInput, CopiaPartidaInput};
use crate::api::handlers::operaciones::xref::XrefInput;
use crate::api::handlers::catalog_g::clients::ClienteInput;
use crate::api::handlers::catalog_g::mtto_center_costs::CentroCostoInput;
use crate::api::handlers::catalog_g::providers::ProveedorInput;
use crate::api::handlers::catalog_g::quick_access::AccesosRapidosInput;
use crate::api::handlers::plan_obra::plan_obra::{PlanObraInput, CreaPlanInput};
use crate::api::handlers::ppto::costos_estimados as han_costos_est;
use crate::api::handlers::ppto::partidas_ppto as han_partidas;
use crate::api::handlers::ppto::presupuesto as han_presupuesto;
use crate::api::handlers::ppto::tipos_costo as han_tipos_costo;
use crate::api::handlers::ppto::unidades as han_unidades;
use crate::api::handlers::ppto::ppto_a_proyecto as han_ppto_proy;
use crate::api::handlers::reportes::financieros as han_rep_fin;
use crate::api::handlers::reportes::proyecto as han_rep_proy;
use crate::api::handlers::sistema::gn_grupos as han_gn_grupos;
use crate::api::handlers::sistema::gn_usuarios as han_gn_usuarios;
use crate::api::handlers::sistema::configura as han_configura;
use crate::api::handlers::sistema::seguridad as han_seguridad;

#[derive(OpenApi)]
#[openapi(
    info(title = "Arqeth Services", version = "0.1.0"),
    paths(
        crate::api::handlers::health::health,
        // General Catalogs
        crate::api::handlers::catalog_g::general_cat::alta,
        crate::api::handlers::catalog_g::general_cat::baja,
        crate::api::handlers::catalog_g::general_cat::cambios,
        crate::api::handlers::catalog_g::general_cat::consulta,
        crate::api::handlers::catalog_g::general_cat::obtiene_todo,
        crate::api::handlers::catalog_g::general_cat::obtiene_por_tipo,
        crate::api::handlers::catalog_g::general_cat::obtiene_tipos,
        // Clients
        crate::api::handlers::catalog_g::clients::alta,
        crate::api::handlers::catalog_g::clients::baja,
        crate::api::handlers::catalog_g::clients::cambios,
        crate::api::handlers::catalog_g::clients::consulta,
        crate::api::handlers::catalog_g::clients::nombre_cliente,
        crate::api::handlers::catalog_g::clients::obtiene_clientes,
        crate::api::handlers::catalog_g::clients::obtiene_tipos,
        // Cost Centers
        crate::api::handlers::catalog_g::mtto_center_costs::alta,
        crate::api::handlers::catalog_g::mtto_center_costs::baja,
        crate::api::handlers::catalog_g::mtto_center_costs::cambios,
        crate::api::handlers::catalog_g::mtto_center_costs::consulta,
        crate::api::handlers::catalog_g::mtto_center_costs::obtiene_centros,
        // Providers
        crate::api::handlers::catalog_g::providers::alta,
        crate::api::handlers::catalog_g::providers::baja,
        crate::api::handlers::catalog_g::providers::cambio,
        crate::api::handlers::catalog_g::providers::consulta,
        crate::api::handlers::catalog_g::providers::carga_proveedores,
        crate::api::handlers::catalog_g::providers::obtiene_tipos,
        crate::api::handlers::catalog_g::providers::obtiene_giros,
        // Quick Access
        crate::api::handlers::catalog_g::quick_access::lista_todos,
        crate::api::handlers::catalog_g::quick_access::cambios,
        crate::api::handlers::catalog_g::quick_access::consulta,
        // Client Portal
        crate::api::handlers::clients::dashboard::llena_det_proyectos,
        crate::api::handlers::clients::dashboard::total_ppto,
        crate::api::handlers::clients::project_tree::arbol_tareas,
        crate::api::handlers::clients::work_progress::work_progress,
        crate::api::handlers::clients::weekly_plan::weekly_plan,
        crate::api::handlers::clients::expense_detail::expense_detail,
        crate::api::handlers::clients::account_statement::account_statement,
        crate::api::handlers::clients::project_tasks::project_tasks,
        crate::api::handlers::clients::project_images::project_images,
        // Auth
        crate::api::handlers::auth::identity::register,
        crate::api::handlers::auth::identity::login,
        crate::api::handlers::auth::identity::logout,
        crate::api::handlers::auth::identity::me,
        crate::api::handlers::auth::tokens::refresh_token,
        crate::api::handlers::auth::tokens::validate_token,
        crate::api::handlers::auth::sessions::revoke_sessions,
        crate::api::handlers::auth::sessions::change_password,
        crate::api::handlers::auth::sessions::random_password,
        crate::api::handlers::auth::users::get_all_users,
        crate::api::handlers::auth::users::create_user,
        crate::api::handlers::auth::users::check_username,
        crate::api::handlers::auth::users::get_user,
        crate::api::handlers::auth::users::delete_user,
        crate::api::handlers::auth::users::update_user,
        crate::api::handlers::auth::users::lock_user,
        crate::api::handlers::auth::roles::list_roles,
        crate::api::handlers::auth::roles::get_user_roles,
        crate::api::handlers::auth::roles::assign_role,
        crate::api::handlers::auth::tenants::create_tenant,
        crate::api::handlers::auth::tenants::list_tenants,
        crate::api::handlers::auth::tenants::get_tenant,
        crate::api::handlers::auth::tenants::update_tenant,
        crate::api::handlers::auth::tenants::delete_tenant,
        crate::api::handlers::auth::tenant_secrets::set_tenant_db_url,
        crate::api::handlers::auth::tenant_secrets::get_tenant_db_url,
        // Finanzas
        crate::api::handlers::finanzas::egresos::alta,
        crate::api::handlers::finanzas::egresos::baja,
        crate::api::handlers::finanzas::egresos::cambios,
        crate::api::handlers::finanzas::egresos::consulta,
        crate::api::handlers::finanzas::egresos::lista,
        crate::api::handlers::finanzas::egresos::total,
        crate::api::handlers::finanzas::ingresos::alta,
        crate::api::handlers::finanzas::ingresos::baja,
        crate::api::handlers::finanzas::ingresos::cambios,
        crate::api::handlers::finanzas::ingresos::consulta,
        crate::api::handlers::finanzas::saldos_bancos::alta,
        crate::api::handlers::finanzas::saldos_bancos::baja,
        crate::api::handlers::finanzas::saldos_bancos::cambios,
        crate::api::handlers::finanzas::saldos_bancos::consulta,
        crate::api::handlers::finanzas::saldos_bancos::saldos_x_banco,
        crate::api::handlers::finanzas::saldos_bancos::saldos_todos,
        // Proyectos
        crate::api::handlers::operaciones::proyectos::lista_grupos,
        crate::api::handlers::operaciones::proyectos::usuarios_grupo,
        crate::api::handlers::operaciones::proyectos::alta,
        crate::api::handlers::operaciones::proyectos::baja,
        crate::api::handlers::operaciones::proyectos::cambio,
        crate::api::handlers::operaciones::proyectos::consulta,
        crate::api::handlers::operaciones::proyectos::lista,
        crate::api::handlers::operaciones::proyectos::gpo_usr_proy,
        crate::api::handlers::operaciones::proyectos::cliente_proy,
        crate::api::handlers::operaciones::proyectos::dir_proy,
        crate::api::handlers::operaciones::proyectos::total_ppto,
        // XRef
        crate::api::handlers::operaciones::xref::alta,
        crate::api::handlers::operaciones::xref::baja,
        crate::api::handlers::operaciones::xref::cambio,
        crate::api::handlers::operaciones::xref::consulta,
        crate::api::handlers::operaciones::xref::egresos_a_partidas,
        crate::api::handlers::operaciones::xref::egresos_no_asignados,
        // Detalle Proyecto
        crate::api::handlers::operaciones::detalle_proyecto::alta,
        crate::api::handlers::operaciones::detalle_proyecto::baja,
        crate::api::handlers::operaciones::detalle_proyecto::cambios,
        crate::api::handlers::operaciones::detalle_proyecto::consulta,
        crate::api::handlers::operaciones::detalle_proyecto::partidas_proyecto,
        crate::api::handlers::operaciones::detalle_proyecto::carga_tareas,
        crate::api::handlers::operaciones::detalle_proyecto::nodos_desc,
        crate::api::handlers::operaciones::detalle_proyecto::actualiza_fechas,
        crate::api::handlers::operaciones::detalle_proyecto::copia_contenido_partidas,
        crate::api::handlers::operaciones::detalle_proyecto::adiciona_partidas_faltantes,
        crate::api::handlers::operaciones::detalle_proyecto::import_csv,
        crate::api::handlers::operaciones::detalle_proyecto::part_no_destino,
        crate::api::handlers::operaciones::detalle_proyecto::carga_nivel,
        crate::api::handlers::operaciones::detalle_proyecto::copia_part_qry,
        // AI
        crate::api::handlers::ai::chat::chat,
        // Finanzas — Flujo de Caja y Reportes
        crate::api::handlers::finanzas::flujo_caja::consulta_flujo,
        crate::api::handlers::finanzas::finanzas_reports::trx_financieras,
        crate::api::handlers::finanzas::finanzas_reports::distribuye_egreso,
        crate::api::handlers::finanzas::finanzas_reports::egresos_proveedor_proyecto,
        crate::api::handlers::finanzas::finanzas_reports::ingresos_detalle,
        crate::api::handlers::finanzas::finanzas_reports::recibo_honorarios,
        // Plan de Obra
        crate::api::handlers::plan_obra::plan_obra::partida_upd_fecha,
        crate::api::handlers::plan_obra::plan_obra::partida_proyecto,
        crate::api::handlers::plan_obra::plan_obra::obtiene_avance,
        crate::api::handlers::plan_obra::plan_obra::existe_plan,
        crate::api::handlers::plan_obra::plan_obra::crea_plan,
        crate::api::handlers::plan_obra::plan_obra::descendientes_nodo,
        // Plan Semanal
        crate::api::handlers::plan_obra::plan_semanal::fechas,
        crate::api::handlers::plan_obra::plan_semanal::carga_partidas,
        // PPTO — Tipos de Costo
        crate::api::handlers::ppto::tipos_costo::alta,
        crate::api::handlers::ppto::tipos_costo::baja,
        crate::api::handlers::ppto::tipos_costo::cambio,
        crate::api::handlers::ppto::tipos_costo::consulta,
        crate::api::handlers::ppto::tipos_costo::carga_tipos,
        // PPTO — Unidades
        crate::api::handlers::ppto::unidades::alta,
        crate::api::handlers::ppto::unidades::baja,
        crate::api::handlers::ppto::unidades::cambio,
        crate::api::handlers::ppto::unidades::consulta,
        crate::api::handlers::ppto::unidades::obtiene_unidades,
        crate::api::handlers::ppto::unidades::carga_arbol,
        // PPTO — Costos Estimados
        crate::api::handlers::ppto::costos_estimados::alta,
        crate::api::handlers::ppto::costos_estimados::baja,
        crate::api::handlers::ppto::costos_estimados::cambios,
        crate::api::handlers::ppto::costos_estimados::consulta,
        crate::api::handlers::ppto::costos_estimados::carga_arbol,
        // PPTO — Presupuesto
        crate::api::handlers::ppto::presupuesto::alta,
        crate::api::handlers::ppto::presupuesto::baja,
        crate::api::handlers::ppto::presupuesto::cambio,
        crate::api::handlers::ppto::presupuesto::consulta,
        crate::api::handlers::ppto::presupuesto::carga_pptos,
        // PPTO — Partidas PPTO
        crate::api::handlers::ppto::partidas_ppto::alta,
        crate::api::handlers::ppto::partidas_ppto::borra,
        crate::api::handlers::ppto::partidas_ppto::cambio,
        crate::api::handlers::ppto::partidas_ppto::carga_partidas,
        crate::api::handlers::ppto::partidas_ppto::actualiza_nodo,
        crate::api::handlers::ppto::partidas_ppto::nuevo_nodo,
        crate::api::handlers::ppto::partidas_ppto::carga_2_nivel,
        // PPTO — Presupuesto a Proyecto
        crate::api::handlers::ppto::ppto_a_proyecto::consulta_numero_partidas,
        crate::api::handlers::ppto::ppto_a_proyecto::carga_nodos,
        crate::api::handlers::ppto::ppto_a_proyecto::crea_partidas_proyecto,
        crate::api::handlers::ppto::ppto_a_proyecto::obtiene_tipo_proyecto,
        // Reportes — Financieros
        crate::api::handlers::reportes::financieros::captura_diaria,
        crate::api::handlers::reportes::financieros::ingresos_reporte,
        crate::api::handlers::reportes::financieros::ingresos_cliente,
        crate::api::handlers::reportes::financieros::egresos_centros_costo,
        crate::api::handlers::reportes::financieros::egresos_proveedor,
        crate::api::handlers::reportes::financieros::egresos_reporte,
        crate::api::handlers::reportes::financieros::reporte_gral_egresos,
        // Reportes — Proyecto
        crate::api::handlers::reportes::proyecto::carga_partidas,
        crate::api::handlers::reportes::proyecto::arbol_tareas_proyecto,
        crate::api::handlers::reportes::proyecto::audita_xref,
        crate::api::handlers::reportes::proyecto::totales_ppto,
        crate::api::handlers::reportes::proyecto::ingresos,
        crate::api::handlers::reportes::proyecto::egresos,
        crate::api::handlers::reportes::proyecto::estado_de_cuenta,
        crate::api::handlers::reportes::proyecto::avance_obra,
        // Sistema — Grupos
        crate::api::handlers::sistema::gn_grupos::alta,
        crate::api::handlers::sistema::gn_grupos::baja,
        crate::api::handlers::sistema::gn_grupos::cambios,
        crate::api::handlers::sistema::gn_grupos::consulta,
        crate::api::handlers::sistema::gn_grupos::obtiene_todo,
        // Sistema — Usuarios
        crate::api::handlers::sistema::gn_usuarios::alta,
        crate::api::handlers::sistema::gn_usuarios::baja,
        crate::api::handlers::sistema::gn_usuarios::cambios,
        crate::api::handlers::sistema::gn_usuarios::consulta,
        crate::api::handlers::sistema::gn_usuarios::obtiene_todo,
        // Sistema — Configura
        crate::api::handlers::sistema::configura::cambia_configuracion,
        crate::api::handlers::sistema::configura::carga_configuracion,
        // Sistema — Seguridad
        crate::api::handlers::sistema::seguridad::carga_variables,
    ),
    components(schemas(
        CatalogGInput,
        ClienteInput,
        CentroCostoInput,
        ProveedorInput,
        AccesosRapidosInput,
        // Auth
        identity::RegisterInput,
        identity::LoginInput,
        identity::LogoutInput,
        tokens::RefreshInput,
        tokens::ValidateInput,
        sessions::RevokeSessionsInput,
        sessions::ChangePasswordInput,
        users::CreateUserInput,
        users::UpdateUserInput,
        users::LockUserInput,
        roles::AssignRoleInput,
        tenants::CreateTenantInput,
        tenants::UpdateTenantInput,
        tenant_secrets::SetTenantDbUrlInput,
        // Finanzas
        han_egr::EgresosInput,
        han_ing::IngresosInput,
        han_sdo::SaldosBancosInput,
        han_fin_rep::DistribuyeInput,
        ProyectosInput,
        GpoUsrInput,
        DetalleProyectosInput,
        ActualizaFechasInput,
        CopiaPartidaInput,
        XrefInput,
        PlanObraInput,
        CreaPlanInput,
        han_tipos_costo::TiposCostoInput,
        han_unidades::UnidadesInput,
        han_costos_est::CostosEstimadosInput,
        han_presupuesto::PresupuestoInput,
        han_partidas::PartidasPptoInput,
        han_partidas::ActualizaNodoInput,
        han_ppto_proy::CreaPptoProyInput,
        han_gn_grupos::GnGruposInput,
        han_gn_usuarios::GnUsuariosInput,
        han_configura::ConfiguraInput,
    )),
    tags(
        (name = "Sistema",          description = "Health check"),
        (name = "General Catalogs", description = "Catálogo general"),
        (name = "Clients",          description = "Catálogo de clientes"),
        (name = "Cost Centers",     description = "Centros de costo"),
        (name = "Providers",        description = "Catálogo de proveedores"),
        (name = "Quick Access",     description = "Botones de acceso rápido"),
        (name = "Client Portal",    description = "Portal de clientes"),
        (name = "Auth",             description = "Authentication & authorization"),
        (name = "Finanzas",         description = "Egresos, ingresos y saldos de bancos"),
        (name = "Proyectos",        description = "CRUD de proyectos y consultas relacionadas"),
        (name = "DetalleProyecto",  description = "Partidas, tareas y nodos de proyectos"),
        (name = "XRef",             description = "Asignación de egresos a partidas de proyecto"),
        (name = "AI",               description = "Asistente de inteligencia artificial"),
        (name = "PlanObra",         description = "Plan de obra: partidas y avance"),
        (name = "PlanSemanal",      description = "Plan semanal: fechas y partidas por semana"),
        (name = "PptoCatalogos",    description = "Catálogos de presupuesto: tipos de costo, unidades, costos estimados"),
        (name = "PptoPresupuestos", description = "Presupuestos: CRUD y partidas"),
        (name = "Reportes",         description = "Reportes financieros y de proyecto"),
        (name = "SistemaGrupos",    description = "Grupos de negocio"),
        (name = "SistemaUsuarios",  description = "Usuarios de grupos de negocio"),
        (name = "SistemaConfigura", description = "Configuración del sistema"),
        (name = "SistemaSeg",       description = "Variables de seguridad del sistema"),
    )
)]
struct ApiDoc;

pub fn build_router(postgres: PgPool, auth_grpc: AuthGrpcClient) -> Router {
    let state = AppState { postgres, auth_grpc };

    // ── Public ─────────────────────────────────────────────────────────────────
    let base = Router::new()
        .route("/health", get(health));

    let han_auth_public = Router::new()
        .route("/auth/register",      post(identity::register))
        .route("/auth/login",         post(identity::login))
        .route("/auth/token/refresh", post(tokens::refresh_token));

    // ── Any authenticated user ─────────────────────────────────────────────────
    // logout, token validation, session revoke — self-service, no role required.
    // AI chat and client portal are also open to any authenticated user.
    let han_auth_any = Router::new()
        .route("/auth/me",              get(identity::me))
        .route("/auth/logout",          post(identity::logout))
        .route("/auth/token/validate",  post(tokens::validate_token))
        .route("/auth/sessions/revoke", post(sessions::revoke_sessions));

    let han_ai = Router::new()
        .route("/ai/chat", post(han_ai_chat::chat));

    let han_client_portal = Router::new()
        .route("/clients/portal/dashboard",                      get(portal::dashboard::llena_det_proyectos))
        .route("/clients/portal/projects/{id}/ppto",             get(portal::dashboard::total_ppto))
        .route("/clients/portal/projects/{id}/tree",             get(portal::project_tree::arbol_tareas))
        .route("/clients/portal/projects/{id}/work-progress",    get(portal::work_progress::work_progress))
        .route("/clients/portal/projects/{id}/weekly-plan",      get(portal::weekly_plan::weekly_plan))
        .route("/clients/portal/projects/{id}/expense-detail",   get(portal::expense_detail::expense_detail))
        .route("/clients/portal/clients/{id}/account-statement", get(portal::account_statement::account_statement))
        .route("/clients/portal/projects/{id}/tasks",            get(portal::project_tasks::project_tasks))
        .route("/clients/portal/projects/{id}/images",           get(han_proj_img::project_images));

    let han_seguridad_routes = Router::new()
        .route("/sistema/seguridad", get(han_seguridad::carga_variables));

    // ── Admin only ─────────────────────────────────────────────────────────────
    // User management, tenant management, connection string, password admin.
    let han_auth_admin = Router::new()
        .route("/auth/password/random",     get(sessions::random_password))
        .route("/auth/password",            put(sessions::change_password))
        .route("/auth/users",               get(users::get_all_users))
        .route("/auth/users",               post(users::create_user))
        .route("/auth/users/check",         get(users::check_username))
        .route("/auth/users/{id}",          get(users::get_user))
        .route("/auth/users/{id}",          delete(users::delete_user))
        .route("/auth/users/{id}",          put(users::update_user))
        .route("/auth/users/{id}/lock",     put(users::lock_user))
        .route("/auth/roles",               get(roles::list_roles))
        .route("/auth/roles/user",          get(roles::get_user_roles))
        .route("/auth/roles/assign",        post(roles::assign_role))
        .route("/auth/tenants",             post(tenants::create_tenant))
        .route("/auth/tenants",             get(tenants::list_tenants))
        .route("/auth/tenants/{id}",        get(tenants::get_tenant))
        .route("/auth/tenants/{id}",        put(tenants::update_tenant))
        .route("/auth/tenants/{id}",        delete(tenants::delete_tenant))
        .route("/auth/tenants/{id}/db-url", get(tenant_secrets::get_tenant_db_url))
        .route("/auth/tenants/{id}/db-url", put(tenant_secrets::set_tenant_db_url))
        .route_layer(middleware::from_fn(require_admin));

    // ── Admin | Finanzas ───────────────────────────────────────────────────────
    // CatCenCosto, CatProvee, CatTpoCost, Egresos, Ingresos, SaldosBanco,
    // Finanzas operations, ReportesFinan, FlujoCaja.
    let han_cost_centers = Router::new()
        .route("/catalog/cost-centers",      post(mtto_center_costs::alta))
        .route("/catalog/cost-centers",      get(mtto_center_costs::obtiene_centros))
        .route("/catalog/cost-centers",      put(mtto_center_costs::cambios))
        .route("/catalog/cost-centers/{id}", get(mtto_center_costs::consulta))
        .route("/catalog/cost-centers/{id}", delete(mtto_center_costs::baja))
        .route_layer(middleware::from_fn(require_finanzas));

    let han_providers = Router::new()
        .route("/catalog/providers/tipos",  get(providers::obtiene_tipos))
        .route("/catalog/providers/giros",  get(providers::obtiene_giros))
        .route("/catalog/providers",        post(providers::alta))
        .route("/catalog/providers",        get(providers::carga_proveedores))
        .route("/catalog/providers",        put(providers::cambio))
        .route("/catalog/providers/{id}",   get(providers::consulta))
        .route("/catalog/providers/{id}",   delete(providers::baja))
        .route_layer(middleware::from_fn(require_finanzas));

    let han_tipos_costo_routes = Router::new()
        .route("/ppto/tipos-costo",      post(han_tipos_costo::alta))
        .route("/ppto/tipos-costo",      get(han_tipos_costo::carga_tipos))
        .route("/ppto/tipos-costo",      put(han_tipos_costo::cambio))
        .route("/ppto/tipos-costo/{id}", get(han_tipos_costo::consulta))
        .route("/ppto/tipos-costo/{id}", delete(han_tipos_costo::baja))
        .route_layer(middleware::from_fn(require_finanzas));

    let han_finanzas = Router::new()
        // Egresos
        .route("/finanzas/egresos/total",            get(han_egr::total))
        .route("/finanzas/egresos",                  post(han_egr::alta))
        .route("/finanzas/egresos",                  get(han_egr::lista))
        .route("/finanzas/egresos",                  put(han_egr::cambios))
        .route("/finanzas/egresos/{id}",             get(han_egr::consulta))
        .route("/finanzas/egresos/{id}",             delete(han_egr::baja))
        // Ingresos
        .route("/finanzas/ingresos",                 post(han_ing::alta))
        .route("/finanzas/ingresos",                 put(han_ing::cambios))
        .route("/finanzas/ingresos/{id}",            get(han_ing::consulta))
        .route("/finanzas/ingresos/{id}",            delete(han_ing::baja))
        // Saldos bancos
        .route("/finanzas/saldos/banco/{id}",        get(han_sdo::saldos_x_banco))
        .route("/finanzas/saldos",                   post(han_sdo::alta))
        .route("/finanzas/saldos",                   get(han_sdo::saldos_todos))
        .route("/finanzas/saldos",                   put(han_sdo::cambios))
        .route("/finanzas/saldos/{id}",              get(han_sdo::consulta))
        .route("/finanzas/saldos/{id}",              delete(han_sdo::baja))
        // Flujo de Caja
        .route("/finanzas/flujo-caja",               get(han_flujo::consulta_flujo))
        // Finanzas Reportes / operaciones
        .route("/finanzas/trx",                      get(han_fin_rep::trx_financieras))
        .route("/finanzas/egresos/{id}/distribuye",  put(han_fin_rep::distribuye_egreso))
        .route("/finanzas/reportes/egresos-proveedor",  get(han_fin_rep::egresos_proveedor_proyecto))
        .route("/finanzas/reportes/ingresos-detalle",   get(han_fin_rep::ingresos_detalle))
        .route("/finanzas/recibo-honorarios/{id}",   get(han_fin_rep::recibo_honorarios))
        .route_layer(middleware::from_fn(require_finanzas));

    let han_rep_fin_routes = Router::new()
        .route("/reportes/financieros/captura-diaria",        get(han_rep_fin::captura_diaria))
        .route("/reportes/financieros/ingresos",              get(han_rep_fin::ingresos_reporte))
        .route("/reportes/financieros/ingresos-cliente",      get(han_rep_fin::ingresos_cliente))
        .route("/reportes/financieros/egresos-centros-costo", get(han_rep_fin::egresos_centros_costo))
        .route("/reportes/financieros/egresos-proveedor",     get(han_rep_fin::egresos_proveedor))
        .route("/reportes/financieros/egresos",               get(han_rep_fin::egresos_reporte))
        .route("/reportes/financieros/egresos-gral",          get(han_rep_fin::reporte_gral_egresos))
        .route_layer(middleware::from_fn(require_finanzas));

    // ── Admin | Arquitecto ─────────────────────────────────────────────────────
    // CatGral, CatUnid, AccRapMgr, ConfigMGR, GposNeg, GNUsuarios,
    // Proyectos, DetalleProyecto, XRef, PlanDeObra, PlanSemanal,
    // PPTOaProy, PPTOPart, Presupuesto.
    let han_cat_general = Router::new()
        .route("/general/catalog-types",        get(general_cat::obtiene_tipos))
        .route("/general/catalogs/tipo/{tipo}", get(general_cat::obtiene_por_tipo))
        .route("/general/catalogs",             post(general_cat::alta))
        .route("/general/catalogs",             get(general_cat::obtiene_todo))
        .route("/general/catalogs",             put(general_cat::cambios))
        .route("/general/catalogs/{id}",        get(general_cat::consulta))
        .route("/general/catalogs/{id}",        delete(general_cat::baja))
        .route_layer(middleware::from_fn(require_arquitecto));

    let han_unidades_routes = Router::new()
        .route("/ppto/unidades/arbol", get(han_unidades::carga_arbol))
        .route("/ppto/unidades",       post(han_unidades::alta))
        .route("/ppto/unidades",       get(han_unidades::obtiene_unidades))
        .route("/ppto/unidades",       put(han_unidades::cambio))
        .route("/ppto/unidades/{id}",  get(han_unidades::consulta))
        .route("/ppto/unidades/{id}",  delete(han_unidades::baja))
        .route_layer(middleware::from_fn(require_arquitecto));

    let han_quick_access = Router::new()
        .route("/catalog/quick-access",      get(quick_access::lista_todos).put(quick_access::cambios))
        .route("/catalog/quick-access/{id}", get(quick_access::consulta))
        .route_layer(middleware::from_fn(require_arquitecto));

    let han_configura_routes = Router::new()
        .route("/sistema/configura", put(han_configura::cambia_configuracion))
        .route("/sistema/configura", get(han_configura::carga_configuracion))
        .route_layer(middleware::from_fn(require_arquitecto));

    let han_gn_grupos_routes = Router::new()
        .route("/sistema/grupos",      post(han_gn_grupos::alta))
        .route("/sistema/grupos",      get(han_gn_grupos::obtiene_todo))
        .route("/sistema/grupos",      put(han_gn_grupos::cambios))
        .route("/sistema/grupos/{id}", get(han_gn_grupos::consulta))
        .route("/sistema/grupos/{id}", delete(han_gn_grupos::baja))
        .route_layer(middleware::from_fn(require_arquitecto));

    let han_gn_usuarios_routes = Router::new()
        .route("/sistema/usuarios",      post(han_gn_usuarios::alta))
        .route("/sistema/usuarios",      get(han_gn_usuarios::obtiene_todo))
        .route("/sistema/usuarios",      put(han_gn_usuarios::cambios))
        .route("/sistema/usuarios/{id}", get(han_gn_usuarios::consulta))
        .route("/sistema/usuarios/{id}", delete(han_gn_usuarios::baja))
        .route_layer(middleware::from_fn(require_arquitecto));

    let han_proyectos = Router::new()
        .route("/operaciones/grupos",                       get(han_proy::lista_grupos))
        .route("/operaciones/grupos/{id}/usuarios",         get(han_proy::usuarios_grupo))
        .route("/operaciones/proyectos",                    post(han_proy::alta))
        .route("/operaciones/proyectos",                    get(han_proy::lista))
        .route("/operaciones/proyectos",                    put(han_proy::cambio))
        .route("/operaciones/proyectos/{id}",               get(han_proy::consulta))
        .route("/operaciones/proyectos/{id}",               delete(han_proy::baja))
        .route("/operaciones/proyectos/{id}/grupo-usuario", put(han_proy::gpo_usr_proy))
        .route("/operaciones/proyectos/{id}/cliente",       get(han_proy::cliente_proy))
        .route("/operaciones/proyectos/{id}/directorio",    get(han_proy::dir_proy))
        .route("/operaciones/proyectos/{id}/total-ppto",    get(han_proy::total_ppto))
        .route_layer(middleware::from_fn(require_arquitecto));

    let han_detalle_proyecto = Router::new()
        .route("/operaciones/detalle-proyecto/nodos-desc",  get(han_det_proy::nodos_desc))
        .route("/operaciones/detalle-proyecto/no-destino",  get(han_det_proy::part_no_destino))
        .route("/operaciones/detalle-proyecto/nivel2",      get(han_det_proy::carga_nivel))
        .route("/operaciones/detalle-proyecto/copia-qry",   get(han_det_proy::copia_part_qry))
        .route("/operaciones/detalle-proyecto/fechas",      put(han_det_proy::actualiza_fechas))
        .route("/operaciones/detalle-proyecto/import-csv",  post(han_det_proy::import_csv))
        .route("/operaciones/detalle-proyecto/copia",       post(han_det_proy::copia_contenido_partidas))
        .route("/operaciones/detalle-proyecto/adiciona",    post(han_det_proy::adiciona_partidas_faltantes))
        .route("/operaciones/detalle-proyecto",             post(han_det_proy::alta))
        .route("/operaciones/detalle-proyecto",             get(han_det_proy::partidas_proyecto))
        .route("/operaciones/detalle-proyecto",             put(han_det_proy::cambios))
        .route("/operaciones/detalle-proyecto/{id}",        get(han_det_proy::consulta))
        .route("/operaciones/detalle-proyecto/{id}",        delete(han_det_proy::baja))
        .route("/operaciones/detalle-proyecto/{proyecto}/tareas", get(han_det_proy::carga_tareas))
        .route_layer(middleware::from_fn(require_arquitecto));

    let han_xref = Router::new()
        .route("/operaciones/xref/no-asignados", get(han_xref::egresos_no_asignados))
        .route("/operaciones/xref",              post(han_xref::alta))
        .route("/operaciones/xref",              put(han_xref::cambio))
        .route("/operaciones/xref/{id}",         get(han_xref::consulta))
        .route("/operaciones/xref/{id}",         delete(han_xref::baja))
        .route("/operaciones/xref/{id}/egresos", get(han_xref::egresos_a_partidas))
        .route_layer(middleware::from_fn(require_arquitecto));

    let han_plan_obra_routes = Router::new()
        .route("/plan-obra/existe-plan",   get(han_plan_obra::existe_plan))
        .route("/plan-obra/descendientes", get(han_plan_obra::descendientes_nodo))
        .route("/plan-obra/crea-plan",     post(han_plan_obra::crea_plan))
        .route("/plan-obra/partidas",      put(han_plan_obra::partida_upd_fecha))
        .route("/plan-obra/partidas",      get(han_plan_obra::partida_proyecto))
        .route("/plan-obra/avance",        get(han_plan_obra::obtiene_avance))
        .route_layer(middleware::from_fn(require_arquitecto));

    let han_plan_sem_routes = Router::new()
        .route("/plan-semanal/fechas",   get(han_plan_sem::fechas))
        .route("/plan-semanal/partidas", get(han_plan_sem::carga_partidas))
        .route_layer(middleware::from_fn(require_arquitecto));

    let han_ppto_proy_routes = Router::new()
        .route("/ppto/a-proyecto/num-partidas",  get(han_ppto_proy::consulta_numero_partidas))
        .route("/ppto/a-proyecto/nodos",         get(han_ppto_proy::carga_nodos))
        .route("/ppto/a-proyecto/tipo-proyecto", get(han_ppto_proy::obtiene_tipo_proyecto))
        .route("/ppto/a-proyecto",               post(han_ppto_proy::crea_partidas_proyecto))
        .route_layer(middleware::from_fn(require_arquitecto));

    let han_partidas_routes = Router::new()
        .route("/ppto/partidas/nuevo-nodo", get(han_partidas::nuevo_nodo))
        .route("/ppto/partidas/nivel2",     get(han_partidas::carga_2_nivel))
        .route("/ppto/partidas",            post(han_partidas::alta))
        .route("/ppto/partidas",            get(han_partidas::carga_partidas))
        .route("/ppto/partidas",            put(han_partidas::cambio))
        .route("/ppto/partidas/{id}",       delete(han_partidas::borra))
        .route("/ppto/partidas/{id}/nodo",  put(han_partidas::actualiza_nodo))
        .route_layer(middleware::from_fn(require_arquitecto));

    let han_presupuesto_routes = Router::new()
        .route("/ppto/presupuestos",      post(han_presupuesto::alta))
        .route("/ppto/presupuestos",      get(han_presupuesto::carga_pptos))
        .route("/ppto/presupuestos",      put(han_presupuesto::cambio))
        .route("/ppto/presupuestos/{id}", get(han_presupuesto::consulta))
        .route("/ppto/presupuestos/{id}", delete(han_presupuesto::baja))
        .route_layer(middleware::from_fn(require_arquitecto));

    // ── Admin | Arquitecto | Finanzas ──────────────────────────────────────────
    // CatCtes (Clients catalog), CatCtosEstim (Estimated Costs).
    let han_clients = Router::new()
        .route("/catalog/clients/tipos",        get(clients::obtiene_tipos))
        .route("/catalog/clients/{id}/nombre",  get(clients::nombre_cliente))
        .route("/catalog/clients",              post(clients::alta))
        .route("/catalog/clients",              get(clients::obtiene_clientes))
        .route("/catalog/clients",              put(clients::cambios))
        .route("/catalog/clients/{id}",         get(clients::consulta))
        .route("/catalog/clients/{id}",         delete(clients::baja))
        .route_layer(middleware::from_fn(require_arquitecto_or_finanzas));

    let han_costos_est_routes = Router::new()
        .route("/ppto/costos-estimados",      post(han_costos_est::alta))
        .route("/ppto/costos-estimados",      get(han_costos_est::carga_arbol))
        .route("/ppto/costos-estimados",      put(han_costos_est::cambios))
        .route("/ppto/costos-estimados/{id}", get(han_costos_est::consulta))
        .route("/ppto/costos-estimados/{id}", delete(han_costos_est::baja))
        .route_layer(middleware::from_fn(require_arquitecto_or_finanzas));

    // ── Admin | Reportes | Finanzas ────────────────────────────────────────────
    // ReprtesPPTO, ReporteProy (budget + project reports).
    // Finanzas included because estado_de_cuenta lives in this group.
    let han_rep_proy_routes = Router::new()
        .route("/reportes/proyecto/partidas",      get(han_rep_proy::carga_partidas))
        .route("/reportes/proyecto/arbol",         get(han_rep_proy::arbol_tareas_proyecto))
        .route("/reportes/proyecto/audita-xref",   get(han_rep_proy::audita_xref))
        .route("/reportes/proyecto/totales-ppto",  get(han_rep_proy::totales_ppto))
        .route("/reportes/proyecto/ingresos",      get(han_rep_proy::ingresos))
        .route("/reportes/proyecto/egresos",       get(han_rep_proy::egresos))
        .route("/reportes/proyecto/estado-cuenta", get(han_rep_proy::estado_de_cuenta))
        .route("/reportes/proyecto/avance-obra",   get(han_rep_proy::avance_obra))
        .route_layer(middleware::from_fn(require_reportes));

    // ── Assemble protected router ──────────────────────────────────────────────
    // require_auth runs first for every route below (outermost layer).
    // Each sub-router's role layer runs after auth populates AuthUser.
    let protected = Router::new()
        // any authenticated
        .merge(han_auth_any)
        .merge(han_ai)
        .merge(han_client_portal)
        .merge(han_seguridad_routes)
        // admin only
        .merge(han_auth_admin)
        // admin | finanzas
        .merge(han_cost_centers)
        .merge(han_providers)
        .merge(han_tipos_costo_routes)
        .merge(han_finanzas)
        .merge(han_rep_fin_routes)
        // admin | arquitecto
        .merge(han_cat_general)
        .merge(han_unidades_routes)
        .merge(han_quick_access)
        .merge(han_configura_routes)
        .merge(han_gn_grupos_routes)
        .merge(han_gn_usuarios_routes)
        .merge(han_proyectos)
        .merge(han_detalle_proyecto)
        .merge(han_xref)
        .merge(han_plan_obra_routes)
        .merge(han_plan_sem_routes)
        .merge(han_ppto_proy_routes)
        .merge(han_partidas_routes)
        .merge(han_presupuesto_routes)
        // admin | arquitecto | finanzas
        .merge(han_clients)
        .merge(han_costos_est_routes)
        // admin | reportes | finanzas
        .merge(han_rep_proy_routes)
        // outermost: validates token and populates AuthUser for all above
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth));

    base
        .merge(han_auth_public)
        .merge(protected)
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", ApiDoc::openapi()),
        )
        .with_state(state)
}
