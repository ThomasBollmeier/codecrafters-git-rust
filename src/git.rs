use anyhow::Result;
use objects::{GitObject, TreeEntryMode};
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
        GitObject::Blob(blob) => Ok(blob.iter().map(|b| *b as char).collect()),
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

pub fn ls_tree(tree: &str, name_only: bool) -> Result<String> {
    let tree_path = format!(".git/objects/{}/{}", &tree[0..2], &tree[2..]);
    let git_object = GitObject::read(&tree_path)?;

    match git_object {
        GitObject::Tree(entries) => {
            let output = entries
                .iter()
                .map(|entry| {
                    if name_only {
                        entry.name.clone()
                    } else {
                        format!(
                            "{} {} {}",
                            mode_to_string(&entry.mode),
                            entry.name,
                            bytes_to_hex(&entry.hash)
                        )
                    }
                })
                .collect::<Vec<String>>()
                .join("\n") + "\n";
            
            Ok(output)
        }
        _ => Err(anyhow::anyhow!("Unsupported git object type")),
    }
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn mode_to_string(mode: &TreeEntryMode) -> String {
    match mode {
        TreeEntryMode::Directory => "40000".to_string(),
        TreeEntryMode::RegularFile => "100644".to_string(),
        TreeEntryMode::ExecutableFile => "100755".to_string(),
        TreeEntryMode::SymbolicLink => "120000".to_string(),
    }
}
