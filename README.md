# Computational Graph Builder
A computational graph builder that allows designs of circuits involving addition, multiplication, equality assertions and hints. Supports concurrency and asynchronous equality constraint checking. 
## Specifying Inputs, Constants and Gates
When building a computational graph, start by calling the ```Builder::new()``` method, which creates an empty circuit. One can specify input nodes to the circuit by calling ```builder.init()```, which returns a node pointer and creates an input node in circuit. One can also create constant nodes by using the ```builder.constant(val: u32)``` method, and addition/multiplication gates can be specified as shown below. 
```rust
fn main() {
    // Example 1: f(x) = x^2 + x + 5
  
    // instantiates an empty circuit with no nodes
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
}
```
The ```fill_nodes``` method allows the rest of the graph to be filled out in parallel according to the input nodes.
## Hints and Constraint Checking
Constraints in the circuit are a set of equality assertions between nodes. The equality assertions can be used to constrain possible values of the circuit and to also check that computations are correct. These are called by declaring ```rust builder.assert_equal(node1, node2)```. Calling ```builder.check_constraints().await``` after filling out the graph will verify that the assertions pass as expected. Note however that since the ```check_constraints``` method is asynchronous it is actually possible to call ```check_constraints``` before actually filling out the circuit. 

Hints are API's provided by ```Builder``` to support operations other than addition and multiplication. Typical usage is to specify a vector function (function that takes ```rust Vec<u32>``` as input and outputs a ```rust u32```), and a vector of the argument nodes to apply the function on. Usually, one also adds an equality assertion on top of this to verify that the vector function correctly computed the values. Example usage is shown below:
```rust
// A simple square root function that rounds to
// the nearest integer.
// To be used as a hint, the function argument
// must be a vector of 32-bit integers and
// output a 32 bit integer. 
fn lambda_sqrt(val: Vec<u32>) -> u32 {
    ((val[0] as f64).sqrt().round()) as u32
}

// To use the check_constraints method this must
// be an async function 
async fn main() {
    // Example 3: f(x) = sqrt(x+7)
    //
    // Assume that x+7 is a perfect square (so x = 2 or 9, etc.).

    let mut builder = Builder::new();
    let x = builder.init();
    let seven = builder.constant(7);
    let x_plus_seven = builder.add(x.clone(), seven.clone());

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

    // API for asserting equality between nodes
    // Asserts equality between the left and right node.
    // Requires calling check_constraints() to validate
    // that the constraint is met. 
    builder.assert_equal(computed_sq.clone(), x_plus_seven.clone());

    builder.set(x.clone(), 2);
    builder.fill_nodes();

    builder.check_constraints().await
}
```
The ```check_constraints``` method was made asynchronous to allow for assertions to be checked as the graph was being filled out. 
## Debugging
The ```check_constraints``` function evaluates constraints in the order that they are specified, and execution halts at the first failed constraint. When the constraint fails, debug information is printed out to the logs. This includes information for the two nodes that failed the equality constraint and the nodes directly influencing the value of the left and right nodes. 
```rust
async fn main() {
    let mut builder = Builder::new();
    let a = builder.init();
    let one = builder.constant(1); 
    let eight = builder.constant(8);

    let b = builder.add(a.clone(), one); 

    let c = builder.init();
    let c_times_8 = builder.mul(c.clone(), eight.clone());

    builder.set(a.clone(), 13);
    builder.set(c.clone(), 2);

    builder.fill_nodes();
    builder.assert_equal(c_times_8.clone(), b.clone());

    builder.check_constraints().await
}
```
The debug information reveals the method used to evaluate the left and right nodes as well. 
```
[2024-08-05T10:54:12Z DEBUG takehome::builder] Equality failed at nodes with id's 5, 3
[2024-08-05T10:54:12Z DEBUG takehome::builder] Node 5 contains Node { value: 16, depth: 1, id: 5, parents: [4, 2], derivation: Multiplication Gate }
[2024-08-05T10:54:12Z DEBUG takehome::builder] Node 5 is directly affected by the following nodes:
[2024-08-05T10:54:12Z DEBUG takehome::builder]     Node 4: Node { value: 2, depth: 0, id: 4, parents: [], derivation: Input }
[2024-08-05T10:54:12Z DEBUG takehome::builder]     Node 2: Node { value: 8, depth: 0, id: 2, parents: [], derivation: Constant }
[2024-08-05T10:54:12Z DEBUG takehome::builder] Node 3 contains Node { value: 14, depth: 1, id: 3, parents: [0, 1], derivation: Addition Gate }
[2024-08-05T10:54:12Z DEBUG takehome::builder] Node 3 is directly affected by the following nodes:
[2024-08-05T10:54:12Z DEBUG takehome::builder]     Node 0: Node { value: 13, depth: 0, id: 0, parents: [], derivation: Input }
[2024-08-05T10:54:12Z DEBUG takehome::builder]     Node 1: Node { value: 1, depth: 0, id: 1, parents: [], derivation: Constant }
```
## Approach 
To evaluate nodes correctly and concurrently, we use the concept of node depth. All ```input``` and ```constant``` nodes are declared to have depth 0. We calculate the depths of all other nodes as follows. Let node $M$ be a function of nodes $N_1,\ldots, N_n$, and for a node $X$, let $\text{depth}(X)$ denote the depth of node $X$. We set
$$\text{depth}(M) = 1 + \text{max}_{1\le i\le n} \text{depth}(N_i).$$
Evaluate nodes at increasing depths, starting from depth 0 and progressing sequentially. The key idea is that nodes at the same depth can be evaluated in parallel, since by the time compute reaches depth $k$, all nodes of previous depths have been fully computed, and there are no dependecies between nodes of the same depth. 

<p align="center">
  <img src=./img/spec.jpg alt="Schematic" width="600">
</p>
#### Additional Considerations
In general multiplication operations are slower than addition operations, and lambda gates can be arbitrarily simple or arbitrarily complex. Instead of splitting all the gates equally across the threads we should split all the different types of gates equally across the threads (this allows the workload across nodes to be more equal). The struct LevelGates keeps track of the gates at each level and seperates by types to facilitate this. 
```rust
#[derive(Debug)]
pub struct LevelGates<F: Field> {
    adder_gates: Vec<AddGate<F>>,
    multiplier_gates: Vec<MultiplyGate<F>>,
    lambda_gates: Vec<LambdaGate<F>>,
}
```
