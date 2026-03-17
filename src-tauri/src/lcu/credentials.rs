//! Discover LCU port and auth token (lockfile or process).

use std::process::Command;

/// Credentials to connect to the local League Client.
#[derive(Debug, Clone)]
pub struct LcuCredentials {
    pub port: u16,
    pub password: String,
}

/// Try to read credentials from the League Client process (Windows).
/// Parses wmic output for --app-port and --remoting-auth-token.
fn credentials_from_process_windows() -> Option<LcuCredentials> {
    let output = Command::new("wmic")
        .args([
            "PROCESS",
            "WHERE",
            "name='LeagueClientUx.exe'",
            "GET",
            "commandline",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let port = regex::Regex::new(r"--app-port=(\d+)")
        .ok()?
        .captures(&stdout)?
        .get(1)?
        .as_str()
        .parse::<u16>()
        .ok()?;
    let password = regex::Regex::new(r"--remoting-auth-token=([\w-]+)")
        .ok()?
        .captures(&stdout)?
        .get(1)?
        .as_str()
        .to_string();
    Some(LcuCredentials { port, password })
}

/// Try to read credentials from lockfile (path from env or default).
fn credentials_from_lockfile() -> Option<LcuCredentials> {
    let path = std::env::var("LEAGUE_LOCKFILE_PATH").ok().or_else(|| {
        #[cfg(windows)]
        {
            // Try Program Files and Program Files (x86)
            let pf = std::env::var("PROGRAMFILES").ok();
            let pf86 = std::env::var("PROGRAMFILES(X86)").ok();
            for base in [pf, pf86].into_iter().flatten() {
                let p = format!(r"{}\Riot Games\League of Legends\lockfile", base);
                if std::path::Path::new(&p).exists() {
                    return Some(p);
                }
            }
            None
        }
        #[cfg(not(windows))]
        {
            None
        }
    })?;
    let content = std::fs::read_to_string(&path).ok()?;
    let mut parts = content.trim().split(':');
    let _name = parts.next()?;
    let _pid = parts.next()?;
    let port = parts.next()?.parse::<u16>().ok()?;
    let password = parts.next()?.to_string();
    Some(LcuCredentials { port, password })
}

/// Discover LCU credentials (process first on Windows, then lockfile).
pub fn discover_credentials() -> Option<LcuCredentials> {
    #[cfg(windows)]
    {
        credentials_from_process_windows().or_else(credentials_from_lockfile)
    }
    #[cfg(not(windows))]
    {
        credentials_from_lockfile()
    }
}
