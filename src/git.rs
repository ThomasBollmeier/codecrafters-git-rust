use anyhow::Result;
use objects::GitObject;
use sha1::Digest;
use std::fs;

pub mod objects;

pub fn init() {
    fs::create_dir(".git").unwrap();
    fs::create_dir(".git/objects").unwrap();
    fs::create_dir(".git/refs").unwrap();
    fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
}

pub fn cat_file(hash: &str) -> Result<String> {
    let object_path = format!(".git/objects/{}/{}", &hash[0..2], &hash[2..]);
    let git_object = GitObject::read(&object_path)?;

    match git_object {
        GitObject::Blob(blob) => Ok(String::from_utf8_lossy(&blob).to_string()),
        _ => Err(anyhow::anyhow!("Unsupported git object type")),
    }
}

pub fn hash_object(file_path: &str, write: bool) -> Result<String> {
    let content = fs::read(file_path)?;
    let content_with_header = GitObject::add_blob_header(&content);

    // use sha1 to hash the content
    let mut hasher = sha1::Sha1::new();
    hasher.update(&content_with_header);

    let hash: String = hasher
        .finalize()
        .as_slice()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect();

    if write {
        let object_path = format!(".git/objects/{}/{}", &hash[0..2], &hash[2..]);
        fs::create_dir_all(format!(".git/objects/{}", &hash[0..2]))?;
        GitObject::write_blob(&object_path, &content_with_header)?;
    }

    Ok(hash)
}
