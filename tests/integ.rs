use takehome::builder::*;
use std::time::Instant;

#[test]
fn test_basic_function() {
    // Example 1: f(x) = x^2 + x + 5
  
    // instantiates an empty circuit with no nodes
    env_logger::init();
    let mut builder = Builder::new();

    // NOTE: nodes here are cloned since Rust consumes
    // the provided arguments to these functions. 

    // create an input node to the circuit
    // value of the input node must be specified later
    // in order for the fill_nodes() method to properly
    // compute values. 
    let x = builder.init();

    // create a multiplication gate using the mul method
    // that creates a new node in the circuit containing
    // the product of the two input nodes (which in this
    // case is x). 
    let x_squared = builder.mul(x.clone(), x.clone());

    // create a constant value in the circuit that can be
    // used in later computations 
    let five = builder.constant(5);

    // create two addition gates 
    let x_squared_plus_5 = builder.add(x_squared.clone(), five.clone());
    let y = builder.add(x_squared_plus_5.clone(), x.clone());

    // set the value of the input nodes
    // Note that if not all input nodes are filled out
    // the builder will try to fill out nodes whose values
    // can be derived and fail when it first encounters a node
    // whose value depends on an unset input. 
    builder.set(x.clone(), 5);

    // fill out the graph based on the input nodes 
    builder.fill_nodes();

    assert_eq!(x.read(), 5);
    assert_eq!(x_squared.read(), 25);
    assert_eq!(five.read(), 5);
    assert_eq!(x_squared_plus_5.read(), 30);
    assert_eq!(y.read(), 35);
}

#[test]
fn test_multiple_access() {
    let mut builder = Builder::new();

    let x = builder.init();
    let y = builder.init();
    let z = builder.init();
    let w = builder.init(); 

    let x2 = builder.mul(x.clone(), x.clone());
    let xy = builder.mul(x.clone(), y.clone());
    let xz = builder.mul(x.clone(), z.clone());
    let xw = builder.mul(x.clone(), w.clone());

    builder.set(x.clone(), 5);
    builder.set(y.clone(), 5);
    builder.set(z.clone(), 45);
    builder.set(w.clone(), 6);

    builder.fill_nodes();
    assert_eq!(x.read(), 5);
    assert_eq!(y.read(), 5);
    assert_eq!(z.read(), 45);
    assert_eq!(w.read(), 6);

    assert_eq!(x2.read(), 25);
    assert_eq!(xy.read(), 25);
    assert_eq!(xz.read(), 225);
    assert_eq!(xw.read(), 30);
}

#[tokio::test]
async fn test_constraints() {
    let mut builder = Builder::new();
    let a = builder.init();
    let one = builder.constant(1); 
    let eight = builder.constant(8);

    let b = builder.add(a.clone(), one.clone()); 

    let c = builder.init();
    let c_times_8 = builder.mul(c.clone(), eight.clone());

    builder.set(a.clone(), 13);
    builder.set(c.clone(), 2);

    builder.fill_nodes();
    builder.assert_equal(c_times_8.clone(), b.clone());

    let constraints_check = builder.check_constraints().await;

    assert!(!constraints_check);
    assert_eq!(a.read(), 13);
    assert_eq!(one.read(), 1);
    assert_eq!(eight.read(), 8);
    assert_eq!(b.read(), 14);
    assert_eq!(c.read(), 2);
    assert_eq!(c_times_8.read(), 16);
}

#[tokio::test]
async fn test_hints() {
    let mut builder = Builder::new();
    let a = builder.init();
    let one = builder.constant(1); 
    let eight = builder.constant(8);

    let b = builder.add(a.clone(), one.clone()); 

    fn lambda_div8(val: Vec<u32>) -> u32 {
        assert_eq!(val.len(), 1);
        val[0] / 8
    }    

    let c = builder.hint(&[b.clone()], lambda_div8);
    let c_times_8 = builder.mul(c.clone(), eight.clone());

    builder.set(a.clone(), 15);
    builder.fill_nodes();
    builder.assert_equal(c_times_8.clone(), b.clone());

    let constraints_check = builder.check_constraints().await;

    assert!(constraints_check);
    assert_eq!(a.read(), 15);
    assert_eq!(one.read(), 1);
    assert_eq!(eight.read(), 8);
    assert_eq!(b.read(), 16);
    assert_eq!(c.read(), 2);
    assert_eq!(c_times_8.read(), 16);
}

#[tokio::test]
async fn test_sqrt_hints() {
    // Example 3: f(x) = sqrt(x+7)
    //
    // Assume that x+7 is a perfect square (so x = 2 or 9, etc.).

    let mut builder = Builder::new();
    let x = builder.init();
    let seven = builder.constant(7);
    let x_plus_seven = builder.add(x.clone(), seven.clone());

    // Function to use for hint 
    fn lambda_sqrt(val: Vec<u32>) -> u32 {
        ((val[0] as f64).sqrt().round()) as u32
    }

    // API for hints.
    // The first argument is a slice containing a
    // vector of nodes to be used as an input
    // to the second argument, which is a
    // user-provided vector function.
    // The general syntax is builder.hint(&[nodes], function)

    // For example, this computes the square root of x+7
    // by passing in the node x_plus_seven as an argument to
    // lambda_sqrt. 
    let sqrt_x_plus_7 = builder.hint(&[x_plus_seven.clone()], lambda_sqrt);
    let computed_sq = builder.mul(sqrt_x_plus_7.clone(), sqrt_x_plus_7.clone());

    builder.assert_equal(computed_sq.clone(), x_plus_seven.clone());
    builder.set(x.clone(), 2);
    builder.fill_nodes();

    assert_eq!(x.read(), 2);
    assert_eq!(seven.read(), 7);
    assert_eq!(sqrt_x_plus_7.read(), 3);
    assert_eq!(computed_sq.read(), 9);
    assert_eq!(x_plus_seven.read(), 9);
    assert!(builder.check_constraints().await);
}

#[tokio::test]
async fn test_lambda_gates() {  
    let mut builder = Builder::new();

    let a = builder.init();
    let b = builder.init();

    let c = builder.mul(a.clone(), b.clone());

    fn lambda_div(params: Vec<u32>) -> u32 {
        params[0] / params[1]
    }

    let d = builder.hint(&[c.clone(), b.clone()], lambda_div);

    builder.assert_equal(d.clone(), a.clone());

    builder.set(a.clone(), 234); 
    builder.set(b.clone(), 123);

    builder.fill_nodes();
    let passed_constraints = builder.check_constraints().await; 

    assert!(passed_constraints);
    assert_eq!(a.read(), 234);
    assert_eq!(b.read(), 123);
    assert_eq!(c.read(), 28782);
    assert_eq!(d.read(), 234);
}

#[tokio::test]
async fn test_large_input_builder() {
    let n: usize = 20; 

    let num_inputs = 2_usize.pow(n as u32); 

    // 1 million inputs 
    let start_time = Instant::now();
    let mut builder = Builder::new();

    let time_to_batch_init = Instant::now();
    let inputs = builder.batch_init(num_inputs);
    println!("Time to batch init: {:?}", Instant::now() - time_to_batch_init);

    let time_to_batch_const = Instant::now();
    let constants = builder.batch_constant(&vec![2; num_inputs]);
    println!("Time to batch const: {:?}", Instant::now() - time_to_batch_const);

    // to catch the intermediate additions and multplications. 
    let mut intermediates = Vec::with_capacity(num_inputs/2);

    for i in 0..num_inputs/4 {
        intermediates.push(builder.add(inputs[2*i].clone(), inputs[2*i + 1].clone())); 
    }

    for i in 0..num_inputs/4 {
        intermediates.push(builder.mul(inputs[0].clone(), inputs[i].clone())); 
    }

    for i in 0..num_inputs/8 {
        builder.add(intermediates[2*i].clone(), intermediates[2*i + 1].clone()); 
    }
    for i in num_inputs/8..num_inputs/4 {
        builder.mul(intermediates[2*i].clone(), intermediates[2*i + 1].clone()); 
    }

    for i in 0..num_inputs {
        builder.mul(constants[i].clone(), inputs[i].clone());
    }
   
    builder.batch_assert_equal(&inputs, &inputs);
    builder.batch_set(&inputs, &vec![100; num_inputs]);

    let time_to_fill_nodes = Instant::now();
    builder.fill_nodes();
    println!("Time to fill nodes: {:?}", Instant::now() - time_to_fill_nodes);

    let check_constraints = builder.check_constraints().await;
    let end_time = Instant::now();    

    println!("Elapsed time: {:?}", end_time - start_time);
    println!("Constraints Passed? {:?}", check_constraints);
}