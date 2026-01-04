use super::types::{AnalysisContext, IcaAnalysisResponse, ProcessCategory};

/// Generate a display name from heuristics when ICA is not available
pub fn generate_fallback(context: &AnalysisContext) -> IcaAnalysisResponse {
    let (display_name, category, description) = analyze_context(context);

    IcaAnalysisResponse {
        display_name,
        description,
        category,
        group_hint: context.container_prefix.clone(),
        confidence: 0.5,
    }
}

fn analyze_context(context: &AnalysisContext) -> (String, ProcessCategory, String) {
    // Try to build a nice name from available context

    // Docker container with prefix
    if let Some(ref prefix) = context.container_prefix {
        let prefix_upper = capitalize_words(prefix);
        if let Some(ref container) = context.container_name {
            // Extract service name from container (e.g., "dss_app" -> "app")
            let service = container
                .strip_prefix(&format!("{}_", prefix))
                .unwrap_or(container);
            let service_upper = capitalize_words(service);

            let category = infer_category_from_name(service);
            let description = format!("{} {} service", prefix_upper, service);

            return (format!("{} {}", prefix_upper, service_upper), category, description);
        }
    }

    // Container name without prefix
    if let Some(ref container) = context.container_name {
        let name = capitalize_words(container);
        let category = infer_category_from_name(container);
        let description = format!("Docker container: {}", container);
        return (name, category, description);
    }

    // Project name + command
    if let Some(ref project) = context.project_name {
        let project_name = capitalize_words(project);
        let command = &context.command;
        let category = infer_category_from_command(command);
        let description = format!("{} running in project {}", command, project);
        return (
            format!("{} ({})", project_name, command),
            category,
            description,
        );
    }

    // Just command
    let category = infer_category_from_command(&context.command);
    let description = format!("{} process", context.command);
    (
        capitalize_words(&context.command),
        category,
        description,
    )
}

fn capitalize_words(s: &str) -> String {
    s.split(|c: char| c == '_' || c == '-' || c == ' ')
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn infer_category_from_name(name: &str) -> ProcessCategory {
    let lower = name.to_lowercase();

    // Database indicators
    if lower.contains("postgres")
        || lower.contains("mysql")
        || lower.contains("mongo")
        || lower.contains("db")
        || lower.contains("database")
    {
        return ProcessCategory::Database;
    }

    // Cache indicators
    if lower.contains("redis") || lower.contains("memcache") || lower.contains("cache") {
        return ProcessCategory::Cache;
    }

    // Proxy indicators
    if lower.contains("nginx")
        || lower.contains("proxy")
        || lower.contains("gateway")
        || lower.contains("lb")
        || lower.contains("loadbalancer")
    {
        return ProcessCategory::Proxy;
    }

    // Frontend indicators
    if lower.contains("frontend")
        || lower.contains("web")
        || lower.contains("ui")
        || lower.contains("client")
        || lower.contains("app")
    {
        return ProcessCategory::Frontend;
    }

    // Backend indicators
    if lower.contains("api")
        || lower.contains("backend")
        || lower.contains("server")
        || lower.contains("service")
    {
        return ProcessCategory::Backend;
    }

    // Infrastructure
    if lower.contains("worker")
        || lower.contains("queue")
        || lower.contains("scheduler")
        || lower.contains("cron")
    {
        return ProcessCategory::Infrastructure;
    }

    ProcessCategory::Unknown
}

fn infer_category_from_command(command: &str) -> ProcessCategory {
    let lower = command.to_lowercase();

    // Databases
    if lower.contains("postgres")
        || lower.contains("mysql")
        || lower.contains("mongo")
        || lower.contains("redis")
    {
        return ProcessCategory::Database;
    }

    // Frontend tools
    if lower.contains("vite")
        || lower.contains("webpack")
        || lower.contains("parcel")
        || lower.contains("next")
        || lower.contains("remix")
    {
        return ProcessCategory::Frontend;
    }

    // Backend runtimes
    if lower.contains("node")
        || lower.contains("python")
        || lower.contains("ruby")
        || lower.contains("go")
        || lower.contains("java")
        || lower.contains("php")
        || lower.contains("bun")
        || lower.contains("deno")
    {
        return ProcessCategory::Backend;
    }

    // Proxies
    if lower.contains("nginx") || lower.contains("caddy") || lower.contains("httpd") {
        return ProcessCategory::Proxy;
    }

    // Docker
    if lower.contains("docker") || lower.contains("orbstack") {
        return ProcessCategory::Infrastructure;
    }

    ProcessCategory::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize_words() {
        assert_eq!(capitalize_words("hello_world"), "Hello World");
        assert_eq!(capitalize_words("dss_app"), "Dss App");
        assert_eq!(capitalize_words("my-project"), "My Project");
    }

    #[test]
    fn test_container_with_prefix() {
        let context = AnalysisContext {
            command: "node".to_string(),
            port: Some(3001),
            project_name: None,
            container_name: Some("dss_app".to_string()),
            container_prefix: Some("dss".to_string()),
        };
        let result = generate_fallback(&context);
        assert_eq!(result.display_name, "Dss App");
        assert_eq!(result.group_hint, Some("dss".to_string()));
    }

    #[test]
    fn test_project_context() {
        let context = AnalysisContext {
            command: "node".to_string(),
            port: Some(3000),
            project_name: Some("my-project".to_string()),
            container_name: None,
            container_prefix: None,
        };
        let result = generate_fallback(&context);
        assert!(result.display_name.contains("My Project"));
    }
}
