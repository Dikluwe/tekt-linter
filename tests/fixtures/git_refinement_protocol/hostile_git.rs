use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Write};

const OID40: &str = "1111111111111111111111111111111111111111";
const OID64: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const BLOB_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const BLOB_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn marker(name: &str) -> bool {
    fs::metadata(format!("scenario_{name}")).is_ok()
}

fn value(name: &str) -> String {
    env::var_os(name)
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn present(name: &str) -> &'static str {
    if env::var_os(name).is_some() {
        "x"
    } else {
        ""
    }
}

fn main() {
    let args = env::args_os()
        .skip(1)
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    let mut log = OpenOptions::new()
        .create(true)
        .append(true)
        .open("protocol.log")
        .unwrap();
    writeln!(log, "BEGIN").unwrap();
    write!(log, "ARGV").unwrap();
    for arg in &args {
        write!(log, " <{arg}>").unwrap();
    }
    writeln!(log).unwrap();
    writeln!(
        log,
        "ENV GIT_TERMINAL_PROMPT=<{}> GIT_NO_LAZY_FETCH=<{}> GIT_OPTIONAL_LOCKS=<{}>",
        value("GIT_TERMINAL_PROMPT"),
        value("GIT_NO_LAZY_FETCH"),
        value("GIT_OPTIONAL_LOCKS")
    )
    .unwrap();
    writeln!(
        log,
        "ENV GIT_NO_REPLACE_OBJECTS=<{}> GIT_CONFIG_NOSYSTEM=<{}> GIT_CONFIG_GLOBAL=<{}> LC_ALL=<{}>",
        value("GIT_NO_REPLACE_OBJECTS"),
        value("GIT_CONFIG_NOSYSTEM"),
        value("GIT_CONFIG_GLOBAL"),
        value("LC_ALL")
    )
    .unwrap();
    writeln!(
        log,
        "ABSENT PATH=<{}> HOME=<{}> XDG_CONFIG_HOME=<{}> GIT_DIR=<{}>",
        present("PATH"),
        present("HOME"),
        present("XDG_CONFIG_HOME"),
        present("GIT_DIR")
    )
    .unwrap();

    let command = args
        .iter()
        .find(|arg| matches!(arg.as_str(), "rev-parse" | "ls-tree" | "cat-file"))
        .map(String::as_str)
        .unwrap_or("");
    let stdout = io::stdout();
    let mut out = stdout.lock();
    match command {
        "rev-parse" => {
            if marker("missing_ref") {
                std::process::exit(1);
            }
            writeln!(out, "{}", if marker("oid64") { OID64 } else { OID40 }).unwrap();
        }
        "ls-tree" => {
            if marker("types") {
                write!(out, "100644 blob {BLOB_A}\ta.rs\0").unwrap();
                write!(out, "160000 commit {BLOB_B}\tgitlink\0").unwrap();
                write!(out, "120000 blob {BLOB_B}\tlink\0").unwrap();
            } else if marker("budget") {
                write!(out, "100644 blob {BLOB_A}\tlarge.bin\0").unwrap();
            } else if marker("bad_framing") {
                write!(out, "100644 blob {BLOB_A}\ta.rs").unwrap();
            } else {
                write!(out, "100755 blob {BLOB_A}\t-odd.rs\0").unwrap();
            }
        }
        "cat-file" => {
            for line in io::stdin().lock().lines() {
                let line = line.unwrap();
                writeln!(log, "STDIN <{line}>").unwrap();
                if line == format!("contents {BLOB_A}") {
                    if marker("budget") {
                        writeln!(out, "{BLOB_A} blob 4194305").unwrap();
                    } else {
                        write!(out, "{BLOB_A} blob 3\nabc\n").unwrap();
                    }
                } else if line != "flush" {
                    std::process::exit(7);
                }
            }
        }
        _ => std::process::exit(9),
    }
}
