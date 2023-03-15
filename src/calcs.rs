use egui::{Color32, RichText, Ui, Widget};
use meval::Expr;
use opencrunch_derive::{crunch_fill, crunch_fill_eval};
use statrs::{
    distribution::{ContinuousCDF, Normal},
    function,
};

use crate::{empty_resp, Constr, NumBox};

#[derive(Default, Clone)]
enum Calcs {
    #[default]
    None,
    SampInf(SampleProbInf),
    SampFin(SampleProbFin),
    Comb(Comb),
    Calc(Calc),
    Cheby(Cheby),
}

#[derive(Default)]
pub(crate) struct OpenCrunchSample {
    sample: Calcs,
}

impl Widget for &mut OpenCrunchSample {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        ui.horizontal(|ui| {
            if ui.button("Probability of Sample").clicked() {
                self.sample = Calcs::SampInf(SampleProbInf::default());
            }
            if ui.button("Probability of Sample from Finite").clicked() {
                self.sample = Calcs::SampFin(SampleProbFin::default());
            }
            if ui.button("Comb/Perm").clicked() {
                self.sample = Calcs::Comb(Comb::default());
            }
            if ui.button("Calculator").clicked() {
                self.sample = Calcs::Calc(Calc::default());
            }
            if ui.button("Chebyshev").clicked() {
                self.sample = Calcs::Cheby(Cheby::default());
            }
        });

        match &mut self.sample {
            Calcs::None => empty_resp(ui),
            Calcs::SampInf(sam) => ui.add(sam),
            Calcs::Comb(c) => ui.add(c),
            Calcs::SampFin(sam) => ui.add(sam),
            Calcs::Calc(calc) => ui.add(calc),
            Calcs::Cheby(cheb) => ui.add(cheb),
        }
    }
}

#[crunch_fill]
#[derive(Clone)]
pub(crate) struct SampleProbInf {
    sample_size: usize,
    mean: f64,
    sd: f64,
    target_mean: Constr<f64>,
    prob: Constr<f64>,
}

impl Default for SampleProbInf {
    fn default() -> Self {
        Self {
            sample_size: 1,
            mean: 0.0,
            sd: 1.0,
            target_mean: Constr::LE(0.0),
            strings: [
                1.to_string(),
                0.0.to_string(),
                1.0.to_string(),
                "<0.0".to_string(),
                0.5.to_string(),
                "".to_string(),
            ],
            prob: Constr::None,
        }
    }
}

impl Widget for &mut SampleProbInf {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let mut resp = ui.num_box("Sample Size", &mut self.strings[0]);
        resp = resp.union(ui.num_box("Population Mean", &mut self.strings[1]));
        resp = resp.union(ui.num_box("Population SD", &mut self.strings[2]));
        resp = resp.union(ui.num_box("Sample Mean", &mut self.strings[3]));
        if resp.changed() {
            self.vfill();
        }
        ui.horizontal(|ui| {
            ui.label("Prob");
            ui.text_edit_singleline(&mut (self.strings[4].clone()))
        });
        ui.label(RichText::new(&self.strings[5]).color(Color32::DARK_RED));
        if resp.changed() {
            //self.strings[4] = self.comp.to_string();
            match Normal::new(self.mean, self.sd / ((self.sample_size as f64).sqrt())) {
                Ok(n) => {
                    let fill = match self.target_mean {
                        Constr::GE(v) | Constr::GT(v) => 1.0 - n.cdf(v),
                        Constr::LE(v) | Constr::LT(v) => n.cdf(v),
                        Constr::EQ(_) | Constr::NE(_) => {
                            self.prob = Constr::None;
                            self.strings[4] = "".to_string();
                            self.strings[5] =
                                "Cannot use exact in a continuous distribution.".to_string();
                            return resp;
                        }
                        Constr::In(a, b) => n.cdf(b) - n.cdf(a),
                        Constr::Out(a, b) => 1.0 - n.cdf(b) + n.cdf(a),
                        _ => {
                            self.prob = Constr::None;
                            self.strings[4] = "".to_string();
                            self.strings[5] = "Target mean must be set".to_string();
                            return resp;
                        }
                    };
                    self.prob = Constr::EQ(fill);
                    self.strings[4] = fill.to_string();
                    self.strings[5] = "".to_string();
                }
                Err(e) => {
                    self.prob = Constr::None;
                    self.strings[4] = "".to_string();
                    self.strings[5] = e.to_string();
                }
            }
        }
        resp
    }
}

#[derive(Clone)]
struct Comb {
    strings: [String; 4],
}

impl Default for Comb {
    fn default() -> Self {
        Self {
            strings: [
                "1".to_string(),
                "1".to_string(),
                "1".to_string(),
                "1".to_string(),
            ],
        }
    }
}

impl Widget for &mut Comb {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let mut resp = ui.num_box("N", &mut self.strings[0]);
        resp = resp.union(ui.num_box("R", &mut self.strings[1]));
        if resp.changed() {
            if let Ok(n) = self.strings[0].parse::<f64>() {
                if let Ok(r) = self.strings[1].parse::<f64>() {
                    let perm =
                        function::gamma::gamma(n + 1.0) / function::gamma::gamma(n - r + 1.0);
                    let comb = perm / function::gamma::gamma(r + 1.0);
                    self.strings[2] = (perm.round() as usize).to_string();
                    self.strings[3] = (comb.round() as usize).to_string();
                }
            }
        }
        ui.num_box("Permutations", &mut (self.strings[2].clone()));
        ui.num_box("Combinations", &mut (self.strings[3].clone()));
        resp
    }
}

#[crunch_fill]
#[derive(Clone)]
pub(crate) struct SampleProbFin {
    pop_size: usize,
    sample_size: usize,
    correct: f64,
    mean: f64,
    sd: f64,
    sample_sd: f64,
    target_mean: Constr<f64>,
    prob: Constr<f64>,
}

impl Default for SampleProbFin {
    fn default() -> Self {
        Self {
            pop_size: 20,
            sample_size: 1,
            correct: 1.0,
            mean: 0.0,
            sd: 1.0,
            target_mean: Constr::LE(0.0),
            strings: [
                20.to_string(),
                1.to_string(),
                1.0.to_string(),
                0.0.to_string(),
                1.0.to_string(),
                1.0.to_string(),
                "<=0.0".to_string(),
                0.5.to_string(),
                "".to_string(),
            ],
            prob: Constr::EQ(0.5),
            sample_sd: 1.0,
        }
    }
}

impl Widget for &mut SampleProbFin {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let mut resp = ui.num_box("Population Size", &mut self.strings[0]);
        resp = resp.union(ui.num_box("Sample Size", &mut self.strings[1]));
        if resp.changed() {
            self.vfill();
            //eprintln!("{} {}",self.pop_size, self.sample_size);
            self.correct =
                (self.pop_size as f64 - self.sample_size as f64) / (self.pop_size as f64 - 1.0);
            //eprintln!("{}", self.correct);
            self.strings[2] = self.correct.to_string();
        }
        ui.horizontal(|ui| {
            ui.label("Correction");
            ui.text_edit_singleline(&mut self.correct.to_string());
        });
        resp = resp.union(ui.num_box("Population Mean", &mut self.strings[3]));
        resp = resp.union(ui.num_box("Population SD", &mut self.strings[4]));
        if resp.changed() {
            self.vfill();
            self.sample_sd = (self.sd * self.sd * self.correct / (self.sample_size as f64)).sqrt();
            self.strings[5] = self.sample_sd.to_string();
        }
        ui.horizontal(|ui| {
            ui.label("Sample SD");
            ui.text_edit_singleline(&mut self.sample_sd.to_string());
        });
        resp = resp.union(ui.num_box("Sample Mean", &mut self.strings[6]));
        ui.horizontal(|ui| {
            ui.label("Prob");
            ui.text_edit_singleline(&mut (self.strings[7].clone()))
        });
        ui.label(RichText::new(&self.strings[8]).color(Color32::DARK_RED));
        if resp.changed() {
            self.vfill();
            //self.strings[4] = self.comp.to_string();
            match Normal::new(self.mean, self.sample_sd) {
                Ok(n) => {
                    let fill = match self.target_mean {
                        Constr::GE(x) | Constr::GT(x) => 1.0 - n.cdf(x),
                        Constr::LE(x) | Constr::LT(x) => n.cdf(x),
                        Constr::EQ(_) | Constr::NE(_) => {
                            self.prob = Constr::None;
                            self.strings[7] = "".to_string();
                            self.strings[8] =
                                "Cannot use exact in a continuous distribution.".to_string();
                            return resp;
                        }
                        Constr::In(a, b) => n.cdf(b) - n.cdf(a),
                        Constr::Out(a, b) => 1.0 - n.cdf(b) + n.cdf(a),
                        _ => {
                            self.prob = Constr::None;
                            self.strings[7] = "".to_string();
                            self.strings[8] = "Must set target mean.".to_string();
                            return resp;
                        }
                    };
                    self.prob = Constr::EQ(fill);
                    self.strings[7] = fill.to_string();
                    self.strings[8] = "".to_string();
                }
                Err(e) => {
                    self.prob = Constr::None;
                    self.strings[8] = e.to_string();
                }
            }
        }
        resp
    }
}

#[crunch_fill]
#[derive(Clone)]
struct Calc {
    field: Expr,
}

impl Default for Calc {
    fn default() -> Self {
        Self { field: "1+2".parse().unwrap(), strings: ["1+2".to_owned(), "".to_owned()] }
    }
}

impl Widget for &mut Calc {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let resp = ui.num_box("", &mut self.strings[0]);
        if resp.changed() {
            self.vfill();
            self.strings[1] = match self.field.eval() {
                Ok(n) => n.to_string(),
                Err(e) => e.to_string(),
            }
        }
        ui.num_box("", &mut self.strings[1].clone());
        resp
    }
}

#[crunch_fill]
#[derive(Clone)]
struct Cheby {
    mean: f64,
    sd: f64,
    sample_size: usize,
    deviation: f64,
}

impl Default for Cheby {
    fn default() -> Self {
        Self { mean: 0.0, sd: 1.0, sample_size: 25, deviation: 1.5, strings: [
            "0.0".to_string(),
            "1.0".to_string(),
            "25".to_string(),
            "1.5".to_string(),
            "".to_string(),
        ],
        }
    }
}

impl Widget for &mut Cheby {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let mut resp = ui.num_box("mean", &mut self.strings[0]);
        resp = resp.union(ui.num_box("sd", &mut self.strings[1]));
        resp = resp.union(ui.num_box("sample size", &mut self.strings[2]));
        resp = resp.union(ui.num_box("deviation", &mut self.strings[3]));
        if resp.changed() {
            self.vfill();
            let ch = self.sd*self.sd/(self.sample_size as f64)/self.deviation/self.deviation;
            self.strings[4] = ch.to_string();
        }
        ui.num_box("", &mut self.strings[4].clone());
        resp
    }
}
