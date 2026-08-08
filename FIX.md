# Complaints & Issues — Status

All items below are now fixed, resolved in documentation, or intentionally retained with rationale.

Collected from [Reddit r/archlinux](https://www.reddit.com/r/archlinux/comments/1qyl2b4/aur_malware_scanner_in_rust/) and [EndeavourOS forum](https://forum.endeavouros.com/t/new-rust-tool-traur-analyzes-arch-aur-packages-for-hidden-risks/78001/22).

## High Priority

### ~~1. megasync `nc -c` false positive (MALICIOUS on legit package)~~

- ~~`P-REVSHELL-NC` fires on `git -C MEGAsync -c protocol.file.allow='always' submodule update` because it contains `nc -c`~~
- ~~Flags megasync as MALICIOUS via override gate~~
- ~~Source: EndeavourOS (dalto identified root cause)~~
- **Fixed**: Added `\b` word boundary before `(nc|ncat)` in P-REVSHELL-NC and G-BINDSHELL-NC patterns

### ~~2. Typosquat false positives on `-bin` variants and `python-*` wrappers~~

- ~~`python-steam` flagged for embedding "steam"~~
- ~~`proton-ge-custom-bin` flagged for embedding "proton-ge-custom"~~
- ~~`-bin` suffix packages of real packages shouldn't trigger typosquat~~
- ~~`python-*` prefix packages wrapping upstream libs shouldn't trigger typosquat~~
- ~~Source: EndeavourOS (multiple users), Reddit~~
- **Fixed**: Exact conventional variants (`foo-bin`, `python-foo`, and their combination) no longer trigger containment typosquat. Raw-name impersonation checks remain active, so names such as `firefox-fix-bin` still get flagged.

### ~~3. Hook asks confirmation for every package, even clean ones~~

- ~~Should only prompt on SKETCHY+ results, not TRUSTED/OK~~
- ~~"That hook is pretty annoying if you have a lot of AUR packages. It asks for confirmation one at a time for every AUR package you update even if there are no issues flagged." — dalto (EOS maintainer)~~
- ~~Source: EndeavourOS~~
- **Fixed**: Hook collects results silently, shows one tier summary, prints detail only for SKETCHY+ packages, skips prompts for TRUSTED/OK, and hard-blocks only MALICIOUS results or scan errors.

### ~~4. Flag spacing bypass (`rm -r -f` vs `rm -rf`)~~

- ~~`rm -r -f /var/log` is not detected, only `rm -rf /var/log`~~
- ~~Patterns need to handle flag variations with spaces~~
- ~~Source: Reddit (ang-p)~~
- **Fixed**: Added flag-absorber regex fragments (`(-\S+\s+)*`, `(\S+\s+)*`, `[^;&|]*`) to 13 patterns across pkgbuild_analysis and install_script_analysis to handle split flags (`rm -r -f`), intervening flags (`chmod -v +x`), and flag+value pairs (`base64 -w 0 -d`)

## Medium Priority

### ~~5. SA-VAR-CONCAT-CMD too noisy~~

- ~~Fires on nearly every package that uses `sh` or `python` in build scripts~~
- ~~radarr, sonarr, peazip, python-ewmh, shell-color-scripts, python-steam, proton-ge-custom-bin all flagged~~
- ~~Needs better heuristic to distinguish suspicious from normal build usage~~
- ~~Source: EndeavourOS (appears in almost every user's results)~~
- **Fixed**: Signal now requires two distinct known variable references and a reconstructed dangerous word in command position. Literal suffixes, arguments, assignments, comments, and URLs no longer trigger it.

### ~~6. Checksum mismatch wording is confusing/alarming~~

- ~~`source count (7) != sha256sums count (5)` — users think this means missing checksums are malicious~~
- ~~Doesn't account for `source_x86_64`/`source_aarch64` having their own checksum arrays~~
- ~~Doesn't account for SKIP entries~~
- ~~Wording should be less alarming for common benign cases~~
- ~~Source: EndeavourOS (fred666, multiple users)~~
- **Fixed**: Mismatch wording now says `Checksum coverage needs review`, architecture-specific arrays are paired by suffix, mixed `SKIP` entries count correctly, all checksum algorithms are checked, and spaced assignments are parsed.

### ~~7. freetube-bin checksum mismatch false positive~~

- ~~`-bin` packages commonly use `source_x86_64` and `source_aarch64` with separate checksum arrays~~
- ~~Cross-array count comparison is wrong for these~~
- ~~Source: EndeavourOS (thefrog), Reddit~~
- **Fixed**: Generic and architecture-specific arrays are compared only with matching suffixes. `filename::url` source syntax is also counted as one source entry.

## Low Priority / Messaging

### ~~8. "Just a big grep against patterns.toml" perception~~

- ~~Highlight shell_analysis, gtfobins_analysis, behavioral features more prominently~~
- ~~ang-p: "essentially a big grep against patterns.toml"~~
- ~~Source: Reddit~~
- **Resolved**: README now highlights shell analysis, GTFOBins, behavioral analysis, git history, metadata, comments, and other independent features.

### ~~9. Branding as "trust engine" not "malware scanner"~~

- ~~Reddit title said "malware scanner" — sets wrong expectations~~
- ~~FanClubof5 (24 upvotes): "You might have better luck branding it as a trust engine"~~
- **Resolved**: Project and README use trust scoring language. Detection limitations are documented.
- Source: Reddit

### ~~10. ALPM hook not "yay/paru hook"~~

- ~~Hook is an ALPM hook (works with pacman too), not specific to AUR helpers~~
- ~~Documentation should use correct terminology~~
- ~~Source: Reddit (Hermocrates)~~
- **Resolved**: README identifies `traur.hook` as an ALPM `PreTransaction` hook and documents pacman behavior.

### ~~11. Rust packaging guidelines not followed in PKGBUILD~~

- ~~Missing `prepare()` steps from Arch Rust package guidelines~~
- ~~`--frozen` flag caused build failures for users with different dependency versions~~
- ~~Source: Reddit (Hermocrates, NeKon69)~~
- **Resolved/stale**: `pkg/PKGBUILD` fetches locked dependencies in `prepare()`, then builds/tests with `--frozen`. This is intentional reproducible Arch packaging; removing `--frozen` would allow lockfile changes and network access during build. `Cargo.lock` must stay synchronized with `Cargo.toml` during releases.
