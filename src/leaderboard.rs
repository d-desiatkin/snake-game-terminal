use crate::LEADERBOARD;

use memmap2::MmapOptions;
use object::{File, Object, ObjectSection};
use std::env;
use std::fs::{self, OpenOptions};
use std::path::PathBuf;


use ratatui::{
  Frame,
  crossterm::event::{KeyEvent, KeyCode, KeyEventKind},
  layout::{Constraint, Layout, Flex},
  widgets::{Row, Table, TableState, Block},
  style::Style,
};

#[derive(Clone, PartialEq)]
pub enum LeaderboardAction {
  DoNothing,
  ReturnToMenu,
}

pub struct AppLeaderboardState {
  leaderboard: [([char; 16], u16); 10],
  table_state: TableState,
  current_exe: PathBuf,
  new_exe: PathBuf,
  buf: memmap2::MmapMut,
}

fn get_section(file: &File, name: &str) -> Option<(u64, u64)> {
    for section in file.sections() {
        match section.name() {
            Ok(n) if n == name => {
                return section.file_range();
            }
            _ => {}
        }
    }
    None
}

impl Drop for AppLeaderboardState {
  fn drop(&mut self) {
    let file = File::parse(&*self.buf).unwrap();

    if let Some(range) = get_section(&file, ".leaderboard") {
      assert_eq!(range.1, 680);
      let base = range.0 as usize;
      let bytes = unsafe { 
        std::slice::from_raw_parts(
          self.leaderboard.as_ptr() as *const u8, 
          680
        )
      };
      self.buf[base..(base + 680)].copy_from_slice(&(bytes));

      let perms = fs::metadata(&self.current_exe).unwrap().permissions();
      fs::set_permissions(&self.new_exe, perms).unwrap();
      fs::rename(&self.new_exe, &self.current_exe).unwrap();
    } else {
      fs::remove_file(&self.new_exe).unwrap();
      panic!("This branch should not be executed");
    }
  }
}

impl AppLeaderboardState {
  pub unsafe fn new() -> Self {
    let leaderboard = unsafe { LEADERBOARD };
    let table_state = TableState::default();
    let current_exe = env::current_exe().unwrap();
    let new_exe = current_exe.with_extension("tmp");
    fs::copy(&current_exe, &new_exe).unwrap();

    let file = OpenOptions::new().read(true).write(true).open(&new_exe).unwrap();
    let buf = unsafe { MmapOptions::new().map_mut(&file) }.unwrap();
    Self {
      leaderboard,
      table_state,
      current_exe,
      new_exe,
      buf,
    }
  }
    
  pub fn draw(&mut self, frame: &mut Frame) {
    let area = frame.area();
    let vertical = Layout::vertical([
      Constraint::Percentage(60),
    ]).flex(Flex::Center);
    let horizontal = Layout::horizontal([
      Constraint::Percentage(34),
    ]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    let mut rows = vec![];
    for lrow in self.leaderboard {
      let (name, score) = lrow;
      let mut str_name = String::new();
      for letter in name {
        str_name.push(letter);
      }
      rows.push(Row::new(vec![str_name, format!("{}", score)]));
    }
    // Columns widths are constrained in the same way as Layout...
    let widths = [
        Constraint::Max(16),
        Constraint::Max(5),
    ];
    let table = Table::new(rows, widths)
        .block(Block::bordered())
        // You can set the style of the entire Table.
        .style(Style::new().blue());
    frame.render_stateful_widget(table, area, &mut self.table_state);
  }
  
  pub fn update_board(&mut self, name: &str, score: f64) {
    let mut index = 11;
    for (i, (_ln, ls)) in self.leaderboard.iter().enumerate() {
      if score as u16 > *ls {
        index = i;
        break;
      }
    }
    if index == 11 {
      return;
    }
    let local_copy = self.leaderboard.clone();
    let (lname, lscore) = &mut self.leaderboard[index];
    *lname = [
      ' ', ' ', ' ', ' ', 
      ' ', ' ', ' ', ' ',
      ' ', ' ', ' ', ' ',
      ' ', ' ', ' ', ' '
    ];
    for (index, letter) in name.chars().enumerate() {
      lname[index] = letter;
    }
    *lscore = score as u16;
    for i in (index + 1)..10 {
      self.leaderboard[i] = local_copy[i-1];
    }
  }
  
  pub fn handle_key_press(&self, key: KeyEvent) -> LeaderboardAction {
    if key.kind != KeyEventKind::Press { return LeaderboardAction::DoNothing; }
    match key.code {
      KeyCode::Char('q') => {
        LeaderboardAction::ReturnToMenu
      }
      _ => LeaderboardAction::DoNothing
    }
  }
}