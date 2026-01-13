use image::ImageReader;
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
use ratatui_image::{picker::Picker, StatefulImage, protocol::StatefulProtocol};

pub struct AppMenuState {
  pub image: StatefulProtocol,
}

impl AppMenuState {
  pub fn new() -> Self {
    let mut picker = Picker::halfblocks();
    let dyn_img = image::ImageReader::open("./assets/snake.jpeg").expect("Image is present");
    let decoded_dyn_img = dyn_img.decode().expect("Image is decodable");
    let app_image = picker.new_resize_protocol(decoded_dyn_img);
    Self { image: app_image }
  }
  
  pub fn draw(&mut self, frame: &mut Frame) {
    let vertical = Layout::vertical([
      Length(64),
      Length(4),
      Min(0)
    ]);
    let [image_area, menu_area, _] = vertical.areas(frame.area());
    let image = StatefulImage::default();
    let centered_image_area = image_area.centered(Length(64), Min(0));
    frame.render_stateful_widget(image, centered_image_area, &mut self.image);
    let greeting = Paragraph::new("Hello World! (press 'q' to quit)");
    frame.render_widget(greeting, menu_area);
  }
}




