
#[derive(Debug, Deserialize)]
pub struct Args {
arg_servers: Vec<String>,
flag_config: String,
}

pub const USAGE: &'static str = "
A tokio-based IRC bounder
Usage:
    bounce [--config=<cfg.toml>] [<servers>...]
Options:
    -h, --help              Display this message
    --config=<cfg.toml>     Specify config file to use [default: ./config.toml]
    [servers]...            Specify which servers to connect to (default is all)
";
