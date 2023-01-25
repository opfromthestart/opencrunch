mod distrs;
mod calcs;

use std::str::FromStr;

use calcs::OpenCrunchSample;
use distrs::OpenCrunchCDistr;
use eframe::App;
use egui::{Ui, Rect, Id, Sense};


#[cfg(not(target_arch = "wasm32"))]
fn main() {

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "OpenCrunch",
        native_options,
        Box::new(|_| Box::new(OpenCrunch::default())),
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
    eframe::start_web(canvas_id, web_options, Box::new(|_| Box::new(OpenCrunchCDistr::default()))).await?;
    Ok(())
}

#[derive(Default)]
enum Active {
    CDistr,
    Sample,
    #[default]
    None,
}

#[derive(Default)]
struct OpenCrunch {
    cdistr: OpenCrunchCDistr,
    sample: OpenCrunchSample,
    active: Active,
}

impl App for OpenCrunch {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::panel::TopBottomPanel::top("Tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Distributions").clicked() {
                    self.active = Active::CDistr;
                }
                if ui.button("Calculations").clicked() {
                    self.active = Active::Sample;
                }
            });
        });

        match self.active {
            Active::CDistr => {
                egui::panel::CentralPanel::default().show(ctx, |ui| {
                    ui.add(&mut self.cdistr);
                });
            },
            Active::Sample => {
                egui::panel::CentralPanel::default().show(ctx, |ui| {
                    ui.add(&mut self.sample);
                });
            },
            Active::None => {},
        }
    }
}

pub(crate) fn empty_resp(ui: &Ui) -> egui::Response {
    ui.interact(Rect::everything_above(0.0), Id::new("none"), Sense::click())
}

fn coerce_numeric(s: &String) -> String {
    s.chars().filter(|c| c.is_ascii_digit() || *c=='.' || *c == '-').collect()
}

trait NumBox {
    fn num_box(&mut self, l: &str, v: &mut String) -> egui::Response;
}

impl NumBox for Ui {
    fn num_box(&mut self, l: &str, v: &mut String) -> egui::Response {
        self.horizontal(|ui| {
            ui.label(l);
            /*
            let resp = if v.is_empty() || v.parse::<f64>().is_ok() {
                ui.text_edit_singleline(v)
            }
            else {
                let mut resp = ui.text_edit_singleline(v);
                resp = resp.union(ui.label(RichText::new("Invalid").color(Color32::DARK_RED)));
                resp
            };
            */
            let resp = ui.text_edit_singleline(v);
            *(v) = coerce_numeric(v);
            resp
        }).inner
    }
}

#[derive(Clone, Debug)]
pub(crate) enum Comp {
    GE,
    LE,
    GT,
    LT,
    EQ,
    NE,
}

impl FromStr for Comp {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ">=" => Ok(Self::GE),
            "<=" => Ok(Self::LE),
            ">" => Ok(Self::GT),
            "<" => Ok(Self::LT),
            "=" | "==" => Ok(Self::EQ),
            "!=" => Ok(Self::NE),
            _ => Err("Not a valid comparison"),
        }
    }
}

impl ToString for Comp {
    fn to_string(&self) -> String {
        match self {
            Comp::GE => ">=".to_owned(),
            Comp::LE => "<=".to_owned(),
            Comp::GT => ">".to_owned(),
            Comp::LT => "<".to_owned(),
            Comp::EQ => "=".to_owned(),
            Comp::NE => "!=".to_owned(),
        }
    }
}