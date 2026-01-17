use std::collections::VecDeque;
use rand::rand_core::{TryRngCore, OsRng};
use ratatui::{
  Frame,
  crossterm::event::{self, Event, KeyEvent, KeyCode, KeyEventKind},
  layout::{Constraint, Layout, Position, Rect, Flex},
  style::{Color, Stylize},
  symbols::Marker,
  text::Line as TLine,
  widgets::{
    Block, Clear,
    canvas::{Canvas, Line, Points},
    Paragraph, Widget, Wrap
  },
};

#[derive(Clone, Copy, PartialEq)]
pub enum SnakeSegmentDirection {
  Up,
  Down,
  Left,
  Right
}

pub struct SnakeSegment {
  line: Line,
  direction: SnakeSegmentDirection,
}

pub struct SnakeState {
  segments: VecDeque<SnakeSegment>,
  total_length: f64,
  consumed_food: VecDeque<(f64, f64)>,
}

pub struct AppGameState {
  snake: SnakeState,
  new_food: Option<(f64, f64)>,
  playground: Rect,
  lost_flag: bool,
  playground_borders_kill: bool,
}

#[derive(Clone, PartialEq)]
pub enum GameAction {
  DoNothing,
  TurnLeft,
  TurnRight,
  TurnUp,
  TurnDown,
  Pause,
  ReturnToMenu(String, f64)
}

impl AppGameState {
  pub fn new() -> Self {
    let playground = Rect::new(10, 10, 50, 50);
    let snake = SnakeState {
      segments: vec![
        SnakeSegment{
          line: Line {
            x1: f64::from(playground.width / 2), 
            y1: f64::from(playground.height / 2), 
            x2: f64::from(playground.width / 2 + 8), 
            y2: f64::from(playground.height / 2), 
            color: Color::Green,
          },
          direction: SnakeSegmentDirection::Right,
        }
      ].into(),
      total_length: 8.0,
      consumed_food: VecDeque::<(f64, f64)>::new(),
    };
    Self {
      snake,
      new_food: None,
      playground,
      lost_flag: false,
      playground_borders_kill: false
    }
  }
  
  pub fn draw(&mut self, frame: &mut Frame) {
    // Draw game
    let area = frame.area();
    let vertical = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(self.playground.height),
        Constraint::Fill(1)
        ]);
    let horizontal = Layout::horizontal([
      Constraint::Fill(1),
      Constraint::Length(self.playground.width * 2),
      Constraint::Fill(1)
      ]);
    let [_, area, _] = vertical.areas(area);
    let [_, area, _] = horizontal.areas(area);
    frame.render_widget(self.snake_canvas(), area);
    
    // Draw lost screen
    if self.lost_flag {
        let block = Block::bordered().title("Popup");
        let area = popup_area(area, 60, 20);
        frame.render_widget(Clear, area); //this clears out the background
        frame.render_widget(block, area);
        let area = popup_area(area, 60, 20);
        let paragraph = Paragraph::new(
            vec![
              TLine::raw("Game Over").slow_blink(),
              TLine::raw(format!("Your current score is: {}", self.snake.total_length)),
              TLine::raw("Press q to return to main menu"),
            ])
            .centered()
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
    }
  }
  
  fn process_user_input(&mut self, action: GameAction) {
    let head_segment = self.snake.segments
      .front_mut()
      .expect("Snake always have 1 segment");
    let line = &mut head_segment.line;
    let x = line.x2;
    let y = line.y2;
    match action {
      GameAction::TurnLeft => {
        self.snake.segments.push_front(SnakeSegment{
          line: Line {x1: x, y1: y, x2: x, y2: y, color: Color::Green},
          direction: SnakeSegmentDirection::Left,
        });
      },
      GameAction::TurnRight => {
        self.snake.segments.push_front(SnakeSegment{
          line: Line {x1: x, y1: y, x2: x, y2: y, color: Color::Green},
          direction: SnakeSegmentDirection::Right,
        });
      },
      GameAction::TurnUp => {
        self.snake.segments.push_front(SnakeSegment{
          line: Line {x1: x, y1: y, x2: x, y2: y, color: Color::Green},
          direction: SnakeSegmentDirection::Up,
        });
      },
      GameAction::TurnDown => {
        self.snake.segments.push_front(SnakeSegment{
          line: Line {x1: x, y1: y, x2: x, y2: y, color: Color::Green},
          direction: SnakeSegmentDirection::Down,
        });
      },
      _ => ()
    }
  }
  
  fn move_snake_head(&mut self){
    let head_segment = self.snake.segments
      .front_mut()
      .expect("Snake always have 1 segment");
    let line = &mut head_segment.line;
    match head_segment.direction {
      SnakeSegmentDirection::Left => {
        line.x2 -= 0.5;
      }
      SnakeSegmentDirection::Up => {
        line.y2 += 0.5;
      }
      SnakeSegmentDirection::Right => {
        line.x2 += 0.5;
      }
      SnakeSegmentDirection::Down => {
        line.y2 -= 0.5;
      }
    }
    if !self.playground_borders_kill {
      match head_segment.direction {
        SnakeSegmentDirection::Left => {
          if line.x2 <= f64::from(self.playground.left()) {
            let x = f64::from(self.playground.right());
            let y = line.y2;
            self.snake.segments.push_front(SnakeSegment{
              line: Line {x1: x, y1: y, x2: x, y2: y, color: Color::Green},
              direction: SnakeSegmentDirection::Left,
            });
          }
        },
        SnakeSegmentDirection::Up => {
          if line.y2 >= f64::from(self.playground.bottom()) {
            let y = f64::from(self.playground.top());
            let x = line.x2;
            self.snake.segments.push_front(SnakeSegment{
              line: Line {x1: x, y1: y, x2: x, y2: y, color: Color::Green},
              direction: SnakeSegmentDirection::Up,
            });
          }
        },
        SnakeSegmentDirection::Right => {
          if line.x2 >= f64::from(self.playground.right()) {
            let x = f64::from(self.playground.left());
            let y = line.y2;
            self.snake.segments.push_front(SnakeSegment{
              line: Line {x1: x, y1: y, x2: x, y2: y, color: Color::Green},
              direction: SnakeSegmentDirection::Right,
            });
          }
        },
        SnakeSegmentDirection::Down => {
          if line.y2 <= f64::from(self.playground.top()) {
            let y = f64::from(self.playground.bottom());
            let x = line.x2;
            self.snake.segments.push_front(SnakeSegment{
              line: Line {x1: x, y1: y, x2: x, y2: y, color: Color::Green},
              direction: SnakeSegmentDirection::Down,
            });
          }
        },
      }
    }
  }
  
  fn move_snake_tail(&mut self) {
    let mut delete_last: bool = false;
    {
      let tail_segment = self.snake.segments
        .back_mut()
        .expect("Snake allways have 1 segment");
      let line = &mut tail_segment.line;
      if let Some(coords) = self.snake.consumed_food.front() {
        if relative_eq!(line.x1, coords.0) && relative_eq!(line.y1, coords.1) {
          self.snake.consumed_food.pop_front();
          return;
        }
      }
      // Mark possibly zero length segments to deletion
      if relative_eq!(line.x1, line.x2) && relative_eq!(line.y1, line.y2) {
        delete_last = true;
      }
      match tail_segment.direction {
        SnakeSegmentDirection::Left => {
          line.x1 -= 0.5;
        }
        SnakeSegmentDirection::Up => {
          line.y1 += 0.5;
        }
        SnakeSegmentDirection::Right => {
          line.x1 += 0.5;
        }
        SnakeSegmentDirection::Down => {
          line.y1 -= 0.5;
        }
      }
      // Mark valid segments after their update
      if relative_eq!(line.x1, line.x2) && relative_eq!(line.y1, line.y2) {
        delete_last = true;
      }
    }
    if delete_last {
      self.snake.segments.pop_back();
    }
  }
  
  fn check_lose_conditions(&mut self) {
    let head_segment = self.snake.segments
      .front()
      .expect("Snake always have 1 segment");
    let hl = &head_segment.line;
    
    if self.playground_borders_kill {
      if hl.x2 >= f64::from(self.playground.right()) || 
         hl.x2 <= f64::from(self.playground.left()) ||
         hl.y2 >= f64::from(self.playground.bottom()) ||
         hl.y2 <= f64::from(self.playground.top()) {
        self.lost_flag = true;
        return;
      }
    }
    for segment in self.snake.segments.iter().skip(1) {
      let sl = &segment.line;
      if (sl.x1 <= hl.x2 && hl.x2 <= sl.x2 && sl.y1 <= hl.y2 && hl.y2 <= sl.y2) ||
         (sl.x1 >= hl.x2 && hl.x2 >= sl.x1 && sl.y1 >= hl.y2 && hl.y2 >= sl.y2) {
        self.lost_flag = true;
        return;
      }
    }
  }
  
  fn generate_and_process_food(&mut self) {
    let head_segment = self.snake.segments
      .front_mut()
      .expect("Snake always have 1 segment");
    let hx = head_segment.line.x2;
    let hy = head_segment.line.y2;
    if self.new_food.is_none() {
      let random_u32 = OsRng.try_next_u32().unwrap();
      let new_x = u32::from(self.playground.left()) + random_u32 % u32::from(self.playground.width);
      let new_y = u32::from(self.playground.top()) + (random_u32 >> 16) % u32::from(self.playground.height);
      self.new_food = Some((f64::from(new_x), f64::from(new_y)));
    }
    if let Some(coords) = self.new_food {
      if relative_eq!(hx, coords.0) && relative_eq!(hy, coords.1) {
        self.snake.consumed_food.push_back(coords);
        self.snake.total_length += 1.0;
        self.new_food = None;
      }
    }
  }
  
  pub fn on_tick(&mut self, actions: &Vec<GameAction>) {
    let mut action = GameAction::DoNothing;
    if let Some(new_action) = actions.iter().rev().next() {
      action = new_action.clone();
    }
    if !self.lost_flag {
      // Generate snake food;
      // food processing must be performed before any
      // new snake segment generation
      // TODO: Add special time limited food
      self.generate_and_process_food();
      // Process action generated in main
      self.process_user_input(action);
      // Move snake head on 1 pt.
      self.move_snake_head();
      // Move snake tail on 1 pt
      self.move_snake_tail();
      // Check lose conditions
      self.check_lose_conditions();
    }
  }
  
  pub fn handle_key_press(&mut self, key: KeyEvent) -> GameAction {
    if key.kind != KeyEventKind::Press { return GameAction::DoNothing; }
    let cur_head_segment = &self.snake.segments
      .front()
      .expect("Snake always have at least 1 segment");
    match key.code {
      KeyCode::Up => {
        if cur_head_segment.direction == SnakeSegmentDirection::Up ||
          cur_head_segment.direction == SnakeSegmentDirection::Down {
          return GameAction::DoNothing;    
        }
        GameAction::TurnUp
      },
      KeyCode::Down => {
        if cur_head_segment.direction == SnakeSegmentDirection::Up ||
          cur_head_segment.direction == SnakeSegmentDirection::Down {
          return GameAction::DoNothing;
        }
        GameAction::TurnDown
      },
      KeyCode::Left => {
        if cur_head_segment.direction == SnakeSegmentDirection::Left ||
          cur_head_segment.direction == SnakeSegmentDirection::Right {
          return GameAction::DoNothing;
        }
        GameAction::TurnLeft
      },
      KeyCode::Right => {
        if cur_head_segment.direction == SnakeSegmentDirection::Left ||
          cur_head_segment.direction == SnakeSegmentDirection::Right {
          return GameAction::DoNothing;
        }
        GameAction::TurnRight
      },
      KeyCode::Char('q') => {
        if self.lost_flag {
          return GameAction::ReturnToMenu("test".to_string(), self.snake.total_length);
        }
        GameAction::DoNothing
      }
      _ => GameAction::DoNothing
    }
  }
  
  fn snake_canvas(&self) -> impl Widget + '_ {
    Canvas::default()
      .block(Block::bordered().title("Snake"))
      .marker(Marker::HalfBlock)
      .paint(|ctx| {
        for segment in self.snake.segments.iter() {
          ctx.draw(&segment.line);
        }
        let (cf_front, cf_back) = self.snake.consumed_food.as_slices();
        ctx.draw(&Points::new(cf_front, Color::Yellow));
        ctx.draw(&Points::new(cf_back, Color::Yellow));
        if let Some(coords) = self.new_food {
          ctx.draw(&Points::new(&[coords], Color::Red));
        }
      })
      .x_bounds([f64::from(self.playground.left()), f64::from(self.playground.right())])
      .y_bounds([f64::from(self.playground.top()), f64::from(self.playground.bottom())])
  }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}