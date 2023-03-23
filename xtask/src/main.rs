use std::{env, error::Error, process::Command, thread, time::Duration};

use crate::utils::{format_arg, project_root};

mod utils;

#[cfg(target_os = "windows")]
use windows::Win32::System::Console::{GenerateConsoleCtrlEvent, CTRL_CLOSE_EVENT};

pub static PROFILE: &str = env!("PROFILE");
pub static OUT_DIR: &str = env!("OUT_DIR");
pub static ASSETS_DIR: &str = env!("ASSETS_DIR");

fn run(num_clients: usize) -> Result<(), Box<dyn Error>> {
    ctrlc::set_handler({
        move || {
            #[cfg(target_os = "windows")]
            unsafe {
                GenerateConsoleCtrlEvent(CTRL_CLOSE_EVENT, 0);
            }
        }
    })
    .unwrap();

    let mut server = {
        #[cfg(target_os = "windows")]
        let server_name = "server.exe";

        #[cfg(target_os = "linux")]
        let server_name = "./server";
        Command::new(server_name).current_dir(OUT_DIR).spawn()?
    };

    let mut clients = Vec::new();
    for _ in 0..num_clients {
        std::thread::sleep(Duration::from_millis(100));

        let client = {
            #[cfg(target_os = "windows")]
            let client_name = "client.exe";

            #[cfg(target_os = "linux")]
            let client_name = "./client";
            Command::new(client_name).current_dir(OUT_DIR).spawn()?
        };

        clients.push(client);
    }

    loop {
        thread::sleep(Duration::from_millis(100));

        let mut all_done = true;

        match server.try_wait() {
            Ok(Some(_)) => {}
            Ok(None) => {
                all_done = false;
            }
            Err(_) => {}
        }

        for client in &mut clients {
            match client.try_wait() {
                Ok(Some(_)) => {}
                Ok(None) => {
                    all_done = false;
                }
                Err(_) => {}
            }
        }

        if all_done {
            break;
        }
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("kill")
            .arg(server.id().to_string())
            .status()
            .ok();
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("kill")
            .arg(client.id().to_string())
            .status()
            .ok();
    }

    Ok(())
}

fn build() -> Result<(), Box<dyn Error>> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());

    println!("Building Blitz Server and Client...");

    let status = Command::new(cargo)
        .current_dir(project_root())
        .args(["build"])
        .arg(format_arg())
        .status()?;

    if !status.success() {
        Err("cargo build failed")?;
    }

    println!("Building Blitz Server and Client... done");

    Ok(())
}

fn run_server_client(n: usize) -> Result<(), Box<dyn Error>> {
    build()?;
    run(n)?;

    Ok(())
}

fn try_main() -> Result<(), Box<dyn Error>> {
    let task = env::args().nth(1);
    match task.as_deref() {
        Some("blitz") => run_server_client(0)?,
        Some("blitz_x") => {
            let n = env::args().nth(2).unwrap().parse::<usize>().unwrap();

            run_server_client(n)?;
        }
        _ => print_help(),
    }
    Ok(())
}

fn print_help() {
    eprintln!(
        "Tasks:
blitz               builds and runs Blitz with a Server and Client
blitz X             builds and runs Blitz with a Server and X number of Client

"
    )
}

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{e}");
        std::process::exit(-1);
    }
}
