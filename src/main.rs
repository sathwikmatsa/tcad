use chrono::prelude::*;
use dotenv::dotenv;
use fs2::FileExt;
use serde::Deserialize;
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::fs::File as FsFile;
use std::path::{Path, PathBuf};
use std::io::Write;
use std::process::Command;

#[cfg(not(target_os = "windows"))]
use notify_rust::Notification;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct File {
    name: String,
    #[serde(rename = "type")]
    kind: String,
    mtime: String,
    locked: bool,
    download_count: u8,
    size: u64,
    childs: Option<Vec<File>>,
    url: String,
    download: Option<String>,
    path: Option<String>,
}

fn create_directories(file: &File, parent_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if file.kind == "folder" {
        // ignore /folder/ which is 8 chars
        let folder_path = parent_dir.join(&file.url[8..]);

        fs::create_dir_all(folder_path)?;

        if let Some(childs) = &file.childs {
            for f in childs.iter() {
                create_directories(f, parent_dir)?;
            }
        }
    }

    Ok(())
}

fn download_files(file: File, parent_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // ignore /folder/ which is 8 chars
    let current_path = parent_dir.join(&file.url[8..]);

    if file.kind == "file" {
        Command::new("wget")
            .arg("-q")
            .arg("-c")
            .arg(format!(
                "{}{}",
                env::var("TCLOUD_URL")?,
                file.download.unwrap()
            ))
            .arg("-O")
            .arg(&current_path)
            .status()?;
    } else {
        if let Some(childs) = file.childs {
            for f in childs.into_iter() {
                download_files(f, &parent_dir)?;
            }
        }
    }

    // delete file/directory after download
    reqwest::Client::new()
        .delete(format!("{}{}", env::var("TCLOUD_URL")?, file.url).as_str())
        .send()?;

    Ok(())
}

#[cfg(target_os = "windows")]
fn send_notification(torrent_name: &str, fpath: &str) -> Result<(), Box<dyn std::error::Error>> {
    let psh_toast_file;
    if Path::new("./toast.ps1").exists() {
        psh_toast_file = String::from("toast.ps1");
    } else {
        let mut toast_path = env::current_exe()?;
        toast_path.pop();
        toast_path.push("toast.ps1");
        // may not exist
        psh_toast_file = toast_path.into_os_string()
            .into_string()
            .unwrap();
    }

    Command::new("powershell")
        .args(&[
            "-nologo",
            "-executionpolicy",
            "bypass",
            "-File",
            &psh_toast_file,
            torrent_name,
            fpath,
        ])
        .status()?;

    Ok(())
}
#[cfg(not(target_os = "windows"))]
fn send_notification(torrent_name: &str, _fpath: &str) -> Result<(), Box<dyn std::error::Error>> {
    Notification::new()
        .summary(torrent_name)
        .body("TCAD: Download Complete")
        .show()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // load .env
    // search priority: cmd arg > pwd > directory of binary
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let env_path = &args[1];
        println!("Load .env from cmd line arg: {}", env_path);
        dotenv::from_path(env_path)?;
    } else if Path::new("./.env").exists() {
        println!("Load .env from PWD: {:#?}", env::current_dir()?);
        dotenv().ok();
    } else {
        let mut env_path = env::current_exe()?;
        env_path.pop();
        env_path.push(".env");
        println!("Load .env from exe_dir: {:#?}", env_path);
        dotenv::from_path(env_path)?;
    }

    // Check for single instance by trying to acquire an exclusive lock
    // on a file. The program quits if the file is already locked (another instance running).
    let mut lock_file_path = PathBuf::new();
    lock_file_path.push(env::var("LOG_DIR")?);
    lock_file_path.push(".tcad.lock");
    let lock_file = FsFile::create(lock_file_path)?;
    lock_file.try_lock_exclusive()?;

    let mut log_file_path = PathBuf::new();
    log_file_path.push(env::var("LOG_DIR")?);
    log_file_path.push("tcad.log");
    let mut log = OpenOptions::new().append(true).create(true).open(log_file_path)?;
    writeln!(log, "[{}] Active Instance", Local::now())?;

    let url = env::var("TCLOUD_URL")?;
    let download_dir = env::var("DOWNLOAD_DIR")?;
    let download_dirpath = Path::new(&download_dir);

    let folder_url = format!("{}/folder", url);
    let mut res_folders = reqwest::get(folder_url.as_str())?;

    let files_section: File = serde_json::from_str(res_folders.text()?.as_str())?;
    if let Some(files) = files_section.childs {
        for file in files.into_iter() {
            let file_name = file.name.clone();
            writeln!(log, "[{}] Downloading {}", Local::now(), file_name)?;
            let parent_dirpath = &download_dirpath;

            create_directories(&file, parent_dirpath)?;
            download_files(file, parent_dirpath)?;

            writeln!(log, "[{}] Finished download: {}", Local::now(), file_name)?;

            // notify
            writeln!(log, "[{}] Sending notification", Local::now())?;
            send_notification(&file_name, download_dirpath.to_str().unwrap())?;
        }
    }

    // release lock
    lock_file.unlock()?;
    writeln!(log, "[{}] Exiting..", Local::now())?;
    Ok(())
}
