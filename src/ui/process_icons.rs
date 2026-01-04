//! Process icon management for menu items
//!
//! Provides icons for different process types (Node.js, Python, Docker, etc.)
//! to help users quickly identify what's running on each port.

use std::collections::HashMap;
use std::sync::OnceLock;

use anyhow::{Result, anyhow};
use png::Decoder;
use tray_icon::menu::Icon;

// Embed all process icons at compile time
static ICON_NODEJS: &[u8] = include_bytes!("../../assets/process-icons/generated/nodejs@2x.png");
static ICON_PYTHON: &[u8] = include_bytes!("../../assets/process-icons/generated/python@2x.png");
static ICON_RUBY: &[u8] = include_bytes!("../../assets/process-icons/generated/ruby@2x.png");
static ICON_GO: &[u8] = include_bytes!("../../assets/process-icons/generated/go@2x.png");
static ICON_RUST: &[u8] = include_bytes!("../../assets/process-icons/generated/rust@2x.png");
static ICON_JAVA: &[u8] = include_bytes!("../../assets/process-icons/generated/java@2x.png");
static ICON_PHP: &[u8] = include_bytes!("../../assets/process-icons/generated/php@2x.png");
static ICON_POSTGRESQL: &[u8] = include_bytes!("../../assets/process-icons/generated/postgresql@2x.png");
static ICON_MYSQL: &[u8] = include_bytes!("../../assets/process-icons/generated/mysql@2x.png");
static ICON_MONGODB: &[u8] = include_bytes!("../../assets/process-icons/generated/mongodb@2x.png");
static ICON_REDIS: &[u8] = include_bytes!("../../assets/process-icons/generated/redis@2x.png");
static ICON_DOCKER: &[u8] = include_bytes!("../../assets/process-icons/generated/docker@2x.png");
static ICON_HOMEBREW: &[u8] = include_bytes!("../../assets/process-icons/generated/homebrew@2x.png");
static ICON_GENERIC: &[u8] = include_bytes!("../../assets/process-icons/generated/generic@2x.png");

/// All supported process icon types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProcessIconType {
    NodeJs,
    Python,
    Ruby,
    Go,
    Rust,
    Java,
    Php,
    PostgreSQL,
    MySQL,
    MongoDB,
    Redis,
    Docker,
    Homebrew,
    Generic,
}

/// Cached decoded icon data
struct CachedIconData {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
}

/// Global icon cache
static ICON_CACHE: OnceLock<HashMap<ProcessIconType, CachedIconData>> = OnceLock::new();

/// Determine icon type from command name with fuzzy matching
pub fn icon_type_from_command(command: &str) -> ProcessIconType {
    let cmd_lower = command.to_lowercase();

    // Node.js variants
    if cmd_lower.contains("node")
        || cmd_lower.contains("npm")
        || cmd_lower.contains("yarn")
        || cmd_lower.contains("pnpm")
        || cmd_lower.contains("bun")
        || cmd_lower.contains("deno")
        || cmd_lower.contains("vite")
        || cmd_lower.contains("next")
        || cmd_lower.contains("nuxt")
        || cmd_lower.contains("esbuild")
        || cmd_lower.contains("webpack")
        || cmd_lower.contains("rollup")
    {
        return ProcessIconType::NodeJs;
    }

    // Python variants
    if cmd_lower.contains("python")
        || cmd_lower.contains("uvicorn")
        || cmd_lower.contains("gunicorn")
        || cmd_lower.contains("flask")
        || cmd_lower.contains("django")
        || cmd_lower.contains("celery")
        || cmd_lower.contains("fastapi")
        || cmd_lower.contains("hypercorn")
    {
        return ProcessIconType::Python;
    }

    // Ruby variants
    if cmd_lower.contains("ruby")
        || cmd_lower.contains("rails")
        || cmd_lower.contains("puma")
        || cmd_lower.contains("unicorn")
        || cmd_lower.contains("sidekiq")
        || cmd_lower.contains("resque")
    {
        return ProcessIconType::Ruby;
    }

    // Go (be careful with short names)
    if cmd_lower == "go" || cmd_lower.starts_with("go ") || cmd_lower.contains("golang") {
        return ProcessIconType::Go;
    }

    // Rust
    if cmd_lower.contains("cargo") || cmd_lower.contains("rustc") {
        return ProcessIconType::Rust;
    }

    // Java variants
    if cmd_lower.contains("java")
        || cmd_lower.contains("gradle")
        || cmd_lower.contains("maven")
        || cmd_lower.contains("kotlin")
        || cmd_lower.contains("spring")
        || cmd_lower.contains("tomcat")
    {
        return ProcessIconType::Java;
    }

    // PHP variants
    if cmd_lower.contains("php")
        || cmd_lower.contains("artisan")
        || cmd_lower.contains("composer")
        || cmd_lower.contains("laravel")
    {
        return ProcessIconType::Php;
    }

    // Databases
    if cmd_lower.contains("postgres") {
        return ProcessIconType::PostgreSQL;
    }
    if cmd_lower.contains("mysql") || cmd_lower.contains("mariadb") {
        return ProcessIconType::MySQL;
    }
    if cmd_lower.contains("mongo") {
        return ProcessIconType::MongoDB;
    }
    if cmd_lower.contains("redis") {
        return ProcessIconType::Redis;
    }

    ProcessIconType::Generic
}

/// Get icon type for Docker containers (always Docker whale)
pub fn icon_type_for_docker() -> ProcessIconType {
    ProcessIconType::Docker
}

/// Get icon type for Homebrew services
/// Maps service names to appropriate icons, falling back to Homebrew icon
pub fn icon_type_for_brew(service_name: &str) -> ProcessIconType {
    let svc_lower = service_name.to_lowercase();

    if svc_lower.contains("postgres") {
        ProcessIconType::PostgreSQL
    } else if svc_lower.contains("mysql") || svc_lower.contains("mariadb") {
        ProcessIconType::MySQL
    } else if svc_lower.contains("mongo") {
        ProcessIconType::MongoDB
    } else if svc_lower.contains("redis") {
        ProcessIconType::Redis
    } else {
        ProcessIconType::Homebrew
    }
}

/// Get a menu Icon for the given ProcessIconType
/// Returns None if icon loading fails (graceful degradation)
pub fn get_process_icon(icon_type: ProcessIconType) -> Option<Icon> {
    let cache = ICON_CACHE.get_or_init(|| {
        let mut map = HashMap::new();

        let icons: [(ProcessIconType, &[u8]); 14] = [
            (ProcessIconType::NodeJs, ICON_NODEJS),
            (ProcessIconType::Python, ICON_PYTHON),
            (ProcessIconType::Ruby, ICON_RUBY),
            (ProcessIconType::Go, ICON_GO),
            (ProcessIconType::Rust, ICON_RUST),
            (ProcessIconType::Java, ICON_JAVA),
            (ProcessIconType::Php, ICON_PHP),
            (ProcessIconType::PostgreSQL, ICON_POSTGRESQL),
            (ProcessIconType::MySQL, ICON_MYSQL),
            (ProcessIconType::MongoDB, ICON_MONGODB),
            (ProcessIconType::Redis, ICON_REDIS),
            (ProcessIconType::Docker, ICON_DOCKER),
            (ProcessIconType::Homebrew, ICON_HOMEBREW),
            (ProcessIconType::Generic, ICON_GENERIC),
        ];

        for (icon_type, data) in icons {
            if let Ok(cached) = decode_png_to_rgba(data) {
                map.insert(icon_type, cached);
            }
        }

        map
    });

    cache.get(&icon_type).and_then(|cached| {
        Icon::from_rgba(cached.rgba.clone(), cached.width, cached.height).ok()
    })
}

/// Decode PNG data to RGBA format
fn decode_png_to_rgba(png_data: &[u8]) -> Result<CachedIconData> {
    let decoder = Decoder::new(png_data);
    let mut reader = decoder
        .read_info()
        .map_err(|e| anyhow!("failed to read PNG header: {e}"))?;

    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|e| anyhow!("failed to decode PNG: {e}"))?;

    let width = info.width;
    let height = info.height;

    // Convert to RGBA if needed
    let rgba = match info.color_type {
        png::ColorType::Rgba => buf[..info.buffer_size()].to_vec(),
        png::ColorType::Rgb => {
            let mut rgba = Vec::with_capacity((width * height * 4) as usize);
            for chunk in buf[..info.buffer_size()].chunks(3) {
                rgba.extend_from_slice(chunk);
                rgba.push(255);
            }
            rgba
        }
        png::ColorType::GrayscaleAlpha => {
            let mut rgba = Vec::with_capacity((width * height * 4) as usize);
            for chunk in buf[..info.buffer_size()].chunks(2) {
                let gray = chunk[0];
                let alpha = chunk[1];
                rgba.extend_from_slice(&[gray, gray, gray, alpha]);
            }
            rgba
        }
        png::ColorType::Grayscale => {
            let mut rgba = Vec::with_capacity((width * height * 4) as usize);
            for &gray in &buf[..info.buffer_size()] {
                rgba.extend_from_slice(&[gray, gray, gray, 255]);
            }
            rgba
        }
        png::ColorType::Indexed => {
            return Err(anyhow!("indexed PNG not supported"));
        }
    };

    Ok(CachedIconData {
        rgba,
        width,
        height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_variants() {
        assert_eq!(icon_type_from_command("node"), ProcessIconType::NodeJs);
        assert_eq!(icon_type_from_command("nodemon"), ProcessIconType::NodeJs);
        assert_eq!(icon_type_from_command("npm"), ProcessIconType::NodeJs);
        assert_eq!(icon_type_from_command("yarn"), ProcessIconType::NodeJs);
        assert_eq!(icon_type_from_command("vite"), ProcessIconType::NodeJs);
        assert_eq!(icon_type_from_command("bun"), ProcessIconType::NodeJs);
    }

    #[test]
    fn test_python_variants() {
        assert_eq!(icon_type_from_command("python"), ProcessIconType::Python);
        assert_eq!(icon_type_from_command("python3"), ProcessIconType::Python);
        assert_eq!(icon_type_from_command("Python3.11"), ProcessIconType::Python);
        assert_eq!(icon_type_from_command("uvicorn"), ProcessIconType::Python);
        assert_eq!(icon_type_from_command("gunicorn"), ProcessIconType::Python);
    }

    #[test]
    fn test_databases() {
        assert_eq!(icon_type_from_command("postgres"), ProcessIconType::PostgreSQL);
        assert_eq!(icon_type_from_command("redis-server"), ProcessIconType::Redis);
        assert_eq!(icon_type_from_command("mongod"), ProcessIconType::MongoDB);
        assert_eq!(icon_type_from_command("mysqld"), ProcessIconType::MySQL);
    }

    #[test]
    fn test_fallback() {
        assert_eq!(icon_type_from_command("unknown-app"), ProcessIconType::Generic);
        assert_eq!(icon_type_from_command("my-custom-server"), ProcessIconType::Generic);
    }

    #[test]
    fn test_brew_service_mapping() {
        assert_eq!(icon_type_for_brew("postgresql"), ProcessIconType::PostgreSQL);
        assert_eq!(icon_type_for_brew("postgresql@14"), ProcessIconType::PostgreSQL);
        assert_eq!(icon_type_for_brew("redis"), ProcessIconType::Redis);
        assert_eq!(icon_type_for_brew("nginx"), ProcessIconType::Homebrew);
    }
}
