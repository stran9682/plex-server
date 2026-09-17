use crate::iroh_runtime::IrohRuntime;

#[tauri::command]
pub async fn import_ticket(
    ticket: String,
    state: tauri::State<'_, IrohRuntime>,
) -> Result<(), String> {
    let iroh_runtime = state.inner();

    iroh_runtime
        .import_ticket(ticket)
        .await
        .map_err(|err| err.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn add_remote_store(
    endpoint: String,
    topic: String,
    state: tauri::State<'_, IrohRuntime>,
) -> Result<(), String> {
    let iroh_runtime = state.inner();

    iroh_runtime
        .add_remote_store(endpoint, topic)
        .await
        .map_err(|err| err.to_string())?;

    Ok(())
}
