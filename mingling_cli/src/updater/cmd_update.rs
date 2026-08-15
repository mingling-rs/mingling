use std::{fs, io, path::Path, path::PathBuf};

use mingling::{
    Grouped, LazyRes, RenderResult, Routable,
    macros::{chain, command, metadata, renderer, routeify},
    metadata::Description,
};

use crate::{Next, config::ResMlingConfig, eprintln_cargo, println_cargo};

/// Config key holding the GitHub repository that publishes the mling packages.
const CONFIG_KEY_GITHUB: &str = "mingling-github";

/// Default update source, used when the config key is unset.
const DEFAULT_GITHUB: &str = "https://github.com/mingling-rs/mingling";

/// Name of the staged update package inside `{data_dir}/mingling`.
const UPDATE_FILE_NAME: &str = "update.tar.gz";

/// The resolved download task: fetch the newest artifact and stage it.
#[derive(Debug, Default, Grouped)]
pub struct StateUpdateDownload {
    pub owner: String,
    pub repo: String,
    pub update_path: PathBuf,
}

/// The staged update package, ready to be applied by the `mling` wrapper.
#[derive(Debug, Default, Grouped)]
pub struct ResultUpdate {
    pub artifact_name: String,
    pub update_path: PathBuf,
}

/// Errors produced by the download pipeline.
#[derive(Debug, Grouped)]
pub enum UpdateError {
    /// The data directory could not be determined.
    NoDataDirectory,
    /// The configured update source is not a GitHub repository URL.
    InvalidRepo(String),
    /// A network or API request failed.
    Network(String),
    /// No matching artifact was found.
    NoArtifact(String),
    /// Writing the staged update package failed.
    Io(String),
}

/// A downloaded artifact with its inner `*.tar.gz` extracted.
struct Artifact {
    name: String,
    tar_gz: Vec<u8>,
}

#[metadata(EntryUpdate)]
pub fn desc_update() -> Description {
    "Update mling to the latest version".into()
}

#[command(routeify)]
pub fn update(config: &mut LazyRes<ResMlingConfig>) -> Next {
    let config = config.get_ref();
    let source = config.get_or(CONFIG_KEY_GITHUB, DEFAULT_GITHUB);
    let Some(update_path) = update_package_path() else {
        return UpdateError::NoDataDirectory.to_chain();
    };
    match parse_github_repo(source) {
        Some((owner, repo)) => StateUpdateDownload {
            owner,
            repo,
            update_path,
        }
        .to_chain(),
        None => UpdateError::InvalidRepo(source.to_string()).to_chain(),
    }
}

/// Fetch the newest artifact for the current platform and stage it at
/// `{data_dir}/mingling/update.tar.gz`.
#[chain(routeify)]
pub async fn handle_state_update_download(state: StateUpdateDownload) -> Next {
    match fetch_latest_artifact(&state.owner, &state.repo).await {
        Ok(artifact) => {
            if let Err(e) = write_update_package(&artifact.tar_gz, &state.update_path) {
                return UpdateError::Io(e).to_chain();
            }
            ResultUpdate {
                artifact_name: artifact.name,
                update_path: state.update_path,
            }
            .to_chain()
        }
        Err(e) => e.to_chain(),
    }
}

#[renderer]
pub fn render_result_update(r: ResultUpdate) -> RenderResult {
    let mut result = RenderResult::new();
    println_cargo!(result, "Downloaded: {}", r.artifact_name);
    println_cargo!(result, "Staged: {}", r.update_path.display());
    println_cargo!(result, "Run `mling` again to apply the update");
    result
}

#[renderer]
pub fn render_error_update(err: UpdateError) -> RenderResult {
    let mut result = RenderResult::new();
    match err {
        UpdateError::NoDataDirectory => {
            eprintln_cargo!(result, "failed to determine the data directory");
        }
        UpdateError::InvalidRepo(source) => {
            eprintln_cargo!(
                result,
                "invalid update source `{}`, expected a GitHub repository URL like `https://github.com/mingling-rs/mingling`",
                source
            );
        }
        UpdateError::Network(msg) | UpdateError::NoArtifact(msg) | UpdateError::Io(msg) => {
            eprintln_cargo!(result, "{}", msg);
        }
    }
    result
}

/// `{data_dir}/mingling/update.tar.gz`, where the wrapper looks for staged updates.
pub fn update_package_path() -> Option<PathBuf> {
    dirs::data_dir().map(|dir| dir.join("mingling").join(UPDATE_FILE_NAME))
}

/// Extract `owner` / `repo` from a GitHub URL such as
/// `https://github.com/mingling-rs/mingling`. Trailing slashes and `.git`
/// suffixes are tolerated, and a bare `owner/repo` is accepted as well.
fn parse_github_repo(source: &str) -> Option<(String, String)> {
    let trimmed = source.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return None;
    }
    let path = match trimmed.rfind("://") {
        Some(idx) => &trimmed[idx + 3..],
        None => trimmed,
    };
    let mut segments = path.split('/').filter(|s| !s.is_empty());
    // The first segment is the host; take the two path segments after it.
    segments.next()?;
    let owner = segments.next()?;
    let repo = segments.next()?.trim_end_matches(".git");
    Some((owner.to_string(), repo.to_string()))
}

/// The platform suffix used by the CI artifact names (`mling-{os}-...`).
fn update_os_name() -> &'static str {
    if cfg!(windows) {
        "win"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "mac"
    } else {
        "unknown"
    }
}

/// Query the GitHub Actions API, pick the newest non-expired artifact for the
/// current platform, download it, and extract the inner `*.tar.gz`.
async fn fetch_latest_artifact(owner: &str, repo: &str) -> Result<Artifact, UpdateError> {
    let client = reqwest::Client::builder()
        .user_agent(format!("mling-updater/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| UpdateError::Network(format!("failed to build HTTP client: {e}")))?;

    let list_url =
        format!("https://api.github.com/repos/{owner}/{repo}/actions/artifacts?per_page=100");
    let mut request = client
        .get(&list_url)
        .header("Accept", "application/vnd.github+json");
    if let Ok(token) = std::env::var("GITHUB_TOKEN")
        && !token.is_empty()
    {
        request = request.header("Authorization", format!("Bearer {token}"));
    }

    let response = request.send().await.map_err(|e| {
        UpdateError::Network(format!("failed to query GitHub Actions artifacts: {e}"))
    })?;
    if !response.status().is_success() {
        return Err(UpdateError::Network(format!(
            "GitHub Actions API returned {} for `{list_url}`",
            response.status()
        )));
    }
    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| UpdateError::Network(format!("failed to parse GitHub response: {e}")))?;

    let os = update_os_name();
    let prefix = format!("mling-{os}-");
    let artifact = json
        .get("artifacts")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter(|a| {
            a.get("expired").and_then(serde_json::Value::as_bool) != Some(true)
                && a.get("name")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|name| name.starts_with(&prefix))
        })
        .max_by_key(|a| {
            a.get("created_at")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
        })
        .ok_or_else(|| {
            UpdateError::NoArtifact(format!("no `{prefix}*` artifact found in {owner}/{repo}"))
        })?;

    let name = artifact
        .get("name")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("mling")
        .to_string();
    let download_url = artifact
        .get("archive_download_url")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(&format!(
            "https://api.github.com/repos/{owner}/{repo}/actions/artifacts/{}/zip",
            artifact
                .get("id")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0)
        ))
        .to_string();

    let response =
        client.get(&download_url).send().await.map_err(|e| {
            UpdateError::Network(format!("failed to download artifact `{name}`: {e}"))
        })?;
    if !response.status().is_success() {
        return Err(UpdateError::Network(format!(
            "failed to download artifact `{name}`: HTTP {}",
            response.status()
        )));
    }
    let zip_bytes = response
        .bytes()
        .await
        .map_err(|e| UpdateError::Network(format!("failed to read artifact `{name}`: {e}")))?;

    let tar_gz = extract_tar_gz_from_zip(&zip_bytes)
        .map_err(|e| UpdateError::Network(format!("invalid artifact `{name}`: {e}")))?;
    Ok(Artifact { name, tar_gz })
}

/// The GitHub artifact is a zip containing the `mling-{os}-{sha}-{date}.tar.gz`
/// built by CI; extract that inner file.
fn extract_tar_gz_from_zip(zip_bytes: &[u8]) -> Result<Vec<u8>, String> {
    let reader = io::Cursor::new(zip_bytes);
    let mut archive = zip::ZipArchive::new(reader).map_err(|e| e.to_string())?;
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).map_err(|e| e.to_string())?;
        let file_name = file.name().to_string();
        if file_name.ends_with(".tar.gz") {
            let mut tar_gz = Vec::with_capacity(file.size() as usize);
            io::copy(&mut file, &mut tar_gz).map_err(|e| e.to_string())?;
            return Ok(tar_gz);
        }
    }
    Err("artifact contains no `*.tar.gz` file".to_string())
}

/// Stage the update package at the wrapper's well-known location. The bytes are
/// written to a temporary file first so a failed download never corrupts a
/// previously staged update.
fn write_update_package(tar_gz: &[u8], update_path: &Path) -> Result<(), String> {
    let parent = update_path
        .parent()
        .ok_or_else(|| format!("no parent directory for {}", update_path.display()))?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let tmp_path = parent.join("update.tar.gz.tmp");
    fs::write(&tmp_path, tar_gz).map_err(|e| e.to_string())?;
    if update_path.exists() {
        fs::remove_file(update_path).map_err(|e| e.to_string())?;
    }
    fs::rename(&tmp_path, update_path).map_err(|e| e.to_string())
}
