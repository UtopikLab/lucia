# Summary — Linux Desktop Environment

*Single source of truth for this exploratory session. User decisions are recorded below; remaining questions are open.*

## Scope

Design and build an integrated Linux desktop distribution for general desktop users, initially targeting modern desktops and laptops. LUCIA's own integrated desktop distribution will provisionally be based on Ubuntu. The specific Ubuntu release and whether it is LTS or non-LTS remain undecided, as do supported configurations, platform interfaces, kernel, and compatibility details. Prioritize simplicity, performance, and consistency. The intended scope includes the shell, compositor/window management, panel/launcher/notifications, settings, and core apps, with most components built in-house. LUCIA intends to build its own GUI framework to replace GTK for LUCIA desktop components and apps; GTK is not the intended product foundation. This does not decide GTK compatibility, complete framework scope, implementation language, renderer, toolkit architecture, or display backend. A GUI/client framework is distinct from a compositor/display server: this direction does not decide to replace Weston or reimplement the Wayland protocol. The entire LUCIA desktop user space is intended to be container-managed, including desktop/session components as well as apps. Linux containers share the host kernel: host boot/kernel and physical hardware are outside ordinary container contents. The exact host/user-space boundary and graphics, input, and device integration remain open. Applications launch as `containerd`-managed instances, with native-feeling integration, permission-mediated host access, and strong isolation by default. These are product directions; detailed architecture and implementation remain open.

Potential areas of investigation include the desktop shell and workflow, window management and display protocol, settings and system integration, application compatibility, accessibility, security, performance, hardware support, and maintenance. These are prompts for exploration, not a prescribed architecture.

## Background and confidence

- 🟢 **Broad orientation:** A complete desktop experience involves more than window placement; the visible shell and its integration with applications and system services affect the overall user experience.
- 🟡 **To verify for LUCIA's target distribution/platform:** Display protocols, compositor capabilities, toolkit support, portal behavior, packaging, and distribution integration vary by software and version. No specific compatibility claim has been confirmed for this session.

## First thinktank topic: one app's lifecycle

This is an exploratory walkthrough, not an architecture proposal. Confirmed user decisions are separated from questions that still need investigation; no package format, compatibility claim, or implementation choice is implied.

### Confirmed user decisions

- **Full desktop runtime direction:** The entire LUCIA desktop user space is container-managed, including desktop/session components and apps. This does not select one container or component boundaries. Linux containers share the host kernel; host boot/kernel and physical hardware are outside ordinary container contents. The exact host/user-space boundary and graphics, input, and device integration remain open.
- **Desktop startup:** The desktop must start without requiring the user to log in through a shell or TTY first. Whether LUCIA requires graphical sign-in, and what greeter or session entry point it uses, remains undecided.
- **App runtime and launch:** the app is managed as a container; `containerd` manages application processes, and launching an app creates a per-app container instance.
- **App/container versioning:** the app manager/package manager is responsible for versioning the app and its container. The exact manager boundary is not decided; this does not establish whether the app manager and package manager are one component or separate components.
- **Update behavior:** the app author determines the app's update behavior, choosing among applying the update on the next launch, a managed restart to update a running app, or allowing old and new versions to overlap. No default policy or policy override is established.
- **Host integration:** apps should feel native, with strong isolation by default and permission-mediated host access. GPU acceleration and inter-app workflows are important.
- **Permission profiles:** LUCIA must define a catalog of permission profiles or access policies; each app must declare which profile(s) it needs; on first launch, LUCIA requests the user's agreement for the app to use those declared profiles. This is a product requirement, not an implementation decision.
- **Crash:** automatically restart app containers after crashes.
- **Normal exit:** stop the container and remove the stopped instance, while preserving app data in standard host locations for later launches. App data should migrate across app updates.

### User-provided scenario direction

- **Install and launch:** The experience should feel like a desktop app; an Android-app-like feel is also acceptable and may be closer. This is a familiar, app-centric UX direction, not a decision to copy Android or its implementation.
- **Host permissions:** Android is an experience analogy only, not an implementation to copy. The profile catalog, app declarations, and first-launch request for user agreement are resolved product requirements; the profile model and exact interaction details remain open.
- **Running updates:** Detailed UX and flow are deferred because it is too soon to design them. The resolved direction that app authors select among the three update policies above remains in force.

### Open lifecycle questions

- Given `containerd` as the app container runtime foundation, what user-facing app management model best meets LUCIA's requirements and representative user scenarios? Docker-like, Kubernetes-like, desktop-native, and other systems may offer useful interaction patterns to investigate; none is a presumed fit or an exhaustive set of choices. The runtime choice does not settle this product-model question.
- What package format and installation details are needed, and what are the exact boundaries of app/package management? Keep the app/container versioning responsibility with app/package management without deciding whether the app manager and package manager are separate components.
- What constraints apply to a managed restart for an update, and under what conditions are overlapping old and new versions compatible?
- What instance lifetime and reuse behavior should launch have, and how should GPU access and inter-app communication work?
- What container topology should manage the desktop session: one desktop-session container or multiple managed components? Should apps use separate containers in the same or distinct lifecycle? These topology and lifecycle relationships remain undecided.
- What permission profiles should LUCIA define, what capabilities and granularity should each include, and how should agreement, denial, persistence, revocation, and temporary versus permanent access work? How should later requests for additional profiles and the first-launch prompt be handled?
- What restart limits, backoff, and user feedback should apply after crashes, especially repeated crashes?
- What should app removal mean for its persistent host-side data?
- How should data migrations be versioned and recovered if interrupted or unsuccessful? What controls should users have to back up, export, reset, or delete app data?

### Exploratory permission-profile sketch (proposed)

This is a candidate sketch to clarify profile purpose, capabilities, scope, and denial or revocation effects. It is not a finalized catalog or security design. Apps must declare the profiles they need, and first launch asks the user to agree to those declared profiles. That requirement does not establish that first-launch agreement replaces later task-specific consent for narrower access.

Shared metadata to consider for each declared profile: stable ID, display name, app-declared purpose, capabilities, scope, required or optional status, duration, and denial/revocation behavior. Exact fields and semantics remain open.

| Candidate profile | Purpose | Narrow candidate scope | If denied or revoked |
| --- | --- | --- | --- |
| App-private persistent data | Keep an app's own data across launches. Whether this is baseline access or consented access remains open. | Only that app's designated persistent data; exact data boundaries remain open. | The app cannot read or write the affected data; behavior for existing data and app recovery remains open. |
| User-selected files | Let a user provide or save files. | Prefer exact files or locations selected for the task, rather than broad filesystem access. | The app cannot access unselected or no-longer-authorized files; the task may be unavailable or need another selection. |
| Camera | Capture images or video for an app-declared purpose. | Camera use only, with task or duration limits still to be decided. | Capture is unavailable; an active capture should stop on revocation. |
| Microphone | Capture audio for an app-declared purpose. | Microphone use only, with task or duration limits still to be decided. | Audio capture is unavailable; an active capture should stop on revocation. |
| Notifications | Allow an app to present notifications. | The app's own notifications; categories, delivery controls, and duration remain open. | Notifications are suppressed; queued or already displayed notifications need a policy. |
| Network | Allow app network communication for a declared purpose. | Destination, protocol, and task boundaries are open, as is whether network access is default or explicitly declared. | Affected communication is blocked; app behavior and treatment of in-flight requests remain open. |
| Inter-app handoff | Transfer user-chosen content or work to another app. | Prefer an explicit, task-scoped transfer of selected content to a selected recipient app. | The transfer does not occur; revocation should prevent further transfer under that grant. |

GPU acceleration is separate from this privacy-oriented candidate list: it may be a system or performance capability and is not automatically a privacy permission. Whether and how GPU access is declared or controlled remains open.

No exact profile contents, defaults, enforcement mechanisms, prompt flow, permanent-versus-task-time grant rules, or Linux implementation are decided here. These are candidates for further design, not accepted requirements beyond the existing app-declaration and first-launch agreement requirements.

### Exploratory analogies: management interactions

**Framing note:** From the user's perspective, LUCIA's overall combination may be novel or may not map neatly to existing systems. This is a hypothesis, not a verified uniqueness claim. Design and evaluation should start from LUCIA's requirements and representative scenarios, not from matching it to an existing product. The examples below are exploratory analogies for learning from specific interaction patterns only; they are not a closed candidate set, a benchmark, or an assumption that LUCIA maps to any of these systems. They do not propose adopting Docker or Kubernetes or claim compatibility with either. Any app management experience must respect the app author's chosen update policy: update on next launch, managed restart, or overlapping versions. Benefits and tradeoffs are hypotheses, not decisions.

| Alternative | How the workflows might work | Likely benefit | Likely tradeoff |
| --- | --- | --- | --- |
| Docker-like management surface | Install and version apps through a manager; apply the author's update policy; launch apps and offer inspectable app/container status, logs, and restart controls. Show mediated permissions and explicit GPU or inter-app access alongside each app. Troubleshooting can expose instance events and recovery actions. | A visible, consistent model for versions, isolation boundaries, and diagnosis may help users understand what is running and what access it has. | Container-oriented concepts and detailed controls may burden users seeking a simple, native-feeling app experience. Exposed controls alone do not ensure good performance or integration. |
| Kubernetes-like declarative/workload management surface | Describe the desired app version and author-selected update behavior; launch and inspect the requested and current state. Represent permission grants, GPU access, and inter-app relationships as managed declarations. Show state changes, failures, and recovery progress. | A consistent account of intended versus current state may make complex updates and recovery easier to reason about, while keeping isolation and relationships explicit. | Desired-state and workload concepts may add substantial cognitive overhead for everyday desktop tasks, and could make direct app interaction feel less immediate or native. Performance still needs separate measurement. |
| Desktop-native or hybrid surface, with optional operational details | Install, update, and launch apps through familiar desktop entry points while honoring the author's update policy. Present mediated permissions in app settings or at the point of use; integrate GPU and inter-app workflows through desktop-facing controls. Offer optional status, logs, and recovery details when troubleshooting. | The everyday path may feel simplest and most native, with isolation and permission decisions presented in context; operational details remain available when needed. | Hiding detail can make versions, isolation, and failures harder to compare or diagnose, and different app experiences could become less consistent. Optional detail must not obscure performance issues or recovery state. |

**Evaluation basis:** use LUCIA's requirements and representative scenarios as the primary basis: install/update an app, launch and inspect it, manage permissions, use GPU or inter-app workflows, and recover from a crash. Identify interaction patterns in existing systems only where they help address a scenario or requirement, and evaluate whether those patterns work for LUCIA. Compare task completion, user understanding, perceived simplicity and native integration, and measured performance as relevant; do not treat existing products or the analogies above as a benchmark or choose architecture by resemblance.

### Recommended order for resolving questions

**Recommendation only; these are not settled decisions.** Resolve the lifecycle questions in this order:

1. **Package and manager details:** define package format and installation details, and clarify manager boundaries. The responsibility for app/container versioning is already settled at the product level.
2. **Update constraints:** assess the app-author-selected update policies in context, including managed-restart constraints and compatibility when versions overlap. The app author owns the policy choice; no default or override is specified.
3. **Persistent data, migration, and recovery:** define migration behavior, failure recovery, and backup, export, reset, and deletion controls.
4. **Permission-mediated integration:** define the permission profile model and agreement experience, then address GPU access and inter-app workflows.
5. **Crash recovery and normal exit/removal:** define restart limits, backoff, user feedback, and data retention or deletion controls.

## Open questions

- Should LUCIA remain a working codename, or become the public product name? Is an acronym expansion desired?

## Codename

**LUCIA** is the current working codename. It is not yet the final public name, and no acronym expansion or name-availability check has been confirmed.

## What's next?

Keep using LUCIA as the working codename while refining the open package/manager details, update constraints, permission-profile model, and data-lifecycle questions. Decide separately whether to adopt it publicly and whether to define an acronym expansion.

## Resolved

- ✅ **RESOLVED — Session goal:** design and build a new Linux desktop environment.
> *Resume:* The session is a design exercise, not a comparison or customization of existing environments. User needs and technical architecture remain undecided.
- ✅ **RESOLVED — Primary audience and workflows:** general desktop users, with everyday browsing, communication, and personal computing as the initial workflow focus.
> *Resume:* Use mainstream personal computing as the baseline. This does not rule out accessibility needs or future specialized workflows; detailed personas and priorities remain open.
- ✅ **RESOLVED — Hardware baseline:** modern desktop and laptop systems, with GPU and graphics support important to the design.
> *Resume:* This establishes a general hardware class, not a minimum specification. GPU vendors, performance targets, memory floor, display requirements, and peripheral support remain unspecified.
- ✅ **RESOLVED — Provisional target distribution:** LUCIA's own integrated desktop distribution will provisionally be based on Ubuntu.
> *Resume:* Ubuntu is the provisional base, not a final baseline selection. The specific Ubuntu release, LTS versus non-LTS status, supported configurations, platform interfaces, kernel, and compatibility details remain open. The current host is only one environment and does not establish final compatibility.
- ✅ **RESOLVED — Leading product priorities:** simplicity, performance, and consistency.
> *Resume:* Use these three qualities to guide tradeoffs and evaluate the experience. Other qualities, including accessibility, privacy, stability, and configurability, remain relevant but were not selected as the top priorities.
- ✅ **RESOLVED — Product scope:** an integrated desktop environment covering shell/workspaces, compositor/window management, panel/launcher/notifications, settings, and core desktop apps; build most components in-house.
> *Resume:* The intended product is broad rather than a shell-only customization. "Most in-house" leaves room to reuse established Linux components when that is the better engineering choice; specific boundaries are not yet selected.
- ✅ **RESOLVED — GUI framework direction:** build a LUCIA GUI framework to replace GTK for LUCIA desktop components and apps; GTK is not the intended product foundation.
> *Resume:* This is a product direction, not a settled framework specification or display-stack choice. GTK compatibility, complete feature scope, implementation language, renderer, toolkit architecture, and display backend remain open. A GUI/client framework is distinct from a compositor/display server; this decision does not mean replacing Weston or reimplementing the Wayland protocol.
- ✅ **RESOLVED — Evaluation evidence:** compatibility testing, performance benchmarks, security review, and maintenance/ecosystem health.
> *Resume:* Use these evidence sources to evaluate design and implementation options for LUCIA's target distribution/platform. User research and accessibility testing were not selected as primary methods in this answer, but remain relevant considerations rather than exclusions.
- ✅ **RESOLVED — Application process model:** `containerd` manages application processes; launching an app creates a container instance.
> *Resume:* Treat app-per-container launch behavior as a product requirement. The shell/compositor placement, container image and packaging format, instance reuse and shutdown behavior, and desktop-service integration remain open.
- ✅ **RESOLVED — Full desktop runtime direction:** the entire LUCIA desktop user space is container-managed, including desktop/session components and apps.
> *Resume:* This is a product direction, not a topology decision. Linux containers share the host kernel; host boot/kernel and physical hardware are outside ordinary container contents. The exact host/user-space boundary, display/graphics/GPU, input and device integration, whether the desktop session is one container or multiple managed components, and whether apps use separate containers in the same or distinct lifecycle remain open. The expanded full-desktop assumption is a hard feasibility item to verify; prior Phase 0 evidence did not test a complete desktop session under `containerd`.
- ✅ **RESOLVED — App runtime ownership:** the app is managed as a container.
> *Resume:* Align lifecycle wording with the per-app container launch model. This does not select package format or implementation boundaries.
- ✅ **RESOLVED — App/container versioning responsibility:** the app manager/package manager is responsible for app and container versioning.
> *Resume:* Keep this as an abstract manager responsibility. Whether “app manager” and “package manager” refer to one component or separate components remains undecided.
- ✅ **RESOLVED — App update policy ownership:** the app author determines update behavior, choosing among apply on next launch, managed restart to update a running app, or allowing old and new versions to overlap.
> *Resume:* These are the valid policy choices; no default policy or policy override is established. Restart constraints and overlap compatibility remain open.
- ✅ **RESOLVED — Containerized app integration:** apps should feel native, with permission-mediated host access and strong isolation by default; GPU acceleration and inter-app workflows are important.
> *Resume:* The target balances desktop usability with least-privilege access. LUCIA must define permission profiles, require apps to declare the profiles they need, and request user agreement on first launch. Profile names, capabilities, granularity, agreement and denial behavior, persistence, revocation, access duration, later requests, prompt UI, host APIs, GPU device access, and security boundaries remain open.
- ✅ **RESOLVED — Crash behavior:** automatically restart app containers after crashes.
> *Resume:* Crash recovery is desired. Restart limits, backoff, user notification, and handling repeated crashes remain to be designed.
- ✅ **RESOLVED — Normal app exit:** stop the container and remove the stopped instance, while preserving app data in standard host locations for later launches.
> *Resume:* Runtime instances are disposable. Persistent app data should use standard host locations and migrate across app updates. Preserving the full writable container layer was not selected as a requirement; exact locations, migration behavior, and cleanup remain open.
- ✅ **RESOLVED — Working codename:** LUCIA.
> *Resume:* Use LUCIA to refer to the project during exploration. This does not decide the final public product name or an acronym expansion.