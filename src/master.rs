use crate::auth;
use crate::config;
use crate::config::Config;
use crate::worker::MetaChunk;
use axum::extract;
use axum::extract::{ConnectInfo, State};
use axum::http::StatusCode;
use axum::middleware;
use axum::response::{IntoResponse, Json, Response};
use axum::routing::post;
use axum::Router;
use chrono::{DateTime, Utc};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Mutex;
use tracing::{error, info, warn};

const FILE_CHUNK_SIZE: u64 = 512;
const TIMEOUT_IN_MINUTES: i64 = 5;
const REPLICATION_FACTOR: usize = 3;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct MetaStore {
    file_name: String,
    hash: String,
    chunk_id: i32,
    hosts: Vec<Host>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
enum Status {
    Unknown,
    Healthy,
    Dead,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Host {
    ip: String,
    status: Status,
}

lazy_static! {
    static ref METASTATE: Mutex<Vec<MetaStore>> = Mutex::new(vec![]);
    static ref HEARTBEAT: Mutex<HashMap<String, DateTime<Utc>>> = Mutex::new(HashMap::new());
}

pub async fn init(port: &i16) {
    println!("{}", crate::LOGO);

    if let Some(config) = config::get() {
        self::load_snapshot();
        let _ = self::export_compacted_snapshot();

        info!("launching node in [master] mode on port {}...", port);

        let app = Router::new()
            .route("/heartbeat", post(heartbeat))
            .route("/list", post(list))
            .route("/get", post(get))
            .route("/upload", post(upload))
            .route("/remove", post(remove))
            .route_layer(middleware::from_fn(auth::authorise))
            .with_state(config.clone());

        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
            .await
            .unwrap();

        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap()
    } else {
        error!("Error: unable able to load the valid cluster configuration. Please make sure the ENV 'RDFS_ENDPOINT' and 'RDFS_TOKEN' are set");
    }
}

async fn heartbeat(ConnectInfo(addr): ConnectInfo<SocketAddr>) -> String {
    info!("got a heartbeat from worker node -> ...{}", addr);

    if let Ok(mut heartbeat) = HEARTBEAT.lock() {
        let ts = chrono::Utc::now();
        heartbeat
            .entry(addr.ip().to_string())
            .and_modify(|x| *x = ts)
            .or_insert(ts);
    }

    "ok".to_string()
}

#[derive(Deserialize, Serialize)]
struct FileMeta {
    name: String,
}

#[derive(Deserialize, Serialize)]
struct FileUploadMeta {
    name: String,
    hash: String,
    size: u64,
}

#[axum::debug_handler]
async fn list() -> Response {
    info!("list all files");

    let mut files: Vec<String> = vec![];

    if let Ok(memory) = METASTATE.lock() {
        files = memory
            .clone()
            .into_iter()
            .map(|x| x.file_name)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .to_vec();
    }

    if files.len() > 0 {
        return Json(files).into_response();
    }

    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}

#[axum::debug_handler]
async fn get(extract::Json(payload): extract::Json<FileMeta>) -> Response {
    info!("get file with name [{}]", &payload.name);

    let mut file: Vec<MetaStore> = vec![];

    if let Ok(memory) = METASTATE.lock() {
        file = memory
            .clone()
            .into_iter()
            .filter(|x| x.file_name == payload.name)
            .collect::<Vec<MetaStore>>();
    }

    if file.len() > 0 {
        return Json(file).into_response();
    }

    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}

#[axum::debug_handler]
async fn upload(extract::Json(payload): extract::Json<FileUploadMeta>) -> Response {
    info!("upload file with name [{}]", &payload.name);

    /* ---------------------------------------------------------------------------------------------
    **CAUTION:**
    
    so this is the one of most complex part of the master node (the other would be re-balancing),
    here we need to take the total size of the file and split into 512kb chunks (we don't actually
    do the splitting here, just dealing with the meta information about the splitting.)

    then for each chunk we need to randomly allocate worker node(s) such that the replication factor
    is satisfied.

    We would then send back that entire meta information so that the client can then go ahead and
    split up the file and then send each chunk to all the various different worker nodes.

    We need to make sure that the selected worker nodes are "alive" based on the timestamped heart
    beat. For this we can use something like a defined upper "threshold" timeout limit, e.g 5
    minutes.

    of course this strategy is not robust and certainly NOT production grade, because there could be
    so many issues, for example the main one being that once the client gets the meta data back
    some of the worker nodes may fail to receive the file chunks, in this case we don't currently
    have any "fallback" way to retry.

    At the moment if the same file is called to be "uploaded" (with a "force" flag) it could simply
    re-calculate chunk distribution across random worker nodes, that itself is fine in terms of a
    retry mechanism, however it would mean we have the potential for ending up with "orhpaned"
    chunks, we don't care about "overlaping" chunks as that would simply overwrite existing chunks
    so no issues there.

    We may need to consider that "re-calculation" to be something abnormal, and only when the client
    fails to upload to worker nodes, so it maybe a good idea that under normal cases we simply check
    the metastore to see if we already have this file uploaded and return that if that is the case
    so that it's an "immutable" call. However we could potentially include an optional "force" flag
    in the payload which the client would only use in cases of retries/failures, in which case we
    skip any existing uploaded files and just re-calculate (we may need to issue a remove file first)

    These "orphaned" chunks would be chunks that exist on a worker node, but technically without any
    reference in the main master metastore. Over time these would take up waste disk space.

    One way to fix this could be to have some kind of "garbage" collection on the work nodes that
    would need the worker nodes to be able to identify that they have chunks that shouldn't exist,
    this could happen slowly when idle as it's not super important.
    --------------------------------------------------------------------------------------------- */

    let mut worker_nodes: Vec<String> = Vec::new();
    let mut heartbeats = HashMap::new();
    let now = chrono::Utc::now();

    if let Ok(x) = HEARTBEAT.lock() {
        heartbeats = x.clone();
    }

    worker_nodes = heartbeats
        .into_iter()
        .filter(|v| (now - v.1).num_minutes() <= TIMEOUT_IN_MINUTES)
        .map(|v| v.0)
        .collect();

    if worker_nodes.len() >= REPLICATION_FACTOR {
        let chunks = payload.size / FILE_CHUNK_SIZE;
        let mut metastore: Vec<MetaStore> = Vec::new();

        for chunk in 1..chunks {
            // randomly pick X worker nodes
            let hosts: Vec<Host> = worker_nodes
                .choose_multiple(&mut rand::thread_rng(), 3)
                .map(|x| Host {
                    ip: x.to_string(),
                    status: Status::Healthy,
                })
                .collect();

            metastore.push(MetaStore {
                file_name: payload.name.to_string(),
                hash: payload.hash.to_string(),
                chunk_id: chunk as i32,
                hosts: hosts,
            });
        }

        return Json(metastore).into_response();
    }

    if worker_nodes.len() < REPLICATION_FACTOR {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }

    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}

#[axum::debug_handler]
async fn remove(
    State(state): State<Config>,
    extract::Json(payload): extract::Json<FileMeta>,
) -> Response {
    info!("remove file with name [{}]", &payload.name);

    let mut kill_list: Vec<MetaStore> = vec![];

    if let Ok(memory) = METASTATE.lock() {
        kill_list = memory
            .clone()
            .into_iter()
            .filter(|x| x.file_name == payload.name)
            .collect::<Vec<_>>()
            .to_vec();
    }

    let mut kill_hash = String::new();

    if kill_list.len() > 0 {
        for chunk in kill_list {
            for worker in chunk.hosts {
                let chunk_id = format!("{}-{}", chunk.chunk_id, chunk.hash);
                kill_hash = chunk.hash.to_string();
                self::delete_remote_chunk(chunk_id, worker.ip, &state.token);
            }
        }
    }

    if let Ok(mut memory) = METASTATE.lock() {
        memory.retain(|x| x.file_name != payload.name)
    }

    self::append("prune", &kill_hash);

    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}

fn delete_remote_chunk(chunk_id: String, remote_ip: String, token: &str) {
    let data = MetaChunk {
        id: chunk_id.clone(),
    };

    if let Ok(_) = ureq::post(&format!("http://{}:8888/delete-chunk", remote_ip))
        .set("x-rdfs-token", token)
        .send_json(data)
    {
        info!("remote chunk deleted ({})", &chunk_id);
    } else {
        warn!("ERROR: unable to delete remote chunk ({})", &chunk_id);
    }
}

#[allow(dead_code)]
fn create_dummy_snapshot() {
    info!("generating dummy snapshot data...");

    let a = MetaStore {
        file_name: String::from("README.md"),
        hash: String::from("5c9d231c8b6d10f43fd0768ca80755d2"),
        chunk_id: 1,
        hosts: vec![
            Host {
                ip: String::from("192.168.1.80"),
                status: Status::Healthy,
            },
            Host {
                ip: String::from("192.168.1.83"),
                status: Status::Healthy,
            },
        ],
    };

    let b = MetaStore {
        file_name: String::from("README.md"),
        hash: String::from("5c9d231c8b6d10f43fd0768ca80755d2"),
        chunk_id: 2,
        hosts: vec![
            Host {
                ip: String::from("192.168.1.81"),
                status: Status::Healthy,
            },
            Host {
                ip: String::from("192.168.1.82"),
                status: Status::Healthy,
            },
        ],
    };

    let c = MetaStore {
        file_name: String::from("README.md"),
        hash: String::from("5c9d231c8b6d10f43fd0768ca80755d2"),
        chunk_id: 2,
        hosts: vec![
            Host {
                ip: String::from("192.168.1.82"),
                status: Status::Healthy,
            },
            Host {
                ip: String::from("192.168.1.255"),
                status: Status::Healthy,
            },
        ],
    };

    let mut w = File::create("snapshot").unwrap();
    writeln!(&mut w, "{}", json!(a)).unwrap();
    writeln!(&mut w, "{}", json!(b)).unwrap();
    writeln!(&mut w, "{}", json!(c)).unwrap();
}

fn load_snapshot() {
    /* ---------------------------------------------------------------------------------------------
    attempt to load from snapshot from disk into memory, we will need to also do
    compaction and then re-save the compacted snapshot back to disk.
    This will then allow the change events to be appended while the process is running.
    ---------------------------------------------------------------------------------------------- */
    info!("attempting to load snapshot...");

    // self::create_dummy_snapshot();

    let mut prune = Vec::new();

    if let Ok(v) = self::read_lines("prune") {
        prune = v;
    }

    if Path::new("snapshot").exists() {
        info!("existing snapshot detected!");
        let snapshot = File::open("snapshot").unwrap();
        let reader = BufReader::new(snapshot);

        let mut compactor: HashMap<(String, i32), MetaStore> = HashMap::new();

        if let Ok(mut memory) = METASTATE.lock() {
            for line in reader.lines() {
                if let Ok(_line) = line {
                    if let Ok(disk) = serde_json::from_str::<MetaStore>(&_line) {
                        if !prune.contains(&disk.hash) {
                            compactor
                                .entry((disk.hash.to_string(), disk.chunk_id))
                                .and_modify(|x| *x = disk.clone())
                                .or_insert(disk);
                        }
                    }
                }
            }
            for (_, v) in compactor {
                memory.push(v);
            }
        }

        if let Ok(memory) = METASTATE.lock() {
            info!(
                "total chunks loaded into memory after compaction: {}",
                memory.len()
            )
        }
    }
}

fn export_compacted_snapshot() -> Result<(), std::io::Error> {
    info!("attempting to export compacted snapshot...");
    // FIX: we're ignoring any errors, at some point we need to consider
    // handling them
    if let Ok(memory) = METASTATE.lock() {
        let mut w = File::create("snapshot.new")?;
        for v in &*memory {
            writeln!(&mut w, "{}", json!(v))?;
        }
    }
    std::fs::remove_file("snapshot")?;
    std::fs::rename("snapshot.new", "snapshot")?;
    std::fs::remove_file("prune")?;
    Ok(())
}

fn append(f: &str, d: &str) {
    let mut h = OpenOptions::new().write(true).append(true).open(f).unwrap();

    if let Err(e) = writeln!(h, "{}", d) {
        warn!("unable to append to file: {}", e);
    }
}

fn read_lines(p: &str) -> Result<Vec<String>, std::io::Error> {
    let f = File::open(p)?;
    let r = BufReader::new(f);
    let mut v = Vec::new();
    for l in r.lines() {
        if let Ok(l) = l {
            v.push(l);
        }
    }
    Ok(v)
}
