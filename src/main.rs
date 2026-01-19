#![feature(const_convert)]
#![feature(const_trait_impl)]

extern crate image;
#[macro_use]
extern crate approx;

mod menu;
mod game;
mod leaderboard;

use menu::{AppMenuState, MenuAction};
use game::{AppGameState, GameAction};
use leaderboard::{AppLeaderboardState, LeaderboardAction};

use std::fmt;
use std::time::{Duration, Instant};
use std::collections::LinkedList;

use color_eyre::{eyre::Context, Result};
use ratatui::{
    backend::Backend,
    crossterm::event::{self, Event},
    DefaultTerminal, Frame,
};

#[used]
#[unsafe(link_section = ".leaderboard")]
static mut LEADERBOARD: [([char; 16], u16); 10] = [
  (['a','a','a','a','a','a','a','a','a','a','a','a','a','a','a','a'],0),
  (['a','a','a','a','a','a','a','a','a','a','a','a','a','a','a','a'],0),
  (['a','a','a','a','a','a','a','a','a','a','a','a','a','a','a','a'],0),
  (['a','a','a','a','a','a','a','a','a','a','a','a','a','a','a','a'],0),
  (['a','a','a','a','a','a','a','a','a','a','a','a','a','a','a','a'],0),
  (['a','a','a','a','a','a','a','a','a','a','a','a','a','a','a','a'],0),
  (['a','a','a','a','a','a','a','a','a','a','a','a','a','a','a','a'],0),
  (['a','a','a','a','a','a','a','a','a','a','a','a','a','a','a','a'],0),
  (['a','a','a','a','a','a','a','a','a','a','a','a','a','a','a','a'],0),
  (['a','a','a','a','a','a','a','a','a','a','a','a','a','a','a','a'],0),].into();

enum AppState {
  Menu(AppMenuState),
  Game(AppGameState),
  Leaderboard(AppLeaderboardState),
}

#[derive(PartialEq)]
enum Action {
  DoNothing,
  CloseApp,
  UpdateGameState(GameAction)
}

#[derive(Debug)]
enum EventHandlerError {
  EventBufferEmpty,
}

impl std::error::Error for EventHandlerError {}

impl fmt::Display for EventHandlerError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      EventHandlerError::EventBufferEmpty => write!(f, "EventBufferEmpty")
    }
  }
}

struct EventHandler {
  rx: std::sync::mpsc::Receiver<Event>,
}

impl EventHandler {
  fn new(tick_rate: Duration) -> Self {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
      loop {
        let _action = Action::DoNothing;
        if event::poll(tick_rate)
            .context("event poll failed").unwrap() {
          let event = event::read().context("event read failed").unwrap();
          tx.send(event).unwrap()
        }
      }
    });
    Self { rx }
  }
  fn get_events(&self) -> Result<Vec<Event>> {
    let mut result = Vec::<Event>::new();
    for event in self.rx.try_iter() {
      result.push(event);
    }
    match result.is_empty() {
      false => Ok(result),
      true => Err(EventHandlerError::EventBufferEmpty.into())
    }
  }
}

struct App {
  states: LinkedList<AppState>,
  time_tick: Instant,
  tick_rate: Duration,
}

impl App {
  fn new() -> Self {
    let menu_state: AppState = AppState::Menu(AppMenuState::new());
    let game_state: AppState = AppState::Game(AppGameState::new());
    let leaderboard_state: AppState = 
      AppState::Leaderboard(unsafe { AppLeaderboardState::new() } );
    
    let mut states = LinkedList::<AppState>::new();
    states.push_front(game_state);
    states.push_front(leaderboard_state);
    states.push_front(menu_state);
    
    
    let time_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);
    
    Self {
      states,
      time_tick,
      tick_rate,
    }
  }
  /// Run the application loop. This is where you would handle events and update the 
  /// application state.
  fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
      let event_handler = EventHandler::new(self.tick_rate / 10);
      let mut actions = Vec::<Action>::new();
      loop {
        // Draw all primitives for current state
        terminal.draw(|frame| self.draw(frame))?;
        // Handle events for current state
        if let Ok(events) = event_handler.get_events() {
          for event in events.into_iter() {
            actions.push(self.handle_event(event)); 
          }
          if actions.iter().any(|a| *a == Action::CloseApp) {
            break;
          }
        }
        
        // Update game world if needed
        {
          let current_state = self.states
              .front_mut()
              .expect("Valid current state on first position");
          let mut game_actions = Vec::<GameAction>::new();
          for action in actions.iter() {
            if let Action::UpdateGameState(new_game_action) = action {
              if *new_game_action == GameAction::DoNothing { continue; }
              game_actions.push(new_game_action.clone());
            }
          }
          if let AppState::Game(app_game_state) = current_state {
            if self.time_tick.elapsed() >= self.tick_rate {
              app_game_state.on_tick(&game_actions);
              game_actions.clear();
              self.time_tick = Instant::now();
            }
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
      AppState::Leaderboard(state) => state.draw(frame),
    }
  } 
    
  pub fn handle_event(&mut self, event: Event) -> Action {
    // Process menu app state events
    {
      let current_state = self.states
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
            MenuAction::SwitchToLeaderboard => {
              let next_state = self.states
                .extract_if(|state| {
                  if let AppState::Leaderboard(_) = state {
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
    }
    // Process game app state events
    {
      let current_state = self.states
          .front_mut()
          .expect("Valid current state on first position");
      if let AppState::Game(app_game_state) = current_state {
        let mut action: GameAction = GameAction::DoNothing;
        if let Event::Key(key) = event {
          action = app_game_state.handle_key_press(key);
        }
        if let GameAction::ReturnToMenu(ref name, score) = action {
          *app_game_state = AppGameState::new();
          let next_state = self.states
            .extract_if(|state| {
              if let AppState::Menu(_) = state {
                return true;
              }
              return false;
            })
            .next().expect("Right new app state should be stored");
          let mut leaderboard_state = self.states
            .extract_if(|state| {
              if let AppState::Leaderboard(_) = state {
                return true;
              }
              return false;
            })
            .next().expect("Right new app state should be stored");
          if let AppState::Leaderboard(ref mut inner_leaderboard_state) = leaderboard_state {
            inner_leaderboard_state.update_board(name, score);
          }
          self.states.push_front(next_state);
          self.states.push_back(leaderboard_state);
        }
        return Action::UpdateGameState(action);
      }
    }
    
    {
      let current_state = self.states
          .front_mut()
          .expect("Valid current state on first position");
      if let AppState::Leaderboard(app_leaderboard_state) = current_state {
        let mut action: LeaderboardAction = LeaderboardAction::DoNothing;
        if let Event::Key(key) = event {
          action = app_leaderboard_state.handle_key_press(key);
        }
        if let LeaderboardAction::ReturnToMenu = action {
          let next_state = self.states
            .extract_if(|state| {
              if let AppState::Menu(_) = state {
                return true;
              }
              return false;
            })
            .next().expect("Right new app state should be stored");
          self.states.push_front(next_state);
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

