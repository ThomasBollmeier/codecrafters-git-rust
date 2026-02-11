use std::fs;
use anyhow::Result;
use objects::GitObject;

pub mod objects;

pub fn init() {
    fs::create_dir(".git").unwrap();
    fs::create_dir(".git/objects").unwrap();
    fs::create_dir(".git/refs").unwrap();
    fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
}

pub fn cat_file(hash: &str) -> Result<String> {
    let object_path = format!(".git/objects/{}/{}", &hash[0..2], &hash[2..]);
    let git_object= GitObject::read(&object_path)?;

    match git_object {
        GitObject::Blob(blob) => {
            Ok(String::from_utf8_lossy(&blob).to_string())
        }
        _ => Err(anyhow::anyhow!("Unsupported git object type")),
    }
}