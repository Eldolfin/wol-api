use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self, File},
    io::Read as _,
    iter,
    path::{Path, PathBuf},
    str::FromStr as _,
};
use thiserror::Error;
use utoipa::ToSchema;
use xdgkit::{basedir, categories::Categories, desktop_entry::DesktopEntry, icon_finder};

#[derive(Debug)]
pub struct Application {
    entry: DesktopEntry,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
#[expect(clippy::module_name_repetitions, reason = "more clear")]
/// Serializable application
pub struct ApplicationInfo {
    name: String,
    // icon: TODO:
    exec: String,
    category: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, ToSchema)]
#[expect(clippy::module_name_repetitions, reason = "more clear")]
/// Application data for the web
pub struct ApplicationDisplay {
    #[schema(example = "Satisfactory")]
    name: String,
    // icon: TODO: (url)
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, ToSchema)]
#[expect(clippy::module_name_repetitions, reason = "more clear")]
pub struct GroupedApplication {
    groups: HashMap<String, Vec<ApplicationDisplay>>,
}

impl Application {
    pub fn parse(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let mut buf = String::new();
        File::open(&path)?.read_to_string(&mut buf)?;

        let entry = DesktopEntry::read(buf);
        Ok(Self { entry })
    }

    pub fn name(&self) -> &Option<String> {
        &self.entry.name
    }

    pub fn icon(&self) -> Option<PathBuf> {
        let path_str = self.entry.icon.clone()?;
        let Ok(path) = PathBuf::from_str(&path_str);
        if path.is_absolute() {
            return Some(path);
        }
        icon_finder::find_icon(path_str, 48, 1)
    }

    pub fn exec(&self) -> &Option<String> {
        &self.entry.exec
    }
    pub fn categories(&self) -> Vec<Categories> {
        self.entry.categories.clone().unwrap_or_default()
    }
}

pub fn list_local_applications() -> anyhow::Result<Vec<Application>> {
    basedir::applications()?
        .split(':')
        .flat_map(fs::read_dir)
        .flat_map(iter::IntoIterator::into_iter)
        .filter_map(|res| res.ok().map(|entry| entry.path()))
        .filter(|path| path.is_file() && path.extension() == Some(OsStr::new("desktop")))
        .map(Application::parse)
        .collect()
}

#[expect(clippy::module_name_repetitions, reason = "more clear")]
#[derive(Debug, Error)]
pub enum ApplicationInfoErrorKind {
    #[error("Missing a name")]
    NoName,
    #[error("Missing the exec field")]
    NoExec,
}

#[expect(clippy::module_name_repetitions, reason = "more clear")]
#[derive(Debug, Error)]
#[error("Application {application:#?}: {kind:#}")]
pub struct ApplicationInfoError {
    application: Application,
    kind: ApplicationInfoErrorKind,
}

impl TryInto<ApplicationInfo> for Application {
    type Error = ApplicationInfoError;

    fn try_into(self) -> Result<ApplicationInfo, Self::Error> {
        let Some(name) = self.name() else {
            return Err(ApplicationInfoError {
                application: self,
                kind: ApplicationInfoErrorKind::NoName,
            });
        };
        let Some(exec) = self.exec() else {
            return Err(ApplicationInfoError {
                application: self,
                kind: ApplicationInfoErrorKind::NoExec,
            });
        };
        let category = self.categories().first().copied().unwrap_or_default();
        let category = category.to_string();
        let category = category.strip_prefix('"').unwrap_or(&category);
        let category = category.strip_suffix('"').unwrap_or(category);
        let category = if category.is_empty() {
            "Misc"
        } else {
            category
        };
        let category = category.to_owned();

        Ok(ApplicationInfo {
            name: name.to_owned(),
            exec: exec.to_owned(),
            category,
        })
    }
}

impl From<Vec<ApplicationInfo>> for GroupedApplication {
    fn from(value: Vec<ApplicationInfo>) -> Self {
        let groups = value
            .into_iter()
            .map(|info| (info.category.clone(), ApplicationDisplay::from(info)))
            .into_group_map();
        Self { groups }
    }
}

impl From<ApplicationInfo> for ApplicationDisplay {
    fn from(value: ApplicationInfo) -> Self {
        Self { name: value.name }
    }
}
