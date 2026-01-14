use ratatui::{
    crossterm::event::{self, Event, KeyEvent, KeyCode, KeyEventKind},
    style::Style,
    widgets::{Paragraph, Row, Table, TableState, Block},
    layout::{
      Constraint::{Length, Min},
      Layout
    }, Frame,
};
use ratatui_image::{picker::Picker, StatefulImage, protocol::StatefulProtocol};

#[derive(PartialEq)]
enum MenuState {
  Game = 0,
  LeaderBoard = 1,
  Exit = 2,
  NothingSelected = 1000,
}

#[derive(PartialEq)]
pub enum MenuAction {
  SwitchToGame,
  SwitchToLeaderboard,
  SwitchToExit,
  DoNothing,
}

pub struct AppMenuState {
  pub image_state: StatefulProtocol,
  pub table_state: TableState,
}

impl AppMenuState {
  pub fn new() -> Self {
    let picker = Picker::from_query_stdio().expect("Term have image capabilities");
    let dyn_img = image::ImageReader::open("./assets/snake_v2.jpeg").expect("Image is present");
    let decoded_dyn_img = dyn_img.decode().expect("Image is decodable");
    let app_image = picker.new_resize_protocol(decoded_dyn_img);
    let table_state = TableState::default();
    Self { image_state: app_image, table_state }
  }
  
  fn next_row(&mut self) {
    self.table_state.select_next();
  }
  
  fn previous_row(&mut self) {
    self.table_state.select_previous();
  }
  
  fn get_current_state(& self) -> MenuState {
    let index = self.table_state.selected();
    match index {
      Some(0) => MenuState::Game,
      Some(1) => MenuState::LeaderBoard,
      Some(2) => MenuState::Exit,
      None => MenuState::NothingSelected,
      _ => panic!("Impossible menu state")
    }
  }
  
  pub fn draw(&mut self, frame: &mut Frame) {
    // Prepare general menu layout
    let vertical = Layout::horizontal([
      Length(64),
      Length(25),
      Min(0)
    ]);
    let [image_area, menu_area, _] = vertical.areas(frame.area());
    
    // Draw image
    let image = StatefulImage::default();
    let centered_image_area = image_area.centered(Length(64), Min(0));
    frame.render_stateful_widget(image, centered_image_area, &mut self.image_state);
    
    // Draw Table
    let centered_menu_area = menu_area.centered(Length(25), Min(0));
    let rows = [
      Row::new(vec!["New Game"]),
      Row::new(vec!["Leader Board"]),
      Row::new(vec!["Exit"])
    ];
    // Columns widths are constrained in the same way as Layout...
    let widths = [
        Length(25)
    ];
    let table = Table::new(rows, widths)
        // You can set the style of the entire Table.
        .style(Style::new().blue())
        // It has an optional footer, which is simply a Row always visible at the bottom.
        .footer(Row::new(vec!["Updated on Dec 28"]))
        // As any other widget, a Table can be wrapped in a Block.
        .block(Block::new().title("Main Menu"))
        // The selected row, column, cell and its content can also be styled.
        .row_highlight_style(Style::new().reversed())
        .column_highlight_style(Style::new().red())
        .cell_highlight_style(Style::new().blue())
        // ...and potentially show a symbol in front of the selection.
        .highlight_symbol(">>");    
    frame.render_stateful_widget(table, centered_menu_area, &mut self.table_state);
  }
  
  pub fn handle_key_press(&mut self, key: KeyEvent) -> MenuAction {
    if key.kind != KeyEventKind::Press { return MenuAction::DoNothing; }
    match key.code {
      KeyCode::Up => {
        self.previous_row();
        return MenuAction::DoNothing;
      },
      KeyCode::Down => {
        self.next_row();
        return MenuAction::DoNothing;
      },
      KeyCode::Char(' ') => {
        let menu_state = self.get_current_state();
        match menu_state {
          MenuState::Exit => return MenuAction::SwitchToExit,
          MenuState::Game => return MenuAction::SwitchToGame,
          MenuState::LeaderBoard => return MenuAction::SwitchToLeaderboard,
          MenuState::NothingSelected => return MenuAction::DoNothing,
        }
      },
      _ => MenuAction::DoNothing,
    }
  }
}




