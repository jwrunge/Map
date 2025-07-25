#[cfg(feature = "windowing")]
use map;

#[cfg(feature = "windowing")]
fn main() {
    map::run();
}

#[cfg(not(feature = "windowing"))]
fn main() {
    println!("Windowing feature disabled. This is a library build.");
    println!("Use HeadlessRenderer for embedded rendering.");
}
