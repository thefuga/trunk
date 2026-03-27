pub mod commands;
pub mod error;
pub mod git;
pub mod state;
pub mod watcher;

use state::{CommitCache, RepoState, RunningOp};
use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::Emitter;
use watcher::WatcherState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let find = MenuItemBuilder::with_id("find", "Find")
                .accelerator("CmdOrCtrl+F")
                .build(app)?;

            let app_menu = SubmenuBuilder::new(app, "Trunk")
                .about(None)
                .separator()
                .quit()
                .build()?;

            let edit_menu = SubmenuBuilder::new(app, "Edit")
                .undo()
                .redo()
                .separator()
                .cut()
                .copy()
                .paste()
                .select_all()
                .separator()
                .item(&find)
                .build()?;

            let view_menu = SubmenuBuilder::new(app, "View").fullscreen().build()?;

            let window_menu = SubmenuBuilder::new(app, "Window").minimize().build()?;

            let menu = MenuBuilder::new(app)
                .item(&app_menu)
                .item(&edit_menu)
                .item(&view_menu)
                .item(&window_menu)
                .build()?;

            app.set_menu(menu)?;

            app.on_menu_event(|app, event| {
                if event.id().as_ref() == "find" {
                    let _ = app.emit("search-toggle", ());
                }
            });

            Ok(())
        })
        .manage(RepoState(Default::default()))
        .manage(CommitCache(Default::default()))
        .manage(RunningOp(Default::default()))
        .manage(WatcherState(Default::default()))
        .invoke_handler(tauri::generate_handler![
            commands::repo::open_repo,
            commands::repo::close_repo,
            commands::repo::force_close_repo,
            commands::history::get_commit_graph,
            commands::history::refresh_commit_graph,
            commands::history::search_commits,
            commands::branches::list_refs,
            commands::branches::resolve_ref,
            commands::branches::checkout_branch,
            commands::branches::fast_forward_to,
            commands::branches::create_branch,
            commands::branches::delete_branch,
            commands::branches::rename_branch,
            commands::staging::get_dirty_counts,
            commands::staging::get_status,
            commands::staging::stage_file,
            commands::staging::unstage_file,
            commands::staging::stage_all,
            commands::staging::unstage_all,
            commands::staging::discard_file,
            commands::staging::discard_all,
            commands::staging::stage_hunk,
            commands::staging::unstage_hunk,
            commands::staging::discard_hunk,
            commands::staging::stage_lines,
            commands::staging::unstage_lines,
            commands::staging::discard_lines,
            commands::commit::create_commit,
            commands::commit::amend_commit,
            commands::commit::get_head_commit_message,
            commands::diff::diff_unstaged,
            commands::diff::diff_staged,
            commands::diff::diff_commit,
            commands::diff::get_commit_detail,
            commands::stash::list_stashes,
            commands::stash::stash_save,
            commands::stash::stash_pop,
            commands::stash::stash_apply,
            commands::stash::stash_drop,
            commands::commit_actions::checkout_commit,
            commands::commit_actions::create_tag,
            commands::commit_actions::delete_tag,
            commands::commit_actions::cherry_pick,
            commands::commit_actions::revert_commit,
            commands::commit_actions::reset_to_commit,
            commands::commit_actions::undo_commit,
            commands::commit_actions::redo_commit,
            commands::commit_actions::check_undo_available,
            commands::remote::git_fetch,
            commands::remote::git_pull,
            commands::remote::git_push,
            commands::remote::cancel_remote_op,
            commands::operation_state::get_operation_state,
            commands::operation_state::merge_continue,
            commands::operation_state::merge_abort,
            commands::operation_state::rebase_continue,
            commands::operation_state::rebase_skip,
            commands::operation_state::rebase_abort,
            commands::operation_state::merge_branch,
            commands::operation_state::rebase_branch,
            commands::merge_editor::get_merge_sides,
            commands::merge_editor::save_merge_result,
            commands::interactive_rebase::get_rebase_todo,
            commands::interactive_rebase::get_fork_point,
            commands::interactive_rebase::start_interactive_rebase,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
