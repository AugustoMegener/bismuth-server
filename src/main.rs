use bismuth_server::db::init_repo_database;
use clap::Parser;
use std::error::Error;
use std::path::PathBuf;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    path: PathBuf
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let path_buf = args.path;
    let path = path_buf.as_path();
    let dir_path_str = path.to_str().ok_or_else(|| "No dir path value!")?;

    if !path.is_dir() { 
        return Err(format!("{dir_path_str} is not a directory!").into());
    }

    init_repo_database(dir_path_str).await?;

    Ok(())
}
