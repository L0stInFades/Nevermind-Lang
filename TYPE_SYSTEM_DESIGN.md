# Nevermind Type System - Detailed Specification

## Table of Contents
1. [Type Inference Algorithm](#type-inference-algorithm)
2. [Generic Types & Variance](#generic-types--variance)
3. [Trait System](#trait-system)
4. [Type Classes](#type-classes)
5. [Dependent Types](#dependent-types)
6. [Effect System](#effect-system)

---

## Type Inference Algorithm

### Hindley-Milner with Extensions

Nevermind uses a **Hindley-Milner-based type inference** algorithm with extensions for:
- Union types (`Int | String`)
- Intersection types (`Readable & Writable`)
- Literal types (`1`, `"hello"`, `true`)
- Row-polymorphic records (structural typing)

### Inference Rules

```
# Variable binding
Γ ⊢ e: τ
───────────────── (LET-POLY)
Γ ⊢ let x = e: τ

# Function abstraction
Γ, x:τ₁ ⊢ e: τ₂
─────────────────── (ABS)
Γ ⊢ fn(x) -> e: τ₁ → τ₂

# Function application
Γ ⊢ e₁: τ₁ → τ₂    Γ ⊢ e₂: τ₁
────────────────────────────── (APP)
Γ ⊢ e₁(e₂): τ₂

# If expression
Γ ⊢ e₁: Bool    Γ ⊢ e₂: τ    Γ ⊢ e₃: τ
──────────────────────────────────── (IF)
Γ ⊢ if e₁ then e₂ else e₃: τ

# Pattern matching
Γ ⊢ e: τ    Γ, p:τ ⊢ b: τᵣ
──────────────────────────────── (MATCH)
Γ ⊢ match e { p => b }: τᵣ
```

### Constraint Generation

```nevermind
# Example: Inference in action
let add = fn(a, b) -> a + b
# Generates constraints:
#   a : Num
#   b : Num
#   a + b : Num (requires both operands to be same type)
# Solution: add : forall a b. (Num a) => a -> a -> a

let numbers = [1, 2, 3]
# Generates:
#   1 : Int
#   2 : Int
#   3 : Int
#   [1, 2, 3] : List[Int]

let first = numbers[0]
# Generates:
#   numbers : List[T]
#   numbers[0] : T
# Solution: first : Int (from previous constraint)
```

### Bidirectional Type Checking

For better error messages and partial type annotations:

```nevermind
# Synthesis mode (type from expression)
let x = 42  # Synthesize: Int

# Checking mode (expression from type annotation)
let y: String = "hello"  # Check: "hello" is String

# Hybrid (both directions)
fn identity<T>(x: T) -> T
  do
    x  # Check mode: x must be T
  end
```

---

## Generic Types & Variance

### Declaration

```nevermind
# Generic function
fn map<T, U>(items: List[T], f: fn(T) -> U) -> List[U]
  do
    items.map(f)
  end

# Generic class
class Box<T>
  let value: T

  fn new(value: T) -> Box<T>
    do
      Box{value}
    end
  end

  fn get(self) -> T
    do
      self.value
    end
  end
end

# Generic type alias
type Result<T, E> = Ok<T> | Error<E>
type Parser<T> = String -> (T, String)
```

### Variance Annotations

```nevermind
# Covariant (produces T)
interface Producer<+T>
  fn produce(self) -> T
end

# Contravariant (consumes T)
interface Consumer<-T>
  fn consume(self, item: T)
end

# Invariant (produces and consumes T)
interface MutableContainer<T>
  fn get(self) -> T
  fn set(self, item: T)
end
```

### Variance Rules

```
Covariant:     A <: B  ⇒  C<A> <: C<B>
Contravariant: A <: B  ⇒  C<B> <: C<A>
Invariant:     A <: B  ⇒  C<A> and C<B> are unrelated
```

### Higher-Kinded Types

```nevermind
# Functor: higher-kinded type
type Functor<F<_>> = trait
  fn map<A, B>(self: F<A>, f: fn(A) -> B) -> F<B>
end

# Monad: higher-kinded with constraints
type Monad<M<_>> = trait
  extends Functor<M>

  fn pure<A>(x: A) -> M<A>
  fn flat_map<A, B>(self: M<A>, f: fn(A) -> M<B>) -> M<B>
end

# Usage
impl Functor<List> for List
  fn map<A, B>(self: List<A>, f: fn(A) -> B) -> List<B>
    do
      # implementation
    end
  end
end
```

---

## Trait System

### Trait Declaration

```nevermind
# Define a trait
trait Hash
  fn hash(self) -> UInt64
  fn eq(self, other: Self) -> Bool
end

trait Iterator
  type Item

  fn next(self) -> Option<Item>
end

trait Num
  fn add(self, other: Self) -> Self
  fn sub(self, other: Self) -> Self
  fn mul(self, other: Self) -> Self
  fn div(self, other: Self) -> Self

  fn from_int(n: Int) -> Self
end
```

### Trait Implementation

```nevermind
# Implement trait for type
impl Hash for Int
  fn hash(self) -> UInt64
    do
      self as UInt64
    end
  end

  fn eq(self, other: Int) -> Bool
    do
      self == other
    end
  end
end

impl Hash for String
  fn hash(self) -> UInt64
    do
      # Actual hash implementation
      compute_hash(self)
    end
  end

  fn eq(self, other: String) -> Bool
    do
      self == other
    end
  end
end

# Generic implementation with trait bounds
impl<T> Hash for List<T> where T: Hash
  fn hash(self) -> UInt64
    do
      self.fold(0, fn(acc, item) acc ^ item.hash() end)
    end
  end

  fn eq(self, other: List<T>) -> Bool
    do
      if self.len() != other.len()
        return false
      end

      for i in range(0, self.len())
        do
          if not self[i].eq(other[i])
            return false
          end
        end
      end

      return true
    end
  end
end
```

### Trait Bounds

```nevermind
# Function with trait bounds
fn hash_item<T>(item: T) -> UInt64 where T: Hash
  do
    item.hash()
  end

# Multiple trait bounds
fn compare<T>(a: T, b: T) -> Int where T: Hash + Eq + Ord
  do
    if a.eq(b)
      return 0
    elif a.less(b)
      return -1
    else
      return 1
    end
  end
end

# Lifetime-like bounds (resource management)
fn use_resource<T>(r: T) where T: Resource
  do
    r.acquire()
    r.use()
    r.release()
  end
end
```

### Dynamic Dispatch

```nevermind
# Trait objects (runtime polymorphism)
fn process_items(items: List<Box<Hash>>)
  do
    for item in items
      do
        print item.hash()  # Dynamic dispatch
      end
    end
  end
end

# Usage
let items: List<Box<Hash>> = [
  Box::new(42),
  Box::new("hello"),
  Box::new(3.14)
]
process_items(items)
```

---

## Type Classes

Type classes provide **ad-hoc polymorphism** with better inference than OOP interfaces.

### Declaration

```nevermind
# Type class: set of types supporting an operation
class Eq<T>
  fn eq(self: T, other: T) -> Bool
  fn ne(self: T, other: T) -> Bool
end

class Ord<T> extends Eq<T>
  fn compare(self: T, other: T) -> Ordering
end

class Show<T>
  fn show(self: T) -> String
end

class Monoid<T>
  fn empty() -> T
  fn append(self: T, other: T) -> T
end
```

### Instances

```nevermind
# Define instances for types
instance Eq<Int>
  fn eq(self, other) -> Bool
    do
      self == other
    end
  end

  fn ne(self, other) -> Bool
    do
      self != other
    end
  end
end

instance Monoid<String>
  fn empty() -> String
    do
      ""
    end
  end

  fn append(self, other) -> String
    do
      self + other
    end
  end
end

instance Monoid<List<T>>
  fn empty<T>() -> List<T>
    do
      []
    end
  end

  fn append<T>(self, other) -> List<T>
    do
      self + other
    end
  end
end
```

### Deriving

```nevermind
# Automatic derivation for common type classes
data Point = {x: Int, y: Int}
  derive(Eq, Ord, Show)

# Equivalent to manually writing:
instance Eq<Point>
  fn eq(self, other) -> Bool
    do
      self.x == other.x and self.y == other.y
    end
  end
end

instance Show<Point>
  fn show(self) -> String
    do
      "Point({x={self.x}, y={self.y}})"
    end
  end
end
```

### Default Implementations

```nevermind
class Eq<T>
  fn eq(self: T, other: T) -> Bool
  fn ne(self: T, other: T) -> Bool
    do
      not self.eq(other)
    end
  end
end

# Only need to implement eq, ne is automatic
instance Eq<Int>
  fn eq(self, other) -> Bool
    do
      self == other
    end
  end
  # ne uses default implementation
end
```

---

## Dependent Types

Nevermind supports **lightweight dependent types** for value-level reasoning.

### Vector Length Tracking

```nevermind
# Type-level natural numbers
type Nat = Z | S(Nat)

# Vector with length in type
struct Vec<T, len: Nat>
  data: Array<T>

  fn new() -> Vec<T, Z>
    do
      Vec{data: []}
    end
  end

  fn push(self, item: T) -> Vec<T, S<len>>
    do
      Vec{data: self.data + [item]}
    end
  end

  fn safe_get(self, index: Int) -> Option<T>
    do
      if index < len
        return Some(self.data[index])
      else
        return None
      end
    end
  end
end

# Compile-time length checking
fn concatenate<T, n: Nat, m: Nat>(
  v1: Vec<T, n>,
  v2: Vec<T, m>
) -> Vec<T, add(n, m)>
  do
    # Result length is statically known to be n + m
    v1.data + v2.data
  end
end
```

### Refinement Types

```nevermind
# Predicate types
type PositiveInt = Int where self > 0
type NonEmptyList<T> = List<T> where self.len() > 0
type SortedList<T> = List<T> where is_sorted(self)

# Function with refinement type
fn sqrt(x: PositiveInt) -> Float
  do
    # x is guaranteed to be positive
    math.sqrt(x as Float)
  end
end

fn first<T>(list: NonEmptyList<T>) -> T
  do
    # Safe! List is guaranteed non-empty
    list[0]
  end
end

# Type checking with refinements
let x: PositiveInt = 5  # OK
let y: PositiveInt = -1  # Compile error!

let numbers: NonEmptyList<Int> = [1, 2, 3]  # OK
let empty: NonEmptyList<Int> = []  # Compile error!
```

### Propositions as Types

```nevermind
# Type-level propositions
type True = {}
type False = {}

# Type-level equality
type Eq<T, U> = True | False

# Prove equality at compile time
fn append_assoc<T>(
  xs: List<T>,
  ys: List<T>,
  zs: List<T>
) -> Eq<
  (xs ++ ys) ++ zs,
  xs ++ (ys ++ zs)
>
  do
    # Proof that (xs ++ ys) ++ zs == xs ++ (ys ++ zs)
    # Compiler verifies this holds
    True
  end
end
```

---

## Effect System

Track side effects in the type system without monad transformers.

### Effect Syntax

```nevermind
# Effect annotations
fn pure_function(x: Int) -> Int  # No effects
  do
    x * 2
  end
end

fn io_function() -> String raises IOError
  do
    read_file("data.txt")
  end
end

fn stateful_function() -> Int ref State
  do
    modify_state()
  end
end

fn async_function() -> Result async
  do
    fetch_data()
  end
end
```

### Effect Types

```nevermind
# Built-in effects
effect IO
effect Exception
effect State
effect Async
effect NonDeterminism

# Composed effects
fn complex_function()
  -> Result
  raises IOError
  async
  ref State
  do
    # Can do I/O, raise exceptions, be async, modify state
    let data = async fetch_from_network()
    let parsed = parse(data)?
    modify_state(parsed)
    return parsed
  end
end
```

### Effect Handlers

```nevermind
# Handle effects with custom interpreters
fn handle_with_logging<T>(comp: fn() -> T raises IOException) -> T
  do
    try
      do
        print "Starting operation"
        let result = comp()
        print "Operation succeeded"
        return result
      end
    catch IOException as e
      do
        print "Operation failed: {e}"
        raise e
      end
  end
end

# Custom effect handler
effect State
  fn get() -> String
  fn set(value: String)

fn run_state<T>(comp: fn() -> T raises State, initial: String) -> T
  do
    let state = ref initial

    try
      do
        return comp()
      end
    catch State.get()
      do
        return *state
      end
    catch State.set(value)
      do
        state = value
      end
  end
end
```

### Effect Inference

```nevermind
# Effects are inferred by default
fn inferred(x: Int)
  do
    print x  # IO effect inferred
    let y = read_line()  # IO effect inferred
    return y
  end
end

# Explicit effect annotation for documentation
fn documented(x: Int) -> Int raises IOError
  do
    let data = read_file("config.txt")?
    return x + data.parse_int()?
  end
end
```

---

## Algebraic Data Types

### Sum Types

```nevermind
# Enum-like types
type Color
  = Red
  | Green
  | Blue
  | Custom(String)

type Option<T>
  = Some(T)
  | None

type Result<T, E>
  = Ok(T)
  | Error(E)

# Pattern matching
fn describe_color(c: Color) -> String
  do
    match c
      Red => "It's red"
      Green => "It's green"
      Blue => "It's blue"
      Custom(name) => "It's custom: {name}"
    end
  end
end
```

### Product Types

```nevermind
# Tuple types (anonymous products)
type Point3D = (Float, Float, Float)

let origin: Point3D = (0.0, 0.0, 0.0)

# Named tuples (records)
type Person = {name: String, age: Int, city: String}

let alice: Person = {
  name: "Alice",
  age: 30,
  city: "NYC"
}

# Access fields
fn get_name(person: Person) -> String
  do
    person.name
  end
end
```

### Recursive Types

```nevermind
# Recursive data structures
type List<T>
  = Nil
  | Cons(T, Box<List<T>>)

type Tree<T>
  = Leaf
  | Node(T, Box<Tree<T>>, Box<Tree<T>>)

type BTree
  = Empty
  | Branch(Int, BTree, BTree)

# Functions over recursive types
fn tree_sum(tree: Tree<Int>) -> Int
  do
    match tree
      Leaf => 0
      Node(value, left, right) =>
        value + tree_sum(*left) + tree_sum(*right)
    end
  end
end

# Generic recursive functions
fn tree_fold<T, U>(
  tree: Tree<T>,
  leaf: fn() -> U,
  node: fn(T, U, U) -> U
) -> U
  do
    match tree
      Leaf => leaf()
      Node(value, left, right) =>
        node(
          value,
          tree_fold(*left, leaf, node),
          tree_fold(*right, leaf, node)
        )
    end
  end
end
```

### GADTs (Generalized Algebraic Data Types)

```nevermind
# Type-safe expressions
type Expr<T>
  = Lit(Int) -> Expr<Int>
  | Bool(Bool) -> Expr<Bool>
  | Add(Expr<Int>, Expr<Int>) -> Expr<Int>
  | If(Expr<Bool>, Expr<T>, Expr<T>) -> Expr<T>

# Type-safe evaluation
fn eval<T>(expr: Expr<T>) -> T
  do
    match expr
      Lit(n) => n as T
      Bool(b) => b as T
      Add(e1, e2) => (eval(e1) + eval(e2)) as T
      If(cond, then_branch, else_branch) =>
        if eval(cond)
          eval(then_branch)
        else
          eval(else_branch)
    end
  end
end

# Usage: type-safe by construction
let expr = Add(
  Lit(5),
  If(Bool(true), Lit(10), Lit(20))
)
# Type: Expr<Int>

# This wouldn't compile:
# let bad = Add(Lit(5), Bool(true))  # Type error!
```

---

## Type Coercion & Casts

### Safe Coercions

```nevermind
# Numeric promotions (safe)
let x: Int8 = 42
let y: Int64 = x  # Safe: widening conversion

# Subtype coercion
let animal: Animal = Dog{name: "Buddy"}  # Dog is subtype of Animal

# Nullable coercion
let name: String = "Alice"
let maybe_name: Option<String> = Some(name)  # Safe wrapping
```

### Unsafe Casts

```nevermind
# Explicit unsafe cast (requires keyword)
let ptr = unsafe_cast<IntPtr>(address)

# Checked cast (runtime check)
let value: Any = 42
if let int_value = value as Int
  do
    print "Got integer: {int_value}"
  end
end

# Forced cast (may panic)
let number = forced_cast<Int>(value)
```

---

## Module-level Types

### Abstract Types

```nevermind
# Module: counter.nm
module Counter
  # Abstract type (hidden implementation)
  type Counter

  # Constructor
  fn new(initial: Int) -> Counter
    do
      Counter{value: initial}
    end
  end

  # Operations
  fn increment(self: Counter)
  fn get(self: Counter) -> Int
end

# Implementation is opaque to users
# Users can only create and use Counter via provided functions
```

### Type Exports

```nevermind
# Export type with public constructor
export type Point = {x: Float, y: Float}

# Export type with private constructor (abstract)
export type opaque Stack = ...

# Export type alias
export type UserId = Int
export type Timestamp = Int64
```

---

## Type-level Programming

### Type Families

```nevermind
# Type-level functions
family Elem: List -> Type where
  Elem<List<Int>> = Int
  Elem<List<String>> = String
  Elem<List<T>> = T

# Type-level conditionals
family IsList: Type -> Bool where
  IsList<List<T>> = True
  IsList<_> = False

# Type-level arithmetic
family Add: Nat * Nat -> Nat where
  Add<Z, n> = n
  Add<S<m>, n> = S<Add<m, n>>
```

### Type-level Computation

```nevermind
# Compute at compile time
type Matrix<T, rows: Nat, cols: Nat>

fn multiply<T, r: Nat, c: Nat, m: Nat>(
  m1: Matrix<T, r, c>,
  m2: Matrix<T, c, m>
) -> Matrix<T, r, m>
  do
    # Result dimensions checked at compile time
    # If c != inner dimension, compile error!
    matrix_multiply(m1, m2)
  end
end

# Type-level fixed-size arrays
type Array<T, size: Nat>

fn append<T, n: Nat, m: Nat>(
  a: Array<T, n>,
  b: Array<T, m>
) -> Array<T, add(n, m)>
  do
    # Length is computed at compile time
    a.concat(b)
  end
end
```

---

## Summary

Nevermind's type system provides:

1. **Strong static typing** with full type inference
2. **Parametric polymorphism** with variance annotations
3. **Ad-hoc polymorphism** via type classes
4. **Subtype polymorphism** via traits
5. **Algebraic data types** with pattern matching
6. **Dependent types** for value-level reasoning
7. **Effect system** for side-effect tracking
8. **Higher-kinded types** for abstractions over type constructors

The type system is designed to be **powerful yet accessible**, catching bugs at compile time while maintaining the "2-hour mastery" principle through smart defaults and excellent inference.

---

*Type System Design Specification v1.0*
