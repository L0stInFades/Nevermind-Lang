//! Python code generator

use super::{BytecodeChunk, CodeEmitter};
use super::emit::Result;
use nevermind_mir::{MirProgram, MirFunction, MirExpr, MirExprStmt, MirStmt, BinOp, UnaryOp, Literal};

/// Python code generator
pub struct PythonGenerator {
    pub indent_level: usize,
}

impl PythonGenerator {
    pub fn new() -> Self {
        Self {
            indent_level: 0,
        }
    }

    pub fn generate(&self, program: &MirProgram) -> Result<String> {
        let mut generator = PythonGenerator::new();
        let chunk = generator.emit_program(program)?;
        Ok(chunk.code)
    }

    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }

    fn output_line(&self, output: &mut BytecodeChunk, text: &str) {
        output.add_line(&format!("{}{}", self.indent(), text));
    }

    fn emit_literal(&self, literal: &Literal) -> String {
        match literal {
            Literal::Int(v) => v.to_string(),
            Literal::Float(v) => v.to_string(),
            Literal::String(v) => {
                // Check for string interpolation: {expr}
                if v.contains('{') && v.contains('}') {
                    format!("f\"{}\"", escape_string(v))
                } else {
                    format!("\"{}\"", escape_string(v))
                }
            }
            Literal::Bool(v) => {
                if *v { "True" } else { "False" }.to_string()
            }
            Literal::Null => "None".to_string(),
        }
    }

    fn map_binop(&self, op: BinOp) -> &'static str {
        match op {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Mod => "%",
            BinOp::Pow => "**",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::And => "and",
            BinOp::Or => "or",
        }
    }

    fn map_unop(&self, op: UnaryOp) -> &'static str {
        match op {
            UnaryOp::Neg => "-",
            UnaryOp::Not => "not ",
        }
    }

    /// Emit a list of MirExprStmt with proper indentation
    fn emit_expr_stmt_list(&mut self, stmts: &[MirExprStmt], output: &mut BytecodeChunk) -> Result<()> {
        if stmts.is_empty() {
            self.output_line(output, "pass");
            return Ok(());
        }
        for stmt in stmts {
            self.emit_expr_stmt(stmt, output)?;
        }
        Ok(())
    }

    /// Emit a single MirExprStmt
    fn emit_expr_stmt(&mut self, stmt: &MirExprStmt, output: &mut BytecodeChunk) -> Result<()> {
        match stmt {
            MirExprStmt::Let { name, value, .. } => {
                let expr_chunk = self.emit_expr(value)?;
                self.output_line(output, &format!("{} = {}", name, expr_chunk.code.trim()));
            }
            MirExprStmt::Assign { target, value, .. } => {
                let chunk = self.emit_expr(value)?;
                self.output_line(output, &format!("{} = {}", target, chunk.code.trim()));
            }
            MirExprStmt::Expr(expr) => {
                let chunk = self.emit_expr(expr)?;
                let code = chunk.code.trim();
                if !code.is_empty() {
                    self.output_line(output, code);
                }
            }
            MirExprStmt::Return { value, .. } => {
                if let Some(v) = value {
                    let chunk = self.emit_expr(v)?;
                    self.output_line(output, &format!("return {}", chunk.code.trim()));
                } else {
                    self.output_line(output, "return");
                }
            }
            MirExprStmt::If { condition, then_body, else_body, .. } => {
                let cond_chunk = self.emit_expr(condition)?;
                self.output_line(output, &format!("if {}:", cond_chunk.code.trim()));
                self.indent_level += 1;
                self.emit_expr_stmt_list(then_body, output)?;
                self.indent_level -= 1;
                if let Some(else_stmts) = else_body {
                    self.output_line(output, "else:");
                    self.indent_level += 1;
                    self.emit_expr_stmt_list(else_stmts, output)?;
                    self.indent_level -= 1;
                }
            }
            MirExprStmt::While { condition, body, .. } => {
                let cond_chunk = self.emit_expr(condition)?;
                self.output_line(output, &format!("while {}:", cond_chunk.code.trim()));
                self.indent_level += 1;
                self.emit_expr_stmt_list(body, output)?;
                self.indent_level -= 1;
            }
            MirExprStmt::For { variable, iter, body, .. } => {
                let iter_chunk = self.emit_expr(iter)?;
                self.output_line(output, &format!("for {} in {}:", variable, iter_chunk.code.trim()));
                self.indent_level += 1;
                self.emit_expr_stmt_list(body, output)?;
                self.indent_level -= 1;
            }
            MirExprStmt::Break { .. } => {
                self.output_line(output, "break");
            }
            MirExprStmt::Continue { .. } => {
                self.output_line(output, "continue");
            }
        }
        Ok(())
    }

    /// Emit a list of top-level MirStmt (used for if/while/for body in MirStmt)
    fn emit_mir_stmt_list(&mut self, stmts: &[MirStmt], output: &mut BytecodeChunk) -> Result<()> {
        if stmts.is_empty() {
            self.output_line(output, "pass");
            return Ok(());
        }
        for stmt in stmts {
            self.emit_mir_stmt(stmt, output)?;
        }
        Ok(())
    }

    /// Emit a single top-level MirStmt
    fn emit_mir_stmt(&mut self, stmt: &MirStmt, output: &mut BytecodeChunk) -> Result<()> {
        match stmt {
            MirStmt::Function { name, params, body, .. } => {
                let params_str: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                let params_str = params_str.join(", ");
                self.output_line(output, &format!("def {}({}):", name, params_str));

                self.indent_level += 1;

                // Emit statements
                for stmt in &body.statements {
                    self.emit_expr_stmt(stmt, output)?;
                }

                // Return expression
                if let Some(expr) = &body.expr {
                    let chunk = self.emit_expr(expr)?;
                    self.output_line(output, &format!("return {}", chunk.code.trim()));
                } else if body.statements.is_empty() {
                    self.output_line(output, "pass");
                }

                self.indent_level -= 1;
                output.add_line("");
            }
            MirStmt::Let { name, value, .. } => {
                let expr_chunk = self.emit_expr(value)?;
                self.output_line(output, &format!("{} = {}", name, expr_chunk.code.trim()));
            }
            MirStmt::Expr(expr) => {
                let chunk = self.emit_expr(expr)?;
                let code = chunk.code.trim();
                if !code.is_empty() {
                    self.output_line(output, code);
                }
            }
            MirStmt::If { condition, then_body, else_body, .. } => {
                let cond_chunk = self.emit_expr(condition)?;
                self.output_line(output, &format!("if {}:", cond_chunk.code.trim()));
                self.indent_level += 1;
                self.emit_mir_stmt_list(then_body, output)?;
                self.indent_level -= 1;
                if let Some(else_stmts) = else_body {
                    self.output_line(output, "else:");
                    self.indent_level += 1;
                    self.emit_mir_stmt_list(else_stmts, output)?;
                    self.indent_level -= 1;
                }
            }
            MirStmt::While { condition, body, .. } => {
                let cond_chunk = self.emit_expr(condition)?;
                self.output_line(output, &format!("while {}:", cond_chunk.code.trim()));
                self.indent_level += 1;
                self.emit_mir_stmt_list(body, output)?;
                self.indent_level -= 1;
            }
            MirStmt::For { variable, iter, body, .. } => {
                let iter_chunk = self.emit_expr(iter)?;
                self.output_line(output, &format!("for {} in {}:", variable, iter_chunk.code.trim()));
                self.indent_level += 1;
                self.emit_mir_stmt_list(body, output)?;
                self.indent_level -= 1;
            }
            MirStmt::Return { value, .. } => {
                if let Some(v) = value {
                    let chunk = self.emit_expr(v)?;
                    self.output_line(output, &format!("return {}", chunk.code.trim()));
                } else {
                    self.output_line(output, "return");
                }
            }
            MirStmt::Break { .. } => {
                self.output_line(output, "break");
            }
            MirStmt::Continue { .. } => {
                self.output_line(output, "continue");
            }
            MirStmt::Match { scrutinee, arms, .. } => {
                let scrut_chunk = self.emit_expr(scrutinee)?;
                self.output_line(output, &format!("match {}:", scrut_chunk.code.trim()));
                self.indent_level += 1;
                for arm in arms {
                    let pattern_str = format_mir_pattern(&arm.pattern);
                    if let Some(guard) = &arm.guard {
                        let guard_chunk = self.emit_expr(guard)?;
                        self.output_line(output, &format!("case {} if {}:", pattern_str, guard_chunk.code.trim()));
                    } else {
                        self.output_line(output, &format!("case {}:", pattern_str));
                    }
                    self.indent_level += 1;
                    self.emit_mir_stmt_list(&arm.body, output)?;
                    self.indent_level -= 1;
                }
                self.indent_level -= 1;
            }
        }
        Ok(())
    }
}

impl CodeEmitter for PythonGenerator {
    fn emit_program(&mut self, program: &MirProgram) -> Result<BytecodeChunk> {
        let mut output = BytecodeChunk::new();

        // Add Python header comment
        output.add_line("# Generated by Nevermind compiler");

        for stmt in &program.statements {
            self.emit_mir_stmt(stmt, &mut output)?;
        }

        // Auto-call main() if it exists
        let has_main = program.statements.iter().any(|s| {
            matches!(s, MirStmt::Function { name, .. } if name == "main")
        });
        if has_main {
            output.add_line("");
            output.add_line("if __name__ == \"__main__\":");
            output.add_line("    main()");
        }

        Ok(output)
    }

    fn emit_function(&mut self, func: &MirFunction) -> Result<BytecodeChunk> {
        let mut output = BytecodeChunk::new();

        let params: Vec<String> = func.params.iter().map(|p| p.name.clone()).collect();
        let params_str = params.join(", ");
        output.add_line(&format!("def {}({}):", func.name, params_str));

        self.indent_level += 1;

        for stmt in &func.body.statements {
            self.emit_expr_stmt(stmt, &mut output)?;
        }

        if let Some(expr) = &func.body.expr {
            let chunk = self.emit_expr(expr)?;
            self.output_line(&mut output, &format!("return {}", chunk.code.trim()));
        } else if func.body.statements.is_empty() {
            self.output_line(&mut output, "pass");
        }

        self.indent_level -= 1;
        output.add_line("");

        Ok(output)
    }

    fn emit_expr(&mut self, expr: &MirExpr) -> Result<BytecodeChunk> {
        let mut output = BytecodeChunk::new();

        match expr {
            MirExpr::Literal { value, .. } => {
                output.add_line(&self.emit_literal(value));
            }

            MirExpr::Variable { name, .. } => {
                output.add_line(name);
            }

            MirExpr::Binary { op, left, right, .. } => {
                let left_chunk = self.emit_expr(left)?;
                let right_chunk = self.emit_expr(right)?;
                let py_op = self.map_binop(*op);

                output.add_line(&format!("({} {} {})",
                    left_chunk.code.trim(),
                    py_op,
                    right_chunk.code.trim()
                ));
            }

            MirExpr::Unary { op, operand, .. } => {
                let operand_chunk = self.emit_expr(operand)?;
                let py_op = self.map_unop(*op);

                output.add_line(&format!("{}{}", py_op, operand_chunk.code.trim()));
            }

            MirExpr::Call { callee, args, .. } => {
                let callee_chunk = self.emit_expr(callee)?;
                let mut arg_strings = Vec::new();

                for arg in args {
                    let chunk = self.emit_expr(arg)?;
                    arg_strings.push(chunk.code.trim().to_string());
                }

                output.add_line(&format!("{}({})", callee_chunk.code.trim(), arg_strings.join(", ")));
            }

            MirExpr::Block { statements, expr, .. } => {
                for stmt in statements {
                    match stmt {
                        MirExprStmt::Let { name, value, .. } => {
                            let value_chunk = self.emit_expr(value)?;
                            output.add_line(&format!("{} = {}", name, value_chunk.code.trim()));
                        }
                        MirExprStmt::Assign { target, value, .. } => {
                            let chunk = self.emit_expr(value)?;
                            output.add_line(&format!("{} = {}", target, chunk.code.trim()));
                        }
                        MirExprStmt::Expr(e) => {
                            let chunk = self.emit_expr(e)?;
                            let code = chunk.code.trim();
                            if !code.is_empty() {
                                output.code.push_str(code);
                                output.code.push('\n');
                            }
                        }
                        MirExprStmt::Return { value, .. } => {
                            if let Some(v) = value {
                                let chunk = self.emit_expr(v)?;
                                output.add_line(&format!("return {}", chunk.code.trim()));
                            } else {
                                output.add_line("return");
                            }
                        }
                        MirExprStmt::If { condition, then_body, else_body, .. } => {
                            let cond_chunk = self.emit_expr(condition)?;
                            output.add_line(&format!("{}if {}:", self.indent(), cond_chunk.code.trim()));
                            self.indent_level += 1;
                            self.emit_expr_stmt_list(then_body, &mut output)?;
                            self.indent_level -= 1;
                            if let Some(else_stmts) = else_body {
                                output.add_line(&format!("{}else:", self.indent()));
                                self.indent_level += 1;
                                self.emit_expr_stmt_list(else_stmts, &mut output)?;
                                self.indent_level -= 1;
                            }
                        }
                        MirExprStmt::While { condition, body, .. } => {
                            let cond_chunk = self.emit_expr(condition)?;
                            output.add_line(&format!("{}while {}:", self.indent(), cond_chunk.code.trim()));
                            self.indent_level += 1;
                            self.emit_expr_stmt_list(body, &mut output)?;
                            self.indent_level -= 1;
                        }
                        MirExprStmt::For { variable, iter, body, .. } => {
                            let iter_chunk = self.emit_expr(iter)?;
                            output.add_line(&format!("{}for {} in {}:", self.indent(), variable, iter_chunk.code.trim()));
                            self.indent_level += 1;
                            self.emit_expr_stmt_list(body, &mut output)?;
                            self.indent_level -= 1;
                        }
                        MirExprStmt::Break { .. } => {
                            output.add_line(&format!("{}break", self.indent()));
                        }
                        MirExprStmt::Continue { .. } => {
                            output.add_line(&format!("{}continue", self.indent()));
                        }
                    }
                }

                // Final expression
                if let Some(e) = expr {
                    let chunk = self.emit_expr(e)?;
                    output.add_line(chunk.code.trim());
                }
            }

            MirExpr::List { elements, .. } => {
                let mut element_strings = Vec::new();
                for elem in elements {
                    let chunk = self.emit_expr(elem)?;
                    element_strings.push(chunk.code.trim().to_string());
                }
                output.add_line(&format!("[{}]", element_strings.join(", ")));
            }

            MirExpr::If { condition, then_branch, else_branch, .. } => {
                let cond_chunk = self.emit_expr(condition)?;
                let then_chunk = self.emit_expr(then_branch)?;
                let else_chunk = self.emit_expr(else_branch)?;

                output.add_line(&format!("({} if {} else {})",
                    then_chunk.code.trim(),
                    cond_chunk.code.trim(),
                    else_chunk.code.trim()
                ));
            }

            MirExpr::Index { array, index, .. } => {
                let array_chunk = self.emit_expr(array)?;
                let index_chunk = self.emit_expr(index)?;

                output.add_line(&format!("{}[{}]",
                    array_chunk.code.trim(),
                    index_chunk.code.trim()
                ));
            }

            MirExpr::Lambda { params, body, .. } => {
                let body_chunk = self.emit_expr(body)?;
                let params_str = params.join(", ");
                output.add_line(&format!("lambda {}: {}", params_str, body_chunk.code.trim()));
            }
        }

        Ok(output)
    }
}

/// Format a MirPattern for Python match/case syntax
fn format_mir_pattern(pattern: &nevermind_mir::MirPattern) -> String {
    match pattern {
        nevermind_mir::MirPattern::Wildcard { .. } => "_".to_string(),
        nevermind_mir::MirPattern::Variable { name, .. } => name.clone(),
        nevermind_mir::MirPattern::Literal { value, .. } => match value {
            Literal::Int(v) => v.to_string(),
            Literal::Float(v) => v.to_string(),
            Literal::String(v) => format!("\"{}\"", v),
            Literal::Bool(v) => if *v { "True" } else { "False" }.to_string(),
            Literal::Null => "None".to_string(),
        },
        nevermind_mir::MirPattern::List { patterns, .. } => {
            let parts: Vec<String> = patterns.iter().map(format_mir_pattern).collect();
            format!("[{}]", parts.join(", "))
        }
        nevermind_mir::MirPattern::Constructor { name, args, .. } => {
            if args.is_empty() {
                name.clone()
            } else {
                let parts: Vec<String> = args.iter().map(format_mir_pattern).collect();
                format!("{}({})", name, parts.join(", "))
            }
        }
    }
}

/// Escape a string literal for Python
fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
