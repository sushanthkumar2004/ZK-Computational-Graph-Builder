use takehome::builder::*;
use rand::{rngs::StdRng, SeedableRng, Rng};

#[test]
fn test_multiplication_gate() {
    env_logger::init();
    let mut builder = Builder::new();

    let seed = [0u8; 32];
    let mut rng = StdRng::from_seed(seed);

    let x = builder.init();
    let y = builder.init();

    let z = builder.mul(x.clone(), y.clone());

    for _ in 0..10 {
        let x_val: u16 = rng.gen(); 
        let y_val: u16 = rng.gen();

        builder.set(x.clone(), x_val.into()); 
        builder.set(y.clone(), y_val.into()); 

        builder.fill_nodes();

        assert_eq!(z.get(), (x_val as u32 * y_val as u32)); 
    }    
}

#[test]
fn test_addition_gate() {
    let mut builder = Builder::new();

    let seed = [0u8; 32];
    let mut rng = StdRng::from_seed(seed);

    let x = builder.init();
    let y = builder.init();

    let z = builder.add(x.clone(), y.clone());

    for _ in 0..10 {
        let x_val: u16 = rng.gen(); 
        let y_val: u16 = rng.gen();

        builder.set(x.clone(), x_val.into()); 
        builder.set(y.clone(), y_val.into()); 

        builder.fill_nodes();

        assert_eq!(z.get(), (x_val as u32 + y_val as u32)); 
    }    
}

#[test]
fn test_builder_set() {
    let mut builder = Builder::new();

    let x = builder.init();
    let y = builder.constant(10);

    let _ = builder.add(x.clone(), y.clone());

    // should fail safely since y is constant node 
    builder.set(y.clone(), 2); 
    assert_eq!(y.get(), 10); 

    // should succeed since x is input node 
    builder.set(x.clone(), 3); 
    assert_eq!(x.get(), 3); 
}

#[test]
#[should_panic]
fn test_builder_invalid_set() {
    let mut builder = Builder::new();

    let x = builder.init();
    let y = builder.constant(10);

    let z = builder.add(x.clone(), y.clone());

    // trying to set an internal node and accessing it should error
    // since the value has not been computed yet
    builder.set(z.clone(), 20); 
    z.get();
}