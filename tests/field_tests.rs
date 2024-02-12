use takehome::field::GaloisField;

#[test]
fn test_field() {
    let x = GaloisField::<13>::from(54);
    let y = GaloisField::<13>::from(12); 
    let z  = x*y; 
    println!("{:?}", z); 
}
