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
pub async fn add_remote_store(
    endpoint: String,
    topic: String,
    state: tauri::State<'_, IrohRuntime>,
) -> Result<(), Error> {
    let iroh_runtime = state.inner();

    iroh_runtime.add_remote_store(endpoint, topic).await?;

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
