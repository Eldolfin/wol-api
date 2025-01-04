use std::{
    fs::{self, File},
    io::Read as _,
    iter,
    path::{Path, PathBuf},
    str::FromStr as _,
};
use xdgkit::{basedir, categories::Categories, desktop_entry::DesktopEntry, icon_finder};

pub struct Application {
    entry: DesktopEntry,
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
        .filter(|path| path.is_file())
        .map(Application::parse)
        .collect()
}
