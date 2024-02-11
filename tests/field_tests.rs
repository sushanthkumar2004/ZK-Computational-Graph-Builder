use takehome::field::{Field, GaloisField};

#[test]
fn test_field() {
    let x = GaloisField::<13>::from(54);
    let y = GaloisField::<13>::from(12); 
    let z: GaloisField<13> = x*y; 
    println!("{:?}", z); 
}
