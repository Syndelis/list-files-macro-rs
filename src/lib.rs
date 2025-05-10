//! A simple proc macro to generate a const list of filenames and optionally apply a macro to each.
//!
//! ```rust
//! use std::fs::canonicalize;
//!
//! use list_files_macro::list_files;
//!
//! fn get_full_path(path: &str) -> String {
//! 	canonicalize(path).unwrap().into_os_string().into_string().unwrap()
//! }
//!
//! const FILENAMES: [&'static str; 3] = list_files!("../tests/files/*.rs");
//!
//! assert_eq!(FILENAMES, [
//! 	"tests/files/a.rs",
//! 	"tests/files/b.rs",
//! 	"tests/files/c.rs",
//! ].map(get_full_path));
//!
//! const CONTENTS: [&'static str; 3] = list_files!(include_str, "../tests/files/*.rs");
//!
//! assert_eq!(CONTENTS[0], r#"
//! pub fn run() -> &'static str {
//! 	"A"
//! }
//! "#);
//!
//! macro_rules! run_file {
//! 	($x:expr) => {
//! 		{
//! 			#[path = $x]
//! 			mod file;
//! 			file::run()
//! 		}
//! 	};
//! }
//!
//! let results = list_files!(run_file, "../tests/files/*.rs");
//!
//! assert_eq!(results, [
//! 	"A",
//! 	"B",
//! 	"C",
//! ]);
//! ```
//!
//! To use this, add it as a dependency to your Cargo.toml:
//! ```toml
//! [dependencies]
//! list_files_macro = "^0.1.0"
//! ```

#![feature(proc_macro_span)]

#![doc(html_root_url = "https://docs.rs/list_files_macro/0.1.0")]

use glob::glob;
use proc_macro::{TokenStream, Span};
use syn::{Expr, punctuated::Punctuated, Token, parse::Parser, ExprLit, Lit, ExprPath};
use quote::quote;

#[proc_macro]
pub fn list_files(input: TokenStream) -> TokenStream {
	// Parse two arguments, a handler macro and a directory to process
	let parser = Punctuated::<Expr, Token![,]>::parse_separated_nonempty;
	let args = parser.parse(input).unwrap().into_iter().collect::<Vec<_>>();
	let (path, handler) = match &args[..] {
		[
			Expr::Path(ExprPath { path: handler, .. }),
			Expr::Lit(ExprLit { lit: Lit::Str(path), .. }),
		] => (path.value(), Some(handler)),
		[Expr::Lit(ExprLit { lit: Lit::Str(path), .. })] => (path.value(), None),
		_ => panic!("Usage: list_files!(\"path/to/dir\") | list_files!(handler_macro, \"path/to/dir\")"),
	};

	// Resolve directory
	let absolute_path =
		if path.starts_with(".") {
			let source_path: std::path::PathBuf = Span::call_site().file().into();
			source_path.parent().unwrap().join(&path).into_os_string().into_string().unwrap()
		} else {
			path
		};

	// Generate list of literal string paths
	let files = glob(&absolute_path).unwrap();
	let file_literals = files.map(|file| {
		let file = file.unwrap().canonicalize().unwrap().into_os_string().into_string().unwrap();
		quote!(#file)
	});

	// Run handlers if required
	if let Some(handler) = handler {
		quote!([#(#handler!(#file_literals)),*])
	} else {
		quote!([#(#file_literals),*])
	}.into()
}

// Include the readme and changelog as hidden documentation so they're tested by cargo test
#[doc = include_str!("../README.md")]
#[doc = include_str!("../CHANGELOG.md")]
type _Doctest = ();
