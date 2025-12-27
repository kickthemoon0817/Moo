mod renderer;
mod window;

fn main() {
    pollster::block_on(window::run());
}
