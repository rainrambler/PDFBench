# PDF Extraction Benchmark Tool (Rust)

A lightweight, extensible Rust command-line tool for **benchmarking and comparing multiple PDF text-extraction libraries**.  
It measures:

- Extraction success / failure    
- Execution time    
- Extracted text length    
- Page count (when available)    

The tool is designed to help developers evaluate the strengths and weaknesses of Rust-native PDF parsers and native-library bindings.

---

## ✨ Supported Backends

The tool supports multiple PDF extraction libraries through **Cargo feature flags**:

|Backend|Type|Notes|
|---|---|---|
|**pdf-extract**|Pure Rust|Simple and fast; handles standard text objects reasonably well.|
|**lopdf**|Pure Rust (low-level)|Full PDF object model; text extraction quality depends on content stream complexity.|
|**pdfium-render**|Native binding|Uses Google PDFium; excellent text extraction and layout accuracy. Requires native PDFium.|
|**poppler-rs**|Native binding|Uses Poppler; strong full-text extraction. Requires system Poppler libraries.|
|**pdftotext (CLI)**|External command|Fallback using `pdftotext` (Poppler utils). Optional and included by default.|

You may enable or disable any backend depending on your platform or available native libraries.

---

## 📦 Features

- Benchmark multiple PDF engines in one run    
- High-level text extraction comparison    
- Simple CLI    
- Optional batch mode to benchmark an entire directory    
- Works cross-platform (Windows / Linux / macOS)    
- Modular structure—easy to add new PDF backends    

---

## 📁 Structure

```bash
src/
 └─ main.rs     # Core benchmark logic and backend dispatch
Cargo.toml      # Features toggle individual backends

```

Each backend is isolated behind a compile-time feature to avoid unwanted dependencies.

---

## ⚙️ Installation

Clone the repository:

```bash
git clone https://github.com/rainrambler/PDFBench.git
cd PDFBench
```

Install dependencies based on the backends you enable:

### Pure Rust (works on all platforms, no native libs):

- `pdf-extract`    
- `lopdf`    

### Requires native libs:

#### PDFium

- Linux: install PDFium (system or vendored)    
- macOS: `brew install pdfium`    
- Windows: download PDFium binaries from the PDFium project    

#### Poppler

- Linux: `apt install libpoppler-dev`    
- macOS: `brew install poppler`    
- Windows: build Poppler manually or use a package    

---

## 🚀 Usage

### Run on a single PDF

```bash
cargo run --release --features "pdf_extract lopdf pdfium poppler" -- /path/to/file.pdf
```

Output example:

```yaml
Backend: pdf-extract
  Time: 18.32 ms
  Extracted bytes: 12403
  Pages: 4
  Success: true
---
Backend: pdfium-render
  Time: 4.91 ms
  Extracted bytes: 12198
  Pages: 4
  Success: true
...

```

---

## 🛠️ Cargo Features

Example `Cargo.toml`:

```toml
[features]
default = []
pdf_extract = ["pdf-extract"]
lopdf = ["lopdf"]
pdfium = ["pdfium-render"]
poppler = ["poppler-rs"]
```

Enable all backends:

`cargo run --release --features "pdf_extract lopdf pdfium poppler"`

Enable only pure-Rust backends:

`cargo run --release --features "pdf_extract lopdf"`

---

## 🧪 What This Tool Measures

For each backend:

- **Parsing time** (ms)    
- **Extracted text byte count**    
- **Page count** (if provided by backend)    
- **Error type** (if extraction fails)    

This makes it easy to compare:

- Speed    
- Text completeness    
- Backend robustness    
- Dependency footprint    

---

## ❗ Known Limitations

- Extraction quality varies **hugely** across libraries:    
    - Some libraries reconstruct layout; others do only raw token extraction.        
    - Fonts with embedded CMaps may require backend-specific handling.        
- Scanned PDFs (image-only) will extract _zero_ text unless OCR is added.    
- Native libraries (PDFium / Poppler) require platform-specific installation.    

If you want, OCR pipelines (Tesseract, PaddleOCR, etc.) can be integrated as optional modules.

---

## 📜 License

MIT License.

---

## 🤝 Contributing

Pull requests are welcome!  
Ideas for contribution:

- Add new backends (pdf-rs, xpdf bindings, MuPDF)    
- Add CSV / JSON export    
- Add HTML benchmark report    
- Add parallel batch mode    
- Add support for text layout reconstruction benchmarking    

---

## ⭐ Support

If you find this project useful, consider giving the repository a star!