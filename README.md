# Computational Graph Builder
A simple class to allow users to build a computational graph with support for concurrency and asynchronous functions. Example usage is shown below: 
```rust
// declare a galois field of order 65537 (only supports fields of prime order)
pub type Fp = GaloisField::<65537>;

fn main() {
    let mut builder = Builder::<Fp>::new(); 

    // declare an input wire. 
    let x = builder.init();

    // build a multiplier gate
    let x_squared = builder.mul(&x, &x);

    // declare a constant node
    let five = builder.constant(Fp::from(5));

    // declare adder gates 
    let x_squared_plus_5 = builder.add(&x_squared, &five);
    let y = builder.add(&x_squared_plus_5, &x);

    // pass in values to the inputs in the order that they were created
    builder.fill_nodes(vec![Fp::from(5)]);
}
```
Refer to ```tests/builder_tests.rs``` to see how to add equality assertions and verify them in circuit. I also designed a slightly different API with a modified underlying implementation. Example usage is shown below:
```rust
pub type Fp = GaloisField::<65537>;

fn test_basic_function() {
    let mut builder = GraphBuilder::<Fp>::new();

    let x = builder.init();
    let x_squared = builder.mul(&x, &x);
    
    let five = builder.constant(Fp::from(5));
    let x_squared_plus_5 = builder.add(&x_squared, &five);
    let y = builder.add(&x_squared_plus_5, &x);

    // the way to set values is a bit different
    builder.set(&x, Fp::from(5));

    // now fill_nodes() accepts no arguments
    builder.fill_nodes();
}
```

## Types of Builders.
There are essentially three types of builders and they represent the three different stages I went through when designing. 

* BuilderSingleThread: this was the first builder I wrote, so it was single threaded and does not use field elements.
* Builder: An improvement to the BuilderSingleThread that adds support for multithreading and async constraint checking.
* GraphBuilder: Maybe a performance improvement over Builder? This uses a vector to keep track of all nodes in the graph and has slightly more helpful debug messages. For the most part this is pretty similar to Builder, except the gates store a vector position rather than a node itself. 

To see how to use BuilderSingleThread, refer to ```benchmarks.rs```. For the others look at their respective tests file. 

## Asserting hints and equality constraints 
To specify a hint you need to specify a vector-function (i.e. a function that accepts a vector of arguments of type ```F``` and outputs something of type ```F```) to be used for the hint. The API for this is the same between both builders. 
```rust
fn lambda_div8(val: Vec<Fp>) -> Fp {
    assert_eq!(val.len(), 1);
    val[0] / Fp::from(8)
}

#[tokio::test]
async fn test_hints() {
    let mut builder = GraphBuilder::<Fp>::new();
    let a = builder.init();
    let one = builder.constant(Fp::from(1)); 
    let eight = builder.constant(Fp::from(8));

    let b = builder.add(&a, &one); 

    let c = builder.hint(&[&b], lambda_div8);
    let c_times_8 = builder.mul(&c, &eight);

    builder.set(&a, Fp::from(13));

    builder.fill_nodes();
    builder.assert_equal(&c_times_8, &b);

    let constraint_check = builder.check_constraints().await; 

    println!("{:?}", constraint_check); 
    println!("{:?}", c_times_8);
    println!("{:?}", b);
}

```
Refer to the tests for more examples. 

## Explanation of the Types
Here are the different types, how to use them, and what purpose they serve. 

### Galois Field Types
To declare a Galois Field of order $p$, you can set ```pub type Fp = GaloisField::<p>;```. The code supports operator overloading for the four basic operations.
```rust
// declare a galois field of order 65537 (only supports fields of prime order)
pub type Fp = GaloisField::<65537>;

fn main() {
    let x = Fp::from(54);
    let y = Fp::from(12); 
    let z  = x * y; 
    println!("{:?}", z); 
}
```

### Node Types
Node types are specified as shown below:
```rust
#[derive(Clone, Default, Debug)]
pub struct Node<F: Field> {
    pub value: Option<F>,
    pub depth: u64,
    pub id: usize,
}
```
The field ```value``` stores an option. Initially when nodes are initialized in the graph they all store ```None```. When ```fill_nodes()``` is called the null values are populated with the correct values. 
### Gate Types
These are the three types of gates supported by the builder.
```rust
// AddGate structure, which has two input nodes and one output node. 
// LEFT_ID is the position of the left node in builder.nodes,
// and RIGHT_ID is the position of the right node. 
// DEPTH is defined as in the README. 
#[derive(Debug)]
pub struct AddGate {
    left_id: usize,
    right_id: usize,
    output_id: usize,
    depth: u64,
}

#[derive(Debug)]
pub struct MultiplyGate {
    left_id: usize,
    right_id: usize,
    output_id: usize,
    depth: u64,
}

pub type Lambda<F> = fn(Vec<F>) -> F;

// LambdaGate structure to define arbitary hints based on other node values
// The function LAMBDA is used to determine the output.
// INPUT_IDS takes all the id's of the inputs
// OUTPUT_ID stores the id of the output
// DEPTH is defined as in README.
#[derive(Debug)]
pub struct LambdaGate<F: Field> {
    input_ids: Vec<usize>,
    output_id: usize,
    lambda: Lambda<F>,
    depth: u64,
}

```
The fields ```left_input``` and ```right_input``` represent the inputs to the adder gate and the multiplier gate, while ```output``` is either the sum of the values or the product. The argument ```depth``` is currently unused, but should be used to offer better debugging support. ```LambdaGate``` is a special type of gate, and is used to provide hints. The user can make ```Lambda<F>``` an arbitrarily complex function, but once all the inputs to the function are known the output is filled in. Usually ```LambdaGate``` should be paired with some sort of constraint since the output cannot be constrained (as the function may not map to addition and multiplication gates). 

### Design Specification
The order of execution is done in such a way to support parallelism. We declare all ```input``` and ```constant``` nodes to have depth $0$. Note that all other non-input and non-constant nodes must be the output of some gate. Suppose $r_1, r_2$ are inputs to gate $G$ with output $s$. Then, we declare
$$\text{depth}(s)  = 1 + \text{max}(\text{depth}(r_1), \text{depth}(r_2)).$$
Note that the only data dependencies occur when we increase depth. Thus, by staying in our "depth-level" we can use threads to evaluate all nodes on this level. A diagram is shown below for clarity. 

<img src=./img/spec.jpg alt="Schematic" width="600">

Note that without threads the best time complexity we can achieve is $O(\text{number input nodes } + \text{number of gates })$. 

#### Additional Considerations
I expect that add gates will be faster than multiplier gates and those will be faster than lambda gates. Thus, instead of splitting all the gates equally across the threads we should split all the different types of gates equally across the threads (that way a thread doesnt end up with all the lambda gates and slow down computation of the rest of the gates at that depth level). The struct LevelGates keeps track of this. 
```rust
#[derive(Debug)]
pub struct LevelGates<F: Field> {
    adder_gates: Vec<AddGate<F>>,
    multiplier_gates: Vec<MultiplyGate<F>>,
    lambda_gates: Vec<LambdaGate<F>>,
}
```

## Issues
The major issue is that somehow ```BuilderSingleThread``` outperforms both ```GraphBuilder``` and ```Builder``` when filling out the graph on my computer. I tested this on my friends computer and somehow on his, ```GraphBuilder``` executes faster than ```BuilderSingleThread```. I'm not sure why this is, but I think it may have something to do with the level 3 optimizations I set in ```cargo.toml```. 

