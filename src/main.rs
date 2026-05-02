use std::time::{ SystemTime, UNIX_EPOCH };
use colored::*;
use knot_sdk::{ KnotClient, KnotCommand };
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let version = env!("CARGO_PKG_VERSION");
    println!("version {}", version);

    println!("██ ▄█▀ ▄▄  ▄▄  ▄▄▄ ▄▄▄▄▄▄    ▄█████ ██     ██");
    println!("████   ███▄██ ██▀██  ██  ▄▄▄ ██     ██     ██");
    println!("██ ▀█▄ ██ ▀██ ▀███▀  ██      ▀█████ ██████ ██ \n");

    // connect
    let knot = match KnotClient::new(7564).await {
        Ok(k) => k,
        Err(e) => {
            eprintln!("Failet to connect KnotClient: {e}");
            std::process::exit(1);
        }
    };

    // listeners
    let mut msg_rx = knot.subscribe_messages();
    tokio::spawn(async move {
        loop {
            match msg_rx.recv().await {
                Ok(msg) => {
                    if msg.response != Some("".into()) {
                        if let Some(response) = msg.response {
                            println!("\r{}", response);
                            //print!("{}", "> ".bold());
                        }
                    } else {
                        eprintln!("Error response: {:?}", msg.error);
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    eprintln!("[Knot] message channel laggedd, skipped {n} messages");
                }
                Err(_) => {
                    break;
                }
            }
        }
    });

    let mut byte_rx = knot.subscribe_bytes();
    tokio::spawn(async move {
        loop {
            match byte_rx.recv().await {
                Ok(msg) => {
                    println!("Peer: {msg}");

                    let sent = timing::parse_timestamp(&msg).unwrap();

                    println!("RTT ms: {}", timing::diff_ms(sent));
                    println!("RTT us: {}", timing::diff_us(sent));
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    eprintln!("[Knot] byte channel lagged, skipped {n} messages");
                }
                Err(_) => {
                    break;
                }
            }
        }
    });

    let _ = knot.send_json(KnotCommand::Register { app_id: 1, port: 7564 }).await;

    let mut rl = DefaultEditor::new()?;
    // for spacing
    let width = 20;

    loop {
        let readline = rl.readline("> ");

        match readline {
            Ok(line) => {
                let input = line.trim();
                let mut input_splited = input.split_whitespace();
                let command = input_splited.next().unwrap_or("");
                let args: Vec<&str> = input_splited.collect();

                // println!("{:?}", args);

                if input.is_empty() {
                    continue;
                }

                match command {
                    "quit" | "exit" => {
                        break;
                    }
                    "help" => {
                        println!(
                            "{} {} {}",
                            ">".bold(),
                            "command".bold().blue(),
                            "[arg1] [arg2]".red().bold()
                        );
                        println!(
                            "  {:<width$} - close CLI",
                            "quit/exit".green().bold(),
                            width = width
                        );
                        println!(
                            "  {:<width$} - test time of frame sender/receiver",
                            "ping [peerid]".green().bold(),
                            width = width
                        );
                        println!(
                            "  {:<width$} - get package version from knotd",
                            "version".green().bold(),
                            width = width
                        );
                        println!(
                            "  {:<width$} - test knotd status",
                            "status".green().bold(),
                            width = width
                        );
                        println!(
                            "  {:<width$} - get peerid from this device",
                            "peerid".green().bold(),
                            width = width
                        );
                        println!(
                            "  {:<width$} - get protocol version of socket (knotd)",
                            "protocol".green().bold(),
                            width = width
                        );
                        println!(
                            "  {:<width$} - get commands list from knotd",
                            "commands".green().bold(),
                            width = width
                        );
                        println!(
                            "  {:<width$} - try connect to multiaddr",
                            "connect [multiaddr]".green().bold(),
                            width = width
                        );
                        println!(
                            "  {:<width$} - listen on relay server",
                            "relay [multiaddr(without peerid)] [peerid]".green().bold(),
                            width = width
                        );
                        println!(
                            "  {:<width$} - get listeners from knotd",
                            "listeners".green().bold(),
                            width = width
                        );
                    }
                    "version" => {
                        knot.send_json(KnotCommand::Version).await.expect("Version command failed");
                    }
                    "status" => {
                        knot.send_json(KnotCommand::Status).await.expect("Status command failed");
                    }
                    "listeners" => {
                        knot.send_json(KnotCommand::Listeners).await.expect("Listeners command failed");
                    }
                    "peerid" => {
                        knot.send_json(KnotCommand::GetPeerId).await.expect("Get PeerId failed");
                    }
                    "protocol" => {
                        knot.send_json(KnotCommand::Protocol).await.expect(
                            "Protocol command failed"
                        );
                    }
                    "commands" => {
                        knot.send_json(KnotCommand::GetCommands).await.expect(
                            "getCommands command failed"
                        );
                    }
                    "connect" => {
                        knot.send_json(KnotCommand::Connect {
                            multiaddr: args[0].to_string(),
                        }).await.expect("Connect command failed");
                    }
                    "relay" => {
                        knot.send_json(KnotCommand::ConnectRelay {
                            relay_addr: args[0].to_string(),
                            relay_id: args[1].to_string(),
                        }).await.expect("Connect command failed");
                    }
                    "ping" => {
                        let now = SystemTime::now();
                        let duration = now.duration_since(UNIX_EPOCH).unwrap();

                        let nanos = duration.as_nanos();
                        let text = nanos.to_string();
                        let _ = knot.send_bytes(args[0], text.as_bytes(), 0).await;
                    }
                    _ => println!("Unknow command"),
                }
            }
            Err(ReadlineError::Interrupted) => {
                break;
            }
            Err(err) => {
                eprintln!("Error loop: {:?}", err);
            }
        }
    }

    //println!("Closing");
    Ok(())
}

pub mod timing {
    use std::time::{ SystemTime, UNIX_EPOCH, Duration };

    /// Convierte texto (u64) → Duration desde UNIX_EPOCH
    pub fn parse_timestamp(text: &str) -> Result<Duration, String> {
        let value: u64 = text
            .trim()
            .parse()
            .map_err(|_| "Invalid timestamp".to_string())?;

        Ok(Duration::from_nanos(value))
    }

    /// Devuelve "ahora" como Duration desde UNIX_EPOCH
    pub fn now() -> Duration {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
    }

    /// Diferencia en milisegundos
    pub fn diff_ms(sent: Duration) -> u128 {
        let now = now();
        now.saturating_sub(sent).as_millis()
    }

    /// Diferencia en microsegundos
    pub fn diff_us(sent: Duration) -> u128 {
        let now = now();
        now.saturating_sub(sent).as_micros()
    }

    /// Diferencia en nanosegundos
    pub fn diff_ns(sent: Duration) -> u128 {
        let now = now();
        now.saturating_sub(sent).as_nanos()
    }
}
