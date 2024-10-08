use regex::Regex;
use std::io::*;
use std::sync::LazyLock;
use vega_lite_3::*;

static RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"test ([a-zA-Z0-9:_]+)\s+... bench:\s+([0-9,\.]+) ns/iter \(\+/- ([0-9,\.]+)\)")
        .unwrap()
});

#[derive(Debug, Default)]
struct BenchmarkResult {
    name: String,
    median: f64,
    deviation: f64,
}

fn parse_cargo_bench_result(input: &str) -> Vec<BenchmarkResult> {
    let mut results: Vec<BenchmarkResult> = vec![];
    for line in input.lines() {
        let mut name = String::new();
        let mut median = 0.0;
        let mut deviation = 0.0;

        // skip to not benchmark result's line
        if !line.contains("... bench:") {
            continue;
        }

        for cap in RE.captures_iter(line) {
            name = cap[1].to_string();
            median = cap[2].replace(",", "").parse().unwrap();
            deviation = cap[3].replace(",", "").parse().unwrap();
        }
        results.push(BenchmarkResult {
            name,
            median,
            deviation,
        });
    }

    results
}

fn plot(results: Vec<BenchmarkResult>) {
    let mut data_strings = vec![];
    for result in results.iter() {
        data_strings.push(format!(
            r#"{{"testname": "{}", "median": {}, "min": {}, "max": {}}}"#,
            result.name,
            result.median,
            result.median - result.deviation,
            result.median + result.deviation
        ));
    }
    let data_string = data_strings.join(",");

    let spec = format!(
        r##"{{
  "$schema": "https://vega.github.io/schema/vega-lite/v4.json",
  "description": "cargo bench result",
  "data": {{"values": [{}]}},
  "layer": [
    {{
      "mark": {{
        "type": "bar"
      }},
      "encoding": {{
        "x": {{"field": "testname", "type": "ordinal"}},
        "y": {{
          "field": "median",
          "type": "quantitative",
          "axis": {{"title": "result [ns/iter]"}}
        }}
      }}
    }},
    {{
      "mark": {{
        "type": "rule"
      }},
      "encoding": {{
        "x": {{"field": "testname", "type": "ordinal"}},
        "y": {{"field": "min", "type": "quantitative"}},
        "y2": {{"field": "max"}}
      }},
      "size": {{"value": 5}}
    }}
  ],
  "config": {{
    "axisX": {{"labelAngle": -25}}
  }}
}}"##,
        data_string
    );

    log::debug!("spec=\n{}", spec);

    // Use you own data to populate the chart
    let chart: Vegalite = serde_json::from_str(spec.as_str()).unwrap();

    // display the chart using `showata`
    chart.show().unwrap();
}

fn main() {
    let mut input = String::new();
    let _ = stdin().lock().read_to_string(&mut input);

    // output cargo bench result
    println!("{}", input);

    let benchmark_results = parse_cargo_bench_result(input.as_str());
    plot(benchmark_results);
}
