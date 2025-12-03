// src/main.rs
// Rust PDF extraction benchmark/compare tool
// 说明：编译时根据 Cargo.toml 中添加的依赖启用各个后端。
// 建议在 release 模式下运行以获得更现实的基准数据： `cargo run --release -- /path/to/file.pdf`

use std::env;
use std::fs;
use std::time::{Duration, Instant};

fn human_ms(d: Duration) -> f64 {
    (d.as_secs_f64() * 1000.0)
}

fn report(name: &str, duration: Duration, bytes: usize, pages: Option<usize>, ok: bool, err: Option<String>) {
    println!("---");
    println!("Backend: {}", name);
    println!("  Time: {:.2} ms", human_ms(duration));
    println!("  Extracted bytes: {}", bytes);
    if let Some(p) = pages { println!("  Pages: {}", p); }
    println!("  Success: {}", ok);
    if let Some(e) = err { println!("  Error: {}", e); }
}

// ========== Backend: pdf-extract (pure Rust wrapper) ==========
// Crate: pdf-extract (docs: crates.io / docs.rs)
// Add to Cargo.toml: pdf-extract = "0.7"  (检查 crates.io 最新版本)
#[cfg(feature = "pdf_extract")]
mod pdf_extract_backend {
    pub fn run(path: &str) -> Result<(usize, Option<usize>), String> {
        let bytes = std::fs::read(path).map_err(|e| format!("read error: {}", e))?;
        // pdf_extract provides extract_text_from_mem and extract_text_by_pages
        match pdf_extract::extract_text_from_mem(&bytes) {
            Ok(text) => {
				//println!("pdf_extract: {}", text);
                let len = text.as_bytes().len();
                // naive page count by splitting on form feed or by pages API could be used
                let pages = text.matches('\x0C').count();
                Ok((len, if pages>0 {Some(pages)} else {None}))
            }
            Err(e) => Err(format!("pdf-extract error: {:?}", e)),
        }
    }
}

// ========== Backend: lopdf (lower-level parsing) ==========
// Crate: lopdf
// Add to Cargo.toml: lopdf = "0.27"
#[cfg(feature = "lopdf")]
mod lopdf_backend {
    use lopdf::{Document, Object, ObjectId};
    use std::collections::HashMap;

    // A simple text extraction attempt using content streams.
    pub fn run(path: &str) -> Result<(usize, Option<usize>), String> {
        let doc = Document::load(path).map_err(|e| format!("lopdf load error: {}", e))?;
        // gather pages
        let mut extracted = String::new();
        let pages = doc.get_pages();
        for (_pageno, &page_id) in pages.iter() {
            let page = doc.get_object(page_id).map_err(|e| format!("get page error: {}", e))?;
            // try to get Contents
            if let Ok(contents) = doc.extract_text(&[*_pageno]) {
                // lopdf has Document::extract_text convenience we can call
                extracted.push_str(&contents);
            } else {
                // fallback: try to parse content stream manually (not implemented here)
            }
        }
		//println!("lopdf: {}", extracted);
        let len = extracted.as_bytes().len();
        Ok((len, Some(pages.len())))
    }
}

// ========== Backend: pdfium-render (bindings to PDFium) ==========
// Crate: pdfium-render
// Add to Cargo.toml: pdfium-render = "1.0" (检查最新版本)
// NOTE: Requires PDFium native library installed or vendored. See docs.
#[cfg(feature = "pdfium")]
mod pdfium_backend {
    use pdfium_render::prelude::*;
    pub fn run(path: &str) -> Result<(usize, Option<usize>), String> {
        // initialize library
		let pdfium = Pdfium::new(
			Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./lib")).unwrap()
		);

        let doc = pdfium.load_pdf_from_file(path, None)
            .map_err(|e| format!("pdfium load error: {:?}", e))?;
        let page_count = doc.pages().len();
        let mut all = String::new();
        for i in 0..page_count {
            let page = doc.pages().get(i).unwrap();
            // pdfium-render supports page.text() to get text
			let text = page.text().unwrap().all(); // https://github.com/ajrcarey/pdfium-render/blob/master/examples/text_extract.rs
            all.push_str(&text);
        }
		println!("pdfium: {}", all);
        Ok((all.as_bytes().len(), Some(page_count.into())))
    }
}

// ========== Backend: poppler-rs (bindings to libpoppler) ==========
// Crate: poppler-rs
// Add to Cargo.toml: poppler-rs = "0.25" (检查最新)
// Requires libpoppler dev headers on system.
#[cfg(feature = "poppler")]
mod poppler_backend {
    use poppler::PopplerDocument;
    pub fn run(path: &str) -> Result<(usize, Option<usize>), String> {
        let doc = PopplerDocument::new_from_file(path, "").map_err(|e| format!("poppler new error: {:?}", e))?;
        let n = doc.get_n_pages();
        let mut all = String::new();
        for i in 0..n {
            let page = doc.get_page(i).ok_or_else(|| format!("poppler get_page {}", i))?;
            let txt = page.get_text().ok_or_else(|| format!("poppler get_text {}", i))?;
            all.push_str(&txt);
        }
        Ok((all.as_bytes().len(), Some(n)))
    }
}

// ========== Fallback: generic CLI pdftotext (if available on system) ==========
mod cli_pdftotext {
    use std::process::Command;
    pub fn run(path: &str) -> Result<(usize, Option<usize>), String> {
        // requires `pdftotext` (poppler-utils) installed
        let out = Command::new("pdftotext")
            .arg("-q") // quiet
            .arg("-layout")
            .arg(path)
            .arg("-") // write to stdout
            .output()
            .map_err(|e| format!("failed to run pdftotext: {}", e))?;
        if !out.status.success() {
            return Err(format!("pdftotext failed: {}", out.status));
        }
        let txt = String::from_utf8_lossy(&out.stdout).to_string();
		//println!("pdftotext: {}", txt);
        Ok((txt.as_bytes().len(), None))
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} /path/to/file.pdf", args[0]);
        std::process::exit(2);
    }
    let path = &args[1];

    // read file quick sanity
    if let Err(e) = fs::metadata(path) {
        eprintln!("Failed to stat file {}: {}", path, e);
        std::process::exit(1);
    }

    println!("PDF extraction benchmark for: {}", path);
    println!("Backends attempted (features):");

    // Attempt pdf-extract if compiled
    #[cfg(feature = "pdf_extract")]
    {
        let start = Instant::now();
        let res = pdf_extract_backend::run(path);
        let dur = start.elapsed();
        match res {
            Ok((bytes, pages)) => report("pdf-extract", dur, bytes, pages, true, None),
            Err(e) => report("pdf-extract", dur, 0, None, false, Some(e)),
        }
    }
    #[cfg(not(feature = "pdf_extract"))]
    {
        println!("  - pdf-extract: (disabled at compile time)");
    }

    // lopdf
    #[cfg(feature = "lopdf")]
    {
        let start = Instant::now();
        let res = lopdf_backend::run(path);
        let dur = start.elapsed();
        match res {
            Ok((bytes, pages)) => report("lopdf", dur, bytes, pages, true, None),
            Err(e) => report("lopdf", dur, 0, None, false, Some(e)),
        }
    }

    #[cfg(not(feature = "lopdf"))]
    {
        println!("  - lopdf: (disabled at compile time)");
    }

    // pdfium
    #[cfg(feature = "pdfium")]
    {
        let start = Instant::now();
        let res = pdfium_backend::run(path);
        let dur = start.elapsed();
        match res {
            Ok((bytes, pages)) => report("pdfium-render", dur, bytes, pages, true, None),
            Err(e) => report("pdfium-render", dur, 0, None, false, Some(e)),
        }
    }
    #[cfg(not(feature = "pdfium"))]
    {
        println!("  - pdfium-render: (disabled at compile time)");
    }

    // poppler
    #[cfg(feature = "poppler")]
    {
        let start = Instant::now();
        let res = poppler_backend::run(path);
        let dur = start.elapsed();
        match res {
            Ok((bytes, pages)) => report("poppler-rs", dur, bytes, pages, true, None),
            Err(e) => report("poppler-rs", dur, 0, None, false, Some(e)),
        }
    }
    #[cfg(not(feature = "poppler"))]
    {
        println!("  - poppler-rs: (disabled at compile time)");
    }

    // CLI pdftotext fallback
    {
        let start = Instant::now();
        match cli_pdftotext::run(path) {
            Ok((bytes, _)) => report("pdftotext (cli)", start.elapsed(), bytes, None, true, None),
            Err(e) => report("pdftotext (cli)", start.elapsed(), 0, None, false, Some(e)),
        }
    }

    println!("Done.");
}
