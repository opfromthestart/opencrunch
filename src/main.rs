mod distrs;

use std::fs::File;

use distrs::{Normal, TryContinuous, Show, ChiSquare, TDist};
use egui::{plot::Line, Ui};


#[cfg(not(target_arch = "wasm32"))]
fn main() {

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "OpenCrunch",
        native_options,
        Box::new(|_| Box::new(OpenCrunch{ distr: Distr::None, graph: vec![] })),
    );
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[cfg(not(target_arch = "wasm32"))]
fn log(s: &str) {
    println!("{}", s);
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn start(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    let web_options = eframe::WebOptions::default();
    //log(&format!("Got id {}", canvas_id));
    //let x: Option<u32> = None;
    //x.unwrap();
    eframe::start_web(canvas_id, web_options, Box::new(|_| Box::new(OpenCrunch{ distr: Distr::None, graph: vec![] }))).await?;
    Ok(())
}

#[derive(Debug)]
enum Distr {
    None, 
    Normal(Normal),
    ChiSquare(ChiSquare),
    TDist(TDist)
}

impl Distr {
    fn pdf(&self, x: f64) -> Option<f64> {
        match self {
            Distr::None => None,
            Distr::Normal(n) => n.pdf(x),
            Distr::ChiSquare(c) => c.pdf(x),
            Distr::TDist(t) => t.pdf(x),
        }
    }

    fn cdf(&self, x: f64) -> Option<f64> {
        match self {
            Distr::None => None,
            Distr::Normal(n) => n.cdf(x),
            Distr::ChiSquare(c) => c.cdf(x),
            Distr::TDist(t) => t.cdf(x),
        }
    }

    fn inverse_cdf(&self, x: f64) -> Option<f64> {
        match self {
            Distr::None => None,
            Distr::Normal(n) => n.inverse_cdf(x),
            Distr::ChiSquare(c) => c.inverse_cdf(x),
            Distr::TDist(t) => t.inverse_cdf(x),
        }
    }

    fn is_none(&self) -> bool {
        matches!(self, Distr::None)
    }

    fn show_inputs(&mut self, ui: &mut Ui) -> Option<egui::Response> {
        match self {
            Distr::None => None,
            Distr::Normal(n) => Some(n.show(ui)),
            Distr::ChiSquare(c) => Some(c.show(ui)),
            Distr::TDist(t) => Some(t.show(ui)),
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
            ui.horizontal(|ui| {
            if ui.button("Normal").clicked() {
                self.distr = Distr::Normal(Normal::default());
                self.graph = vec![];
            }
            else if ui.button("Chi Squared").clicked() {
                self.distr = Distr::ChiSquare(ChiSquare::default());
                self.graph = vec![];
            }
            else if ui.button("T Distribution").clicked() {
                self.distr = Distr::TDist(TDist::default());
                self.graph = vec![];
            }
            });
        });

        for file in &ctx.input().raw.dropped_files {
            let path = file.path.clone().unwrap();
            let len = File::open(path.clone()).unwrap().metadata().unwrap().len();
            let name = path.file_name().unwrap().to_str().unwrap();
            eprintln!("{}: {}", name, len);
        }

        let resp = egui::panel::TopBottomPanel::bottom("Interactive").show(ctx, |ui| {
            self.distr.show_inputs(ui)
        }).inner;

        if (self.graph.is_empty() || (resp.is_some() && resp.unwrap().changed())) && !self.distr.is_none() {
            //println!("{:?}", self.distr);
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