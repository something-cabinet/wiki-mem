---
title: Design Patterns Reference
type: reference
tags: [reference, design-patterns, oop, ddd, cdd]
---

# Design Patterns Reference

Comprehensive reference covering the 22 classic GoF design patterns, DDD tactical patterns, OOP/SOLID principles, and CDD patterns. Source: [refactoring.guru](https://refactoring.guru/design-patterns).

## Classification

Patterns are categorized by intent:

| Group | Purpose | Count |
|---|---|---|
| **Creational** | Object creation mechanisms, increasing flexibility and reuse | 5 |
| **Structural** | How to assemble objects/classes into larger structures | 7 |
| **Behavioral** | Algorithms and assignment of responsibilities between objects | 10 |

---

## Creational Patterns

### Factory Method
**Intent:** Define an interface for creating an object, but let subclasses decide which class to instantiate.

**Problem:** A class can't anticipate the class of objects it must create. Adding new types requires modifying existing code.

**Solution:** Replace direct constructor calls with a factory method in a superclass. Subclasses override the method to return different product types. All products share a common interface.

**Use when:** You don't know the exact types and dependencies of objects beforehand; you want to provide a way for users to extend internal components; you want to reuse existing objects instead of rebuilding them.

**Relations:** Factory Method is a specialization of Template Method. Many designs evolve from Factory Method toward Abstract Factory, Prototype, or Builder.

### Abstract Factory
**Intent:** Create families of related product objects without specifying their concrete classes.

**Problem:** A system needs to work with multiple product families, but shouldn't depend on concrete implementations.

**Solution:** Define an interface for creating each distinct product. Each concrete factory implements creation for a specific variant. Client code uses only the abstract factory interface.

**Use when:** Your code needs to work with various families of related products; you want to provide a library of products without exposing implementation details.

**Relations:** Often based on a set of Factory Methods. Can use Prototype instead.

### Builder
**Intent:** Construct complex objects step by step, allowing different representations of the same construction process.

**Problem:** An object requires many initialization parameters, some optional, with complex construction logic.

**Solution:** Extract object construction code out of its own class and move it to separate builder objects. The builder constructs the object step by step via a common interface. A director class defines the order of construction steps.

**Use when:** Constructing objects with many optional components; you want to create different representations of the same object.

**Relations:** Builder constructs objects step by step, while Abstract Factory returns the product immediately.

### Prototype
**Intent:** Clone existing objects without coupling to their concrete classes.

**Problem:** Copying objects requires knowing their classes, violating encapsulation.

**Solution:** Declare a common clone method on a prototype interface. Each concrete class implements cloning by copying its own fields.

**Use when:** The classes to instantiate are determined at runtime; you want to avoid building a class hierarchy of factories; instances can have only a few combinations of state.

**Relations:** Prototype doesn't require inheritance like Factory Method, but needs complicated initialization of cloned objects.

### Singleton
**Intent:** Ensure a class has only one instance and provide a global access point to it.

**Problem:** Some resources (database connections, file systems) should have exactly one instance. Regular constructors always return new objects.

**Solution:** Make the constructor private. Create a static method that calls the private constructor once and caches the instance. All subsequent calls return the cached instance.

**Use when:** A class must have exactly one instance accessible by all clients; you need stricter control over global variables.

**Rust equivalent:** `OnceLock<T>`, `lazy_static!`, or `std::sync::OnceLock`.

---

## Structural Patterns

### Adapter
**Intent:** Allow incompatible interfaces to work together.

**Problem:** An existing class provides the functionality you need but with a different interface.

**Solution:** Create an adapter class that wraps the adaptee and implements the target interface. Client code calls the adapter's methods, which delegate to the adaptee.

**Use when:** You want to use an existing class with an incompatible interface; you want to create a reusable class that cooperates with unrelated classes.

**Rust equivalent:** Newtype wrapping + `From`/`Into` trait implementations.

### Bridge
**Intent:** Decouple an abstraction from its implementation so the two can vary independently.

**Problem:** Inheritance binds abstraction to implementation permanently. Changing either affects the other.

**Solution:** Split the class into two hierarchies: abstraction and implementation. The abstraction holds a reference to the implementation and delegates to it.

**Use when:** You want to avoid a permanent binding between abstraction and implementation; both should be extensible independently.

**Rust equivalent:** Trait objects (`Box<dyn Trait>`) to separate interface from implementation.

### Composite
**Intent:** Compose objects into tree structures to represent part-whole hierarchies.

**Problem:** Client code must treat leaf and container objects differently, increasing complexity.

**Solution:** Define a common interface for both simple and complex objects. Containers delegate work to their children via the interface.

**Use when:** You have a tree structure of objects; you want clients to treat individual and composite objects uniformly.

**Rust equivalent:** `enum Node { Leaf(...), Branch(Vec<Node>) }` with shared behavior via trait.

### Decorator
**Intent:** Attach new behaviors to objects by placing them inside wrapper objects that contain the behaviors.

**Problem:** Static inheritance adds behavior to all instances at compile time. You need runtime, per-instance behavior.

**Solution:** Create a decorator class that implements the same interface as the component and holds a reference to it. The decorator adds behavior before/after delegating.

**Use when:** You need to add responsibilities dynamically; you want to avoid subclassing for every combination.

**Rust equivalent:** Middleware pattern — function composition with `Box<dyn Fn>`, tower-rs `Layer`/`Service`.

### Facade
**Intent:** Provide a simplified interface to a complex subsystem.

**Problem:** Client code must interact with many classes, increasing coupling.

**Solution:** Create a facade class that provides a simple interface to the complex subsystem. Client code interacts only with the facade.

**Use when:** You want a simple entry point to a complex system; you want to layer your subsystems.

**Rust equivalent:** A module's `mod.rs` that re-exports only the public API, hiding internal modules. `pub use` pattern.

### Flyweight
**Intent:** Share common parts of state between multiple objects to save memory.

**Problem:** Many fine-grained objects consume too much memory by duplicating identical intrinsic state.

**Solution:** Extract intrinsic (shared) state into flyweight objects. Store extrinsic (context-specific) state outside. Flyweight factory returns existing flyweight objects, creating new ones only when needed.

**Use when:** Application uses a large number of objects with shared state; most object state can be made extrinsic.

**Rust equivalent:** `Arc<T>` sharing, string interning, `Rc<T>` for shared immutable data.

### Proxy
**Intent:** Provide a substitute or placeholder for another object to control access to it.

**Problem:** Direct access to an object is expensive, requires permission, or should be deferred.

**Solution:** Create a proxy class with the same interface as the real subject. The proxy controls access, caching, or lazy initialization.

**Variants:** Virtual proxy (lazy loading), Protection proxy (access control), Remote proxy (network stub), Logging proxy.

**Rust equivalent:** `Arc<Mutex<T>>` for concurrent access control.

---

## Behavioral Patterns

### Chain of Responsibility
**Intent:** Pass requests along a chain of handlers, each deciding to process or forward.

**Problem:** A request's handler should be determined at runtime based on conditions.

**Solution:** Transform handlers into objects with a common interface. Each handler decides to process the request or pass it to the next handler in the chain.

**Use when:** More than one handler may process a request; you don't know which handler should handle a request in advance.

**Rust equivalent:** Middleware stacks in web frameworks, iterator adapters, error handling with `?` operator.

### Command
**Intent:** Encapsulate a request as an object, parameterizing clients with different requests.

**Problem:** Need to parameterize objects with operations, queue operations, or support undo/redo.

**Solution:** Define a command interface with an `execute` method. Concrete commands store all needed parameters. A invoker triggers commands; commands can be queued or logged for undo.

**Use when:** You need to parameterize objects with actions; you need queue, log, or undo operations.

**Rust equivalent:** Closures (`Box<dyn Fn>`) stored in a Vec for undo stack, or enum dispatch.

### Iterator
**Intent:** Provide a way to access elements of a collection sequentially without exposing its representation.

**Problem:** Different collections need different traversal logic, but client code shouldn't depend on internal structures.

**Solution:** Extract traversal into separate iterator objects that implement a common interface.

**Use when:** You want to hide a collection's internal structure; you need multiple traversal ways.

**Rust equivalent:** The `Iterator` trait — `next()`, `into_iter()`, adapter methods. Built into the language.

### Mediator
**Intent:** Reduce coupling between communicating objects by introducing a mediator object.

**Problem:** Many objects communicate directly, creating a tangled web of dependencies.

**Solution:** Create a mediator that encapsulates the interaction logic. Objects notify the mediator instead of communicating directly.

**Use when:** Communication between components is complex and hard to reuse; you want to centralize control logic.

**Rust equivalent:** Channels (`mpsc`, `broadcast`), event buses, actor frameworks (actix, riker).

### Memento
**Intent:** Capture and externalize an object's internal state without violating encapsulation, for later restoration.

**Problem:** Saving state snapshots requires accessing private fields, breaking encapsulation.

**Solution:** The originator creates a memento containing a snapshot of its state. The caretaker stores mementos but never modifies them. The originator restores from a memento when needed.

**Use when:** You need undo/rollback; snapshots should not expose internal details.

**Rust equivalent:** Serialize state to a value (`serde_json::Value`), store in a Vec for undo history.

### Observer
**Intent:** Define a one-to-many dependency so that when one object changes state, all dependents are notified.

**Problem:** An object needs to notify other objects about state changes without knowing who or how many they are.

**Solution:** The publisher maintains a list of subscribers with a common notification interface. When an event occurs, the publisher iterates and notifies all subscribers.

**Use when:** Changes to one object require changing others; the set of dependent objects is unknown or dynamic.

**Rust equivalent:** `tokio::sync::broadcast`, event emitters, `Arc<watch::Receiver<T>>`.

### State
**Intent:** Allow an object to alter its behavior when its internal state changes.

**Problem:** Object behavior depends on its state, leading to large conditional statements.

**Solution:** Extract state-specific behavior into separate state classes. The context delegates to a current state object. States can transition the context to other states.

**Use when:** Object behavior depends on its state and must change at runtime; state-specific logic fills many conditionals.

**Rust equivalent:** Enum dispatch with state machine encoded in types (typestate pattern).

### Strategy
**Intent:** Define a family of algorithms, encapsulate each, and make them interchangeable.

**Problem:** A class has many variants of an algorithm selected by conditionals.

**Solution:** Extract each algorithm into its own class implementing a common strategy interface. The context delegates work to the current strategy.

**Use when:** You need different variants of an algorithm; you want to isolate algorithm implementation from its usage.

**Rust equivalent:** Trait objects (`Box<dyn Strategy>`), closures, function pointers.

### Template Method
**Intent:** Define the skeleton of an algorithm in a base class, letting subclasses override specific steps.

**Problem:** Two classes share the same algorithm structure but differ in specific steps.

**Solution:** Implement the algorithm skeleton once in a base class. Declare steps that vary as abstract or overridable methods. Subclasses implement only the varying steps.

**Use when:** You want to let subclasses extend only particular parts of an algorithm; you have several classes with nearly identical algorithms.

**Rust equivalent:** Default trait methods with associated types, `impl Trait for Type`.

### Visitor
**Intent:** Separate algorithms from the objects they operate on, allowing new operations without modifying the objects.

**Problem:** Adding new operations to a stable class hierarchy requires modifying every class.

**Solution:** Define a visitor interface with visit methods for each element type. Elements accept a visitor, calling the appropriate visit method. New operations mean new visitors, not new element code.

**Use when:** You need to perform operations on all elements of a complex object structure; the class hierarchy is stable but operations change frequently.

**Rust equivalent:** Enum dispatch with pattern matching on variants.

---

## DDD — Domain-Driven Design Tactical Patterns

### Entity
An object with a distinct identity that runs through time and different states. Not defined by its attributes but by a thread of continuity.

**Rust:** Struct with an `id: Uuid` or `id: String` field. `PartialEq` implemented by id only.

### Value Object
An immutable object defined by its attributes. Two value objects with the same attributes are interchangeable.

**Rust:** Struct with `PartialEq` derived on all fields. Prefer `#[derive(Clone, Copy)]` for small VOs.

### Aggregate
A cluster of associated objects treated as a unit for data changes. One entity is the root, responsible for enforcing invariants. External objects reference the aggregate root only.

**Rust:** Root struct owns child entities/values. Repository loads/saves the aggregate atomically.

### Repository
A mechanism for encapsulating storage, retrieval, and search behavior, emulating a collection of objects. Mediates between the domain and data mapping layers.

**Rust:** Trait with methods like `find_by_id()`, `save()`, `delete()`. Implemented for each aggregate root. See `PageRepo` pattern in this codebase.

### Domain Service
A stateless object that implements business logic that doesn't naturally fit within an Entity or Value Object. Operates on multiple aggregates.

**Rust:** Struct with methods that take repositories as parameters. Stateless, holds dependencies only.

### Domain Event
Something that happened in the domain that other parts of the system should know about. Published by aggregates, consumed by subscribers.

**Rust:** Struct with timestamp and event data. Published via `tokio::sync::broadcast` or an event bus.

### Factory
Encapsulates complex creation logic for aggregates and entities, separate from the domain objects themselves.

**Rust:** Functions or structs that encapsulate creation. Often a module-level function: `pub fn create(...) -> Result<Aggregate, Error>`.

### Specification
Predicate-like object that determines whether an object satisfies some criteria. Encapsulates business rules.

**Rust:** Function or trait with a method returning `bool`. Combinable with `and`, `or`, `not`.

---

## OOP Principles & SOLID

### S — Single Responsibility Principle (SRP)
A class should have only one reason to change. Each module/class/function should be responsible for a single part of the functionality.

**Signal:** When you struggle to name the class concisely, it likely has too many responsibilities.

### O — Open/Closed Principle (OCP)
Software entities should be open for extension but closed for modification. Add new behavior without changing existing code.

**Rust strategy:** Traits with default implementations, generic parameters, strategy pattern, `Box<dyn Trait>`.

### L — Liskov Substitution Principle (LSP)
Subtypes must be substitutable for their base types without altering the correctness of the program.

**Rust:** Since Rust doesn't have classical inheritance, this applies to trait implementations: a `impl Trait for T` must honor `Trait`'s contracts.

### I — Interface Segregation Principle (ISP)
Clients should not be forced to depend on interfaces they don't use. Prefer small, focused traits over large, monolithic ones.

**Rust:** Split large traits into smaller ones. Use trait bounds with `where` clauses to compose.

### D — Dependency Inversion Principle (DIP)
High-level modules should not depend on low-level modules. Both should depend on abstractions. Abstractions should not depend on details.

**Rust:** Depend on traits, not concrete implementations. Use `Box<dyn Trait>` or generics `T: Trait`.

### Additional OOP Principles

| Principle | Description |
|---|---|
| **Encapsulation** | Bundle data with methods. Hide internal state, expose only necessary operations. |
| **Composition over Inheritance** | Favor composing behaviors from smaller objects over class inheritance hierarchies. |
| **Program to an Interface** | Code against abstractions (traits) not concrete implementations. |
| **Tell, Don't Ask** | Tell objects what to do rather than querying their state and making decisions externally. |
| **Law of Demeter** | Only talk to your immediate friends. Don't chain method calls across multiple objects. |

---

## CDD — Compiler-Driven Development

CDD is a development workflow that uses the compiler as the primary feedback loop, equivalent to TDD's Red/Green/Refactor but for type systems.

### Core Loop

```
Compile error? → Improve the code.
Compiled OK?   → Improve the model (types).
```

### Techniques

| Technique | Description | Example |
|---|---|---|
| **Make Invalid States Unrepresentable** | Model your domain so illegal states cannot compile. | `enum PageStatus { Todo, Done }` instead of `String` |
| **Newtype Wrappers** | Wrap primitives in single-field structs with constructors. | `struct UserId(String)` instead of `String` |
| **Typestate Pattern** | Encode state machine transitions in types. | `struct NewFile; struct ValidatedFile; struct File { state: S }` |
| **Enum Dispatch** | Replace conditional branches with enum variant matching. | `match self { Page::Task {..} => .., Page::Spec {..} => .. }` |
| **Result Types** | Return `Result<T, E>` instead of throwing exceptions. | Errors are values, checked at compile time. |
| **Option over Null** | Use `Option<T>` instead of null/nil references. | Compiler forces None handling. |
| **Type-Level State Machines** | Encode allowed transitions in the type system. | `can_transition_to(&self, to: &PageStatus) -> Result<(), String>` |

### CDD in Practice

1. **Write the types first** — Define the data structures that model your domain. Make illegal states unrepresentable.
2. **Let the compiler guide you** — Unused fields, missing pattern matches, wrong type signatures.
3. **Refine types** — When patterns don't fit, adjust the model. The compiler catches everything.
4. **Add logic last** — Only after types compile cleanly, implement business logic.

---

## Pattern Relationships Summary

```
                    ┌─────────────┐
                    │ Template    │
                    │ Method      │
                    └──────┬──────┘
                           │ specialization
                    ┌──────┴──────┐
                    │ Factory     │
                    │ Method      │
                    └──────┬──────┘
                           │ evolves into
              ┌────────────┼────────────┐
              │            │            │
       ┌──────┴──────┐ ┌──┴───┐ ┌─────┴─────┐
       │ Abstract    │ │      │ │  Builder  │
       │ Factory     │ │ Proto│ │           │
       └─────────────┘ └──────┘ └───────────┘

  ┌──────┐    ┌─────────┐    ┌──────────┐
  │Bridge│    │ Strategy│    │  State   │
  └──────┘    └─────────┘    └──────────┘
  All based on composition (delegation to interface)

  ┌─────────┐   ┌──────────┐   ┌──────────┐
  │Command  │   │ Strategy │   │ Observer │
  └─────────┘   └──────────┘   └──────────┘
  Parameterize  Same struct   One-to-many
  operations     different     notification
                 algorithms

  ┌─────────┐   ┌──────────┐
  │Adapter  │   │ Decorator│
  └─────────┘   └──────────┘
  Makes things   Adds behavior
  compatible     transparently
```

### Quick Reference

| Pattern | Type | Intent |
|---|---|---|
| Factory Method | Creational | Subclass decides which class to instantiate |
| Abstract Factory | Creational | Family of related products |
| Builder | Creational | Step-by-step construction |
| Prototype | Creational | Clone existing objects |
| Singleton | Creational | Single instance, global access |
| Adapter | Structural | Match interfaces |
| Bridge | Structural | Decouple abstraction from implementation |
| Composite | Structural | Tree structures, uniform treatment |
| Decorator | Structural | Add behaviors dynamically |
| Facade | Structural | Simplified interface to subsystem |
| Flyweight | Structural | Share common state to save memory |
| Proxy | Structural | Control access to another object |
| Chain of Resp. | Behavioral | Pass request along handlers |
| Command | Behavioral | Encapsulate request as object |
| Iterator | Behavioral | Sequential access to elements |
| Mediator | Behavioral | Reduce coupling between objects |
| Memento | Behavioral | Save/restore state without exposing internals |
| Observer | Behavioral | Notify dependents of state changes |
| State | Behavioral | Alter behavior when state changes |
| Strategy | Behavioral | Family of interchangeable algorithms |
| Template Method | Behavioral | Skeleton algorithm with overridable steps |
| Visitor | Behavioral | Separate algorithm from object structure |