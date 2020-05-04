pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

pub struct Display {
    screen: [u8; SCREEN_WIDTH * SCREEN_HEIGHT],
}

impl Display {
    pub fn new() -> Self {
        Display {
            screen: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> u8 {
        self.screen[y * SCREEN_WIDTH + x]
    }

    fn set_pixel(&mut self, x: usize, y: usize, value: u8) {
        self.screen[y * SCREEN_WIDTH + x] = value;
    }

    pub fn clear(&mut self) {
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                self.set_pixel(x, y, 0);
            }
        }
    }

    pub fn draw_sprite(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool {
        let mut collision = false;
        for (i, &row) in sprite.iter().enumerate() {
            for j in 0..8 {
                let new_x = (x + j) % SCREEN_WIDTH;
                let new_y = (y + i) % SCREEN_HEIGHT;
                let new_value = row >> (7 - j) & 1;
                let old_value = self.get_pixel(new_x, new_y);
                self.set_pixel(new_x, new_y, new_value ^ old_value);

                if old_value == 1 && self.get_pixel(new_x, new_y) == 0 {
                    collision = true;
                }
            }
        }

        collision
    }
}