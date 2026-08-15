use std::{fs, path::Path, path::PathBuf};

use mingling::{
    Grouped, LazyRes, RenderResult, Routable,
    macros::{chain, command, metadata, renderer, routeify},
    metadata::Description,
};
use sha2::{Digest, Sha256};

use crate::{Next, config::ResMlingConfig, eprintln_cargo, println_cargo};

/// Config key holding the base URL that hosts the mling release packages.
const CONFIG_KEY_UPDATE_URL: &str = "update-url";

/// Default update source, used when the config key is unset.
const DEFAULT_UPDATE_URL: &str = "https://mingling-rs.github.io/mingling/dist";

/// Name of the staged update package inside `{data_dir}/mingling`.
const UPDATE_FILE_NAME: &str = "update.tar.gz";

/// Records the sha256 of the last applied update, written by the wrapper.
const LAST_UPDATE_FILE_NAME: &str = "last-update.sha256";

/// The resolved download task: check the remote checksum and stage the package.
#[derive(Debug, Default, Grouped)]
pub struct StateUpdateDownload {
    pub base_url: String,
    pub update_path: PathBuf,
}

/// The latest package is already installed.
#[derive(Debug, Default, Grouped)]
pub struct ResultUpdateUpToDate;

/// The latest package was downloaded, verified, and staged for the wrapper.
#[derive(Debug, Default, Grouped)]
pub struct ResultUpdateStaged {
    pub update_path: PathBuf,
}

/// Errors produced by the download pipeline.
#[derive(Debug, Grouped)]
pub enum UpdateError {
    /// The data directory could not be determined.
    NoDataDirectory,
    /// The configured update URL is not a valid `http(s)://` URL.
    InvalidUrl(String),
    /// A network request failed, or the remote responded with an error.
    Network(String),
    /// The downloaded package failed its sha256 verification.
    ChecksumMismatch(String),
    /// Writing the staged update package failed.
    Io(String),
}

#[metadata(EntryUpdate)]
pub fn desc_update() -> Description {
    "Update mling to the latest version".into()
}

#[command(routeify)]
pub fn update(config: &mut LazyRes<ResMlingConfig>) -> Next {
    let config = config.get_ref();
    let source = config.get_or(CONFIG_KEY_UPDATE_URL, DEFAULT_UPDATE_URL);
    let Some(update_path) = update_package_path() else {
        return UpdateError::NoDataDirectory.to_chain();
    };
    if !is_http_url(source) {
        return UpdateError::InvalidUrl(source.to_string()).to_chain();
    }
    StateUpdateDownload {
        base_url: source.to_string(),
        update_path,
    }
    .to_chain()
}

/// Check the remote checksum against the installed version; if they differ,
/// download the package, verify its checksum, and stage it at
/// `{data_dir}/mingling/update.tar.gz`.
#[chain(routeify)]
pub async fn handle_state_update_download(state: StateUpdateDownload) -> Next {
    match check_and_fetch(&state.base_url, &state.update_path).await {
        Ok(FetchOutcome::UpToDate) => ResultUpdateUpToDate.to_chain(),
        Ok(FetchOutcome::Staged) => ResultUpdateStaged {
            update_path: state.update_path,
        }
        .to_chain(),
        Err(e) => e.to_chain(),
    }
}

#[renderer]
pub fn render_result_update_up_to_date(_: ResultUpdateUpToDate) -> RenderResult {
    let mut result = RenderResult::new();
    println_cargo!(result, "mling is already up to date");
    result
}

#[renderer]
pub fn render_result_update_staged(r: ResultUpdateStaged) -> RenderResult {
    let mut result = RenderResult::new();
    println_cargo!(result, "Downloaded: {}", r.update_path.display());
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
        UpdateError::InvalidUrl(source) => {
            eprintln_cargo!(
                result,
                "invalid update URL `{}`, expected an `http(s)://` URL such as `https://mingling-rs.github.io/mingling/dist`",
                source
            );
        }
        UpdateError::Network(msg) | UpdateError::ChecksumMismatch(msg) | UpdateError::Io(msg) => {
            eprintln_cargo!(result, "{}", msg);
        }
    }
    result
}

/// `{data_dir}/mingling/update.tar.gz`, where the wrapper looks for staged updates.
pub fn update_package_path() -> Option<PathBuf> {
    dirs::data_dir().map(|dir| dir.join("mingling").join(UPDATE_FILE_NAME))
}

/// `{data_dir}/mingling/last-update.sha256`, the checksum of the last applied update.
fn last_update_checksum_path() -> Option<PathBuf> {
    dirs::data_dir().map(|dir| dir.join("mingling").join(LAST_UPDATE_FILE_NAME))
}

fn is_http_url(source: &str) -> bool {
    let source = source.trim();
    source.starts_with("http://") || source.starts_with("https://")
}

/// The platform suffix used by the package names (`mling-{os}.tar.gz`).
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

enum FetchOutcome {
    UpToDate,
    Staged,
}

/// Fetch `mling-{os}.tar.gz.sha256`, skip the download when the installed
/// version already matches, then download and verify the package before
/// staging it.
async fn check_and_fetch(base_url: &str, update_path: &Path) -> Result<FetchOutcome, UpdateError> {
    let os = update_os_name();
    let base = base_url.trim_end_matches('/');
    let checksum_url = format!("{base}/mling-{os}.tar.gz.sha256");
    let package_url = format!("{base}/mling-{os}.tar.gz");

    let client = reqwest::Client::builder()
        .user_agent(format!("mling-updater/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| UpdateError::Network(format!("failed to build HTTP client: {e}")))?;

    // 1. Fetch the remote checksum first.
    let response = client.get(&checksum_url).send().await.map_err(|e| {
        UpdateError::Network(format!(
            "failed to fetch checksum from `{checksum_url}`: {e}"
        ))
    })?;
    if !response.status().is_success() {
        return Err(UpdateError::Network(format!(
            "failed to fetch checksum from `{checksum_url}`: HTTP {}",
            response.status()
        )));
    }
    let checksum_text = response
        .text()
        .await
        .map_err(|e| UpdateError::Network(format!("failed to read checksum: {e}")))?;
    let remote_sha = parse_sha256(&checksum_text).ok_or_else(|| {
        UpdateError::Network(format!("invalid checksum file at `{checksum_url}`"))
    })?;

    // 2. Skip the download when the installed version already matches.
    if let Some(local_sha) = read_last_update_checksum()
        && local_sha == remote_sha
    {
        return Ok(FetchOutcome::UpToDate);
    }

    // 3. Download the package.
    let response =
        client.get(&package_url).send().await.map_err(|e| {
            UpdateError::Network(format!("failed to download `{package_url}`: {e}"))
        })?;
    if !response.status().is_success() {
        return Err(UpdateError::Network(format!(
            "failed to download `{package_url}`: HTTP {}",
            response.status()
        )));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|e| UpdateError::Network(format!("failed to read package body: {e}")))?;

    // 4. Verify the package before staging it.
    let actual_sha = sha256_hex(&bytes);
    if actual_sha != remote_sha {
        return Err(UpdateError::ChecksumMismatch(format!(
            "checksum mismatch for `{package_url}`: expected {remote_sha}, got {actual_sha}"
        )));
    }

    // 5. Stage it for the wrapper.
    write_update_package(&bytes, update_path)?;
    Ok(FetchOutcome::Staged)
}

/// Parse the sha256 hex digest from a `sha256sum`-style line (`<hash>  <file>`).
fn parse_sha256(line: &str) -> Option<String> {
    let hash = line.split_whitespace().next()?;
    (hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit())).then(|| hash.to_string())
}

/// The checksum of the last update applied by the wrapper, if recorded.
fn read_last_update_checksum() -> Option<String> {
    let path = last_update_checksum_path()?;
    let content = fs::read_to_string(path).ok()?;
    let sha = content.trim();
    (!sha.is_empty()).then(|| sha.to_string())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// Stage the update package at the wrapper's well-known location. The bytes are
/// written to a temporary file first so a failed download never corrupts a
/// previously staged update.
fn write_update_package(package: &[u8], update_path: &Path) -> Result<(), UpdateError> {
    let parent = update_path.parent().ok_or_else(|| {
        UpdateError::Io(format!("no parent directory for {}", update_path.display()))
    })?;
    fs::create_dir_all(parent).map_err(|e| UpdateError::Io(e.to_string()))?;
    let tmp_path = parent.join("update.tar.gz.tmp");
    fs::write(&tmp_path, package).map_err(|e| UpdateError::Io(e.to_string()))?;
    if update_path.exists() {
        fs::remove_file(update_path).map_err(|e| UpdateError::Io(e.to_string()))?;
    }
    fs::rename(&tmp_path, update_path).map_err(|e| UpdateError::Io(e.to_string()))
}
