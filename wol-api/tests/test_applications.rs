use itertools::Itertools as _;
use rstest::rstest;
use std::path::PathBuf;
use std::time::Duration;
use wol_relay_server::machine::application::{list_local_applications, Application};

#[rstest]
#[timeout(Duration::from_secs(1))]
#[case("Satisfactory.desktop", "Satisfactory", None)]
#[case(
    "jetbrains-pycharm-4735460c-16ab-49d5-ba0f-c0d6a1f1099a.desktop",
    "PyCharm Professional 2024.1.4",
    Some("/home/oscar/.local/share/JetBrains/Toolbox/apps/pycharm-professional/bin/pycharm.svg")
)]
#[case("net.lutris.outerwilds3-3.desktop", "outerwilds3", None)]
#[case("userapp-Firefox-5MR5S2.desktop", "Firefox", None)]
#[tokio::test]
async fn test_parse_application(
    #[case] filename: &str,
    #[case] name: &str,
    #[case] icon_path: Option<&str>,
) -> anyhow::Result<()> {
    let dir = PathBuf::from("tests/assets/applications/");
    let application = Application::parse(dir.join(filename)).await?;
    assert_eq!(
        application
            .name()
            .clone()
            .expect("Application to have a name")
            .clone(),
        name
    );
    assert_eq!(
        application
            .icon()
            .clone()
            .map(|path| path.display().to_string()),
        icon_path.map(ToOwned::to_owned)
    );
    Ok(())
}

#[tokio::test]
async fn test_list_applications() -> anyhow::Result<()> {
    let mut applications: Vec<String> = list_local_applications()
        .await?
        .into_iter()
        .filter_map(|application| application.name().clone())
        .unique()
        .collect();
    applications.sort();
    assert!(!applications.is_empty());
    Ok(())
}
