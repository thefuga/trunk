use std::sync::OnceLock;

static SYSTEM_PATH: OnceLock<String> = OnceLock::new();

/// System PATH that includes package-manager entries (Homebrew, MacPorts, etc.).
///
/// On macOS, GUI apps launched from Finder/Dock inherit launchd's minimal PATH
/// (`/usr/bin:/bin:/usr/sbin:/sbin`), which excludes tools installed by package
/// managers. Apple's `/usr/libexec/path_helper` reads `/etc/paths` and
/// `/etc/paths.d/*` to construct the full system PATH — the same one login
/// shells start with before any user dotfile customization.
///
/// Returns the inherited PATH unchanged on non-macOS or if path_helper fails.
pub fn system_path() -> &'static str {
    SYSTEM_PATH.get_or_init(resolve)
}

fn resolve() -> String {
    #[cfg(target_os = "macos")]
    if let Some(path) = from_path_helper() {
        return path;
    }
    std::env::var("PATH").unwrap_or_default()
}

#[cfg(target_os = "macos")]
fn from_path_helper() -> Option<String> {
    let output = std::process::Command::new("/usr/libexec/path_helper")
        .arg("-s")
        .stdin(std::process::Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    parse_path_helper_output(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(any(target_os = "macos", test))]
fn parse_path_helper_output(stdout: &str) -> Option<String> {
    // Output format: PATH="<paths>"; export PATH;\n
    let start = stdout.find('"')? + 1;
    let end = start + stdout[start..].find('"')?;
    Some(stdout[start..end].to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sh_format() {
        let input = r#"PATH="/usr/bin:/opt/homebrew/bin:/usr/sbin"; export PATH;"#;
        assert_eq!(
            parse_path_helper_output(input).as_deref(),
            Some("/usr/bin:/opt/homebrew/bin:/usr/sbin")
        );
    }

    #[test]
    fn parses_csh_format() {
        // path_helper -c outputs: setenv PATH "/usr/bin:/bin";
        let input = r#"setenv PATH "/usr/bin:/bin";"#;
        assert_eq!(
            parse_path_helper_output(input).as_deref(),
            Some("/usr/bin:/bin")
        );
    }

    #[test]
    fn returns_none_on_empty() {
        assert!(parse_path_helper_output("").is_none());
    }

    #[test]
    fn returns_none_on_no_quotes() {
        assert!(parse_path_helper_output("PATH=/usr/bin").is_none());
    }

    #[test]
    fn returns_none_on_single_quote() {
        assert!(parse_path_helper_output(r#"PATH="/usr/bin"#).is_none());
    }
}
