use super::graphics::*;
use super::system::Vector2f;
use super::super::settings::{HUMANS_MAX_HP, BUILDING_BASE_MAX_HP};


struct UIString;


struct UICardsInfo;


pub trait GeoInfo {
    fn get_position(&self) -> Vector2f;
    fn get_size(&self) -> Vector2f;
    fn is_human(&self) -> bool;
    fn geoinfo_get_hp(&self) -> f32;
}


pub trait DrawHP: GeoInfo {
    fn draw_hp(&self, win: &mut RenderWindow) {
        let mut hp_rect = RectangleShape::new();
        let mut hp_bound = RectangleShape::new();
        const HUMAN_HP_BAR_WIDTH: f32 = 60.0;
        const BUILDING_HP_BAR_WIDTH: f32 = 100.0;
        if self.is_human() {
            hp_rect.set_size(Vector2f::new(HUMAN_HP_BAR_WIDTH * self.geoinfo_get_hp() / HUMANS_MAX_HP, 10.0));
            hp_bound.set_size(Vector2f::new(HUMAN_HP_BAR_WIDTH, 10.0));
            hp_rect.set_position(Vector2f::new(self.get_position().x - HUMAN_HP_BAR_WIDTH / 2.0, self.get_position().y - self.get_size().y - 25.0));
            hp_bound.set_position(Vector2f::new(self.get_position().x - HUMAN_HP_BAR_WIDTH / 2.0, self.get_position().y - self.get_size().y - 25.0));
        } else {
            hp_rect.set_size(Vector2f::new(BUILDING_HP_BAR_WIDTH * self.geoinfo_get_hp() / BUILDING_BASE_MAX_HP, 10.0));
            hp_bound.set_size(Vector2f::new(BUILDING_HP_BAR_WIDTH, 10.0));
            hp_rect.set_position(Vector2f::new(self.get_position().x - BUILDING_HP_BAR_WIDTH / 2.0, self.get_position().y - self.get_size().y - 25.0));
            hp_bound.set_position(Vector2f::new(self.get_position().x - BUILDING_HP_BAR_WIDTH / 2.0, self.get_position().y - self.get_size().y - 25.0));
        }
        hp_bound.set_outline_thickness(2.0);
        hp_bound.set_outline_color(Color::WHITE);
        hp_bound.set_fill_color(Color::TRANSPARENT);
        hp_rect.set_fill_color(Color::GREEN);
        win.draw(&hp_rect);
        win.draw(&hp_bound);
    }
}
