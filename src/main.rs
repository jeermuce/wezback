use rand::seq::IndexedRandom;
use std::env;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

const PATH_OF_LIST: &str = "~/.config/wezterm/list_of_wallpapers.txt";
const PATH_OF_CONFIG: &str = "~/.config/wezterm/wezterm.lua";
const INTERVAL_SECONDS: u64 = 600;
const PATH_OF_IMAGES: &str = "well/wallpaper/";

fn expand_tilde(path: &str) -> String {
    if path.starts_with("~") {
        if let Some(home) = env::var_os("HOME") {
            return path.replacen("~", home.to_str().unwrap(), 1);
        }
    }
    path.to_string()
}

fn select_random_wallpaper(path: &str) -> Option<String> {
    let expanded_path = expand_tilde(path);
    let contents = fs::read_to_string(expanded_path).ok()?;
    let lines: Vec<&str> = contents.lines().filter(|line| !line.is_empty()).collect();

    lines.choose(&mut rand::rng()).map(|s| s.to_string())
}

fn update_config_file(config_path: &str, new_image: &str) {
    let new_image = format!("{PATH_OF_IMAGES}{new_image}");
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
            if line.trim_start().starts_with("local image_path") {
                format!("local image_path = '{}'", new_image)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n");

    if let Err(e) = fs::write(path, updated_contents) {
        eprintln!("Error writing config file: {}", e);
    } else {
        println!("Updated config with new image: {}", new_image);
    }
}

fn main() {
    loop {
        if let Some(new_image) = select_random_wallpaper(PATH_OF_LIST) {
            update_config_file(PATH_OF_CONFIG, &new_image);
        } else {
            eprintln!("Could not select a wallpaper.");
        }
        thread::sleep(Duration::from_secs(INTERVAL_SECONDS));
    }
}
