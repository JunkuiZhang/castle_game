extern crate sfml;
extern crate rand;
mod state_machine;
// mod texture_manager;
mod entity;
mod ui;
mod cards;

use entity::{Damageable, Entity, HumanState, EnemyState};
use sfml::{graphics, window, system};
use std::option::Option::Some;
use self::sfml::graphics::{RenderTarget, Transformable, Color};
use crate::settings::{WINDOW_WIDTH, GROUND_POS_Y, WINDOW_HEIGHT};
use crate::game::entity::{Building, BuildingType};
use crate::game::ui::{DrawHP, GeoInfo};
use self::sfml::system::Vector2f;
use std::collections::{HashSet, HashMap};
use std::intrinsics::transmute;
use std::ops::Index;


#[derive(Clone, Debug)]
enum EnemyComing {
    RIGHT,
    LEFT,
}


pub struct Game {
    win: graphics::RenderWindow,
    humans: Vec<entity::Human<'static>>,
    humans_pos_list: HashMap<u32, Vector2f>,
    enemies: Vec<entity::Enemy<'static>>,
    buildings: Vec<entity::Building<'static>>,
    // buildings_pos_list: HashMap<u32, Vector2f>,
    buildings_pos_list: Vec<Vector2f>,
    base_ground: Vec<entity::BaseGround<'static>>,
    enemies_pos_list: HashMap<u32, Vector2f>,
    clock: system::Clock,
    is_paused: bool,
    is_game_over: bool,
    attacked_human_list: Vec<Box<AttackInfo>>,
    attacked_human_ids: HashSet<u32>,
    attacked_enemy_list: Vec<Box<AttackInfo>>,
    attacked_enemy_ids: HashSet<u32>,
    enemy_coming: Option<EnemyComing>,
}

impl Game {

    pub fn new(width: u32, height: u32, title: &str) -> Game {
        let win = graphics::RenderWindow::new(window::VideoMode::new(width, height,
                                                                    window::VideoMode::desktop_mode().bits_per_pixel),
                                            title, window::Style::default(), &window::ContextSettings::default());

        let mut humans_pos_list = HashMap::new();
        let mut enemies_pos_list = HashMap::new();
        let humans = vec![entity::Human::new(0)];
        let enemies = vec![entity::Enemy::new(0)];
        for human in humans.iter() {
            humans_pos_list.insert(human.get_id(), human.get_position());
        }
        for enemy in enemies.iter() {
            enemies_pos_list.insert(enemy.get_id(), enemy.get_position());
        }

        let mut buildings_pos_list = Vec::new();
        let mut buildings = vec![entity::Building::new(BuildingType::Base)];
        for building in buildings.iter() {
            buildings_pos_list.push(building.get_position());
        }

        let base_ground = vec![entity::BaseGround::new()];
        Game {
            win,
            humans,
            humans_pos_list,
            enemies,
            buildings,
            buildings_pos_list,
            base_ground,
            enemies_pos_list,
            clock: system::Clock::default(),
            is_paused: false,
            is_game_over: false,
            attacked_human_list: Vec::new(),
            attacked_human_ids: HashSet::new(),
            attacked_enemy_list: Vec::new(),
            attacked_enemy_ids: HashSet::new(),
            enemy_coming: Some(EnemyComing::RIGHT),
        }
    }

    fn events(&mut self) {
        while let Some(event) = self.win.poll_event() {
            match event {
                window::Event::Closed => self.win.close(),
                window::Event::KeyPressed {code: window::Key::Escape, ..} => self.win.close(),
                window::Event::KeyPressed {code: window::Key::F9, ..} => self.is_paused = !self.is_paused,
                _ => {},
            }
        }
    }

    fn update_and_draw(&mut self, dt: f32) {
        self.win.clear(graphics::Color::BLACK);

        let mut player_team_remove_index = Vec::new();
        let mut enemy_team_remove_index = Vec::new();

        // Drawing
        for base_ground in &self.base_ground {
            self.win.draw(&base_ground.image);
        }
        for building in &mut self.buildings {
            if self.attacked_human_ids.contains(&building.get_id()) {
                let mut drop_index = Vec::new();
                for (index, item_box) in self.attacked_human_list.iter().enumerate() {
                    let id = item_box.attacked_id;
                    let dmg = item_box.dmg_taken;
                    if building.get_id() != id {
                        continue;
                    } else {
                        building.set_hp(building.get_hp() - dmg);
                        drop_index.push(index);
                    }
                }

                self.attacked_human_ids.remove(&building.get_id());
                while drop_index.len() > 0 {
                    self.attacked_human_list.remove(drop_index[0]);
                    drop_index.remove(0);
                    drop_index.iter().map(|x| x - 1);
                }
            }
            self.win.draw(&building.image);
            building.draw_hp(&mut self.win);
        }

        self.attacked_human_list.clear();
        self.attacked_human_ids.clear();

        for enemy in &mut self.enemies {
            if enemy.get_hp() <= 0.0 {
                enemy_team_remove_index.push(enemy.get_id());
                continue;
            }
            if self.attacked_enemy_ids.contains(&enemy.get_id()) {
                let mut drop_index = Vec::new();
                for (index, item_box) in self.attacked_enemy_list.iter().enumerate() {
                    let id = item_box.attacked_id;
                    let dmg = item_box.dmg_taken;
                    if enemy.get_id() != id {
                        continue;
                    } else {
                        enemy.set_hp(enemy.get_hp() - dmg);
                        drop_index.push(index);
                    }
                }

                self.attacked_enemy_ids.remove(&enemy.get_id());
                while drop_index.len() > 0 {
                    self.attacked_enemy_list.remove(drop_index[0]);
                    drop_index.remove(0);
                    drop_index.iter_mut().map(|x| *x - 1);
                }
            }

            enemy.update_building_pos_list(self.buildings_pos_list.clone());
            enemy.update(dt, &mut self.enemies_pos_list, &Some(EnemyComing::LEFT), &self.humans_pos_list);
            if let Some((attacked_human_id, dmg)) = enemy.get_attack_target() {
                self.attacked_human_ids.insert(attacked_human_id);
                self.attacked_human_list.push(Box::new(AttackInfo {attacked_id: attacked_human_id,
                                                                dmg_taken: dmg}));
            }
            self.win.draw(&enemy.image);
            enemy.draw_hp(&mut self.win);
        }

        for human in &mut self.humans {
            if human.get_hp() <= 0.0 {
                player_team_remove_index.push(human.get_id());
                continue;
            }
            if self.attacked_human_ids.contains(&human.get_id()) {
                let mut drop_index = Vec::new();
                for (index, item_box) in self.attacked_human_list.iter().enumerate() {
                    let id = item_box.attacked_id;
                    let dmg = item_box.dmg_taken;
                    if human.get_id() != id {
                        continue;
                    } else {
                        human.set_hp(human.get_hp() - dmg);
                        drop_index.push(index);
                    }
                }

                self.attacked_human_ids.remove(&human.get_id());
                while drop_index.len() > 0 {
                    self.attacked_human_list.remove(drop_index[0]);
                    drop_index.remove(0);
                    drop_index.iter_mut().map(|x| *x - 1);
                }
            }

            human.update(dt, &mut self.humans_pos_list, &self.enemy_coming, &self.enemies_pos_list);
            if let Some((attacked_enemy_id, dmg)) = human.get_attack_target() {
                self.attacked_enemy_ids.insert(attacked_enemy_id);
                self.attacked_enemy_list.push(Box::new(AttackInfo {attacked_id: attacked_enemy_id,
                    dmg_taken: dmg}));
            }
            human.draw_hp(&mut self.win);
            self.win.draw(&human.image);
        }

        let mut humans_drop_index = Vec::new();
        for player_id in player_team_remove_index.iter() {
            let mut temp_list = self.humans_pos_list.clone();
            for (id, _) in self.humans_pos_list.iter() {
                if *id != *player_id { continue; }
                temp_list.remove(id);
            }
            self.humans_pos_list = temp_list;
            for (index, item) in self.humans.iter().enumerate() {
                if *player_id == item.get_id() {
                    humans_drop_index.push(index);
                }
            }
        }
        while humans_drop_index.len() > 0 {
            self.humans.remove(humans_drop_index[0]);
            humans_drop_index.remove(0);
            humans_drop_index.iter_mut().map(|x| *x - 1);
        }

        let mut enemies_drop_index = Vec::new();
        for enemy_id in enemy_team_remove_index.iter() {
            let mut temp_list = self.enemies_pos_list.clone();
            for (id, _) in self.enemies_pos_list.iter() {
                if *id != *enemy_id { continue; }
                temp_list.remove(id);
            }
            self.enemies_pos_list = temp_list;
            for (index, item) in self.enemies.iter().enumerate() {
                if *enemy_id == item.get_id() {
                    enemies_drop_index.push(index);
                }
            }
        }
        while enemies_drop_index.len() > 0 {
            self.enemies.remove(enemies_drop_index[0]);
            enemies_drop_index.remove(0);
            enemies_drop_index.iter_mut().map(|x| *x - 1);
        }

        self.game_over_update();

        self.win.display();
    }

    pub fn run(&mut self) {
        while self.win.is_open() {
            self.events();
            let dt = self.clock.restart().as_seconds();
            if !self.is_paused && !self.is_game_over {
                self.update_and_draw(dt);
            }
            if self.is_game_over {
                self.show_game_over_screen();
            }
        }
    }

    fn game_over_update(&mut self) {
        if self.buildings[0].get_hp() <= 0.0 {
            self.is_game_over = true;
        }
    }

    fn show_game_over_screen(&mut self) {
        let mut font = graphics::Font::from_file("src/res/fonts/SourceCodePro.ttf")
            .expect("Error loading fonts");
        let mut text = graphics::Text::new("GAME OVER!", &font, 20);
        text.set_fill_color(Color::WHITE);
        text.set_origin(Vector2f::new(text.global_bounds().width / 2.0, text.global_bounds().height / 2.0));
        text.set_position(Vector2f::new(WINDOW_WIDTH as f32 / 2.0, WINDOW_HEIGHT as f32 / 2.0));
        self.win.draw(&text);
    }
}


struct AttackInfo {
    attacked_id: u32,
    dmg_taken: f32,
}
