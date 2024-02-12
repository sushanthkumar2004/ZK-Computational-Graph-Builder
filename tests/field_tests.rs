use takehome::field::{extended_euclidean, reciprocal, GaloisField};

#[test]
fn test_field() {
    let x = GaloisField::<13>::from(54);
    let y = GaloisField::<13>::from(12); 
    let z  = x*y; 

    let w = z / x;
    let u = z / y;
    
    println!("{:?}", z); 
    println!("{:?}", w); 
    println!("{:?}", u); 
}

#[test]
fn test_euclidean() {
    let a: i128 = 5;
    let b: i128 = 13;
    println!("{:?}", extended_euclidean(a, b));

    let a: i128 = 3255;
    let b: i128 = 218;
    println!("{:?}", extended_euclidean(a, b));
}

#[test]
fn test_reciprocal() {
    let a: u64 = 5;
    let b: u64 = 29;

    println!("{:?}", reciprocal(a, b));
}