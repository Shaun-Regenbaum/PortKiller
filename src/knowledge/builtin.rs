use std::time::{SystemTime, UNIX_EPOCH};

use super::types::{
    KnowledgeBase, KnowledgeEntry, KnowledgeSource, ProcessCategory, ProcessFingerprint,
};

/// Populate the knowledge base with builtin entries for common processes
pub fn populate_builtins(kb: &mut KnowledgeBase) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let builtins = vec![
        // Docker/Container tools
        builtin_entry(
            "com.docker.backend",
            "Docker Desktop",
            "Docker container runtime and management",
            ProcessCategory::Infrastructure,
            now,
        ),
        builtin_entry(
            "orbstack",
            "OrbStack",
            "Fast Docker and Linux VM runtime for macOS",
            ProcessCategory::Infrastructure,
            now,
        ),
        builtin_entry(
            "OrbStack Helper",
            "OrbStack Helper",
            "OrbStack background service",
            ProcessCategory::Infrastructure,
            now,
        ),
        // Databases
        builtin_entry(
            "postgres",
            "PostgreSQL Database",
            "PostgreSQL relational database server",
            ProcessCategory::Database,
            now,
        ),
        builtin_entry(
            "mysqld",
            "MySQL Database",
            "MySQL relational database server",
            ProcessCategory::Database,
            now,
        ),
        builtin_entry(
            "mongod",
            "MongoDB",
            "MongoDB NoSQL document database",
            ProcessCategory::Database,
            now,
        ),
        builtin_entry(
            "redis-server",
            "Redis Cache",
            "Redis in-memory data structure store",
            ProcessCategory::Cache,
            now,
        ),
        builtin_entry(
            "memcached",
            "Memcached",
            "Distributed memory object caching system",
            ProcessCategory::Cache,
            now,
        ),
        // Web servers
        builtin_entry(
            "nginx",
            "NGINX",
            "High-performance web server and reverse proxy",
            ProcessCategory::Proxy,
            now,
        ),
        builtin_entry(
            "httpd",
            "Apache HTTP Server",
            "Apache web server",
            ProcessCategory::Proxy,
            now,
        ),
        builtin_entry(
            "caddy",
            "Caddy",
            "Modern web server with automatic HTTPS",
            ProcessCategory::Proxy,
            now,
        ),
        // Node.js ecosystem
        builtin_entry(
            "node",
            "Node.js Server",
            "Node.js JavaScript runtime",
            ProcessCategory::Backend,
            now,
        ),
        builtin_entry(
            "bun",
            "Bun Server",
            "Bun JavaScript runtime and bundler",
            ProcessCategory::Backend,
            now,
        ),
        builtin_entry(
            "deno",
            "Deno Server",
            "Deno secure JavaScript/TypeScript runtime",
            ProcessCategory::Backend,
            now,
        ),
        // Python
        builtin_entry(
            "python",
            "Python Server",
            "Python application server",
            ProcessCategory::Backend,
            now,
        ),
        builtin_entry(
            "python3",
            "Python 3 Server",
            "Python 3 application server",
            ProcessCategory::Backend,
            now,
        ),
        builtin_entry(
            "uvicorn",
            "Uvicorn (ASGI)",
            "Lightning-fast ASGI server for Python",
            ProcessCategory::Backend,
            now,
        ),
        builtin_entry(
            "gunicorn",
            "Gunicorn (WSGI)",
            "Python WSGI HTTP server",
            ProcessCategory::Backend,
            now,
        ),
        // Ruby
        builtin_entry(
            "ruby",
            "Ruby Server",
            "Ruby application server",
            ProcessCategory::Backend,
            now,
        ),
        builtin_entry(
            "puma",
            "Puma",
            "Concurrent web server for Ruby/Rails",
            ProcessCategory::Backend,
            now,
        ),
        // Go
        builtin_entry(
            "go",
            "Go Server",
            "Go application server",
            ProcessCategory::Backend,
            now,
        ),
        builtin_entry(
            "golink",
            "golink",
            "Tailscale private shortlink service",
            ProcessCategory::DevTool,
            now,
        ),
        // Java
        builtin_entry(
            "java",
            "Java Server",
            "Java application server",
            ProcessCategory::Backend,
            now,
        ),
        // Rust
        builtin_entry(
            "cargo",
            "Cargo Dev Server",
            "Rust package manager running a dev server",
            ProcessCategory::DevTool,
            now,
        ),
        // PHP
        builtin_entry(
            "php",
            "PHP Server",
            "PHP application server",
            ProcessCategory::Backend,
            now,
        ),
        builtin_entry(
            "php-fpm",
            "PHP-FPM",
            "PHP FastCGI Process Manager",
            ProcessCategory::Backend,
            now,
        ),
        // Dev tools
        builtin_entry(
            "vite",
            "Vite Dev Server",
            "Next-generation frontend build tool",
            ProcessCategory::DevTool,
            now,
        ),
        builtin_entry(
            "webpack",
            "Webpack Dev Server",
            "JavaScript module bundler dev server",
            ProcessCategory::DevTool,
            now,
        ),
        builtin_entry(
            "next",
            "Next.js Dev Server",
            "React framework development server",
            ProcessCategory::Frontend,
            now,
        ),
        builtin_entry(
            "remix",
            "Remix Dev Server",
            "Full-stack React framework",
            ProcessCategory::Frontend,
            now,
        ),
        builtin_entry(
            "turbo",
            "Turborepo",
            "Monorepo build system",
            ProcessCategory::DevTool,
            now,
        ),
        // Message queues
        builtin_entry(
            "rabbitmq-server",
            "RabbitMQ",
            "Message broker and queue server",
            ProcessCategory::Infrastructure,
            now,
        ),
        // Tailscale services
        builtin_entry(
            "tailscaled",
            "Tailscale Daemon",
            "Tailscale VPN daemon",
            ProcessCategory::Infrastructure,
            now,
        ),
        // Homebrew
        builtin_entry(
            "brew",
            "Homebrew",
            "macOS package manager",
            ProcessCategory::DevTool,
            now,
        ),
    ];

    for entry in builtins {
        let key = entry.hash_key();
        kb.entries.insert(key, entry);
    }
}

fn builtin_entry(
    command: &str,
    display_name: &str,
    description: &str,
    category: ProcessCategory,
    timestamp: i64,
) -> KnowledgeEntry {
    KnowledgeEntry {
        fingerprint: ProcessFingerprint::new(command),
        display_name: display_name.to_string(),
        description: description.to_string(),
        category,
        group_id: None,
        confidence: 1.0,
        source: KnowledgeSource::Builtin,
        sightings: 0,
        updated_at: timestamp,
    }
}
