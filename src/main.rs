use iroh::{Endpoint, PublicKey, SecretKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

const KEY_DIR: &str = "./.keys";
const ALPN_PING: &[u8] = b"nateha/iroh-cli/ping";
const ALPN_SYNC: &[u8] = b"nateha/iroh-cli/sync";

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
        let (mut send, mut recv) = conn.accept_bi().await?;
        let m = recv.read_to_end(100).await?;
        println!("{}", String::from_utf8(m)?);
        send.write_all(b"looks like we made it").await?;
        send.finish()?;
        conn.closed().await;
    }
    Ok(())
}

async fn iroh_ping_connect(from_keyname: &str, to_endpoint: &str) -> anyhow::Result<()> {
    println!("pinging from {} to {}", from_keyname, to_endpoint);
    let secret_key = get_secret_key(from_keyname)?;
    let endpoint = Endpoint::builder().secret_key(secret_key).bind().await?;
    let addr: PublicKey = to_endpoint.parse()?;
    let conn = endpoint.connect(addr, b"nateha/iroh-cli/ping").await?;
    let (mut send, mut recv) = conn.open_bi().await?;
    println!("connection opened");
    send.write_all(b"did we make it?").await?;
    println!("checking to see if we made it");
    send.finish()?;
    let m = recv.read_to_end(100).await?;
    println!("{}", String::from_utf8(m)?);
    conn.close(0u8.into(), b"done");
    conn.closed().await;
    Ok(())
}

#[derive(Debug, Clone)]
struct Sync;

impl iroh::protocol::ProtocolHandler for Sync {
    // basic protocal for manifest sync
    // connector sends manifest
    // reciever checks remote manifest against local
    // for each different file, get remote
    fn accept(
        &self,
        connection: iroh::endpoint::Connection,
    ) -> n0_future::boxed::BoxFuture<anyhow::Result<()>> {
        Box::pin(async move {
            let remote_endpoint_id = connection.remote_id()?;
            let remote_device_name = endpoint_to_device_name(&remote_endpoint_id).await?;
            // create manifest for ./data/
            // read manifest length
            // read bytes, deserialize to manifest

            // - compare manifest to on-disk layout at data/{my_endpoint_id}/mirror_for/{their_petname}
            // - for each file, request file
            //   - wait for file
            //   - write file to disk
            // - send eof

            Ok(())
        })
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
        .alpns(vec![ALPN_SYNC.to_vec()])
        .bind()
        .await?;

    let router = iroh::protocol::Router::builder(endpoint)
        .accept(ALPN_SYNC, Sync)
        .spawn();

    Ok(())
}

async fn sync_push(from_keyname: &str, to_keyname: &str) -> anyhow::Result<()> {
    let dir = format!("./data/from_{}/to_{}", from_keyname, to_keyname);
    let manifest = directory_to_manifest(&dir);

    let secret_key = get_secret_key(from_keyname)?;
    let endpoint = Endpoint::builder().secret_key(secret_key).bind().await?;
    let to_endpoint = get_secret_key(to_keyname)?.public();
    let conn = endpoint
        .connect(to_endpoint, b"nateha/iroh-cli/sync")
        .await?;
    let (mut send, mut recv) = conn.open_bi().await?;

    let encoded = bincode::serialize(&manifest)?;
    let len = encoded.len() as u32;
    send.write_all(&len.to_be_bytes()).await?;
    send.write_all(&encoded).await?;

    // while not remote_eof:
    // read(file_name)
    // if file_name not in manifest: bail
    // read_file
    // send_file_len
    // send_file_contents

    send.write_all(b"did we make it?").await?;
    send.finish()?;
    let m = recv.read_to_end(100).await?;
    conn.close(0u8.into(), b"done");
    conn.closed().await;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct Manifest {
    files: HashMap<String, String>, // path -> checksum
}

async fn directory_to_manifest(path_to_dir: &str) -> anyhow::Result<Manifest> {
    let mut m = Manifest {
        files: HashMap::new(),
    };
    for entry in WalkDir::new(path_to_dir) {
        let entry = entry?;
        if entry.file_type().is_file() {
            m.files.insert(
                entry.file_name().to_string_lossy().to_string(),
                hash_file(entry.path()).await?,
            );
        }
    }

    Ok(m)
}

async fn hash_file(path: &Path) -> anyhow::Result<String> {
    let data = fs::read(path)?;
    let hash = blake3::hash(&data);
    Ok(hash.to_hex().to_string())
}

async fn endpoint_to_device_name(endpoint: &PublicKey) -> anyhow::Result<String> {
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

    anyhow::bail!("no device found for endpoint {}", endpoint)
}

fn get_secret_key(name: &str) -> anyhow::Result<SecretKey> {
    let key_file = Path::new(KEY_DIR).join(name);
    if !key_file.exists() {
        anyhow::bail!("no key for {}", name);
    }
    let secret_key_bytes = fs::read(key_file)?;
    let secret_key_array: [u8; 32] = secret_key_bytes.try_into().expect("failed to read key");
    let secret_key = SecretKey::from_bytes(&secret_key_array);
    Ok(secret_key)
}
