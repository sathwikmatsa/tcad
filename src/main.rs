use dotenv::dotenv;
use std::env;
use serde::{Deserialize};

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

fn create_directories(file: &File, ignore: bool) {
    if file.kind == "folder" {
        if !ignore {
            println!("{}", file.url);
        }

        if let Some(childs) = &file.childs {
            for f in childs.iter() {
                create_directories(f, false);
            }
        }
    }
}

fn download_files(file: &File) {
    if file.kind == "file" {
        println!("{}", file.url);
        println!("{:#?}", file.download);
    } else {
        if let Some(childs) = &file.childs {
            for f in childs.iter() {
                download_files(f);
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let url = env::var("TCLOUD_URL")?;
    let folder_url = format!("{}/folder", url);
    let mut res_folders = reqwest::get(folder_url.as_str())?;

    let files : File = serde_json::from_str(res_folders.text()?.as_str())?;
    create_directories(&files, true);
    download_files(&files);

    Ok(())
}
