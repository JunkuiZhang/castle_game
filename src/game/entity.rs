use super::{graphics, system};
use super::sfml::graphics::{Shape, Transformable, Color, RenderTarget, RectangleShape};
use super::rand;
use super::super::settings::{HUMANS_HEIGHT, HUMANS_POS_Y, GROUND_HEIGHT, GROUND_POS_Y, WINDOW_WIDTH,
                            HUMANS_IDLE_WALK_SPEED_FACTOR, HUMANS_WALK_SPEED, ENEMY_WALK_SPEED,
                            HUMANS_MAX_HP, BUILDING_BASE_MAX_HP, BUILDING_OTHERS_MAX_HP};
use super::rand::Rng;
use super::ui::DrawHP;
use super::sfml::system::{Vector2f, Vector2};
use crate::game::ui::GeoInfo;
use crate::game::EnemyComing;
use std::collections::HashMap;


#[derive(Debug, Eq, PartialEq)]
pub enum HumanState {
    Idle,
    Walking,
    Running,
    Attacking,
    AttackWaiting,
}

pub enum EnemyState {
    Running,
    Attacking,
    AttackWaiting,
}


pub enum BuildingType {
    Base,
    Others,
}


struct PhysicalStates {
    velocity: f32,
    friction: f32,
}


struct EntityFightStatus {
    attack_damage: f32,
    armor: f32,
    // 是否是近战
    is_infant: Option<f32>,
    hp: f32,
}


pub trait Damageable {
    fn get_id(&self) -> u32;
    fn get_hp(&self) -> f32;
    fn fight_status(&mut self) -> &mut EntityFightStatus;
    fn set_hp(&mut self, new_hp: f32) {
        if new_hp < 0.0 { self.fight_status().hp = 0.0; }
        else if new_hp > HUMANS_MAX_HP { self.fight_status().hp = HUMANS_MAX_HP; }
        else { self.fight_status().hp = new_hp; }
    }
}


pub trait Entity<'this, T> : Damageable + GeoInfo {
    fn image(&mut self) -> &mut RectangleShape<'this>;

    fn is_player(&self) -> bool;

    fn current_state(&self) -> &T;
    fn set_current_state(&mut self, state: T);

    fn physical_states(&mut self) -> &mut PhysicalStates;

    fn state_timer(&mut self) -> &mut system::Clock;
    fn attack_timer(&mut self) -> &mut system::Clock;

    fn attack_target(&mut self) -> &mut Option<(u32, f32)>;
    fn set_attack_target(&mut self, target: Option<(u32, f32)>);

    fn rival_dir(&self) -> &Option<EnemyComing>;
    fn set_rival_dir(&mut self, dir: Option<EnemyComing>);

    fn move_(&mut self, vec: Vector2f, dt: f32) {
        self.image().move_(vec * system::Vector2f::new(dt * 60.0, dt * 60.0));
    }

    fn rival_coming_state_changer(&mut self);
    fn entity_behaviour_control(&mut self, dt: f32, rival_pos_list: &HashMap<u32, Vector2f>);
    fn velocity_update(&mut self, dt: f32) {
        if self.physical_states().velocity.abs() < 0.001 {
            self.physical_states().velocity = 0.0;
        }
        if self.physical_states().velocity > 0.0 {
            self.physical_states().velocity -= self.physical_states().velocity * self.physical_states().friction * dt;
        }
        if self.physical_states().velocity < 0.0 {
            self.physical_states().velocity -= self.physical_states().velocity * self.physical_states().friction * dt;
        }
    }

    fn position_check(&mut self) {
        let Vector2f {x, y} = self.image().position();
        if x < 0.0 {
            self.image().set_position(system::Vector2f::new(WINDOW_WIDTH as f32 + x, y));
        }
        if x > WINDOW_WIDTH as f32 && self.is_player() {
            self.image().set_position(system::Vector2f::new(x - WINDOW_WIDTH as f32, y));
        }
    }

    fn state_changer(&mut self, rival_coming: &Option<EnemyComing>) {}

    fn update(&mut self, dt: f32, pos_list: &mut HashMap<u32, Vector2f>, rival_coming: &Option<EnemyComing>,
              rival_pos_list: &HashMap<u32, Vector2f>) {

        if let Some(enemies_coming_dir) = rival_coming {
            match self.rival_dir() {
                None => {
                    self.set_rival_dir(Some((*enemies_coming_dir).clone()));
                },
                _ => {},
            }
            self.rival_coming_state_changer();
        }
        self.state_changer(rival_coming);
        self.entity_behaviour_control(dt, &rival_pos_list);
        self.velocity_update(dt);
        self.position_check();
        pos_list.insert(self.get_id(), self.get_position());
    }

    fn get_attack_target(&mut self) -> Option<(u32, f32)> {
        if self.attack_timer().elapsed_time().as_seconds() > 1.5 {
            return (*self.attack_target()).clone();
        }
        return None;
    }

    fn generate_target_to_attack(&mut self, rival_pos_list: &HashMap<u32, Vector2f>) -> Option<u32>;
}


pub struct Human<'a> {
    pub image: graphics::RectangleShape<'a>,
    id: u32,
    current_state: HumanState,
    state_timer: system::Clock,
    physical_states: PhysicalStates,
    fight_status: EntityFightStatus,
    attack_timer: system::Clock,
    attack_target: Option<(u32, f32)>,
    enemy_dir: Option<EnemyComing>,
}

impl<'a> Human<'a> {

    pub fn new(id: u32) -> Human<'a> {
        let mut rect = graphics::RectangleShape::new();
        rect.set_size(system::Vector2f::new(HUMANS_HEIGHT - 20.0, HUMANS_HEIGHT));
        rect.set_origin(system::Vector2f::new(rect.size().x / 2.0, rect.size().y));
        rect.set_fill_color(graphics::Color::GREEN);
        rect.set_position(system::Vector2f::new(30.0 + rect.size().x / 2.0, GROUND_POS_Y));
        let mut timer = system::Clock::default();
        timer.restart();
        let mut attack_timer = system::Clock::default();
        attack_timer.restart();
        Human {
            image: rect,
            current_state: HumanState::Idle,
            state_timer: timer,
            id,
            attack_timer,
            physical_states: PhysicalStates{
                velocity: 0.0,
                friction: 0.2,
            },
            fight_status: EntityFightStatus{
                attack_damage: 20.0,
                armor: 0.0,
                is_infant: None,
                hp: HUMANS_MAX_HP,
            },
            attack_target: None,
            enemy_dir: None,
        }
    }
}

impl GeoInfo for Human<'_> {
    fn get_position(&self) -> Vector2f {
        self.image.position()
    }
    fn get_size(&self) -> Vector2f {
        self.image.size()
    }
    fn is_human(&self) -> bool { true }
    fn geoinfo_get_hp(&self) -> f32 { self.get_hp() }
}

impl DrawHP for Human<'_> {
}

impl Damageable for Human<'_> {
    fn get_id(&self) -> u32 { self.id }
    fn get_hp(&self) -> f32 { self.fight_status.hp }
    fn fight_status(&mut self) -> &mut EntityFightStatus { &mut self.fight_status }
}

impl<'a> Entity<'a, HumanState> for Human<'a> {
    fn image(&mut self) -> &mut RectangleShape<'a> { &mut self.image }

    fn is_player(&self) -> bool { true }

    fn current_state(&self) -> &HumanState { &self.current_state }
    fn set_current_state(&mut self, state: HumanState) { self.current_state = state }

    fn physical_states(&mut self) -> &mut PhysicalStates { &mut self.physical_states }

    fn state_timer(&mut self) -> &mut system::Clock { &mut self.state_timer }
    fn attack_timer(&mut self) -> &mut system::Clock { &mut self.attack_timer }

    fn attack_target(&mut self) -> &mut Option<(u32, f32)> { &mut self.attack_target }
    fn set_attack_target(&mut self, target: Option<(u32, f32)>) { self.attack_target = target }

    fn rival_dir(&self) -> &Option<EnemyComing> { &self.enemy_dir }
    fn set_rival_dir(&mut self, dir: Option<EnemyComing>) { self.enemy_dir = dir }

    fn rival_coming_state_changer(&mut self) {
        match self.current_state() {
            HumanState::Idle | HumanState::Walking => {
                self.set_current_state(HumanState::Running);
            },
            _ => {},
        }
    }

    fn state_changer(&mut self, rival_coming: &Option<EnemyComing>) {
        match rival_coming {
            None => {
                if self.current_state == HumanState::AttackWaiting {
                    self.enemy_dir = None;
                    self.attack_target = None;
                    self.current_state = HumanState::Walking;
                }
            },
            _ => {},
        }
    }

    fn entity_behaviour_control(&mut self, dt: f32, rival_pos_list: &HashMap<u32, Vector2f>) {

        match self.current_state {
            HumanState::Idle => {
                if self.state_timer.elapsed_time().as_seconds() > 3.0 {
                    self.state_timer.restart();
                    self.current_state = HumanState::Walking;

                    let mut rand_gen = rand::thread_rng();
                    let unif = rand::distributions::Uniform::new(0.0, 1.0);
                    let speed = HUMANS_WALK_SPEED * HUMANS_IDLE_WALK_SPEED_FACTOR;
                    if rand_gen.sample(&unif) < 0.5 {
                        self.physical_states.velocity = speed;
                    } else {
                        self.physical_states.velocity = -speed;
                    }
                }
            },
            HumanState::Walking => {
                self.move_(system::Vector2f::new(self.physical_states.velocity, 0.0), dt);
                if self.state_timer.elapsed_time().as_seconds() > 1.5 {
                    self.state_timer.restart();
                    self.current_state = HumanState::Idle;
                }
            },
            HumanState::Running => {
                self.physical_states.velocity = HUMANS_WALK_SPEED;
                let mut velocity = Vector2f::new(self.physical_states.velocity, 0.0);
                match self.enemy_dir {
                    Some(EnemyComing::RIGHT) => {

                    },
                    Some(EnemyComing::LEFT) => {
                        velocity = Vector2f::new(- self.physical_states.velocity, 0.0);
                    },
                    None => {
                        self.current_state = HumanState::Walking;
                        self.attack_target = None;
                        self.enemy_dir = None;
                    },
                }
                self.move_(velocity, dt);
                if let Some(enemy_id) = self.generate_target_to_attack(&rival_pos_list) {
                    self.current_state = HumanState::Attacking;
                    self.attack_target = Some((enemy_id, self.fight_status.attack_damage));
                }
            },
            HumanState::Attacking => {
                if self.attack_timer.elapsed_time().as_seconds() > 1.5 {
                    if let Some((id, dmg)) = self.attack_target {
                        self.attack_target = Some((id, self.fight_status.attack_damage));
                        self.attack_timer.restart();
                    }
                } else {
                    self.current_state = HumanState::AttackWaiting;
                }
            },
            HumanState::AttackWaiting => {
                if self.attack_timer.elapsed_time().as_seconds() > 1.5 {
                    self.current_state = HumanState::Attacking;
                }
            },
        }
    }

    fn generate_target_to_attack(&mut self, rival_pos_list: &HashMap<u32, Vector2f>) -> Option<u32> {
        for (rival_id, rival_pos) in rival_pos_list.iter() {
            if self.get_position().x + self.get_size().x / 2.0 > rival_pos.x - self.get_size().x / 2.0 {
                return Some(*rival_id);
            }
        }
        return None;
    }
}


pub struct Enemy<'a> {
    pub image: graphics::RectangleShape<'a>,
    id: u32,
    current_state: EnemyState,
    state_timer: system::Clock,
    fight_status: EntityFightStatus,
    velocity: f32,
    attack_target: Option<(u32, f32)>,
    attack_timer: system::Clock,
    physical_states: PhysicalStates,
    rival_direction: Option<EnemyComing>,
    building_pos_list: Vec<Vector2f>,
}

impl<'a> Enemy<'a> {

    pub fn new(id: u32) -> Enemy<'a> {
        let mut rect = graphics::RectangleShape::new();
        rect.set_size(system::Vector2f::new(HUMANS_HEIGHT - 20.0, HUMANS_HEIGHT));
        rect.set_origin(system::Vector2f::new(rect.size().x / 2.0, rect.size().y));
        rect.set_fill_color(graphics::Color::RED);
        rect.set_position(system::Vector2f::new(1300.0, GROUND_POS_Y));
        let mut timer = system::Clock::default();
        timer.restart();
        let mut attack_timer = system::Clock::default();
        attack_timer.restart();
        Enemy {
            image: rect,
            current_state: EnemyState::Running,
            state_timer: timer,
            attack_timer,
            id,
            velocity: 0.0,
            attack_target: None,
            fight_status: EntityFightStatus{
                attack_damage: 20.0,
                armor: 0.0,
                is_infant: None,
                hp: HUMANS_MAX_HP,
            },
            physical_states: PhysicalStates{
                velocity: 0.0,
                friction: 0.2,
            },
            rival_direction: None,
            building_pos_list: Vec::new(),
        }
    }

    pub fn update_building_pos_list(&mut self, list: Vec<Vector2f>) { self.building_pos_list = list; }
}

impl GeoInfo for Enemy<'_> {
    fn get_position(&self) -> Vector2f { self.image.position() }
    fn get_size(&self) -> Vector2f { self.image.size() }
    fn is_human(&self) -> bool { true }
    fn geoinfo_get_hp(&self) -> f32 { self.get_hp() }
}

impl DrawHP for Enemy<'_> {}

impl Damageable for Enemy<'_> {
    fn get_id(&self) -> u32 { self.id }
    fn get_hp(&self) -> f32 { self.fight_status.hp }
    fn fight_status(&mut self) -> &mut EntityFightStatus { &mut self.fight_status }
}

impl<'a> Entity<'a, EnemyState> for Enemy<'a> {
    fn image(&mut self) -> &mut RectangleShape<'a> { &mut self.image }

    fn is_player(&self) -> bool { false }

    fn current_state(&self) -> &EnemyState { &self.current_state }
    fn set_current_state(&mut self, state: EnemyState) { self.current_state = state; }

    fn physical_states(&mut self) -> &mut PhysicalStates { &mut self.physical_states }

    fn state_timer(&mut self) -> &mut system::Clock { &mut self.state_timer }
    fn attack_timer(&mut self) -> &mut system::Clock { &mut self.attack_timer }

    fn attack_target(&mut self) -> &mut Option<(u32, f32)> { &mut self.attack_target }
    fn set_attack_target(&mut self, target: Option<(u32, f32)>) { self.attack_target = target; }

    fn rival_dir(&self) -> &Option<EnemyComing> { &self.rival_direction }
    fn set_rival_dir(&mut self, dir: Option<EnemyComing>) { self.rival_direction = dir; }

    fn rival_coming_state_changer(&mut self) {

    }

    fn entity_behaviour_control(&mut self, dt: f32, rival_pos_list: &HashMap<u32, Vector2f>) {
        match self.current_state {
            EnemyState::Running => {
                if self.get_position().x > WINDOW_WIDTH as f32 / 2.0 {
                    self.move_(system::Vector2f::new(-ENEMY_WALK_SPEED, 0.0), dt);
                } else {
                    self.move_(system::Vector2f::new(ENEMY_WALK_SPEED, 0.0), dt);
                }
                if let Some(attack_target_id) = self.generate_target_to_attack(rival_pos_list) {
                    self.current_state = EnemyState::Attacking;
                    self.attack_target = Some((attack_target_id, self.fight_status.attack_damage));
                }
            },
            EnemyState::Attacking => {
                if self.attack_timer.elapsed_time().as_seconds() > 1.5 {
                    if let Some((id, dmg)) = self.attack_target {
                        self.attack_target = Some((id, self.fight_status.attack_damage));
                        self.attack_timer.restart();
                    }
                } else {
                    self.current_state = EnemyState::AttackWaiting;
                }
            },
            EnemyState::AttackWaiting => {
                if self.attack_timer.elapsed_time().as_seconds() > 1.5 {
                    self.current_state = EnemyState::Attacking;
                }
            }
        }
    }

    fn generate_target_to_attack(&mut self, rival_pos_list: &HashMap<u32, Vector2f>) -> Option<u32> {
        if self.attack_timer.elapsed_time().as_seconds() > 1.5 {
            for (human_id, human_pos) in rival_pos_list.iter() {
                if self.image.position().x - self.image.size().x / 2.0 < human_pos.x + self.image.size().x / 2.0 {
                    return Some(*human_id);
                }
            }
            for (building_id, building_pos) in self.building_pos_list.iter().enumerate() {
                if self.image.position().x - self.image.size().x / 2.0 < building_pos.x + 50.0 {
                    return Some(building_id as u32 + 100);
                }
            }
        }
        return None;
    }
}


pub struct Building<'a> {
    pub image: graphics::RectangleShape<'a>,
    fight_status: EntityFightStatus,
    pub building_type: BuildingType,
    id: u32,
}

impl<'a> Building<'a> {

    pub fn new(building_type: BuildingType) -> Building<'a> {
        let mut rect = graphics::RectangleShape::new();
        rect.set_size(system::Vector2f::new(100.0, 100.0));
        rect.set_origin(system::Vector2f::new(rect.size().x / 2.0, rect.size().y));
        rect.set_fill_color(graphics::Color::BLUE);
        let mut hp: f32 = 100.0;
        match building_type {
            BuildingType::Base => {
                hp = BUILDING_BASE_MAX_HP;
                rect.set_position(system::Vector2f::new(20.0 + rect.size().x / 2.0, GROUND_POS_Y));
            },
            BuildingType::Others => {
                hp = BUILDING_OTHERS_MAX_HP;
                rect.set_position(system::Vector2f::new(20.0 + rect.size().x / 2.0, GROUND_POS_Y));
            },
        }
        Building {
            image: rect,
            fight_status: EntityFightStatus {
                attack_damage: 50.0,
                armor: 0.0,
                // 是否是近战
                is_infant: None,
                hp: 500.0,
            },
            building_type,
            id: 100,
        }
    }

    pub fn get_position(&self) -> system::Vector2f {
        self.image.position()
    }

    pub fn set_position(&mut self, pos: system::Vector2f) {
        self.image.set_position(pos);
    }

    pub fn resize(&mut self, size: system::Vector2f) {
        self.image.set_size(size);
    }
}


impl GeoInfo for Building<'_> {
    fn get_position(&self) -> Vector2f {
        self.image.position()
    }
    fn get_size(&self) -> Vector2f {
        self.image.size()
    }
    fn is_human(&self) -> bool { false }
    fn geoinfo_get_hp(&self) -> f32 { self.fight_status.hp }
}

impl DrawHP for Building<'_> {
}

impl Damageable for Building<'_> {
    fn get_id(&self) -> u32 { self.id }
    fn get_hp(&self) -> f32 { self.fight_status.hp }
    fn fight_status(&mut self) -> &mut EntityFightStatus { &mut self.fight_status }
}


pub struct BaseGround<'a> {
    pub image: graphics::RectangleShape<'a>,
}

impl<'a> BaseGround<'a> {
    pub fn new() -> BaseGround<'a> {
        let mut rect = graphics::RectangleShape::new();
        rect.set_size(system::Vector2f::new(WINDOW_WIDTH as f32, GROUND_HEIGHT));
        rect.set_fill_color(graphics::Color::MAGENTA);
        rect.set_position(system::Vector2f::new(0.0, GROUND_POS_Y));
        BaseGround {
            image: rect,
        }
    }
}
