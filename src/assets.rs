


enum AppAsset {
  MainMenuImage,
  Leaderboard,
}

struct Asset<'static> {
  bytes: &'static[u8]
}

impl Asset<'static> {
  fn new(bytes: &'static[u8]) -> Self {
    Self { bytes }
  }
}

fn get_resource(asset: AppAssets) -> Asset {
  match asset {
    AppAsset::MainMenuImage {
      
    },
    AppAsset::Leaderboard {
      
    }
  }
}