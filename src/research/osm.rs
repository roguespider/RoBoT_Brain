//! OpenStreetMap adapter (research_engine.md Tool Catalog: `search_osm`).
//!
//! Backed by the public Nominatim geocoding API, which needs no API key but
//! does require a descriptive User-Agent (supplied by the shared HTTP client).
//! Every result is a real mapped feature with a real `osm` type and a citable
//! link, so a place citation resolves to the object it names (Rule 3).

use crate::research::errors::ResearchError;
use crate::research::provider::{SearchProvider, SearchResult, SearchResults, SearchSource};
use serde::Deserialize;

const ENDPOINT: &str = "https://nominatim.openstreetmap.org/search";
const PROVIDER: &str = "osm";

#[derive(Debug, Deserialize)]
struct NominatimPlace {
    #[serde(default)]
    place_id: Option<i64>,
    #[serde(default)]
    osm_type: Option<String>,
    #[serde(default)]
    osm_id: Option<i64>,
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    lat: Option<String>,
    #[serde(default)]
    lon: Option<String>,
    #[serde(default, rename = "class")]
    place_class: Option<String>,
    #[serde(default, rename = "type")]
    place_type: Option<String>,
}

/// The citable OpenStreetMap URL for a returned feature.
fn feature_url(osm_type: &str, osm_id: i64) -> String {
    match osm_type {
        "node" => format!("https://www.openstreetmap.org/node/{osm_id}"),
        "way" => format!("https://www.openstreetmap.org/way/{osm_id}"),
        "relation" => format!("https://www.openstreetmap.org/relation/{osm_id}"),
        // An unrecognised type is not a citable OSM object.
        _ => String::new(),
    }
}

/// Compose a snippet from the fields Nominatim genuinely returns.
fn snippet_for(place: &NominatimPlace) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let (Some(class), Some(kind)) = (
        place.place_class.as_deref().filter(|c| !c.is_empty()),
        place.place_type.as_deref().filter(|t| !t.is_empty()),
    ) {
        parts.push(format!("Feature type: {class}/{kind}"));
    }
    if let (Some(lat), Some(lon)) = (
        place.lat.as_deref().filter(|v| !v.is_empty()),
        place.lon.as_deref().filter(|v| !v.is_empty()),
    ) {
        parts.push(format!("Coordinates: {lat}, {lon}"));
    }
    if let Some(id) = place.place_id {
        parts.push(format!("Nominatim place id: {id}"));
    }
    if parts.is_empty() {
        return "Mapped feature returned without classification.".to_string();
    }
    parts.join("\n")
}

pub struct OsmProvider;

impl OsmProvider {
    pub fn new() -> Self {
        Self
    }

    pub async fn fetch(
        &self,
        query: &str,
        location: Option<&str>,
        max_results: usize,
    ) -> Result<Vec<SearchResult>, ResearchError> {
        // Nominatim's `q` parameter already accepts "city, country" style input,
        // so a region hint is folded into the free-text query it understands.
        let effective_query = match location.map(str::trim).filter(|l| !l.is_empty()) {
            Some(region) => format!("{query}, {region}"),
            None => query.to_string(),
        };
        let url = format!(
            "{ENDPOINT}?q={}&format=jsonv2&limit={}&addressdetails=1",
            urlencoding::encode(&effective_query),
            max_results.clamp(1, 50)
        );
        let places: Vec<NominatimPlace> = crate::research::http::get_json(PROVIDER, &url).await?;

        let total = places.len();
        let mut accepted = 0usize;
        let results: Vec<SearchResult> = places
            .into_iter()
            .filter_map(|place| {
                let osm_type = place.osm_type.clone().filter(|t| !t.trim().is_empty())?;
                let osm_id = place.osm_id?;
                let url = feature_url(&osm_type, osm_id);
                if !crate::research::sanitize::is_citable_url(&url) {
                    return None;
                }
                let title = place
                    .name
                    .clone()
                    .filter(|n| !n.trim().is_empty())
                    .or_else(|| place.display_name.clone())
                    .unwrap_or_else(|| format!("{osm_type} {osm_id}"));
                // Nominatim orders by importance but exposes no score, so the
                // accepted sequence order carries the rank (R5).
                let relevance = crate::research::sanitize::positional_relevance(accepted, total);
                accepted += 1;
                Some(crate::research::sanitize::build_result(
                    &title,
                    &url,
                    &snippet_for(&place),
                    relevance,
                    SearchSource::Web,
                ))
            })
            .collect();

        crate::research::http::require_results(PROVIDER, query, results)
    }
}

impl Default for OsmProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchProvider for OsmProvider {
    fn search(
        &self,
        query: &str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<SearchResults, ResearchError>> + Send>,
    > {
        let query = query.to_string();
        let provider = OsmProvider::new();
        Box::pin(async move {
            let results = provider.fetch(&query, None, 15).await?;
            Ok(SearchResults {
                results,
                provider: PROVIDER.to_string(),
                query,
                retrieved_at: chrono::Utc::now(),
            })
        })
    }

    fn name(&self) -> &str {
        PROVIDER
    }

    fn supports(&self, source: SearchSource) -> bool {
        source == SearchSource::Web
    }
}
