// use std::{cmp::max, cell::RefCell, rc::Rc};

// #[derive(Debug)]
// pub struct LevelGates {
//     adder_gates: Vec<AddGate>,
//     multiplier_gates: Vec<MultiplyGate>,
// }

// #[derive(Debug)]
// pub struct EqualityAssertion {
//     left_node: Rc<RefCell<Node>>,
//     right_node: Rc<RefCell<Node>>,
// }

// #[derive(Debug)]
// pub struct Builder {
//     input_nodes: Vec<Rc<RefCell<Node>>>,
//     constant_nodes: Vec<Rc<RefCell<Node>>>,
//     gates_per_level: Vec<LevelGates>,
//     assertions: Vec<EqualityAssertion>,
// }

// #[derive(Clone, Default, Debug)]
// pub struct Node {
//     value: Option<u64>,
//     depth: u64,
// }

// impl Node {
//     fn set_value(&mut self, value: Option<u64>) {
//         self.value = value;
//     }
// }

// #[derive(Debug)]
// pub struct AddGate {
//     left_input: Rc<RefCell<Node>>,
//     right_input: Rc<RefCell<Node>>,
//     output: Rc<RefCell<Node>>,
//     depth: u64,
// }

// #[derive(Debug)]
// pub struct MultiplyGate {
//     left_input: Rc<RefCell<Node>>,
//     right_input: Rc<RefCell<Node>>,
//     output: Rc<RefCell<Node>>,
//     depth: u64,
// }

// impl Builder {
//     pub fn new() -> Self {
//         Self {
//             input_nodes: Vec::new(),
//             constant_nodes: Vec::new(),
//             gates_per_level: Vec::new(),
//             assertions: Vec::new(),
//         }
//     }
    
//     pub fn init(&mut self) -> Rc<RefCell<Node>> {
//         let node = Rc::new(RefCell::new(Node {
//             value: None,
//             depth: 0,
//         }));
//         self.input_nodes.push(node.clone());
//         node
//     }
    
//     pub fn constant(&mut self, value: u64) -> Rc<RefCell<Node>> {
//         let node = Rc::new(RefCell::new(Node {
//             value: Some(value),
//             depth: 0,
//         }));
//         self.constant_nodes.push(node.clone());
//         node
//     }
    
//     pub fn add(&mut self, a: &Rc<RefCell<Node>>, b: &Rc<RefCell<Node>>) -> Rc<RefCell<Node>> {
//         let depth_gate = max(a.borrow().depth, b.borrow().depth);
//         let output_node = Rc::new(RefCell::new(Node {
//             value: None,
//             depth: depth_gate + 1,
//         }));
//         let add_gate = AddGate {
//             left_input: a.clone(),
//             right_input: b.clone(),
//             output: output_node.clone(),
//             depth: depth_gate,
//         };

//         if self.gates_per_level.len() > depth_gate as usize {
//             self.gates_per_level[depth_gate as usize].adder_gates.push(add_gate);
//         } else {
//             self.gates_per_level.push(LevelGates {
//                 adder_gates: Vec::new(),
//                 multiplier_gates: Vec::new(),
//             });
//             self.gates_per_level[depth_gate as usize].adder_gates.push(add_gate);
//         }
//         output_node
//     }
    
//     pub fn mul(&mut self, a: &Rc<RefCell<Node>>, b: &Rc<RefCell<Node>>) -> Rc<RefCell<Node>> {
//         let depth_gate = max(a.borrow().depth, b.borrow().depth);
//         let output_node = Rc::new(RefCell::new(Node {
//             value: None,
//             depth: depth_gate + 1,
//         }));

//         let multiply_gate = MultiplyGate {
//             left_input: a.clone(),
//             right_input: b.clone(),
//             output: output_node.clone(),
//             depth: depth_gate,
//         };

//         if self.gates_per_level.len() > depth_gate as usize {
//             self.gates_per_level[depth_gate as usize].multiplier_gates.push(multiply_gate);
//         } else {
//             self.gates_per_level.push(LevelGates {
//                 adder_gates: Vec::new(),
//                 multiplier_gates: Vec::new(),
//             });
//             self.gates_per_level[depth_gate as usize].multiplier_gates.push(multiply_gate);
//         }
//         output_node
//     }
    
//     pub fn assert_equal(&mut self, a: &Rc<RefCell<Node>>, b: &Rc<RefCell<Node>>) {
//         let assertion = EqualityAssertion {
//             left_node: a.clone(),
//             right_node: b.clone(),
//         };
//         self.assertions.push(assertion);
//     }
    
//     pub fn fill_nodes(&mut self, node_values: Vec<u64>) {
//         for i in 0..node_values.len() {
//             self.input_nodes[i].borrow_mut().value = Some(node_values[i]);
//         }

//         for level_gate in &self.gates_per_level {
//             let add_gates = &level_gate.adder_gates;
//             let multiply_gates = &level_gate.multiplier_gates; 

//             for gate in add_gates {
//                 gate.output.borrow_mut().set_value(Some(gate.left_input.borrow().value.unwrap() + gate.right_input.borrow().value.unwrap()));
//             }

//             for gate in multiply_gates {
//                 gate.output.borrow_mut().set_value(Some(gate.left_input.borrow().value.unwrap() * gate.right_input.borrow().value.unwrap()));
//             }
//         }
//     }
    
//     pub fn check_constraints(&mut self) -> bool {
//         for assertion in &self.assertions {
//             let left_value = assertion.left_node.borrow().value;
//             let right_value = assertion.right_node.borrow().value;
//             if left_value != right_value {
//                 eprint!("Equality Assertion failed!");
//                 return false;
//             }
//         }
//         true
//     }
// }
use std::{cmp::max, sync::{Arc, RwLock}};
use futures::StreamExt;
use rayon::prelude::*;

use crate::field::Field;

#[derive(Debug)]
pub struct LevelGates<F: Field> {
    adder_gates: Vec<AddGate<F>>,
    multiplier_gates: Vec<MultiplyGate<F>>,
}

#[derive(Debug)]
pub struct EqualityAssertion<F: Field> {
    left_node: Arc<RwLock<Node<F>>>,
    right_node: Arc<RwLock<Node<F>>>,
}

#[derive(Debug, Default)]
pub struct Builder<F: Field> {
    input_nodes: Vec<Arc<RwLock<Node<F>>>>,
    constant_nodes: Vec<Arc<RwLock<Node<F>>>>,
    gates_per_level: Vec<LevelGates<F>>,
    assertions: Vec<EqualityAssertion<F>>,
    internal_count: u64,
}

#[derive(Clone, Default, Debug)]
pub struct Node<F: Field> {
    value: Option<F>,
    depth: u64,
    id: u64,
}

impl<F: Field> Node<F> {
    fn set_value(&mut self, value: Option<F>) {
        self.value = value;
    }
}

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
    
    pub fn init(&mut self) -> Arc<RwLock<Node<F>>> {
        let node = Arc::new(RwLock::new(Node {
            value: None,
            depth: 0,
            id: self.internal_count,
        }));
        self.internal_count += 1; 
        self.input_nodes.push(node.clone());
        node
    }
    
    pub fn constant(&mut self, value: F) -> Arc<RwLock<Node<F>>> {
        let node = Arc::new(RwLock::new(Node {
            value: Some(value),
            depth: 0,
            id: self.internal_count,
        }));
        self.internal_count += 1; 
        self.constant_nodes.push(node.clone());
        node
    }
    
    pub fn add(&mut self, a: &Arc<RwLock<Node<F>>>, b: &Arc<RwLock<Node<F>>>) -> Arc<RwLock<Node<F>>> {
        let a_depth = a.read().unwrap().depth;
        let b_depth = b.read().unwrap().depth;

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
            });
            self.gates_per_level[depth_gate as usize].adder_gates.push(add_gate);
        }
        output_node
    }
    
    pub fn mul(&mut self, a: &Arc<RwLock<Node<F>>>, b: &Arc<RwLock<Node<F>>>) -> Arc<RwLock<Node<F>>> {
        let a_depth = a.read().unwrap().depth;
        let b_depth = b.read().unwrap().depth;

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
            });
            self.gates_per_level[depth_gate as usize].multiplier_gates.push(multiply_gate);
        }
        output_node
    }
    
    pub fn assert_equal(&mut self, a: &Arc<RwLock<Node<F>>>, b: &Arc<RwLock<Node<F>>>) {
        let assertion = EqualityAssertion {
            left_node: a.clone(),
            right_node: b.clone(),
        };
        self.assertions.push(assertion);
    }
    
    pub fn fill_nodes(&mut self, node_values: Vec<F>) {
        self.input_nodes.par_iter()
            .zip(node_values.into_par_iter())
            .for_each(|(node, value)| {
                    let mut locked_node = node.write().unwrap();
                    locked_node.value = Some(value);
            });
    
            
        for level_gate in &self.gates_per_level {
            let add_gates = &level_gate.adder_gates;
            let multiply_gates = &level_gate.multiplier_gates; 

            add_gates.par_iter().for_each(|gate| {
                let mut output = gate.output.write().unwrap();
                let left_value = gate.left_input.read().unwrap().value.unwrap();
                let right_value = gate.right_input.read().unwrap().value.unwrap();
                output.set_value(Some(left_value + right_value));
            });

            multiply_gates.par_iter().for_each(|gate| {
                let mut output = gate.output.write().unwrap();
                let left_value = gate.left_input.read().unwrap().value.unwrap();
                let right_value = gate.right_input.read().unwrap().value.unwrap();
                output.set_value(Some(left_value * right_value));
            });
        }
    }

    pub async fn check_constraints(&mut self) -> bool {
        for assertion in &self.assertions {
            let future_left_value = async {
                assertion.left_node.read().unwrap().value.unwrap()
            };
            let future_left_value = future_left_value.await;

            let future_right_value = async {
                assertion.right_node.read().unwrap().value.unwrap()
            };
            let future_right_value = future_right_value.await;
            
            if future_left_value != future_right_value {
                let left_value = assertion.left_node.read().unwrap();
                let right_value = assertion.right_node.read().unwrap();

                eprintln!("Equality failed at following nodes: {:?}, {:?}", left_value, right_value);
                return false;
            }
        }
        true
    }
}

