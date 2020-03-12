use clap::{App, Arg};
use cursive::view::Margins;
use cursive::views::{Button, LinearLayout, PaddedView, TextView};
use cursive::Cursive;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use tsumego_solver::go::GoGame;
use tsumego_solver::puzzle::Puzzle;

fn load_puzzle() -> Puzzle {
    let matches = App::new("Tsumego Solver Debugger")
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    let filename = matches.value_of("file").unwrap();

    let game = GoGame::from_sgf(&fs::read_to_string(Path::new(filename)).unwrap());

    Puzzle::new(game)
}

fn create_layer(puzzle: Rc<Puzzle>, node_id: NodeIndex) -> LinearLayout {
    let edges = puzzle.tree.edges(node_id);
    let parent_id = puzzle
        .tree
        .neighbors_directed(node_id, Direction::Incoming)
        .next();

    let up_view = PaddedView::new(
        Margins::lrtb(0, 0, 0, 2),
        Button::new("Up", {
            let puzzle = puzzle.clone();
            move |s| {
                if let Some(parent_id) = parent_id {
                    s.pop_layer();
                    s.add_layer(create_layer(puzzle.clone(), parent_id));
                }
            }
        }),
    );

    let mut children = LinearLayout::horizontal();

    for edge in edges {
        let target_id = edge.target();

        let button = Button::new(format!("{}", edge.weight()), {
            let puzzle = puzzle.clone();
            move |s| {
                s.pop_layer();
                s.add_layer(create_layer(puzzle.clone(), target_id));
            }
        });
        children.add_child(PaddedView::lrtb(0, 2, 0, 0, button));
    }

    let node_display = PaddedView::new(
        Margins::lrtb(0, 0, 0, 2),
        TextView::new(format!("{:?}", puzzle.tree[node_id])),
    );

    LinearLayout::vertical()
        .child(up_view)
        .child(node_display)
        .child(children)
}

fn main() {
    let mut puzzle = load_puzzle();

    puzzle.solve();

    let mut siv = Cursive::default();

    let root_id = puzzle.root_id;

    siv.add_layer(create_layer(Rc::new(puzzle), root_id));

    siv.run();
}
