use ast::{CallExpr, Callee, Expr, ExprOrSpread, Ident, Lit};
use rspack_swc::{swc_atoms, swc_ecma_ast as ast};
use swc_atoms::js_word;

pub fn is_dynamic_import(e: &mut CallExpr) -> bool {
  matches!(e.callee, Callee::Import(_))
}
pub fn dynamic_import_with_literal(e: &mut CallExpr) -> Option<String> {
  if is_dynamic_import(e) {
    if let Some(ExprOrSpread {
      spread: None,
      expr: box Expr::Lit(Lit::Str(str)),
    }) = e.args.first()
    {
      return Some(str.value.to_string());
    }
  }
  return None;
}
pub fn is_require(e: &mut CallExpr) -> bool {
  matches!(
    e.callee,
    Callee::Expr(box Expr::Ident(Ident {
      sym: js_word!("require"),
      ..
    }))
  )
}

#[cfg(test)]
mod swc_builder_test {
  use crate::{
    swc_builder::{dynamic_import_with_literal, is_dynamic_import, is_require},
    test_runner::compile,
  };
  use ast::{CallExpr, Ident, Lit};
  use rspack_swc::{
    swc_common, swc_ecma_ast as ast, swc_ecma_transforms_base, swc_ecma_transforms_react,
    swc_ecma_utils, swc_ecma_visit,
  };
  use swc_common::{chain, comments::SingleThreadedComments, Globals, Mark, DUMMY_SP};
  use swc_ecma_transforms_base::resolver::resolver_with_mark;
  use swc_ecma_transforms_react as swc_react;
  use swc_ecma_utils::ExprFactory;
  use swc_ecma_visit::{FoldWith, VisitMut, VisitMutWith};
  use swc_react::{RefreshOptions, Runtime};
  #[test]
  fn react_fresh() {
    let globals = Globals::new();
    swc_common::GLOBALS.set(&globals, || {
      let (mut ast, code, compiler) = compile(
        r#"
      import React from 'react';
      export const App = () => {
        return <div>app</div>
      }
    "#
        .into(),
        None,
      );
      let top_level_mark = Mark::fresh(Mark::root());
      let mut react_folder = swc_react::react::<SingleThreadedComments>(
        compiler.cm.clone(),
        None,
        swc_react::Options {
          runtime: Some(Runtime::Automatic),
          refresh: Some(RefreshOptions {
            ..Default::default()
          }),
          development: true,
          ..Default::default()
        },
        top_level_mark,
      );
      let mut folds = chain!(resolver_with_mark(top_level_mark), &mut react_folder);
      let ast = ast.fold_with(&mut folds);
      let (_, code, _) = compile(Default::default(), Some(ast));
      dbg!(code);
    });
  }

  #[test]
  fn dynamic_require() {
    let (mut ast, ..) = compile(
      r#"
      const x = import('./a');
      const y = require('./b');
    "#
      .into(),
      None,
    );
    #[derive(Debug)]
    struct CheckVisitor {
      dynamic_called: usize,
      require_called: usize,
      string_literal_called: usize,
    }
    impl CheckVisitor {
      fn new() -> Self {
        Self {
          dynamic_called: 0,
          require_called: 0,
          string_literal_called: 0,
        }
      }
    }

    impl VisitMut for CheckVisitor {
      fn visit_mut_call_expr(&mut self, node: &mut CallExpr) {
        if is_dynamic_import(node) {
          self.dynamic_called += 1;
        }
        if is_require(node) {
          self.require_called += 1;
        }
        if let Some(_) = dynamic_import_with_literal(node) {
          self.string_literal_called += 1;
        }
      }
    }

    struct TransformVisitor {}
    impl TransformVisitor {
      fn new() -> TransformVisitor {
        TransformVisitor {}
      }
    }
    impl VisitMut for TransformVisitor {
      fn visit_mut_call_expr(&mut self, node: &mut CallExpr) {
        if let Some(str) = dynamic_import_with_literal(node) {
          let callee = Ident::new("require".into(), DUMMY_SP).as_callee();
          let arg = Lit::Str(str.into()).as_arg();
          node.callee = callee;
          node.args = vec![arg];
        }
      }
    }
    let mut check_visitor = CheckVisitor::new();
    let mut transform_visitor = TransformVisitor::new();
    ast.visit_mut_with(&mut check_visitor);

    assert_eq!(check_visitor.dynamic_called, 1);
    assert_eq!(check_visitor.require_called, 1);
    assert_eq!(check_visitor.string_literal_called, 1);
    ast.visit_mut_with(&mut transform_visitor);
    let (_, code, _) = compile(Default::default(), Some(ast));
    assert_eq!(
      code.code,
      "const x = require(\"./a\");\nconst y = require('./b');\n"
    );
  }
}