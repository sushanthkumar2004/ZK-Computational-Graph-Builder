use std::{cmp::max, fmt, sync::Arc};
use parking_lot::RwLock;
use rayon::prelude::*;
use log::{debug, warn};

// Node is required to be wrapped in Arc for multiple thread access,
// and to support user having pointers to node objects in circuit 
type Node = Arc<RawNode>;

// Keeps track of all gates at the level
// Note that the gates are seperated by type
// since otherwise some threads could take much longer than others to finish. 
#[derive(Debug)]
pub struct LevelGates {
    adder_gates: Vec<AddGate>,
    multiplier_gates: Vec<MultiplyGate>,
    lambda_gates: Vec<LambdaGate>,
}

// Struct to assert equality between the node with id 
// left_id and the node with id right_id. 

// id's are assigned to nodes by builder as they are created. 
#[derive(Debug)]
pub struct EqualityAssertion {
    left_id: usize,
    right_id: usize,
}

// Struct that tracks the overall circuit.
// nodes: a vector of all the nodes in the circuit 
// gates: a vector of LevelGates. The ith element contains
// a LevelGates structure containing all gates present at depth i.
// assertions: a vector of equality assertions
// next_id: the next node added to the circuit will have this id. 
// Every time a new node is added, this value will be incremented. 
#[derive(Debug, Default)]
pub struct Builder {
    nodes: Vec<Node>, 
    gates: Vec<LevelGates>,
    assertions: Vec<EqualityAssertion>,
    next_id: usize,
}

// Used to track how each value in a node was computed, and mainly
// for user to debug constraint failures in circuit. 
#[derive(Debug, PartialEq)]
pub enum Derivation {
    Const,
    Input,
    Add,
    Mul,
    Hint,
}

// RawNode struct to track information in Node
// value: A mutable pointer to the value 
// (which is Option to handle unfilled values)
// depth: the depth of the node (i.e. the level it is at)
// id: the id of the node
// parents: the id's of the nodes used to derive this nodes value
// derivation: the method used to derive this nodes value 
#[derive(Debug)]
pub struct RawNode {
    pub value: RwLock<Option<u32>>,
    pub depth: u64,
    pub id: usize,
    pub parents: Vec<usize>, 
    pub derivation: Derivation
}

impl RawNode {
    /*
        Allows value of a raw node to be set

        ARGS:
            value: value to set the node to 
     */
    fn set(&self, value: Option<u32>) {
        *self.value.write() = value; 
    }

    /*
        Reads the value of a node

        RETURNS: 
            The value located at the AtomicPtr value field in RawNode
     */
    pub fn read(&self) -> u32 {
        self.value.read().unwrap_or_else(|| panic!("Value unfilled at node with id {:?}", self.id))
    }
}

impl fmt::Display for RawNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
// left_id is the position of the left node in builder.nodes,
// and right_id is the position of the right node. 
// output_id is the id of the output node containing the sum. 
#[derive(Debug)]
pub struct AddGate {
    left_id: usize,
    right_id: usize,
    output_id: usize,
}

// MultiplyGate structure, which has two input nodes and one output node. 
// left_id is the position of the left node in builder.nodes,
// and right_id is the position of the right node. 
// output_id is the id of the output node containing the product. 
#[derive(Debug)]
pub struct MultiplyGate {
    left_id: usize,
    right_id: usize,
    output_id: usize,
}

// Lambda type to use in order to specify a hint 
pub type Lambda = fn(Vec<u32>) -> u32;

// LambdaGate structure to define arbitary hints based on other node values
// input_ids: ids of input nodes to use 
// output_id: id of the output node 
// lambda: function used to determine the output.
#[derive(Debug)]
pub struct LambdaGate {
    input_ids: Vec<usize>,
    output_id: usize,
    lambda: Lambda,
}

impl Builder {
    /*
        Creates a new empty circuit

        RETURNS:
            An empty circuit with no nodes 
     */
    pub fn new() -> Self {
        Builder::default()
    }
    
    /*
        Initializes a new node

        RETURNS:
            An unfilled node object 
     */
    pub fn init(&mut self) -> Node {
        let node = Arc::new(RawNode {
            value: RwLock::new(None),
            depth: 0,
            id: self.next_id,
            parents: Vec::new(),
            derivation: Derivation::Input,
        });
        self.next_id += 1; 
        self.nodes.push(node.clone());
        node

    }

    /*
        Initializes a new node

        ARGS:
            num_inputs: the number of input nodes to initialize 

        RETURNS:
            A vector of input nodes to use for the circuit  
     */
    pub fn batch_init(&mut self, num_inputs: usize) -> Vec<Node> {
        let init_count = self.next_id; 
        let vector_input: Vec<Node> = (0..num_inputs).into_par_iter().map(|i| {
            Arc::new(RawNode {
                value: RwLock::new(None),
                depth: 0,
                id: init_count + i,
                parents: Vec::new(),
                derivation: Derivation::Input,
            })}).collect();
        self.nodes.extend(vector_input.clone());
        self.next_id += num_inputs;
        vector_input
    }

    /*
        Sets the value of a node in the graph. Does not allow setting the value 
        of a node that is driven by other nodes (as the output of a hint, or an
        arithmetic gate).

        ARGS:
            node: the node to change the value of
            value: the new value node should hold  
     */
    pub fn set(&mut self, node: Node, value: u32) {
        if node.depth == 0 && node.derivation != Derivation::Const {
            node.set(Some(value));
        } else {
            warn!("Cannot set value of non-input node {} as it is derived.", node)
        }
    }

    /*
        Sets the value of a a vector of nodes in the graph. Does not allow 
        setting the value of a node that is driven by other nodes 
        (as the output of a hint, or an arithmetic gate).

        ARGS:
            nodes: the vector of nodes to change the value of
            values: the new values node should hold  
     */
    pub fn batch_set(&mut self, nodes: &[Node], values: &[u32]) {
        assert_eq!(nodes.len(), values.len());
        nodes.par_iter().enumerate().for_each(|(i, node)| {
            if node.depth == 0 && node.derivation != Derivation::Const {
                node.set(Some(values[i]));
            } else {
                warn!("Cannot set value of non-input node {:?} as it is derived.", node)
            }
        });        
    }
    
    /*
        Initializes a new node holding a constant value

        ARGS:
            value: set a constant node to this value

        RETURNS:
            A constant node containing value 
     */
    pub fn constant(&mut self, value: u32) -> Node {
        let node = Arc::new(RawNode {
            value: RwLock::new(Some(value)),
            depth: 0,
            id: self.next_id,
            parents: Vec::new(),
            derivation: Derivation::Const,
        });
        self.next_id += 1; 
        self.nodes.push(node.clone());
        node
    }

    /*
        Initializes a vector of constant nodes

        ARGS:
            values: the constant values that the new nodes should hold

        RETURNS:
            A vector of constant nodes 
     */
    pub fn batch_constant(&mut self, values: &[u32]) -> Vec<Node> {
        let init_count = self.next_id; 
        let vector_constant: Vec<Node> = (0..values.len()).into_par_iter().map(|i| {
            Arc::new(RawNode {
                value: RwLock::new(Some(values[i])),
                depth: 0,
                id: init_count + i,
                parents: Vec::new(),
                derivation: Derivation::Const,
            })}).collect();
        self.nodes.extend(vector_constant.clone());
        self.next_id += values.len();
        vector_constant
    }
    
    /*
        Initializes a new node that is the output of an addition gate
        taking in two already existing nodes in the graph. 

        ARGS:
            a: the first input to the addition gate
            b: the second input to the addition gate

        RETURNS:
            A node holding the formal sum of node a and node b  
     */
    pub fn add(&mut self, a: Node, b: Node) -> Node {
        let a_depth = a.depth;
        let b_depth = b.depth;

        let depth_gate = max(a_depth, b_depth);

        let output_node = Arc::new(RawNode {
            value: RwLock::new(None),
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
    
    /*
        Initializes a new node that is the output of a multiplication gate
        taking in two already existing nodes in the graph. 

        ARGS:
            a: the first input to the multiplication gate
            b: the second input to the multiplication gate

        RETURNS:
            A node holding the formal product of node a and node b  
     */
    pub fn mul(&mut self, a: Node, b: Node) -> Node {
        let a_depth = a.depth;
        let b_depth = b.depth;

        let depth_gate = max(a_depth, b_depth);

        let output_node = Arc::new(RawNode {
            value: RwLock::new(None),
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
        Allows for a hint to be given (useful for operations like division)

        ARGS:
            arguments: an array of nodes that serve as inputs to the lambda
            lambda: a function that relates the values of these nodes to the value of the output (which is returned)

        RETURNS:
            Returns a node corresponding to the output of the lambda gate that is just in time filled once the arguments are computed. 
     */
    pub fn hint(&mut self, arguments: &[Node], lambda: Lambda) -> Node {
        // read in arguments which should be other nodes in the graph
        let depth_gate = arguments.iter().map(|arg| arg.depth).max().unwrap();

        // create an output node to store the value in
        let output_node = Arc::new(RawNode {
            value: RwLock::new(None),
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
        Allows for a single assertion to be declared. Declares
        left_arg node to equal right_arg node

        ARGS:
            left_arg: the left inputs
            right_arg: the right inputs
     */
    pub fn assert_equal(&mut self, left_arg: Node, right_arg: Node) {
        let assertion = EqualityAssertion {
            left_id: left_arg.id,
            right_id: right_arg.id,
        };
        self.assertions.push(assertion);
    }

    /*
        Allows for a batch of assertions to be declared. 
        Declares left_args[i] node to equal right_args[i] node
        for all i. 

        ARGS:
            left_args: the vector of left inputs
            right_arg: the vector of right inputs
     */
    pub fn batch_assert_equal(&mut self, left_args: &[Node], right_args: &[Node]) {
        assert_eq!(left_args.len(), right_args.len());

        let new_assertions: Vec<EqualityAssertion> = (0..right_args.len()).into_par_iter().map(|i| {
            EqualityAssertion {
                left_id: left_args[i].id,
                right_id: right_args[i].id,
            }}).collect();
        self.assertions.extend(new_assertions);
    }

    /*
        Multithreaded function to fill in all the nodes of the graph given inputs. 
        Expects that all inputs have already been set. If it encounters an unfilled 
        node in the graph, it throws an error message. 
     */
    pub fn fill_nodes(&mut self) {   
        for level_gate in &self.gates {
            let add_gates = &level_gate.adder_gates;
            let multiply_gates = &level_gate.multiplier_gates; 
            let lambda_gates = &level_gate.lambda_gates; 

            // iterate over all the gates, read the inputs and drive the outputs accordingly. 
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
        Async function to check that constraints between nodes are satisfied once nodes are filled in.

        RETURNS:
            a boolean value representing whether or not all equality constraints passed
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
                if !left_value.parents.is_empty() {
                    debug!("Node {} is directly affected by the following nodes:", left_value.id);
                    left_value.parents.iter().for_each(|node_id| 
                        debug!("    Node {}: {}", *node_id, self.nodes[*node_id])
                    );
                } else {
                    debug!("Node {} is an input node.", left_value.id);
                }

                debug!("Node {} contains {}", right_value.id, right_value);
                if !right_value.parents.is_empty() {
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