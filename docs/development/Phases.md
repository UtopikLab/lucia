# LUCIA Development Phases

This document proposes a risk-first sequence for developing LUCIA. It translates the confirmed product direction in [the thinktank summary](../thinktank/Summary.md) into gated phases; it does not approve an architecture or resolve that document's open questions.

## Confirmed Direction and Proposed Sequence

The following are existing user decisions, not new roadmap proposals:

- LUCIA initially targets Ubuntu LTS and general desktop users, prioritizing simplicity, performance, and consistency.
- The product is an integrated desktop environment: shell, compositor and window management, panel, launcher, notifications, settings, and core apps. Most components are intended to be built in-house.
- `containerd` manages an app container instance for each launch. The app/package manager is responsible for app and container versioning, but its boundaries and the package format are undecided.
- App authors choose among updating on the next launch, a managed restart, or allowing old and new versions to overlap. Normal exit stops and removes the instance; app data persists and migrates. Restarting after crashes is desired.
- Apps declare LUCIA permission profiles, and first launch asks the user to agree. Install and launch should feel app-centric; Android is an interaction analogy, not an implementation choice.
- Detailed update UX is explicitly too early to design. Permission-profile contents and enforcement are not finalized.

The phase order below is proposed sequencing. Each phase has an evidence-based exit gate; a failed gate can lead to revisiting assumptions, narrowing scope, or stopping before broader investment. No dates or staffing estimates are implied.

## Phase 0: Feasibility and Vertical Slice

**Purpose:** Test the highest-risk runtime, security, and platform assumptions before committing to broad desktop implementation.

**Key work:**

- Build or evaluate a narrow technical vertical slice around a `containerd`-managed per-launch app instance, including normal exit and persistent app data across launches.
- Gather evidence that the desired isolation and permission-mediated host access can be enforced on the Linux/Ubuntu LTS target. Evaluate feasibility without selecting an enforcement mechanism or final profile catalog.
- Validate at the contract level that an app author can express the three decided update-policy choices and that the runtime/manager model can represent them. Do not design detailed update UX or implement all three policies in this phase.
- Assess platform feasibility for the integrated desktop scope, including the constraints that could block a native-feeling app experience. Identify uncertainties rather than assuming compatibility.
- Record risks, evidence, and decisions that need human approval before proceeding.

**Exit criteria:**

- A working slice or equivalent technical evidence demonstrates that the per-launch container lifecycle is feasible, including stopping/removing an instance on normal exit while retaining app data outside that disposable instance.
- Evidence supports a viable path to Linux isolation and permission enforcement for the product premise, with material limitations and unresolved risks documented. No particular enforcement mechanism is presumed.
- The app-author update-policy contract is shown to be representable at an appropriate level, without resolving detailed UX or implementing each policy.
- Platform feasibility findings identify whether the integrated Ubuntu LTS direction can proceed, what assumptions remain unverified, and which gates require explicit decisions.
- The evidence and remaining risks are reviewed at a go/no-go checkpoint before broad investment in the shell and app ecosystem.

**Dependencies and open gates:**

- This is the first phase and has no implementation-phase dependency. Its evidence informs every later phase.
- Linux enforcement feasibility is a hard gate. A weak or unsupported enforcement premise requires revisiting the product/security approach before proceeding.
- The Ubuntu LTS release, display protocol, package format, manager boundaries, profile catalog, enforcement mechanism, performance targets, and detailed update UX remain undecided; this phase must not silently settle them.

## Phase 1: Ubuntu LTS Platform and Minimal Shell

**Purpose:** Establish the smallest integrated desktop foundation on the initial distribution target, using Phase 0 evidence to constrain platform choices.

**Key work:**

- Select and validate the specific Ubuntu LTS baseline when implementation timing and compatibility needs are known.
- Establish the platform integration boundaries for the shell, compositor/window management, system services, and in-house components.
- Implement a minimal usable shell foundation with panel, launcher, notifications, and a settings entry point.
- Verify basic app presentation and system integration paths against the Phase 0 feasibility findings.

**Exit criteria:**

- LUCIA boots and provides a coherent minimal desktop session on the selected Ubuntu LTS baseline.
- The initial shell components work together for basic navigation and app entry points, with known platform constraints recorded.
- Platform and shell boundaries needed by runtime integration are documented and stable enough for Phase 2 work.
- Compatibility evidence identifies supported baseline configurations and outstanding blockers; no unverified broad compatibility claim is made.

**Dependencies and open gates:**

- Requires a Phase 0 go decision and its documented risk findings.
- The specific Ubuntu LTS release and display protocol must be decided through platform evaluation; neither is selected by this roadmap.
- Changes to shared platform boundaries after Phase 2 integration begins may require revisiting its contracts.

## Phase 2: App Manager, Runtime Lifecycle, and Permission Profiles

**Purpose:** Make app installation and launch reliable while preserving the agreed lifecycle and permission requirements.

**Key work:**

- Define and implement app management around `containerd`-managed per-launch instances, including the app/container versioning responsibility already assigned to app/package management.
- Support normal-exit cleanup with persistent app data and data migration across app updates; design recovery for interrupted or unsuccessful migrations.
- Implement the desired restart-after-crash behavior and develop policy for limits, backoff, and repeated failures.
- Define the permission-profile model: app declarations, first-launch agreement, grant and denial behavior, and relevant later changes. Select profiles and enforcement only through explicit design decisions supported by security evidence.
- Specify the app-author update-policy contract sufficiently for manager/runtime integration. Keep detailed end-user update UX deferred; do not assume all three policies must be implemented in this phase.

**Exit criteria:**

- Representative apps can be installed and launched through an app-centric flow, creating the required per-launch managed instance.
- Normal exit removes the stopped instance without losing persistent app data; update migration behavior has tested success and failure handling.
- Crash recovery behavior is tested, and the remaining restart-limit and user-feedback policy is decided or explicitly gated before release.
- Apps can declare the approved permission profiles, and first launch obtains user agreement under an explicitly approved permission design. Enforcement and denial behavior are tested against the approved design.
- App/container versioning works across the approved management boundary, and any update-policy behavior implemented is consistent with the app author's chosen policy.

**Dependencies and open gates:**

- Requires stable Phase 1 platform integration boundaries and Phase 0 evidence supporting the runtime and security premise.
- Decide package format and whether app management and package management are one component or separate components; neither is prescribed here.
- Approve profile contents, consent and revocation semantics, and enforcement design before treating permission handling as release-ready.
- The three author-selected update policies are established product direction, but implementation scope, restart constraints, overlap compatibility, and detailed UX remain open. Detailed UX is not a Phase 0 gate and is not specified here.

## Phase 3: Core Apps and Everyday Workflows

**Purpose:** Deliver a useful desktop for general users and validate that the integrated shell and app model work together in everyday use.

**Key work:**

- Build or integrate the initial core apps, with most components developed in-house as intended.
- Complete the everyday workflows for browsing, communication, and personal computing across app launch, notifications, settings, and common file or system interactions.
- Validate native-feeling integration, permission prompts and outcomes, GPU needs, and inter-app workflows against approved platform and security contracts.
- Use representative user scenarios to find friction in install, launch, recovery, and common app interactions. Do not infer detailed update UX choices from the app-author policy requirement.

**Exit criteria:**

- A representative set of everyday tasks can be completed using the integrated shell and core apps, without relying on placeholder integrations for essential flows.
- Apps follow the approved lifecycle and permission contracts, and users can understand and complete the supported install, launch, and access-agreement flows.
- Compatibility and workflow findings are documented, with critical blockers assigned to a later gate or resolved before hardening.
- Performance is measured on representative workflows; acceptance thresholds remain an explicit decision rather than an assumed target in this roadmap.

**Dependencies and open gates:**

- Requires stable shell, app lifecycle, permission, and data contracts from Phases 1 and 2.
- Final core-app scope and reuse boundaries remain implementation decisions consistent with the in-house direction.
- Detailed update UX remains deferred until the manager/runtime behavior and user scenarios provide enough evidence to design it.

## Phase 4: Hardening and Initial Release Readiness

**Purpose:** Reduce security, reliability, performance, compatibility, and maintenance risks to an explicitly reviewed initial-release level.

**Key work:**

- Conduct security review of isolation, permission enforcement, app/container lifecycle, and update/data-migration paths.
- Benchmark representative system and app workflows; agree on performance targets before using them as release gates.
- Run compatibility testing across the declared initial support scope and verify failure and recovery behavior.
- Assess maintenance burden, dependency health, and operational needs for the components LUCIA builds and reuses.
- Close or formally accept release-blocking risks and document limitations of the initial release.

**Exit criteria:**

- Security review findings are resolved or explicitly accepted by the appropriate decision authority; no unreviewed critical risk is carried into release.
- Benchmarks are evaluated against approved performance targets, with regressions and limitations documented.
- Compatibility tests pass for the declared initial support scope, and unsupported configurations are clearly identified.
- Crash recovery, normal-exit cleanup, persistent-data migration, and permission behavior have release-level validation.
- Maintenance and ecosystem-health evidence supports an initial release decision, and remaining risks have named owners or an explicit acceptance decision outside this roadmap.

**Dependencies and open gates:**

- Requires the integrated workflows and contracts from Phases 1–3 to be sufficiently stable for meaningful hardening.
- Performance targets, initial compatibility scope, release risk tolerance, and final update UX are not decided by this document and require explicit decisions where they affect release acceptance.
- Initial release readiness is a gate, not a promise of release or publication.

## Phase Overlap

Phases are gates, not a requirement that all work happen serially. Work may overlap only after the shared contracts it depends on are stable and the owning phase's relevant gate has passed. In particular, app UX and shell polish must not force premature choices about package format, display protocol, permission profiles or enforcement, performance targets, or detailed update UX.