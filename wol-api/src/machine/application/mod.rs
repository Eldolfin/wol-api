use anyhow::Context;
mod icon;
use image::{DynamicImage, GenericImageView, ImageBuffer, ImageReader, Pixel, Rgba};
use itertools::Itertools as _;
use log::warn;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self, File},
    io::{Error, Read as _},
    iter,
    path::{Path, PathBuf},
    str::FromStr as _,
};
use thiserror::Error;
use utoipa::ToSchema;
use xdgkit::{basedir, categories::Categories, desktop_entry::DesktopEntry, icon_finder};

use crate::cache;

#[derive(Debug, Clone)]
pub struct Application {
    entry: DesktopEntry,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
#[expect(clippy::module_name_repetitions, reason = "more clear")]
/// Serializable application
pub struct ApplicationInfo {
    pub name: String,
    icon_bytes: Vec<u8>,
    icon_name: String,
    pub exec: String,
    category: String,
}
impl ApplicationInfo {
    fn icon(&self) -> image::DynamicImage {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "It won't be truncated because it's < 2^52 or something"
        )]
        let size = ((self.icon_bytes.len() / 4) as f64).sqrt() as u32;
        let mut buf: ImageBuffer<Rgba<u8>, Vec<_>> = ImageBuffer::new(size, size);
        buf.copy_from_slice(&self.icon_bytes);
        DynamicImage::from(buf)
    }

    fn icon_name(&self) -> &str {
        &self.icon_name
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, ToSchema)]
#[expect(clippy::module_name_repetitions, reason = "more clear")]
/// Application data for the web
pub struct ApplicationDisplay {
    #[schema(example = "Satisfactory")]
    name: String,
    #[schema(example = "/api/cache/images/steam_icon_526870.png")]
    icon: String,
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
        icon_finder::find_icon(path_str, cache::IMAGE_SIZE.try_into().unwrap(), 1)
        // .or(icon_finder::find_icon( "dialog-question".to_owned(), 48, 1, ))
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
    #[error("Missing an icon")]
    NoIcon,
    #[error("Failed to read icon {0}")]
    FailedToReadIcon(Error),
    #[error("Failed to decode icon {0}")]
    FailedToDecodeIcon(image::ImageError),
}

#[expect(clippy::module_name_repetitions, reason = "more clear")]
#[derive(Debug, Error)]
#[error("Application {}: {kind:#}", application.name().clone().unwrap_or_else(|| "<No Name>".to_owned()))]
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

        let icon = ImageReader::open(self.icon().ok_or(ApplicationInfoError {
            application: self.clone(),
            kind: ApplicationInfoErrorKind::NoIcon,
        })?)
        .map_err(|err| ApplicationInfoError {
            application: self.clone(),
            kind: ApplicationInfoErrorKind::FailedToReadIcon(err),
        })?
        .decode()
        .map_err(|err| ApplicationInfoError {
            application: self.clone(),
            kind: ApplicationInfoErrorKind::FailedToDecodeIcon(err),
        })?;
        let icon_bytes = icon
            .pixels()
            .flat_map(|(_, _, rgba)| rgba.channels().to_vec())
            .collect_vec();

        #[expect(clippy::map_unwrap_or, reason = "unreachable :)")]
        let icon_name = self
            .icon()
            .map(|path| path.file_name().unwrap().to_str().unwrap().to_string())
            .unwrap_or_else(|| "no-icon".to_string());

        Ok(ApplicationInfo {
            name: name.to_owned(),
            exec: exec.to_owned(),
            category,
            icon_bytes,
            icon_name,
        })
    }
}

impl TryFrom<ApplicationInfo> for ApplicationDisplay {
    type Error = anyhow::Error;

    fn try_from(value: ApplicationInfo) -> Result<Self, Self::Error> {
        let icon = value.icon();
        let icon = cache::cache_image(value.icon_name(), icon)
            .with_context(|| format!("Error while trying to cache icon {}", value.icon_name()))?;
        Ok(Self {
            name: value.name,
            icon,
        })
    }
}

impl From<Vec<ApplicationInfo>> for GroupedApplication {
    fn from(value: Vec<ApplicationInfo>) -> Self {
        let groups = value
            .into_iter()
            .filter_map(|info| {
                Some((
                    info.category.clone(),
                    ApplicationDisplay::try_from(info)
                        .inspect_err(|err| warn!("while processing applications: {:#}", err))
                        .ok()?,
                ))
            })
            .into_group_map();
        Self { groups }
    }
}
