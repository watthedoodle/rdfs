use tracing::info;

pub fn list(path: &Option<String>) {
    info!(
        "todo: list all remote objects based on optional path '{:?}'...",
        path
    )
}

pub fn get(file: &String) {
    info!("todo: get remote objects based on file name '{}'...", file);
}

pub fn add(file: &String) {
    info!("todo: add remote objects based on file name '{}'...", file);
}

pub fn remove(file: &String) {
    info!(
        "todo: remove remote object based on file name '{}'...",
        file
    );
}
