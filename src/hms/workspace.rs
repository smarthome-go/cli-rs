use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use anstream::{print, println};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use similar::TextDiff;
use smarthome_sdk_rs::{Client, HmsRunMode, HomescriptData, HomescriptType};

use super::errors::{Error, Result, render_diagnostics};
use crate::term::{self, ACCENT, ADDED, BAD, BOLD, DIM, GOOD, HUNK, REMOVED, WARN, paint};

pub const MANIFEST_FILE: &str = ".hms.toml";

#[derive(Serialize, Deserialize, Debug)]
pub struct HomescriptMetadata {
    pub id: String,
    pub is_driver: bool,
    /// SHA-256 of the server's code at the last clone, pull or push.
    /// Tells local changes apart from changes made on the server since then.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub synced_hash: Option<String>,
}

/// How a workspace relates to the server's copy of its script
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncState {
    /// The local code and the server's code are identical
    UpToDate,
    /// Only the local code changed since the last sync
    LocalChanges,
    /// Only the server's code changed since the last sync
    RemoteChanges,
    /// Both changed since the last sync
    Diverged,
    /// The code differs, but there is no sync record to tell which side changed
    Differs,
    /// The script does not exist on the server (anymore)
    MissingOnServer,
}

impl SyncState {
    fn describe(self) -> String {
        match self {
            Self::UpToDate => paint(GOOD, "up to date"),
            Self::LocalChanges => format!("{} {}", paint(WARN, "local changes"), paint(DIM, "(push to upload)")),
            Self::RemoteChanges => format!("{} {}", paint(ACCENT, "server changed"), paint(DIM, "(pull to update)")),
            Self::Diverged => format!("{} {}", paint(BAD, "diverged"), paint(DIM, "(both changed, see `diff`)")),
            Self::Differs => format!("{} {}", paint(WARN, "differs"), paint(DIM, "(no sync record yet, see `diff`)")),
            Self::MissingOnServer => paint(BAD, "missing on server"),
        }
    }
}

/// A local directory holding a script's code and its manifest
pub struct Workspace {
    pub dir: PathBuf,
    pub manifest: HomescriptMetadata,
    pub code: String,
}

impl Workspace {
    pub fn open(dir: &Path) -> Result<Self> {
        let manifest_path = dir.join(MANIFEST_FILE);
        if !manifest_path.exists() {
            return Err(Error::NotAWorkspace);
        }
        let manifest: HomescriptMetadata = toml::from_str(&fs::read_to_string(manifest_path)?)?;
        let code_path = dir.join(format!("{}.hms", manifest.id));
        if !code_path.exists() {
            return Err(Error::NotAWorkspace);
        }
        let code = fs::read_to_string(code_path)?;
        Ok(Self {
            dir: dir.to_path_buf(),
            manifest,
            code,
        })
    }

    /// Creates a workspace in `./<id>` holding `local_code`, synced to the server's `script`
    pub fn create(script: &HomescriptData, local_code: &str) -> Result<Self> {
        let dir = PathBuf::from(&script.id);
        if dir.exists() {
            return Err(Error::CloneDirAlreadyExists(script.id.clone()));
        }
        fs::create_dir_all(&dir)?;

        let workspace = Self {
            dir,
            manifest: HomescriptMetadata {
                id: script.id.clone(),
                is_driver: script.type_ == HomescriptType::Driver,
                synced_hash: Some(hash(&script.code)),
            },
            code: local_code.to_string(),
        };
        fs::write(workspace.code_path(), &workspace.code)?;
        workspace.save_manifest()?;
        Ok(workspace)
    }

    fn code_path(&self) -> PathBuf {
        self.dir.join(format!("{}.hms", self.manifest.id))
    }

    fn save_manifest(&self) -> Result<()> {
        fs::write(self.dir.join(MANIFEST_FILE), toml::to_string_pretty(&self.manifest)?)?;
        Ok(())
    }

    /// Records `remote_code` as the server's code at the last sync
    fn mark_synced(&mut self, remote_code: &str) -> Result<()> {
        let hash = hash(remote_code);
        if self.manifest.synced_hash.as_ref() != Some(&hash) {
            debug!("Recording sync state of `{}`", self.manifest.id);
            self.manifest.synced_hash = Some(hash);
            self.save_manifest()?;
        }
        Ok(())
    }

    pub fn state(&self, remote_code: Option<&str>) -> SyncState {
        let Some(remote_code) = remote_code else {
            return SyncState::MissingOnServer;
        };
        if self.code == remote_code {
            return SyncState::UpToDate;
        }
        let Some(synced_hash) = &self.manifest.synced_hash else {
            return SyncState::Differs;
        };
        match (hash(&self.code) != *synced_hash, hash(remote_code) != *synced_hash) {
            (true, false) => SyncState::LocalChanges,
            (false, true) => SyncState::RemoteChanges,
            // Both cannot match the record while differing from each other
            _ => SyncState::Diverged,
        }
    }
}

fn hash(code: &str) -> String {
    Sha256::digest(code.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn is_workspace(dir: &Path) -> bool {
    dir.join(MANIFEST_FILE).exists()
}

/// Selects the workspaces to operate on: the current directory or, with `all`,
/// every workspace in or directly below it
fn discover(all: bool) -> Result<Vec<Workspace>> {
    let cwd = Path::new(".");
    if !all {
        return Ok(vec![Workspace::open(cwd)?]);
    }

    let mut dirs = Vec::new();
    if is_workspace(cwd) {
        dirs.push(cwd.to_path_buf());
    }
    let mut children: Vec<PathBuf> = fs::read_dir(cwd)?
        .filter_map(|entry| Some(entry.ok()?.path()))
        .filter(|path| path.is_dir() && is_workspace(path))
        .collect();
    children.sort();
    dirs.extend(children);

    let workspaces: Vec<Workspace> = dirs
        .iter()
        .filter_map(|dir| match Workspace::open(dir) {
            Ok(workspace) => Some(workspace),
            Err(err) => {
                warn!("Skipping `{}`: {err}", dir.display());
                None
            }
        })
        .collect();

    match workspaces.is_empty() {
        true => Err(Error::NoWorkspaces),
        false => Ok(workspaces),
    }
}

async fn fetch_remote(client: &Client) -> Result<HashMap<String, HomescriptData>> {
    Ok(term::spin("Fetching scripts", client.list_personal_homescripts())
        .await?
        .into_iter()
        .map(|script| (script.data.id.clone(), script.data))
        .collect())
}

fn host(client: &Client) -> &str {
    client.smarthome_url.host_str().unwrap_or_default()
}

/// Reports the errors of a multi-workspace operation; a single workspace's error is returned as is
fn summarize(mut results: Vec<Result<()>>) -> Result<()> {
    if results.len() == 1 {
        return results.remove(0);
    }
    let total = results.len();
    let mut failed = 0;
    for result in results {
        if let Err(err) = result {
            error!("{err}");
            failed += 1;
        }
    }
    match failed {
        0 => Ok(()),
        failed => Err(Error::Failed { failed, total }),
    }
}

#[derive(Serialize)]
struct StatusEntry {
    id: String,
    path: PathBuf,
    state: SyncState,
}

pub async fn status(client: &Client, all: bool, json: bool) -> Result<()> {
    // Outside of a workspace, show the overview of all workspaces below
    let workspaces = discover(all || !is_workspace(Path::new(".")))?;
    let remote = fetch_remote(client).await?;

    let entries: Vec<StatusEntry> = workspaces
        .into_iter()
        .map(|workspace| StatusEntry {
            state: workspace.state(remote.get(&workspace.manifest.id).map(|data| data.code.as_str())),
            id: workspace.manifest.id,
            path: workspace.dir,
        })
        .collect();

    if json {
        println!("{}", serde_json::to_string_pretty(&entries)?);
        return Ok(());
    }

    let width = entries.iter().map(|entry| entry.id.len()).max().unwrap_or_default();
    for entry in entries {
        println!("{:<width$}  {}", entry.id, entry.state.describe());
    }
    Ok(())
}

pub async fn diff(client: &Client, all: bool) -> Result<()> {
    let workspaces = discover(all)?;
    let remote = fetch_remote(client).await?;

    for workspace in &workspaces {
        let id = &workspace.manifest.id;
        let Some(remote) = remote.get(id) else {
            warn!("`{id}` does not exist on the server");
            continue;
        };
        if remote.code == workspace.code {
            continue;
        }

        let diff = TextDiff::from_lines(remote.code.as_str(), workspace.code.as_str());
        let unified = diff
            .unified_diff()
            .context_radius(3)
            .header(&format!("server/{id}.hms"), &format!("local/{id}.hms"))
            .to_string();
        for line in unified.lines() {
            let style = match line {
                _ if line.starts_with("+++") || line.starts_with("---") => BOLD,
                _ if line.starts_with('+') => ADDED,
                _ if line.starts_with('-') => REMOVED,
                _ if line.starts_with("@@") => HUNK,
                _ => anstyle::Style::new(),
            };
            println!("{}", paint(style, line));
        }
    }
    Ok(())
}

pub async fn push(client: &Client, lint_hook: bool, force: bool, all: bool) -> Result<()> {
    let workspaces = discover(all)?;
    let remote = fetch_remote(client).await?;

    let mut results = Vec::with_capacity(workspaces.len());
    for workspace in workspaces {
        let remote = remote.get(&workspace.manifest.id);
        results.push(push_one(client, workspace, remote, lint_hook, force).await);
    }
    summarize(results)
}

const DIVERGED_HINT: &str = "both the local code and the server's copy changed since the last sync\n => Compare them with `shome hms script diff`, then use `push --force` to overwrite the server's copy or `pull --force` to discard the local changes";

async fn push_one(
    client: &Client,
    mut workspace: Workspace,
    remote: Option<&HomescriptData>,
    lint_hook: bool,
    force: bool,
) -> Result<()> {
    let id = workspace.manifest.id.clone();
    let Some(remote) = remote else {
        return Err(Error::InvalidHomescript(id));
    };

    match workspace.state(Some(&remote.code)) {
        SyncState::UpToDate => {
            workspace.mark_synced(&remote.code)?;
            info!("`{id}` is already up to date");
            return Ok(());
        }
        SyncState::LocalChanges => {}
        SyncState::Differs => warn!(
            "`{id}` has no sync record yet, so changes made on the server since cloning cannot be detected"
        ),
        SyncState::RemoteChanges if !force => {
            return Err(Error::Conflict {
                id,
                message: "the server's copy changed since the last sync and there are no local changes to push\n => Run `shome hms script pull` to update, or `push --force` to overwrite the server's copy with the older local code".to_string(),
            });
        }
        SyncState::Diverged if !force => {
            return Err(Error::Conflict {
                id,
                message: DIVERGED_HINT.to_string(),
            });
        }
        SyncState::RemoteChanges | SyncState::Diverged => {
            warn!("Overwriting the server's changes to `{id}` (--force)")
        }
        SyncState::MissingOnServer => unreachable!("The server's copy was found above"),
    }

    // Running the pre-push lint hook if required
    if lint_hook {
        let response = term::spin(
            format!("Linting {id}"),
            client.exec_homescript_code(
                &workspace.code,
                vec![],
                HmsRunMode::Lint {
                    module_name: &id,
                    is_driver: workspace.manifest.is_driver,
                },
            ),
        )
        .await?;
        match (response.success, force) {
            (true, _) => debug!("Linting discovered no problems"),
            (false, false) => {
                return Err(Error::LintErrors {
                    errors: response.errors,
                    code: workspace.code,
                    file_contents: response.file_contents,
                });
            }
            (false, true) => warn!("Linting discovered errors in `{id}`: force-pushing anyway"),
        }
    }

    term::spin(
        format!("Pushing {id}"),
        client.modify_homescript_code(&id, &workspace.code),
    )
    .await?;
    let pushed = workspace.code.clone();
    workspace.mark_synced(&pushed)?;
    info!("{} `{id}` to {}", paint(GOOD, "Pushed"), host(client));
    Ok(())
}

pub async fn pull(client: &Client, force: bool, all: bool) -> Result<()> {
    let workspaces = discover(all)?;
    let remote = fetch_remote(client).await?;

    let results = workspaces
        .into_iter()
        .map(|workspace| {
            let remote = remote.get(&workspace.manifest.id);
            pull_one(client, workspace, remote, force)
        })
        .collect();
    summarize(results)
}

fn pull_one(
    client: &Client,
    mut workspace: Workspace,
    remote: Option<&HomescriptData>,
    force: bool,
) -> Result<()> {
    let id = workspace.manifest.id.clone();
    let Some(remote) = remote else {
        return Err(Error::InvalidHomescript(id));
    };

    match workspace.state(Some(&remote.code)) {
        SyncState::UpToDate => {
            workspace.mark_synced(&remote.code)?;
            info!("`{id}` is already up to date");
            return Ok(());
        }
        SyncState::RemoteChanges => {}
        SyncState::LocalChanges if !force => {
            return Err(Error::Conflict {
                id,
                message: "there are local changes which are not on the server\n => Run `shome hms script push` to upload them, or `pull --force` to discard them".to_string(),
            });
        }
        SyncState::Differs if !force => {
            return Err(Error::Conflict {
                id,
                message: "the local code differs from the server's copy and there is no sync record to tell whether it has unpushed changes\n => Compare them with `shome hms script diff`, then use `pull --force` to take the server's copy".to_string(),
            });
        }
        SyncState::Diverged if !force => {
            return Err(Error::Conflict {
                id,
                message: DIVERGED_HINT.to_string(),
            });
        }
        SyncState::LocalChanges | SyncState::Differs | SyncState::Diverged => {
            warn!("Discarding local changes to `{id}` (--force)")
        }
        SyncState::MissingOnServer => unreachable!("The server's copy was found above"),
    }

    fs::write(workspace.code_path(), &remote.code)?;
    workspace.code = remote.code.clone();
    workspace.mark_synced(&remote.code)?;
    info!("{} `{id}` from {}", paint(GOOD, "Pulled"), host(client));
    Ok(())
}

pub async fn clone(script_ids: &[String], all: bool, client: &Client) -> Result<()> {
    let scripts = term::spin("Fetching scripts", client.list_personal_homescripts()).await?;

    let selected: Vec<&HomescriptData> = match all {
        true => scripts.iter().map(|script| &script.data).collect(),
        false => script_ids
            .iter()
            .map(|id| {
                scripts
                    .iter()
                    .find(|script| script.data.id == *id)
                    .map(|script| &script.data)
                    .ok_or_else(|| Error::ScriptDoesNotExist(id.clone()))
            })
            .collect::<Result<_>>()?,
    };

    for script in selected {
        match Workspace::create(script, &script.code) {
            Ok(_) => info!("{} `{}` into `./{}`", paint(GOOD, "Cloned"), script.id, script.id),
            // When cloning everything, keep going past scripts which are already cloned
            Err(Error::CloneDirAlreadyExists(dir)) if all => {
                warn!("Skipping `{dir}`: the directory already exists")
            }
            Err(err) => return Err(err),
        }
    }
    Ok(())
}

pub async fn exec_current_script(client: &Client, lint: bool) -> Result<()> {
    let workspace = Workspace::open(Path::new("."))?;
    let id = &workspace.manifest.id;
    debug!("Found workspace of `{id}`");

    if workspace.manifest.is_driver && !lint {
        return Err(Error::CannotRunDriver(id.clone()));
    }

    let response = term::spin(
        format!("{} {id}", if lint { "Linting" } else { "Running" }),
        client.exec_homescript_code(
            &workspace.code,
            vec![],
            match lint {
                true => HmsRunMode::Lint {
                    module_name: id,
                    is_driver: workspace.manifest.is_driver,
                },
                false => HmsRunMode::Execute,
            },
        ),
    )
    .await?;

    if !response.success {
        return Err(match lint {
            true => Error::LintErrors {
                errors: response.errors,
                code: workspace.code,
                file_contents: response.file_contents,
            },
            false => Error::RunErrors {
                errors: response.errors,
                code: workspace.code,
                file_contents: response.file_contents,
            },
        });
    }

    match lint {
        true if response.errors.is_empty() => info!("Linting discovered no problems"),
        true => println!(
            "{}",
            render_diagnostics(&response.errors, &workspace.code, &response.file_contents)
        ),
        false => {
            info!("Program executed successfully");
            print!("{}", response.output);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workspace(code: &str, synced: Option<&str>) -> Workspace {
        Workspace {
            dir: PathBuf::from("."),
            manifest: HomescriptMetadata {
                id: "test".to_string(),
                is_driver: false,
                synced_hash: synced.map(hash),
            },
            code: code.to_string(),
        }
    }

    #[test]
    fn sync_states() {
        assert_eq!(workspace("a", Some("a")).state(Some("a")), SyncState::UpToDate);
        assert_eq!(workspace("b", Some("a")).state(Some("a")), SyncState::LocalChanges);
        assert_eq!(workspace("a", Some("a")).state(Some("b")), SyncState::RemoteChanges);
        assert_eq!(workspace("b", Some("a")).state(Some("c")), SyncState::Diverged);
        assert_eq!(workspace("a", None).state(Some("b")), SyncState::Differs);
        assert_eq!(workspace("a", None).state(Some("a")), SyncState::UpToDate);
        assert_eq!(workspace("a", Some("a")).state(None), SyncState::MissingOnServer);
    }

    #[test]
    fn reads_manifests_without_sync_record() {
        let manifest: HomescriptMetadata =
            toml::from_str("id = \"button_listener\"\nis_driver = false\n").unwrap();
        assert_eq!(manifest.id, "button_listener");
        assert_eq!(manifest.synced_hash, None);
    }

    #[test]
    fn omits_missing_sync_record() {
        let manifest = HomescriptMetadata {
            id: "a".to_string(),
            is_driver: false,
            synced_hash: None,
        };
        assert!(!toml::to_string_pretty(&manifest).unwrap().contains("synced_hash"));
    }
}
