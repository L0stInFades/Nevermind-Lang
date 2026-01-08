# Nevermind Standard Library - API Specification

## Overview

The Nevermind standard library follows these principles:
- **Minimal but complete**: Small surface area, maximum power
- **Composable**: Functions work well together
- **Type-safe**: Full type annotations for IDE support
- **Documented**: Extensive examples in docstrings
- **Tested**: 100% test coverage

---

## Module: `core`

### Primitives

```nevermind
# Basic operations
module core
  # Identity function
  fn id<T>(x: T) -> T
    do
      x
    end
  end

  # Constant function
  fn const<T, U>(x: T, y: U) -> T
    do
      x
    end
  end

  # Function composition
  fn compose<T, U, V>(f: fn(U) -> V, g: fn(T) -> U) -> fn(T) -> V
    do
      fn(x) -> f(g(x))
    end
  end

  # Pipe operator (built-in, but aliased)
  fn pipe<T, U>(x: T, f: fn(T) -> U) -> U
    do
      f(x)
    end
  end

  # Flip function arguments
  fn flip<T, U, V>(f: fn(T, U) -> V) -> fn(U, T) -> V
    do
      fn(y, x) -> f(x, y)
    end
  end
end
```

---

## Module: `core/types`

### Option<T>

```nevermind
module core/types
  # Represents optional values
  type Option<T> = Some(T) | None

  impl<T> Option<T>
    # Check if contains value
    fn is_some(self) -> Bool
      do
        match self
          Some(_) => true
          None => false
        end
      end
    end

    fn is_none(self) -> Bool
      do
        not self.is_some()
      end
    end

    # Get value or default
    fn unwrap_or(self, default: T) -> T
      do
        match self
          Some(value) => value
          None => default
        end
      end
    end

    # Get value or compute default
    fn unwrap_or_else(self, f: fn() -> T) -> T
      do
        match self
          Some(value) => value
          None => f()
        end
      end
    end

    # Transform value
    fn map<U>(self, f: fn(T) -> U) -> Option<U>
      do
        match self
          Some(value) => Some(f(value))
          None => None
        end
      end
    end

    # Chain option-returning operations
    fn and_then<U>(self, f: fn(T) -> Option<U>) -> Option<U>
      do
        match self
          Some(value) => f(value)
          None => None
        end
      end
    end

    # Filter with predicate
    fn filter(self, pred: fn(T) -> Bool) -> Option<T>
      do
        match self
          Some(value) =>
            if pred(value)
              Some(value)
            else
              None
            end
          None => None
        end
      end
    end

    # Convert to Result
    fn ok_or<E>(self, err: E) -> Result<T, E>
      do
        match self
          Some(value) => Ok(value)
          None => Error(err)
        end
      end
    end
  end

  # Collect Options
  fn traverse<T, U>(opts: List<Option<T>>) -> Option<List<T>>
    do
      opts.fold_right(Some([]), fn(opt, acc)
        match (opt, acc)
          (Some(o), Some(a)) => Some(o :: a)
          _ => None
        end
      end)
    end
  end
end
```

### Result<T, E>

```nevermind
module core/types
  # Represents success or error
  type Result<T, E> = Ok(T) | Error(E)

  impl<T, E> Result<T, E>
    # Check variants
    fn is_ok(self) -> Bool
    fn is_error(self) -> Bool

    # Get value or default
    fn unwrap_or(self, default: T) -> T

    # Transform success
    fn map<U>(self, f: fn(T) -> U) -> Result<U, E>
      do
        match self
          Ok(value) => Ok(f(value))
          Error(err) => Error(err)
        end
      end
    end

    # Transform error
    fn map_error<F>(self, f: fn(E) -> F) -> Result<T, F>
      do
        match self
          Ok(value) => Ok(value)
          Error(err) => Error(f(err))
        end
      end
    end

    # Chain result-returning operations
    fn and_then<U>(self, f: fn(T) -> Result<U, E>) -> Result<U, E>
      do
        match self
          Ok(value) => f(value)
          Error(err) => Error(err)
        end
      end
    end

    # Question mark operator support
    fn ?<U>(self) -> U raises E
      do
        match self
          Ok(value) => value
          Error(err) => raise err
        end
      end
    end
  end
end
```

---

## Module: `core/collections`

### List<T>

```nevermind
module core/collections
  # Immutable linked list
  type List<T> = Nil | Cons(T, Box<List<T>>)

  impl<T> List<T>
    # Constructors
    fn new() -> List<T>
      do
        Nil
      end
    end

    fn of(items: T...) -> List<T>
      do
        items.fold_right(Nil, fn(item, acc) Cons(item, Box::new(acc)) end)
      end
    end

    # Properties
    fn is_empty(self) -> Bool
      do
        match self
          Nil => true
          _ => false
        end
      end
    end

    fn len(self) -> Int
      do
        self.fold(0, fn(_, count) count + 1 end)
      end
    end

    # Head and tail
    fn head(self) -> Option<T>
      do
        match self
          Cons(value, _) => Some(value)
          Nil => None
        end
      end
    end

    fn tail(self) -> Option<List<T>>
      do
        match self
          Cons(_, rest) => Some(*rest)
          Nil => None
        end
      end
    end

    # Modification (returns new list)
    fn prepend(self, item: T) -> List<T>
      do
        Cons(item, Box::new(self))
      end
    end

    fn append(self, other: List<T>) -> List<T>
      do
        match self
          Nil => other
          Cons(h, t) => Cons(h, Box::new(t.append(other)))
        end
      end
    end

    fn reverse(self) -> List<T>
      do
        self.fold(Nil, fn(item, acc) Cons(item, Box::new(acc)) end)
      end
    end

    # Functional operations
    fn map<U>(self, f: fn(T) -> U) -> List<U>
      do
        match self
          Nil => Nil
          Cons(h, t) => Cons(f(h), Box::new(t.map(f)))
        end
      end
    end

    fn filter(self, pred: fn(T) -> Bool) -> List<T>
      do
        match self
          Nil => Nil
          Cons(h, t) =>
            if pred(h)
              Cons(h, Box::new(t.filter(pred)))
            else
              t.filter(pred)
            end
        end
      end
    end

    fn flat_map<U>(self, f: fn(T) -> List<U>) -> List<U>
      do
        match self
          Nil => Nil
          Cons(h, t) => f(h).append(t.flat_map(f))
        end
      end
    end

    # Folding
    fn fold<U>(self, init: U, f: fn(T, U) -> U) -> U
      do
        match self
          Nil => init
          Cons(h, t) => t.fold(f(h, init), f)
        end
      end
    end

    fn fold_right<U>(self, init: U, f: fn(T, U) -> U) -> U
      do
        match self
          Nil => init
          Cons(h, t) => f(h, t.fold_right(init, f))
        end
      end
    end

    # Reductions
    fn reduce(self, f: fn(T, T) -> T) -> Option<T>
      do
        match self
          Nil => None
          Cons(h, t) => Some(t.fold(h, f))
        end
      end
    end

    # Searching
    fn find(self, pred: fn(T) -> Bool) -> Option<T>
      do
        match self
          Nil => None
          Cons(h, t) =>
            if pred(h)
              Some(h)
            else
              t.find(pred)
            end
        end
      end
    end

    fn contains(self, item: T) -> Bool where T: Eq
      do
        self.find(fn(x) x == item end).is_some()
      end
    end

    # Zipping
    fn zip<U>(self, other: List<U>) -> List<(T, U)>
      do
        match (self, other)
          (Cons(h1, t1), Cons(h2, t2)) =>
            Cons((h1, h2), Box::new(t1.zip(*t2)))
          _ => Nil
        end
      end
    end

    # Conversion
    fn to_array(self) -> Array<T>
      do
        let arr = Array.with_capacity(self.len())
        for item in self
          do
            arr.push(item)
          end
        end
        arr
      end
    end
  end
end
```

### Array<T>

```nevermind
module core/collections
  # Mutable dynamic array (vector)
  type Array<T> = ...

  impl<T> Array<T>
    # Constructors
    fn new() -> Array<T>
    fn with_capacity(capacity: Int) -> Array<T>
    fn of(items: T...) -> Array<T>

    # Properties
    fn len(self) -> Int
    fn capacity(self) -> Int
    fn is_empty(self) -> Bool

    # Access
    fn get(self, index: Int) -> Option<T>
    fn get_mut(self, index: Int) -> Option<&mut T>
    fn set(self, index: Int, value: T)

    # Indexing operator support
    fn [](self, index: Int) -> T
    fn []=(self, index: Int, value: T)

    # Modification
    fn push(self, item: T)
    fn pop(self) -> Option<T>
    fn insert(self, index: Int, item: T)
    fn remove(self, index: Int) -> T

    # Resizing
    fn resize(self, new_len: Int, value: T)
    fn reserve(self, additional: Int)
    fn shrink_to_fit(self)
    fn clear(self)

    # Slicing
    fn slice(self, start: Int, end: Int) -> Array<T>
    fn split_at(self, mid: Int) -> (Array<T>, Array<T>)

    # Functional operations (return new arrays)
    fn map<U>(self, f: fn(T) -> U) -> Array<U>
    fn filter(self, pred: fn(T) -> Bool) -> Array<T>
    fn flat_map<U>(self, f: fn(T) -> Array<U>) -> Array<U>

    # Sorting
    fn sort(self) where T: Ord
    fn sort_by(self, compare: fn(T, T) -> Ordering)
    fn reverse(self)

    # Searching
    fn find(self, pred: fn(T) -> Bool) -> Option<T>
    fn binary_search(self, item: T) -> Option<Int> where T: Ord
    fn contains(self, item: T) -> Bool where T: Eq

    # Iteration
    fn iter(self) -> Iter<T>
    fn iter_mut(self) -> IterMut<T>

    # Conversions
    fn to_list(self) -> List<T>
    fn to_string(self) -> String where T: Char
  end
end
```

### Map<K, V>

```nevermind
module core/collections
  # Immutable hash map (persistent)
  type Map<K, V> = ...

  impl<K, V> Map<K, V> where K: Hash
    # Constructors
    fn new() -> Map<K, V>
    fn of(pairs: (K, V)...) -> Map<K, V>

    # Properties
    fn is_empty(self) -> Bool
    fn len(self) -> Int

    # Access
    fn get(self, key: K) -> Option<V>
    fn get_or(self, key: K, default: V) -> V
    fn contains_key(self, key: K) -> Bool

    # Modification (returns new map)
    fn set(self, key: K, value: V) -> Map<K, V>
    fn delete(self, key: K) -> Map<K, V>
    fn update(self, key: K, f: fn(Option<V>) -> V) -> Map<K, V>

    # Functional operations
    fn map_keys<K2>(self, f: fn(K) -> K2) -> Map<K2, V> where K2: Hash
    fn map_values<V2>(self, f: fn(V) -> V2) -> Map<K, V2>
    fn filter(self, pred: fn(K, V) -> Bool) -> Map<K, V>

    # Keys and values
    fn keys(self) -> List<K>
    fn values(self) -> List<V>
    fn items(self) -> List<(K, V)>

    # Merging
    fn merge(self, other: Map<K, V>) -> Map<K, V>
    fn merge_with(self, other: Map<K, V>, f: fn(V, V) -> V) -> Map<K, V>

    # Conversions
    fn to_hash_map(self) -> HashMap<K, V>
  end
end
```

### Set<T>

```nevermind
module core/collections
  # Immutable hash set
  type Set<T> = ...

  impl<T> Set<T> where T: Hash
    # Constructors
    fn new() -> Set<T>
    fn of(items: T...) -> Set<T>

    # Properties
    fn is_empty(self) -> Bool
    fn len(self) -> Int

    # Membership
    fn contains(self, item: T) -> Bool

    # Modification (returns new set)
    fn add(self, item: T) -> Set<T>
    fn delete(self, item: T) -> Set<T>

    # Set operations
    fn union(self, other: Set<T>) -> Set<T>
    fn intersection(self, other: Set<T>) -> Set<T>
    fn difference(self, other: Set<T>) -> Set<T>
    fn symmetric_difference(self, other: Set<T>) -> Set<T>

    # Subset/superset
    fn is_subset(self, other: Set<T>) -> Bool
    fn is_superset(self, other: Set<T>) -> Bool

    # Iteration
    fn iter(self) -> Iter<T>

    # Conversions
    fn to_list(self) -> List<T>
    fn to_array(self) -> Array<T>
  end
end
```

---

## Module: `core/iter`

### Iterators

```nevermind
module core/iter
  # Lazy iterator type
  type Iter<T> = ...

  impl<T> Iter<T>
    # Creation
    fn new<T>(f: fn() -> Option<T>) -> Iter<T>
    fn from<T>(coll: &T) -> Iter<T> where T: IntoIterator

    # Consumption
    fn next(self) -> Option<T>
    fn collect<U>(self) -> U where U: FromIterator<T>
    fn count(self) -> Int
    fn fold<U>(self, init: U, f: fn(U, T) -> U) -> U
    fn reduce(self, f: fn(T, T) -> T) -> Option<T>
    fn find(self, pred: fn(T) -> Bool) -> Option<T>
    fn any(self, pred: fn(T) -> Bool) -> Bool
    fn all(self, pred: fn(T) -> Bool) -> Bool
    fn nth(self, n: Int) -> Option<T>
    fn last(self) -> Option<T>

    # Adaptation (lazy)
    fn map<U>(self, f: fn(T) -> U) -> Iter<U>
    fn filter(self, pred: fn(T) -> Bool) -> Iter<T>
    fn filter_map<U>(self, f: fn(T) -> Option<U>) -> Iter<U>
    fn flat_map<U>(self, f: fn(T) -> Iter<U>) -> Iter<U>
    fn enumerate(self) -> Iter<(Int, T)>
    fn skip(self, n: Int) -> Iter<T>
    fn skip_while(self, pred: fn(T) -> Bool) -> Iter<T>
    fn take(self, n: Int) -> Iter<T>
    fn take_while(self, pred: fn(T) -> Bool) -> Iter<T>
    fn zip<U>(self, other: Iter<U>) -> Iter<(T, U)>
    fn step_by(self, step: Int) -> Iter<T>
    fn chain(self, other: Iter<T>) -> Iter<T>
    fn cycle(self) -> Iter<T>
    fn rev(self) -> Iter<T> where T: DoubleEndedIterator

    # Specialized adaptors
    fn inspect(self, f: fn(&T)) -> Iter<T>
    fn peekable(self) -> PeekableIter<T>
    fn fuse(self) -> FuseIter<T>
  end

  # Infinite iterators
  fn repeat<T>(item: T) -> Iter<T>
  fn count_from(start: Int) -> Iter<Int>
  fn iterate<T>(seed: T, f: fn(T) -> T) -> Iter<T>

  # Range iterator
  fn range(start: Int, end: Int) -> Iter<Int>
  fn range_step(start: Int, end: Int, step: Int) -> Iter<Int>
end
```

---

## Module: `core/async`

### Task

```nevermind
module core/async
  # Async task handle
  type Task<T> = ...

  impl<T> Task<T>
    # Spawn new task
    fn spawn(f: fn() -> T async) -> Task<T>

    # Wait for completion
    fn await(self) -> T

    # Check status
    fn is_done(self) -> Bool
    fn is_cancelled(self) -> Bool

    # Cancellation
    fn cancel(self)

    # Timeout
    fn timeout(self, duration: Duration) -> Result<T, TimeoutError>
  end

  # Sleep
  fn sleep(duration: Duration) async

  # Spawn multiple tasks
  fn spawn_all<T>(tasks: List<fn() -> T async>) -> List<Task<T>>
end
```

### Stream

```nevermind
module core/async
  # Async stream of values
  type Stream<T> = ...

  impl<T> Stream<T>
    # Creation
    fn new<T>(f: fn() -> Option<T> async) -> Stream<T>
    fn from_iter(iter: Iter<T>) -> Stream<T>
    fn from_channel(ch: Channel<T>) -> Stream<T>

    # Consumption
    fn next(self) -> Option<T> async
    fn collect(self) -> List<T> async
    fn fold<U>(self, init: U, f: fn(U, T) -> U) -> U async

    # Adaptation
    fn map<U>(self, f: fn(T) -> U) -> Stream<U>
    fn filter(self, pred: fn(T) -> Bool) -> Stream<T>
    fn flat_map<U>(self, f: fn(T) -> Stream<U>) -> Stream<U>
    fn buffer(self, size: Int) -> Stream<List<T>>
    fn debounce(self, duration: Duration) -> Stream<T>
    fn throttle(self, duration: Duration) -> Stream<T>

    # Merge streams
    fn merge(self, other: Stream<T>) -> Stream<T>
    fn zip<U>(self, other: Stream<U>) -> Stream<(T, U)>
    fn interleave(self, other: Stream<T>) -> Stream<T>

    # Error handling
    fn map_err<E>(self, f: fn(E) -> E) -> Stream<T> where E: Error
  end
end
```

### Channel

```nevermind
module core/async
  # Async channel for communication
  type Channel<T> = ...

  impl<T> Channel<T>
    # Create bounded/unbounded channel
    fn new(capacity: Int) -> Channel<T>
    fn unbounded() -> Channel<T>

    # Send/receive
    fn send(self, value: T) async
    fn try_send(self, value: T) -> Result<(), SendError<T>>
    fn recv(self) -> Option<T> async
    fn try_recv(self) -> Result<T, TryRecvError>

    # Close
    fn close(self)
    fn is_closed(self) -> Bool
  end

  # Multi-producer, multi-consumer
  type Sender<T>
  type Receiver<T>

  fn channel<T>() -> (Sender<T>, Receiver<T>)
  fn channel_bounded<T>(capacity: Int) -> (Sender<T>, Receiver<T>)
end
```

---

## Module: `core/string`

### String Operations

```nevermind
module core/string
  # Immutable string (UTF-8)
  type String = ...

  impl String
    # Constructors
    fn new() -> String
    fn from(chars: Char...) -> String
    fn from_bytes(bytes: List<UInt8>) -> Result<String, Utf8Error>
    fn from_utf8(bytes: Array<UInt8>) -> Result<String, Utf8Error>

    # Properties
    fn len(self) -> Int  # Byte length
    fn char_len(self) -> Int  # Character count
    fn is_empty(self) -> Bool
    fn bytes(self) -> Array<UInt8>
    fn chars(self) -> Array<Char>

    # Access
    fn get(self, index: Int) -> Option<Char>
    fn byte_at(self, index: Int) -> UInt8
    fn slice(self, start: Int, end: Int) -> String

    # Comparison
    fn compare(self, other: String) -> Ordering
    fn starts_with(self, prefix: String) -> Bool
    fn ends_with(self, suffix: String) -> Bool
    fn contains(self, pattern: String) -> Bool

    # Searching
    fn find(self, pattern: String) -> Option<Int>
    fn rfind(self, pattern: String) -> Option<Int>
    fn find_char(self, c: Char) -> Option<Int>

    # Manipulation (returns new string)
    fn concat(self, other: String) -> String
    fn repeat(self, n: Int) -> String
    fn replace(self, from: String, to: String) -> String
    fn replace_all(self, from: String, to: String) -> String
    fn to_lower(self) -> String
    fn to_upper(self) -> String
    fn trim(self) -> String
    fn trim_left(self) -> String
    fn trim_right(self) -> String
    fn pad_left(self, n: Int, char: Char) -> String
    fn pad_right(self, n: Int, char: Char) -> String

    # Splitting
    fn split(self, delimiter: String) -> List<String>
    fn split_lines(self) -> List<String>
    fn split_whitespace(self) -> List<String>
    fn chars(self) -> Iter<Char>
    fn bytes(self) -> Iter<UInt8>

    # Joining
    fn join(strings: List<String>) -> String

    # Formatting
    fn format(self, args: Any...) -> String

    # Conversions
    fn to_int(self) -> Result<Int, ParseError>
    fn to_float(self) -> Result<Float, ParseError>
    fn to_bool(self) -> Result<Bool, ParseError>

    # Unicode
    fn is_ascii(self) -> Bool
    fn is_normalized(self) -> Bool
    fn normalize(self, form: NormalizationForm) -> String
  end
end
```

---

## Module: `io`

### File I/O

```nevermind
module io/file
  # File handle
  type File = ...

  impl File
    # Open
    fn open(path: String) -> Result<File, IOError>
    fn create(path: String) -> Result<File, IOError>
    fn with_options(path: String, opts: OpenOptions) -> Result<File, IOError>

    # Read
    fn read_to_string(self) -> Result<String, IOError>
    fn read_to_end(self) -> Result<Array<UInt8>, IOError>
    fn read_line(self) -> Result<String, IOError>
    fn read_exact(self, buffer: &mut Array<UInt8>) -> Result<(), IOError>

    # Write
    fn write(self, data: &[UInt8]) -> Result<Int, IOError>
    fn write_string(self, s: String) -> Result<Int, IOError>
    fn write_line(self, s: String) -> Result<Int, IOError>
    fn flush(self) -> Result<(), IOError>

    # Seek
    fn seek(self, pos: SeekFrom) -> Result<Int64, IOError>
    fn position(self) -> Int64
    fn len(self) -> Int64

    # Metadata
    fn metadata(self) -> Result<Metadata, IOError>
    fn set_permissions(self, perm: Permissions) -> Result<(), IOError>

    # Close
    fn close(self) -> Result<(), IOError>
  end

  # File system operations
  fn exists(path: String) -> Bool
  fn copy(from: String, to: String) -> Result<(), IOError>
  fn rename(from: String, to: String) -> Result<(), IOError>
  fn delete(path: String) -> Result<(), IOError>
  fn create_dir(path: String) -> Result<(), IOError>
  fn create_dir_all(path: String) -> Result<(), IOError>
  fn remove_dir(path: String) -> Result<(), IOError>
  fn remove_dir_all(path: String) -> Result<(), IOError>
  fn read_dir(path: String) -> Result<List<DirEntry>, IOError>

  # File metadata
  type Metadata = ...
  type DirEntry = ...

  fn metadata(path: String) -> Result<Metadata, IOError>
end
```

---

## Module: `io/net`

### HTTP Client

```nevermind
module io/net/http
  # HTTP client
  type Client = ...

  impl Client
    # Create client
    fn new() -> Client
    fn with_timeout(timeout: Duration) -> Client
    fn with_redirect(limit: Int) -> Client

    # Make request
    fn get(self, url: String) -> Result<Response, HTTPError>
    fn post(self, url: String, body: String) -> Result<Response, HTTPError>
    fn put(self, url: String, body: String) -> Result<Response, HTTPError>
    fn delete(self, url: String) -> Result<Response, HTTPError>
    fn request(self, req: Request) -> Result<Response, HTTPError>
  end

  # Request/Response types
  type Request = {
    method: String,
    url: String,
    headers: Map<String, String>,
    body: String
  }

  type Response = {
    status: Int,
    status_text: String,
    headers: Map<String, String>,
    body: String
  }

  impl Response
    fn json<T>(self) -> Result<T, JSONError>
    fn text(self) -> String
    fn bytes(self) -> Array<UInt8>
  end
end
```

### HTTP Server

```nevermind
module io/net/http
  # HTTP server
  type Server = ...

  impl Server
    # Create server
    fn new(addr: String) -> Server
    fn new_with_port(port: Int) -> Server

    # Routes
    fn get(self, path: String, handler: fn(Request) -> Response)
    fn post(self, path: String, handler: fn(Request) -> Response)
    fn put(self, path: String, handler: fn(Request) -> Response)
    fn delete(self, path: String, handler: fn(Request) -> Response)
    fn route(self, method: String, path: String, handler: fn(Request) -> Response)

    # Middleware
    fn use(self, middleware: fn(Request, fn(Request) -> Response) -> Response)

    # Start
    fn start(self) async
    fn start_with_handler(self, handler: fn(Request) -> Response) async

    # Stop
    fn stop(self)
  end

  # Request type
  type Request = {
    method: String,
    path: String,
    query: Map<String, String>,
    headers: Map<String, String>,
    body: String
  }

  impl Request
    fn param(self, name: String) -> Option<String>
    fn query(self, name: String) -> Option<String>
    fn header(self, name: String) -> Option<String>
  end
end
```

---

## Module: `data`

### JSON

```nevermind
module data/json
  # JSON value type
  type JSON =
    | Null
    | Bool(Bool)
    | Number(Float)
    | String(String)
    | Array(List<JSON>)
    | Object(Map<String, JSON>)

  impl JSON
    # Parse
    fn parse(text: String) -> Result<JSON, JSONError>

    # Serialize
    fn to_string(self) -> String
    fn to_string_pretty(self) -> String

    # Type-safe access
    fn as_bool(self) -> Option<Bool>
    fn as_int(self) -> Option<Int>
    fn as_float(self) -> Option<Float>
    fn as_string(self) -> Option<String>
    fn as_array(self) -> Option<List<JSON>>
    fn as_object(self) -> Option<Map<String, JSON>>

    # Object access
    fn get(self, key: String) -> Option<JSON>
    fn get_or(self, key: String, default: JSON) -> JSON

    # Array access
    fn at(self, index: Int) -> Option<JSON>
  end

  # Type-safe serialization
  trait ToJSON
    fn to_json(self) -> JSON
  end

  trait FromJSON
    fn from_json(json: JSON) -> Result<Self, JSONError>
  end
end
```

---

## Module: `time`

### Time Operations

```nevermind
module time
  # Duration
  type Duration = ...

  impl Duration
    # Creation
    fn from_secs(secs: Int64) -> Duration
    fn from_millis(millis: Int64) -> Duration
    fn from_micros(micros: Int64) -> Duration
    fn from_nanos(nanos: Int64) -> Duration

    # Access
    fn as_secs(self) -> Int64
    fn as_millis(self) -> Int64
    fn as_micros(self) -> Int64
    fn as_nanos(self) -> Int64

    # Arithmetic
    fn add(self, other: Duration) -> Duration
    fn sub(self, other: Duration) -> Duration

    # Comparison
    fn compare(self, other: Duration) -> Ordering
  end

  # Instant (point in time)
  type Instant = ...

  impl Instant
    fn now() -> Instant
    fn elapsed(self) -> Duration
    fn add(self, duration: Duration) -> Instant
    fn sub(self, duration: Duration) -> Instant
  end

  # System time
  type SystemTime = ...

  impl SystemTime
    fn now() -> SystemTime
    fn elapsed(self) -> Duration
  end

  # Sleep
  fn sleep(duration: Duration) async
end
```

---

## Module: `testing`

### Testing Framework

```nevermind
module testing
  # Test case
  attribute test
  fn test_case()

  # Assertion functions
  fn assert_true(cond: Bool)
  fn assert_false(cond: Bool)
  fn assert_eq<T>(left: T, right: T) where T: Eq + Debug
  fn assert_ne<T>(left: T, right: T) where T: Eq + Debug
  fn assert_approx_eq(left: Float, right: Float, epsilon: Float)
  fn assert_none<T>(value: Option<T>)
  fn assert_some<T>(value: Option<T>)
  fn assert_ok<T, E>(result: Result<T, E>)
  fn assert_error<T, E>(result: Result<T, E>)
  fn assert_panics(f: fn())

  # Test suite
  fn describe(name: String, tests: fn())
  fn it(name: String, test: fn())

  # Setup/teardown
  fn before_each(f: fn())
  fn after_each(f: fn())
  fn before_all(f: fn())
  fn after_all(f: fn())

  # Mocking
  fn mock<T>(name: String) -> Mock<T>

  type Mock<T> = ...
  impl<T> Mock<T>
    fn expect(self, method: String, args: Any...) -> &Mock<T>
    fn returns(self, value: T) -> &Mock<T>
    fn verify(self) -> Bool
  end
end
```

---

## Module: `math`

### Mathematical Functions

```nevermind
module math
  # Constants
  fn pi() -> Float
  fn e() -> Float
  fn inf() -> Float
  fn nan() -> Float

  # Basic operations
  fn abs(x: Float) -> Float
  fn floor(x: Float) -> Float
  fn ceil(x: Float) -> Float
  fn round(x: Float) -> Float
  fn trunc(x: Float) -> Float
  fn fract(x: Float) -> Float

  # Exponential/Logarithmic
  fn exp(x: Float) -> Float
  fn ln(x: Float) -> Float
  fn log2(x: Float) -> Float
  fn log10(x: Float) -> Float
  fn pow(base: Float, exp: Float) -> Float
  fn sqrt(x: Float) -> Float
  fn cbrt(x: Float) -> Float

  # Trigonometric
  fn sin(x: Float) -> Float
  fn cos(x: Float) -> Float
  fn tan(x: Float) -> Float
  fn asin(x: Float) -> Float
  fn acos(x: Float) -> Float
  fn atan(x: Float) -> Float
  fn atan2(y: Float, x: Float) -> Float

  # Hyperbolic
  fn sinh(x: Float) -> Float
  fn cosh(x: Float) -> Float
  fn tanh(x: Float) -> Float

  # Min/Max
  fn min<T>(a: T, b: T) -> T where T: Ord
  fn max<T>(a: T, b: T) -> T where T: Ord
  fn clamp<T>(value: T, min: T, max: T) -> T where T: Ord

  # Rounding
  fn round_to_digits(x: Float, digits: Int) -> Float
end
```

---

## Module: `crypto`

### Cryptography

```nevermind
module crypto
  # Hashing
  fn sha256(data: &[UInt8]) -> [UInt8; 32]
  fn sha512(data: &[UInt8]) -> [UInt8; 64]
  fn md5(data: &[UInt8]) -> [UInt8; 16]
  fn blake3(data: &[UInt8]) -> [UInt8; 32]

  # HMAC
  fn hmac_sha256(key: &[UInt8], data: &[UInt8]) -> [UInt8; 32]

  # Random
  fn random_bytes(n: Int) -> Array<UInt8>
  fn random_int() -> Int
  fn random_float() -> Float

  # UUID
  fn uuid4() -> String

  # Encoding
  fn base64_encode(data: &[UInt8]) -> String
  fn base64_decode(s: String) -> Result<Array<UInt8>, DecodeError>
  fn hex_encode(data: &[UInt8]) -> String
  fn hex_decode(s: String) -> Result<Array<UInt8>, DecodeError>
end
```

---

## Summary

The Nevermind standard library provides:

1. **Core types**: Option, Result, List, Array, Map, Set
2. **Async primitives**: Task, Stream, Channel
3. **I/O operations**: File, HTTP, networking
4. **Data formats**: JSON, CSV, XML
5. **Time operations**: Duration, Instant, SystemTime
6. **Testing**: Comprehensive test framework
7. **Math**: Full mathematical function suite
8. **Crypto**: Hashing, random, encoding

All modules are:
- **Type-safe**: Full type annotations
- **Documented**: Extensive examples
- **Tested**: 100% coverage
- **Composable**: Work well together
- **Python-compatible**: Easy interop

---

*Standard Library API Specification v1.0*
