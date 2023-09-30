use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;
use clap::Parser;
use log::{error, info};

/// Simple program to query Mellanox ConnectX temperature and write it to file
#[derive(Parser, Debug)]
struct Args {
    /// Device path for mget_temp (e.g.: mt4099_pci_cr0)
    #[arg(short, long, default_value_t = String::from("mt4103_pci_cr0"))]
    device: String,

    /// Path to mget_temp
    #[arg(short, long, default_value_t = String::from("mget_temp"))]
    exec_path: String,

    /// Path to output file
    #[arg(short, long, required = false)]
    out_path: String,

    /// Update interval (in seconds)
    #[arg(short, long, default_value_t = 5)]
    interval: u64
}

fn main() {
    if !cfg!(windows) {
        panic!("OS not supported!");
    }

    let args = Args::parse();

    let exec_path = args.exec_path;
    let out_path = args.out_path;
    let device = args.device;
    let interval = args.interval;

    info!("Exec Path: {}", exec_path);
    info!("Out Path: {}", out_path);
    info!("Device: {}", device);
    info!("Update interval: {}s", interval);

    loop {
        let output = match Command::new(&exec_path)
            .args(["-d", &device])
            .output() {
            Ok(out) => out,
            Err(err) => panic!(r#"[Error]: Error getting temp using mget_temp.
            Please check your program path and parameters.
            Additional Info:
            Command: {} -d {}
            Err: {:?}"#, &exec_path, &device, err.to_string())
        };

        if output.stderr.len() > 0 {
            error!("[ERROR]: {}", String::from_utf8(output.stderr).unwrap_or(String::from("")).trim());
        }

        let output_text = match String::from_utf8(output.stdout) {
            Ok(text) => text,
            Err(err) => {
                error!("[ERROR]: Error converting temperature to string: {:?}", err);
                String::from("-10000")
            }
        };
        let temp = output_text.trim();
        info!("{}°C", &temp);

        let temp_i32 = temp.parse::<i32>().unwrap_or(0);
        match temp_i32 {
            -40..=150 => {
                let mut file = File::create(&out_path).unwrap();
                file.write_all(temp.as_ref()).unwrap();
            },
            _ => error!("[ERROR]: Invalid temp ({})!", temp_i32)
        }

        sleep(Duration::from_secs(interval));
    }
}
