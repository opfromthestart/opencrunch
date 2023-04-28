use std::fs::File;

use egui::{
    plot::{Line, Polygon},
    Color32, RichText, Ui, Widget,
};
use meval::Expr;
use opencrunch_derive::crunch_fill_eval;
use statrs::distribution::{Continuous, ContinuousCDF};

use crate::{empty_resp, Constr, NumBox};

trait TryContinuous {
    fn pdf(&self, x: f64) -> Option<f64>;
    fn cdf(&self, x: f64) -> Option<f64>;
    fn inverse_cdf(&self, x: f64) -> Option<f64>;
}

trait Fillable {
    fn fill(&mut self) -> Result<(), &str>;
}

trait Graph: TryContinuous {
    fn get_height(&self, pos: f64) -> Option<f64>;
    fn start(&self) -> f64;
    fn end(&self) -> f64;

    fn get_gap(&self) -> f64 {
        (self.end() - self.start()) / (self.get_terms() as f64)
    }

    fn get_terms(&self) -> usize {
        480
    }

    /// Gets a printable line
    fn get_line(&self) -> Vec<[f64; 2]> {
        let s = self.start();
        let gap = self.get_gap();
        (0..self.get_terms())
            .map(|x| s + (x as f64) * gap)
            .map(|x| (x, self.pdf(x)))
            .filter(|(_, v)| v.is_some())
            .map(|(x, v)| [x, v.unwrap()])
            .collect()
    }

    fn is_selected(&self, pos: f64) -> bool;

    /// Like get_line but filled in
    fn get_fill(&self) -> Vec<[[f64; 2]; 4]> {
        let top: Vec<_> = self
            .get_line()
            .into_iter()
            .filter(|[x, _]| self.is_selected(*x))
            .collect();
        let gap = self.get_gap();
        if !top.is_empty() {
            top.windows(2)
                .filter_map(|l| {
                    let s = l[0][0];
                    let e = l[1][0];
                    if e - s > gap * 1.0001 {
                        None
                    } else {
                        Some([[s, 0.0], l[0], l[1], [e, 0.0]])
                    }
                })
                .collect()
        } else {
            vec![]
        }
    }
}

fn find_zero(f: impl Fn(f64) -> Option<f64>) -> Option<f64> {
    let mut high = 1.0;
    let mut low = -1.0;
    if f(high).is_none() && f(low).is_none() {
        return None;
    }
    let mut count = 0;
    while f(high).is_none() {
        high = 0.9 * high + 0.1 * low;
        count += 1;
        if count > 100 {
            //eprintln!("No high");
            return None;
        }
    }
    count = 0;
    while f(low).is_none() {
        low = 0.9 * low + 0.1 * high;
        count += 1;
        if count > 100 {
            //eprintln!("No low");
            return None;
        }
    }
    let f = |x| f(x).expect("Both set to not none");
    if f(high).signum() == f(low).signum() {
        loop {
            high = high + high;
            if f(high).signum() != f(low).signum() {
                break;
            }
            low = low + low;
            if f(high).signum() != f(low).signum() {
                break;
            }
            if high.is_infinite() || low.is_infinite() {
                return None;
            }
            //eprintln!("{}, {}", high, low);
        }
    }
    if f(high).signum() == -1.0 {
        (high, low) = (low, high);
    }
    let mut middle = (high + low) / 2.0;
    while (high - low) / middle > 0.00000001 {
        let fv = f(middle);
        if fv > 0.0 {
            high = middle;
        } else if fv < 0.0 {
            low = middle;
        } else {
            break;
        }
        middle = (high + low) / 2.0;
        //eprintln!("f={}, hl={}", fv, high);
    }
    Some((high + low) / 2.0)
}

#[derive(Clone)]
pub(crate) struct OpenCrunchCDistr {
    distr: CDistr,
    graph: Vec<[f64; 2]>,
    fill: Vec<[[f64; 2]; 4]>,
}

impl Default for OpenCrunchCDistr {
    fn default() -> Self {
        Self {
            distr: CDistr::None,
            graph: vec![],
            fill: vec![],
        }
    }
}

impl Widget for &mut OpenCrunchCDistr {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let ctx = ui.ctx();

        egui::panel::TopBottomPanel::top("Distribution").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Normal").clicked() {
                    self.distr = CDistr::Normal(Normal::default());
                    self.graph = vec![];
                }
                if ui.button("Chi Squared").clicked() {
                    self.distr = CDistr::ChiSquare(ChiSquare::default());
                    self.graph = vec![];
                }
                if ui.button("T Distribution").clicked() {
                    self.distr = CDistr::TDist(TDist::default());
                    self.graph = vec![];
                }
                if ui.button("F Distribution").clicked() {
                    self.distr = CDistr::FDist(FDist::default());
                    self.graph = vec![];
                }
                if ui.button("Exponential").clicked() {
                    self.distr = CDistr::Exp(Expon::default());
                    self.graph = vec![];
                }
            });
        });

        for file in ctx.input(|i| i.raw.dropped_files.clone()) {
            let path = file.path.clone().unwrap();
            let len = File::open(path.clone()).unwrap().metadata().unwrap().len();
            let name = path.file_name().unwrap().to_str().unwrap();
            eprintln!("{}: {}", name, len);
        }

        let resp = egui::panel::TopBottomPanel::bottom("Interactive")
            .show(ctx, |ui| ui.add(&mut self.distr))
            .inner;

        if (self.graph.is_empty() || resp.changed()) && !self.distr.is_none() {
            self.graph = self.distr.get_line();
            self.fill = self.distr.get_fill();
        }
        let line = Line::new(self.graph.clone());
        let polys: Vec<_> = self
            .fill
            .iter()
            .map(|x| Polygon::new(x.to_vec()).color(Color32::RED).fill_alpha(1.0))
            .collect();
        egui::panel::CentralPanel::default()
            .show(ctx, |ui| {
                eframe::egui::widgets::plot::Plot::new("Main").show(ui, |ui| {
                    ui.line(line);
                    for p in polys {
                        ui.polygon(p);
                    }
                });
            })
            .response
    }
}

impl ToString for OpenCrunchCDistr {
    fn to_string(&self) -> String {
        match self.distr {
            CDistr::None => "OpenCrunch - Distributions".to_owned(),
            CDistr::Normal(_) => "OpenCrunch - Distributions - Normal".to_owned(),
            CDistr::ChiSquare(_) => "OpenCrunch - Distributions - Chi Square".to_owned(),
            CDistr::TDist(_) => "OpenCrunch - Distributions - T".to_owned(),
            CDistr::FDist(_) => "OpenCrunch - Distributions - F".to_owned(),
            CDistr::Exp(_) => "OpenCrunch - Distributions - Exponential".to_owned(),
        }
    }
}

#[derive(Debug, Clone)]
enum CDistr {
    None,
    Normal(Normal),
    ChiSquare(ChiSquare),
    TDist(TDist),
    FDist(FDist),
    Exp(Expon),
}

impl TryContinuous for CDistr {
    fn pdf(&self, x: f64) -> Option<f64> {
        match self {
            CDistr::None => None,
            CDistr::Normal(n) => n.pdf(x),
            CDistr::ChiSquare(c) => c.pdf(x),
            CDistr::TDist(t) => t.pdf(x),
            CDistr::Exp(e) => e.pdf(x),
            CDistr::FDist(f) => f.pdf(x),
        }
    }

    fn cdf(&self, x: f64) -> Option<f64> {
        match self {
            CDistr::None => None,
            CDistr::Normal(n) => n.cdf(x),
            CDistr::ChiSquare(c) => c.cdf(x),
            CDistr::TDist(t) => t.cdf(x),
            CDistr::Exp(e) => e.cdf(x),
            CDistr::FDist(f) => f.cdf(x),
        }
    }

    fn inverse_cdf(&self, x: f64) -> Option<f64> {
        match self {
            CDistr::None => None,
            CDistr::Normal(n) => n.inverse_cdf(x),
            CDistr::ChiSquare(c) => c.inverse_cdf(x),
            CDistr::TDist(t) => t.inverse_cdf(x),
            CDistr::Exp(e) => e.inverse_cdf(x),
            CDistr::FDist(f) => f.inverse_cdf(x),
        }
    }
}

impl Graph for CDistr {
    fn get_height(&self, pos: f64) -> Option<f64> {
        match self {
            CDistr::None => None,
            CDistr::Normal(n) => n.pdf(pos),
            CDistr::ChiSquare(c) => c.pdf(pos),
            CDistr::TDist(t) => t.pdf(pos),
            CDistr::Exp(e) => e.pdf(pos),
            CDistr::FDist(f) => f.pdf(pos),
        }
    }

    fn start(&self) -> f64 {
        match self {
            CDistr::None => 0.0,
            CDistr::Normal(n) => n.start(),
            CDistr::ChiSquare(c) => c.start(),
            CDistr::TDist(t) => t.start(),
            CDistr::Exp(e) => e.start(),
            CDistr::FDist(f) => f.start(),
        }
    }

    fn end(&self) -> f64 {
        match self {
            CDistr::None => 0.0,
            CDistr::Normal(n) => n.end(),
            CDistr::ChiSquare(c) => c.end(),
            CDistr::TDist(t) => t.end(),
            CDistr::Exp(e) => e.end(),
            CDistr::FDist(f) => f.end(),
        }
    }

    fn is_selected(&self, pos: f64) -> bool {
        match self {
            CDistr::None => false,
            CDistr::Normal(n) => n.is_selected(pos),
            CDistr::ChiSquare(c) => c.is_selected(pos),
            CDistr::TDist(t) => t.is_selected(pos),
            CDistr::Exp(e) => e.is_selected(pos),
            CDistr::FDist(f) => f.is_selected(pos),
        }
    }
}

impl CDistr {
    fn is_none(&self) -> bool {
        matches!(self, CDistr::None)
    }
}

impl Widget for &mut CDistr {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        match self {
            CDistr::None => empty_resp(ui),
            CDistr::Normal(n) => n.ui(ui),
            CDistr::ChiSquare(c) => c.ui(ui),
            CDistr::TDist(t) => t.ui(ui),
            CDistr::Exp(e) => e.ui(ui),
            CDistr::FDist(f) => f.ui(ui),
        }
    }
}

#[crunch_fill_eval]
#[derive(Debug, Clone)]
struct Normal {
    mean: Constr<f64>,
    sd: Constr<f64>,
    xval: Constr<f64>,
    pval: Constr<f64>,
}

impl Default for Normal {
    fn default() -> Self {
        Self {
            strings: [
                String::from("0.0"),
                String::from("1.0"),
                String::from("<0.0"),
                "".to_string(),
                "".to_string(),
            ],
            mean: Constr::EQ(0.0),
            sd: Constr::EQ(1.0),
            xval: Constr::LT(0.0),
            pval: Constr::EQ(0.5),
        }
    }
}

impl TryContinuous for Normal {
    fn pdf(&self, x: f64) -> Option<f64> {
        Some(
            statrs::distribution::Normal::new(*self.mean.as_val()?, *self.sd.as_val()?)
                .ok()?
                .pdf(x),
        )
    }

    fn cdf(&self, x: f64) -> Option<f64> {
        Some(
            statrs::distribution::Normal::new(*self.mean.as_val()?, *self.sd.as_val()?)
                .ok()?
                .cdf(x),
        )
    }

    fn inverse_cdf(&self, x: f64) -> Option<f64> {
        Some(
            statrs::distribution::Normal::new(*self.mean.as_val()?, *self.sd.as_val()?)
                .ok()?
                .inverse_cdf(x),
        )
    }
}

impl Graph for Normal {
    fn get_height(&self, pos: f64) -> Option<f64> {
        self.pdf(pos)
    }

    fn start(&self) -> f64 {
        self.mean.as_val().unwrap_or(&0.0) - 3.0 * self.sd.as_val().unwrap_or(&0.0)
    }

    fn end(&self) -> f64 {
        self.mean.as_val().cloned().unwrap_or(0.0) + 3.0 * self.sd.as_val().cloned().unwrap_or(0.0)
    }

    fn is_selected(&self, pos: f64) -> bool {
        self.xval.comp(&pos)
    }
}

impl Widget for &mut Normal {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let mut resp = ui.num_box("mean", &mut self.strings[0]);
        resp = resp.union(ui.num_box("std dev", &mut self.strings[1]));
        let x = ui.num_box("x value", &mut self.strings[2]);
        resp = resp.union(x);
        resp = resp.union(ui.num_box("p value", &mut self.strings[3]));
        if resp.changed() {
            self.vfill();
        }
        if ui.button("Calculate").clicked() {
            resp.mark_changed();
            if let Err(s) = self.fill() {
                self.strings[4] = s.to_owned();
            } else {
                self.strings[4] = "".to_owned();
            }
        }
        resp = resp
            .union(ui.label(RichText::new(&self.strings[4]).background_color(Color32::DARK_RED)));
        resp
    }
}

impl Fillable for Normal {
    fn fill(&mut self) -> Result<(), &str> {
        let filled = [self.mean, self.sd, self.xval, self.pval]
            .iter()
            .filter(|x| x.is_some())
            .count();
        match filled {
            0..=2 => Err("Not enough filled"),
            3 => {
                if !self.xval.is_some() {
                    let Constr::EQ(p) = self.pval else {
                        return Err("Probability must be set");
                    };
                    let fill = match self.xval {
                        Constr::GENone | Constr::GTNone => match self.inverse_cdf(1.0 - p) {
                            Some(n) => Constr::GE(n),
                            None => return Err("Not a valid probability."),
                        },
                        Constr::LENone | Constr::LTNone => match self.inverse_cdf(p) {
                            Some(n) => Constr::LE(n),
                            None => return Err("Not a valid probability."),
                        },
                        eq if eq.is_eq() => {
                            return Err("Cannot use exact in a continuous distribution.");
                        }
                        _ => return Err("Cannot use ranges for solving for x values."),
                    };
                    self.xval = fill;
                    self.strings[2] = fill.to_string();
                } else if !self.pval.is_some() {
                    let fill = match self.xval {
                        Constr::GE(x) | Constr::GT(x) => {
                            1.0 - self.cdf(x).ok_or("Invalid x value")?
                        }
                        Constr::LE(x) | Constr::LT(x) => self.cdf(x).ok_or("Invalid x value")?,
                        Constr::In(a, b) => {
                            self.cdf(b).ok_or("Invalid x value")?
                                - self.cdf(a).ok_or("Invalid x value")?
                        }
                        Constr::Out(a, b) => {
                            1.0 + self.cdf(a).ok_or("Invalid x value")?
                                - self.cdf(b).ok_or("Invalid x value")?
                        }
                        _ => {
                            return Err("Cannot use exact in a continuous distribution.");
                        }
                    };
                    self.pval = Constr::EQ(fill);
                    self.strings[3] = fill.to_string();
                } else if !self.mean.is_some() {
                    let Constr::EQ(p) = self.pval else {
                        return Err("Probability must be set");
                    };
                    let (x, p) = match self.xval {
                        Constr::GE(x) | Constr::GT(x) => (x, 1.0 - p),
                        Constr::LE(x) | Constr::LT(x) => (x, p),
                        rng if rng.is_range() => {
                            return Err("Cannot use range to solve for mean.");
                        }
                        _ => {
                            return Err("Cannot use exact in a continuous distribution.");
                        }
                    };
                    let inv = statrs::distribution::Normal::new(0., 1.)
                        .expect("SND cant fail")
                        .inverse_cdf(p);
                    let fill = x - (self.sd.as_val().expect("Mean was only none") * inv);
                    self.mean = Constr::EQ(fill);
                    self.strings[0] = fill.to_string();
                } else if !self.sd.is_some() {
                    let Constr::EQ(p) = self.pval else {
                        return Err("Probability must be set");
                    };
                    let (x, p) = match self.xval {
                        Constr::GE(x) | Constr::GT(x) => (x, 1.0 - p),
                        Constr::LE(x) | Constr::LT(x) => (x, p),
                        rng if rng.is_range() => {
                            return Err("Cannot use range to solve for mean.");
                        }
                        _ => {
                            return Err("Cannot use exact in a continuous distribution.");
                        }
                    };
                    let inv = statrs::distribution::Normal::new(0., 1.)
                        .expect("SND cant fail")
                        .inverse_cdf(p);
                    if inv == 0.0 {
                        return Err("Not enough information, prob must not be 0.5");
                    }
                    let fill = (x - self.mean.as_val().ok_or("Mean must be set")?) / inv;
                    if fill < 0.0 {
                        if inv > 0.0 {
                            return Err("Prob > 0.5 but x value is less than the mean");
                        } else {
                            return Err("Prob < 0.5 but x value is greater than the mean");
                        }
                    }
                    if fill == 0.0 {
                        return Err("Not enough information, mean and x value are the same");
                    }
                    self.sd = Constr::EQ(fill);
                    self.strings[1] = fill.to_string();
                } else {
                    unreachable!();
                }
                Ok(())
            }
            4 => Ok(()),
            _ => {
                unreachable!();
            }
        }
    }
}

#[crunch_fill_eval]
#[derive(Debug, Clone)]
struct ChiSquare {
    freedom: Constr<f64>,
    xval: Constr<f64>,
    pval: Constr<f64>,
}

impl Default for ChiSquare {
    fn default() -> Self {
        Self {
            freedom: Constr::EQ(10.0),
            xval: Constr::LT(1.0),
            pval: Constr::None,
            strings: [
                "10.0".to_owned(),
                "<1.0".to_owned(),
                "".to_owned(),
                "".to_owned(),
            ],
        }
    }
}

impl TryContinuous for ChiSquare {
    fn pdf(&self, x: f64) -> Option<f64> {
        Some(
            statrs::distribution::ChiSquared::new(*self.freedom.as_val()?)
                .ok()?
                .pdf(x),
        )
    }

    fn cdf(&self, x: f64) -> Option<f64> {
        Some(
            statrs::distribution::ChiSquared::new(*self.freedom.as_val()?)
                .ok()?
                .cdf(x),
        )
    }

    fn inverse_cdf(&self, x: f64) -> Option<f64> {
        Some(
            statrs::distribution::ChiSquared::new(*self.freedom.as_val()?)
                .ok()?
                .inverse_cdf(x),
        )
    }
}

impl Widget for &mut ChiSquare {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let mut resp = ui.num_box("freedom", &mut self.strings[0]);
        let x = ui.num_box("x value", &mut self.strings[1]);
        resp = resp.union(x);
        let p = ui.num_box("prob", &mut self.strings[2]);
        resp = resp.union(p);
        if resp.changed() {
            self.vfill();
        }
        if ui.button("Calculate").clicked() {
            resp.mark_changed();
            if let Err(s) = self.fill() {
                self.strings[3] = s.to_owned();
            } else {
                self.strings[3] = "".to_owned();
            }
        }
        resp = resp
            .union(ui.label(RichText::new(&self.strings[3]).background_color(Color32::DARK_RED)));
        resp
    }
}

impl Fillable for ChiSquare {
    fn fill(&mut self) -> Result<(), &str> {
        let filled = [self.freedom, self.xval, self.pval]
            .iter()
            .filter(|x| x.is_some())
            .count();
        match filled {
            0..=1 => Err("Not enough filled"),
            2 => {
                if !self.xval.is_some() {
                    let Constr::EQ(p) = self.pval else {
                        return Err("Probability must be set");
                    };
                    let fill = match self.xval {
                        Constr::GENone | Constr::GTNone => match self.inverse_cdf(1.0 - p) {
                            Some(n) => Constr::GE(n),
                            None => return Err("Not a valid probability."),
                        },
                        Constr::LENone | Constr::LTNone => match self.inverse_cdf(p) {
                            Some(n) => Constr::LE(n),
                            None => return Err("Not a valid probability."),
                        },
                        eq if eq.is_eq() => {
                            return Err("Cannot use exact in a continuous distribution.");
                        }
                        _ => return Err("Cannot use ranges for solving for x values."),
                    };
                    self.xval = fill;
                    self.strings[1] = fill.to_string();
                } else if !self.pval.is_some() {
                    let fill = match self.xval {
                        Constr::GE(x) | Constr::GT(x) => {
                            1.0 - self.cdf(x).expect("Pval was only None, and distr is ok")
                        }
                        Constr::LE(x) | Constr::LT(x) => {
                            self.cdf(x).expect("Pval was only None, and distr is ok")
                        }
                        Constr::In(a, b) => {
                            self.cdf(b).expect("Pval was only None, and distr is ok")
                                - self.cdf(a).expect("Pval was only None, and distr is ok")
                        }
                        Constr::Out(a, b) => {
                            1.0 - self.cdf(b).expect("Pval was only None, and distr is ok")
                                + self.cdf(a).expect("Pval was only None, and distr is ok")
                        }
                        _ => return Err("X value must be an inequality"),
                    };
                    self.pval = Constr::EQ(fill);
                    self.strings[2] = fill.to_string();
                } else if !self.freedom.is_some() {
                    let Constr::EQ(p) = self.pval else {
                        return Err("Probability must be set");
                    };
                    let fill = match self.xval {
                        Constr::LE(x) | Constr::LT(x) => find_zero(|f| {
                            Some(statrs::distribution::ChiSquared::new(f).ok()?.cdf(x) - p)
                        }),
                        Constr::GE(x) | Constr::GT(x) => find_zero(|f| {
                            Some(statrs::distribution::ChiSquared::new(f).ok()?.cdf(x) + 1.0 - p)
                        }),
                        _ => return Err("X value must be an inequality."),
                    };
                    match fill {
                        Some(n) => {
                            self.freedom = Constr::EQ(n);
                            self.strings[0] = n.to_string();
                        }
                        None => {
                            return Err("No freedom value found");
                        }
                    }
                } else {
                    unreachable!();
                }
                Ok(())
            }
            3 => Ok(()),
            _ => {
                unreachable!();
            }
        }
    }
}

impl Graph for ChiSquare {
    fn get_height(&self, pos: f64) -> Option<f64> {
        self.pdf(pos)
    }

    fn start(&self) -> f64 {
        0.0
    }

    fn end(&self) -> f64 {
        self.inverse_cdf(0.999).unwrap_or(0.0)
    }

    fn is_selected(&self, pos: f64) -> bool {
        self.xval.comp(&pos)
    }
}

#[crunch_fill_eval]
#[derive(Debug, Clone)]
struct TDist {
    location: Constr<f64>,
    scale: Constr<f64>,
    freedom: Constr<f64>,
    xval: Constr<f64>,
    pval: Constr<f64>,
}

impl Default for TDist {
    fn default() -> Self {
        Self {
            freedom: Constr::EQ(4.0),
            xval: Constr::LE(0.0),
            pval: Constr::None,
            strings: [
                "0.0".to_owned(),
                "1.0".to_owned(),
                "4.0".to_owned(),
                "<=0.0".to_owned(),
                "".to_string(),
                "".to_string(),
            ],
            location: Constr::EQ(0.0),
            scale: Constr::EQ(1.0),
        }
    }
}

impl TryContinuous for TDist {
    fn pdf(&self, x: f64) -> Option<f64> {
        Some(
            statrs::distribution::StudentsT::new(
                *self.location.as_val()?,
                *self.scale.as_val()?,
                *self.freedom.as_val()?,
            )
            .ok()?
            .pdf(x),
        )
    }

    fn cdf(&self, x: f64) -> Option<f64> {
        Some(
            statrs::distribution::StudentsT::new(
                *self.location.as_val()?,
                *self.scale.as_val()?,
                *self.freedom.as_val()?,
            )
            .ok()?
            .cdf(x),
        )
    }

    fn inverse_cdf(&self, x: f64) -> Option<f64> {
        Some(
            statrs::distribution::StudentsT::new(
                *self.location.as_val()?,
                *self.scale.as_val()?,
                *self.freedom.as_val()?,
            )
            .ok()?
            .inverse_cdf(x),
        )
    }
}

impl Widget for &mut TDist {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let mut resp = ui.num_box("location", &mut self.strings[0]);
        resp = resp.union(ui.num_box("scale", &mut self.strings[1]));
        resp = resp.union(ui.num_box("freedom", &mut self.strings[2]));
        let x = ui.num_box("x value", &mut self.strings[3]);
        resp = resp.union(x);
        let p = ui.num_box("prob", &mut self.strings[4]);
        resp = resp.union(p);
        if resp.changed() {
            self.vfill();
        }
        if ui.button("Calculate").clicked() {
            resp.mark_changed();
            if let Err(s) = self.fill() {
                self.strings[5] = s.to_owned();
            } else {
                self.strings[5] = "".to_owned();
            }
        }
        resp = resp
            .union(ui.label(RichText::new(&self.strings[5]).background_color(Color32::DARK_RED)));
        resp
    }
}

impl Fillable for TDist {
    fn fill(&mut self) -> Result<(), &str> {
        let filled = [
            self.location,
            self.scale,
            self.freedom,
            self.xval,
            self.pval,
        ]
        .iter()
        .filter(|x| x.is_some())
        .count();
        match filled {
            0..=3 => Err("Not enough filled"),
            4 => {
                if !self.xval.is_some() {
                    let Constr::EQ(p) = self.pval else {
                        return Err("Probability must be set");
                    };
                    let fill = match self.xval {
                        Constr::GENone | Constr::GTNone => Constr::GE(
                            self.inverse_cdf(1.0 - p)
                                .expect("Xval was only None")
                                .to_string()
                                .parse()
                                .unwrap(),
                        ),
                        Constr::LENone | Constr::LTNone => Constr::LE(
                            self.inverse_cdf(p)
                                .expect("Xval was only None")
                                .to_string()
                                .parse()
                                .unwrap(),
                        ),
                        _ => {
                            return Err("Cannot use equal on x value.");
                        }
                    };
                    self.xval = fill;
                    self.strings[3] = fill.to_string();
                } else if !self.pval.is_some() {
                    let fill = match self.xval {
                        Constr::GE(x) | Constr::GT(x) => {
                            1.0 - self.cdf(x).expect("Pval was only None, and distr is ok")
                        }
                        Constr::LE(x) | Constr::LT(x) => {
                            self.cdf(x).expect("Pval was only None, and distr is ok")
                        }
                        Constr::In(a, b) => {
                            self.cdf(b).expect("Pval was only None, and distr is ok")
                                - self.cdf(a).expect("Pval was only None, and distr is ok")
                        }
                        Constr::Out(a, b) => {
                            1.0 - self.cdf(b).expect("Pval was only None, and distr is ok")
                                + self.cdf(a).expect("Pval was only None, and distr is ok")
                        }
                        _ => return Err("X value must be an inequality"),
                    };
                    self.pval = Constr::EQ(fill);
                    self.strings[4] = fill.to_string();
                } else if !self.freedom.is_some() {
                    let Constr::EQ(l) = self.location else {
                        return Err("Location must be set");
                    };
                    let Constr::EQ(sc) = self.scale else {
                        return Err("Scale must be set");
                    };
                    let Constr::EQ(p) = self.pval else {
                        return Err("Probability must be set");
                    };
                    let fill_rv = match self.xval {
                        Constr::GE(x) | Constr::GT(x) => {
                            if l - x == 0.0 {
                                return Err("X value must not be the location");
                            }
                            let fill = find_zero(|f| {
                                let distr = statrs::distribution::StudentsT::new(l, sc, f).ok()?;
                                let test_p = distr.cdf(x);
                                Some(test_p + p - 1.0)
                            });
                            fill.map(|fill| {
                                (
                                    fill,
                                    1.0 - statrs::distribution::StudentsT::new(l, sc, fill)
                                        .unwrap()
                                        .cdf(x),
                                )
                            })
                        }
                        Constr::LE(x) | Constr::LT(x) => {
                            if l - x == 0.0 {
                                return Err("X value must not be the location");
                            }
                            let fill = find_zero(|f| {
                                let distr = statrs::distribution::StudentsT::new(l, sc, f).ok()?;
                                let test_p = distr.cdf(x);
                                Some(test_p - p)
                            });
                            fill.map(|fill| {
                                (
                                    fill,
                                    statrs::distribution::StudentsT::new(l, sc, fill)
                                        .unwrap()
                                        .cdf(x),
                                )
                            })
                        }
                        _ => return Err("X value must use inequality to solve for other value"),
                    };
                    //println!("Escaped find");
                    match fill_rv {
                        Some((n, _)) if n < 100000. => {
                            self.freedom = Constr::EQ(n);
                            self.strings[2] = n.to_string();
                        }
                        Some((n, rv)) => {
                            if (rv - p).abs() > 0.0001 {
                                return Err("No freedom value found. P value must be closer to 0.5 than for the normal distribution");
                            } else {
                                self.freedom = Constr::EQ(n);
                                self.strings[2] = n.to_string();
                            }
                        }
                        None => {
                            return Err("No freedom value found");
                        }
                    }
                } else if !self.location.is_some() {
                    let Constr::EQ(f) = self.freedom else {
                        return Err("Location must be set");
                    };
                    let Constr::EQ(sc) = self.scale else {
                        return Err("Scale must be set");
                    };
                    let Constr::EQ(p) = self.pval else {
                        return Err("Probability must be set");
                    };
                    let x = match self.xval {
                        Constr::GE(x) | Constr::GT(x) => -x,
                        Constr::LE(x) | Constr::LT(x) => x,
                        _ => return Err("Must use inequality to solve for another value."),
                    };
                    let inv = statrs::distribution::StudentsT::new(0.0, 1.0, f)
                        .expect("Location was only None")
                        .inverse_cdf(p);
                    let fill = x - (sc * inv);
                    self.location = Constr::EQ(fill);
                    self.strings[0] = fill.to_string();
                } else if !self.scale.is_some() {
                    let Constr::EQ(f) = self.freedom else {
                        return Err("Location must be set");
                    };
                    let Constr::EQ(l) = self.location else {
                        return Err("Scale must be set");
                    };
                    let Constr::EQ(p) = self.pval else {
                        return Err("Probability must be set");
                    };
                    let x = match self.xval {
                        Constr::GE(x) | Constr::GT(x) => -x,
                        Constr::LE(x) | Constr::LT(x) => x,
                        _ => return Err("Must use inequality to solve for another value."),
                    };
                    let inv = statrs::distribution::StudentsT::new(0., 1., f)
                        .map_err(|_| "Freedom cannot be negative")?
                        .inverse_cdf(p);
                    if inv == 0.0 {
                        return Err("Not enough information, prob must not be 0.5");
                    }
                    let fill = (x - l) / inv;
                    if fill < 0.0 {
                        if inv > 0.0 {
                            return Err("Prob > 0.5 but x value is less than the location");
                        } else {
                            return Err("Prob < 0.5 but x value is greater than the location");
                        }
                    }
                    if fill == 0.0 {
                        return Err("Not enough information, location and x value are the same");
                    }
                    self.scale = Constr::EQ(fill);
                    self.strings[1] = fill.to_string();
                } else {
                    unreachable!();
                }
                Ok(())
            }
            5 => Ok(()),
            _ => {
                unreachable!();
            }
        }
    }
}

impl Graph for TDist {
    fn get_height(&self, pos: f64) -> Option<f64> {
        self.pdf(pos)
    }

    fn start(&self) -> f64 {
        self.inverse_cdf(0.001).unwrap_or(0.0)
    }

    fn end(&self) -> f64 {
        self.inverse_cdf(0.999).unwrap_or(0.0)
    }

    fn is_selected(&self, pos: f64) -> bool {
        self.xval.comp(&pos)
    }
}

#[crunch_fill_eval]
#[derive(Debug, Clone)]
struct FDist {
    freedom1: Constr<f64>,
    freedom2: Constr<f64>,
    xval: Constr<f64>,
    pval: Constr<f64>,
}

impl Default for FDist {
    fn default() -> Self {
        Self {
            freedom1: Constr::EQ(4.0),
            xval: Constr::LE(1.0),
            pval: Constr::None,
            strings: [
                "4.0".to_owned(),
                "4.0".to_owned(),
                "<1.0".to_owned(),
                "".to_string(),
                "".to_string(),
            ],
            freedom2: Constr::EQ(4.0),
        }
    }
}

impl TryContinuous for FDist {
    fn pdf(&self, x: f64) -> Option<f64> {
        Some(
            statrs::distribution::FisherSnedecor::new(
                *self.freedom1.as_val()?,
                *self.freedom2.as_val()?,
            )
            .ok()?
            .pdf(x),
        )
    }

    fn cdf(&self, x: f64) -> Option<f64> {
        Some(
            statrs::distribution::FisherSnedecor::new(
                *self.freedom1.as_val()?,
                *self.freedom2.as_val()?,
            )
            .ok()?
            .cdf(x),
        )
    }

    fn inverse_cdf(&self, x: f64) -> Option<f64> {
        Some(
            statrs::distribution::FisherSnedecor::new(
                *self.freedom1.as_val()?,
                *self.freedom2.as_val()?,
            )
            .ok()?
            .inverse_cdf(x),
        )
    }
}

impl Widget for &mut FDist {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let mut resp = ui.num_box("freedom1", &mut self.strings[0]);
        resp = resp.union(ui.num_box("freedom2", &mut self.strings[1]));
        let x = ui.num_box("x value", &mut self.strings[2]);
        resp = resp.union(x);
        let p = ui.num_box("prob", &mut self.strings[3]);
        resp = resp.union(p);
        if resp.changed() {
            self.vfill()
        }
        if ui.button("Calculate").clicked() {
            resp.mark_changed();
            if let Err(s) = self.fill() {
                self.strings[4] = s.to_owned();
            } else {
                self.strings[4] = "".to_owned();
            }
        }
        resp = resp
            .union(ui.label(RichText::new(&self.strings[4]).background_color(Color32::DARK_RED)));
        resp
    }
}

impl Fillable for FDist {
    fn fill(&mut self) -> Result<(), &str> {
        let filled = [self.freedom1, self.freedom2, self.xval, self.pval]
            .iter()
            .filter(|x| x.is_some())
            .count();
        match filled {
            0..=2 => Err("Not enough filled"),
            3 => {
                if !self.xval.is_some() {
                    let Constr::EQ(p) = self.pval else {
                        return Err("Probability must be set");
                    };
                    let fill = match self.xval {
                        Constr::GENone | Constr::GTNone => {
                            Constr::GE(self.inverse_cdf(1.0 - p).expect("Xval was only None"))
                        }
                        Constr::LENone | Constr::LTNone => {
                            Constr::LE(self.inverse_cdf(p).expect("Xval was only None"))
                        }
                        _ => {
                            return Err("Cannot use equal on x value.");
                        }
                    };
                    self.xval = fill;
                    self.strings[2] = fill.to_string();
                } else if !self.pval.is_some() {
                    let fill = match self.xval {
                        Constr::GE(x) | Constr::GT(x) => {
                            1.0 - self.cdf(x).expect("Pval was only None, and distr is ok")
                        }
                        Constr::LE(x) | Constr::LT(x) => {
                            self.cdf(x).expect("Pval was only None, and distr is ok")
                        }
                        Constr::In(a, b) => {
                            self.cdf(b).expect("Pval was only None, and distr is ok")
                                - self.cdf(a).expect("Pval was only None, and distr is ok")
                        }
                        Constr::Out(a, b) => {
                            1.0 - self.cdf(b).expect("Pval was only None, and distr is ok")
                                + self.cdf(a).expect("Pval was only None, and distr is ok")
                        }
                        _ => return Err("X value must be an inequality"),
                    };
                    self.pval = Constr::EQ(fill);
                    self.strings[3] = fill.to_string();
                } else if !self.freedom1.is_some() || !self.freedom2.is_some() {
                    self.strings[4] = "Cannot solve for freedom yet".to_string();
                } else {
                    unreachable!();
                }
                Ok(())
            }
            4 => Ok(()),
            _ => {
                unreachable!();
            }
        }
    }
}

impl Graph for FDist {
    fn get_height(&self, pos: f64) -> Option<f64> {
        self.pdf(pos)
    }

    fn start(&self) -> f64 {
        0.0
    }

    fn end(&self) -> f64 {
        self.inverse_cdf(0.99).unwrap_or(0.0)
    }

    fn is_selected(&self, pos: f64) -> bool {
        self.xval.comp(&pos)
    }
}

#[derive(Debug, Clone)]
struct Expon {
    mean: Option<f64>,
    xval: Option<f64>,
    pval: Option<f64>,
    strings: [String; 4],
}

impl Default for Expon {
    fn default() -> Self {
        todo!();
        Self {
            mean: Some(1.0),
            xval: Some(1.0),
            pval: None,
            strings: [
                "1.0".to_string(),
                "1.0".to_string(),
                "".to_string(),
                "".to_string(),
            ],
        }
    }
}

impl TryContinuous for Expon {
    fn pdf(&self, x: f64) -> Option<f64> {
        Some(statrs::distribution::Exp::new(self.mean?).ok()?.pdf(x))
    }

    fn cdf(&self, x: f64) -> Option<f64> {
        Some(statrs::distribution::Exp::new(self.mean?).ok()?.cdf(x))
    }

    fn inverse_cdf(&self, x: f64) -> Option<f64> {
        Some(
            statrs::distribution::Exp::new(self.mean?)
                .ok()?
                .inverse_cdf(x),
        )
    }
}

impl Graph for Expon {
    fn get_height(&self, pos: f64) -> Option<f64> {
        self.pdf(pos)
    }

    fn start(&self) -> f64 {
        0.0
    }

    fn end(&self) -> f64 {
        self.inverse_cdf(0.99).unwrap_or(0.0)
    }

    fn is_selected(&self, pos: f64) -> bool {
        self.xval.unwrap_or(0.0) >= pos
    }
}

impl Widget for &mut Expon {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let mut resp = ui.num_box("mean", &mut self.strings[0]);
        let x = ui.num_box("x value", &mut self.strings[1]);
        if x.changed() && !self.strings[1].is_empty() {
            self.strings[2] = "".to_owned();
        }
        resp = resp.union(x);
        let p = ui.num_box("prob", &mut self.strings[2]);
        if p.changed() && !self.strings[2].is_empty() {
            self.strings[1] = "".to_owned();
        }
        resp = resp.union(p);
        if resp.changed() {
            self.mean = self.strings[0].parse().ok();
            self.xval = self.strings[1].parse().ok();
            self.pval = self.strings[2].parse().ok();
        }
        if ui.button("Calculate").clicked() {
            resp.mark_changed();
            if let Err(s) = self.fill() {
                self.strings[3] = s.to_owned();
            } else {
                self.strings[3] = "".to_owned();
            }
        }
        resp = resp
            .union(ui.label(RichText::new(&self.strings[3]).background_color(Color32::DARK_RED)));
        resp
    }
}

impl Fillable for Expon {
    fn fill(&mut self) -> Result<(), &str> {
        let filled = [self.mean, self.xval, self.pval]
            .iter()
            .filter(|x| x.is_some())
            .count();
        match filled {
            0..=1 => Err("Not enough values"),
            2 => {
                if self.mean.is_none() {
                    todo!()
                } else if self.xval.is_none() {
                    self.xval = self.inverse_cdf(self.pval.expect("Xval was only None"));
                    self.strings[1] = self.xval.unwrap().to_string();
                    Ok(())
                } else if self.pval.is_none() {
                    self.pval = self.cdf(self.xval.expect("Pval was only None"));
                    self.strings[2] = self.pval.unwrap().to_string();
                    Ok(())
                } else {
                    unreachable!()
                }
            }
            3 => Ok(()),
            _ => unreachable!(),
        }
    }
}

//Ignore all of this, I'll generalize later
/*
enum ConstrErr {
    OOB,
    IsNone,
}

trait Constraint {
    const SIZE: usize;

    fn fill(&mut self) {
        self.try_fill();
        if self.solved() {
            return;
        }
        let Some(target) = self.solv().iter().zip((0..Self::SIZE).map(|i| self.get_field(i)))
            .position(|(b, field)| *b && field.is_ok()) else {
            log("No field to use to solve");
            return;
        };
        for field in 0..Self::SIZE {
            if matches!(self.get_field(field), Err(ConstrErr::IsNone)) {
                if self.solv()[field] {
                    log("Not enough information");
                    log("Field {field} should have been filled");
                    return;
                }
                let mut low = self.field_default(field).unwrap_or(0.0);
                let mut high = low;
                //while
            }
        }
    }

    fn try_fill(&mut self);

    fn get_field(&self, field: usize) -> Result<f64, ConstrErr>;

    fn field_default(&self, field: usize) -> Result<f64, ConstrErr>;

    fn solv(&self) -> &[bool];

    fn solved(&self) -> bool;
}
*/
