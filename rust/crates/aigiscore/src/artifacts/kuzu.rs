//! Publish immutable, derived query databases independently of the JSON artifact family.

use super::{default_output_dir, write_json};
use crate::graph::SemanticGraph;
use crate::ingestion::hash::hash_file_xxh3;
use crate::kuzu_index::{
    native, write_nodes_csv, write_relations_csv, KuzuIndexError, KUZU_DB_NAME,
};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const CURRENT: &str = "kuzu-current.json";
const STORE: &str = ".kuzu-generations";
const SEAL: &str = "kuzu-manifest.json";

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Manifest {
    version: u32,
    generation: String,
    root: PathBuf,
    engine: String,
    nodes_xxh3: String,
    relations_xxh3: String,
    database_xxh3: String,
    database_bytes: u64,
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn read_manifest(path: &Path) -> io::Result<Option<Manifest>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() && metadata.len() <= 65_536 => {}
        Ok(_) => {
            return Err(invalid(
                "Kuzu manifest must be a regular file of at most 64 KiB",
            ))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    }
    let mut bytes = Vec::new();
    File::open(path)?.take(65_537).read_to_end(&mut bytes)?;
    if bytes.len() > 65_536 {
        return Err(invalid("Kuzu manifest exceeds 64 KiB"));
    }
    let manifest: Manifest = serde_json::from_slice(&bytes).map_err(io::Error::other)?;
    if manifest.version != 1
        || manifest.generation.is_empty()
        || manifest.generation.len() > 100
        || !manifest
            .generation
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() || byte == b'-')
    {
        return Err(invalid("unsupported or invalid Kuzu generation manifest"));
    }
    Ok(Some(manifest))
}

fn verify(path: &Path, manifest: &Manifest) -> io::Result<()> {
    let directory = path
        .parent()
        .ok_or_else(|| invalid("Kuzu path has no parent"))?;
    if !fs::symlink_metadata(directory)?.is_dir()
        || directory.file_name().and_then(|name| name.to_str()) != Some(&manifest.generation)
    {
        return Err(invalid(
            "Kuzu generation directory does not match its manifest",
        ));
    }
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file()
        || metadata.len() != manifest.database_bytes
        || hash_file_xxh3(path)?.to_string() != manifest.database_xxh3
    {
        return Err(invalid("Kuzu database failed content verification"));
    }
    Ok(())
}

pub(crate) fn verify_kuzu_pin(path: &Path) -> io::Result<()> {
    let manifest = read_manifest(&path.with_file_name(SEAL))?
        .ok_or_else(|| invalid("Kuzu database has no generation manifest; rebuild from source"))?;
    verify(path, &manifest)
}

/// Locate a sealed export. This establishes integrity, not freshness against source.
pub fn published_kuzu_path(root: &Path, output_dir: Option<&Path>) -> io::Result<Option<PathBuf>> {
    let output = output_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_output_dir(root));
    let Some(manifest) = read_manifest(&output.join(CURRENT))? else {
        return Ok(None);
    };
    if !fs::symlink_metadata(output.join(STORE))?.is_dir() {
        return Err(invalid(
            "Kuzu generation store must be a directory, not a symlink",
        ));
    }
    let path = output
        .join(STORE)
        .join(&manifest.generation)
        .join(KUZU_DB_NAME);
    if read_manifest(&path.with_file_name(SEAL))?.as_ref() != Some(&manifest) {
        return Err(invalid(
            "Kuzu generation seal differs from its publication marker",
        ));
    }
    verify(&path, &manifest)?;
    Ok(Some(path))
}

pub fn write_semantic_graph_kuzu_artifact(
    root: &Path,
    graph: &SemanticGraph,
    output_dir: Option<&Path>,
) -> Result<PathBuf, KuzuIndexError> {
    let output = output_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_output_dir(root));
    fs::create_dir_all(&output)?;
    if output.join(super::publication::SEAL).try_exists()? {
        return Err(invalid("cannot add Kuzu to a sealed analysis generation").into());
    }
    let lease = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(output.join(".kuzu-writer.lock"))?;
    fs4::FileExt::lock(&lease)?;
    let store = output.join(STORE);
    fs::create_dir_all(&store)?;
    if !fs::symlink_metadata(&store)?.is_dir() {
        return Err(invalid("Kuzu generation store must be a directory, not a symlink").into());
    }
    let generation = format!(
        "{:x}-{:x}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos(),
        std::process::id()
    );
    let directory = store.join(&generation);
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(&directory)?;
    let mut published = false;
    let result = (|| {
        let nodes = directory.join("nodes.csv");
        let relations = directory.join("relations.csv");
        write_nodes_csv(&nodes, graph)?;
        write_relations_csv(&relations, graph)?;
        let mut manifest = Manifest {
            version: 1,
            generation,
            root: fs::canonicalize(root)?,
            engine: env!("AIGISCODE_ENGINE_FINGERPRINT").to_owned(),
            nodes_xxh3: hash_file_xxh3(&nodes)?.to_string(),
            relations_xxh3: hash_file_xxh3(&relations)?.to_string(),
            database_xxh3: String::new(),
            database_bytes: 0,
        };
        if let Some(previous) = read_manifest(&output.join(CURRENT))? {
            if previous.root == manifest.root
                && previous.engine == manifest.engine
                && previous.nodes_xxh3 == manifest.nodes_xxh3
                && previous.relations_xxh3 == manifest.relations_xxh3
            {
                return published_kuzu_path(root, Some(&output))?.ok_or_else(|| {
                    invalid("Kuzu publication disappeared while writer lock held").into()
                });
            }
        }
        let db = directory.join(KUZU_DB_NAME);
        native::materialize(&db, &nodes, &relations)?;
        File::open(&db)?.sync_all()?;
        manifest.database_bytes = fs::metadata(&db)?.len();
        manifest.database_xxh3 = hash_file_xxh3(&db)?.to_string();
        fs::remove_file(nodes)?;
        fs::remove_file(relations)?;
        write_json("kuzu.seal", &directory.join(SEAL), &manifest)?;
        sync_directory(&directory)?;
        sync_directory(&store)?;
        write_json("kuzu.current", &output.join(CURRENT), &manifest)?;
        // From this point readers may hold the path, even if the final fsync fails.
        published = true;
        sync_directory(&output)?;
        Ok(db)
    })();
    if !published {
        fs::remove_dir_all(&directory)?;
    }
    result
}

fn sync_directory(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    File::open(path)?.sync_all()?;
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}
