//! Name resolution for Nevermind AST

use std::collections::HashMap;
use std::path::PathBuf;

use nevermind_ast::{Expr, Pattern, Stmt};

use crate::error::{NameError, Result};
use crate::symbol::Symbol;
use crate::symbol_table::SymbolTable;

#[derive(Debug, Clone)]
struct ModuleExports {
    path: PathBuf,
    symbols: HashMap<String, Symbol>,
}

/// The name resolver
pub struct NameResolver {
    /// The symbol table
    symbol_table: SymbolTable,

    /// Collected errors
    errors: Vec<NameError>,

    /// Base directory used to locate imported `.nm` files.
    /// `None` in REPL / in-memory compilation contexts.
    base_dir: Option<PathBuf>,

    /// Cached exports for local modules that have already been loaded.
    module_exports: HashMap<String, ModuleExports>,
}

impl NameResolver {
    /// Create a new name resolver with no file-system context.
    pub fn new() -> Self {
        let mut resolver = Self {
            symbol_table: SymbolTable::new(),
            errors: Vec::new(),
            base_dir: None,
            module_exports: HashMap::new(),
        };
        resolver.register_builtins();
        resolver
    }

    /// Create a name resolver that can locate imported modules on disk.
    ///
    /// `base_dir` should be the directory containing the source file being
    /// compiled (e.g. `input.parent()`).
    pub fn with_base_dir(base_dir: PathBuf) -> Self {
        let mut resolver = Self {
            symbol_table: SymbolTable::new(),
            errors: Vec::new(),
            base_dir: Some(base_dir),
            module_exports: HashMap::new(),
        };
        resolver.register_builtins();
        resolver
    }

    /// Register built-in functions in the global scope
    fn register_builtins(&mut self) {
        let builtins = [
            ("print", 1),
            ("println", 1),
            ("len", 1),
            ("str", 1),
            ("int", 1),
            ("float", 1),
            ("bool", 1),
            ("type", 1),
            ("input", 1),
            ("range", 2),
            ("abs", 1),
            ("min", 2),
            ("max", 2),
        ];
        for (name, param_count) in builtins {
            let symbol = Symbol::function(
                name.to_string(),
                param_count,
                nevermind_common::Span::dummy(),
            );
            let _ = self.symbol_table.declare(name.to_string(), symbol);
        }
    }

    /// Resolve a list of statements
    pub fn resolve(&mut self, stmts: &[Stmt]) -> Result<(), Vec<NameError>> {
        for stmt in stmts {
            if let Err(err) = self.resolve_statement(stmt) {
                self.errors.push(err);
            }
        }

        // After resolving all statements, check for undefined variables
        self.check_undefined();

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    /// Resolve a statement
    fn resolve_statement(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Export { stmt, .. } => self.resolve_statement(stmt),
            Stmt::Let {
                name,
                is_mutable,
                value,
                span,
                ..
            } => {
                // First resolve the initializer expression
                self.resolve_expression(value)?;

                // Then declare the variable
                let symbol = Symbol::variable(name.clone(), *is_mutable, span.clone());
                self.symbol_table.declare(name.clone(), symbol)?;
                Ok(())
            }

            Stmt::Function {
                name,
                params,
                body,
                span,
                ..
            } => {
                // Declare the function in the current scope
                let func_symbol = Symbol::function(name.clone(), params.len(), span.clone());
                self.symbol_table.declare(name.clone(), func_symbol)?;

                // Enter a new function scope
                self.symbol_table.enter_function();

                // Declare parameters
                for (i, param) in params.iter().enumerate() {
                    let param_symbol =
                        Symbol::parameter(param.name.clone(), i, nevermind_common::Span::dummy());
                    self.symbol_table
                        .declare(param.name.clone(), param_symbol)?;
                }

                // Resolve the function body
                self.resolve_expression(body)?;

                // Exit function scope
                self.symbol_table.exit_function()?;
                Ok(())
            }

            Stmt::TypeAlias { name, span, .. } => {
                // Declare the type
                let type_symbol = Symbol::type_(name.clone(), span.clone());
                self.symbol_table.declare(name.clone(), type_symbol)
            }

            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                // Resolve condition
                self.resolve_expression(condition)?;

                // Resolve then branch
                self.symbol_table.enter_scope();
                for stmt in then_branch {
                    self.resolve_statement(stmt)?;
                }
                self.symbol_table.exit_scope()?;

                // Resolve else branch if present
                if let Some(else_branch) = else_branch {
                    self.symbol_table.enter_scope();
                    for stmt in else_branch {
                        self.resolve_statement(stmt)?;
                    }
                    self.symbol_table.exit_scope()?;
                }

                Ok(())
            }

            Stmt::While {
                condition, body, ..
            } => {
                // Resolve condition
                self.resolve_expression(condition)?;

                // Resolve body in a loop scope
                self.symbol_table.enter_loop();
                for stmt in body {
                    self.resolve_statement(stmt)?;
                }
                self.symbol_table.exit_loop()?;
                Ok(())
            }

            Stmt::For {
                variable,
                iter,
                body,
                ..
            } => {
                // Resolve iterator
                self.resolve_expression(iter)?;

                // Enter loop scope
                self.symbol_table.enter_loop();

                // Declare loop variable (resolve_pattern already declares Variable patterns,
                // so only declare explicitly for non-variable patterns like Tuple/Wildcard)
                match variable {
                    Pattern::Variable { .. } => {
                        // resolve_pattern will declare it
                        self.resolve_pattern(variable)?;
                    }
                    _ => {
                        self.resolve_pattern(variable)?;
                        let name = self.pattern_name(variable)?;
                        if name != "_" && name != "<pattern>" {
                            self.symbol_table.declare(
                                name.clone(),
                                Symbol::loop_variable(name, nevermind_common::Span::dummy()),
                            )?;
                        }
                    }
                }

                // Resolve body
                for stmt in body {
                    self.resolve_statement(stmt)?;
                }

                self.symbol_table.exit_loop()?;
                Ok(())
            }

            Stmt::Match {
                scrutinee, arms, ..
            } => {
                // Resolve scrutinee
                self.resolve_expression(scrutinee)?;

                // Resolve each arm
                for arm in arms {
                    // Each arm creates a new scope for the pattern
                    self.symbol_table.enter_scope();
                    self.resolve_pattern(&arm.pattern)?;

                    // Resolve guard if present
                    if let Some(guard) = &arm.guard {
                        self.resolve_expression(guard)?;
                    }

                    // Resolve body
                    self.resolve_expression(&arm.body)?;

                    self.symbol_table.exit_scope()?;
                }

                Ok(())
            }

            Stmt::Return { value, span, .. } => {
                if !self.symbol_table.in_function() {
                    return Err(NameError::invalid_return(span.clone()));
                }

                if let Some(value) = value {
                    self.resolve_expression(value)?;
                }

                Ok(())
            }

            Stmt::Break { span, .. } => {
                if !self.symbol_table.in_loop() {
                    return Err(NameError::invalid_break(span.clone()));
                }
                Ok(())
            }

            Stmt::Continue { span, .. } => {
                if !self.symbol_table.in_loop() {
                    return Err(NameError::invalid_continue(span.clone()));
                }
                Ok(())
            }

            Stmt::ExprStmt { expr, .. } => self.resolve_expression(expr),

            Stmt::Import {
                module,
                symbols,
                span,
                ..
            } => self.resolve_import(module, symbols.as_deref(), span),

            Stmt::Class { name, members, .. } => {
                // Declare the class as a type
                let class_symbol = Symbol::type_(name.clone(), nevermind_common::Span::dummy());
                self.symbol_table.declare(name.clone(), class_symbol)?;

                // Enter class scope
                self.symbol_table.enter_scope();

                // Resolve members
                for member in members {
                    match member {
                        nevermind_ast::stmt::ClassMember::Field { name, .. } => {
                            let field_symbol = Symbol::variable(
                                name.clone(),
                                false,
                                nevermind_common::Span::dummy(),
                            );
                            self.symbol_table.declare(name.clone(), field_symbol)?;
                        }
                        nevermind_ast::stmt::ClassMember::Method {
                            name, params, body, ..
                        } => {
                            let method_symbol = Symbol::function(
                                name.clone(),
                                params.len(),
                                nevermind_common::Span::dummy(),
                            );
                            self.symbol_table.declare(name.clone(), method_symbol)?;

                            // Resolve method body
                            self.symbol_table.enter_function();

                            // Declare parameters
                            for (i, param) in params.iter().enumerate() {
                                let param_symbol = Symbol::parameter(
                                    param.name.clone(),
                                    i,
                                    nevermind_common::Span::dummy(),
                                );
                                self.symbol_table
                                    .declare(param.name.clone(), param_symbol)?;
                            }

                            self.resolve_expression(body)?;
                            self.symbol_table.exit_function()?;
                        }
                    }
                }

                self.symbol_table.exit_scope()?;
                Ok(())
            }
        }
    }

    // -----------------------------------------------------------------------
    // Module system helpers
    // -----------------------------------------------------------------------

    /// Resolve an import statement.
    ///
    /// Two forms are supported:
    /// - `from "module" import sym1, sym2` — selective import
    /// - `use "module"`                    — namespace import
    fn resolve_import(
        &mut self,
        module: &str,
        symbols: Option<&[String]>,
        span: &nevermind_common::Span,
    ) -> Result<()> {
        let local_module = self.load_module_exports(module, span)?;

        match symbols {
            Some(syms) => {
                if let Some(local_module) = local_module.as_ref() {
                    for sym_name in syms {
                        let export = match local_module.symbols.get(sym_name).cloned() {
                            Some(export) => export,
                            None => {
                                let mut error = NameError::undefined_import(
                                    module.to_string(),
                                    sym_name.clone(),
                                    span.clone(),
                                )
                                .with_context(
                                    format!("local module resolved to {}", local_module.path.display()),
                                    None,
                                );

                                if !local_module.symbols.is_empty() {
                                    error = error.with_context(
                                        format!(
                                            "available exports: {}",
                                            Self::format_export_list(&local_module.symbols)
                                        ),
                                        None,
                                    );
                                }

                                return Err(error);
                            }
                        };

                        self.declare_imported_symbol(sym_name, &export, span)?;
                    }
                } else {
                    for sym_name in syms {
                        if !self.symbol_table.is_defined(sym_name) {
                            let sym = Symbol::variable(sym_name.clone(), false, span.clone());
                            self.symbol_table.declare(sym_name.clone(), sym)?;
                        }
                    }
                }
            }
            None => {
                let namespace = module.split('/').next_back().unwrap_or(module);
                if !self.symbol_table.is_defined(namespace) {
                    let sym = Symbol::variable(namespace.to_string(), false, span.clone());
                    self.symbol_table.declare(namespace.to_string(), sym)?;
                }
            }
        }

        Ok(())
    }

    fn declare_imported_symbol(
        &mut self,
        import_name: &str,
        export: &Symbol,
        span: &nevermind_common::Span,
    ) -> Result<()> {
        let imported = Symbol {
            name: import_name.to_string(),
            kind: export.kind.clone(),
            span: span.clone(),
            type_: export.type_.clone(),
        };
        self.symbol_table.declare(import_name.to_string(), imported)
    }

    fn format_export_list(symbols: &HashMap<String, Symbol>) -> String {
        let mut names: Vec<&str> = symbols.keys().map(String::as_str).collect();
        names.sort_unstable();
        names.join(", ")
    }

    fn export_symbol(stmt: &Stmt) -> Option<(String, Symbol)> {
        match stmt {
            Stmt::Export { stmt, .. } => match stmt.as_ref() {
                Stmt::Function {
                    name, params, span, ..
                } => Some((
                    name.clone(),
                    Symbol::function(name.clone(), params.len(), span.clone()),
                )),
                Stmt::Let {
                    name,
                    is_mutable,
                    span,
                    ..
                } => Some((
                    name.clone(),
                    Symbol::variable(name.clone(), *is_mutable, span.clone()),
                )),
                Stmt::TypeAlias { name, span, .. } | Stmt::Class { name, span, .. } => {
                    Some((name.clone(), Symbol::type_(name.clone(), span.clone())))
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn module_file_path(base_dir: &std::path::Path, module: &str) -> PathBuf {
        module
            .split('/')
            .fold(base_dir.to_path_buf(), |path, segment| path.join(segment))
            .with_extension("nm")
    }

    /// Load and parse a `.nm` module file, returning its top-level exports.
    ///
    /// When no local module file exists, this returns `Ok(None)` so external
    /// Python modules can still flow through to runtime unchanged.
    fn load_module_exports(
        &mut self,
        module: &str,
        import_span: &nevermind_common::Span,
    ) -> Result<Option<ModuleExports>> {
        if let Some(exports) = self.module_exports.get(module) {
            return Ok(Some(exports.clone()));
        }

        let base_dir = match self.base_dir.clone() {
            Some(dir) => dir,
            None => return Ok(None),
        };

        let file_path = Self::module_file_path(&base_dir, module);
        if !file_path.exists() {
            return Ok(None);
        }

        let source = std::fs::read_to_string(&file_path).map_err(|error| {
            NameError::module_load_failed(
                module.to_string(),
                format!("could not read {}", file_path.display()),
                import_span.clone(),
            )
            .with_context(error.to_string(), None)
        })?;

        let mut lexer = nevermind_lexer::Lexer::from_file(&source, file_path.clone());
        let tokens = lexer.tokenize().map_err(|error| {
            NameError::module_load_failed(module.to_string(), error.message.clone(), error.span.clone())
                .with_context(
                    format!("imported here from '{}'", module),
                    Some(import_span.clone()),
                )
                .with_context(format!("while reading {}", file_path.display()), None)
        })?;

        let mut parser = nevermind_parser::Parser::from_tokens(tokens);
        let stmts = parser.parse().map_err(|error| {
            NameError::module_load_failed(module.to_string(), error.message.clone(), error.span.clone())
                .with_context(
                    format!("imported here from '{}'", module),
                    Some(import_span.clone()),
                )
                .with_context(format!("while parsing {}", file_path.display()), None)
        })?;

        let mut symbols = HashMap::new();
        for stmt in &stmts {
            if let Some((name, symbol)) = Self::export_symbol(stmt) {
                symbols.insert(name, symbol);
            }
        }

        let exports = ModuleExports {
            path: file_path,
            symbols,
        };

        self.module_exports
            .insert(module.to_string(), exports.clone());

        Ok(Some(exports))
    }

    /// Resolve an expression
    fn resolve_expression(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Literal(_) => Ok(()),

            Expr::Variable { name, .. } => {
                self.symbol_table.resolve(name)?;
                Ok(())
            }

            Expr::Binary { left, right, .. } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
                Ok(())
            }

            Expr::Comparison { left, right, .. } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
                Ok(())
            }

            Expr::Logical { left, right, .. } => {
                self.resolve_expression(left)?;
                self.resolve_expression(right)?;
                Ok(())
            }

            Expr::Unary { expr, .. } => self.resolve_expression(expr),

            Expr::Call { callee, args, .. } => {
                self.resolve_expression(callee)?;

                for arg in args {
                    self.resolve_expression(arg)?;
                }

                Ok(())
            }

            Expr::Pipeline { stages, .. } => {
                for stage in stages {
                    self.resolve_expression(stage)?;
                }
                Ok(())
            }

            Expr::Lambda { params, body, .. } => {
                // Enter a new scope for the lambda
                self.symbol_table.enter_scope();

                // Declare parameters
                for (i, param) in params.iter().enumerate() {
                    let param_symbol =
                        Symbol::parameter(param.name.clone(), i, nevermind_common::Span::dummy());
                    self.symbol_table
                        .declare(param.name.clone(), param_symbol)?;
                }

                // Resolve body
                self.resolve_expression(body)?;

                // Exit scope
                self.symbol_table.exit_scope()?;
                Ok(())
            }

            Expr::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.resolve_expression(condition)?;
                self.resolve_expression(then_branch)?;
                self.resolve_expression(else_branch)?;
                Ok(())
            }

            Expr::Block { statements, .. } => {
                self.symbol_table.enter_scope();

                for stmt in statements {
                    self.resolve_statement(stmt)?;
                }

                self.symbol_table.exit_scope()?;
                Ok(())
            }

            Expr::List { elements, .. } => {
                for elem in elements {
                    self.resolve_expression(elem)?;
                }
                Ok(())
            }

            Expr::Map { entries, .. } => {
                for (key, value) in entries {
                    self.resolve_expression(key)?;
                    self.resolve_expression(value)?;
                }
                Ok(())
            }

            Expr::Match {
                scrutinee, arms, ..
            } => {
                self.resolve_expression(scrutinee)?;

                for arm in arms {
                    self.symbol_table.enter_scope();
                    self.resolve_pattern(&arm.pattern)?;

                    if let Some(guard) = &arm.guard {
                        self.resolve_expression(guard)?;
                    }

                    self.resolve_expression(&arm.body)?;
                    self.symbol_table.exit_scope()?;
                }

                Ok(())
            }

            Expr::Index { array, index, .. } => {
                self.resolve_expression(array)?;
                self.resolve_expression(index)
            }

            Expr::Assign { target, value, .. } => {
                self.resolve_expression(target)?;
                self.resolve_expression(value)
            }

            Expr::MemberAccess { object, .. } => self.resolve_expression(object),
        }
    }

    /// Resolve a pattern
    fn resolve_pattern(&mut self, pattern: &Pattern) -> Result<()> {
        match pattern {
            Pattern::Literal { .. } => Ok(()),
            Pattern::Variable { name, .. } => {
                // Declare pattern variable
                let symbol = Symbol::variable(name.clone(), false, nevermind_common::Span::dummy());
                self.symbol_table.declare(name.clone(), symbol)
            }
            Pattern::Wildcard { .. } => Ok(()),
            Pattern::Tuple { patterns, .. } => {
                for pat in patterns {
                    self.resolve_pattern(pat)?;
                }
                Ok(())
            }
            Pattern::List { patterns, .. } => {
                for pat in patterns {
                    self.resolve_pattern(pat)?;
                }
                Ok(())
            }
            Pattern::ListCons { head, tail, .. } => {
                self.resolve_pattern(head)?;
                self.resolve_pattern(tail)?;
                Ok(())
            }
            Pattern::Struct { fields, .. } => {
                for field in fields {
                    self.resolve_pattern(&field.pattern)?;
                }
                Ok(())
            }
            Pattern::Or { patterns, .. } => {
                for pat in patterns {
                    self.resolve_pattern(pat)?;
                }
                Ok(())
            }
            Pattern::Range { start, end, .. } => {
                self.resolve_pattern(start)?;
                self.resolve_pattern(end)?;
                Ok(())
            }
            Pattern::Constructor { args, .. } => {
                for arg in args {
                    self.resolve_pattern(arg)?;
                }
                Ok(())
            }
        }
    }

    /// Extract a name from a pattern (for error messages and scoping)
    fn pattern_name(&self, pattern: &Pattern) -> Result<String> {
        match pattern {
            Pattern::Variable { name, .. } => Ok(name.clone()),
            Pattern::Wildcard { .. } => Ok("_".to_string()),
            _ => Ok("<pattern>".to_string()),
        }
    }

    /// Check for undefined variables (this is called after all resolution is done)
    fn check_undefined(&mut self) {
        // This is a placeholder for additional validation
        // In practice, undefined variables are caught during resolve()
    }
}

impl Default for NameResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;
    use nevermind_ast::Literal;
    use nevermind_ast::{Expr, Parameter, Stmt};

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new(prefix: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!("{}_{}", prefix, unique));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn parse_statements(source: &str) -> Vec<Stmt> {
        let mut parser = nevermind_parser::Parser::new(source).unwrap();
        parser.parse().unwrap()
    }

    #[test]
    fn test_resolve_variable() {
        let mut resolver = NameResolver::new();

        // Create a simple variable declaration
        let stmt = Stmt::Let {
            id: 1,
            is_mutable: false,
            name: "x".to_string(),
            type_annotation: None,
            value: Expr::Literal(Literal::Integer(42, nevermind_common::Span::dummy())),
            span: nevermind_common::Span::dummy(),
        };

        resolver.resolve_statement(&stmt).unwrap();

        // Variable should be defined
        assert!(resolver.symbol_table.is_defined("x"));
    }

    #[test]
    fn test_resolve_undefined_variable() {
        let mut resolver = NameResolver::new();

        // Create a variable reference without declaration
        let expr = Expr::Variable {
            id: 1,
            name: "undefined_var".to_string(),
            span: nevermind_common::Span::dummy(),
        };

        let result = resolver.resolve_expression(&expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_function() {
        let mut resolver = NameResolver::new();

        // Create a simple function declaration
        let stmt = Stmt::Function {
            id: 1,
            name: "add".to_string(),
            params: vec![
                Parameter {
                    id: 2,
                    name: "a".to_string(),
                    type_annotation: None,
                    default_value: None,
                },
                Parameter {
                    id: 3,
                    name: "b".to_string(),
                    type_annotation: None,
                    default_value: None,
                },
            ],
            return_type: None,
            body: Expr::Literal(Literal::Integer(0, nevermind_common::Span::dummy())),
            span: nevermind_common::Span::dummy(),
        };

        resolver.resolve_statement(&stmt).unwrap();

        // Function should be defined
        assert!(resolver.symbol_table.is_defined("add"));
    }

    #[test]
    fn test_nested_scopes() {
        let mut resolver = NameResolver::new();

        // Declare variable in outer scope
        let stmt1 = Stmt::Let {
            id: 1,
            is_mutable: false,
            name: "x".to_string(),
            type_annotation: None,
            value: Expr::Literal(Literal::Integer(10, nevermind_common::Span::dummy())),
            span: nevermind_common::Span::dummy(),
        };

        resolver.resolve_statement(&stmt1).unwrap();

        // Create a block with inner scope
        let block_expr = Expr::Block {
            id: 2,
            statements: vec![Stmt::Let {
                id: 3,
                is_mutable: false,
                name: "y".to_string(),
                type_annotation: None,
                value: Expr::Literal(Literal::Integer(20, nevermind_common::Span::dummy())),
                span: nevermind_common::Span::dummy(),
            }],
            span: nevermind_common::Span::dummy(),
        };

        resolver.resolve_expression(&block_expr).unwrap();

        // Outer variable should still be defined
        assert!(resolver.symbol_table.is_defined("x"));
    }

    #[test]
    fn test_selective_import_only_declares_requested_names() {
        let temp_dir = TestDir::new("nevermind_name_resolver_selective_import");
        fs::write(
            temp_dir.path.join("mathutils.nm"),
            "export fn square(n) do n end\nexport fn cube(n) do n end\n",
        )
        .unwrap();

        let statements = parse_statements("from \"mathutils\" import square\n");
        let mut resolver = NameResolver::with_base_dir(temp_dir.path.clone());

        resolver.resolve(&statements).unwrap();

        assert!(resolver.symbol_table.is_defined("square"));
        assert!(!resolver.symbol_table.is_defined("cube"));
    }

    #[test]
    fn test_missing_local_import_reports_error() {
        let temp_dir = TestDir::new("nevermind_name_resolver_missing_import");
        fs::write(
            temp_dir.path.join("mathutils.nm"),
            "export fn square(n) do n end\n",
        )
        .unwrap();

        let statements = parse_statements("from \"mathutils\" import cube\n");
        let mut resolver = NameResolver::with_base_dir(temp_dir.path.clone());
        let errors = resolver.resolve(&statements).unwrap_err();

        assert!(errors
            .iter()
            .any(|error| error.message.contains("does not export 'cube'")));
    }

    #[test]
    fn test_broken_local_module_reports_error() {
        let temp_dir = TestDir::new("nevermind_name_resolver_broken_module");
        fs::write(temp_dir.path.join("broken.nm"), "fn broken( do\nend\n").unwrap();

        let statements = parse_statements("use \"broken\"\n");
        let mut resolver = NameResolver::with_base_dir(temp_dir.path.clone());
        let errors = resolver.resolve(&statements).unwrap_err();

        assert!(errors
            .iter()
            .any(|error| error.message.contains("Failed to load local module 'broken'")));
    }

    #[test]
    fn test_unexported_local_symbol_reports_error() {
        let temp_dir = TestDir::new("nevermind_name_resolver_unexported_import");
        fs::write(
            temp_dir.path.join("mathutils.nm"),
            "export fn square(n) do n end\nfn helper(n) do n end\n",
        )
        .unwrap();

        let statements = parse_statements("from \"mathutils\" import helper\n");
        let mut resolver = NameResolver::with_base_dir(temp_dir.path.clone());
        let errors = resolver.resolve(&statements).unwrap_err();

        assert!(errors
            .iter()
            .any(|error| error.message.contains("does not export 'helper'")));
    }

    #[test]
    fn test_missing_local_module_falls_back_to_external_python_imports() {
        let temp_dir = TestDir::new("nevermind_name_resolver_external_import");
        let statements = parse_statements(
            "from \"json\" import dumps\nuse \"collections\"\n",
        );
        let mut resolver = NameResolver::with_base_dir(temp_dir.path.clone());

        resolver.resolve(&statements).unwrap();

        assert!(resolver.symbol_table.is_defined("dumps"));
        assert!(resolver.symbol_table.is_defined("collections"));
    }
}
