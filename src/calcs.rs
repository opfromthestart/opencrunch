use std::rc::Rc;

use egui::{gui_zoom::zoom_with_keyboard_shortcuts, Color32, Response, RichText, Ui, Widget};
use meval::Expr;
use opencrunch_derive::crunch_fill;
use statrs::{
    distribution::{ChiSquared, ContinuousCDF, FisherSnedecor, Normal, StudentsT},
    function,
};

use crate::{empty_resp, Constr, GridNumBox, NumBox};

#[derive(Default, Clone)]
enum Calcs {
    #[default]
    None,
    SampInf(SampleProbInf),
    SampFin(SampleProbFin),
    Comb(Comb),
    Calc(Calc),
    Cheby(Cheby),
    ZOneStats(ZOneStats),
    TOneStats(TOneStats),
    ZTwoStats(ZTwoStats),
    TTwoStats(TTwoStats),
    VarOneStats(VarOneStats),
    VarTwoStats(VarTwoStats),
    KStats(KStats),
    SampleStat(SampleStat),
    RCTable(RCTable),
}

#[derive(Default)]
pub(crate) struct OpenCrunchCalcs {
    sample: Calcs,
}

impl Widget for &mut OpenCrunchCalcs {
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
            if ui.button("Sample Stats").clicked() {
                self.sample = Calcs::SampleStat(SampleStat::default());
            }
        });
        ui.horizontal(|ui| {
            if ui.button("Z Stats").clicked() {
                self.sample = Calcs::ZOneStats(ZOneStats::default());
            }
            if ui.button("Z Stats 2").clicked() {
                self.sample = Calcs::ZTwoStats(ZTwoStats::default());
            }
            if ui.button("T Stats").clicked() {
                self.sample = Calcs::TOneStats(TOneStats::default());
            }
            if ui.button("T Stats 2").clicked() {
                self.sample = Calcs::TTwoStats(TTwoStats::default());
            }
            if ui.button("Var Stats").clicked() {
                self.sample = Calcs::VarOneStats(VarOneStats::default());
            }
            if ui.button("Var 2 Stats").clicked() {
                self.sample = Calcs::VarTwoStats(VarTwoStats::default());
            }
            if ui.button("K Stats").clicked() {
                self.sample = Calcs::KStats(KStats::default());
            }
            if ui.button("RxC Table").clicked() {
                self.sample = Calcs::RCTable(RCTable::default());
            }
        });

        match &mut self.sample {
            Calcs::None => empty_resp(ui),
            Calcs::SampInf(sam) => ui.add(sam),
            Calcs::Comb(c) => ui.add(c),
            Calcs::SampFin(sam) => ui.add(sam),
            Calcs::Calc(calc) => ui.add(calc),
            Calcs::Cheby(cheb) => ui.add(cheb),
            Calcs::ZOneStats(z) => ui.add(z),
            Calcs::TOneStats(t) => ui.add(t),
            Calcs::ZTwoStats(z) => ui.add(z),
            Calcs::TTwoStats(t) => ui.add(t),
            Calcs::VarOneStats(v) => ui.add(v),
            Calcs::VarTwoStats(v) => ui.add(v),
            Calcs::KStats(k) => ui.add(k),
            Calcs::SampleStat(s) => ui.add(s),
            Calcs::RCTable(r) => ui.add(r),
        }
    }
}

impl ToString for OpenCrunchCalcs {
    fn to_string(&self) -> String {
        match self.sample {
            Calcs::None => "OpenCrunch - Calcs".to_owned(),
            Calcs::SampInf(_) => "OpenCrunch - Calcs - Sample".to_owned(),
            Calcs::SampFin(_) => "OpenCrunch - Calcs - Sample Finite".to_owned(),
            Calcs::Comb(_) => "OpenCrunch - Calcs - Combinatorics".to_owned(),
            Calcs::Calc(_) => "OpenCrunch - Calcs - Calculator".to_owned(),
            Calcs::Cheby(_) => "OpenCrunch - Calcs - Chebyshev".to_owned(),
            Calcs::ZOneStats(_) => "OpenCrunch - Calcs - Z Stats".to_owned(),
            Calcs::TOneStats(_) => "OpenCrunch - Calcs - T Stats".to_owned(),
            Calcs::ZTwoStats(_) => "OpenCrunch - Calcs - 2 Z Stats".to_owned(),
            Calcs::TTwoStats(_) => "OpenCrunch - Calcs - 2 T Stats".to_owned(),
            Calcs::VarOneStats(_) => "OpenCrunch - Calcs - Var Stats".to_owned(),
            Calcs::VarTwoStats(_) => "OpenCrunch - Calcs - 2 Var Stats".to_owned(),
            Calcs::KStats(_) => "OpenCrunch - Calcs - K Stats".to_owned(),
            Calcs::RCTable(_) => "OpenCrunch - Calcs - RxC Table".to_owned(),
            Calcs::SampleStat(_) => "OpenCrunch - Calcs - Sample Stats".to_owned(),
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
        Self {
            field: "1+2".parse().unwrap(),
            strings: ["1+2".to_owned(), "".to_owned()],
        }
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
        Self {
            mean: 0.0,
            sd: 1.0,
            sample_size: 25,
            deviation: 1.5,
            strings: [
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
            let ch =
                self.sd * self.sd / (self.sample_size as f64) / self.deviation / self.deviation;
            self.strings[4] = ch.to_string();
        }
        ui.num_box("", &mut self.strings[4].clone());
        resp
    }
}

#[crunch_fill]
#[derive(Clone)]
pub(crate) struct ZOneStats {
    sample_mean: Expr,
    sample_dev: Expr,
    sample_size: usize,
    confidence: f32,
    interval: Constr<f32>,
    hypothesis: Constr<f32>,
    pval: f32,
}

impl Default for ZOneStats {
    fn default() -> Self {
        Self {
            sample_mean: "0.0".parse().unwrap(),
            sample_dev: "1.0".parse().unwrap(),
            sample_size: 30,
            strings: [
                "0.0".to_string(),
                "1.0".to_string(),
                "30".to_string(),
                "0.95".to_string(),
                "[-1.96, 1.96]".to_string(),
                "!=0.0".to_string(),
                "".to_string(),
                "".to_string(),
            ],
            confidence: 0.95,
            interval: Constr::In(-1.96, 1.96),
            hypothesis: Constr::LENone,
            pval: 0.05,
        }
    }
}

impl Widget for &mut ZOneStats {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let mut resp = ui.num_box("mean", &mut self.strings[0]);
        resp = resp.union(ui.num_box("sd", &mut self.strings[1]));
        resp = resp.union(ui.num_box("sample size", &mut self.strings[2]));
        resp = resp.union(ui.num_box("confidence", &mut self.strings[3]));
        if resp.changed() {
            self.vfill();
            let mut f = || {
                let Ok(mean) = self.sample_mean.eval() else {
                self.strings[7] = "Mean is invalid".to_owned();
                return;
            };
                let Ok(dev) = self.sample_dev.eval() else {
                self.strings[7] = "Standard deviation is invalid".to_owned();
                return;
            };

                let std_err = dev / (self.sample_size as f64).sqrt();

                let Ok(n) = Normal::new(mean, std_err) else {
                self.strings[7] = "Not a valid normal distr".to_string();
                return;
            };

                let int_l = n.inverse_cdf((1.0 - self.confidence as f64) / 2.0);
                let int_h = n.inverse_cdf((1.0 + self.confidence as f64) / 2.0);

                self.interval = Constr::In(int_l as f32, int_h as f32);

                self.strings[4] = self.interval.to_string();
                self.strings[7].clear();
            };
            f();
        }
        ui.num_box("", &mut self.strings[4].clone());
        ui.label("Hypothesis");
        resp = resp.union(ui.num_box("H1", &mut self.strings[5]));
        if resp.changed() {
            self.vfill();
            let mut f = || {
                let Ok(mean) = self.sample_mean.eval() else {
                self.strings[7] = "Mean is invalid".to_owned();
                return;
            };
                let Ok(dev) = self.sample_dev.eval() else {
                self.strings[7] = "Standard deviation is invalid".to_owned();
                return;
            };

                let std_err = dev / (self.sample_size as f64).sqrt();

                let Ok(n) = Normal::new(mean, std_err) else {
                self.strings[7] = "Not a valid normal distr".to_string();
                return;
            };

                self.pval = match self.hypothesis {
                    Constr::GE(v) | Constr::GT(v) => n.cdf(v as f64),
                    Constr::LE(v) | Constr::LT(v) => 1. - n.cdf(v as f64),
                    Constr::NE(v) => {
                        if mean > v as f64 {
                            2.0 * n.cdf(v as f64)
                        } else {
                            2.0 - 2.0 * n.cdf(v as f64)
                        }
                    }
                    _ => {
                        self.strings[7] = "Not valid hypothesis".to_owned();
                        return;
                    }
                } as f32;

                self.strings[6] = self.pval.to_string();
                self.strings[7].clear();
            };
            f();
        }
        ui.num_box("", &mut self.strings[6].clone());
        ui.label(&self.strings[7]);
        resp
    }
}

#[crunch_fill]
#[derive(Clone)]
pub(crate) struct TOneStats {
    sample_mean: Expr,
    sample_dev: Expr,
    sample_size: usize,
    confidence: f32,
    interval: Constr<f32>,
    hypothesis: Constr<f32>,
    pval: f32,
}

impl Default for TOneStats {
    fn default() -> Self {
        Self {
            sample_mean: "0.0".parse().unwrap(),
            sample_dev: "1.0".parse().unwrap(),
            sample_size: 30,
            strings: [
                "0.0".to_string(),
                "1.0".to_string(),
                "30".to_string(),
                "0.95".to_string(),
                "[-1.96, 1.96]".to_string(),
                "!=0.0".to_string(),
                "".to_string(),
                "".to_string(),
            ],
            confidence: 0.95,
            interval: Constr::In(-1.96, 1.96),
            hypothesis: Constr::NENone,
            pval: 0.05,
        }
    }
}

impl Widget for &mut TOneStats {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let mut resp = ui.num_box("mean", &mut self.strings[0]);
        resp = resp.union(ui.num_box("sd", &mut self.strings[1]));
        resp = resp.union(ui.num_box("sample size", &mut self.strings[2]));
        resp = resp.union(ui.num_box("confidence", &mut self.strings[3]));
        if resp.changed() {
            self.vfill();
            let mut f = || {
                let Ok(mean) = self.sample_mean.eval() else {
                    self.strings[7] = "Mean is invalid".to_owned();
                    return;
                };
                let Ok(dev) = self.sample_dev.eval() else {
                    self.strings[7] = "Standard deviation is invalid".to_owned();
                    return;
                };

                let std_err = dev / (self.sample_size as f64).sqrt();

                let Ok(n) = StudentsT::new(mean, std_err, self.sample_size as f64 - 1.0) else {
                    self.strings[7] = "Not a valid T distr".to_string();
                    return;
                };

                let int_l = n.inverse_cdf((1.0 - self.confidence as f64) / 2.0);
                let int_h = n.inverse_cdf((1.0 + self.confidence as f64) / 2.0);

                self.interval = Constr::In(int_l as f32, int_h as f32);

                self.strings[4] = self.interval.to_string();
                self.strings[7].clear();
            };
            f();
        }
        ui.num_box("", &mut self.strings[4].clone());
        ui.label("Hypothesis");
        resp = resp.union(ui.num_box("H1", &mut self.strings[5]));
        if resp.changed() {
            self.vfill();
            let mut f = || {
                let Ok(mean) = self.sample_mean.eval() else {
                    self.strings[7] = "Mean is invalid".to_owned();
                    return;
                };
                let Ok(dev) = self.sample_dev.eval() else {
                    self.strings[7] = "Standard deviation is invalid".to_owned();
                    return;
                };

                let std_err = dev / (self.sample_size as f64).sqrt();

                let Ok(n) = StudentsT::new(mean, std_err, self.sample_size as f64 - 1.0) else {
                    self.strings[7] = "Not a valid T distr".to_string();
                    return;
                };

                self.pval = match self.hypothesis {
                    Constr::GE(v) | Constr::GT(v) => n.cdf(v as f64),
                    Constr::LE(v) | Constr::LT(v) => 1. - n.cdf(v as f64),
                    Constr::NE(v) => {
                        if mean > v as f64 {
                            2.0 * n.cdf(v as f64)
                        } else {
                            2.0 - 2.0 * n.cdf(v as f64)
                        }
                    }
                    _ => {
                        self.strings[7] = "Not valid hypothesis".to_owned();
                        return;
                    }
                } as f32;

                self.strings[6] = self.pval.to_string();
                self.strings[7].clear();
            };
            f();
        }
        ui.num_box("", &mut self.strings[6].clone());
        ui.label(&self.strings[7]);
        resp
    }
}

#[crunch_fill]
#[derive(Clone)]
pub(crate) struct ZTwoStats {
    sample_mean_1: Expr,
    sample_dev_1: Expr,
    sample_size_1: usize,
    sample_mean_2: Expr,
    sample_dev_2: Expr,
    sample_size_2: usize,
    confidence: f32,
    interval: Constr<f32>,
    hypothesis: Constr<f32>,
    pval: f32,
}

impl Default for ZTwoStats {
    fn default() -> Self {
        Self {
            sample_mean_1: "0.0".parse().unwrap(),
            sample_dev_1: "1.0".parse().unwrap(),
            sample_size_1: 30,
            sample_mean_2: "0.0".parse().unwrap(),
            sample_dev_2: "1.0".parse().unwrap(),
            sample_size_2: 30,
            confidence: 0.95,
            interval: Constr::None,
            strings: [
                "0.0".to_string(),
                "1.0".to_string(),
                "30".to_string(),
                "0.0".to_string(),
                "1.0".to_string(),
                "30".to_string(),
                "0.95".to_string(),
                "".to_string(),
                "!=0.0".to_string(),
                "".to_string(),
                "".to_string(),
            ],
            hypothesis: Constr::NE(0.0),
            pval: 0.05,
        }
    }
}

impl Widget for &mut ZTwoStats {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let mut resp = ui.num_box("mean 1", &mut self.strings[0]);
        resp = resp.union(ui.num_box("sd 1", &mut self.strings[1]));
        resp = resp.union(ui.num_box("sample size 1", &mut self.strings[2]));
        resp = resp.union(ui.num_box("mean 2", &mut self.strings[3]));
        resp = resp.union(ui.num_box("sd 2", &mut self.strings[4]));
        resp = resp.union(ui.num_box("sample size 2", &mut self.strings[5]));
        resp = resp.union(ui.num_box("confidence", &mut self.strings[6]));
        if resp.changed() {
            self.vfill();
            let mut f = || {
                let Ok(mean1) = self.sample_mean_1.eval() else {
                    self.strings[10] = "Mean 1 is invalid".to_owned();
                    return;
                };
                let Ok(dev1) = self.sample_dev_1.eval() else {
                    self.strings[10] = "Standard deviation 1 is invalid".to_owned();
                    return;
                };
                let Ok(mean2) = self.sample_mean_2.eval() else {
                    self.strings[10] = "Mean 1 is invalid".to_owned();
                    return;
                };
                let Ok(dev2) = self.sample_dev_2.eval() else {
                    self.strings[10] = "Standard deviation 1 is invalid".to_owned();
                    return;
                };

                let std_err = (dev1 * dev1 / self.sample_size_1 as f64
                    + dev2 * dev2 / self.sample_size_2 as f64)
                    .sqrt();

                let Ok(n) = Normal::new(mean1 - mean2, std_err) else {
                    self.strings[10] = "Not a valid Normal distr".to_string();
                    return;
                };

                let int_l = n.inverse_cdf((1.0 - self.confidence as f64) / 2.0);
                let int_h = n.inverse_cdf((1.0 + self.confidence as f64) / 2.0);

                self.interval = Constr::In(int_l as f32, int_h as f32);

                self.strings[7] = self.interval.to_string();
                self.strings[10].clear();
            };
            f();
        }
        ui.num_box("", &mut self.strings[7].clone());
        ui.label("Hypothesis");
        resp = resp.union(ui.num_box("H1", &mut self.strings[8]));
        if resp.changed() {
            self.vfill();
            let mut f = || {
                let Ok(mean1) = self.sample_mean_1.eval() else {
                    self.strings[10] = "Mean 1 is invalid".to_owned();
                    return;
                };
                let Ok(dev1) = self.sample_dev_1.eval() else {
                    self.strings[10] = "Standard deviation 1 is invalid".to_owned();
                    return;
                };
                let Ok(mean2) = self.sample_mean_2.eval() else {
                    self.strings[10] = "Mean 1 is invalid".to_owned();
                    return;
                };
                let Ok(dev2) = self.sample_dev_2.eval() else {
                    self.strings[10] = "Standard deviation 1 is invalid".to_owned();
                    return;
                };

                let std_err = (dev1 * dev1 / self.sample_size_1 as f64
                    + dev2 * dev2 / self.sample_size_2 as f64)
                    .sqrt();

                let Ok(n) = Normal::new(mean1 - mean2, std_err) else {
                    self.strings[10] = "Not a valid Normal distr".to_string();
                    return;
                };

                self.pval = match self.hypothesis {
                    Constr::GE(v) | Constr::GT(v) => n.cdf(v as f64),
                    Constr::LE(v) | Constr::LT(v) => 1. - n.cdf(v as f64),
                    Constr::NE(v) => {
                        if mean1 - mean2 > v as f64 {
                            2.0 * n.cdf(v as f64)
                        } else {
                            2.0 - 2.0 * n.cdf(v as f64)
                        }
                    }
                    _ => {
                        self.strings[10] = "Not valid hypothesis".to_owned();
                        return;
                    }
                } as f32;

                self.strings[9] = self.pval.to_string();
                self.strings[10].clear();
            };
            f();
        }
        ui.num_box("", &mut self.strings[9].clone());
        ui.label(&self.strings[10]);
        resp
    }
}

#[crunch_fill]
#[derive(Clone)]
pub(crate) struct TTwoStats {
    sample_mean_1: Expr,
    sample_dev_1: Expr,
    sample_size_1: usize,
    sample_mean_2: Expr,
    sample_dev_2: Expr,
    sample_size_2: usize,
    confidence: f32,
    interval: Constr<f32>,
    hypothesis: Constr<f32>,
    pval: f32,
}

impl Default for TTwoStats {
    fn default() -> Self {
        Self {
            sample_mean_1: "0.0".parse().unwrap(),
            sample_dev_1: "1.0".parse().unwrap(),
            sample_size_1: 30,
            sample_mean_2: "0.0".parse().unwrap(),
            sample_dev_2: "1.0".parse().unwrap(),
            sample_size_2: 30,
            confidence: 0.95,
            interval: Constr::None,
            strings: [
                "0.0".to_string(),
                "1.0".to_string(),
                "30".to_string(),
                "0.0".to_string(),
                "1.0".to_string(),
                "30".to_string(),
                "0.95".to_string(),
                "".to_string(),
                "!=0.0".to_string(),
                "".to_string(),
                "".to_string(),
            ],
            hypothesis: Constr::NE(0.0),
            pval: 0.05,
        }
    }
}

impl Widget for &mut TTwoStats {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let mut resp = ui.num_box("mean 1", &mut self.strings[0]);
        resp = resp.union(ui.num_box("sd 1", &mut self.strings[1]));
        resp = resp.union(ui.num_box("sample size 1", &mut self.strings[2]));
        resp = resp.union(ui.num_box("mean 2", &mut self.strings[3]));
        resp = resp.union(ui.num_box("sd 2", &mut self.strings[4]));
        resp = resp.union(ui.num_box("sample size 2", &mut self.strings[5]));
        resp = resp.union(ui.num_box("confidence", &mut self.strings[6]));
        if resp.changed() {
            self.vfill();
            let mut f = || {
                let Ok(mean1) = self.sample_mean_1.eval() else {
                    self.strings[10] = "Mean 1 is invalid".to_owned();
                    return;
                };
                let Ok(dev1) = self.sample_dev_1.eval() else {
                    self.strings[10] = "Standard deviation 1 is invalid".to_owned();
                    return;
                };
                let Ok(mean2) = self.sample_mean_2.eval() else {
                    self.strings[10] = "Mean 1 is invalid".to_owned();
                    return;
                };
                let Ok(dev2) = self.sample_dev_2.eval() else {
                    self.strings[10] = "Standard deviation 1 is invalid".to_owned();
                    return;
                };

                let std_err = (dev1 * dev1 / self.sample_size_1 as f64
                    + dev2 * dev2 / self.sample_size_2 as f64)
                    .sqrt();
                let a = dev1 * dev1 / self.sample_size_1 as f64;
                let b = dev2 * dev2 / self.sample_size_2 as f64;
                let df = (a + b) * (a + b)
                    / (a * a / (self.sample_size_1 as f64 - 1.0)
                        + b * b / (self.sample_size_2 as f64 - 1.0));

                let Ok(n) = StudentsT::new(mean1 - mean2, std_err, df) else {
                    self.strings[10] = "Not a valid T distr".to_string();
                    return;
                };

                let int_l = n.inverse_cdf((1.0 - self.confidence as f64) / 2.0);
                let int_h = n.inverse_cdf((1.0 + self.confidence as f64) / 2.0);

                self.interval = Constr::In(int_l as f32, int_h as f32);

                self.strings[7] = self.interval.to_string();
                self.strings[10].clear();
            };
            f();
        }
        ui.num_box("", &mut self.strings[7].clone());
        ui.label("Hypothesis");
        resp = resp.union(ui.num_box("H1", &mut self.strings[8]));
        if resp.changed() {
            self.vfill();
            let mut f = || {
                let Ok(mean1) = self.sample_mean_1.eval() else {
                    self.strings[10] = "Mean 1 is invalid".to_owned();
                    return;
                };
                let Ok(dev1) = self.sample_dev_1.eval() else {
                    self.strings[10] = "Standard deviation 1 is invalid".to_owned();
                    return;
                };
                let Ok(mean2) = self.sample_mean_2.eval() else {
                    self.strings[10] = "Mean 1 is invalid".to_owned();
                    return;
                };
                let Ok(dev2) = self.sample_dev_2.eval() else {
                    self.strings[10] = "Standard deviation 1 is invalid".to_owned();
                    return;
                };

                let std_err = (dev1 * dev1 / self.sample_size_1 as f64
                    + dev2 * dev2 / self.sample_size_2 as f64)
                    .sqrt();
                let a = dev1 * dev1 / self.sample_size_1 as f64;
                let b = dev2 * dev2 / self.sample_size_2 as f64;
                let df = (a + b) * (a + b)
                    / (a * a / (self.sample_size_1 as f64 - 1.0)
                        + b * b / (self.sample_size_2 as f64 - 1.0));

                let Ok(n) = StudentsT::new(mean1 - mean2, std_err, df) else {
                    self.strings[10] = "Not a valid Normal distr".to_string();
                    return;
                };

                self.pval = match self.hypothesis {
                    Constr::GE(v) | Constr::GT(v) => n.cdf(v as f64),
                    Constr::LE(v) | Constr::LT(v) => 1. - n.cdf(v as f64),
                    Constr::NE(v) => {
                        if mean1 - mean2 > v as f64 {
                            2.0 * n.cdf(v as f64)
                        } else {
                            2.0 - 2.0 * n.cdf(v as f64)
                        }
                    }
                    _ => {
                        self.strings[10] = "Not valid hypothesis".to_owned();
                        return;
                    }
                } as f32;

                self.strings[9] = self.pval.to_string();
                self.strings[10].clear();
            };
            f();
        }
        ui.num_box("", &mut self.strings[9].clone());
        ui.label(&self.strings[10]);
        resp
    }
}

#[crunch_fill]
#[derive(Clone)]
pub(crate) struct VarOneStats {
    sample_dev: Expr,
    sample_size: usize,
    confidence: f32,
    intervalvar: Constr<f32>,
    intervaldev: Constr<f32>,
    hypothesis: Constr<f32>,
    pval: f32,
}

impl Default for VarOneStats {
    fn default() -> Self {
        Self {
            sample_dev: "1.0".parse().unwrap(),
            sample_size: 30,
            strings: [
                "1.0".to_string(),
                "30".to_string(),
                "0.95".to_string(),
                "[0.63, 1.81]".to_string(),
                "[0.80, 1.34]".to_string(),
                "!=1".to_string(),
                "".to_string(),
                "".to_string(),
            ],
            confidence: 0.95,
            intervalvar: Constr::In(0.63, 1.81),
            intervaldev: Constr::In(0.80, 1.34),
            hypothesis: Constr::NE(1.0),
            pval: 0.05,
        }
    }
}

impl Widget for &mut VarOneStats {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let mut resp = ui.num_box("sd", &mut self.strings[0]);
        resp = resp.union(ui.num_box("sample size", &mut self.strings[1]));
        resp = resp.union(ui.num_box("confidence", &mut self.strings[2]));
        if resp.changed() {
            self.vfill();
            let mut f = || {
                let Ok(dev) = self.sample_dev.eval() else {
                self.strings[7] = "Standard deviation is invalid".to_owned();
                return;
            };
                let dev = dev as f32;

                let free = (self.sample_size - 1) as f32;
                let Ok(n) = ChiSquared::new(free as f64) else {
                self.strings[7] = "Not a valid Chi Squared distr".to_string();
                return;
            };

                let int_l = n.inverse_cdf((1.0 - self.confidence as f64) / 2.0) as f32;
                let int_h = n.inverse_cdf((1.0 + self.confidence as f64) / 2.0) as f32;

                let err = free * dev * dev;
                self.intervalvar = Constr::In(err / int_h, err / int_l);
                self.intervaldev = Constr::In((err / int_h).sqrt(), (err / int_l).sqrt());

                self.strings[3] = self.intervalvar.to_string();
                self.strings[4] = self.intervaldev.to_string();
                self.strings[7].clear();
            };
            f();
        }
        ui.num_box("Var", &mut self.strings[3].clone());
        ui.num_box("SD ", &mut self.strings[4].clone());
        ui.label("Hypothesis");
        resp = resp.union(ui.num_box("H1: sd", &mut self.strings[5]));
        if resp.changed() {
            self.vfill();
            let mut f = || {
                let Ok(dev) = self.sample_dev.eval() else {
                    self.strings[7] = "Standard deviation is invalid".to_owned();
                    return;
                };

                let free = (self.sample_size - 1) as f32;
                let Ok(n) = ChiSquared::new(free as f64) else {
                    self.strings[7] = "Not a valid Chi Squared distr".to_string();
                    return;
                };

                let err = free * (dev * dev) as f32;

                self.pval = match self.hypothesis {
                    Constr::GE(v) | Constr::GT(v) => 1. - n.cdf((err / v / v) as f64),
                    Constr::LE(v) | Constr::LT(v) => n.cdf((err / v / v) as f64),
                    Constr::NE(v) => {
                        if dev < v as f64 {
                            2.0 * n.cdf((err / v / v) as f64)
                        } else {
                            2.0 - 2.0 * n.cdf((err / v / v) as f64)
                        }
                    }
                    _ => {
                        self.strings[7] = "Not valid hypothesis".to_owned();
                        return;
                    }
                } as f32;

                self.strings[6] = self.pval.to_string();
                self.strings[7].clear();
            };
            f();
        }
        ui.num_box("", &mut self.strings[6].clone());
        ui.label(&self.strings[7]);
        resp
    }
}

#[crunch_fill]
#[derive(Clone)]
pub(crate) struct VarTwoStats {
    sample_dev_1: Expr,
    sample_size_1: usize,
    sample_dev_2: Expr,
    sample_size_2: usize,
    confidence: f32,
    intervalvar: Constr<f32>,
    intervaldev: Constr<f32>,
    hypothesis: Constr<f32>,
    pval: f32,
}

impl Default for VarTwoStats {
    fn default() -> Self {
        Self {
            sample_dev_1: "1.0".parse().unwrap(),
            sample_size_1: 30,
            sample_dev_2: "1.2".parse().unwrap(),
            sample_size_2: 30,
            strings: [
                "1.0".to_string(),
                "30".to_string(),
                "1.2".to_string(),
                "30".to_string(),
                "0.95".to_string(),
                "[0.33, 1.46]".to_string(),
                "[0.57, 1.21]".to_string(),
                "!=1".to_string(),
                "".to_string(),
                "".to_string(),
            ],
            confidence: 0.95,
            intervalvar: Constr::In(0.63, 1.81),
            intervaldev: Constr::In(0.80, 1.34),
            hypothesis: Constr::NE(1.0),
            pval: 0.05,
        }
    }
}

impl Widget for &mut VarTwoStats {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let mut resp = ui.num_box("sd 1", &mut self.strings[0]);
        resp = resp.union(ui.num_box("sample size 1", &mut self.strings[1]));
        resp = resp.union(ui.num_box("sd 2", &mut self.strings[2]));
        resp = resp.union(ui.num_box("sample size 2", &mut self.strings[3]));
        resp = resp.union(ui.num_box("confidence", &mut self.strings[4]));
        if resp.changed() {
            self.vfill();
            let mut f = || {
                let Ok(dev1) = self.sample_dev_1.eval() else {
                self.strings[9] = "Standard deviation 1 is invalid".to_owned();
                return;
            };
                let dev1 = dev1 as f32;

                let Ok(dev2) = self.sample_dev_2.eval() else {
                self.strings[9] = "Standard deviation 2 is invalid".to_owned();
                return;
            };
                let dev2 = dev2 as f32;

                let free1 = (self.sample_size_1 - 1) as f64;
                let free2 = (self.sample_size_2 - 1) as f64;
                let Ok(n) = FisherSnedecor::new(free1, free2) else {
                self.strings[9] = "Not a valid F distr".to_string();
                return;
            };

                let int_l = n.inverse_cdf((1.0 - self.confidence as f64) / 2.0) as f32;
                let int_h = n.inverse_cdf((1.0 + self.confidence as f64) / 2.0) as f32;

                let err = dev1 * dev1 / dev2 / dev2;
                self.intervalvar = Constr::In(err / int_h, err / int_l);
                self.intervaldev = Constr::In((err / int_h).sqrt(), (err / int_l).sqrt());

                self.strings[5] = self.intervalvar.to_string();
                self.strings[6] = self.intervaldev.to_string();
                self.strings[9].clear();
            };
            f();
        }
        ui.num_box("Var", &mut self.strings[5].clone());
        ui.num_box("SD ", &mut self.strings[6].clone());
        ui.label("Hypothesis");
        resp = resp.union(ui.num_box("H1 sd1/sd2", &mut self.strings[7]));
        if resp.changed() {
            self.vfill();
            let mut f = || {
                let Ok(dev1) = self.sample_dev_1.eval() else {
                self.strings[9] = "Standard deviation 1 is invalid".to_owned();
                return;
            };

                let Ok(dev2) = self.sample_dev_2.eval() else {
                self.strings[9] = "Standard deviation 2 is invalid".to_owned();
                return;
            };

                let free1 = (self.sample_size_1 - 1) as f64;
                let free2 = (self.sample_size_2 - 1) as f64;
                let Ok(n) = FisherSnedecor::new(free1, free2) else {
                self.strings[9] = "Not a valid F distr".to_string();
                return;
            };

                let err = dev1 * dev1 / dev2 / dev2;

                self.pval = match self.hypothesis {
                    Constr::GE(v) | Constr::GT(v) => 1. - n.cdf(err / (v * v) as f64),
                    Constr::LE(v) | Constr::LT(v) => n.cdf(err / (v * v) as f64),
                    Constr::NE(v) => {
                        if dev1 / dev2 < v as f64 {
                            2.0 * n.cdf(err / (v * v) as f64)
                        } else {
                            2.0 - 2.0 * n.cdf(err / (v * v) as f64)
                        }
                    }
                    _ => {
                        self.strings[9] = "Not valid hypothesis".to_owned();
                        return;
                    }
                } as f32;

                self.strings[8] = self.pval.to_string();
                self.strings[9].clear();
            };
            f();
        }
        ui.num_box("", &mut self.strings[8].clone());
        ui.label(&self.strings[9]);
        resp
    }
}

#[derive(Clone)]
pub(crate) struct KStats {
    sample_means: Vec<Expr>,
    sample_devs: Vec<Expr>,
    sample_sizes: Vec<usize>,
    kstrings: Vec<(String, String, String)>,
    pool_avg: f32,
    hypothesis: Constr<f32>,
    pval: f32,
    strings: [String; 4],
}

impl Default for KStats {
    fn default() -> Self {
        Self {
            sample_means: vec!["0.0".parse().unwrap()],
            sample_devs: vec!["1.0".parse().unwrap()],
            sample_sizes: vec![30],
            pool_avg: 0.0,
            pval: 0.05,
            kstrings: vec![("0.0".to_string(), "1.0".to_string(), "30".to_string())],
            strings: [
                "0.0".to_string(),
                "!=".to_string(),
                "".to_string(),
                "".to_string(),
            ],
            hypothesis: Constr::NENone,
        }
    }
}

impl Widget for &mut KStats {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let rba = ui.button("Add sample");
        if rba.clicked() {
            self.sample_means.push("1.0".parse().unwrap());
            self.sample_devs.push("0.0".parse().unwrap());
            self.sample_sizes.push(30);
            self.kstrings
                .push(("0.0".to_string(), "1.0".to_string(), "30".to_string()))
        }
        let rbr = ui.button("Remove sample");
        if rbr.clicked() {
            self.sample_means.pop();
            self.sample_devs.pop();
            self.sample_sizes.pop();
            self.kstrings.pop();
        }
        let mut resp = rba.union(rbr);
        for (m, d, c) in self.kstrings.iter_mut() {
            resp = resp.union(ui.num_box("mean", m));
            resp = resp.union(ui.num_box("sd", d));
            resp = resp.union(ui.num_box("sample size", c));
        }
        ui.num_box("Pool", &mut self.strings[0].clone());
        resp = resp.union(ui.num_box("Hypothesis", &mut self.strings[1]));
        if resp.changed() {
            self.vfill();
            let mut f = || {
                let means: Vec<_> = self
                    .sample_means
                    .iter()
                    .filter_map(|x| x.eval().ok())
                    .collect();
                if means.len() < self.sample_means.len() {
                    self.strings[3] = "Mean is invalid".to_owned();
                    return;
                };

                let devs: Vec<_> = self
                    .sample_devs
                    .iter()
                    .filter_map(|x| x.eval().ok())
                    .collect();
                if devs.len() < self.sample_devs.len() {
                    self.strings[3] = "Standard deviation is invalid".to_owned();
                    return;
                };

                let pooled = means
                    .iter()
                    .zip(self.sample_sizes.iter())
                    .map(|(m, s)| m * *s as f64)
                    .sum::<f64>()
                    / (self.sample_sizes.iter().sum::<usize>() as f64);

                self.strings[0] = pooled.to_string();

                let free = match self.hypothesis {
                    Constr::NE(_) => self.sample_means.len(),
                    Constr::NENone => self.sample_means.len() - 1,
                    _ => {
                        self.strings[3] = "Not a valid hypothesis".to_string();
                        return;
                    }
                } as f64;
                let Ok(n) = ChiSquared::new(free) else {
                self.strings[3] = "Not a valid Chi squared distr".to_string();
                return;
            };

                if !matches!(self.hypothesis, Constr::NE(_) | Constr::NENone) {
                    self.strings[2] = "Not a valid hypothesis".to_string();
                    return;
                }

                let crit = means
                    .iter()
                    .zip(devs.iter())
                    .zip(self.sample_sizes.iter())
                    .map(|((m, d), s)| match self.hypothesis {
                        Constr::NE(v) => (*m - v as f64).powi(2) / (d * d / (*s as f64)),
                        Constr::NENone => (m - pooled).powi(2) / (d * d / (*s as f64)),
                        _ => {
                            unreachable!("Not a valid hypothesis");
                        }
                    })
                    .sum();

                self.pval = 1.0 - n.cdf(crit) as f32;

                self.strings[2] = self.pval.to_string();
                self.strings[3].clear();
            };
            f();
        }
        ui.num_box("", &mut self.strings[2].clone());
        ui.label(&self.strings[3]);
        resp
    }
}

impl KStats {
    fn vfill(&mut self) {
        if let Ok(val) = self.strings[1].parse() {
            self.hypothesis = val;
        }
        for (((m, d), s), st) in self
            .sample_means
            .iter_mut()
            .zip(self.sample_devs.iter_mut())
            .zip(self.sample_sizes.iter_mut())
            .zip(self.kstrings.iter())
        {
            if let Ok(val) = st.0.parse() {
                *m = val;
            }
            if let Ok(val) = st.1.parse() {
                *d = val;
            }
            if let Ok(val) = st.2.parse() {
                *s = val;
            }
        }
    }
}

#[derive(Clone)]
pub(crate) struct SampleStat {
    sample_vals: Vec<Option<f32>>,
    kstrings: Vec<String>,
    mean: f32,
    var: f32,
    sd: f32,
    strings: [String; 4],
}

impl Default for SampleStat {
    fn default() -> Self {
        Self {
            sample_vals: vec![],
            kstrings: vec![],
            mean: 0.,
            var: 1.,
            sd: 1.,
            strings: [
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            ],
        }
    }
}

impl Widget for &mut SampleStat {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let mut resp = ui.label("Enter values");
        for m in self.kstrings.iter_mut() {
            resp = resp.union(ui.num_box("", m));
        }
        if self.kstrings.is_empty() || self.kstrings.last().unwrap() != "" {
            self.sample_vals.push(None);
            self.kstrings.push("".to_string());
        } else if self.kstrings.len() > 1 && self.kstrings[self.kstrings.len() - 2] == "" {
            self.sample_vals.pop();
            self.kstrings.pop();
        }
        if resp.changed() {
            self.vfill();
            let mut f = || {
                if self.sample_vals.len() < 2 {
                    self.strings[3] = "Not enough values".to_string();
                    return;
                }

                let len = self.sample_vals.iter().filter(|x| x.is_some()).count();
                let mean =
                    self.sample_vals.iter().fold(0., |x, y| x + y.unwrap_or(0.)) / (len as f32);
                let var = self.sample_vals.iter().fold(0., |x, y| {
                    x + y.map(|y| (mean - y).powi(2) as f64).unwrap_or(0.)
                }) / ((len - 1) as f64);

                self.mean = mean as f32;
                self.var = var as f32;
                self.sd = var.sqrt() as f32;

                self.strings[0] = self.mean.to_string();
                self.strings[1] = self.var.to_string();
                self.strings[2] = self.sd.to_string();
                self.strings[3].clear();
            };
            f();
        }
        ui.num_box("Mean", &mut self.strings[0].clone());
        ui.num_box("Var", &mut self.strings[1].clone());
        ui.num_box("SD", &mut self.strings[2].clone());
        ui.label(&self.strings[3]);
        resp
    }
}

impl SampleStat {
    fn vfill(&mut self) {
        for (v, s) in self.sample_vals.iter_mut().zip(self.kstrings.iter()) {
            if let Ok(val) = s.parse::<Expr>() {
                *v = val.eval().ok().map(|x| x as f32);
            } else {
                *v = None;
            }
        }
    }
}

#[derive(Clone)]
pub(crate) struct RCTable {
    sample_vals: Vec<Vec<Option<f32>>>,
    kstrings: Vec<Vec<String>>,
    pval: f32,
    strings: [String; 2],
}

impl Default for RCTable {
    fn default() -> Self {
        Self {
            sample_vals: vec![],
            kstrings: vec![],
            pval: 0.,
            strings: ["".to_string(), "".to_string()],
        }
    }
}

fn get_fill_shape<T>(g: &[Vec<Option<T>>]) -> (usize, usize) {
    let (mut w, mut h) = (0, 0);
    for (i, r) in g.iter().enumerate() {
        for (j, v) in r.iter().enumerate() {
            if v.is_some() {
                w = w.max(j + 1);
                h = h.max(i + 1);
            }
        }
    }
    (w, h)
}

fn get_all_fill<T>(g: &[Vec<Option<T>>], fill: (usize, usize)) -> bool {
    for (i, r) in g.iter().enumerate() {
        if i >= fill.1 {
            continue;
        }
        for (j, v) in r.iter().enumerate() {
            if j >= fill.0 {
                continue;
            }
            if v.is_none() {
                return false;
            }
        }
    }
    true
}

fn get_shape<T>(g: &[Vec<T>]) -> (usize, usize) {
    (g.get(0).map(|x| x.len()).unwrap_or(0), g.len())
}

fn add_row<T: Default>(g: &mut Vec<Vec<T>>) {
    if g.is_empty() {
        g.push(vec![]);
    } else {
        g.push((0..g[0].len()).map(|_| T::default()).collect());
    }
}

fn add_col<T: Default>(g: &mut Vec<Vec<T>>) {
    for r in g.iter_mut() {
        r.push(T::default());
    }
}

fn rem_row<T>(g: &mut Vec<Vec<T>>) {
    g.pop();
}

fn rem_col<T>(g: &mut Vec<Vec<T>>) {
    for r in g.iter_mut() {
        r.pop();
    }
}

impl Widget for &mut RCTable {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let mut resp = ui.label("Enter values, checks independence of rows");
        let respm = resp.clone();
        for mv in self.kstrings.iter_mut() {
            resp = resp.union(
                ui.horizontal(|ui| {
                    let mut resp = respm.clone();
                    for m in mv.iter_mut() {
                        resp = resp.union(ui.grid_num_box(40, m));
                    }
                    resp
                })
                .inner,
            );
        }
        if resp.changed() {
            self.vfill();
        }
        let s = get_shape(&self.sample_vals);
        let fs = get_fill_shape(&self.sample_vals);
        // println!("{:?}, {:?}", s, fs);
        if s.0 == fs.0 {
            add_col(&mut self.kstrings);
            add_col(&mut self.sample_vals);
        }
        if s.1 == fs.1 {
            add_row(&mut self.kstrings);
            add_row(&mut self.sample_vals);
        }
        if s.0 > fs.0 + 1 {
            rem_col(&mut self.kstrings);
            rem_col(&mut self.sample_vals);
        }
        if s.1 > fs.1 + 1 {
            rem_row(&mut self.kstrings);
            rem_row(&mut self.sample_vals);
        }
        if resp.changed() {
            let mut f = || {
                // println!("{:?} {:?}", fs, get_all_fill(&self.sample_vals));

                if !get_all_fill(&self.sample_vals, fs) {
                    self.strings[1] = "Not a filled table".to_string();
                    return;
                }

                let mut pooled = self
                    .sample_vals
                    .iter()
                    .cloned()
                    .reduce(|x, y| {
                        x.into_iter()
                            .zip(y.into_iter())
                            .map(|(x, y)| {
                                let x = x.unwrap_or(0.);
                                let y = y.unwrap_or(0.);
                                Some(x + y)
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap();

                let pooled_row = self
                    .sample_vals
                    .iter()
                    .map(|x| {
                        x.iter().fold(Some(0.), |x, y| {
                            let x = x.unwrap_or(0.);
                            let y = y.unwrap_or(0.);
                            Some(x + y)
                        })
                    })
                    .filter_map(|x| x)
                    .collect::<Vec<_>>();

                if pooled_row.is_empty() {
                    self.strings[1] = "Not large enough table".to_string();
                    return;
                }

                let total = pooled_row.iter().cloned().reduce(|x, y| x + y).unwrap();

                let pooled_p = pooled
                    .into_iter()
                    .map(|x| x.unwrap() / total)
                    .collect::<Vec<_>>();
                let pooled_row_p = pooled_row
                    .into_iter()
                    .map(|x| x / total)
                    .collect::<Vec<_>>();

                let exp = (0..fs.1)
                    .map(|i| {
                        (0..fs.0)
                            .map(|j| total * pooled_p[j] * pooled_row_p[i])
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();

                if fs.0 < 1 || fs.1 < 1 {
                    self.strings[1] = "Not large enough table".to_string();
                    return;
                }
                let free = ((fs.0 - 1) * (fs.1 - 1)) as f64;

                let Ok(n) = ChiSquared::new(free) else {
                self.strings[1] = "Not a valid Chi squared distr".to_string();
                return;
            };

                // println!("{:?}\n{:?}\n", exp, self.sample_vals);

                let crit = exp
                    .iter()
                    .zip(self.sample_vals.iter())
                    .map(|(e, o)| {
                        e.iter()
                            .zip(o.iter())
                            .map(|(e, o)| (e - o.clone().unwrap()).powi(2) / e)
                            .sum::<f32>()
                    })
                    .sum::<f32>() as f64;

                // println!("{crit}");

                self.pval = 1.0 - n.cdf(crit) as f32;

                self.strings[0] = self.pval.to_string();
                self.strings[1].clear();
            };
            f();
        }
        ui.num_box("", &mut self.strings[0].clone());
        ui.label(&self.strings[1]);
        resp
    }
}

impl RCTable {
    fn vfill(&mut self) {
        for (sr, vr) in self.kstrings.iter().zip(self.sample_vals.iter_mut()) {
            for (s, v) in sr.iter().zip(vr.iter_mut()) {
                *v = match s.parse::<f32>() {
                    Ok(v) => Some(v),
                    Err(_) => None,
                };
            }
        }
    }
}
