use Point2;
use ggez::graphics::Rect;

pub type Id = usize;

/// Gui context
pub struct Gui {
    hot: Id,
    active: Id,

    mouse_loc: Point2,
}

impl Gui {
    fn mouse_button(&mut self, button: (), location: Point2, pressed: bool) {
        
    }

    fn mouse_motion(&mut self, location: Point2) {
        self.mouse_loc = location;
    }
}

pub fn button(gui: &mut Gui, id: Id, title: &str, rect: Rect) -> bool {
    if gui.hot == id {
        if gui.active == id {
        }
    }
    false
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_test() {
    }
}
