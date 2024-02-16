use std::{cmp::max, sync::Arc};
use rayon::prelude::*;
use parking_lot::RwLock;

use crate::field::Field;

type WrappedNode<F> = Arc<RwLock<Node<F>>>;

// Keeps track of all gates at the level
// Note that the gates are seperated by type
// since otherwise some threads could take much longer than others to finish. 
#[derive(Debug)]
pub struct LevelGates<F: Field> {
    adder_gates: Vec<AddGate<F>>,
    multiplier_gates: Vec<MultiplyGate<F>>,
    lambda_gates: Vec<LambdaGate<F>>,
}

// Struct to keep track of an equality assertion between two nodes
#[derive(Debug, Clone)]
pub struct EqualityAssertion<F: Field> {
    left_node: WrappedNode<F>,
    right_node: WrappedNode<F>,
}

// builder struct
// input_nodes keeps track of the nodes at the input layer,
// constant_nodes keeps track of all the constants
// gates_per_level stores all the gates at a certain level in a LevelGates structure.
// internal_count is basically used to assign an identifier to each node.  
#[derive(Debug, Default)]
pub struct Builder<F: Field> {
    input_nodes: Vec<WrappedNode<F>>,
    constant_nodes: Vec<WrappedNode<F>>,
    gates_per_level: Vec<LevelGates<F>>,
    assertions: Vec<EqualityAssertion<F>>,
    internal_count: u64,
}

// node struct to store the value and depth of the node
// id is just used for debugging purposes. 
#[derive(Clone, Default, Debug)]
pub struct Node<F: Field> {
    pub value: Option<F>,
    pub depth: u64,
    pub id: u64,
}

// sets the value of the node
impl<F: Field> Node<F> {
    fn set_value(&mut self, value: Option<F>) {
        self.value = value;
    }
}

// AddGate structure, which has two input nodes and one output node. 
#[derive(Debug)]
pub struct AddGate<F: Field> {
    left_input: WrappedNode<F>,
    right_input: WrappedNode<F>,
    output: WrappedNode<F>,
    depth: u64,
}

// Same structure as above, except it does multiplication instead. 
#[derive(Debug)]
pub struct MultiplyGate<F: Field> {
    left_input: WrappedNode<F>,
    right_input: WrappedNode<F>,
    output: WrappedNode<F>,
    depth: u64,
}

pub type Lambda<F> = fn(Vec<F>) -> F;

// LambdaGate structure to define arbitary hints based on other node values
// The function @lambda is used to determine the output.
#[derive(Debug)]
pub struct LambdaGate<F: Field> {
    inputs: Vec<WrappedNode<F>>,
    output: WrappedNode<F>,
    lambda: Lambda<F>
}

impl<F: Field> Builder<F> {
    pub fn new() -> Self {
        Self {
            input_nodes: Vec::new(),
            constant_nodes: Vec::new(),
            gates_per_level: Vec::new(),
            assertions: Vec::new(),
            internal_count: 0,
        }
    }
    
    // allows you to initialize a single input 
    // node in the graph 
    pub fn init(&mut self) -> WrappedNode<F> {
        let node = Arc::new(RwLock::new(Node {
            value: None,
            depth: 0,
            id: self.internal_count,
        }));
        self.internal_count += 1; 
        self.input_nodes.push(node.clone());
        node
    }

    // same as above but lets you initialize a vector of
    // inputs, and updates counts as needed. 
    pub fn batch_init(&mut self, num_inputs: u64) -> Vec<WrappedNode<F>> {
        let init_count = self.internal_count; 
        let vector_input: Vec<WrappedNode<F>> = (0..num_inputs).into_par_iter().map(|i| {
            Arc::new(RwLock::new(Node {
                value: None,
                depth: 0,
                id: init_count + i,
            }))}).collect();
        self.input_nodes.extend(vector_input.clone());
        self.internal_count += num_inputs;
        vector_input
    }
    
    // declare a single constant node 
    pub fn constant(&mut self, value: F) -> WrappedNode<F> {
        let node = Arc::new(RwLock::new(Node {
            value: Some(value),
            depth: 0,
            id: self.internal_count,
        }));
        // make sure to update internal_count of nodes
        // so that we can assign the correct id to the next one 
        self.internal_count += 1; 
        self.constant_nodes.push(node.clone());
        node
    }

    // same as above but lets you declare a vector of constants
    pub fn batch_constant(&mut self, values: &[F]) -> Vec<WrappedNode<F>> {
        let init_count = self.internal_count; 
        let vector_constant: Vec<WrappedNode<F>> = (0..values.len()).into_par_iter().map(|i| {
            Arc::new(RwLock::new(Node {
                value: Some(values[i]),
                depth: 0,
                id: init_count + i as u64,
            }))}).collect();
        self.constant_nodes.extend(vector_constant.clone());
        self.internal_count += values.len() as u64;
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
            id: self.internal_count,
        }));
        self.internal_count += 1; 

        let add_gate = AddGate {
            left_input: a.clone(),
            right_input: b.clone(),
            output: output_node.clone(),
            depth: depth_gate,
        };

        if self.gates_per_level.len() > depth_gate as usize {
            self.gates_per_level[depth_gate as usize].adder_gates.push(add_gate);
        } else {
            self.gates_per_level.push(LevelGates {
                adder_gates: Vec::new(),
                multiplier_gates: Vec::new(),
                lambda_gates: Vec::new(),
            });
            self.gates_per_level[depth_gate as usize].adder_gates.push(add_gate);
        }
        output_node
    }

    fn batch_add(&mut self, _left_arguments: &[WrappedNode<F>], _right_arguments: &[WrappedNode<F>]) -> Vec<WrappedNode<F>> {
        todo!()
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
            id: self.internal_count,
        }));
        self.internal_count += 1; 

        let multiply_gate = MultiplyGate {
            left_input: a.clone(),
            right_input: b.clone(),
            output: output_node.clone(),
            depth: depth_gate,
        };

        if self.gates_per_level.len() > depth_gate as usize {
            self.gates_per_level[depth_gate as usize].multiplier_gates.push(multiply_gate);
        } else {
            self.gates_per_level.push(LevelGates {
                adder_gates: Vec::new(),
                multiplier_gates: Vec::new(),
                lambda_gates: Vec::new(),
            });
            self.gates_per_level[depth_gate as usize].multiplier_gates.push(multiply_gate);
        }
        output_node
    }

    fn batch_multiply(&mut self, _left_arguments: &[WrappedNode<F>], _right_arguments: &[WrappedNode<F>]) -> Vec<WrappedNode<F>> {
        todo!()
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
            id: self.internal_count,
        }));
        self.internal_count += 1; 

        let cloned_arguments: Vec<_> = arguments.iter().cloned().cloned().collect();

        let lambda_gate = LambdaGate {
            inputs: cloned_arguments,
            output: output_node.clone(),
            lambda,
        };

        if self.gates_per_level.len() > depth_gate as usize {
            self.gates_per_level[depth_gate as usize].lambda_gates.push(lambda_gate);
        } else {
            self.gates_per_level.push(LevelGates {
                adder_gates: Vec::new(),
                multiplier_gates: Vec::new(),
                lambda_gates: Vec::new(),
            });
            self.gates_per_level[depth_gate as usize].lambda_gates.push(lambda_gate);
        }
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
    pub fn assert_equal(&mut self, a: &WrappedNode<F>, b: &WrappedNode<F>) -> EqualityAssertion<F> {
        let assertion = EqualityAssertion {
            left_node: a.clone(),
            right_node: b.clone(),
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
    pub fn batch_assert_equal(&mut self, left_args: &[WrappedNode<F>], right_args: &[WrappedNode<F>]) -> Vec<EqualityAssertion<F>> {
        assert_eq!(left_args.len(), right_args.len());

        let new_assertions: Vec<EqualityAssertion<F>> = (0..right_args.len()).into_par_iter().map(|i| {
            EqualityAssertion {
                left_node: left_args[i].clone(),
                right_node: right_args[i].clone(),
            }}).collect();
        self.assertions.extend(new_assertions.clone());
        new_assertions
    }


    /*
     * Multithreaded function to fill in all the nodes of the graph given inputs. Expects a list of
     * field elements for the values. Assigns values to variables based on the order they were initialized. 
     * 
     * ARGS: Vec<F>
     * Requires a vector of inputs 
     * RETURNS:
     * a boolean representing whether or not all equality constraints passed
     */
    pub fn fill_nodes(&mut self, node_values: Vec<F>) {
        assert_eq!(node_values.len(), self.input_nodes.len());

        self.input_nodes.par_iter()
            .zip(node_values.into_par_iter())
            .for_each(|(node, value)| {
                    let mut locked_node = node.write();
                    locked_node.value = Some(value);
            });
    
            
        for level_gate in &self.gates_per_level {
            let add_gates = &level_gate.adder_gates;
            let multiply_gates = &level_gate.multiplier_gates; 
            let lambda_gates = &level_gate.lambda_gates; 

            add_gates.par_iter().for_each(|gate| {
                let mut output = gate.output.write();
                let left_value = gate.left_input.read().value.unwrap();
                let right_value = gate.right_input.read().value.unwrap();
                output.set_value(Some(left_value + right_value));
            });

            multiply_gates.par_iter().for_each(|gate| {
                let mut output = gate.output.write();
                let left_value = gate.left_input.read().value.unwrap();
                let right_value = gate.right_input.read().value.unwrap();
                output.set_value(Some(left_value * right_value));
            });

            lambda_gates.par_iter().for_each(|gate| {
                let mut output = gate.output.write();
                let arguments: Vec<F> = gate.inputs.iter().map(|val| val.read().value.unwrap()).collect(); 
                output.set_value(Some((gate.lambda)(arguments)));
            });
        }
    }

    /*
     * Async function to check that constraints between nodes are satisfied once nodes are filled in.
     * 
     * ARGS: 
     * none 
     * RETURNS:
     * a boolean representing whether or not all equality constraints passed
     */
    pub async fn check_constraints(&mut self) -> bool {
        for assertion in &self.assertions {
            let future_left_value = async {
                assertion.left_node.read().value.unwrap()
            }.await;

            let future_right_value = async {
                assertion.right_node.read().value.unwrap()
            }.await;
            
            if future_left_value != future_right_value {
                let left_value = assertion.left_node.read();
                let right_value = assertion.right_node.read();

                eprintln!("Equality failed at following nodes: {:?}, {:?}", left_value, right_value);
                return false;
            }
        }
        true
    }
}

