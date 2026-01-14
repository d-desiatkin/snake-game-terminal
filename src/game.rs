use std::collections::VecDeque;
use ratatui::{
  Frame,
  layout::{Constraint, Layout, Position, Rect},
  style::{Color, Stylize},
  symbols::Marker,
  widgets::{
    Block,
    canvas::{Canvas, Line, Points},
    Widget,
  },
};

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
}

pub struct FoodState<'a> {
  positions: Points<'a>
}

pub struct AppGameState {
  snake: SnakeState,
  playground: Rect,
}

impl AppGameState {
  pub fn new() -> Self {
    let snake = SnakeState {
      segments: vec![
        SnakeSegment{
          line: Line {x1: 100.0, y1: 50.0, x2: 105.0, y2: 50.0, color: Color::White},
          direction: SnakeSegmentDirection::Left,
        }
      ].into(),
      total_length: 5.0
    };
    let playground = Rect::new(10, 10, 200, 100);
    Self {snake, playground}
  }
  
  pub fn draw(&mut self, frame: &mut Frame) {
    frame.render_widget(self.snake_canvas(), frame.area());
  }
  
  fn snake_canvas(&self) -> impl Widget + '_ {
    Canvas::default()
      .block(Block::bordered().title("Snake"))
      .marker(Marker::Block)
      .paint(|ctx| {
        for segment in self.snake.segments.iter() {
          ctx.draw(&segment.line)
        }
      })
      .x_bounds([10.0, 210.0])
      .y_bounds([10.0, 110.0])
  }
}
