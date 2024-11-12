use std::process::{Command};
use std::env;
use tonic::{transport::Server, Request, Response, Status};
use remote_shell::{remote_command_server::{RemoteCommand, RemoteCommandServer}, CommandRequest, CommandReply};
use std::sync::{Arc, Mutex};
use std::fs;

mod remote_shell {
    tonic::include_proto!("remote_shell"); // Include the generated gRPC code from the .proto file
}

const PORT: &str = ":12021";

// Server struct to hold current directory info.
#[derive(Default)]
pub struct MyServer {
    current_dir: Arc<Mutex<String>>,  // To hold the current directory in a thread-safe manner
}

impl MyServer {
    fn execute_command(&self, command_name: &str, command_args: Vec<String>) -> String {
        let mut command = Command::new("cmd.exe");

        let current_dir = self.current_dir.lock().unwrap();
        
        // Set the current working directory for commands
        command.current_dir(&*current_dir);

        // Add the base command and its arguments
        command.arg("/C").arg(command_name); // `/C` ensures the command is executed and the shell exits

        for arg in command_args {
            command.arg(arg);
        }

        // Execute the command and capture the output
        let output = command.output();

        match output {
            Ok(output) => {
                if !output.stdout.is_empty() {
                    String::from_utf8_lossy(&output.stdout).to_string()
                } else {
                    String::from_utf8_lossy(&output.stderr).to_string()
                }
            }
            Err(e) => e.to_string(),
        }
    }

    fn handle_cd(&self, new_dir: &str) {
        // Change the current working directory
        let new_dir = new_dir.to_string();
        if std::path::Path::new(&new_dir).exists() {
            let mut current_dir = self.current_dir.lock().unwrap();
            *current_dir = new_dir;
        }
    }

    fn handle_mkdir(&self, new_dir: &str) -> String {
        // Create a new directory
        let new_dir = new_dir.to_string();
        let result = fs::create_dir_all(&new_dir);

        match result {
            Ok(_) => format!("Directory '{}' created successfully.", new_dir),
            Err(e) => format!("Error creating directory '{}': {}", new_dir, e),
        }
    }
}

#[tonic::async_trait]
impl RemoteCommand for MyServer {
    async fn send_command(
        &self,
        request: Request<CommandRequest>,
    ) -> Result<Response<CommandReply>, Status> {
        let req = request.into_inner();
        let cmd_name = req.cmd_name;
        let cmd_args = req.cmd_args;

        // Handle the `cd` command (change the current working directory)
        if cmd_name == "cd" {
            if let Some(new_dir) = cmd_args.get(0) {
                self.handle_cd(new_dir);
                return Ok(Response::new(CommandReply {
                    output: format!("Changed directory to: {}", new_dir),
                }));
            }
        }

        // Handle the `mkdir` command (create a directory)
        if cmd_name == "mkdir" {
            if let Some(new_dir) = cmd_args.get(0) {
                let result = self.handle_mkdir(new_dir);
                return Ok(Response::new(CommandReply { output: result }));
            }
        }

        // For all other commands, execute them as they are.
        let output = self.execute_command(&cmd_name, cmd_args);

        Ok(Response::new(CommandReply {
            output,
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("127.0.0.1{}", PORT).parse()?;
    let server = MyServer {
        current_dir: Arc::new(Mutex::new("C:/".to_string())), // Default to C: drive
    };

    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(RemoteCommandServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
