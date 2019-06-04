/// Represents a controller
#[derive(Default)]
pub struct Controller {
    /// A bitfield of the buttons, in the following order:
    /// A, B, Select, Start, Up, Down, Left, Right
    buttons: [bool; 8],
    /// What button is currently being read
    index: u8,
    strobe: bool,
}

impl Controller {
    pub fn new() -> Self {
        Controller::default()
    }

    pub fn set_buttons(&mut self, buttons: [bool; 8]) {
        self.buttons = buttons;
    }

    pub fn read(&mut self) -> u8 {
        let index = self.index as usize;
        let res = if *self.buttons.get(index).unwrap_or(&false) {
            1
        } else {
            0
        };
        self.index += 1;
        if self.strobe {
            self.index = 0;
        }
        res
    }

    pub fn write(&mut self, value: u8) {
        self.strobe = value & 1 == 1;
        if self.strobe {
            self.index = 0;
        }
    }
}
