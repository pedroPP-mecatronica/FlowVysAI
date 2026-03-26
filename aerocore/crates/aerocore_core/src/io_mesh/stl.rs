let (_, triangles) = count(parse_triangle, tri_count)(body).map_err(|e| {
    MeshError::ParseError {
        line: 0,
        message: format!("failed to parse binary STL triangle data: {}", e),
    }
})?;