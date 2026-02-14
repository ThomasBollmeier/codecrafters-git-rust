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

pub fn hash_object(file_path: &str, write: bool) -> Result<Vec<u8>> {
    let content = fs::read(file_path)?;
    let content_with_header = GitObject::add_blob_header(&content);

    let hash = create_sha1_hash(&content_with_header);

    if write {
        let hash_str: String = bytes_to_hex(&hash);
        let object_path = format!(".git/objects/{}/{}", &hash_str[0..2], &hash_str[2..]);
        fs::create_dir_all(format!(".git/objects/{}", &hash_str[0..2]))?;
        GitObject::write_object(&object_path, &content_with_header)?;
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
                .join("\n")
                + "\n";

            Ok(output)
        }
        _ => Err(anyhow::anyhow!("Unsupported git object type")),
    }
}

pub fn write_tree(path: &str) -> Result<Vec<u8>> {
    let entries = write_directory_entries(path)?;
    let mut content = Vec::new();

    for (mode, name, hash) in &entries {
        content.extend_from_slice(mode_to_bytes(mode));
        content.push(b' ');
        content.extend_from_slice(name.as_bytes());
        content.push(0); // null byte
        content.extend_from_slice(hash);
    }

    let content_with_header = GitObject::add_tree_header(&content);
    let hash = create_sha1_hash(&content_with_header);

    let hash_str: String = bytes_to_hex(&hash);
    let object_path = format!(".git/objects/{}/{}", &hash_str[0..2], &hash_str[2..]);
    fs::create_dir_all(format!(".git/objects/{}", &hash_str[0..2]))?;
    GitObject::write_object(&object_path, &content_with_header)?;

    Ok(hash)
}

fn write_directory_entries(path: &str) -> Result<Vec<(TreeEntryMode, String, Vec<u8>)>> {
    // This function should read the directory entries from the given path and return a vector of tuples containing the mode, name, and hash of each entry.
    let mut entries = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let file_type = metadata.file_type();

        let name = entry.file_name().into_string().unwrap();
        if name == ".git" {
            continue; // Skip the .git directory
        }

        let mut hash = vec![0u8; 20];
        let mode = if file_type.is_dir() {
            hash = write_tree(&entry.path().to_string_lossy())?; // Recursively write the tree for the directory
            TreeEntryMode::Directory
        } else if file_type.is_file() {
            hash = hash_object(&entry.path().to_string_lossy(), true)?;
            //  Check if the file is executable by checking the execute permission bits
            if is_executable(&metadata) {
                TreeEntryMode::ExecutableFile
            } else {
                TreeEntryMode::RegularFile
            }
        } else if file_type.is_symlink() {
            hash = hash_object(&entry.path().to_string_lossy(), true)?;
            TreeEntryMode::SymbolicLink
        } else {
            continue; // Skip unsupported file types
        };

        entries.push((mode, name, hash));
    }

    // Sort entries by name in lexicographical order
    entries.sort_by(|a, b| a.1.cmp(&b.1));

    Ok(entries)
}

fn create_sha1_hash(content: &[u8]) -> Vec<u8> {
    let mut hasher = sha1::Sha1::new();
    hasher.update(content);
    hasher.finalize().as_slice().to_vec()
}

#[cfg(unix)]
fn is_executable(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(windows)]
fn is_executable(metadata: &std::fs::Metadata) -> bool {
    let path = metadata.permissions();
    path.readonly() == false // Simplified for Windows
}

#[cfg(not(any(unix, windows)))]
fn is_executable(_metadata: &std::fs::Metadata) -> bool {
    false // Default for other platforms
}

pub fn bytes_to_hex(bytes: &[u8]) -> String {
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

fn mode_to_bytes(mode: &TreeEntryMode) -> &[u8] {
    match mode {
        TreeEntryMode::Directory => b"40000",
        TreeEntryMode::RegularFile => b"100644",
        TreeEntryMode::ExecutableFile => b"100755",
        TreeEntryMode::SymbolicLink => b"120000",
    }
}
