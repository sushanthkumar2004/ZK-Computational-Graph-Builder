use std::{cmp::max, cell::RefCell, rc::Rc};

use crate::field::Field;

#[derive(Debug)]
pub struct LevelGates<F: Field> {
    adder_gates: Vec<AddGate<F>>,
    multiplier_gates: Vec<MultiplyGate<F>>,
}

#[derive(Debug, Clone)]
pub struct EqualityAssertion<F: Field> {
    left_node: Rc<RefCell<Node<F>>>,
    right_node: Rc<RefCell<Node<F>>>,
}

#[derive(Debug)]
pub struct BuilderSingleThread<F: Field> {
    input_nodes: Vec<Rc<RefCell<Node<F>>>>,
    constant_nodes: Vec<Rc<RefCell<Node<F>>>>,
    gates_per_level: Vec<LevelGates<F>>,
    assertions: Vec<EqualityAssertion<F>>,
}

#[derive(Clone, Default, Debug)]
pub struct Node<F: Field> {
    value: Option<F>,
    depth: u64,
}

impl<F: Field> Node<F> {
    fn set_value(&mut self, value: Option<F>) {
        self.value = value;
    }
}

#[derive(Debug)]
pub struct AddGate<F: Field> {
    left_input: Rc<RefCell<Node<F>>>,
    right_input: Rc<RefCell<Node<F>>>,
    output: Rc<RefCell<Node<F>>>,
}

#[derive(Debug)]
pub struct MultiplyGate<F: Field> {
    left_input: Rc<RefCell<Node<F>>>,
    right_input: Rc<RefCell<Node<F>>>,
    output: Rc<RefCell<Node<F>>>,
}

impl<F: Field> Default for BuilderSingleThread<F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F: Field> BuilderSingleThread<F> {
    pub fn new() -> Self {
        Self {
            input_nodes: Vec::new(),
            constant_nodes: Vec::new(),
            gates_per_level: Vec::new(),
            assertions: Vec::new(),
        }
    }
    
    pub fn init(&mut self) -> Rc<RefCell<Node<F>>> {
        let node = Rc::new(RefCell::new(Node {
            value: None,
            depth: 0,
        }));
        self.input_nodes.push(node.clone());
        node
    }
    
    pub fn constant(&mut self, value: F) -> Rc<RefCell<Node<F>>> {
        let node = Rc::new(RefCell::new(Node {
            value: Some(value),
            depth: 0,
        }));
        self.constant_nodes.push(node.clone());
        node
    }
    
    pub fn add(&mut self, a: &Rc<RefCell<Node<F>>>, b: &Rc<RefCell<Node<F>>>) -> Rc<RefCell<Node<F>>> {
        let depth_gate = max(a.borrow().depth, b.borrow().depth);
        let output_node = Rc::new(RefCell::new(Node {
            value: None,
            depth: depth_gate + 1,
        }));
        let add_gate = AddGate {
            left_input: a.clone(),
            right_input: b.clone(),
            output: output_node.clone(),
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
    
    pub fn mul(&mut self, a: &Rc<RefCell<Node<F>>>, b: &Rc<RefCell<Node<F>>>) -> Rc<RefCell<Node<F>>> {
        let depth_gate = max(a.borrow().depth, b.borrow().depth);
        let output_node = Rc::new(RefCell::new(Node {
            value: None,
            depth: depth_gate + 1,
        }));

        let multiply_gate = MultiplyGate {
            left_input: a.clone(),
            right_input: b.clone(),
            output: output_node.clone(),
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
    
    pub fn assert_equal(&mut self, a: &Rc<RefCell<Node<F>>>, b: &Rc<RefCell<Node<F>>>) -> EqualityAssertion<F> {
        let assertion = EqualityAssertion {
            left_node: a.clone(),
            right_node: b.clone(),
        };
        self.assertions.push(assertion.clone());
        assertion
    }
    
    pub fn fill_nodes(&mut self, node_values: Vec<F>) {
        for i in 0..node_values.len() {
            self.input_nodes[i].borrow_mut().value = Some(node_values[i]);
        }

        for level_gate in &self.gates_per_level {
            let add_gates = &level_gate.adder_gates;
            let multiply_gates = &level_gate.multiplier_gates; 

            for gate in add_gates {
                gate.output.borrow_mut().set_value(Some(gate.left_input.borrow().value.unwrap() + gate.right_input.borrow().value.unwrap()));
            }

            for gate in multiply_gates {
                gate.output.borrow_mut().set_value(Some(gate.left_input.borrow().value.unwrap() * gate.right_input.borrow().value.unwrap()));
            }
        }
    }
    
    pub async fn check_constraints(&mut self) -> bool {
        for assertion in &self.assertions {
            let future_left_value = async {
                assertion.left_node.borrow().value.unwrap()
            }.await;

            let future_right_value = async {
                assertion.right_node.borrow().value.unwrap()
            }.await;
            
            if future_left_value != future_right_value {
                let left_value = assertion.left_node.borrow();
                let right_value = assertion.right_node.borrow();

                eprintln!("Equality failed at following nodes: {:?}, {:?}", left_value, right_value);
                return false;
            }
        }
        true
    }
}