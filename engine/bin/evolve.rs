use engine::Engine;

fn main() {
    let engine = Engine::new_random();
    engine.save("engine.rew");
}
