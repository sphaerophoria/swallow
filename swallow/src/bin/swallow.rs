extern crate clangize;
#[macro_use]
extern crate common_failures;
extern crate failure;
#[macro_use]
extern crate lazy_static;
extern crate pretty_env_logger;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate swallow;

use common_failures::prelude::*;
use std::process::Command;
use std::time::Duration;
use std::fs::OpenOptions;
use serde::Serializer;
use serde::ser::SerializeSeq;
use std::fs::File;
use std::path::PathBuf;
use swallow::CommandServer;
use clangize::Clangizer;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    /// compile_commands.json formatted output file
    #[structopt(short = "-o", long = "--output", default_value = "compile_commands.json")]
    output: String,

    /// Clangize the command
    #[structopt(short = "-c", long = "--clangize")]
    clangize: bool,

    /// Command to swallow
    #[structopt(default_value = "make")]
    cmd: Vec<String>,
}

lazy_static! {
    static ref OPTIONS: Opt = Opt::from_args();
}

#[derive(Serialize)]
struct CommandEntry {
    directory: PathBuf,
    command: String,
    file: String,
}

fn start_command_server() -> Result<CommandServer> {
    let mut args_iter = OPTIONS.cmd.iter();

    let cmd = args_iter
        .next()
        .ok_or_else(|| failure::err_msg("No command given"))?;

    let mut command = Command::new(cmd);
    command.args(args_iter);

    Ok(CommandServer::new(command)?)
}

fn open_output_file() -> Result<File> {
    Ok(OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&OPTIONS.output)
        .context("Failed to create compile_commands.json")?)
}

fn try_main() -> Result<()> {
    pretty_env_logger::init();
    let output_file = open_output_file()?;

    let mut serializer = serde_json::Serializer::new(output_file);
    let mut seq = serializer
        .serialize_seq(None)
        .context("Failed to create streaming serializer")?;

    let mut clangizer = Clangizer::new();

    let mut command_server = start_command_server()?;

    while command_server.is_running() {
        let recv_timeout = Duration::new(0, 50_000_000);
        while let Ok(command) = command_server.recv_timeout(recv_timeout) {
            let cmdline = if OPTIONS.clangize {
                clangizer.clangize(command.command).join(" ")
            } else {
                command.command.join(" ")
            };

            let entry = CommandEntry {
                directory: command.directory,
                command: cmdline,
                file: command.file,
            };

            seq.serialize_element(&entry)
                .context("Failed to serialize element")?;
        }
    }

    seq.end()?;

    Ok(())
}

quick_main!(try_main);
