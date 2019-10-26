use dotenv::dotenv;
use regex::Regex;
use regex::RegexBuilder;
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::fs::File as FsFile;
use std::os::unix::io::AsRawFd;
use nix::fcntl::{flock, FlockArg};

#[macro_use]
extern crate lazy_static;

lazy_static! {
    // .S01E07. or .S01. or S01E08 or s01 or S01 or season 1 or .complete.
    static ref TV_RE : Regex = RegexBuilder::new(r"([\.\s]S\d+E\d+[\.\s])|([\.\s]S\d+[\.\s])|(\sseason\s\d+\s)|(\.complete\.)")
        .case_insensitive(true)
        .build()
        .expect("Invalid Regex");
}

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
        Command::new("aria2c")
            .arg(format!(
                "{}{}",
                env::var("TCLOUD_URL")?,
                file.download.unwrap()
            ))
            .arg("-d")
            .arg(&parent_dir)
            .status()?;
    } else {
        if let Some(childs) = file.childs {
            for f in childs.into_iter() {
                download_files(f, &current_path)?;
            }
        }
    }

    // delete file/directory after download
    reqwest::Client::new()
        .delete(format!("{}{}", env::var("TCLOUD_URL")?, file.url).as_str())
        .send()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // load .env
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let env_path = &args[1];
        dotenv::from_path(env_path)?;
    } else {
        dotenv().ok();
    }

    // Check for single instance by trying to acquire an exclusive lock
    // on a file. The program quits if the file is already locked (another instance running).
    let lock_file = FsFile::create(env::var("LOCK_FILE")?)?;
    let lfd = lock_file.as_raw_fd();
    flock(lfd, FlockArg::LockExclusiveNonblock)?;

    let url = env::var("TCLOUD_URL")?;
    let movies_dir = env::var("MOVIES_DIR")?;
    let movies_dirpath = Path::new(&movies_dir);

    let tv_dir = env::var("TV_DIR")?;
    let tv_dirpath = Path::new(&tv_dir);

    let folder_url = format!("{}/folder", url);
    let mut res_folders = reqwest::get(folder_url.as_str())?;

    let files_section: File = serde_json::from_str(res_folders.text()?.as_str())?;
    if let Some(files) = files_section.childs {
        for file in files.into_iter() {
            let parent_dirpath;
            if TV_RE.is_match(&file.name) {
                parent_dirpath = &tv_dirpath;
            } else {
                parent_dirpath = &movies_dirpath;
            }
            create_directories(&file, parent_dirpath)?;
            download_files(file, parent_dirpath)?;
        }
    }

    // release lock
    drop(lock_file);

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    #[test]
    fn is_tv_torrent() {
        let one = "Peaky.Blinders.S05E01.Black.Tuesday.1080p.AMZN.WEB-DL.DD+5.1.H.264-AJP69[TGx]";
        let two = "Daybreak.2019.S01.COMPLETE.720p.NF.WEBRip.x264-GalaxyTV";
        let three =
            "Daybreak (2019) S01 COMPLETE PROPER 720p NF WEB-DL x264 AAC 3.5GB ESub [MOVCR]";
        let four = "Guilt S01E01 HDTV x264-MTB [eztv]";
        let five = "Daybreak (2019) S01 Complete [Hindi 5.1 + English] 720p WEB-DL x264 MSub ";

        let six = "The Art of Racing in the Rain (2019) [BluRay] [720p] [YTS] [YIFY]";
        let seven = "Fast & Furious Presents: Hobbs & Shaw (2019) [BluRay] [720p] [YTS] [YIFY]";
        let eight = "Fourplay (2018) [WEBRip] [720p] [YTS] [YIFY]";
        let nine = "A Good Woman Is Hard to Find (2019) [WEBRip] [720p] [YTS] [YIFY]";
        let ten = " Limbo.2019.HDRip.XviD.AC3-EVO ";

        let eleven = "Money Heist season 1 complete English x264 1080p Obey[TGx]";
        let twelve = "Money.Heist.S01.SPANISH.1080p.NF.WEBRip.DDP2.0.x264-Mooi1990[rartv]";
        let thirteen = "La.Casa.de.Papel.COMPLETE.1080p.NF.WEBRip.x265.HEVC.2CH-MRN";

        assert!(TV_RE.is_match(one));
        assert!(TV_RE.is_match(two));
        assert!(TV_RE.is_match(three));
        assert!(TV_RE.is_match(four));
        assert!(TV_RE.is_match(five));

        assert!(!TV_RE.is_match(six));
        assert!(!TV_RE.is_match(seven));
        assert!(!TV_RE.is_match(eight));
        assert!(!TV_RE.is_match(nine));
        assert!(!TV_RE.is_match(ten));

        assert!(TV_RE.is_match(eleven));
        assert!(TV_RE.is_match(twelve));
        assert!(TV_RE.is_match(thirteen));
    }
}
