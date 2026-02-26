fn main() {
    let report = southpaw::check_or_exit("southpaw.yaml");
    println!("demo=rust status={:?}", report.status);
}
