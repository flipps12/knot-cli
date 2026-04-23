use knot_sdk::{ KnotClient, KnotCommand };
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Knot-CLI v0.1.0");

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
                        eprintln!("Error: {:?}", msg.error);
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

                if input.is_empty() {
                    continue;
                }

                match input {
                    "quit" | "exit" => break,
                    "status" => knot.send_json(KnotCommand::Status).await.expect("Status command failed"),
                    "peerid" => knot.send_json(KnotCommand::GetPeerId).await.expect("Get PeerId failed"),
                    "protocol" => knot.send_json(KnotCommand::Protocol).await.expect("Protocol command failed"),
                    "commands" => knot.send_json(KnotCommand::GetCommands).await.expect("getCommands command failed"),
                    _ => println!("Unknow command"),
                }
            }
            Err(ReadlineError::Interrupted) => {
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
            }
        }
    }

    //println!("Closing");
    Ok(())   
}
