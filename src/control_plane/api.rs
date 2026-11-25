//! Control plane API per Architecture Chapter 28.
//! API endpoints for developer interface access.

/// API route definition.
#[derive(Debug, Clone, PartialEq)]
pub struct ApiRoute {
    /// HTTP method.
    pub method: String,
    /// Route path.
    pub path: String,
    /// Handler name.
    pub handler: String,
}

/// Control plane API definition.
pub struct ControlPlaneApi {
    /// Available routes.
    pub routes: Vec<ApiRoute>,
}

impl ControlPlaneApi {
    /// Create a new API with default routes.
    pub fn new() -> Self {
        Self {
            routes: vec![
                ApiRoute {
                    method: "GET".to_string(),
                    path: "/state".to_string(),
                    handler: "get_state".to_string(),
                },
                ApiRoute {
                    method: "GET".to_string(),
                    path: "/metrics".to_string(),
                    handler: "get_metrics".to_string(),
                },
                ApiRoute {
                    method: "POST".to_string(),
                    path: "/command".to_string(),
                    handler: "execute_command".to_string(),
                },
            ],
        }
    }

    /// Get the list of available routes.
    pub fn routes(&self) -> &[ApiRoute] {
        &self.routes
    }
}

impl Default for ControlPlaneApi {
    fn default() -> Self {
        Self::new()
    }
}
