extern crate image;

mod menu;
mod game;

use menu::AppMenuState;
use game::AppGameState;

use std::time::Duration;
use std::collections::LinkedList;

use color_eyre::{eyre::Context, Result};
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    widgets::Paragraph,
    layout::{
      Constraint::{self, Length, Max, Min, Percentage, Ratio},
      Layout, Rect
    },
    text::Line,
    DefaultTerminal, Frame,
};
use ratatui_image::StatefulImage;

enum AppState {
  Menu(AppMenuState),
  Game(AppGameState),
}

struct App {
  current_state: AppState,
  states: LinkedList<AppState>,
}

impl App {
  fn new() -> Self {
    let mut MenuState: AppState = AppState::Menu(AppMenuState::new());
    let mut GameState: AppState = AppState::Game(AppGameState::new());
    
    let mut states = LinkedList::<AppState>::new();
    states.push_front(GameState);
    let mut current_state = MenuState;
    
    Self {
      current_state,
      states
    }
  }
  /// Run the application loop. This is where you would handle events and update the 
  /// application state.
  fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
      loop {
          terminal.draw(|frame| self.draw(frame))?;
          if event::poll(Duration::from_millis(250)).context("event poll failed")? {
            if let Event::Key(key) = event::read().context("event read failed")? {
              break;
            }
          }
      }
      Ok(())
  }
  
  fn draw(&mut self, frame: &mut Frame) {
    match &mut self.current_state {
      AppState::Menu(state) => state.draw(frame),
      AppState::Game(state) => state.draw(frame),
    }
  }
}


fn main() -> Result<()> {
    color_eyre::install()?; // augment errors / panics with easy to read messages
    let terminal = ratatui::init();
    let app_result = App::new().run(terminal);
    ratatui::restore();
    app_result
}

