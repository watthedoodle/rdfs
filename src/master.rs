use crate::auth;
use crate::config;
use axum::extract;
use axum::extract::ConnectInfo;
use axum::http::StatusCode;
use axum::middleware;
use axum::response::{IntoResponse, Json, Response};
use axum::routing::post;
use axum::Router;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Mutex;
use tracing::{error, info, warn};

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
    size: Option<u64>,
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
async fn upload(extract::Json(payload): extract::Json<FileMeta>) -> Response {
    info!("upload file with name [{}]", &payload.name);
    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}

#[axum::debug_handler]
async fn remove(extract::Json(payload): extract::Json<FileMeta>) -> Response {
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
                self::delete_remote_chunk(chunk_id, worker.ip);
            }
        }
    }

    if let Ok(mut memory) = METASTATE.lock() {
        memory.retain(|x| x.file_name != payload.name)
    }

    self::append("prune", &kill_hash);

    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}

fn delete_remote_chunk(chunk_id: String, remote_ip: String) {
    // TOOD
    info!("todo")
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
