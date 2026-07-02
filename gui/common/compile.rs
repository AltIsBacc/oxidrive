// A stripped down version of 
// https://docs.rs/crate/slint-build/1.16.1/source/lib.rs
//

use i_slint_compiler::generator;

pub struct LibraryPath {
    pub name: String,
    pub path: std::path::PathBuf
}

impl LibraryPath {
    pub fn new(
        name: impl Into<String>,
        path: impl Into<std::path::PathBuf>,
    ) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
        }
    }
}

pub struct CompilerConfigurationBuilder {
    config: i_slint_compiler::CompilerConfiguration,
    library_paths: Vec<LibraryPath>,
}

impl Default for CompilerConfigurationBuilder {
    fn default() -> Self {
        Self {
            config: i_slint_compiler::CompilerConfiguration::new(
                generator::OutputFormat::Rust,
            ),
            library_paths: Vec::new(),
        }
    }
}

impl CompilerConfigurationBuilder {
    #[allow(unused)]
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(unused)]
    pub fn with_include_paths(mut self, include_paths: Vec<std::path::PathBuf>) -> Self {
        self.config.include_paths = include_paths;
        self
    }

    pub fn with_library_paths(mut self, library_paths: Vec<LibraryPath>) -> Self {
        for new in library_paths {
            if !self.library_paths.iter().any(|l| l.name == new.name) {
                self.library_paths.push(new);
            }
        }
        self
    }

    pub fn with_style(mut self, style: &str) -> Self {
        self.config.style = Some(style.to_string());
        self
    }

    pub fn build(mut self) -> i_slint_compiler::CompilerConfiguration {
        self.config.library_paths = self.library_paths
            .into_iter()
            .map(|l| (l.name, l.path))
            .collect();

        self.config
    }
}

#[derive(derive_more::Error, derive_more::Display, Debug)]
#[non_exhaustive]
pub enum CompileError {
    #[display("{_0:?}")]
    CompileError(#[error(not(source))] Vec<String>),
    #[display("failed to create output file: {_0}")]
    SaveError(#[error(source)] std::io::Error),
}

#[allow(unused)]
pub fn compile(
    target: impl AsRef<std::path::Path>,
) -> Result<(), CompileError> {
    compile_with_config(
        target,
        CompilerConfigurationBuilder::default().build(),
    )
}

pub fn compile_with_config(
    target: impl AsRef<std::path::Path>,
    config: i_slint_compiler::CompilerConfiguration,
) -> Result<(), CompileError> {
    let manifest_dir = std::path::PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR")
            .expect("compile_with_config must be called from a build script"),
    );

    let input_path = manifest_dir.join(target.as_ref());

    let out_dir = std::env::var_os("OUT_DIR")
        .expect("compile_with_config must be called from a build script");

    let output_path = std::path::Path::new(&out_dir).join(
        input_path
            .file_stem()
            .map(std::path::Path::new)
            .unwrap_or_else(|| std::path::Path::new("slint_out"))
            .with_extension("rs"),
    );

    let mut diag = i_slint_compiler::diagnostics::BuildDiagnostics::default();

    let syntax_node = i_slint_compiler::parser::parse_file(&input_path, &mut diag);

    if diag.has_errors() {
        let vec = diag.to_string_vec();
        diag.print();
        return Err(CompileError::CompileError(vec));
    }

    let syntax_node = syntax_node.expect("no errors but missing syntax node");

    let mut compiler_config = config;
    compiler_config.translation_domain = std::env::var("CARGO_PKG_NAME").ok();

    let (doc, diag, loader) =
        futures::executor::block_on(i_slint_compiler::compile_syntax_node(
            syntax_node,
            diag,
            compiler_config,
        ));

    if diag.has_errors() {
        let vec = diag.to_string_vec();
        diag.print();
        return Err(CompileError::CompileError(vec));
    }

    for path in &diag.all_loaded_files {
        if path.is_absolute() {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
    println!("cargo:rerun-if-changed={}", input_path.display());

    diag.diagnostics_as_string().lines().for_each(|w| {
        if !w.is_empty() {
            println!("cargo:warning={}", w.strip_prefix("warning: ").unwrap_or(w));
        }
    });

    let generated =
        i_slint_compiler::generator::rust::generate(&doc, &loader.compiler_config)
            .map_err(|e| CompileError::CompileError(vec![e.to_string()]))?;

    let output_file =
        std::fs::File::create(&output_path).map_err(CompileError::SaveError)?;

    let mut writer = std::io::BufWriter::new(output_file);
    std::io::Write::write_fmt(&mut writer, format_args!("{generated}"))
        .map_err(CompileError::SaveError)?;

    println!(
        "cargo:rustc-env=SLINT_INCLUDE_GENERATED={}",
        output_path.display()
    );

    for var in &[
        "SLINT_STYLE",
        "SLINT_FONT_SIZES",
        "SLINT_SCALE_FACTOR",
        "SLINT_EMBED_RESOURCES",
        "SLINT_EMIT_DEBUG_INFO",
    ] {
        println!("cargo:rerun-if-env-changed={var}");
    }

    Ok(())
}

