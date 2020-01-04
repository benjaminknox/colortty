use crate::color::ColorScheme;
use crate::error::{ErrorKind, Result};
use dirs;
use failure::ResultExt;
use git2::Repository;
use std::fs;
use std::path::Path;

/// A GitHub repository that provides color schemes.
pub struct Provider {
    user_name: String,
    repo_name: String,
    list_path: String,
    extension: String,
}

impl Provider {
    /// Returns a provider for `mbadolato/iTerm2-Color-Schemes`.
    pub fn iterm() -> Self {
        Provider::new(
            "mbadolato",
            "iTerm2-Color-Schemes",
            "schemes",
            ".itermcolors",
        )
    }

    /// Returns a provider for `Mayccoll/Gogh`.
    pub fn gogh() -> Self {
        Provider::new("Mayccoll", "Gogh", "themes", ".sh")
    }

    /// Returns a provider instance.
    fn new(user_name: &str, repo_name: &str, list_path: &str, extension: &str) -> Self {
        Provider {
            user_name: user_name.to_string(),
            repo_name: repo_name.to_string(),
            list_path: list_path.to_string(),
            extension: extension.to_string(),
        }
    }

    /// Fetches the raw content of the color scheme for the given name.
    pub fn get(&self, name: &str) -> Result<ColorScheme> {
        let url = format!(
            "https://raw.githubusercontent.com/{}/{}/master/{}/{}{}",
            self.user_name, self.repo_name, self.list_path, name, self.extension
        );
        let body = http_get(&url)?;

        // TODO: Think about better abstraction.
        if self.extension == ".itermcolors" {
            ColorScheme::from_iterm(&body)
        } else {
            ColorScheme::from_gogh(&body)
        }
    }

    /// Returns names of all color schemes in the provider.
    pub fn list(&self) -> Result<Vec<String>> {
        // The parent directory to clone the repository cache into.
        let mut parent_dir = dirs::cache_dir().ok_or(ErrorKind::NoCacheDir)?;
        parent_dir.push("colortty");
        parent_dir.push("repositories");
        parent_dir.push(&self.user_name);
        // The repository cache directory.
        let repo_dir = parent_dir.join(&self.repo_name);
        // The directory of all color schemes in the repository.
        let schemes_dir = repo_dir.join(&self.list_path);

        // Create the parent directory if it doesn't exist.
        fs::create_dir_all(&parent_dir).context(ErrorKind::CreateDirAll)?;

        if Path::new(&repo_dir).exists() {
            // TODO: Checkout if the local clone exists.
        } else {
            // Clone the repository.
            let repo_url = format!("https://github.com/{}/{}", self.user_name, self.repo_name);
            println!("Cloning {}", repo_url);
            Repository::clone(&repo_url, &repo_dir).context(ErrorKind::GitClone)?;
        }

        let mut names: Vec<String> = Vec::new();

        let entries = fs::read_dir(&schemes_dir).context(ErrorKind::ReadDir)?;
        for entry in entries {
            let dir_entry = entry.context(ErrorKind::ReadDirEntry)?;
            let filename = dir_entry.file_name().into_string().unwrap();

            // Ignoring files starting with `_` for Gogh.
            if filename.starts_with('_') || !filename.ends_with(&self.extension) {
                continue;
            }

            names.push(filename.replace(&self.extension, "").to_string());
        }

        Ok(names)
    }
}

/// Returns the body of the given URL.
fn http_get(url: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let mut res = client
        .get(url)
        .header(reqwest::header::USER_AGENT, "colortty")
        .send()
        .context(ErrorKind::HttpGet)?;

    if !res.status().is_success() {
        return Err(ErrorKind::HttpGet.into());
    }

    let body = res.text().context(ErrorKind::HttpGet)?;
    Ok(body)
}
