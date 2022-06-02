#![deny(clippy::all)]
#![feature(box_patterns)]
#![feature(iter_intersperse)]

mod bundle;
mod bundle_context;
mod chunk;
mod dependency_scanner;
pub mod finalize;
mod js_module;
mod module_graph;
mod options;
mod plugin;
mod plugin_driver;
mod result;
mod runtime;
mod task;
mod utils;
pub use bundle::*;
pub use bundle_context::*;
pub use chunk::*;
pub use js_module::*;
pub use module_graph::*;
pub use options::*;
pub use plugin::*;
pub use plugin_driver::*;
pub use result::*;
pub use runtime::*;

pub use rspack_swc::swc_ecma_ast as ast;

pub use utils::*;

mod chunk_graph;
pub use chunk_graph::*;

mod visitors;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ResolvedURI {
  pub uri: String,
  pub kind: ImportKind,
  pub external: bool,
  pub ignored: bool,
}

impl ResolvedURI {
  pub fn new<T: Into<String>>(path: T, external: bool, kind: ImportKind, ignored: bool) -> Self {
    Self {
      uri: path.into(),
      external,
      kind, // module_side_effects: false,
      ignored,
    }
  }
}
