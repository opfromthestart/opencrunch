mod calcs;
mod distrs;

use std::{str::FromStr, fmt::Display};

use calcs::OpenCrunchSample;
use distrs::OpenCrunchCDistr;
use eframe::App;
use egui::{Id, Rect, Sense, Ui};

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
    eframe::start_web(
        canvas_id,
        web_options,
        Box::new(|_| Box::new(OpenCrunchCDistr::default())),
    )
    .await?;
    Ok(())
}

#[derive(Default)]
enum Active {
    CDistr,
    Calcs,
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
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::panel::TopBottomPanel::top("Tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Distributions").clicked() {
                    self.active = Active::CDistr;
                }
                if ui.button("Calculations").clicked() {
                    self.active = Active::Calcs;
                }
            });
        });

        match self.active {
            Active::CDistr => {
                egui::panel::CentralPanel::default().show(ctx, |ui| {
                    ui.add(&mut self.cdistr);
                });
            }
            Active::Calcs => {
                egui::panel::CentralPanel::default().show(ctx, |ui| {
                    ui.add(&mut self.sample);
                });
            }
            Active::None => {}
        }
    }
}

pub(crate) fn empty_resp(ui: &Ui) -> egui::Response {
    ui.interact(Rect::everything_above(0.0), Id::new("none"), Sense::click())
}

fn coerce_numeric(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect()
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
        })
        .inner
    }
}

#[derive(Clone, Debug)]
pub(crate) enum Constr<T: Display + PartialOrd + PartialEq> {
    GE(T),
    LE(T),
    GT(T),
    LT(T),
    EQ(T),
    NE(T),
    IN(T,T),
    OUT(T,T),
    None,
}

impl<T: Display + PartialOrd + PartialEq + std::str::FromStr> FromStr for Constr<T> {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const inv: &'static str = "Not a valid input";
        if &s[..2] == ">=" {
            match s[2..].parse() {
                Ok(n) => Ok(Self::GE(n)),
                Err(e) => Err(inv),
            }
        }
        else if &s[..2] == "<=" {
            match s[2..].parse() {
                Ok(n) => Ok(Self::LE(n)),
                Err(e) => Err(inv),
            }
        }
        else if &s[..2] == "==" {
            match s[2..].parse() {
                Ok(n) => Ok(Self::EQ(n)),
                Err(e) => Err(inv),
            }
        }
        else if &s[..2] == "!=" {
            match s[2..].parse() {
                Ok(n) => Ok(Self::NE(n)),
                Err(e) => Err(inv),
            }
        }
        else if &s[..1] == ">" {
            match s[1..].parse() {
                Ok(n) => Ok(Self::GT(n)),
                Err(e) => Err(inv),
            }
        }
        else if &s[..1] == "<" {
            match s[1..].parse() {
                Ok(n) => Ok(Self::LT(n)),
                Err(e) => Err(inv),
            }
        }
        else if &s[..1] == "=" {
            match s[1..].parse() {
                Ok(n) => Ok(Self::EQ(n)),
                Err(e) => Err(inv),
            }
        }
        else if &s[..1] == "[" {
            let Some(split) = s[1..].find(',') else {
                return Err("Comma expected in range");
            };
            let Some(end) = s[split..].find(']') else {
                return Err("End ] expected");
            };
            match s[1..split].parse() {
                Ok(a) => {
                    match s[split+1..end].parse() {
                        Ok(b) => Ok(Self::IN(a, b)),
                        Err(_) => Err(inv),
                    }
                },
                Err(_) => Err(inv),
            }
        }
        else {
            Err("Not a valid constraint")
        }
    }
}

impl<T: Display + PartialOrd + PartialEq> ToString for Constr<T> {
    fn to_string(&self) -> String {
        match self {
            Constr::GE(v) => format!(">={v}"),
            Constr::LE(v) => format!("<={v}"),
            Constr::GT(v) => format!(">{v}"),
            Constr::LT(v) => format!("<{v}"),
            Constr::EQ(v) => format!("=={v}"),
            Constr::NE(v) => format!("!={v}"),
            Constr::IN(a, b) => format!("[{a},{b}]"),
            Constr::OUT(a, b) => format!("]{a},{b}["),
            Constr::None => format!(""),
        }
    }
}

impl<T: Display + PartialOrd + PartialEq> Constr<T> {
    fn comp(&self, arg: &T) -> bool {
        match self {
            Constr::GE(v) => arg>=v,
            Constr::LE(v) => arg<=v,
            Constr::GT(v) => arg>v,
            Constr::LT(v) => arg<v,
            Constr::EQ(v) => arg==v,
            Constr::NE(v) => arg!=v,
            Constr::IN(a, b) => arg>=a && arg<=b,
            Constr::OUT(a, b) => arg<a || arg>b,
            Constr::None => true,
        }
    }
}

trait UseComp<T: Display + PartialOrd + PartialEq> {
    fn apply(&self, comp: Constr<T>) -> Result<T, String>;
}