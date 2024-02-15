use takehome::{field::GaloisField, graph_builder::*, builder::Builder, builder_single_thread::BuilderSingleThread};
use std::time::Instant;

pub type Fp = GaloisField::<65537>;

#[tokio::test]
async fn test_large_input_graphbuilder() {
    let n: usize = 24; 

    let num_inputs = 2_usize.pow(n as u32); 

    // 1 million inputs 
    let start_time = Instant::now();
    let mut builder = GraphBuilder::<Fp>::new();
    let inputs = builder.batch_init(num_inputs);

    let constants = builder.batch_constant(&vec![Fp::from(2); num_inputs]);

    let mut intermediates = Vec::with_capacity(num_inputs/2);

    for i in 0..num_inputs/4 {
        intermediates.push(builder.add(&inputs[2*i], &inputs[2*i + 1])); 
    }

    for i in num_inputs/4..num_inputs/2 {
        intermediates.push(builder.mul(&inputs[2*i], &inputs[2*i + 1])); 
    }


    for i in 0..num_inputs/8 {
        builder.add(&intermediates[2*i], &intermediates[2*i + 1]); 
    }
    for i in num_inputs/8..num_inputs/4 {
        builder.mul(&intermediates[2*i], &intermediates[2*i + 1]); 
    }

    for i in 0..num_inputs {
        builder.mul(&constants[i], &inputs[i]);
    }
   
    builder.batch_assert_equal(&inputs, &inputs);
    builder.batch_set(&inputs, &vec![Fp::from(1); num_inputs]);

    let time_to_fill_nodes = Instant::now();
    builder.fill_nodes();
    println!("Time to fill nodes: {:?}", Instant::now() - time_to_fill_nodes);

    let check_constraints = builder.check_constraints().await;
    let end_time = Instant::now();    

    println!("Elapsed time: {:?}", end_time - start_time);
    println!("Constraints Passed? {:?}", check_constraints);
}

#[tokio::test]
async fn test_large_input_builder() {
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

    builder.batch_assert_equal(&inputs, &inputs);

    let time_to_fill_nodes = Instant::now();
    builder.fill_nodes(vec![Fp::from(1); num_inputs as usize]);
    println!("Time to fill nodes: {:?}", Instant::now() - time_to_fill_nodes);

    let check_constraints = builder.check_constraints().await;
    let end_time = Instant::now();    

    println!("Elapsed time: {:?}", end_time - start_time);
    println!("Constraints Passed? {:?}", check_constraints);
}


// Somehow this ends up filling the graph the fastest. 
#[test]
fn test_large_input_buildersinglethread() {
    let n: usize = 24; 

    let num_inputs = 2_i32.pow(n as u32); 
    let mut inputs = Vec::with_capacity(num_inputs as usize);
    let mut constants = Vec::with_capacity(num_inputs as usize);

    // 1 million inputs 
    let start_time = Instant::now();

    let mut builder = BuilderSingleThread::new();
    for _i in 0..num_inputs {
        inputs.push(builder.init());
    }

    for _i in 0..num_inputs {
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

    for i in 0..num_inputs {
        builder.assert_equal(&inputs[i as usize], &inputs[i as usize]);
    }

    let time_to_fill_nodes = Instant::now();
    builder.fill_nodes(vec![1; num_inputs as usize]);
    println!("Time to fill nodes: {:?}", Instant::now() - time_to_fill_nodes);

    let check_constraints = builder.check_constraints();
    let end_time = Instant::now();    
    println!("Elapsed time: {:?}", end_time - start_time);
    println!("Constraints Passed? {:?}", check_constraints);
}