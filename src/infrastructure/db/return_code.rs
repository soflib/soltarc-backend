// Programa...: ReturnCode
// Descripción: Código de retorno de operaciones de datos
// Origen.....: ReturnCode.cs

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ReturnCode {
    pub codigo: i32,    // positivo = éxito, negativo = error
    pub afectado: i32,  // registros afectados o Id generado en alta
    pub mensaje: String,
}

impl Default for ReturnCode {
    fn default() -> Self {
        ReturnCode {
            codigo: 0,
            afectado: 0,
            mensaje: String::new(),
        }
    }
}

impl ReturnCode {
    pub fn ok(codigo: i32, afectado: i32, mensaje: &str) -> Self {
        ReturnCode { codigo, afectado, mensaje: mensaje.to_string() }
    }

    pub fn error(codigo: i32, mensaje: &str) -> Self {
        ReturnCode { codigo, afectado: 0, mensaje: mensaje.to_string() }
    }

    pub fn es_ok(&self) -> bool {
        self.codigo > 0
    }
}