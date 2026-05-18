# Software Architect Knowledge Pack

# Influences

- Eric Evans, *Domain-Driven Design* (2003): the strategic and tactical patterns for modeling complex domains. Core ideas: ubiquitous language, bounded context, aggregate, value object, entity, repository, domain service, domain event, context map, anti-corruption layer.
- Robert C. Martin (Uncle Bob), *Clean Code*, *Clean Architecture*, *The Clean Coder*: SOLID, separation of concerns, dependency rule, screaming architecture, professional discipline, TDD as a craft.
- Martin Fowler, *Refactoring*, *Patterns of Enterprise Application Architecture*, "Microservices" article series, *bliki* on martinfowler.com: refactoring catalog, evolutionary design, integration patterns, when (and when not) to use microservices, YAGNI, "Tell, Don't Ask".
- Kent Beck, *Test-Driven Development: By Example*, *Tidy First?*: red-green-refactor, simple design rules, separating tidying from behavior change.
- Vaughn Vernon, *Implementing Domain-Driven Design*: how DDD plays out in code, integration between contexts, event-driven systems.
- Sam Newman, *Building Microservices*: when to split, how to split safely, ownership, decoupling deployment from architecture.
- Gregor Hohpe, *Enterprise Integration Patterns*: messaging, channels, routers, transformation patterns.

# SOLID (Robert C. Martin)

- **Single Responsibility Principle (SRP).** A module should have one reason to change. The "reason" is a stakeholder or actor, not a feature.
- **Open/Closed Principle (OCP).** A module should be open for extension, closed for modification. Achieved with abstractions and polymorphism, not flags.
- **Liskov Substitution Principle (LSP).** Subtypes must be usable where their base types are expected, without surprising the caller.
- **Interface Segregation Principle (ISP).** Clients should not depend on methods they do not use. Prefer many small role interfaces over one fat interface.
- **Dependency Inversion Principle (DIP).** High-level policy must not depend on low-level details. Both should depend on abstractions defined by the policy.

When violations are found, the smallest fix is usually: extract an interface owned by the policy, then move the dependency direction so it points inward.

# Clean Architecture (Uncle Bob)

Concentric layers. Dependencies point inward only.

1. **Entities / domain.** Pure business rules. No framework, no IO, no database.
2. **Use cases / application.** Orchestrate entities to perform application-specific behavior. Define ports (interfaces) for what they need from the outside.
3. **Interface adapters.** Controllers, presenters, gateways. Translate between use cases and external IO.
4. **Frameworks and drivers.** Web, DB, message broker, devices, UI. Replaceable.

Key tests:
- Could you swap the framework or database without changing entities or use cases? If not, the dependency rule is broken.
- Does the directory structure scream the domain or the framework? It should scream the domain.

# Domain-Driven Design (Eric Evans)

Strategic patterns:
- **Ubiquitous language.** Build one shared language between developers and domain experts. The code uses the same words.
- **Bounded context.** A boundary inside which a model is consistent. Different contexts may use the same word with different meanings; that is fine, name them differently across the boundary.
- **Context map.** Document how bounded contexts relate: shared kernel, customer-supplier, conformist, anti-corruption layer, open host service, published language, separate ways.
- **Core domain vs supporting vs generic.** Invest most modeling effort and best engineers in the core domain. Buy or copy the generic.

Tactical patterns:
- **Entity.** Identity matters across time. Two entities with the same attributes are not equal if their IDs differ.
- **Value object.** Identity does not matter. Equality is by attributes. Prefer immutability.
- **Aggregate.** A cluster of entities and value objects with one root. The root enforces invariants. External code only references the root. Aggregates are the unit of consistency.
- **Repository.** Returns aggregates by identity or query. Hides persistence.
- **Domain service.** Behavior that does not naturally belong to one entity or value object.
- **Domain event.** A named fact that something important happened in the domain.
- **Anti-corruption layer (ACL).** A translation layer that protects your model from a different model in another context or legacy system.

Modeling moves used in this Agent:
- Start from the use case in the user's words. Extract nouns and verbs. Cluster nouns into candidate aggregates and value objects. Verbs become methods or domain events.
- Draw the invariants first. The aggregate root must enforce them in one transaction.
- Avoid anaemic models: behavior should live with the data it operates on.

# Test-Driven Development (Kent Beck)

The cycle:
1. **Red.** Write the smallest failing test that expresses the next behavior.
2. **Green.** Write the smallest production code that passes the test, even if it is naive.
3. **Refactor.** With the bar green, improve names and structure without changing behavior.

Heuristics:
- One assertion per test where possible.
- Use the test name to describe behavior, not implementation.
- Outside-in for new features. Inside-out for tricky algorithms.
- Triangulate when one example is not enough.
- Refactor only on green. If a refactor introduces a red, revert and take a smaller step.

Beck's four rules of simple design (in order):
1. Passes the tests.
2. Reveals intent.
3. No duplication.
4. Fewest elements.

# Refactoring (Martin Fowler)

Refactoring is small, behavior-preserving change under a green test bar. Always have tests before you start.

A short list of refactoring moves the Agent should name:
- Extract Function / Inline Function.
- Rename Variable / Rename Function.
- Move Function / Move Field.
- Extract Class / Inline Class.
- Replace Conditional with Polymorphism.
- Replace Magic Number with Symbolic Constant.
- Introduce Parameter Object.
- Replace Temp with Query.
- Decompose Conditional.
- Replace Loop with Pipeline.
- Replace Type Code with Subclasses or Strategy.
- Encapsulate Variable / Encapsulate Field.
- Hide Delegate / Remove Middle Man.
- Pull Up Method / Push Down Method.

Common code smells to look for:
- Long function, large class, long parameter list.
- Feature envy, inappropriate intimacy, message chains.
- Primitive obsession, data clumps, repeated switches.
- Shotgun surgery (one change, many files), divergent change (one file, many reasons).
- Comments that explain bad code instead of fixing it.

# Evolutionary Architecture (Fowler, Ford, Parsons)

- Architecture decisions should be reversible where possible. Note one-way doors.
- Use **fitness functions**: automated checks that an architectural property still holds (e.g. dependency direction, response time budget, max coupling).
- Prefer the strangler fig pattern over rewrites: route traffic gradually from the old system to the new one as capabilities are reimplemented.
- Defer commitments. YAGNI. Avoid speculative generality.

# When NOT to Microservice

(Direct echo of Martin Fowler's repeated guidance.)

- A microservices architecture trades intra-process complexity for distributed systems complexity.
- Pre-conditions: rapid provisioning, basic monitoring, rapid deployment, devops culture, mature CI/CD.
- Default to a well-modularized monolith. Extract a service when you have a clear bounded context, an independent change rate, an operational reason (scaling profile or fault isolation), and the team capability to operate it.
- Service boundaries should follow domain boundaries discovered by DDD, not org-chart boundaries.

# Metrics-Driven Design (MDD)

A discipline: every important architectural decision is backed by a measurement that would tell us if the decision was wrong.

For each decision, define:
- **The decision.** "Use a single relational DB for orders and inventory."
- **The bet.** What we believe will be true.
- **The metric.** What you will measure (e.g. p95 write latency, deployment lead time, change failure rate, defect escape rate, code churn in the module, coupling between modules).
- **The unit and threshold.** "p95 write latency below 80 ms at 200 RPS."
- **The review cadence.** When we will look at it (per release, monthly, after a load test).
- **The trigger.** What value would force us to revisit the decision.

Useful sources of metrics:
- DORA / Accelerate: deployment frequency, lead time for changes, change failure rate, mean time to restore.
- Code-level: cyclomatic complexity, coupling between modules, code churn, hotspots (high churn + high complexity).
- Quality: defect density per module, escaped defect rate, test coverage on changed lines (not absolute coverage).
- Runtime: latency percentiles, error rate, saturation, cost per transaction.
- Product: activation rate, retention, conversion. Tie technical decisions back to a product metric where possible.

Anti-patterns:
- Vanity metrics (lines of code written, number of tickets closed, "AI productivity").
- Single-number dashboards that hide regression in tails.
- Metrics that no one is committed to act on.

# Architectural Decision Records (ADRs)

Use a short, version-controlled document per decision. Suggested fields:

1. Title.
2. Status (proposed / accepted / superseded by ADR-XYZ).
3. Context.
4. Decision.
5. Consequences (positive, negative, neutral).
6. Alternatives considered.
7. The MDD signal that would force a revisit.

Keep ADRs short. The point is the decision and its trade-offs, not prose.

# C4 Model (Simon Brown) Quick Reference

Useful for talking about a system at four zoom levels:

1. **Context.** The system in its world: users and external systems.
2. **Container.** Deployable / runnable units (web app, API, DB, queue).
3. **Component.** Major pieces inside a container.
4. **Code.** Class diagrams, only when needed.

Most teams need only the first three.

# Hexagonal / Ports and Adapters (Alistair Cockburn)

- The application defines **ports** (interfaces) it needs.
- **Adapters** implement those ports for specific technologies (HTTP, DB, queue, CLI).
- Drives testability: in tests, swap adapters for in-memory ones.
- Aligns naturally with Clean Architecture and DDD.

# CQRS and Event Sourcing — Use With Care

- CQRS separates the model used to write from the model used to read. Useful when read and write workloads diverge sharply.
- Event sourcing stores domain events as the system of record and derives state from them. Powerful but expensive in operational complexity, schema evolution, and developer cognitive load.
- Default position: do not introduce them unless you have a clear, concrete business reason and the team will operate them. Many use cases are better served by a relational store and a transactional outbox.

# Working With Legacy

- Characterization tests first: pin existing behavior with tests before you change it (Michael Feathers, *Working Effectively with Legacy Code*).
- Find seams. A seam is a place where you can change behavior without editing in place.
- Strangler fig: front the legacy with a façade, route capability by capability to the new implementation, retire legacy when traffic for that capability is zero.
- Branch by abstraction for in-process replacement of a component.

# Code Review Checklist (Architecture-First)

In order:
1. Does the change respect the dependency direction?
2. Is the bounded context clear? Is the language consistent with the model?
3. Are invariants enforced in the aggregate root?
4. Is there a test that would fail if this change broke behavior?
5. Are abstractions justified by current need or speculative?
6. Are names accurate and aligned with the ubiquitous language?
7. Will this change make the next change easier or harder?
8. Style and formatting come last.

# Output Heuristics for the Agent

- Always state assumptions before recommending.
- When recommending, end with **What we are betting on** and **How we will know we were wrong**.
- When suggesting a refactor, list the steps in safe order, and call out the test bar required first.
- Quote the principle by name (SRP, DIP, Bounded Context, Anti-Corruption Layer, Strangler Fig, YAGNI).
- Prefer the smallest first slice that produces real feedback.
