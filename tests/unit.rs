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

        assert_eq!(z.read(), (x_val as u32 * y_val as u32)); 
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

        assert_eq!(z.read(), (x_val as u32 + y_val as u32)); 
    }    
}

#[test]
fn test_builder_set() {
    let mut builder = Builder::new();

    let x = builder.init();
    let y = builder.constant(10);

    let z = builder.add(x.clone(), y.clone());

    // should fail since z is a derived node 
    builder.set(z.clone(), 1); 
    assert!(z.value.read().is_none());

    // should fail since y is constant node 
    builder.set(y.clone(), 2); 
    assert_eq!(y.read(), 10); 

    // should succeed since x is input node 
    builder.set(x.clone(), 3); 
    assert_eq!(x.read(), 3); 
}