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
                            println!("{}", response);
                        }
                        print!("> ");
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

    let mut rl = DefaultEditor::new()?;

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
                    "quit" | "exit" => break,
                    "help" => {
                        println!("");
                        println!("quit/exit - close CLI");
                        println!("version - get package version from knotd");
                        println!("status - test knotd status");
                        println!("peerid - get peerid from this device");
                        println!("protocol - get protocol version of socket (knotd)");
                        println!("commands - get commands list from knotd");
                        println!("connect [multiaddr] - try connect to multiaddr");
                    },
                    "version" => { knot.send_json(KnotCommand::Version).await.expect("Version command failed"); },
                    "status" => { knot.send_json(KnotCommand::Status).await.expect("Status command failed"); },
                    "peerid" => { knot.send_json(KnotCommand::GetPeerId).await.expect("Get PeerId failed"); },
                    "protocol" => { knot.send_json(KnotCommand::Protocol).await.expect("Protocol command failed"); },
                    "commands" => { knot.send_json(KnotCommand::GetCommands).await.expect("getCommands command failed"); },
                    "connect" => { knot.send_json(KnotCommand::Connect{ multiaddr: args[0].to_string() }).await.expect("Connect command failed"); },
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
