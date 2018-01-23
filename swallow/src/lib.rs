extern crate bincode;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::sync::mpsc::{self, Receiver, Sender};
use std::process::{Child, Command};
use std::{fs, thread};
use std::os::unix::net::UnixListener;
use std::ops::{Deref, DerefMut};
use std::io::Error as IoError;
use std::path::PathBuf;

/// Owned variant of Cmdline in generate_compile_commands.
/// This works under the assumption that the impl of Serialize/Deserialize is mirrored for owned
/// and borrowed variants.
#[derive(Debug, Deserialize)]
pub struct Cmdline {
    pub prog: String,
    pub argv: Vec<String>,
    pub dir: PathBuf,
}

/// Ensure this matches the compile_commands.json spec
#[derive(Debug, Serialize)]
pub struct CompilationCommand {
    pub directory: PathBuf,
    pub command: Vec<String>,
    pub file: String,
}

pub struct CommandServer {
    child: Child,
    receiver: Receiver<CompilationCommand>,
}

const SOCKET_PATH: &str = "/tmp/clangizer.sock";

fn start_ipc_server() -> Result<UnixListener, IoError> {
    // Don't care if this fails, if it exists we'll fail in the next step anyways
    let _ = fs::remove_file(SOCKET_PATH);

    let ipc_server = UnixListener::bind(SOCKET_PATH)?;

    Ok(ipc_server)
}

fn filename_from_args<S: AsRef<str>>(args: &[S]) -> Option<String>
where
    S: AsRef<str> + Clone + Into<String>,
{
    lazy_static! {
        static ref RE: regex::RegexSet = regex::RegexSetBuilder::new(&[
            r"\.cpp$", r"\.c$",
        ])
        .case_insensitive(true)
        .build().unwrap();
    }

    args.iter()
        .find(|item| RE.is_match(item.as_ref()))
        .map(|item| item.as_ref().into())
}

fn forward_ipc_calls(tx: Sender<CompilationCommand>) {
    let ipc_server = match start_ipc_server() {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to start ipc server: {}", e);
            return;
        }
    };

    while let Ok((mut stream, _)) = ipc_server.accept() {
        while let Ok(mut item) =
            bincode::deserialize_from::<_, Cmdline, _>(&mut stream, bincode::Infinite)
        {
            let file = match filename_from_args(&item.argv) {
                Some(f) => f,
                None => continue,
            };

            item.argv[0] = item.prog;

            let command = CompilationCommand {
                directory: item.dir,
                command: item.argv,
                file: file,
            };

            if tx.send(command).is_err() {
                warn!("Failed to send item");
            }
        }
    }
}

impl CommandServer {
    pub fn new(mut cmd: Command) -> Result<CommandServer, IoError> {
        let child = cmd.env("COMPILE_COMMAND_CHANNEL", SOCKET_PATH)
            .env("LD_PRELOAD", "libswallow_client.so")
            .spawn()?;

        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            forward_ipc_calls(tx);
        });

        Ok(CommandServer {
            child: child,
            receiver: rx,
        })
    }

    pub fn is_running(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(None) => true,
            _ => false,
        }
    }
}

impl Deref for CommandServer {
    type Target = Receiver<CompilationCommand>;

    fn deref(&self) -> &Self::Target {
        &self.receiver
    }
}

impl DerefMut for CommandServer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.receiver
    }
}
