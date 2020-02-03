pub enum ProofNumber {
    Infinity,
    Finite(u32),
}

pub enum NodeValue {
    True,
    False,
    Unknown,
}

pub enum NodeType {
    And,
    Or,
}

pub struct Node<T> {
    is_type: NodeType,
    value: NodeValue,
    proof_number: ProofNumber,
    disproof_number: ProofNumber,
    data: T,
}

impl<T> Node<T> {
    pub fn create_unknown_leaf(is_type: NodeType, data: T) -> Node<T> {
        Node {
            is_type,
            value: NodeValue::Unknown,
            proof_number: ProofNumber::Finite(1),
            disproof_number: ProofNumber::Finite(1),
            data,
        }
    }

    pub fn create_true_leaf(is_type: NodeType, data: T) -> Node<T> {
        Node {
            is_type,
            value: NodeValue::True,
            proof_number: ProofNumber::Finite(0),
            disproof_number: ProofNumber::Infinity,
            data,
        }
    }

    pub fn create_false_leaf(is_type: NodeType, data: T) -> Node<T> {
        Node {
            is_type,
            value: NodeValue::False,
            proof_number: ProofNumber::Infinity,
            disproof_number: ProofNumber::Finite(0),
            data,
        }
    }
}
