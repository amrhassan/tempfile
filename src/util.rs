use crate::error::IoResultExt;
use camino::{Utf8Path, Utf8PathBuf};
use std::env;
use std::{io, iter::repeat_with};

fn tmpname(prefix: &str, suffix: &str, rand_len: usize) -> String {
    let mut buf = String::with_capacity(prefix.len() + suffix.len() + rand_len);
    buf.extend(prefix.chars());
    buf.extend(repeat_with(fastrand::alphanumeric).take(rand_len));
    buf.extend(suffix.chars());
    buf
}

pub fn create_helper<F, R>(
    base: &Utf8Path,
    prefix: &str,
    suffix: &str,
    random_len: usize,
    mut f: F,
) -> io::Result<R>
where
    F: FnMut(Utf8PathBuf) -> io::Result<R>,
{
    let num_retries = if random_len != 0 {
        crate::NUM_RETRIES
    } else {
        1
    };

    for _ in 0..num_retries {
        let path = base.join(tmpname(prefix, suffix, random_len));
        return match f(path) {
            Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists && num_retries > 1 => continue,
            // AddrInUse can happen if we're creating a UNIX domain socket and
            // the path already exists.
            Err(ref e) if e.kind() == io::ErrorKind::AddrInUse && num_retries > 1 => continue,
            res => res,
        };
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "too many temporary files exist",
    ))
    .with_err_path(|| base)
}

pub fn temp_dir() -> io::Result<Utf8PathBuf> {
    Utf8PathBuf::from_path_buf(env::temp_dir())
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "OS temp_dir() is not UTF8"))
}

pub fn current_dir() -> io::Result<Utf8PathBuf> {
    Utf8PathBuf::from_path_buf(env::current_dir()?)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "OS current_dir() is not UTF8"))
}
