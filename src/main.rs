use plotlib::page::Page;
use plotlib::repr::{ BoxPlot, Plot };
use plotlib::view::{ CategoricalView, ContinuousView };
use plotlib::style::{ BoxStyle, PointMarker, PointStyle };
use rand::prelude::*;

#[derive(Clone, Copy, Debug)]
struct EnhancerParams {
  pub max_level: i32,

  // At level 0, value == min_value
  // For each level after 1, value += value_increment
  pub value_increment: f32,
  pub min_value: f32,

  // At level 0, upgrade rate == max_upgrade_rate
  // For each level after 1, upgrade_rate *= upgrade_rate_curve
  pub upgrade_rate_curve: f32,
  pub max_upgrade_rate: f32,
  pub min_upgrade_rate: f32,

  // At level max, downgrade_rate == max_downgrade_rate
  // For each level below max, downgrade_rate *= downgrade_rate_curve
  pub downgrade_rate_curve: f32,
  pub max_downgrade_rate: f32,
  pub halve_ratio: f32,
  pub reset_ratio: f32,
  pub min_downgrade_level: i32,
  pub min_halve_level: i32,
  pub min_reset_level: i32
}

#[derive(Clone, Copy, Debug)]
struct EnhanceRate {
  pub level: i32,
  pub value: f32,
  pub upgrade: f32,
  pub downgrade: f32,
  pub halve: f32,
  pub reset: f32,
}

impl EnhanceRate {
  const FORMAT_PRECISION: usize = 1;
  const FORMAT_COLUMN_WIDTH: usize = (1 + 5);
  const FORMAT_LEVEL_WIDTH: usize = 3;
  const FORMAT_SEPARATOR_HEAD: &'static str = " | ";
  const FORMAT_SEPARATOR_BODY: &'static str = " | ";

  pub fn no_change_rate(&self) -> f32 {
    return 1.0 - (self.reset + self.halve + self.downgrade + self.upgrade);
  }
  
  pub fn format_table(rates: &Vec::<EnhanceRate>) -> String {
    let heading = Self::format_table_heading();
    let rows = rates.iter()
      .map(| rate | Self::format_table_row(rate))
      .collect::<Vec::<String>>()
      .concat();

    return format!("{heading}{rows}");
  }

  pub fn format_table_heading() -> String {
    let level = format!("{:<1$}", "LVL", Self::FORMAT_LEVEL_WIDTH);
    let value = format!("{:<1$}", "VALUE", Self::FORMAT_COLUMN_WIDTH);
    let upgrade = format!("{:<1$}", "GAIN", Self::FORMAT_COLUMN_WIDTH);
    let no_change = format!("{:<1$}", "NONE", Self::FORMAT_COLUMN_WIDTH);
    let downgrade = format!("{:<1$}", "LOSE", Self::FORMAT_COLUMN_WIDTH);
    let halve = format!("{:<1$}", "HALVE", Self::FORMAT_COLUMN_WIDTH);
    let reset = format!("{:<1$}", "RESET", Self::FORMAT_COLUMN_WIDTH);

    return format!("{1}{0}{2}{0}{3}{0}{4}{0}{5}{0}{6}{0}{7}\n", Self::FORMAT_SEPARATOR_HEAD, level, value, upgrade, no_change, downgrade, halve, reset);
  }

  pub fn format_table_row(rate: &EnhanceRate) -> String {
    let level = format!("{:>1$}", rate.level, Self::FORMAT_LEVEL_WIDTH);
    let value = Self::format_rate_ex(rate.value * 100.0, 1, Self::FORMAT_COLUMN_WIDTH);
    let upgrade = Self::format_rate(rate.upgrade);
    let no_change = Self::format_rate(rate.no_change_rate());
    let downgrade = Self::format_rate(rate.downgrade);
    let halve = Self::format_rate(rate.halve);
    let reset = Self::format_rate(rate.reset);

    return format!("{1}{0}{2}{0}{3}{0}{4}{0}{5}{0}{6}{0}{7}\n", Self::FORMAT_SEPARATOR_BODY, level, value, upgrade, no_change, downgrade, halve, reset);
  }

  pub fn format_rate(rate: f32) -> String {
    return Self::format_rate_ex(rate * 100.0, Self::FORMAT_PRECISION, Self::FORMAT_COLUMN_WIDTH);
  }

  pub fn format_rate_ex(rate: f32, precision: usize, max_width: usize) -> String {
    let number = format!("{rate:.precision$}%");
    return format!("{number:>max_width$}");
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum EnhanceResult {
  NoChange,
  Upgrade,
  Downgrade,
  Halve,
  Reset
}

#[derive(Clone, Debug)]
struct EnhancerSimulation<'a> {
  pub level: i32,
  pub attempt_count: i32,
  pub rates: &'a Vec::<EnhanceRate>,
  pub history: Vec::<i32>
}

impl EnhancerSimulation<'_> {
  pub fn boxplot_data(simulations: &Vec::<EnhancerSimulation>) -> Vec::<Vec::<f64>> {
    let mut output = Vec::<Vec::<f64>>::new();

    for sim in simulations {
      let history = &sim.history;

      for i in 0..history.len() {
        if i == output.len() {
          output.push(Vec::<f64>::new());
        }

        output[i].push(history[i] as f64);
      }
    }

    return output;
  }

  pub fn scatterplot_data(simulations: &Vec::<EnhancerSimulation>) -> Vec::<(f64, f64)> {
    let mut output = Vec::<(f64, f64)>::new();
    for sim in simulations {
      let history = &sim.history;
      for i in 0..history.len() {
        let x = i as f64;
        let y = history[i] as f64;

        output.push((x, y));
      }
    }

    return output;
  }

  pub fn create_many(rates: &Vec::<EnhanceRate>, count: i32) -> Vec::<EnhancerSimulation> {
    let mut output = Vec::<EnhancerSimulation>::with_capacity(count as usize);

    for _i in 0..count {
      output.push(Self::create(rates));
    }

    return output;
  }

  pub fn create(rates: &Vec::<EnhanceRate>) -> EnhancerSimulation {
    let level = 0;
    let count = 0;
    let mut history = Vec::<i32>::new();
    history.push(0);
    return EnhancerSimulation { level, attempt_count: count, rates, history };
  }

  // Returns true if the set has reached max level
  pub fn enhance_many(simulations: &mut Vec::<EnhancerSimulation>) -> bool {
    let mut all_maxed = true;

    for sim in simulations {
      if !sim.enhance() {
        all_maxed = false;
      }
    }

    return all_maxed;
  }
  
  // Returns true if it has reached max level
  pub fn enhance(&mut self) -> bool {
    let i = self.level as usize;
    if i >= (self.rates.len() - 1) {
      return true;
    }

    let rate = self.rates[i];
    let result = roll(rate);
    let level = apply_result(self.level, result);
    let attempt_count = self.attempt_count + 1;

    self.level = level;
    self.attempt_count = attempt_count;

    if level as usize == self.history.len() {
      self.history.push(attempt_count);
    }

    return false;
  }
}

fn gen_value(params: EnhancerParams, level: i32) -> f32 {
  let mut value = params.min_value;
  for _i in 0..level {
    value += params.value_increment;
  }

  return value;
}

fn gen_upgrade_rate(params: EnhancerParams, level: i32) -> f32 {
  if level >= params.max_level {
    return 0.0;
  }

  let mut upgrade_rate = params.max_upgrade_rate;
  for _i in 0..level {
    upgrade_rate = f32::max(params.min_upgrade_rate, upgrade_rate * params.upgrade_rate_curve);
  }

  return upgrade_rate;
}

fn gen_downgrade_rate(params: EnhancerParams, level: i32) -> f32 {
  if level >= params.max_level {
    return 0.0;
  }

  if level < params.min_downgrade_level {
    return 0.0;
  }

  let mut downgrade_rate = params.max_downgrade_rate;
  let count = params.max_level - level - 1;
  for _i in 0..count {
    downgrade_rate *= params.downgrade_rate_curve;
  }

  return downgrade_rate;
}

fn gen_halve_rate(params: EnhancerParams, level: i32) -> f32 {
  if level < params.min_halve_level {
    return 0.0;
  }

  return gen_downgrade_rate(params, level) * params.halve_ratio;
}

fn gen_reset_rate(params: EnhancerParams, level: i32) -> f32 {
  if level < params.min_reset_level {
    return 0.0;
  }

  return gen_downgrade_rate(params, level) * params.reset_ratio;
}

fn generate_rates(params: EnhancerParams) -> Vec::<EnhanceRate> {
  let count = params.max_level + 1;
  let mut rates = Vec::<EnhanceRate>::with_capacity(count as usize);

  for level in 0..count {
    let value = gen_value(params, level);
    let upgrade = gen_upgrade_rate(params, level);
    let downgrade = gen_downgrade_rate(params, level);
    let halve = gen_halve_rate(params, level);
    let reset = gen_reset_rate(params, level);

    rates.push(EnhanceRate { level, value, upgrade, downgrade, halve, reset });
  }

  return rates;
}

fn roll(rate: EnhanceRate) -> EnhanceResult {
  let roll = thread_rng().gen::<f32>();

  let mut threshold = rate.reset;
  if roll < threshold {
    return EnhanceResult::Reset;
  }

  threshold += rate.halve;
  if roll < threshold {
    return EnhanceResult::Halve;
  }

  threshold += rate.downgrade;
  if roll < threshold {
    return EnhanceResult::Downgrade;
  }

  threshold += rate.upgrade;
  if roll < threshold {
    return EnhanceResult::Upgrade;
  }

  return EnhanceResult::NoChange;
}

fn apply_result(level: i32, result: EnhanceResult) -> i32 {
  match result {
    EnhanceResult::NoChange => level,
    EnhanceResult::Downgrade => level - 1,
    EnhanceResult::Halve => level / 2,
    EnhanceResult::Reset => 0,
    EnhanceResult::Upgrade => level + 1
  }
}

fn default_params() -> EnhancerParams {
  return EnhancerParams {
    max_level: 10,
    value_increment: 0.125,
    min_value: 1.0,
    upgrade_rate_curve: 0.5,
    max_upgrade_rate: 1.0,
    min_upgrade_rate: 0.125,
    downgrade_rate_curve: 0.5,
    max_downgrade_rate: 0.5,
    halve_ratio: 0.25,
    reset_ratio: 0.0625,
    min_downgrade_level: 1,
    min_halve_level: 3,
    min_reset_level: 5
  };
}

fn main() {
  let params = default_params();
  let rates = generate_rates(params);
  let mut simulations = EnhancerSimulation::create_many(&rates, 10000);
  let mut iterations = 0;
  let mut all_maxed = false;

  let rates_table = EnhanceRate::format_table(&rates);
  println!("Computed enhancement rates:");
  print!("{rates_table}");

  println!("Starting simulation of {} actors", simulations.len());
  while !all_maxed {
    iterations += 1;
    all_maxed = EnhancerSimulation::enhance_many(&mut simulations);

    if iterations % 2500 == 0 {
      println!("Reached {iterations} iterations");
    }
  }
  println!("Simulation complete at {iterations} iterations");

  println!("Drawing scatterplot");
  draw_scatter_plot(&simulations);

  println!("Drawing box plot");
  draw_box_plot(&simulations);

  println!("Data saved")
}

fn scatter_x_axis(history_data: &mut Vec::<(f64, f64)>) {
  let max_offset: f64 = 0.25;
  let mut random = thread_rng();

  for point in history_data {
    let signed_roll = (random.gen::<f64>() * 2.0) -1.0;
    let offset = max_offset * signed_roll;
    point.0 = point.0 + offset;
  }
}

fn draw_box_plot(simulations: &Vec::<EnhancerSimulation>) {
  let history_data = EnhancerSimulation::boxplot_data(simulations);

  let mut m_box_plots = Vec::<BoxPlot>::new();
  let mut m_level_labels = Vec::<String>::new();
  for i in 0..history_data.len() {
    let level_set = &history_data[i];
    let label = format!("{}", i);
    m_box_plots.push(BoxPlot::from_vec(level_set.clone()).label(String::from(&label)).style(&BoxStyle::new().fill("#808080FF")));
    m_level_labels.push(label);
  }

  let box_plots = m_box_plots;
  let level_labels = m_level_labels;

  let mut m_view = CategoricalView::new();
    
  for box_plot in box_plots {
    m_view = m_view.add(box_plot);
  }

  let view = m_view
    .x_ticks(&level_labels)
    .y_range(0.0, 800.0)
    .x_label("Enhancement Level")
    .y_label("Total Attempts Taken To Reach (First Time)");

  let page = Page::single(&view);
  
  page.save("box.svg").unwrap();
}

fn draw_scatter_plot(simulations: &Vec::<EnhancerSimulation>) {
  // Scatter plots expect a list of pairs
  let mut history = EnhancerSimulation::scatterplot_data(simulations);
  scatter_x_axis(&mut history);
  let history_data = history;

  // We create our scatter plot from the data
  let scatter_plot: Plot = Plot::new(history_data).point_style(
    PointStyle::new()
      .marker(PointMarker::Square)
      .colour("#19CEA540")
      .size(0.5)
  );

  // The 'view' describes what set of data is drawn
  let v = ContinuousView::new()
    .add(scatter_plot)
    .x_range(0.0, 11.0)
    .y_range(0.0, 800.0)
    .x_label("Enhancement Level")
    .y_label("Total Attempts Taken To Reach (First Time)");

  // A page with a single view is then saved to an SVG file
  Page::single(&v).save("scatter.svg").unwrap();
}
