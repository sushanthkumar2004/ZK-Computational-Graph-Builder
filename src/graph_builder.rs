use std::{cmp::max, sync::{Arc}};
use parking_lot::RwLock;
use rayon::prelude::*;

use crate::field::Field;

type WrappedNode<F> = Arc<RwLock<Node<F>>>;

// Keeps track of all gates at the level
// Note that the gates are seperated by type
// since otherwise some threads could take much longer than others to finish. 
#[derive(Debug)]
pub struct LevelGates<F: Field> {
    adder_gates: Vec<AddGate>,
    multiplier_gates: Vec<MultiplyGate>,
    lambda_gates: Vec<LambdaGate<F>>,
}

// Struct to keep track of an equality assertion between two nodes
// LEFT_ID stores the index of the left_node and RIGHT_ID stores 
// the index of the right_node. Asserts right_node == left_node
#[derive(Debug, Clone)]
pub struct EqualityAssertion {
    left_id: usize,
    right_id: usize,
}

// builder struct
// NODES is a vector that keeps track of all the nodes in the graph,
// GATES is a set of gates aggregated by depth and seperated by type
// Note: Gates[i] will return a LevelGates structure that stores all the gates
// in depth level i by their type. 
// ASSERTIONS stores all the equality assertions that the user makes
// NEXT_ID is basically used to assign an identifier to each node. 
// As a node is added to the graph, NEXT_ID is incremented by 1
#[derive(Debug, Default)]
pub struct GraphBuilder<F: Field> {
    nodes: Vec<WrappedNode<F>>, 
    gates: Vec<LevelGates<F>>,
    assertions: Vec<EqualityAssertion>,
    next_id: usize,
}

// node struct to store the value and depth of the node
// ID is just used for debugging purposes. 
#[derive(Clone, Default, Debug)]
pub struct Node<F: Field> {
    pub value: Option<F>,
    pub depth: u64,
    pub id: usize,
}

// sets the value of the node
impl<F: Field> Node<F> {
    pub fn set_value(&mut self, value: Option<F>) {
        self.value = value;
    }
}

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

// Note that all operations are done in F
impl<F: Field> GraphBuilder<F> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            gates: Vec::new(),
            assertions: Vec::new(),
            next_id: 0,
        }
    }
    
    // method to initialize a new node. 
    pub fn init(&mut self) -> WrappedNode<F> {
        let node = Arc::new(RwLock::new(Node {
            value: None,
            depth: 0,
            id: self.next_id,
        }));
        self.next_id += 1; 
        self.nodes.push(node.clone());
        node

    }

    // allows one to batch initialize a vector of inputs of size num_inputs using multithreading
    pub fn batch_init(&mut self, num_inputs: usize) -> Vec<WrappedNode<F>> {
        let init_count = self.next_id; 
        let vector_input: Vec<WrappedNode<F>> = (0..num_inputs).into_par_iter().map(|i| {
            Arc::new(RwLock::new(Node {
                value: None,
                depth: 0,
                id: init_count + i,
            }))}).collect();
        self.nodes.extend(vector_input.clone());
        self.next_id += num_inputs;
        vector_input
    }

    // slightly different API. Allows the user to set any node in the graph. 
    // NOTE: builder.set() allows the user to set ANY node in the graph including those
    // that are driven by outputs in gates. Calling builder.fill_nodes() will
    // safely override the nodes that it needs, but the asynchronous function builder.check_constraints()
    // may fail if the value is overriden by the user. 
    pub fn set(&mut self, node: &WrappedNode<F>, value: F) {
        node.write().set_value(Some(value));
    }

    // Allows you to set a vector of inputs 
    pub fn batch_set(&mut self, nodes: &[WrappedNode<F>], values: &[F]) {
        nodes.par_iter().enumerate().for_each(|(i, node)| {
            node.write().set_value(Some(values[i]));
        });        
    }
    
    // declare a constant node in the graph 
    pub fn constant(&mut self, value: F) -> WrappedNode<F> {
        let node = Arc::new(RwLock::new(Node {
            value: Some(value),
            depth: 0,
            id: self.next_id,
        }));
        self.next_id += 1; 
        self.nodes.push(node.clone());
        node
    }

    // declare a batch of constants given a vector of values
    pub fn batch_constant(&mut self, values: &[F]) -> Vec<WrappedNode<F>> {
        let init_count = self.next_id; 
        let vector_constant: Vec<WrappedNode<F>> = (0..values.len()).into_par_iter().map(|i| {
            Arc::new(RwLock::new(Node {
                value: Some(values[i]),
                depth: 0,
                id: init_count + i,
            }))}).collect();
        self.nodes.extend(vector_constant.clone());
        self.next_id += values.len();
        vector_constant
    }
    
    // instantiate an add gate between two nodes and get an output node
    // that represents the addition of the two supplied nodes
    pub fn add(&mut self, a: &WrappedNode<F>, b: &WrappedNode<F>) -> WrappedNode<F> {
        let a_depth = a.read().depth;
        let b_depth = b.read().depth;

        let depth_gate = max(a_depth, b_depth);

        let output_node = Arc::new(RwLock::new(Node {
            value: None,
            depth: depth_gate + 1,
            id: self.next_id,
        }));
        
        let add_gate = AddGate {
            left_id: a.read().id,
            right_id: b.read().id,
            output_id: output_node.read().id,
            depth: depth_gate,
        };

        self.nodes.push(output_node.clone());
        self.next_id += 1; 

        if self.gates.len() <= depth_gate as usize {
            self.gates.push(LevelGates {
                adder_gates: Vec::new(),
                multiplier_gates: Vec::new(),
                lambda_gates: Vec::new(),
            });
        }

        self.gates[depth_gate as usize].adder_gates.push(add_gate);
        output_node
    }
    
    // instantiate a multiply gate between two nodes and get an output node
    // that represents the addition of the two supplied nodes
    pub fn mul(&mut self, a: &WrappedNode<F>, b: &WrappedNode<F>) -> WrappedNode<F> {
        let a_depth = a.read().depth;
        let b_depth = b.read().depth;

        let depth_gate = max(a_depth, b_depth);

        let output_node = Arc::new(RwLock::new(Node {
            value: None,
            depth: depth_gate + 1,
            id: self.next_id,
        }));

        let multiply_gate = MultiplyGate {
            left_id: a.read().id,
            right_id: b.read().id,
            output_id: output_node.read().id,
            depth: depth_gate,
        };

        self.nodes.push(output_node.clone());
        self.next_id += 1; 

        if self.gates.len() <= depth_gate as usize {
            self.gates.push(LevelGates {
                adder_gates: Vec::new(),
                multiplier_gates: Vec::new(),
                lambda_gates: Vec::new(),
            });
        }

        self.gates[depth_gate as usize].multiplier_gates.push(multiply_gate);
        output_node
    }

    /*
     * Allows for a hint to be given (useful for operations like division)
     * 
     * ARGS: 
     * arguments: an array of nodes that serve as inputs to the lambda
     * lambda: a function that relates the values of these nodes to the value of the output (which is returned)
     * RETURNS:
     * returns a node corresponding to the output of the lambda gate that is just in time filled once the arguments are computed. 
     */

    pub fn hint(&mut self, arguments: &[&WrappedNode<F>], lambda: Lambda<F>) -> WrappedNode<F> {
        let depth_gate = arguments.iter().map(|arg| arg.read().depth).max().unwrap();

        let output_node = Arc::new(RwLock::new(Node {
            value: None,
            depth: depth_gate + 1,
            id: self.next_id,
        }));
        

        let argument_ids: Vec<_> = arguments.iter().map(|node| node.read().id).collect();

        let lambda_gate = LambdaGate {
            input_ids: argument_ids,
            output_id: output_node.read().id,
            lambda,
            depth: depth_gate,
        };

        self.nodes.push(output_node.clone());
        self.next_id += 1; 

        if self.gates.len() <= depth_gate as usize {
            self.gates.push(LevelGates {
                adder_gates: Vec::new(),
                multiplier_gates: Vec::new(),
                lambda_gates: Vec::new(),
            });
        }

        self.gates[depth_gate as usize].lambda_gates.push(lambda_gate);
        output_node
    }
    
    /*
     * Allows for a single assertion to be declared
     * 
     * ARGS: 
     * left_arg: the left inputs
     * right_arg: the right inputs
     * The assertions will assert that left_args[i] = right_args[i]
     * RETURNS:
     * returns a vector of equality assertions
     */

    pub fn assert_equal(&mut self, left_arg: &WrappedNode<F>, right_arg: &WrappedNode<F>) -> EqualityAssertion {
        let assertion = EqualityAssertion {
            left_id: left_arg.read().id,
            right_id: right_arg.read().id,
        };
        self.assertions.push(assertion.clone());
        assertion
    }

    /*
     * Allows for a batch of assertions to be declared
     * 
     * ARGS: 
     * left_args: All the left inputs
     * right_args: all the right inputs
     * All assertions will be of the form left_args[i] = right_args[i]
     * RETURNS:
     * returns a vector of equality assertions
     */

    pub fn batch_assert_equal(&mut self, left_args: &[WrappedNode<F>], right_args: &[WrappedNode<F>]) -> Vec<EqualityAssertion> {
        assert_eq!(left_args.len(), right_args.len());

        let new_assertions: Vec<EqualityAssertion> = (0..right_args.len()).into_par_iter().map(|i| {
            EqualityAssertion {
                left_id: left_args[i].read().id,
                right_id: right_args[i].read().id,
            }}).collect();
        self.assertions.extend(new_assertions.clone());
        new_assertions
    }

    /*
     * Multithreaded function to fill in all the nodes of the graph given inputs. Expects that all inputs
     * have already been set. If it encounters an unfilled node in the graph, it throws an error message. 
     * 
     * ARGS: 
     * none
     * RETURNS:
     * none
     */
    pub fn fill_nodes(&mut self) {   
        for level_gate in &self.gates {
            let add_gates = &level_gate.adder_gates;
            let multiply_gates = &level_gate.multiplier_gates; 
            let lambda_gates = &level_gate.lambda_gates; 

            // parallel iterate over all the gates, read the inputs and drive the outputs accordingly. 
            // I used unwrap_or_else to handle values that were unfilled. 

            add_gates.par_iter().for_each(|gate| {
                let mut output = self.nodes[gate.output_id].write();
                let left_value = self.nodes[gate.left_id].read().value.unwrap_or_else(|| panic!("Value not filled at depth {}! Did you set all inputs?", gate.depth));
                let right_value = self.nodes[gate.right_id].read().value.unwrap_or_else(|| panic!("Value not filled at depth {}! Did you set all inputs?", gate.depth));
                output.set_value(Some(left_value + right_value));
            });

            multiply_gates.par_iter().for_each(|gate| {
                let mut output = self.nodes[gate.output_id].write();
                let left_value = self.nodes[gate.left_id].read().value.unwrap_or_else(|| panic!("Value not filled at depth {}! Did you set all inputs?", gate.depth));
                let right_value = self.nodes[gate.right_id].read().value.unwrap_or_else(|| panic!("Value not filled at depth {}! Did you set all inputs?", gate.depth));
                output.set_value(Some(left_value * right_value));
            });

            lambda_gates.par_iter().for_each(|gate| {
                let mut output = self.nodes[gate.output_id].write();
                let arguments: Vec<_> = gate.input_ids.iter().map(|&i| self.nodes[i].read().value.unwrap_or_else(|| panic!("Value not filled at depth {}! Did you set all inputs?", gate.depth))).collect();
                output.set_value(Some((gate.lambda)(arguments)));
            });

        }
    }

    /*
     * Async function to check that constraints between nodes are satisfied once nodes are filled in.
     * 
     * RETURNS:
     * a boolean representing whether or not all equality constraints passed
     */
    pub async fn check_constraints(&mut self) -> bool {
        for assertion in &self.assertions {
            let future_left_value = async {
                self.nodes[assertion.left_id].read().value.unwrap()
            }.await;

            let future_right_value = async {
                self.nodes[assertion.right_id].read().value.unwrap()
            }.await;
            
            if future_left_value != future_right_value {
                let left_value = self.nodes[assertion.left_id].read();
                let right_value = self.nodes[assertion.right_id].read();

                eprintln!("Equality failed at following nodes: {:?}, {:?}", left_value, right_value);
                return false;
            }
        }
        true
    }
}

