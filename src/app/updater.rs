use serde::Deserialize;
use std::sync::{Arc, Mutex};
use semver::Version;

#[derive(Clone, Debug, Deserialize)]
pub struct GithubRelease {
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub html_url: String,
    pub assets: Vec<GithubAsset>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GithubAsset {
    pub name: String,
    pub browser_download_url: String,
}

#[derive(Clone, Debug)]
pub struct UpdateInfo {
    pub version: String,
    pub title: String,
    pub body: String,
    pub url: String,
    pub download_url: Option<String>,
}

pub fn check_for_updates(current_version_str: &str) -> Option<UpdateInfo> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("Copaiba-NEO-Updater")
        .build()
        .ok()?;

    let response = client
        .get("https://api.github.com/repos/studiopomar/Copaiba-NEO/releases/latest")
        .send()
        .ok()?;

    if !response.status().is_success() {
        return None;
    }

    let release: GithubRelease = response.json().ok()?;
    
    // Clean version string (e.g., "v0.200.0" -> "0.200.0")
    let latest_ver_str = release.tag_name.trim_start_matches('v');
    let current_ver_str = current_version_str.trim_start_matches('v');

    let latest_ver = Version::parse(latest_ver_str).ok()?;
    let current_ver = Version::parse(current_ver_str).ok()?;

    if latest_ver > current_ver {
        // Find suitable asset for Windows (assuming .exe or .zip)
        let download_url = release.assets.iter()
            .find(|a| a.name.ends_with(".exe") || a.name.contains("windows"))
            .map(|a| a.browser_download_url.clone());

        Some(UpdateInfo {
            version: release.tag_name,
            title: release.name,
            body: release.body,
            url: release.html_url,
            download_url,
        })
    } else {
        None
    }
}

pub fn spawn_update_check(current_version: String, result_arc: Arc<Mutex<Option<UpdateInfo>>>) {
    std::thread::spawn(move || {
        if let Some(info) = check_for_updates(&current_version) {
            let mut lock = result_arc.lock().unwrap();
            *lock = Some(info);
        }
    });
}
