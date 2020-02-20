mod go;
use go::{BoardCell, GoGame};
mod puzzle;
// use pn_search::{AndOrNode, NodeType};

// fn generate_children(node: AndOrNode<GoGame>) -> Vec<AndOrNode<GoGame>> {
//     node.data
//         .generate_moves()
//         .iter()
//         .map(|new_state| AndOrNode {})
//         .collect()
// }

fn main() {
    let tsumego = GoGame::empty();

    // let mut tree = TreeBuilder::new()
    //     .with_root(Node::new(AndOrNode::create_unknown_leaf(    tsumego )))
    //     .build();

    // let arena = &mut Arena::<Node<BoardState>>::new();
    // let root = arena.new_node(Node::create_unknown_leaf(
    //     NodeType::And,
    //     tsumego,
    // ));
}
