use futures::SinkExt;
use takehome::{builder::Builder, builder_deprecated::BuilderSingleThread, field::GaloisField};
use core::num;
use std::time::Instant;

pub type Fp = GaloisField::<65537>;

#[test]
fn test_basic_function() {
    let mut builder = Builder::<Fp>::new();

    let x = builder.init();
    let x_squared = builder.mul(&x, &x);
    let five = builder.constant(Fp::from(5));
    let x_squared_plus_5 = builder.add(&x_squared, &five);
    let y = builder.add(&x_squared_plus_5, &x);

    builder.fill_nodes(vec![Fp::from(5)]);

    assert_eq!(x.read().unwrap().value.unwrap().value, 5);
    assert_eq!(x_squared.read().unwrap().value.unwrap().value, 25);
    assert_eq!(five.read().unwrap().value.unwrap().value, 5);
    assert_eq!(x_squared_plus_5.read().unwrap().value.unwrap().value, 30);
    assert_eq!(y.read().unwrap().value.unwrap().value, 35);
}

#[test]
fn test_multiple_access() {
    let mut builder = Builder::<Fp>::new();

    let x = builder.init();
    let y = builder.init();
    let z = builder.init();
    let w = builder.init(); 

    let x2 = builder.mul(&x, &x);
    let xy = builder.mul(&x, &y);
    let xz = builder.mul(&x, &z);
    let xw = builder.mul(&x, &w);

    builder.fill_nodes(vec![Fp::from(5), Fp::from(5), Fp::from(45), Fp::from(6)]);
    assert_eq!(x.read().unwrap().value.unwrap().value, 5);
    assert_eq!(y.read().unwrap().value.unwrap().value, 5);
    assert_eq!(z.read().unwrap().value.unwrap().value, 45);
    assert_eq!(w.read().unwrap().value.unwrap().value, 6);

    assert_eq!(x2.read().unwrap().value.unwrap().value, 25);
    assert_eq!(xy.read().unwrap().value.unwrap().value, 25);
    assert_eq!(xz.read().unwrap().value.unwrap().value, 225);
    assert_eq!(xw.read().unwrap().value.unwrap().value, 30);
}

#[tokio::test]
async fn test_constraints() {
    let mut builder = Builder::<Fp>::new();
    let a = builder.init();
    let one = builder.constant(Fp::from(1)); 
    let eight = builder.constant(Fp::from(8));

    let b = builder.add(&a, &one); 

    let c = builder.init();
    let c_times_8 = builder.mul(&c, &eight);

    builder.fill_nodes(vec![Fp::from(13), Fp::from(2)]);
    builder.assert_equal(&c_times_8, &b);

    let constraint_check = builder.check_constraints().await; 

    println!("{:?}", constraint_check); 
    println!("{:?}", c_times_8);
    println!("{:?}", b);
}

#[tokio::test]
async fn test_large_input() {
    let n: usize = 24; 

    let num_inputs = 2_i32.pow(n as u32); 

    // 1 million inputs 
    let start_time = Instant::now();
    let mut builder = Builder::<Fp>::new();
    let inputs = builder.batch_init(num_inputs as u64);

    let constants = builder.batch_constant(&vec![Fp::from(2); num_inputs as usize]);

    let mut intermediates = Vec::with_capacity((num_inputs/2) as usize);

    for i in 0..num_inputs/4 {
        intermediates.push(builder.add(&inputs[(2*i) as usize], &inputs[(2*i + 1) as usize])); 
    }

    for i in num_inputs/4..num_inputs/2 {
        intermediates.push(builder.mul(&inputs[(2*i) as usize], &inputs[(2*i + 1) as usize])); 
    }


    for i in 0..num_inputs/8 {
        builder.add(&intermediates[(2*i) as usize], &intermediates[(2*i + 1) as usize]); 
    }
    for i in num_inputs/8..num_inputs/4 {
        builder.mul(&intermediates[(2*i) as usize], &intermediates[(2*i + 1) as usize]); 
    }

    for i in 0..num_inputs {
        builder.mul(&constants[i as usize], &inputs[i as usize]);
    }

    for i in 0..num_inputs-1 {
        builder.assert_equal(&inputs[i as usize], &inputs[(i+1) as usize]);
    }

    builder.fill_nodes(vec![Fp::from(1); num_inputs as usize]);
    let check_constraints = builder.check_constraints().await;
    let end_time = Instant::now();    

    println!("Elapsed time: {:?}", end_time - start_time);
    println!("Constraints Passed? {:?}", check_constraints);
}

#[test]
fn test_large_input_deprecated() {
    let n: usize = 24; 

    let num_inputs = 2_i32.pow(n as u32); 
    let mut inputs = Vec::with_capacity(num_inputs as usize);
    let mut constants = Vec::with_capacity(num_inputs as usize);

    // 1 million inputs 
    let start_time = Instant::now();

    let mut builder = BuilderSingleThread::new();
    for i in 0..num_inputs {
        inputs.push(builder.init());
    }

    for i in 0..num_inputs {
        constants.push(builder.constant(2));
    }

    let mut intermediates = Vec::with_capacity((num_inputs/2) as usize);

    for i in 0..num_inputs/4 {
        intermediates.push(builder.add(&inputs[(2*i) as usize], &inputs[(2*i + 1) as usize])); 
    }

    for i in num_inputs/4..num_inputs/2 {
        intermediates.push(builder.mul(&inputs[(2*i) as usize], &inputs[(2*i + 1) as usize])); 
    }


    for i in 0..num_inputs/8 {
        builder.add(&intermediates[(2*i) as usize], &intermediates[(2*i + 1) as usize]); 
    }
    for i in num_inputs/8..num_inputs/4 {
        builder.mul(&intermediates[(2*i) as usize], &intermediates[(2*i + 1) as usize]); 
    }

    for i in 0..num_inputs {
        builder.mul(&constants[i as usize], &inputs[i as usize]);
    }

    for i in 0..num_inputs-1 {
        builder.assert_equal(&inputs[i as usize], &inputs[(i+1) as usize]);
    }

    builder.fill_nodes(vec![1; num_inputs as usize]);
    let check_constraints = builder.check_constraints();
    let end_time = Instant::now();    
    println!("Elapsed time: {:?}", end_time - start_time);
    println!("Constraints Passed? {:?}", check_constraints);
}