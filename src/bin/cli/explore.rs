use cursive::view::Margins;
use cursive::views::{Button, LinearLayout, PaddedView, TextView};
use cursive::Cursive;
use petgraph::visit::EdgeRef;
use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use tsumego_solver::go::GoGame;
use tsumego_solver::puzzle::{Profile, Puzzle};

fn load_puzzle(filename: &str) -> Puzzle<Profile> {
    let game = GoGame::from_sgf(&fs::read_to_string(Path::new(filename)).unwrap());

    Puzzle::new(game)
}

fn create_layer(puzzle_cell: Rc<RefCell<Puzzle<Profile>>>) -> LinearLayout {
    let puzzle = puzzle_cell.borrow();
    let edges = puzzle.tree.edges(puzzle.current_node_id);

    let up_view = PaddedView::new(
        Margins::lrtb(0, 0, 1, 2),
        Button::new("Up", {
            let puzzle_cell = puzzle_cell.clone();
            move |s| {
                if puzzle_cell.borrow_mut().move_up() {
                    s.pop_layer();
                    s.add_layer(create_layer(puzzle_cell.clone()));
                }
            }
        }),
    );

    let mut children = LinearLayout::horizontal();

    for edge in edges {
        let target_id = edge.target();
        let go_move = *edge.weight();

        let button = Button::new(format!("{}", edge.weight()), {
            let puzzle_cell = puzzle_cell.clone();
            move |s| {
                puzzle_cell.borrow_mut().move_down(target_id, go_move);
                s.pop_layer();
                s.add_layer(create_layer(puzzle_cell.clone()));
            }
        });
        children.add_child(PaddedView::lrtb(0, 2, 0, 0, button));
    }

    let node_display = PaddedView::new(
        Margins::lr(0, 2),
        TextView::new(format!(
            "{:?}\n\n{}",
            puzzle.tree[puzzle.current_node_id],
            puzzle.current_game().get_board()
        )),
    );

    let middle = PaddedView::new(
        Margins::lrtb(2, 2, 0, 2),
        LinearLayout::horizontal()
            .child(node_display)
            .child(TextView::new(puzzle.profiler.print())),
    );

    LinearLayout::vertical()
        .child(up_view)
        .child(middle)
        .child(PaddedView::new(Margins::lrtb(2, 0, 0, 1), children))
}

pub fn run(filename: &str) {
    let mut puzzle = load_puzzle(filename);

    puzzle.solve();

    let mut siv = Cursive::default();

    siv.add_layer(create_layer(Rc::new(RefCell::new(puzzle))));

    siv.run();
}
