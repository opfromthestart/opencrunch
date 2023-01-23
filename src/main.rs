mod distrs;

use distrs::{Normal, TryContinuous, Show};
use egui::{plot::Line, Ui};

// When compiling natively:
fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    //tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "OpenCrunch",
        native_options,
        Box::new(|_| Box::new(OpenCrunch{ distr: Distr::None, graph: vec![] })),
    );
}

enum Distr {
    None, 
    Normal(Normal),
}

impl Distr {
    fn pdf(&self, x: f64) -> Option<f64> {
        match self {
            Distr::None => None,
            Distr::Normal(n) => n.pdf(x),
        }
    }

    fn cdf(&self, x: f64) -> Option<f64> {
        match self {
            Distr::None => None,
            Distr::Normal(n) => n.cdf(x),
        }
    }

    fn inverse_cdf(&self, x: f64) -> Option<f64> {
        match self {
            Distr::None => None,
            Distr::Normal(n) => n.inverse_cdf(x),
        }
    }

    fn is_none(&self) -> bool {
        matches!(self, Distr::None)
    }

    fn show_inputs(&mut self, ui: &mut Ui) -> Option<egui::Response> {
        match self {
            Distr::None => None,
            Distr::Normal(n) => Some(n.show(ui)),
        }
    }
}

struct OpenCrunch{
    distr: Distr,
    graph: Vec<[f64;2]>,
}

impl Default for OpenCrunch {
    fn default() -> Self {
        Self { distr: Distr::None, graph: vec![] }
    }
}

impl eframe::App for OpenCrunch {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

        egui::panel::TopBottomPanel::top("Distribution").show(ctx, |ui| {
            if ui.button("Normal").clicked() {
                self.distr = Distr::Normal(Normal::default());
                self.graph = vec![];
            }
        });

        let resp = egui::panel::TopBottomPanel::bottom("Interactive").show(ctx, |ui| {
            self.distr.show_inputs(ui)
        }).inner;

        if (self.graph.is_empty() || (resp.is_some() && resp.unwrap().changed())) && !self.distr.is_none() {
            if let Some(bottom) = self.distr.inverse_cdf(0.0001) {
                if let Some(top) = self.distr.inverse_cdf(0.9999) {
                    let points = (0..=300).into_iter()
                        .map(|x| (bottom + (top-bottom)*((x as f64)/300.)))
                        .map(|x| [x, self.distr.pdf(x).expect("distribution is not none")]).collect();
                    self.graph = points;
                }
                else {
                    self.graph = vec![];
                }
            }
            else {
                self.graph = vec![];
            }
        }
        let line = Line::new(self.graph.clone());
        egui::panel::CentralPanel::default().show(ctx, |ui| {
            eframe::egui::widgets::plot::Plot::new("Main").show(ui, |ui| {
                ui.line(line);
            });
        });
    }
}