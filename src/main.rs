use anyhow::Context;
use anyhow::{Error, Result};
use clearscreen::clear;
use rand::seq::IndexedRandom;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::str::FromStr;
const PATH_OF_CONFIG: &str = "~/.config/wezback";
const EXTENSIONS: [&str; 12] = [
    "jpeg", "jpg", "png", "gif", "bmp", "ico", "webp", "tiff", "pnm", "dds", "tga", "farbfeld",
];
fn expand_tilde(path: &str) -> Result<String, Error> {
    if path.starts_with("~") {
        let home = match env::var_os("HOME") {
            Some(v) => v.display().to_string(),
            None => {
                eprintln!("Unable to access HOME variable.");
                process::exit(-1)
            }
        };
        return Ok(path.replacen("~", &home, 1));
    }
    Ok(path.to_string())
}

fn load_wezback_config() -> Result<(String, String, String)> {
    let config = expand_tilde(PATH_OF_CONFIG)?;
    let config_contents = fs::read_to_string(&config)?;

    let mut images = None;
    let mut wezlua = None;
    let mut animations = None;

    for line in config_contents.lines() {
        if let Some(value) = line.strip_prefix("images = ") {
            images = Some(expand_tilde(value.trim_matches('"')));
        } else if let Some(value) = line.strip_prefix("wezlua = ") {
            wezlua = Some(expand_tilde(value.trim_matches('"')));
        } else if let Some(value) = line.strip_prefix("animations = ") {
            animations = Some(expand_tilde(value.trim_matches('"')));
        }
    }

    Ok((
        images.context("Missing 'images' key in config")??,
        wezlua.context("Missing 'wezlua' key in config")??,
        animations.context("Missing 'images' key in config")??,
    ))
}

fn load_list_of_images(path_of_images: &str) -> Result<Vec<String>, Error> {
    let expanded_path_of_images = expand_tilde(path_of_images)?;

    let wallpaper_dir = PathBuf::from_str(&expanded_path_of_images)?.canonicalize()?;

    let paths = fs::read_dir(&wallpaper_dir)?;

    let mut images = Vec::new();

    let home = env::var_os("HOME")
        .ok_or_else(|| anyhow::anyhow!("HOME environment variable not found"))?;
    let home_path = Path::new(&home);

    for entry in paths {
        let entry = entry?;
        let path = entry.path();

        let extension = match path.extension().and_then(OsStr::to_str) {
            Some(ext) => ext,
            None => continue,
        };

        if EXTENSIONS.contains(&extension) {
            let stripped_path = match path.strip_prefix(home_path) {
                Ok(stripped) => stripped.to_string_lossy().to_string(),
                Err(_) => continue,
            };

            images.push(stripped_path);
        }
    }

    Ok(images)
}

fn select_random_wallpaper(images: &[String]) -> Option<String> {
    images.choose(&mut rand::rng()).map(|s| s.to_string())
}

fn update_config_file(config_path: &str, new_image: &str) -> Result<()> {
    let new_image = new_image.to_string();
    let conf_line = format!("local image_path = home .. '/{}'", &new_image);

    let expanded_path = expand_tilde(config_path)?;
    let path = Path::new(&expanded_path);

    let config_contents = fs::read_to_string(path)?;
    let updated_contents = config_contents
        .lines()
        .map(|line| {
            if line.trim_start().starts_with("local image_path") && line != conf_line {
                conf_line.clone()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n");

    fs::write(path, updated_contents)?;
    Ok(())
}
use clap::Parser;

/// Command-line arguments for the Wezback application
#[derive(Parser, Debug)]
#[command(version, about = "Wallpaper changer for Wezterm")]
struct Args {
    /// Include all images
    #[arg(short = 'a', long = "all", conflicts_with = "no_static")]
    all: bool,

    /// Use only animations
    #[arg(short = 'n', long = "no-static", conflicts_with = "all")]
    no_static: bool,

    /// Change wallpaper once and exit
    #[arg(short = 'o', long = "once")]
    once: bool,
    /// Config help
    #[arg(short = 'c', long = "config-help")]
    config_help: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let (path_of_images, path_of_wezlua, animations) = load_wezback_config()?;

    let mut images = load_list_of_images(&path_of_images)?;

    if args.once {
        if let Some(new_image) = select_random_wallpaper(&images) {
            update_config_file(&path_of_wezlua, &new_image)?;
        } else {
            eprintln!("Could not select a wallpaper.");
        }
        return Ok(());
    }

    if args.config_help {
        let help = "Configured in ~/.config/wezback
images = \"[path of images, absolute or relative to home]\"
wezlua = \"[location of the wezterm.lua configuration file, absolute or relative to home]\"
animations = \"[location of animated images, absolute or relative to home]\""
            .to_string();
        println!("{help}");
        return Ok(());
    }

    if args.all {
        let animations = load_list_of_images(&animations)?;
        images.extend(animations);
    } else if args.no_static {
        images = load_list_of_images(&animations)?;
    }

    loop {
        if let Some(new_image) = select_random_wallpaper(&images) {
            update_config_file(&path_of_wezlua, &new_image)?;
        } else {
            eprintln!("Could not select a wallpaper.");
        }
        print!("Press Enter to change the wallpaper or Ctrl+C to exit...");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        clear()?;
    }
}
