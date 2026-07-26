mod diagnostics;
mod transport;

use diagnostics::{DiagnosticInfo, DiagnosticLog};
use serde_json::Value;
use std::sync::Arc;
use tauri::{Manager, State};
use transport::{CoreLaunchTarget, CoreStatus, CoreSupervisor, DesktopError};

#[tauri::command]
fn core_status(supervisor: State<'_, Arc<CoreSupervisor>>) -> CoreStatus {
    supervisor.status()
}

#[tauri::command]
fn diagnostics_info(diagnostics: State<'_, Arc<DiagnosticLog>>) -> DiagnosticInfo {
    diagnostics.info()
}

#[tauri::command]
fn core_configure(
    supervisor: State<'_, Arc<CoreSupervisor>>,
    target: CoreLaunchTarget,
) -> Result<CoreStatus, DesktopError> {
    supervisor.configure(target)
}

#[tauri::command]
async fn core_start(
    supervisor: State<'_, Arc<CoreSupervisor>>,
) -> Result<CoreStatus, DesktopError> {
    let supervisor = supervisor.inner().clone();
    tauri::async_runtime::spawn_blocking(move || supervisor.start())
        .await
        .map_err(|_| DesktopError::host_task_failed())?
}

#[tauri::command]
async fn core_request(
    supervisor: State<'_, Arc<CoreSupervisor>>,
    method: String,
    params: Value,
) -> Result<Value, DesktopError> {
    let supervisor = supervisor.inner().clone();
    tauri::async_runtime::spawn_blocking(move || supervisor.request(&method, params))
        .await
        .map_err(|_| DesktopError::host_task_failed())?
}

#[tauri::command]
async fn core_shutdown(
    supervisor: State<'_, Arc<CoreSupervisor>>,
) -> Result<CoreStatus, DesktopError> {
    let supervisor = supervisor.inner().clone();
    tauri::async_runtime::spawn_blocking(move || supervisor.shutdown())
        .await
        .map_err(|_| DesktopError::host_task_failed())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let diagnostics = Arc::new(DiagnosticLog::new(app.path().app_log_dir()?));
            diagnostics.app_started(env!("CARGO_PKG_VERSION"));
            let supervisor = Arc::new(
                CoreSupervisor::discover(diagnostics.clone())
                    .expect("failed to resolve GitNova Core executable location"),
            );
            app.manage(diagnostics);
            app.manage(supervisor);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            diagnostics_info,
            core_status,
            core_configure,
            core_start,
            core_request,
            core_shutdown
        ])
        .build(tauri::generate_context!())
        .expect("failed to build GitNova Desktop Host");
    app.run(|app_handle, event| {
        if matches!(
            event,
            tauri::RunEvent::Exit | tauri::RunEvent::ExitRequested { .. }
        ) {
            let supervisor = app_handle.state::<Arc<CoreSupervisor>>();
            let _ = supervisor.shutdown();
        }
    });
}
