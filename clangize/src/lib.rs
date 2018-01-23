use std::collections::BTreeMap;
use std::process::Command;
use std::path::Path;

type ClangizedOptions = Vec<String>;

fn trimmed_stdout(stdout: Vec<u8>) -> Option<String> {
    let mut stdout_string = String::from_utf8(stdout).ok()?;

    let trimmed_length = stdout_string.trim_right().len();

    if trimmed_length == 0 {
        return None;
    }

    stdout_string.truncate(trimmed_length);
    Some(stdout_string)
}

fn gcc_target_from_prog(prog: &str) -> Option<String> {
    Command::new(prog)
        .arg("-print-multiarch")
        .output()
        .ok()
        .and_then(|output| trimmed_stdout(output.stdout))
}

fn gcc_sysroot_from_prog(prog: &str) -> Option<String> {
    Command::new(prog)
        .arg("-print-sysroot")
        .output()
        .ok()
        .and_then(|output| trimmed_stdout(output.stdout))
}

fn gcc_toolchain_from_prog(prog: &str) -> Option<String> {
    Path::new(prog)
        .parent()?
        .parent()?
        .to_str()
        .map(|s| s.to_string())
}

fn gcc_clangized_options(prog: &str) -> ClangizedOptions {
    let mut ret = ClangizedOptions::new();

    if let Some(sysroot) = gcc_sysroot_from_prog(prog) {
        ret.push(format!("--sysroot={}", sysroot));
    }

    if let Some(target) = gcc_target_from_prog(prog) {
        ret.push("-target".to_string());
        ret.push(target);
    }

    if let Some(toolchain) = gcc_toolchain_from_prog(prog) {
        ret.push(format!("--gcc-toolchain={}", toolchain));
    }

    ret
}

#[derive(Default)]
pub struct Clangizer {
    clangized_map: BTreeMap<String, ClangizedOptions>,
}

impl Clangizer {
    pub fn new() -> Clangizer {
        Clangizer::default()
    }
    pub fn clangize(&mut self, mut argv: Vec<String>) -> Vec<String> {
        let mut clangized_entries = self.clangized_map
            .entry(argv[0].clone())
            .or_insert_with(|| gcc_clangized_options(&argv[0]))
            .clone();

        argv[0] = "clang".to_string();
        argv.append(&mut clangized_entries);
        argv
    }
}
