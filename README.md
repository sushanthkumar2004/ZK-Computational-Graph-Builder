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

## Using add and multiply gates

## Asserting hints and equality constraints 

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

### Gate Types
These are the three types of gates supported by the builder.
```rust
#[derive(Debug)]
pub struct AddGate<F: Field> {
    left_input: Arc<RwLock<Node<F>>>,
    right_input: Arc<RwLock<Node<F>>>,
    output: Arc<RwLock<Node<F>>>,
    depth: u64,
}

#[derive(Debug)]
pub struct MultiplyGate<F: Field> {
    left_input: Arc<RwLock<Node<F>>>,
    right_input: Arc<RwLock<Node<F>>>,
    output: Arc<RwLock<Node<F>>>,
    depth: u64,
}

pub type Lambda<F> = fn(Vec<F>) -> F;

#[derive(Debug)]
pub struct LambdaGate<F: Field> {
    inputs: Vec<Arc<RwLock<Node<F>>>>,
    output: Arc<RwLock<Node<F>>>,
    lambda: Lambda<F>
}

```
The fields ```left_input``` and ```right_input``` represent the inputs to the adder gate and the multiplier gate, while ```output``` is either the sum of the values or the product. The argument ```depth``` is currently unused, but should be used to offer better debugging support. ```LambdaGate``` is a special type of gate, and is used to provide hints. The user can make ```Lambda<F>``` an arbitrarily complex function, but once all the inputs to the function are known the output is filled in. Usually ```LambdaGate``` should be paired with some sort of constraint since the output cannot be constrained (since the function may not map to addition and multiplication gates). 

### Design Specification
The order of execution is done in such a way to support parallelism. We declare all ```input``` and ```constant``` nodes to have depth $0$. Note that all other non-input and non-constant nodes must be the output of some gate. Suppose $r_1, r_2$ are inputs to gate $G$ with output $s$. Then, we declare
$$\text{depth}(s)  = 1 + \text{max}(\text{depth}(r1), \text{depth}(r2)).$$
Note that the only data dependencies occur when we increase depth. Thus, by staying in our "depth-level" we can use threads to evaluate all nodes on this level. A diagram is shown below for clarity. 

<img src=./img/spec.jpg alt="Schematic" width="600">

Note that without threads the best time complexity we can achieve is $O(\text{\# input nodes } + \text{\# gates })$. The overhead due to threading is not much as can be seen with the following benchmarks:

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

