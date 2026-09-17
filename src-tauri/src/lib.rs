use anyhow::Context;
use sea_orm::Database;
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
            ipc::add_remote_store
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
