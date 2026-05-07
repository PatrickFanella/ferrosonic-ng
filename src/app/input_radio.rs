use crossterm::event::{self, KeyCode};

use crate::error::Error;
use crate::subsonic::models::InternetRadioStation;

use super::*;

impl App {
    /// Handle radio page keys
    pub(super) async fn handle_radio_key(&mut self, key: event::KeyEvent) -> Result<(), Error> {
        let mut state = self.state.write().await;

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                state.radio.selected =
                    move_selection_up(state.radio.selected, state.radio.stations.len());
            }
            KeyCode::Down | KeyCode::Char('j') => {
                state.radio.selected =
                    move_selection_down(state.radio.selected, state.radio.stations.len());
            }
            KeyCode::Enter => {
                if let Some(station) = selected_station(&state) {
                    drop(state);
                    return self.play_radio_station(station).await;
                }
            }
            KeyCode::Char(' ') => {
                if let Some(station) = selected_station(&state) {
                    let is_current = state
                        .now_playing
                        .radio_station
                        .as_ref()
                        .map(|current| current.id == station.id)
                        .unwrap_or(false);
                    drop(state);

                    if is_current {
                        return self.toggle_pause().await;
                    }
                    return self.play_radio_station(station).await;
                }

                drop(state);
                return self.toggle_pause().await;
            }
            _ => {}
        }

        Ok(())
    }
}

fn selected_station(state: &AppState) -> Option<InternetRadioStation> {
    state
        .radio
        .selected
        .and_then(|idx| state.radio.stations.get(idx).cloned())
}

fn move_selection_up(selected: Option<usize>, len: usize) -> Option<usize> {
    match (selected, len) {
        (_, 0) => None,
        (Some(sel), _) if sel > 0 => Some(sel - 1),
        (Some(sel), _) => Some(sel),
        (None, _) => Some(0),
    }
}

fn move_selection_down(selected: Option<usize>, len: usize) -> Option<usize> {
    match (selected, len) {
        (_, 0) => None,
        (Some(sel), _) if sel + 1 < len => Some(sel + 1),
        (Some(sel), _) => Some(sel),
        (None, _) => Some(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radio_selection_moves_with_bounds() {
        assert_eq!(move_selection_up(None, 0), None);
        assert_eq!(move_selection_down(None, 0), None);
        assert_eq!(move_selection_up(None, 3), Some(0));
        assert_eq!(move_selection_down(None, 3), Some(0));
        assert_eq!(move_selection_up(Some(0), 3), Some(0));
        assert_eq!(move_selection_up(Some(2), 3), Some(1));
        assert_eq!(move_selection_down(Some(0), 3), Some(1));
        assert_eq!(move_selection_down(Some(2), 3), Some(2));
    }

    #[test]
    fn selected_station_returns_cloned_station() {
        let mut state = AppState::default();
        state.radio.stations = vec![InternetRadioStation {
            id: "id-1".to_string(),
            name: "Station".to_string(),
            stream_url: "https://example.com/stream".to_string(),
            home_page_url: None,
            cover_art: None,
        }];
        state.radio.selected = Some(0);

        let station = selected_station(&state).expect("station expected");
        assert_eq!(station.id, "id-1");
        assert_eq!(station.name, "Station");
    }
}
