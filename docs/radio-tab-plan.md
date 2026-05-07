# Radio Tab Implementation Plan

## Goal

Add a Radio tab for browsing and playing Navidrome/Subsonic internet radio stations.

## First PR Scope

- Add a Radio tab before Server.
- Browse stations returned by `getInternetRadioStations`.
- Play selected station via its direct `streamUrl`.
- Display live stream metadata from mpv when available.
- Fall back to the station name when live metadata is unavailable.
- Preserve global play/pause behavior while making `Space` useful on Radio.
- Do not add station creation/editing/deletion in this PR.

## Tab Order and Shortcuts

| Key | Page |
| --- | --- |
| `F1` | Browse |
| `F2` | Artists |
| `F3` | Queue |
| `F4` | Playlists |
| `F5` | Radio |
| `F6` | Server |
| `F7` | Settings |

## Radio Keybindings

| Key | Action |
| --- | --- |
| `Up` / `k` | Move selection up |
| `Down` / `j` | Move selection down |
| `Enter` | Play selected station |
| `Space` | Play selected station if not current; otherwise toggle pause |
| `Ctrl+R` | Refresh all data, including radio stations |

## API Research

Use the Subsonic/OpenSubsonic internet radio endpoint:

- `getInternetRadioStations`
- Introduced in Subsonic API 1.9.0.
- No extra params.
- Navidrome lists it as supported.
- Response contains `internetRadioStations.internetRadioStation[]`.

Station fields seen in OpenSubsonic examples:

- `id`
- `name`
- `streamUrl`
- `homePageUrl`
- `coverArt`

Important: station playback should use `streamUrl` directly. Do not call `/rest/stream?id={station.id}`; radio station IDs are not normal media IDs.

## Data Model Changes

### `src/subsonic/models.rs`

Add:

```rust
pub struct InternetRadioStation {
    pub id: String,
    pub name: String,
    #[serde(rename = "streamUrl")]
    pub stream_url: String,
    #[serde(rename = "homePageUrl")]
    pub home_page_url: Option<String>,
    #[serde(rename = "coverArt")]
    pub cover_art: Option<String>,
}

pub struct InternetRadioStationsData {
    #[serde(rename = "internetRadioStation", default)]
    pub internet_radio_station: Vec<InternetRadioStation>,
}
```

Names may be adjusted to match repo style.

### `src/subsonic/client.rs`

Add:

```rust
pub async fn get_internet_radio_stations(
    &self,
) -> Result<Vec<InternetRadioStation>, SubsonicError>
```

Implementation pattern should match `get_playlists()`.

## App State Changes

### `src/app/state.rs`

Add `Page::Radio` between `Playlists` and `Server`.

Update:

- `Page::index()`
- `Page::label()`
- `Page::shortcut()`

Add:

```rust
pub struct RadioState {
    pub stations: Vec<InternetRadioStation>,
    pub selected: Option<usize>,
    pub scroll_offset: usize,
}
```

Add to `AppState`:

```rust
pub radio: RadioState,
```

Extend `NowPlaying` with radio-specific fields:

```rust
pub radio_station: Option<InternetRadioStation>,
pub radio_title: Option<String>,
pub radio_artist: Option<String>,
```

Song playback must clear `radio_station`, `radio_title`, and `radio_artist`.
Radio playback must set `song = None`.

## Loading Stations

### `src/app/repo.rs`

Add `get_radio_stations()`:

- If no client, return.
- Call `client.get_internet_radio_stations()`.
- Store stations in `state.radio.stations`.
- Set `state.radio.selected = Some(0)` if non-empty.
- Reset selection to `None` if empty.
- Notify error on failure.

### `src/app/mod.rs`

Update `load_initial_data()` to also load radio stations.

`Ctrl+R` already calls `load_initial_data()`, so refresh should include Radio.

## Playback Changes

### `src/app/playback.rs`

Add `play_radio_station(station: InternetRadioStation)`.

Behavior:

- Clear or detach song queue state. Recommendation: clear queue to prevent `Next` / `Previous` from unexpectedly leaving radio for an old queue.
- Set `queue_position = None`.
- Set `now_playing.song = None`.
- Set `now_playing.radio_station = Some(station.clone())`.
- Set `now_playing.radio_title = None` and `radio_artist = None` initially.
- Set `now_playing.state = PlaybackState::Playing`.
- Set `position = 0.0`, `duration = 0.0`.
- Reset audio quality fields.
- Call `mpv.loadfile(&station.stream_url)`.
- Do not call `preload_next_track()`.
- Do not scrobble.

Update `update_playback_info()`:

- Scrobble only when `now_playing.song.is_some()`.
- For radio, poll mpv metadata and update `radio_title` / `radio_artist`.
- If mpv is idle while radio is active, stop radio playback instead of calling `next_track()`.

Update `next_track()` / `prev_track()`:

- If radio is active and queue is empty, no-op or stop.
- Do not attempt queue advancement for radio.

## mpv Metadata

### `src/audio/mpv.rs`

Add a helper around `get_property metadata`.

Potential metadata fields to inspect:

- `icy-title`
- `title`
- `artist`
- `album`
- `icy-name`

Parsing priority:

1. Use `artist` + `title` if both exist.
2. Use `icy-title` if present.
3. If `icy-title` looks like `Artist - Title`, split it into artist/title.
4. Else set title to raw metadata string.
5. Fallback title is station name.
6. Fallback artist is `Internet Radio`.

Only poll metadata for active radio playback.

## Input Changes

### `src/app/input.rs`

Update global F-key routing:

- `F5` -> `Page::Radio`
- `F6` -> `Page::Server`
- `F7` -> `Page::Settings`

Add page dispatch:

```rust
Page::Radio => self.handle_radio_key(key).await,
```

Special-case `Space` while on Radio:

- If selected station is not current, play it.
- If selected station is current, use existing toggle pause behavior.

### New file: `src/app/input_radio.rs`

Implement list navigation and play behavior.

## UI Changes

### New file: `src/ui/pages/radio.rs`

Render a single-pane station list.

Recommended display:

- Title: `Radio ({count})`
- Empty state: `No radio stations found`
- Current station marker: `▶`
- Row content: station name plus optional homepage or stream URL in muted style.

Highlight current station by comparing `now_playing.radio_station.id` to the row station ID.

### `src/ui/layout.rs`

Render `Page::Radio`.

Radio is single-pane; do not include it in dual-pane layout unless later needed.

### `src/ui/header.rs`

Update tab labels and mouse hit testing arrays.

### `src/ui/footer.rs`

Add Radio key hints and shift Server/Settings shortcuts.

### `src/ui/pages/mod.rs`

Export `radio` module.

## Now Playing Widget

### `src/ui/widgets/now_playing.rs`

Current widget only knows `now_playing.song`.

Update it to display either:

- song metadata, current behavior; or
- radio metadata.

Radio display:

- Title: `radio_title.unwrap_or(station.name)`
- Artist: `radio_artist.unwrap_or("Internet Radio")`
- Subtitle/album line: station homepage if present, otherwise station name or stream host.
- Quality line: use existing mpv audio quality fields when present.
- Progress: for streams with no duration, avoid `00:00 / 00:00`; show elapsed only or omit duration bar.

## MPRIS Changes

### `src/mpris/server.rs`

Current metadata uses `state.current_song()` only.

Add radio metadata support:

- Track ID: `/org/mpris/MediaPlayer2/Radio/{id}`
- Title: live radio title or station name
- Artist: live radio artist or `Internet Radio`
- Album: station name or homepage
- Length omitted for radio streams
- Cover art URL: if station `cover_art` exists, use existing `getCoverArt` URL builder

Update capability methods so radio does not advertise queue next/previous unless a queue is actually active.

## Mouse Changes

### New file: `src/app/mouse_radio.rs`

Behavior:

- Click station row: select station.
- Double-click station row: play station.
- Scroll: update `radio.scroll_offset`.

### `src/app/mouse.rs`

Route Radio content clicks and scroll events.

## Tests

### `src/subsonic/tests.rs`

Add tests:

- `get_internet_radio_stations_ok`
- `get_internet_radio_stations_empty`
- `get_internet_radio_stations_api_error`
- `homePageUrl` deserializes into `home_page_url`
- optional `coverArt` deserializes into `cover_art`

Possible future tests:

- mpv metadata parser splits `Artist - Title`.
- radio fallback uses station name when no metadata exists.

## Docs

Update:

- `README.md`
- `docs/keybindings.md`

Mention:

- Radio tab support.
- New F-key order.
- Radio keybindings.

## Deferred Work

Add station management in a later PR.

Potential endpoint:

- `createInternetRadioStation`

Params:

- `name`
- `streamUrl`
- optional `homepageUrl`

Reason deferred:

- Adds a text-entry form and validation flow.
- Standard Subsonic API does not appear to support setting a radio logo/cover.
- Better after browse/play/metadata path is stable.

Future add-station UI:

- `a` opens form.
- Fields: name, stream URL, homepage URL.
- `Tab` moves fields.
- `Enter` on Save submits.
- `Esc` cancels.
- Refresh station list after successful create.

## Verification

Run:

```bash
cargo fmt
cargo test
```

Manual checks:

- Radio tab appears at `F5`.
- Server appears at `F6`.
- Settings appears at `F7`.
- Radio station list loads from Navidrome.
- `Enter` plays selected station.
- `Space` plays selected station or toggles pause if current.
- Now Playing shows live metadata when station provides it.
- Now Playing falls back to station name when metadata is missing.
- MPRIS shows radio metadata.
- Song playback still works after radio playback.
- Radio playback still works after song playback.
