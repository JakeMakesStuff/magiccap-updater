extern crate sysinfo;
extern crate reqwest;

use sysinfo::SystemExt;
use sysinfo::ProcessExt;
use std::io::prelude::*;
use std::io;

fn do_fail() -> reqwest::Response {
    print!(" fail!\n");
    std::process::exit(1);
}

#[cfg(target_os = "linux")]
fn is_linux() -> bool {
    return true;
}

#[cfg(target_os = "macos")]
fn is_linux() -> bool {
    return false;
}

fn do_request(url: String) -> reqwest::Response {
    match reqwest::get(&url[..]) {
        Err(_) => do_fail(),
        Ok(value) => return value,
    }
}

fn do_copy(buf: &mut std::fs::File, mut response: reqwest::Response) {
    match response.copy_to(buf) {
        Err(_) => do_fail(),
        Ok(_) => return,
    };
}

fn main() {
    println!("MagicCap update tool. Copyright (C) Jake Gealer 2018.");

    print!("Getting tag argument...");

    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        do_fail();
    }
    print!(" done!\n");
    let _arg = &args[1];

    print!("Finding running MagicCap copy...");
    io::stdout().flush().unwrap();

    let system = sysinfo::System::new();

    let mut found = false;

    for (_, magiccap_process) in sysinfo::SystemExt::get_process_list(&system) {
        if magiccap_process.name() == "MagicCap" {
            print!(" done!\n");

            print!("Getting MagicCap binary...");
            io::stdout().flush().unwrap();

            let os;
            if is_linux() {
                os = "linux";
            } else {
                os = "mac";
            };

            let url = "https://s3.magiccap.me/upgrades/".to_string() +  _arg + "/magiccap-" + os + ".zip";
            let response = do_request(url);

            if !response.status().is_success() {
                do_fail();
            }
            print!(" done!\n");

            print!("Copying to disk...");
            io::stdout().flush().unwrap();
            let mut temp_path = std::env::temp_dir();
            temp_path.push("magiccap-upgrade.zip");
            let mut file = std::fs::File::create(&temp_path).unwrap();
            do_copy(&mut file, response);
            print!(" done!\n");

            let mut magiccap_executable_path = magiccap_process.exe();
            if !is_linux() {
                magiccap_executable_path = magiccap_executable_path.parent().unwrap().parent().unwrap().parent().unwrap();
            }

            print!("Closing MagicCap...");
            io::stdout().flush().unwrap();
            magiccap_process.kill(sysinfo::Signal::Quit);
            print!(" done!\n");

            if is_linux() {
                let magiccap_root_folder = magiccap_executable_path.parent().unwrap();

                print!("Deleting current MagicCap...");
                io::stdout().flush().unwrap();
                std::fs::remove_dir_all(&magiccap_root_folder).unwrap();
                std::fs::create_dir(&magiccap_root_folder).unwrap();
                print!(" done!\n");

                print!("Extracting the new MagicCap release...");
                io::stdout().flush().unwrap();
                std::process::Command::new("unzip")
                    .args(&["-q", *&temp_path.to_str().unwrap(), "-d", *&magiccap_root_folder.to_str().unwrap()])
                    .output()
                    .unwrap();
                print!(" done!\n");

                print!("Starting MagicCap...");
                io::stdout().flush().unwrap();
                std::process::Command::new(*&magiccap_executable_path.to_str().unwrap())
                    .spawn()
                    .unwrap();
                print!(" done!\n");
            } else {
                print!("Deleting current MagicCap...");
                io::stdout().flush().unwrap();
                std::fs::remove_dir_all(&magiccap_executable_path).unwrap();
                print!(" done!\n");

                print!("Extracting the new MagicCap release...");
                io::stdout().flush().unwrap();
                let applications_folder = magiccap_executable_path.parent().unwrap();
                std::process::Command::new("unzip")
                    .args(&["-q", *&temp_path.to_str().unwrap(), "MagicCap.app/*", "-d", *&applications_folder.to_str().unwrap()])
                    .output()
                    .unwrap();
                print!(" done!\n");

                print!("Creating Gatekeeper exception...");
                io::stdout().flush().unwrap();
                std::process::Command::new("spctl")
                    .args(&["--add", *&magiccap_executable_path.to_str().unwrap()])
                    .output()
                    .unwrap();
                print!(" done!\n");

                print!("Starting MagicCap...");
                io::stdout().flush().unwrap();
                std::process::Command::new("open")
                    .args(&[*&magiccap_executable_path.to_str().unwrap()])
                    .output()
                    .unwrap();
                print!(" done!\n");
            }

            print!("Garbage collecting...");
            io::stdout().flush().unwrap();
            std::fs::remove_file(temp_path).unwrap();
            print!(" done!\n");

            found = true;
            break;
        }
    }

    if !found {
        do_fail();
    }
}
