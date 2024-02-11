use takehome::{builder::Builder, field::GaloisField};
use std::time::Instant;

pub type Fp = GaloisField::<65537>;

#[test]
fn test_function() {
    let mut builder = Builder::<Fp>::new();

    let x = builder.init();
    let x_squared = builder.mul(&x, &x);
    let five = builder.constant(Fp::from(5));
    let x_squared_plus_5 = builder.add(&x_squared, &five);
    let y = builder.add(&x_squared_plus_5, &x);

    builder.fill_nodes(vec![Fp::from(5)]);
    println!("{:?}", y);
}

#[test]
fn test_multiple_access() {
    let mut builder = Builder::<Fp>::new();

    let x = builder.init();
    let y = builder.init();
    let z = builder.init();
    let w = builder.init(); 

    let x_squared = builder.mul(&x, &x);
    let x_squared_2 = builder.mul(&x, &y);
    let x_squared_3 = builder.mul(&x, &z);
    let x_squared_4 = builder.mul(&x, &w);

    builder.fill_nodes(vec![Fp::from(5), Fp::from(5), Fp::from(45), Fp::from(6)]);
    println!("{:?}", x_squared_4);
}

#[test]
fn test_constraints() {
    let mut builder = Builder::<Fp>::new();
    let a = builder.init();
    let one = builder.constant(Fp::from(1)); 
    let eight = builder.constant(Fp::from(8));

    let b = builder.add(&a, &one); 

    let c = builder.init();
    let c_times_8 = builder.mul(&c, &eight);

    builder.fill_nodes(vec![Fp::from(15), Fp::from(2)]);
    builder.assert_equal(&c_times_8, &b);

    println!("{:?}", builder.check_constraints()); 
    println!("{:?}", c_times_8);
    println!("{:?}", b);
}

#[test]
fn test_large_input() {
    let n: usize = 25; 

    let num_inputs = 2_i32.pow(n as u32); 
    let mut inputs = Vec::with_capacity(num_inputs as usize);
    let mut builder = Builder::<Fp>::new();

    // 1 million inputs 
    for i in 0..num_inputs {
        inputs.push(builder.init());
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

    let start_time = Instant::now();
    builder.fill_nodes(vec![Fp::from(2); num_inputs as usize]);
    let end_time = Instant::now();

    let elapsed_time = end_time - start_time;
    println!("Elapsed time: {:?}", elapsed_time);
}