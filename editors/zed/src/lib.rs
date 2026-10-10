use std::fs;

use zed_extension_api::{
    self as zed, settings::LspSettings, Architecture, LanguageServerId,
    LanguageServerInstallationStatus, Os, Result,
};

const SERVER_NAME: &str = "vhdl_ls";
const RELEASE_REPO: &str = "VHDL-LS/rust_hdl";

struct VhdlLsExtension {
    cached_binary_path: Option<String>,
}

impl VhdlLsExtension {
    fn binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        if let Some(path) = worktree.which(SERVER_NAME) {
            return Ok(path);
        }

        if let Some(path) = &self.cached_binary_path {
            if is_file(path) {
                return Ok(path.clone());
            }
        }

        let path = install_from_github(language_server_id)?;
        self.cached_binary_path = Some(path.clone());
        Ok(path)
    }
}

fn is_file(path: &str) -> bool {
    fs::metadata(path).is_ok_and(|metadata| metadata.is_file())
}

/// Returns the target triple of the release asset that runs on this machine.
fn release_target() -> Result<&'static str> {
    match zed::current_platform() {
        (Os::Linux, Architecture::X8664) => Ok("x86_64-unknown-linux-musl"),
        (Os::Mac, Architecture::Aarch64) => Ok("aarch64-apple-darwin"),
        // Windows on ARM runs x86_64 binaries through emulation.
        (Os::Windows, Architecture::X8664 | Architecture::Aarch64) => Ok("x86_64-pc-windows-msvc"),
        (os, arch) => Err(format!(
            "{RELEASE_REPO} has no prebuilt {SERVER_NAME} for {os:?} on {arch:?}. \
             Put {SERVER_NAME} on your PATH or set `lsp.{SERVER_NAME}.binary.path` in your Zed settings."
        )),
    }
}

/// The server finds `vhdl_libraries` relative to its own path, so the archive layout must stay intact.
fn binary_path_in(version_dir: &str, target: &str) -> String {
    let extension = match zed::current_platform().0 {
        Os::Windows => ".exe",
        Os::Linux | Os::Mac => "",
    };
    format!("{version_dir}/{SERVER_NAME}-{target}/bin/{SERVER_NAME}{extension}")
}

fn install_from_github(language_server_id: &LanguageServerId) -> Result<String> {
    let target = release_target()?;

    zed::set_language_server_installation_status(
        language_server_id,
        &LanguageServerInstallationStatus::CheckingForUpdate,
    );
    let release = match zed::latest_github_release(
        RELEASE_REPO,
        zed::GithubReleaseOptions {
            require_assets: true,
            pre_release: false,
        },
    ) {
        Ok(release) => release,
        // Work offline with the last version that was installed.
        Err(err) => return installed_binary(target).ok_or(err),
    };

    let asset_name = format!("{SERVER_NAME}-{target}.zip");
    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name == asset_name)
        .ok_or_else(|| format!("release {} has no asset {asset_name}", release.version))?;

    let version_dir = format!("{SERVER_NAME}-{}", release.version);
    let binary_path = binary_path_in(&version_dir, target);
    if is_file(&binary_path) {
        return Ok(binary_path);
    }

    zed::set_language_server_installation_status(
        language_server_id,
        &LanguageServerInstallationStatus::Downloading,
    );
    zed::download_file(
        &asset.download_url,
        &version_dir,
        zed::DownloadedFileType::Zip,
    )
    .map_err(|err| format!("failed to download {asset_name}: {err}"))?;
    if !is_file(&binary_path) {
        return Err(format!("{asset_name} does not contain {binary_path}"));
    }
    zed::make_file_executable(&binary_path)?;

    remove_other_versions(&version_dir);
    Ok(binary_path)
}

fn installed_binary(target: &str) -> Option<String> {
    fs::read_dir(".")
        .ok()?
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.starts_with(SERVER_NAME))
        .map(|version_dir| binary_path_in(&version_dir, target))
        .find(|path| is_file(path))
}

fn remove_other_versions(keep: &str) {
    let Ok(entries) = fs::read_dir(".") else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_name().to_str() != Some(keep) {
            fs::remove_dir_all(entry.path()).ok();
        }
    }
}

impl zed::Extension for VhdlLsExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let binary = LspSettings::for_worktree(SERVER_NAME, worktree)
            .ok()
            .and_then(|settings| settings.binary);
        let args = binary
            .as_ref()
            .and_then(|binary| binary.arguments.clone())
            .unwrap_or_default();
        let mut env = worktree.shell_env();
        if let Some(extra) = binary.as_ref().and_then(|binary| binary.env.clone()) {
            env.extend(extra);
        }
        let command = match binary.and_then(|binary| binary.path) {
            Some(path) => path,
            None => self.binary_path(language_server_id, worktree)?,
        };

        Ok(zed::Command { command, args, env })
    }

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        Ok(LspSettings::for_worktree(SERVER_NAME, worktree)
            .ok()
            .and_then(|settings| settings.initialization_options))
    }
}

zed::register_extension!(VhdlLsExtension);
