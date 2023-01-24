use egui::{Ui, RichText, Color32};
use statrs::distribution::{Continuous, ContinuousCDF};

use crate::log;

pub(crate) trait TryContinuous {
    fn pdf(&self, x: f64) -> Option<f64>;
    fn cdf(&self, x: f64) -> Option<f64>;
    fn inverse_cdf(&self, x: f64) -> Option<f64>;
}

pub(crate) trait Show {
    fn show(&mut self, ui: &mut Ui) -> egui::Response;
}

pub(crate) trait Fillable {
    fn fill(&mut self) -> Result<(), &str>;
}

fn coerce_numeric(s: &String) -> String {
    s.chars().filter(|c| c.is_ascii_digit() || *c=='.' || *c == '-').collect()
}

fn find_zero(f: impl Fn(f64) -> Option<f64>) -> Option<f64> {
    let mut high = 1.0;
    let mut low = -1.0;
    if f(high).is_none() && f(low).is_none() {
        return None
    }
    while f(high).is_none() {
        high = 0.9*high+0.1*low;
    }
    while f(low).is_none() {
        low = 0.9*low+0.1*high;
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
        }
    }
    if f(high).signum() == -1.0 {
        (high, low) = (low, high);
    }
    while (high-low) > 0.0000001 {
        if f((high+low)/2.0) > 0.0 {
            high = (high+low)/2.0;
        }
        else {
            low = (high+low)/2.0;
        }
    }
    Some((high+low)/2.0)
}

pub(crate) struct Normal{
    mean: Option<f64>,
    sd: Option<f64>,
    xval: Option<f64>,
    pval: Option<f64>,
    /// \[mean, sd, xval, pval, error\]
    strings: [String; 5],
}

impl Default for Normal {
    fn default() -> Self {
        Self { mean: Some(0.0), sd: Some(1.0), xval: Some(0.0), pval: None, strings: [
            String::from("0.0"),
            String::from("1.0"),
            String::from("0.0"),
            "".to_string(),
            "".to_string(),
        ] }
    }
}

impl TryContinuous for Normal {
    fn pdf(&self, x: f64) -> Option<f64> {
        Some(statrs::distribution::Normal::new(self.mean?, self.sd?).ok()?.pdf(x))
    }

    fn cdf(&self, x: f64) -> Option<f64> {
        Some(statrs::distribution::Normal::new(self.mean?, self.sd?).ok()?.cdf(x))
    }

    fn inverse_cdf(&self, x: f64) -> Option<f64> {
        Some(statrs::distribution::Normal::new(self.mean?, self.sd?).ok()?.inverse_cdf(x))
    }
}

impl Show for Normal {
    fn show(&mut self, ui: &mut Ui) -> egui::Response {
        let mut resp = ui.num_box("mean", &mut self.strings[0]);
        resp = resp.union(ui.num_box("std dev", &mut self.strings[1]));
        let clear = self.mean.is_some() && self.sd.is_some();
        let x = ui.num_box("x value", &mut self.strings[2]);
        if x.changed() && clear && !self.strings[2].is_empty() {
            self.strings[3] = "".to_owned();
        }
        resp = resp.union(x);
        let p = ui.num_box("prob", &mut self.strings[3]);
        if p.changed() && clear && !self.strings[3].is_empty() {
            self.strings[2] = "".to_owned();
        }
        resp = resp.union(p);
        self.mean = self.strings[0].parse::<f64>().ok();
        self.sd = self.strings[1].parse::<f64>().ok();
        self.xval = self.strings[2].parse::<f64>().ok();
        self.pval = self.strings[3].parse::<f64>().ok();
        if ui.button("Calculate").clicked() {
            resp.mark_changed();
            if let Err(s) = self.fill() {
                self.strings[4] = s.to_owned();
            }
            else {
                self.strings[4] = "".to_owned();
            }
        }
        resp = resp.union(ui.label(RichText::new(&self.strings[4]).background_color(Color32::DARK_RED)));
        resp
    }
}

impl Fillable for Normal {
    fn fill(&mut self) -> Result<(), &str> {
        let filled = [self.mean, self.sd, self.xval, self.pval].iter().filter(|x| x.is_some()).count();
        match filled {
            0..=2 => {
                return Err("Not enough filled");
            }
            3 => {
                if self.xval.is_none() {
                    let fill = self.inverse_cdf(self.pval.expect("Xval was only None")).expect("Xval was only None");
                    self.xval = Some(fill);
                    self.strings[2] = fill.to_string();
                }
                else if self.pval.is_none() {
                    let fill = self.cdf(self.xval.expect("Pval was only None")).expect("Pval was only None, and distr is ok");
                    self.pval = Some(fill);
                    self.strings[3] = fill.to_string();
                }
                else if self.mean.is_none() {
                    let inv = statrs::distribution::Normal::new(0., 1.).expect("SND cant fail").inverse_cdf(self.pval.expect("Mean was only none"));
                    let fill = self.xval.expect("Mean was only none") - 
                        (self.sd.expect("Mean was only none")*inv);
                    self.mean = Some(fill);
                    self.strings[0] = fill.to_string();
                }
                else if self.sd.is_none() {
                    let inv = statrs::distribution::Normal::new(0., 1.).expect("SND cant fail").inverse_cdf(self.pval.expect("SD was only none"));
                    if inv == 0.0 {
                        return Err("Not enough information, prob must not be 0.5");
                    }
                    let fill = (self.xval.expect("SD was only none")-self.mean.expect("SD was only none"))/inv;
                    if fill < 0.0 {
                        if inv > 0.0 {
                            return Err("Prob > 0.5 but x value is less than the mean")
                        }
                        else {
                            return Err("Prob < 0.5 but x value is greater than the mean")
                        }
                    }
                    if fill == 0.0 {
                        return Err("Not enough information, mean and x value are the same")
                    }
                    self.sd = Some(fill);
                    self.strings[1] = fill.to_string();
                }
                else {
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

pub(crate) struct ChiSquare {
    freedom: Option<f64>,
    xval: Option<f64>,
    pval: Option<f64>,
    /// \[freedom, xval, pval\]
    strings: [String; 4],
}

impl Default for ChiSquare {
    fn default() -> Self {
        Self { freedom: Some(10.0), xval: Some(1.0), pval: None, strings: [
            "10.0".to_owned(),
            "1.0".to_owned(),
            "".to_owned(),
            "".to_owned(),
        ] }
    }
}

impl TryContinuous for ChiSquare {
    fn pdf(&self, x: f64) -> Option<f64> {
        Some(statrs::distribution::ChiSquared::new(self.freedom?).ok()?.pdf(x))
    }

    fn cdf(&self, x: f64) -> Option<f64> {
        Some(statrs::distribution::ChiSquared::new(self.freedom?).ok()?.cdf(x))
    }

    fn inverse_cdf(&self, x: f64) -> Option<f64> {
        Some(statrs::distribution::ChiSquared::new(self.freedom?).ok()?.inverse_cdf(x))
    }
}

impl Show for ChiSquare {
    fn show(&mut self, ui: &mut Ui) -> egui::Response {
        let mut resp = ui.num_box("freedom", &mut self.strings[0]);
        let clear = self.freedom.is_some();
        let x = ui.num_box("x value", &mut self.strings[1]);
        if x.changed() && clear && !self.strings[1].is_empty() {
            self.strings[2] = "".to_owned();
        }
        resp = resp.union(x);
        let p = ui.num_box("prob", &mut self.strings[2]);
        if p.changed() && clear && !self.strings[2].is_empty() {
            self.strings[1] = "".to_owned();
        }
        resp = resp.union(p);
        self.freedom = self.strings[0].parse::<f64>().ok();
        self.xval = self.strings[1].parse::<f64>().ok();
        self.pval = self.strings[2].parse::<f64>().ok();
        if ui.button("Calculate").clicked() {
            resp.mark_changed();
            if let Err(s) = self.fill() {
                self.strings[3] = s.to_owned();
            }
            else {
                self.strings[3] = "".to_owned();
            }
        }
        resp = resp.union(ui.label(RichText::new(&self.strings[3]).background_color(Color32::DARK_RED)));
        resp
    }
}

impl Fillable for ChiSquare {
    fn fill(&mut self) -> Result<(), &str> {
        let filled = [self.freedom, self.xval, self.pval].iter().filter(|x| x.is_some()).count();
        match filled {
            0..=1 => {
                return Err("Not enough filled");
            }
            2 => {
                if self.xval.is_none() {
                    let fill = self.inverse_cdf(self.pval.expect("Xval was only None")).expect("Xval was only None");
                    self.xval = Some(fill);
                    self.strings[1] = fill.to_string();
                }
                else if self.pval.is_none() {
                    let fill = self.cdf(self.xval.expect("Pval was only None")).expect("Pval was only None, and distr is ok");
                    self.pval = Some(fill);
                    self.strings[2] = fill.to_string();
                }
                else if self.freedom.is_none() {
                    let fill = find_zero(|f| Some(statrs::distribution::ChiSquared::new(f)
                    .ok()?.cdf(self.xval.expect("Freedom was only None"))-self.pval.expect("Freedom was only None")));
                    match fill {
                        Some(n) => {
                            self.freedom = Some(n);
                            self.strings[0] = n.to_string();
                        },
                        None => {
                            return Err("No freedom value found");
                        },
                    }
                }
                else {
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

pub(crate) struct TDist {
    location: Option<f64>,
    scale: Option<f64>,
    freedom: Option<f64>,
    xval: Option<f64>,
    pval: Option<f64>,
    /// \[location, scale, freedom, xval, pval\]
    strings: [String; 6],
}

impl Default for TDist {
    fn default() -> Self {
        Self { freedom: Some(4.0), xval: Some(1.0), pval: None, strings: [
            "0.0".to_owned(),
            "1.0".to_owned(),
            "4.0".to_owned(),
            "0.0".to_owned(),
            "".to_string(),
            "".to_string(),
        ],
            location: Some(0.0),
            scale: Some(1.0), }
    }
}

impl TryContinuous for TDist {
    fn pdf(&self, x: f64) -> Option<f64> {
        Some(statrs::distribution::StudentsT::new(self.location?, self.scale?, self.freedom?).ok()?.pdf(x))
    }

    fn cdf(&self, x: f64) -> Option<f64> {
        Some(statrs::distribution::StudentsT::new(self.location?, self.scale?, self.freedom?).ok()?.cdf(x))
    }

    fn inverse_cdf(&self, x: f64) -> Option<f64> {
        Some(statrs::distribution::StudentsT::new(self.location?, self.scale?, self.freedom?).ok()?.inverse_cdf(x))
    }
}

impl Show for TDist {
    fn show(&mut self, ui: &mut Ui) -> egui::Response {
        let mut resp = ui.num_box("location", &mut self.strings[0]);
        resp = ui.num_box("scale", &mut self.strings[1]);
        resp = ui.num_box("freedom", &mut self.strings[2]);
        let clear = self.location.is_some() && self.scale.is_some() && self.freedom.is_some();
        let x = ui.num_box("x value", &mut self.strings[3]);
        if x.changed() && clear && !self.strings[3].is_empty() {
            self.strings[4] = "".to_owned();
        }
        resp = resp.union(x);
        let p = ui.num_box("prob", &mut self.strings[4]);
        if p.changed() && clear && !self.strings[4].is_empty() {
            self.strings[3] = "".to_owned();
        }
        resp = resp.union(p);
        self.location = self.strings[0].parse::<f64>().ok();
        self.scale = self.strings[1].parse::<f64>().ok();
        self.freedom = self.strings[2].parse::<f64>().ok();
        self.xval = self.strings[3].parse::<f64>().ok();
        self.pval = self.strings[4].parse::<f64>().ok();
        if ui.button("Calculate").clicked() {
            resp.mark_changed();
            if let Err(s) = self.fill() {
                self.strings[5] = s.to_owned();
            }
            else {
                self.strings[5] = "".to_owned();
            }
        }
        resp = resp.union(ui.label(RichText::new(&self.strings[5]).background_color(Color32::DARK_RED)));
        resp
    }
}

impl Fillable for TDist {
    fn fill(&mut self) -> Result<(), &str> {
        let filled = [self.location, self.scale, self.freedom, self.xval, self.pval].iter().filter(|x| x.is_some()).count();
        match filled {
            0..=3 => {
                return Err("Not enough filled");
            }
            4 => {
                if self.xval.is_none() {
                    let fill = self.inverse_cdf(self.pval.expect("Xval was only None")).expect("Xval was only None");
                    self.xval = Some(fill);
                    self.strings[3] = fill.to_string();
                }
                else if self.pval.is_none() {
                    let fill = self.cdf(self.xval.expect("Pval was only None")).expect("Pval was only None, and distr is ok");
                    self.pval = Some(fill);
                    self.strings[4] = fill.to_string();
                }
                else if self.freedom.is_none() {
                    let l = self.location.expect("Freedon was only None");
                    let sc = self.scale.expect("Freedom was only None");
                    //TODO make this more readable
                    //TODO fix setting to numbers outside SND
                    let fill = find_zero(|f| Some(statrs::distribution::StudentsT::new(l, sc, f)
                    .ok()?.cdf(self.xval.expect("Freedom was only None"))-self.pval.expect("Freedom was only None")));
                    match fill {
                        Some(n) => {
                            self.freedom = Some(n);
                            self.strings[2] = n.to_string();
                        },
                        None => {
                            return Err("No freedom value found");
                        },
                    }
                }
                else if self.location.is_none() {
                    let inv = statrs::distribution::StudentsT::new(0.0, 1.0, self.freedom.expect("Location was only None")).expect("Location was only None").inverse_cdf(self.pval.expect("Location was only None"));
                    let fill = self.xval.expect("Location was only None") - 
                        (self.scale.expect("Location was only None")*inv);
                    self.location = Some(fill);
                    self.strings[0] = fill.to_string();
                }
                else if self.scale.is_none() {
                    let inv = statrs::distribution::StudentsT::new(0., 1., self.freedom.expect("Scale was only None")).map_err(|x| "Freedom cannot be negative")?.inverse_cdf(self.pval.expect("Scale was only none"));
                    if inv == 0.0 {
                        return Err("Not enough information, prob must not be 0.5");
                    }
                    let fill = (self.xval.expect("Scale was only none")-self.location.expect("Scale was only none"))/inv;
                    if fill < 0.0 {
                        if inv > 0.0 {
                            return Err("Prob > 0.5 but x value is less than the location")
                        }
                        else {
                            return Err("Prob < 0.5 but x value is greater than the location")
                        }
                    }
                    if fill == 0.0 {
                        return Err("Not enough information, location and x value are the same")
                    }
                    self.scale = Some(fill);
                    self.strings[1] = fill.to_string();
                }
                else {
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

trait NumBox {
    fn num_box(&mut self, l: &str, v: &mut String) -> egui::Response;
}

impl NumBox for Ui {
    fn num_box(&mut self, l: &str, v: &mut String) -> egui::Response {
        self.horizontal(|ui| {
            ui.label(l);
            let resp = ui.text_edit_singleline(v);
            *(v) = coerce_numeric(v);
            resp
        }).inner
    }
}

//Ignore all of this, I'll generalize later
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