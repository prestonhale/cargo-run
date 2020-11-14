//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

extern crate cargo_run;
use cargo_run::Universe;
use cargo_run::Map;
use cargo_run::Position;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

// TODO: Are these unit or integration tests, if unit, put in the module they're testing
#[cfg(test)]
fn get_universe() -> Universe {
    let map = Map::new(5, 5);
    let universe = Universe::new(map);
    universe
}

#[wasm_bindgen_test]
fn test_tick_do_nothing() {
    let cell_locs = &[(3, 3)];

    let mut input_universe = get_universe();
    input_universe.set_cells(cell_locs);

    let mut expected_universe = get_universe();
    expected_universe.set_cells(cell_locs);

    input_universe.tick();
    assert_eq!(&input_universe.get_cells(), &expected_universe.get_cells());
}

#[wasm_bindgen_test]
fn test_shoot() {
    let mut input_universe = get_universe();
    let cell_locs = &[(3, 3)];
    input_universe.set_cells(cell_locs);
    input_universe.set_player_position(Position::new(3, 3));
    input_universe.shoot();

    let mut expected_universe = get_universe();
    let cell_locs = &[(3, 3), (3, 2)];
    expected_universe.set_cells(cell_locs);

    assert_eq!(&input_universe.get_cells(), &expected_universe.get_cells());
}
