use std::io::{BufReader, Write};
use std::net::TcpListener;

use clap::Parser;
use hookcatch::format::format_request;
use hookcatch::http::parse_request;

#[derive(Parser)]
#[command(
    name = "hookcatch",
    about = "A local webhook receiver — logs every incoming HTTP request in real time"
)]
struct Cli {
    /// Port to listen on.
    #[arg(short, long, default_value_t = 8787)]
    port: u16,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let listener = TcpListener::bind(("0.0.0.0", cli.port))?;
    println!("hookcatch: listening on http://0.0.0.0:{}", cli.port);

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("hookcatch: connection error: {e}");
                continue;
            }
        };
        let mut reader = BufReader::new(stream.try_clone()?);
        let req = match parse_request(&mut reader) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("hookcatch: could not parse request: {e}");
                continue;
            }
        };
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        println!("{}", format_request(&req, &timestamp));

        let response = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok";
        let _ = stream.write_all(response);
    }
    Ok(())
}
