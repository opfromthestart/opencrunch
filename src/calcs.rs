use egui::{Color32, RichText, Ui, Widget};
use meval::Expr;
use opencrunch_derive::{crunch_fill};
use statrs::{
    distribution::{ContinuousCDF, Normal, StudentsT},
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
    ZOneStats(ZOneStats),
    TOneStats(TOneStats),
    ZTwoStats(ZTwoStats),
    TTwoStats(TTwoStats),
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

#[crunch_fill]
#[derive(Clone)]
pub(crate) struct ZOneStats {
    sample_mean: Expr,
    sample_dev: Expr,
    sample_size: usize,
    confidence: f32,
    interval: Constr<f32>,
}

impl Default for ZOneStats {
    fn default() -> Self {
        Self { sample_mean: "0.0".parse().unwrap(), 
            sample_dev: "1.0".parse().unwrap(), 
            sample_size: 30, 
            strings: [
                "0.0".to_string(),
                "1.0".to_string(),
                "30".to_string(),
                "0.95".to_string(),
                "[-1.96, 1.96]".to_string(),
                "".to_string(),
            ],
            confidence: 0.95,
            interval: Constr::In(-1.96, 1.96), }
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
                self.strings[5] = "Mean is invalid".to_owned();
                return;
            };
            let Ok(dev) = self.sample_dev.eval() else {
                self.strings[5] = "Standard deviation is invalid".to_owned();
                return;
            };

            let std_err = dev / (self.sample_size as f64).sqrt();

            let Ok(n) = Normal::new(mean, std_err) else {
                self.strings[5] = "Not a valid normal distr".to_string();
                return;
            };

            let int_l = n.inverse_cdf((1.0-self.confidence as f64)/2.0);
            let int_h = n.inverse_cdf((1.0+self.confidence as f64)/2.0);

            self.interval = Constr::In(int_l as f32, int_h as f32);

            self.strings[4] = self.interval.to_string();
            self.strings[5].clear();
            };
            f();
        }
        ui.num_box("", &mut self.strings[4].clone());
        ui.label(&self.strings[5]);
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
}

impl Default for TOneStats {
    fn default() -> Self {
        Self { sample_mean: "0.0".parse().unwrap(), 
            sample_dev: "1.0".parse().unwrap(), 
            sample_size: 30, 
            strings: [
                "0.0".to_string(),
                "1.0".to_string(),
                "30".to_string(),
                "0.95".to_string(),
                "[-1.96, 1.96]".to_string(),
                "".to_string(),
            ],
            confidence: 0.95,
            interval: Constr::In(-1.96, 1.96), }
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
                    self.strings[5] = "Mean is invalid".to_owned();
                    return;
                };
                let Ok(dev) = self.sample_dev.eval() else {
                    self.strings[5] = "Standard deviation is invalid".to_owned();
                    return;
                };
    
                let std_err = dev / (self.sample_size as f64).sqrt();
    
                let Ok(n) = StudentsT::new(mean, std_err, self.sample_size as f64 - 1.0) else {
                    self.strings[5] = "Not a valid T distr".to_string();
                    return;
                };
    
                let int_l = n.inverse_cdf((1.0-self.confidence as f64)/2.0);
                let int_h = n.inverse_cdf((1.0+self.confidence as f64)/2.0);
    
                self.interval = Constr::In(int_l as f32, int_h as f32);
    
                self.strings[4] = self.interval.to_string();
                self.strings[5].clear();
            };
            f();
        }
        ui.num_box("", &mut self.strings[4].clone());
        ui.label(&self.strings[5]);
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
}

impl Default for ZTwoStats {
    fn default() -> Self {
        Self { sample_mean_1: "0.0".parse().unwrap(), 
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
                "".to_string(),
            ] }
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
                    self.strings[8] = "Mean 1 is invalid".to_owned();
                    return;
                };
                let Ok(dev1) = self.sample_dev_1.eval() else {
                    self.strings[8] = "Standard deviation 1 is invalid".to_owned();
                    return;
                };
                let Ok(mean2) = self.sample_mean_2.eval() else {
                    self.strings[8] = "Mean 1 is invalid".to_owned();
                    return;
                };
                let Ok(dev2) = self.sample_dev_2.eval() else {
                    self.strings[8] = "Standard deviation 1 is invalid".to_owned();
                    return;
                };
    
                let std_err = (dev1*dev1 / self.sample_size_1 as f64 + dev2*dev2 / self.sample_size_2 as f64).sqrt();
    
                let Ok(n) = Normal::new(mean1 - mean2, std_err) else {
                    self.strings[8] = "Not a valid Normal distr".to_string();
                    return;
                };
    
                let int_l = n.inverse_cdf((1.0-self.confidence as f64)/2.0);
                let int_h = n.inverse_cdf((1.0+self.confidence as f64)/2.0);
    
                self.interval = Constr::In(int_l as f32, int_h as f32);
    
                self.strings[7] = self.interval.to_string();
                self.strings[8].clear();
            };
            f();
        }
        ui.num_box("", &mut self.strings[7].clone());
        ui.label(&self.strings[8]);
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
}

impl Default for TTwoStats {
    fn default() -> Self {
        Self { sample_mean_1: "0.0".parse().unwrap(), 
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
                "".to_string(),
            ] }
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
                    self.strings[8] = "Mean 1 is invalid".to_owned();
                    return;
                };
                let Ok(dev1) = self.sample_dev_1.eval() else {
                    self.strings[8] = "Standard deviation 1 is invalid".to_owned();
                    return;
                };
                let Ok(mean2) = self.sample_mean_2.eval() else {
                    self.strings[8] = "Mean 1 is invalid".to_owned();
                    return;
                };
                let Ok(dev2) = self.sample_dev_2.eval() else {
                    self.strings[8] = "Standard deviation 1 is invalid".to_owned();
                    return;
                };
    
                let std_err = (dev1*dev1 / self.sample_size_1 as f64 + dev2*dev2 / self.sample_size_2 as f64).sqrt();
                let a = dev1*dev1/self.sample_size_1 as f64;
                let b = dev2*dev2/self.sample_size_2 as f64;
                let df = (a+b)*(a+b)/(a*a/(self.sample_size_1 as f64 - 1.0) + b*b/(self.sample_size_2 as f64 - 1.0));
    
                let Ok(n) = StudentsT::new(mean1 - mean2, std_err, df) else {
                    self.strings[8] = "Not a valid T distr".to_string();
                    return;
                };
    
                let int_l = n.inverse_cdf((1.0-self.confidence as f64)/2.0);
                let int_h = n.inverse_cdf((1.0+self.confidence as f64)/2.0);
    
                self.interval = Constr::In(int_l as f32, int_h as f32);
    
                self.strings[7] = self.interval.to_string();
                self.strings[8].clear();
            };
            f();
        }
        ui.num_box("", &mut self.strings[7].clone());
        ui.label(&self.strings[8]);
        resp
    }
}