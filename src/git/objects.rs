use std::fs;
use std::io::{Read, Write};
use anyhow::Result;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;

#[derive(Debug)]
pub enum GitObject {
    Blob(Vec<u8>),
    Tree(Vec<(String, String)>), // (name, hash)
    Commit {
        tree: String,
        parent: Option<String>,
        message: String,
    },
}

impl GitObject {

    pub fn read(object_path: &str) -> Result<GitObject> {
        let compressed_data = fs::read(object_path)?;

        // use flate2 to decompress the data
        let mut decoder = ZlibDecoder::new(&compressed_data[..]);
        let mut decompressed: Vec<u8> = Vec::new();
        decoder.read_to_end(&mut decompressed)?;

        let prefix_blob = b"blob ";

        if decompressed.starts_with(prefix_blob) {
            let mut offset = prefix_blob.len() + 1; // skip the "blob " prefix and the size
            while decompressed[offset] != 0 {
                offset += 1;
            }
            let content = decompressed[offset+1..].to_vec();
            return Ok(GitObject::Blob(content));
        }

        Err(anyhow::anyhow!("Unsupported object type"))
    }

    pub fn add_blob_header (content: &[u8]) -> Vec<u8> {
        let content_size = content.len();
        let header = format!("blob {content_size}\0");
        [header.as_bytes(), content].concat()
    }

    pub fn write_blob(object_path: &str, content_with_header: &[u8]) -> Result<()> {
        // use flate2 to compress the data and write it to the object path
        let mut encoder = ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(content_with_header)?;
        let compressed_data = encoder.finish()?;
        fs::write(object_path, compressed_data)?;
        Ok(())
    }

}