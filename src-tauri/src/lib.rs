mod ai;
mod lcu;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            lcu::get_lcu_status,
            lcu::get_live_draft_state,
            lcu::get_champion_masterdata,
            lcu::get_my_champion_mastery,
            lcu::get_lcu_current_summoner_debug,
            lcu::get_lcu_collections_spec,
            lcu::get_lcu_mastery_debug,
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            if let Some(win) = app.get_webview_window("main") {
                win.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Maximum League application");
}
