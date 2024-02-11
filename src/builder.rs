use std::{cmp::max, sync::{Arc, RwLock}};
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
    pub value: Option<F>,
    pub depth: u64,
    pub id: u64,
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

    pub fn batch_init(&mut self, num_inputs: u64) -> Vec<Arc<RwLock<Node<F>>>> {
        let init_count = self.internal_count; 
        let vector_input: Vec<Arc<RwLock<Node<F>>>> = (0..num_inputs).into_par_iter().map(|i| {
            Arc::new(RwLock::new(Node {
                value: None,
                depth: 0,
                id: init_count + i,
            }))}).collect();
        self.input_nodes.extend(vector_input.clone());
        self.internal_count += num_inputs;
        vector_input
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

    pub fn batch_constant(&mut self, values: &[F]) -> Vec<Arc<RwLock<Node<F>>>> {
        let init_count = self.internal_count; 
        let vector_constant: Vec<Arc<RwLock<Node<F>>>> = (0..values.len()).into_par_iter().map(|i| {
            Arc::new(RwLock::new(Node {
                value: Some(values[i]),
                depth: 0,
                id: init_count + i as u64,
            }))}).collect();
        self.constant_nodes.extend(vector_constant.clone());
        self.internal_count += values.len() as u64;
        vector_constant
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
        assert_eq!(node_values.len(), self.input_nodes.len());
        if node_values.len() != self.input_nodes.len() {
            panic!("Not enough inputs provided: expected {}, got {}", self.input_nodes.len(), node_values.len());
        }

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

