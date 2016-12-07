extern crate rust_network_manager_library;

fn main() {
    let access_points = rust_network_manager_library::scan::scan().unwrap();
    println!("{:?}", access_points);
}