// Sous Windows, une application de bureau ne doit pas ouvrir de console derrière sa fenêtre.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    candilog_lib::run();
}
