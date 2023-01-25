use egui::{Widget, Ui};
use statrs::distribution::{Normal, ContinuousCDF};

use crate::{empty_resp, NumBox, Comp};

#[derive(Default, Clone)]
enum Sample {
    #[default]
    None,
    Inf(SampleProbInf),
}

#[derive(Default)]
pub(crate) struct OpenCrunchSample{
    sample: Sample
}

impl Widget for &mut OpenCrunchSample {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        if ui.button("Probability of Sample").clicked() {
            self.sample = Sample::Inf(SampleProbInf::default());
        }

        match &mut self.sample {
            Sample::None => {empty_resp(ui)},
            Sample::Inf(sam) => {ui.add(sam)},
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
    strings: [String; 6]
}

impl Default for SampleProbInf {
    fn default() -> Self {
        Self { sample_size: 1, mean: 0.0, sd: 1.0, target_mean: 0.0, strings: [
            1.to_string(),
            0.0.to_string(),
            1.0.to_string(),
            0.0.to_string(),
            "<=".to_string(),
            "".to_string(),
        ],
            prob: Err("".to_string()),
            comp: Comp::LE, }
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
            if let Some(sample_size) = self.strings[0].parse().ok() {
                self.sample_size = sample_size;
            }
            if let Some(mean) = self.strings[1].parse().ok() {
                self.mean = mean;
            }
            if let Some(sd) = self.strings[2].parse().ok() {
                self.sd = sd;
            }
            if let Some(target) = self.strings[3].parse().ok() {
                self.target_mean = target;
            }
            if let Some(comp) = self.strings[4].parse().ok() {
                self.comp = comp;
            }
        }
        ui.horizontal(|ui| {
            ui.label("Prob");
            ui.text_edit_singleline(&mut (self.strings[5].clone()))
        });
        if resp.changed() {
            //self.strings[4] = self.comp.to_string();
            match Normal::new(self.mean, self.sd/((self.sample_size as f64).sqrt())) {
                Ok(n) => {
                    let fill = n.cdf(self.target_mean);
                    let fill = match self.comp {
                        Comp::GE | Comp::GT => {
                            1.0-fill
                        },
                        Comp::LE | Comp::LT => {
                            fill
                        },
                        Comp::EQ | Comp::NE => {
                            self.prob = Err("Cannot use exact in a continuous distribution.".to_string());
                            self.strings[5] = "Cannot use exact in a continuous distribution.".to_string();
                            return resp;
                        },
                    };
                    self.prob = Ok(fill);
                    self.strings[5] = fill.to_string();
                },
                Err(e) => {
                    self.prob = Err(e.to_string());
                    self.strings[5] = e.to_string();
                },
            }
        }
        resp
    }
}