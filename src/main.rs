const LOGO: &'static str = r#"

██████  ██████  ███████ ███████
██   ██ ██   ██ ██      ██
██████  ██   ██ █████   ███████
██   ██ ██   ██ ██           ██
██   ██ ██████  ██      ███████

 a toy distributed file system
"#;

fn main() {
    /* ---------------------------------------------------------------------------------------------
    ideally we would like to make this single binary polymorphic, so we need to check the ENV and
    flags/arguments in order to decide which "mode" to run

    1. Default mode couls he client CLI
    2. If the --join-url is passed as an argument flag we should run as a "worker" node
    3. If the --master is passed as an argument flag we should run as a "master" node
    --------------------------------------------------------------------------------------------- */
    println!("{}", LOGO);
}
