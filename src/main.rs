use bincode::{Decode, Encode};
use iroh::endpoint::Connection;
use iroh::protocol::AcceptError;
use iroh::{Endpoint, PublicKey, SecretKey};
use n0_error::{Result as N0Result, StdResultExt};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const KEY_DIR: &str = "./.keys";
const ALPN_PING: &[u8] = b"nhardt/iroh-cli/ping";
const ALPN_REMOTE_MIRROR: &[u8] = b"nhardt/iroh-cli/remote_mirror";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();

    match arg_refs.as_slice() {
        ["endpoint", "create"] => {
            create_secret_key("secret").await?;
        }
        ["endpoint", "create", keyname] => {
            // note: this is just for testing, keys will be stored in ./.keys/keyname
            create_secret_key(&keyname).await?;
        }
        ["endpoint", "read"] => {
            print_endpoint("secret").await?;
        }
        ["endpoint", "read", keyname] => {
            print_endpoint(keyname).await?;
        }
        ["ping", "listen"] => {
            iroh_ping_listen("secret").await?;
        }
        ["ping", "listen", keyname] => {
            iroh_ping_listen(keyname).await?;
        }
        ["ping", "connect", addr] => {
            iroh_ping_connect("secret", addr).await?;
        }
        ["ping", "connect", from_keyname, to_endpoint_id] => {
            iroh_ping_connect(from_keyname, to_endpoint_id).await?;
        }
        ["sync", "listen", keyname] => {
            sync_listen(keyname).await?;
        }
        ["sync", "push", from_keyname, to_keyname] => {
            sync_push(from_keyname, to_keyname).await?;
        }
        _ => {
            println!("unknown command");
        }
    }

    Ok(())
}

async fn create_secret_key(name: &str) -> anyhow::Result<()> {
    let key_file = Path::new(KEY_DIR).join(name);
    if key_file.exists() {
        anyhow::bail!("endpoint for {} already exists", name);
    }

    println!("generated key and storing at .keys/{}", name);
    let key = SecretKey::generate(&mut rand::rng()).to_bytes();
    fs::create_dir_all(KEY_DIR)?;
    fs::write(key_file, key)?;
    println!("wrote private key to {}", name);

    Ok(())
}

async fn print_endpoint(name: &str) -> anyhow::Result<()> {
    let secret_key = get_secret_key(name)?;
    eprintln!("this public key (and endpoint id) for {}:", name);
    println!("{}", secret_key.public());

    Ok(())
}

async fn iroh_ping_listen(keyname: &str) -> anyhow::Result<()> {
    let secret_key = get_secret_key(keyname)?;
    println!(
        "listening for ping on key '{}' at {}",
        keyname,
        secret_key.public()
    );
    //let endpoint_id: EndpointId = secret_key.public();
    let endpoint = Endpoint::builder()
        .secret_key(secret_key)
        .alpns(vec![ALPN_PING.to_vec()])
        .bind()
        .await?;
    if let Some(incoming) = endpoint.accept().await {
        println!("someone wants to know");
        let iconn = incoming.accept()?;
        let conn = iconn.await?;
        let (mut send, mut recv) = conn.accept_bi().await.anyerr()?;
        let m = recv.read_to_end(100).await.anyerr()?;
        println!("{}", String::from_utf8(m)?);
        send.write_all(b"looks like we made it").await.anyerr()?;
        send.finish().anyerr()?;
        conn.closed().await;
    }
    Ok(())
}

async fn iroh_ping_connect(from_keyname: &str, to_endpoint: &str) -> anyhow::Result<()> {
    println!("pinging from {} to {}", from_keyname, to_endpoint);
    let secret_key = get_secret_key(from_keyname)?;
    let endpoint = Endpoint::builder().secret_key(secret_key).bind().await?;
    let addr: PublicKey = to_endpoint.parse()?;
    let conn = endpoint.connect(addr, ALPN_PING).await?;
    let (mut send, mut recv) = conn.open_bi().await.anyerr()?;
    println!("connection opened");
    send.write_all(b"did we make it?").await.anyerr()?;
    println!("checking to see if we made it");
    send.finish().anyerr()?;
    let m = recv.read_to_end(100).await.anyerr()?;
    println!("{}", String::from_utf8(m)?);
    conn.close(0u8.into(), b"done");
    conn.closed().await;
    Ok(())
}

#[derive(Debug, Clone)]
struct RemoteMirror {
    local_keyname: String,
}

impl iroh::protocol::ProtocolHandler for RemoteMirror {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let remote_endpoint_id = connection.remote_id();
        let remote_device_name = endpoint_to_device_name(&remote_endpoint_id).await?;

        println!(
            "Receiving sync from {} ({})",
            remote_device_name, remote_endpoint_id
        );

        // Open bidirectional stream
        let (mut send, mut recv) = connection.accept_bi().await?;

        // Read manifest length
        let mut len_buf = [0u8; 4];
        recv.read_exact(&mut len_buf).await.anyerr()?;
        let manifest_len = u32::from_be_bytes(len_buf) as usize;

        // Read and deserialize manifest
        let mut manifest_buf = vec![0u8; manifest_len];
        recv.read_exact(&mut manifest_buf).await.anyerr()?;
        let (remote_manifest, _): (Manifest, _) =
            bincode::decode_from_slice(&manifest_buf, bincode::config::standard()).anyerr()?;

        println!(
            "Received manifest with {} files",
            remote_manifest.files.len()
        );

        // Create local directory and manifest
        let local_dir = format!(
            "./data/{}/mirror_from/{}",
            self.local_keyname, remote_device_name
        );
        fs::create_dir_all(&local_dir)?;
        let local_manifest = directory_to_manifest(&local_dir).await?;

        // Diff: find files to request
        let mut files_to_request = Vec::new();
        for (path, remote_hash) in &remote_manifest.files {
            match local_manifest.files.get(path.as_str()) {
                Some(local_hash) if local_hash == remote_hash => {
                    // File exists and matches, skip
                }
                _ => {
                    // File missing or different, request it
                    files_to_request.push(path.clone());
                }
            }
        }

        println!("Requesting {} files", files_to_request.len());

        // Request and receive each file
        for file_path in &files_to_request {
            // Send file request: path length + path
            let path_bytes = file_path.as_bytes();
            let path_len = path_bytes.len() as u32;
            send.write_all(&path_len.to_be_bytes()).await.anyerr()?;
            send.write_all(path_bytes).await.anyerr()?;

            // Receive file length
            let mut file_len_buf = [0u8; 8];
            recv.read_exact(&mut file_len_buf).await.anyerr()?;
            let file_len = u64::from_be_bytes(file_len_buf) as usize;

            // Receive file contents
            let mut file_contents = vec![0u8; file_len];
            recv.read_exact(&mut file_contents).await.anyerr()?;

            // Write file to disk
            let full_path = PathBuf::from(&local_dir).join(file_path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&full_path, file_contents)?;
            println!("Wrote file: {}", file_path);
        }

        // Send EOF signal (0-length path)
        send.write_all(&0u32.to_be_bytes()).await.anyerr()?;
        send.finish().anyerr()?;

        // Delete local files not in remote manifest
        println!("checking {} for files not in remote", local_dir);
        for (local_path, _) in &local_manifest.files {
            if !remote_manifest.files.contains_key(local_path) {
                let full_path = PathBuf::from(&local_dir).join(local_path);
                if full_path.exists() {
                    fs::remove_file(&full_path)?;
                    println!(
                        "Deleted {}/{}, key {} not in remote",
                        local_dir, local_path, local_path
                    );
                }
            }
        }

        println!("Sync complete!");
        connection.closed().await;

        Ok(())
    }
}

async fn sync_listen(keyname: &str) -> anyhow::Result<()> {
    let secret_key = get_secret_key(keyname)?;
    println!(
        "starting sync listen for key '{}' at {}",
        keyname,
        secret_key.public()
    );
    let endpoint = Endpoint::builder()
        .secret_key(secret_key)
        .alpns(vec![ALPN_REMOTE_MIRROR.to_vec()])
        .bind()
        .await?;

    let _router = iroh::protocol::Router::builder(endpoint)
        .accept(
            ALPN_REMOTE_MIRROR,
            RemoteMirror {
                local_keyname: keyname.to_string(),
            },
        )
        .spawn();

    // Keep the process running
    tokio::signal::ctrl_c().await?;
    Ok(())
}

async fn sync_push(from_keyname: &str, to_keyname: &str) -> anyhow::Result<()> {
    let dir = format!("./data/{}/mirror_to/{}", from_keyname, to_keyname);
    let manifest = directory_to_manifest(&dir).await?;

    println!("Pushing {} files to {}", manifest.files.len(), to_keyname);

    let secret_key = get_secret_key(from_keyname)?;
    let endpoint = Endpoint::builder().secret_key(secret_key).bind().await?;
    let to_endpoint = get_secret_key(to_keyname)?.public();
    let conn = endpoint.connect(to_endpoint, ALPN_REMOTE_MIRROR).await?;
    let (mut send, mut recv) = conn.open_bi().await.anyerr()?;

    // Send manifest
    let encoded = bincode::encode_to_vec(&manifest, bincode::config::standard())?;
    let len = encoded.len() as u32;
    send.write_all(&len.to_be_bytes()).await.anyerr()?;
    send.write_all(&encoded).await.anyerr()?;

    println!("Manifest sent, waiting for file requests...");

    // Loop: read file requests and serve files
    loop {
        // Read file path length
        let mut path_len_buf = [0u8; 4];
        recv.read_exact(&mut path_len_buf).await.anyerr()?;
        let path_len = u32::from_be_bytes(path_len_buf);

        // EOF signal (0-length path)
        if path_len == 0 {
            println!("Received EOF signal, sync complete");
            break;
        }

        // Read file path
        let mut path_buf = vec![0u8; path_len as usize];
        recv.read_exact(&mut path_buf).await.anyerr()?;
        let file_path = String::from_utf8(path_buf)?;

        // Verify file is in manifest
        if !manifest.files.contains_key(&file_path) {
            anyhow::bail!("Requested file not in manifest: {}", file_path);
        }

        // Read file from disk
        let full_path = PathBuf::from(&dir).join(&file_path);
        let file_contents = fs::read(&full_path)?;

        // Send file length + contents
        let file_len = file_contents.len() as u64;
        send.write_all(&file_len.to_be_bytes()).await.anyerr()?;
        send.write_all(&file_contents).await.anyerr()?;

        println!("Sent file: {} ({} bytes)", file_path, file_len);
    }

    send.finish().anyerr()?;
    conn.close(0u8.into(), b"done");
    conn.closed().await;
    Ok(())
}

#[derive(Encode, Decode, Debug)]
struct Manifest {
    files: HashMap<String, String>, // path -> checksum
}

async fn directory_to_manifest(path_to_dir: &str) -> N0Result<Manifest> {
    println!("building manifest for {}", path_to_dir);
    let mut m = Manifest {
        files: HashMap::new(),
    };
    for entry in WalkDir::new(path_to_dir) {
        let entry = entry.anyerr()?;
        if entry.file_type().is_file() {
            m.files.insert(
                entry.file_name().to_string_lossy().to_string(),
                hash_file(entry.path()).await?,
            );
        }
    }

    Ok(m)
}

async fn hash_file(path: &Path) -> N0Result<String> {
    let data = fs::read(path)?;
    let hash = blake3::hash(&data);
    Ok(hash.to_hex().to_string())
}

async fn endpoint_to_device_name(endpoint: &PublicKey) -> N0Result<String> {
    let key_dir = Path::new(KEY_DIR);

    for entry in fs::read_dir(key_dir)? {
        let entry = entry?;
        let keyname = entry.file_name().to_string_lossy().to_string();

        if let Ok(secret_key) = get_secret_key(&keyname) {
            if &secret_key.public() == endpoint {
                return Ok(keyname);
            }
        }
    }

    n0_error::bail_any!("no device found for endpoint {}", endpoint)
}

fn get_secret_key(name: &str) -> N0Result<SecretKey> {
    let key_file = Path::new(KEY_DIR).join(name);
    if !key_file.exists() {
        n0_error::bail_any!("no key for {}", name);
    }
    let secret_key_bytes = fs::read(key_file)?;
    let secret_key_array: [u8; 32] = secret_key_bytes.try_into().expect("failed to read key");
    let secret_key = SecretKey::from_bytes(&secret_key_array);
    Ok(secret_key)
}
