# LUCIA GUI Framework Contract

**Status: Proposed.** This document defines the first visible desktop milestone and a minimal subsequent app-content contract for evaluation, not an accepted implementation specification. The product direction is to build a LUCIA GUI framework for LUCIA desktop components and apps, replacing GTK as the intended product foundation. Apps provide typed descriptions of their content plus app state and actions. LUCIA provides the desktop frame and taskbar, window decorations, widget layout and drawing, view composition, and all desktop graphics handling. Apps do not render pixels or surfaces and do not process desktop graphics. The framework is distinct from a compositor/display server. The architecture below proposes a LUCIA language for frontend and backend code and a local RPC protocol, but does not approve or finalize them. The compositor, long-term display protocol, compiler/runtime design, renderer implementation, text shaper, packaging details, and accessibility API also remain open.

## First Visible Milestone: LUCIA-Owned Desktop Frame

Before app-content or window demos, the first visible implementation milestone is a LUCIA-owned desktop frame: a restrained background and bottom taskbar rendered by LUCIA across the target output. The taskbar contains only LUCIA identity and one static workspace indicator. It has no application window/content, simulated system status, or unimplemented controls. It is not Weston's fixture panel and is not content submitted by an app. This bounded prototype is evidence for the desktop-frame milestone only; it is not a full framework, compositor, desktop shell, or app-content implementation.

**Acceptance criterion:** The LUCIA-owned background fills the compositor-configured output and its bottom taskbar bounds match that output. Weston uses its kiosk shell with no Weston panel, and no app window or app-submitted content is shown. Output dimensions are derived from compositor configure events rather than fixed to one display size. The visual treatment and implementation technology remain provisional.

## Subsequent App-Content Slice

After the desktop-frame milestone, an application using the framework can:

- Submit a typed description for one top-level window and its content, including a vertical layout, text label, and button.
- Provide app state and declare an action for the button; receive a semantic activation event and publish updated state/content.
- Expose accessible label, role, and name as content metadata.
- Express dimensions in scaling-friendly logical units rather than physical pixels.
- Report startup, runtime, and shutdown lifecycle events and receive errors.

LUCIA owns the window frame and decorations, layout, widget drawing, composition, pointer and keyboard interpretation, focus order and visible focus, and the graphics work needed to present the view. Apps own their content description, state, and action behavior. The app/framework interface carries typed content and semantic action/lifecycle/error events; it does not expose a drawing API or require apps to handle raw desktop input or graphics.

This slice establishes basic client behavior only. It does not imply a full widget set or settle how applications are structured.

## Proposed Application Architecture

The following is a concrete direction for evaluation, not an approved language, compiler, runtime, or protocol specification:

- Use one LUCIA language and compiler for both frontend and backend code. Give the language distinct declarative UI component modules and backend service modules, with explicit effect permissions for service capabilities.
- Compile frontend and backend modules into separate OCI container artifacts. An app may have one or more backend services. The frontend describes semantic content, state, and behavior; LUCIA owns the renderer and session/service broker. Neither frontend nor backend containers draw pixels, access graphics APIs, or submit surfaces.
- Compose small UI components inside the frontend artifact instead of making every label, button, or internal component a container or service. Container boundaries are for app and backend isolation, permissions, and lifecycle, not individual widgets.
- Keep app containers unprivileged and service-isolated. Do not pass compositor sockets, graphics devices, host devices, host D-Bus, or privileged broker/compositor interfaces into them. Apps use no JavaScript frontend or rendering path.

## Proposed Engine MVP Contract

**Status: Proposed, implementation-neutral.** This section defines observable engine behavior for evaluation; it does not select a language implementation, runtime, renderer/toolkit, IPC transport, database, or model provider. The product direction remains a custom LUCIA language for UI and backend modules, support for one or more backend services per app, semantic app content and behavior, and LUCIA ownership of the renderer, background, and taskbar. Apps do not submit desktop pixels or use JavaScript.

### Responsibilities and Component Contract

- The LUCIA engine owns the desktop frame, background and taskbar, semantic-tree validation, layout, rendering and presentation, input interpretation, focus, and delivery of declared semantic events. Only LUCIA communicates with the compositor or handles desktop graphics.
- An app publishes a versioned semantic tree, not pixels or drawing commands. The MVP tree supports stable node identity, container/layout nodes, text, buttons, logical dimensions, and accessible role/name/label metadata. The engine rejects malformed or unsupported trees as typed app errors without replacing the last valid view.
- A component owns its declared state and handles only its declared events. For each event, it computes a state transition and publishes the resulting semantic tree; the engine presents that tree. Engine-generated activation includes the target node and event identity. State ownership, event ordering, and tree replacement versus incremental updates must be consistent within a running app instance.
- Components have observable `created`, `active`, and `stopped` lifecycle states. Events are delivered only while active; shutdown stops new events, then reports completion or failure. Lifecycle notifications are engine-owned and cannot be forged by app content.

### Service Broker and Call Contract

- An app may declare one or more backend services with typed operations. The broker routes only between that app instance and its declared services; it enforces caller and service identity plus the capabilities granted for each operation. Calls without an applicable grant are denied before dispatch. Apps and services receive no implicit host, graphics, device, or arbitrary-network access.
- Every operation has a typed request and either a typed reply or a typed stream. A request has a correlation identity and deadline; expiry stops waiting and requests cancellation of work. Explicit cancellation is propagated to the service and terminates the caller's wait. Cancellation is best-effort once work has begun, so callers must not assume a side effect was rolled back.
- A stream delivers ordered typed items, then exactly one terminal success or error. Cancellation closes delivery; implementations bound buffering and apply backpressure rather than silently dropping items. Calls report distinguishable validation, authorization, unavailable-service, deadline, cancellation, and service-failure outcomes. No implicit retry may duplicate a non-idempotent operation.
- Identity and capability checks apply to every call, including calls within the same app. The broker exposes no ambient service discovery or cross-app access. Identity issuance, capability-grant UX/representation, and enforcement mechanism remain open decisions.

### Process, Model, and Failure Contract

- The engine starts only declared app/backend workloads and assigns each running app instance an identity boundary. Workloads are unprivileged and isolated from compositor/graphics interfaces and host resources. Backend readiness is observable before calls are accepted.
- Normal shutdown stops new work, signals active components/services to stop, and enforces a bounded grace period before reporting forced termination. Startup failure, crash, timeout, and forced termination are visible to the app/session; pending calls fail with a typed outcome. Restart/backoff policy and persistent-data behavior are separate open decisions, not implicit engine guarantees.
- Model access is an ordinary explicitly granted backend capability behind a provider-neutral typed boundary. The first path is a deterministic mock with fixed, testable replies and streams; it needs no provider credential or external network. Live providers are outside this MVP contract.
- Model output is untrusted data. It cannot mutate the semantic tree, change capabilities, or invoke actions directly. The app validates it against its typed state/operation contract. Any sensitive or consequential action requires a distinct, explicit user confirmation before execution.

### Observability, Testing, and MVP Acceptance

- The engine exposes structured lifecycle, call, stream, cancellation, deadline, authorization, and failure events correlated to an app instance and operation. Diagnostics must not expose credentials; prompt/response payload capture is off by default.
- Contract tests cover valid and invalid trees, state/event transitions, focus and semantic activation, capability denial, one-way isolation between app identities, request/reply and stream termination, deadlines, cancellation, service failure, and shutdown. Mock-model tests assert deterministic output, validation of untrusted data, and confirmation gating. Tests do not require a live provider.
- The initial vertical slice reuses the current LUCIA-owned taskbar and current app-component scenario (status text and an action button) as semantic content, with one declared mock backend and a deterministic mock-LLM stream. Activating the action requests the stream, updates app state from validated output, and republishes semantic content; a separate sensitive action remains blocked until explicit confirmation. The existing direct-rendering prototype is test evidence only and does not satisfy this engine contract.
- MVP acceptance requires the slice to run with LUCIA drawing both taskbar and app content; no app receives pixels, graphics access, or JavaScript execution. It must demonstrate typed-tree rejection while retaining the last valid view, event-driven state updates, broker identity/capability enforcement, ordered streaming, deadline and cancellation propagation, typed failure and shutdown reporting, deterministic mock behavior, and user confirmation before a sensitive action. The checks must be repeatable without a live model provider or physical display.

### Explicit Non-Goals and Open Decisions

The MVP does not build a compositor/display server, define a complete widget library or accessibility API, support arbitrary app networking, permit apps to render pixels, select a production model provider, or settle app-to-app communication. It does not choose the compiler/runtime, rendering toolkit, IPC transport, database, identity/capability mechanism, tree update encoding, exact lifecycle state machine, timeout defaults, retry/restart policy, or persistence model. These remain proposals for evidence and explicit review, not approved decisions.

### Proposed Backend Communication

For local app-to-backend calls, evaluate typed interface declarations in LUCIA mapped to versioned Protobuf schemas, with generated bindings and gRPC over Unix-domain sockets (UDS). The LUCIA-owned broker would expose only explicitly granted service endpoints; this proposal does not include arbitrary app networking or service discovery. The transport is a candidate, not a decision.

Keep request/response calls, one-way events, and server streams as distinct interface forms. Calls should carry deadlines and cancellation and return typed status/errors. Server streaming can carry LLM tokens and other ordered events, with defined cancellation and backpressure behavior. Version schemas deliberately and generate frontend/backend bindings from LUCIA interface declarations rather than maintaining parallel handwritten contracts.

This favors typed contracts and local process isolation, but adds compiler/binding work and RPC operational complexity. UDS confines this proposal to local services; remote-provider access, if permitted, belongs behind an explicitly authorized LUCIA-owned service rather than direct app connectivity. The exact schema evolution, authentication/capability enforcement, and stream flow-control rules remain to be evaluated.

### Early LLM Integration

Treat LLM support as an early framework integration, exercised with a deterministic mock before any live provider. The proposal is a LUCIA-owned `ModelProvider` backend service with a swappable local or remote provider implementation and streamed responses through the typed service interface. Provider credentials stay with the provider service and are never placed in app containers.

Model output is untrusted data or a proposal: it cannot directly mutate LUCIA's UI tree, grant capabilities, or invoke privileged actions. The app validates outputs against its own typed state and behavior; sensitive or consequential actions require explicit user confirmation. Provider access, privacy policy, and any remote connectivity remain open decisions.

### Proposed MVP Stages

1. Review the language/module split, effect permissions, typed interface shape, and isolation model. Keep the compiler, runtime, and protocol choices provisional until this review is explicit.
2. Specify the semantic component tree and broker boundary, then evaluate a small frontend component and one separately packaged backend service using a deterministic local response. Preserve LUCIA ownership of all rendering and desktop graphics.
3. Use a LUCIA taskbar component and a minimal app component with typed state/action behavior as the first end-to-end example; connect the app to its backend over the candidate local interface.
4. Add a mock `ModelProvider` stream to that flow, checking schema validation, cancellation/error handling, credential isolation, and confirmation before sensitive actions. Only after this works should provider implementations and broader component/service coverage be considered.

These stages describe a proposed evaluation path; they do not authorize or specify compiler/runtime implementation in this document.

## Illustrative API

The following language-neutral pseudocode illustrates the responsibility split under test. Names, types, and signatures are not a selected API:

```text
app = Application()
content = WindowContent(
	title="Example",
	logical_size=(480, 320),
	body=Column(
		Label(text=app.state.status, accessible_name="Status"),
		Button(text="Run", accessible_name="Run", action="run")
	)
)

app.on_action("run", () => {
	app.state.status = "Activated"
	app.publish(content.with_status(app.state.status))
})
app.publish(content)
app.run()
```

LUCIA renders the described window and controls, interprets pointer and keyboard input, manages focus, and sends the declared `run` action to the app when activated. The app updates its state and publishes the revised content; it does not draw the updated view. Startup, runtime, and shutdown failures must be reportable to the app; exact error types and recovery behavior remain open.

## Client and Container Boundary

Each app frontend and each backend service runs in an unprivileged OCI container and communicates through typed interfaces mediated by LUCIA. The LUCIA rendering service, not the app, communicates with the compositor and handles graphics needed to present app content. App and backend containers must not receive a compositor surface/socket, seatd access, DRM or other host devices, host D-Bus, or a rootful `containerd` socket. Apps provide semantic content through the framework; there is no JavaScript frontend or app-owned rendering path. The candidate UDS/gRPC transport and broker placement remain proposals, not decisions about desktop-session topology or the final compositor/protocol.

## Evaluation

1. First, verify the desktop-frame acceptance criterion on the target display: the LUCIA fullscreen surface fills the configured output, the bottom taskbar is within that frame, Weston uses its kiosk shell with no desktop panel, and no app window or content is shown.
2. After that milestone, run an app in an unprivileged OCI container with a LUCIA rendering service and a separate headless compositor; only the LUCIA service receives the private compositor client socket. Verify typed content presentation, LUCIA-managed pointer and keyboard activation and focus, semantic action delivery, app state/content updates, scaling-friendly logical dimensions, lifecycle reporting, and errors. This is a test arrangement, not a long-term Wayland protocol or compositor selection.
3. The physical Weston kiosk-shell fixture is used only to present the background prototype on hardware; its desktop shell and panel are not visible. Existing Weston/GTK launcher evidence remains limited to that experimental integration fixture and does not establish framework behavior or select Weston.
4. Decide the final compositor and display protocol separately, based on reviewed evidence and an explicit decision.

## Prototype Evidence (2026-11-02)

The physical presentation in this prototype is the LUCIA-owned fullscreen
background and bottom taskbar from
[`desktop-container/gui-prototype`](../../desktop-container/gui-prototype/README.md)
using a separate C/Wayland/Cairo renderer that requests a fullscreen surface,
uses compositor-configured dimensions, and draws a taskbar with only LUCIA
identity and one workspace indicator. The physical runner checks the downloaded
Weston runtime's advertised shell option and kiosk-shell module, then requires a
fullscreen configure, matching bottom-taskbar bounds, and presented-frame
marker. Weston runs with that kiosk shell, not its desktop panel. This
demonstrates only the LUCIA-owned desktop frame on one output; it does not
demonstrate a framework, app composition, or production desktop session.

Separately, the provisional C/Wayland/Cairo window client in
[`desktop-container/gui-prototype`](../../desktop-container/gui-prototype/README.md)
is used only by the headless window regression. The background/taskbar client is
also built and run by the headless check at a 1024 by 768 output size, with
fullscreen, bottom-taskbar, static-shell-element, and presented-frame markers.
It builds from Ubuntu archive
packages and runs against the Weston headless fixture; it is not shown on the
physical display and does not launch an app. The separate window client draws its
own content and communicates with Wayland directly, so it does not implement
the app/LUCIA responsibility or container boundary clarified in this proposal.
Its smoke check covers a configured and presented 480 by 320
logical-unit window, vertically arranged text and button, visible focus,
keyboard and pointer activation handler paths, status updates, client-side
accessible metadata, lifecycle markers, and the private client/compositor
container boundary. Weston headless advertises no input seat, so the activation
check calls the same event handlers in-process; this is not compositor-injected
or physical input validation. Core Wayland does not expose the prototype's
accessible metadata to assistive technology. See the prototype matrix for all
coverage and gaps. This is evaluation evidence only, not a client-rendered or
hybrid architecture recommendation: GTK remains an experimental fixture, and
no app/framework language or protocol, renderer implementation, compositor, or
display protocol is selected.

## Non-Goals

- Building a custom compositor or display server.
- Building a graphics driver or low-level input/seat broker.
- Launching actual applications as part of the taskbar milestone.
- Treating the GTK launcher fixture as the GUI framework implementation. It remains an experimental integration fixture.

## Open Decisions

The proposal does not settle the LUCIA language syntax or type/effect system, compiler/runtime architecture, exact UI module model, final app/framework interface protocol or serialization format, broker placement, service capability and lifecycle rules, renderer implementation, text shaping, packaging details, complete scope, accessibility API, GTK compatibility, compositor, or long-term display protocol. Protobuf with gRPC over UDS is a candidate for evaluation, not an approved protocol. The ModelProvider provider policy, privacy rules, and local-versus-remote support also require explicit decisions. Do not treat any proposed choice in this document as final without that review.