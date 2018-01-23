extern crate bincode;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate libc;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::slice;
use libc::c_char;
use regex::Regex;
use failure::Error;
use std::ffi::CStr;
use failure::ResultExt;
use std::os::unix::net::UnixStream;
use std::path::Path;

#[derive(Debug, Fail)]
enum InterceptError {
    #[fail(display = "Invalid argument")] InvalidArgument,
}

unsafe fn to_str_vec<'a>(arr: *const *const c_char) -> Result<Vec<&'a str>, Error> {
    let mut arr_iter = arr;
    let mut arr_len = 0;

    // Count how many elements there are
    while !(*arr_iter).is_null() {
        arr_iter = arr_iter.offset(1);
        arr_len += 1;
    }

    let slice = slice::from_raw_parts(arr, arr_len);

    Ok(slice
        .into_iter()
        .map(|item| CStr::from_ptr(*item).to_str())
        .collect::<Result<Vec<_>, _>>()
        .context(InterceptError::InvalidArgument)?)
}

#[repr(C)]
pub enum HookReturn {
    Success = 0,
    CallReal = 1,
}

#[derive(Debug, Serialize)]
struct Cmdline<'a> {
    prog: &'a str,
    argv: &'a [&'a str],
    dir: &'a Path,
}

fn safe_report_call(prog: &str, argv: &[&str]) -> Result<HookReturn, Error> {
    // We can unwrap here, if the string literal is wrong we'll notice
    // Compiler line stolen from rizsotto/Bear and slightly modified
    // Regex in english...
    // 1. "Anything until a slash, until there are no more slashes"
    // 2. "before the term "gcc" we can have anything as long as it's separated by dashes
    // 3. "we must see the term gcc"
    let re = Regex::new(r"^(.*/)*([^-]*-)*g?(cc|\+\+)(-\d+(\.\d+){0,2})?$").unwrap();

    let matched = re.is_match(argv[0]);
    debug!("Regex returned {} for {}", matched, argv[0]);
    if !matched {
        return Ok(HookReturn::CallReal);
    }
    debug!("Matched: {:?}", argv);

    if argv.iter().any(|item| item == &"-o") {
        info!("Blocking command {:?}", argv);
        let socket_name =
            std::env::var("COMPILE_COMMAND_CHANNEL").context("IPC socket does not exist")?;

        let mut sender = UnixStream::connect(socket_name).context("Failed to open socket")?;

        let current_dir = std::env::current_dir().context("Failed to get cwd")?;

        let cmdline = Cmdline {
            prog: prog,
            argv: argv,
            dir: &current_dir,
        };

        bincode::serialize_into(&mut sender, &cmdline, bincode::Infinite)
            .context("Failed to bincode")?;

        return Ok(HookReturn::Success);
    }

    Ok(HookReturn::CallReal)
}

#[no_mangle]
pub unsafe extern "C" fn report_call_rust(prog: *const c_char, argv: *const *const c_char) -> i32 {
    let _ = pretty_env_logger::try_init();

    let str_vec_res = to_str_vec(argv);
    let prog_res = CStr::from_ptr(prog).to_str();

    let res = if let (Ok(str_vec), Ok(prog)) = (str_vec_res, prog_res) {
        safe_report_call(prog, &str_vec)
    } else {
        Err(failure::err_msg("Invalid arguments"))
    };

    match res {
        Ok(x) => x as i32,
        Err(e) => {
            let mut causes = e.causes();
            println!("{}", causes.next().unwrap());
            for e in causes {
                println!("caused by {}", e);
            }
            HookReturn::CallReal as i32
        }
    }
}
