use anyhow::Result;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use std::fs;
use std::io::{Read, Write};

pub type Hash = [u8; 20];

#[derive(Debug)]
pub enum GitObject {
    Blob(Vec<u8>),
    Tree(Vec<TreeEntry>),
    Commit {
        tree: Hash,
        parent: Option<Hash>,
        author: PersonInfo,
        committer: PersonInfo,
        message: String,
    },
}

#[derive(Debug)]
pub struct TreeEntry {
    pub mode: TreeEntryMode,
    pub name: String,
    pub hash: Hash,
}

#[derive(Debug)]
pub enum TreeEntryMode {
    Directory,
    RegularFile,
    ExecutableFile,
    SymbolicLink,
}

#[derive(Debug)]
pub struct PersonInfo {
    pub name: String,
    pub email: String,
    pub timestamp: i64,
    pub timezone_offset: i32,
}

impl Default for PersonInfo {
    fn default() -> Self {
        PersonInfo {
            name: "John Doe".to_string(),
            email: "<john.dow@example.com>".to_string(),
            timestamp: 1234567890,
            timezone_offset: 0,
        }
    }
}

impl GitObject {
    pub fn read(object_path: &str) -> Result<GitObject> {
        let compressed_data = fs::read(object_path)?;

        // use flate2 to decompress the data
        let mut decoder = ZlibDecoder::new(&compressed_data[..]);
        let mut decompressed: Vec<u8> = Vec::new();
        decoder.read_to_end(&mut decompressed)?;

        let prefix_blob = b"blob ";
        let prefix_tree = b"tree ";

        if decompressed.starts_with(prefix_blob) {
            let mut offset = prefix_blob.len() + 1; // skip the "blob " prefix and the size
            while decompressed[offset] != 0 {
                offset += 1;
            }
            let content = decompressed[offset + 1..].to_vec();
            return Ok(GitObject::Blob(content));
        } else if decompressed.starts_with(prefix_tree) {
            let mut offset = prefix_tree.len() + 1; // skip the "blob " prefix and the size
            while decompressed[offset] != 0 {
                offset += 1;
            }
            let entries = Self::read_tree_entries(&decompressed[offset + 1..]);
            return Ok(GitObject::Tree(entries));
        }

        Err(anyhow::anyhow!("Unsupported object type"))
    }

    pub fn add_blob_header(content: &[u8]) -> Vec<u8> {
        Self::add_header("blob", content)
    }

    pub fn write_object(object_path: &str, content_with_header: &[u8]) -> Result<()> {
        // use flate2 to compress the data and write it to the object path
        let mut encoder = ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(content_with_header)?;
        let compressed_data = encoder.finish()?;
        fs::write(object_path, compressed_data)?;
        Ok(())
    }

    fn read_tree_entries(content: &[u8]) -> Vec<TreeEntry> {
        let size = content.len();
        let mut size_processed = 0;
        let mut offset = 0;
        let mut entries: Vec<TreeEntry> = Vec::new();

        while size_processed < size {
            let mode_end = content[offset..].iter().position(|&b| b == b' ').unwrap();
            let mode_str = std::str::from_utf8(&content[offset..offset + mode_end]).unwrap();
            let mode = match mode_str {
                "40000" => TreeEntryMode::Directory,
                "100644" => TreeEntryMode::RegularFile,
                "100755" => TreeEntryMode::ExecutableFile,
                "120000" => TreeEntryMode::SymbolicLink,
                _ => panic!("Unknown tree entry mode: {mode_str}"),
            };
            offset += mode_end + 1;

            let name_end = content[offset..].iter().position(|&b| b == 0).unwrap();
            let name = std::str::from_utf8(&content[offset..offset + name_end])
                .unwrap()
                .to_string();
            offset += name_end + 1;

            let hash_bytes: [u8; 20] = content[offset..offset + 20].try_into().unwrap();
            offset += 20;

            size_processed += mode_end + 1 + name_end + 1 + 20;

            let entry = TreeEntry {
                mode,
                name,
                hash: hash_bytes,
            };

            entries.push(entry);
        }

        entries
    }

    pub fn add_tree_header(content: &[u8]) -> Vec<u8> {
        Self::add_header("tree", content)
    }

    pub fn add_commit_header(content: &[u8]) -> Vec<u8> {
        Self::add_header("commit", content)
    }

    pub fn add_header(name: &str, content: &[u8]) -> Vec<u8> {
        let content_size = content.len();
        let header = format!("{name} {content_size}\0");
        [header.as_bytes(), content].concat()
    }
}
