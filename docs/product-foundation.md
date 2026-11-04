# LUCIA Product Foundation

## Product Workspace

Product implementation lives in `product/`, independently of the experiments
under `desktop-container/`. The first workspace member is the `lucia-engine`
library. It owns typed semantic content contracts; it does not depend on or
carry forward prototype code. Product changes and product validation use the
workspace rooted at `product/Cargo.toml`.

## Naming Rules

- Rust crate names use `lucia-<purpose>`.
- Rust modules and source files use `snake_case`.
- Types, traits, and enums use `PascalCase`.
- Functions and methods use `snake_case`.
- Tests are named for the behavior they verify.
- Product code does not add compatibility aliases, shims, or parallel legacy
  paths. There is no compatibility requirement for the experimental code.

## Current Scope

`lucia-engine` defines a typed semantic tree with stable node and action
identities, containers and layout, text, buttons, accessible metadata, optional
logical dimensions, and snapshot validation. Validation enforces nonempty
identities and accessible labels, unique node IDs within a snapshot, matching
semantic roles, and positive dimensions. Publishers retain an ID when
publishing the same logical node in later snapshots; this library validates a
single snapshot and does not claim to verify continuity over time.

LUCIA owns rendering and desktop graphics. App content is semantic data and
declared actions, not pixels or JavaScript. This initial library does not
implement rendering, input, actions at runtime, app execution, or backend
communication.

## Deferred Decisions

Rust is selected for the compiler/engine host, and this first crate establishes
the product workspace. The following remain outside this foundation slice:

- LUCIA language syntax, type/effect system, compiler stages, and generated
  interface bindings.
- WebAssembly frontend execution, sandbox boundaries, and runtime lifecycle.
- Broker design, capability approval and grants, service identity, and RPC
  transport/schema choices.
- Renderer/toolkit, compositor, display protocol, layout behavior beyond the
  semantic contract, and accessibility integration.
- OCI backend packaging, ModelProvider implementation, and assistant/agent
  runtime behavior.

These are not selected or implemented by the workspace foundation.