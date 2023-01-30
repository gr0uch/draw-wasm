mod utils;

use serde::{Serialize, Deserialize};
use rayon::prelude::*;
use sublime_fuzzy::{best_match, format_simple, Match};
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub use wasm_bindgen_rayon::init_thread_pool;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct SearchIndex {
    sample_space: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SearchResults {
    count: usize,
    results: Vec<(String, String)>,
}

#[wasm_bindgen(js_name = matchSingle)]
pub fn match_single(input: &str, target: &str, should_format: bool) -> JsValue {
    let match_opt = best_match(input, target);

    if match_opt.is_none() {
        return serde_wasm_bindgen::to_value(&()).unwrap();
    }

    match should_format {
        true => {
            let formatted = format_simple(&match_opt.unwrap(), target, "<strong>", "</strong>");
            serde_wasm_bindgen::to_value(&formatted).unwrap()
        }
        _ => serde_wasm_bindgen::to_value(&match_opt.unwrap()).unwrap(),
    }
}

#[wasm_bindgen]
impl SearchIndex {
    pub fn new() -> Self {
        utils::set_panic_hook();
        let sample_space = Vec::new();
        Self { sample_space }
    }

    #[wasm_bindgen(js_name = loadResult)]
    pub fn load_result(&mut self, result: String) {
        self.sample_space.push(result)
    }

    pub fn search(&mut self, input: String, results_length: usize) -> JsValue {
        let sample_space = &self.sample_space;
        let mut results: Vec<(&str, Match)> = sample_space
            // WTF: this is somehow slower than .iter()?
            .par_iter()
            .filter_map(|sample| {
                if input.len() > sample.len() {
                    return None;
                }
                let match_opt = best_match(&input, &sample);
                if match_opt.is_none() {
                    return None;
                }
                Some((sample.as_str(), match_opt.unwrap()))
            })
            .collect();

        results.sort_by(|(s1, m1), (s2, m2)| {
            (m2.score() + s1.len() as isize)
                .partial_cmp(&(m1.score() + s2.len() as isize))
                .unwrap()
        });

        let count = results.len();
        results.truncate(results_length);

        let formatted_results: Vec<(String, String)> = results
            .into_iter()
            .map(|(s, match_obj)| {
                let formatted = format_simple(&match_obj, &s, "<strong>", "</strong>");
                (s.to_owned(), formatted)
            })
            .collect();

        let search_results = SearchResults {
            count,
            results: formatted_results,
        };

        serde_wasm_bindgen::to_value(&search_results).unwrap()
    }
}
