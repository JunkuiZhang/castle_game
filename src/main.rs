mod settings;
mod game;

fn main() {
    let mut main_game = game::Game::new(settings::WINDOW_WIDTH,
                                        settings::WINDOW_HEIGHT, settings::TITLE);
    main_game.run();
}
