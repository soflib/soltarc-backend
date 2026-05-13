// Programa...: GnUsuarios
// Descripción: Tabla de usuarios de un grupo de negocio
// Origen.....: eGnUsuarios.cs

use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema, sqlx::FromRow)]
pub struct GnUsuarios {
    pub id: i32,
    pub user_id: String,
    pub grupo_negocio: i32,
    pub activo: bool,
    pub nivel: i32,
    pub opt_cte_1: bool,
    pub opt_cte_2: bool,
    pub opt_cte_3: bool,
    pub opt_cte_4: bool,
    pub opt_cte_5: bool,
    pub opt_cte_6: bool,
}
