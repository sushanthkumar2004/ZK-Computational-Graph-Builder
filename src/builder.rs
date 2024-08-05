use std::{cmp::max, fmt, sync::{atomic::{AtomicPtr, Ordering}, Arc}};
use rayon::prelude::*;
use log::{debug, warn};

// all nodes passed between the graphs need to be wrapped in Arc
// so that multiple threads can read the same node concurrently. 
type WrappedNode = Arc<Node>;

// Keeps track of all gates at the level
// Note that the gates are seperated by type
// since otherwise some threads could take much longer than others to finish. 
#[derive(Debug)]
pub struct LevelGates {
    adder_gates: Vec<AddGate>,
    multiplier_gates: Vec<MultiplyGate>,
    lambda_gates: Vec<LambdaGate>,
}

// Struct to keep track of an equality assertion between two nodes
// LEFT_ID stores the index of the left_node and RIGHT_ID stores 
// the index of the right_node. Asserts right_node == left_node
#[derive(Debug)]
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
#[derive(Debug)]
pub struct Builder {
    nodes: Vec<WrappedNode>, 
    gates: Vec<LevelGates>,
    assertions: Vec<EqualityAssertion>,
    next_id: usize,
}

#[derive(Debug)]
pub enum Derivation {
    Const,
    Input,
    Add,
    Mul,
    Hint,
}

// node struct to store the value and depth of the node
// ID is just used for debugging purposes. 
#[derive(Debug)]
pub struct Node {
    pub value: AtomicPtr<Option<u32>>,
    pub depth: u64,
    pub id: usize,
    pub parents: Vec<usize>, 
    pub derivation: Derivation
}

impl Node {
    // allows the user to essentially reset the value in the box
    pub fn set(&self, value: Option<u32>) {
        let value_ptr = Box::into_raw(Box::new(value));
        self.value.store(value_ptr, Ordering::Relaxed);
    }

    // unsafe function that returns self.value as a field element.
    // Note that Rust cannot gaurantee the .as_ref() operation is safe,
    // but I can ensure that this will not lead to undefined behavior. 
    // Also, there seems to be no other way to even access the value. 
    pub fn read(&self) -> u32 {
        unsafe { self.value.load(Ordering::Relaxed).as_ref().unwrap_or_else(|| panic!("Raw dereference failed!")).unwrap_or_else(|| panic!("Value unfilled at id {}!", self.id)) }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write the desired format using the `write!` macro
        match self.derivation {
            Derivation::Const => write!(f, "Node {{ value: {}, depth: {}, id: {}, parents: {:?}, derivation: Constant }}", self.read(), self.depth, self.id, self.parents),
            Derivation::Input => write!(f, "Node {{ value: {}, depth: {}, id: {}, parents: {:?}, derivation: Input }}", self.read(), self.depth, self.id, self.parents),
            Derivation::Add => write!(f, "Node {{ value: {}, depth: {}, id: {}, parents: {:?}, derivation: Addition Gate }}", self.read(), self.depth, self.id, self.parents),
            Derivation::Mul => write!(f, "Node {{ value: {}, depth: {}, id: {}, parents: {:?}, derivation: Multiplication Gate }}", self.read(), self.depth, self.id, self.parents),
            Derivation::Hint => write!(f, "Node {{ value: {}, depth: {}, id: {}, parents: {:?}, derivation: Hint }}", self.read(), self.depth, self.id, self.parents),
        }
        
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
}

#[derive(Debug)]
pub struct MultiplyGate {
    left_id: usize,
    right_id: usize,
    output_id: usize,
}

pub type Lambda = fn(Vec<u32>) -> u32;

// LambdaGate structure to define arbitary hints based on other node values
// The function LAMBDA is used to determine the output.
// INPUT_IDS takes all the id's of the inputs
// OUTPUT_ID stores the id of the output
// DEPTH is defined as in README.
#[derive(Debug)]
pub struct LambdaGate {
    input_ids: Vec<usize>,
    output_id: usize,
    lambda: Lambda,
}

// Note that all operations are done in F
impl Builder {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            gates: Vec::new(),
            assertions: Vec::new(),
            next_id: 0,
        }
    }
    
    // method to initialize a new node. 
    pub fn init(&mut self) -> WrappedNode {
        let node = Arc::new(Node {
            value: AtomicPtr::new(Box::into_raw(Box::new(None))),
            depth: 0,
            id: self.next_id,
            parents: Vec::new(),
            derivation: Derivation::Input,
        });
        self.next_id += 1; 
        self.nodes.push(node.clone());
        node

    }

    // allows one to batch initialize a vector of inputs of size num_inputs using multithreading
    pub fn batch_init(&mut self, num_inputs: usize) -> Vec<WrappedNode> {
        let init_count = self.next_id; 
        let vector_input: Vec<WrappedNode> = (0..num_inputs).into_par_iter().map(|i| {
            Arc::new(Node {
                value: AtomicPtr::new(Box::into_raw(Box::new(None))),
                depth: 0,
                id: init_count + i,
                parents: Vec::new(),
                derivation: Derivation::Input,
            })}).collect();
        self.nodes.extend(vector_input.clone());
        self.next_id += num_inputs;
        vector_input
    }

    // slightly different API. Allows the user to set any node in the graph. 
    // NOTE: builder.set() allows the user to set ANY node in the graph including those
    // that are driven by outputs in gates. Calling builder.fill_nodes() will
    // safely override the nodes that it needs, but the asynchronous function builder.check_constraints()
    // may fail if the value is overriden by the user. 
    pub fn set(&mut self, node: WrappedNode, value: u32) {
        if node.depth == 0 {
            node.set(Some(value));
        } else {
            warn!("Cannot set value of non-input node {:?} as it is derived.", node)
        }
    }

    // Allows you to set a vector of inputs 
    pub fn batch_set(&mut self, nodes: &[WrappedNode], values: &[u32]) {
        nodes.par_iter().enumerate().for_each(|(i, node)| {
            if node.depth == 0 {
                node.set(Some(values[i]));
            } else {
                warn!("Cannot set value of non-input node {:?} as it is derived.", node)
            }
        });        
    }
    
    // declare a constant node in the graph 
    pub fn constant(&mut self, value: u32) -> WrappedNode {
        let node = Arc::new(Node {
            value: AtomicPtr::new(Box::into_raw(Box::new(Some(value)))),
            depth: 0,
            id: self.next_id,
            parents: Vec::new(),
            derivation: Derivation::Const,
        });
        self.next_id += 1; 
        self.nodes.push(node.clone());
        node
    }

    // declare a batch of constants given a vector of values
    pub fn batch_constant(&mut self, values: &[u32]) -> Vec<WrappedNode> {
        let init_count = self.next_id; 
        let vector_constant: Vec<WrappedNode> = (0..values.len()).into_par_iter().map(|i| {
            Arc::new(Node {
                value: AtomicPtr::new(Box::into_raw(Box::new(Some(values[i])))),
                depth: 0,
                id: init_count + i,
                parents: Vec::new(),
                derivation: Derivation::Const,
            })}).collect();
        self.nodes.extend(vector_constant.clone());
        self.next_id += values.len();
        vector_constant
    }
    
    // instantiate an add gate between two nodes and get an output node
    // that represents the addition of the two supplied nodes
    pub fn add(&mut self, a: WrappedNode, b: WrappedNode) -> WrappedNode {
        let a_depth = a.depth;
        let b_depth = b.depth;

        let depth_gate = max(a_depth, b_depth);

        let output_node = Arc::new(Node {
            value: AtomicPtr::new(Box::into_raw(Box::new(None))),
            depth: depth_gate + 1,
            id: self.next_id,
            parents: vec![a.id, b.id],
            derivation: Derivation::Add
        });
        
        let add_gate = AddGate {
            left_id: a.id,
            right_id: b.id,
            output_id: output_node.id,
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
    pub fn mul(&mut self, a: WrappedNode, b: WrappedNode) -> WrappedNode {
        let a_depth = a.depth;
        let b_depth = b.depth;

        let depth_gate = max(a_depth, b_depth);

        let output_node = Arc::new(Node {
            value: AtomicPtr::new(Box::into_raw(Box::new(None))),
            depth: depth_gate + 1,
            id: self.next_id,
            parents: vec![a.id, b.id],
            derivation: Derivation::Mul
        });

        let multiply_gate = MultiplyGate {
            left_id: a.id,
            right_id: b.id,
            output_id: output_node.id,
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

    pub fn hint(&mut self, arguments: &[WrappedNode], lambda: Lambda) -> WrappedNode {
        // read in arguments which should be other nodes in the graph
        let depth_gate = arguments.iter().map(|arg| arg.depth).max().unwrap();

        // create an output node to store the value in
        let output_node = Arc::new(Node {
            value: AtomicPtr::new(Box::into_raw(Box::new(None))),
            depth: depth_gate + 1,
            id: self.next_id,
            parents: arguments.iter().map(|arg| arg.id).collect(),
            derivation: Derivation::Hint
        });
        
        // get the positions of the nodes in the vector self.nodes, 
        // so that the values can be extracted later
        let argument_ids: Vec<_> = arguments.iter().map(|node| node.id).collect();

        let lambda_gate = LambdaGate {
            input_ids: argument_ids,
            output_id: output_node.id,
            lambda,
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

    pub fn assert_equal(&mut self, left_arg: WrappedNode, right_arg: WrappedNode) {
        let assertion = EqualityAssertion {
            left_id: left_arg.id,
            right_id: right_arg.id,
        };
        self.assertions.push(assertion);
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

    pub fn batch_assert_equal(&mut self, left_args: &[WrappedNode], right_args: &[WrappedNode]) {
        assert_eq!(left_args.len(), right_args.len());

        let new_assertions: Vec<EqualityAssertion> = (0..right_args.len()).into_par_iter().map(|i| {
            EqualityAssertion {
                left_id: left_args[i].id,
                right_id: right_args[i].id,
            }}).collect();
        self.assertions.extend(new_assertions);
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
                let left_value = self.nodes[gate.left_id].read();
                let right_value = self.nodes[gate.right_id].read();
                self.nodes[gate.output_id].set(Some(left_value + right_value));
            });

            multiply_gates.par_iter().for_each(|gate| {
                let left_value = self.nodes[gate.left_id].read();
                let right_value = self.nodes[gate.right_id].read();
                self.nodes[gate.output_id].set(Some(left_value * right_value));
            });
            
            lambda_gates.par_iter().for_each(|gate| {
                let arguments: Vec<_> = gate.input_ids.iter().map(|&i| self.nodes[i].read()).collect();
                self.nodes[gate.output_id].set(Some((gate.lambda)(arguments)));
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
                self.nodes[assertion.left_id].read()
            }.await;

            let future_right_value = async {
                self.nodes[assertion.right_id].read()
            }.await;
            
            if future_left_value != future_right_value {
                let left_value = self.nodes[assertion.left_id].clone();
                let right_value = self.nodes[assertion.right_id].clone();

                debug!("Equality failed at nodes with id's {}, {}", left_value.id, right_value.id);
                debug!("Node {} contains {}", left_value.id, left_value);
                if left_value.parents.len() != 0 {
                    debug!("Node {} is directly affected by the following nodes:", left_value.id);
                    left_value.parents.iter().for_each(|node_id| 
                        debug!("    Node {}: {}", *node_id, self.nodes[*node_id])
                    );
                } else {
                    debug!("Node {} is an input node.", left_value.id);
                }

                debug!("Node {} contains {}", right_value.id, right_value);
                if right_value.parents.len() != 0 {
                    debug!("Node {} is directly affected by the following nodes:", right_value.id);
                    right_value.parents.iter().for_each(|node_id| 
                        debug!("    Node {}: {}", *node_id, self.nodes[*node_id])
                    );
                } else {
                    debug!("Node {} is an input node.", right_value.id);
                }
                
                return false;
            }
        }
        true
    }
}

