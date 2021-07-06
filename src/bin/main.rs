use libymfm::driver::VgmPlay;

fn main() {
    println!("hello");
    VgmPlay::new(44_100, 1024, 20000);
}
