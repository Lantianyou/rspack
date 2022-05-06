use std::sync::{
  atomic::{AtomicUsize, Ordering},
  Arc,
};

use dashmap::{DashMap, DashSet};
use smol_str::SmolStr;
use swc_atoms::JsWord;
use swc_ecma_visit::VisitMutWith;
use tokio::sync::mpsc::UnboundedSender;
use tracing::instrument;

use crate::{
  bundle::Msg, dependency_scanner::DependencyScanner, plugin_hook, utils::parse_file, JsModule,
  PluginDriver, ResolvedId,
};

#[derive(Debug)]
pub struct DependencyIdResolver {
  pub module_id: SmolStr,
  pub resolved_ids: DashMap<JsWord, ResolvedId>,
  pub plugin_driver: Arc<PluginDriver>,
}

impl DependencyIdResolver {
  pub async fn resolve_id(&self, dep_src: &JsWord) -> ResolvedId {
    let resolved_id;
    if let Some(cached) = self.resolved_ids.get(dep_src) {
      resolved_id = cached.clone();
    } else {
      resolved_id =
        plugin_hook::resolve_id(dep_src, Some(&self.module_id), false, &self.plugin_driver).await;
      self
        .resolved_ids
        .insert(dep_src.clone(), resolved_id.clone());
    }
    resolved_id
  }
}

#[derive(Debug)]
pub struct Task {
  pub resolved_id: ResolvedId,
  pub active_task_count: Arc<AtomicUsize>,
  pub tx: UnboundedSender<Msg>,
  pub visited_module_id: Arc<DashSet<SmolStr>>,
  pub plugin_driver: Arc<PluginDriver>,
}

impl Task {
  #[instrument(skip(self))]
  pub async fn run(&mut self) {
    let resolved_id = self.resolved_id.clone();
    if resolved_id.external {
      // TODO: external module
    } else {
      tracing::debug!("start process {:?}", resolved_id);
      let id_resolver = DependencyIdResolver {
        module_id: resolved_id.id.clone(),
        resolved_ids: Default::default(),
        plugin_driver: self.plugin_driver.clone(),
      };

      let module_id: &str = &resolved_id.id;
      let source = plugin_hook::load(module_id, &self.plugin_driver).await;
      let mut dependency_scanner = DependencyScanner::default();

      let raw_ast = parse_file(source, module_id).expect_module();
      let mut ast = plugin_hook::transform(raw_ast, &self.plugin_driver);

      self.pre_analyze_imported_module(&id_resolver, &ast).await;

      ast.visit_mut_with(&mut dependency_scanner);

      for dyn_import in &dependency_scanner.dyn_dependencies {
        let resolved_id = id_resolver.resolve_id(&dyn_import.argument).await;

        self.spawn_new_task(resolved_id);
      }
      for (import, _) in &dependency_scanner.dependencies {
        let resolved_id = id_resolver.resolve_id(import).await;
        self.spawn_new_task(resolved_id);
      }
      let module = JsModule {
        exec_order: Default::default(),
        id: resolved_id.id.clone(),
        ast,
        dependencies: dependency_scanner.dependencies,
        dyn_imports: dependency_scanner.dyn_dependencies,
        is_user_defined_entry_point: Default::default(),
        resolved_ids: id_resolver
          .resolved_ids
          .into_iter()
          .map(|(key, value)| (key, value))
          .collect(),
      };

      self.tx.send(Msg::TaskFinished(module)).unwrap()
    }
  }

  pub fn spawn_new_task(&self, id: ResolvedId) {
    if !self.visited_module_id.contains(&id.id) {
      self.visited_module_id.insert(id.id.clone());
      self.active_task_count.fetch_add(1, Ordering::SeqCst);
      let mut task = Task {
        resolved_id: id,
        active_task_count: self.active_task_count.clone(),
        visited_module_id: self.visited_module_id.clone(),
        tx: self.tx.clone(),
        plugin_driver: self.plugin_driver.clone(),
      };
      tokio::task::spawn(async move {
        task.run().await;
      });
    }
  }

  // Fast path for analyzing static import and export.
  pub async fn pre_analyze_imported_module(
    &self,
    resolver: &DependencyIdResolver,
    ast: &ast::Module,
  ) {
    for module_item in &ast.body {
      if let ast::ModuleItem::ModuleDecl(module_decl) = module_item {
        let mut depended = None;
        match module_decl {
          ast::ModuleDecl::Import(import_decl) => {
            depended = Some(&import_decl.src.value);
          }
          ast::ModuleDecl::ExportNamed(node) => {
            if let Some(source_node) = &node.src {
              depended = Some(&source_node.value);
            }
          }
          ast::ModuleDecl::ExportAll(node) => {
            depended = Some(&node.src.value);
          }
          _ => {}
        }
        if let Some(depended) = depended {
          let resolved_id = resolver.resolve_id(depended).await;
          self.spawn_new_task(resolved_id);
        }
      }
    }
  }
}