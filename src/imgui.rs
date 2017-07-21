
pub type Id = usize;

/// Gui context
pub struct Gui {
    hot: Id,
    active: Id,
}

pub fn button(gui: &mut Gui, id: Id, title: &str) -> bool {
    false
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_test() {
    }
}
