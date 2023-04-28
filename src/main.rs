mod calcs;
mod distrs;

use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use calcs::OpenCrunchCalcs;
use distrs::OpenCrunchCDistr;
use eframe::App;
use egui::{Id, Rect, Sense, TextEdit, Ui};

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "OpenCrunch",
        native_options,
        Box::new(|_| Box::<OpenCrunch>::default()),
    )
    .unwrap();
}

use meval::Expr;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/*
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
*/

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
        Box::new(|_| Box::new(OpenCrunch::default())),
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
    calcs: OpenCrunchCalcs,
    active: Active,
}

impl App for OpenCrunch {
    fn update(&mut self, ctx: &egui::Context, f: &mut eframe::Frame) {
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
                f.set_window_title(&self.cdistr.to_string());
            }
            Active::Calcs => {
                egui::panel::CentralPanel::default().show(ctx, |ui| {
                    ui.add(&mut self.calcs);
                });
                f.set_window_title(&self.calcs.to_string());
            }
            Active::None => {}
        }
    }
}

pub(crate) fn empty_resp(ui: &Ui) -> egui::Response {
    ui.interact(Rect::everything_above(0.0), Id::new("none"), Sense::click())
}

fn coerce_expr(s: &str) -> String {
    //s.chars()
    //    .filter(|c| c.is_ascii_digit() || ".-<=>![,]".contains(*c))
    //    .collect()
    s.to_string()
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
            *(v) = coerce_expr(v);
            resp
        })
        .inner
    }
}

trait GridNumBox {
    fn grid_num_box(&mut self, width: usize, v: &mut String) -> egui::Response;
}

impl GridNumBox for Ui {
    fn grid_num_box(&mut self, width: usize, v: &mut String) -> egui::Response {
        self.horizontal(|ui| {
            let resp = ui.add_sized((width as f32, 20.), TextEdit::singleline(v));
            *(v) = coerce_expr(v);
            resp
        })
        .inner
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Constr<T> {
    GE(T),
    LE(T),
    GT(T),
    LT(T),
    GENone,
    GTNone,
    LENone,
    LTNone,
    EQ(T),
    NE(T),
    EQNone,
    NENone,
    In(T, T),
    Out(T, T),
    None,
}

impl<T: std::str::FromStr + Debug> FromStr for Constr<T> {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const INV: &str = "Not a valid input";
        let l = s.len();
        if s == ">=" {
            Ok(Self::GENone)
        } else if s == ">" {
            Ok(Self::GTNone)
        } else if s == "<" {
            Ok(Self::LTNone)
        } else if s == "<=" {
            Ok(Self::LENone)
        } else if s == "!=" {
            Ok(Self::NENone)
        } else if s == "=" {
            Ok(Self::EQNone)
        } else if l >= 2 && &s[..2] == ">=" {
            match s[2..].parse() {
                Ok(n) => Ok(Self::GE(n)),
                Err(_) => Err(INV),
            }
        } else if l >= 2 && &s[..2] == "<=" {
            match s[2..].parse() {
                Ok(n) => Ok(Self::LE(n)),
                Err(_) => Err(INV),
            }
        } else if l >= 2 && &s[..2] == "==" {
            match s[2..].parse() {
                Ok(n) => Ok(Self::EQ(n)),
                Err(_) => Err(INV),
            }
        } else if l >= 2 && &s[..2] == "!=" {
            match s[2..].parse() {
                Ok(n) => Ok(Self::NE(n)),
                Err(_) => Err(INV),
            }
        } else if l >= 1 && &s[..1] == ">" {
            match s[1..].parse() {
                Ok(n) => Ok(Self::GT(n)),
                Err(_) => Err(INV),
            }
        } else if l >= 1 && &s[..1] == "<" {
            match s[1..].parse() {
                Ok(n) => Ok(Self::LT(n)),
                Err(_) => Err(INV),
            }
        } else if l >= 1 && &s[..1] == "=" {
            match s[1..].parse() {
                Ok(n) => Ok(Self::EQ(n)),
                Err(_) => Err(INV),
            }
        } else if l >= 1 && &s[..1] == "[" {
            let Some(split) = s[1..].find(',') else {
                return Err("Comma expected in range");
            };
            let split = split + 1;
            let Some(end) = s[split..].find(']') else {
                return Err("End ] expected");
            };
            let end = split + end;
            match s[1..split].parse() {
                Ok(a) => match s[split + 1..end].parse() {
                    Ok(b) => Ok(Self::In(a, b)),
                    Err(_) => Err(INV),
                },
                Err(_) => Err(INV),
            }
        } else if l >= 1 && &s[..1] == "]" {
            let Some(split) = s[1..].find(',') else {
                return Err("Comma expected in range");
            };
            let split = split + 1;
            let Some(end) = s[split..].find('[') else {
                return Err("End [ expected");
            };
            let end = split + end;
            match s[1..split].parse() {
                Ok(a) => match s[split + 1..end].parse() {
                    Ok(b) => Ok(Self::Out(a, b)),
                    Err(_) => Err(INV),
                },
                Err(_) => Err(INV),
            }
        } else if let Ok(v) = s.parse() {
            Ok(Self::EQ(v))
        } else if l == 0 {
            Ok(Self::None)
        } else {
            Err("Not a valid constraint")
        }
    }
}

impl<T: Display> ToString for Constr<T> {
    fn to_string(&self) -> String {
        match self {
            Constr::GE(v) => format!(">={v}"),
            Constr::LE(v) => format!("<={v}"),
            Constr::GT(v) => format!(">{v}"),
            Constr::LT(v) => format!("<{v}"),
            Constr::EQ(v) => format!("=={v}"),
            Constr::NE(v) => format!("!={v}"),
            Constr::In(a, b) => format!("[{a},{b}]"),
            Constr::Out(a, b) => format!("]{a},{b}["),
            Constr::None => "".to_string(),
            Constr::GENone => ">=".to_string(),
            Constr::GTNone => ">".to_string(),
            Constr::LENone => "<=".to_string(),
            Constr::LTNone => "<".to_string(),
            Constr::EQNone => "=".to_string(),
            Constr::NENone => "!=".to_string(),
        }
    }
}

impl<T: PartialOrd + PartialEq> Constr<T> {
    fn comp(&self, arg: &T) -> bool {
        match self {
            Constr::GE(v) => arg >= v,
            Constr::LE(v) => arg <= v,
            Constr::GT(v) => arg > v,
            Constr::LT(v) => arg < v,
            Constr::EQ(v) => arg == v,
            Constr::NE(v) => arg != v,
            Constr::In(a, b) => arg >= a && arg <= b,
            Constr::Out(a, b) => arg < a || arg > b,
            Constr::None => true,
            Constr::GENone => false,
            Constr::GTNone => false,
            Constr::LENone => false,
            Constr::LTNone => false,
            Constr::EQNone => false,
            Constr::NENone => false,
        }
    }
}

impl<T> Constr<T> {
    fn as_val(&self) -> Option<&T> {
        match self {
            Constr::EQ(n) | Constr::NE(n) => Some(n),
            _ => None,
        }
    }

    fn is_ineq(&self) -> bool {
        matches!(
            self,
            Constr::GE(_)
                | Constr::LE(_)
                | Constr::GT(_)
                | Constr::LT(_)
                | Constr::GENone
                | Constr::LENone
                | Constr::GTNone
                | Constr::LTNone
                | Constr::In(_, _)
                | Constr::Out(_, _)
        )
    }

    fn is_eq(&self) -> bool {
        matches!(
            self,
            Constr::EQ(_) | Constr::NE(_) | Constr::EQNone | Constr::NENone
        )
    }

    fn is_range(&self) -> bool {
        matches!(self, Constr::In(_, _) | Constr::Out(_, _))
    }

    fn is_some(&self) -> bool {
        !matches!(
            self,
            Constr::None
                | Constr::GENone
                | Constr::GTNone
                | Constr::LENone
                | Constr::LTNone
                | Constr::EQNone
                | Constr::NENone
        )
    }
}

impl Constr<Expr> {
    fn eval(&self) -> Result<Constr<f64>, meval::Error> {
        match self {
            Constr::GE(x) => Ok(Constr::GE(x.eval()?)),
            Constr::LE(x) => Ok(Constr::LE(x.eval()?)),
            Constr::GT(x) => Ok(Constr::GT(x.eval()?)),
            Constr::LT(x) => Ok(Constr::LT(x.eval()?)),
            Constr::GENone => Ok(Constr::GENone),
            Constr::GTNone => Ok(Constr::GTNone),
            Constr::LENone => Ok(Constr::LENone),
            Constr::LTNone => Ok(Constr::LTNone),
            Constr::EQ(x) => Ok(Constr::EQ(x.eval()?)),
            Constr::NE(x) => Ok(Constr::NE(x.eval()?)),
            Constr::In(a, b) => Ok(Constr::In(a.eval()?, b.eval()?)),
            Constr::Out(a, b) => Ok(Constr::Out(a.eval()?, b.eval()?)),
            Constr::None => Ok(Constr::None),
            Constr::EQNone => Ok(Constr::EQNone),
            Constr::NENone => Ok(Constr::NENone),
        }
    }
}
