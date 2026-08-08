# traur

Trust scoring for AUR packages, written in Rust. Analyzes PKGBUILDs, install scripts, source URLs, metadata, and git history to score how much you should trust a package before installing it. Includes an ALPM hook that automatically scans packages before any install or upgrade transaction, plus a makepkg wrapper for pre-build scanning.

<img width="859" height="640" alt="image" src="https://github.com/user-attachments/assets/768915bd-4aa2-4450-96c7-408e73e0d103" />

## Installation

```bash
paru -S traur
```

## Usage

```bash
traur scan                    # scan all installed AUR packages
traur scan <pkg> [<pkg>...]   # scan one or more packages
traur scan --pkgbuild PKGBUILD # scan a local PKGBUILD
traur allow <package>         # whitelist a package
```

## Setup: automatic pre-build and pre-install scanning

traur uses two security gates:

1. **Pre-build:** `traur-makepkg` scans PKGBUILD before `makepkg` can source or execute it.
2. **Pre-install:** ALPM `PreTransaction` hook scans packages before pacman installs or upgrades them.

The normal flow is:

```text
yay / paru / manual build
  -> traur-makepkg
  -> PKGBUILD scan
  -> /usr/bin/makepkg
  -> pacman PreTransaction scan
  -> package installation
```

### 1. Install traur

Install from AUR with either helper:

```bash
paru -S traur
# or
yay -S traur
```

Package installs:

- `/usr/bin/traur` — scanner
- `/usr/bin/traur-hook` — pacman transaction scanner
- `/usr/bin/traur-makepkg` — pre-build wrapper
- `/usr/share/libalpm/hooks/traur.hook` — pre-install hook

If building traur from this repository instead:

```bash
cargo build --release
sudo install -Dm755 target/release/traur /usr/bin/traur
sudo install -Dm755 target/release/traur-hook /usr/bin/traur-hook
sudo install -Dm755 scripts/traur-makepkg /usr/bin/traur-makepkg
sudo install -Dm644 hook/traur.hook /usr/share/libalpm/hooks/traur.hook
```

### 2. Configure yay

Tell yay to use wrapper instead of raw `makepkg`:

```bash
yay --makepkg /usr/bin/traur-makepkg --save
```

Verify:

```bash
yay -P -g | grep -E 'makepkgbin|pacmanbin'
```

Expected:

```json
"makepkgbin": "/usr/bin/traur-makepkg",
"pacmanbin": "/usr/bin/pacman"
```

Keep `pacmanbin` as `/usr/bin/pacman`. Do not use network-disabling pacman wrappers.

### 3. Configure paru

Paru has separate config. It does not read yay settings.

Create or open:

```bash
mkdir -p ~/.config/paru
# Open ~/.config/paru/paru.conf in your editor
```

Add these options under `[bin]`. Reuse existing `[bin]`; do not create duplicate sections:

```ini
[bin]
Makepkg = /usr/bin/traur-makepkg
Pacman = /usr/bin/pacman
```

One-shot alternative:

```bash
paru --makepkg /usr/bin/traur-makepkg -S PACKAGE_NAME
```

The one-shot option affects only that command. Config file setting persists.

### 4. Configure arch-update

Set arch-update to use yay or paru, then configure that helper above. No extra traur setting is needed.

### 5. Verify pre-build scanning

Run an AUR install or build:

```bash
yay -S PACKAGE_NAME
# or
paru -S PACKAGE_NAME
```

First scan should show:

```text
traur: scanning PKGBUILD before makepkg...
traur: PACKAGE_NAME (trust: SCORE/100)
```

A blocked scan stops the build:

```text
traur: pre-build scan blocked makepkg
```

The wrapper caches successful scans during current login session. Cache key includes PKGBUILD path, PKGBUILD SHA-256, and scanner SHA-256. Changed PKGBUILD creates new cache key and scans again. Repeated paru/yay makepkg phases may therefore show no second scan line.

To force a fresh scan, remove cache directory:

```text
/run/user/<uid>/traur-prebuild-<uid>
```

### Manual makepkg builds

Use wrapper, not raw `makepkg`:

```bash
cd /path/to/aur-package
/usr/bin/traur-makepkg
```

Wrapper scans first, then runs `/usr/bin/makepkg` with original arguments. Failed scans block build.

### Checksum rename syntax

PKGBUILDs commonly map a local filename to a remote URL:

```bash
source=("package.tar.xz"::"https://example.com/package.tar.xz")
sha256sums=('...')
```

traur counts this as one source entry. It will not falsely report a checksum-count mismatch for this syntax.

### Pacman behavior

Pacman does not build AUR packages. It installs already-built packages. Its hook runs before package files and install scripts are committed:

```text
pacman transaction
  -> traur PreTransaction hook
  -> package installation
```

The hook skips official repository packages and scans AUR targets. `AbortOnFail` blocks transactions on malicious results or scan errors.

Hook settings:

```ini
When = PreTransaction
Exec = /usr/bin/traur-hook
AbortOnFail
NetworkAccess = allowed
```

`NetworkAccess = allowed` grants network only to `traur-hook`. Do not add `DisableSandboxNetwork` to `/etc/pacman.conf`.

Verify hook installation:

```bash
grep -E 'When|Exec|AbortOnFail|NetworkAccess' \
  /usr/share/libalpm/hooks/traur.hook
```

### Important limit

Pre-build scan analyzes PKGBUILD before makepkg executes it. It cannot inspect commands hidden inside downloaded source archives. The pre-install hook remains final gate before pacman commits package files and install scripts.

## How it works

14 independent features emit scored signals per package, then a context-aware scoring
pipeline computes the final trust score.

| Feature | What it checks |
| --------- | --------------- |
| PKGBUILD analysis | Dangerous shell code, NPM obfuscated exec, atomic-lockfile |
| Install script analysis | Suspicious .install hooks |
| Source URL analysis | Untrusted source domains |
| Checksum analysis | Missing, skipped, or weak checksums |
| Metadata analysis | AUR votes, popularity, maintainer status |
| Name analysis | Typosquatting and brand impersonation |
| Maintainer analysis | New accounts, batch uploads |
| Orphan takeover analysis | Submitter != maintainer, orphan takeover patterns |
| Git history analysis | New network code, author changes |
| Shell analysis | Beyond-regex obfuscation |
| PKGBUILD diff analysis | Checksum changes, domain changes, major rewrites |
| GTFOBins analysis | Legitimate binary abuse |
| Bin source verification | -bin package source domain vs upstream URL mismatch |
| AUR comments analysis | Security keywords in AUR comments (time-aware) |

### Scoring pipeline

1. **Community gate** — time-aware AUR comment threat evaluation
2. **Critical gate** — signals that alone classify a package as Malicious
3. **Override gate** — high-severity signals (curl-pipe-bash, reverse shells, etc.)
4. **Weighted risk** — composite score from all signals (15% Metadata, 45% PKGBUILD, 25% Behavioral, 15% Temporal)
5. **Maintainer trust** — account age, package count, takeover recency multiplier
6. **Popularity penalty** — low votes/usage increases risk
7. **Orphan + malicious diff boost** — takeover combined with new suspicious diff → risk ≥ 95
8. **NPM risk** — suspicious install scripts, new maintainers, dead repos
9. **Clamp & tier** — 5 tiers: Trusted(81-100), OK(61-80), Sketchy(41-60), Suspicious(21-40), Malicious(0-20)

### Time-aware comment threat

AUR comments mentioning malware/backdoor/etc. are evaluated with time-awareness and
popularity context, preventing stale or mitigated warnings from falsely classifying
packages as Malicious.

High-popularity repos (≥3 votes or ≥0.01 popularity):

- < 7 days old → Malicious override
- 7–60 days → degraded signal
- > 60 days → ignored

Low-popularity repos:

- Degraded if mitigation/follow-up comments exist after the warning
- Always fires if no mitigation and the warning is > 60 days old (orphaned concern)

Mitigation phrases ("patched", "fixed", "not compromised", "different package", etc.)
in comments after a warning automatically downgrade the threat.

## Detection coverage

Patterns derived from real AUR malware incidents:

- **CHAOS RAT (2025)** — browser impersonation packages, RAT distribution
- **Google Chrome RAT (2025)** — .install script, Python download+execute
- **Acroread (2018)** — orphan takeover, curl from paste service, systemd persistence

Categories: download-and-execute, reverse shells, credential theft, persistence mechanisms, privilege escalation, C2/exfiltration, cryptocurrency mining, code obfuscation, kernel module loading, environment variable theft, system reconnaissance.

## Automatic GitHub releases

Every push to `main` runs `.github/workflows/release-on-main.yml`:

1. Runs `cargo test --locked`.
2. Builds release binaries inside Arch Linux.
3. Creates a pacman-installable `.pkg.tar.zst` package and `.sha256` file.
4. Creates prerelease tag `v<CargoVersion>-<commit-sha>`.
5. Creates GitHub release notes from tip commit message and uploads both assets.

Example tag:

```text
v0.4.1-a1b2c3d4e5f6
```

Download the `.pkg.tar.zst` asset, verify it, then install it:

```bash
sha256sum -c traur-*.pkg.tar.zst.sha256
sudo pacman -U traur-*.pkg.tar.zst
```

These are development snapshots. Versioned stable releases still need a Cargo version bump and the normal package/AUR release workflow.

## License

MIT
