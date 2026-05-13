// Programa...: services::clients::project_images
// Descripción: Imágenes del proyecto para el portal de clientes
// Origen.....: Cte_Imagenes.aspx.cs
// Nota: listado de archivos bloqueado hasta decidir backend de almacenamiento (CC-5)

use crate::dal::proyectos;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

pub struct ProjectImages {
    pub directorio: String,
    pub archivos:   Vec<String>,
}

pub async fn get_images(pool: &PgPool, proyecto: i32) -> Result<ProjectImages, ReturnCode> {
    let dir = proyectos::dir_proyecto(pool, proyecto).await?;
    Ok(ProjectImages {
        directorio: dir,
        archivos:   vec![],
    })
}
