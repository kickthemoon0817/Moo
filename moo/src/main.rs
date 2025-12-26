use moo::investigation::viz::window;

fn main() {
    pollster::block_on(window::run());
}
