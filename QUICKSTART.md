# Nevermind Quick Start

## 5-Minute Introduction to Nevermind

Nevermind is a programming language designed for **zero cognitive friction**. It's like Python, but cleaner, safer, and more powerful.

---

## Hello World

```nevermind
fn main()
  do
    print "Hello, World!"
  end
```

That's it! No semicolons, no curly braces, no `public static void`.

---

## Variables

```nevermind
# Immutable by default
let name = "Alice"
let age = 30

# Mutable when you need it
var score = 0
score = score + 1
```

**Why?** Most variables shouldn't change. Make immutability the default, mutation explicit.

---

## Functions

```nevermind
fn add(a: Int, b: Int) -> Int
  do
    a + b
  end

# Call it
let result = add(5, 3)  # 8
```

**Clean syntax**: No `return` keyword needed (last expression is returned).

---

## Conditionals

```nevermind
if age >= 18
  do
    print "Adult"
  end
else
  do
    print "Minor"
  end
```

**Reads like English**: "If age is greater than or equal to 18, then..."

---

## Lists and Operations

```nevermind
let numbers = [1, 2, 3, 4, 5]

# Map
let doubled = numbers.map |n| n * 2 |
# [2, 4, 6, 8, 10]

# Filter
let evens = numbers.filter |n| n % 2 == 0 |
# [2, 4]

# Reduce/Fold
let sum = numbers.fold(0, |acc, n| acc + n)
# 15
```

**Functional by default**: No `for` loops needed for common operations.

---

## The Pipeline Operator

```nevermind
# Write data transformations left-to-right (natural flow)
let result = numbers
  |> filter |n| n > 2 |
  |> map |n| n * 2 |
  |> sum

# Instead of:
# let result = sum(map(filter(numbers, |n| n > 2|), |n| n * 2|))
```

**Why?** Data flows left-to-right, matching how we think.

---

## Pattern Matching

```nevermind
match result
  Ok(value) => print "Success: {value}"
  Error(err) => print "Error: {err}"
end

match number
  0 => "Zero"
  1 | 2 | 3 => "Small"
  _ => "Large"
end
```

**Exhaustive checking**: Compiler ensures you handle all cases.

---

## Option & Result Types

```nevermind
# Option (no nulls!)
let maybe_name: Option[String] = Some("Alice")
match maybe_name
  Some(name) => print "Got: {name}"
  None => print "Nothing"
end

# Result (explicit error handling)
fn divide(a: Int, b: Int) -> Result[Int, String]
  do
    if b == 0
      return Error("Division by zero")
    end
    return Ok(a / b)
  end

# Use it
match divide(10, 2)
  Ok(result) => print "Result: {result}"
  Error(err) => print "Error: {err}"
end
```

**No null pointer exceptions**: `null` doesn't exist, use `Option` instead.

---

## Async (Implicit!)

```nevermind
# No 'await' keyword - it's automatic!
fn fetch_data(url: String) -> String
  do
    let response = http_get(url)  # Automatically awaited
    return response.body
  end

# Just call it normally
let data = fetch_data("https://api.example.com")
```

**Zero async complexity**: The compiler detects and handles async for you!

---

## Parallel Processing

```nevermind
# Run multiple operations in parallel
let (result1, result2, result3) = parallel
  (fetch(url1),
   fetch(url2),
   fetch(url3))
```

**Easy parallelism**: No manual thread management.

---

## Type Inference

```nevermind
# Types are inferred automatically
let number = 42           # Int
let name = "Alice"       # String
let numbers = [1, 2, 3]  # List[Int]

# Optional: specify types for clarity
let count: Int = 42
let items: List[String] = ["a", "b", "c"]
```

**Best of both worlds**: Safety of static types, convenience of dynamic types.

---

## Python Interoperability

```nevermind
# Use Python libraries directly
use "pandas"
use "numpy"

let df = pandas.DataFrame({"a": [1, 2, 3]})
let arr = numpy.array([1, 2, 3, 4])
```

**Full compatibility**: Access the entire Python ecosystem.

---

## Complete Example

```nevermind
use "pandas"

fn process_data(filename: String) -> Int
  do
    let df = pandas.read_csv(filename)

    let result = df
      |> filter |row| row["age"] > 18 |
      |> map |row| transform(row) |
      |> map |row| row["score"] |
      |> sum

    return result
  end

fn main()
  do
    let total = process_data("data.csv")
    print "Total score: {total}"
  end
```

---

## Key Takeaways

1. **Minimal punctuation**: No semicolons, no curly braces
2. **Natural syntax**: Reads like English
3. **Immutable by default**: `let` vs `var`
4. **Functional operations**: `.map()`, `.filter()`, `.fold()`
5. **Pipeline operator**: `|>` for data flow
6. **Pattern matching**: `match` expressions
7. **Implicit async**: No `await` needed
8. **Strong typing**: With full inference
9. **Python interop**: Use Python libraries directly

---

## Next Steps

1. **Install** Nevermind: `cargo build --release`
2. **Read** the [full documentation](README.md)
3. **Try** the [example programs](examples/): `nevermind run examples/hello.nm`
4. **Join** the community

---

**Nevermind: Forget the syntax, remember the algorithm.**
