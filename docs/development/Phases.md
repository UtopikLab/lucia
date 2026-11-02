**Bounded input probe follow-up (2026-11-02; broader input integration remains unvalidated):** After the earlier Weston/seatd container showed a basic desktop, a follow-up attempted keyboard and pointer validation. The nonprivileged `containerd`/`runc` container had only host `/dev/input/event3` and `/dev/input/event4` mounted, with read-only `/run/udev` and `/sys`; it had no extra capabilities, host D-Bus, or direct DRM mounts. Through seatd, Weston opened `event3` and `event4` as well as `event0`, `event1`, `event2`, and `event5` through `event8`. The worker stopped the run immediately because access exceeded the intended exact-device allowlist; precise single-device isolation was not demonstrated. The user later confirmed seeing terminal rendering and a prompt, typing into it successfully, and moving the pointer during the short Weston session. This confirms only basic display and input for that session, not full desktop usability, app integration, graphical login, boot startup, a sustained session, or production policy. Weston produced an empty log. Cleanup verified removal of the task, image, namespace, and temporary data; the session returned to tty1; seatd is inactive and disabled. The `seatd` and `libseat1` packages remain installed. GRUB rotation remains intact and unmodified, and no system configuration was changed. Source evidence establishes that seatd brokers the entire active seat, not an arbitrary caller-selected subset of event nodes. A design choice remains whether the desktop compositor container may access all seat devices or LUCIA needs a narrower host input broker/device policy. No implementation or architecture is prescribed by this result.
**Provisional validation matrix (proposal; not a support commitment):**
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
- The full-desktop container direction is a new hard feasibility item, distinct from the per-app lifecycle premise tested here. Prior Phase 0 evidence covers per-app container lifecycle and narrow isolation/permission probes, not a complete desktop session running under `containerd`. The Phase 0 conditional GO remains valid only for that previously scoped premise; Phase 1 must verify the expanded premise before broad shell implementation, and may reopen or revisit the Phase 0 gate if evidence is insufficient or negative.
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

**Phase 0 checkpoint and human decision (2026-11-01): Conditional GO for the previously scoped premise only.** The user approved proceeding to Phase 1 after reviewing the Phase 0 evidence package. That evidence supports the narrow per-app lifecycle and probe scope described above; it did not test a complete desktop session running under `containerd`. It therefore does not pass or establish feasibility of the newly confirmed full-desktop container direction. The full-desktop assumption is a new hard feasibility item for Phase 1 and can reopen or revisit the Phase 0 gate. This checkpoint does not resolve open risks, finalize architecture, establish release readiness, or approve a final platform baseline.

## Phase 1: Target Distribution and Minimal Shell

**Status (updated 2026-11-02): Evaluation active; GUI framework implementation has not begun.** See the proposed [GUI framework contract](GUI-Framework.md). Ubuntu 26.04 LTS is a release candidate with standard maintenance through May 2031 and ESM through May 2036; Ubuntu 26.10 is a candidate supported through July 2027. These dates are from Ubuntu's [release cycle and support table](https://ubuntu.com/about/release-cycle), previously checked 2026-11-01. The longer maintenance horizon makes 26.04 LTS a candidate, not the selected baseline. Ubuntu remains provisional, and LTS status is not a hard requirement. A temporary Weston/seatd container probe on 2026-11-02 programmatically launched the desktop shell; user feedback confirms basic terminal display, keyboard input, and pointer movement during the short session. Weston and seatd are provisional feasibility tools, not selected product compositor or seat-broker choices. The seat/session phase gate remains open and no architecture has been selected. Supported hardware and use cases, graphics/input/display and sleep/resume, desktop API needs, integration boundaries, and the exact Ubuntu release remain to be evaluated.

**Read-only hardware inventory (2026-11-01):** Ubuntu 26.04, kernel `7.0.0-38-generic`, x86_64. The detected system is a desktop-class mini PC: DMI identifies Intel Client Systems / NUC8i5BEK, chassis type 35; `systemd-detect-virt` reports none and the cgroup path is `/init.scope`, evidence indicating a physical host rather than a guest, not absolute hardware proof. It has an Intel Core i5-8259U (4 cores / 8 threads), 14 GiB RAM, and Intel Iris Plus Graphics 655 (PCI `8086:3ea5`) using `i915`; no discrete GPU was detected. DP-1 is connected and enabled; DP-2 and HDMI-A-1 are disconnected. The user reports direct visual access to the monitor. This agent's session is TTY-only and did not visually inspect the screen; the active resolution and refresh rate remain unverified programmatically. No interactive graphics or suspend/resume tests have yet been reported. Input is through a Microsoft USB receiver; no trackpad or battery was found. The kernel exposes freeze and mem suspend states and `s2idle`/`deep` modes, but suspend/resume was not tested. This is one desktop-class inventory data point only: it is not a laptop, includes no interactive graphics or suspend validation, and is not compatibility proof.

**Full-desktop container feasibility attempt (2026-11-01; gate remains open):** A temporary Ubuntu 26.04 container image containing Weston `14.0.2-5` was built and imported into `containerd`. Weston was attempted via `containerd`/`runc` with a read-only root filesystem, all capabilities dropped, and only `/dev/dri/card1`, `/dev/dri/renderD128`, `/dev/tty0`, and `/dev/tty2` exposed. The attempt had no network, D-Bus, input devices, privileged mode, host package installs, or persistent configuration edits. Weston logged `Could not get primary session for user: No data available`; its embedded seat manager attempted VT1, could not open it, then failed opening DRM. It exited 1 before acquiring DRM or launching the shell. No active VT handoff occurred and tty1 remained active. No user-reported visual outcome is recorded. The user confirmed vertical display rotation is 90 degrees; live rotation remained `1`, and the existing `/etc/default/grub` setting `fbcon=rotate:1` was not changed. Temporary task, container, image, namespace, and build context were removed; the build cache was retained and `containerd` remains active. This ad-hoc remote test harness required an active local logind session and did not provide host seat/session integration; that was a limitation of the test setup, not a product requirement. The attempt did not test successful DRM/KMS operation or prove the full-desktop container direction infeasible. Do not propose broad capability grants or mounting the system D-Bus without design review. The Phase 1 full-desktop container gate remains explicitly open.

**Read-only seatd package investigation (2026-11-02; feasibility remains open):**

- **Package and service facts:** Ubuntu 26.04 `resolute/universe` amd64 offers `seatd` and `libseat1` version `0.9.2-2`; an apt install would add two packages. Inspection of package maintainer scripts shows a normal install requests that the seatd service be enabled and started. The service runs as root with `seatd -g video`, is wanted by `multi-user.target`, and restarts on failure. No explicit capability or device sandbox restrictions are declared. The default `/run/seatd.sock` has mode `0770` and group `video`; seatd brokers DRM, input, HID, and VT access.
- **Risk assessment and gate:** This is a substantial host-trust boundary. Package metadata and documentation do not establish safe coexistence with an active seat/logind session or validate the full-desktop design. No package installation or service action was performed. Keep Phase 1 seat/session feasibility open pending a deliberate broker/topology choice and explicit approval before any install or service start.

**Temporary Weston/seatd container probe (2026-11-02; seat/session gate remains open):** The user authorized a temporary image build/download and installation/start of seatd for this test. Ubuntu 26.04 `seatd` and `libseat1` `0.9.2-2` were installed; seatd was enabled during package installation, then stopped and disabled after the test. Both packages remain installed. A temporary Ubuntu 26.04 image with Weston `14.0.2-5` ran under `containerd`/`runc` on tty2 with a 180-second watchdog. The worker reports Weston connected through seatd, acquired `/dev/dri/card1`, detected connected DP-1, and launched the desktop shell. The container used no privileged mode or host device mounts; seatd's socket brokered device access. No broad host configuration changes were made. The workload auto-exited at the watchdog, and its namespace, image, and temporary build context were cleaned up. The session returned to tty1; `containerd` remains active and seatd is inactive/disabled. `/etc/default/grub` remains `GRUB_CMDLINE_LINUX_DEFAULT="fbcon=rotate:1"`, and live rotation remains `1`. This demonstrates that a containerized compositor can acquire DRM and start the shell through seatd on this hardware/runtime. The user later confirmed seeing terminal rendering and a prompt, typing into it successfully, and moving the pointer during the short session. The probe stopped when seatd opened the full seat input device set, `event0` through `event8`, beyond the original `event3`/`event4` allowlist; precise single-device isolation was not demonstrated. This confirms only basic display and input for this short Weston session, not full desktop usability, app integration, graphical login, boot startup, a sustained session, or production policy. Keep the seat/session phase gate open; no architecture has been selected.

**User visual and input confirmation (2026-11-02):** The user reported seeing terminal rendering and a prompt, typing into it successfully, and moving the pointer during the short Weston session. This supersedes the earlier report that no reliable pointer/keyboard result was collected and the note that visual feedback was pending. It confirms only basic display and input in that session, not full desktop usability, app integration, graphical login, boot startup, a sustained session, or production policy. The probe stopped after seatd opened `event0` through `event8`, beyond the intended `event3`/`event4` allowlist, so precise single-device isolation was not demonstrated. The seat/session gate remains open.

**Provisional validation matrix (proposal; not a support commitment):** The following are test categories for gathering initial evidence, not compatibility guarantees. Scope remains a proposal pending hardware availability, test evidence, and human approval.

| Test category | Proposed coverage |
| --- | --- |
| Reference systems | One reference laptop and one desktop, both with integrated graphics. |
| Displays | The laptop's built-in display and one ordinary external display. |
| Input | Keyboard, mouse, and trackpad. |
| Network | Ordinary wired and wireless network connections. |
| Optional or extended coverage | Docking, multiple or high-DPI displays, dedicated GPU, touch, and less common input devices. |

| Representative scenario | Observations |
| --- | --- |
| First use, setup, sign-in, app launch, and close | Clarity, consistency, and responsiveness. |
| Laptop suspend and resume | Recovery of session, display, network, and input. |
| Display hotplug and scaling | Detection, layout, readability, and recovery. |
| Desktop graphics and local video | Rendering, smoothness, and resource use; dedicated GPU is optional. |
| Input | Focus, accuracy, and consistency. |
| Notification and permission allow/deny | Delivery, understandable outcome, and enforcement. |
| File open/create/save/reopen and app access | Correct path and access behavior. |

**Draft Linux host requirements (validation proposal; not a support commitment):** Use the following as a representative validation set, not a popularity ranking or approved support list: Ubuntu/Debian (optionally Linux Mint as an Ubuntu-family candidate), Fedora Workstation, Arch Linux, and openSUSE Leap/Tumbleweed. The exact support list, kernel and architecture floors, and compatible runtime versions require current-source research and actual tests; no distribution is currently marked supported.

| Requirement area | Draft requirement and validation notes |
| --- | --- |
| Host kernel and architecture | Define the required host kernel interfaces and explicitly test each claimed CPU architecture, container runtime, and snapshotter combination. Minimum kernel versions and supported combinations are TBD; Linux containers use the host kernel. |
| Container runtime | Validate `containerd` with an OCI-compatible runtime compatible with each host. Define a per-distribution host adapter for system startup and updates; exact versions and adapter behavior are TBD. |
| Boot and session startup | Start the desktop through the system's managed boot/service mechanism without requiring a shell login or prior user TTY session. |
| Seat and device access | Integrate through an existing host seat/device broker such as logind or seatd, with the exact broker and adapter TBD. Keep access narrowly scoped; do not introduce a custom security framework or assume unrestricted device access. |
| Graphics and display | Validate DRM/KMS access and compatibility across host GPU drivers and userspace, including display discovery and recovery. Hardware and driver coverage remain TBD. |
| Input and system integration | Validate input devices, audio, networking, D-Bus, and runtime/portal integration, granting only the access each integration needs. Exact interfaces and policies remain TBD. |
| Host security controls | Respect host security controls, including SELinux and AppArmor where present. Do not disable or broadly weaken them to make a candidate work. |
| Distribution-specific operations | Isolate package and service differences in a thin host adapter and test each candidate: APT/Deb/dpkg, DNF/RPM, pacman, and Zypper. Do not let distro-specific changes require edits to the desktop OCI image. |

**Draft acceptance criteria:** The system boots to a visible desktop without shell login; keyboard and pointer, GPU/display, suspend/resume, audio/network, and start/stop with seat restoration are validated. The same desktop image is tested across the selected candidate distributions without image-specific edits, with distribution differences handled only by the host adapter. Record failures and untested combinations rather than implying compatibility.

The desktop shell/session and apps remain in OCI containers; the host retains the Linux kernel, device discovery, and a narrowly scoped hardware/seat broker. This boundary does not select the container topology or broker implementation. Confirm the exact supported distributions, kernels, architectures, and version floors through current-source research and actual tests before making any support claim.

**Purpose:** Establish whether LUCIA's full container-managed desktop/session direction is feasible on its provisionally Ubuntu-based distribution before broad shell implementation. Evaluate a system/boot-started desktop-session container, including whatever initial greeter or session entry point is later approved, across a host seat/session broker boundary. This must not assume a prior user-shell or TTY login. Exercise display, input, graphics/GPU, system integration, and app lifecycle across the container boundary. Use evidence to inform platform choices; keep the broker mechanism and container topology open, and do not select an Ubuntu release, architecture, or enforcement mechanism by default.

**Key work:**

- First verify whether a system/boot-managed `containerd` desktop-session container, including the initially approved greeter/session entry point, can reach a usable desktop without requiring a user-shell or TTY login. Investigate the host seat/session broker boundary and validate whether the container can acquire and release the display/input seat safely without broad privilege. Exercise display, input, graphics/GPU, system integration, and app lifecycle across the container boundary; record blockers and host-side integration requirements before committing to broad shell implementation. Prior Phase 0 per-app lifecycle and narrow isolation/permission probes are not evidence that this full-session premise works. Keep broker mechanism and container topology undecided.
- Determine the host/user-space boundary and investigate graphics, input, GPU, and device integration as feasibility questions. Linux containers share the host kernel; host boot/kernel and physical hardware are outside ordinary container contents. Keep topology open, including one desktop-session container versus multiple managed components, and whether apps are separate containers in the same or distinct lifecycle.
- Evaluate baseline/platform compatibility and integration boundaries for the shell, compositor/window management, system services, and in-house components before selecting a specific Ubuntu release or settling architecture. Keep the release and LTS status, supported hardware scope, display protocol, compositor, package format, and manager/reuse boundaries open pending evidence and explicit decisions; do not treat the current Ubuntu 26.04 host as the final baseline by default.
- Define and vertically prototype LUCIA GUI framework primitives and the host/client boundary; use the resulting contract to replace GTK in the temporary launcher fixture. This is GUI/client framework work, not a decision to replace the compositor or display protocol. Keep framework scope, language, renderer, toolkit architecture, GTK compatibility, and display backend open until separately evaluated and decided.
- Continue shell foundation work with panel, launcher, notifications, and a settings entry point against the evolving framework contracts.
- Verify basic app presentation and system integration paths against the Phase 0 feasibility findings.

**Exit criteria:**

- Before broad shell implementation, evidence demonstrates whether a `containerd`-managed full desktop/session vertical slice can support the required display, input, graphics/GPU, system integration, and app lifecycle across the container boundary. A failed or materially incomplete result reopens the Phase 0 feasibility gate and blocks broad shell implementation pending a reviewed decision.
- LUCIA boots and provides a coherent minimal desktop session on the specifically selected and validated Ubuntu release and target configuration.
- The initial shell components work together for basic navigation and app entry points, with known platform constraints recorded.
- Platform and shell boundaries needed by runtime integration are documented and stable enough for Phase 2 work.
- Compatibility evidence identifies supported target-platform configurations and outstanding blockers; no unverified broad compatibility claim is made.

**Dependencies and open gates:**

- Carries forward the Phase 0 conditional GO only for the previously scoped per-app lifecycle and narrow probes. The full-desktop container assumption is a new hard feasibility item; validate it first in this phase and revisit the Phase 0 gate if it is not supported.
- Ubuntu is the provisional distribution base. The exact Ubuntu release, CPU architecture, support scope, display protocol/compositor, package format, and reuse boundaries remain open until supported by evidence and explicitly decided. LTS versus non-LTS status, platform interfaces, kernel, and compatibility details also remain open.
- Container topology remains open: one desktop-session container versus multiple managed components, and whether app containers share or have a distinct lifecycle. The host/user-space boundary and graphics, input, GPU, and device integration must be investigated without presuming an enforcement mechanism.
- Changes to shared platform boundaries after Phase 2 integration begins may require revisiting its contracts.

**Temporary GTK4 launcher client integration fixture (2026-11-02; not framework implementation or physical display proof):** The experimental GTK4/PyGObject launcher in [desktop-container/launcher](../../desktop-container/launcher/README.md) is implemented and retained as a temporary Wayland client/container integration test fixture. It is not the LUCIA GUI framework implementation or an architecture choice; the confirmed direction is to build a LUCIA GUI framework to replace GTK for LUCIA desktop components and apps. A smoke run used `containerd` 2.3.3, a unique namespace, Ubuntu 26.04, and separate Weston headless compositor and launcher client tasks. Ubuntu archive packages were fetched by temporary package-fetch tasks and installed offline in the final tasks. The final compositor and client shared only a private runtime directory, verified as UID:GID `1000:1000`, mode `0700`; role-specific package, source, and log mounts were not shared. The package-fetch tasks briefly used host networking; the final tasks did not. No host D-Bus or host devices were mounted, no privileged mode or capabilities were added, and both compositor/client processes were launched non-root with capabilities dropped. The client asserted effective capabilities were zero, connected to Wayland, presented its window, and invoked the Files button signal through GTK, updating the status text. Tested package versions: Weston `14.0.2-5`, Python `3.14.3-0ubuntu2`, PyGObject `3.56.2-1`, and GTK4 `gir1.2-gtk-4.0`/`libgtk-4-1` `4.22.4+ds-0ubuntu0.1`. The temporary namespace, image reference, tasks, and build/runtime files were cleaned up. This validates only a headless client integration and toolkit callback; it does not prove physical display output/input, visual rendering on hardware, broader desktop usability, or the full desktop/session container gate. Weston here is a temporary test compositor and does not imply a LUCIA compositor choice or Wayland protocol reimplementation.

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