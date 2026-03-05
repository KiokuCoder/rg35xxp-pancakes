use egui::RawInput;
use rg35xxp_egui::{Builder, HatState, SharedState};

#[derive(Default)]
struct State {
    input: RawInput,
    counter: usize,
    hat_state: HatState,
}
impl SharedState for State {
    fn input(&mut self) -> &mut RawInput {
        &mut self.input
    }

    fn hat(&mut self) -> &mut HatState {
        &mut self.hat_state
    }
}
fn main() {
    let state = State::default();
    let builder = Builder::default();
    builder.run(state, |state, ui| {
        ui.label("hello world");
        if ui.button("click me").clicked() {
            state.counter += 1;
        }
        ui.label(egui::RichText::new(format!("{:?}", state.counter)));
    })
}
