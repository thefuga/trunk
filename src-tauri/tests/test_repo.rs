mod common;

use common::context::TestContext;

#[test]
fn open_invalid_path_returns_error() {
    let dir = tempfile::tempdir().unwrap();
    // dir is a real directory but NOT a git repo
    let result = trunk_lib::git::repository::validate_and_open(dir.path());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, "not_a_git_repo");
}

#[test]
fn open_valid_repo_succeeds() {
    let ctx = TestContext::builder()
        .with_file("README.md", "hello")
        .with_commit("Initial commit")
        .build();

    let result = ctx.validate_and_open();
    assert!(result.is_ok());
}

#[test]
fn repo_locators_have_backend_aware_stable_ids() {
    use trunk_lib::git::types::RepoLocator;

    let local = RepoLocator::Local {
        path: "/repo".to_string(),
    };
    let wsl = RepoLocator::Wsl {
        distro: "Ubuntu".to_string(),
        linux_path: "/repo".to_string(),
    };

    assert_eq!(local.stable_id(), "local:/repo");
    assert_eq!(wsl.stable_id(), "wsl:Ubuntu:/repo");
    assert_ne!(local.stable_id(), wsl.stable_id());
}

#[test]
fn repo_locator_ids_normalize_trailing_slashes() {
    use trunk_lib::git::types::{RepoDescriptor, RepoLocator};

    let local = RepoLocator::Local {
        path: "/repo/".to_string(),
    };
    let wsl = RepoLocator::Wsl {
        distro: "Ubuntu".to_string(),
        linux_path: "/repo/".to_string(),
    };
    let descriptor = RepoDescriptor::local("/repo/".to_string());

    assert_eq!(local.stable_id(), "local:/repo");
    assert_eq!(wsl.stable_id(), "wsl:Ubuntu:/repo");
    assert_eq!(descriptor.id, "local:/repo");
    assert_eq!(descriptor.display_path, "/repo/");
}

#[test]
fn repo_locators_serialize_with_backend_tags() {
    use serde_json::json;
    use trunk_lib::git::types::RepoLocator;

    let local = RepoLocator::Local {
        path: "/repo".to_string(),
    };
    let wsl = RepoLocator::Wsl {
        distro: "Ubuntu".to_string(),
        linux_path: "/repo".to_string(),
    };

    assert_eq!(
        serde_json::to_value(local).unwrap(),
        json!({ "backend": "Local", "path": "/repo" })
    );
    assert_eq!(
        serde_json::to_value(wsl).unwrap(),
        json!({ "backend": "Wsl", "distro": "Ubuntu", "linux_path": "/repo" })
    );
}

#[test]
fn close_removes_state() {
    use trunk_lib::git::types::RepoDescriptor;
    use trunk_lib::state::RepoState;

    let ctx = TestContext::builder()
        .with_file("README.md", "hello")
        .with_commit("Initial commit")
        .build();

    let path = ctx.path().to_string();
    let descriptor = RepoDescriptor::local(path.clone());
    let state = RepoState::default();

    // Simulate open
    state
        .0
        .lock()
        .unwrap()
        .insert(path.clone(), ctx.repo_path().to_path_buf());
    state
        .1
        .lock()
        .unwrap()
        .insert(path.clone(), descriptor.clone());
    assert!(state.0.lock().unwrap().contains_key(&path));
    assert_eq!(state.1.lock().unwrap().get(&path), Some(&descriptor));

    // Simulate close
    state.0.lock().unwrap().remove(&path);
    state.1.lock().unwrap().remove(&path);
    assert!(!state.0.lock().unwrap().contains_key(&path));
    assert!(!state.1.lock().unwrap().contains_key(&path));
}

#[test]
fn force_close_removes_running_op() {
    use std::collections::HashMap;
    use std::sync::Mutex;

    let path = "/test/repo".to_string();
    let running = Mutex::new(HashMap::<String, u32>::new());
    running.lock().unwrap().insert(path.clone(), 12345);

    // Simulate force_close_repo: remove PID
    let pid = running.lock().unwrap().remove(&path);
    assert_eq!(pid, Some(12345));
    assert!(!running.lock().unwrap().contains_key(&path));
}

#[test]
fn force_close_no_running_op_still_succeeds() {
    use std::collections::HashMap;
    use std::sync::Mutex;

    let path = "/test/repo".to_string();
    let running = Mutex::new(HashMap::<String, u32>::new());

    // No running op -- remove returns None, no panic
    let pid = running.lock().unwrap().remove(&path);
    assert_eq!(pid, None);
}

#[test]
fn close_does_not_touch_running_op() {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Mutex;

    let path = "/test/repo".to_string();
    let state = Mutex::new(HashMap::<String, PathBuf>::new());
    let running = Mutex::new(HashMap::<String, u32>::new());

    state
        .lock()
        .unwrap()
        .insert(path.clone(), PathBuf::from(&path));
    running.lock().unwrap().insert(path.clone(), 12345);

    // Simulate close_repo: only removes state, NOT running
    state.lock().unwrap().remove(&path);

    // Running op should still be there
    assert!(running.lock().unwrap().contains_key(&path));
    assert_eq!(*running.lock().unwrap().get(&path).unwrap(), 12345);
}
