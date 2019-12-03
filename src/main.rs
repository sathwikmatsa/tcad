use chrono::prelude::*;
use clap::{App, Arg, SubCommand};
use dotenv::dotenv;
use fs2::FileExt;
use serde::Deserialize;
use std::env;
use std::fs;
use std::fs::File as FsFile;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(not(target_os = "windows"))]
use notify_rust::Notification;

const WGETLOG_HEAD : usize = 7;
const WGETLOG_TAIL : usize = 3;

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
    let mut log_file_path = PathBuf::from(env::var("LOG_DIR")?);
    log_file_path.push("wget.log");

    if file.kind == "file" {
        Command::new("wget")
            .arg("-c")
            .arg(format!(
                "{}{}",
                env::var("TCLOUD_URL")?,
                file.download.unwrap()
            ))
            .arg("-O")
            .arg(&current_path)
            .arg("-o")
            .arg(&log_file_path)
            .status()?;
    } else {
        if let Some(childs) = file.childs {
            for f in childs.into_iter() {
                download_files(f, &parent_dir)?;
            }
        }
    }

    // delete file after download
    reqwest::Client::new()
        .delete(format!("{}{}", env::var("TCLOUD_URL")?, file.url).as_str())
        .send()?;

    // empty the contents of wget log
    FsFile::create(log_file_path)?;

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
        psh_toast_file = toast_path.into_os_string().into_string().unwrap();
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

fn display_progress() -> Result<(), Box<dyn std::error::Error>> {
    let mut wgetlogfile = PathBuf::from(env::var("LOG_DIR")?);
    wgetlogfile.push("wget.log");

    let nlines: usize = BufReader::new(
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&wgetlogfile)?,
    )
    .lines()
    .count();

    if nlines == 0 {
        println!("No downloads in progress.");
    } else {
        let wgetlog = BufReader::new(FsFile::open(&wgetlogfile)?);
        let mut linenum = 1;
        for line in wgetlog.lines() {
            if linenum <= WGETLOG_HEAD {
                println!("{}", line?);
            } else if linenum >= (nlines - WGETLOG_HEAD + WGETLOG_TAIL) {
                println!("{}", line?);
            }
            linenum += 1;
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let matches = App::new("TCAD")
        .version("0.3")
        .author("Sathwik Matsa <sathwikmatsa@gmail.com>")
        .about("TCloud Automatic Downloader <github.com/sathwikmatsa/tcad>")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom dotenv file")
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("progress")
                .about("prints the progress of current download")
                .version("0.1")
                .author("Sathwik Matsa <sathwikmatsa@gmail.com>"),
        )
        .get_matches();

    let config_env = matches.value_of("config").unwrap_or("");

    // load .env
    // search priority: cmd arg > pwd > directory of binary
    if config_env != "" {
        //println!("Load .env from cmd line arg: {}", config_env);
        dotenv::from_path(config_env)?;
    } else if Path::new("./.env").exists() {
        //println!("Load .env from PWD: {:#?}", env::current_dir()?);
        dotenv().ok();
    } else {
        let mut env_path = env::current_exe()?;
        env_path.pop();
        env_path.push(".env");
        //println!("Load .env from exe_dir: {:#?}", env_path);
        dotenv::from_path(env_path)?;
    }

    if let Some(_) = matches.subcommand_matches("progress") {
        display_progress()?;
        return Ok(());
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
    let mut log = OpenOptions::new()
        .append(true)
        .create(true)
        .open(log_file_path)?;
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
            let root_url = file.url.clone();
            writeln!(log, "[{}] Downloading {}", Local::now(), file_name)?;
            let parent_dirpath = &download_dirpath;

            create_directories(&file, parent_dirpath)?;
            download_files(file, parent_dirpath)?;


            // delete root directory after download
            reqwest::Client::new()
                .delete(format!("{}{}", env::var("TCLOUD_URL")?, root_url).as_str())
                .send()?;

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
