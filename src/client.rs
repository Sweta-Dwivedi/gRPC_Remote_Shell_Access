use tonic::transport::Channel;
use tonic::Request;
use std::io::{self, Write};
use remote_shell::remote_command_client::RemoteCommandClient;
use remote_shell::CommandRequest;

pub mod remote_shell {
    tonic::include_proto!("remote_shell"); // Path to your proto file
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the server address
    let server_address = "http://127.0.0.1:12021";
    let channel = Channel::from_shared(server_address.to_string())?.connect().await?;
    let mut client = RemoteCommandClient::new(channel);

    println!("Connected to server at {}", server_address);

    loop {
        // Read command from user input
        print!("$ ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let cmd = input.trim();

        // Exit condition
        if cmd == "exit" {
            println!("Disconnecting from server...");
            break;
        }

        // Split the command into name and arguments
        let mut cmd_parts = cmd.split_whitespace();
        let cmd_name = cmd_parts.next().unwrap_or("");
        let cmd_args: Vec<String> = cmd_parts.map(|s| s.to_string()).collect();

        // Send the command request to the server
        let request = Request::new(CommandRequest {
            cmd_name: cmd_name.to_string(),
            cmd_args,
        });

        match client.send_command(request).await {
            Ok(response) => {
                println!("Server response: {}", response.into_inner().output);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    Ok(())
}
