# Volume Controls and Hotkey Menu Plan

## Goal

Add keyboard volume control and an in-app hotkey reference menu.

Initial scope:

- `-` lowers volume by 2%.
- `+` raises volume by 2%.
- `=` also raises volume by 2% for keyboards where `+` requires Shift.
- `?` opens/closes a hotkey menu.
- `Esc` closes the hotkey menu.

## Wave 0: Confirm Shape

1. Volume step is `2%`.
2. Hotkeys:
   - `-` volume down.
   - `+` volume up.
   - `=` volume up.
3. Help menu:
   - `?` opens/closes.
   - `Esc` closes.
4. Volume range is clamped to `0..=100`.

## Wave 1: State and MPV Volume

Tasks:

1. Add `volume: i32` to `AppState`, defaulting to `100`.
2. Add `MpvController::get_volume() -> Result<i32, AudioError>`.
3. Keep `MpvController::set_volume()` as the clamp-safe setter.
4. On app start, optionally read mpv volume after mpv starts and sync `state.volume`.

Acceptance:

- State has current volume.
- MPV volume read/write compiles.
- No UI behavior changes yet.

## Wave 2: Volume Hotkeys

Tasks:

1. Add an app helper: `adjust_volume(delta: i32)`.
2. Helper behavior:
   - Read current `state.volume`.
   - Add `delta`.
   - Clamp to `0..=100`.
   - Call `mpv.set_volume(new_volume)`.
   - Save `state.volume = new_volume`.
   - Notify `Volume: {new_volume}%`.
3. Add global key handling:
   - `-` -> `adjust_volume(-2)`.
   - `+` -> `adjust_volume(2)`.
   - `=` -> `adjust_volume(2)`.
4. Preserve text-entry behavior:
   - Server fields should still type these characters normally.
   - Artist/Browse active filters should still type these characters normally.

Acceptance:

- `-` lowers by 2.
- `+` raises by 2.
- `=` raises by 2.
- Volume never goes below 0 or above 100.
- Server/filter typing is not broken.

## Wave 3: MPRIS Volume Sync

Tasks:

1. Change `mpris::volume()` to return `state.volume / 100.0`.
2. Change `AudioAction::SetVolume(vol)` handling:
   - Clamp `vol`.
   - Call `mpv.set_volume(vol)`.
   - Update `state.volume`.
3. Avoid noisy UI notifications for MPRIS volume changes.

Acceptance:

- Desktop volume control and TUI volume hotkeys stay aligned.

## Wave 4: Hotkey Menu State

Tasks:

1. Add `show_hotkeys: bool` to `AppState`.
2. Add global `?` handling to toggle `show_hotkeys`.
3. Add early close behavior:
   - If `show_hotkeys` and key is `Esc`, close and consume key.
   - If `show_hotkeys` and key is `?`, close and consume key.
4. While menu is open, only these keys should remain active:
   - `Esc`
   - `?`
   - `q`
5. Ignore other keys while the menu is open.

Acceptance:

- `?` opens the menu.
- `?` closes the menu.
- `Esc` closes the menu.
- Help overlay blocks accidental playback/navigation.

## Wave 5: Hotkey Menu UI

Tasks:

1. Add a new widget or render helper, likely one of:
   - `src/ui/widgets/hotkeys.rs`
   - `src/ui/help.rs`
2. Render a centered modal over the current screen.
3. Include sections:
   - Global
   - Current page
   - Volume
4. Include core global rows:
   - `q` quit
   - `p` / `Space` play/pause
   - `h` previous
   - `l` next
   - `-` volume down
   - `+` / `=` volume up
   - `Ctrl+R` refresh
   - `t` cycle theme
   - `F1..F7` page navigation
   - `?` hotkey menu
5. Include page-specific rows based on `state.page`.
6. Render from `src/ui/layout.rs` after normal UI render so the menu appears on top.

Acceptance:

- Menu is readable on normal terminal size.
- Small terminals degrade safely.
- Current page shortcuts are visible.

## Wave 6: Footer Hints

Tasks:

1. Add `-/+ Volume` to global footer bindings.
2. Add `? Help` to global footer bindings.
3. Remove duplicate Browse `/ Search` while touching footer.

Acceptance:

- Footer shows new shortcuts.
- No footer overflow panic.

## Wave 7: Documentation

Tasks:

1. Update `README.md`:
   - Mention volume hotkeys.
   - Mention the hotkey menu.
2. Update `docs/keybindings.md`:
   - Add `-` volume down.
   - Add `+` / `=` volume up.
   - Add `?` hotkey menu.
   - Add `Esc` closes menu when menu is open.

Acceptance:

- Docs match implementation.

## Wave 8: Verification

Commands:

```bash
cargo fmt
cargo test
```

If compile fails, fix the smallest issue.

If tests fail for unrelated reasons, record the exact failing test and reason.

Manual checklist:

- `-` lowers volume by `2%`.
- `+` raises volume by `2%`.
- `=` raises volume by `2%`.
- Volume clamps at `0` and `100`.
- `?` opens help.
- `Esc` closes help.
- Help overlay does not trigger playback/navigation while open.
- Server text entry still accepts `-`, `+`, `=`, and `?` normally when a text field is focused.

## Deferred Decisions

- Whether to show current volume in the now-playing widget or footer permanently.
- Whether to support mouse interaction for the hotkey menu.
- Whether to add configurable volume step size later.
