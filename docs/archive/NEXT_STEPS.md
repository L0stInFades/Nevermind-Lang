# Nevermind 下一步开发计划

## 📊 当前状态 (2025-01-08)

### ✅ 已完成
- [x] Lexer - 词法分析器 (100%)
- [x] Parser - 语法分析器 (90%)
  - [x] 基础表达式 (变量、字面量、运算符)
  - [x] 函数定义和调用
  - [x] If 表达式
  - [x] 列表和映射
  - [ ] While 循环 (需要修复)
  - [ ] For 循环 (需要测试)
  - [ ] Match 表达式 (需要修复)
  - [ ] Return/Break/Continue (需要测试)
- [x] CLI 工具 (100%)
- [x] 错误报告系统 (100%)
- [x] Git 仓库和文档 (100%)

### ⚠️ 需要修复的 Parser 问题
1. **While 循环解析** - `end` 关键字处理问题
2. **Match 表达式** - 模式解析需要完善
3. **Pattern 解析** - 模式匹配语法需要实现

---

## 🎯 短期目标 (1-2 周)

### 优先级 1: 修复 Parser 剩余问题

#### 1.1 修复 While 循环 (预计 1-2 小时)

**问题**: `do...end` 嵌套时的 `end` 关键字冲突

**解决方案**:
```rust
// parse_while_statement 需要检查 then 块类型
// 类似 if 语句的处理方式
pub fn parse_while_statement(&mut self) -> ParseResult<Option<Stmt>> {
    let start = self.peek_span();
    self.consume_keyword(Keyword::While, "expected 'while'")?;

    let condition = self.parse_expression()?;

    // 检查是 do...end 还是单表达式
    if self.match_keyword(Keyword::Do) {
        // do...end 块
        let mut body = Vec::new();
        while !self.check_keyword(Keyword::End) && !self.parser.is_at_end() {
            if let Some(stmt) = self.parse_statement()? {
                body.push(stmt);
            }
        }
        self.consume_keyword(Keyword::End, "expected 'end' to close while loop")?;

        // 消费 while 语句的 end
        self.consume_keyword(Keyword::End, "expected 'end' to close while statement")?;

        Ok(Some(Stmt::While { ... }))
    } else {
        // 单表达式形式（如果支持）
        ...
    }
}
```

#### 1.2 修复 Match 表达式 (预计 2-3 小时)

**问题**: 模式解析未完全实现

**需要实现**:
- 字面量模式
- 变量模式
- 通配符模式 (`_`)
- 析构模式 (列表、元组)
- 模式守卫 (`when`/`if`)

**示例**:
```nevermind
match x
{
  1 => "one",
  2 | 3 => "two or three",
  n if n > 10 => "large",
  _ => "other"
}
```

#### 1.3 实现 Pattern 解析器 (预计 3-4 小时)

**新文件**: `crates/parser/src/pattern_parser.rs`

**功能**:
```rust
pub struct PatternParser<'a> {
    parser: &'a mut Parser,
}

impl<'a> PatternParser<'a> {
    pub fn parse_pattern(&mut self) -> ParseResult<Pattern> {
        match self.parser.peek_token_type() {
            TokenType::Literal(_) => self.parse_literal_pattern(),
            TokenType::Identifier => self.parse_var_or_wildcard(),
            TokenType::Delimiter(Delimiter::LBracket) => self.parse_list_pattern(),
            TokenType::Delimiter(Delimiter::LBrace) => self.parse_struct_pattern(),
            _ => Err(...),
        }
    }
}
```

#### 1.4 测试 Return/Break/Continue (预计 1 小时)

**测试文件**:
```nevermind
# examples/test_return.nm
fn test()
do
  return 42
end
end

# examples/test_loop_control.nm
while true
do
  break
end
end

for i in [1, 2, 3]
do
  if i == 2 then break end
end
end
```

---

### 优先级 2: 添加单元测试 (预计 1-2 天)

#### 2.1 Lexer 测试

**文件**: `crates/lexer/tests/lexer_tests.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let input = "let fn if then else end";
        let tokens = Lexer::new(input).tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenType::Keyword(Keyword::Let));
        assert_eq!(tokens[1].kind, TokenType::Keyword(Keyword::Fn));
        // ...
    }

    #[test]
    fn test_numbers() {
        let input = "42 3.14 1e10";
        let tokens = Lexer::new(input).tokenize().unwrap();
        // ...
    }

    #[test]
    fn test_strings() {
        let input = r#""hello" "world\n""#;
        // ...
    }
}
```

#### 2.2 Parser 测试

**文件**: `crates/parser/tests/parser_tests.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_let_statement() {
        let input = "let x = 42";
        let mut parser = Parser::new(input);
        let stmts = parser.parse().unwrap();
        assert_eq!(stmts.len(), 1);
        // ...
    }

    #[test]
    fn test_function_definition() {
        let input = "fn add(a, b) do a + b end end";
        // ...
    }

    #[test]
    fn test_if_expression() {
        let input = "let x = if true then 1 else 0 end";
        // ...
    }

    #[test]
    fn test_function_call() {
        let input = "add(1, 2)";
        // ...
    }
}
```

#### 2.3 集成测试

**文件**: `tests/integration_tests.rs`

```rust
#[test]
fn test_full_program() {
    let input = r#"
    fn main()
    do
      let x = 10
      let y = 20
      print add(x, y)
    end
    end

    fn add(a, b)
    do
      a + b
    end
    end
    "#;

    let mut parser = Parser::new(input);
    let stmts = parser.parse().unwrap();
    assert!(stmts.len() > 0);
}
```

---

## 🚀 中期目标 (2-4 周)

### Phase 1.2: Name Resolution (名称解析)

#### 目标
实现符号表和作用域管理，检测未定义变量和重复定义。

#### 实现计划

**新建 Crate**: `crates/name-resolver/`

**目录结构**:
```
crates/name-resolver/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── symbol.rs          # 符号定义
    ├── scope.rs           # 作用域管理
    ├── symbol_table.rs    # 符号表
    ├── resolver.rs        # 名称解析器
    └── error.rs           # 错误类型
```

**核心数据结构**:

```rust
// src/symbol.rs
pub enum SymbolKind {
    Variable,
    Function,
    Parameter,
    Type,
}

pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub span: Span,
    pub type_: Option<Type>,
}

// src/scope.rs
pub struct Scope {
    pub parent: Option<Box<Scope>>,
    pub symbols: HashMap<String, Symbol>,
    pub level: u32,
}

impl Scope {
    pub fn new(parent: Option<Scope>) -> Self { ... }
    pub fn insert(&mut self, name: String, symbol: Symbol) -> Result<(), NameError> { ... }
    pub fn lookup(&self, name: &str) -> Option<&Symbol> { ... }
}

// src/symbol_table.rs
pub struct SymbolTable {
    pub current_scope: Scope,
    pub scopes: Vec<Scope>,
}

impl SymbolTable {
    pub fn new() -> Self { ... }
    pub fn enter_scope(&mut self) { ... }
    pub fn exit_scope(&mut self) { ... }
    pub fn declare(&mut self, name: String, symbol: Symbol) -> Result<(), NameError> { ... }
    pub fn resolve(&self, name: &str) -> Result<&Symbol, NameError> { ... }
}
```

**使用示例**:

```rust
// src/resolver.rs
pub struct NameResolver {
    symbol_table: SymbolTable,
    errors: Vec<NameError>,
}

impl NameResolver {
    pub fn new() -> Self { ... }

    pub fn resolve(&mut self, stmts: &[Stmt]) -> Result<(), Vec<NameError>> {
        for stmt in stmts {
            self.resolve_statement(stmt)?;
        }
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    fn resolve_statement(&mut self, stmt: &Stmt) -> Result<(), NameError> {
        match stmt {
            Stmt::Let { name, .. } => {
                let symbol = Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Variable,
                    span: stmt.span(),
                    type_: None,
                };
                self.symbol_table.declare(name.clone(), symbol)?;
            }
            Stmt::ExprStmt { expr, .. } => {
                self.resolve_expression(expr)?;
            }
            // ...
        }
        Ok(())
    }
}
```

**测试**:
```nevermind
# 应该报错：未定义变量
let x = y  # Error: undefined variable 'y'

# 应该报错：重复定义
let x = 1
let x = 2  # Error: duplicate definition of 'x'

# 应该正常工作
let x = 1
let y = x  # OK
```

---

### Phase 1.3: Type Checker (类型检查器)

#### 目标
实现 Hindley-Milner 类型推断，支持基本类型和泛型。

#### 实现计划

**新建 Crate**: `crates/type-checker/`

**目录结构**:
```
crates/type-checker/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── types.rs           # 类型定义
    ├── env.rs             # 类型环境
    ├── inference.rs       # 类型推断
    ├── constraints.rs     # 约束求解
    ├── checker.rs         # 类型检查器
    └── error.rs           # 类型错误
```

**核心功能**:

1. **基本类型**: Int, Float, String, Bool, Null, List, Map, Function
2. **类型推断**: 自动推断表达式类型
3. **泛型支持**: 泛型函数和类型
4. **类型约束**: 检查类型匹配

**示例**:
```nevermind
# 类型推断示例
let x = 42        # Int
let y = 3.14      # Float
let z = x + y     # Error: type mismatch

# 泛型函数
fn id[T](x: T) -> T
do
  x
end
end

let a = id(42)     # Int
let b = id("hello") # String
```

---

## 📋 长期目标 (1-3 个月)

### Phase 2: Code Generation

**目标**: 生成 Python 字节码

**主要任务**:
1. HIR (High-level IR) 设计和实现
2. MIR (Mid-level IR) 设计和实现
3. Python bytecode emitter
4. 运行时系统

### Phase 3: Runtime & Standard Library

**目标**: 实现运行时和标准库

**主要任务**:
1. 内存管理 (GC)
2. Python FFI
3. 标准库实现
4. 并发运行时

---

## 🎯 下一步行动 (优先级排序)

### 立即开始 (今天)
1. ✅ 修复 While 循环解析
2. ✅ 修复 Match 表达式
3. ✅ 实现 Pattern 解析器
4. ✅ 测试 Return/Break/Continue

### 本周完成
5. ⏳ 添加 Parser 单元测试
6. ⏳ 添加 Lexer 单元测试
7. ⏳ 创建集成测试套件
8. ⏳ 完善错误信息

### 下周计划
9. ⏳ 实现 Name Resolver (符号表)
10. ⏳ 实现 Scope 管理
11. ⏳ 添加变量检查
12. ⏳ 集成到 CLI 工具

---

## 📊 进度追踪

```
Phase 1: Foundation            [████████░░] 80%
├── Lexer & Parser            [█████████░] 90%
├── Name Resolution           [░░░░░░░░░░]   0%
├── Type Checker              [░░░░░░░░░░]   0%
└── HIR Lowering              [░░░░░░░░░░]   0%

Phase 2: Code Generation       [░░░░░░░░░░]   0%
Phase 3: Runtime & Stdlib     [░░░░░░░░░░]   0%

Overall Progress:             [███░░░░░░░]  27%
```

---

## 🔗 相关文档

- [DESIGN_SPEC.md](./DESIGN_SPEC.md) - 语言设计规范
- [TYPE_SYSTEM_DESIGN.md](./TYPE_SYSTEM_DESIGN.md) - 类型系统设计
- [COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md) - 编译器架构
- [ROADMAP.md](./ROADMAP.md) - 完整路线图

---

*最后更新: 2025-01-08*
*负责人: Claude & 用户*
