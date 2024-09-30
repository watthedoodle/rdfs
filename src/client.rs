pub fn list(path: &Option<String>) {
    println!("==> todo: list all remote objects based on optional path...");
}

pub fn get(file: &String) {
    println!(
        "==> todo: get remote objects based on file name '{}'...",
        file
    );
}

pub fn add(file: &String) {
    println!(
        "==> todo: add remote objects based on file name '{}'...",
        file
    );
}

pub fn remove(file: &String) {
    println!(
        "==> todo: remove remote object based on file name '{}'...",
        file
    );
}
