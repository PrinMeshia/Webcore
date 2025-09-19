mod parser;
mod ast;
pub mod codegen { pub mod codegen_html; pub mod codegen_css; pub mod codegen_js; }
mod theme;
mod css_processor;

use std::env;
use std::fs;
use std::path::Path;
use std::collections::HashMap;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::net::UdpSocket;
use qrcode::QrCode;
use qrcode::render::unicode;
use notify::{RecommendedWatcher, RecursiveMode, Watcher, EventKind};
use tiny_http::{Response, Server, Request};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: webc <command> [options]");
        println!("Commands:");
        println!("  build    Build the project");
        println!("  dev      Start development server (not implemented yet)");
        return;
    }
    
    match args[1].as_str() {
        "build" => {
            if let Err(e) = build_project() {
                eprintln!("Build failed: {}", e);
                std::process::exit(1);
            }
        }
        "dev" => {
            let mut port: u16 = 3000;
            let mut host: Option<String> = None;
            let mut auto_open = false;
            // Args: dev [port] [--host 0.0.0.0] [--open]
            let mut i = 2;
            // Back-compat: if a bare number is provided, treat as port
            if let Some(arg) = args.get(i) {
                if let Ok(p) = arg.parse::<u16>() { port = p; i += 1; }
            }
            while i < args.len() {
                match args[i].as_str() {
                    "--host" => {
                        if let Some(h) = args.get(i+1) { host = Some(h.clone()); }
                        i += 2;
                    }
                    "--open" => { auto_open = true; i += 1; }
                    _ => { i += 1; }
                }
            }
            if let Err(e) = dev_server_with_options(port, host, auto_open) {
                eprintln!("Dev server error: {}", e);
                std::process::exit(1);
            }
        }
        _ => {
            println!("Unknown command: {}", args[1]);
            std::process::exit(1);
        }
    }
}

fn build_project() -> Result<(), String> {
    println!("🔨 Building WebCore project...");
    
    // Read project config
    let config = read_config()?;
    println!("📁 Project: {}", config.app_title);
    
    // Create dist directory
    let dist_dir = Path::new("dist");
    if dist_dir.exists() {
        fs::remove_dir_all(dist_dir).map_err(|e| format!("Failed to clean dist: {}", e))?;
    }
    fs::create_dir_all(dist_dir).map_err(|e| format!("Failed to create dist: {}", e))?;
    
    // Load theme
    let theme = if Path::new("theme.toml").exists() {
        println!("🎨 Loading theme...");
        Some(theme::load_theme("theme.toml")?)
    } else {
        println!("⚠️  No theme.toml found, using default theme");
        None
    };
    
    // Load and parse all WebCore files
    let mut document = ast::WebCoreDocument {
        app: None,
        layouts: HashMap::new(),
        pages: HashMap::new(),
        components: HashMap::new(),
    };
    
    // Load app.webc first
    let app_path = Path::new("src/app.webc");
    if app_path.exists() {
        let content = fs::read_to_string(app_path).map_err(|e| format!("Failed to read app.webc: {}", e))?;
        let parsed = parser::parse_webc(&content).map_err(|e| format!("Parse error in app.webc: {:?}", e))?;
        document.app = parsed.app;
    }

    // Load layouts
    let layouts_dir = Path::new("src/layouts");
    if layouts_dir.exists() {
        for entry in fs::read_dir(layouts_dir).map_err(|e| format!("Failed to read layouts: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("webc") {
                let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read {:?}: {}", path, e))?;
                let parsed = parser::parse_webc(&content).map_err(|e| format!("Parse error in {:?}: {:?}", path, e))?;
                
                // Merge layouts
                for (name, layout) in parsed.layouts {
                    document.layouts.insert(name, layout);
                }
            }
        }
    }
    
    // Load components
    let components_dir = Path::new("src/components");
    if components_dir.exists() {
        for entry in fs::read_dir(components_dir).map_err(|e| format!("Failed to read components: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("webc") {
                let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read {:?}: {}", path, e))?;
                let parsed = parser::parse_webc(&content).map_err(|e| format!("Parse error in {:?}: {:?}", path, e))?;
                
                // Merge components
                for (name, component) in parsed.components {
                    document.components.insert(name, component);
                }
            }
        }
    }

    // Load pages
    let pages_dir = Path::new("src/pages");
    if pages_dir.exists() {
        for entry in fs::read_dir(pages_dir).map_err(|e| format!("Failed to read pages: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("webc") {
                let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read {:?}: {}", path, e))?;
                let parsed = parser::parse_webc(&content).map_err(|e| format!("Parse error in {:?}: {:?}", path, e))?;
                
                // Merge pages and components
                for (name, page) in parsed.pages {
                    document.pages.insert(name, page);
                }
                for (name, component) in parsed.components {
                    document.components.insert(name, component);
                }
            }
        }
    }
    
    // Collect all handlers from all pages
    let mut all_handlers = Vec::new();
    
    // Generate HTML for each page
    for (page_name, _) in &document.pages {
        println!("📄 Generating: {}.html", page_name);
        let options = codegen::codegen_html::HtmlPageOptions {
            lang: config.app_lang.clone(),
            title: config.app_title.clone(),
        };
        let html_result = codegen::codegen_html::generate_html(&document, page_name, &options)?;
        all_handlers.extend(html_result.handlers);
        let output_path = dist_dir.join(format!("{}.html", page_name));
        fs::write(&output_path, html_result.html).map_err(|e| format!("Failed to write {:?}: {}", output_path, e))?;
    }
    
    // Generate HTML for each component that looks like a page
    println!("🔍 Found {} components", document.components.len());
    for (component_name, component) in &document.components {
        println!("🔍 Component: {} (ends with Page: {})", component_name, component_name.ends_with("Page"));
        if component_name.ends_with("Page") {
            println!("📄 Generating: {}.html", component_name);
            // Create a temporary page from the component
            let temp_page = ast::Page {
                name: component_name.clone(),
                content: component.view.clone(),
            };
            let mut temp_doc = document.clone();
            temp_doc.pages.insert(component_name.clone(), temp_page);
            
            let options = codegen::codegen_html::HtmlPageOptions {
                lang: config.app_lang.clone(),
                title: config.app_title.clone(),
            };
            let html_result = codegen::codegen_html::generate_html(&temp_doc, component_name, &options)?;
            all_handlers.extend(html_result.handlers);
            let output_path = dist_dir.join(format!("{}.html", component_name));
            fs::write(&output_path, html_result.html).map_err(|e| format!("Failed to write {:?}: {}", output_path, e))?;
        }
    }
    
    // If no pages were generated, create a default index.html
    if document.pages.is_empty() && !document.components.iter().any(|(name, _)| name.ends_with("Page")) {
        println!("📄 Generating: index.html (default)");
        let default_page = ast::Page {
            name: "index".to_string(),
            content: vec![
                ast::Element::Tag {
                    name: "h1".to_string(),
                    attributes: vec![],
                    content: vec![ast::Element::Text("Welcome to WebCore".to_string())],
                },
                ast::Element::Tag {
                    name: "p".to_string(),
                    attributes: vec![],
                    content: vec![ast::Element::Text("This is a default page.".to_string())],
                },
            ],
        };
        let mut temp_doc = document.clone();
        temp_doc.pages.insert("index".to_string(), default_page);
        
        let options = codegen::codegen_html::HtmlPageOptions {
            lang: config.app_lang.clone(),
            title: config.app_title.clone(),
        };
        let html_result = codegen::codegen_html::generate_html(&temp_doc, "index", &options)?;
        all_handlers.extend(html_result.handlers);
        let output_path = dist_dir.join("index.html");
        fs::write(&output_path, html_result.html).map_err(|e| format!("Failed to write {:?}: {}", output_path, e))?;
    }
    
    // Generate theme files
    if let Some(theme) = &theme {
        println!("🎨 Generating theme files...");
        
        // Generate CSS variables using codegen_css
        let css_variables = codegen::codegen_css::generate_theme_css(theme);
        
        // Post-process CSS with LightningCSS
        let processed_css = if config.mode == "prod" {
            println!("🔧 Minifying CSS with LightningCSS...");
            css_processor::minify_css(&css_variables)?
        } else {
            css_processor::format_css(&css_variables)?
        };
        
        let css_path = dist_dir.join("theme.css");
        fs::write(&css_path, processed_css).map_err(|e| format!("Failed to write theme.css: {}", e))?;
        
        // Generate WebCore runtime JS with compiled handlers
        let mut runtime_js = codegen::codegen_js::generate_runtime_js(&all_handlers);
        
        // Add component-specific JavaScript
        for component in document.components.values() {
            if !component.state.is_empty() {
                runtime_js.push_str(&format!("\n// Component: {}\n", component.name));
                for state_var in &component.state {
                    runtime_js.push_str(&format!(
                        "window.__webcore_state__.set('{}', {});\n",
                        state_var.name,
                        state_var.default_value.as_ref().unwrap_or(&"null".to_string())
                    ));
                }
            }
        }
        
        let js_path = dist_dir.join("webcore.js");
        fs::write(&js_path, runtime_js).map_err(|e| format!("Failed to write webcore.js: {}", e))?;
    }
    
    // Copy public assets
    let public_dir = Path::new("public");
    if public_dir.exists() {
        println!("📁 Copying public assets...");
        copy_dir_recursive(public_dir, dist_dir)?;
    }

    // Generate index.html with links to pages/components
    generate_index_html(&document)?;
    
    println!("✅ Build completed successfully!");
    Ok(())
}

fn read_config() -> Result<Config, String> {
    let config_path = Path::new("webc.toml");
    if !config_path.exists() {
        return Err("webc.toml not found".to_string());
    }
    
    let content = fs::read_to_string(config_path).map_err(|e| format!("Failed to read webc.toml: {}", e))?;
    
    let parsed: WebcToml = toml::from_str(&content).map_err(|e| format!("Failed to parse webc.toml: {}", e))?;
    let app_title = parsed.app.as_ref().and_then(|a| a.title.clone()).unwrap_or_else(|| "WebCore App".to_string());
    let app_lang = parsed.app.as_ref().and_then(|a| a.lang.clone()).unwrap_or_else(|| "fr".to_string());
    let mode = parsed.app.as_ref().and_then(|a| a.mode.clone()).unwrap_or_else(|| "dev".to_string());
    
    Ok(Config { 
        app_title,
        app_lang,
        mode,
    })
}

#[derive(Debug)]
struct Config {
    app_title: String,
    app_lang: String,
    mode: String,
}

#[derive(Debug, Deserialize)]
struct WebcToml {
    app: Option<AppSection>,
}

#[derive(Debug, Deserialize)]
struct AppSection {
    title: Option<String>,
    lang: Option<String>,
    mode: Option<String>,
}

fn generate_index_html(document: &ast::WebCoreDocument) -> Result<(), String> {
    let mut links: Vec<(String, String)> = Vec::new();
    for page_name in document.pages.keys() {
        links.push((format!("{}.html", page_name), page_name.clone()));
    }
    for (component_name, _) in &document.components {
        if component_name.ends_with("Page") {
            links.push((format!("{}.html", component_name), component_name.clone()));
        }
    }
    links.sort_by(|a, b| a.1.cmp(&b.1));

    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html lang=\"fr\">\n<head>\n");
    html.push_str("  <meta charset=\"UTF-8\">\n");
    html.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    html.push_str("  <title>Index</title>\n");
    html.push_str("  <link rel=\"stylesheet\" href=\"theme.css\">\n");
    html.push_str("</head>\n<body>\n<h1>Pages</h1>\n<ul>\n");
    for (href, label) in links {
        html.push_str(&format!("  <li><a href=\"{}\">{}</a></li>\n", href, label));
    }
    html.push_str("</ul>\n<script src=\"webcore.js\"></script>\n</body>\n</html>\n");

    fs::write("dist/index.html", html).map_err(|e| format!("Failed to write index.html: {}", e))
}

fn dev_server_with_options(port: u16, host: Option<String>, auto_open: bool) -> Result<(), String> {
    // initial build
    build_project()?;

    // start file watcher
    let rebuild_flag = Arc::new(Mutex::new(false));
    let flag_clone = rebuild_flag.clone();

    let mut watcher: RecommendedWatcher = notify::recommended_watcher(move |res: std::result::Result<notify::Event, notify::Error>| {
        if let Ok(event) = res {
            match event.kind {
                EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
                    if let Ok(mut f) = flag_clone.lock() { *f = true; }
                }
                _ => {}
            }
        }
    }).map_err(|e| format!("watcher error: {}", e))?;

    watcher.watch(Path::new("src"), RecursiveMode::Recursive).map_err(|e| format!("watch error: {}", e))?;
    if Path::new("theme.toml").exists() { watcher.watch(Path::new("theme.toml"), RecursiveMode::NonRecursive).map_err(|e| format!("watch error: {}", e))?; }
    watcher.watch(Path::new("webc.toml"), RecursiveMode::NonRecursive).map_err(|e| format!("watch error: {}", e))?;

    // start server with port auto-increment if in use
    let (server, bound_port) = bind_server_with_fallback(port, 50)?;
    let local_host = match host.as_deref() {
        Some("0.0.0.0") => "localhost".to_string(),
        Some(h) => h.to_string(),
        None => "localhost".to_string(),
    };
    println!("🚀 Dev server running at:");
    println!("  Local:   http://{}:{}", local_host, bound_port);
    let network_ip = match host.as_deref() {
        Some("0.0.0.0") | None => get_primary_ipv4(),
        Some(h) => Some(h.to_string()),
    };
    let mut qr_url: Option<String> = None;
    if let Some(ip) = network_ip.clone() {
        if ip != "127.0.0.1" && ip != "localhost" && ip != "0.0.0.0" {
            let url = format!("http://{}:{}", ip, bound_port);
            println!("  Network: {}", url);
            qr_url = Some(url);
        }
    }

    // auto-open browser
    if auto_open {
        let open_url = format!("http://{}:{}", local_host, bound_port);
        let _ = open::that_detached(open_url);
    }

    // print QR code for network URL if available
    if let Some(url) = qr_url {
        if let Ok(code) = QrCode::new(url.as_bytes()) {
            println!("\n  Scan QR (Network):");
            let qr = code
                .render::<unicode::Dense1x2>()
                .quiet_zone(true)
                .build();
            println!("{}", qr);
        }
    }

    // spawn rebuild loop
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(200));
            let mut do_rebuild = false;
            if let Ok(mut f) = rebuild_flag.lock() {
                if *f { do_rebuild = true; *f = false; }
            }
            if do_rebuild {
                println!("♻️  Rebuilding...");
                if let Err(e) = build_project() { eprintln!("Rebuild failed: {}", e); }
            }
        }
    });

    // serve loop
    for request in server.incoming_requests() {
        if let Err(e) = handle_request(request) {
            eprintln!("request error: {}", e);
        }
    }

    Ok(())
}

fn handle_request(request: Request) -> Result<(), String> {
    let url = request.url();
    let path = if url == "/" { "dist/index.html".to_string() } else { format!("dist{}", url) };
    let path = Path::new(&path);
    let file_path = if path.is_dir() { path.join("index.html") } else { path.to_path_buf() };
    match fs::read(&file_path) {
        Ok(bytes) => {
            let content_type = match file_path.extension().and_then(|e| e.to_str()).unwrap_or("") {
                "html" => "text/html; charset=utf-8",
                "css" => "text/css; charset=utf-8",
                "js" => "application/javascript; charset=utf-8",
                "png" => "image/png",
                "jpg" | "jpeg" => "image/jpeg",
                _ => "application/octet-stream",
            };
            let response = Response::from_data(bytes).with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()).unwrap());
            request.respond(response).map_err(|e| format!("respond error: {}", e))
        }
        Err(_) => {
            let response = Response::from_string("Not Found").with_status_code(404);
            request.respond(response).map_err(|e| format!("respond error: {}", e))
        }
    }
}

fn bind_server_with_fallback(start_port: u16, max_tries: u16) -> Result<(Server, u16), String> {
    let mut port = start_port;
    for _ in 0..max_tries {
        match Server::http(("0.0.0.0", port)) {
            Ok(server) => return Ok((server, port)),
            Err(e) => {
                // Try to detect port-in-use and fallback to next port
                let is_in_use = e.as_ref()
                    .downcast_ref::<std::io::Error>()
                    .map(|ioe| ioe.kind() == std::io::ErrorKind::AddrInUse)
                    .unwrap_or(false);
                if is_in_use {
                    port = port.saturating_add(1);
                    continue;
                }
                return Err(format!("server error: {}", e));
            }
        }
    }
    Err(format!("no free port in range {}..{}", start_port, start_port.saturating_add(max_tries)))
}

fn get_primary_ipv4() -> Option<String> {
    // Determine the primary outbound IP by opening a UDP socket
    if let Ok(socket) = UdpSocket::bind(("0.0.0.0", 0)) {
        if socket.connect(("8.8.8.8", 80)).is_ok() {
            if let Ok(addr) = socket.local_addr() {
                if let std::net::IpAddr::V4(ipv4) = addr.ip() {
                    // Skip loopback just in case
                    if !ipv4.is_loopback() {
                        return Some(ipv4.to_string());
                    }
                }
            }
        }
    }
    None
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    if src.is_dir() {
        fs::create_dir_all(dst).map_err(|e| format!("Failed to create dir {:?}: {}", dst, e))?;
        for entry in fs::read_dir(src).map_err(|e| format!("Failed to read dir {:?}: {}", src, e))? {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            copy_dir_recursive(&src_path, &dst_path)?;
        }
    } else {
        fs::copy(src, dst).map_err(|e| format!("Failed to copy {:?} to {:?}: {}", src, dst, e))?;
    }
    Ok(())
}
