use axum::Json;
use serde::Serialize;

/// Successful API envelope: `{ "success": true, "data": T }`.
#[derive(Debug, Serialize)]
#[serde(bound(serialize = "T: Serialize"))]
pub struct ApiSuccess<T> {
    pub success: bool,
    pub data: T,
}

/// Wrap `value` in the standard success envelope for JSON responses.
pub fn json<T: Serialize>(value: T) -> Json<ApiSuccess<T>> {
    Json(ApiSuccess {
        success: true,
        data: value,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_envelope_serializes() {
        let Json(inner) = json("Hello from Rezis");
        let v = serde_json::to_value(&inner).unwrap();
        assert_eq!(v["success"], true);
        assert_eq!(v["data"], "Hello from Rezis");
    }
}
