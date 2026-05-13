// Programa...: Presupuesto
// Descripción: Tabla Presupuesto
// Origen.....: ePresupuesto.cs
//
// NOTA: El campo fecha viene como String en C# (no DateTime).
// Se mantiene igual — probablemente es un texto formateado como "YYYY/MM/DD".

use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema, sqlx::FromRow)]
pub struct Presupuesto {
    pub id: Option<i32>, 
    pub nombre: String,
    pub descripcion: String,
    pub direccion: String,
    pub comentarios: String,
    pub fecha: String,   // viene como String desde C#, no DateTime
    pub cliente: i32,
    pub activo: bool,
    pub estado: i32,
    pub pie_pagina: String,
    pub gn_id: i32,
    pub gn_user_id: i32,
}
