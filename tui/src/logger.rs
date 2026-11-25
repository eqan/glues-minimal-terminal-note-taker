use std::{
    fmt::{Debug, Display},
    fs::{self, File, OpenOptions},
    io::Write,
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

static LOG_FILE: OnceLock<Mutex<File>> = OnceLock::new();

pub async fn init() {
    let directory = crate::config::config_dir();
    if fs::create_dir_all(&directory).is_err() {
        return;
    }

    if let Ok(file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(directory.join("glues.log"))
    {
        let _ = LOG_FILE.set(Mutex::new(file));
    }
}

pub async fn log(message: &str) {
    let Some(file) = LOG_FILE.get() else {
        return;
    };
    let Ok(mut file) = file.lock() else {
        return;
    };
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());
    let _ = writeln!(file, "{timestamp} {message}");
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::logger::log(&format!($($arg)*)).await;
    };
}

pub trait LogExpectExt<V> {
    fn log_expect(self, message: &str) -> V;
}

impl<V> LogExpectExt<V> for Option<V> {
    fn log_expect(self, message: &str) -> V {
        if let Some(value) = self {
            value
        } else {
            panic!("{message}");
        }
    }
}

#[allow(dead_code)]
pub trait LogUnwrapExt<V> {
    fn log_unwrap(self) -> V;
}

impl<V, E> LogUnwrapExt<V> for Result<V, E>
where
    E: Debug + Display,
{
    fn log_unwrap(self) -> V {
        match self {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        }
    }
}
