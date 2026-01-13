# Contributing to Nevermind

感谢你对 Nevermind 的关注！这是一份详细的开发者指南，帮助新人快速上手。

## 📋 目录

1. [项目概述](#项目概述)
2. [开发环境设置](#开发环境设置)
3. [项目结构](#项目结构)
4. [编译流程](#编译流程)
5. [测试](#测试)
6. [代码规范](#代码规范)
7. [提交 Pull Request](#提交-pull-request)
8. [获取帮助](#获取帮助)

---

## 项目概述

Nevermind 是一个用 **Rust** 编写的现代编程语言编译器。它实现了：

- **完整的编译器前端**
  - Lexer (词法分析)
  - Parser (语法分析)
  - Name Resolver (名称解析)
  - Type Checker (类型检查)

- **Hindley-Milner 类型推断**
- **模式匹配**
- **Python 互操作性**（计划中）

---

## 开发环境设置

### 前置要求

- **Rust** 1.70 或更高版本
- **Git** 2.0 或更高版本
- **操作系统**: Windows, macOS, Linux

### 安装 Rust

```bash
# Windows (下载并运行 rustup-init.exe)
# https://rustup.rs/

# macOS/Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 克隆项目

```bash
git clone https://github.com/L0stInFades/Nevermind-Lang.git
cd Nevermind-Lang
```

### 验证安装

```bash
# 检查版本
rustc --version
cargo --version

# 构建项目
cargo build

# 运行测试
cargo test
```

---

## 项目结构

```
nevermind/
├── Cargo.toml              # 主项目配置
├── README.md               # 项目说明
├── CONTRIBUTING.md         # 本文件
├── DESIGN_SPEC.md          # 语言设计规范
├── COMPILER_ARCHITECTURE.md # 编译器架构
│
├── src/                    # CLI 工具源码
│   └── main.rs
│
├── crates/                 # 编译器 crates
│   ├── common/             # 公共类型和工具
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── span.rs     # 源码位置
│   │   │   ├── error.rs    # 错误类型
│   │   │   └── node_id.rs  # AST 节点 ID
│   │   └── Cargo.toml
│   │
│   ├── ast/                # 抽象语法树
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── expr.rs     # 表达式节点
│   │   │   ├── stmt.rs     # 语句节点
│   │   │   ├── pattern.rs  # 模式
│   │   │   └── op.rs       # 运算符
│   │   └── Cargo.toml
│   │
│   ├── lexer/              # 词法分析器
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── token.rs    # Token 定义
│   │   │   └── lexer.rs    # Lexer 实现
│   │   ├── tests/
│   │   │   └── lexer_tests.rs  # 108 个测试
│   │   └── Cargo.toml
│   │
│   ├── parser/             # 语法分析器
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── parser.rs   # 主解析器
│   │   │   ├── expr_parser.rs  # 表达式解析
│   │   │   └── pattern_parser.rs # 模式解析
│   │   ├── tests/
│   │   │   └── parser_tests.rs  # 100+ 个测试
│   │   └── Cargo.toml
│   │
│   ├── name-resolver/      # 名称解析器
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── symbol.rs   # 符号定义
│   │   │   ├── scope.rs    # 作用域
│   │   │   ├── symbol_table.rs # 符号表
│   │   │   ├── error.rs    # 错误
│   │   │   └── resolver.rs # 解析器
│   │   └── Cargo.toml
│   │
│   └── type-checker/       # 类型检查器
│       ├── src/
│       │   ├── lib.rs
│       │   ├── types.rs    # 类型表示
│       │   ├── ty.rs       # TypeVar 和 TypeScheme
│       │   ├── environment.rs # 类型环境
│       │   ├── unification.rs  # 统一算法
│       │   ├── checker.rs  # 类型检查器
│       │   └── error.rs    # 类型错误
│       └── Cargo.toml
│
├── examples/               # 示例程序
│   ├── simple.nm
│   ├── math.nm
│   ├── test_fn.nm
│   └── ...
│
└── tests/                  # 集成测试
    └── ...
```

---

## 编译流程

Nevermind 编译器的编译流程：

```
源代码 (.nm)
    ↓
Lexer (Token 序列)
    ↓
Parser (AST)
    ↓
Name Resolver (带作用域信息的 AST)
    ↓
Type Checker (带类型信息的 AST)
    ↓
[未来] Code Generation (字节码)
```

### 1. Lexer (词法分析)

**文件**: `crates/lexer/src/lexer.rs`

将源代码转换为 Token 序列：

```nevermind
let x = 42
```

转换为：

```rust
[
    Token::Let,
    Token::Identifier("x"),
    Token::Equal,
    Token::Integer(42),
]
```

**关键函数**:
- `Lexer::new(source)` - 创建 Lexer
- `Lexer::next_token()` - 获取下一个 Token
- `Lexer::tokenize()` - 完全词法分析

**测试**: 108 个单元测试 (100% 通过)

---

### 2. Parser (语法分析)

**文件**: `crates/parser/src/parser.rs`

将 Token 序列转换为 AST：

```rust
Stmt::Let {
    name: "x",
    value: Expr::Literal(Literal::Integer(42)),
    ...
}
```

**关键函数**:
- `Parser::new(tokens)` - 创建 Parser
- `Parser::parse()` - 完全解析
- `Parser::parse_statement()` - 解析语句
- `Parser::parse_expression()` - 解析表达式

**解析技术**:
- **递归下降解析** (语句)
- **Pratt 解析** (表达式)
- **模式匹配解析**

**测试**: 100+ 个单元测试 (100% 通过)

---

### 3. Name Resolver (名称解析)

**文件**: `crates/name-resolver/src/resolver.rs`

检查变量和函数的定义和使用：

```rust
let x = 42
print y  # 错误: y 未定义
```

**关键功能**:
- 符号表管理
- 作用域嵌套
- 变量遮蔽检测
- 未定义变量检测
- 重复定义检测

**测试**: 包含在 resolver.rs 中

---

### 4. Type Checker (类型检查)

**文件**: `crates/type-checker/src/checker.rs`

实现 Hindley-Milner 类型推断：

```nevermind
fn id(x) = x
# 推断类型: forall a. a -> a
```

**关键算法**:
- **统一** (Unification)
- **泛化** (Generalization)
- **实例化** (Instantiation)
- **Occurs Check** (防止无限类型)

**关键函数**:
- `TypeChecker::check()` - 类型检查
- `TypeChecker::infer_expression()` - 表达式类型推断
- `Unifier::unify()` - 类型统一

**测试**: 30 个单元测试 (100% 通过)

---

## 测试

### 运行所有测试

```bash
cargo test
```

### 运行特定 crate 的测试

```bash
# Lexer 测试
cargo test --package nevermind-lexer

# Parser 测试
cargo test --package nevermind-parser

# 类型检查器测试
cargo test --package nevermind-type-checker
```

### 运行特定测试

```bash
# 运行单个测试
cargo test test_unify_same_types

# 运行某个模块的测试
cargo test types::tests
```

### 测试覆盖率

```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 生成覆盖率报告
cargo tarpaulin --out Html
```

当前测试覆盖率：
- Lexer: > 95%
- Parser: > 90%
- Name Resolver: > 85%
- Type Checker: > 80%

---

## 代码规范

### Rust 代码风格

遵循标准的 Rust 风格指南：

```bash
# 自动格式化代码
cargo fmt

# 检查代码风格
cargo clippy
```

### 命名约定

- **类型**: `PascalCase` (例如: `TypeChecker`)
- **函数**: `snake_case` (例如: `parse_expression`)
- **常量**: `SCREAMING_SNAKE_CASE` (例如: `MAX_DEPTH`)
- **宏**: `snake_case!` (例如: `vec![]`)

### 注释规范

```rust
//! 模块级文档（crate/lib.rs）
//!
//! 详细描述...

/// 项文档（函数/结构体/类型）
///
/// # Examples
///
/// ```
/// let result = function();
/// ```
pub fn function() -> Type {
    // ...
}
```

### 错误处理

使用 `Result<T, E>` 和 `thiserror`:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error at {location}")]
    Parse { location: Span },
}

pub fn do_work() -> Result<(), MyError> {
    // ...
}
```

---

## 添加新功能

### 1. 添加新的 AST 节点

**步骤**:

1. 在 `crates/ast/src/expr.rs` 或 `stmt.rs` 中添加新的枚举变体
2. 在 `crates/parser/src/` 中添加解析逻辑
3. 在 `crates/name-resolver/src/resolver.rs` 中添加名称解析
4. 在 `crates/type-checker/src/checker.rs` 中添加类型检查
5. 添加测试

**示例**：添加 `try...catch` 表达式

```rust
// crates/ast/src/stmt.rs
pub enum Stmt {
    // ...
    TryCatch {
        try_block: Vec<Stmt>,
        catch_var: String,
        catch_block: Vec<Stmt>,
        span: Span,
    },
}
```

### 2. 添加新的类型

**步骤**:

1. 在 `crates/type-checker/src/types.rs` 中添加新的类型变体
2. 更新 `unify()` 方法
3. 更新 `free_vars()` 方法
4. 添加测试

**示例**：添加 `Option<T>` 类型

```rust
// crates/type-checker/src/types.rs
pub enum Type {
    // ...
    Option {
        inner: Box<Type>,
    },
}
```

### 3. 添加新的运算符

**步骤**:

1. 在 `crates/ast/src/op.rs` 中添加运算符定义
2. 在 `crates/lexer/src/lexer.rs` 中添加 Token
3. 在 `crates/parser/src/expr_parser.rs` 中添加解析
4. 在 `crates/type-checker/src/checker.rs` 中添加类型检查

---

## 提交 Pull Request

### 1. Fork 仓库

点击 GitHub 页面右上角的 "Fork" 按钮。

### 2. 创建分支

```bash
git checkout -b feature/your-feature-name
```

### 3. 编写代码

- 遵循代码规范
- 添加测试
- 更新文档

### 4. 提交代码

```bash
git add .
git commit -m "Add: feature description"

# 推送到你的 fork
git push origin feature/your-feature-name
```

### 5. 创建 Pull Request

在 GitHub 上创建 PR，填写：

- **标题**: `[Feature/Bugfix/Docs] 简短描述`
- **描述**:
  - 为什么需要这个改动
  - 改动做了什么
  - 相关的 issue
  - 测试方法

### PR 模板

```markdown
## 改动类型
- [ ] Bugfix
- [ ] Feature
- [ ] Breaking change
- [ ] Documentation

## 描述
<!-- 详细描述你的改动 -->

## 相关 Issue
Closes #(issue number)

## 测试
- [ ] 单元测试通过
- [ ] 添加了新测试
- [ ] 手动测试

## 检查清单
- [ ] 代码遵循项目规范
- [ ] 已添加测试
- [ ] 已更新文档
- [ ] 所有测试通过
```

---

## 获取帮助

### 文档

- **[README.md](./README.md)** - 项目概述
- **[DESIGN_SPEC.md](./DESIGN_SPEC.md)** - 语言设计
- **[COMPILER_ARCHITECTURE.md](./COMPILER_ARCHITECTURE.md)** - 编译器架构
- **[PROJECT_SUMMARY.md](./PROJECT_SUMMARY.md)** - 项目总结

### 模块文档

每个 crate 都有详细的文档：

- **[crates/name-resolver/README.md](./crates/name-resolver/README.md)**
- **[crates/name-resolver/IMPLEMENTATION_SUMMARY.md](./crates/name-resolver/IMPLEMENTATION_SUMMARY.md)**

### 联系方式

- **GitHub Issues**: https://github.com/L0stInFades/Nevermind-Lang/issues
- **Discussions**: https://github.com/L0stInFades/Nevermind-Lang/discussions

### 常见问题

**Q: 如何运行示例程序？**

```bash
# 编译
cargo build --release

# 运行示例
./target/release/nevermind run examples/simple.nm
```

**Q: 如何添加新的关键字？**

1. 在 `crates/ast/src/op.rs` 或相应位置添加
2. 在 `crates/lexer/src/token.rs` 中添加 Token
3. 在 Lexer 中添加识别逻辑
4. 在 Parser 中添加解析逻辑

**Q: 类型推断失败如何调试？**

使用 `cargo test` 查看详细错误信息，或添加 `dbg!` 宏：

```rust
let ty = self.infer_expression(expr)?;
dbg!(&ty);
```

---

## 性能优化

### 性能测试

```bash
# 安装 bench 工具
cargo install cargo-criterion

# 运行性能测试
cargo bench
```

### 优化建议

1. **避免不必要的克隆**
   ```rust
   // 不好
   fn process(data: Vec<String>) -> Vec<String>

   // 好
   fn process(data: &[String]) -> Vec<String>
   ```

2. **使用 `Cow` 避免分配**
   ```rust
   use std::borrow::Cow;

   fn get_name(s: &str) -> Cow<str> {
       if s.contains("bad") {
           Cow::Owned(s.replace("bad", "good"))
       } else {
           Cow::Borrowed(s)
       }
   }
   ```

3. **使用迭代器而不是循环**
   ```rust
   // 好
   let result: Vec<_> = items.iter().map(|x| x * 2).collect();

   // 不好
   let mut result = Vec::new();
   for item in items {
       result.push(item * 2);
   }
   ```

---

## 调试技巧

### 使用日志

```bash
# 启用日志
RUST_LOG=debug cargo run

# 特定模块
RUST_LOG=nevermind_parser=debug cargo run
```

### 使用 GDB/LLDB

```bash
# 编译调试版本
cargo build

# 使用 GDB (Linux)
gdb target/debug/nevermind

# 使用 LLDB (macOS)
lldb target/debug/nevermind
```

### Visual Studio Code 调试

创建 `.vscode/launch.json`:

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug nevermind",
            "cargo": {
                "args": ["build"],
                "filter": {
                    "name": "nevermind",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}"
        }
    ]
}
```

---

## 许可证

贡献的代码将使用 MIT 许可证发布。

---

## 致谢

感谢所有贡献者！

<!-- 添加贡献者列表 -->

---

**Happy Coding! 🚀**

有任何问题欢迎提 Issue 或 PR！
