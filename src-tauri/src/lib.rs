pub mod commands;
pub mod error;
pub mod git;
pub mod shell_env;
pub mod state;
pub mod watcher;

use state::{CommitCache, RepoState, ReviewSessionsState, RunningOp};
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

            // View → Start/End Code Review menu item; emits `review-toggle` so the
            // frontend can flip review mode. No keyboard accelerator — user UAT
            // surfaced a clash with launcher-tool shortcuts (Phase 72 gap closure).
            let review_item =
                MenuItemBuilder::with_id("review-toggle", "Start/End Code Review").build(app)?;

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

            let view_menu = SubmenuBuilder::new(app, "View")
                .item(&review_item)
                .fullscreen()
                .build()?;

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
                } else if event.id().as_ref() == "review-toggle" {
                    let _ = app.emit("review-toggle", ());
                }
            });

            Ok(())
        })
        .manage(RepoState(Default::default()))
        .manage(CommitCache(Default::default()))
        .manage(RunningOp(Default::default()))
        .manage(WatcherState(Default::default()))
        .manage(ReviewSessionsState(Default::default()))
        .invoke_handler(tauri::generate_handler![
            commands::repo::open_repo,
            commands::repo::close_repo,
            commands::repo::force_close_repo,
            commands::fs::validate_recent_path,
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
            commands::staging::stage_files,
            commands::staging::unstage_files,
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
            commands::diff::list_commit_files,
            commands::diff::diff_commit_file,
            commands::diff::get_commit_detail,
            commands::stash::list_stashes,
            commands::stash::stash_save,
            commands::stash::stash_pop,
            commands::stash::stash_apply,
            commands::stash::stash_drop,
            commands::review::start_review_session,
            commands::review::resume_review_session,
            commands::review::end_review_session,
            commands::review::get_review_session_status,
            commands::review::seed_review_range,
            commands::review::add_review_commit,
            commands::review::remove_review_commit,
            commands::review::list_session_commits,
            commands::review::add_comment,
            commands::review::save_draft_comment,
            commands::review::add_commit_comment,
            commands::review::edit_comment,
            commands::review::delete_comment,
            commands::review::list_session_comments,
            commands::review::resolve_session_comments,
            commands::review::generate_review_doc,
            commands::commit_actions::checkout_commit,
            commands::commit_actions::create_tag,
            commands::commit_actions::delete_tag,
            commands::commit_actions::cherry_pick,
            commands::commit_actions::revert_commit_begin,
            commands::commit_actions::revert_continue,
            commands::commit_actions::revert_abort,
            commands::commit_actions::reset_to_commit,
            commands::commit_actions::undo_commit,
            commands::commit_actions::redo_commit,
            commands::commit_actions::check_undo_available,
            commands::remote::git_fetch,
            commands::remote::git_fetch_background,
            commands::remote::git_pull,
            commands::remote::git_push,
            commands::remote::delete_remote_branch,
            commands::remote::cancel_remote_op,
            commands::operation_state::get_operation_state,
            commands::operation_state::get_merge_message,
            commands::operation_state::merge_continue,
            commands::operation_state::merge_abort,
            commands::operation_state::rebase_continue,
            commands::operation_state::rebase_skip,
            commands::operation_state::rebase_abort,
            commands::operation_state::merge_branch_begin,
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
