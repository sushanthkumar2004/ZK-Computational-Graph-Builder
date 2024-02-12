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
Refer to ```tests/builder_tests.rs``` to see how to add equality assertions and verify them in circuit. 

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
#### Tracking Depth

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
The order of execution is straightforward. To fill in the nodes, once the inputs are driven, simply go from lowest to highest depth and evaluate each gate. We can include concurrency by noting that when we evaluate all the gates at a certain level the computation can be run in parallel since the nodes we write to are different and there are no data dependecies. 
