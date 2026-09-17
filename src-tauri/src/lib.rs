use anyhow::Context;
use sea_orm::{Database, DbErr};
use tauri::Manager;

use crate::iroh_runtime::IrohRuntime;

mod access_list;
mod discovery;
mod entities;
mod ipc;
mod iroh;
mod iroh_runtime;
mod protocol;
mod store;

pub const ALPN: &[u8] = b"gate";
pub const DISCOVERY_ALPN: &[u8] = b"discovery";

#[repr(u8)]
#[derive(Debug)]
pub enum Status {
    Denied,
    Allowed,
    FileNotFound,
    ResourceNotFound,
    UnknownError,
}

impl TryFrom<u8> for Status {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Status::Denied),
            1 => Ok(Status::Allowed),
            2 => Ok(Status::FileNotFound),
            3 => Ok(Status::ResourceNotFound),
            _ => Ok(Status::UnknownError),
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    DatabaseErr(#[from] DbErr),

    #[error("Data has invalid format, {0}")]
    InputErr(String),

    #[error("Iroh had an error, {0}")]
    IrohErr(String),
}

#[derive(serde::Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
enum ErrorKind {
    DatabaseErr(String),
    IrohErr(String),
    InputErr(String),
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let error_message = self.to_string();
        let error_kind = match self {
            Self::DatabaseErr(_) => ErrorKind::DatabaseErr(error_message),
            Self::IrohErr(_) => ErrorKind::IrohErr(error_message),
            Self::InputErr(_) => ErrorKind::InputErr(error_message),
        };
        error_kind.serialize(serializer)
    }
}

async fn setup(app_handle: tauri::AppHandle) -> anyhow::Result<()> {
    let path = app_handle.path().app_data_dir()?;
    let db = Database::connect(format!(
        "sqlite://{}db.sqlite?mode=rwc",
        path.to_string_lossy()
    ))
    .await?;

    let iroh = IrohRuntime::new(db).await?;
    app_handle.manage(iroh);

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = setup(app_handle).await {
                    eprintln!("failed, {}", e);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::import_ticket,
            ipc::add_remote_store,
            ipc::get_authorized_videos,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
