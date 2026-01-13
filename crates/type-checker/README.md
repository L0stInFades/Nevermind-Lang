# Nevermind Type Checker

**Hindley-Milner 类型推断系统的完整实现**

## 📋 概述

类型检查器是 Nevermind 编译器的第四个阶段，负责：

- ✅ 类型推断（Hindley-Milner 算法）
- ✅ 多态性支持（泛型）
- ✅ 类型统一
- ✅ 丰富的错误报告

## 🏗️ 架构

```
TypeChecker
    ├── Type Environment (作用域管理)
    ├── TypeVar & TypeScheme (多态性)
    ├── Unifier (类型统一)
    └── Error Reporting (错误报告)
```

## 📁 文件结构

```
type-checker/
├── Cargo.toml
├── README.md (本文件)
└── src/
    ├── lib.rs              # 模块导出
    ├── types.rs            # 类型表示
    ├── ty.rs               # TypeVar 和 TypeScheme
    ├── environment.rs      # 类型环境
    ├── unification.rs      # 统一算法
    ├── checker.rs          # 主类型检查器
    └── error.rs            # 类型错误
```

## 🧰 核心组件

### 1. Type (`types.rs`)

类型的表示：

```rust
pub enum Type {
    // 类型变量（用于推断）
    Var(TypeVarRef),

    // 基本类型
    Int,
    Float,
    String,
    Bool,
    Null,
    Unit,

    // 复合类型
    Function(Vec<Type>, Box<Type>),  // 函数类型
    List(Box<Type>),                  // 列表 [T]
    Map(Box<Type>),                   # 映射 {String: T}
    Tuple(Vec<Type>),                 # 元组 (T1, T2, ...)

    // 用户定义类型
    User(String),
}
```

**主要功能**:
- `Type::var(id)` - 创建类型变量
- `Type::function(params, ret)` - 创建函数类型
- `Type::list(elem)` - 创建列表类型
- `Type::display_name()` - 获取类型的显示名称

**示例**:

```rust
// Int 类型
let int_ty = Type::Int;

// 函数类型: Int -> Int
let fn_ty = Type::function(vec![Type::Int], Type::Int);

// 列表类型: [Int]
let list_ty = Type::list(Type::Int);

// 元组类型: (Int, Bool)
let tuple_ty = Type::tuple(vec![Type::Int, Type::Bool]);
```

---

### 2. TypeVar & TypeScheme (`ty.rs`)

多态性的实现：

```rust
// 类型变量
pub struct TypeVar {
    id: usize,
}

// 类型方案: ∀α1...αn. type
pub struct TypeScheme {
    pub vars: Vec<TypeVar>,  // 全称量化的变量
    pub ty: Type,             // 类型本身
}
```

**主要功能**:

#### TypeVar
- `TypeVar::new(id)` - 创建新的类型变量
- 用于类型推断中的未知类型

#### TypeScheme
- `TypeScheme::generalize(ty, free_vars)` - 泛化类型
- `TypeScheme::instantiate(ctx)` - 实例化类型方案
- `Type::free_vars(ty)` - 获取类型中的自由变量

**示例**:

```rust
// 创建多态类型: ∀a. a -> a
let identity_ty = Type::function(
    vec![Type::Var(TypeVarRef::new(0))],
    Type::Var(TypeVarRef::new(0))
);

let scheme = TypeScheme::new(
    vec![TypeVar::new(0)],
    identity_ty
);

// 实例化为具体类型
let mut ctx = TypeContext::new();
let instance = scheme.instantiate(&mut ctx);
// 结果: t1 -> t1 (新的类型变量)
```

---

### 3. TypeEnvironment (`environment.rs`)

类型环境管理作用域和类型绑定：

```rust
pub struct TypeEnvironment {
    scopes: Vec<Scope>,  // 作用域栈
}
```

**主要功能**:
- `env.enter_scope()` - 进入新作用域
- `env.exit_scope()` - 退出当前作用域
- `env.insert(name, scheme)` - 插入变量绑定
- `env.lookup(name)` - 查找变量
- `env.free_vars()` - 获取环境中的自由变量

**示例**:

```rust
let mut env = TypeEnvironment::new();

// 全局作用域
env.insert("x".to_string(),
    TypeScheme::monomorphic(Type::Int)).unwrap();

// 进入函数作用域
env.enter_scope();
env.insert("y".to_string(),
    TypeScheme::monomorphic(Type::Bool)).unwrap();

// 查找变量（会在所有作用域中搜索）
let x_scheme = env.lookup("x");  // Some(...)

// 退出作用域
env.exit_scope().unwrap();
```

---

### 4. Unifier (`unification.rs`)

类型统一算法：

```rust
pub struct Unifier {
    subst: Substitution,  // 当前替换
}
```

**主要功能**:
- `unifier.unify(ty1, ty2, span)` - 统一两个类型
- `unifier.apply(ty)` - 应用替换到类型
- `unifier.occurs(var, ty)` - Occurs check

**统一规则**:

```rust
Int ~ Int        ✓
Int ~ Bool        ✗ (类型错误)
t0 ~ Int         ✓ (记录替换 t0 -> Int)
t0 ~ t1          ✓ (记录替换 t0 -> t1 或 t1 -> t0)
[A] ~ [B]         ✓ 如果 A ~ B
(A -> B) ~ (C -> D)  ✓ 如果 A ~ C 且 B ~ D
```

**Occurs Check**:

防止无限类型：

```rust
// 错误示例
let t = Type::var(0);
// 尝试统一: t ~ [t]
// 结果: t = [t] = [[t]] = [[[t]]] = ... (无限)
// occurs check 会检测并拒绝这种情况
```

**示例**:

```rust
let mut unifier = Unifier::new();
let span = Span::dummy();

// 统一两个相同类型
unifier.unify(&Type::Int, &Type::Int, &span).unwrap();

// 统一类型变量和类型
let var = Type::Var(TypeVarRef::new(0));
unifier.unify(&var, &Type::Int, &span).unwrap();

// 检查替换
assert_eq!(unifier.get_subst().get(&0), Some(&Type::Int));
```

---

### 5. TypeChecker (`checker.rs`)

主类型检查器：

```rust
pub struct TypeChecker {
    env: TypeEnvironment,
    ctx: TypeContext,
    unifier: Unifier,
}
```

**主要功能**:
- `checker.check(stmts)` - 类型检查语句列表
- `checker.check_statement(stmt)` - 检查单个语句
- `checker.infer_expression(expr)` - 推断表达式类型
- `checker.check_pattern(pat, expected_ty)` - 检查模式

**支持的语言构造**:

| 构造 | 类型规则 |
|------|---------|
| 字面量 | 字面量本身的类型 |
| 变量 | 从环境中查找类型 |
| 二元运算 `a + b` | 若 `a: Int`, `b: Int` 则 `Int` |
| 比较运算 `a == b` | 若 `a: T`, `b: T` 则 `Bool` |
| 函数调用 `f(x)` | 若 `f: A -> B`, `x: A` 则 `B` |
| Lambda `\x -> e` | 函数类型 |
| If 表达式 | 分支类型必须相同 |
| List `[e1, e2, ...]` | 所有元素类型相同 |
| Map `{k: v}` | 键是 `String`，值类型相同 |

**示例**:

```rust
let mut checker = TypeChecker::new();

// 类型检查程序
let stmts = vec![
    Stmt::Let {
        name: "x".to_string(),
        value: Expr::Literal(Literal::Integer(42, span)),
        // ...
    },
];

checker.check(&stmts).unwrap();
```

---

### 6. Error (`error.rs`)

类型错误报告：

```rust
pub enum TypeErrorKind {
    TypeMismatch { expected: Type, found: Type },
    UndefinedVariable(String),
    DuplicateDefinition(String),
    ArityMismatch { expected: usize, found: usize },
    NotAFunction(Type),
    // ...
}
```

**错误显示**:

```
error: type mismatch: expected Int, found Bool
  --> examples/test.nm:10:15
   |
10 |     let x: Int = true
   |                ^^^^ expected Int, found Bool
```

---

## 🧪 测试

运行测试：

```bash
cargo test --package nevermind-type-checker
```

测试覆盖：

| 模块 | 测试数 | 状态 |
|------|--------|------|
| types | 8 | ✅ |
| ty (TypeVar/TypeScheme) | 6 | ✅ |
| environment | 6 | ✅ |
| unification | 7 | ✅ |
| checker | 3 | ✅ |
| **总计** | **30** | ✅ |

---

## 🔬 算法详解

### Hindley-Milner 类型推断

**算法步骤**:

1. **生成约束** - 遍历 AST，生成类型约束
2. **统一** - 使用统一算法求解约束
3. **泛化** - 在 let 绑定处泛化类型
4. **实例化** - 在变量使用处实例化类型方案

**示例**:

```nevermind
let id = fn(x) = x
in
  id(42)
```

类型推断过程：

```
1. 推断 id 的定义:
   - x: t0 (新类型变量)
   - 函数体: t0
   - id: t0 -> t0

2. 泛化:
   - id: ∀a. a -> a

3. 使用 id:
   - 实例化: t1 -> t1
   - 参数 42: Int
   - 统一: t1 ~ Int
   - 结果: Int

最终类型: Int
```

---

## 💡 使用示例

### 基本使用

```rust
use nevermind_type_checker::{TypeChecker, TypeEnvironment};

// 创建类型检查器
let mut checker = TypeChecker::new();

// 类型检查 AST
let result = checker.check(&stmts);

match result {
    Ok(ty) => println!("Type: {}", ty.display_name()),
    Err(errors) => {
        for error in errors {
            eprintln!("{}", error.display(Some(source)));
        }
    }
}
```

### 自定义类型环境

```rust
let mut checker = TypeChecker::new();

// 添加预定义函数
let env = checker.env();
env.insert("print".to_string(),
    TypeScheme::monomorphic(
        Type::function(vec![Type::String], Type::Unit)
    )).unwrap();

// 现在可以使用 print 函数
checker.check(&stmts).unwrap();
```

---

## 📚 相关文档

- **[Hindley-Milner 类型推断](https://en.wikipedia.org/wiki/Hindley%E2%80%Milner_type_system)**
- **[类型统一](https://en.wikipedia.org/wiki/Unification_(computer_science))**
- **[Algorithm W](https://www.youtube.com/watch?v=ILrLQFcGI7Y)**

---

## 🚧 未来改进

- [ ] 支持类型类 (Type Classes)
- [ ] 支持更高种类类型 (Higher-Kinded Types)
- [ ] 类型推断错误恢复
- [ ] 类型推导注释
- [ ] 性能优化

---

## 📝 贡献

欢迎贡献！请参阅 [CONTRIBUTING.md](../../CONTRIBUTING.md)。

---

**版本**: 0.1.0
**状态**: ✅ 完整实现，30 个测试全部通过
