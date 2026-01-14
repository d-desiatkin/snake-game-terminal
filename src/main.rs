extern crate image;

mod menu;
mod game;

use menu::{AppMenuState, MenuAction};
use game::AppGameState;

use std::io;
use std::time::Duration;
use std::collections::LinkedList;

use color_eyre::{eyre::Context, Result};
use ratatui::{
    Terminal,
    backend::{Backend, TermionBackend},
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
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
  LeaderBoard(()),
}

#[derive(PartialEq)]
enum Action {
  DoNothing,
  CloseApp,
}

struct App {
  states: LinkedList<AppState>,
}

impl App {
  fn new() -> Self {
    let mut menu_state: AppState = AppState::Menu(AppMenuState::new());
    let mut game_state: AppState = AppState::Game(AppGameState::new());
    
    let mut states = LinkedList::<AppState>::new();
    states.push_front(game_state);
    states.push_front(menu_state);
    
    Self {
      states
    }
  }
  /// Run the application loop. This is where you would handle events and update the 
  /// application state.
  fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
      loop {
          // Draw all primitives for current state
          terminal.draw(|frame| self.draw(frame))?;
          // Handle events for current state
          if event::poll(Duration::from_millis(250)).context("event poll failed")? {
            let event = event::read().context("event read failed")?;
            let action = self.handle_event(event);
            if action == Action::CloseApp {
              break;
            }
          }
      }
      Ok(())
  }
  
  fn draw(&mut self, frame: &mut Frame) {
    let mut current_state = self.states
      .front_mut()
      .expect("Valid current state on first pos");
    match &mut current_state {
      AppState::Menu(state) => state.draw(frame),
      AppState::Game(state) => state.draw(frame),
      AppState::LeaderBoard(_) => todo!(),
    }
  } 
    
  pub fn handle_event(&mut self, event: Event) -> Action {
    let mut current_state = self.states
      .front_mut()
      .expect("Valid current state on first position");
    if let AppState::Menu(app_menu_state) = current_state {
      if let Event::Key(key) = event {
        let action = app_menu_state.handle_key_press(key);
        match action {
          MenuAction::SwitchToExit => {
            return Action::CloseApp;
          },
          MenuAction::SwitchToGame => {
            let next_state = self.states
              .extract_if(|state| {
                if let AppState::Game(_) = state {
                  return true;
                }
                return false;
              })
              .next().expect("Right new app state should be stored");
            self.states.push_front(next_state);
          },
          _ => ()
        }
      }
    }
    Action::DoNothing
  }
}


fn main() -> Result<()> {
    color_eyre::install()?; // augment errors / panics with easy to read messages
    let terminal = ratatui::init();
    let app_result = App::new().run(terminal);
    ratatui::restore();
    app_result
}

