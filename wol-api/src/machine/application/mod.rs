use anyhow::Context;
use futures_util::future::join_all;
use image::{DynamicImage, GenericImageView, ImageBuffer, ImageReader, Pixel, Rgba};
use itertools::Itertools as _;
use log::warn;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs,
    io::Error,
    iter,
    path::{Path, PathBuf},
    str::FromStr as _,
    sync::LazyLock,
};
use thiserror::Error;
use tokio::{fs::File, io::AsyncReadExt as _};
use utoipa::ToSchema;
use xdgkit::{
    basedir,
    categories::Categories,
    desktop_entry::DesktopEntry,
    icon_finder::{generate_dir_list, multiple_find_icon, user_theme, DirList},
    icon_theme::IconTheme,
};

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
    icon_bytes: Option<Vec<u8>>,
    icon_name: String,
    pub exec: String,
    category: String,
}
impl ApplicationInfo {
    fn icon(&self) -> Option<image::DynamicImage> {
        if let Some(icon_bytes) = &self.icon_bytes {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "It won't be truncated because it's < 2^52 or something"
            )]
            let size = ((icon_bytes.len() / 4) as f64).sqrt() as u32;
            let mut buf: ImageBuffer<Rgba<u8>, Vec<_>> = ImageBuffer::new(size, size);
            buf.copy_from_slice(&icon_bytes);
            Some(DynamicImage::from(buf))
        } else {
            None
        }
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

static DIR_LIST: LazyLock<Vec<DirList>> = LazyLock::new(generate_dir_list);
static THEME: LazyLock<IconTheme> =
    LazyLock::new(|| user_theme(DIR_LIST.clone()).unwrap_or_else(IconTheme::empty));

impl Application {
    pub async fn parse(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let mut buf = String::new();
        File::open(&path).await?.read_to_string(&mut buf).await?;

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

        multiple_find_icon(
            path_str,
            cache::IMAGE_SIZE.try_into().unwrap(),
            1,
            DIR_LIST.to_owned(),
            THEME.to_owned(),
        )
        // .or(icon_finder::find_icon( "dialog-question".to_owned(), 48, 1, ))
    }

    pub fn exec(&self) -> &Option<String> {
        &self.entry.exec
    }
    pub fn categories(&self) -> Vec<Categories> {
        self.entry.categories.clone().unwrap_or_default()
    }
}

pub async fn list_local_applications() -> anyhow::Result<Vec<Application>> {
    let applications = basedir::applications()?;
    let futures = applications
        .split(':')
        .flat_map(fs::read_dir)
        .flat_map(iter::IntoIterator::into_iter)
        .filter_map(|res| res.ok().map(|entry| entry.path()))
        .filter(|path| path.is_file() && path.extension() == Some(OsStr::new("desktop")))
        .map(Application::parse);
    Ok(join_all(futures)
        .await
        .into_iter()
        .filter_map(Result::ok)
        .collect())
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

        // nice 😐️
        let icon_bytes = (|| ImageReader::open(self.icon()?).ok()?.decode().ok())().map(|icon| {
            icon.pixels()
                .flat_map(|(_, _, rgba)| rgba.channels().to_vec())
                .collect_vec()
        });

        #[expect(clippy::map_unwrap_or, reason = "unreachable :)")]
        let icon_name = self
            .icon()
            .map(|path| path.file_name().unwrap().to_str().unwrap().to_owned())
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

impl ApplicationDisplay {
    async fn try_from(value: ApplicationInfo) -> anyhow::Result<Self> {
        let icon = if let Some(icon) = value.icon() {
            cache::cache_image(value.icon_name(), icon).with_context(|| {
                format!("Error while trying to cache icon {}", value.icon_name())
            })?
        } else {
            cache::icon::cache_find_icon(&value.name)
                .await
                .with_context(|| {
                    format!(
                        "Failed to search an icon on the web for application `{}`",
                        &value.name
                    )
                })?
        };
        Ok(Self {
            name: value.name,
            icon,
        })
    }
}

impl GroupedApplication {
    pub async fn from_list(value: Vec<ApplicationInfo>) -> Self {
        let groups = value.into_iter().map(|info| async {
            Some((
                info.category.clone(),
                ApplicationDisplay::try_from(info)
                    .await
                    .inspect_err(|err| warn!("while processing applications: {:#}", err))
                    .ok()?,
            ))
        });
        let groups = join_all(groups)
            .await
            .into_iter()
            .filter_map(|x| x)
            .into_group_map();
        Self { groups }
    }
}
