use crate::{Config, LogType};

use super::structs::LogMessage;

const ERRBUFSIZE: usize = 20;

/// General text file to string function.
pub async fn load_file_content(file_path: String) -> (Option<String>, Option<String>) {
    match tokio::fs::read_to_string(file_path).await {
        Ok(contents) => (Some(contents), None),
        Err(errmsg) => (None, Some(format!("{}", errmsg))),
    }
}

/// General string to json text file function.
pub async fn save_str_to_json(
    folder: String,
    content: String,
    file_path: String,
) -> Option<String> {
    match tokio::fs::write(format!("{}/{}.json", folder, file_path), content).await {
        Ok(_) => None,
        Err(errmsg) => Some(format!("{}", errmsg)),
    }
}

/// Used to update available list of contents of given directory. Format (Vec<String>, Error)
pub async fn update_dir_lists(file_path: String) -> (Option<Vec<String>>, Option<String>) {
    match tokio::fs::read_dir(file_path).await {
        Ok(mut readdir) => {
            let mut file_list = Vec::new();
            while let Some(dir_info) = readdir.next_entry().await.unwrap() {
                file_list.push(dir_info.path().display().to_string());
            }
            (Some(file_list), None)
        }
        Err(errmsg) => (None, Some(format!("{}", errmsg))),
    }
}

/// Function for adding an error message to an error array (Used for displaying error history in GUI, not terminal)
pub fn append_log(error_array: &mut Vec<LogMessage>, log_type: LogType, log_message: String) {
    if error_array.is_empty() {
        error_array.push(LogMessage {
            index: 0,
            kind: log_type,
            logmsg: log_message.clone(),
        });
    } else {
        let cur_idx = error_array.iter().last().clone().unwrap().index;
        error_array.push(LogMessage {
            index: cur_idx + 1,
            kind: log_type,
            logmsg: log_message.clone(),
        });

        if error_array.len() > ERRBUFSIZE {
            let _ = error_array.remove(0);
        }
    }
}

/// Loads check if config exists and attempt to create the folders in the config
pub fn configure_startup() {
    // Load config or create one.
    let configuration = match std::fs::read_to_string("./.config.json") {
        Ok(contents) => match serde_json::from_str(&contents) {
            Ok(config_json) => config_json,
            Err(errmsg) => {
                log::error!(
                    "Configuration .config.json couldn't be parsed! It might be corrupted. {}",
                    errmsg
                );
                std::process::exit(1);
            }
        },
        Err(errmsg) => {
            if errmsg.kind() != std::io::ErrorKind::NotFound {
                log::error!("Configuration .config.json rare error! {:?}", errmsg);
                std::process::exit(1);
            } else {
                let new_conf = Config::default();
                match serde_json::to_string_pretty(&new_conf) {
                    Ok(new_conf_text) => match std::fs::write("./.config.json", new_conf_text) {
                        Ok(_) => new_conf,
                        Err(errmsg) => {
                            log::error!(
                                "Error while trying to create new configuration file! {}",
                                errmsg
                            );
                            std::process::exit(1);
                        }
                    },
                    Err(errmsg) => {
                        log::error!("Error while trying to create new configuration! {}", errmsg);
                        std::process::exit(1);
                    }
                }
            }
        }
    };

    let paths = [
        &configuration.font_path,
        &configuration.results_path,
        &configuration.sequences_path,
    ];

    for folder_path in paths {
        create_dir_or_die(&folder_path);
    }
}

/// Does what it says on the tin
fn create_dir_or_die(paths: &String) {
    if let Err(someerr) = std::fs::create_dir(paths) {
        if someerr.kind() != std::io::ErrorKind::AlreadyExists {
            log::error!("Cannot create folder {}", paths);
            std::process::exit(1);
        }
    }
}

/// Delay function, used for arbitrary waiting or long process simulation
pub async fn _wait_n_ms(dur: u64) {
    tokio::time::sleep(tokio::time::Duration::from_millis(dur)).await;
}
