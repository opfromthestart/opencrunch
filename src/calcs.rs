use egui::{Ui, Widget};
use statrs::{
    distribution::{ContinuousCDF, Normal},
    function,
};

use crate::{empty_resp, Comp, NumBox};

#[derive(Default, Clone)]
enum Calcs {
    #[default]
    None,
    SampInf(SampleProbInf),
    Comb(Comb),
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
            if ui.button("Comb/Perm").clicked() {
                self.sample = Calcs::Comb(Comb::default());
            }
        });

        match &mut self.sample {
            Calcs::None => empty_resp(ui),
            Calcs::SampInf(sam) => ui.add(sam),
            Calcs::Comb(c) => ui.add(c),
        }
    }
}

#[derive(Clone)]
pub(crate) struct SampleProbInf {
    sample_size: usize,
    mean: f64,
    sd: f64,
    target_mean: f64,
    prob: Result<f64, String>,
    comp: Comp,
    /// sample_size, mean, sd, target_mean, comp
    strings: [String; 6],
}

impl Default for SampleProbInf {
    fn default() -> Self {
        Self {
            sample_size: 1,
            mean: 0.0,
            sd: 1.0,
            target_mean: 0.0,
            strings: [
                1.to_string(),
                0.0.to_string(),
                1.0.to_string(),
                0.0.to_string(),
                "<=".to_string(),
                "".to_string(),
            ],
            prob: Err("".to_string()),
            comp: Comp::LE,
        }
    }
}

impl Widget for &mut SampleProbInf {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let mut resp = ui.num_box("Sample Size", &mut self.strings[0]);
        resp = resp.union(ui.num_box("Population Mean", &mut self.strings[1]));
        resp = resp.union(ui.num_box("Population SD", &mut self.strings[2]));
        resp = resp.union(ui.text_edit_singleline(&mut self.strings[4]));
        resp = resp.union(ui.num_box("Sample Mean", &mut self.strings[3]));
        if resp.changed() {
            if let Ok(sample_size) = self.strings[0].parse() {
                self.sample_size = sample_size;
            }
            if let Ok(mean) = self.strings[1].parse() {
                self.mean = mean;
            }
            if let Ok(sd) = self.strings[2].parse() {
                self.sd = sd;
            }
            if let Ok(target) = self.strings[3].parse() {
                self.target_mean = target;
            }
            if let Ok(comp) = self.strings[4].parse() {
                self.comp = comp;
            }
        }
        ui.horizontal(|ui| {
            ui.label("Prob");
            ui.text_edit_singleline(&mut (self.strings[5].clone()))
        });
        if resp.changed() {
            //self.strings[4] = self.comp.to_string();
            match Normal::new(self.mean, self.sd / ((self.sample_size as f64).sqrt())) {
                Ok(n) => {
                    let fill = n.cdf(self.target_mean);
                    let fill = match self.comp {
                        Comp::GE | Comp::GT => 1.0 - fill,
                        Comp::LE | Comp::LT => fill,
                        Comp::EQ | Comp::NE => {
                            self.prob =
                                Err("Cannot use exact in a continuous distribution.".to_string());
                            self.strings[5] =
                                "Cannot use exact in a continuous distribution.".to_string();
                            return resp;
                        }
                    };
                    self.prob = Ok(fill);
                    self.strings[5] = fill.to_string();
                }
                Err(e) => {
                    self.prob = Err(e.to_string());
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
