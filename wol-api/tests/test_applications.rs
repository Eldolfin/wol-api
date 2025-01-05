use itertools::Itertools as _;
use rstest::rstest;
use std::path::PathBuf;
use wol_relay_server::machine::application::{list_local_applications, Application};

#[rstest]
#[case("Satisfactory.desktop", "Satisfactory", None)]
#[case(
    "jetbrains-pycharm-4735460c-16ab-49d5-ba0f-c0d6a1f1099a.desktop",
    "PyCharm Professional 2024.1.4",
    Some("/home/oscar/.local/share/JetBrains/Toolbox/apps/pycharm-professional/bin/pycharm.svg")
)]
#[case("net.lutris.outerwilds3-3.desktop", "outerwilds3", None)]
#[case("userapp-Firefox-5MR5S2.desktop", "Firefox", None)]
fn test_parse_application(
    #[case] filename: &str,
    #[case] name: &str,
    #[case] icon_path: Option<&str>,
) -> anyhow::Result<()> {
    let dir = PathBuf::from("tests/assets/applications/");
    let application = Application::parse(dir.join(filename))?;
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

#[test]
fn test_list_applications() -> anyhow::Result<()> {
    let mut applications: Vec<String> = list_local_applications()?
        .into_iter()
        .filter_map(|application| application.name().clone())
        .unique()
        .collect();
    applications.sort();
    assert_eq!(
        applications,
        vec![
            "A Wine application",
            "About Xfce",
            "Advanced Network Configuration",
            "Alacritty",
            "Bluetooth Adapters",
            "Bluetooth Manager",
            "Chromium",
            "Coin pusher casino",
            "Cool Retro Term",
            "CopyQ",
            "Emote",
            "Firefox",
            "Flameshot",
            "GParted",
            "GTK+ Demo",
            "GVim",
            "Helix",
            "Icon Browser",
            "Jellyfin Media Player",
            "Log Out",
            "Lutris",
            "Manage Printing",
            "NVIDIA X Server Settings",
            "NetworkManager Applet",
            "NixOS Manual",
            "NoiseTorch",
            "Outer wilds",
            "Proton 9.0",
            "Proton Experimental",
            "Proton Hotfix",
            "PyCharm Professional 2024.1.4",
            "Redshift",
            "Ristretto Image Viewer",
            "Rofi",
            "Rofi Theme Selector",
            "Satisfactory",
            "Session and Startup",
            "Signal",
            "Steam",
            "Steam Linux Runtime 1.0 (scout)",
            "Steam Linux Runtime 2.0 (soldier)",
            "Steam Linux Runtime 3.0 (sniper)",
            "Thunderbird",
            "Ubuntu-22-04",
            "Vesktop",
            "Vim",
            "Volume Control",
            "Widget Factory",
            "Wine Installer",
            "Wine Windows Program Loader",
            "Winetricks",
            "XTerm",
            "Yazi",
            "btop++",
            "dev",
            "feh",
            "hh",
            "i3",
            "mpv Media Player",
            "nvtop",
            "outer-wilds-2",
            "outerwilds3",
            "umpv Media Player",
            "winhlp32",
            "wscript"
        ]
    );
    Ok(())
}
