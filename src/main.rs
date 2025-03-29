use anyhow::Context;
use anyhow::Error;
use anyhow::Result;
use clearscreen::clear;
use rand::seq::IndexedRandom;
use std::collections::HashSet;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::str::FromStr;

const PATH_OF_CONFIG: &str = "~/.config/wezback";

fn expand_tilde(path: &str) -> String {
    if path.starts_with("~") {
        let home = match env::var_os("HOME") {
            Some(v) => v.display().to_string(),
            None => {
                eprintln!("Unable to access HOME variable.");
                process::exit(-1)
            }
        };
        return path.replacen("~", &home, 1);
    }
    path.to_string()
}

fn load_wezback_config() -> Result<(String, String)> {
    let config = expand_tilde(PATH_OF_CONFIG);
    let config_contents = fs::read_to_string(&config)
        .with_context(|| format!("Failed to read config file: {}", config))?;

    let mut images = None;
    let mut wezlua = None;

    for line in config_contents.lines() {
        if let Some(value) = line.strip_prefix("images = ") {
            images = Some(expand_tilde(value.trim_matches('"')));
        } else if let Some(value) = line.strip_prefix("wezlua = ") {
            wezlua = Some(expand_tilde(value.trim_matches('"')));
        }
    }

    Ok((
        images.context("Missing 'images' key in config")?,
        wezlua.context("Missing 'wezlua' key in config")?,
    ))
}

fn load_list_of_images(path_of_images: &str) -> Result<Vec<String>, Error> {
    let expanded_path_of_images = expand_tilde(path_of_images);
    let wallpaper_dir = PathBuf::from_str(&expanded_path_of_images)?.canonicalize()?;

    let paths = fs::read_dir(&wallpaper_dir)?;
    let mut extensions = HashSet::new();
    extensions.insert("jpeg");
    extensions.insert("jpg");
    extensions.insert("png");
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
        if extensions.contains(extension) {
            let stripped_path = match path.strip_prefix(home_path) {
                Ok(stripped) => stripped.to_string_lossy().to_string(),
                Err(_) => continue,
            };

            images.push(stripped_path);
        } else {
            continue;
        }
    }

    Ok(images)
}

fn select_random_wallpaper(images: &Vec<String>) -> Option<String> {
    images.choose(&mut rand::rng()).map(|s| s.to_string())
}

fn update_config_file(config_path: &str, new_image: &str) {
    let new_image = new_image.to_string();
    let conf_line = format!("local image_path = home .. '/{}'", &new_image);

    let expanded_path = expand_tilde(config_path);
    let path = Path::new(&expanded_path);

    let config_contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("Error reading config file: {}", e);
            return;
        }
    };

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

    if let Err(e) = fs::write(path, updated_contents) {
        eprintln!("Error writing config file: {}", e);
    } else {
        println!("Updated config with new image: {}", &conf_line);
    }
}

fn main() -> Result<()> {
    let args = env::args().collect::<Vec<String>>();
    let (path_of_images, path_of_wezlua) = load_wezback_config()?;
    let list = load_list_of_images(&path_of_images)?;

    if args.iter().any(|v| v == "--once" || v == "-o") {
        if let Some(new_image) = select_random_wallpaper(&list) {
            update_config_file(&path_of_wezlua, &new_image);
        } else {
            eprintln!("Could not select a wallpaper.");
        }
        Ok(())
    } else {
        loop {
            let _ = clear();
            println!("Press Enter to change the wallpaper or Ctrl+C to exit...");
            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            if let Some(new_image) = select_random_wallpaper(&list) {
                update_config_file(&path_of_wezlua, &new_image);
            } else {
                eprintln!("Could not select a wallpaper.");
            }
        }
    }
}
