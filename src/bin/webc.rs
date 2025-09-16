use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "webc", version = "0.1.0", about = "WebCore CLI (prototype)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Build {
        #[arg(short, long, default_value = "examples/hello.webc")]
        input: PathBuf,
        #[arg(short, long, default_value = "dist")]
        out: PathBuf,
    },
    Dev {
        #[arg(short, long, default_value = "examples/hello.webc")]
        input: PathBuf,
        #[arg(short, long, default_value = "dist")]
        out: PathBuf,
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
    Upgrade,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { input, out } => {
            println!("🔨 Building project: {:?} -> {:?}", input, out);
            if let Err(e) = webcore_v2::build_project(input.to_str().unwrap(), out.to_str().unwrap()) {
                eprintln!("Build failed: {}", e);
            }
        }
        Commands::Dev { input, out, port } => {
            println!("🚀 Dev server on port {}", port);
            // Simple dev: run build then serve dist
            if let Err(e) = webcore_v2::build_project(input.to_str().unwrap(), out.to_str().unwrap()) {
                eprintln!("Build failed: {}", e);
                return;
            }
            // serve using tiny-http (blocking simple server)
            use tiny_http::{Server, Response};
            let server = Server::http(format!("0.0.0.0:{}", port)).unwrap();
            let out_dir = out;
            println!("Serving {}", out_dir.display());
            for request in server.incoming_requests() {
                let url = request.url();
                let path = if url == "/" { out_dir.join("index.html") } else { out_dir.join(&url[1..]) };
                if path.exists() {
                    if let Ok(data) = std::fs::read(&path) {
                        let resp = Response::from_data(data);
                        let _ = request.respond(resp);
                    } else {
                        let _ = request.respond(Response::from_string("500"));    
                    }
                } else {
                    let _ = request.respond(Response::from_string("404"));
                }
            }
        }
        Commands::Upgrade => {
            println!("⬆️  Upgrade not implemented in prototype.");
        }
    }
}
