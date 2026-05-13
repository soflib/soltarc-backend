use std::fmt;
use std::sync::OnceLock;

// ── Nivel de log ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Level::Debug    => write!(f, "DEBUG"),
            Level::Info     => write!(f, "INFO"),
            Level::Warning  => write!(f, "WARNING"),
            Level::Error    => write!(f, "ERROR"),
            Level::Critical => write!(f, "CRITICAL"),
        }
    }
}

// ── Logger ────────────────────────────────────────────────────────────────────

/// Thin wrapper that emits through the global `tracing` subscriber,
/// so all output lands in the same rotating log file configured in main.
pub struct Logger {
    pub min_level: Level,
    pub user:      String,
}

impl Logger {
    pub fn new(user: impl Into<String>, min_level: Level) -> Self {
        Self { min_level, user: user.into() }
    }

    pub fn log(&self, level: Level, message: impl AsRef<str>) {
        if level < self.min_level {
            return;
        }
        let msg  = message.as_ref();
        let user = &self.user;
        match level {
            Level::Debug    => tracing::debug!(user, "{msg}"),
            Level::Info     => tracing::info!(user,  "{msg}"),
            Level::Warning  => tracing::warn!(user,  "{msg}"),
            Level::Error    => tracing::error!(user, "{msg}"),
            Level::Critical => tracing::error!(user, "[CRITICAL] {msg}"),
        }
    }

    #[allow(dead_code)]
    pub fn debug(&self, msg: impl AsRef<str>) {
        self.log(Level::Debug, msg);
    }

    #[allow(dead_code)]
    pub fn info(&self, msg: impl AsRef<str>) {
        self.log(Level::Info, msg);
    }

    #[allow(dead_code)]
    pub fn warning(&self, msg: impl AsRef<str>) {
        self.log(Level::Warning, msg);
    }

    #[allow(dead_code)]
    pub fn error(&self, msg: impl AsRef<str>) {
        self.log(Level::Error, msg);
    }

    #[allow(dead_code)]
    pub fn critical(&self, msg: impl AsRef<str>) {
        self.log(Level::Critical, msg);
    }
}

// ── Logger global (opcional) ──────────────────────────────────────────────────

static GLOBAL_LOGGER: OnceLock<Logger> = OnceLock::new();

#[allow(dead_code)]
pub fn init_global(user: impl Into<String>, min_level: Level) {
    GLOBAL_LOGGER.get_or_init(|| Logger::new(user, min_level));
}

#[allow(dead_code)]
pub fn global() -> &'static Logger {
    GLOBAL_LOGGER.get().expect("Logger no inicializado — llama init_global() primero")
}
