use fbas_analyzer::*;
use wasm_bindgen::prelude::*;
#[macro_use]
extern crate serde_derive;

use std::collections::HashMap;
#[macro_use]
extern crate lazy_static;
use std::sync::Mutex;

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
#[derive(PartialEq)]
pub enum MergeBy {
    DoNotMerge,
    Orgs,
    ISPs,
    Countries,
}

#[derive(Serialize, Default)]
pub struct AnalysedValues {
    minimal_quorums: String,
    minimal_quorums_size: usize,
    has_intersection: bool,
    minimal_blocking_sets: String,
    minimal_blocking_sets_size: usize,
    smallest_blocking_set_size: usize,
    minimal_splitting_sets: String,
    minimal_splitting_sets_size: usize,
    smallest_splitting_set_size: usize,
    top_tier: Vec<String>,
    top_tier_size: usize,
    symmetric_top_tier_exists: bool,
    symmetric_top_tier: String,
    cache_hit: bool,
}

#[derive(Debug, Clone, Default)]
struct CustomResultsStruct {
    minimal_quorums: NodeIdSetVecResult,
    minimal_quorums_size: usize,
    minimal_blocking_sets: NodeIdSetVecResult,
    minimal_splitting_sets: NodeIdSetVecResult,
    top_tier: NodeIdSetResult,
    top_tier_size: usize,
    has_quorum_intersection: bool,
    symmetric_clusters: Vec<QuorumSet>,
}

fn do_analysis(fbas: &Fbas) -> CustomResultsStruct {
    let analysis = Analysis::new(fbas);

    CustomResultsStruct {
        minimal_quorums: analysis.minimal_quorums(),
        minimal_quorums_size: analysis.minimal_quorums().len(),
        minimal_blocking_sets: analysis.minimal_blocking_sets(),
        minimal_splitting_sets: analysis.minimal_splitting_sets(),
        top_tier: analysis.top_tier(),
        top_tier_size: analysis.top_tier().len(),
        has_quorum_intersection: analysis.has_quorum_intersection(),
        symmetric_clusters: analysis.symmetric_clusters(),
    }
}

lazy_static! {
    static ref RESULTS_CACHE: Mutex<HashMap<Fbas, CustomResultsStruct>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
}

fn fbas_has_been_analysed(fbas: &Fbas) -> Option<CustomResultsStruct> {
    let cache = RESULTS_CACHE.lock().unwrap();
    let value = cache.get(&fbas);
    if let Some(cached_results) = value {
        Some(cached_results.clone())
    } else {
        None
    }
}

#[wasm_bindgen]
pub fn fbas_analysis(
    json_fbas: String,
    json_orgs: String,
    faulty_nodes: String,
    merge_by: MergeBy,
) -> JsValue {
    let fbas: Fbas = Fbas::from_json_str(&json_fbas).to_standard_form();
    let grouping = match merge_by {
        MergeBy::Orgs => Some(Groupings::organizations_from_json_str(&json_orgs, &fbas)),
        MergeBy::ISPs => Some(Groupings::isps_from_json_str(&json_fbas, &fbas)),
        MergeBy::Countries => Some(Groupings::countries_from_json_str(&json_fbas, &fbas)),
        MergeBy::DoNotMerge => None,
    };
    let inactive_nodes: Vec<String> = serde_json::from_str(&faulty_nodes).unwrap();
    let inactive_nodes: Vec<&str> = inactive_nodes.iter().map(|s| s.as_ref()).collect();
    let mut cache_hit = false;
    let analysis_results = if let Some(cached_results) = fbas_has_been_analysed(&fbas) {
        cache_hit = true;
        cached_results
    } else {
        let new_results = do_analysis(&fbas);
        let mut results_cache = RESULTS_CACHE.lock().unwrap();
        results_cache.insert(fbas.clone(), new_results.clone());
        new_results
    };

    let min_mqs = if merge_by != MergeBy::DoNotMerge {
        analysis_results
            .minimal_quorums
            .merged_by_group(&grouping.clone().unwrap())
            .minimal_sets()
    } else {
        analysis_results.minimal_quorums.minimal_sets()
    };

    let (minimal_quorums_size, minimal_quorums) = if merge_by != MergeBy::DoNotMerge {
        (
            min_mqs.len(),
            min_mqs.into_pretty_string(&fbas, grouping.as_ref()),
        )
    } else {
        (min_mqs.len(), min_mqs.into_pretty_string(&fbas, None))
    };

    let min_mbs_without_faulty =
        analysis_results
            .minimal_blocking_sets
            .without_nodes_pretty(&inactive_nodes, &fbas, None);

    let min_mbs = if merge_by != MergeBy::DoNotMerge {
        min_mbs_without_faulty
            .merged_by_group(&grouping.clone().unwrap())
            .minimal_sets()
    } else {
        min_mbs_without_faulty.minimal_sets()
    };
    let (minimal_blocking_sets_size, smallest_blocking_set_size, minimal_blocking_sets) =
        if merge_by != MergeBy::DoNotMerge {
            (
                min_mbs.len(),
                min_mbs.min(),
                min_mbs.into_pretty_string(&fbas, grouping.as_ref()),
            )
        } else {
            (
                min_mbs.len(),
                min_mbs.min(),
                min_mbs.into_pretty_string(&fbas, None),
            )
        };

    let min_mss = if merge_by != MergeBy::DoNotMerge {
        analysis_results
            .minimal_splitting_sets
            .merged_by_group(&grouping.clone().unwrap())
            .minimal_sets()
    } else {
        analysis_results.minimal_splitting_sets.minimal_sets()
    };
    let (minimal_splitting_sets_size, smallest_splitting_set_size, minimal_splitting_sets) =
        if merge_by != MergeBy::DoNotMerge {
            (
                min_mss.len(),
                min_mss.min(),
                min_mss.into_pretty_string(&fbas, grouping.as_ref()),
            )
        } else {
            (
                min_mss.len(),
                min_mss.min(),
                min_mss.into_pretty_string(&fbas, None),
            )
        };

    let top_tier = if merge_by != MergeBy::DoNotMerge {
        analysis_results
            .top_tier
            .merged_by_group(&grouping.clone().unwrap())
            .into_pretty_vec(&fbas, grouping.as_ref())
    } else {
        analysis_results.top_tier.into_pretty_vec(&fbas, None)
    };

    let has_intersection = analysis_results.has_quorum_intersection;
    let top_tier_size = top_tier.len();

    let sc = if merge_by != MergeBy::DoNotMerge {
        grouping
            .clone()
            .unwrap()
            .merge_quorum_sets(analysis_results.symmetric_clusters)
    } else {
        analysis_results.symmetric_clusters
    };
    let (symmetric_top_tier, symmetric_top_tier_exists) = if has_intersection && (sc.len() == 1) {
        if merge_by != MergeBy::DoNotMerge {
            (sc.into_pretty_string(&fbas, grouping.as_ref()), true)
        } else {
            (sc.into_pretty_string(&fbas, None), true)
        }
    } else {
        (String::default(), false)
    };

    let analysed_values = AnalysedValues {
        minimal_quorums,
        minimal_quorums_size,
        has_intersection,
        minimal_blocking_sets,
        minimal_blocking_sets_size,
        smallest_blocking_set_size,
        minimal_splitting_sets,
        minimal_splitting_sets_size,
        smallest_splitting_set_size,
        top_tier,
        top_tier_size,
        symmetric_top_tier_exists,
        symmetric_top_tier,
        cache_hit,
    };
    JsValue::from_serde(&analysed_values).unwrap()
}
