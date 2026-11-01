# LUCIA Development Phases

This document proposes a risk-first sequence for developing LUCIA. It translates the confirmed product direction in [the thinktank summary](../thinktank/Summary.md) into gated phases; it does not approve an architecture or resolve that document's open questions.

## Confirmed Direction and Proposed Sequence

The following are existing user decisions, not new roadmap proposals:

- LUCIA targets its own integrated desktop distribution, provisionally based on Ubuntu, and general desktop users, prioritizing simplicity, performance, and consistency. The specific Ubuntu release and LTS versus non-LTS status remain undecided; supported configurations, platform interfaces, kernel, and compatibility details also remain open.
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
- Gather initial Ubuntu-target feasibility evidence for the desired isolation and permission-mediated host access. Evaluate feasibility without selecting an enforcement mechanism or final profile catalog; the specific Ubuntu release remains for Phase 1 to choose.
- Validate at the contract level that an app author can express the three decided update-policy choices and that the runtime/manager model can represent them. Do not design detailed update UX or implement all three policies in this phase.
- Assess Ubuntu-target feasibility for the integrated desktop scope, including constraints that could block a native-feeling app experience. Treat findings from the current host as evidence from one environment only; identify uncertainties rather than assuming compatibility.
- Record risks, evidence, and decisions that need human approval before proceeding.

**Exit criteria:**

- A working slice or equivalent technical evidence demonstrates that the per-launch container lifecycle is feasible, including stopping/removing an instance on normal exit while retaining app data outside that disposable instance.
- Evidence supports a viable path to Linux isolation and permission enforcement for the product premise, with material limitations and unresolved risks documented. No particular enforcement mechanism is presumed.
- The app-author update-policy contract is shown to be representable at an appropriate level, without resolving detailed UX or implementing each policy.
- Platform feasibility findings identify whether the integrated desktop direction can proceed on the provisional Ubuntu target, what assumptions remain unverified, and which gates require explicit decisions. They do not establish final compatibility or select the specific Ubuntu release.
- The evidence and remaining risks are reviewed at a go/no-go checkpoint before broad investment in the shell and app ecosystem.

**Dependencies and open gates:**

- This is the first phase and has no implementation-phase dependency. Its evidence informs every later phase.
- Linux enforcement feasibility is a hard gate. A weak or unsupported enforcement premise requires revisiting the product/security approach before proceeding.
- Ubuntu is the provisional distribution base. The specific Ubuntu release, LTS versus non-LTS status, supported configurations, platform interfaces, kernel, compatibility details, display protocol, package format, manager boundaries, profile catalog, enforcement mechanism, performance targets, and detailed update UX remain undecided; this phase must not silently settle them.

**Progress evidence (2026-11-01; partial, gate remains open):**

- A disposable `containerd` lifecycle probe used the cached `docker.io/library/alpine:3.16.3` image (`sha256:b95359c2505145f16c6aa384f9cc74eeff78eb36d308ca4fd902eeeb0a0b161b`). The root `ctr` client was required because unprivileged `ctr` was blocked at the root-owned overlay snapshot path. The workload ran as UID:GID `1000:1000`, without a privileged flag, CNI/host-network flag, or mounts other than a temporary data-directory bind mount. The first launch wrote a marker and exited; a fresh second launch read it and exited. Tasks, containers, temporary data, probe namespace, and imported image were removed; the original cached image remained untouched, and the probe made no repository or host configuration changes. This supports lifecycle cleanup and persistence only for this narrow setup; it does not demonstrate isolation, permission enforcement, or network behavior.
- A read-only snapshot of this Ubuntu 26.04 LTS host found kernel `7.0.0-38-generic` on x86_64, and user-namespace settings `unprivileged_userns_clone=1` and `max_user_namespaces=61178`; namespace creation was not tested. Cgroup v2 is mounted, with `cpuset`, `cpu`, `io`, `memory`, `hugetlb`, `pids`, `rdma`, `misc`, and `dmem` listed at its root; the inspected shell is in `/user.slice/user-1000.slice/session-6.scope`, which does not establish app-level delegation. The inspected shell reports `Seccomp=0`, `Seccomp_filters=0`, `NoNewPrivs=0`, and `CapEff=0`; these are process-specific observations, not a system-wide policy assessment. This is relevant initial evidence for the provisional Ubuntu target, but it represents only one host/environment and does not select Ubuntu 26.04 as LUCIA's baseline or prove final compatibility.
- The kernel LSM list includes AppArmor and its module reports enabled. Unprivileged `aa-status` confirms the module is loaded but cannot read the profile set, so applied profiles could not be assessed. `unshare`, `aa-status`, `apparmor_parser`, `setpriv`, `nsenter`, `lsns`, `ctr`, and `containerd` are installed; `bwrap` and `getenforce` are absent. These observations are relevant initial evidence for Ubuntu, but are specific to this one host/environment; they do not establish a final target enforcement path or pass this phase's security gate.
- A follow-up read-only check on this host (2026-11-01) found Ubuntu 26.04 with kernel `7.0.0-38-generic`; AppArmor is enabled and loaded, and `aa-status` reports 166 profiles, 90 enforcing. The `unprivileged_userns` profile is enforcing; kernel journal records for `unshare -Ur true` show denial of `CAP_SYS_ADMIN` under its `audit deny capability` rule. Kernel user namespaces are compiled in and enabled, so this is evidence of a policy boundary, not proof that namespaces are unavailable. `containerd` 2.3.3 and `runc` report AppArmor support, and CRI does not disable it; no application workload was tested, so this establishes runtime capability, not permission-enforcement behavior. This is host-specific evidence only, not a final release baseline or permission-design approval; no AppArmor policy change is proposed, and no host configuration was changed.
- A controlled workload probe on a disposable Ubuntu 26.04 host loaded a temporary AppArmor profile in enforce mode with `audit deny /probe/denied r,`. A container using the cached Alpine 3.16.3 image (`sha256:b95359c2505145f16c6aa384f9cc74eeff78eb36d308ca4fd902eeeb0a0b161b`; no network pull) ran as UID:GID `65534:65534`, with a read-only bind mount, ordinary nonprivileged defaults, and no added capabilities or network. An allowed file read succeeded; the targeted read failed with `Permission denied`. A matching kernel AppArmor `DENIED` audit record confirmed profile `aa-enforce-probe-20261101b`, operation `open`, path `/probe/denied`, and process `cat`. The temporary profile, import, namespace, and temporary files were cleaned up; the original moby image was preserved, `containerd` remained active, and the test made no persistent host-configuration or repository changes. This confirms a narrowly scoped AppArmor denial in this Ubuntu/containerd/runc/image stack only; it does not validate production policy or select the final enforcement mechanism, and the target Ubuntu release remains undecided.
- A host-resource boundary probe (2026-11-01) on disposable Ubuntu 26.04 with `containerd` 2.3.3 and `runc` 1.4.3 used the cached Alpine 3.16.3 image (`sha256:b95359c2505145f16c6aa384f9cc74eeff78eb36d308ca4fd902eeeb0a0b161b`). The workload ran as UID:GID `65534:65534`, with AppArmor explicitly unconfined, zero effective capabilities, a read-only root filesystem, no host mounts, and CNI networking rather than host networking. A host-only sentinel and a live host PID were present on the host but absent inside the container. Test resources and processes were cleaned up; the original image remains and the service is active. The test made no repository or host-configuration changes. This is evidence only for direct filesystem-path visibility and the PID namespace boundary; it is not a general isolation proof or a container-escape test, and it selects neither architecture nor an enforcement mechanism.
- A separate disposable-container boundary probe (2026-11-01) on Ubuntu 26.04 used `containerd` 2.3.3, `runc` 1.4.3, and the cached Alpine 3.16.3 image (`sha256:b95359c2505145f16c6aa384f9cc74eeff78eb36d308ca4fd902eeeb0a0b161b`). The workload ran as UID:GID `65534:65534`, with a read-only root filesystem, all capabilities dropped, and AppArmor unconfined. Observed mount, network, and IPC namespace handles were distinct from the host. Among observed `/dev` entries, only `full`, `null`, `pts`, `ptmx`, `random`, `tty`, `urandom`, and `zero` were exposed; `/dev/dri`, `/dev/kvm`, `/dev/nvme0n1`, `/dev/input`, and `/dev/snd` were absent. No image pulls or configuration changes occurred, and cleanup was verified. Scope is limited to namespace separation and default device-node visibility; this does not establish network-egress behavior, access denial, escape resistance, or a production profile.
- The already-decided author update choices have an abstract contract-level representation as one required enum field: `update_policy: next_launch | managed_restart | allow_overlap`. These values map respectively to applying an update on the next launch, a managed restart to update a running app, or allowing old and new versions to overlap. This demonstrates representation only; it does not prescribe a package format, metadata schema, manager boundary, runtime behavior, implementation scope, or UX.

**Live-source Ubuntu platform research (2026-11-01):**

- **Source facts:** Ubuntu's [release cycle and support table](https://ubuntu.com/about/release-cycle) (live read 2026-11-01) lists Ubuntu 26.10 with support through July 2027 and Ubuntu 26.04 LTS with standard support through May 2031 and ESM through May 2036. Ubuntu documents [image build processes](https://ubuntu.com/project/docs/how-ubuntu-is-made/processes/image-builds/) and its [package archive](https://ubuntu.com/project/docs/how-ubuntu-is-made/concepts/package-archive/) (both live read 2026-11-01). The [Wayland introduction](https://wayland.freedesktop.org/docs/html/ch01.html) and [compositor documentation](https://wayland.freedesktop.org/docs/book/Compositors.html) describe Wayland client/compositor interfaces (live read 2026-11-01). The systemd [logind service](https://www.freedesktop.org/software/systemd/man/latest/systemd-logind.service.html) and [login1 D-Bus API](https://www.freedesktop.org/software/systemd/man/latest/org.freedesktop.login1.html) documentation identify systemd version 262 (live read 2026-11-01). XDG portal documentation describes [FileChooser API version 4](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.FileChooser.html) and [Notification API version 2](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Notification.html) (live read 2026-11-01). Ubuntu also documents [Intel graphics update maintenance guidance](https://ubuntu.com/project/docs/SRU/reference/exception-Intel-Graphics-Updates/) (live read 2026-11-01).
- **Implications and gates, not source guarantees:** These sources indicate available Ubuntu image customization/build pipelines, an apt archive, systemd/logind integration, Wayland compositor interfaces, XDG portal APIs, and Ubuntu graphics-stack maintenance processes. They support conditional technical feasibility for an integrated LUCIA desktop distribution provisionally based on Ubuntu; they do not establish downstream compatibility or support. LUCIA would own downstream image, package, update, and maintenance work. The exact Ubuntu release/LTS and support horizon, supported hardware, and any platform design remain open. Validate graphics/GPU, input/display, sleep/resume, hardware, and apps' portal and notification needs; assess redistribution, licensing, branding, and update delivery. Upstream documentation does not guarantee downstream support.

**Phase 0 checkpoint and human decision (2026-11-01): Passed for proceeding to Phase 1 only.** The user approved a conditional GO to proceed into Phase 1 after reviewing the Phase 0 evidence package. The evidence supports technical feasibility, but does not resolve open risks, finalize architecture, establish release readiness, or approve a final platform baseline. Remaining open decisions and risks must carry forward; this checkpoint is not evidence that all platform or security requirements are release-ready.

## Phase 1: Target Distribution and Minimal Shell

**Status (2026-11-01): Authorized/started for evaluation.** Phase 1 is authorized to begin, but implementation is not claimed to have begun. First evaluate baseline/platform compatibility and integration boundaries. The exact Ubuntu release and LTS status, supported hardware scope, display protocol, compositor, package format, and app-manager/package-manager boundaries and reuse choices remain undecided; Ubuntu 26.04 is host evidence only, not a selected baseline.

**Purpose:** Establish the smallest integrated desktop foundation for LUCIA's own Ubuntu-based distribution, using Phase 0 evidence to choose a specific Ubuntu release and constrain platform choices.

**Key work:**

- Evaluate baseline/platform compatibility and integration boundaries for the shell, compositor/window management, system services, and in-house components before selecting a specific Ubuntu release or settling architecture. Keep the release and LTS status, supported hardware scope, display protocol, compositor, package format, and manager/reuse boundaries open pending evidence and explicit decisions; do not treat the current Ubuntu 26.04 host as the final baseline by default.
- Implement a minimal usable shell foundation with panel, launcher, notifications, and a settings entry point.
- Verify basic app presentation and system integration paths against the Phase 0 feasibility findings.

**Exit criteria:**

- LUCIA boots and provides a coherent minimal desktop session on the specifically selected and validated Ubuntu release and target configuration.
- The initial shell components work together for basic navigation and app entry points, with known platform constraints recorded.
- Platform and shell boundaries needed by runtime integration are documented and stable enough for Phase 2 work.
- Compatibility evidence identifies supported target-platform configurations and outstanding blockers; no unverified broad compatibility claim is made.

**Dependencies and open gates:**

- Requires a Phase 0 go decision and its documented risk findings.
- Ubuntu is the provisional distribution base. Phase 1 must choose the specific release; LTS versus non-LTS status, supported configurations, platform interfaces, kernel, compatibility details, and display protocol remain open until explicitly decided.
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
- Run compatibility testing across LUCIA's declared target-distribution/platform support scope and verify failure and recovery behavior.
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