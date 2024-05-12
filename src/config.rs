use std::path::{Path, PathBuf};

use anyhow::Context;
use colored::Colorize;
use die_exit::die;
use serde::{Deserialize, Serialize};

use crate::{
    crypt::{COMPRESSED_EXTENSION, ENCRYPTED_EXTENSION},
    utils::PathToAbsolute,
};

pub const CONFIG_FILE: &str = concat!(env!("CARGO_CRATE_NAME"), ".toml");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub path: PathBuf,
    pub use_zstd: bool,
    pub zstd_level: u8,
    pub crypt_list: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            path: PathBuf::from(CONFIG_FILE),
            use_zstd: true,
            zstd_level: 15,
            crypt_list: vec![],
        }
    }
}

impl Config {
    pub fn with_path(mut self, path: impl AsRef<Path>) -> Self {
        self.path = path.as_ref().to_path_buf();
        self
    }
    pub fn default_with_path(path: impl AsRef<Path>) -> Self {
        Self::default().with_path(path)
    }
    pub fn load(&self) -> anyhow::Result<Self> {
        Self::load_from(&self.path)
    }
    pub fn load_from(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).with_context(|| {
            format!("Warning: config not found in `{}`", path.display()).yellow()
        })?;
        Ok(toml::from_str(&content)?)
    }
    pub fn save(&self) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&self.path, content)?;
        Ok(())
    }
    fn _load_or_create_inner(path: impl AsRef<Path>, loaded: anyhow::Result<Self>) -> Self {
        loaded.unwrap_or_else(|e| {
            eprintln!("{e}{}", ", use default config...".yellow());
            let config = Self::default_with_path(path);
            config
                .save()
                .unwrap_or_else(|e| die!("Failed to save config file: {}", e));
            config
        })
    }
    pub fn load_or_create(&self) -> Self {
        Self::_load_or_create_inner(&self.path, self.load())
    }
    pub fn load_or_create_from(path: impl AsRef<Path>) -> Self {
        Self::_load_or_create_inner(&path, Self::load_from(&path))
    }
    pub fn add_to_crypt_list_one(&mut self, path: &str) {
        let p = Path::new(path);
        assert2::assert!(
            !p.is_absolute(),
            "Error: `{:?}`. Please use relative path.",
            p
        );
        assert2::assert!(
            ![ENCRYPTED_EXTENSION, COMPRESSED_EXTENSION].contains(
                &p.extension()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default()
            ),
            "Cannot add file with extension `{}`, `{}`",
            ENCRYPTED_EXTENSION,
            COMPRESSED_EXTENSION
        );
        let joined_path = self
            .path
            .parent()
            .expect("parent dir of config file must exist")
            .join(path);
        debug_assert!(
            joined_path.exists(),
            "file not exist: `{:?}`, abs: `{:?}`",
            joined_path,
            joined_path.absolute()
        );
        println!(
            "Add to crypt list: {}",
            format!("{:?}", joined_path.absolute()).green()
        );
        self.crypt_list
            .push(path.to_string() + if p.is_dir() { "/**" } else { "" });
    }
    pub fn add_to_crypt_list(&mut self, paths: &[&str]) -> anyhow::Result<()> {
        paths.iter().for_each(|x| self.add_to_crypt_list_one(x));
        self.save()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Ok;
    use temp_testdir::TempDir;

    use super::*;

    #[test]
    fn test_save_load() -> anyhow::Result<()> {
        let temp_dir = TempDir::default();
        let file_path = temp_dir.join("test");
        let config = Config::default_with_path(&file_path);
        config.save()?;
        assert!(file_path.exists());
        let config_load = Config::load_from(file_path);
        assert!(config_load.is_ok());
        Ok(())
    }

    #[test]
    fn test() -> anyhow::Result<()> {
        let temp_dir = TempDir::default();
        let file_path = temp_dir.join("test");
        assert!(!&file_path.exists());
        let mut config = Config::load_or_create_from(&file_path);
        assert!(config.zstd_level == Config::default().zstd_level);
        assert!(&file_path.exists());

        fs::create_dir(temp_dir.join("123"))?;
        config.add_to_crypt_list_one(temp_dir.join("123").as_os_str().to_string_lossy().as_ref());
        config.save()?;

        let config = Config::default_with_path(&file_path).load()?;
        assert!(config.crypt_list[0].ends_with("/**"));
        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_add_enc_file() {
        let temp_dir = TempDir::default();
        std::fs::File::create(temp_dir.join("test.enc")).unwrap();
        let file_path = temp_dir.join("test");
        let mut config = Config::load_or_create_from(file_path);
        config.add_to_crypt_list_one("test.enc");
    }
}
