use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, PartialEq)]
pub enum MovementKind {
  Continuous,
  Relative,
  Absolute,
}

#[derive(Debug, Clone, Serialize)]
pub struct PtzCapabilities {
  pub profile_token: String,
  pub supported_movements: Vec<MovementKind>,
  pub supported_zoom: Vec<MovementKind>
}

impl PtzCapabilities {
  pub fn preferred_movement(&self, direction: PtzDirection) -> Option<MovementKind> {
    let group = self.direction_group(direction);
    if group.contains(&MovementKind::Relative) {
      Some(MovementKind::Relative)
    } else if group.contains(&MovementKind::Absolute) {
      Some(MovementKind::Absolute)
    } else if group.contains(&MovementKind::Continuous) {
      Some(MovementKind::Continuous)
    } else {
      None
    }
  }

  fn direction_group(&self, direction: PtzDirection) -> &Vec<MovementKind> {
    match direction {
      PtzDirection::Up | PtzDirection::Down | PtzDirection::Left | PtzDirection::Right => {
        &self.supported_movements
      }
      PtzDirection::ZoomIn | PtzDirection::ZoomOut => &self.supported_zoom
    }
  }
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub enum PtzDirection {
  Up, Down, Left, Right, ZoomIn, ZoomOut
}

impl PtzDirection {
  pub fn is_zoom(&self) -> bool {
    match self {
      PtzDirection::ZoomIn | PtzDirection::ZoomOut => true,
      _ => false
    }
  }
}
