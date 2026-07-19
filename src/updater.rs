use std::sync::mpsc;

use velopack::*;

const GITHUB_REPO: &str = "https://github.com/Liiesl/NeverLiieStatusBar";

fn create_manager() -> Option<UpdateManager> {
    let source = sources::GithubSource::new(GITHUB_REPO, None, false);
    UpdateManager::new(source, None, None).ok()
}

pub fn check_for_updates() -> Option<UpdateInfo> {
    let um = create_manager()?;
    match um.check_for_updates().ok()? {
        UpdateCheck::UpdateAvailable(info) => Some(*info),
        _ => None,
    }
}

pub fn get_current_version() -> String {
    create_manager()
        .map(|um| um.get_current_version_as_string())
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn download_updates(info: &UpdateInfo, progress_tx: mpsc::Sender<i16>) -> bool {
    let Some(um) = create_manager() else {
        return false;
    };
    um.download_updates(info, Some(progress_tx)).is_ok()
}

pub fn apply_updates(info: &UpdateInfo) -> bool {
    let Some(um) = create_manager() else {
        return false;
    };
    um.apply_updates_and_restart(info).is_ok()
}
