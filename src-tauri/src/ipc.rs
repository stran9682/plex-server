use std::path::PathBuf;

use crate::{iroh_runtime::IrohRuntime, Error};

#[tauri::command]
pub async fn import_ticket(
    ticket: String,
    state: tauri::State<'_, IrohRuntime>,
) -> Result<(), Error> {
    let iroh_runtime = state.inner();

    iroh_runtime.import_ticket(ticket).await?;

    Ok(())
}

#[tauri::command]
pub async fn add_dir(
    file_path: PathBuf,
    namespace: Option<String>,
    state: tauri::State<'_, IrohRuntime>,
) -> Result<(), Error> {
    let iroh_runtime = state.inner();

    iroh_runtime.add_dir(file_path, namespace).await?;

    Ok(())
}

#[tauri::command]
pub async fn add_remote_store(
    endpoint: String,
    namespace: String,
    state: tauri::State<'_, IrohRuntime>,
) -> Result<(), Error> {
    let iroh_runtime = state.inner();

    iroh_runtime.add_remote_store(endpoint, namespace).await?;

    Ok(())
}

#[tauri::command]
pub async fn get_authorized_videos(
    namespace: String,
    state: tauri::State<'_, IrohRuntime>,
) -> Result<Option<Vec<String>>, Error> {
    let iroh_runtime = state.inner();

    Ok(iroh_runtime.get_authorized_videos(namespace).await?)
}

#[tauri::command]
pub async fn start_adding_topic_peers(
    namespace: String,
    state: tauri::State<'_, IrohRuntime>,
) -> Result<(), Error> {
    let iroh_runtime = state.inner();
    Ok(iroh_runtime.start_adding_topic_peers(namespace).await?)
}

#[tauri::command]
pub async fn stop_adding_topic_peers(
    namespace: String,
    state: tauri::State<'_, IrohRuntime>,
) -> Result<bool, Error> {
    let iroh_runtime = state.inner();
    Ok(iroh_runtime.stop_adding_topic_peers(namespace))
}
